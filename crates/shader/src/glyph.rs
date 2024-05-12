use glam::*;

#[derive(Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct InstancedGlyph {
    pub bottom_left: Vec2,
    pub atlas_top_left: Vec2,
    pub atlas_size: Vec2,
    // Need a Vec2 of padding here so that the first 4 fields
    // Are some multiple of 16 bytes in size.
    // Vec2s are 8 bytes, Vec4s are 16 bytes.
    pub _padding: Vec2,
    pub color: Vec4,
}
