use glamour::{vec2, Point2, Rect};
use palette::Srgba;
use parley::Layout;
use serde::{Deserialize, Serialize};

use super::{Glyph, GlyphRun, Path, Quad, Resources, Sprite, TextureId};

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Layer {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clip: Option<Rect<u32>>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mask: Option<LayerContents>,
    #[serde(default)]
    #[serde(flatten)]
    pub contents: LayerContents,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LayerContents {
    #[serde(default)]
    #[serde(skip_serializing_if = "is_zero")]
    pub background_blur_radius: f32,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<Srgba>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub quads: Vec<Quad>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub glyph_runs: Vec<GlyphRun>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub paths: Vec<Path>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub sprites: Vec<Sprite<TextureId>>,
}

impl Default for LayerContents {
    fn default() -> Self {
        Self {
            background_blur_radius: 0.0,
            background_color: None,
            quads: Vec::new(),
            glyph_runs: Vec::new(),
            paths: Vec::new(),
            sprites: Vec::new(),
        }
    }
}

fn is_zero(value: &f32) -> bool {
    *value == 0.0
}

impl Layer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_clip(mut self, clip: Rect<u32>) -> Self {
        self.set_clip(clip);
        self
    }

    pub fn set_clip(&mut self, clip: Rect<u32>) {
        self.clip = Some(clip);
    }

    pub fn with_mask(mut self, mask_layer: Layer) -> Self {
        self.set_mask(mask_layer);
        self
    }

    pub fn set_mask(&mut self, mask_layer: Layer) {
        self.mask = Some(mask_layer.contents);
    }

    pub fn with_blur(mut self, radius: f32) -> Self {
        self.set_blur(radius);
        self
    }

    pub fn set_blur(&mut self, radius: f32) {
        self.contents.background_blur_radius = radius;
    }

    pub fn with_background(mut self, color: Srgba) -> Self {
        self.set_background(color);
        self
    }

    pub fn set_background(&mut self, color: Srgba) {
        self.contents.background_color = Some(color);
    }

    pub fn add_quad(&mut self, quad: Quad) {
        self.contents.quads.push(quad);
    }

    pub fn with_quad(mut self, quad: Quad) -> Self {
        self.add_quad(quad);
        self
    }

    pub fn add_glyph_run(&mut self, glyph_run: GlyphRun) {
        self.contents.glyph_runs.push(glyph_run);
    }

    pub fn with_glyph_run(mut self, glyph_run: GlyphRun) -> Self {
        self.add_glyph_run(glyph_run);
        self
    }

    pub fn add_path(&mut self, path: Path) {
        self.contents.paths.push(path);
    }

    pub fn with_path(mut self, path: Path) -> Self {
        self.add_path(path);
        self
    }

    pub fn add_sprite(&mut self, sprite: Sprite<TextureId>) {
        self.contents.sprites.push(sprite);
    }

    pub fn with_sprite(mut self, sprite: Sprite<TextureId>) -> Self {
        self.add_sprite(sprite);
        self
    }

    pub fn add_text_layout(
        &mut self,
        resources: &mut Resources,
        layout: Layout<Srgba>,
        position: Point2,
    ) {
        for line in layout.lines() {
            for glyph_run in line.glyph_runs() {
                let run = glyph_run.run();
                let font = run.font();
                let font_id = resources.store_font(font);
                let style = glyph_run.style();
                let color = style.brush;
                let synthesis = glyph_run.run().synthesis().into();

                let font_index = font.index as usize;
                let size = run.font_size();
                let normalized_coords = run.normalized_coords().to_vec();
                let mut glyphs = Vec::new();
                let mut current_x = 0.0;
                for glyph in glyph_run.glyphs() {
                    glyphs.push(Glyph {
                        id: glyph.id,
                        offset: vec2!(current_x + glyph.x, -glyph.y),
                    });
                    current_x += glyph.advance;
                }

                self.add_glyph_run(GlyphRun {
                    position: position + vec2!(glyph_run.offset(), glyph_run.baseline()),
                    font_id,
                    font_index,
                    color,
                    size,
                    normalized_coords,
                    glyphs,
                    synthesis,
                });
            }
        }
    }

    pub fn with_text_layout(
        mut self,
        resources: &mut Resources,
        layout: Layout<Srgba>,
        position: Point2,
    ) -> Self {
        self.add_text_layout(resources, layout, position);
        self
    }
}
