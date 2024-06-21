use glam::Vec4;
use glam::*;
use glamour::AsRaw;
use wgpu::{BindGroupLayout, RenderPipeline, *};

use crate::pipeline_builder::{Atlas, InstanceBuffer, PipelineBuilder};
use crate::{
    drawable::Drawable,
    scene::{Layer, Sprite},
    shader::{ShaderConstants, ShaderModules},
    Renderer,
};
use crate::{Resources, TextureId};

#[derive(Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct InstancedSprite {
    pub top_left: Vec2,
    pub size: Vec2,
    pub atlas_top_left: Vec2,
    pub atlas_size: Vec2,
    pub color: Vec4,
}

pub struct SpriteState {
    pipeline_builder: PipelineBuilder,
    sprite_buffer: InstanceBuffer<InstancedSprite>,
    atlas: Atlas<TextureId>,
}

impl SpriteState {
    pub fn upload_sprite(
        &mut self,
        resources: &Resources,
        queue: &Queue,
        sprite: &Sprite<TextureId>,
    ) -> InstancedSprite {
        let Some((_, sprite_location)) = self.atlas.lookup_or_upload(queue, sprite.texture, || {
            let texture = resources.textures.get(&sprite.texture)?;

            Some(((), texture.data.clone(), texture.size))
        }) else {
            panic!("Referenced texture not in scene resources");
        };

        InstancedSprite {
            top_left: *sprite.top_left.as_raw(),
            size: *sprite.size.as_raw(),
            atlas_top_left: sprite_location.min.as_raw().as_vec2(),
            atlas_size: sprite_location.size().as_raw().as_vec2(),
            color: Vec4::from_array(sprite.color.into_linear().into()),
        }
    }
}

impl Drawable for SpriteState {
    fn new(renderer: &Renderer) -> Self {
        let sprite_buffer = InstanceBuffer::new(renderer, "sprite");
        let atlas = Atlas::new(renderer, "sprite");
        let pipeline_builder =
            PipelineBuilder::new(renderer, "Sprite", "sprite", &[&sprite_buffer, &atlas]);

        Self {
            sprite_buffer,
            atlas,
            pipeline_builder,
        }
    }

    fn create_pipeline(
        &self,
        device: &Device,
        shaders: &ShaderModules,
        format: &TextureFormat,
        universal_bind_group_layout: &BindGroupLayout,
    ) -> Result<RenderPipeline, String> {
        self.pipeline_builder.build(
            device,
            shaders,
            format,
            universal_bind_group_layout,
            &[&self.sprite_buffer, &self.atlas],
        )
    }

    fn draw<'b, 'a: 'b>(
        &'a mut self,
        queue: &Queue,
        render_pass: &mut RenderPass<'b>,
        constants: ShaderConstants,
        universal_bind_group: &'a BindGroup,
        resources: &Resources,
        layer: &Layer,
    ) {
        let sprites: Vec<_> = layer
            .sprites
            .iter()
            .map(|sprite| self.upload_sprite(resources, queue, sprite))
            .collect();

        self.pipeline_builder
            .set_bind_groups(render_pass, constants, universal_bind_group);
        self.sprite_buffer.upload(sprites, queue);
        self.sprite_buffer.draw(render_pass);
    }
}
