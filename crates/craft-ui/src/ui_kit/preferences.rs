//! Shared preference compositions: page scroller, headings, sections, theme rail and the layout
//! specimen. Every colour and control metric comes from tokens.
//!
//! Adapted from SolidCraft (`crates/ui-egui/src/ui_kit/preferences.rs`), MIT OR Apache-2.0.
//! The analysis, navigation and figure specimens were removed. See ATTRIBUTION.md.

use super::metrics::dimension;
use super::text::paint_text;
use crate::{scenario, theme::Tokens};
use egui::{Color32, FontId, Rect, Response, Sense, Stroke, Ui, pos2, vec2};

pub const PAGE_SCROLL_GUTTER: &str = "craft_page_scroll_gutter";

/// Page scroller keeps content aligned while centring its bar in the window's right inset.
pub fn settings_scroll<R>(ui: &mut Ui, id: impl egui::AsIdSalt, add: impl FnOnce(&mut Ui) -> R) -> egui::scroll_area::ScrollAreaOutput<R> {
    let t = Tokens::get();
    let gutter = ui.stack().iter().find_map(|node| node.tags().get_downcast::<f32>(PAGE_SCROLL_GUTTER)).copied().unwrap_or(0.0);
    let width = dimension(ui.available_width());
    let height = if ui.available_height().is_finite() { dimension(ui.available_height()) } else { t.control_height * 16.0 };
    let bar = ui.spacing().scroll.floating_width;
    let extension = if gutter > 0.0 { gutter / 2.0 + bar / 2.0 } else { 0.0 };
    let host = Rect::from_min_size(ui.next_widget_position(), vec2(width + extension, height));
    ui.scope_builder(egui::UiBuilder::new().max_rect(host).layout(egui::Layout::top_down(egui::Align::Min)), |ui| {
        if gutter > 0.0 {
            let scroll = &mut ui.style_mut().spacing.scroll;
            scroll.floating = true;
            scroll.floating_allocated_width = 0.0;
            scroll.bar_outer_margin = 0.0;
        }
        egui::ScrollArea::vertical().id_salt(id).max_height(height).auto_shrink([false, false]).show(ui, |ui| {
            if gutter > 0.0 {
                ui.set_width(width);
            }
            add(ui)
        })
    })
    .inner
}

pub fn settings_heading(ui: &mut Ui, title: &str, description: &str) {
    let t = Tokens::get();
    ui.scope(|ui| {
        ui.spacing_mut().item_spacing.y = t.space_3;
        let (rect, response) = ui.allocate_exact_size(vec2(dimension(ui.available_width()), t.space_16 + t.space_3), Sense::hover());
        response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, ui.is_enabled(), title));
        paint_text(ui, rect, title, crate::fonts::semibold(t.font_intro), t.form_ink, false);
        super::text(ui, description, super::Text::Hint("settings_heading"));
    });
}

/// Settings reuse the same disclosure, uppercase typography and rule as the design system.
pub fn settings_section<R>(ui: &mut Ui, key: &str, title: &str, add: impl FnOnce(&mut Ui) -> R) -> Option<R> {
    let t = Tokens::get();
    ui.add_space(t.space_8);
    super::section(ui, key, title, true, |ui| {
        if let Some(rect) = scenario::rect_of(&format!("dialog:section:{title}")) {
            scenario::publish_rect(&format!("prefs:section:{key}"), rect);
        }
        ui.spacing_mut().item_spacing.y = t.space_8;
        add(ui)
    })
}

/// Only the four palette roles needed by a swatch, avoiding a copy of the full theme.
#[derive(Clone, Copy)]
pub struct ThemeSwatch {
    pub background: Color32,
    pub panel: Color32,
    pub accent: Color32,
    pub dim: Color32,
}

impl From<Tokens> for ThemeSwatch {
    fn from(t: Tokens) -> Self {
        Self { background: t.toolbar, panel: t.panel, accent: t.accent, dim: t.text_dim }
    }
}

/// Controls and one explanatory specimen side by side, stacked on narrow viewports.
pub fn settings_pair<R>(ui: &mut Ui, controls: impl FnOnce(&mut Ui) -> R, preview: impl FnOnce(&mut Ui, R)) {
    let t = Tokens::get();
    let preview_width = t.form_card_width - t.space_8;
    let gap = t.space_16 + t.space_4;
    let width = dimension(ui.available_width());
    if width < preview_width * 2.0 + gap {
        let state = controls(ui);
        preview(ui, state);
    } else {
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            let state = ui
                .allocate_ui_with_layout(vec2(width - preview_width - gap, 0.0), egui::Layout::top_down(egui::Align::Min), |ui| {
                    ui.spacing_mut().item_spacing.x = t.space_6;
                    controls(ui)
                })
                .inner;
            ui.allocate_ui_with_layout(vec2(preview_width, 0.0), egui::Layout::top_down(egui::Align::Min), |ui| preview(ui, state));
        });
    }
}

/// A horizontally scrollable, draggable tile rail with its trailing action pinned outside it.
pub fn theme_rail(ui: &mut Ui, tiles: impl FnOnce(&mut Ui), action: impl FnOnce(&mut Ui)) {
    let t = Tokens::get();
    let action_width = t.form_label_width + t.space_24 + t.space_6;
    let rail_width = (dimension(ui.available_width()) - action_width - t.space_8).max(0.0);
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = t.space_8;
        ui.allocate_ui_with_layout(vec2(rail_width, 0.0), egui::Layout::top_down(egui::Align::Min), |ui| {
            ui.style_mut().always_scroll_the_only_direction = true;
            egui::ScrollArea::horizontal()
                .id_salt("settings_theme_rail")
                .max_width(rail_width)
                .auto_shrink([false, true])
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .scroll_source(egui::scroll_area::ScrollSource { drag: egui::scroll_area::DragScroll::Always, ..Default::default() })
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = t.space_8;
                        tiles(ui);
                    });
                });
        });
        ui.allocate_ui_with_layout(vec2(action_width, 0.0), egui::Layout::top_down(egui::Align::Min), action);
    });
}

/// One theme tile: a small palette specimen, its name and the selected ring.
pub fn theme_tile(ui: &mut Ui, label: &str, swatch: ThemeSwatch, selected: bool) -> Response {
    let t = Tokens::get();
    let width = t.form_label_width + t.space_24 + t.space_6;
    let height = t.control_height + t.space_16 + t.space_16;
    let (r, response) = ui.allocate_exact_size(vec2(width.min(dimension(ui.available_width())), height), Sense::click());
    if selected {
        ui.painter().rect_stroke(r, t.radius_lg, Stroke::new(2.0, t.form_primary), egui::StrokeKind::Inside);
    } else if response.hovered() {
        ui.painter().rect_filled(r, t.radius_lg, t.hover);
    }
    let preview = Rect::from_min_size(r.min + vec2(t.space_6, t.space_6), vec2((r.width() - t.space_12).max(0.0), t.control_height));
    theme_swatch(ui.painter(), preview, swatch);
    ui.painter().text(
        pos2(r.center().x, preview.bottom() + t.space_8),
        egui::Align2::CENTER_TOP,
        crate::i18n::tr_label(label),
        FontId::proportional(t.font_small),
        t.form_dim,
    );
    response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::RadioButton, ui.is_enabled(), selected, label));
    scenario::publish_target(&format!("prefs:theme:{label}"), r);
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn theme_swatch(p: &egui::Painter, r: Rect, c: ThemeSwatch) {
    let t = Tokens::get();
    let p = p.with_clip_rect(r.intersect(p.clip_rect()));
    p.rect_filled(r, t.radius_md, c.background);
    let side = Rect::from_min_max(r.min, pos2(r.left() + r.width() * 0.34, r.bottom()));
    p.rect_filled(side, egui::CornerRadius { nw: t.radius_md, sw: t.radius_md, ..Default::default() }, c.panel);
    let line = |y: f32, w: f32| {
        p.rect_filled(
            Rect::from_min_size(pos2(r.left() + r.width() * 0.07, r.top() + r.height() * y), vec2(r.width() * w, r.height() * 0.1)),
            t.space_2,
            c.dim,
        );
    };
    line(0.29, 0.2);
    line(0.54, 0.13);
    p.circle_filled(pos2(r.left() + r.width() * 0.66, r.center().y), r.height() * 0.25, c.accent);
    p.rect_stroke(r, t.radius_md, Stroke::new(1.0, c.dim.gamma_multiply(0.35)), egui::StrokeKind::Inside);
}

/// Compact informational state badge for settings; never an interactive control.
pub fn settings_badge(ui: &mut Ui, text: &str, warning: bool) {
    let t = Tokens::get();
    let ink =
        if warning { if crate::theme::contrast_ratio(t.warning, t.dialog_body) >= 4.5 { t.warning } else { t.form_ink } } else { t.form_success };
    let fill = if warning { t.warning.gamma_multiply(0.1) } else { t.form_success_fill };
    let galley = ui.painter().layout_no_wrap(text.to_owned(), crate::fonts::semibold(t.font_small), ink);
    let (r, _) = ui.allocate_exact_size(vec2(galley.size().x + t.space_16, t.space_24), Sense::hover());
    ui.painter().rect_filled(r, t.radius_xl, fill);
    ui.painter().galley(r.center() - galley.size() / 2.0, galley, ink);
}

/// Clockwise rounded boundary shared by dashed outlines and clipped specimens.
pub fn rounded_perimeter(r: Rect, radius: f32) -> Vec<egui::Pos2> {
    let radius = radius.min(r.width() / 2.0).min(r.height() / 2.0).max(0.0);
    let corners = [
        (pos2(r.right() - radius, r.top() + radius), -std::f32::consts::FRAC_PI_2),
        (pos2(r.right() - radius, r.bottom() - radius), 0.0),
        (pos2(r.left() + radius, r.bottom() - radius), std::f32::consts::FRAC_PI_2),
        (pos2(r.left() + radius, r.top() + radius), std::f32::consts::PI),
    ];
    corners
        .into_iter()
        .flat_map(|(centre, start)| {
            (0..=8).map(move |step| {
                let angle = start + std::f32::consts::FRAC_PI_2 * step as f32 / 8.0;
                centre + vec2(angle.cos(), angle.sin()) * radius
            })
        })
        .collect()
}

/// What the layout specimen shows: docked or floating panels, with placeholder text or without.
#[derive(Clone, Copy)]
pub struct LayoutPreview {
    pub compact: bool,
    pub labels: bool,
}

/// One live shell specimen follows both layout choices; it is a schematic, not a second scene.
pub fn layout_preview(ui: &mut Ui, state: LayoutPreview) {
    let t = Tokens::get();
    let width = dimension(ui.available_width());
    let (r, _) = ui.allocate_exact_size(vec2(width, t.control_height * 3.0), Sense::hover());
    ui.painter().rect(r, t.radius_lg, t.field, Stroke::new(1.0, t.border), egui::StrokeKind::Inside);
    let gap = if state.compact { 0.0 } else { t.space_6 };
    let rail = (r.width() * 0.24).max(t.space_24);
    let header = t.control_height * 0.8;
    let left = Rect::from_min_max(r.min, pos2(r.left() + rail, r.bottom()));
    let right = Rect::from_min_max(pos2(r.right() - rail, r.top()), r.max);
    let top = Rect::from_min_max(pos2(left.right() + gap, r.top()), pos2(right.left() - gap, r.top() + header));
    ui.painter().rect_filled(left.shrink(gap), t.control_radius, t.panel);
    ui.painter().rect_filled(right.shrink(gap), t.control_radius, t.panel);
    ui.painter().rect_filled(top.shrink2(vec2(0.0, gap)), t.control_radius, t.panel_header);
    ui.painter().rect_stroke(left.shrink(gap), t.control_radius, Stroke::new(1.0, t.border), egui::StrokeKind::Inside);
    ui.painter().rect_stroke(right.shrink(gap), t.control_radius, Stroke::new(1.0, t.border), egui::StrokeKind::Inside);
    let accent = Rect::from_min_size(top.min + vec2(t.space_8, gap + header / 2.0 - t.space_2), vec2(top.width() * 0.4, t.space_4));
    ui.painter().rect_filled(accent.intersect(top), t.space_2, t.accent_soft);
    if state.labels {
        for (rect, rows) in [(left, 3), (right, 2)] {
            for k in 0..rows {
                let y = rect.top() + t.space_10 + k as f32 * t.space_10;
                let line = Rect::from_min_size(pos2(rect.left() + t.space_8, y), vec2((rect.width() - t.space_16).max(0.0), t.space_3));
                ui.painter().rect_filled(line, t.space_2, t.text_dim.gamma_multiply(0.5));
            }
        }
    }
    scenario::publish_rect("prefs:layout_preview", r);
}
