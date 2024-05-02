use std::collections::HashMap;

use etagere::{size2, AllocId, AtlasAllocator};
use glam::{vec2, Vec2, Vec4};
use shader::{InstancedGlyph, ShaderConstants};
use swash::{
    scale::{Render, ScaleContext, Source, StrikeWith},
    zeno::Format,
    FontRef, GlyphId,
};
use wgpu::{
    BindGroup, BindGroupLayout, BlendState, Buffer, BufferDescriptor, ColorTargetState,
    ColorWrites, Device, Extent3d, ImageCopyTexture, ImageDataLayout, Origin3d, Queue, RenderPass,
    RenderPipeline, ShaderModule, ShaderStages, Texture, TextureAspect, TextureFormat,
};

use crate::Drawable;

pub struct GlyphState {
    buffer: Buffer,
    atlas_texture: Texture,
    pub glyphs: Vec<InstancedGlyph>,
    bind_group: BindGroup,
    render_pipeline: RenderPipeline,

    glyph_lookup: HashMap<GlyphId, AllocId>,
    scale_context: ScaleContext,
    atlas_allocator: AtlasAllocator,
}

impl GlyphState {
    pub fn new(
        device: &Device,
        shader: &ShaderModule,
        swapchain_format: TextureFormat,
        universal_bind_group_layout: &BindGroupLayout,
    ) -> Self {
        let buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Glyph buffer"),
            size: std::mem::size_of::<InstancedGlyph>() as u64 * 100000,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let atlas_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Glyph atlas texture descriptor"),
            size: wgpu::Extent3d {
                width: 1000,
                height: 1000,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Glyph bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        let atlas_texture_view = atlas_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Glyph bind group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&atlas_texture_view),
                },
            ],
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Glyph Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout, &universal_bind_group_layout],
                push_constant_ranges: &[wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::all(),
                    range: 0..std::mem::size_of::<ShaderConstants>() as u32,
                }],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Glyph Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "glyph::glyph_vertex",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "glyph::glyph_fragment",
                targets: &[Some(ColorTargetState {
                    format: swapchain_format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self {
            buffer,
            atlas_texture,
            bind_group,
            glyphs: Vec::new(),
            render_pipeline,
            scale_context: ScaleContext::new(),
            atlas_allocator: AtlasAllocator::new(size2(1024, 1024)),
            glyph_lookup: HashMap::new(),
        }
    }

    pub fn add_glyph<'a, 'b: 'a>(
        &'b mut self,
        queue: &mut Queue,
        font_ref: FontRef<'a>,
        glyph: swash::GlyphId,
        bottom_left: Vec2,
        size: f32,
        color: Vec4,
    ) {
        // Create a font scaler for the given font and size
        let mut scaler = self
            .scale_context
            .builder(font_ref)
            .size(size)
            .hint(true)
            .build();

        // Compute fractional offset
        // TODO: Quantize this like the swash demo: https://github.com/dfrg/swash_demo/blob/master/src/comp/image_cache/glyph.rs#L325
        let offset = swash::zeno::Vector::new(bottom_left.x.fract(), bottom_left.y.fract());

        // Get or find atlas allocation
        let allocation_rectangle = if let Some(alloc_id) = self.glyph_lookup.get(&glyph) {
            self.atlas_allocator.get(*alloc_id)
        } else {
            let image = Render::new(&[
                Source::ColorOutline(0),
                Source::ColorBitmap(StrikeWith::BestFit),
                Source::Outline,
            ])
            // Select a subpixel format
            .format(Format::Subpixel)
            // Apply the fractional offset
            .offset(offset)
            // Render the image
            .render(&mut scaler, glyph)
            .expect("Could not render glyph into an image");

            if image.placement.width == 0 || image.placement.height == 0 {
                return;
            }

            let allocation = self
                .atlas_allocator
                .allocate(size2(
                    image.placement.width as i32,
                    image.placement.height as i32,
                ))
                .expect("Could not allocate glyph to atlas");

            self.glyph_lookup.insert(glyph, allocation.id);

            queue.write_texture(
                ImageCopyTexture {
                    texture: &self.atlas_texture,
                    mip_level: 0,
                    origin: Origin3d {
                        x: allocation.rectangle.min.x as u32,
                        y: allocation.rectangle.min.y as u32,
                        z: 0,
                    },
                    aspect: TextureAspect::All,
                },
                &image.data,
                ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * image.placement.width),
                    rows_per_image: Some(image.placement.height),
                },
                Extent3d {
                    width: image.placement.width,
                    height: image.placement.height,
                    depth_or_array_layers: 1,
                },
            );

            allocation.rectangle
        };

        // Add the glyph to instances
        self.glyphs.push(InstancedGlyph {
            bottom_left,
            atlas_top_left: vec2(
                allocation_rectangle.min.x as f32,
                allocation_rectangle.min.y as f32,
            ),
            atlas_size: vec2(
                allocation_rectangle.width() as f32,
                allocation_rectangle.height() as f32,
            ),
            _padding: Default::default(),
            color,
        });
    }

    pub fn clear(&mut self) {
        self.glyphs.clear();
    }
}

impl Drawable for GlyphState {
    fn draw<'b, 'a: 'b>(
        &'a self,
        queue: &Queue,
        render_pass: &mut RenderPass<'b>,
        constants: ShaderConstants,
        universal_bind_group: &'a BindGroup,
    ) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_push_constants(ShaderStages::all(), 0, bytemuck::cast_slice(&[constants]));
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.glyphs[..]));
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_bind_group(1, &universal_bind_group, &[]);
        render_pass.draw(0..6, 0..self.glyphs.len() as u32);
    }
}
