mod font;
mod glyph;
mod path;
mod quad;
mod renderer;
mod scene;

use glam::{vec2, Vec2};
use rust_embed::*;

pub use renderer::Renderer;
pub use scene::Scene;

pub const ATLAS_SIZE: Vec2 = vec2(1024., 1024.);

#[derive(RustEmbed)]
#[folder = "spirv"]
struct Asset;
