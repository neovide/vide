use glam::*;

#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug, Default)]
#[repr(C)]
// NOTE: Keep the ATTRIBS array in sync with this struct
pub struct PathVertex {
    pub color: Vec4,
    pub position: Vec2,
    pub _padding: Vec2,
}
