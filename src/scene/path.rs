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
    #[serde(default)]
    pub open: bool,
}

impl Path {
    pub fn new_fill(fill: Srgba, start: Point2) -> Self {
        Self {
            fill: Some(fill),
            stroke: None,
            start,
            commands: Vec::new(),
            open: false,
        }
    }

    pub fn new_stroke(width: f32, color: Srgba, start: Point2) -> Self {
        Self {
            fill: None,
            stroke: Some((width, color)),
            start,
            commands: Vec::new(),
            open: false,
        }
    }

    pub fn new_open_stroke(width: f32, color: Srgba, start: Point2) -> Self {
        Self {
            fill: None,
            stroke: Some((width, color)),
            start,
            commands: Vec::new(),
            open: true,
        }
    }

    pub fn new(start: Point2) -> Self {
        Self {
            fill: None,
            stroke: None,
            start,
            commands: Vec::new(),
            open: false,
        }
    }

    pub fn set_fill(&mut self, fill: Srgba) {
        self.fill = Some(fill);
        assert!(!self.open);
    }

    pub fn with_fill(mut self, fill: Srgba) -> Self {
        self.set_fill(fill);
        self
    }

    pub fn set_stroke(&mut self, width: f32, color: Srgba) {
        self.stroke = Some((width, color));
    }

    pub fn with_stroke(mut self, width: f32, color: Srgba) -> Self {
        self.set_stroke(width, color);
        self
    }

    pub fn add_cubic_bezier_to(&mut self, control1: Point2, control2: Point2, to: Point2) {
        self.commands.push(PathCommand::CubicBezierTo {
            control1,
            control2,
            to,
        });
    }

    pub fn with_cubic_bezier_to(mut self, control1: Point2, control2: Point2, to: Point2) -> Self {
        self.add_cubic_bezier_to(control1, control2, to);
        self
    }

    pub fn add_quadratic_bezier_to(&mut self, control: Point2, to: Point2) {
        self.commands
            .push(PathCommand::QuadraticBezierTo { control, to });
    }

    pub fn with_quadratic_bezier_to(mut self, control: Point2, to: Point2) -> Self {
        self.add_quadratic_bezier_to(control, to);
        self
    }

    pub fn add_line_to(&mut self, to: Point2) {
        self.commands.push(PathCommand::LineTo { to });
    }

    pub fn with_line_to(mut self, to: Point2) -> Self {
        self.add_line_to(to);
        self
    }
}
