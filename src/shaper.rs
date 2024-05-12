use std::{collections::HashMap, sync::Arc};

use glamour::{Rect, Size2, Vector2};
use lazy_static::lazy_static;
use ordered_float::OrderedFloat;
use swash::{
    shape::{cluster::Glyph, ShapeContext},
    CacheKey, FontRef,
};

use crate::font::Font;

lazy_static! {
    pub static ref SHAPER: Shaper = Shaper::new();
}

#[derive(Clone)]
pub struct GlyphRun {
    font_name: String,
    size: f32,
    glyphs: Vec<Glyph>,
    bounds: Rect,
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

pub struct Shaper {
    shaping_context: ShapeContext,
    font_lookup: HashMap<String, Arc<Font>>,
    shaped_text_lookup: HashMap<ShapeKey, Arc<ShapedTextRun>>,
}

impl Shaper {
    pub fn new() -> Self {
        Self {
            shaping_context: ShapeContext::new(),
            font_lookup: HashMap::new(),
            shaped_text_lookup: HashMap::new(),
        }
    }

    pub fn shape(&mut self, text: &str, font_name: &str, size: f32) -> Arc<ShapedTextRun> {
        let font = self
            .font_lookup
            .entry(font_name.to_string())
            .or_insert_with(|| Arc::new(Font::from_name(font_name).unwrap()));
        let font_ref = font.as_font_ref().unwrap();

        let key = ShapeKey::new(Arc::from(text), font_ref, size);

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

                    let mut shaped_text = ShapedTextRun {
                        font_name: font_name.to_string(),
                        size,
                        glyphs: Vec::new(),
                        bounds: Rect::new(Vector2::ZERO, Size2::ZERO),
                    };

                    shaper.shape_with(|cluster| {
                        for glyph in cluster.glyphs {
                            shaped_text.glyphs.push(*glyph);
                        }
                    });

                    Arc::new(shaped_text)
                }
            })
            .clone()
    }
}
