use glam::{Vec2, Vec4};
use serde::Deserialize;
use shader::InstancedQuad;

#[derive(Deserialize, Debug, Clone)]
pub struct Quad {
    top_left: Vec2,
    size: Vec2,
    color: Vec4,
    #[serde(default)]
    corner_radius: f32,
    #[serde(default)]
    blur: f32,
}

impl Quad {
    pub fn new(top_left: Vec2, size: Vec2, color: Vec4) -> Self {
        Self {
            top_left,
            size,
            color,
            corner_radius: 0.0,
            blur: 0.0,
        }
    }

    pub fn with_corner_radius(mut self, corner_radius: f32) -> Self {
        self.corner_radius = corner_radius;
        self
    }

    pub fn with_background_blur(mut self, blur: f32) -> Self {
        self.blur = -blur;
        self
    }

    pub fn with_blur(mut self, blur: f32) -> Self {
        self.blur = blur;
        self
    }

    pub fn to_instanced(&self) -> InstancedQuad {
        InstancedQuad {
            top_left: self.top_left,
            size: self.size,
            color: self.color,
            corner_radius: self.corner_radius,
            blur: self.blur,
            ..Default::default()
        }
    }
}
