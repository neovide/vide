mod layer;
mod path;
mod quad;
mod sprite;
mod text;

use glamour::Rect;
use palette::Srgba;
use serde::Deserialize;

pub use layer::*;
pub use path::*;
pub use quad::*;
pub use sprite::*;
pub use text::*;

#[derive(Deserialize, Debug, Clone)]
pub struct Scene {
    pub layers: Vec<Layer>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
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

    pub fn with_font(mut self, font_name: String) -> Self {
        self.layer_mut().font_name = font_name;
        self
    }

    pub fn font(&self) -> &str {
        self.layer().font_name.as_str()
    }

    pub fn add_quad(&mut self, quad: Quad) {
        self.layer_mut().add_quad(quad);
    }

    pub fn with_quad(mut self, quad: Quad) -> Self {
        self.add_quad(quad);
        self
    }

    pub fn add_text(&mut self, text: Text) {
        self.layer_mut().add_text(text);
    }

    pub fn with_text(mut self, text: Text) -> Self {
        self.add_text(text);
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
