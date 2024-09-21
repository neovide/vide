use glamour::point2;
use palette::Srgba;
use parley::{
    style::{FontStack, FontStretch, FontStyle, FontWeight, StyleProperty},
    swash,
    swash::{tag_from_bytes, Attributes, Stretch, Style, Tag, Weight},
};

use super::assert_no_regressions;
use crate::{scene::Scene, scene::Synthesis, Shaper};

const WGHT: Tag = tag_from_bytes(b"wght");
const WDTH: Tag = tag_from_bytes(b"wdth");
const SLNT: Tag = tag_from_bytes(b"slnt");

fn assert_attributes(
    style_properties: Vec<StyleProperty<'_, Srgba>>,
    text: &str,
    expected_full_name: &'static str,
    expected_attributes: Attributes,
    expected_synthesis: Synthesis,
) {
    let mut scene = Scene::new();
    let mut shaper = Shaper::new();
    let padding = 10.;

    let layout = shaper.layout_within_with(text, 800., |builder| {
        builder.push_default(&StyleProperty::Brush(Srgba::new(0., 0., 0., 1.)));
        builder.push_default(&StyleProperty::FontSize(16.));
        for prop in style_properties {
            builder.push(&prop, 0..text.len());
        }
    });

    let layout_width = layout.width();
    let layout_height = layout.height();
    scene.add_text_layout(&layout, point2!(padding, padding));

    let current_layer = scene.layer();

    let glyph_run = &current_layer
        .contents
        .primitives
        .last()
        .unwrap()
        .as_glyph_run_vec()
        .unwrap()[0];
    let font = scene.resources.fonts.get(&glyph_run.font_id).unwrap();
    let font_ref = font.as_swash_font_ref(glyph_run.font_index).unwrap();
    let fullname = font_ref
        .localized_strings()
        .find_by_id(swash::StringId::Full, None)
        .map_or("".into(), |str| str.chars().collect::<String>());
    let attributes = font_ref.attributes();
    let synthesis = glyph_run.synthesis.clone();

    assert_eq!(fullname, expected_full_name);
    assert_eq!(attributes, expected_attributes);
    assert_eq!(synthesis, expected_synthesis);

    assert_no_regressions(
        (layout_width + padding * 2.) as u32,
        (layout_height + padding * 2.) as u32,
        scene,
    );
}

#[test]
fn firacode_normal() {
    assert_attributes(
        vec![StyleProperty::FontStack(FontStack::Source(
            "FiraCode Nerd Font",
        ))],
        "FiraCode Normal",
        "FiraCode Nerd Font Reg",
        Attributes::new(Stretch::NORMAL, Weight::NORMAL, Style::Normal),
        Synthesis {
            vars: vec![],
            embolden: false,
            skew: 0.0.into(),
        },
    );
}

#[test]
fn firacode_native_bold() {
    assert_attributes(
        vec![
            StyleProperty::FontStack(FontStack::Source("FiraCode Nerd Font")),
            StyleProperty::FontWeight(FontWeight::BOLD),
        ],
        "FiraCode (Native) Bold",
        "FiraCode Nerd Font Bold",
        Attributes::new(Stretch::NORMAL, Weight::BOLD, Style::Normal),
        Synthesis {
            vars: vec![],
            embolden: false,
            skew: 0.0.into(),
        },
    );
}

#[test]
fn firacode_faux_italic() {
    assert_attributes(
        vec![
            StyleProperty::FontStack(FontStack::Source("FiraCode Nerd Font")),
            StyleProperty::FontStyle(FontStyle::Italic),
        ],
        "FiraCode (Faux) Italic",
        "FiraCode Nerd Font Reg",
        Attributes::new(Stretch::NORMAL, Weight::NORMAL, Style::Normal),
        Synthesis {
            vars: vec![],
            embolden: false,
            skew: 14.0.into(),
        },
    );
}

#[test]
fn firacode_oblique_5_degrees() {
    assert_attributes(
        vec![
            StyleProperty::FontStack(FontStack::Source("FiraCode Nerd Font")),
            StyleProperty::FontStyle(FontStyle::Oblique(Some(5.0))),
        ],
        "FiraCode Oblique 5 degrees",
        "FiraCode Nerd Font Reg",
        Attributes::new(Stretch::NORMAL, Weight::NORMAL, Style::Normal),
        Synthesis {
            vars: vec![],
            embolden: false,
            skew: 5.0.into(),
        },
    );
}

#[test]
fn firacode_synthetic_stretch_wide() {
    assert_attributes(
        vec![
            StyleProperty::FontStack(FontStack::Source("FiraCode Nerd Font")),
            StyleProperty::FontStretch(FontStretch::EXPANDED),
        ],
        "FiraCode Synthetic Stretch Wide",
        "FiraCode Nerd Font Reg",
        // FIXME: Fontique does not support synthetic stretch
        Attributes::new(Stretch::NORMAL, Weight::NORMAL, Style::Normal),
        Synthesis {
            vars: vec![],
            embolden: false,
            skew: 0.0.into(),
        },
    );
}

#[test]
fn profontwindows_nerd_font_faux_bold() {
    assert_attributes(
        vec![
            StyleProperty::FontStack(FontStack::Source("ProFontWindows Nerd Font")),
            StyleProperty::FontWeight(FontWeight::BOLD),
        ],
        "ProFontWindows Nerd Font (Faux) Bold",
        "ProFontWindows Nerd Font",
        Attributes::new(Stretch::NORMAL, Weight::NORMAL, Style::Normal),
        Synthesis {
            vars: vec![],
            embolden: true,
            skew: 0.0.into(),
        },
    );
}

#[test]
#[cfg_attr(target_os = "windows", ignore = "Font not found on windows")]
fn caskaydiacove_nerd_font_italic() {
    assert_attributes(
        vec![
            StyleProperty::FontStack(FontStack::Source("CaskaydiaCove Nerd Font")),
            StyleProperty::FontStyle(FontStyle::Italic),
        ],
        "CaskaydiaCove Nerd Font Italic",
        "CaskaydiaCove NF Italic",
        Attributes::new(Stretch::NORMAL, Weight::NORMAL, Style::Italic),
        Synthesis {
            vars: vec![],
            embolden: false,
            skew: 0.0.into(),
        },
    );
}

#[test]
fn monaspace_xenon_var() {
    assert_attributes(
        vec![StyleProperty::FontStack(FontStack::Source(
            "Monaspace Xenon Var",
        ))],
        "Monaspace Xenon Var",
        "Monaspace Xenon Var Regular",
        Attributes::new(Stretch::NORMAL, Weight::NORMAL, Style::Normal),
        Synthesis {
            vars: vec![],
            embolden: false,
            skew: 0.0.into(),
        },
    );
}

#[test]
fn monaspace_xenon_var_variadic_bold() {
    assert_attributes(
        vec![
            StyleProperty::FontStack(FontStack::Source("Monaspace Xenon Var")),
            StyleProperty::FontWeight(FontWeight::BOLD),
        ],
        "Monaspace Xenon Var (Variadic) Bold",
        "Monaspace Xenon Var Regular",
        Attributes::new(Stretch::NORMAL, Weight::NORMAL, Style::Normal),
        Synthesis {
            vars: vec![crate::Setting {
                tag: WGHT,
                value: swash::Weight::BOLD.0.into(),
            }],
            embolden: false,
            skew: 0.0.into(),
        },
    );
}

#[test]
fn monaspace_xenon_var_variadic_italic_buggy() {
    assert_attributes(
        vec![
            StyleProperty::FontStack(FontStack::Source("Monaspace Xenon Var")),
            StyleProperty::FontStyle(FontStyle::Italic),
        ],
        "Monaspace Xenon Var (Variadic) Italic",
        "Monaspace Xenon Var Regular",
        Attributes::new(Stretch::NORMAL, Weight::NORMAL, Style::Normal),
        Synthesis {
            vars: vec![crate::Setting {
                tag: SLNT,
                // FIXME: This should be -11
                // See: https://github.com/linebender/parley/issues/94
                value: (14.0).into(),
            }],
            embolden: false,
            skew: 0.0.into(),
        },
    );
}

#[test]
fn monaspace_xenon_var_oblique_minus_10_degrees() {
    assert_attributes(
        vec![
            StyleProperty::FontStack(FontStack::Source("Monaspace Xenon Var")),
            StyleProperty::FontStyle(FontStyle::Oblique(Some(-10.0))),
        ],
        "Monaspace Xenon Var Oblique -10 degrees",
        "Monaspace Xenon Var Regular",
        Attributes::new(Stretch::NORMAL, Weight::NORMAL, Style::Normal),
        Synthesis {
            vars: vec![crate::Setting {
                tag: SLNT,
                value: (-10.0).into(),
            }],
            embolden: false,
            skew: 0.0.into(),
        },
    );
}

#[test]
fn monaspace_xenon_var_oblique_5_degrees_no_italic() {
    assert_attributes(
        vec![
            StyleProperty::FontStack(FontStack::Source("Monaspace Xenon Var")),
            StyleProperty::FontStyle(FontStyle::Oblique(Some(-5.0))),
        ],
        "Monaspace Xenon Var Oblique 5 degreees = no italic",
        "Monaspace Xenon Var Regular",
        Attributes::new(Stretch::NORMAL, Weight::NORMAL, Style::Normal),
        Synthesis {
            vars: vec![crate::Setting {
                tag: SLNT,
                value: (-5.0).into(),
            }],
            embolden: false,
            skew: 0.0.into(),
        },
    );
}

#[test]
fn monaspace_xenon_var_stretch_113_weight_637_oblique_minus_8_degrees() {
    assert_attributes(
        vec![
            StyleProperty::FontStack(FontStack::Source("Monaspace Xenon Var")),
            StyleProperty::FontWeight(FontWeight::new(637.0)),
            StyleProperty::FontStyle(FontStyle::Oblique(Some(-8.0))),
            StyleProperty::FontStretch(FontStretch::from_percentage(113.0)),
        ],
        "Monaspace Xenon Var Stretch=113 Weight = 637, Oblique -8 degrees",
        "Monaspace Xenon Var Regular",
        Attributes::new(Stretch::NORMAL, Weight::NORMAL, Style::Normal),
        Synthesis {
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
    );
}

#[test]
#[cfg_attr(target_os = "windows", ignore = "Font not found on windows")]
fn notoserif_normal() {
    assert_attributes(
        vec![StyleProperty::FontStack(FontStack::Source(
            "NotoSerif Nerd Font",
        ))],
        "Noto Serif Normal",
        "NotoSerif NF Reg",
        Attributes::new(Stretch::NORMAL, Weight::NORMAL, Style::Normal),
        Synthesis {
            vars: vec![],
            embolden: false,
            skew: 0.0.into(),
        },
    );
}

#[test]
#[cfg_attr(target_os = "windows", ignore = "Font not found on windows")]
fn notoserif_bold() {
    assert_attributes(
        vec![
            StyleProperty::FontStack(FontStack::Source("NotoSerif Nerd Font")),
            StyleProperty::FontWeight(FontWeight::BOLD),
        ],
        "Noto Serif Bold",
        "NotoSerif NF Bold",
        Attributes::new(Stretch::NORMAL, Weight::BOLD, Style::Normal),
        Synthesis {
            vars: vec![],
            embolden: false,
            skew: 0.0.into(),
        },
    );
}

#[test]
#[cfg_attr(target_os = "windows", ignore = "Font not found on windows")]
fn notoserif_condensed() {
    assert_attributes(
        vec![
            StyleProperty::FontStack(FontStack::Source("NotoSerif Nerd Font")),
            StyleProperty::FontStretch(FontStretch::CONDENSED),
        ],
        "Noto Serif Condensed",
        "NotoSerif NF Cond Reg",
        Attributes::new(Stretch::CONDENSED, Weight::NORMAL, Style::Normal),
        Synthesis {
            vars: vec![],
            embolden: false,
            skew: 0.0.into(),
        },
    );
}

#[test]
#[cfg_attr(target_os = "windows", ignore = "Font not found on windows")]
fn notoserif_bold_condensed() {
    assert_attributes(
        vec![
            StyleProperty::FontStack(FontStack::Source("NotoSerif Nerd Font")),
            StyleProperty::FontWeight(FontWeight::BOLD),
            StyleProperty::FontStretch(FontStretch::CONDENSED),
        ],
        "Noto Serif Bold Condensed",
        "NotoSerif NF Cond Bold",
        Attributes::new(Stretch::CONDENSED, Weight::BOLD, Style::Normal),
        Synthesis {
            vars: vec![],
            embolden: false,
            skew: 0.0.into(),
        },
    );
}

#[test]
#[cfg_attr(target_os = "windows", ignore = "Font not found on windows")]
fn notoserif_italic() {
    assert_attributes(
        vec![
            StyleProperty::FontStack(FontStack::Source("NotoSerif Nerd Font")),
            StyleProperty::FontStyle(FontStyle::Italic),
        ],
        "Noto Serif Italic",
        "NotoSerif NF Italic",
        Attributes::new(Stretch::NORMAL, Weight::NORMAL, Style::Italic),
        Synthesis {
            vars: vec![],
            embolden: false,
            skew: 0.0.into(),
        },
    );
}

#[test]
#[cfg_attr(target_os = "windows", ignore = "Font not found on windows")]
fn notoserif_bold_condensed_italic() {
    assert_attributes(
        vec![
            StyleProperty::FontStack(FontStack::Source("NotoSerif Nerd Font")),
            StyleProperty::FontWeight(FontWeight::BOLD),
            StyleProperty::FontStretch(FontStretch::CONDENSED),
            StyleProperty::FontStyle(FontStyle::Italic),
        ],
        "Noto Serif Bold Condensed Italic",
        "NotoSerif NF Cond Bold Italic",
        Attributes::new(Stretch::CONDENSED, Weight::BOLD, Style::Italic),
        Synthesis {
            vars: vec![],
            embolden: false,
            skew: 0.0.into(),
        },
    );
}
