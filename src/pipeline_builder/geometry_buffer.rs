use std::{
    marker::PhantomData,
    sync::atomic::{AtomicU32, Ordering},
};

use wgpu::*;

use crate::Renderer;

use super::PipelineReference;

pub trait GeometryVertex {
    fn vertex_attributes() -> Vec<VertexAttribute>;
}

pub struct GeometryBuffer<Vertex> {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    // Use an AtomicU32 here because of lifetime struggles. We only need to mutate this one thing
    // on upload, so swapping to an atomic lets us remove the requirement.
    //
    // I expect there should be a way to do this without but its not coming to me atm.
    index_count: AtomicU32,
    vertex_attr_array: Vec<VertexAttribute>,

    _phantom: PhantomData<Vertex>,
}

impl<Vertex: bytemuck::Pod + GeometryVertex> GeometryBuffer<Vertex> {
    pub fn new(Renderer { device, .. }: &Renderer, name: &str) -> Self {
        let vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: Some(&format!("{} vertex buffer", name)),
            size: std::mem::size_of::<Vertex>() as u64 * 100000,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&BufferDescriptor {
            label: Some(&format!("{} index buffer", name)),
            size: std::mem::size_of::<u32>() as u64 * 100000,
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            vertex_buffer,
            index_buffer,
            index_count: 0.into(),
            vertex_attr_array: Vertex::vertex_attributes(),
            _phantom: PhantomData,
        }
    }

    pub fn upload(&self, vertices: &Vec<Vertex>, indices: &Vec<u32>, queue: &Queue) {
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices[..]));
        queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&indices[..]));
        self.index_count
            .store(indices.len() as u32, Ordering::Relaxed);
    }

    pub fn draw<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.index_count.load(Ordering::Relaxed), 0, 0..1);
    }
}

impl<Vertex> PipelineReference for GeometryBuffer<Vertex> {
    fn vertex<'b, 'a: 'b>(&'a self) -> Option<VertexBufferLayout<'b>> {
        Some(VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &self.vertex_attr_array,
        })
    }
}
