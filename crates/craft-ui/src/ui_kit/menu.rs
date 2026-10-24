//! Menus share one popup surface and one row painter. Command dispatch stays with callers.
use crate::ui_kit::{Container, Surface};
use crate::{icons, theme::Tokens};
use egui::{Color32, FontId, Rect, Response, Sense, Stroke, Ui, pos2, vec2};

/// The shared menu popup surface and row states. Command dispatch stays with callers.
pub struct Menu;

/// Width of a submenu sheet.
const LIST_W: f32 = 250.0;

impl Menu {
    pub fn context<R>(response: &Response, add: impl FnOnce(&mut Ui) -> R) -> Option<egui::InnerResponse<R>> {
        Container::new(&response.ctx, Surface::Popup).context_menu(response).frame(Self::frame()).show(add)
    }

    pub fn button<R>(ui: &mut Ui, label: impl Into<egui::WidgetText>, add: impl FnOnce(&mut Ui) -> R) -> egui::InnerResponse<Option<R>> {
        let previous = ui.style().clone();
        let frame = Self::frame();
        let style = ui.style_mut();
        style.spacing.menu_margin = frame.inner_margin;
        style.visuals.menu_corner_radius = frame.corner_radius;
        style.visuals.popup_shadow = frame.shadow;
        style.visuals.window_fill = frame.fill;
        style.visuals.window_stroke = frame.stroke;
        let nested = egui::menu::is_in_menu(ui);
        let button = egui::Button::new(label.into());
        let response = ui.add(if nested { button.right_text(egui::menu::SubMenuButton::RIGHT_ARROW) } else { button });
        let popup_id = if nested { egui::menu::SubMenu::id_from_widget_id(response.id) } else { egui::Popup::default_response_id(&response) };
        let was_open = ui.ctx().read_response(popup_id).is_some();
        let gate_id = popup_id.with("opening_click");
        let first_click = !was_open || ui.ctx().data(|d| d.get_temp::<bool>(gate_id)).unwrap_or(true);
        let (opening_press, released) = ui.input(|i| {
            (
                first_click
                    && (i.pointer.any_down() || i.pointer.any_released())
                    && i.pointer.press_origin().or_else(|| i.pointer.interact_pos()).is_some_and(|p| response.rect.contains(p)),
                i.pointer.any_released(),
            )
        });
        let content = |ui: &mut Ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            // The opening gesture belongs to the parent even when a narrow popup overlaps it.
            ui.ctx().data_mut(|d| d.insert_temp(gate_id, first_click && !released));
            ui.add_enabled_ui(!opening_press, add).inner
        };
        let popup = if nested {
            let submenu = Container::new(ui.ctx(), Surface::Popup).submenu();
            let submenu = if opening_press {
                submenu.config(egui::menu::MenuConfig::new().close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside))
            } else {
                submenu
            };
            submenu.show(ui, &response, content)
        } else {
            Container::new(ui.ctx(), Surface::Popup).menu(&response).frame(frame).show(content)
        };
        let response = egui::InnerResponse { response, inner: popup.map(|r| r.inner) };
        ui.set_style(previous);
        response
    }

    pub fn frame() -> egui::Frame {
        super::Container::menu_frame()
    }

    pub fn show<R>(ui: &mut Ui, width: f32, add: impl FnOnce(&mut Ui) -> R) -> R {
        Self::frame()
            .show(ui, |ui| {
                ui.set_width((width - Tokens::get().space_6 * 2.0 - 2.0).max(0.0));
                ui.spacing_mut().item_spacing.y = 0.0;
                ui.with_layout(egui::Layout::top_down(egui::Align::Min), add).inner
            })
            .inner
    }

    pub fn separator(ui: &mut Ui) {
        let t = Tokens::get();
        let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), t.space_10 + 1.0), Sense::hover());
        ui.painter().hline(r.x_range(), r.center().y, Stroke::new(1.0, t.dialog_rule));
    }

    pub fn heading(ui: &mut Ui, label: &str) {
        let t = Tokens::get();
        let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), t.space_24), Sense::hover());
        let mut job =
            egui::text::LayoutJob::simple_singleline(crate::i18n::tr_label(label).to_uppercase(), crate::fonts::semibold(t.font_small), t.form_dim);
        for section in &mut job.sections {
            section.format.extra_letter_spacing = t.font_small * t.font_section_tracking;
        }
        let galley = ui.painter().layout_job(job);
        ui.painter().galley(pos2(r.left() + t.space_8, r.center().y - galley.size().y / 2.0), galley, t.form_dim);
    }

    /// Bounded keyboard navigation skips disabled entries and structural rows.
    pub fn navigate(ctx: &egui::Context, enabled: &[bool], selected: &mut Option<usize>) -> bool {
        let (up, down, enter) =
            ctx.input(|i| (i.key_pressed(egui::Key::ArrowUp), i.key_pressed(egui::Key::ArrowDown), i.key_pressed(egui::Key::Enter)));
        let usable: Vec<_> = enabled.iter().enumerate().filter_map(|(k, on)| on.then_some(k)).collect();
        if up || down {
            let at = selected.and_then(|r| usable.iter().position(|u| *u == r));
            let len = usable.len().max(1);
            let next = match (at, down) {
                (None, true) => 0,
                (None, false) => len - 1,
                (Some(k), true) => (k + 1) % len,
                (Some(k), false) => (k + len - 1) % len,
            };
            *selected = usable.get(next).copied();
        }
        enter && selected.is_some_and(|k| enabled.get(k).copied().unwrap_or(false))
    }

    /// Place a submenu beside its row, flipping at the screen edge.
    pub fn submenu<R>(ctx: &egui::Context, id: egui::Id, row: Rect, height: f32, add: impl FnOnce(&mut Ui) -> R) -> egui::InnerResponse<R> {
        let screen = ctx.content_rect();
        let width = LIST_W;
        let x = if row.right() + width <= screen.right() { row.right() } else { (row.left() - width).max(screen.left()) };
        let at = pos2(x, row.top().min((screen.bottom() - height).max(screen.top())));
        Container::new(ctx, Surface::Popup).area(id).fixed_pos(at).order(egui::Order::Foreground).show(ctx, |ui| Self::show(ui, width, add))
    }
}

/// Shared row states. Swatches are document colour data; chrome always uses Tokens.
pub struct MenuItem<'a> {
    pub id: &'a str,
    pub label: &'a str,
    pub icon: &'a str,
    pub shortcut: &'a str,
    pub enabled: bool,
    pub highlighted: bool,
    pub checked: Option<bool>,
    pub swatch: Option<Color32>,
    pub danger: bool,
    pub submenu: bool,
}

impl<'a> MenuItem<'a> {
    pub fn new(id: &'a str, label: &'a str, icon: &'a str) -> Self {
        Self { id, label, icon, shortcut: "", enabled: true, highlighted: false, checked: None, swatch: None, danger: false, submenu: false }
    }

    pub fn show(&self, ui: &mut Ui) -> Response {
        let t = Tokens::get();
        // Areas first measure intrinsic content. Filling their provisional available
        // width would make a flipped submenu cover its parent and consume the opening click.
        let width = if ui.is_sizing_pass() {
            let label = ui.painter().layout_no_wrap(crate::i18n::tr_label(self.label).to_string(), FontId::proportional(t.font_label), t.form_ink);
            let shortcut = ui.painter().layout_no_wrap(self.shortcut.to_string(), FontId::monospace(t.font_small), t.form_dim);
            let trailing = if self.submenu { t.space_16 } else { 0.0 };
            (t.control_height + t.space_2 + label.size().x + shortcut.size().x + t.space_24 + trailing).min(ui.available_width())
        } else {
            ui.available_width()
        };
        let (r, response) =
            ui.allocate_exact_size(vec2(width, t.control_height + t.space_2), if self.enabled { Sense::click() } else { Sense::hover() });
        self.paint(ui, r, &response, 0.0);
        response
    }

    /// Ribbon rows retain their own drag/focus IDs and reserve a trailing pin target.
    pub fn paint(&self, ui: &Ui, r: Rect, response: &Response, trailing: f32) {
        let t = Tokens::get();
        let enabled = self.enabled && ui.is_enabled();
        let hot = enabled && (self.highlighted || response.hovered() || response.has_focus());
        let ink = if self.danger {
            t.form_error
        } else if hot {
            t.form_primary
        } else {
            t.form_ink
        };
        let dim = if self.danger {
            t.form_error
        } else if hot {
            t.form_primary
        } else {
            t.form_dim
        };
        let painter = ui.painter().with_clip_rect(ui.clip_rect().intersect(r));
        let mut painter = painter;
        if !enabled {
            painter.multiply_opacity(0.4);
        }
        if hot {
            painter.rect_filled(r, t.control_radius, t.accent_soft);
        }
        let icon = Rect::from_center_size(pos2(r.left() + t.control_height / 2.0, r.center().y), vec2(t.space_16, t.space_16));
        if let Some(on) = self.checked {
            if on {
                super::checks::paint_check_glyph(&painter, icon, false, t.form_primary);
            }
        } else if let Some(color) = self.swatch {
            painter.rect(icon, t.radius_sm, color, Stroke::new(1.0, t.control_border), egui::StrokeKind::Inside);
        } else if !self.icon.is_empty() {
            if self.danger {
                icons::paint_danger(&painter, icon, self.icon, 1.0);
            } else if hot {
                icons::paint_ink(&painter, icon, self.icon, dim);
            } else {
                icons::paint_icon(&painter, icon, self.icon, 1.0);
            }
        }
        let label = crate::i18n::tr_label(self.label);
        let mut right = r.right() - t.space_8 - trailing;
        if self.submenu {
            let c = pos2(right - t.space_4, r.center().y);
            painter.add(egui::Shape::line(
                vec![c + vec2(-t.space_2, -t.space_4), c + vec2(t.space_2, 0.0), c + vec2(-t.space_2, t.space_4)],
                Stroke::new(1.0, dim),
            ));
            right -= t.space_16;
        }
        if !self.shortcut.is_empty() {
            let key = painter.layout_no_wrap(self.shortcut.to_string(), FontId::monospace(t.font_small), dim);
            painter.galley(pos2(right - key.size().x, r.center().y - key.size().y / 2.0), key.clone(), dim);
            right -= key.size().x + t.space_8;
        }
        let left = r.left() + t.control_height + t.space_2;
        let mut job = egui::text::LayoutJob::simple_singleline(label.to_string(), FontId::proportional(t.font_label), ink);
        job.wrap.max_width = (right - left).max(0.0);
        job.wrap.max_rows = 1;
        let galley = painter.layout_job(job);
        painter.galley(pos2(left, r.center().y - galley.size().y / 2.0), galley, ink);
        response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, label));
        crate::scenario::publish_target(&format!("menu:{}", self.id), r);
        if enabled && response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
    }
}
