use wgpu::{BindGroup, BindGroupLayout, Device, Queue, RenderPass, RenderPipeline, TextureFormat};

use crate::{Layer, Renderer, Resources, ShaderConstants, ShaderModules};

pub trait Drawable {
    fn new(renderer: &Renderer) -> Self
    where
        Self: Sized;

    fn create_pipeline(
        &self,
        device: &Device,
        shaders: &ShaderModules,
        format: &TextureFormat,
        universal_bind_group_layout: &BindGroupLayout,
    ) -> Result<RenderPipeline, String>;

    fn draw<'b, 'a: 'b>(
        &'a mut self,
        queue: &Queue,
        render_pass: &mut RenderPass<'b>,
        constants: ShaderConstants,
        universal_bind_group: &'a BindGroup,
        resources: &Resources,
        layer: &Layer,
    );
}
