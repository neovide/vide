mod layer;
mod path;
mod quad;
mod sprite;
mod text;

use std::{collections::HashMap, sync::Arc};

use glamour::{vec2, Point2, Rect};
use palette::Srgba;
use parley::layout::Layout;
use serde::{Deserialize, Serialize};

pub use layer::*;
pub use path::*;
pub use quad::*;
pub use sprite::*;
pub use text::*;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Scene {
    pub resources: Resources,
    pub layers: Vec<Layer>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            resources: Default::default(),
            layers: vec![Default::default()],
        }
    }

    pub fn add_layer(&mut self, layer: Layer) {
        self.layers.push(layer);
    }

    pub fn with_layer(mut self, layer: Layer) -> Self {
        self.add_layer(layer);
        self
    }

    pub fn layer(&self) -> &Layer {
        self.layers.last().unwrap()
    }

    pub fn layer_mut(&mut self) -> &mut Layer {
        self.layers.last_mut().unwrap()
    }

    pub fn with_clip(mut self, clip: Rect<u32>) -> Self {
        self.layer_mut().clip = Some(clip);
        self
    }

    pub fn with_blur(mut self, radius: f32) -> Self {
        self.layer_mut().background_blur_radius = radius;
        self
    }

    pub fn with_background(mut self, color: Srgba) -> Self {
        self.layer_mut().background_color = Some(color);
        self
    }

    pub fn with_font_features(mut self, font_features: Vec<FontFeature>) -> Self {
        self.layer_mut().font_features = font_features;
        self
    }

    pub fn with_parsed_font_features(self, font_features: Vec<&str>) -> Self {
        self.with_font_features(
            font_features
                .iter()
                .map(|feature| FontFeature::parse(feature).unwrap())
                .collect(),
        )
    }

    pub fn font_features(&self) -> &[FontFeature] {
        &self.layer().font_features
    }

    pub fn add_quad(&mut self, quad: Quad) {
        self.layer_mut().add_quad(quad);
    }

    pub fn with_quad(mut self, quad: Quad) -> Self {
        self.add_quad(quad);
        self
    }

    pub fn add_text_layout(&mut self, layout: Layout<Srgba>, position: Point2) {
        for line in layout.lines() {
            for glyph_run in line.glyph_runs() {
                let run = glyph_run.run();
                let font = run.font();
                let font_id = font.data.id();
                if !self.resources.fonts.contains_key(&font_id) {
                    self.resources.fonts.insert(
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

                self.layer_mut().add_glyph_run(GlyphRun {
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

    pub fn with_text_layout(mut self, layout: Layout<Srgba>, position: Point2) -> Self {
        self.add_text_layout(layout, position);
        self
    }

    pub fn add_path(&mut self, path: Path) {
        self.layer_mut().add_path(path);
    }

    pub fn with_path(mut self, path: Path) -> Self {
        self.add_path(path);
        self
    }

    pub fn add_sprite(&mut self, sprite: Sprite) {
        self.layer_mut().add_sprite(sprite);
    }

    pub fn with_sprite(mut self, sprite: Sprite) -> Self {
        self.add_sprite(sprite);
        self
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Resources {
    pub fonts: HashMap<u64, Font>,
}
