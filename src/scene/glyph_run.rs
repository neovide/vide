use std::{fmt::Debug, sync::Arc};

use base64::prelude::*;
use glamour::{Point2, Vector2};
use palette::Srgba;
use parley::Font as ParleyFont;
use serde::{Deserialize, Serialize};
use swash::FontRef;

use crate::Resources;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GlyphRun {
    // TODO: Store synthesis paramaters from swash here
    pub position: Point2,
    pub font_id: FontId,
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

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq)]
pub struct FontId(u64);

#[cfg(test)]
impl FontId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

impl Resources {
    pub fn store_font(&mut self, font: &ParleyFont) -> FontId {
        let id = FontId(font.data.id());
        self.fonts.entry(id).or_insert_with(|| Font {
            data: Arc::from(font.data.data().to_vec()),
        });

        id
    }
}

#[derive(Clone)]
pub struct Font {
    pub data: Arc<Vec<u8>>,
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
        Ok(Self { data: data.into() })
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
        f.debug_struct("Font").finish()
    }
}
