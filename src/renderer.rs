use futures::executor::block_on;
use glam::*;
use glamour::{AsRaw, Rect};
use wgpu::*;
use wgpu_profiler::{GpuProfiler, GpuProfilerSettings};

use crate::{
    default_drawables::{GlyphState, PathState, QuadState, SpriteState},
    drawable::{Drawable, DrawablePipeline},
    drawable_reference::ATLAS_SIZE,
    shader::{ShaderConstants, ShaderLoader, ShaderModules},
    LayerContents, Resources, Scene,
};

pub struct Renderer {
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
    pub shaders: ShaderModules,
    pub profiler: GpuProfiler,

    pub format: TextureFormat,
    pub width: u32,
    pub height: u32,

    pub offscreen_texture: ViewedTexture,
    pub mask_texture: ViewedTexture,
    pub blank_texture: ViewedTexture,

    pub sampler: Sampler,
    pub universal_bind_group_layout: BindGroupLayout,
    pub universal_content_bind_group: BindGroup,
    pub universal_mask_bind_group: BindGroup,
    drawables: Vec<DrawablePipeline>,

    pub(crate) shader_loader: ShaderLoader,
}

impl Renderer {
    // Creating some of the wgpu types requires async code
    pub async fn new(width: u32, height: u32, adapter: Adapter, format: TextureFormat) -> Self {
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    required_features: Features::PUSH_CONSTANTS
                        | Features::SPIRV_SHADER_PASSTHROUGH
                        | Features::VERTEX_WRITABLE_STORAGE
                        | Features::CLEAR_TEXTURE
                        | GpuProfiler::ALL_WGPU_TIMER_FEATURES,
                    required_limits: Limits {
                        max_push_constant_size: 256,
                        ..Default::default()
                    },
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let shader_loader = ShaderLoader::new();

        let shaders = shader_loader.load(&device);

        let offscreen_texture = ViewedTexture::new(&device, width, height, format, 1, "Offscreen");
        let mask_texture = ViewedTexture::new(&device, width, height, format, 1, "Mask");
        let blank_texture = ViewedTexture::new(&device, 1, 1, format, 1, "Blank");
        queue.write_texture(
            ImageCopyTexture {
                texture: &blank_texture.texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[255, 255, 255, 255],
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        let universal_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Universal bind group layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
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
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let universal_content_bind_group = create_bind_group(
            &device,
            &universal_bind_group_layout,
            &offscreen_texture.texture,
            &mask_texture.texture,
            &sampler,
        );
        let universal_mask_bind_group = create_bind_group(
            &device,
            &universal_bind_group_layout,
            &blank_texture.texture,
            &blank_texture.texture,
            &sampler,
        );
        let shaders = shaders.await;

        let profiler = GpuProfiler::new_with_tracy_client(
            GpuProfilerSettings::default(),
            adapter.get_info().backend,
            &device,
            &queue,
        )
        .expect("Could not create profiler");

        Self {
            adapter,
            device,
            queue,
            shaders,
            profiler,

            format,
            width,
            height,

            offscreen_texture,
            mask_texture,
            blank_texture,

            sampler,
            universal_bind_group_layout,
            universal_content_bind_group,
            universal_mask_bind_group,

            drawables: Vec::new(),

            shader_loader,
        }
    }

    pub async fn add_drawable<T: Drawable + 'static>(&mut self) {
        let drawable = T::new(self);
        let mut drawable_pipeline = DrawablePipeline::new(self, drawable);
        drawable_pipeline
            .create_pipeline(
                &self.device,
                &self.shaders,
                &self.format,
                &self.universal_bind_group_layout,
            )
            .await;
        self.drawables.push(drawable_pipeline);
    }

    pub async fn with_drawable<T: Drawable + 'static>(mut self) -> Self {
        self.add_drawable::<T>().await;
        self
    }

    pub async fn add_default_drawables(&mut self) {
        self.add_drawable::<QuadState>().await;
        self.add_drawable::<GlyphState>().await;
        self.add_drawable::<PathState>().await;
        self.add_drawable::<SpriteState>().await;
    }

    pub async fn with_default_drawables(mut self) -> Self {
        self.add_default_drawables().await;
        self
    }

    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        if new_width != 0 && new_height != 0 {
            self.width = new_width;
            self.height = new_height;
            self.offscreen_texture = ViewedTexture::new(
                &self.device,
                new_width,
                new_height,
                self.format,
                1,
                "Offscreen",
            );
            self.mask_texture =
                ViewedTexture::new(&self.device, new_width, new_height, self.format, 1, "Mask");

            self.queue.write_texture(
                ImageCopyTexture {
                    texture: &self.blank_texture.texture,
                    mip_level: 0,
                    origin: Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &[255, 255, 255, 255],
                ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4),
                    rows_per_image: Some(1),
                },
                Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
            );

            self.universal_content_bind_group = create_bind_group(
                &self.device,
                &self.universal_bind_group_layout,
                &self.offscreen_texture.texture,
                &self.mask_texture.texture,
                &self.sampler,
            );
            self.universal_mask_bind_group = create_bind_group(
                &self.device,
                &self.universal_bind_group_layout,
                &self.blank_texture.texture,
                &self.blank_texture.texture,
                &self.sampler,
            );
        }
    }

    pub fn render(&mut self, scene: &Scene, frame: &Texture) {
        profiling::scope!("Render Frame");
        if self.width == 0 || self.height == 0 {
            return;
        }

        if let Some(shaders) = self.shader_loader.try_reload(&self.device) {
            profiling::scope!("Reload Shaders");
            self.shaders = shaders;
            for drawable in &mut self.drawables {
                // TODO: A more sane block on behaviour, combined with try_reload, but not async unless needed
                block_on(drawable.create_pipeline(
                    &self.device,
                    &self.shaders,
                    &self.format,
                    &self.universal_bind_group_layout,
                ));
            }
        }

        let frame_view = frame.create_view(&Default::default());

        let constants = ShaderConstants {
            surface_size: vec2(self.width as f32, self.height as f32),
            atlas_size: ATLAS_SIZE.as_raw().as_vec2(),
        };

        for drawable in self.drawables.iter_mut() {
            drawable.start_frame();
        }

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        let mut first = true;
        for layer in scene.layers.iter() {
            profiling::scope!("Layer");

            self.draw_mask(
                layer.mask.as_ref(),
                &mut encoder,
                layer.clip,
                constants,
                &scene.resources,
            );

            self.draw_content(
                &layer.contents,
                &mut encoder,
                &mut first,
                frame,
                &frame_view,
                layer.clip,
                constants,
                &scene.resources,
            );
        }
        self.profiler.resolve_queries(&mut encoder);
        self.queue.submit(Some(encoder.finish()));
    }

    pub fn draw_mask(
        &mut self,
        mask_contents: Option<&LayerContents>,
        encoder: &mut CommandEncoder,
        clip: Option<Rect<u32>>,
        constants: ShaderConstants,
        resources: &Resources,
    ) {
        let mut mask_scope = self.profiler.scope("mask", encoder, &self.device);
        if let Some(mask_contents) = mask_contents {
            let mut render_pass = mask_scope.scoped_render_pass(
                "Mask",
                &self.device,
                RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: &self.mask_texture.view,
                        resolve_target: None,
                        ops: Operations::<Color> {
                            load: LoadOp::<_>::Clear(Color::TRANSPARENT),
                            store: StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    ..Default::default()
                },
            );

            for drawable in self.drawables.iter_mut() {
                profiling::scope!("drawable", &drawable.name);
                let mut drawable_scope = render_pass.scope(&drawable.name, &self.device);
                if !drawable.ready() {
                    continue;
                }

                if let Some(clip) = clip {
                    let x = clip.origin.x.min(self.width);
                    let y = clip.origin.y.min(self.height);
                    let w = clip.width().min(self.width - x);
                    let h = clip.height().min(self.height - y);
                    drawable_scope.set_scissor_rect(x, y, w, h);
                }

                drawable.draw_mask(
                    &self.queue,
                    &mut drawable_scope,
                    constants,
                    &self.universal_mask_bind_group,
                    resources,
                    clip,
                    mask_contents,
                );
            }
        } else {
            mask_scope.scoped_render_pass(
                "Clear Mask Texture",
                &self.device,
                RenderPassDescriptor {
                    label: Some("Clear Mask"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: &self.mask_texture.view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(Color {
                                a: 1.,
                                ..Default::default()
                            }),
                            store: StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                },
            );
        }
    }

    pub fn draw_content(
        &mut self,
        contents: &LayerContents,
        encoder: &mut CommandEncoder,
        first: &mut bool,
        frame: &Texture,
        frame_view: &TextureView,
        clip: Option<Rect<u32>>,
        constants: ShaderConstants,
        resources: &Resources,
    ) {
        let mut content_scope = self.profiler.scope("content", encoder, &self.device);

        for drawable in self.drawables.iter_mut() {
            if !drawable.has_work(contents) {
                continue;
            }

            // Either clear the offscreen texture or copy the previous layer to it
            if *first {
                content_scope.scoped_render_pass(
                    "Clear Offscreen Texture",
                    &self.device,
                    RenderPassDescriptor {
                        label: Some("Clear Offscreen Texture Pass"),
                        color_attachments: &[Some(RenderPassColorAttachment {
                            view: &self.offscreen_texture.view,
                            resolve_target: None,
                            ops: Operations {
                                load: LoadOp::Clear(Color::WHITE),
                                store: StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        occlusion_query_set: None,
                        timestamp_writes: None,
                    },
                );
            } else if drawable.needs_offscreen_copy() {
                let mut copy_scope = content_scope.scope("Copy Frame to Offscreen", &self.device);
                copy_scope.copy_texture_to_texture(
                    ImageCopyTexture {
                        texture: frame,
                        mip_level: 0,
                        origin: Origin3d::ZERO,
                        aspect: Default::default(),
                    },
                    ImageCopyTexture {
                        texture: &self.offscreen_texture.texture,
                        mip_level: 0,
                        origin: Origin3d::ZERO,
                        aspect: Default::default(),
                    },
                    Extent3d {
                        width: self.width,
                        height: self.height,
                        depth_or_array_layers: 1,
                    },
                );
            }

            // The first drawable should clear the output texture
            let attachment_op = if *first {
                Operations::<Color> {
                    load: LoadOp::<_>::Clear(Color::WHITE),
                    store: StoreOp::Store,
                }
            } else {
                Operations::<Color> {
                    load: LoadOp::<_>::Load,
                    store: StoreOp::Store,
                }
            };

            let mut render_pass = content_scope.scoped_render_pass(
                "Layer Render Pass",
                &self.device,
                RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: frame_view,
                        resolve_target: None,
                        ops: attachment_op,
                    })],
                    depth_stencil_attachment: None,
                    ..Default::default()
                },
            );

            profiling::scope!("drawable", &drawable.name);
            let mut drawable_scope = render_pass.scope(&drawable.name, &self.device);
            if !drawable.ready() {
                continue;
            }

            if let Some(clip) = clip {
                let x = clip.origin.x.min(self.width);
                let y = clip.origin.y.min(self.height);
                let w = clip.width().min(self.width - x);
                let h = clip.height().min(self.height - y);
                drawable_scope.set_scissor_rect(x, y, w, h);
            }

            drawable.draw_content(
                &self.queue,
                &mut drawable_scope,
                constants,
                &self.universal_content_bind_group,
                resources,
                clip,
                contents,
            );

            *first = false;
        }
    }

    pub fn watch_shaders<F: FnMut() + Send + 'static>(&mut self, shaders_changed: F) {
        self.shader_loader.watch(shaders_changed)
    }
}

pub struct ViewedTexture {
    texture: Texture,
    view: TextureView,
}

impl ViewedTexture {
    fn new(
        device: &Device,
        width: u32,
        height: u32,
        format: TextureFormat,
        samples: u32,
        label: &'static str,
    ) -> Self {
        let texture = device.create_texture(&TextureDescriptor {
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: samples,
            dimension: TextureDimension::D2,
            format,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            label: Some(label),
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());
        Self { texture, view }
    }
}

fn create_bind_group(
    device: &Device,
    bind_group_layout: &BindGroupLayout,
    offscreen_texture: &Texture,
    mask_texture: &Texture,
    sampler: &Sampler,
) -> BindGroup {
    let offscreen_texture_view = offscreen_texture.create_view(&TextureViewDescriptor::default());
    let mask_texture_view = mask_texture.create_view(&TextureViewDescriptor::default());

    device.create_bind_group(&BindGroupDescriptor {
        label: Some("Universal bind group"),
        layout: bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&offscreen_texture_view),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::TextureView(&mask_texture_view),
            },
            BindGroupEntry {
                binding: 2,
                resource: BindingResource::Sampler(sampler),
            },
        ],
    })
}
