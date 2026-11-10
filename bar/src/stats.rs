// System numbers for the right side of the bar, read straight from /proc and
// /sys every tick: no daemons, no D-Bus.

use std::fs;
use std::process::Command;

use crate::gamepad::Gamepad;
use crate::mediakeys::MediaKeys;

#[derive(Clone, PartialEq, Default)]
pub struct Stats {
    pub cpu_percent: u32,
    pub mem_used_gib: f32,
    pub mem_total_gib: f32,
    /// (capacity %, charging), or None on machines without a battery.
    pub battery: Option<(u32, bool)>,
    /// (level, charging) for a connected peripheral's battery — a game
    /// controller, most likely — or None when none is connected. `level` is
    /// "<n>%" if the device reports a number, otherwise the word from
    /// `capacity_level` ("Low", "Normal", ...).
    pub gamepad_battery: Option<(String, bool)>,
    /// Last button pressed on a connected gamepad, external or a handheld's
    /// built-in one ("A", "LT", "ARROW", ...), or None if none is connected or nothing has been
    /// pressed yet. See `gamepad.rs`.
    pub gamepad_button: Option<String>,
    /// Last top-row function/media key pressed on the internal keyboard
    /// ("F1", "BR-DOWN", "MUTE", ...), or None if nothing has been pressed
    /// yet. See `mediakeys.rs`.
    pub last_key: Option<String>,
    /// CPU package temperature in °C, or None if no `x86_pkg_temp` zone exists.
    pub cpu_temp_c: Option<u32>,
    /// Screen backlight, as a percent of max, or None without a backlight.
    pub screen_brightness: Option<u32>,
    /// Keyboard backlight, as a percent of max, or None without one.
    pub kbd_backlight: Option<u32>,
    /// (volume %, muted), from the default PipeWire sink via `wpctl`, or
    /// None if `wpctl` fails (no default sink, PipeWire not running).
    pub volume: Option<(u32, bool)>,
    pub time: String,
    pub date: String,
}

#[derive(Default)]
pub struct Sampler {
    prev_idle: u64,
    prev_total: u64,
    gamepad: Gamepad,
    media_keys: MediaKeys,
}

impl Sampler {
    /// Initial full sample, taken once at startup before the fast/slow timers
    /// below start (see main.rs).
    pub fn sample(&mut self) -> Stats {
        let now = chrono::Local::now();
        let (mem_used_gib, mem_total_gib) = memory();
        Stats {
            cpu_percent: self.cpu(),
            mem_used_gib,
            mem_total_gib,
            battery: battery(),
            gamepad_battery: gamepad_battery(),
            gamepad_button: self.gamepad.sample(),
            last_key: self.media_keys.sample(),
            cpu_temp_c: cpu_temperature(),
            screen_brightness: screen_brightness(),
            kbd_backlight: kbd_backlight(),
            volume: volume(),
            time: now.format("%H:%M").to_string(),
            date: now.format("%d.%m.%Y").to_string(),
        }
    }

    /// Shortcuts (gamepad button, media key), brightness and volume: all
    /// evdev/sysfs reads cheap enough, and latency-sensitive enough (feedback
    /// for a key just pressed), to poll on the fast tick. Carries every other
    /// field over from `prev` unchanged.
    pub fn sample_fast(&mut self, prev: &Stats) -> Stats {
        Stats {
            gamepad_button: self.gamepad.sample(),
            last_key: self.media_keys.sample(),
            screen_brightness: screen_brightness(),
            kbd_backlight: kbd_backlight(),
            volume: volume(),
            ..prev.clone()
        }
    }

    /// CPU, memory, battery (system and peripheral) and the clock: nothing
    /// here needs sub-second freshness, so these poll on the slow tick
    /// instead. Carries every other field over from `prev` unchanged.
    pub fn sample_slow(&mut self, prev: &Stats) -> Stats {
        let now = chrono::Local::now();
        let (mem_used_gib, mem_total_gib) = memory();
        Stats {
            cpu_percent: self.cpu(),
            mem_used_gib,
            mem_total_gib,
            battery: battery(),
            gamepad_battery: gamepad_battery(),
            cpu_temp_c: cpu_temperature(),
            time: now.format("%H:%M").to_string(),
            date: now.format("%d.%m.%Y").to_string(),
            ..prev.clone()
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
        // Peripheral batteries (game controllers, mice, keyboards, headsets)
        // also register as type "Battery" but set scope "Device"; the
        // system's own battery has no `scope` file at all (defaults to
        // "System"). Skip peripherals instead of letting a missing/
        // non-numeric `capacity` file (e.g. an Xbox controller only
        // exposes `capacity_level`, a string) abort the whole scan.
        let scope = fs::read_to_string(path.join("scope")).unwrap_or_default();
        if scope.trim() == "Device" {
            continue;
        }
        let Some(capacity) = fs::read_to_string(path.join("capacity"))
            .ok()
            .and_then(|s| s.trim().parse().ok())
        else {
            continue;
        };
        let status = fs::read_to_string(path.join("status")).unwrap_or_default();
        return Some((capacity, status.trim() == "Charging"));
    }
    None
}

/// Battery of a connected peripheral — the mirror image of `battery()`
/// above: this one *only* looks at `scope: Device` entries, the ones the
/// system-battery scan skips. Prefers a numeric `capacity` file if the
/// device has one; falls back to `capacity_level` (a word like "Normal" or
/// "Low"), which is all the `xone` driver's Xbox controllers expose. Keeps
/// scanning past any entry with neither, so one odd peripheral can't hide
/// another's battery.
fn gamepad_battery() -> Option<(String, bool)> {
    let dir = fs::read_dir("/sys/class/power_supply").ok()?;
    for entry in dir.flatten() {
        let path = entry.path();
        if fs::read_to_string(path.join("type")).unwrap_or_default().trim() != "Battery" {
            continue;
        }
        if fs::read_to_string(path.join("scope")).unwrap_or_default().trim() != "Device" {
            continue;
        }
        let numeric = fs::read_to_string(path.join("capacity"))
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok())
            .map(|c| format!("{c}%"));
        let level = numeric.or_else(|| {
            fs::read_to_string(path.join("capacity_level"))
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty() && s != "Unknown")
        });
        let Some(level) = level else { continue };
        let charging = fs::read_to_string(path.join("status")).unwrap_or_default().trim() == "Charging";
        return Some((level, charging));
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

/// Reads `<dir>/brightness` as a percent of `<dir>/max_brightness`.
fn brightness_percent(dir: &std::path::Path) -> Option<u32> {
    let read = |name: &str| -> Option<u32> { fs::read_to_string(dir.join(name)).ok()?.trim().parse().ok() };
    let (brightness, max) = (read("brightness")?, read("max_brightness")?);
    if max == 0 {
        return None;
    }
    Some(100 * brightness / max)
}

/// First device in `/sys/class/backlight` (there's normally only one).
fn screen_brightness() -> Option<u32> {
    let dir = fs::read_dir("/sys/class/backlight").ok()?;
    dir.flatten().find_map(|entry| brightness_percent(&entry.path()))
}

/// The `leds` entry whose name contains "kbd_backlight" (e.g. this
/// machine's `smc::kbd_backlight`), distinct from caps-lock/scroll-lock LEDs
/// in the same directory.
fn kbd_backlight() -> Option<u32> {
    let dir = fs::read_dir("/sys/class/leds").ok()?;
    dir.flatten()
        .find(|entry| entry.file_name().to_string_lossy().contains("kbd_backlight"))
        .and_then(|entry| brightness_percent(&entry.path()))
}

/// All the audible loudness range on the Sony wireless USB headset lives in
/// the bottom 10% of PipeWire's volume scale (its amp's gain curve); the
/// built-in speaker/headphone jack uses the full range. volume.sh applies
/// the matching cap to the real volume per sink - mirror that here so the
/// 0-100% display always matches what the keys actually do. Keep in sync
/// with `SONY_HEADSET_SINK` in `~/.config/hypr/volume.sh`.
const SONY_HEADSET_SINK: &str = "alsa_output.usb-Sony_Interactive_Entertainment_Wireless_Headset-00.analog-stereo";

fn volume_cap() -> f32 {
    let output = Command::new("pactl").arg("get-default-sink").output();
    match output {
        Ok(o) if String::from_utf8_lossy(&o.stdout).trim() == SONY_HEADSET_SINK => 0.10,
        _ => 1.0,
    }
}

/// (volume %, muted) of the default PipeWire sink, via `wpctl get-volume`
/// (output like "Volume: 0.45" or "Volume: 0.45 [MUTED]"), rescaled from
/// the per-sink cap (see `volume_cap`) to 0-100%. No sysfs equivalent
/// exists for PipeWire's software volume, unlike the backlight numbers
/// above, so this is the one stat here that shells out.
fn volume() -> Option<(u32, bool)> {
    let output = Command::new("wpctl").args(["get-volume", "@DEFAULT_AUDIO_SINK@"]).output().ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    let fraction: f32 = text.split_whitespace().nth(1)?.parse().ok()?;
    let percent = (100.0 * fraction / volume_cap()).round().clamp(0.0, 100.0) as u32;
    Some((percent, text.contains("[MUTED]")))
}
