use glamour::Rect;
use std::collections::HashMap;
use wgpu::*;

use crate::{drawable::Drawable, PrimitiveBatch, Renderer, Resources, ShaderConstants};

pub(crate) struct DrawablePipeline {
    drawable: Box<dyn Drawable>,

    pub name: String,

    bind_group_layout: BindGroupLayout,
    bind_group: BindGroup,

    render_content_pipeline: Option<RenderPipeline>,
    render_mask_pipeline: Option<RenderPipeline>,
}

pub(crate) struct DrawableContext<'a> {
    pub queue: &'a Queue,
    pub universal_bind_group: &'a BindGroup,
}

pub(crate) struct RenderContentParams<'a> {
    pub constants: ShaderConstants,
    pub resources: &'a Resources,
    pub clip: Option<Rect<u32>>,
    pub batch: &'a PrimitiveBatch,
}

pub(crate) struct RenderDrawableParams<'a> {
    pub constants: ShaderConstants,
    pub resources: &'a Resources,
    pub clip: Option<Rect<u32>>,
    pub batch: &'a PrimitiveBatch,
}

impl DrawablePipeline {
    pub fn new<T: Drawable + 'static>(Renderer { device, .. }: &Renderer, drawable: T) -> Self {
        let drawable = Box::new(drawable);

        let name = drawable.name().to_string();

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some(&format!("{} bind group layout", &name)),
            entries: drawable
                .references()
                .iter()
                .filter_map(|reference| reference.layout())
                .enumerate()
                .map(|(index, mut layout)| {
                    layout.binding = index as u32;
                    layout
                })
                .collect::<Vec<_>>()
                .as_slice(),
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some(&format!("{} bind group", &name)),
            layout: &bind_group_layout,
            entries: drawable
                .references()
                .iter()
                .filter_map(|reference| reference.entry())
                .enumerate()
                .map(|(index, mut entry)| {
                    entry.binding = index as u32;
                    entry
                })
                .collect::<Vec<_>>()
                .as_slice(),
        });

        Self {
            drawable,
            name,
            bind_group_layout,
            bind_group,
            render_content_pipeline: None,
            render_mask_pipeline: None,
        }
    }

    fn try_create_pipelines(
        &mut self,
        device: &Device,
        shaders: &HashMap<String, ShaderModule>,
        format: &TextureFormat,
        universal_bind_group_layout: &BindGroupLayout,
    ) -> Result<(), String> {
        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some(&format!("{} Pipeline Layout", self.name)),
            bind_group_layouts: &[&self.bind_group_layout, universal_bind_group_layout],
            push_constant_ranges: &[PushConstantRange {
                stages: ShaderStages::VERTEX_FRAGMENT,
                range: 0..std::mem::size_of::<ShaderConstants>() as u32,
            }],
        });

        let vertex_buffer_layouts = self
            .drawable
            .references()
            .iter()
            .filter_map(|reference| reference.vertex())
            .collect::<Vec<_>>();
        let shader_module = shaders
            .get(&self.name)
            .ok_or_else(|| format!("Shader module not found for {}", &self.name))?;

        let vertex = VertexState {
            module: shader_module,
            entry_point: "vert",
            buffers: &vertex_buffer_layouts,
            compilation_options: Default::default(),
        };

        let targets = self.drawable.targets(*format);
        let fragment = Some(FragmentState {
            module: shader_module,
            entry_point: "frag",
            targets: &targets,
            compilation_options: Default::default(),
        });
        let primitive = PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: FrontFace::Ccw,
            cull_mode: None,
            unclipped_depth: false,
            polygon_mode: PolygonMode::Fill,
            conservative: false,
        };

        self.render_content_pipeline =
            Some(device.create_render_pipeline(&RenderPipelineDescriptor {
                label: Some(&format!("{} Content Pipeline", self.name)),
                layout: Some(&render_pipeline_layout),
                vertex: vertex.clone(),
                fragment: fragment.clone(),
                primitive,
                depth_stencil: None,
                multisample: MultisampleState {
                    count: 1,
                    ..Default::default()
                },
                multiview: None,
                cache: None,
            }));

        self.render_mask_pipeline =
            Some(device.create_render_pipeline(&RenderPipelineDescriptor {
                label: Some(&format!("{} Mask Pipeline", self.name)),
                layout: Some(&render_pipeline_layout),
                vertex: vertex.clone(),
                fragment: fragment.clone(),
                primitive,
                depth_stencil: None,
                multisample: MultisampleState {
                    count: 1,
                    ..Default::default()
                },
                multiview: None,
                cache: None,
            }));

        Ok(())
    }

    pub async fn create_pipeline(
        &mut self,
        device: &Device,
        shaders: &HashMap<String, ShaderModule>,
        format: &TextureFormat,
        universal_bind_group_layout: &BindGroupLayout,
    ) {
        device.push_error_scope(ErrorFilter::Validation);
        self.try_create_pipelines(device, shaders, format, universal_bind_group_layout)
            .ok();
        let validation_error = device.pop_error_scope().await;

        if validation_error.is_some() {
            self.render_content_pipeline = None;
            self.render_mask_pipeline = None;
        }
    }

    pub fn ready(&self) -> bool {
        self.render_content_pipeline.is_some() && self.render_mask_pipeline.is_some()
    }

    pub fn start_frame(&mut self) {
        self.drawable.start_frame();
    }

    pub fn has_work(&self, batch: &PrimitiveBatch) -> bool {
        self.drawable.has_work(batch)
    }

    pub fn requires_offscreen_copy(&self) -> bool {
        self.drawable.requires_offscreen_copy()
    }

    pub fn draw_content<'b, 'a: 'b>(
        &'a mut self,
        draw_context: &DrawableContext,
        render_pass: &mut RenderPass<'b>,
        render_params: &RenderContentParams<'a>,
    ) {
        render_pass.set_pipeline(self.render_content_pipeline.as_ref().unwrap());

        render_pass.set_push_constants(
            ShaderStages::VERTEX_FRAGMENT,
            0,
            bytemuck::cast_slice(&[render_params.constants]),
        );

        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_bind_group(1, draw_context.universal_bind_group, &[]);

        self.drawable.draw(
            draw_context.queue,
            render_pass,
            render_params.constants,
            render_params.resources,
            render_params.clip,
            render_params.batch,
        );
    }

    pub fn draw_mask<'b, 'a: 'b>(
        &'a mut self,
        draw_context: &DrawableContext,
        render_pass: &mut RenderPass<'b>,
        render_params: &RenderDrawableParams<'a>,
    ) {
        render_pass.set_pipeline(self.render_mask_pipeline.as_ref().unwrap());

        render_pass.set_push_constants(
            ShaderStages::VERTEX_FRAGMENT,
            0,
            bytemuck::cast_slice(&[render_params.constants]),
        );

        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_bind_group(1, draw_context.universal_bind_group, &[]);

        self.drawable.draw(
            draw_context.queue,
            render_pass,
            render_params.constants,
            render_params.resources,
            render_params.clip,
            render_params.batch,
        );
    }
}
