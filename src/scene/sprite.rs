use glam::{Vec2, Vec4};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Sprite {
    pub top_left: Vec2,
    pub size: Vec2,
    pub color: Vec4,
    pub texture: String,
}

impl Sprite {
    pub fn new(texture: String, top_left: Vec2, size: Vec2) -> Self {
        Self {
            top_left,
            size,
            color: Vec4::ONE,
            texture,
        }
    }

    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = color;
        self
    }
}
