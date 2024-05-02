use std::time::Instant;
use std::sync::Arc;

use chrono::Timelike;
use glam::{vec2, vec4};
use rust_embed::*;
use shader::{ShaderConstants, InstancedQuad};
use wgpu::util::DeviceExt;
use winit::{
    window::Window,
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};

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

    quad_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
}

impl GraphicsState {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        let instance = wgpu::Instance::default();
        let surface = unsafe { instance.create_surface(window) }.unwrap();
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::PUSH_CONSTANTS 
                    | wgpu::Features::SPIRV_SHADER_PASSTHROUGH
                    | wgpu::Features::VERTEX_WRITABLE_STORAGE,
                limits: wgpu::Limits {
                    max_push_constant_size: 256,
                    ..Default::default()
                },
                label: None,
            },
            None,
        ).await.unwrap();

        // Create uniform buffer bind group layout
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Uniform Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let quads = vec! [
            InstancedQuad {
                top_left: vec2(10.0, 10.0),
                size: vec2(30.0, 30.0),
                color: vec4(1.0, 1.0, 0.0, 1.0),
            },
            InstancedQuad {
                top_left: vec2(100.0, 100.0),
                size: vec2(300.0, 300.0),
                color: vec4(0.0, 1.0, 1.0, 1.0),
            },
        ];

        let quad_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&quads[..]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Uniform Bind Group"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: quad_buffer.as_entire_binding(),
            }],
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&uniform_bind_group_layout],
                push_constant_ranges: &[wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::all(),
                    range: 0..std::mem::size_of::<ShaderConstants>() as u32,
                }],
            });

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![]
        };

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::util::make_spirv(&Asset::get("shader.spv").expect("Could not load shader")),
        });

        surface.configure(&device, &surface_config);

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "quad::vertex",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "quad::fragment",
                targets: &[Some(wgpu::ColorTargetState {
                    format: swapchain_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
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
            depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1, // 2.
                mask: !0, // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None,
        });

        Self {
            instance,
            surface: Some(surface),
            surface_config,
            adapter,
            device,
            queue,
            render_pipeline,
            quad_buffer,
            uniform_bind_group,
        }
    }

    pub fn render(&mut self, window: &Window, constants: ShaderConstants) -> Result<(), wgpu::SurfaceError> {
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
                        Some(wgpu::RenderPassColorAttachment {
                            view: &frame_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                                store: true,
                            }
                        })
                    ],
                    depth_stencil_attachment: None,
                });

                render_pass.set_pipeline(&self.render_pipeline); // 2.
                render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
                render_pass.set_push_constants(wgpu::ShaderStages::all(), 0, unsafe {
                    ::std::slice::from_raw_parts((&constants as *const ShaderConstants) as *const u8, ::std::mem::size_of::<ShaderConstants>())
                });
                render_pass.draw(0..6, 0..2); // 3.
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
                let surface = unsafe { self.instance.create_surface(window) }.unwrap();

                let swapchain_capabilities = surface.get_capabilities(&self.adapter);
                let swapchain_format = swapchain_capabilities.formats[0];
                self.surface_config.format = swapchain_format;
                self.surface_config.alpha_mode = swapchain_capabilities.alpha_modes[0];
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
                if let Err(render_error) = self.render(window, construct_constants()) {
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
        };
    }
}
