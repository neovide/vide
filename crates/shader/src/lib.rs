mod glyph;
mod path;
mod quad;
mod sprite;

pub use glyph::*;
pub use path::*;
pub use quad::*;
pub use sprite::*;

use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use glam::*;
use notify_debouncer_full::{
    new_debouncer,
    notify::{RecommendedWatcher, RecursiveMode, Watcher},
    DebounceEventResult, DebouncedEvent, Debouncer, FileIdMap,
};
use rust_embed::*;
use wgpu::{Device, ShaderModule, ShaderModuleDescriptor, ShaderSource};

#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct ShaderConstants {
    pub surface_size: Vec2,
    pub atlas_size: Vec2,
    pub clip: Vec4,
}

#[derive(RustEmbed)]
#[folder = "wgsl/"]
struct Shader;

pub struct ShaderLoader {
    changed: Arc<AtomicBool>,
    watcher: Option<Debouncer<RecommendedWatcher, FileIdMap>>,
}

impl Default for ShaderLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl ShaderLoader {
    pub fn new() -> Self {
        Self {
            changed: Arc::default(),
            watcher: None,
        }
    }

    pub fn watch<F: FnMut() + Send + 'static>(&mut self, shaders_changed: F) {
        let wgsl_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("wgsl");
        let watcher = {
            let changed: Arc<AtomicBool> = Arc::clone(&self.changed);
            let mut shaders_changed = shaders_changed;
            let f = move |res: DebounceEventResult| {
                if let Ok(res) = res {
                    if res
                        .iter()
                        .any(|event| matches!(event, DebouncedEvent { .. }))
                    {
                        changed.store(true, Ordering::SeqCst);
                        shaders_changed();
                    }
                }
            };
            let mut watcher = new_debouncer(Duration::from_millis(500), None, f).ok();
            if let Some(watcher) = &mut watcher {
                watcher
                    .watcher()
                    .watch(&wgsl_dir, RecursiveMode::Recursive)
                    .ok();
                watcher.cache().add_root(wgsl_dir, RecursiveMode::Recursive);
            }
            watcher
        };
        self.watcher = watcher;
    }

    pub fn load(&self, device: &Device) -> ShaderModule {
        let mut source = String::new();
        for path in Shader::iter() {
            if let Some(file) = Shader::get(&path) {
                source += std::str::from_utf8(file.data.as_ref()).unwrap();
            }
        }
        device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shader"),
            source: ShaderSource::Wgsl(source.into()),
        })
    }

    pub fn try_reload(&mut self, device: &Device) -> Option<ShaderModule> {
        if !self.changed.load(Ordering::SeqCst) {
            return None;
        }
        self.changed.store(false, Ordering::SeqCst);
        Some(self.load(device))
    }
}
