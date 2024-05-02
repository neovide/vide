use bedrock::*;
use glam::*;
use lazy_static::lazy_static;

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

fn hex_to_vec(hex: &str) -> Vec4 {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap() as f32 / 255.;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap() as f32 / 255.;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap() as f32 / 255.;
    let a = if hex.len() == 8 {
        u8::from_str_radix(&hex[6..8], 16).unwrap() as f32 / 255.
    } else {
        1.
    };
    vec4(r, g, b, a)
}

lazy_static! {
    static ref BACKGROUND_DIM: Vec4 = hex_to_vec("#1e2326");
    static ref BACKGROUND0: Vec4 = hex_to_vec("#272e33");
    static ref BACKGROUND1: Vec4 = hex_to_vec("#2e383c");
    static ref BACKGROUND2: Vec4 = hex_to_vec("#374145");
    static ref BACKGROUND3: Vec4 = hex_to_vec("#414b50");
    static ref BACKGROUND4: Vec4 = hex_to_vec("#495156");
    static ref BACKGROUND5: Vec4 = hex_to_vec("#4f5b58");
    static ref BACKGROUND_RED: Vec4 = hex_to_vec("#4c3743");
    static ref BACKGROUND_VISUAL: Vec4 = hex_to_vec("#493B40");
    static ref BACKGROUND_YELLOW: Vec4 = hex_to_vec("#45443C");
    static ref BACKGROUND_GREEN: Vec4 = hex_to_vec("#3C4841");
    static ref BACKGROUND_BLUE: Vec4 = hex_to_vec("#384B55");
    static ref RED: Vec4 = hex_to_vec("#e67e80");
    static ref ORANGE: Vec4 = hex_to_vec("#e69875");
    static ref YELLOW: Vec4 = hex_to_vec("#dbbc7f");
    static ref GREEN: Vec4 = hex_to_vec("#a7c080");
    static ref BLUE: Vec4 = hex_to_vec("#7fbbb4");
    static ref AQUA: Vec4 = hex_to_vec("#83C092");
    static ref PURPLE: Vec4 = hex_to_vec("#d699b6");
    static ref FOREGROUND: Vec4 = hex_to_vec("#d3c6aa");
    static ref STATUSLINE_1: Vec4 = hex_to_vec("#a7c080");
    static ref STATUSLINE_2: Vec4 = hex_to_vec("#d3c6aa");
    static ref STATUSLINE_3: Vec4 = hex_to_vec("#e67e80");
    static ref GRAY_0: Vec4 = hex_to_vec("#7a8478");
    static ref GRAY_1: Vec4 = hex_to_vec("#859289");
    static ref GRAY_2: Vec4 = hex_to_vec("#9da9a0");
}

pub struct App {
    pub mouse_position: Vec2,
    pub window_size: Vec2,

    pub offset: Vec2,
}

impl App {
    pub fn new() -> App {
        App {
            mouse_position: vec2(0., 0.),
            window_size: vec2(0., 0.),

            offset: vec2(0., 0.),
        }
    }

    pub fn update(&mut self) {}

    pub fn draw(&self) -> Scene {
        let mut scene = Scene::new().with_background(*BACKGROUND_DIM);

        let mut x = self.offset.x % 50.;
        loop {
            let mut y = self.offset.y % 50.;
            loop {
                scene = scene.with_quad(Quad {
                    top_left: vec2(x, y),
                    size: vec2(2., 2.),
                    color: *BACKGROUND3,
                });
                y += 50.;
                if y > self.window_size.y {
                    break;
                }
            }
            x += 50.;
            if x > self.window_size.x {
                break;
            }
        }

        scene
    }
}
