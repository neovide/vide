use glam::{vec2, Vec2, Vec4};
use swash::{shape::ShapeContext, FontRef};
use wgpu::Queue;

use crate::glyph::GlyphState;

pub struct TextShapingState {
    context: ShapeContext,
}

impl TextShapingState {
    pub fn new() -> Self {
        Self {
            context: ShapeContext::new(),
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
        let mut shaper = self.context.builder(font_ref).size(size).build();

        shaper.add_str(text);

        let mut current_x = 0.;
        shaper.shape_with(|cluster| {
            for glyph in cluster.glyphs {
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
        });
    }
}
