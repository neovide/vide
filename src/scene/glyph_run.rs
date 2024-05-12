#[derive(Deserialize, Debug, Clone)]
pub struct GlyphRun {
    pub glyphs: Vec<Glyph>,
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

pub struct Glyph {
    pub glyph: GlyphId,
    pub offset: Vector2,
}

#[cfg(not(target_os = "windows"))]
fn default_font() -> String {
    "monospace".to_string()
}

#[cfg(target_os = "windows")]
fn default_font() -> String {
    "Courier New".to_string()
}
