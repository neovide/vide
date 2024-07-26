use std::{env::temp_dir, fs::create_dir_all, path::PathBuf, thread};

use git2::Repository;
use glamour::{point2, size2, vec2, Rect};
use image::ImageReader;
use lazy_static::lazy_static;
use palette::Srgba;
use parley::{
    style::{
        FontFamily, FontSettings, FontStack, FontStretch, FontStyle, FontWeight, StyleProperty,
    },
    swash,
    swash::{tag_from_bytes, Attributes, Setting, Stretch, Style, Tag, Weight},
};
use rust_embed::RustEmbed;

use crate::{
    offscreen_renderer::OffscreenRenderer, scene::Scene, scene::Synthesis, Layer, Path, Quad,
    Shaper, Sprite, Texture,
};

const WGHT: Tag = tag_from_bytes(b"wght");
const WDTH: Tag = tag_from_bytes(b"wdth");
const SLNT: Tag = tag_from_bytes(b"slnt");

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
fn font_styles() {
    let mut scene = Scene::new();
    let mut shaper = Shaper::new();

    let padding = 10.;
    #[derive(Debug, PartialEq, Eq)]
    struct Expected {
        fullname: String,
        attributes: Attributes,
        synthesis: Synthesis,
    }
    let lines = vec![
        (
            "FiraCode Normal (Buggy)",
            vec![StyleProperty::FontStack(FontStack::Source(
                "FiraCode Nerd Font",
            ))],
            Expected {
                // FIXME: There's a bug in fontique, the retina style is loaded instead of the regular one
                // See: https://github.com/linebender/parley/issues/92
                fullname: "FiraCode Nerd Font Ret".into(),
                attributes: Attributes::new(Stretch::NORMAL, Weight(450), Style::Normal),
                synthesis: Synthesis {
                    vars: vec![],
                    embolden: false,
                    skew: 0.0.into(),
                },
            },
        ),
        (
            "FiraCode (Native) Bold",
            vec![
                StyleProperty::FontStack(FontStack::Source("FiraCode Nerd Font")),
                StyleProperty::FontWeight(FontWeight::BOLD),
            ],
            Expected {
                fullname: "FiraCode Nerd Font Bold".into(),
                attributes: Attributes::new(Stretch::NORMAL, Weight::BOLD, Style::Normal),
                synthesis: Synthesis {
                    vars: vec![],
                    embolden: false,
                    skew: 0.0.into(),
                },
            },
        ),
        (
            "FiraCode (Faux) Italic (Buggy)",
            vec![
                StyleProperty::FontStack(FontStack::Source("FiraCode Nerd Font")),
                StyleProperty::FontStyle(FontStyle::Italic),
            ],
            Expected {
                // FIXME: There's a bug in fontique, the retina style is loaded instead of the regular one
                // See: https://github.com/linebender/parley/issues/92
                fullname: "FiraCode Nerd Font Ret".into(),
                attributes: Attributes::new(Stretch::NORMAL, Weight(450), Style::Normal),
                synthesis: Synthesis {
                    vars: vec![],
                    embolden: false,
                    skew: 14.0.into(),
                },
            },
        ),
        (
            "FiraCode Oblique 5 degrees (Buggy)",
            vec![
                StyleProperty::FontStack(FontStack::Source("FiraCode Nerd Font")),
                StyleProperty::FontStyle(FontStyle::Oblique(Some(5.0))),
            ],
            Expected {
                // FIXME: There's a bug in fontique, the retina style is loaded instead of the regular one
                // See: https://github.com/linebender/parley/issues/92
                fullname: "FiraCode Nerd Font Ret".into(),
                attributes: Attributes::new(Stretch::NORMAL, Weight(450), Style::Normal),
                synthesis: Synthesis {
                    vars: vec![],
                    embolden: false,
                    skew: 5.0.into(),
                },
            },
        ),
        (
            "FiraCode Synthetic Stretch Wide (Buggy)",
            vec![
                StyleProperty::FontStack(FontStack::Source("FiraCode Nerd Font")),
                StyleProperty::FontStretch(FontStretch::EXPANDED),
            ],
            Expected {
                // FIXME: Fontique does not support synthetic stretch
                fullname: "FiraCode Nerd Font Ret".into(),
                attributes: Attributes::new(Stretch::NORMAL, Weight(450), Style::Normal),
                synthesis: Synthesis {
                    vars: vec![],
                    embolden: false,
                    skew: 0.0.into(),
                },
            },
        ),
        (
            "ProFontWindows Nerd Font",
            vec![StyleProperty::FontStack(FontStack::Source(
                "ProFontWindows Nerd Font",
            ))],
            Expected {
                fullname: "ProFontWindows Nerd Font".into(),
                attributes: Attributes::new(Stretch::NORMAL, Weight::NORMAL, Style::Normal),
                synthesis: Synthesis {
                    vars: vec![],
                    embolden: false,
                    skew: 0.0.into(),
                },
            },
        ),
        (
            "ProFontWindows Nerd Font (Faux) Bold (Buggy)",
            vec![
                StyleProperty::FontStack(FontStack::Source("ProFontWindows Nerd Font")),
                StyleProperty::FontWeight(FontWeight::BOLD),
            ],
            Expected {
                fullname: "ProFontWindows Nerd Font".into(),
                attributes: Attributes::new(Stretch::NORMAL, Weight::NORMAL, Style::Normal),
                synthesis: Synthesis {
                    vars: vec![],
                    // FIXME: embolden should be set to true, but it is only set for BLACK and EXTRA_BLACK
                    // See: https://github.com/linebender/parley/issues/93
                    embolden: false,
                    skew: 0.0.into(),
                },
            },
        ),
        (
            "CaskaydiaCove Nerd Font Italic (Buggy)",
            vec![
                StyleProperty::FontStack(FontStack::Source("CaskaydiaCove Nerd Font")),
                StyleProperty::FontStyle(FontStyle::Italic),
            ],
            Expected {
                // FIXME: This should load the regular italic style
                // See: https://github.com/linebender/parley/issues/95
                fullname: "CaskaydiaCove NF SemiLight Italic".into(),
                attributes: Attributes::new(Stretch::NORMAL, Weight(350), Style::Italic),
                synthesis: Synthesis {
                    vars: vec![],
                    // FIXME: Embolden should be false
                    embolden: true,
                    skew: 0.0.into(),
                },
            },
        ),
        (
            "Monaspace Xenon Var",
            vec![StyleProperty::FontStack(FontStack::Source(
                "Monaspace Xenon Var",
            ))],
            Expected {
                fullname: "Monaspace Xenon Var Regular".into(),
                attributes: Attributes::new(Stretch::NORMAL, Weight::NORMAL, Style::Normal),
                synthesis: Synthesis {
                    vars: vec![crate::Setting {
                        tag: WGHT,
                        value: swash::Weight::NORMAL.0.into(),
                    }],
                    embolden: false,
                    skew: 0.0.into(),
                },
            },
        ),
        (
            "Monaspace Xenon Var (Variadic) Bold",
            vec![
                StyleProperty::FontStack(FontStack::Source("Monaspace Xenon Var")),
                StyleProperty::FontWeight(FontWeight::BOLD),
            ],
            Expected {
                fullname: "Monaspace Xenon Var Regular".into(),
                attributes: Attributes::new(Stretch::NORMAL, Weight::NORMAL, Style::Normal),
                synthesis: Synthesis {
                    vars: vec![crate::Setting {
                        tag: WGHT,
                        value: swash::Weight::BOLD.0.into(),
                    }],
                    embolden: false,
                    skew: 0.0.into(),
                },
            },
        ),
        (
            "Monaspace Xenon Var (Variadic) Italic (Buggy)",
            vec![
                StyleProperty::FontStack(FontStack::Source("Monaspace Xenon Var")),
                StyleProperty::FontStyle(FontStyle::Italic),
            ],
            Expected {
                fullname: "Monaspace Xenon Var Regular".into(),
                attributes: Attributes::new(Stretch::NORMAL, Weight::NORMAL, Style::Normal),
                synthesis: Synthesis {
                    vars: vec![
                        crate::Setting {
                            tag: WGHT,
                            value: swash::Weight::NORMAL.0.into(),
                        },
                        crate::Setting {
                            tag: SLNT,
                            // FIXME: This should be -11
                            // See: https://github.com/linebender/parley/issues/94
                            value: (14.0).into(),
                        },
                    ],
                    embolden: false,
                    skew: 0.0.into(),
                },
            },
        ),
        (
            "Monaspace Xenon Var Oblique -10 degrees",
            vec![
                StyleProperty::FontStack(FontStack::Source("Monaspace Xenon Var")),
                StyleProperty::FontStyle(FontStyle::Oblique(Some(-10.0))),
            ],
            Expected {
                fullname: "Monaspace Xenon Var Regular".into(),
                attributes: Attributes::new(Stretch::NORMAL, Weight::NORMAL, Style::Normal),
                synthesis: Synthesis {
                    vars: vec![
                        crate::Setting {
                            tag: WGHT,
                            value: swash::Weight::NORMAL.0.into(),
                        },
                        crate::Setting {
                            tag: SLNT,
                            value: (-10.0).into(),
                        },
                    ],
                    embolden: false,
                    skew: 0.0.into(),
                },
            },
        ),
        (
            "Monaspace Xenon Var Oblique 5 degreees = no italic",
            vec![
                StyleProperty::FontStack(FontStack::Source("Monaspace Xenon Var")),
                StyleProperty::FontStyle(FontStyle::Oblique(Some(-5.0))),
            ],
            Expected {
                fullname: "Monaspace Xenon Var Regular".into(),
                attributes: Attributes::new(Stretch::NORMAL, Weight::NORMAL, Style::Normal),
                synthesis: Synthesis {
                    vars: vec![
                        crate::Setting {
                            tag: WGHT,
                            value: swash::Weight::NORMAL.0.into(),
                        },
                        crate::Setting {
                            tag: SLNT,
                            value: (-5.0).into(),
                        },
                    ],
                    embolden: false,
                    skew: 0.0.into(),
                },
            },
        ),
        (
            "Monaspace Xenon Var Stretch=113 Weight = 637, Oblique -8 degrees",
            vec![
                StyleProperty::FontStack(FontStack::Source("Monaspace Xenon Var")),
                StyleProperty::FontWeight(FontWeight::new(637.0)),
                StyleProperty::FontStyle(FontStyle::Oblique(Some(-8.0))),
                StyleProperty::FontStretch(FontStretch::from_percentage(113.0)),
            ],
            Expected {
                fullname: "Monaspace Xenon Var Regular".into(),
                attributes: Attributes::new(Stretch::NORMAL, Weight::NORMAL, Style::Normal),
                synthesis: Synthesis {
                    vars: vec![
                        crate::Setting {
                            tag: WDTH,
                            value: 113.0.into(),
                        },
                        crate::Setting {
                            tag: WGHT,
                            value: 637.0.into(),
                        },
                        crate::Setting {
                            tag: SLNT,
                            value: (-8.0).into(),
                        },
                    ],
                    embolden: false,
                    skew: 0.0.into(),
                },
            },
        ),
        (
            "Noto Serif Normal (Buggy)",
            vec![StyleProperty::FontStack(FontStack::Source(
                "NotoSerif Nerd Font",
            ))],
            Expected {
                // FIXME: There's a bug in fontique, the medium style is loaded instead of the regular one
                // See: https://github.com/linebender/parley/issues/92
                fullname: "NotoSerif NF Med".into(),
                attributes: Attributes::new(Stretch::NORMAL, Weight::MEDIUM, Style::Normal),
                synthesis: Synthesis {
                    vars: vec![],
                    embolden: false,
                    skew: 0.0.into(),
                },
            },
        ),
        (
            "Noto Serif Bold",
            vec![
                StyleProperty::FontStack(FontStack::Source("NotoSerif Nerd Font")),
                StyleProperty::FontWeight(FontWeight::BOLD),
            ],
            Expected {
                fullname: "NotoSerif NF Bold".into(),
                attributes: Attributes::new(Stretch::NORMAL, Weight::BOLD, Style::Normal),
                synthesis: Synthesis {
                    vars: vec![],
                    embolden: false,
                    skew: 0.0.into(),
                },
            },
        ),
        (
            "Noto Serif Condensed (Buggy)",
            vec![
                StyleProperty::FontStack(FontStack::Source("NotoSerif Nerd Font")),
                StyleProperty::FontStretch(FontStretch::CONDENSED),
            ],
            Expected {
                // FIXME: There's a bug in fontique, the medium style is loaded instead of the regular one
                // See: https://github.com/linebender/parley/issues/92
                fullname: "NotoSerif NF Cond Med".into(),
                attributes: Attributes::new(Stretch::CONDENSED, Weight::MEDIUM, Style::Normal),
                synthesis: Synthesis {
                    vars: vec![],
                    embolden: false,
                    skew: 0.0.into(),
                },
            },
        ),
        (
            "Noto Serif Bold Condensed",
            vec![
                StyleProperty::FontStack(FontStack::Source("NotoSerif Nerd Font")),
                StyleProperty::FontWeight(FontWeight::BOLD),
                StyleProperty::FontStretch(FontStretch::CONDENSED),
            ],
            Expected {
                fullname: "NotoSerif NF Cond Bold".into(),
                attributes: Attributes::new(Stretch::CONDENSED, Weight::BOLD, Style::Normal),
                synthesis: Synthesis {
                    vars: vec![],
                    embolden: false,
                    skew: 0.0.into(),
                },
            },
        ),
        (
            "Noto Serif Italic (Buggy)",
            vec![
                StyleProperty::FontStack(FontStack::Source("NotoSerif Nerd Font")),
                StyleProperty::FontStyle(FontStyle::Italic),
            ],
            Expected {
                // FIXME: There's a bug in fontique, the medium style is loaded instead of the regular one
                // See: https://github.com/linebender/parley/issues/92
                fullname: "NotoSerif NF Med Italic".into(),
                attributes: Attributes::new(Stretch::NORMAL, Weight::MEDIUM, Style::Italic),
                synthesis: Synthesis {
                    vars: vec![],
                    embolden: false,
                    skew: 0.0.into(),
                },
            },
        ),
        (
            "Noto Serif Bold Condensed Italic",
            vec![
                StyleProperty::FontStack(FontStack::Source("NotoSerif Nerd Font")),
                StyleProperty::FontWeight(FontWeight::BOLD),
                StyleProperty::FontStretch(FontStretch::CONDENSED),
                StyleProperty::FontStyle(FontStyle::Italic),
            ],
            Expected {
                fullname: "NotoSerif NF Cond Bold Italic".into(),
                attributes: Attributes::new(Stretch::CONDENSED, Weight::BOLD, Style::Italic),
                synthesis: Synthesis {
                    vars: vec![],
                    embolden: false,
                    skew: 0.0.into(),
                },
            },
        ),
    ];
    let layout = shaper.layout_within_with(
        &lines
            .iter()
            .map(|line| line.0)
            .collect::<Vec<_>>()
            .join("\n"),
        800.,
        |builder| {
            builder.push_default(&StyleProperty::Brush(Srgba::new(0., 0., 0., 1.)));
            builder.push_default(&StyleProperty::FontSize(16.));
            let mut start = 0;
            for line in &lines {
                let line_len = line.0.len();
                let range = start..start + line_len;
                for prop in &line.1 {
                    builder.push(prop, range.clone());
                }
                start += line_len + 1;
            }
        },
    );

    let layout_width = layout.width();
    let layout_height = layout.height();
    scene.add_text_layout(layout, point2!(padding, padding));

    let current_layer = scene.layer();
    assert_eq!(current_layer.contents.glyph_runs.len(), lines.len());
    for (index, line) in lines.iter().enumerate() {
        let glyph_run = &current_layer.contents.glyph_runs[index];
        let font = scene.resources.fonts.get(&glyph_run.font_id).unwrap();
        let font_ref = font.as_swash_font_ref(glyph_run.font_index).unwrap();
        let fullname = font_ref
            .localized_strings()
            .find_by_id(swash::StringId::Full, None)
            .map_or("".into(), |str| str.chars().collect::<String>());
        let attributes = font_ref.attributes();
        let synthesis = glyph_run.synthesis.clone();
        let actual = Expected {
            fullname,
            attributes,
            synthesis,
        };
        assert_eq!(line.2, actual, "line number {index}");
    }

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
