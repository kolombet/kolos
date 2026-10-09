// System numbers for the right side of the bar, read straight from /proc and
// /sys every tick: no daemons, no D-Bus.

use std::fs;

#[derive(Clone, PartialEq, Default)]
pub struct Stats {
    pub cpu_percent: u32,
    pub mem_used_gib: f32,
    pub mem_total_gib: f32,
    /// (capacity %, charging), or None on machines without a battery.
    pub battery: Option<(u32, bool)>,
    /// CPU package temperature in °C, or None if no `x86_pkg_temp` zone exists.
    pub cpu_temp_c: Option<u32>,
    pub time: String,
    pub date: String,
}

#[derive(Default)]
pub struct Sampler {
    prev_idle: u64,
    prev_total: u64,
}

impl Sampler {
    pub fn sample(&mut self) -> Stats {
        let now = chrono::Local::now();
        let (mem_used_gib, mem_total_gib) = memory();
        Stats {
            cpu_percent: self.cpu(),
            mem_used_gib,
            mem_total_gib,
            battery: battery(),
            cpu_temp_c: cpu_temperature(),
            time: now.format("%H:%M").to_string(),
            date: now.format("%d.%m.%Y").to_string(),
        }
    }

    /// Busy share of all CPU time since the previous sample (0 on the first).
    fn cpu(&mut self) -> u32 {
        let stat = fs::read_to_string("/proc/stat").unwrap_or_default();
        let fields: Vec<u64> = stat
            .lines()
            .next()
            .unwrap_or_default()
            .split_whitespace()
            .skip(1) // "cpu"
            .filter_map(|f| f.parse().ok())
            .collect();
        if fields.len() < 5 {
            return 0;
        }
        let idle = fields[3] + fields[4]; // idle + iowait
        let total: u64 = fields.iter().sum();
        let (d_idle, d_total) = (idle - self.prev_idle, total - self.prev_total);
        let first = self.prev_total == 0;
        self.prev_idle = idle;
        self.prev_total = total;
        if first || d_total == 0 {
            return 0;
        }
        (100 * (d_total - d_idle) / d_total) as u32
    }
}

/// (used, total) in GiB, "used" meaning MemTotal - MemAvailable like `free`.
fn memory() -> (f32, f32) {
    let info = fs::read_to_string("/proc/meminfo").unwrap_or_default();
    let field = |name: &str| -> f32 {
        info.lines()
            .find(|l| l.starts_with(name))
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|v| v.parse::<f32>().ok())
            .unwrap_or(0.0)
    };
    let kib_per_gib = 1024.0 * 1024.0;
    let total = field("MemTotal:");
    let available = field("MemAvailable:");
    ((total - available) / kib_per_gib, total / kib_per_gib)
}

fn battery() -> Option<(u32, bool)> {
    let dir = fs::read_dir("/sys/class/power_supply").ok()?;
    for entry in dir.flatten() {
        let path = entry.path();
        let kind = fs::read_to_string(path.join("type")).unwrap_or_default();
        if kind.trim() != "Battery" {
            continue;
        }
        let capacity = fs::read_to_string(path.join("capacity")).ok()?.trim().parse().ok()?;
        let status = fs::read_to_string(path.join("status")).unwrap_or_default();
        return Some((capacity, status.trim() == "Charging"));
    }
    None
}

/// CPU package temperature from the `x86_pkg_temp` thermal zone (not
/// `thermal_zone0`, which varies by machine — on this one it's `BAT0`).
fn cpu_temperature() -> Option<u32> {
    let dir = fs::read_dir("/sys/class/thermal").ok()?;
    for entry in dir.flatten() {
        let path = entry.path();
        if fs::read_to_string(path.join("type")).unwrap_or_default().trim() != "x86_pkg_temp" {
            continue;
        }
        let milli: i64 = fs::read_to_string(path.join("temp")).ok()?.trim().parse().ok()?;
        return Some((milli / 1000) as u32);
    }
    None
}
