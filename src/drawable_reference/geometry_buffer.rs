use std::{
    marker::PhantomData,
    sync::atomic::{AtomicU32, Ordering},
};

use wgpu::*;

use crate::Renderer;

use super::DrawableReference;

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
    previous_vertex_count: AtomicU32,
    vertex_attr_array: Vec<VertexAttribute>,
    previous_index_count: AtomicU32,
    vertex_count: AtomicU32,

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
            vertex_count: 0.into(),
            previous_vertex_count: 0.into(),
            index_count: 0.into(),
            previous_index_count: 0.into(),
            vertex_attr_array: Vertex::vertex_attributes(),
            _phantom: PhantomData,
        }
    }

    pub fn start_frame(&mut self) {
        self.vertex_count.store(0, Ordering::Relaxed);
        self.previous_vertex_count.store(0, Ordering::Relaxed);
        self.index_count.store(0, Ordering::Relaxed);
        self.previous_index_count.store(0, Ordering::Relaxed);
    }

    pub fn upload(&self, vertices: &[Vertex], indices: &[u32], queue: &Queue) {
        let previous_vertex_count = self.previous_vertex_count.load(Ordering::Relaxed);
        let new_vertex_memory_start =
            previous_vertex_count as u64 * std::mem::size_of::<Vertex>() as u64;
        let previous_index_count = self.previous_index_count.load(Ordering::Relaxed);
        let new_index_memory_start =
            previous_index_count as u64 * std::mem::size_of::<u32>() as u64;

        queue.write_buffer(
            &self.vertex_buffer,
            new_vertex_memory_start as u64,
            bytemuck::cast_slice(vertices),
        );
        queue.write_buffer(
            &self.index_buffer,
            new_index_memory_start as u64,
            bytemuck::cast_slice(indices),
        );

        self.vertex_count.store(
            previous_vertex_count + vertices.len() as u32,
            Ordering::Relaxed,
        );
        self.index_count.store(
            previous_index_count + indices.len() as u32,
            Ordering::Relaxed,
        );
    }

    pub fn draw<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        let vertex_count = self.vertex_count.load(Ordering::Relaxed);
        let previous_vertex_count = self.previous_vertex_count.load(Ordering::Relaxed);
        let new_vertex_memory_start =
            previous_vertex_count as u64 * std::mem::size_of::<Vertex>() as u64;
        let new_vertex_memory_end = vertex_count as u64 * std::mem::size_of::<Vertex>() as u64;
        let index_count = self.index_count.load(Ordering::Relaxed);
        let previous_index_count = self.previous_index_count.load(Ordering::Relaxed);
        let new_index_memory_start =
            previous_index_count as u64 * std::mem::size_of::<u32>() as u64;
        let new_index_memory_end = index_count as u64 * std::mem::size_of::<u32>() as u64;

        render_pass.set_vertex_buffer(
            0,
            self.vertex_buffer
                .slice(new_vertex_memory_start..new_vertex_memory_end),
        );
        render_pass.set_index_buffer(
            self.index_buffer
                .slice(new_index_memory_start..new_index_memory_end),
            IndexFormat::Uint32,
        );
        render_pass.draw_indexed(0..index_count - previous_index_count, 0, 0..1);

        self.previous_vertex_count
            .store(vertex_count, Ordering::Relaxed);
        self.previous_index_count
            .store(index_count, Ordering::Relaxed);
    }
}

impl<Vertex> DrawableReference for GeometryBuffer<Vertex> {
    fn vertex<'b, 'a: 'b>(&'a self) -> Option<VertexBufferLayout<'b>> {
        Some(VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &self.vertex_attr_array,
        })
    }
}
