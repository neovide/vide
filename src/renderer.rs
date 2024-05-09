use rust_embed::RustEmbed;
use wgpu::*;

use crate::{
    glyph::GlyphState, path::PathState, quad::QuadState, scene::Layer, sprite::SpriteState, Scene,
    ATLAS_SIZE,
};
use glam::*;
use shader::{ShaderConstants, ShaderLoader, ShaderModules};

pub trait Drawable {
    fn new(renderer: &Renderer) -> Self
    where
        Self: Sized;

    fn draw<'b, 'a: 'b>(
        &'a mut self,
        queue: &Queue,
        render_pass: &mut RenderPass<'b>,
        constants: ShaderConstants,
        universal_bind_group: &'a BindGroup,
        layer: &Layer,
    );

    fn reload(
        &mut self,
        device: &Device,
        shaders: &ShaderModules,
        format: &TextureFormat,
        universal_bind_group_layout: &BindGroupLayout,
    );
}

pub struct Renderer {
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
    pub shaders: ShaderModules,

    pub format: TextureFormat,
    pub width: u32,
    pub height: u32,

    pub offscreen_texture: Texture,
    pub multisampled_texture: Texture,
    pub sampler: Sampler,
    pub universal_bind_group_layout: BindGroupLayout,
    pub universal_bind_group: BindGroup,
    pub(crate) drawables: Vec<Box<dyn Drawable>>,

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
                        | Features::CLEAR_TEXTURE,
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

        let offscreen_texture =
            create_texture(&device, width, height, format, 1, "Offscreen Texture");
        let multisampled_texture =
            create_texture(&device, width, height, format, 4, "Output Texture");

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
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let universal_bind_group = create_bind_group(
            &device,
            &universal_bind_group_layout,
            &offscreen_texture,
            &sampler,
        );

        Self {
            adapter,
            device,
            queue,
            shaders,

            format,
            width,
            height,

            offscreen_texture,
            multisampled_texture,
            sampler,
            universal_bind_group_layout,
            universal_bind_group,

            drawables: Vec::new(),

            shader_loader,
        }
    }

    pub fn add_drawable<T: Drawable + 'static>(&mut self) {
        let drawable = T::new(self);
        self.drawables.push(Box::new(drawable));
    }

    pub fn with_drawable<T: Drawable + 'static>(mut self) -> Self {
        self.add_drawable::<T>();
        self
    }

    pub fn add_default_drawables<A: RustEmbed + 'static>(&mut self) {
        self.add_drawable::<QuadState>();
        self.add_drawable::<GlyphState>();
        self.add_drawable::<PathState>();
        self.add_drawable::<SpriteState<A>>();
    }

    pub fn with_default_drawables<A: RustEmbed + 'static>(mut self) -> Self {
        self.add_default_drawables::<A>();
        self
    }

    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        if new_width != 0 && new_height != 0 {
            self.width = new_width;
            self.height = new_height;
            self.offscreen_texture = create_texture(
                &self.device,
                new_width,
                new_height,
                self.format,
                1,
                "Offscreen Texture",
            );
            self.multisampled_texture = create_texture(
                &self.device,
                new_width,
                new_height,
                self.format,
                4,
                "Multisampled Texture",
            );

            self.universal_bind_group = create_bind_group(
                &self.device,
                &self.universal_bind_group_layout,
                &self.offscreen_texture,
                &self.sampler,
            );
        }
    }

    pub fn render(&mut self, scene: &Scene, frame: &Texture) {
        if self.width == 0 || self.height == 0 {
            return;
        }

        if let Some(shaders) = self.shader_loader.try_reload(&self.device) {
            self.shaders = shaders;
            for drawable in &mut self.drawables {
                drawable.reload(
                    &self.device,
                    &self.shaders,
                    &self.format,
                    &self.universal_bind_group_layout,
                )
            }
        }

        let frame_view = frame.create_view(&Default::default());
        let multisampled_view = self.multisampled_texture.create_view(&Default::default());

        let constants = ShaderConstants {
            surface_size: vec2(self.width as f32, self.height as f32),
            atlas_size: ATLAS_SIZE,
            clip: Vec4::ZERO,
        };

        let mut first = true;
        for layer in scene.layers.iter() {
            let mut encoder = self
                .device
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });
            for drawable in self.drawables.iter_mut() {
                // Either clear the offscreen texture or copy the previous layer to it
                if first {
                    encoder.clear_texture(
                        &self.offscreen_texture,
                        &ImageSubresourceRange {
                            aspect: TextureAspect::All,
                            base_mip_level: 0,
                            mip_level_count: None,
                            base_array_layer: 0,
                            array_layer_count: None,
                        },
                    );
                } else {
                    encoder.copy_texture_to_texture(
                        ImageCopyTexture {
                            texture: frame,
                            mip_level: 0,
                            origin: Origin3d::ZERO,
                            aspect: Default::default(),
                        },
                        ImageCopyTexture {
                            texture: &self.offscreen_texture,
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
                let attachment_op = if first {
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

                let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: &multisampled_view,
                        resolve_target: Some(&frame_view),
                        ops: attachment_op,
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                if let Some(clip) = layer.clip {
                    let x = clip.origin.x.min(self.width);
                    let y = clip.origin.y.min(self.height);
                    let w = clip.width().min(self.width - x);
                    let h = clip.height().min(self.height - y);
                    render_pass.set_scissor_rect(x, y, w, h);
                }

                drawable.draw(
                    &self.queue,
                    &mut render_pass,
                    constants,
                    &self.universal_bind_group,
                    layer,
                );

                first = false;
            }
            self.queue.submit(std::iter::once(encoder.finish()));
        }
    }

    pub fn watch_shaders<F: FnMut() + Send + 'static>(&mut self, shaders_changed: F) {
        self.shader_loader.watch(shaders_changed)
    }
}

fn create_texture(
    device: &Device,
    width: u32,
    height: u32,
    format: TextureFormat,
    samples: u32,
    label: &'static str,
) -> Texture {
    device.create_texture(&TextureDescriptor {
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
    })
}

fn create_bind_group(
    device: &Device,
    bind_group_layout: &BindGroupLayout,
    offscreen_texture: &Texture,
    sampler: &Sampler,
) -> BindGroup {
    let offscreen_texture_view = offscreen_texture.create_view(&TextureViewDescriptor::default());

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
                resource: BindingResource::Sampler(sampler),
            },
        ],
    })
}
