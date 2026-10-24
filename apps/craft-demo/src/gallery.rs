//! The kitchen-sink page: every retained kit element on one scrollable page, with the accent and
//! theme switches, plus the three overlay kinds (window, confirm, toast). It is both the visual
//! reference and the headless test's evidence that the kit draws in both themes.

use craft_theme::{Accent, Tokens};
use craft_ui::i18n::tr;
use craft_ui::ui_kit::{self, Button, Column, Container, Surface, Text};
use egui::{Id, Sense, Ui, vec2};

use crate::app::DemoApp;

/// The generic specimen rows: no application domain, just stable labels and icons.
struct Sample {
    id: &'static str,
    label: &'static str,
    icon: &'static str,
}

const SAMPLES: [Sample; 3] = [
    Sample { id: "alpha", label: "ui.info", icon: "info" },
    Sample { id: "beta", label: "ui.warning", icon: "warning" },
    Sample { id: "gamma", label: "ui.enabled", icon: "check" },
];

/// Publish a handle for one gallery specimen.
fn handle(ui: &Ui, id: &str) {
    // The last control just drawn: the bottom strip of the content so far.
    let t = Tokens::get();
    let r = ui.min_rect();
    let rect = egui::Rect::from_min_size(egui::pos2(r.left(), r.bottom() - t.control_height), egui::vec2(r.width(), t.control_height));
    craft_ui::scenario::publish_target(&format!("gallery:{id}"), rect);
}

pub fn show(app: &mut DemoApp, ui: &mut Ui) {
    let t = Tokens::get();
    ui_kit::settings_scroll(ui, "gallery_scroll", |ui| {
        ui_kit::settings_heading(ui, tr("menu.kitchen_sink"), tr("gallery.intro"));
        ui.horizontal(|ui| {
            if ui_kit::button(ui, tr("cmd.app.start"), Button::Normal).clicked() {
                app.pending = Some("app.start");
            }
            let mut theme = usize::from(!craft_ui::theme::is_dark());
            if ui_kit::segmented(ui, &mut theme, &[tr("prefs.theme_dark"), tr("prefs.theme_light")], ui_kit::Segments::default()) {
                app.pending_theme = Some(theme == 0);
            }
            handle(ui, "theme");
        });

        ui_kit::settings_section(ui, "controls", tr("prefs.interface"), |ui| {
            let mut checked = true;
            if ui_kit::checkbox(ui, tr("ui.enabled"), &mut checked).changed() {
                app.notice = Some(tr("ui.enabled").to_string());
            }
            handle(ui, "checkbox");
            let mut choice = 0;
            if ui_kit::segmented(ui, &mut choice, &[tr("prefs.theme_dark"), tr("prefs.theme_light"), tr("prefs.layout")], ui_kit::Segments::default())
            {
                app.notice = Some(tr("prefs.interface").to_string());
            }
            handle(ui, "segmented");
            let mut index = 0;
            ui_kit::dropdown(ui, "gallery_dropdown", &mut index, &[tr("ui.info"), tr("ui.warning"), tr("ui.error")]);
            handle(ui, "dropdown");
            let mut text = String::from("Craft");
            ui_kit::text_field(ui, &mut text, tr("ui.name"));
            handle(ui, "field");
            let mut value = 1.5_f32;
            ui_kit::slider(ui, &mut value, 0.5..=4.0, ui_kit::Slider::default());
            handle(ui, "slider");
            let mut note = String::new();
            ui_kit::text_area(ui, tr("ui.notes"), Id::new("gallery_note"), &mut note, 3, 500);
            handle(ui, "textarea");
        });

        ui_kit::settings_section(ui, "colors", tr("ui.color"), |ui| {
            for accent in Accent::ALL {
                let color = accent.colors(&t);
                let response = ui_kit::color_swatch(ui, Id::new(("gallery_swatch", accent.id())), color.color, false, accent.id(), accent);
                craft_ui::scenario::publish_handle(&format!("gallery:swatch:{}", accent.id()), response.rect.center());
            }
        });

        ui_kit::settings_section(ui, "table", tr("gallery.samples"), |ui| {
            let columns = [Column::new("ui.name", 3.0).flex(), Column::new("ui.info", 2.0)];
            let empty = tr("ui.no_results");
            let mut clicked = None;
            ui_kit::table(ui, "gallery", &columns, empty, |ui, table| {
                for sample in &SAMPLES {
                    let response = table.row(ui, sample.id, false, |row| {
                        row.cell(|ui| {
                            ui_kit::text(ui, tr(sample.label), Text::Cell);
                        });
                        row.cell(|ui| {
                            ui_kit::text(ui, sample.icon, Text::MonoCell);
                        });
                    });
                    if response.clicked() {
                        clicked = Some(sample.label);
                    }
                }
            });
            handle(ui, "table");
            if let Some(label) = clicked {
                app.notice = Some(tr(label).to_string());
            }
        });

        ui_kit::settings_section(ui, "menu", tr("ui.more"), |ui| {
            ui_kit::Menu::button(ui, tr("ui.more"), |ui| {
                for sample in &SAMPLES {
                    let mut item = ui_kit::MenuItem::new(sample.id, sample.label, sample.icon);
                    item.shortcut = "";
                    if item.show(ui).clicked() {
                        app.notice = Some(tr(sample.label).to_string());
                    }
                }
                ui_kit::Menu::separator(ui);
                ui_kit::Menu::heading(ui, "gallery.samples");
            });
            handle(ui, "menu");
        });

        ui_kit::settings_section(ui, "text", tr("ui.info"), |ui| {
            ui_kit::text(ui, tr("ui.loading"), Text::Hint("gallery_hint"));
            ui_kit::text(ui, tr("ui.no_results"), Text::Note("gallery_note"));
            ui_kit::text(ui, tr("app.about_text"), Text::Inline);
            ui_kit::text(ui, tr("error.command_failed"), Text::Error("gallery_error"));
            handle(ui, "text");
        });

        ui_kit::settings_section(ui, "status", tr("status.messages"), |ui| {
            ui.label(egui::RichText::new(tr("status.ready")).size(t.font_small).color(t.text_dim));
            let (track, _) = ui.allocate_exact_size(vec2(ui.available_width(), t.space_6), Sense::hover());
            ui.painter().rect_filled(track, t.space_3, t.dialog_rule);
            let half = egui::Rect::from_min_size(track.min, vec2(track.width() * 0.5, track.height()));
            ui.painter().rect_filled(half, t.space_3, t.accent);
            handle(ui, "progress");
            ui_kit::settings_badge(ui, tr("ui.enabled"), false);
            handle(ui, "badge");
        });

        ui_kit::settings_section(ui, "overlays", tr("menu.help"), |ui| {
            ui.horizontal_wrapped(|ui| {
                if ui_kit::button(ui, tr("ui.add"), Button::Normal).clicked() {
                    app.gallery_open = true;
                }
                if ui_kit::button(ui, tr("gallery.confirm"), Button::Normal).clicked() {
                    app.pending_confirm = Some(tr("gallery.confirm_message").to_string());
                }
                if ui_kit::button(ui, tr("gallery.toast"), Button::Normal).clicked() {
                    app.pending_toast = Some(tr("status.ready").to_string());
                }
            });
            handle(ui, "overlays");
        });
    });

    if app.gallery_open {
        let mut open = app.gallery_open;
        let window = Container::new(ui.ctx(), Surface::Window).window(tr("menu.kitchen_sink"));
        window.open(&mut open).show(ui.ctx(), |ui| {
            ui.set_width(Tokens::get().form_card_width * 2.0);
            ui_kit::text(ui, tr("app.about_text"), Text::Note("gallery_window_note"));
            ui_kit::settings_badge(ui, tr("ui.info"), false);
        });
        app.gallery_open = open;
    }
}
