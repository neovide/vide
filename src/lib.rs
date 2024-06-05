mod glyph;
mod offscreen_renderer;
mod path;
mod quad;
mod renderer;
mod scene;
mod shaper;
mod sprite;
mod winit_renderer;

#[cfg(test)]
mod test;

use glam::{vec2, Vec2};

pub use offscreen_renderer::OffscreenRenderer;
pub use renderer::Renderer;
pub use scene::*;
pub use shaper::Shaper;
pub use winit_renderer::WinitRenderer;

pub const ATLAS_SIZE: Vec2 = vec2(1024., 1024.);
