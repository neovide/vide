use palette::Srgba;
use parley::{context::RangedBuilder, style::StyleProperty, FontContext, LayoutContext};

pub struct Shaper {
    font_context: FontContext,
    layout_context: LayoutContext<Srgba>,
    default_styles: Vec<StyleProperty<'static, Srgba>>,
}

impl Shaper {
    pub fn new() -> Self {
        Self {
            font_context: FontContext::default(),
            layout_context: LayoutContext::new(),
            default_styles: Vec::new(),
        }
    }

    pub fn layout<'a>(&'a mut self, text: &'a str) -> RangedBuilder<'a, Srgba, &'a str> {
        let mut builder =
            // TODO: Dig through if this display scale is doing something important we need to
            // replicate
            self.layout_context
                .ranged_builder(&mut self.font_context, text, 1.);
        for style in &self.default_styles {
            builder.push_default(style);
        }

        builder
    }

    pub fn push_default(&mut self, style: StyleProperty<'static, Srgba>) {
        self.default_styles.push(style);
    }

    pub fn clear_defaults(&mut self) {
        self.default_styles.clear();
    }
}
