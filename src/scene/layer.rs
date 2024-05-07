use glamour::Rect;
use palette::Srgba;
use serde::Deserialize;

use super::Path;
use super::Quad;
use super::Sprite;
use super::Text;

#[derive(Deserialize, Debug, Clone)]
pub struct Layer {
    #[serde(default)]
    pub clip: Option<Rect<u32>>,
    #[serde(default)]
    pub background_blur_radius: f32,
    #[serde(default)]
    pub background_color: Option<Srgba>,
    #[serde(default = "default_font")]
    pub font_name: String,
    #[serde(default)]
    pub quads: Vec<Quad>,
    #[serde(default)]
    pub texts: Vec<Text>,
    #[serde(default)]
    pub paths: Vec<Path>,
    #[serde(default)]
    pub sprites: Vec<Sprite>,
}

impl Default for Layer {
    fn default() -> Self {
        Self {
            clip: None,
            background_blur_radius: 0.0,
            background_color: Some(Srgba::new(1.0, 1.0, 1.0, 1.0)),
            font_name: default_font(),
            quads: Vec::new(),
            texts: Vec::new(),
            paths: Vec::new(),
            sprites: Vec::new(),
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn default_font() -> String {
    "monospace".to_string()
}

#[cfg(target_os = "windows")]
fn default_font() -> String {
    "Courier New".to_string()
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

    pub fn with_font(mut self, font_name: String) -> Self {
        self.font_name = font_name;
        self
    }

    pub fn set_font(&mut self, font_name: String) {
        self.font_name = font_name;
    }

    pub fn add_quad(&mut self, quad: Quad) {
        self.quads.push(quad);
    }

    pub fn with_quad(mut self, quad: Quad) -> Self {
        self.add_quad(quad);
        self
    }

    pub fn add_text(&mut self, text: Text) {
        self.texts.push(text);
    }

    pub fn with_text(mut self, text: Text) -> Self {
        self.add_text(text);
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
}
