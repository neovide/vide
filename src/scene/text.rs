use std::{fmt::Debug, sync::Arc};

use base64::prelude::*;
use glamour::{Point2, Vector2};
use palette::Srgba;
use serde::{Deserialize, Serialize};
use swash::FontRef;

pub struct Font {
    pub data: Arc<Vec<u8>>,
    pub id: u64,
}

impl Font {
    pub fn as_swash_font_ref(&self, index: usize) -> Option<FontRef<'_>> {
        FontRef::from_index(self.data.as_ref(), index)
    }
}

impl<'a> Deserialize<'a> for Font {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        let data = String::deserialize(deserializer)?;
        let data = BASE64_STANDARD.decode(data).unwrap();
        Ok(Self {
            data: data.into(),
            id: 0,
        })
    }
}

impl Serialize for Font {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let data = BASE64_STANDARD.encode(self.data.as_ref());
        data.serialize(serializer)
    }
}

impl Debug for Font {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Font").field("id", &self.id).finish()
    }
}

impl Clone for Font {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            id: self.id,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GlyphRun {
    // TODO: Store synthesis paramaters from swash here
    pub position: Point2,
    pub font_id: u64,
    pub font_index: usize,
    pub color: Srgba,
    pub size: f32,
    pub normalized_coords: Vec<i16>,

    pub glyphs: Vec<Glyph>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Glyph {
    pub id: u16,
    pub offset: Vector2,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FontFeature {
    tag: String,
    value: f32,
}

impl FontFeature {
    pub fn parse(feature: &str) -> Result<Self, String> {
        if let Some(name) = feature.strip_prefix('+') {
            Ok(FontFeature {
                tag: name.trim().to_string(),
                value: 1.,
            })
        } else if let Some(name) = feature.strip_prefix('-') {
            Ok(FontFeature {
                tag: name.trim().to_string(),
                value: 0.,
            })
        } else if let Some((name, value)) = feature.split_once('=') {
            let value = value.parse();
            if let Ok(value) = value {
                Ok(FontFeature {
                    tag: name.to_string(),
                    value,
                })
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
        if self.value == 0. {
            serializer.serialize_str(&format!("-{}", self.tag))
        } else if self.value == 1. {
            serializer.serialize_str(&format!("+{}", self.tag))
        } else {
            serializer.serialize_str(&format!("{}={}", self.tag, self.value))
        }
    }
}

impl Into<FontVariation> for &FontFeature {
    fn into(self) -> Setting<f32> {
        (self.tag.as_ref(), self.value).into()
    }
}
