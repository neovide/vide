use std::{collections::HashMap, hash::Hash};

use etagere::{euclid, AllocId, AtlasAllocator};
use glamour::{point2, size2, Box2, Size2};
use wgpu::*;

use crate::Renderer;

use super::DrawableReference;

pub const ATLAS_SIZE: Size2<u32> = size2!(1024, 1024);

pub struct Atlas<Key, UserData = ()> {
    texture: Texture,
    texture_view: TextureView,
    allocator: AtlasAllocator,
    // When the value is None, the key is in the atlas but the construction resulted in an empty
    // image and is tombstoned so that construction isn't attempted again for that key.
    lookup: HashMap<Key, Option<(UserData, AllocId)>>,
}

pub enum ConstructResult<UserData> {
    // Returned when the result is constructed and should be uploaded to the atlas.
    Constructed(UserData, Vec<u8>, Size2<u32>),
    // Returned when the result is empty on purpose and construction for this key shouldn't be
    // attempted again.
    Empty,
    // Returned when the result is empty because construction failed and should be attempted again.
    Failed,
}

impl<Key: Eq + Hash, UserData: Clone> Atlas<Key, UserData> {
    pub fn new(Renderer { device, .. }: &Renderer, name: &str) -> Self {
        let texture = device.create_texture(&TextureDescriptor {
            label: Some(&format!("{} atlas texture", name)),
            size: Extent3d {
                width: ATLAS_SIZE.width,
                height: ATLAS_SIZE.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let texture_view = texture.create_view(&TextureViewDescriptor::default());

        let allocator = AtlasAllocator::new(euclid::size2(
            ATLAS_SIZE.width as i32,
            ATLAS_SIZE.height as i32,
        ));

        let lookup = HashMap::new();

        Self {
            texture,
            texture_view,
            allocator,
            lookup,
        }
    }

    pub fn lookup_or_upload(
        &mut self,
        queue: &Queue,
        key: Key,
        construct_image: impl FnOnce() -> ConstructResult<UserData>,
    ) -> Option<(UserData, Box2<i32>)> {
        match self.lookup.get(&key) {
            Some(Some((user_data, alloc_id))) => {
                let allocation_rectangle = self.allocator.get(*alloc_id);
                Some((user_data.clone(), euclid_to_glamour(allocation_rectangle)))
            }
            // Tombstone. Don't attempt to construct
            Some(None) => None,
            // Not in the atlas yet. Attempt to construct an image to upload
            None => {
                profiling::scope!("Atlas upload");

                let (user_data, image_data, image_size) = match construct_image() {
                    ConstructResult::Constructed(user_data, image_data, image_size) => {
                        (user_data, image_data, image_size)
                    }
                    ConstructResult::Empty => {
                        self.lookup.insert(key, None);
                        return None;
                    }
                    ConstructResult::Failed => {
                        dbg!("Could not construct glyph image");
                        return None;
                    }
                };

                let allocation = self
                    .allocator
                    .allocate(euclid::size2(
                        image_size.width as i32,
                        image_size.height as i32,
                    ))
                    .expect("Could not allocate glyph to atlas");

                self.lookup
                    .insert(key, Some((user_data.clone(), allocation.id)));

                queue.write_texture(
                    ImageCopyTexture {
                        texture: &self.texture,
                        mip_level: 0,
                        origin: Origin3d {
                            x: allocation.rectangle.min.x as u32,
                            y: allocation.rectangle.min.y as u32,
                            z: 0,
                        },
                        aspect: TextureAspect::All,
                    },
                    &image_data,
                    ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(4 * image_size.width),
                        rows_per_image: Some(image_size.height),
                    },
                    Extent3d {
                        width: image_size.width,
                        height: image_size.height,
                        depth_or_array_layers: 1,
                    },
                );

                Some((user_data, euclid_to_glamour(allocation.rectangle)))
            }
        }
    }
}

fn euclid_to_glamour<Units>(euclid_rectangle: euclid::Box2D<i32, Units>) -> Box2<i32> {
    Box2::new(
        point2!(euclid_rectangle.min.x, euclid_rectangle.min.y),
        point2!(euclid_rectangle.max.x, euclid_rectangle.max.y),
    )
}

impl<Key, UserData> DrawableReference for Atlas<Key, UserData> {
    fn layout(&self) -> Option<BindGroupLayoutEntry> {
        Some(BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Texture {
                sample_type: TextureSampleType::Float { filterable: true },
                view_dimension: TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        })
    }

    fn entry(&self) -> Option<BindGroupEntry> {
        Some(BindGroupEntry {
            binding: 0,
            resource: BindingResource::TextureView(&self.texture_view),
        })
    }
}
