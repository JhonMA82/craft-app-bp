#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use egui::Color32;
use serde_json::json;

use super::files::{self, Base};
use super::{Accent, ColorVision, Tokens, contrast_ratio};

/// The inks the UI sets text in, against the surfaces they are drawn on, in both palettes and
/// every colour vision mode. WCAG AA for normal text is 4.5:1.
#[test]
fn text_keeps_wcag_contrast_in_every_vision_mode() {
    for dark in [true, false] {
        for mode in ColorVision::ALL {
            let t = Tokens::themed(dark, mode);
            let some = |what: &str, ink: Color32, surface: Color32| {
                let ratio = contrast_ratio(ink, surface);
                assert!(ratio >= 4.5, "{what} on {:?} in {} {:?}: {ratio:.2}", surface, if dark { "dark" } else { "light" }, mode);
            };
            for surface in [t.panel, t.toolbar, t.panel_header, t.dialog_body, t.dialog_bg, t.dialog_footer, t.field, t.control_fill] {
                some("text", t.text, surface);
            }
            for surface in [t.panel, t.dialog_bg] {
                some("text_dim", t.text_dim, surface);
            }
            for surface in [t.dialog_body, t.dialog_bg, t.dialog_footer, t.field, t.control_fill, t.panel] {
                some("form_ink", t.form_ink, surface);
            }
            for surface in [t.dialog_body, t.dialog_footer, t.field, t.control_fill] {
                some("form_dim", t.form_dim, surface);
            }
            some("selected_ink", t.selected_ink, t.selected_bg);
            some("app_bar_text", t.app_bar_text, t.app_bar);
            some("app_bar_text", t.app_bar_text, t.app_bar_tab_active);
        }
    }
}

#[test]
fn every_accent_has_a_distinct_tint_and_round_trips_its_id() {
    for dark in [true, false] {
        let t = Tokens::themed(dark, ColorVision::Off);
        for accent in Accent::ALL {
            let colors = accent.colors(&t);
            assert_ne!(colors.color, colors.soft, "{} soft tint collapses into its colour", accent.id());
            assert_eq!(Accent::from_id(accent.id()), Some(accent));
        }
    }
}

#[test]
fn theme_file_round_trips_every_colour() {
    for (base, t) in [(Base::Dark, Tokens::dark()), (Base::Light, Tokens::light())] {
        let exported = files::export("Round Trip", base, &t);
        let parsed = files::parse(&exported.to_string()).expect("an export parses");
        assert_eq!(parsed.id, "round-trip");
        assert_eq!(parsed.colors.len(), files::keys().len(), "an export writes every key");
        assert_eq!(files::resolve(&parsed), t, "export -> parse -> resolve keeps the palette");
    }
}

#[test]
fn theme_file_rejects_unknown_keys_and_bad_colours_by_name() {
    let unknown = json!({"format": files::FORMAT, "name": "Bad", "base": "dark", "colors": {"bogus": "#000000"}});
    let err = files::from_value(&unknown).unwrap_err();
    assert!(err.contains("bogus"), "{err}");

    let invalid = json!({"format": files::FORMAT, "name": "Bad", "base": "dark", "colors": {"panel": "not-a-colour"}});
    let err = files::from_value(&invalid).unwrap_err();
    assert!(err.contains("panel"), "{err}");

    let wrong_format = json!({"format": "solidcraft-theme/1", "name": "Old", "base": "dark"});
    assert!(files::from_value(&wrong_format).unwrap_err().contains("craft-theme/1"));
}

#[test]
fn a_theme_file_only_overrides_what_it_names() {
    let file = json!({"format": files::FORMAT, "name": "Mine", "base": "light", "colors": {"panel": "#101010"}});
    let parsed = files::from_value(&file).expect("valid");
    let resolved = files::resolve(&parsed);
    assert_eq!(resolved.panel, Color32::from_rgb(16, 16, 16));
    assert_eq!(resolved.toolbar, Tokens::light().toolbar);
}
