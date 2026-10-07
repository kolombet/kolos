// Shared by askpass/ (kolos-askpass), keys/ (kolos-keys) and bar/ (kolos-bar)
// via `#[path]`, so a change here applies to all three.
//
// ab_glyph (ttf-parser underneath) reads glyph outlines lazily from the font
// bytes. fontdue, used before, parsed every glyph up front, which for the
// ~10k-glyph Nerd Font meant ~40 MiB of heap per process.

use ab_glyph::{Font as _, FontVec, PxScale, ScaleFont};

pub struct Canvas<'a> {
    pub pixels: &'a mut [u32],
    pub width: u32,
    pub height: u32,
}

pub struct Font {
    inner: FontVec,
    size: f32,
}

impl Font {
    #[allow(dead_code)] // bar/ always picks a style
    pub fn load(size: f32) -> Self {
        Self::load_style(size, None)
    }

    /// `style` is a fontconfig style name such as "SemiBold"; `None` picks the default.
    pub fn load_style(size: f32, style: Option<&str>) -> Self {
        let fc = fontconfig::Fontconfig::new().expect("unable to initialize fontconfig");
        let font = fc
            .find("JetBrainsMono Nerd Font Mono", style)
            .expect("fontconfig found no matching font");

        let bytes = std::fs::read(&font.path).expect("failed to read font file");
        let inner = FontVec::try_from_vec(bytes).expect("failed to parse font file");

        Self { inner, size }
    }

    #[allow(dead_code)] // only used by bar/, which draws at several output scales
    pub fn set_size(&mut self, size: f32) {
        self.size = size;
    }

    /// ab_glyph's PxScale is the font's full ascent-to-descent height; `size` means
    /// the em size (as fontdue and CSS use it), so convert.
    fn px_scale(&self) -> PxScale {
        let em = self.inner.units_per_em().unwrap_or(1000.0);
        let height = self.inner.ascent_unscaled() - self.inner.descent_unscaled();
        PxScale::from(self.size * height / em)
    }

    /// Advance width of `text` in pixels, without drawing it.
    #[allow(dead_code)] // not used by askpass/
    pub fn measure(&self, text: &str) -> u32 {
        let scaled = self.inner.as_scaled(self.px_scale());
        text.chars().map(|c| scaled.h_advance(scaled.glyph_id(c))).sum::<f32>().round() as u32
    }

    /// Blends `text` into `canvas` with top-left origin at (x, y). Returns the advanced
    /// width in pixels.
    pub fn render(&self, text: &str, color: (u8, u8, u8), canvas: Canvas, x: u32, y: u32) -> u32 {
        let Canvas { pixels, width: canvas_width, height: canvas_height } = canvas;
        let scaled = self.inner.as_scaled(self.px_scale());
        let baseline = y as f32 + scaled.ascent();
        let mut pen_x = x as f32;

        for c in text.chars() {
            let id = scaled.glyph_id(c);
            let glyph = id.with_scale_and_position(scaled.scale(), ab_glyph::point(pen_x, baseline));
            if let Some(outlined) = self.inner.outline_glyph(glyph) {
                let bounds = outlined.px_bounds();
                outlined.draw(|gx, gy, coverage| {
                    let px = bounds.min.x as i32 + gx as i32;
                    let py = bounds.min.y as i32 + gy as i32;
                    if px < 0 || py < 0 || px as u32 >= canvas_width || py as u32 >= canvas_height {
                        return;
                    }
                    let coverage = (coverage.clamp(0.0, 1.0) * 255.0) as u8;
                    if coverage == 0 {
                        return;
                    }
                    let idx = (py as u32 * canvas_width + px as u32) as usize;
                    pixels[idx] = blend(pixels[idx], color, coverage);
                });
            }
            pen_x += scaled.h_advance(id);
        }

        (pen_x - x as f32).round().max(0.0) as u32
    }
}

/// Premultiplied-alpha "over": an opaque `color` at `coverage` on top of `dst`
/// (wl_shm Argb8888 is premultiplied), so text on a translucent background stays
/// translucent around the glyph edges instead of turning those pixels opaque.
fn blend(dst: u32, color: (u8, u8, u8), coverage: u8) -> u32 {
    let a = coverage as u32;
    let inv_a = 255 - a;
    let over = |src: u32, shift: u32| (src * a + ((dst >> shift) & 0xff) * inv_a) / 255;

    (over(255, 24) << 24)
        | (over(color.0 as u32, 16) << 16)
        | (over(color.1 as u32, 8) << 8)
        | over(color.2 as u32, 0)
}
