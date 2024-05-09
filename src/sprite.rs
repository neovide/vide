use std::{collections::HashMap, marker::PhantomData};

use etagere::{size2, AllocId, AtlasAllocator};
use glam::{vec2, Vec4};
use glamour::AsRaw;
use image::GenericImageView;
use rust_embed::RustEmbed;
use shader::{InstancedSprite, ShaderConstants, ShaderModules};
use wgpu::{BindGroupLayout, RenderPipeline, *};

use crate::{
    renderer::Drawable,
    scene::{Layer, Sprite},
    Renderer, ATLAS_SIZE,
};

pub struct SpriteState<A: RustEmbed> {
    buffer: Buffer,
    atlas_texture: Texture,
    bind_group_layout: BindGroupLayout,
    bind_group: BindGroup,
    render_pipeline: RenderPipeline,

    image_lookup: HashMap<String, AllocId>,
    atlas_allocator: AtlasAllocator,
    _assets: PhantomData<*const A>,
}

impl<A: RustEmbed> SpriteState<A> {
    pub fn upload_sprite(&mut self, queue: &Queue, sprite: &Sprite) -> InstancedSprite {
        let allocation_rectangle = if let Some(alloc_id) = self.image_lookup.get(&sprite.texture) {
            self.atlas_allocator.get(*alloc_id)
        } else {
            let image_file = A::get(&sprite.texture).unwrap();
            let image = image::load_from_memory(image_file.data.as_ref()).unwrap();
            let data = image.to_rgba8();
            let (image_width, image_height) = image.dimensions();

            let allocation = self
                .atlas_allocator
                .allocate(size2(image_width as i32, image_height as i32))
                .expect("Could not allocate glyph to atlas");

            self.image_lookup
                .insert(sprite.texture.clone(), allocation.id);

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
                &data,
                ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * image_width),
                    rows_per_image: Some(image_height),
                },
                Extent3d {
                    width: image_width,
                    height: image_height,
                    depth_or_array_layers: 1,
                },
            );

            allocation.rectangle
        };

        InstancedSprite {
            top_left: *sprite.top_left.as_raw(),
            size: *sprite.size.as_raw(),
            atlas_top_left: vec2(
                allocation_rectangle.min.x as f32,
                allocation_rectangle.min.y as f32,
            ),
            atlas_size: vec2(
                allocation_rectangle.width() as f32,
                allocation_rectangle.height() as f32,
            ),
            color: Vec4::from_array(sprite.color.into_linear().into()),
        }
    }
}

fn create_render_pipeline(
    device: &Device,
    universal_bind_group_layout: &BindGroupLayout,
    shaders: &ShaderModules,
    format: &TextureFormat,
    bind_group_layout: &BindGroupLayout,
) -> RenderPipeline {
    let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("Sprite Pipeline Layout"),
        bind_group_layouts: &[bind_group_layout, &universal_bind_group_layout],
        push_constant_ranges: &[PushConstantRange {
            stages: ShaderStages::all(),
            range: 0..std::mem::size_of::<ShaderConstants>() as u32,
        }],
    });

    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("Sprite Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: VertexState {
            module: shaders.get_vertex("sprite"),

            entry_point: "sprite_vertex",
            buffers: &[],
        },
        fragment: Some(FragmentState {
            module: shaders.get_fragment("sprite"),
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

impl<A: RustEmbed> Drawable for SpriteState<A> {
    fn new(
        Renderer {
            device,
            shaders,
            format,
            universal_bind_group_layout,
            ..
        }: &Renderer,
    ) -> Self {
        let buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Sprite buffer"),
            size: std::mem::size_of::<InstancedSprite>() as u64 * 100000,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let atlas_texture = device.create_texture(&TextureDescriptor {
            label: Some("Sprite atlas texture descriptor"),
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
            label: Some("Sprite bind group layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
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
            label: Some("Sprite bind group"),
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

        let render_pipeline = create_render_pipeline(
            device,
            universal_bind_group_layout,
            shaders,
            format,
            &bind_group_layout,
        );

        Self {
            buffer,
            atlas_texture,
            bind_group_layout,
            bind_group,
            render_pipeline,

            image_lookup: HashMap::new(),
            atlas_allocator: AtlasAllocator::new(size2(ATLAS_SIZE.x as i32, ATLAS_SIZE.y as i32)),
            _assets: PhantomData,
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
        let sprites: Vec<_> = layer
            .sprites
            .iter()
            .map(|sprite| self.upload_sprite(queue, sprite))
            .collect();

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_push_constants(ShaderStages::all(), 0, bytemuck::cast_slice(&[constants]));

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&sprites[..]));
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_bind_group(1, universal_bind_group, &[]);
        render_pass.draw(0..6, 0..sprites.len() as u32);
    }
}
