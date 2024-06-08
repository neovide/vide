use std::{
    fmt::Debug,
    sync::atomic::{AtomicU64, Ordering},
};

use base64::prelude::*;
use glamour::{Point2, Size2};
use image::{DynamicImage, GenericImageView};
use palette::Srgba;
use serde::{Deserialize, Serialize};

use crate::Resources;

static ID_COUNTER: AtomicU64 = AtomicU64::new(0);

// Private Sealed trait is used here to ensure that the only two types allowed as generic type
// arguments to sprite are Texture and TextureId. This way the same type can be used both before
// the texture is stored in resources with Texture and after with TextureId.
//
// Further, since TextureId has a private field, it also can only be constructed inside this file
// ensuring that the only way to add a sprite to a layer is by first adding the texture to the
// resources. This doesn't prevent a user from adding the texture to a separate scene and then
// adding the resulting texture id to a scene that doesn't contain the texture id, but its good
// enough protection for now.
trait Sealed {}
#[allow(private_bounds)]
pub trait SpriteTexture: Sealed {}
impl Sealed for Texture {}
impl SpriteTexture for Texture {}
impl Sealed for TextureId {}
impl SpriteTexture for TextureId {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Sprite<T: SpriteTexture> {
    pub top_left: Point2,
    pub size: Size2,
    pub color: Srgba,
    pub texture: T,
}

impl<T: SpriteTexture> Sprite<T> {
    pub fn new(texture: T, top_left: Point2, size: Size2) -> Self {
        Self {
            top_left,
            size,
            color: Srgba::new(1., 1., 1., 1.),
            texture,
        }
    }

    pub fn with_color(mut self, color: Srgba) -> Self {
        self.color = color;
        self
    }
}

impl Sprite<Texture> {
    pub fn redirect_texture(&self, resources: &mut Resources) -> Sprite<TextureId> {
        let texture_id = resources.store_texture(self.texture.clone());
        Sprite {
            top_left: self.top_left,
            size: self.size,
            color: self.color,
            texture: texture_id,
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq)]
pub struct TextureId(u64);

impl Resources {
    pub fn store_texture(&mut self, texture: Texture) -> TextureId {
        let id = ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        self.textures.insert(TextureId(id), texture);
        TextureId(id)
    }
}

#[derive(Clone)]
pub struct Texture {
    pub data: Vec<u8>,
    pub size: Size2<u32>,
}

impl Texture {
    pub fn from_image(image: DynamicImage) -> Self {
        let data = image.to_rgba8();
        let (image_width, image_height) = image.dimensions();
        Self {
            data: data.to_vec(),
            size: Size2::new(image_width, image_height),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct SerializableTexture {
    pub data: String,
    pub size: Size2<u32>,
}

impl<'a> Deserialize<'a> for Texture {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        let serializable_texture = SerializableTexture::deserialize(deserializer)?;
        let data = BASE64_STANDARD.decode(serializable_texture.data).unwrap();
        Ok(Self {
            data,
            size: serializable_texture.size,
        })
    }
}

impl Serialize for Texture {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let data = BASE64_STANDARD.encode(&self.data);
        SerializableTexture {
            data,
            size: self.size,
        }
        .serialize(serializer)
    }
}

impl Debug for Texture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Texture").field("size", &self.size).finish()
    }
}
