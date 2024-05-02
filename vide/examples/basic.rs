use std::{fs::File, io::Read, path::Path, sync::Arc};

use futures::executor::block_on;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use vide::{Scene, Vide};

fn main() {
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut vide = Vide::new(Box::new(|_| {}), &window);

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => {}
            },
            Event::RedrawRequested(_) => {
                vide.draw();
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        };

        vide.handle_event(&window, &event);
    });
}
