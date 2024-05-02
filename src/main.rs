mod graphics;

use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
use futures::executor::block_on;
use glam::{vec3, Vec3, Quat};

use shader::{terrain, ShaderConstants};
use graphics::GraphicsState;

const FRICTION: f32 = 0.9;
const SPEED: f32 = 0.01;
const SENSITIVITY: f32 = 0.001;

struct State {
    size: winit::dpi::PhysicalSize<u32>,

    horizontal_rotation: f32,
    vertical_rotation: f32,
    position: Vec3,

    input_dir: Vec3,
    velocity: Vec3,

    grabbed: bool,

    left: bool, right: bool,
    forward: bool, back: bool,
    up: bool, down: bool,

    time: f32,
}

impl State {
    fn new(window: &Window) -> Self {
        let size = window.inner_size();

        Self {
            size,
            horizontal_rotation: 0.0,
            vertical_rotation: 0.0,
            position: vec3(0.0, 0.0, terrain(Vec3::ZERO)),

            input_dir: Vec3::ZERO,
            velocity: Vec3::ZERO,

            grabbed: false,

            left: false, right: false,
            forward: false, back: false,
            up: false, down: false,

            time: 0.0,
        }
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.size = new_size;
    }

    fn focus_changed(&mut self, focused: bool) {
        if !focused {
            self.grabbed = false;
        }
    }

    fn mouse_input(&mut self, state: &ElementState) {
        if *state == ElementState::Pressed {
            self.grabbed = true;
        }
    }

    fn mouse_moved(&mut self, (delta_x, delta_y): (f64, f64)) {
        if self.grabbed {
            self.horizontal_rotation -= delta_x as f32 * SENSITIVITY;

            use std::f32::consts;

            let vertical_range = consts::FRAC_PI_2 - consts::FRAC_PI_8 / 4.0;
            self.vertical_rotation += delta_y as f32 * SENSITIVITY;
            self.vertical_rotation = self.vertical_rotation.min(vertical_range).max(-vertical_range);
        }
    }

    fn keyboard_input(&mut self, event: &KeyboardInput) {
        if event.state == ElementState::Pressed {
            match event.virtual_keycode {
                Some(VirtualKeyCode::D) => self.right = true,
                Some(VirtualKeyCode::A) => self.left = true,
                Some(VirtualKeyCode::W) => self.forward = true,
                Some(VirtualKeyCode::S) => self.back = true,
                Some(VirtualKeyCode::E) => self.up = true,
                Some(VirtualKeyCode::C) => self.down = true,
                _ => {}
            }
        } else if event.state == ElementState::Released {
            match event.virtual_keycode {
                Some(VirtualKeyCode::D) => self.right = false,
                Some(VirtualKeyCode::A) => self.left = false,
                Some(VirtualKeyCode::W) => self.forward = false,
                Some(VirtualKeyCode::S) => self.back = false,
                Some(VirtualKeyCode::E) => self.up = false,
                Some(VirtualKeyCode::C) => self.down = false,
                _ => {}
            }
        }
    }

    fn update(&mut self) {
        let forward_vec = 
            Quat::from_rotation_y(self.horizontal_rotation) * 
            Quat::from_rotation_x(self.vertical_rotation) * 
            Vec3::Z;
        let up_vec = Vec3::Y;
        let right_vec = forward_vec.cross(up_vec);

        let horizontal = 
            (if self.left { -1.0 } else { 0.0 } +
            if self.right { 1.0 } else { 0.0 }) * right_vec;

        let vertical = 
            (if self.down { -1.0 } else { 0.0 } +
            if self.up { 1.0 } else { 0.0 }) * up_vec;

        let aligned =
            (if self.back { -1.0 } else { 0.0 } +
            if self.forward { 1.0 } else { 0.0 }) * forward_vec;

        self.input_dir = horizontal + vertical + aligned;

        self.time += 0.005;

        self.velocity += self.input_dir * SPEED;
        self.velocity *= FRICTION;

        self.position += self.velocity;
    }

    fn construct_constants(&mut self) -> ShaderConstants {
        let forward_vec = 
            Quat::from_rotation_y(self.horizontal_rotation) * 
            Quat::from_rotation_x(self.vertical_rotation) * 
            Vec3::Z;

        ShaderConstants {
            pixel_width: self.size.width,
            pixel_height: self.size.height,
            screen_width: 0.1,
            screen_height: 0.1,
            screen_depth: 0.1,
            position: self.position.into(),
            forward: forward_vec.into(),
            time: self.time,
        }
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut graphics_state = block_on(GraphicsState::new(&window));
    let mut game_state = State::new(&window);

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Focused(focused) => {
                if !focused {
                    window.set_cursor_grab(false).expect("Could not grab cursor");
                    window.set_cursor_visible(true);
                }
                game_state.focus_changed(*focused);
            },
            WindowEvent::MouseInput { state, .. } => {
                if *state == ElementState::Pressed {
                    window.set_cursor_grab(true).expect("Could not grab cursor");
                    window.set_cursor_visible(false);
                }
                game_state.mouse_input(state)
            },
            WindowEvent::KeyboardInput { input, .. } => match input {
                KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Escape),
                    ..
                } => {
                    window.set_cursor_grab(false).expect("Could not grab cursor");
                    window.set_cursor_visible(true);
                    game_state.focus_changed(false);
                },
                keyboard_input => game_state.keyboard_input(keyboard_input),
            },
            WindowEvent::Resized(physical_size) => {
                game_state.resize(*physical_size);
                graphics_state.resize(*physical_size);
            },
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                game_state.resize(**new_inner_size);
                graphics_state.resize(**new_inner_size);
            },
            _ => {}
        },
        Event::DeviceEvent {
            event: DeviceEvent::MouseMotion {
                delta
            },
            ..
        } => {
            game_state.mouse_moved(delta);
        },
        Event::RedrawRequested(_) => {
            game_state.update();
            let constants = game_state.construct_constants();
            match graphics_state.render(constants) {
                Ok(_) => {}
                // Recreate the swap_chain if lost
                Err(wgpu::SwapChainError::Lost) => {
                    graphics_state.resize(game_state.size);
                },
                // The system is out of memory, we should probably quit
                Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{:?}", e),
            }
        },
        Event::MainEventsCleared => {
            window.request_redraw();
        }
        _ => {}
    });
}
