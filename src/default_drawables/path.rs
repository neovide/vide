use glam::*;
use glam::{vec2, Vec4};
use glamour::Rect;
use lyon::{
    geom::point,
    lyon_tessellation::{
        BuffersBuilder, FillOptions, FillTessellator, FillVertex, StrokeOptions, StrokeTessellator,
        StrokeVertex, VertexBuffers,
    },
    path::Path,
};
use wgpu::*;

use crate::{
    drawable::Drawable,
    drawable_reference::{DrawableReference, GeometryBuffer, GeometryVertex},
    renderer::Renderer,
    scene::PathCommand,
    shader::ShaderConstants,
    LayerContents, PrimitiveBatch, Resources,
};

#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug, Default)]
#[repr(C)]
// NOTE: Keep the ATTRIBS array in sync with this struct
pub struct PathVertex {
    pub color: Vec4,
    pub position: Vec2,
    pub _padding: Vec2,
}

impl GeometryVertex for PathVertex {
    fn vertex_attributes() -> Vec<VertexAttribute> {
        vertex_attr_array![0 => Float32x4, 1 => Float32x2, 2 => Float32x2]
            .into_iter()
            .collect()
    }
}

pub struct PathState {
    geometry_buffer: GeometryBuffer<PathVertex>,
}

impl Drawable for PathState {
    fn new(renderer: &Renderer) -> Self {
        let geometry_buffer = GeometryBuffer::new(renderer, "path");

        Self { geometry_buffer }
    }

    fn name(&self) -> &str {
        "path"
    }

    fn references(&self) -> Vec<&dyn DrawableReference> {
        vec![&self.geometry_buffer]
    }

    fn start_frame(&mut self) {
        self.geometry_buffer.start_frame();
    }

    fn has_work(&self, batch: &PrimitiveBatch) -> bool {
        batch.is_paths()
    }

    fn draw<'b, 'a: 'b>(
        &'a mut self,
        queue: &Queue,
        render_pass: &mut RenderPass<'b>,
        _constants: ShaderConstants,
        _resources: &Resources,
        _clip: Option<Rect<u32>>,
        batch: &PrimitiveBatch,
    ) {
        if let Some(paths) = batch.as_path_vec() {
            if paths.is_empty() {
                return;
            }

            let mut geometry: VertexBuffers<PathVertex, u32> = VertexBuffers::new();
            let mut fill_tesselator = FillTessellator::new();
            let mut stroke_tesselator = StrokeTessellator::new();

            for scene_path in paths.iter() {
                let mut builder = Path::builder();
                builder.begin(point(scene_path.start.x, scene_path.start.y));
                for path_command in scene_path.commands.iter() {
                    match path_command {
                        PathCommand::LineTo { to } => {
                            builder.line_to(point(to.x, to.y));
                        }
                        PathCommand::QuadraticBezierTo { control, to } => {
                            builder.quadratic_bezier_to(
                                point(control.x, control.y),
                                point(to.x, to.y),
                            );
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

                builder.end(!scene_path.open);
                let path = builder.build();

                if let Some(fill) = scene_path.fill {
                    let fill = Vec4::from_array(fill.into_linear().into());
                    fill_tesselator
                        .tessellate_path(
                            &path,
                            &FillOptions::default(),
                            &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| {
                                PathVertex {
                                    color: fill,
                                    position: vec2(vertex.position().x, vertex.position().y),
                                    ..Default::default()
                                }
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
            }

            self.geometry_buffer
                .upload(&geometry.vertices, &geometry.indices, queue);

            self.geometry_buffer.draw(render_pass);
        }
    }
}
