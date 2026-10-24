//! File/start presentation; callers retain commands, caches and selection ownership.
//!
//! Adapted from SolidCraft (`crates/ui-egui/src/ui_kit/file_view.rs`), MIT OR Apache-2.0. The
//! recovery banner was removed. See ATTRIBUTION.md.

use super::{Container, Surface, metrics::dimension, text::paint_text};
use crate::theme::Tokens;
use egui::{Align2, FontId, InnerResponse, Rect, Response, Sense, Stroke, TextureId, Ui, Vec2, pos2, vec2};

pub struct RecentRow<'a> {
    pub title: &'a str,
    pub path: &'a str,
    pub when: &'a str,
    pub workspace: &'a str,
    pub thumbnail: Option<(TextureId, Vec2)>,
}

#[derive(Clone, Copy)]
pub struct SampleCard<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub icon: &'a str,
    pub thumbnail: Option<(TextureId, Vec2)>,
}

/// Shared start-page surfaces. Every allocation is bounded by the caller's width.
pub struct FileView;

impl FileView {
    pub fn page<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let container = Container::new(ui.ctx(), Surface::Viewport);
        container.central().frame(container.edge_frame(Tokens::get().file_background)).show(ui, add)
    }

    pub fn content<R>(ui: &mut Ui, narrow: bool, add: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let t = Tokens::get();
        let inset = if narrow { t.space_16 } else { t.file_content_inset_x };
        egui::Frame::NONE.inner_margin(egui::Margin::symmetric(inset as i8, t.file_content_inset_y as i8)).show(ui, add)
    }

    pub fn subtitle(ui: &mut Ui, text: &str) {
        let t = Tokens::get();
        let galley = ui.painter().layout(text.to_owned(), FontId::proportional(t.font_heading), t.file_dim, dimension(ui.available_width()));
        let (rect, _) = ui.allocate_exact_size(galley.size(), Sense::hover());
        ui.painter().galley(rect.min, galley, t.file_dim);
    }

    pub fn detail(ui: &mut Ui, text: &str) {
        let t = Tokens::get();
        let galley = ui.painter().layout(text.to_owned(), FontId::proportional(t.font_label), t.file_dim, dimension(ui.available_width()));
        let (rect, _) = ui.allocate_exact_size(galley.size(), Sense::hover());
        ui.painter().galley(rect.min, galley, t.file_dim);
    }

    pub fn section<R>(ui: &mut Ui, title: &str, gap: f32, add: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            Self::section_title(ui, title);
            ui.add_space(dimension(gap));
            add(ui)
        })
    }

    pub fn grid(ui: &mut Ui, id: &str, min_width: f32, count: usize, mut cell: impl FnMut(&mut Ui, usize, f32)) {
        let t = Tokens::get();
        let gap = t.file_grid_gap;
        let (columns, width) = grid_geometry(ui.available_width(), min_width, count);
        ui.push_id(id, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            for row in (0..count).step_by(columns) {
                if row > 0 {
                    ui.add_space(gap);
                }
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = gap;
                    for index in row..(row + columns).min(count) {
                        ui.scope_builder(
                            egui::UiBuilder::new()
                                .max_rect(Rect::from_min_size(ui.next_widget_position(), vec2(width, 0.0)))
                                .layout(egui::Layout::top_down(egui::Align::Min)),
                            |ui| {
                                ui.set_width(width);
                                cell(ui, index, width);
                            },
                        );
                    }
                });
            }
        });
    }

    pub fn recent_list<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let t = Tokens::get();
        egui::Frame::NONE.fill(t.file_surface).stroke(Stroke::new(1.0, t.file_border)).corner_radius(t.radius_xl).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            let key = ui.id().with("file_recent_index");
            ui.ctx().data_mut(|data| data.insert_temp(key, 0_usize));
            let result = add(ui);
            ui.ctx().data_mut(|data| data.remove::<usize>(key));
            result
        })
    }

    pub fn title(ui: &mut Ui, title: &str) {
        let t = Tokens::get();
        let galley = ui.painter().layout_no_wrap(title.to_owned(), crate::fonts::semibold(t.font_start_title), t.form_ink);
        let (rect, _) = ui.allocate_exact_size(vec2(dimension(ui.available_width()), galley.size().y), Sense::hover());
        ui.painter().with_clip_rect(ui.clip_rect().intersect(rect)).galley(rect.min, galley, t.form_ink);
    }

    pub fn section_title(ui: &mut Ui, title: &str) {
        let t = Tokens::get();
        let galley = ui.painter().layout_no_wrap(title.to_owned(), crate::fonts::semibold(t.font_intro), t.form_ink);
        let (rect, _) = ui.allocate_exact_size(vec2(dimension(ui.available_width()), galley.size().y), Sense::hover());
        ui.painter().with_clip_rect(ui.clip_rect().intersect(rect)).galley(rect.min, galley, t.form_ink);
    }

    pub fn tile_min_width() -> f32 {
        Tokens::get().form_card_width - Tokens::get().space_24
    }

    pub fn sample_min_width() -> f32 {
        Tokens::get().file_nav_width - Tokens::get().file_tile_height
    }

    pub fn rail<R>(ui: &mut Ui, rect: Rect, add: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        ui.scope_builder(egui::UiBuilder::new().max_rect(rect).layout(egui::Layout::top_down(egui::Align::Min)), |ui| {
            ui.set_clip_rect(ui.clip_rect().intersect(rect));
            ui.painter().rect_filled(rect, 0, Tokens::get().file_rail);
            Self::navigation(ui, add).inner
        })
    }

    pub fn navigation<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let t = Tokens::get();
        egui::Frame::NONE.fill(t.file_rail).inner_margin(egui::Margin::symmetric(t.space_10 as i8, t.space_14 as i8)).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = t.space_2;
            add(ui)
        })
    }

    pub fn nav_row(ui: &mut Ui, id: &str, label: &str, shortcut: &str, selected: bool, enabled: bool) -> Response {
        let t = Tokens::get();
        ui.add_enabled_ui(enabled, |ui| {
            let (rect, response) = allocate(ui, id, ui.available_width(), t.file_nav_height, label);
            if selected || response.hovered() || response.has_focus() {
                ui.painter().rect_filled(rect, t.radius_lg, t.file_rail_hover);
            }
            let inner = rect.shrink2(vec2(t.space_12, 0.0));
            let key_width =
                ui.painter().layout_no_wrap(shortcut.to_owned(), FontId::monospace(t.font_small), t.file_rail_ink).size().x.min(inner.width());
            let label_rect = Rect::from_min_max(inner.min, pos2((inner.right() - key_width - t.space_8).max(inner.left()), inner.bottom()));
            let ink = if ui.is_enabled() { t.file_rail_ink } else { t.file_rail_ink.gamma_multiply(0.5) };
            paint_text(
                ui,
                label_rect,
                label,
                if selected { crate::fonts::semibold(t.font_heading) } else { FontId::proportional(t.font_heading) },
                ink,
                false,
            );
            paint_text(
                ui,
                Rect::from_min_max(pos2(inner.right() - key_width, inner.top()), inner.max),
                shortcut,
                FontId::monospace(t.font_small),
                ink,
                false,
            );
            response
        })
        .inner
    }

    pub fn back_to(ui: &mut Ui, id: &str, template: &str, document: &str, shortcut: &str) -> Response {
        let t = Tokens::get();
        let (rect, response) = allocate(ui, id, ui.available_width(), t.file_back_height, &template.replace("{name}", document));
        if response.hovered() || response.has_focus() {
            ui.painter().rect_filled(rect, t.radius_lg, t.file_rail_hover);
        }
        let inner = rect.shrink2(vec2(t.space_12, 0.0));
        let icon = Rect::from_center_size(pos2(inner.left() + t.form_icon_size / 2.0, inner.center().y), Vec2::splat(t.form_icon_size));
        crate::icons::paint_ink_mirrored(ui.painter(), icon, "chevron_right", t.file_rail_ink);
        let key = ui.painter().layout_no_wrap(shortcut.to_owned(), FontId::monospace(t.font_small), t.file_rail_ink);
        let key_rect = Rect::from_center_size(
            pos2(inner.right() - (key.size().x + t.space_10) / 2.0, inner.center().y),
            key.size() + vec2(t.space_10, t.space_4),
        );
        ui.painter().rect_stroke(key_rect, t.radius_sm, Stroke::new(1.0, t.file_rail_ink.gamma_multiply(0.5)), egui::StrokeKind::Inside);
        paint_text(ui, key_rect.shrink2(vec2(t.space_4, 0.0)), shortcut, FontId::monospace(t.font_small), t.file_rail_ink, false);
        let (before, after) = template.split_once("{name}").unwrap_or((template, ""));
        let mut job = egui::text::LayoutJob::default();
        for (text, font) in [
            (before, FontId::proportional(t.font_button)),
            (document, crate::fonts::semibold(t.font_button)),
            (after, FontId::proportional(t.font_button)),
        ] {
            job.append(text, 0.0, egui::TextFormat::simple(font, t.file_rail_ink));
        }
        let text_rect = Rect::from_min_max(
            pos2(icon.right() + t.space_10, inner.top()),
            pos2((key_rect.left() - t.space_8).max(icon.right() + t.space_10), inner.bottom()),
        );
        job.wrap.max_width = text_rect.width();
        job.wrap.max_rows = 1;
        let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
        ui.painter().with_clip_rect(ui.clip_rect().intersect(text_rect)).galley(
            pos2(text_rect.left(), text_rect.center().y - galley.size().y / 2.0),
            galley,
            t.file_rail_ink,
        );
        ui.add_space(t.space_10);
        response
    }

    pub fn separator(ui: &mut Ui) {
        let t = Tokens::get();
        ui.add_space(t.space_10);
        let (rect, _) = ui.allocate_exact_size(vec2(dimension(ui.available_width()), 1.0), Sense::hover());
        ui.painter().hline(
            (rect.left() + t.space_12)..=(rect.right() - t.space_12).max(rect.left() + t.space_12),
            rect.center().y,
            Stroke::new(1.0, t.file_rail_ink.gamma_multiply(0.25)),
        );
        ui.add_space(t.space_10);
    }

    pub fn action_tile(ui: &mut Ui, id: &str, width: f32, icon: &str, label: &str, subtitle: &str, primary: bool) -> Response {
        let t = Tokens::get();
        let (rect, response) = allocate(ui, id, width, t.file_tile_height, label);
        card(ui, rect, &response, primary);
        let inner = rect.shrink(t.space_16);
        let icon_size = t.file_tile_icon.min(inner.height()).min(inner.width());
        let icon_rect = Rect::from_center_size(pos2(inner.left() + icon_size / 2.0, inner.center().y), Vec2::splat(icon_size));
        crate::icons::paint_accented(ui.painter(), icon_rect, icon, None, if ui.is_enabled() { 1.0 } else { 0.45 });
        let text_rect = Rect::from_min_max(pos2((icon_rect.right() + t.space_14).min(inner.right()), inner.top()), inner.max);
        let ink = if primary { t.file_primary_ink } else { t.form_ink };
        let mut title_job = egui::text::LayoutJob::simple(label.to_owned(), crate::fonts::semibold(t.font_heading), ink, text_rect.width());
        title_job.wrap.max_rows = 1;
        let title = ui.fonts_mut(|fonts| fonts.layout_job(title_job));
        let mut caption_job = egui::text::LayoutJob::simple(subtitle.to_owned(), FontId::proportional(t.font_label), t.file_dim, text_rect.width());
        caption_job.wrap.max_rows = 2;
        let caption = ui.fonts_mut(|fonts| fonts.layout_job(caption_job));
        let group_height = title.size().y + t.space_2 + caption.size().y;
        let top = text_rect.center().y - group_height / 2.0;
        let painter = ui.painter().with_clip_rect(ui.clip_rect().intersect(text_rect));
        painter.galley(pos2(text_rect.left(), top), title.clone(), ink);
        painter.galley(pos2(text_rect.left(), top + title.size().y + t.space_2), caption, t.file_dim);
        response.on_hover_text(format!("{label}\n{subtitle}"))
    }

    pub fn recent_row(ui: &mut Ui, id: &str, row: RecentRow<'_>) -> Response {
        let t = Tokens::get();
        let (rect, response) = allocate(ui, id, ui.available_width(), t.file_recent_height, row.title);
        if response.hovered() || response.has_focus() {
            ui.painter().rect_filled(rect, t.radius_lg, t.hover);
        }
        let list = ui.stack().iter().find_map(|node| {
            let key = node.id.with("file_recent_index");
            ui.ctx().data(|data| data.get_temp::<usize>(key)).map(|index| (key, index))
        });
        if let Some((key, index)) = list {
            if index > 0 {
                ui.painter().hline(rect.x_range(), rect.top(), Stroke::new(1.0, t.file_border));
            }
            ui.ctx().data_mut(|data| data.insert_temp(key, index.saturating_add(1)));
        }
        let inner = rect.shrink2(vec2(t.space_14, t.space_8));
        let preview = Rect::from_min_size(inner.min, vec2(t.space_24 * 2.0, inner.height()).min(inner.size()));
        thumbnail(ui, preview, row.thumbnail);
        if row.thumbnail.is_none() {
            crate::icons::paint_accented(
                ui.painter(),
                Rect::from_center_size(preview.center(), Vec2::splat(t.space_24.min(preview.height()))),
                "box",
                None,
                1.0,
            );
        }
        let left = (preview.right() + t.space_14).min(inner.right());
        let content = Rect::from_min_max(pos2(left, inner.top()), inner.max);
        // At narrow widths the same row keeps title and path, leaving optional metadata out.
        if content.width() < t.form_card_width * 2.0 {
            let half = content.center().y;
            paint_text(
                ui,
                Rect::from_min_max(content.min, pos2(content.right(), half)),
                row.title,
                crate::fonts::medium(t.font_button),
                t.form_ink,
                false,
            );
            paint_text(ui, Rect::from_min_max(pos2(left, half), content.max), row.path, FontId::monospace(t.font_body), t.file_dim, false);
        } else {
            let when_width = t.space_24 * 5.0;
            let workspace_width = t.form_label_width * 2.0;
            let remaining = (content.width() - when_width - workspace_width - t.space_14 * 3.0).max(0.0);
            let title_width = remaining * 0.4;
            let path_left = left + title_width + t.space_14;
            let when_left = content.right() - workspace_width - t.space_14 - when_width;
            let workspace_left = content.right() - workspace_width;
            paint_text(
                ui,
                Rect::from_min_max(content.min, pos2(left + title_width, content.bottom())),
                row.title,
                crate::fonts::medium(t.font_button),
                t.form_ink,
                false,
            );
            paint_text(
                ui,
                Rect::from_min_max(pos2(path_left, content.top()), pos2((when_left - t.space_14).max(path_left), content.bottom())),
                row.path,
                FontId::monospace(t.font_body),
                t.file_dim,
                false,
            );
            paint_text(
                ui,
                Rect::from_min_max(pos2(when_left, content.top()), pos2(when_left + when_width, content.bottom())),
                row.when,
                FontId::proportional(t.font_label),
                t.file_dim,
                false,
            );
            ui.painter().circle_filled(pos2(workspace_left + t.space_4, content.center().y), t.space_4, t.file_rail);
            paint_text(
                ui,
                Rect::from_min_max(pos2(workspace_left + t.space_8 + t.space_6, content.top()), content.max),
                row.workspace,
                FontId::proportional(t.font_label),
                t.file_dim,
                false,
            );
        }
        response.on_hover_text(row.path)
    }

    pub fn sample_grid(ui: &mut Ui, id: &str, samples: &[SampleCard<'_>]) -> Vec<Response> {
        let (columns, width) = grid_geometry(ui.available_width(), Self::sample_min_width(), samples.len());
        let heights: Vec<f32> =
            samples.chunks(columns).map(|row| row.iter().map(|sample| sample_height(ui, width, sample)).fold(0.0, f32::max)).collect();
        let mut responses = Vec::with_capacity(samples.len());
        Self::grid(ui, id, Self::sample_min_width(), samples.len(), |ui, index, width| {
            if let Some(sample) = samples.get(index) {
                let height = heights.get(index / columns).copied().unwrap_or(0.0);
                responses.push(sample_card_with_height(ui, &format!("{id}:{index}"), width, sample, height));
            }
        });
        responses
    }
}

fn allocate(ui: &mut Ui, id: &str, width: f32, height: f32, label: &str) -> (Rect, Response) {
    let (rect, _) = ui.allocate_exact_size(vec2(dimension(width).min(dimension(ui.available_width())), height), Sense::hover());
    let response = ui.interact(rect, ui.make_persistent_id(id), Sense::click());
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), label));
    if response.clicked() {
        response.request_focus();
    }
    if response.enabled() && response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    (rect, response)
}

fn card(ui: &Ui, rect: Rect, response: &Response, primary: bool) {
    let t = Tokens::get();
    let fill = if primary {
        t.file_primary
    } else if response.hovered() || response.has_focus() {
        t.hover
    } else {
        t.file_surface
    };
    let border = if primary || response.has_focus() { t.file_primary_border } else { t.file_border };
    ui.painter().rect(rect, t.radius_xl, fill, Stroke::new(if primary { 1.5 } else { 1.0 }, border), egui::StrokeKind::Inside);
}

fn thumbnail(ui: &Ui, rect: Rect, texture: Option<(TextureId, Vec2)>) {
    let t = Tokens::get();
    ui.painter().rect(rect, t.radius_md, t.file_surface, Stroke::new(1.0, t.file_border), egui::StrokeKind::Inside);
    if let Some((texture, size)) = texture.filter(|(_, size)| size.is_finite() && size.x > 0.0 && size.y > 0.0) {
        let scale = (rect.width() / size.x).min(rect.height() / size.y);
        let target = Align2::CENTER_CENTER.align_size_within_rect(size * scale, rect);
        ui.painter().image(texture, target, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), t.file_image_tint);
    }
}

fn grid_geometry(available: f32, min_width: f32, count: usize) -> (usize, f32) {
    let gap = Tokens::get().file_grid_gap;
    let available = dimension(available);
    let columns = (((available + gap) / (dimension(min_width).max(1.0) + gap)).floor() as usize).max(1).min(count.max(1));
    let width = ((available - gap * columns.saturating_sub(1) as f32) / columns as f32).max(0.0);
    (columns, width)
}

fn sample_height(ui: &Ui, width: f32, sample: &SampleCard<'_>) -> f32 {
    let t = Tokens::get();
    let inner_width = (dimension(width) - t.space_10 * 2.0).max(0.0);
    let title = ui.painter().layout(sample.title.to_owned(), crate::fonts::semibold(t.font_button), t.form_ink, inner_width);
    let description = ui.painter().layout(sample.description.to_owned(), FontId::proportional(t.font_body), t.file_dim, inner_width);
    t.file_sample_image_height + t.space_10 * 2.0 + t.space_8 * 2.0 + title.size().y + description.size().y
}

fn sample_card_with_height(ui: &mut Ui, id: &str, width: f32, sample: &SampleCard<'_>, min_height: f32) -> Response {
    let t = Tokens::get();
    let inner_width = (dimension(width).min(dimension(ui.available_width())) - t.space_10 * 2.0).max(0.0);
    let title = ui.painter().layout(sample.title.to_owned(), crate::fonts::semibold(t.font_button), t.form_ink, inner_width);
    let description = ui.painter().layout(sample.description.to_owned(), FontId::proportional(t.font_body), t.file_dim, inner_width);
    let height = sample_height(ui, width, sample).max(min_height);
    let (rect, response) = allocate(ui, id, width, height, sample.title);
    card(ui, rect, &response, false);
    let inner = rect.shrink(t.space_10);
    let image = Rect::from_min_size(inner.min, vec2(inner.width(), t.file_sample_image_height));
    thumbnail(ui, image, sample.thumbnail);
    if sample.thumbnail.is_none() {
        crate::icons::paint_accented(
            ui.painter(),
            Rect::from_center_size(image.center(), Vec2::splat(t.file_tile_icon.min(image.width()))),
            sample.icon,
            None,
            1.0,
        );
    }
    let painter = ui.painter().with_clip_rect(ui.clip_rect().intersect(inner));
    let title_top = image.bottom() + t.space_8;
    let description_top = title_top + title.size().y + t.space_8;
    painter.galley(pos2(inner.left(), title_top), title, t.form_ink);
    painter.galley(pos2(inner.left(), description_top), description, t.file_dim);
    response.on_hover_text(sample.description)
}
