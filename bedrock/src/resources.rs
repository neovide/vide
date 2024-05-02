use glam::{vec2, Vec4};
use shader::ShaderConstants;
use wgpu::*;
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    window::Window,
};

use crate::{renderer::Drawable, Asset, Scene, ATLAS_SIZE};

pub struct Resources<'a> {
    pub window: &'a Window,
    pub instance: Instance,
    pub surface: Option<Surface<'a>>,
    pub surface_config: SurfaceConfiguration,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
    pub shader: ShaderModule,
    pub swapchain_format: TextureFormat,

    pub offscreen_texture: Texture,
    pub multisampled_texture: Texture,
    pub sampler: Sampler,
    pub universal_bind_group_layout: BindGroupLayout,
    pub universal_bind_group: BindGroup,
}

impl<'a> Resources<'a> {
    pub async fn new(window: &'a Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        let instance = Instance::default();
        let surface = instance.create_surface(window).unwrap();
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

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

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
            format: swapchain_format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &surface_config);

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shader"),
            source: util::make_spirv(
                &Asset::get("shader.spv")
                    .expect("Could not load shader")
                    .data,
            ),
        });

        let offscreen_texture =
            create_texture(&device, &size, swapchain_format, 1, "Offscreen Texture");
        let multisampled_texture =
            create_texture(&device, &size, swapchain_format, 4, "Output Texture");

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
            window,
            instance,
            surface: Some(surface),
            surface_config,
            adapter,
            device,
            queue,
            shader,
            swapchain_format,

            offscreen_texture,
            multisampled_texture,
            sampler,
            universal_bind_group_layout,
            universal_bind_group,
        }
    }

    pub fn handle_event(&mut self, window: &'a Window, event: &Event<()>) {
        match event {
            Event::Resumed => {
                let surface = self.instance.create_surface(window).unwrap();

                let swapchain_capabilities = surface.get_capabilities(&self.adapter);
                let swapchain_format = swapchain_capabilities.formats[0];
                self.surface_config.format = swapchain_format;
                self.surface_config.alpha_mode = swapchain_capabilities.alpha_modes[0];
                surface.configure(&self.device, &self.surface_config);
                self.surface = Some(surface);
            }
            Event::Suspended => {
                self.surface = None;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                self.surface_config.width = new_size.width;
                self.surface_config.height = new_size.height;

                if new_size.width != 0 && new_size.height != 0 {
                    if let Some(surface) = &self.surface {
                        surface.configure(&self.device, &self.surface_config);
                    }

                    self.offscreen_texture = create_texture(
                        &self.device,
                        &new_size,
                        self.surface_config.format,
                        1,
                        "Offscreen Texture",
                    );
                    self.multisampled_texture = create_texture(
                        &self.device,
                        &new_size,
                        self.surface_config.format,
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
            _ => {}
        };
    }

    pub fn render(
        &mut self,
        scene: &Scene,
        drawables: &mut [Box<dyn Drawable>],
    ) -> Result<(), SurfaceError> {
        if self.surface_config.width == 0 || self.surface_config.height == 0 {
            return Ok(());
        }

        if let Some(surface) = &mut self.surface {
            let frame = surface.get_current_texture()?;
            let frame_view = frame.texture.create_view(&Default::default());
            let multisampled_view = self.multisampled_texture.create_view(&Default::default());

            let constants = ShaderConstants {
                surface_size: vec2(
                    self.surface_config.width as f32,
                    self.surface_config.height as f32,
                ),
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
                for drawable in drawables.iter_mut() {
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
                                texture: &frame.texture,
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
                                width: self.surface_config.width,
                                height: self.surface_config.height,
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
                        render_pass.set_scissor_rect(
                            clip.x.max(0.0) as u32,
                            clip.y.max(0.0) as u32,
                            (clip.z as u32).min(self.surface_config.width),
                            (clip.w as u32).min(self.surface_config.height),
                        );
                    }

                    drawable.draw(
                        &self.queue,
                        &mut render_pass,
                        constants,
                        &self.universal_bind_group,
                        &layer,
                    );

                    first = false;
                }
                self.queue.submit(std::iter::once(encoder.finish()));
            }

            frame.present();
        }

        Ok(())
    }
}

fn create_texture(
    device: &Device,
    size: &PhysicalSize<u32>,
    format: TextureFormat,
    samples: u32,
    label: &'static str,
) -> Texture {
    device.create_texture(&TextureDescriptor {
        size: Extent3d {
            width: size.width,
            height: size.height,
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
