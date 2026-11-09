// Last button pressed on a connected Xbox controller (xone's native `gip`
// device, or its `xpad`-compatible virtual twin — either works, both expose
// the standard gamepad evdev codes), read straight from its event node via
// `evdev.rs`.

use std::fs::{self, File};
use std::io::{self, Read};
use std::mem::size_of;
use std::os::unix::fs::OpenOptionsExt;

use bytemuck::Zeroable;

use crate::evdev::{find_device, InputEvent, EV_ABS, EV_KEY};

const BTN_SOUTH: u16 = 0x130; // A
const BTN_EAST: u16 = 0x131; // B
const BTN_NORTH: u16 = 0x133; // Y
const BTN_WEST: u16 = 0x134; // X
const BTN_TL: u16 = 0x136; // LB
const BTN_TR: u16 = 0x137; // RB

const ABS_Z: u16 = 0x02; // LT
const ABS_RZ: u16 = 0x05; // RT
const ABS_HAT0X: u16 = 0x10; // d-pad
const ABS_HAT0Y: u16 = 0x11;

// LT/RT report 0..=1023 on this controller (confirmed via EVIOCGABS, not
// guessed); treat crossing the midpoint as a press, like a digital button.
const TRIGGER_THRESHOLD: i32 = 512;

#[derive(Default)]
pub struct Gamepad {
    file: Option<File>,
    lt_down: bool,
    rt_down: bool,
    last: Option<String>,
}

impl Gamepad {
    /// Last button pressed since the controller connected ("A", "LT",
    /// "ARROW", ...), or None if no Xbox controller is connected or nothing
    /// has been pressed yet. Drains every pending event on each call, so
    /// it's safe to call once per `TICK` without losing presses in between.
    pub fn sample(&mut self) -> Option<String> {
        if self.file.is_none() {
            self.try_open();
        }
        loop {
            if self.file.is_none() {
                break;
            }
            let mut events = [InputEvent::zeroed(); 16];
            // Scoped so the borrow of `self.file` ends before `self.handle`/
            // `self.disconnect` need `&mut self` again below.
            let result = self.file.as_mut().unwrap().read(bytemuck::cast_slice_mut(&mut events));
            match result {
                Ok(0) => {
                    self.disconnect();
                    break;
                }
                Ok(n) => {
                    for ev in &events[..n / size_of::<InputEvent>()] {
                        self.handle(ev);
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(_) => {
                    self.disconnect();
                    break;
                }
            }
        }
        self.last.clone()
    }

    fn try_open(&mut self) {
        let Some(path) = find_device("xbox", "js") else { return };
        self.file = fs::OpenOptions::new().read(true).custom_flags(libc::O_NONBLOCK).open(path).ok();
    }

    fn disconnect(&mut self) {
        self.file = None;
        self.lt_down = false;
        self.rt_down = false;
        self.last = None;
    }

    fn handle(&mut self, ev: &InputEvent) {
        match (ev.ev_type, ev.code) {
            (EV_KEY, BTN_SOUTH) if ev.value == 1 => self.last = Some("A".into()),
            (EV_KEY, BTN_EAST) if ev.value == 1 => self.last = Some("B".into()),
            (EV_KEY, BTN_NORTH) if ev.value == 1 => self.last = Some("Y".into()),
            (EV_KEY, BTN_WEST) if ev.value == 1 => self.last = Some("X".into()),
            (EV_KEY, BTN_TL) if ev.value == 1 => self.last = Some("LB".into()),
            (EV_KEY, BTN_TR) if ev.value == 1 => self.last = Some("RB".into()),
            (EV_ABS, ABS_HAT0X) | (EV_ABS, ABS_HAT0Y) if ev.value != 0 => self.last = Some("ARROW".into()),
            (EV_ABS, ABS_Z) => {
                let down = ev.value > TRIGGER_THRESHOLD;
                if down && !self.lt_down {
                    self.last = Some("LT".into());
                }
                self.lt_down = down;
            }
            (EV_ABS, ABS_RZ) => {
                let down = ev.value > TRIGGER_THRESHOLD;
                if down && !self.rt_down {
                    self.last = Some("RT".into());
                }
                self.rt_down = down;
            }
            _ => {}
        }
    }
}
