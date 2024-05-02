use rust_embed::RustEmbed;
use wgpu::*;

use shader::ShaderConstants;
use winit::{event::Event, window::Window};
use glam::*;

pub use crate::resources::Resources;
use crate::{
    glyph::GlyphState, path::PathState, quad::QuadState, scene::Layer, sprite::SpriteState, Scene,
};

pub trait Drawable {
    fn new(resources: &Resources) -> Self
    where
        Self: Sized;

    fn draw<'b, 'a: 'b>(
        &'a mut self,
        queue: &Queue,
        render_pass: &mut RenderPass<'b>,
        constants: ShaderConstants,
        universal_bind_group: &'a BindGroup,
        layer: &Layer,
    );
}

pub struct Renderer<'a> {
    pub(crate) resources: Resources<'a>,
    pub(crate) drawables: Vec<Box<dyn Drawable>>,
}

impl<'a> Renderer<'a> {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: &'a Window) -> Self {
        let resources = Resources::new(window).await;

        Self {
            resources,
            drawables: Vec::new(),
        }
    }

    pub fn with_drawable<T: Drawable + 'static>(mut self) -> Self {
        let drawable = T::new(&self.resources);
        self.drawables.push(Box::new(drawable));
        self
    }

    pub fn with_default_drawables<A: RustEmbed + 'static>(self) -> Self {
        self.with_drawable::<QuadState>()
            .with_drawable::<GlyphState>()
            .with_drawable::<PathState>()
            .with_drawable::<SpriteState<A>>()
    }

    pub fn draw_scene(&mut self, scene: &Scene) -> bool {
        if let Err(render_error) = self.resources.render(scene, self.drawables.as_mut_slice()) {
            eprintln!("Render error: {:?}", render_error);
            match render_error {
                SurfaceError::Lost => {
                    if let Some(surface) = &self.resources.surface {
                        surface.configure(&self.resources.device, &self.resources.surface_config);
                    }
                    true
                }
                SurfaceError::OutOfMemory => {
                    eprintln!("Out of memory");
                    false
                }
                _ => true,
            }
        } else {
            true
        }
    }

    pub fn handle_event(&mut self, window: &'a Window, event: &Event<()>) {
        self.resources.handle_event(window, event);
    }
}
