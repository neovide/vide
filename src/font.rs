use std::sync::Arc;

use font_kit::{handle::Handle, source::SystemSource};
use swash::FontRef;

#[derive(Clone)]
pub struct Font {
    index: usize,
    data: Arc<Vec<u8>>,
}

impl Font {
    pub fn from_name(font_name: &str) -> Option<Self> {
        let font = &SystemSource::new()
            .select_family_by_name(font_name)
            .ok()?
            .fonts()[0]
            .clone();

        match font {
            Handle::Path { path, font_index } => {
                let data = std::fs::read(path).ok()?;
                Some(Self {
                    data: Arc::new(data),
                    index: *font_index as usize,
                })
            }
            Handle::Memory { bytes, font_index } => Some(Self {
                data: bytes.clone(),
                index: *font_index as usize,
            }),
        }
    }

    pub fn as_ref<'a>(&'a self) -> Option<FontRef<'a>> {
        FontRef::from_index(self.data.as_ref(), self.index)
    }
}
