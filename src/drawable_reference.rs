mod atlas;
mod geometry_buffer;
mod instance_buffer;

use wgpu::*;

pub use atlas::*;
pub use geometry_buffer::*;
pub use instance_buffer::*;

pub trait DrawableReference {
    fn layout(&self) -> Option<BindGroupLayoutEntry> {
        None
    }
    fn entry(&self) -> Option<BindGroupEntry> {
        None
    }
    fn vertex<'b, 'a: 'b>(&'a self) -> Option<VertexBufferLayout<'b>> {
        None
    }
}
