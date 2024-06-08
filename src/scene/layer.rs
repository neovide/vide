use std::sync::Arc;

use glamour::{vec2, Point2, Rect};
use palette::Srgba;
use parley::Layout;
use serde::{Deserialize, Serialize};

use super::{Font, Glyph, GlyphRun, Path, Quad, Resources, Sprite};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Layer {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clip: Option<Rect<u32>>,
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
    pub sprites: Vec<Sprite>,
}

fn is_zero(value: &f32) -> bool {
    *value == 0.0
}

impl Default for Layer {
    fn default() -> Self {
        Self {
            clip: None,
            background_blur_radius: 0.0,
            background_color: Some(Srgba::new(1.0, 1.0, 1.0, 1.0)),
            quads: Vec::new(),
            glyph_runs: Vec::new(),
            paths: Vec::new(),
            sprites: Vec::new(),
        }
    }
}

impl Layer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_clip(mut self, clip: Rect<u32>) -> Self {
        self.clip = Some(clip);
        self
    }

    pub fn set_clip(&mut self, clip: Rect<u32>) {
        self.clip = Some(clip);
    }

    pub fn with_blur(mut self, radius: f32) -> Self {
        self.background_blur_radius = radius;
        self
    }

    pub fn set_blur(&mut self, radius: f32) {
        self.background_blur_radius = radius;
    }

    pub fn with_background(mut self, color: Srgba) -> Self {
        self.background_color = Some(color);
        self
    }

    pub fn set_background(&mut self, color: Srgba) {
        self.background_color = Some(color);
    }

    pub fn add_quad(&mut self, quad: Quad) {
        self.quads.push(quad);
    }

    pub fn with_quad(mut self, quad: Quad) -> Self {
        self.add_quad(quad);
        self
    }

    pub fn add_glyph_run(&mut self, glyph_run: GlyphRun) {
        self.glyph_runs.push(glyph_run);
    }

    pub fn with_glyph_run(mut self, glyph_run: GlyphRun) -> Self {
        self.add_glyph_run(glyph_run);
        self
    }

    pub fn add_path(&mut self, path: Path) {
        self.paths.push(path);
    }

    pub fn with_path(mut self, path: Path) -> Self {
        self.add_path(path);
        self
    }

    pub fn add_sprite(&mut self, sprite: Sprite) {
        self.sprites.push(sprite);
    }

    pub fn with_sprite(mut self, sprite: Sprite) -> Self {
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
                let font_id = font.data.id();
                if !resources.fonts.contains_key(&font_id) {
                    resources.fonts.insert(
                        font_id,
                        Font {
                            data: Arc::from(font.data.data().to_vec()),
                            id: font_id,
                        },
                    );
                }
                let style = glyph_run.style();
                let color = style.brush.into();

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
