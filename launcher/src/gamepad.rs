// Gamepad input for controller mode, read straight from the pad's evdev node
// (same approach as kolos-bar's gamepad.rs): an Xbox controller, or a
// handheld's built-in controls that InputPlumber exposes as
// "Microsoft X-Box 360 pad". Needs read access to /dev/input/event*, which
// the `input` group gives.

use std::fs::{self, File, OpenOptions};
use std::io;
use std::mem::size_of;
use std::os::fd::AsRawFd;
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;
use std::time::{Duration, Instant};

const DEVICE_NAMES: &[&str] = &["xbox", "x-box"];
// Handhelds whose printed A and B are the reverse of what InputPlumber reports.
const AB_SWAPPED_HANDHELDS: &[&str] = &["ayn odin2 gamepad"];

const EV_KEY: u16 = 0x01;
const EV_ABS: u16 = 0x03;
const BTN_SOUTH: u16 = 0x130;
const BTN_EAST: u16 = 0x131;
const BTN_NORTH: u16 = 0x133; // the button printed "Y" (kernel calls it BTN_X: historical alias)
const BTN_WEST: u16 = 0x134; // the button printed "X" (kernel: BTN_Y)
const BTN_TL: u16 = 0x136;
const BTN_TR: u16 = 0x137;
const BTN_SELECT: u16 = 0x13a;
const BTN_START: u16 = 0x13b;
const ABS_X: u16 = 0x00;
const ABS_Y: u16 = 0x01;
const ABS_HAT0X: u16 = 0x10;
const ABS_HAT0Y: u16 = 0x11;

const STICK_THRESHOLD: i32 = 16000;
const REPEAT_DELAY: Duration = Duration::from_millis(350);
const REPEAT_RATE: Duration = Duration::from_millis(110);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PadAction {
    Up,
    Down,
    Left,
    Right,
    PageUp,
    PageDown,
    Confirm, // A
    Back,    // B
    Pin,     // X
    Switch,  // Y: favourites <-> all apps
    Exit,    // Start
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Dir {
    None,
    Up,
    Down,
    Left,
    Right,
}

impl Dir {
    const fn action(self) -> Option<PadAction> {
        match self {
            Self::None => None,
            Self::Up => Some(PadAction::Up),
            Self::Down => Some(PadAction::Down),
            Self::Left => Some(PadAction::Left),
            Self::Right => Some(PadAction::Right),
        }
    }
}

pub struct Gamepad {
    file: File,
    swap_ab: bool,
    // Select+button chords belong to Hyprland's gamepad-shortcuts.py (Select+A
    // toggles this launcher), so buttons pressed while Select is held are ignored.
    select_held: bool,
    hat: (i32, i32),
    stick: (i32, i32),
    held: Dir,
    next_repeat: Option<Instant>,
}

fn device_names() -> Vec<(PathBuf, String)> {
    let Ok(dir) = fs::read_dir("/sys/class/input") else { return Vec::new() };
    dir.filter_map(Result::ok)
        .filter_map(|entry| {
            let name = entry.file_name().into_string().ok()?;
            let label = fs::read_to_string(entry.path().join("device/name")).ok()?;
            Some((PathBuf::from("/dev/input").join(name), label.trim().to_lowercase()))
        })
        .collect()
}

impl Gamepad {
    /// The first connected XInput-style pad, if any.
    pub fn open() -> Option<Self> {
        let devices = device_names();
        let swap_handheld = devices.iter().any(|(_, n)| AB_SWAPPED_HANDHELDS.contains(&n.as_str()));
        for (path, name) in &devices {
            let is_event = path.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with("event"));
            if !is_event || !DEVICE_NAMES.iter().any(|d| name.contains(d)) {
                continue;
            }
            let Ok(file) = OpenOptions::new().read(true).custom_flags(libc::O_NONBLOCK).open(path) else {
                continue;
            };
            // Only InputPlumber's virtual pad is the handheld's own controls; a
            // real Xbox controller plugged into the same device keeps its layout.
            let event = path.file_name().unwrap().to_string_lossy().into_owned();
            let virtual_pad = fs::canonicalize(format!("/sys/class/input/{event}/device"))
                .is_ok_and(|p| p.starts_with("/sys/devices/virtual"));
            return Some(Self {
                file,
                swap_ab: swap_handheld && virtual_pad,
                select_held: false,
                hat: (0, 0),
                stick: (0, 0),
                held: Dir::None,
                next_repeat: None,
            });
        }
        None
    }

    pub fn fd(&self) -> &File {
        &self.file
    }

    /// Reads every pending event. Err once the device is gone.
    pub fn drain(&mut self, out: &mut Vec<PadAction>) -> io::Result<()> {
        const N: usize = 32;
        let mut events: [libc::input_event; N] = unsafe { std::mem::zeroed() };
        loop {
            let n = unsafe {
                libc::read(
                    self.file.as_raw_fd(),
                    events.as_mut_ptr().cast(),
                    size_of::<libc::input_event>() * N,
                )
            };
            if n < 0 {
                let err = io::Error::last_os_error();
                return if err.kind() == io::ErrorKind::WouldBlock { Ok(()) } else { Err(err) };
            }
            if n == 0 {
                return Err(io::ErrorKind::UnexpectedEof.into());
            }
            for ev in &events[..n as usize / size_of::<libc::input_event>()] {
                self.handle(ev.type_, ev.code, ev.value, out);
            }
        }
    }

    fn handle(&mut self, kind: u16, code: u16, value: i32, out: &mut Vec<PadAction>) {
        match (kind, code) {
            (EV_KEY, BTN_SELECT) => {
                self.select_held = value != 0;
                return;
            }
            (EV_KEY, _) if self.select_held => return,
            (EV_KEY, code) if value == 1 => {
                let code = match code {
                    BTN_SOUTH if self.swap_ab => BTN_EAST,
                    BTN_EAST if self.swap_ab => BTN_SOUTH,
                    other => other,
                };
                let action = match code {
                    BTN_SOUTH => PadAction::Confirm,
                    BTN_EAST => PadAction::Back,
                    BTN_WEST => PadAction::Pin,
                    BTN_NORTH => PadAction::Switch,
                    BTN_TL => PadAction::PageUp,
                    BTN_TR => PadAction::PageDown,
                    BTN_START => PadAction::Exit,
                    _ => return,
                };
                out.push(action);
            }
            (EV_ABS, ABS_HAT0X) => self.hat.0 = value,
            (EV_ABS, ABS_HAT0Y) => self.hat.1 = value,
            (EV_ABS, ABS_X) => self.stick.0 = value,
            (EV_ABS, ABS_Y) => self.stick.1 = value,
            _ => return,
        }
        let dir = self.direction();
        if dir != self.held {
            self.held = dir;
            if let Some(action) = dir.action() {
                out.push(action);
                self.next_repeat = Some(Instant::now() + REPEAT_DELAY);
            } else {
                self.next_repeat = None;
            }
        }
    }

    fn direction(&self) -> Dir {
        let (mut x, mut y) = self.hat;
        if x == 0 && y == 0 {
            let axis = |v: i32| if v < -STICK_THRESHOLD { -1 } else if v > STICK_THRESHOLD { 1 } else { 0 };
            // Dominant axis only, so a diagonal stick doesn't jitter between two.
            if self.stick.0.abs() > self.stick.1.abs() {
                x = axis(self.stick.0);
            } else {
                y = axis(self.stick.1);
            }
        }
        match (x, y) {
            (_, -1) => Dir::Up,
            (_, 1) => Dir::Down,
            (-1, _) => Dir::Left,
            (1, _) => Dir::Right,
            _ => Dir::None,
        }
    }

    /// Key-repeat for a held direction; call regularly from the event loop.
    pub fn repeat(&mut self, out: &mut Vec<PadAction>) {
        if let (Some(at), Some(action)) = (self.next_repeat, self.held.action()) {
            if Instant::now() >= at {
                out.push(action);
                self.next_repeat = Some(at + REPEAT_RATE);
            }
        }
    }
}
