use glam::*;
use glam::{vec2, Vec4};
use lyon::{
    geom::point,
    lyon_tessellation::{
        BuffersBuilder, FillOptions, FillTessellator, FillVertex, StrokeOptions, StrokeTessellator,
        StrokeVertex, VertexBuffers,
    },
    path::Path,
};
use shader::{ShaderConstants, ShaderModules};
use wgpu::*;

use crate::{
    renderer::{Drawable, Renderer},
    scene::{Layer, PathCommand},
};

#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug, Default)]
#[repr(C)]
// NOTE: Keep the ATTRIBS array in sync with this struct
pub struct PathVertex {
    pub color: Vec4,
    pub position: Vec2,
    pub _padding: Vec2,
}

pub struct PathState {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
}

impl Drawable for PathState {
    fn new(Renderer { device, .. }: &Renderer) -> Self {
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

        Self {
            vertex_buffer,
            index_buffer,
        }
    }

    fn create_pipeline(
        &self,
        device: &Device,
        shaders: &ShaderModules,
        format: &TextureFormat,
        _universal_bind_group_layout: &BindGroupLayout,
    ) -> Result<RenderPipeline, String> {
        Ok(device.create_render_pipeline(&RenderPipelineDescriptor {
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
                module: shaders.get_vertex("path")?,
                entry_point: "main",
                buffers: &[VertexBufferLayout {
                    array_stride: std::mem::size_of::<PathVertex>() as BufferAddress,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &vertex_attr_array![0 => Float32x4, 1 => Float32x2, 2 => Float32x2],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: shaders.get_fragment("path")?,
                entry_point: "main",
                targets: &[Some(ColorTargetState {
                    format: *format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
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
        }))
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
                let fill = Vec4::from_array(fill.into_linear().into());
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
                let stroke = Vec4::from_array(stroke.into_linear().into());
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
