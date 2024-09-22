//! ```cargo
//! [dependencies]
//! vide = { path = "./" }
//! serde = "1.0.196"
//! serde_derive = "1.0.196"
//! serde_json = "1.0.113"
//! ```
extern crate serde_json;
extern crate vide;

use vide::glamour::{point2, size2, vec2, Rect};
use vide::palette::Srgba;
use vide::parley::style::{FontStack, StyleProperty};
use vide::*;

fn main() {
    let mut scene = Scene::new();
    let mut shaper = Shaper::new();
    shaper.push_default(StyleProperty::FontStack(FontStack::Source("monospace")));
    shaper.push_default(StyleProperty::Brush(Srgba::new(0., 0., 0., 1.)));

    for i in 0..20 {
        let font_size = 8. + 2. * i as f32;
        let bottom = 0.2366 * font_size * font_size - 0.3481 * font_size - 1.1405;

        let layout = shaper.layout_with("Sphinx of black quartz judge my vow.", |builder| {
            builder.push_default(&StyleProperty::FontSize(font_size));
        });
        scene.add_text_layout(&layout, point2!(0., bottom));
    }

    scene.add_layer(
        Layer::new()
            .with_clip(Rect::new(point2!(0, 0), size2!(200, 200)))
            .with_blurred_clear(Srgba::new(0.0, 0.0, 0.0, 0.5), 3.)
            .with_path(
                Path::new(point2!(20., 20.))
                    .with_fill(Srgba::new(0., 1., 0., 1.))
                    .with_stroke(5., Srgba::new(0., 0., 0., 1.))
                    .with_line_to(point2!(180., 20.))
                    .with_quadratic_bezier_to(point2!(180., 180.), point2!(20., 180.)),
            )
            .with_quad(Quad::new(
                Rect::new(point2!(150., 100.), size2!(100., 100.)),
                Srgba::new(1., 0., 0., 1.),
            )),
    );

    scene.add_layer(Default::default());

    let colors = [
        Srgba::new(1., 0., 0., 0.5),
        Srgba::new(1., 1., 0., 0.5),
        Srgba::new(0., 1., 0., 0.5),
        Srgba::new(0., 1., 1., 0.5),
        Srgba::new(0., 0., 1., 0.5),
    ];

    for (i, color) in colors.iter().enumerate() {
        scene.add_quad(Quad::new(
            Rect::new(
                point2!(500., 10.) + vec2!(i as f32 * 10., i as f32 * 10.),
                size2!(50., 50.),
            ),
            *color,
        ));
    }

    let mut mask_layer = Layer::new();
    shaper.clear_defaults();
    shaper.push_default(StyleProperty::FontStack(FontStack::Source("monospace")));
    shaper.push_default(StyleProperty::Brush(Srgba::new(0., 0., 0., 1.)));

    for i in 0..20 {
        let bottom = 15. * i as f32;
        let layout = shaper.layout_with("TestTestTestTestTestTestTestTest", |builder| {
            builder.push_default(&StyleProperty::FontSize(15.));
        });
        mask_layer.add_text_layout(&mut scene.resources, &layout, point2!(500., bottom));
    }

    scene.set_mask(mask_layer);

    // Serialize the scene to a json string and write the string to ./scene.json
    let scene_json = serde_json::to_string_pretty(&scene).unwrap();
    std::fs::write("scene.json", scene_json).unwrap();

    println!("Debug scene written to scene.json");
}
