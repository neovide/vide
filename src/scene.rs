mod glyph_run;
mod layer;
mod path;
mod quad;
mod sprite;

use std::collections::HashMap;

use glamour::{Point2, Rect};
use palette::Srgba;
use parley::Layout;
use serde::{Deserialize, Serialize};

pub use glyph_run::*;
pub use layer::*;
pub use path::*;
pub use quad::*;
pub use sprite::*;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Scene {
    pub layers: Vec<Layer>,
    pub resources: Resources,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            resources: Default::default(),
            layers: vec![Default::default()],
        }
    }

    pub fn add_layer(&mut self, layer: Layer) {
        self.layers.push(layer);
    }

    pub fn with_layer(mut self, layer: Layer) -> Self {
        self.add_layer(layer);
        self
    }

    pub fn layer(&self) -> &Layer {
        self.layers.last().unwrap()
    }

    pub fn layer_mut(&mut self) -> &mut Layer {
        self.layers.last_mut().unwrap()
    }

    pub fn update_layer(&mut self, update: impl FnOnce(&mut Resources, &mut Layer)) {
        let resources = &mut self.resources;
        let layer = self.layers.last_mut().unwrap();
        update(resources, layer);
    }

    pub fn with_clip(mut self, clip: Rect<u32>) -> Self {
        self.layer_mut().clip = Some(clip);
        self
    }

    pub fn with_blur(mut self, radius: f32) -> Self {
        self.layer_mut().background_blur_radius = radius;
        self
    }

    pub fn background(&mut self, color: Srgba) {
        self.layer_mut().background_color = Some(color);
    }

    pub fn with_background(mut self, color: Srgba) -> Self {
        self.background(color);
        self
    }

    pub fn add_quad(&mut self, quad: Quad) {
        self.layer_mut().add_quad(quad);
    }

    pub fn with_quad(mut self, quad: Quad) -> Self {
        self.add_quad(quad);
        self
    }

    pub fn add_path(&mut self, path: Path) {
        self.layer_mut().add_path(path);
    }

    pub fn with_path(mut self, path: Path) -> Self {
        self.add_path(path);
        self
    }

    /// Adds the given sprite with included texture to the current layer.
    ///
    /// WARNING: This will store whatever texture is associated with this sprite in the resources
    /// without checking for duplication. If you want to avoid duplication, you should use
    /// `store_texture` on `self.resources` to get a TextureId, and then use that TextureId in the
    /// sprites. To add a Sprite<TextureId>, use `add_sprite` on the layer like this:
    /// `scene.layer_mut().add_sprite(sprite)`.
    pub fn add_sprite(&mut self, sprite: Sprite<Texture>) {
        let sprite = sprite.redirect_texture(&mut self.resources);
        self.layer_mut().add_sprite(sprite);
    }

    /// Adds the given sprite with included texture to the current layer.
    ///
    /// WARNING: This will store whatever texture is associated with this sprite in the resources
    /// without checking for duplication. If you want to avoid duplication, you should use
    /// `store_texture` on `self.resources` to get a TextureId, and then use that TextureId in the
    /// sprites. To add a Sprite<TextureId>, use `add_sprite` on the layer like this:
    /// `scene.layer_mut().add_sprite(sprite)`.
    pub fn with_sprite(mut self, sprite: Sprite<Texture>) -> Self {
        self.add_sprite(sprite);
        self
    }

    pub fn add_text_layout(&mut self, layout: Layout<Srgba>, top_left: Point2) {
        self.update_layer(|resources, layer| {
            layer.add_text_layout(resources, layout, top_left);
        });
    }

    pub fn with_text_layout(mut self, layout: Layout<Srgba>, top_left: Point2) -> Self {
        self.add_text_layout(layout, top_left);
        self
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Resources {
    pub fonts: HashMap<FontId, Font>,
    pub textures: HashMap<TextureId, Texture>,
}
