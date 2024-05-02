mod font;
mod glyph;
mod graphics;
mod quad;
mod shape;

use futures::executor::block_on;
use glam::{vec2, vec4, Vec2, Vec4};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use font::Font;
use graphics::GraphicsState;

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut gfx = block_on(GraphicsState::new(&window));

    let font = Font::from_name("Courier New").unwrap();

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::RedrawRequested(_) => {
                gfx.clear();

                gfx.add_quad(vec2(0., 25.), vec2(300., 25.), vec4(1., 0., 0., 1.));

                for i in 0..5000 {
                    gfx.add_text(
                        font.as_ref().unwrap(),
                        "Hello, world!",
                        vec2(rand::random::<f32>() * 800., rand::random::<f32>() * 600.),
                        32.0,
                        vec4(0.0, 0.0, 0.0, 1.0),
                    );
                }
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => {}
            },
            _ => {}
        };

        gfx.handle_event(&window, &event, control_flow);
    });
}
