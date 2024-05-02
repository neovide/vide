mod font;
mod glyph;
mod quad;
mod scene;
mod shape;

use glam::{vec2, Vec2, Vec4};
use rust_embed::*;
use shader::{InstancedQuad, ShaderConstants};
use swash::FontRef;
use wgpu::{
    util, Adapter, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Color,
    CommandEncoderDescriptor, Device, DeviceDescriptor, Extent3d, Features, ImageCopyTexture,
    Instance, Limits, LoadOp, Operations, Origin3d, PowerPreference, PresentMode, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, Sampler,
    SamplerBindingType, ShaderModuleDescriptor, ShaderStages, Surface, SurfaceConfiguration,
    SurfaceError, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType,
    TextureUsages, TextureViewDescriptor, TextureViewDimension,
};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
    window::Window,
};

pub use crate::scene::Scene;
use crate::{font::Font, glyph::GlyphState, quad::QuadState, shape::TextShapingState};

pub const ATLAS_SIZE: Vec2 = vec2(1024., 1024.);

#[derive(RustEmbed)]
#[folder = "spirv"]
struct Asset;

pub trait Drawable {
    fn draw<'b, 'a: 'b>(
        &'a self,
        queue: &Queue,
        render_pass: &mut wgpu::RenderPass<'b>,
        constants: ShaderConstants,
        universal_bind_group: &'a BindGroup,
    );
}

pub struct Renderer {
    instance: Instance,
    surface: Option<Surface>,
    surface_config: SurfaceConfiguration,
    adapter: Adapter,
    device: Device,
    queue: Queue,

    offscreen_texture: Texture,
    sampler: Sampler,
    universal_bind_group_layout: BindGroupLayout,
    universal_bind_group: BindGroup,
    clear_color: Vec4,
    quad_state: QuadState,
    glyph_state: GlyphState,
    text_shaping_state: TextShapingState,
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

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
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

        let quad_state = QuadState::new(&device, &shader, swapchain_format);
        let glyph_state = GlyphState::new(
            &device,
            &shader,
            swapchain_format,
            &universal_bind_group_layout,
        );
        let text_shaping_state = TextShapingState::new();

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
            clear_color: Vec4::ONE,
            quad_state,
            glyph_state,
            text_shaping_state,
        }
    }

    pub fn render(&mut self) -> Result<(), SurfaceError> {
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

            let drawables = [&self.quad_state as &dyn Drawable, &self.glyph_state];
            for (index, drawable) in drawables.iter().enumerate() {
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

                // The first drawable should clear the output textures
                let attachment_op = if index == 0 {
                    Operations::<Color> {
                        load: LoadOp::<_>::Clear(Color {
                            r: self.clear_color.x as f64,
                            g: self.clear_color.y as f64,
                            b: self.clear_color.z as f64,
                            a: self.clear_color.w as f64,
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

                drawable.draw(
                    &self.queue,
                    &mut render_pass,
                    constants,
                    &self.universal_bind_group,
                );
            }

            self.queue.submit(std::iter::once(encoder.finish()));
            frame.present();
        }

        Ok(())
    }

    pub fn clear(&mut self, color: Vec4) {
        self.clear_color = color;

        self.quad_state.clear();
        self.glyph_state.clear();
    }

    pub fn add_quad(&mut self, top_left: Vec2, size: Vec2, color: Vec4) {
        self.quad_state.quads.push(InstancedQuad {
            top_left,
            size,
            color,
        });
    }

    pub fn add_glyph<'a, 'b: 'a>(
        &'b mut self,
        font_ref: FontRef<'a>,
        glyph: swash::GlyphId,
        bottom_left: Vec2,
        size: f32,
        color: Vec4,
    ) {
        self.glyph_state
            .add_glyph(&mut self.queue, font_ref, glyph, bottom_left, size, color);
    }

    pub fn add_text<'a, 'b: 'a>(
        &'b mut self,
        font_ref: FontRef<'a>,
        text: &str,
        bottom_left: Vec2,
        size: f32,
        color: Vec4,
    ) {
        self.text_shaping_state.add_text(
            &mut self.queue,
            &mut self.glyph_state,
            font_ref,
            text,
            bottom_left,
            size,
            color,
        );
    }

    pub fn draw_scene(&mut self, scene: &Scene, window: &Window) {
        window.set_inner_size(PhysicalSize::new(
            scene.window_size.x as u32,
            scene.window_size.y as u32,
        ));

        self.clear(scene.background_color);

        for quad in scene.quads.iter() {
            self.add_quad(quad.top_left, quad.size, quad.color);
        }

        let font = Font::from_name(&scene.font_name).unwrap();
        let font_ref = font.as_ref().unwrap();

        for glyph in scene.glyphs.iter() {
            self.add_glyph(
                font_ref,
                font_ref.charmap().map(glyph.character),
                glyph.bottom_left,
                glyph.size,
                glyph.color,
            );
        }

        for text in scene.texts.iter() {
            self.add_text(
                font_ref,
                &text.text,
                text.bottom_left,
                text.size,
                text.color,
            );
        }
    }

    pub fn handle_event(
        &mut self,
        window: &Window,
        event: &Event<()>,
        control_flow: &mut ControlFlow,
    ) {
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
            // If doesn't resize properly on scale change, also handle ScaleFactorChanged using
            // new_inner_size
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
            Event::RedrawRequested(_) => {
                if let Err(render_error) = self.render() {
                    eprintln!("Render error: {:?}", render_error);
                    match render_error {
                        SurfaceError::Lost => {
                            if let Some(surface) = &self.surface {
                                surface.configure(&self.device, &self.surface_config);
                            }
                        }
                        SurfaceError::OutOfMemory => {
                            eprintln!("Out of memory");
                            *control_flow = ControlFlow::Exit;
                        }
                        _ => {}
                    }
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
    let offscreen_texture_view =
        offscreen_texture.create_view(&wgpu::TextureViewDescriptor::default());

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
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    })
}
