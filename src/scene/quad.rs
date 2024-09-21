use crate::default_drawables::InstancedQuad;
use glam::Vec4;
use glamour::{AsRaw, Point2, Rect, Size2};
use palette::Srgba;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Quad {
    pub region: Rect,
    pub color: Srgba,
    #[serde(default)]
    pub corner_radius: f32,
    #[serde(default)]
    pub edge_blur: f32,
}

impl Quad {
    pub fn new(region: Rect, color: Srgba) -> Self {
        Self {
            region,
            color,
            corner_radius: 0.0,
            edge_blur: 0.0,
        }
    }

    pub fn with_corner_radius(mut self, corner_radius: f32) -> Self {
        self.corner_radius = corner_radius;
        self
    }

    pub fn with_edge_blur(mut self, blur: f32) -> Self {
        self.edge_blur = blur;
        self
    }

    pub fn to_instanced(&self) -> InstancedQuad {
        InstancedQuad {
            top_left: *self.region.origin.as_raw(),
            size: *self.region.size.as_raw(),
            color: Vec4::from_array(self.color.into_linear().into()),
            corner_radius: self.corner_radius,
            edge_blur: self.edge_blur,
            ..Default::default()
        }
    }
}
