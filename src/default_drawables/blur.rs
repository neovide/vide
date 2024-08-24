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
// An axis aligned background blur supporting positioning, scaling, corner radius.
pub struct InstancedBlur {
    pub color: Vec4,
    pub _padding: Vec4,
    pub top_left: Vec2,
    pub size: Vec2,
    pub __padding: Vec2,
    pub corner_radius: f32,
    pub blur: f32,
}

pub struct BlurState {
    blur_buffer: InstanceBuffer<InstancedBlur>,
}

impl Drawable for BlurState {
    fn new(renderer: &Renderer) -> Self {
        let blur_buffer = InstanceBuffer::new(renderer, "blur");
        Self { blur_buffer }
    }

    fn name(&self) -> &str {
        "blur"
    }

    fn references(&self) -> Vec<&dyn DrawableReference> {
        vec![&self.blur_buffer]
    }

    fn start_frame(&mut self) {
        self.blur_buffer.start_frame();
    }

    fn has_work(&self, batch: &PrimitiveBatch) -> bool {
        batch.is_blurs()
    }

    fn requires_offscreen_copy(&self) -> bool {
        true
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
        if let Some(blurs) = batch.as_blur_vec() {
            self.blur_buffer.upload(
                blurs.iter().map(|blur| blur.to_instanced()).collect(),
                queue,
            );
            self.blur_buffer.draw(render_pass);
        }
    }
}
