use glamour::Point2;
use palette::Srgba;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PathCommand {
    CubicBezierTo {
        control1: Point2,
        control2: Point2,
        to: Point2,
    },
    QuadraticBezierTo {
        control: Point2,
        to: Point2,
    },
    LineTo {
        to: Point2,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Path {
    #[serde(default)]
    pub fill: Option<Srgba>,
    #[serde(default)]
    pub stroke: Option<(f32, Srgba)>,
    pub start: Point2,
    pub commands: Vec<PathCommand>,
}

impl Path {
    pub fn new_fill(fill: Srgba, start: Point2) -> Self {
        Self {
            fill: Some(fill),
            stroke: None,
            start,
            commands: Vec::new(),
        }
    }

    pub fn new_stroke(stroke: (f32, Srgba), start: Point2) -> Self {
        Self {
            fill: None,
            stroke: Some(stroke),
            start,
            commands: Vec::new(),
        }
    }

    pub fn new(start: Point2) -> Self {
        Self {
            fill: None,
            stroke: None,
            start,
            commands: Vec::new(),
        }
    }

    pub fn with_fill(mut self, fill: Srgba) -> Self {
        self.fill = Some(fill);
        self
    }

    pub fn with_stroke(mut self, width: f32, color: Srgba) -> Self {
        self.stroke = Some((width, color));
        self
    }

    pub fn cubic_bezier_to(mut self, control1: Point2, control2: Point2, to: Point2) -> Self {
        self.commands.push(PathCommand::CubicBezierTo {
            control1,
            control2,
            to,
        });
        self
    }

    pub fn quadratic_bezier_to(mut self, control: Point2, to: Point2) -> Self {
        self.commands
            .push(PathCommand::QuadraticBezierTo { control, to });
        self
    }

    pub fn line_to(mut self, to: Point2) -> Self {
        self.commands.push(PathCommand::LineTo { to });
        self
    }
}
