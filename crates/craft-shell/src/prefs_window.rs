//! Preferences: general UI configuration, interface, shortcuts, and whatever pages the app adds.
//! The shared card, the painted page rail, the OK/Apply/Cancel footer and Restore defaults follow
//! SolidCraft's preferences dialog.
//!
//! Adapted from SolidCraft (`crates/ui-egui/src/prefs.rs`, `prefs/general.rs`,
//! `prefs/interface.rs`, `ui_kit/panel.rs::page_list_sized`), MIT OR Apache-2.0. The CAD pages and
//! controls are not ported. See ATTRIBUTION.md.

use craft_ui::i18n::{self, tr};
use craft_ui::layout::LayoutMode;
use craft_ui::theme::{ColorVision, Tokens};
use craft_ui::ui_kit::{self, Action, Button, LayoutPreview, Segments, Slider, Text, ThemeSwatch, Window};
use egui::{Align2, FontId, Id, Rect, Sense, Stroke, StrokeKind, Ui, UiBuilder, Vec2, pos2, vec2};

use crate::keymap;
use crate::panels::Panels;
use crate::settings::UiSettings;
use crate::{Shell, ShellApp};

/// Built-in page order. The shortcuts command opens the same index.
pub(crate) const GENERAL_PAGE: usize = 0;
pub(crate) const SHORTCUTS_PAGE: usize = 2;

/// The settings as they were when the dialog opened: Cancel and the close button put them back.
const SNAPSHOT: &str = "craft_prefs_snapshot";

type Page = (&'static str, &'static str, &'static str, Option<&'static str>);

pub fn show<A: ShellApp>(ui: &mut Ui, shell: &mut Shell, app: &mut A) {
    if !shell.prefs.open {
        ui.ctx().data_mut(|d| d.remove::<(UiSettings, Panels)>(Id::new(SNAPSHOT)));
        return;
    }
    let ctx = ui.ctx().clone();
    let t = Tokens::get();
    // Copy the page table out, so the app stays free to draw its own page.
    // The reference rail starts with the first two pages ungrouped; INPUT follows them.
    let mut pages: Vec<Page> = vec![
        ("general", "prefs.general", "settings", None),
        ("interface", "prefs.interface", "palette", None),
        ("shortcuts", "prefs.shortcuts", "keyboard", Some("prefs.group_input")),
    ];
    pages.extend(app.preference_pages().iter().map(|page| (page.id, page.label, page.icon, Some("prefs.group_app"))));
    if shell.prefs.page >= pages.len() {
        shell.prefs.page = GENERAL_PAGE;
    }
    if ctx.data(|d| d.get_temp::<(UiSettings, Panels)>(Id::new(SNAPSHOT))).is_none() {
        remember(shell, &ctx);
    }
    let mut defaults = false;
    let (closed, action) = Window::new(Id::new("craft_preferences"), "settings", tr("prefs.title"))
        .cards(10.0 / 3.0)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .fields()
        .actions(tr("ui.ok"), tr("ui.cancel"), Some(tr("ui.apply")))
        .show_with_left(
            &ctx,
            true,
            |ui| body(ui, shell, app, &pages, &t),
            |ui| {
                if ui_kit::button(ui, tr("ui.restore_defaults"), Button::Normal).on_hover_text(tr("ui.reset_the_settings_on_this_page")).clicked() {
                    defaults = true;
                }
            },
        );
    let page = pages.get(shell.prefs.page).map(|page| page.0).unwrap_or("general");
    if defaults {
        page_defaults(shell, page);
    }
    match action {
        Action::Apply => remember(shell, &ctx),
        Action::Ok => close(shell, &ctx),
        Action::Cancel => cancel(shell, &ctx),
        Action::None => {}
    }
    if closed {
        cancel(shell, &ctx);
    }
}

/// The page rail and the selected page, in the dialog's body.
fn body<A: ShellApp>(ui: &mut Ui, shell: &mut Shell, app: &mut A, pages: &[Page], t: &Tokens) {
    // The rail and the page share one bounded height, as in the reference dialog: the card does
    // not jump when a page is shorter.
    let height =
        (ui.ctx().content_rect().height() - t.form_header_height - t.form_footer_height - t.space_24 * 3.0).clamp(0.0, t.control_height * 24.0);
    let (body, _) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
    let width = (t.form_label_width * 2.0 + t.space_24 * 2.0).min(body.width());
    let nav = Rect::from_min_size(body.min, vec2(width, body.height()));
    ui.painter().rect_filled(nav, 0.0, t.dialog_footer);
    ui.painter().vline(nav.right(), nav.y_range(), Stroke::new(1.0, t.dialog_rule));
    craft_ui::scenario::publish_rect("prefs:pages", nav);
    ui.scope_builder(UiBuilder::new().max_rect(nav.shrink2(vec2(t.space_8, t.space_6))), |ui| {
        ui.set_clip_rect(ui.clip_rect().intersect(nav));
        ui.spacing_mut().item_spacing.y = t.space_2;
        egui::ScrollArea::vertical().id_salt("prefs_pages").show(ui, |ui| {
            for (index, (_, label, icon, group)) in pages.iter().enumerate() {
                if let Some(group) = group {
                    ui.add_space(t.space_8);
                    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), t.space_24), Sense::hover());
                    let rect = rect.shrink2(vec2(t.space_10, 0.0));
                    ui_kit::paint_text(ui, rect, &tr(group).to_uppercase(), craft_ui::fonts::semibold(t.font_small), t.form_dim, true);
                }
                let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), t.control_height + t.space_4), Sense::click());
                if response.clicked() {
                    shell.prefs.page = index;
                }
                let on = shell.prefs.page == index;
                if on || response.hovered() || response.has_focus() {
                    ui.painter().rect_filled(rect, t.control_radius, if on { t.selected_bg } else { t.hover });
                }
                let ink = if on { t.selected_ink } else { t.form_ink };
                let icon_rect = Rect::from_center_size(pos2(rect.left() + t.space_10 + t.space_8, rect.center().y), Vec2::splat(t.space_16));
                craft_ui::icons::paint_ink(ui.painter(), icon_rect, icon, if on { ink } else { t.form_dim });
                let text_rect = Rect::from_min_max(pos2(icon_rect.right() + t.space_8, rect.top()), rect.right_bottom());
                ui_kit::paint_text(
                    ui,
                    text_rect,
                    tr(label),
                    if on { craft_ui::fonts::semibold(t.font_label) } else { FontId::proportional(t.font_label) },
                    ink,
                    false,
                );
                response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, ui.is_enabled(), on, tr(label)));
                if response.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                craft_ui::scenario::publish_target(&format!("prefs:page:{label}"), rect);
                response.on_hover_text(tr(label));
            }
        });
    });
    // The page: padded from the rail, clipped to the body, scrolling on its own.
    let padding = vec2(t.space_16 + t.space_4, t.space_16);
    let content = Rect::from_min_max(pos2(nav.right(), body.top()), body.max);
    let content = content.shrink2(vec2(padding.x.min(content.width() / 2.0), padding.y.min(content.height() / 2.0)));
    ui.scope_builder(UiBuilder::new().max_rect(content), |ui| {
        let clip = Rect::from_min_max(content.min, pos2(body.right(), content.bottom()));
        ui.set_clip_rect(ui.clip_rect().intersect(clip));
        ui.spacing_mut().item_spacing.y = t.form_gap_y;
        ui_kit::settings_scroll(ui, ("craft_prefs_page", shell.prefs.page), |ui| match pages.get(shell.prefs.page).map(|page| page.0) {
            Some("general") => general(ui, shell),
            Some("interface") => interface(ui, shell),
            Some("shortcuts") => match keymap::page(ui, &mut shell.keymap, app.commands(), app.reserved_keys()) {
                keymap::KeymapAction::None => {}
                keymap::KeymapAction::Reset(id) => keymap::reset(&mut shell.keymap.custom, Some(&id)),
                keymap::KeymapAction::ResetAll => keymap::reset(&mut shell.keymap.custom, None),
                keymap::KeymapAction::Close => close(shell, ui.ctx()),
            },
            Some(id) => app.draw_preferences_page(ui, id),
            None => {}
        });
    });
}

fn general(ui: &mut Ui, shell: &mut Shell) {
    let t = Tokens::get();
    let labels = t.space_24 * 6.0 + t.space_6;
    ui_kit::settings_heading(ui, tr("prefs.general"), tr("prefs.general_note"));
    ui_kit::settings_section(ui, "language", tr("prefs.language"), |ui| {
        ui_kit::settings_form(ui, labels, |ui| {
            ui_kit::row(ui, tr("prefs.interface_language"), |ui| {
                ui_kit::settings_control_width(ui, t.form_card_width - t.space_4, |ui| {
                    let current =
                        i18n::LOCALES.iter().find(|catalog| catalog.id == shell.settings.language).map_or("English", |catalog| catalog.name);
                    ui_kit::dropdown_menu(ui, "pref_language", current, |ui| {
                        for catalog in i18n::LOCALES {
                            if ui_kit::list_picker(ui, catalog.id, catalog.name, shell.settings.language == catalog.id).clicked() {
                                shell.settings.language = catalog.id.to_string();
                            }
                        }
                    });
                });
            });
        });
    });
    ui_kit::settings_section(ui, "version", tr("prefs.version"), |ui| {
        ui_kit::text(ui, &i18n::format("about.version", &[("version", env!("CARGO_PKG_VERSION"))]), Text::Note("prefs_version"));
        ui_kit::text(ui, tr("about.license"), Text::Note("prefs_license"));
    });
}

fn interface(ui: &mut Ui, shell: &mut Shell) {
    let t = Tokens::get();
    ui_kit::settings_heading(ui, tr("prefs.interface"), tr("prefs.interface_note"));
    ui_kit::settings_section(ui, "theme", tr("prefs.theme"), |ui| {
        ui.horizontal_wrapped(|ui| {
            for (id, label, palette) in [("dark", "prefs.theme_dark", Tokens::dark()), ("light", "prefs.theme_light", Tokens::light())] {
                if ui_kit::theme_tile(ui, label, ThemeSwatch::from(palette), shell.settings.theme == id).clicked() {
                    shell.settings.theme = id.to_string();
                }
            }
        });
    });
    ui_kit::settings_section(ui, "layout", tr("prefs.layout"), |ui| {
        ui_kit::settings_pair(
            ui,
            |ui| {
                ui_kit::settings_form(ui, t.form_label_width + t.space_16 + t.space_6, |ui| {
                    ui_kit::row(ui, tr("prefs.workspace"), |ui| {
                        let mut selected = usize::from(shell.settings.layout == LayoutMode::Docked);
                        if ui_kit::segmented(ui, &mut selected, &[tr("prefs.layout_floating"), tr("prefs.layout_docked")], Segments::default()) {
                            shell.settings.layout = if selected == 1 { LayoutMode::Docked } else { LayoutMode::Floating };
                        }
                    });
                    for (label, open) in [
                        ("prefs.browser", &mut shell.panels.left.open),
                        ("prefs.properties", &mut shell.panels.right.open),
                        ("prefs.history", &mut shell.panels.bottom.open),
                    ] {
                        ui_kit::row(ui, tr(label), |ui| {
                            let mut selected = usize::from(!*open);
                            if ui_kit::segmented(ui, &mut selected, &[tr("prefs.shown"), tr("prefs.hidden")], Segments::default()) {
                                *open = selected == 0;
                            }
                        });
                    }
                    ui_kit::text(ui, tr("prefs.layout_hint"), Text::Hint("prefs_layout_hint"));
                    LayoutPreview { compact: shell.settings.layout == LayoutMode::Docked, labels: true }
                })
            },
            ui_kit::layout_preview,
        );
    });
    ui_kit::settings_section(ui, "scale", tr("prefs.scale_text"), |ui| {
        ui_kit::settings_pair(
            ui,
            |ui| {
                ui_kit::slider(ui, &mut shell.settings.scale, 0.75..=2.0, Slider::default());
                ui.horizontal_wrapped(|ui| {
                    for percent in [75, 100, 110, 125, 150, 200] {
                        let scale = percent as f32 / 100.0;
                        let selected = (shell.settings.scale - scale).abs() < 0.005;
                        let style = if selected { Button::SmallSelected } else { Button::Small };
                        if ui_kit::button(ui, format!("{percent}%"), style).clicked() {
                            shell.settings.scale = scale;
                        }
                    }
                });
                ui_kit::text(ui, tr("prefs.scale_note"), Text::Hint("prefs_scale_note"));
                shell.settings.scale
            },
            scale_specimen,
        );
        ui_kit::divider(ui, "scale_font");
        ui_kit::text(ui, tr("prefs.font_bundled"), Text::Note("prefs_font"));
        ui_kit::text(ui, tr("prefs.font_sample"), Text::Inline);
    });
    ui_kit::settings_section(ui, "colour_vision", tr("ui.colour_vision"), |ui| {
        let modes = ColorVision::ALL;
        let labels: Vec<&str> = modes.iter().map(|mode| tr(vision_label(*mode))).collect();
        let mut selected = modes.iter().position(|mode| *mode == shell.settings.color_vision).unwrap_or(0);
        if ui_kit::segmented(ui, &mut selected, &labels, Segments::default())
            && let Some(mode) = modes.get(selected)
        {
            shell.settings.color_vision = *mode;
        }
        ui_kit::text(ui, tr("ui.colour_vision_note"), Text::Hint("colour_vision"));
    });
}

fn vision_label(mode: ColorVision) -> &'static str {
    match mode {
        ColorVision::Off => "ui.colour_vision_off",
        ColorVision::Protan => "ui.colour_vision_protan",
        ColorVision::Deutan => "ui.colour_vision_deutan",
        ColorVision::Tritan => "ui.colour_vision_tritan",
    }
}

/// A small form specimen drawn at the chosen scale: a label, a field, a primary action and a row.
fn scale_specimen(ui: &mut Ui, scale: f32) {
    let t = Tokens::get();
    let scale = if scale.is_finite() { scale / ui.ctx().zoom_factor().max(0.5) } else { 1.0 };
    let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), t.space_24 * 4.0 + t.space_16), Sense::hover());
    ui.painter().rect(r, t.radius_lg, t.control_fill, Stroke::new(1.0, t.dialog_rule), StrokeKind::Inside);
    let p = ui.painter().with_clip_rect(r.shrink(t.space_8));
    p.text(
        r.right_top() + vec2(-t.space_8, t.space_8),
        Align2::RIGHT_TOP,
        tr("prefs.live_preview").to_uppercase(),
        FontId::proportional(t.font_micro),
        t.form_dim,
    );
    let origin = r.min + vec2(t.space_10, t.space_24);
    let line_h = t.control_height * scale;
    let font = FontId::proportional(t.font_label * scale);
    let label = p.layout_no_wrap(tr("ui.size").to_owned(), font, t.form_dim);
    p.galley(origin + vec2(0.0, (line_h - label.size().y) / 2.0), label.clone(), t.form_dim);
    let field = Rect::from_min_size(origin + vec2(label.size().x + t.space_8 * scale, 0.0), vec2((t.form_label_width + t.space_6) * scale, line_h));
    p.rect(field, t.control_radius, t.control_fill, Stroke::new(1.0, t.control_border), StrokeKind::Inside);
    p.text(field.center(), Align2::CENTER_CENTER, "12.5", FontId::monospace(t.font_body * scale), t.form_ink);
    let button = Rect::from_min_size(field.right_top() + vec2(t.space_8 * scale, 0.0), vec2(t.space_24 * 1.5 * scale, line_h));
    p.rect_filled(button, t.control_radius, t.form_primary);
    p.text(button.center(), Align2::CENTER_CENTER, tr("ui.ok"), craft_ui::fonts::semibold(t.font_label * scale), t.on_accent);
    let row = Rect::from_min_size(origin + vec2(0.0, line_h + t.space_8 * scale), vec2(r.width() - t.space_10 * 2.0, line_h));
    p.rect_filled(row, t.control_radius, t.selected_bg);
    let icon = Rect::from_center_size(pos2(row.left() + t.space_16 * scale, row.center().y), Vec2::splat(t.space_16 * scale));
    craft_ui::icons::paint_icon(&p, icon, "box", 1.0);
    p.text(
        pos2(icon.right() + t.space_6 * scale, row.center().y),
        Align2::LEFT_CENTER,
        tr("prefs.sample"),
        craft_ui::fonts::semibold(t.font_label * scale),
        t.selected_ink,
    );
}

/// Restore Defaults on the open page.
fn page_defaults(shell: &mut Shell, page: &str) {
    match page {
        "general" => shell.settings.language = "en".into(),
        "interface" => {
            let defaults = UiSettings::default();
            shell.settings.theme = defaults.theme;
            shell.settings.layout = defaults.layout;
            shell.settings.scale = defaults.scale;
            shell.settings.color_vision = defaults.color_vision;
            shell.panels = Panels::default();
        }
        "shortcuts" => shell.keymap.custom.clear(),
        _ => {}
    }
    appearance_changed(shell);
}

fn remember(shell: &Shell, ctx: &egui::Context) {
    ctx.data_mut(|d| d.insert_temp(Id::new(SNAPSHOT), (shell.settings.clone(), shell.panels)));
}

fn restore(shell: &mut Shell, ctx: &egui::Context) {
    if let Some((settings, panels)) = ctx.data(|d| d.get_temp::<(UiSettings, Panels)>(Id::new(SNAPSHOT))) {
        shell.settings = settings;
        shell.panels = panels;
        appearance_changed(shell);
    }
}

/// The dialog's close button: the settings on screen are kept.
fn close(shell: &mut Shell, ctx: &egui::Context) {
    shell.prefs.open = false;
    ctx.data_mut(|d| d.remove::<(UiSettings, Panels)>(Id::new(SNAPSHOT)));
}

/// Cancel, Esc and the header's close: put back the settings from when the dialog opened.
pub(crate) fn cancel(shell: &mut Shell, ctx: &egui::Context) {
    if !shell.prefs.open {
        return;
    }
    restore(shell, ctx);
    shell.prefs.open = false;
    ctx.data_mut(|d| d.remove::<(UiSettings, Panels)>(Id::new(SNAPSHOT)));
}

fn appearance_changed(shell: &mut Shell) {
    // The next frame applies the theme, colour vision and scale afresh.
    shell.last_theme = None;
    shell.last_scale = f32::NAN;
}
