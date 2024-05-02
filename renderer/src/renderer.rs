use glam::{vec2, Vec4};
use wgpu::*;

use shader::ShaderConstants;
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    window::Window,
};

use crate::{glyph::GlyphState, quad::QuadState, scene::Layer, Asset, Scene, ATLAS_SIZE};

pub trait Drawable {
    fn draw<'b, 'a: 'b>(
        &'a mut self,
        queue: &Queue,
        render_pass: &mut RenderPass<'b>,
        constants: ShaderConstants,
        universal_bind_group: &'a BindGroup,
        layer: &Layer,
    );
}

pub struct Renderer {
    instance: Instance,
    surface: Option<Surface>,
    surface_config: SurfaceConfiguration,
    adapter: Adapter,
    device: Device,
    pub(crate) queue: Queue,

    offscreen_texture: Texture,
    sampler: Sampler,
    universal_bind_group_layout: BindGroupLayout,
    universal_bind_group: BindGroup,

    pub(crate) quad_state: QuadState,
    pub(crate) glyph_state: GlyphState,
}

impl Renderer {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        let instance = Instance::default();
        let surface = unsafe { instance.create_surface(window) }.unwrap();
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
                    features: Features::PUSH_CONSTANTS
                        | Features::SPIRV_SHADER_PASSTHROUGH
                        | Features::VERTEX_WRITABLE_STORAGE,
                    limits: Limits {
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
        };

        surface.configure(&device, &surface_config);

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shader"),
            source: util::make_spirv(&Asset::get("shader.spv").expect("Could not load shader")),
        });

        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        let offscreen_texture = create_offscreen_texture(&device, &size, swapchain_format);

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

        let quad_state = QuadState::new(
            &device,
            &shader,
            swapchain_format,
            &universal_bind_group_layout,
        );
        let glyph_state = GlyphState::new(
            &device,
            &shader,
            swapchain_format,
            &universal_bind_group_layout,
        );

        Self {
            instance,
            surface: Some(surface),
            surface_config,
            adapter,
            device,
            queue,

            offscreen_texture,
            universal_bind_group_layout,
            universal_bind_group,
            sampler,

            quad_state,
            glyph_state,
        }
    }

    fn render(&mut self, scene: &Scene) -> Result<(), SurfaceError> {
        if self.surface_config.width == 0 || self.surface_config.height == 0 {
            return Ok(());
        }

        if let Some(surface) = &mut self.surface {
            let frame = surface.get_current_texture()?;
            let frame_view = frame.texture.create_view(&TextureViewDescriptor::default());

            let mut encoder = self
                .device
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

            let constants = ShaderConstants {
                surface_size: vec2(
                    self.surface_config.width as f32,
                    self.surface_config.height as f32,
                ),
                atlas_size: ATLAS_SIZE,
            };

            let mut first = true;
            for layer in scene.layers.iter() {
                Self::draw_drawable(
                    first,
                    &mut self.quad_state,
                    &mut encoder,
                    &self.queue,
                    &frame,
                    &frame_view,
                    &self.offscreen_texture,
                    &self.surface_config,
                    &layer,
                    constants,
                    &self.universal_bind_group,
                );

                Self::draw_drawable(
                    false,
                    &mut self.glyph_state,
                    &mut encoder,
                    &self.queue,
                    &frame,
                    &frame_view,
                    &self.offscreen_texture,
                    &self.surface_config,
                    &layer,
                    constants,
                    &self.universal_bind_group,
                );

                first = false;
            }

            self.queue.submit(std::iter::once(encoder.finish()));
            frame.present();
        }

        Ok(())
    }

    fn draw_drawable(
        first: bool,
        drawable: &mut impl Drawable,
        encoder: &mut CommandEncoder,
        queue: &Queue,
        frame: &SurfaceTexture,
        frame_view: &TextureView,
        offscreen_texture: &Texture,
        surface_config: &SurfaceConfiguration,
        layer: &Layer,
        constants: ShaderConstants,
        universal_bind_group: &BindGroup,
    ) {
        encoder.copy_texture_to_texture(
            ImageCopyTexture {
                texture: &frame.texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: Default::default(),
            },
            ImageCopyTexture {
                texture: offscreen_texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: Default::default(),
            },
            Extent3d {
                width: surface_config.width,
                height: surface_config.height,
                depth_or_array_layers: 1,
            },
        );

        // The first drawable should clear the output textures
        let attachment_op = if first {
            let clear_color = layer.background_color.unwrap_or(Vec4::ONE);
            Operations::<Color> {
                load: LoadOp::<_>::Clear(Color {
                    r: clear_color.x as f64,
                    g: clear_color.y as f64,
                    b: clear_color.z as f64,
                    a: clear_color.w as f64,
                }),
                store: true,
            }
        } else {
            Operations::<Color> {
                load: LoadOp::<_>::Load,
                store: true,
            }
        };

        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &frame_view,
                resolve_target: None,
                ops: attachment_op,
            })],
            depth_stencil_attachment: None,
        });

        if let Some(clip) = layer.clip {
            render_pass.set_scissor_rect(
                clip.x as u32,
                clip.y as u32,
                clip.z as u32,
                clip.w as u32,
            );
        }

        drawable.draw(
            &queue,
            &mut render_pass,
            constants,
            &universal_bind_group,
            &layer,
        );
    }

    pub fn draw_scene(&mut self, scene: &Scene) -> bool {
        if let Err(render_error) = self.render(scene) {
            eprintln!("Render error: {:?}", render_error);
            match render_error {
                SurfaceError::Lost => {
                    if let Some(surface) = &self.surface {
                        surface.configure(&self.device, &self.surface_config);
                    }
                    true
                }
                SurfaceError::OutOfMemory => {
                    eprintln!("Out of memory");
                    false
                }
                _ => true,
            }
        } else {
            true
        }
    }

    pub fn handle_event(&mut self, window: &Window, event: &Event<()>) {
        match event {
            Event::Resumed => {
                let surface = unsafe { self.instance.create_surface(window) }.unwrap();

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

                    self.offscreen_texture = create_offscreen_texture(
                        &self.device,
                        &new_size,
                        self.surface_config.format,
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
}

fn create_offscreen_texture(
    device: &Device,
    size: &PhysicalSize<u32>,
    format: TextureFormat,
) -> Texture {
    device.create_texture(&TextureDescriptor {
        size: Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        label: Some(&format!("Offscreen Texture")),
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
