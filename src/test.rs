use std::{path::PathBuf, thread};

use glamour::{point2, size2, vec2, Rect};
use image::io::Reader as ImageReader;
use lazy_static::lazy_static;
use palette::Srgba;
use rust_embed::RustEmbed;

use crate::{offscreen_renderer::OffscreenRenderer, scene::Scene, Layer, Path, Quad, Sprite, Text};

#[derive(RustEmbed)]
#[folder = "test_data/assets"]
struct Assets;

lazy_static! {
    static ref TEMP_DIR: PathBuf = std::env::temp_dir();
}

fn assert_no_regressions(width: u32, height: u32, scene: Scene) {
    let thread = thread::current();
    let test_name = thread
        .name()
        .unwrap()
        .split(':')
        .last()
        .unwrap()
        .to_string();
    let expected_path = format!("./test_data/{}.png", test_name);
    let expected = ImageReader::open(&expected_path).ok().map(|reader| {
        reader
            .decode()
            .expect("Could not decode regression image")
            .into_rgba8()
    });

    let actual = smol::block_on(async {
        let mut renderer = OffscreenRenderer::new(width, height)
            .await
            .with_default_drawables::<Assets>();
        renderer.draw(&scene).await
    });

    if let Some(expected) = expected {
        // Compare the actual image to the expected baseline. If they are not the same,
        // write the diff image to a temp directory and print the file path
        let result = image_compare::rgba_hybrid_compare(&expected, &actual)
            .expect("Images had different dimensions");
        if result.score != 1.0 {
            let diff_path = TEMP_DIR.join(format!("{}.png", test_name));
            let diff_image = result.image.to_color_map();
            diff_image.save(&diff_path).unwrap();
            panic!(
                "Regression detected. Diff image saved to {}",
                diff_path.display()
            );
        }
    } else {
        // No baseline file exists. Write the actual to disk as the new baseline
        actual.save(&expected_path).unwrap();
    }
}

#[test]
fn simple_quad() {
    assert_no_regressions(
        70,
        70,
        Scene::new()
            .with_background(Srgba::new(1., 0., 0.5, 1.))
            .with_quad(Quad::new(
                point2!(10., 10.),
                size2!(50., 50.),
                Srgba::new(0., 0., 1., 1.),
            )),
    );
}

#[test]
fn simple_text() {
    let positions = [
        (10., 8.),
        (20., 10.),
        (30., 12.),
        (40., 14.),
        (55., 16.),
        (70., 18.),
        (85., 20.),
        (105., 22.),
        (125., 24.),
        (150., 26.),
        (175., 28.),
        (200., 30.),
        (230., 32.),
        (260., 34.),
        (295., 36.),
        (330., 38.),
        (365., 40.),
        (400., 42.),
        (440., 44.),
    ];

    let mut scene = Scene::new().with_background(Srgba::new(1., 1., 1., 1.));
    for (y, size) in positions.into_iter() {
        scene.add_text(Text::new(
            "Sphinx of black quartz, judge my vow.".to_owned(),
            point2!(0., y),
            size,
            Srgba::new(0., 0., 0., 1.),
        ));
    }

    assert_no_regressions(1000, 500, scene);
}

#[test]
fn simple_path() {
    let scene = Scene::new().with_path(
        Path::new(point2!(20., 20.))
            .with_fill(Srgba::new(0., 1., 0., 1.))
            .with_stroke(5., Srgba::new(0., 0., 0., 1.))
            .line_to(point2!(180., 20.))
            .quadratic_bezier_to(point2!(180., 180.), point2!(20., 180.)),
    );

    assert_no_regressions(200, 200, scene);
}

#[test]
fn simple_sprite() {
    let scene = Scene::new().with_sprite(Sprite::new(
        "Leaf.png".to_owned(),
        point2!(10., 10.),
        size2!(100., 100.),
    ));

    assert_no_regressions(120, 120, scene);
}

#[test]
fn simple_blur() {
    let mut scene = Scene::new();

    for i in 0..20 {
        scene.add_text(Text::new(
            "TestTestTestTestTestTestTestTest".to_owned(),
            point2!(0., 15. * i as f32),
            15.,
            Srgba::new(0., 0., 0., 1.),
        ));
    }

    for x in 0..3 {
        for y in 0..3 {
            scene.add_layer(
                Layer::new()
                    .with_blur(2.)
                    .with_clip(
                        Rect::new(point2!(15, 15), size2!(50, 50)).translate(vec2!(x * 60, y * 60)),
                    )
                    .with_background(Srgba::new(0., 1., 0., 0.1)),
            );
        }
    }

    assert_no_regressions(200, 200, scene);
}

#[test]
fn simple_blurred_quad() {
    let mut scene = Scene::new();
    for x in 0..5 {
        for y in 0..5 {
            scene.add_quad(
                Quad::new(
                    point2!(15., 15.) + vec2!(x as f32 * 60., y as f32 * 60.),
                    size2!(50., 50.),
                    Srgba::new(x as f32 / 5., y as f32 / 5., 1., 1.),
                )
                .with_corner_radius(x as f32 * 2.)
                .with_blur(y as f32),
            )
        }
    }

    assert_no_regressions(325, 325, scene);
}
