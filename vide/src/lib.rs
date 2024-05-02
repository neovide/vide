mod commands;

use std::time::Instant;

use futures::executor::block_on;
use winit::{event::Event, window::Window};

use bedrock::Renderer;
pub use bedrock::Scene;
pub use commands::*;

pub struct Vide {
    request_frame: Box<dyn Fn(RequestFrame)>,
    renderer: Renderer,
}

impl Vide {
    pub fn new(request_frame: Box<dyn Fn(RequestFrame)>, window: &Window) -> Self {
        Self {
            request_frame,
            renderer: block_on(Renderer::new(window)),
        }
    }

    pub fn handle_draw_command(command: DrawCommand) {}

    pub fn draw(&mut self) -> bool {
        self.renderer.draw_scene(&Scene::new())
    }

    pub fn handle_event(&mut self, window: &Window, event: &Event<()>) {
        self.renderer.handle_event(window, event);
    }
}

pub enum RequestFrame {
    Next,
    Future(Instant),
}
