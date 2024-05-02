use glam::{Vec2, Vec4};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Scene {
    #[serde(default = "default_background_color")]
    pub background_color: Vec4,
    #[serde(default = "default_font")]
    pub font_name: String,
    #[serde(default = "default_window_size")]
    pub window_size: Vec2,
    #[serde(default)]
    pub quads: Vec<Quad>,
    #[serde(default)]
    pub glyphs: Vec<Glyph>,
    #[serde(default)]
    pub texts: Vec<Text>,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            background_color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            font_name: "Courier New".to_string(),
            window_size: Vec2::new(800.0, 600.0),
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
pub struct Glyph {
    pub character: char,
    pub bottom_left: Vec2,
    pub size: f32,
    pub color: Vec4,
}

#[derive(Deserialize, Debug)]
pub struct Text {
    pub text: String,
    pub bottom_left: Vec2,
    pub size: f32,
    pub color: Vec4,
}
