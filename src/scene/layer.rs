use std::sync::Arc;

use glamour::{point2, size2, vec2, Point2, Rect};
use palette::Srgba;
use parley::{layout::PositionedLayoutItem, Layout};
use serde::{Deserialize, Serialize};

use super::{Blur, Glyph, GlyphRun, Path, Quad, Resources, Sprite, TextureId};

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Layer {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clip: Option<Rect<u32>>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mask: Option<LayerContents>,
    #[serde(default)]
    #[serde(flatten)]
    pub contents: LayerContents,
}

impl Layer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_clip(mut self, clip: Rect<u32>) -> Self {
        self.set_clip(clip);
        self
    }

    pub fn set_clip(&mut self, clip: Rect<u32>) {
        self.clip = Some(clip);
    }

    pub fn with_mask(mut self, mask_layer: Layer) -> Self {
        self.set_mask(mask_layer);
        self
    }

    pub fn set_mask(&mut self, mask_layer: Layer) {
        self.mask = Some(mask_layer.contents);
    }

    pub fn add_quad(&mut self, quad: Quad) {
        self.contents.add_quad(quad);
    }

    pub fn add_quads(&mut self, quads: Arc<Vec<Quad>>) {
        self.contents.add_quads(quads);
    }

    pub fn with_quad(mut self, quad: Quad) -> Self {
        self.add_quad(quad);
        self
    }

    pub fn with_quads(mut self, quads: Arc<Vec<Quad>>) -> Self {
        self.add_quads(quads);
        self
    }

    pub fn add_clear(&mut self, color: Srgba) {
        self.add_quad(Quad::new(
            Rect::new(point2!(0.0, 0.0), size2!(f32::MAX / 2., f32::MAX / 2.)),
            color,
        ));
    }

    pub fn with_clear(mut self, color: Srgba) -> Self {
        self.add_clear(color);
        self
    }

    pub fn add_blur(&mut self, blur: Blur) {
        self.contents.add_blur(blur);
    }

    pub fn with_blur(mut self, blur: Blur) -> Self {
        self.add_blur(blur);
        self
    }

    pub fn add_blurs(&mut self, blurs: Arc<Vec<Blur>>) {
        self.contents.add_blurs(blurs);
    }

    pub fn with_blurs(mut self, blurs: Arc<Vec<Blur>>) -> Self {
        self.add_blurs(blurs);
        self
    }

    pub fn add_blurred_clear(&mut self, color: Srgba, blur: f32) {
        self.add_blur(Blur::new(
            point2!(0.0, 0.0),
            size2!(f32::MAX / 2., f32::MAX / 2.),
            color,
            blur,
        ));
    }

    pub fn with_blurred_clear(mut self, color: Srgba, blur: f32) -> Self {
        self.add_blurred_clear(color, blur);
        self
    }

    pub fn add_glyph_run(&mut self, glyph_run: GlyphRun) {
        self.contents.add_glyph_run(glyph_run);
    }

    pub fn add_glyph_runs(&mut self, glyph_runs: Arc<Vec<GlyphRun>>) {
        self.contents.add_glyph_runs(glyph_runs);
    }

    pub fn with_glyph_run(mut self, glyph_run: GlyphRun) -> Self {
        self.add_glyph_run(glyph_run);
        self
    }

    pub fn with_glyph_runs(mut self, glyph_runs: Arc<Vec<GlyphRun>>) -> Self {
        self.add_glyph_runs(glyph_runs);
        self
    }

    pub fn add_path(&mut self, path: Path) {
        self.contents.add_path(path);
    }

    pub fn add_paths(&mut self, paths: Arc<Vec<Path>>) {
        self.contents.add_paths(paths);
    }

    pub fn with_path(mut self, path: Path) -> Self {
        self.add_path(path);
        self
    }

    pub fn with_paths(mut self, paths: Arc<Vec<Path>>) -> Self {
        self.add_paths(paths);
        self
    }

    pub fn add_sprite(&mut self, sprite: Sprite<TextureId>) {
        self.contents.add_sprite(sprite);
    }

    pub fn add_sprites(&mut self, sprites: Arc<Vec<Sprite<TextureId>>>) {
        self.contents.add_sprites(sprites);
    }

    pub fn with_sprite(mut self, sprite: Sprite<TextureId>) -> Self {
        self.add_sprite(sprite);
        self
    }

    pub fn with_sprites(mut self, sprites: Arc<Vec<Sprite<TextureId>>>) -> Self {
        self.add_sprites(sprites);
        self
    }

    pub fn add_text_layout(
        &mut self,
        resources: &mut Resources,
        layout: &Layout<Srgba>,
        position: Point2,
    ) {
        for line in layout.lines() {
            for item in line.items() {
                let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                    continue;
                };
                let run = glyph_run.run();
                let font = run.font();
                let font_id = resources.store_font(font);
                let style = glyph_run.style();
                let color = style.brush;
                let synthesis = glyph_run.run().synthesis().into();

                let font_index = font.index as usize;
                let size = run.font_size();
                let normalized_coords = run.normalized_coords().to_vec();
                let mut glyphs = Vec::new();
                let mut current_x = 0.0;
                for glyph in glyph_run.glyphs() {
                    glyphs.push(Glyph {
                        id: glyph.id,
                        offset: vec2!(current_x + glyph.x, -glyph.y),
                    });
                    current_x += glyph.advance;
                }

                self.add_glyph_run(GlyphRun {
                    position: position + vec2!(glyph_run.offset(), glyph_run.baseline()),
                    font_id,
                    font_index,
                    color,
                    size,
                    normalized_coords,
                    glyphs,
                    synthesis,
                });
            }
        }
    }

    pub fn with_text_layout(
        mut self,
        resources: &mut Resources,
        layout: &Layout<Srgba>,
        position: Point2,
    ) -> Self {
        self.add_text_layout(resources, layout, position);
        self
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct LayerContents {
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub primitives: Vec<PrimitiveBatch>,
}

impl LayerContents {
    pub fn add_quad(&mut self, quad: Quad) {
        match self.primitives.last_mut() {
            Some(PrimitiveBatch::Mutable(MutablePrimitiveBatch::Quads(quads))) => {
                quads.push(quad);
            }
            _ => {
                self.primitives
                    .push(PrimitiveBatch::Mutable(MutablePrimitiveBatch::Quads(vec![
                        quad,
                    ])));
            }
        }
    }

    pub fn add_quads(&mut self, quads: Arc<Vec<Quad>>) {
        self.primitives
            .push(PrimitiveBatch::Shared(SharedPrimitiveBatch::Quads(quads)));
    }

    pub fn add_blur(&mut self, blur: Blur) {
        match self.primitives.last_mut() {
            Some(PrimitiveBatch::Mutable(MutablePrimitiveBatch::Blurs(blurs))) => {
                blurs.push(blur);
            }
            _ => {
                self.primitives
                    .push(PrimitiveBatch::Mutable(MutablePrimitiveBatch::Blurs(vec![
                        blur,
                    ])));
            }
        }
    }

    pub fn add_blurs(&mut self, blurs: Arc<Vec<Blur>>) {
        self.primitives
            .push(PrimitiveBatch::Shared(SharedPrimitiveBatch::Blurs(blurs)));
    }

    pub fn add_glyph_run(&mut self, glyph_run: GlyphRun) {
        match self.primitives.last_mut() {
            Some(PrimitiveBatch::Mutable(MutablePrimitiveBatch::GlyphRuns(glyph_runs))) => {
                glyph_runs.push(glyph_run);
            }
            _ => {
                self.primitives
                    .push(PrimitiveBatch::Mutable(MutablePrimitiveBatch::GlyphRuns(
                        vec![glyph_run],
                    )));
            }
        }
    }

    pub fn add_glyph_runs(&mut self, glyph_runs: Arc<Vec<GlyphRun>>) {
        self.primitives
            .push(PrimitiveBatch::Shared(SharedPrimitiveBatch::GlyphRuns(
                glyph_runs,
            )));
    }

    pub fn add_path(&mut self, path: Path) {
        match self.primitives.last_mut() {
            Some(PrimitiveBatch::Mutable(MutablePrimitiveBatch::Paths(paths))) => {
                paths.push(path);
            }
            _ => {
                self.primitives
                    .push(PrimitiveBatch::Mutable(MutablePrimitiveBatch::Paths(vec![
                        path,
                    ])));
            }
        }
    }

    pub fn add_paths(&mut self, paths: Arc<Vec<Path>>) {
        self.primitives
            .push(PrimitiveBatch::Shared(SharedPrimitiveBatch::Paths(paths)));
    }

    pub fn add_sprite(&mut self, sprite: Sprite<TextureId>) {
        match self.primitives.last_mut() {
            Some(PrimitiveBatch::Mutable(MutablePrimitiveBatch::Sprites(sprites))) => {
                sprites.push(sprite);
            }
            _ => {
                self.primitives
                    .push(PrimitiveBatch::Mutable(MutablePrimitiveBatch::Sprites(
                        vec![sprite],
                    )));
            }
        }
    }

    pub fn add_sprites(&mut self, sprites: Arc<Vec<Sprite<TextureId>>>) {
        self.primitives
            .push(PrimitiveBatch::Shared(SharedPrimitiveBatch::Sprites(
                sprites,
            )));
    }
}

#[derive(Clone, Debug)]
pub enum PrimitiveBatch {
    Mutable(MutablePrimitiveBatch),
    Shared(SharedPrimitiveBatch),
}

impl PrimitiveBatch {
    pub fn is_blurs(&self) -> bool {
        matches!(
            self,
            Self::Mutable(MutablePrimitiveBatch::Blurs(_))
                | Self::Shared(SharedPrimitiveBatch::Blurs(_))
        )
    }

    pub fn as_blur_vec(&self) -> Option<&Vec<Blur>> {
        match self {
            Self::Mutable(MutablePrimitiveBatch::Blurs(blurs)) => Some(blurs),
            Self::Shared(SharedPrimitiveBatch::Blurs(blurs)) => Some(blurs),
            _ => None,
        }
    }

    pub fn is_quads(&self) -> bool {
        matches!(
            self,
            Self::Mutable(MutablePrimitiveBatch::Quads(_))
                | Self::Shared(SharedPrimitiveBatch::Quads(_))
        )
    }

    pub fn as_quad_vec(&self) -> Option<&Vec<Quad>> {
        match self {
            Self::Mutable(MutablePrimitiveBatch::Quads(quads)) => Some(quads),
            Self::Shared(SharedPrimitiveBatch::Quads(quads)) => Some(quads),
            _ => None,
        }
    }

    pub fn is_glyph_runs(&self) -> bool {
        matches!(
            self,
            Self::Mutable(MutablePrimitiveBatch::GlyphRuns(_))
                | Self::Shared(SharedPrimitiveBatch::GlyphRuns(_))
        )
    }

    pub fn as_glyph_run_vec(&self) -> Option<&Vec<GlyphRun>> {
        match self {
            Self::Mutable(MutablePrimitiveBatch::GlyphRuns(glyph_runs)) => Some(glyph_runs),
            Self::Shared(SharedPrimitiveBatch::GlyphRuns(glyph_runs)) => Some(glyph_runs),
            _ => None,
        }
    }

    pub fn is_paths(&self) -> bool {
        matches!(
            self,
            Self::Mutable(MutablePrimitiveBatch::Paths(_))
                | Self::Shared(SharedPrimitiveBatch::Paths(_))
        )
    }

    pub fn as_path_vec(&self) -> Option<&Vec<Path>> {
        match self {
            Self::Mutable(MutablePrimitiveBatch::Paths(paths)) => Some(paths),
            Self::Shared(SharedPrimitiveBatch::Paths(paths)) => Some(paths),
            _ => None,
        }
    }

    pub fn is_sprites(&self) -> bool {
        matches!(
            self,
            Self::Mutable(MutablePrimitiveBatch::Sprites(_))
                | Self::Shared(SharedPrimitiveBatch::Sprites(_))
        )
    }

    pub fn as_sprite_vec(&self) -> Option<&Vec<Sprite<TextureId>>> {
        match self {
            Self::Mutable(MutablePrimitiveBatch::Sprites(sprites)) => Some(sprites),
            Self::Shared(SharedPrimitiveBatch::Sprites(sprites)) => Some(sprites),
            _ => None,
        }
    }
}

impl Serialize for PrimitiveBatch {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Mutable(batch) => batch.serialize(serializer),
            Self::Shared(batch) => batch.to_mutable().serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for PrimitiveBatch {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self::Mutable(MutablePrimitiveBatch::deserialize(
            deserializer,
        )?))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum MutablePrimitiveBatch {
    Blurs(Vec<Blur>),
    Quads(Vec<Quad>),
    GlyphRuns(Vec<GlyphRun>),
    Paths(Vec<Path>),
    Sprites(Vec<Sprite<TextureId>>),
}

#[derive(Clone, Debug)]
pub enum SharedPrimitiveBatch {
    Blurs(Arc<Vec<Blur>>),
    Quads(Arc<Vec<Quad>>),
    GlyphRuns(Arc<Vec<GlyphRun>>),
    Paths(Arc<Vec<Path>>),
    Sprites(Arc<Vec<Sprite<TextureId>>>),
}

impl SharedPrimitiveBatch {
    pub fn to_mutable(&self) -> MutablePrimitiveBatch {
        match self {
            Self::Blurs(blurs) => MutablePrimitiveBatch::Blurs(blurs.to_vec()),
            Self::Quads(quads) => MutablePrimitiveBatch::Quads(quads.to_vec()),
            Self::GlyphRuns(glyph_runs) => MutablePrimitiveBatch::GlyphRuns(glyph_runs.to_vec()),
            Self::Paths(paths) => MutablePrimitiveBatch::Paths(paths.to_vec()),
            Self::Sprites(sprites) => MutablePrimitiveBatch::Sprites(sprites.to_vec()),
        }
    }
}
