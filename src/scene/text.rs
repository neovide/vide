use glamour::Point2;
use palette::Srgba;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Text {
    pub text: String,
    pub bottom_left: Point2,
    #[serde(default = "default_font")]
    pub font_name: String,
    pub size: f32,
    pub color: Srgba,
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

#[cfg(not(target_os = "windows"))]
fn default_font() -> String {
    "monospace".to_string()
}

#[cfg(target_os = "windows")]
fn default_font() -> String {
    "Courier New".to_string()
}

impl Text {
    pub fn new(text: String, bottom_left: Point2, size: f32, color: Srgba) -> Self {
        Self {
            text,
            bottom_left,
            font_name: default_font(),
            size,
            color,
            bold: false,
            italic: false,
            subpixel: true,
        }
    }

    pub fn with_font(mut self, font_name: String) -> Self {
        self.font_name = font_name;
        self
    }

    pub fn set_font(&mut self, font_name: String) {
        self.font_name = font_name;
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
