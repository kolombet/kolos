// Last button pressed on any connected XInput-style gamepad, read straight
// from its event node via `evdev.rs`. Covers an Xbox controller (xone's native
// `gip` device, or its `xpad`-compatible virtual twin) and a handheld's
// built-in controls, which InputPlumber exposes as "Microsoft X-Box 360 pad".
// All of them use the standard gamepad evdev codes.

use std::fs::{self, File};
use std::io::{self, Read};
use std::mem::{size_of, size_of_val};
use std::os::fd::AsRawFd;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

use bytemuck::Zeroable;

use crate::evdev::{find_devices, InputEvent, EV_ABS, EV_KEY};

const DEVICE_NAMES: &[&str] = &["xbox", "x-box"];

// Handhelds whose printed A and B are the reverse of what InputPlumber reports.
const AB_SWAPPED_HANDHELDS: &[&str] = &["ayn odin2 gamepad"];

/// True for InputPlumber's virtual pad on one of those handhelds. The virtual
/// sysfs path keeps a real Xbox controller plugged into the same device out.
fn ab_swapped(path: &Path) -> bool {
    let Some(event) = path.file_name().and_then(|n| n.to_str()) else { return false };
    let virtual_pad = fs::canonicalize(format!("/sys/class/input/{event}/device"))
        .is_ok_and(|p| p.starts_with("/sys/devices/virtual"));
    virtual_pad && !find_devices(AB_SWAPPED_HANDHELDS, "js").is_empty()
}

const ABS_Z: u16 = 0x02; // LT
const ABS_RZ: u16 = 0x05; // RT
const ABS_HAT0X: u16 = 0x10; // d-pad
const ABS_HAT0Y: u16 = 0x11;

// Used when the device won't report its range: the Xbox controllers' 0..=1023.
const DEFAULT_TRIGGER_MAX: i32 = 1023;

fn button_label(code: u16) -> Option<&'static str> {
    Some(match code {
        0x130 => "A",      // BTN_SOUTH
        0x131 => "B",      // BTN_EAST
        0x133 => "Y",      // BTN_NORTH
        0x134 => "X",      // BTN_WEST
        0x136 => "LB",     // BTN_TL
        0x137 => "RB",     // BTN_TR
        0x13a => "SELECT", // BTN_SELECT
        0x13b => "START",  // BTN_START
        0x13c => "GUIDE",  // BTN_MODE
        0x13d => "L3",     // BTN_THUMBL
        0x13e => "R3",     // BTN_THUMBR
        0x2c0 => "P1",     // BTN_TRIGGER_HAPPY1..4: extra/back buttons
        0x2c1 => "P2",
        0x2c2 => "P3",
        0x2c3 => "P4",
        _ => return None,
    })
}

/// Trigger range differs per device (1023 on Xbox controllers, 255 on
/// InputPlumber's virtual pad), so ask with EVIOCGABS instead of assuming one.
fn trigger_max(file: &File) -> Option<i32> {
    // struct input_absinfo: value, minimum, maximum, fuzz, flat, resolution.
    let mut info = [0i32; 6];
    // _IOR('E', 0x40 + abs, struct input_absinfo)
    let request = (2u64 << 30) | ((size_of_val(&info) as u64) << 16) | ((b'E' as u64) << 8) | (0x40 + ABS_Z as u64);
    let rc = unsafe { libc::ioctl(file.as_raw_fd(), request as _, info.as_mut_ptr()) };
    (rc == 0 && info[2] > 0).then_some(info[2])
}

struct Pad {
    path: PathBuf,
    file: File,
    // Crossing the midpoint counts as a press, like a digital button.
    trigger_threshold: i32,
    swap_ab: bool,
    lt_down: bool,
    rt_down: bool,
}

impl Pad {
    fn open(path: PathBuf) -> Option<Pad> {
        let file = fs::OpenOptions::new().read(true).custom_flags(libc::O_NONBLOCK).open(&path).ok()?;
        let trigger_threshold = (trigger_max(&file).unwrap_or(DEFAULT_TRIGGER_MAX) + 1) / 2;
        let swap_ab = ab_swapped(&path);
        Some(Pad { path, file, trigger_threshold, swap_ab, lt_down: false, rt_down: false })
    }

    /// Applies every pending event to `last`. False once the device is gone.
    fn drain(&mut self, last: &mut Option<String>) -> bool {
        loop {
            let mut events = [InputEvent::zeroed(); 16];
            match self.file.read(bytemuck::cast_slice_mut(&mut events)) {
                Ok(0) => return false,
                Ok(n) => {
                    for ev in &events[..n / size_of::<InputEvent>()] {
                        self.handle(ev, last);
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => return true,
                Err(_) => return false,
            }
        }
    }

    fn handle(&mut self, ev: &InputEvent, last: &mut Option<String>) {
        match (ev.ev_type, ev.code) {
            (EV_KEY, code) if ev.value == 1 => {
                let code = match code {
                    0x130 if self.swap_ab => 0x131,
                    0x131 if self.swap_ab => 0x130,
                    other => other,
                };
                if let Some(label) = button_label(code) {
                    *last = Some(label.into());
                }
            }
            (EV_ABS, ABS_HAT0X) | (EV_ABS, ABS_HAT0Y) if ev.value != 0 => *last = Some("ARROW".into()),
            (EV_ABS, ABS_Z) => {
                let down = ev.value > self.trigger_threshold;
                if down && !self.lt_down {
                    *last = Some("LT".into());
                }
                self.lt_down = down;
            }
            (EV_ABS, ABS_RZ) => {
                let down = ev.value > self.trigger_threshold;
                if down && !self.rt_down {
                    *last = Some("RT".into());
                }
                self.rt_down = down;
            }
            _ => {}
        }
    }
}

#[derive(Default)]
pub struct Gamepad {
    pads: Vec<Pad>,
    last: Option<String>,
}

impl Gamepad {
    /// Last button pressed on any connected pad ("A", "LT", "ARROW", ...), or
    /// None if none is connected or nothing has been pressed yet. Drains every
    /// pending event on each call, so it's safe to call once per `TICK`
    /// without losing presses in between.
    pub fn sample(&mut self) -> Option<String> {
        for path in find_devices(DEVICE_NAMES, "js") {
            if !self.pads.iter().any(|p| p.path == path) {
                self.pads.extend(Pad::open(path));
            }
        }
        let last = &mut self.last;
        self.pads.retain_mut(|pad| pad.drain(last));
        if self.pads.is_empty() {
            self.last = None;
        }
        self.last.clone()
    }
}
