// Keybinding cheat sheet: a click-through layer-shell panel in the top-right
// corner of the focused output, drawn without any GUI toolkit. Shares its
// text rendering with askpass/ (kolos-askpass). Toggled from bindings.lua via
// `pkill -x kolos-keys || kolos-keys`; it just runs until killed.

#[path = "../../askpass/src/font.rs"]
mod font;

use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState, Region},
    delegate_dispatch2, delegate_registry,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
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
    protocol::{wl_output, wl_shm, wl_surface},
    Connection, QueueHandle,
};

use font::Font;

const FONT_SIZE: f32 = 15.0;
const LINE_HEIGHT: u32 = 22;
const BLANK_HEIGHT: u32 = 10;
const PAD_X: u32 = 16;
const PAD_Y: u32 = 12;
const COLUMN_GAP: u32 = 24;
const MARGIN: i32 = 20;
const BOX_RADIUS: i64 = 8;

const COLOR_BG: (u8, u8, u8) = (25, 25, 25);
const BG_ALPHA: u8 = 217; // 0.85; the old nwg-wrapper 0.65 was hard to read over text
const COLOR_HEADER: (u8, u8, u8) = (0xaa, 0xaa, 0xaa);
const COLOR_LABEL: (u8, u8, u8) = (0x88, 0x88, 0x88);
const COLOR_KEYS: (u8, u8, u8) = (0xff, 0xff, 0xff);

/// One line of the cheat-sheet file:
///   `# title`            section header
///   `label   keys`       two columns, split on the first run of 2+ spaces
///   anything else        plain note; an empty line is a small gap
enum Line {
    Blank,
    Header(String),
    Note(String),
    Entry(String, String),
}

fn parse(text: &str) -> Vec<Line> {
    text.lines()
        .map(|l| {
            let l = l.trim_end();
            if l.is_empty() {
                Line::Blank
            } else if let Some(h) = l.strip_prefix("# ") {
                Line::Header(h.to_string())
            } else if let Some(i) = l.find("  ") {
                Line::Entry(l[..i].to_string(), l[i..].trim_start().to_string())
            } else {
                Line::Note(l.to_string())
            }
        })
        .collect()
}

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

fn argb_a(color: (u8, u8, u8), a: u8) -> u32 {
    // wl_shm's Argb8888 expects premultiplied alpha.
    let premul = |c: u8| (c as u32 * a as u32) / 255;
    (a as u32) << 24 | (premul(color.0) << 16) | (premul(color.1) << 8) | premul(color.2)
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        format!("{}/.config/hypr/keys.txt", std::env::var("HOME").unwrap_or_default())
    });
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| format!("cannot read {path}:\n{e}"));
    let lines = parse(&text);

    let mut font = Font::load(FONT_SIZE);
    // Scale 1 until the compositor tells us the output's scale
    // (scale_factor_changed, right after the surface is mapped).
    let layout = Layout::compute(&mut font, &lines, 1);

    let conn = Connection::connect_to_env().expect("no wayland connection");
    let (globals, mut event_queue) = registry_queue_init(&conn).unwrap();
    let qh = event_queue.handle();

    let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor missing");
    let layer_shell = LayerShell::bind(&globals, &qh).expect("wlr-layer-shell missing");
    let shm = Shm::bind(&globals, &qh).expect("wl_shm missing");

    let surface = compositor.create_surface(&qh);
    // No output given: the compositor puts it on the focused one.
    let layer =
        layer_shell.create_layer_surface(&qh, surface, Layer::Overlay, Some("kolos-keys"), None);
    layer.set_anchor(Anchor::TOP | Anchor::RIGHT);
    layer.set_margin(MARGIN, MARGIN, 0, 0);
    layer.set_size(layout.width, layout.height);
    layer.set_keyboard_interactivity(KeyboardInteractivity::None);
    // Empty input region: clicks fall through to the windows underneath.
    let region = Region::new(&compositor).expect("failed to create region");
    layer.wl_surface().set_input_region(Some(region.wl_region()));
    layer.commit();

    let pool = SlotPool::new((layout.width * layout.height * 4) as usize, &shm)
        .expect("failed to create shm pool");

    let mut state = Keys {
        registry_state: RegistryState::new(&globals),
        output_state: OutputState::new(&globals, &qh),
        shm,
        pool,
        layer,
        font,
        lines,
        layout,
        configured: false,
        exit: false,
    };

    while !state.exit {
        event_queue.blocking_dispatch(&mut state).unwrap();
    }
}

struct Keys {
    registry_state: RegistryState,
    output_state: OutputState,
    shm: Shm,
    pool: SlotPool,
    layer: LayerSurface,
    font: Font,
    lines: Vec<Line>,
    layout: Layout,
    configured: bool,
    exit: bool,
}

/// Sizes for one buffer scale. `width`/`height` are logical (what the layer
/// surface asks for); the buffer is `scale` times that, so text is rasterized
/// at the output's real resolution instead of being upscaled by the compositor.
struct Layout {
    scale: u32,
    width: u32,
    height: u32,
    label_px: u32,
}

impl Layout {
    fn compute(font: &mut Font, lines: &[Line], scale: u32) -> Self {
        font.set_size(FONT_SIZE * scale as f32);
        let label_px = lines
            .iter()
            .filter_map(|l| match l {
                Line::Entry(label, _) => Some(font.measure(label)),
                _ => None,
            })
            .max()
            .unwrap_or(0);
        let content_px = lines
            .iter()
            .map(|l| match l {
                Line::Blank => 0,
                Line::Header(s) | Line::Note(s) => font.measure(s),
                Line::Entry(_, keys) => label_px + COLUMN_GAP * scale + font.measure(keys),
            })
            .max()
            .unwrap_or(0);
        let content_height: u32 = lines
            .iter()
            .map(|l| if matches!(l, Line::Blank) { BLANK_HEIGHT } else { LINE_HEIGHT })
            .sum();
        Self {
            scale,
            width: content_px.div_ceil(scale) + 2 * PAD_X,
            height: content_height + 2 * PAD_Y,
            label_px,
        }
    }
}

impl Keys {
    fn draw(&mut self) {
        if !self.configured {
            return;
        }
        let s = self.layout.scale;
        let (width, height) = (self.layout.width * s, self.layout.height * s);
        let (buffer, canvas) = self
            .pool
            .create_buffer(width as i32, height as i32, width as i32 * 4, wl_shm::Format::Argb8888)
            .expect("failed to create buffer");

        let pixels: &mut [u32] = bytemuck::cast_slice_mut(canvas);
        let bg = argb_a(COLOR_BG, BG_ALPHA);
        for y in 0..height {
            for x in 0..width {
                let inside =
                    in_rounded_rect(x as i64, y as i64, width as i64, height as i64, BOX_RADIUS * s as i64);
                pixels[(y * width + x) as usize] = if inside { bg } else { 0 };
            }
        }

        let (pad_x, mut y) = (PAD_X * s, PAD_Y * s);
        for line in &self.lines {
            let canvas = font::Canvas { pixels: &mut *pixels, width, height };
            match line {
                Line::Blank => {
                    y += BLANK_HEIGHT * s;
                    continue;
                }
                Line::Header(s) => {
                    self.font.render(s, COLOR_HEADER, canvas, pad_x, y);
                }
                Line::Note(s) => {
                    self.font.render(s, COLOR_LABEL, canvas, pad_x, y);
                }
                Line::Entry(label, keys) => {
                    self.font.render(label, COLOR_LABEL, canvas, pad_x, y);
                    let canvas = font::Canvas { pixels: &mut *pixels, width, height };
                    let x = pad_x + self.layout.label_px + COLUMN_GAP * s;
                    self.font.render(keys, COLOR_KEYS, canvas, x, y);
                }
            }
            y += LINE_HEIGHT * s;
        }

        self.layer.wl_surface().set_buffer_scale(s as i32);
        self.layer.wl_surface().damage_buffer(0, 0, width as i32, height as i32);
        buffer.attach_to(self.layer.wl_surface()).expect("buffer attach failed");
        self.layer.commit();
    }
}

impl CompositorHandler for Keys {
    fn scale_factor_changed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        factor: i32,
    ) {
        let scale = factor.max(1) as u32;
        if scale == self.layout.scale {
            return;
        }
        let layout = Layout::compute(&mut self.font, &self.lines, scale);
        let resized = (layout.width, layout.height) != (self.layout.width, self.layout.height);
        self.layout = layout;
        if resized {
            // Rounding can shift the logical size by a pixel; the redraw then
            // happens on the configure that answers this.
            self.layer.set_size(self.layout.width, self.layout.height);
            self.layer.commit();
        } else {
            self.draw();
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

impl OutputHandler for Keys {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }
    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
}

impl LayerShellHandler for Keys {
    fn closed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &LayerSurface) {
        self.exit = true;
    }

    fn configure(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &LayerSurface,
        _: LayerSurfaceConfigure,
        _: u32,
    ) {
        // Fixed size, static content: every configure just redraws the same image.
        self.configured = true;
        self.draw();
    }
}

impl ShmHandler for Keys {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

delegate_registry!(Keys);
delegate_dispatch2!(Keys);

impl ProvidesRegistryState for Keys {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState];
}
