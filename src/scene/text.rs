use glam::{Vec2, Vec4};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
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
