use std::{env::temp_dir, fs::create_dir_all, path::PathBuf, thread};

use git2::Repository;
use glamour::{point2, size2, vec2, Rect};
use image::io::Reader as ImageReader;
use lazy_static::lazy_static;
use palette::Srgba;
use parley::style::{FontFamily, FontSettings, FontStack, FontWeight, StyleProperty};
use rust_embed::RustEmbed;
use swash::Setting;

use crate::{
    offscreen_renderer::OffscreenRenderer, scene::Scene, Layer, Path, Quad, Shaper, Sprite, Texture,
};

#[derive(RustEmbed)]
#[folder = "test_data/assets"]
struct Assets;

lazy_static! {
    static ref TEMP_DIR: PathBuf = temp_dir();
    static ref GIT_USER: String = {
        let repo = Repository::open_from_env().expect("Could not read git repository");
        let config = repo.config().expect("Could not read config for repo");
        config
            .get_string("user.name")
            .expect("Could not read user name")
    };
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
    let user_test_data_path = format!("./test_data/{}/", *GIT_USER);
    create_dir_all(&user_test_data_path).unwrap();

    let expected_path = format!("{}/{}.png", user_test_data_path, test_name);
    let expected = ImageReader::open(&expected_path).ok().map(|reader| {
        reader
            .decode()
            .expect("Could not decode regression image")
            .into_rgba8()
    });

    let actual = smol::block_on(async {
        let mut renderer = OffscreenRenderer::new(width, height)
            .await
            .with_default_drawables()
            .await;
        renderer.draw(&scene).await
    });

    if let Some(expected) = expected {
        // Compare the actual image to the expected baseline. If they are not the same,
        // write the diff image to a temp directory and print the file path
        let result = image_compare::rgba_hybrid_compare(&expected, &actual)
            .expect("Images had different dimensions");
        if result.score != 1.0 {
            let temp_dir = TEMP_DIR.join("vide");
            create_dir_all(&temp_dir).unwrap();
            let diff_path = temp_dir.join(format!("{}_diff.png", test_name));
            let actual_path = temp_dir.join(format!("{}_actual.png", test_name));
            let diff_image = result.image.to_color_map();
            diff_image.save(&diff_path).unwrap();
            actual.save(&actual_path).unwrap();
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
    let mut scene = Scene::new().with_background(Srgba::new(1., 1., 1., 1.));
    let mut shaper = Shaper::new();
    shaper.push_default(StyleProperty::FontStack(FontStack::Source("monospace")));
    shaper.push_default(StyleProperty::Brush(Srgba::new(0., 0., 0., 1.)));

    for i in 0..20 {
        let font_size = 8. + 2. * i as f32;
        let y = 0.2366 * font_size * font_size - 0.3481 * font_size - 11.1405;

        let layout = shaper.layout_with("Sphinx of black quartz judge my vow.", |builder| {
            builder.push_default(&StyleProperty::FontSize(font_size));
        });

        scene.add_text_layout(layout, point2!(0., y));
    }

    assert_eq!(scene.resources.fonts.len(), 1);

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
    let image_file = Assets::get("Leaf.png").unwrap();
    let image = image::load_from_memory(image_file.data.as_ref()).unwrap();
    let texture = Texture::from_image(image);

    let scene =
        Scene::new().with_sprite(Sprite::new(texture, point2!(10., 10.), size2!(100., 100.)));

    assert_no_regressions(120, 120, scene);
}

#[test]
fn simple_blur() {
    let mut scene = Scene::new();
    let mut shaper = Shaper::new();
    shaper.push_default(StyleProperty::FontStack(FontStack::Source("monospace")));
    shaper.push_default(StyleProperty::Brush(Srgba::new(0., 0., 0., 1.)));

    for i in 0..20 {
        let bottom = 15. * i as f32;
        let layout = shaper.layout_with("TestTestTestTestTestTestTestTest", |builder| {
            builder.push_default(&StyleProperty::FontSize(15.));
        });
        scene.add_text_layout(layout, point2!(0., bottom));
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

#[test]
fn overlapping_quads() {
    let mut scene = Scene::new();
    let colors = [
        Srgba::new(1., 0., 0., 0.5),
        Srgba::new(1., 1., 0., 0.5),
        Srgba::new(0., 1., 0., 0.5),
        Srgba::new(0., 1., 1., 0.5),
        Srgba::new(0., 0., 1., 0.5),
    ];

    for (i, color) in colors.into_iter().enumerate() {
        scene.add_quad(Quad::new(
            point2!(10., 10.) + vec2!(i as f32 * 10., i as f32 * 10.),
            size2!(50., 50.),
            color,
        ));
    }

    assert_no_regressions(110, 110, scene);
}

#[test]
fn swash_modern_ligatures() {
    let mut scene = Scene::new();
    let mut shaper = Shaper::new();

    let layout = shaper.layout_with("a==b a//b a~~b", |builder| {
        builder.push_default(&StyleProperty::FontStack(FontStack::Single(
            FontFamily::Named("Monaspace Krypton Var"),
        )));
        builder.push_default(&StyleProperty::Brush(Srgba::new(0., 0., 0., 1.)));
        builder.push_default(&StyleProperty::FontSize(16.));
        let features = [
            "calt", "ss01", "ss02", "ss03", "ss04", "ss05", "ss06", "ss07", "ss08", "ss09", "liga",
        ];
        builder.push_default(&StyleProperty::FontFeatures(FontSettings::List(
            &features
                .into_iter()
                .map(|feature| (feature, 1).into())
                .collect::<Vec<Setting<u16>>>(),
        )));
    });
    scene.add_text_layout(layout, point2!(5., 5.));

    assert_no_regressions(200, 30, scene);
}

#[test]
fn text_layout_bounds() {
    let mut scene = Scene::new().with_background(Srgba::new(1., 1., 1., 1.));
    let mut shaper = Shaper::new();

    let layout = shaper.layout_with("Sphinx of black quartz judge my vow.", |builder| {
        builder.push_default(&StyleProperty::FontStack(FontStack::Source("monospace")));
        builder.push_default(&StyleProperty::Brush(Srgba::new(0., 0., 0., 1.)));
        builder.push_default(&StyleProperty::FontSize(14.0));
    });

    scene.add_quad(Quad::new(
        point2!(10., 10.),
        size2!(layout.width(), layout.height()),
        Srgba::new(0., 1., 0., 0.5),
    ));
    scene.add_text_layout(layout, point2!(10., 10.));

    assert_no_regressions(325, 35, scene);
}

#[test]
fn parley_line_breaking_and_font_fallback() {
    let mut scene = Scene::new();
    let mut shaper = Shaper::new();

    let padding = 10.;
    let layout = shaper.layout_within_with(
        "Some text here. Let's make it a bit longer so that line wrapping kicks in 😊. And also some اللغة العربية arabic text.",
        400.,
        |builder| {
            builder.push_default(&StyleProperty::FontStack(FontStack::Source("serif")));
            builder.push_default(&StyleProperty::Brush(Srgba::new(0., 0., 0., 1.)));
            builder.push_default(&StyleProperty::FontSize(24.));

            builder.push(&StyleProperty::FontWeight(FontWeight::new(600.)), 0..4);
        });

    let layout_width = layout.width();
    let layout_height = layout.height();
    scene.add_text_layout(layout, point2!(padding, padding));

    assert_no_regressions(
        (layout_width + padding * 2.) as u32,
        (layout_height + padding * 2.) as u32,
        scene,
    );
}

#[test]
fn complex_mask_clips_properly() {
    let mut scene = Scene::new();

    let colors = [
        Srgba::new(1., 0., 0., 0.5),
        Srgba::new(1., 1., 0., 0.5),
        Srgba::new(0., 1., 0., 0.5),
        Srgba::new(0., 1., 1., 0.5),
        Srgba::new(0., 0., 1., 0.5),
    ];

    for (i, color) in colors.into_iter().enumerate() {
        scene.add_quad(Quad::new(
            point2!(10., 10.) + vec2!(i as f32 * 20., i as f32 * 20.),
            size2!(100., 100.),
            color,
        ));
    }

    let mut mask_layer = Layer::new();
    let mut shaper = Shaper::new();
    shaper.push_default(StyleProperty::FontStack(FontStack::Source("monospace")));
    shaper.push_default(StyleProperty::Brush(Srgba::new(0., 0., 0., 1.)));

    for i in 0..20 {
        let bottom = 20. * i as f32;
        let layout = shaper.layout_with("TestTestTestTestTestTestTestTest", |builder| {
            builder.push_default(&StyleProperty::FontSize(20.));
        });
        mask_layer.add_text_layout(&mut scene.resources, layout, point2!(0., bottom));
    }

    scene.set_mask(mask_layer);

    assert_no_regressions(210, 210, scene);
}
