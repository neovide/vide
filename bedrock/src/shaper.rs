use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};

use glam::Vec4;
use lazy_static::lazy_static;
use ordered_float::OrderedFloat;
use swash::{shape::ShapeContext, CacheKey, FontRef};
use thread_local::ThreadLocal;

lazy_static! {
    pub static ref SHAPER: Shaper = Shaper::new();
}

pub struct ShapedText {
    pub glyph: Vec<Glyph>,
    pub bounds: Vec4,
}

pub struct Shaper {
    shaping_context: ThreadLocal<Mutex<ShapeContext>>,
    shaped_text_lookup: RwLock<HashMap<ShapeKey, Vec<Glyph>>>,
}

impl Shaper {
    pub fn new() -> Self {
        Self {
            shaping_context: ThreadLocal::new(),
            shaped_text_lookup: RwLock::new(HashMap::new()),
        }
    }

    pub fn shape(text: &str, font: &FontRef) -> ShapedText {
        unimplemented!();
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

pub struct Glyph {
    pub id: u32,
}
