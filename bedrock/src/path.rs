use glam::vec2;
use lyon::{
    geom::point,
    lyon_tessellation::{
        BuffersBuilder, FillOptions, FillTessellator, FillVertex, StrokeOptions, StrokeTessellator,
        StrokeVertex, VertexBuffers,
    },
    path::Path,
};
use shader::{PathVertex, ShaderConstants};
use wgpu::*;

use crate::{
    renderer::{Drawable, Resources},
    scene::{Layer, PathCommand},
};

pub struct PathState {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    render_pipeline: RenderPipeline,
}

impl Drawable for PathState {
    fn new(
        Resources {
            device,
            shader,
            swapchain_format,
            ..
        }: &Resources,
    ) -> Self {
        let vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Path Vertex Buffer"),
            size: std::mem::size_of::<PathVertex>() as u64 * 100000,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Path Index Buffer"),
            size: std::mem::size_of::<u32>() as u64 * 100000,
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Path render pipeline"),
            layout: Some(&device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Path Pipeline layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[PushConstantRange {
                    stages: ShaderStages::all(),
                    range: 0..std::mem::size_of::<ShaderConstants>() as u32,
                }],
            })),
            vertex: VertexState {
                module: &shader,
                entry_point: "path::path_vertex",
                buffers: &[VertexBufferLayout {
                    array_stride: std::mem::size_of::<PathVertex>() as BufferAddress,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &vertex_attr_array![0 => Float32x4, 1 => Float32x2, 2 => Float32x2],
                }],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "path::path_fragment",
                targets: &[Some(ColorTargetState {
                    format: *swapchain_format,
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
        });

        Self {
            vertex_buffer,
            index_buffer,
            render_pipeline,
        }
    }

    fn draw<'b, 'a: 'b>(
        &'a mut self,
        queue: &Queue,
        render_pass: &mut RenderPass<'b>,
        constants: ShaderConstants,
        _universal_bind_group: &'a BindGroup,
        layer: &Layer,
    ) {
        let mut geometry: VertexBuffers<PathVertex, u32> = VertexBuffers::new();
        let mut fill_tesselator = FillTessellator::new();
        let mut stroke_tesselator = StrokeTessellator::new();

        for scene_path in layer.paths.iter() {
            let mut builder = Path::builder();
            builder.begin(point(scene_path.start.x, scene_path.start.y));
            for path_command in scene_path.commands.iter() {
                match path_command {
                    PathCommand::LineTo { to } => {
                        builder.line_to(point(to.x, to.y));
                    }
                    PathCommand::QuadraticBezierTo { control, to } => {
                        builder.quadratic_bezier_to(point(control.x, control.y), point(to.x, to.y));
                    }
                    PathCommand::CubicBezierTo {
                        control1,
                        control2,
                        to,
                    } => {
                        builder.cubic_bezier_to(
                            point(control1.x, control1.y),
                            point(control2.x, control2.y),
                            point(to.x, to.y),
                        );
                    }
                }
            }
            builder.close();
            let path = builder.build();

            if let Some(fill) = scene_path.fill {
                fill_tesselator
                    .tessellate_path(
                        &path,
                        &FillOptions::default(),
                        &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| PathVertex {
                            color: fill,
                            position: vec2(vertex.position().x, vertex.position().y),
                            ..Default::default()
                        }),
                    )
                    .expect("Could not tesselate path");
            }

            if let Some((width, stroke)) = scene_path.stroke {
                stroke_tesselator
                    .tessellate_path(
                        &path,
                        &StrokeOptions::default().with_line_width(width),
                        &mut BuffersBuilder::new(&mut geometry, |vertex: StrokeVertex| {
                            PathVertex {
                                color: stroke,
                                position: vec2(vertex.position().x, vertex.position().y),
                                ..Default::default()
                            }
                        }),
                    )
                    .expect("Could not tesselate path");
            }

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_push_constants(
                ShaderStages::all(),
                0,
                bytemuck::cast_slice(&[constants]),
            );

            queue.write_buffer(
                &self.vertex_buffer,
                0,
                bytemuck::cast_slice(&geometry.vertices[..]),
            );
            queue.write_buffer(
                &self.index_buffer,
                0,
                bytemuck::cast_slice(&geometry.indices[..]),
            );

            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
            render_pass.draw_indexed(0..geometry.indices.len() as u32, 0, 0..1);
        }
    }
}
