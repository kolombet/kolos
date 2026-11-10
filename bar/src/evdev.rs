// Shared raw-evdev plumbing for gamepad.rs and mediakeys.rs: no
// joystick/libinput/udev dependency, same spirit as `hypr.rs` talking to
// Hyprland's sockets directly instead of shelling out.

use std::fs;
use std::path::PathBuf;

pub const EV_KEY: u16 = 1;
pub const EV_ABS: u16 = 3;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InputEvent {
    pub tv_sec: i64,
    pub tv_usec: i64,
    pub ev_type: u16,
    pub code: u16,
    pub value: i32,
}

/// Finds the event node of the first input device in `/proc/bus/input/devices`
/// whose name contains `name` (case-insensitive) and whose Handlers list has
/// an entry starting with `handler_prefix` — e.g. "js" to pick a gamepad's
/// joystick-capable node, or "kbd" to pick a keyboard node over a
/// mouse/trackpad node sharing the same product name.
pub fn find_device(name: &str, handler_prefix: &str) -> Option<PathBuf> {
    find_devices(&[name], handler_prefix).into_iter().next()
}

/// Like `find_device`, but returns every device whose name contains any of
/// `names` (given in lowercase).
pub fn find_devices(names: &[&str], handler_prefix: &str) -> Vec<PathBuf> {
    let Ok(content) = fs::read_to_string("/proc/bus/input/devices") else { return Vec::new() };
    let mut found = Vec::new();
    for block in content.split("\n\n") {
        let Some(n) = block.lines().find_map(|l| l.strip_prefix("N: Name=\"")) else { continue };
        let n = n.to_lowercase();
        if !names.iter().any(|name| n.contains(&name.to_lowercase())) {
            continue;
        }
        let Some(handlers) = block.lines().find_map(|l| l.strip_prefix("H: Handlers=")) else { continue };
        if !handlers.split_whitespace().any(|h| h.starts_with(handler_prefix)) {
            continue;
        }
        let Some(event) = handlers.split_whitespace().find(|h| h.starts_with("event")) else { continue };
        found.push(PathBuf::from(format!("/dev/input/{event}")));
    }
    found
}
