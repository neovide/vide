use glam::*;
use glamour::{size2, vec2, Point2, Rect, ToRaw};
use ordered_float::OrderedFloat;
use palette::Srgba;
use parley::swash::{
    scale::{image::Content, Render, ScaleContext, Source, StrikeWith},
    zeno::{Format, Placement, Vector},
    FontRef, GlyphId,
};
use wgpu::*;

use crate::{
    drawable::Drawable,
    drawable_reference::{Atlas, ConstructResult, DrawableReference, InstanceBuffer},
    renderer::Renderer,
    scene::GlyphRun,
    shader::ShaderConstants,
    FontId, LayerContents, Resources,
};

#[derive(Copy, Clone, Default)]
#[repr(u32)]
pub enum GlyphKind {
    #[default]
    Mask = 0,
    Subpixel = 1,
    Color = 2,
}

#[derive(Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct InstancedGlyph {
    pub bottom_left: Vec2,
    pub atlas_top_left: Vec2,
    pub atlas_size: Vec2,
    pub kind: i32,
    // Need a float of padding here so that fields are all aligned to
    // 16 bytes in size. Vec2s are 8 bytes, Vec4s are 16 bytes.
    _padding: f32,
    pub color: Vec4,
}

pub struct GlyphState {
    glyph_buffer: InstanceBuffer<InstancedGlyph>,
    atlas: Atlas<GlyphKey, (Placement, Content)>,
    scale_context: ScaleContext,
}

impl GlyphState {
    #[allow(clippy::too_many_arguments)]
    fn prepare_glyph<'a, 'b: 'a>(
        &'b mut self,
        queue: &Queue,
        font_id: FontId,
        font_ref: FontRef<'a>,
        glyph: GlyphId,
        bottom_left: Point2,
        size: f32,
        color: Srgba,
        normalized_coords: &[i16],
    ) -> Option<InstancedGlyph> {
        // Create a font scaler for the given font and size

        let glyph_key = GlyphKey::new(font_id, glyph, size, bottom_left);

        // Get or find atlas allocation
        let ((placement, content), glyph_location) =
            self.atlas.lookup_or_upload(queue, glyph_key.clone(), || {
                let mut scaler = {
                    profiling::scope!("Creating font scaler");
                    self.scale_context
                        .builder(font_ref)
                        .size(size)
                        .hint(true)
                        .normalized_coords(normalized_coords)
                        .build()
                };
                let image = Render::new(&[
                    Source::ColorOutline(0),
                    Source::ColorBitmap(StrikeWith::BestFit),
                    Source::Outline,
                ])
                // Select a subpixel format
                .format(Format::Subpixel)
                // Apply the fractional offset
                .offset(glyph_key.quantized_offset())
                // Render the image
                .render(&mut scaler, glyph)
                .expect("Could not render glyph into an image");

                if image.placement.width == 0 || image.placement.height == 0 {
                    return ConstructResult::Empty;
                }

                ConstructResult::Constructed(
                    (image.placement, image.content),
                    image.data,
                    size2!(image.placement.width, image.placement.height),
                )
            })?;

        let bottom_left = bottom_left.floor()
            + vec2!(
                placement.left as f32,
                placement.height as f32 - placement.top as f32
            );

        // Add the glyph to instances
        Some(InstancedGlyph {
            bottom_left: bottom_left.to_raw(),
            atlas_top_left: glam::vec2(glyph_location.min.x as f32, glyph_location.min.y as f32),
            atlas_size: glam::vec2(placement.width as f32, placement.height as f32),
            kind: match content {
                Content::Mask => GlyphKind::Mask as i32,
                Content::SubpixelMask => GlyphKind::Subpixel as i32,
                Content::Color => GlyphKind::Color as i32,
            },
            _padding: Default::default(),
            color: Vec4::from_array(color.into_linear().into()),
        })
    }

    pub fn rasterize_glyph_run<'a, 'b: 'a>(
        &mut self,
        queue: &Queue,
        font_ref: FontRef<'a>,
        glyph_run: &GlyphRun,
    ) -> Vec<InstancedGlyph> {
        glyph_run
            .glyphs
            .iter()
            .filter_map(|glyph| {
                self.prepare_glyph(
                    queue,
                    glyph_run.font_id,
                    font_ref,
                    glyph.id,
                    glyph_run.position + glyph.offset,
                    glyph_run.size,
                    glyph_run.color,
                    &glyph_run.normalized_coords,
                )
            })
            .collect()
    }
}

impl Drawable for GlyphState {
    fn new(renderer: &Renderer) -> Self {
        let glyph_buffer = InstanceBuffer::new(renderer, "glyph");
        let atlas = Atlas::new(renderer, "glyph");

        Self {
            glyph_buffer,
            atlas,

            scale_context: ScaleContext::new(),
        }
    }

    fn name(&self) -> &str {
        "glyph"
    }

    fn references(&self) -> Vec<&dyn DrawableReference> {
        vec![&self.glyph_buffer, &self.atlas]
    }

    fn needs_offscreen_copy(&self) -> bool {
        true
    }

    fn start_frame(&mut self) {
        self.glyph_buffer.start_frame();
    }

    fn has_work(&self, contents: &LayerContents) -> bool {
        !contents.glyph_runs.is_empty()
    }

    fn draw<'b, 'a: 'b>(
        &'a mut self,
        queue: &Queue,
        render_pass: &mut RenderPass<'b>,
        _constants: ShaderConstants,
        resources: &Resources,
        _clip: Option<Rect<u32>>,
        layer: &LayerContents,
    ) {
        profiling::scope!("Glyph::draw");
        let glyphs: Vec<_> = layer
            .glyph_runs
            .iter()
            .flat_map(|glyph_run| {
                let font = resources.fonts.get(&glyph_run.font_id).unwrap();
                let font_ref = font.as_swash_font_ref(glyph_run.font_index).unwrap();
                self.rasterize_glyph_run(queue, font_ref, glyph_run)
                    .into_iter()
            })
            .collect();
        self.glyph_buffer.upload(glyphs, queue);
        self.glyph_buffer.draw(render_pass);
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum SubpixelOffset {
    Zero,
    Quarter,
    Half,
    ThreeQuarters,
}

impl SubpixelOffset {
    fn quantize(value: f32) -> Self {
        let value = value.fract();
        if value < 0.125 {
            Self::Zero
        } else if value < 0.375 {
            Self::Quarter
        } else if value < 0.625 {
            Self::Half
        } else if value < 0.875 {
            Self::ThreeQuarters
        } else {
            Self::Zero
        }
    }

    fn as_f32(&self) -> f32 {
        match self {
            Self::Zero => 0.0,
            Self::Quarter => 0.25,
            Self::Half => 0.5,
            Self::ThreeQuarters => 0.75,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct GlyphKey {
    glyph: GlyphId,
    font_id: FontId,
    size: OrderedFloat<f32>,
    x_offset: SubpixelOffset,
}

impl GlyphKey {
    fn new(font_id: FontId, glyph: GlyphId, size: f32, offset: Point2) -> Self {
        let size = size.into();
        let x_offset = SubpixelOffset::quantize(offset.x);
        Self {
            glyph,
            font_id,
            size,
            x_offset,
        }
    }

    fn quantized_offset(&self) -> Vector {
        Vector::new(self.x_offset.as_f32(), 0.)
    }
}
