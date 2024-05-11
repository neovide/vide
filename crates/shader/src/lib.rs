mod error;
mod glyph;
mod path;
mod quad;
mod sprite;

pub use glyph::*;
pub use path::*;
pub use quad::*;
pub use sprite::*;

use error::ErrorLogger;

use std::{
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
    naga::{FastHashMap, ShaderStage},
    Device, ErrorFilter, ShaderModule, ShaderModuleDescriptor, ShaderSource,
};

#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct ShaderConstants {
    pub surface_size: Vec2,
    pub atlas_size: Vec2,
    pub clip: Vec4,
}

#[derive(Default)]
pub struct ShaderModules {
    vertex: HashMap<String, ShaderModule>,
    fragment: HashMap<String, ShaderModule>,
    compute: HashMap<String, ShaderModule>,
}

impl ShaderModules {
    pub fn get_vertex(&self, name: &str) -> Result<&ShaderModule, String> {
        self.vertex
            .get(name)
            .ok_or_else(|| format!("Vertex shader '{}' not found!", name))
    }

    pub fn get_fragment(&self, name: &str) -> Result<&ShaderModule, String> {
        self.fragment
            .get(name)
            .ok_or_else(|| format!("Fragment shader '{}' not found!", name))
    }

    pub fn get_compute(&self, name: &str) -> Result<&ShaderModule, String> {
        self.compute
            .get(name)
            .ok_or_else(|| format!("Compute shader '{}' not found!", name))
    }
}

#[derive(RustEmbed)]
#[folder = "glsl/"]
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

    pub async fn load(&self, device: &Device) -> ShaderModules {
        //device.push_error_scope(ErrorFilter::Internal);
        let mut modules = ShaderModules::default();
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
                let stage = match ext {
                    "vert" => Some(ShaderStage::Vertex),
                    "frag" => Some(ShaderStage::Fragment),
                    "comp" => Some(ShaderStage::Compute),
                    _ => None,
                };
                if let Some(stage) = stage {
                    let label = format!("{}_{}", &name, &ext).to_string();
                    let descriptor = ShaderModuleDescriptor {
                        label: Some(&label),
                        source: ShaderSource::Glsl {
                            shader: std::str::from_utf8(file.data.as_ref()).unwrap().into(),
                            stage,
                            defines: FastHashMap::default(),
                        },
                    };
                    let module = device.create_shader_module(descriptor);
                    if let Some(error) = device.pop_error_scope().await {
                        error.log_errors(path);
                    } else {
                        match stage {
                            ShaderStage::Vertex => modules.vertex.insert(name.to_string(), module),
                            ShaderStage::Fragment => {
                                modules.fragment.insert(name.to_string(), module)
                            }
                            ShaderStage::Compute => {
                                modules.compute.insert(name.to_string(), module)
                            }
                        };
                    }
                };
            }
        }
        modules
    }

    pub fn try_reload(&mut self, device: &Device) -> Option<ShaderModules> {
        if !self.changed.load(Ordering::SeqCst) {
            return None;
        }
        self.changed.store(false, Ordering::SeqCst);
        // Internally block instead of making try_reload async to avoid taking a performance hit
        // during normal rendering
        Some(block_on(self.load(device)))
    }
}
