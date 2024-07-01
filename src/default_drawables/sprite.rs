use glam::Vec4;
use glam::*;
use glamour::{AsRaw, Rect};
use log::warn;
use wgpu::*;

use crate::{
    drawable::Drawable,
    drawable_reference::{Atlas, ConstructResult, DrawableReference, InstanceBuffer},
    scene::Sprite,
    shader::ShaderConstants,
    LayerContents, Renderer,
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
            let Some(texture) = resources.textures.get(&sprite.texture) else {
                warn!("Sprite texture not in resources");
                return ConstructResult::Failed;
            };

            ConstructResult::Constructed((), texture.data.clone(), texture.size)
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

        Self {
            sprite_buffer,
            atlas,
        }
    }

    fn name(&self) -> &str {
        "sprite"
    }

    fn references(&self) -> Vec<&dyn DrawableReference> {
        vec![&self.sprite_buffer, &self.atlas]
    }

    fn start_frame(&mut self) {
        self.sprite_buffer.start_frame();
    }

    fn draw<'b, 'a: 'b>(
        &'a mut self,
        queue: &Queue,
        render_pass: &mut RenderPass<'b>,
        _constants: ShaderConstants,
        resources: &Resources,
        _clip: Option<Rect<u32>>,
        layer: &LayerContents,
    ) {
        let sprites: Vec<_> = layer
            .sprites
            .iter()
            .map(|sprite| self.upload_sprite(resources, queue, sprite))
            .collect();

        self.sprite_buffer.upload(sprites, queue);
        self.sprite_buffer.draw(render_pass);
    }
}
