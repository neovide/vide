use std::marker::PhantomData;

use wgpu::*;

use crate::Renderer;

use super::DrawableReference;

/// A buffer containing instances of a struct which will be drawn with a standard 6 vertex quad.
pub struct InstanceBuffer<Instance> {
    instance_buffer: Buffer,
    instance_count: u32,

    previous_instance_count: u32,

    _phantom: PhantomData<Instance>,
}

impl<Instance: bytemuck::Pod> InstanceBuffer<Instance> {
    pub fn new(Renderer { device, .. }: &Renderer, name: &str) -> Self {
        let instance_buffer = device.create_buffer(&BufferDescriptor {
            label: Some(&format!("{} instance buffer", name)),
            size: std::mem::size_of::<Instance>() as u64 * 100000,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            instance_buffer,
            instance_count: 0,
            previous_instance_count: 0,
            _phantom: PhantomData,
        }
    }

    pub fn start_frame(&mut self) {
        self.previous_instance_count = 0;
        self.instance_count = 0;
    }

    pub fn upload(&mut self, instances: Vec<Instance>, queue: &Queue) {
        let instance_data: &[u8] = bytemuck::cast_slice(&instances[..]);

        queue.write_buffer(
            &self.instance_buffer,
            std::mem::size_of::<Instance>() as u64 * self.previous_instance_count as u64,
            instance_data,
        );
        self.instance_count += instances.len() as u32;
    }

    pub fn draw(&mut self, render_pass: &mut RenderPass<'_>) {
        render_pass.draw(0..6, self.previous_instance_count..self.instance_count);
        self.previous_instance_count = self.instance_count;
    }
}

impl<Instance> DrawableReference for InstanceBuffer<Instance> {
    fn layout(&self) -> Option<BindGroupLayoutEntry> {
        Some(BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        })
    }

    fn entry(&self) -> Option<BindGroupEntry> {
        Some(BindGroupEntry {
            binding: 0,
            resource: self.instance_buffer.as_entire_binding(),
        })
    }
}
