use glamour::{Point2, Size2};
use palette::Srgba;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Sprite {
    pub top_left: Point2,
    pub size: Size2,
    pub color: Srgba,
    pub texture: String,
}

impl Sprite {
    pub fn new(texture: String, top_left: Point2, size: Size2) -> Self {
        Self {
            top_left,
            size,
            color: Srgba::new(1., 1., 1., 1.),
            texture,
        }
    }

    pub fn with_color(mut self, color: Srgba) -> Self {
        self.color = color;
        self
    }
}
