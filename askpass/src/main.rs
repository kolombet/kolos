mod font;

use std::io::Write;
use std::time::Duration;

use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState, FrameCallbackData},
    delegate_dispatch2, delegate_registry,
    output::{OutputHandler, OutputState},
    reexports::{calloop::EventLoop, calloop_wayland_source::WaylandSource},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        keyboard::{KeyEvent, KeyboardHandler, Keysym, Modifiers, RawModifiers},
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
    protocol::{wl_keyboard, wl_output, wl_seat, wl_shm, wl_surface},
    Connection, QueueHandle,
};

use font::Font;

const BOX_WIDTH: u32 = 420;
const BOX_HEIGHT: u32 = 140;
const FONT_SIZE: f32 = 18.0;

const BOX_RADIUS: i64 = 14;

const COLOR_BG: (u8, u8, u8) = (0x00, 0x00, 0x00);
const BG_ALPHA: u8 = 235; // ~0.92, matching foot.ini's [colors-dark] alpha
const COLOR_PROMPT: (u8, u8, u8) = (0xd8, 0xd8, 0xdc);
const COLOR_DOTS: (u8, u8, u8) = (0xff, 0xff, 0xff);

fn in_rounded_rect(lx: i64, ly: i64, w: i64, h: i64, r: i64) -> bool {
    let cx = if lx < r {
        r
    } else if lx >= w - r {
        w - r - 1
    } else {
        return true;
    };
    let cy = if ly < r {
        r
    } else if ly >= h - r {
        h - r - 1
    } else {
        return true;
    };
    let (dx, dy) = (lx - cx, ly - cy);
    dx * dx + dy * dy <= r * r
}

fn main() {
    env_logger::init();

    let prompt: String = std::env::args().nth(1).unwrap_or_else(|| "Password: ".to_string());

    let conn = Connection::connect_to_env().expect("no wayland connection");
    let (globals, event_queue) = registry_queue_init(&conn).unwrap();
    let qh = event_queue.handle();

    let mut event_loop: EventLoop<AskPass> =
        EventLoop::try_new().expect("failed to create event loop");
    let loop_handle = event_loop.handle();
    WaylandSource::new(conn, event_queue).insert(loop_handle.clone()).unwrap();

    let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor missing");
    let layer_shell = LayerShell::bind(&globals, &qh).expect("wlr-layer-shell missing");
    let shm = Shm::bind(&globals, &qh).expect("wl_shm missing");

    let surface = compositor.create_surface(&qh);
    let layer = layer_shell.create_layer_surface(
        &qh,
        surface,
        Layer::Overlay,
        Some("kolos-askpass"),
        None,
    );
    layer.set_anchor(Anchor::all());
    layer.set_exclusive_zone(-1);
    layer.set_keyboard_interactivity(KeyboardInteractivity::Exclusive);
    layer.commit();

    let pool = SlotPool::new(1920 * 1080 * 4, &shm).expect("failed to create shm pool");

    let mut state = AskPass {
        registry_state: RegistryState::new(&globals),
        seat_state: SeatState::new(&globals, &qh),
        output_state: OutputState::new(&globals, &qh),
        shm,
        pool,
        layer,
        loop_handle,
        exit_code: None,
        first_configure: true,
        width: 1,
        height: 1,
        keyboard: None,
        font: Font::load(FONT_SIZE),
        prompt,
        buffer: String::new(),
    };

    loop {
        event_loop.dispatch(Duration::from_millis(16), &mut state).unwrap();

        if let Some(code) = state.exit_code {
            std::process::exit(code);
        }
    }
}

struct AskPass {
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    shm: Shm,
    pool: SlotPool,
    layer: LayerSurface,
    loop_handle: smithay_client_toolkit::reexports::calloop::LoopHandle<'static, AskPass>,
    exit_code: Option<i32>,
    first_configure: bool,
    width: u32,
    height: u32,
    keyboard: Option<wl_keyboard::WlKeyboard>,
    font: Font,
    prompt: String,
    buffer: String,
}

impl AskPass {
    fn submit(&mut self) {
        let mut stdout = std::io::stdout();
        let _ = writeln!(stdout, "{}", self.buffer);
        let _ = stdout.flush();
        self.exit_code = Some(0);
    }

    fn cancel(&mut self) {
        self.exit_code = Some(1);
    }

    fn handle_key(&mut self, event: KeyEvent) {
        match event.keysym {
            Keysym::Return | Keysym::KP_Enter => {
                self.submit();
                return;
            }
            Keysym::Escape => {
                self.cancel();
                return;
            }
            Keysym::BackSpace => {
                self.buffer.pop();
                return;
            }
            _ => {}
        }
        if let Some(text) = event.utf8
            && !text.chars().any(|c| c.is_control()) {
                self.buffer.push_str(&text);
            }
    }

    fn draw(&mut self, qh: &QueueHandle<Self>) {
        let width = self.width;
        let height = self.height;
        let stride = width as i32 * 4;

        let (buffer, canvas) = self
            .pool
            .create_buffer(width as i32, height as i32, stride, wl_shm::Format::Argb8888)
            .expect("failed to create buffer");

        let pixels: &mut [u32] = bytemuck::cast_slice_mut(canvas);
        pixels.fill(0);

        let box_x = width.saturating_sub(BOX_WIDTH) / 2;
        let box_y = height.saturating_sub(BOX_HEIGHT) / 2;

        for y in box_y..(box_y + BOX_HEIGHT).min(height) {
            for x in box_x..(box_x + BOX_WIDTH).min(width) {
                let (lx, ly) = ((x - box_x) as i64, (y - box_y) as i64);
                if !in_rounded_rect(lx, ly, BOX_WIDTH as i64, BOX_HEIGHT as i64, BOX_RADIUS) {
                    continue;
                }
                pixels[(y * width + x) as usize] = argb_a(COLOR_BG, BG_ALPHA);
            }
        }

        let pad = 24;
        self.font.render(
            &self.prompt,
            COLOR_PROMPT,
            font::Canvas { pixels, width, height },
            box_x + pad,
            box_y + pad,
        );

        let dots: String = "\u{2022}".repeat(self.buffer.chars().count());
        self.font.render(
            &dots,
            COLOR_DOTS,
            font::Canvas { pixels, width, height },
            box_x + pad,
            box_y + pad + FONT_SIZE as u32 + 16,
        );

        self.layer.wl_surface().damage_buffer(0, 0, width as i32, height as i32);
        self.layer
            .wl_surface()
            .frame(qh, FrameCallbackData(self.layer.wl_surface().clone()));
        buffer.attach_to(self.layer.wl_surface()).expect("buffer attach failed");
        self.layer.commit();
    }
}

fn argb_a(color: (u8, u8, u8), a: u8) -> u32 {
    // wl_shm's Argb8888 expects premultiplied alpha.
    let premul = |c: u8| (c as u32 * a as u32) / 255;
    (a as u32) << 24 | (premul(color.0) << 16) | (premul(color.1) << 8) | premul(color.2)
}

impl CompositorHandler for AskPass {
    fn scale_factor_changed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: i32,
    ) {
    }

    fn transform_changed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: wl_output::Transform,
    ) {
    }

    fn frame(&mut self, _: &Connection, qh: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: u32) {
        self.draw(qh);
    }

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

impl OutputHandler for AskPass {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }
    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
}

impl LayerShellHandler for AskPass {
    fn closed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &LayerSurface) {
        if self.exit_code.is_none() {
            self.exit_code = Some(1);
        }
    }

    fn configure(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        _: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _: u32,
    ) {
        if configure.new_size.0 > 0 && configure.new_size.1 > 0 {
            self.width = configure.new_size.0;
            self.height = configure.new_size.1;
        }
        if self.first_configure {
            self.first_configure = false;
            self.draw(qh);
        }
    }
}

impl SeatHandler for AskPass {
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
        if capability == Capability::Keyboard && self.keyboard.is_none() {
            let keyboard = self
                .seat_state
                .get_keyboard_with_repeat(
                    qh,
                    &seat,
                    None,
                    self.loop_handle.clone(),
                    Box::new(|state, _, event| state.handle_key(event)),
                )
                .expect("failed to create keyboard");
            self.keyboard = Some(keyboard);
        }
    }

    fn remove_capability(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Keyboard
            && let Some(k) = self.keyboard.take() {
                k.release();
            }
    }

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl KeyboardHandler for AskPass {
    fn enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: &wl_surface::WlSurface,
        _: u32,
        _: &[u32],
        _: &[Keysym],
    ) {
    }

    fn leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: &wl_surface::WlSurface,
        _: u32,
    ) {
    }

    fn press_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        event: KeyEvent,
    ) {
        self.handle_key(event);
    }

    fn release_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        _: KeyEvent,
    ) {
    }

    fn update_modifiers(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        _: Modifiers,
        _: RawModifiers,
        _: u32,
    ) {
    }

    fn repeat_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        event: KeyEvent,
    ) {
        self.handle_key(event);
    }
}

impl ShmHandler for AskPass {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

delegate_registry!(AskPass);
delegate_dispatch2!(AskPass);

impl ProvidesRegistryState for AskPass {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState, SeatState];
}
