mod resources;

use wgpu::*;

use shader::ShaderConstants;
use winit::{event::Event, window::Window};

use crate::{glyph::GlyphState, path::PathState, quad::QuadState, scene::Layer, Scene};
pub(crate) use resources::Resources;

pub trait Drawable {
    fn draw<'b, 'a: 'b>(
        &'a mut self,
        queue: &Queue,
        render_pass: &mut RenderPass<'b>,
        constants: ShaderConstants,
        universal_bind_group: &'a BindGroup,
        layer: &Layer,
    );
}

pub struct Renderer {
    pub(crate) resources: Resources,

    pub(crate) quad_state: QuadState,
    pub(crate) glyph_state: GlyphState,
    pub(crate) path_state: PathState,
}

impl Renderer {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: &Window) -> Self {
        let resources = Resources::new(window).await;

        let quad_state = QuadState::new(&resources);
        let glyph_state = GlyphState::new(&resources);
        let path_state = PathState::new(&resources);

        Self {
            resources,

            quad_state,
            glyph_state,
            path_state,
        }
    }

    pub fn draw_scene(&mut self, scene: &Scene) -> bool {
        if let Err(render_error) = self.resources.render(
            scene,
            [
                &mut self.quad_state as &mut dyn Drawable,
                &mut self.glyph_state,
                &mut self.path_state,
            ],
        ) {
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

    pub fn handle_event(&mut self, window: &Window, event: &Event<()>) {
        self.resources.handle_event(window, event);
    }
}
