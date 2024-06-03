use glamour::Rect;
use palette::Srgba;
use serde::de::Error;
use serde::Deserialize;
use serde::Serialize;
use serde::Serializer;
use swash::Setting;

use super::Path;
use super::Quad;
use super::Sprite;
use super::Text;

#[derive(Deserialize, Debug, Clone)]
pub struct Layer {
    #[serde(default)]
    pub clip: Option<Rect<u32>>,
    #[serde(default)]
    pub background_blur_radius: f32,
    #[serde(default)]
    pub background_color: Option<Srgba>,
    #[serde(default = "default_font")]
    pub font_name: String,
    #[serde(default)]
    pub font_features: Vec<FontFeature>,
    #[serde(default)]
    pub quads: Vec<Quad>,
    #[serde(default)]
    pub texts: Vec<Text>,
    #[serde(default)]
    pub paths: Vec<Path>,
    #[serde(default)]
    pub sprites: Vec<Sprite>,
}

impl Default for Layer {
    fn default() -> Self {
        Self {
            clip: None,
            background_blur_radius: 0.0,
            background_color: Some(Srgba::new(1.0, 1.0, 1.0, 1.0)),
            font_name: default_font(),
            font_features: Vec::new(),
            quads: Vec::new(),
            texts: Vec::new(),
            paths: Vec::new(),
            sprites: Vec::new(),
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn default_font() -> String {
    "monospace".to_string()
}

#[cfg(target_os = "windows")]
fn default_font() -> String {
    "Courier New".to_string()
}

impl Layer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_clip(mut self, clip: Rect<u32>) -> Self {
        self.clip = Some(clip);
        self
    }

    pub fn set_clip(&mut self, clip: Rect<u32>) {
        self.clip = Some(clip);
    }

    pub fn with_blur(mut self, radius: f32) -> Self {
        self.background_blur_radius = radius;
        self
    }

    pub fn set_blur(&mut self, radius: f32) {
        self.background_blur_radius = radius;
    }

    pub fn with_background(mut self, color: Srgba) -> Self {
        self.background_color = Some(color);
        self
    }

    pub fn set_background(&mut self, color: Srgba) {
        self.background_color = Some(color);
    }

    pub fn with_font(mut self, font_name: String) -> Self {
        self.font_name = font_name;
        self
    }

    pub fn set_font(&mut self, font_name: String) {
        self.font_name = font_name;
    }

    pub fn set_font_features(&mut self, font_features: Vec<FontFeature>) {
        self.font_features = font_features;
    }

    pub fn set_parsed_font_features(&mut self, font_features: &[&str]) {
        self.font_features = font_features
            .iter()
            .map(|feature| FontFeature::parse(feature).unwrap())
            .collect();
    }

    pub fn with_font_features(mut self, font_features: Vec<FontFeature>) -> Self {
        self.set_font_features(font_features);
        self
    }

    pub fn with_parsed_font_features(mut self, font_features: &[&str]) -> Self {
        self.set_parsed_font_features(font_features);
        self
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

    pub fn add_sprite(&mut self, sprite: Sprite) {
        self.sprites.push(sprite);
    }

    pub fn with_sprite(mut self, sprite: Sprite) -> Self {
        self.add_sprite(sprite);
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FontFeature(pub String, pub u16);

impl FontFeature {
    pub fn parse(feature: &str) -> Result<Self, String> {
        if let Some(name) = feature.strip_prefix('+') {
            Ok(FontFeature(name.trim().to_string(), 1u16))
        } else if let Some(name) = feature.strip_prefix('-') {
            Ok(FontFeature(name.trim().to_string(), 0u16))
        } else if let Some((name, value)) = feature.split_once('=') {
            let value = value.parse();
            if let Ok(value) = value {
                Ok(FontFeature(name.to_string(), value))
            } else {
                Err("Value assigned to font feature is not an integer".to_string())
            }
        } else {
            Err("Font feature is not prefixed with +, - or contains =".to_string())
        }
    }
}

impl<'de> Deserialize<'de> for FontFeature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let text = String::deserialize(deserializer)?;
        FontFeature::parse(&text).map_err(D::Error::custom)
    }
}

impl Serialize for FontFeature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.1 {
            0 => serializer.serialize_str(&format!("-{}", self.0)),
            1 => serializer.serialize_str(&format!("+{}", self.0)),
            value => serializer.serialize_str(&format!("{}={}", self.0, value)),
        }
    }
}

impl Into<Setting<u16>> for &FontFeature {
    fn into(self) -> Setting<u16> {
        (self.0.as_ref(), self.1).into()
    }
}
