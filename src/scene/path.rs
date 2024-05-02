use glam::{Vec2, Vec4};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum PathCommand {
    CubicBezierTo {
        control1: Vec2,
        control2: Vec2,
        to: Vec2,
    },
    QuadraticBezierTo {
        control: Vec2,
        to: Vec2,
    },
    LineTo {
        to: Vec2,
    },
}

#[derive(Deserialize, Debug, Clone)]
pub struct Path {
    #[serde(default)]
    pub fill: Option<Vec4>,
    #[serde(default)]
    pub stroke: Option<(f32, Vec4)>,
    pub start: Vec2,
    pub commands: Vec<PathCommand>,
}

impl Path {
    pub fn new_fill(fill: Vec4, start: Vec2) -> Self {
        Self {
            fill: Some(fill),
            stroke: None,
            start,
            commands: Vec::new(),
        }
    }

    pub fn new_stroke(stroke: (f32, Vec4), start: Vec2) -> Self {
        Self {
            fill: None,
            stroke: Some(stroke),
            start,
            commands: Vec::new(),
        }
    }

    pub fn new(start: Vec2) -> Self {
        Self {
            fill: None,
            stroke: None,
            start,
            commands: Vec::new(),
        }
    }

    pub fn with_fill(mut self, fill: Vec4) -> Self {
        self.fill = Some(fill);
        self
    }

    pub fn with_stroke(mut self, width: f32, color: Vec4) -> Self {
        self.stroke = Some((width, color));
        self
    }

    pub fn cubic_bezier_to(mut self, control1: Vec2, control2: Vec2, to: Vec2) -> Self {
        self.commands.push(PathCommand::CubicBezierTo {
            control1,
            control2,
            to,
        });
        self
    }

    pub fn quadratic_bezier_to(mut self, control: Vec2, to: Vec2) -> Self {
        self.commands
            .push(PathCommand::QuadraticBezierTo { control, to });
        self
    }

    pub fn line_to(mut self, to: Vec2) -> Self {
        self.commands.push(PathCommand::LineTo { to });
        self
    }
}
