use egui::FontDefinitions;
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use rust_embed::*;
use shader::ShaderConstants;
use winit::{
    window::Window,
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};

use super::RepaintSignaler;

#[derive(RustEmbed)]
#[folder = "spirv"]
struct Asset;

pub struct GraphicsState {
    instance: wgpu::Instance,
    surface: Option<wgpu::Surface>,
    surface_config: wgpu::SurfaceConfiguration,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
}

impl GraphicsState {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::Backends::VULKAN);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::PUSH_CONSTANTS | wgpu::Features::SPIRV_SHADER_PASSTHROUGH,
                limits: wgpu::Limits {
                    max_push_constant_size: 256,
                    ..Default::default()
                },
                label: None,
            },
            None,
        ).await.unwrap();

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::all(),
                    range: 0..std::mem::size_of::<ShaderConstants>() as u32,
                }],
            });

        let preferred_format = surface.get_preferred_format(&adapter).unwrap();

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: preferred_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::util::make_spirv(&Asset::get("shader.spv").expect("Could not load shader")),
        });

        surface.configure(&device, &surface_config);

        let mut platform = Platform::new(PlatformDescriptor {
            physical_width: size.width as u32,
            physical_height: size.height as u32,
            scale_factor: window.scale_factor(),
            font_definitions: FontDefinitions::default(),
            style: Default::default(),
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vertex",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fragment",
                targets: &[wgpu::ColorTargetState {
                    format: preferred_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                clamp_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1, // 2.
                mask: !0, // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
        });

        Self {
            instance,
            surface: Some(surface),
            surface_config,
            adapter,
            device,
            queue,
            render_pipeline,
        }
    }

    pub fn render(&mut self, constants: ShaderConstants) -> Result<(), wgpu::SurfaceError> {
        if let Some(surface) = &mut self.surface {
            let frame = surface.get_current_texture()?;
            let frame_view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

            let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[
                        wgpu::RenderPassColorAttachment {
                            view: &frame_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                                store: true,
                            }
                        }
                    ],
                    depth_stencil_attachment: None,
                });

                render_pass.set_pipeline(&self.render_pipeline); // 2.
                render_pass.set_push_constants(wgpu::ShaderStages::all(), 0, unsafe {
                    ::std::slice::from_raw_parts((&constants as *const ShaderConstants) as *const u8, ::std::mem::size_of::<ShaderConstants>())
                });
                render_pass.draw(0..3, 0..1); // 3.
            }

            // submit will accept anything that implements IntoIter
            self.queue.submit(std::iter::once(encoder.finish()));
            frame.present();
        }

        Ok(())
    }

    pub fn handle_event<F>(&mut self, window: &Window, event: &Event<()>, control_flow: &mut ControlFlow, construct_constants: F)
    where F: FnOnce() -> ShaderConstants {
        match event {
            Event::MainEventsCleared => {
                window.request_redraw();
            },
            Event::Resumed => {
                let surface = unsafe { self.instance.create_surface(window) };
                self.surface_config.format = surface.get_preferred_format(&self.adapter).unwrap();
                surface.configure(&self.device, &self.surface_config);
                self.surface = Some(surface);
            },
            Event::Suspended => {
                self.surface = None;
            },
            // If doesn't resize properly on scale change, also handle ScaleFactorChanged using
            // new_inner_size
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                if new_size.width != 0 && new_size.height != 0 {
                    self.surface_config.width = new_size.width;
                    self.surface_config.height = new_size.height;

                    if let Some(surface) = &self.surface {
                        surface.configure(&self.device, &self.surface_config);
                    }
                }
            },
            Event::RedrawRequested(_) => {
                if let Err(render_error) = self.render(construct_constants()) {
                    eprintln!("Render error: {:?}", render_error);
                    match render_error {
                        wgpu::SurfaceError::Lost => {
                            if let Some(surface) = &self.surface {
                                surface.configure(&self.device, &self.surface_config);
                            }
                        },
                        wgpu::SurfaceError::OutOfMemory => {
                            eprintln!("Out of memory");
                            *control_flow = ControlFlow::Exit;
                        },
                        _ => {},
                    }
                }
            },
            _ => {} 
        }
    }
}
