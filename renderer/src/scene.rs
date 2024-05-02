use glam::{Vec2, Vec4};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Scene {
    #[serde(default = "default_window_size")]
    pub window_size: Vec2,
    pub layers: Vec<Layer>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            window_size: Vec2::new(800.0, 600.0),
            layers: vec![Default::default()],
        }
    }

    pub fn push_layer(
        &mut self,
        clip: Option<Vec4>,
        background_blur: bool,
        background_color: Vec4,
        font_name: String,
    ) {
        self.layers.push(Layer {
            clip,
            background_blur,
            background_color,
            font_name,
            ..Default::default()
        });
    }

    pub fn push_quad(&mut self, quad: Quad) {
        self.layers.last_mut().unwrap().quads.push(quad);
    }

    pub fn push_glyph(&mut self, glyph: Glyph) {
        self.layers.last_mut().unwrap().glyphs.push(glyph);
    }

    pub fn push_text(&mut self, text: Text) {
        self.layers.last_mut().unwrap().texts.push(text);
    }
}

#[derive(Deserialize, Debug)]
pub struct Layer {
    #[serde(default)]
    pub clip: Option<Vec4>,
    #[serde(default)]
    pub background_blur: bool,
    #[serde(default)]
    pub background_color: Option<Vec4>,
    #[serde(default = "default_font")]
    pub font_name: String,
    #[serde(default)]
    pub quads: Vec<Quad>,
    #[serde(default)]
    pub texts: Vec<Text>,
}

impl Default for Layer {
    fn default() -> Self {
        Self {
            clip: None,
            background_blur: false,
            background_color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            font_name: "Courier New".to_string(),
            quads: Vec::new(),
            glyphs: Vec::new(),
            texts: Vec::new(),
        }
    }
}

fn default_background_color() -> Vec4 {
    Vec4::ONE
}

fn default_font() -> String {
    "Courier New".to_string()
}

fn default_window_size() -> Vec2 {
    Vec2::new(800.0, 600.0)
}

#[derive(Deserialize, Debug)]
pub struct Quad {
    pub top_left: Vec2,
    pub size: Vec2,
    pub color: Vec4,
}

#[derive(Deserialize, Debug)]
pub struct Text {
    pub text: String,
    pub bottom_left: Vec2,
    pub size: f32,
    pub color: Vec4,
}
