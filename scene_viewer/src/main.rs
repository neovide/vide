use std::{fs::File, io::Read, path::Path, sync::Arc};

use futures::executor::block_on;

use notify::{recommended_watcher, RecursiveMode, Watcher};
use parking_lot::RwLock;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use bedrock::{Renderer, Scene};

fn main() {
    let event_loop = EventLoop::new();

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

    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut renderer = block_on(Renderer::new(&window));

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::RedrawRequested(_) => {
                renderer.draw_scene(&scene.read());
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => {}
            },
            Event::UserEvent(()) => {
                window.request_redraw();
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        };

        renderer.handle_event(&window, &event);
    });
}

fn read_scene(path: &Path, scene: &RwLock<Scene>) {
    let mut file = File::open(&path).expect("Could not read file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Could not read file");

    let new_scene: Scene = serde_json::from_str(&contents).expect("Could not parse scene file");

    let mut scene = scene.write();
    *scene = new_scene;
}
