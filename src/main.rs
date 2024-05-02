mod graphics;

use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
use futures::executor::block_on;
use glam::Vec3;

use shader::ShaderConstants;
use graphics::GraphicsState;

struct State {
    size: winit::dpi::PhysicalSize<u32>,
}

impl State {
    fn new(window: &Window) -> Self {
        let size = window.inner_size();

        Self {
            size,
        }
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.size = new_size;
    }

    fn update(&mut self) {
        let camera_forward_vec = 
            Vec3::Z;

        let up_vec = Vec3::Y;
        let right_vec = camera_forward_vec.cross(Vec3::Y);
        let forward_vec = up_vec.cross(right_vec);
    }

    fn construct_constants(&mut self) -> ShaderConstants {
        ShaderConstants {
            pixel_width: self.size.width,
            pixel_height: self.size.height,
            screen_width: 0.1,
            screen_height: 0.1,
            screen_depth: 0.1,
        }
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut graphics_state = block_on(GraphicsState::new(&window));
    let mut game_state = State::new(&window);

    event_loop.run(move |event, _, control_flow| {
        graphics_state.handle_event(&window, &event, control_flow, || game_state.construct_constants());

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) => {
                    game_state.resize(*physical_size);
                },
                _ => {}
            },
            Event::RedrawRequested(_) => {
                game_state.update();
            },
            _ => {}
        };
    });
}
