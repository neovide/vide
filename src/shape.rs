use std::{collections::HashMap, sync::Arc};

use glam::{vec2, Vec2, Vec4};
use ordered_float::OrderedFloat;
use swash::{
    shape::{cluster::Glyph, ShapeContext},
    CacheKey, FontRef,
};
use wgpu::Queue;

use crate::glyph::GlyphState;

pub struct TextShapingState {
    context: ShapeContext,
    cache: HashMap<(Arc<str>, OrderedFloat<f32>, CacheKey), Vec<Glyph>>,
}

impl TextShapingState {
    pub fn new() -> Self {
        Self {
            context: ShapeContext::new(),
            cache: HashMap::new(),
        }
    }

    pub fn add_text<'a, 'b: 'a>(
        &'b mut self,
        queue: &mut Queue,
        glyph_state: &'b mut GlyphState,
        font_ref: FontRef<'a>,
        text: &str,
        bottom_left: Vec2,
        size: f32,
        color: Vec4,
    ) {
        let key = (Arc::from(text), OrderedFloat(size), font_ref.key);
        let mut shaper = self.context.builder(font_ref).size(size).build();
        let glyphs = self.cache.entry(key).or_insert_with(|| {
            shaper.add_str(text);

            let mut glyphs = Vec::new();

            shaper.shape_with(|cluster| {
                for glyph in cluster.glyphs {
                    glyphs.push(*glyph);
                }
            });

            glyphs
        });

        let mut current_x = 0.;

        for glyph in glyphs {
            glyph_state.add_glyph(
                queue,
                font_ref,
                glyph.id,
                bottom_left + vec2(current_x + glyph.x, -glyph.y),
                size,
                color,
            );
            current_x += glyph.advance + glyph.x;
        }
    }
}
