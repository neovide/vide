use std::{fs::File, io::Read, path::Path, sync::Arc};

use futures::executor::block_on;

use glam::{vec2, vec4, Vec4};
use notify::{recommended_watcher, RecursiveMode, Watcher};
use parking_lot::RwLock;
use rust_embed::RustEmbed;
use winit::{
    dpi::PhysicalPosition,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use bedrock::{Quad, Renderer, Scene, Sprite};

#[derive(RustEmbed)]
#[folder = "assets"]
struct Assets;

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

    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut renderer = block_on(Renderer::new(&window)).with_default_drawables::<Assets>();
    let mut mouse_pos: PhysicalPosition<f64> = Default::default();

    event_loop
        .run(|event, target| {
            match event {
                Event::NewEvents(_) => {
                    let mut scene = scene.read().clone();
                    scene.add_quad(
                        Quad::new(
                            vec2(mouse_pos.x as f32, mouse_pos.y as f32),
                            vec2(100., 100.),
                            vec4(0., 0., 0., 0.5),
                        )
                        .with_corner_radius(30.0)
                        .with_blur(10.0),
                    );

                    renderer.draw_scene(&scene);
                }
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == window.id() => match event {
                    WindowEvent::CloseRequested => target.exit(),
                    WindowEvent::CursorMoved { position, .. } => {
                        mouse_pos = *position;
                    }
                    _ => {}
                },
                Event::UserEvent(()) => {
                    window.request_redraw();
                }
                _ => {}
            };

            renderer.handle_event(&window, &event);
        })
        .expect("Could not run event loop");
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
