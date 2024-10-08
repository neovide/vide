use glam::*;
use glamour::Rect;
use wgpu::*;

use crate::{
    drawable::Drawable,
    drawable_reference::{DrawableReference, InstanceBuffer},
    shader::ShaderConstants,
    PrimitiveBatch, Renderer, Resources,
};

#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Default)]
#[repr(C, align(64))]
// An axis aligned quad supporting positioning, scaling, corner radius, and optionally an internal blur with
// the previous layer or an external blur for use with shadows.
pub struct InstancedQuad {
    pub color: Vec4,
    pub _padding: Vec4,
    pub top_left: Vec2,
    pub size: Vec2,
    pub __padding: Vec2,
    pub corner_radius: f32,
    pub edge_blur: f32,
}

pub struct QuadState {
    quad_buffer: InstanceBuffer<InstancedQuad>,
}

impl Drawable for QuadState {
    fn new(renderer: &Renderer) -> Self {
        let quad_buffer = InstanceBuffer::new(renderer, "quad");
        Self { quad_buffer }
    }

    fn name(&self) -> &str {
        "quad"
    }

    fn references(&self) -> Vec<&dyn DrawableReference> {
        vec![&self.quad_buffer]
    }

    fn start_frame(&mut self) {
        self.quad_buffer.start_frame();
    }

    fn has_work(&self, batch: &PrimitiveBatch) -> bool {
        batch.is_quads()
    }

    fn draw<'b, 'a: 'b>(
        &'a mut self,
        queue: &Queue,
        render_pass: &mut RenderPass<'b>,
        _constants: ShaderConstants,
        _resources: &Resources,
        _clip: Option<Rect<u32>>,
        batch: &PrimitiveBatch,
    ) {
        if let Some(quads) = batch.as_quad_vec() {
            self.quad_buffer.upload(
                quads.iter().map(|quad| quad.to_instanced()).collect(),
                queue,
            );
            self.quad_buffer.draw(render_pass);
        }
    }
}
