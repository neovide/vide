mod app;

use futures::executor::block_on;

use glam::*;
use rust_embed::RustEmbed;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use app::App;
use bedrock::Renderer;

#[derive(RustEmbed)]
#[folder = "assets"]
struct Assets;

fn main() {
    let event_loop = EventLoop::new().expect("Could not create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut renderer = block_on(Renderer::new(&window)).with_default_drawables::<Assets>();

    let mut app = App::new();

    event_loop
        .run(|event, target| {
            match event {
                Event::NewEvents(_) => {
                    app.update();
                    renderer.draw_scene(&app.draw());
                }
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == window.id() => match event {
                    WindowEvent::CloseRequested => target.exit(),
                    WindowEvent::CursorMoved { position, .. } => {
                        app.mouse_position = Vec2::new(position.x as f32, position.y as f32)
                    }
                    _ => {}
                },
                Event::UserEvent(()) => {
                    window.request_redraw();
                }
                _ => {}
            };

            renderer.handle_event(&window, &event);

            let window_size = window.inner_size();
            app.window_size = Vec2::new(window_size.width as f32, window_size.height as f32);
        })
        .expect("Could not run event loop");
}
