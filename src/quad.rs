use glamour::{Point2, Size2};
use palette::Srgba;
use shader::{InstancedQuad, ShaderConstants, ShaderModules};
use wgpu::*;

use crate::{renderer::Drawable, scene::Layer, Quad, Renderer};

pub struct QuadState {
    buffer: Buffer,
    bind_group_layout: BindGroupLayout,
    bind_group: BindGroup,
    render_pipeline: RenderPipeline,
}

fn create_render_pipeline(
    device: &Device,
    universal_bind_group_layout: &BindGroupLayout,
    shaders: &ShaderModules,
    format: &TextureFormat,
    bind_group_layout: &BindGroupLayout,
) -> RenderPipeline {
    let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("Quad Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout, &universal_bind_group_layout],
        push_constant_ranges: &[PushConstantRange {
            stages: ShaderStages::all(),
            range: 0..std::mem::size_of::<ShaderConstants>() as u32,
        }],
    });

    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("Quad Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: VertexState {
            module: shaders.get_vertex("quad"),
            entry_point: "main",
            buffers: &[],
        },
        fragment: Some(FragmentState {
            module: shaders.get_fragment("quad"),
            entry_point: "main",
            targets: &[Some(ColorTargetState {
                format: *format,
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
            count: 4,
            ..Default::default()
        },
        multiview: None,
    })
}

impl Drawable for QuadState {
    fn new(
        Renderer {
            device,
            universal_bind_group_layout,
            shaders,
            format,
            ..
        }: &Renderer,
    ) -> Self {
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
                    ty: BufferBindingType::Storage { read_only: true },
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

        let render_pipeline = create_render_pipeline(
            device,
            universal_bind_group_layout,
            shaders,
            format,
            &bind_group_layout,
        );

        Self {
            buffer,
            bind_group_layout,
            bind_group,
            render_pipeline,
        }
    }

    fn reload(
        &mut self,
        device: &Device,
        shaders: &ShaderModules,
        format: &TextureFormat,
        universal_bind_group_layout: &BindGroupLayout,
    ) {
        self.render_pipeline = create_render_pipeline(
            device,
            universal_bind_group_layout,
            shaders,
            format,
            &self.bind_group_layout,
        )
    }

    fn draw<'b, 'a: 'b>(
        &'a mut self,
        queue: &Queue,
        render_pass: &mut RenderPass<'b>,
        constants: ShaderConstants,
        universal_bind_group: &'a BindGroup,
        layer: &Layer,
    ) {
        let mut quads = Vec::new();
        if layer.background_color.is_some() || layer.background_blur_radius != 0.0 {
            quads.push(
                Quad::new(
                    layer
                        .clip
                        .map(|clip| clip.origin)
                        .unwrap_or(Point2::<u32>::ZERO)
                        .try_cast()
                        .unwrap(),
                    layer
                        .clip
                        .map(|clip| clip.size.try_cast().unwrap())
                        .unwrap_or(Size2::new(
                            constants.surface_size.x,
                            constants.surface_size.y,
                        )),
                    layer.background_color.unwrap_or(Srgba::new(1., 1., 1., 1.)),
                )
                .with_background_blur(layer.background_blur_radius)
                .to_instanced(),
            );
        }

        quads.extend(layer.quads.iter().map(|quad| quad.to_instanced()));

        render_pass.set_pipeline(&self.render_pipeline); // 2.
        render_pass.set_push_constants(ShaderStages::all(), 0, bytemuck::cast_slice(&[constants]));

        let quad_data: &[u8] = bytemuck::cast_slice(&quads[..]);
        queue.write_buffer(&self.buffer, 0, quad_data);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_bind_group(1, universal_bind_group, &[]);
        render_pass.draw(0..6, 0..quads.len() as u32);
    }
}
