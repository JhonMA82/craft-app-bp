#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! The kit's own invariants: outer-host policy, the published accent, semantic text colours and
//! control metrics. The app-level interactions live in `craft-demo`'s headless tests.

use craft_theme::Accent;
use egui::Color32;
use egui::Id;

use crate::frame::Side;
use crate::layout::{LayoutMode, SHELL_GAP};
use crate::theme::{AccentColors, Tokens};
use crate::ui_kit::{self, Container, Surface, Text};

fn run<R>(mut add: impl FnMut(&mut egui::Ui) -> R) -> (egui::FullOutput, R) {
    let ctx = egui::Context::default();
    let out = std::cell::RefCell::new(None);
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| *out.borrow_mut() = Some(add(ui)));
    output.textures_delta.clear();
    (output, out.into_inner().expect("the frame ran"))
}

#[test]
fn docked_cards_collapse_their_borders_and_floating_cards_keep_the_shell_gap() {
    let ctx = egui::Context::default();
    let docked = Container::for_layout(LayoutMode::Docked, Surface::Panel);
    assert_eq!(docked.gap(), 0.0);
    assert!(!docked.floating());
    let frame = docked.frame(&ctx, Color32::BLACK, Side::Right);
    assert_eq!(frame.outer_margin.left, -1, "the shared border sits outside the allocation");
    assert_eq!(frame.outer_margin.top, -1);
    assert_eq!(frame.shadow.blur, 0, "a docked card casts no shadow");

    let floating = Container::for_layout(LayoutMode::Floating, Surface::Panel);
    assert!(floating.floating());
    assert_eq!(floating.gap(), SHELL_GAP);
    let frame = floating.frame(&ctx, Color32::BLACK, Side::Right);
    assert_eq!(frame.outer_margin, egui::Margin::ZERO);
    assert!(frame.shadow.blur > 0, "a floating card is elevated");
}

#[test]
fn the_active_accent_is_published_for_later_widgets() {
    let ctx = egui::Context::default();
    let t = Tokens::get();
    for accent in Accent::ALL {
        AccentColors::publish(&ctx, accent);
        assert_eq!(AccentColors::current(&ctx), accent.colors(&t), "{} round trips", accent.id());
    }
    // The helper picks readable ink for a bright fill.
    let bright = AccentColors { color: Color32::from_rgb(250, 240, 120), ink: t.text, soft: t.accent_soft };
    assert_eq!(bright.on(&t), t.tab_on_fill_dark);
    let dark = AccentColors { color: Color32::from_rgb(20, 30, 40), ink: t.text, soft: t.accent_soft };
    assert_eq!(dark.on(&t), t.on_accent);
}

#[test]
fn error_text_paints_the_semantic_error_fill() {
    let t = Tokens::get();
    let (output, ()) = run(|ui| {
        ui_kit::text(ui, "that failed", Text::Error("craft_test_error"));
    });
    let filled = output.shapes.iter().any(|clipped| match &clipped.shape {
        egui::Shape::Rect(rect) => rect.fill == t.form_error_fill && rect.rect.width() > 0.0,
        _ => false,
    });
    assert!(filled, "the error text sits on form_error_fill");
}

#[test]
fn fields_keep_the_control_height() {
    let t = Tokens::get();
    let (_, rect) = run(|ui| {
        let mut value = String::new();
        ui_kit::cell_editor(ui, Id::new("craft_test_field"), &mut value, "", false).rect
    });
    assert_eq!(rect.height(), t.control_height);
}

#[test]
fn tmp_label_wrap_measure() {
    let ctx = egui::Context::default();
    crate::fonts::setup_bundled_fonts(&ctx);
    let mut measured = Vec::new();
    let mut out = ctx.run_ui(
        egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 600.0))), ..Default::default() },
        |ui| {
            let t = Tokens::get();
            for width in [54.0_f32, 64.0, 70.0, 76.0] {
                let mut job =
                    egui::text::LayoutJob::simple("Añadir nota".to_string(), egui::FontId::proportional(t.font_small), t.text, width - t.space_6);
                job.wrap.max_rows = 2;
                job.halign = egui::Align::Center;
                let g = ui.painter().layout_job(job);
                measured.push((width, g.size(), g.rows.len(), g.rows.iter().map(|r| r.rect().width()).collect::<Vec<_>>()));
            }
            let mut job = egui::text::LayoutJob::simple("Añadir nota".to_string(), egui::FontId::proportional(t.font_small), t.text, 0.0);
            job.wrap.max_rows = 2;
            job.halign = egui::Align::Center;
            let g = ui.painter().layout_job(job);
            measured.push((0.0, g.size(), g.rows.len(), g.rows.iter().map(|r| r.rect().width()).collect::<Vec<_>>()));
        },
    );
    out.textures_delta.clear();
    eprintln!("MEASURED: {measured:?}");
}
