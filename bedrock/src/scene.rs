use glam::{Vec2, Vec4};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Scene {
    pub layers: Vec<Layer>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            layers: vec![Default::default()],
        }
    }

    pub fn add_layer(&mut self, layer: Layer) {
        self.layers.push(layer);
    }

    pub fn with_layer(mut self, layer: Layer) -> Self {
        self.add_layer(layer);
        self
    }

    pub fn with_clip(mut self, clip: Vec4) -> Self {
        self.layers.last_mut().unwrap().clip = Some(clip);
        self
    }

    pub fn with_blur(mut self, radius: u32) -> Self {
        self.layers.last_mut().unwrap().background_blur_radius = radius;
        self
    }

    pub fn with_background(mut self, color: Vec4) -> Self {
        self.layers.last_mut().unwrap().background_color = Some(color);
        self
    }

    pub fn with_font(mut self, font_name: String) -> Self {
        self.layers.last_mut().unwrap().font_name = font_name;
        self
    }

    pub fn add_quad(&mut self, quad: Quad) {
        self.layers.last_mut().unwrap().add_quad(quad);
    }

    pub fn add_text(&mut self, text: Text) {
        self.layers.last_mut().unwrap().add_text(text);
    }

    pub fn add_path(&mut self, path: Path) {
        self.layers.last_mut().unwrap().add_path(path);
    }
}

#[derive(Deserialize, Debug)]
pub struct Layer {
    #[serde(default)]
    pub clip: Option<Vec4>,
    #[serde(default)]
    pub background_blur_radius: u32,
    #[serde(default)]
    pub background_color: Option<Vec4>,
    #[serde(default = "default_font")]
    pub font_name: String,
    #[serde(default)]
    pub quads: Vec<Quad>,
    #[serde(default)]
    pub texts: Vec<Text>,
    #[serde(default)]
    pub paths: Vec<Path>,
}

impl Default for Layer {
    fn default() -> Self {
        Self {
            clip: None,
            background_blur_radius: 0,
            background_color: Some(Vec4::new(1.0, 1.0, 1.0, 1.0)),
            font_name: "Courier New".to_string(),
            quads: Vec::new(),
            texts: Vec::new(),
            paths: Vec::new(),
        }
    }
}

fn default_font() -> String {
    "Courier New".to_string()
}

impl Layer {
    pub fn with_clip(mut self, clip: Vec4) -> Self {
        self.clip = Some(clip);
        self
    }

    pub fn set_clip(&mut self, clip: Vec4) {
        self.clip = Some(clip);
    }

    pub fn with_blur(mut self, radius: u32) -> Self {
        self.background_blur_radius = radius;
        self
    }

    pub fn set_blur(&mut self, radius: u32) {
        self.background_blur_radius = radius;
    }

    pub fn with_background(mut self, color: Vec4) -> Self {
        self.background_color = Some(color);
        self
    }

    pub fn set_background(&mut self, color: Vec4) {
        self.background_color = Some(color);
    }

    pub fn with_font(mut self, font_name: String) -> Self {
        self.font_name = font_name;
        self
    }

    pub fn set_font(&mut self, font_name: String) {
        self.font_name = font_name;
    }

    pub fn add_quad(&mut self, quad: Quad) {
        self.quads.push(quad);
    }

    pub fn with_quad(mut self, quad: Quad) -> Self {
        self.add_quad(quad);
        self
    }

    pub fn add_text(&mut self, text: Text) {
        self.texts.push(text);
    }

    pub fn with_text(mut self, text: Text) -> Self {
        self.add_text(text);
        self
    }

    pub fn add_path(&mut self, path: Path) {
        self.paths.push(path);
    }

    pub fn with_path(mut self, path: Path) -> Self {
        self.add_path(path);
        self
    }
}

#[derive(Deserialize, Debug)]
pub struct Quad {
    pub top_left: Vec2,
    pub size: Vec2,
    pub color: Vec4,
}

#[derive(Deserialize, Debug)]
pub struct Text {
    pub text: String,
    pub bottom_left: Vec2,
    pub size: f32,
    pub color: Vec4,
    #[serde(default)]
    pub bold: bool,
    #[serde(default)]
    pub italic: bool,
    #[serde(default = "default_subpixel")]
    pub subpixel: bool,
}

fn default_subpixel() -> bool {
    true
}

impl Text {
    pub fn new(text: String, bottom_left: Vec2, size: f32, color: Vec4) -> Self {
        Self {
            text,
            bottom_left,
            size,
            color,
            bold: false,
            italic: false,
            subpixel: true,
        }
    }

    pub fn with_bold(mut self) -> Self {
        self.bold = true;
        self
    }

    pub fn with_italic(mut self) -> Self {
        self.italic = true;
        self
    }

    pub fn without_subpixel(mut self) -> Self {
        self.subpixel = false;
        self
    }
}

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
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

    pub fn with_stroke(mut self, stroke: (f32, Vec4)) -> Self {
        self.stroke = Some(stroke);
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
