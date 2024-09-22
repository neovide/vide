use std::{fs::File, io::Read, path::Path, sync::Arc};

use futures::executor::block_on;

use notify::{recommended_watcher, RecursiveMode, Watcher};
use parking_lot::RwLock;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalPosition,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowAttributes},
};

use vide::{Scene, WinitRenderer};

async fn create_renderer(window: Arc<Window>) -> WinitRenderer {
    WinitRenderer::new(window)
        .await
        .with_default_drawables()
        .await
}

struct App {
    scene: Arc<RwLock<Scene>>,
    renderer: Option<WinitRenderer>,
    mouse_pos: PhysicalPosition<f64>,
}

impl App {
    fn new(scene: Arc<RwLock<Scene>>) -> Self {
        App {
            scene,
            renderer: None,
            mouse_pos: PhysicalPosition::default(),
        }
    }
}

impl ApplicationHandler for App {
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_pos = position;
                self.renderer.as_ref().unwrap().window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let scene = self.scene.read();
                self.renderer.as_mut().unwrap().draw(&scene);
            }
            WindowEvent::Resized(new_size) => {
                self.renderer
                    .as_mut()
                    .unwrap()
                    .resize(new_size.width, new_size.height);
            }
            _ => {}
        }
    }

    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.renderer.is_none() {
            let attributes = WindowAttributes::default();
            let window = Arc::new(
                event_loop
                    .create_window(attributes)
                    .expect("Failed to create window"),
            );
            self.renderer = Some(block_on(create_renderer(window)));
        } else {
            self.renderer.as_mut().unwrap().resumed();
        }
    }

    fn suspended(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        self.renderer.as_mut().unwrap().suspended();
    }

    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, _event: ()) {
        self.renderer.as_ref().unwrap().window.request_redraw();
    }
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().expect("Couldn't create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let scene: Arc<RwLock<Scene>> = Arc::new(RwLock::new(Scene::new()));
    let scene_path = Arc::from(Path::new("./scene.json"));
    read_scene(&scene_path, &scene);

    let mut watcher = recommended_watcher({
        let scene_path = scene_path.clone();
        let event_loop = event_loop.create_proxy();
        let scene = scene.clone();
        move |event| {
            if let Ok(notify::event::Event {
                kind: notify::event::EventKind::Modify(_),
                ..
            }) = event
            {
                read_scene(&scene_path, &scene);
                event_loop.send_event(()).unwrap();
            }
        }
    })
    .expect("Could not watch scene file");

    watcher
        .watch(&scene_path, RecursiveMode::NonRecursive)
        .unwrap();

    let mut app = App::new(scene);
    event_loop.run_app(&mut app).ok();
}

fn read_scene(path: &Path, scene: &RwLock<Scene>) {
    let mut file = File::open(path).expect("Could not read file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Could not read file");

    let new_scene: Scene = serde_json::from_str(&contents).expect("Could not parse scene file");

    let mut scene = scene.write();
    *scene = new_scene;
}
