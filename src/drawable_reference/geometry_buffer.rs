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

#[cfg(test)]
mod tests {
    use super::*;

    #[repr(C)]
    #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug, Default)]
    struct TestVertex {
        position: [f32; 2],
    }

    impl GeometryVertex for TestVertex {
        fn vertex_attributes() -> Vec<VertexAttribute> {
            vec![VertexAttribute {
                format: VertexFormat::Float32x2,
                offset: 0,
                shader_location: 0,
            }]
        }
    }

    #[test]
    fn test_geometry_buffer() {
        let width = 100;
        let height = 100;

        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::VULKAN,
            ..Default::default()
        });

        let renderer = smol::block_on(async {
            let adapter = instance
                .request_adapter(&RequestAdapterOptions {
                    power_preference: PowerPreference::default(),
                    force_fallback_adapter: false,
                    compatible_surface: None,
                })
                .await
                .unwrap();
            Renderer::new(width, height, adapter, TextureFormat::Rgba8UnormSrgb).await
        });

        let geometry_buffer = GeometryBuffer::<TestVertex>::new(&renderer, "test");

        let texture_desc = TextureDescriptor {
            size: Extent3d {
                width: renderer.width,
                height: renderer.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: None,
            view_formats: &[],
        };
        let texture = renderer.device.create_texture(&texture_desc);
        let texture_view = texture.create_view(&Default::default());

        let mut encoder = renderer
            .device
            .create_command_encoder(&CommandEncoderDescriptor { label: None });
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            let vertices = vec![
                TestVertex {
                    position: [0.0, 0.0],
                },
                TestVertex {
                    position: [0.0, 1.0],
                },
                TestVertex {
                    position: [1.0, 1.0],
                },
                TestVertex {
                    position: [1.0, 0.0],
                },
            ];
            let indices = vec![0, 1, 2, 0, 2, 3];

            geometry_buffer.upload(&vertices, &indices, &renderer.queue);
            assert_eq!(
                geometry_buffer
                    .previous_vertex_count
                    .load(Ordering::Relaxed),
                0
            );
            assert_eq!(
                geometry_buffer.previous_index_count.load(Ordering::Relaxed),
                0
            );
            assert_eq!(geometry_buffer.vertex_count.load(Ordering::Relaxed), 4);
            assert_eq!(geometry_buffer.index_count.load(Ordering::Relaxed), 6);
            geometry_buffer.draw(&mut render_pass);
            assert_eq!(
                geometry_buffer
                    .previous_vertex_count
                    .load(Ordering::Relaxed),
                4
            );
            assert_eq!(
                geometry_buffer.previous_index_count.load(Ordering::Relaxed),
                6
            );
            assert_eq!(geometry_buffer.vertex_count.load(Ordering::Relaxed), 4);
            assert_eq!(geometry_buffer.index_count.load(Ordering::Relaxed), 6);

            geometry_buffer.upload(&vertices, &indices, &renderer.queue);
            assert_eq!(
                geometry_buffer
                    .previous_vertex_count
                    .load(Ordering::Relaxed),
                4
            );
            assert_eq!(
                geometry_buffer.previous_index_count.load(Ordering::Relaxed),
                6
            );
            assert_eq!(geometry_buffer.vertex_count.load(Ordering::Relaxed), 8);
            assert_eq!(geometry_buffer.index_count.load(Ordering::Relaxed), 12);
            geometry_buffer.draw(&mut render_pass);
            assert_eq!(
                geometry_buffer
                    .previous_vertex_count
                    .load(Ordering::Relaxed),
                8
            );
            assert_eq!(
                geometry_buffer.previous_index_count.load(Ordering::Relaxed),
                12
            );
            assert_eq!(geometry_buffer.vertex_count.load(Ordering::Relaxed), 8);
            assert_eq!(geometry_buffer.index_count.load(Ordering::Relaxed), 12);
        }
    }
}
