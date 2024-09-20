use glamour::point2;
use palette::Srgba;
use parley::{
    style::{FontStack, FontStretch, FontStyle, FontWeight, StyleProperty},
    swash,
    swash::{tag_from_bytes, Attributes, Setting, Stretch, Style, Tag, Weight},
};

use crate::{scene::Scene, scene::Synthesis, Shaper};

const WGHT: Tag = tag_from_bytes(b"wght");
const WDTH: Tag = tag_from_bytes(b"wdth");
const SLNT: Tag = tag_from_bytes(b"slnt");

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
            "FiraCode Normal",
            vec![StyleProperty::FontStack(FontStack::Source(
                "FiraCode Nerd Font",
            ))],
            Expected {
                fullname: "FiraCode Nerd Font Reg".into(),
                attributes: Attributes::new(Stretch::NORMAL, Weight::NORMAL, Style::Normal),
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
            "FiraCode (Faux) Italic",
            vec![
                StyleProperty::FontStack(FontStack::Source("FiraCode Nerd Font")),
                StyleProperty::FontStyle(FontStyle::Italic),
            ],
            Expected {
                fullname: "FiraCode Nerd Font Reg".into(),
                attributes: Attributes::new(Stretch::NORMAL, Weight::NORMAL, Style::Normal),
                synthesis: Synthesis {
                    vars: vec![],
                    embolden: false,
                    skew: 14.0.into(),
                },
            },
        ),
        (
            "FiraCode Oblique 5 degrees",
            vec![
                StyleProperty::FontStack(FontStack::Source("FiraCode Nerd Font")),
                StyleProperty::FontStyle(FontStyle::Oblique(Some(5.0))),
            ],
            Expected {
                fullname: "FiraCode Nerd Font Reg".into(),
                attributes: Attributes::new(Stretch::NORMAL, Weight::NORMAL, Style::Normal),
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
                fullname: "FiraCode Nerd Font Reg".into(),
                attributes: Attributes::new(Stretch::NORMAL, Weight::NORMAL, Style::Normal),
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
            "ProFontWindows Nerd Font (Faux) Bold",
            vec![
                StyleProperty::FontStack(FontStack::Source("ProFontWindows Nerd Font")),
                StyleProperty::FontWeight(FontWeight::BOLD),
            ],
            Expected {
                fullname: "ProFontWindows Nerd Font".into(),
                attributes: Attributes::new(Stretch::NORMAL, Weight::NORMAL, Style::Normal),
                synthesis: Synthesis {
                    vars: vec![],
                    embolden: true,
                    skew: 0.0.into(),
                },
            },
        ),
        (
            "CaskaydiaCove Nerd Font Italic",
            vec![
                StyleProperty::FontStack(FontStack::Source("CaskaydiaCove Nerd Font")),
                StyleProperty::FontStyle(FontStyle::Italic),
            ],
            Expected {
                fullname: "CaskaydiaCove NF Italic".into(),
                attributes: Attributes::new(Stretch::NORMAL, Weight::NORMAL, Style::Italic),
                synthesis: Synthesis {
                    vars: vec![],
                    embolden: false,
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
                    vars: vec![],
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
                    vars: vec![crate::Setting {
                        tag: SLNT,
                        // FIXME: This should be -11
                        // See: https://github.com/linebender/parley/issues/94
                        value: (14.0).into(),
                    }],
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
                    vars: vec![crate::Setting {
                        tag: SLNT,
                        value: (-10.0).into(),
                    }],
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
                    vars: vec![crate::Setting {
                        tag: SLNT,
                        value: (-5.0).into(),
                    }],
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
            "Noto Serif Normal",
            vec![StyleProperty::FontStack(FontStack::Source(
                "NotoSerif Nerd Font",
            ))],
            Expected {
                fullname: "NotoSerif NF Reg".into(),
                attributes: Attributes::new(Stretch::NORMAL, Weight::NORMAL, Style::Normal),
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
            "Noto Serif Condensed",
            vec![
                StyleProperty::FontStack(FontStack::Source("NotoSerif Nerd Font")),
                StyleProperty::FontStretch(FontStretch::CONDENSED),
            ],
            Expected {
                fullname: "NotoSerif NF Cond Reg".into(),
                attributes: Attributes::new(Stretch::CONDENSED, Weight::NORMAL, Style::Normal),
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
            "Noto Serif Italic",
            vec![
                StyleProperty::FontStack(FontStack::Source("NotoSerif Nerd Font")),
                StyleProperty::FontStyle(FontStyle::Italic),
            ],
            Expected {
                fullname: "NotoSerif NF Italic".into(),
                attributes: Attributes::new(Stretch::NORMAL, Weight::NORMAL, Style::Italic),
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
    scene.add_text_layout(&layout, point2!(padding, padding));

    let current_layer = scene.layer();
    assert_eq!(
        current_layer
            .contents
            .primitives
            .last()
            .unwrap()
            .as_glyph_run_vec()
            .unwrap()
            .len(),
        lines.len()
    );
    for (index, line) in lines.iter().enumerate() {
        let glyph_runs = current_layer
            .contents
            .primitives
            .last()
            .unwrap()
            .as_glyph_run_vec()
            .unwrap();
        let glyph_run = &glyph_runs[index];
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
}
