use glamour::Rect;
use wgpu::*;

use crate::{
    drawable_reference::DrawableReference, LayerContents, PrimitiveBatch, Renderer, Resources,
    ShaderConstants,
};

pub trait Drawable {
    fn new(renderer: &Renderer) -> Self
    where
        Self: Sized;

    fn name(&self) -> &str;
    fn references(&self) -> Vec<&dyn DrawableReference>;
    fn needs_offscreen_copy(&self) -> bool {
        false
    }
    fn start_frame(&mut self);
    fn has_work(&self, batch: &PrimitiveBatch) -> bool;
    fn targets(&self, format: TextureFormat) -> Vec<Option<ColorTargetState>> {
        vec![Some(ColorTargetState {
            format,
            blend: Some(BlendState::ALPHA_BLENDING),
            write_mask: ColorWrites::ALL,
        })]
    }

    fn draw<'b, 'a: 'b>(
        &'a mut self,
        queue: &Queue,
        render_pass: &mut RenderPass<'b>,
        constants: ShaderConstants,
        resources: &Resources,
        clip: Option<Rect<u32>>,
        layer: &PrimitiveBatch,
    );
}
