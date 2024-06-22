use glam::*;
use glamour::{Point2, Size2};
use palette::Srgba;
use wgpu::*;

use crate::{
    drawable::{Drawable, DrawableReference, InstanceBuffer},
    scene::Layer,
    shader::ShaderConstants,
    Quad, Renderer, Resources,
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
    // 0: no blur
    // <0: internal blur of the background with kernel radius `blur`
    // >0: external blur of quad edge with radius `blur`
    pub blur: f32,
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

    fn references<'a>(&'a self) -> Vec<&'a dyn DrawableReference> {
        vec![&self.quad_buffer]
    }

    fn draw<'b, 'a: 'b>(
        &'a mut self,
        queue: &Queue,
        render_pass: &mut RenderPass<'b>,
        constants: ShaderConstants,
        _resources: &Resources,
        layer: &Layer,
    ) {
        let mut quads = Vec::new();
        if layer.background_color.is_some() || layer.background_blur_radius != 0.0 {
            quads.push(
                Quad::new(
                    layer
                        .clip
                        .map(|clip| clip.origin)
                        .unwrap_or(Point2::<u32>::ZERO)
                        .try_cast()
                        .unwrap(),
                    layer
                        .clip
                        .map(|clip| clip.size.try_cast().unwrap())
                        .unwrap_or(Size2::new(
                            constants.surface_size.x,
                            constants.surface_size.y,
                        )),
                    layer.background_color.unwrap_or(Srgba::new(1., 1., 1., 1.)),
                )
                .with_background_blur(layer.background_blur_radius)
                .to_instanced(),
            );
        }

        quads.extend(layer.quads.iter().map(|quad| quad.to_instanced()));
        self.quad_buffer.upload(quads, queue);
        self.quad_buffer.draw(render_pass);
    }
}
