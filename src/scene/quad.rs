use crate::quad::InstancedQuad;
use glam::Vec4;
use glamour::{AsRaw, Point2, Size2};
use palette::Srgba;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Quad {
    top_left: Point2,
    size: Size2,
    color: Srgba,
    #[serde(default)]
    corner_radius: f32,
    #[serde(default)]
    blur: f32,
}

impl Quad {
    pub fn new(top_left: Point2, size: Size2, color: Srgba) -> Self {
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
            top_left: *self.top_left.as_raw(),
            size: *self.size.as_raw(),
            color: Vec4::from_array(self.color.into_linear().into()),
            corner_radius: self.corner_radius,
            blur: self.blur,
            ..Default::default()
        }
    }
}
