use bedrock::*;
use glam::*;

// - Connect points snap to corners, box middles, and existing connect points
// - On drag, show a radial menu with line types
//   - If no direction is chosen, cancel the connection
//   - Once a direction is chosen, the line starts drawing
//   - Each option is rendered as a path cut off slice of the circle centered
//     on the click point
//   - The closest option's box is brighter and slightly larger
// - On release if the line is not connected to anything, create a new box
//   and focus the text input
// - Clicking the center of a box focuses the text input
// - Escape clears the selection
// - Only support text input and backspace. No arrow keys or mouse or anything else.
// - When text input is focused, the box is highlighted

pub struct App {
    pub mouse_position: Vec2,
    pub window_size: Vec2,
}

impl App {
    pub fn new() -> App {
        App {
            mouse_position: vec2(0., 0.),
            window_size: vec2(0., 0.),
        }
    }

    pub fn update(&mut self) {}

    pub fn draw(&self) -> Scene {
        Scene::new()
            .with_background(vec4(1., 1., 0., 1.))
            .with_quad(Quad {
                top_left: self.mouse_position,
                size: self.window_size / 10.,
                color: vec4(0., 1., 0., 1.),
            })
            .with_quad(Quad {
                top_left: self.window_size / 2. - self.window_size / 20.,
                size: self.window_size / 10.,
                color: vec4(1., 0., 0., 1.),
            })
    }
}
