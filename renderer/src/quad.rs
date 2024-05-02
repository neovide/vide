use shader::{InstancedQuad, ShaderConstants};
use wgpu::*;

use crate::renderer::Drawable;

#[cfg(not(target_arch = "spirv"))]
pub struct QuadState {
    buffer: Buffer,
    pub quads: Vec<InstancedQuad>,
    bind_group: BindGroup,
    render_pipeline: RenderPipeline,
}

impl QuadState {
    #[cfg(not(target_arch = "spirv"))]
    pub fn new(device: &Device, shader: &ShaderModule, swapchain_format: TextureFormat) -> Self {
        let buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Quad buffer"),
            size: std::mem::size_of::<InstancedQuad>() as u64 * 100000,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Quad bind group layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Quad bind group"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Quad Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[PushConstantRange {
                stages: ShaderStages::all(),
                range: 0..std::mem::size_of::<ShaderConstants>() as u32,
            }],
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Quad Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "quad::vertex",
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "quad::fragment",
                targets: &[Some(ColorTargetState {
                    format: swapchain_format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self {
            buffer,
            bind_group,
            quads: Vec::new(),
            render_pipeline,
        }
    }

    pub fn clear(&mut self) {
        self.quads.clear();
    }
}

impl Drawable for QuadState {
    fn draw<'b, 'a: 'b>(
        &'a self,
        queue: &Queue,
        render_pass: &mut RenderPass<'b>,
        constants: ShaderConstants,
        _universal_bind_group: &'a BindGroup,
    ) {
        render_pass.set_pipeline(&self.render_pipeline); // 2.
        render_pass.set_push_constants(ShaderStages::all(), 0, bytemuck::cast_slice(&[constants]));

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.quads[..]));
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..6, 0..self.quads.len() as u32);
    }
}
