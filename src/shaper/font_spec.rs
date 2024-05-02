use glam::Vec4;

pub struct FontSpec<'a> {
    pub font_name: &'a str,
    pub size: Option<f32>,
    pub color: Option<Vec4>,
}

pub trait IntoFontSpec {
    fn into_font_spec<'a>(&'a self) -> FontSpec<'a>;
}

impl IntoFontSpec for &str {
    fn into_font_spec<'a>(&'a self) -> FontSpec<'a> {
        FontSpec {
            font_name: self,
            size: None,
            color: None,
        }
    }
}

impl IntoFontSpec for (&str, f32) {
    fn into_font_spec<'a>(&'a self) -> FontSpec<'a> {
        FontSpec {
            font_name: self.0,
            size: Some(self.1),
            color: None,
        }
    }
}

impl IntoFontSpec for (&str, Vec4) {
    fn into_font_spec<'a>(&'a self) -> FontSpec<'a> {
        FontSpec {
            font_name: self.0,
            size: None,
            color: Some(self.1),
        }
    }
}

impl IntoFontSpec for (&str, f32, Vec4) {
    fn into_font_spec<'a>(&'a self) -> FontSpec<'a> {
        FontSpec {
            font_name: self.0,
            size: Some(self.1),
            color: Some(self.2),
        }
    }
}
