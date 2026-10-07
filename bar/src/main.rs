// kolos-bar: the bottom bar, replacing Waybar (GTK3) with a toolkit-free
// layer-shell surface per output, drawn into shm buffers like askpass/ and
// keys/ (whose font code it shares). Left: workspaces and a taskbar of the
// output's windows; right: CPU, memory, keyboard layout, battery, clock.
// State comes from Hyprland's IPC sockets (see hypr.rs) and /proc (stats.rs).

#[path = "../../askpass/src/font.rs"]
mod font;
mod hypr;
mod stats;

use std::io::Read;
use std::time::Duration;

use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_dispatch2, delegate_registry,
    output::{OutputHandler, OutputState},
    reexports::{
        calloop::{
            generic::Generic,
            timer::{TimeoutAction, Timer},
            EventLoop, Interest, Mode, PostAction,
        },
        calloop_wayland_source::WaylandSource,
    },
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        pointer::{PointerEvent, PointerEventKind, PointerHandler},
        Capability, SeatHandler, SeatState,
    },
    shell::{
        wlr_layer::{
            Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
            LayerSurfaceConfigure,
        },
        WaylandSurface,
    },
    shm::{slot::SlotPool, Shm, ShmHandler},
};
use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_output, wl_pointer, wl_seat, wl_shm, wl_surface},
    Connection, QueueHandle,
};

use font::Font;
use hypr::Snapshot;
use stats::{Sampler, Stats};

// All sizes are logical pixels; drawing multiplies by the output's scale.
const HEIGHT: u32 = 42;
const FONT_SIZE: f32 = 13.0;
const PERSISTENT_WORKSPACES: i64 = 5;
const WS_WIDTH: u32 = 20; // minimum; two-digit ids grow to fit
const WS_PAD: u32 = 4;
const TASK_PAD: u32 = 8;
const TASK_MAX_CHARS: usize = 16;
const UNDERLINE: u32 = 3;
const SECTION_GAP: u32 = 12;
const TICK: Duration = Duration::from_secs(2);

// Colors follow the old waybar/style.css.
const BG: (u8, u8, u8, u8) = (25, 25, 25, 166); // rgba(25,25,25,.65)
const BORDER_TOP: (u8, u8, u8, u8) = (255, 255, 255, 51); // rgba(255,255,255,.2)
const ACTIVE_BG: (u8, u8, u8, u8) = (100, 100, 100, 128); // rgba(100,100,100,.5)
const WHITE: (u8, u8, u8, u8) = (255, 255, 255, 255);
const DIM_LINE: (u8, u8, u8, u8) = (255, 255, 255, 77); // rgba(255,255,255,.3)
const TEXT: (u8, u8, u8) = (0xff, 0xff, 0xff);
const TEXT_OCCUPIED: (u8, u8, u8) = (0xb0, 0xb0, 0xb0);
const TEXT_EMPTY: (u8, u8, u8) = (0x60, 0x60, 0x60);


#[derive(Clone)]
enum Hit {
    Workspace(i64),
    Window(String),
}

struct Bar {
    output: wl_output::WlOutput,
    name: String,
    layer: LayerSurface,
    width: u32,
    scale: i32,
    configured: bool,
    /// (x0, x1) in logical pixels.
    hits: Vec<(f64, f64, Hit)>,
}

struct App {
    registry_state: RegistryState,
    output_state: OutputState,
    seat_state: SeatState,
    compositor: CompositorState,
    layer_shell: LayerShell,
    shm: Shm,
    pool: SlotPool,
    pointer: Option<wl_pointer::WlPointer>,
    font: Font,
    bars: Vec<Bar>,
    hypr: Snapshot,
    /// Window addresses in the order the taskbar shows them: first seen first.
    /// Hyprland's `j/clients` comes in stacking order, which changes whenever a
    /// window is raised (every focus change does that, see bindings.lua), so the
    /// taskbar can't follow it.
    window_order: Vec<String>,
    stats: Stats,
    sampler: Sampler,
    exit: bool,
}

fn main() {
    let conn = Connection::connect_to_env().expect("no wayland connection");
    let (globals, event_queue) = registry_queue_init(&conn).unwrap();
    let qh = event_queue.handle();

    let mut event_loop: EventLoop<App> = EventLoop::try_new().expect("failed to create event loop");
    let loop_handle = event_loop.handle();
    WaylandSource::new(conn, event_queue).insert(loop_handle.clone()).unwrap();

    let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor missing");
    let layer_shell = LayerShell::bind(&globals, &qh).expect("wlr-layer-shell missing");
    let shm = Shm::bind(&globals, &qh).expect("wl_shm missing");
    let pool = SlotPool::new(4096 * HEIGHT as usize * 4, &shm).expect("failed to create shm pool");

    let mut sampler = Sampler::default();
    let stats = sampler.sample();
    let mut app = App {
        registry_state: RegistryState::new(&globals),
        output_state: OutputState::new(&globals, &qh),
        seat_state: SeatState::new(&globals, &qh),
        compositor,
        layer_shell,
        shm,
        pool,
        pointer: None,
        font: Font::load_style(FONT_SIZE, Some("SemiBold")),
        bars: Vec::new(),
        hypr: Snapshot::default(),
        window_order: Vec::new(),
        stats,
        sampler,
        exit: false,
    };
    app.hypr = app.ordered(Snapshot::fetch());

    let events = hypr::event_stream().expect("cannot connect to Hyprland's event socket");
    events.set_nonblocking(true).unwrap();
    let mut pending = String::new();
    loop_handle
        .insert_source(Generic::new(events, Interest::READ, Mode::Level), move |_, stream, app| {
            let mut buf = [0u8; 4096];
            let mut relevant = false;
            loop {
                match (&**stream).read(&mut buf) {
                    Ok(0) => {
                        app.exit = true; // Hyprland went away
                        return Ok(PostAction::Remove);
                    }
                    Ok(n) => pending.push_str(&String::from_utf8_lossy(&buf[..n])),
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                    Err(e) => return Err(e),
                }
            }
            while let Some(end) = pending.find('\n') {
                let line: String = pending.drain(..=end).collect();
                let event = line.split(">>").next().unwrap_or_default();
                relevant |= hypr::is_relevant(event);
            }
            if relevant {
                app.refresh_hypr();
            }
            Ok(PostAction::Continue)
        })
        .unwrap();

    loop_handle
        // First tick soon: CPU usage needs two samples, the first one was taken above.
        .insert_source(Timer::from_duration(Duration::from_millis(500)), |_, _, app| {
            let stats = app.sampler.sample();
            if stats != app.stats {
                app.stats = stats;
                app.draw_all();
            }
            TimeoutAction::ToDuration(TICK)
        })
        .unwrap();

    while !app.exit {
        event_loop.dispatch(None, &mut app).unwrap();
    }
}

impl App {
    /// Sorts the snapshot's windows by `window_order`, updating it: closed windows
    /// drop out, new ones go to the end.
    fn ordered(&mut self, mut snapshot: Snapshot) -> Snapshot {
        self.window_order.retain(|a| snapshot.clients.iter().any(|c| &c.address == a));
        for c in &snapshot.clients {
            if !self.window_order.contains(&c.address) {
                self.window_order.push(c.address.clone());
            }
        }
        let order = &self.window_order;
        snapshot.clients.sort_by_key(|c| order.iter().position(|a| *a == c.address));
        snapshot
    }

    fn refresh_hypr(&mut self) {
        let snapshot = self.ordered(Snapshot::fetch());
        if snapshot != self.hypr {
            self.hypr = snapshot;
            self.draw_all();
        }
    }

    fn add_bar(&mut self, qh: &QueueHandle<Self>, output: wl_output::WlOutput) {
        let info = self.output_state.info(&output);
        let name = info.as_ref().and_then(|i| i.name.clone()).unwrap_or_default();
        let scale = info.map(|i| i.scale_factor).unwrap_or(1).max(1);

        let surface = self.compositor.create_surface(qh);
        let layer = self.layer_shell.create_layer_surface(
            qh,
            surface,
            Layer::Top,
            Some("kolos-bar"),
            Some(&output),
        );
        layer.set_anchor(Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT);
        layer.set_size(0, HEIGHT);
        layer.set_exclusive_zone(HEIGHT as i32);
        layer.set_keyboard_interactivity(KeyboardInteractivity::None);
        layer.wl_surface().set_buffer_scale(scale);
        layer.commit();

        self.bars.push(Bar {
            output,
            name,
            layer,
            width: 0,
            scale,
            configured: false,
            hits: Vec::new(),
        });
    }

    fn draw_all(&mut self) {
        for i in 0..self.bars.len() {
            self.draw(i);
        }
    }

    fn draw(&mut self, index: usize) {
        let bar = &self.bars[index];
        if !bar.configured || bar.width == 0 {
            return;
        }
        let s = bar.scale as u32;
        let (width, height) = (bar.width * s, HEIGHT * s);
        let Ok((buffer, canvas)) = self.pool.create_buffer(
            width as i32,
            height as i32,
            width as i32 * 4,
            wl_shm::Format::Argb8888,
        ) else {
            return;
        };
        let pixels: &mut [u32] = bytemuck::cast_slice_mut(canvas);
        pixels.fill(premul(BG));
        let mut painter = Painter { pixels, width, height, scale: s, font: &mut self.font };
        painter.font.set_size(FONT_SIZE * s as f32);
        painter.rect(0, 0, bar.width, 1, BORDER_TOP);

        let mut hits = Vec::new();
        let text_y = (HEIGHT - FONT_SIZE as u32) / 2 - 1;

        // Workspaces: 1..=5 always, plus any other workspace on this output.
        let active_ws = self
            .hypr
            .monitors
            .iter()
            .find(|(name, _)| *name == bar.name)
            .map(|(_, ws)| *ws)
            .unwrap_or(0);
        let mut ids: Vec<i64> = (1..=PERSISTENT_WORKSPACES).collect();
        for w in &self.hypr.workspaces {
            if w.monitor == bar.name && !ids.contains(&w.id) {
                ids.push(w.id);
            }
        }
        ids.sort();
        let mut x = 4;
        for id in ids {
            let windows = self.hypr.workspaces.iter().find(|w| w.id == id).map_or(0, |w| w.windows);
            let (color, underline) = if id == active_ws {
                (TEXT, true)
            } else if windows > 0 {
                (TEXT_OCCUPIED, false)
            } else {
                (TEXT_EMPTY, false)
            };
            let label = id.to_string();
            let tw = painter.measure(&label);
            let w = WS_WIDTH.max(tw + 2 * WS_PAD);
            painter.text(&label, color, x + (w - tw) / 2, text_y);
            if underline {
                painter.rect(x, HEIGHT - UNDERLINE, w, UNDERLINE, WHITE);
            }
            hits.push((x as f64, (x + w) as f64, Hit::Workspace(id)));
            x += w;
        }

        // Taskbar: this output's windows, by workspace, then in first-seen order
        // (the sort is stable and `clients` is already in that order).
        x += SECTION_GAP;
        let mut clients: Vec<_> = self.hypr.clients.iter().filter(|c| c.monitor == bar.name).collect();
        clients.sort_by_key(|c| c.workspace);
        for c in clients {
            let label = short_class(&c.class);
            let w = painter.measure(&label) + 2 * TASK_PAD;
            let active = c.address == self.hypr.active_window;
            if active {
                painter.rect(x, 1, w, HEIGHT - 1, ACTIVE_BG);
            }
            painter.rect(x + 2, HEIGHT - UNDERLINE, w - 4, UNDERLINE, if active { WHITE } else { DIM_LINE });
            painter.text(&label, TEXT, x + TASK_PAD, text_y);
            hits.push((x as f64, (x + w) as f64, Hit::Window(c.address.clone())));
            x += w + 4;
        }

        // Right side, laid out from the right edge: clock (two lines), then sysinfo.
        let st = &self.stats;
        let clock_w = painter.measure(&st.time).max(painter.measure(&st.date)) + 2 * TASK_PAD;
        let clock_x = bar.width.saturating_sub(clock_w);
        let line_h = FONT_SIZE as u32 + 2;
        let top = (HEIGHT - 2 * line_h) / 2;
        let tw = painter.measure(&st.time);
        painter.text(&st.time, TEXT, clock_x + (clock_w - tw) / 2, top);
        let dw = painter.measure(&st.date);
        painter.text(&st.date, TEXT, clock_x + (clock_w - dw) / 2, top + line_h);

        // Plain text, like the live Waybar: the Mono Nerd Font squeezes icons to one
        // cell, too small to read at this size.
        let mut info = format!(
            "{}%   {:.2}/{:.2}GiB   {}",
            st.cpu_percent, st.mem_used_gib, st.mem_total_gib, self.hypr.layout
        );
        if let Some((capacity, charging)) = st.battery {
            let plug = if charging { "+" } else { "" };
            info = format!("BAT {capacity}%{plug}   {info}");
        }
        let iw = painter.measure(&info);
        painter.text(&info, TEXT, clock_x.saturating_sub(iw + SECTION_GAP), text_y);

        let bar = &mut self.bars[index];
        bar.hits = hits;
        bar.layer.wl_surface().damage_buffer(0, 0, width as i32, height as i32);
        buffer.attach_to(bar.layer.wl_surface()).expect("buffer attach failed");
        bar.layer.commit();
    }

    fn click(&mut self, surface: &wl_surface::WlSurface, x: f64, button: u32) {
        let Some(bar) = self.bars.iter().find(|b| b.layer.wl_surface() == surface) else {
            return;
        };
        let Some((_, _, hit)) = bar.hits.iter().find(|(x0, x1, _)| x >= *x0 && x < *x1) else {
            return;
        };
        const BTN_LEFT: u32 = 0x110;
        const BTN_MIDDLE: u32 = 0x112;
        match (hit, button) {
            (Hit::Workspace(id), BTN_LEFT) => hypr::dispatch(&format!("hl.dsp.focus({{ workspace = {id} }})")),
            (Hit::Window(addr), BTN_LEFT) => {
                hypr::dispatch(&format!("hl.dsp.focus({{ window = 'address:{addr}' }})"))
            }
            (Hit::Window(addr), BTN_MIDDLE) => {
                hypr::dispatch(&format!("hl.dsp.window.close({{ window = 'address:{addr}' }})"))
            }
            _ => {}
        }
    }
}

/// "com.system76.CosmicFiles" -> "CosmicFiles", truncated with an ellipsis.
fn short_class(class: &str) -> String {
    let name = class.rsplit('.').next().unwrap_or(class);
    let name = if name.is_empty() { "?" } else { name };
    if name.chars().count() > TASK_MAX_CHARS {
        let cut: String = name.chars().take(TASK_MAX_CHARS - 1).collect();
        format!("{cut}\u{2026}")
    } else {
        name.to_string()
    }
}

fn premul((r, g, b, a): (u8, u8, u8, u8)) -> u32 {
    let p = |c: u8| c as u32 * a as u32 / 255;
    (a as u32) << 24 | p(r) << 16 | p(g) << 8 | p(b)
}

/// Draws in logical coordinates onto a buffer of `scale`x physical pixels.
struct Painter<'a> {
    pixels: &'a mut [u32],
    width: u32,
    height: u32,
    scale: u32,
    font: &'a mut Font,
}

impl Painter<'_> {
    fn measure(&self, text: &str) -> u32 {
        self.font.measure(text).div_ceil(self.scale)
    }

    fn text(&mut self, text: &str, color: (u8, u8, u8), x: u32, y: u32) {
        let canvas = font::Canvas { pixels: self.pixels, width: self.width, height: self.height };
        self.font.render(text, color, canvas, x * self.scale, y * self.scale);
    }

    /// Premultiplied "over" fill.
    fn rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: (u8, u8, u8, u8)) {
        let src = premul(color);
        let inv = 255 - color.3 as u32;
        let s = self.scale;
        for py in (y * s)..((y + h) * s).min(self.height) {
            for px in (x * s)..((x + w) * s).min(self.width) {
                let i = (py * self.width + px) as usize;
                let dst = self.pixels[i];
                let ch = |shift: u32| ((src >> shift) & 0xff) + ((dst >> shift) & 0xff) * inv / 255;
                self.pixels[i] = ch(24) << 24 | ch(16) << 16 | ch(8) << 8 | ch(0);
            }
        }
    }
}

impl CompositorHandler for App {
    fn scale_factor_changed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        surface: &wl_surface::WlSurface,
        factor: i32,
    ) {
        if let Some(i) = self.bars.iter().position(|b| b.layer.wl_surface() == surface) {
            self.bars[i].scale = factor.max(1);
            surface.set_buffer_scale(self.bars[i].scale);
            self.draw(i);
        }
    }

    fn transform_changed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: wl_output::Transform,
    ) {
    }

    fn frame(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: u32) {}

    fn surface_enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: &wl_output::WlOutput,
    ) {
    }

    fn surface_leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: &wl_output::WlOutput,
    ) {
    }
}

impl OutputHandler for App {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(&mut self, _: &Connection, qh: &QueueHandle<Self>, output: wl_output::WlOutput) {
        self.add_bar(qh, output);
    }

    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, output: wl_output::WlOutput) {
        let Some(info) = self.output_state.info(&output) else { return };
        if let Some(i) = self.bars.iter().position(|b| b.output == output) {
            self.bars[i].name = info.name.unwrap_or_default();
            self.bars[i].scale = info.scale_factor.max(1);
            self.bars[i].layer.wl_surface().set_buffer_scale(self.bars[i].scale);
            self.draw(i);
        }
    }

    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, output: wl_output::WlOutput) {
        self.bars.retain(|b| b.output != output);
    }
}

impl LayerShellHandler for App {
    fn closed(&mut self, _: &Connection, _: &QueueHandle<Self>, layer: &LayerSurface) {
        self.bars.retain(|b| &b.layer != layer);
    }

    fn configure(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _: u32,
    ) {
        if let Some(i) = self.bars.iter().position(|b| &b.layer == layer) {
            if configure.new_size.0 > 0 {
                self.bars[i].width = configure.new_size.0;
            }
            self.bars[i].configured = true;
            self.draw(i);
        }
    }
}

impl SeatHandler for App {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }
    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}

    fn new_capability(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Pointer && self.pointer.is_none() {
            self.pointer = self.seat_state.get_pointer(qh, &seat).ok();
        }
    }

    fn remove_capability(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Pointer
            && let Some(p) = self.pointer.take()
        {
            p.release();
        }
    }

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl PointerHandler for App {
    fn pointer_frame(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_pointer::WlPointer,
        events: &[PointerEvent],
    ) {
        for event in events {
            match &event.kind {
                PointerEventKind::Press { button, .. } => {
                    self.click(&event.surface, event.position.0, *button);
                }
                PointerEventKind::Axis { vertical, .. } => {
                    // Wheel notches only (value120/discrete); touchpad scrolling
                    // would otherwise flip through workspaces far too fast.
                    let steps = if vertical.value120 != 0 { vertical.value120 } else { vertical.discrete };
                    if steps != 0 {
                        let dir = if steps > 0 { "e+1" } else { "e-1" };
                        hypr::dispatch(&format!("hl.dsp.focus({{ workspace = '{dir}' }})"));
                    }
                }
                _ => {}
            }
        }
    }
}

impl ShmHandler for App {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

delegate_registry!(App);
delegate_dispatch2!(App);

impl ProvidesRegistryState for App {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState, SeatState];
}
