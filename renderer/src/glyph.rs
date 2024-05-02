use std::{collections::HashMap, sync::Arc};

use etagere::{size2, AllocId, AtlasAllocator};
use glam::{vec2, Vec2, Vec4};
use ordered_float::OrderedFloat;
use shader::{InstancedGlyph, ShaderConstants};
use swash::{
    scale::{Render, ScaleContext, Source, StrikeWith},
    shape::{cluster::Glyph, ShapeContext},
    zeno::{Format, Placement, Vector},
    CacheKey, FontRef, GlyphId,
};
use wgpu::*;

use crate::{font::Font, renderer::Drawable, scene::Layer, ATLAS_SIZE};

pub struct GlyphState {
    buffer: Buffer,
    atlas_texture: Texture,
    bind_group: BindGroup,
    render_pipeline: RenderPipeline,

    scale_context: ScaleContext,
    shaping_context: ShapeContext,
    glyph_lookup: HashMap<GlyphKey, (Placement, AllocId)>,
    shaped_text_lookup: HashMap<ShapeKey, Vec<Glyph>>,
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
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let atlas_texture = device.create_texture(&TextureDescriptor {
            label: Some("Glyph atlas texture descriptor"),
            size: Extent3d {
                width: ATLAS_SIZE.x as u32,
                height: ATLAS_SIZE.y as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Glyph bind group layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        let atlas_texture_view = atlas_texture.create_view(&TextureViewDescriptor::default());

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Glyph bind group"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&atlas_texture_view),
                },
            ],
        });

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Glyph Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout, &universal_bind_group_layout],
            push_constant_ranges: &[PushConstantRange {
                stages: ShaderStages::all(),
                range: 0..std::mem::size_of::<ShaderConstants>() as u32,
            }],
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Glyph Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "glyph::glyph_vertex",
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "glyph::glyph_fragment",
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
            atlas_texture,
            bind_group,
            render_pipeline,

            scale_context: ScaleContext::new(),
            shaping_context: ShapeContext::new(),
            atlas_allocator: AtlasAllocator::new(size2(ATLAS_SIZE.x as i32, ATLAS_SIZE.y as i32)),
            glyph_lookup: HashMap::new(),
            shaped_text_lookup: HashMap::new(),
        }
    }

    fn prepare_glyph<'a, 'b: 'a>(
        &'b mut self,
        queue: &Queue,
        font_name: &str,
        font_ref: FontRef<'a>,
        glyph: swash::GlyphId,
        bottom_left: Vec2,
        size: f32,
        color: Vec4,
    ) -> Option<InstancedGlyph> {
        // Create a font scaler for the given font and size
        let mut scaler = self
            .scale_context
            .builder(font_ref)
            .size(size)
            .hint(true)
            .build();

        let glyph_key = GlyphKey::new(font_name, glyph, size, bottom_left);

        // Get or find atlas allocation
        let (placement, allocation_rectangle) =
            if let Some((placement, alloc_id)) = self.glyph_lookup.get(&glyph_key) {
                (*placement, self.atlas_allocator.get(*alloc_id))
            } else {
                let image = Render::new(&[
                    Source::ColorOutline(0),
                    Source::ColorBitmap(StrikeWith::BestFit),
                    Source::Outline,
                ])
                // Select a subpixel format
                .format(Format::Subpixel)
                // Apply the fractional offset
                .offset(glyph_key.quantized_offset())
                // Render the image
                .render(&mut scaler, glyph)
                .expect("Could not render glyph into an image");

                if image.placement.width == 0 || image.placement.height == 0 {
                    return None;
                }

                let allocation = self
                    .atlas_allocator
                    .allocate(size2(
                        image.placement.width as i32,
                        image.placement.height as i32,
                    ))
                    .expect("Could not allocate glyph to atlas");

                self.glyph_lookup
                    .insert(glyph_key, (image.placement, allocation.id));

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
                        bytes_per_row: Some(4 * image.placement.width as u32),
                        rows_per_image: Some(image.placement.height as u32),
                    },
                    Extent3d {
                        width: image.placement.width as u32,
                        height: image.placement.height as u32,
                        depth_or_array_layers: 1,
                    },
                );

                (image.placement, allocation.rectangle)
            };

        // Add the glyph to instances
        Some(InstancedGlyph {
            bottom_left: bottom_left
                + vec2(
                    placement.left as f32,
                    placement.height as f32 - placement.top as f32,
                ),
            atlas_top_left: vec2(
                allocation_rectangle.min.x as f32,
                allocation_rectangle.min.y as f32,
            ),
            atlas_size: vec2(placement.width as f32, placement.height as f32),
            _padding: Default::default(),
            color,
        })
    }

    pub fn shape_and_rasterize_text<'a, 'b: 'a>(
        &mut self,
        queue: &Queue,
        font_name: &str,
        font_ref: FontRef<'a>,
        text: &str,
        bottom_left: Vec2,
        size: f32,
        color: Vec4,
    ) -> Vec<InstancedGlyph> {
        let key = ShapeKey::new(Arc::from(text), font_ref, size.into());

        let mut shaper = self.shaping_context.builder(font_ref).size(size).build();
        let glyphs = self
            .shaped_text_lookup
            .entry(key)
            .or_insert_with(|| {
                shaper.add_str(text);

                let mut glyphs = Vec::new();

                shaper.shape_with(|cluster| {
                    for glyph in cluster.glyphs {
                        glyphs.push(*glyph);
                    }
                });

                glyphs
            })
            .clone();

        let mut current_x = 0.;
        glyphs
            .iter()
            .filter_map(|glyph| {
                let instance = self.prepare_glyph(
                    queue,
                    font_name,
                    font_ref,
                    glyph.id,
                    bottom_left + vec2(current_x + glyph.x, -glyph.y),
                    size,
                    color,
                );
                current_x += glyph.advance;
                instance
            })
            .collect()
    }
}

impl Drawable for GlyphState {
    fn draw<'b, 'a: 'b>(
        &'a mut self,
        queue: &Queue,
        render_pass: &mut RenderPass<'b>,
        constants: ShaderConstants,
        universal_bind_group: &'a BindGroup,
        layer: &Layer,
    ) {
        let font = Font::from_name(&layer.font_name).unwrap();
        let font_ref = font.as_ref().unwrap();

        let glyphs: Vec<_> = layer
            .texts
            .iter()
            .map(|text| {
                self.shape_and_rasterize_text(
                    queue,
                    &layer.font_name,
                    font_ref,
                    text.text.as_ref(),
                    text.bottom_left,
                    text.size,
                    text.color,
                )
                .into_iter()
            })
            .flatten()
            .collect();

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_push_constants(ShaderStages::all(), 0, bytemuck::cast_slice(&[constants]));

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&glyphs[..]));
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_bind_group(1, &universal_bind_group, &[]);
        render_pass.draw(0..6, 0..glyphs.len() as u32);
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum SubpixelOffset {
    Zero,
    Quarter,
    Half,
    ThreeQuarters,
}

impl SubpixelOffset {
    fn quantize(value: f32) -> Self {
        let value = value.fract();
        if value < 0.125 {
            Self::Zero
        } else if value < 0.375 {
            Self::Quarter
        } else if value < 0.625 {
            Self::Half
        } else if value < 0.875 {
            Self::ThreeQuarters
        } else {
            Self::Zero
        }
    }

    fn to_f32(&self) -> f32 {
        match self {
            Self::Zero => 0.0,
            Self::Quarter => 0.25,
            Self::Half => 0.5,
            Self::ThreeQuarters => 0.75,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct GlyphKey {
    glyph: GlyphId,
    font_name: Arc<str>,
    size: OrderedFloat<f32>,
    x_offset: SubpixelOffset,
    y_offset: SubpixelOffset,
}

impl GlyphKey {
    fn new(font_name: &str, glyph: GlyphId, size: f32, offset: Vec2) -> Self {
        let size = size.into();
        let x_offset = SubpixelOffset::quantize(offset.x);
        let y_offset = SubpixelOffset::quantize(offset.y);
        Self {
            glyph,
            font_name: Arc::from(font_name),
            size,
            x_offset,
            y_offset,
        }
    }

    fn quantized_offset(&self) -> Vector {
        Vector::new(self.x_offset.to_f32(), self.y_offset.to_f32())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ShapeKey {
    text: Arc<str>,
    size: OrderedFloat<f32>,
    font_cache_key: CacheKey,
}

impl ShapeKey {
    fn new(text: Arc<str>, font_ref: FontRef, size: f32) -> Self {
        let font_cache_key = font_ref.key;
        let size = size.into();
        Self {
            text,
            size,
            font_cache_key,
        }
    }
}
