use wgpu::*;

use crate::{pipeline_builder::PipelineReference, Layer, Renderer, Resources, ShaderConstants};

pub trait Drawable {
    fn new(renderer: &Renderer) -> Self
    where
        Self: Sized;

    fn name(&self) -> &str;
    fn references<'a>(&'a self) -> Vec<&'a dyn PipelineReference>;

    fn draw<'b, 'a: 'b>(
        &'a mut self,
        queue: &Queue,
        render_pass: &mut RenderPass<'b>,
        constants: ShaderConstants,
        resources: &Resources,
        layer: &Layer,
    );
}
