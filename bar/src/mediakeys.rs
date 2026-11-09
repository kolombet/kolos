// Last top-row function/media key pressed on the internal Apple keyboard
// (F1 brightness, F5/F6 keyboard backlight, F10 mute, F11/F12 volume, ...).
// Exists so `hid_apple`'s `fnmode` setting (which picks whether a bare Fx key
// sends the literal function key or the media code) can be checked by tapping
// a key and reading the bar, instead of capturing evdev from a shell. Limited
// to this fixed set of codes, not every key on the keyboard, so normal typing
// doesn't show up here.

use std::fs::{self, File};
use std::io::{self, Read};
use std::mem::size_of;
use std::os::unix::fs::OpenOptionsExt;

use bytemuck::Zeroable;

use crate::evdev::{find_device, InputEvent, EV_KEY};

const KEY_F1: u16 = 59;
const KEY_F2: u16 = 60;
const KEY_F3: u16 = 61;
const KEY_F4: u16 = 62;
const KEY_F5: u16 = 63;
const KEY_F6: u16 = 64;
const KEY_F7: u16 = 65;
const KEY_F8: u16 = 66;
const KEY_F9: u16 = 67;
const KEY_F10: u16 = 68;
const KEY_F11: u16 = 87;
const KEY_F12: u16 = 88;
const KEY_BRIGHTNESSDOWN: u16 = 224;
const KEY_BRIGHTNESSUP: u16 = 225;
const KEY_KBDILLUMTOGGLE: u16 = 228;
const KEY_KBDILLUMDOWN: u16 = 229;
const KEY_KBDILLUMUP: u16 = 230;
const KEY_MUTE: u16 = 113;
const KEY_VOLUMEDOWN: u16 = 114;
const KEY_VOLUMEUP: u16 = 115;
const KEY_PREVIOUSSONG: u16 = 165;
const KEY_PLAYPAUSE: u16 = 164;
const KEY_NEXTSONG: u16 = 163;
const KEY_EJECTCD: u16 = 161;

#[derive(Default)]
pub struct MediaKeys {
    file: Option<File>,
    last: Option<String>,
}

impl MediaKeys {
    /// Last recognized top-row key pressed ("F1", "BR-DOWN", "MUTE", ...), or
    /// None if the keyboard isn't found or nothing has been pressed yet.
    pub fn sample(&mut self) -> Option<String> {
        if self.file.is_none() {
            self.try_open();
        }
        loop {
            if self.file.is_none() {
                break;
            }
            let mut events = [InputEvent::zeroed(); 16];
            // Scoped so the borrow of `self.file` ends before `self.handle`
            // needs `&mut self` again below.
            let result = self.file.as_mut().unwrap().read(bytemuck::cast_slice_mut(&mut events));
            match result {
                Ok(0) => {
                    self.file = None;
                    self.last = None;
                    break;
                }
                Ok(n) => {
                    for ev in &events[..n / size_of::<InputEvent>()] {
                        self.handle(ev);
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(_) => {
                    self.file = None;
                    self.last = None;
                    break;
                }
            }
        }
        self.last.clone()
    }

    fn try_open(&mut self) {
        let Some(path) = find_device("apple internal keyboard", "kbd") else { return };
        self.file = fs::OpenOptions::new().read(true).custom_flags(libc::O_NONBLOCK).open(path).ok();
    }

    fn handle(&mut self, ev: &InputEvent) {
        if ev.ev_type != EV_KEY || ev.value != 1 {
            return;
        }
        let label = match ev.code {
            KEY_F1 => "F1",
            KEY_F2 => "F2",
            KEY_F3 => "F3",
            KEY_F4 => "F4",
            KEY_F5 => "F5",
            KEY_F6 => "F6",
            KEY_F7 => "F7",
            KEY_F8 => "F8",
            KEY_F9 => "F9",
            KEY_F10 => "F10",
            KEY_F11 => "F11",
            KEY_F12 => "F12",
            KEY_BRIGHTNESSDOWN => "BR-DOWN",
            KEY_BRIGHTNESSUP => "BR-UP",
            KEY_KBDILLUMTOGGLE => "KBD-TOGGLE",
            KEY_KBDILLUMDOWN => "KBD-DOWN",
            KEY_KBDILLUMUP => "KBD-UP",
            KEY_MUTE => "MUTE",
            KEY_VOLUMEDOWN => "VOL-DOWN",
            KEY_VOLUMEUP => "VOL-UP",
            KEY_PREVIOUSSONG => "PREV",
            KEY_PLAYPAUSE => "PLAY",
            KEY_NEXTSONG => "NEXT",
            KEY_EJECTCD => "EJECT",
            _ => return,
        };
        self.last = Some(label.to_string());
    }
}
