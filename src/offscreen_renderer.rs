use futures_intrusive::channel::shared::oneshot_channel;
use image::{imageops::crop_imm, ImageBuffer, Rgba};
use wgpu::{Backends, Instance, PowerPreference, RequestAdapterOptions};

use crate::{drawable::Drawable, Renderer, Scene};

pub struct OffscreenRenderer {
    pub instance: Instance,
    pub renderer: Renderer,
}

impl OffscreenRenderer {
    // Creating some of the wgpu types requires async code
    pub async fn new(width: u32, height: u32) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: Self::instance_backends(),
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                force_fallback_adapter: true,
                compatible_surface: None,
            })
            .await
            .unwrap();

        let renderer =
            Renderer::new(width, height, adapter, wgpu::TextureFormat::Rgba8UnormSrgb).await;

        Self { instance, renderer }
    }

    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        self.renderer.resize(new_width, new_height);
    }

    pub async fn add_drawable<T: Drawable + 'static>(&mut self) {
        self.renderer.add_drawable::<T>().await;
    }

    pub async fn with_drawable<T: Drawable + 'static>(mut self) -> Self {
        self.add_drawable::<T>().await;
        self
    }

    pub async fn add_default_drawables(&mut self) {
        self.renderer.add_default_drawables().await;
    }

    pub async fn with_default_drawables(mut self) -> Self {
        self.add_default_drawables().await;
        self
    }

    fn instance_backends() -> Backends {
        #[cfg(target_os = "macos")]
        {
            wgpu::Backends::METAL
        }

        #[cfg(not(target_os = "macos"))]
        {
            wgpu::Backends::VULKAN
        }
    }

    pub async fn draw(&mut self, scene: &Scene) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let texture_desc = wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: self.renderer.width,
                height: self.renderer.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: None,
            view_formats: &[],
        };
        let texture = self.renderer.device.create_texture(&texture_desc);

        self.renderer.render(scene, &texture);

        let mut encoder = self
            .renderer
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let u32_size = std::mem::size_of::<u32>() as u32;
        let bytes_per_row = u32_size * self.renderer.width;
        // The bytes_per_row must be padded to be aligned to COPY_BYTES_PER_ROW_ALIGNMENT (256)
        let padding =
            wgpu::COPY_BYTES_PER_ROW_ALIGNMENT - bytes_per_row % wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_bytes_per_row = bytes_per_row + padding;
        let padded_width = padded_bytes_per_row / u32_size;
        let output_buffer_size =
            (padded_bytes_per_row * self.renderer.height) as wgpu::BufferAddress;
        let output_buffer_desc = wgpu::BufferDescriptor {
            size: output_buffer_size,
            usage: wgpu::BufferUsages::COPY_DST
        // this tells wpgu that we want to read this buffer from the cpu
        | wgpu::BufferUsages::MAP_READ,
            label: None,
            mapped_at_creation: false,
        };
        let output_buffer = self.renderer.device.create_buffer(&output_buffer_desc);

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::ImageCopyBuffer {
                buffer: &output_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(self.renderer.height),
                },
            },
            texture_desc.size,
        );

        self.renderer.queue.submit(Some(encoder.finish()));

        let buffer_slice = output_buffer.slice(..);

        // NOTE: We have to create the mapping THEN device.poll() before await
        // the future. Otherwise the application will freeze.
        let (tx, rx) = oneshot_channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });
        self.renderer.device.poll(wgpu::Maintain::Wait);
        rx.receive().await.unwrap().unwrap();

        let data = buffer_slice.get_mapped_range().to_vec();
        let padded_image =
            ImageBuffer::<Rgba<u8>, _>::from_raw(padded_width, self.renderer.height, data).unwrap();

        crop_imm(
            &padded_image,
            0,
            0,
            self.renderer.width,
            self.renderer.height,
        )
        .to_image()
    }
}
