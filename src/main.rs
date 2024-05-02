mod font;
mod glyph;
mod graphics;
mod quad;
mod scene;
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
                let font_ref = font.as_ref().unwrap();
                let glyph = font_ref.charmap().map('A');
                gfx.add_quad(vec2(0., 50.), vec2(32., 32.), vec4(1.0, 0.0, 0.0, 1.0));
                gfx.add_glyph(
                    font.as_ref().unwrap(),
                    glyph,
                    vec2(0., 50.),
                    32.0,
                    vec4(0.0, 0.0, 0.0, 1.0),
                );
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => {}
            },
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        };

        gfx.handle_event(&window, &event, control_flow);
    });
}
