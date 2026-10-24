//! The workspace cards: Browser, Properties and History. Floating, they are cards over the
//! canvas; docked, they are flush edges of the window.
//!
//! Adapted from SolidCraft (`crates/ui-egui/src/browser.rs`: `browser`, `browser_header`;
//! `timeline.rs`: `timeline`, `timeline_contents`), MIT OR Apache-2.0. The CAD tree, feature
//! kinds and recovery states are not ported. See ATTRIBUTION.md.

use craft_ui::i18n::tr;
use craft_ui::layout::SHELL_GAP;
use craft_ui::theme::Tokens;
use craft_ui::ui_kit::{Container, DockEdges, Edge, HistoryChip, HistoryTransport, Surface, history_marker, history_transport_width, panel_grip};
use egui::{Id, Rect, Sense, Ui, pos2, vec2};
use serde::{Deserialize, Serialize};

use crate::{Shell, ShellApp};

/// The width of the strip that resizes the browser card.
const HANDLE: f32 = 6.0;
/// The browser card's header band.
const HEADER_H: f32 = 26.0;
/// The browser card's content inset.
const CARD_INSET: f32 = 4.0;
/// The history card's height.
pub(crate) const HISTORY_H: f32 = 40.0;
/// The chip row's height.
const CHIP_H: f32 = 26.0;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PanelState {
    pub open: bool,
    pub size: f32,
    /// The browser's floating card folded to its header row.
    pub collapsed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Panels {
    pub left: PanelState,
    pub right: PanelState,
    pub bottom: PanelState,
}

impl Default for PanelState {
    fn default() -> Self {
        Self { open: true, size: 250.0, collapsed: false }
    }
}

impl Default for Panels {
    fn default() -> Self {
        Self {
            left: PanelState { open: true, size: 250.0, collapsed: false },
            right: PanelState { open: false, size: 264.0, collapsed: false },
            bottom: PanelState { open: true, size: 40.0, collapsed: false },
        }
    }
}

impl Panels {
    /// Clamp every size into what the shell can draw.
    pub fn sanitize(&mut self) {
        self.left.size = clamp_size(self.left.size, 170.0, 620.0, 250.0);
        self.right.size = clamp_size(self.right.size, 220.0, 480.0, 264.0);
        self.bottom.size = clamp_size(self.bottom.size, 40.0, 320.0, 40.0);
    }
}

fn clamp_size(size: f32, min: f32, max: f32, default: f32) -> f32 {
    if size.is_finite() { size.clamp(min, max) } else { default }
}

/// Whether the history card is drawn this frame; both other cards clear it.
fn history_shown<A: ShellApp>(shell: &Shell, app: &A) -> bool {
    shell.panels.bottom.open && app.history().is_some()
}

/// The vertical space the history card takes from the other cards and overlays, gaps included.
pub(crate) fn history_inset<A: ShellApp>(shell: &Shell, app: &A) -> f32 {
    if history_shown(shell, app) { HISTORY_H + SHELL_GAP } else { 0.0 }
}

pub fn show_browser<A: ShellApp>(ui: &mut Ui, shell: &mut Shell, app: &mut A) {
    if !shell.panels.left.open {
        return;
    }
    let floating = shell.settings.layout == craft_ui::layout::LayoutMode::Floating;
    if floating {
        browser_floating(ui, shell, app);
    } else {
        browser_docked(ui, shell, app);
    }
}

fn browser_floating<A: ShellApp>(ui: &mut Ui, shell: &mut Shell, app: &mut A) {
    let t = Tokens::get();
    let content = ui.available_rect_before_wrap();
    let width = shell.panels.left.size;
    let bottom = content.bottom() - history_inset(shell, app) - SHELL_GAP;
    let bounds = Rect::from_min_max(pos2(content.left() + SHELL_GAP, content.top()), pos2(content.left() + SHELL_GAP + width, bottom));
    if !(bounds.is_finite() && bounds.is_positive() && bounds.height() >= HEADER_H) {
        return;
    }
    let container = Container::new(ui.ctx(), Surface::Panel).joined(DockEdges { left: true, top: true, bottom: false, right: false });
    let frame = container.frame(ui.ctx(), t.panel, craft_ui::frame::Side::Left).inner_margin(CARD_INSET as i8);
    let inner_width = (bounds.width() - frame.total_margin().sum().x).max(0.0);
    let inner_height = (bounds.height() - frame.total_margin().sum().y).max(0.0);
    let title = app.left_panel_title().unwrap_or(tr("ui.browser_title")).to_string();
    let subtitle = app.left_panel_subtitle();
    let collapsed = shell.panels.left.collapsed;
    let mut toggle = false;
    let card = Container::new(ui.ctx(), Surface::Panel).area(Id::new("craft_browser_card")).fixed_pos(bounds.min).order(egui::Order::Middle).show(
        ui.ctx(),
        |ui| {
            let response = container.show_frame(ui, frame, bounds, |ui| {
                ui.set_width(inner_width);
                ui.set_max_height(inner_height);
                if let Some(action) = browser_header(ui, &title, subtitle.as_deref(), collapsed) {
                    toggle = action;
                }
                if !collapsed {
                    ui.painter().hline(
                        ui.max_rect().x_range(),
                        ui.cursor().top() + 1.0,
                        egui::Stroke::new(0.5, t.toolbar.lerp_to_gamma(t.border, 0.5)),
                    );
                    ui.add_space(t.space_3);
                    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| app.left_panel(ui));
                }
            });
            if !collapsed {
                resize(ui, response.response.rect, &mut shell.panels.left.size);
            }
        },
    );
    if toggle {
        shell.panels.left.collapsed = !shell.panels.left.collapsed;
    }
    craft_ui::scenario::publish_rect("browser:card", card.response.rect);
}

fn browser_docked<A: ShellApp>(ui: &mut Ui, shell: &mut Shell, app: &mut A) {
    let t = Tokens::get();
    let size = shell.panels.left.size;
    let title = app.left_panel_title().unwrap_or(tr("ui.browser_title")).to_string();
    let subtitle = app.left_panel_subtitle();
    let container = Container::new(ui.ctx(), Surface::Panel);
    let response = container.edge("browser", Edge::Left).exact_size(size).frame(container.edge_frame(t.panel)).show(ui, |ui| {
        ui.add_space(CARD_INSET);
        if let Some(_toggle) = browser_header(ui, &title, subtitle.as_deref(), shell.panels.left.collapsed) {
            shell.panels.left.collapsed = !shell.panels.left.collapsed;
        }
        if !shell.panels.left.collapsed {
            ui.painter().hline(ui.max_rect().x_range(), ui.cursor().top() + 1.0, egui::Stroke::new(0.5, t.toolbar.lerp_to_gamma(t.border, 0.5)));
            ui.add_space(t.space_3);
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                egui::Frame::NONE
                    .inner_margin(egui::Margin { left: CARD_INSET as i8, right: CARD_INSET as i8, top: 0, bottom: CARD_INSET as i8 })
                    .show(ui, |ui| app.left_panel(ui))
            });
        }
    });
    craft_ui::scenario::publish_rect("browser:card", response.response.rect);
}

/// The card's header: title, subtitle and the fold chevron.
fn browser_header(ui: &mut Ui, title: &str, subtitle: Option<&str>, collapsed: bool) -> Option<bool> {
    let t = Tokens::get();
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), HEADER_H), Sense::hover());
    let response = ui.interact(rect, ui.id().with("browser_header"), Sense::click());
    craft_ui::scenario::publish_target("browser:header", rect);
    if response.hovered() || response.has_focus() {
        craft_ui::ui_kit::panel_header_hover(ui.painter(), rect);
    }
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, title));
    let galley = ui.painter().layout_no_wrap(title.to_owned(), craft_ui::fonts::semibold(t.font_heading), t.text);
    ui.painter().galley(pos2(rect.left() + t.space_6, rect.center().y - galley.size().y / 2.0), galley, t.text);
    let mut right = rect.right() - t.space_6;
    disclosure(ui.painter(), pos2(right - 5.5, rect.center().y), !collapsed);
    right -= 18.0;
    if let Some(subtitle) = subtitle {
        let galley = ui.painter().layout_no_wrap(subtitle.to_owned(), egui::FontId::proportional(t.font_body), t.text_dim);
        ui.painter().galley(pos2(right - galley.size().x, rect.center().y - galley.size().y / 2.0), galley, t.text_dim);
    }
    if response.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() && craft_ui::input::once_this_pass(ui.ctx(), "browser_fold") {
        return Some(!collapsed);
    }
    None
}

/// A chevron: down when open, right when folded.
fn disclosure(painter: &egui::Painter, center: egui::Pos2, open: bool) {
    let ink = Tokens::get().text_dim;
    let half = 5.5;
    let points = if open {
        vec![pos2(center.x - half, center.y - 2.5), pos2(center.x, center.y + 3.0), pos2(center.x + half, center.y - 2.5)]
    } else {
        vec![pos2(center.x - 2.5, center.y - half), pos2(center.x + 3.0, center.y), pos2(center.x - 2.5, center.y + half)]
    };
    painter.add(egui::Shape::line(points, egui::Stroke::new(1.4, ink)));
}

/// A hairline strip on the card's right border: drag it to change the browser's width.
fn resize(ui: &mut Ui, card: Rect, size: &mut f32) {
    let rail = Rect::from_x_y_ranges(card.right() - HANDLE..=card.right(), card.y_range());
    let response = ui.interact(rail, ui.id().with("browser_rail"), Sense::drag()).on_hover_cursor(egui::CursorIcon::ResizeHorizontal);
    if response.dragged() && craft_ui::input::once_this_pass(ui.ctx(), "browser_resize") {
        let delta = ui.input(|input| input.pointer.delta().x);
        if delta.is_finite() {
            *size = (*size + delta).clamp(170.0, 620.0);
        }
    }
    craft_ui::scenario::publish_handle("browser:rail", pos2(card.right() - HANDLE / 2.0, card.center().y));
}

pub fn show_properties<A: ShellApp>(ui: &mut Ui, shell: &mut Shell, app: &mut A) {
    if !shell.panels.right.open {
        return;
    }
    let t = Tokens::get();
    let floating = shell.settings.layout == craft_ui::layout::LayoutMode::Floating;
    let title = app.right_panel_title().unwrap_or(tr("panel.properties")).to_string();
    if floating {
        let content = ui.available_rect_before_wrap();
        let width = shell.panels.right.size;
        let top = content.top() + craft_ui::canvas::AXIS_VIEW_CLEARANCE;
        let bottom = content.bottom() - history_inset(shell, app);
        let rect = Rect::from_min_max(pos2(content.right() - SHELL_GAP - width, top), pos2(content.right() - SHELL_GAP, bottom));
        if !(rect.is_finite() && rect.is_positive()) {
            return;
        }
        let container = Container::new(ui.ctx(), Surface::Feature);
        let frame = container.frame(ui.ctx(), t.dialog_body, craft_ui::frame::Side::Right);
        let mut close = false;
        let card = Container::new(ui.ctx(), Surface::Feature)
            .area(Id::new("craft_properties_card"))
            .fixed_pos(rect.min)
            .order(egui::Order::Middle)
            .show(ui.ctx(), |ui| {
                let response = frame.show(ui, |ui| {
                    ui.set_width(rect.width() - frame.total_margin().sum().x);
                    ui.set_max_height(rect.height() - frame.total_margin().sum().y);
                    craft_ui::ui_kit::header(ui, "panel_right", &title, || close = true);
                    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| app.right_panel(ui));
                });
                response.response
            });
        if close {
            shell.panels.right.open = false;
        }
        craft_ui::scenario::publish_rect("properties:card", card.inner.rect);
    } else {
        let size = shell.panels.right.size;
        let container = Container::new(ui.ctx(), Surface::Feature);
        let response = container.edge("properties", Edge::Right).exact_size(size).frame(container.edge_frame(t.dialog_body)).show(ui, |ui| {
            let mut close = false;
            craft_ui::ui_kit::header(ui, "panel_right", &title, || close = true);
            if close {
                shell.panels.right.open = false;
            }
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| app.right_panel(ui));
        });
        craft_ui::scenario::publish_rect("properties:card", response.response.rect);
    }
}

pub fn show_history<A: ShellApp>(ui: &mut Ui, shell: &mut Shell, app: &mut A) {
    if !history_shown(shell, app) {
        return;
    }
    let t = Tokens::get();
    let floating = shell.settings.layout == craft_ui::layout::LayoutMode::Floating;
    let mut picked: Option<String> = None;
    let mut transport: Option<HistoryTransport> = None;
    if floating {
        let content = ui.available_rect_before_wrap();
        let rect = Rect::from_min_size(
            pos2(content.left() + SHELL_GAP, content.bottom() - HISTORY_H - SHELL_GAP),
            vec2((content.width() - 2.0 * SHELL_GAP).max(0.0), HISTORY_H),
        );
        if !(rect.is_finite() && rect.is_positive()) {
            return;
        }
        // The card lives on the middle layer, so the canvas painted below never covers it.
        let frame = Container::new(ui.ctx(), Surface::Panel).frame(ui.ctx(), t.timeline, craft_ui::frame::Side::Left);
        let card = Container::new(ui.ctx(), Surface::Panel).area(Id::new("craft_history_card")).fixed_pos(rect.min).order(egui::Order::Middle).show(
            ui.ctx(),
            |ui| {
                let card = Rect::from_min_size(ui.min_rect().min, rect.size());
                ui.painter().add(frame.paint(card));
                let mut child = ui.new_child(egui::UiBuilder::new().id_salt("craft_history").max_rect(card));
                child.set_clip_rect(card);
                history_contents(&mut child, app, &mut picked, &mut transport);
            },
        );
        let drawn = Rect::from_min_size(card.response.rect.min, rect.size());
        craft_ui::scenario::publish_rect("history:card", drawn);
    } else {
        let container = Container::new(ui.ctx(), Surface::Panel);
        let response = container
            .edge("history", Edge::Bottom)
            .exact_size(HISTORY_H)
            .frame(container.edge_frame(t.timeline))
            .show(ui, |ui| history_contents(ui, app, &mut picked, &mut transport));
        craft_ui::scenario::publish_rect("history:card", response.response.rect);
    }
    if let Some(id) = picked {
        app.history_selected(&id);
    }
    if let Some(action) = transport {
        app.history_transport(action);
    }
}

/// Grip, title, transport and the step chips of the history strip.
fn history_contents<A: ShellApp>(ui: &mut Ui, app: &mut A, picked: &mut Option<String>, transport: &mut Option<HistoryTransport>) {
    let t = Tokens::get();
    let accent = craft_ui::theme::AccentColors::current(ui.ctx());
    let Some(history) = app.history() else { return };
    let r = ui.max_rect();
    let p = ui.painter().clone();
    let mut x = r.left() + t.space_10;
    let grip_rect = Rect::from_center_size(pos2(x + t.space_10 / 2.0, r.center().y), vec2(t.space_10, t.space_16 + t.space_2));
    let mut grip_ui = ui.new_child(egui::UiBuilder::new().max_rect(grip_rect).layout(egui::Layout::left_to_right(egui::Align::Center)));
    panel_grip(&mut grip_ui);
    x += t.space_10 + t.space_8;
    let label = p.layout_no_wrap(tr("ui.history_title").to_owned(), craft_ui::fonts::semibold(t.font_heading), t.text);
    if x + label.size().x + t.space_12 + history_transport_width() <= r.right() {
        p.galley(pos2(x, r.center().y - label.size().y / 2.0), label.clone(), t.text);
        x += label.size().x + t.space_12;
    }
    for control in HistoryTransport::ALL {
        let br = Rect::from_center_size(pos2(x + t.space_24 / 2.0, r.center().y), vec2(t.space_24, t.space_24));
        if control.show(ui, br).clicked() {
            *transport = Some(control);
        }
        x += t.space_24 + t.space_2;
    }
    x += 14.0;
    let strip = Rect::from_min_max(pos2(x, r.top()), r.right_bottom());
    ui.scope_builder(egui::UiBuilder::new().max_rect(strip), |ui| {
        ui.style_mut().always_scroll_the_only_direction = true;
        egui::ScrollArea::horizontal().auto_shrink([false, false]).show(ui, |ui| {
            let p = ui.painter().clone();
            let mut x = ui.max_rect().left();
            for (index, item) in history.items.iter().enumerate() {
                if index == history.marker {
                    let marker = Rect::from_min_size(pos2(x, r.top() + 5.0), vec2(t.space_12, r.height() - 10.0));
                    history_marker(&p, marker, accent);
                    x += 16.0;
                }
                let text = ui.painter().layout_no_wrap(item.label.clone(), craft_ui::fonts::semibold(t.font_body), t.text);
                let width = (text.size().x + t.space_8 * 2.0 + 15.0 + t.space_4).max(CHIP_H);
                let br = Rect::from_min_size(pos2(x, r.top() + 7.0), vec2(width, CHIP_H));
                let response = ui.interact(br, ui.id().with(("history", item.id.as_str())), Sense::click());
                let selected = index == history.marker.min(history.items.len().saturating_sub(1));
                HistoryChip { icon: item.icon, name: &item.label, selected, hovered: response.hovered(), accent }.paint(&p, br);
                response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &item.label));
                craft_ui::scenario::publish_target(&format!("history:{}", item.id), br);
                if response.clicked() {
                    *picked = Some(item.id.clone());
                }
                x += width + t.space_4;
            }
            if history.marker >= history.items.len() {
                let marker = Rect::from_min_size(pos2(x, r.top() + 5.0), vec2(t.space_12, r.height() - 10.0));
                history_marker(&p, marker, accent);
            }
        });
    });
}
