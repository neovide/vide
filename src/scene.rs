use glam::{Vec2, Vec4};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Scene {
    pub background_color: Vec4,
    pub font_name: String,
    pub quads: Vec<Quad>,
    pub glyphs: Vec<Glyph>,
    pub texts: Vec<Text>,
}

#[derive(Deserialize)]
pub struct Quad {
    pub top_left: Vec2,
    pub size: Vec2,
    pub color: Vec4,
}

pub struct Glyph {
    pub character: char,
    pub bottom_left: Vec2,
    pub size: Vec2,
    pub color: Vec4,
}

pub struct Text {
    pub text: String,
    pub bottom_left: Vec2,
    pub size: Vec2,
    pub color: Vec4,
}
