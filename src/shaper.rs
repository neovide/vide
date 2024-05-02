mod font_spec;

use std::{collections::HashMap, sync::Arc};

use glam::Vec4;
use lazy_static::lazy_static;
use ordered_float::OrderedFloat;
use swash::{
    shape::{cluster::Glyph, ShapeContext},
    CacheKey, FontRef,
};

use crate::{font::Font, Scene};

use self::font_spec::IntoFontSpec;

lazy_static! {
    pub static ref SHAPER: Shaper = Shaper::new();
}

#[derive(Clone)]
pub struct ShapedText {
    shape_key: ShapeKey,
    pub glyphs: Vec<Glyph>,
    pub bounds: Vec4,
}

pub struct Shaper {
    shaping_context: ShapeContext,
    shaped_text_lookup: HashMap<ShapeKey, ShapedText>,
}

impl Shaper {
    pub fn new() -> Self {
        Self {
            shaping_context: ShapeContext::new(),
            shaped_text_lookup: HashMap::new(),
        }
    }

    pub fn shape(&mut self, text: &str, font: &str, size: f32) -> ShapedText {
        let font = Font::from_name(font).unwrap();
        let font_ref = font.as_ref().unwrap();

        let key = ShapeKey::new(Arc::from(text), font_ref, size.into());

        self.shaped_text_lookup
            .entry(key.clone())
            .or_insert_with({
                let mut shaper = self
                    .shaping_context
                    .builder(font_ref)
                    .size(*key.size)
                    .build();

                move || {
                    shaper.add_str(key.text.as_ref());

                    let mut shaped_text = ShapedText {
                        shape_key: key.clone(),
                        glyphs: Vec::new(),
                        bounds: Vec4::ZERO,
                    };

                    shaper.shape_with(|cluster| {
                        for glyph in cluster.glyphs {
                            shaped_text.glyphs.push(*glyph);
                        }
                    });

                    shaped_text
                }
            })
            .clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ShapeKey {
    text: Arc<str>,
    size: OrderedFloat<f32>,
    font_cache_key: CacheKey,
}

impl ShapeKey {
    fn new(text: Arc<str>, font_ref: FontRef, size: f32) -> Self {
        let font_cache_key = font_ref.key;
        let size = size.into();
        Self {
            text,
            size,
            font_cache_key,
        }
    }
}
