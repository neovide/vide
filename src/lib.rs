mod default_drawables;
mod drawable;
mod offscreen_renderer;
mod renderer;
mod scene;
mod shader;
mod shaper;
mod winit_renderer;

#[cfg(test)]
mod test;

use glam::{vec2, Vec2};

pub use offscreen_renderer::OffscreenRenderer;
pub use renderer::Renderer;
pub use scene::*;
pub use shader::*;
pub use shaper::Shaper;
pub use winit_renderer::WinitRenderer;

pub const ATLAS_SIZE: Vec2 = vec2(1024., 1024.);
