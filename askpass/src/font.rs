use std::collections::HashMap;

pub struct Canvas<'a> {
    pub pixels: &'a mut [u32],
    pub width: u32,
    pub height: u32,
}

pub struct Font {
    inner: fontdue::Font,
    size: f32,
    glyph_cache: HashMap<char, (fontdue::Metrics, Vec<u8>)>,
}

impl Font {
    pub fn load(size: f32) -> Self {
        let fc = fontconfig::Fontconfig::new().expect("unable to initialize fontconfig");
        let font = fc
            .find("JetBrainsMono Nerd Font Mono", None)
            .expect("fontconfig found no matching font");

        let bytes = std::fs::read(&font.path).expect("failed to read font file");
        let inner = fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default())
            .expect("failed to parse font file");

        Self { inner, size, glyph_cache: HashMap::new() }
    }

    fn glyph(&mut self, c: char) -> &(fontdue::Metrics, Vec<u8>) {
        self.glyph_cache
            .entry(c)
            .or_insert_with(|| self.inner.rasterize(c, self.size))
    }

    /// Advance width of `text` in pixels, without drawing it.
    #[allow(dead_code)] // only used by keys/ (kolos-keys), which shares this file
    pub fn measure(&mut self, text: &str) -> u32 {
        text.chars().map(|c| self.glyph(c).0.advance_width).sum::<f32>().round() as u32
    }

    /// Blends `text` into `canvas` with top-left origin at (x, y). Returns the advanced
    /// width in pixels.
    pub fn render(&mut self, text: &str, color: (u8, u8, u8), canvas: Canvas, x: u32, y: u32) -> u32 {
        let Canvas { pixels, width: canvas_width, height: canvas_height } = canvas;
        let ascent = self.size as i32;
        let mut pen_x = x as i32;

        for c in text.chars() {
            let (metrics, bitmap) = self.glyph(c);
            let (metrics, bitmap) = (*metrics, bitmap.clone());

            let glyph_x = pen_x + metrics.xmin;
            let glyph_y = y as i32 + ascent - metrics.height as i32 - metrics.ymin;

            for row in 0..metrics.height {
                for col in 0..metrics.width {
                    let coverage = bitmap[row * metrics.width + col];
                    if coverage == 0 {
                        continue;
                    }
                    let px = glyph_x + col as i32;
                    let py = glyph_y + row as i32;
                    if px < 0 || py < 0 || px as u32 >= canvas_width || py as u32 >= canvas_height
                    {
                        continue;
                    }
                    let idx = (py as u32 * canvas_width + px as u32) as usize;
                    pixels[idx] = blend(pixels[idx], color, coverage);
                }
            }

            pen_x += metrics.advance_width.round() as i32;
        }

        (pen_x - x as i32).max(0) as u32
    }
}

fn blend(dst: u32, color: (u8, u8, u8), coverage: u8) -> u32 {
    let (r, g, b) = color;
    let a = coverage as u32;
    let inv_a = 255 - a;

    let dst_r = (dst >> 16) & 0xff;
    let dst_g = (dst >> 8) & 0xff;
    let dst_b = dst & 0xff;

    let out_r = (r as u32 * a + dst_r * inv_a) / 255;
    let out_g = (g as u32 * a + dst_g * inv_a) / 255;
    let out_b = (b as u32 * a + dst_b * inv_a) / 255;

    0xff000000 | (out_r << 16) | (out_g << 8) | out_b
}
