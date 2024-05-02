mod font;
mod glyph;
mod quad;
mod renderer;
mod scene;
mod shape;

use font::Font;
use glam::{vec2, Vec2, Vec4};
use rust_embed::*;

pub use renderer::Renderer;
pub use scene::Scene;
use shader::InstancedQuad;
use swash::FontRef;
use winit::{dpi::PhysicalSize, window::Window};

pub const ATLAS_SIZE: Vec2 = vec2(1024., 1024.);

#[derive(RustEmbed)]
#[folder = "spirv"]
struct Asset;

impl Renderer {
    pub fn clear(&mut self, color: Vec4) {
        self.clear_color = color;

        self.quad_state.clear();
        self.glyph_state.clear();
    }

    pub fn add_quad(&mut self, top_left: Vec2, size: Vec2, color: Vec4) {
        self.quad_state.quads.push(InstancedQuad {
            top_left,
            size,
            color,
        });
    }

    pub fn add_glyph<'a, 'b: 'a>(
        &'b mut self,
        font_ref: FontRef<'a>,
        glyph: swash::GlyphId,
        bottom_left: Vec2,
        size: f32,
        color: Vec4,
    ) {
        self.glyph_state
            .add_glyph(&mut self.queue, font_ref, glyph, bottom_left, size, color);
    }

    pub fn add_text<'a, 'b: 'a>(
        &'b mut self,
        font_ref: FontRef<'a>,
        text: &str,
        bottom_left: Vec2,
        size: f32,
        color: Vec4,
    ) {
        self.text_shaping_state.add_text(
            &mut self.queue,
            &mut self.glyph_state,
            font_ref,
            text,
            bottom_left,
            size,
            color,
        );
    }

    pub fn draw_scene(&mut self, scene: &Scene, window: &Window) {
        window.set_inner_size(PhysicalSize::new(
            scene.window_size.x as u32,
            scene.window_size.y as u32,
        ));

        self.clear(scene.background_color);

        for quad in scene.quads.iter() {
            self.add_quad(quad.top_left, quad.size, quad.color);
        }

        let font = Font::from_name(&scene.font_name).unwrap();
        let font_ref = font.as_ref().unwrap();

        for glyph in scene.glyphs.iter() {
            self.add_glyph(
                font_ref,
                font_ref.charmap().map(glyph.character),
                glyph.bottom_left,
                glyph.size,
                glyph.color,
            );
        }

        for text in scene.texts.iter() {
            self.add_text(
                font_ref,
                &text.text,
                text.bottom_left,
                text.size,
                text.color,
            );
        }
    }
}
