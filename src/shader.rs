mod error;
mod preprocessor;

use error::ErrorLogger;
use preprocessor::Preprocessor;

use std::{
    borrow::Cow,
    collections::HashMap,
    ffi::OsStr,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use futures::executor::block_on;
use glam::*;
use notify_debouncer_full::{
    new_debouncer,
    notify::{RecommendedWatcher, RecursiveMode, Watcher},
    DebounceEventResult, DebouncedEvent, Debouncer, FileIdMap,
};
use rust_embed::*;
use wgpu::{
    Device, ErrorFilter, ShaderModule, ShaderModuleDescriptor, ShaderSource,
};

#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct ShaderConstants {
    pub surface_size: Vec2,
    pub atlas_size: Vec2,
}

#[derive(RustEmbed)]
#[folder = "shaders/"]
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
        let wgsl_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("glsl");
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

    pub async fn load(&self, device: &Device) -> HashMap<String, ShaderModule> {
        let mut modules = HashMap::new();
        for path in Shader::iter() {
            if let Some(file) = Shader::get(&path) {
                device.push_error_scope(ErrorFilter::Validation);
                let os_str: &OsStr = OsStr::new(path.as_ref());
                let path = Path::new(os_str);
                let ext = path
                    .extension()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default();
                let name = path
                    .file_stem()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default();

                let preprocessor = Preprocessor::new(&file.data, path.to_str().unwrap());

                let label = format!("{}_{}", &name, &ext).to_string();
                let descriptor = ShaderModuleDescriptor {
                    label: Some(&label),
                    source: ShaderSource::Wgsl(Cow::from(&preprocessor.content))
                };
                let module = device.create_shader_module(descriptor);
                if let Some(error) = device.pop_error_scope().await {
                    error.log_errors(&preprocessor);
                } else {
                    modules.insert(name.to_string(), module);
                }
            }
        }
        modules
    }

    pub fn try_reload(&mut self, device: &Device) -> Option<HashMap<String, ShaderModule>> {
        if !self.changed.load(Ordering::SeqCst) {
            return None;
        }
        self.changed.store(false, Ordering::SeqCst);
        // Internally block instead of making try_reload async to avoid taking a performance hit
        // during normal rendering
        Some(block_on(self.load(device)))
    }
}
