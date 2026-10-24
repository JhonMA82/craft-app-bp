//! The ribbon: a workspace switcher card, the tab row with the command search, and each tab's
//! command panels fitted to the width. Fitting is a pure function of the available room, so it
//! never flickers.
//!
//! Adapted from SolidCraft (`crates/ui-egui/src/toolbar.rs`: `toolbar`, `ribbon_tabs`,
//! `ribbon_label_lines`, `fit`), MIT OR Apache-2.0. The CAD command families, pinning and
//! drag-and-drop customisation are not ported. See ATTRIBUTION.md.

use craft_ui::i18n::tr;
use craft_ui::icons;
use craft_ui::layout::LayoutMode;
use craft_ui::theme::{AccentColors, Tokens};
use craft_ui::ui_kit::{CommandSearch, Container, Edge, Menu, MenuItem, Surface};
use egui::{Color32, FontId, Id, Rect, Sense, Stroke, Ui, pos2, vec2};

use crate::commands::{CommandSpec, TabSpec};
use crate::geometry::Cards;
use crate::{Shell, ShellApp};

pub const TAB_ROW_HEIGHT: f32 = 30.0;
pub const GROUP_HEIGHT: f32 = 86.0;
pub const RIBBON_HEIGHT: f32 = 117.0;
/// The workspace switcher card's width.
pub const WORKSPACE_WIDTH: f32 = 88.0;
pub const BUTTON_HEIGHT: f32 = 60.0;
pub const BUTTON_GAP: f32 = 3.0;
pub const BUTTON_MIN_WIDTH: f32 = 54.0;
pub const BUTTON_MAX_WIDTH: f32 = 76.0;
/// A panel collapsed to one button that opens its drop-down.
pub const COLLAPSED_WIDTH: f32 = 72.0;
/// The » button's room at the end of the panels, kept only while some panel is in it.
pub const MORE_ROOM: f32 = 40.0;
const PANEL_GAP: f32 = 1.0;
const SECTION_HEIGHT: f32 = 20.0;
/// Inset of the first tab and of every section label from the card's edges.
const RIBBON_TAB_INSET: f32 = 4.0;
const BUTTON_LABEL_PADDING: f32 = 10.0;
/// The search editor's rect, published for the palette to draw in.
pub const SEARCH_RECT: &str = "craft_ribbon_search";

/// How a panel is drawn on a ribbon that may be too narrow for every button.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fit {
    /// Its first buttons (all of them, or the ones that fit); the panel's drop-down lists every
    /// command.
    Buttons,
    /// One button that opens the panel's drop-down.
    Collapsed,
    /// In the » menu at the end of the ribbon.
    More,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ButtonSize {
    pub width: f32,
    pub min_width: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PanelSize {
    pub buttons: Vec<ButtonSize>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Plan {
    pub fit: Fit,
    pub shown: usize,
    pub widths: Vec<f32>,
}

impl Plan {
    fn full(panel: &PanelSize) -> Self {
        Self { fit: Fit::Buttons, shown: panel.buttons.len(), widths: panel.buttons.iter().map(|b| b.width).collect() }
    }

    fn more() -> Self {
        Self { fit: Fit::More, shown: 0, widths: Vec::new() }
    }

    /// The panel's width on the ribbon (0 in the » menu).
    pub fn width(&self) -> f32 {
        match self.fit {
            Fit::More => 0.0,
            Fit::Collapsed => COLLAPSED_WIDTH,
            Fit::Buttons => panel_width(self.widths.get(..self.shown).unwrap_or_default()),
        }
    }
}

/// A panel's width with these individual button widths.
pub fn panel_width(buttons: &[f32]) -> f32 {
    (buttons.iter().sum::<f32>() + buttons.len().saturating_sub(1) as f32 * BUTTON_GAP + 8.0).max(COLLAPSED_WIDTH)
}

/// Fit panels into `room`, giving up as little as it takes, one small step at a time:
/// buttons narrow to their minimum, then panels hand their last buttons to a drop-down, then the
/// last panels go into the » menu. Nothing is dropped.
pub fn fit(panels: &[PanelSize], room: f32) -> Vec<Plan> {
    let room = if room.is_finite() { room } else { 0.0 };
    if let Some(plans) = shrink(panels, room) {
        return plans;
    }
    for kept in (0..panels.len()).rev() {
        if let Some(mut plans) = shrink(panels.get(..kept).unwrap_or_default(), room - MORE_ROOM) {
            plans.extend(panels.iter().skip(kept).map(|_| Plan::more()));
            return plans;
        }
    }
    panels.iter().map(|_| Plan::more()).collect()
}

fn used(plans: &[Plan]) -> f32 {
    plans.iter().filter(|p| p.fit != Fit::More).map(|p| p.width() + PANEL_GAP).sum()
}

fn shrink(panels: &[PanelSize], room: f32) -> Option<Vec<Plan>> {
    let mut plans: Vec<Plan> = panels.iter().map(Plan::full).collect();
    if used(&plans) <= room {
        return Some(plans);
    }
    for reduction in 1..=((BUTTON_MAX_WIDTH - BUTTON_MIN_WIDTH) as usize) {
        for (plan, panel) in plans.iter_mut().zip(panels) {
            for (width, button) in plan.widths.iter_mut().zip(&panel.buttons) {
                *width = (button.width - reduction as f32).max(button.min_width);
            }
        }
        if used(&plans) <= room {
            return Some(plans);
        }
    }
    while used(&plans) > room {
        let pick = plans
            .iter()
            .enumerate()
            .filter(|(_, p)| p.fit == Fit::Buttons && (p.shown > 1 || p.width() > COLLAPSED_WIDTH))
            .max_by(|(i, a), (j, b)| a.shown.cmp(&b.shown).then(i.cmp(j)));
        let (i, _) = pick?;
        if let Some(p) = plans.get_mut(i) {
            p.shown = p.shown.saturating_sub(1).max(1);
            if p.shown == 1 {
                p.fit = Fit::Collapsed;
            }
        }
    }
    Some(plans)
}

/// Draw the ribbon. Commands are dispatched after painting, so no app borrow outlives the frame.
pub fn show<A: ShellApp>(ui: &mut Ui, shell: &mut Shell, app: &mut A) {
    let t = Tokens::get();
    let layout = shell.settings.layout;
    let floating = layout == LayoutMode::Floating;
    let container = Container::new(ui.ctx(), Surface::Chrome);
    let fill = if floating { Color32::TRANSPARENT } else { t.toolbar };
    // The floating ribbon's band reaches from the content's top to the cards' bottom edge.
    let available = ui.available_rect_before_wrap();
    let cards = Cards::new(available, layout, RIBBON_HEIGHT);
    let height = if floating { cards.bottom - available.top() } else { RIBBON_HEIGHT };
    let mut action: Option<&'static str> = None;
    let mut switch_tab: Option<String> = None;
    container
        .edge("ribbon", Edge::Top)
        .exact_size(height)
        .frame(container.edge_frame(fill))
        .show(ui, |ui| body(ui, shell, app, floating, cards, &mut action, &mut switch_tab));
    if let Some(id) = switch_tab {
        shell.ribbon.active_tab = id;
    }
    if let Some(id) = action {
        shell.run(app, id);
    }
}

fn body<A: ShellApp>(
    ui: &mut Ui,
    shell: &Shell,
    app: &A,
    floating: bool,
    cards: Cards,
    action: &mut Option<&'static str>,
    switch_tab: &mut Option<String>,
) {
    let t = Tokens::get();
    ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
    let r = if floating { Rect::from_min_max(cards.workspace.min, cards.ribbon.max) } else { ui.max_rect() };
    let ws = if floating { cards.workspace } else { Rect::from_min_size(r.left_top(), vec2(WORKSPACE_WIDTH, RIBBON_HEIGHT - 1.0)) };
    let ribbon_left = if floating { cards.ribbon.left() } else { ws.right() };
    if floating {
        for card in [ws, cards.ribbon] {
            let frame = Container::new(ui.ctx(), Surface::Chrome).frame(ui.ctx(), t.toolbar, craft_ui::frame::Side::Left);
            ui.painter().add(frame.paint(card));
            // Blank card space owns clicks as well as its buttons.
            ui.interact(card, ui.id().with(("shell_card", card.left().to_bits())), Sense::click_and_drag());
        }
        craft_ui::scenario::publish_rect("ribbon:card", cards.ribbon);
    }
    let painter = ui.painter().clone();
    let active = shell.ribbon.active_tab.clone();
    let tabs = shell.workspace.tabs.as_slice();
    workspace_switcher(ui, &painter, ws, floating, app.title(), tabs, &active, switch_tab);
    craft_ui::scenario::publish_rect("toolbar", r);
    craft_ui::scenario::publish_rect("workspace_switcher", ws);
    craft_ui::scenario::publish_target("ribbon:workspace", ws);
    let row = Rect::from_min_max(pos2(ribbon_left, r.top()), pos2(r.right(), r.top() + TAB_ROW_HEIGHT));
    ribbon_tabs(ui, &painter, row, tabs, &active, floating, switch_tab);
    // Panels.
    let Some(tab) = tabs.iter().find(|tab| tab.id == active).or_else(|| tabs.first()) else { return };
    let top = r.top() + TAB_ROW_HEIGHT;
    let registry = app.commands();
    let sizes: Vec<PanelSize> = tab
        .panels
        .iter()
        .map(|panel| PanelSize {
            buttons: registry
                .resolve(panel.commands)
                .into_iter()
                .map(|command| {
                    let width = ribbon_button_width(&painter, tr(command.label));
                    ButtonSize { width, min_width: BUTTON_MIN_WIDTH }
                })
                .collect(),
        })
        .collect();
    let plans = fit(&sizes, r.right() - 4.0 - ribbon_left);
    craft_ui::scenario::publish_count("ribbon_collapsed", plans.iter().filter(|p| p.fit == Fit::Collapsed).count() as f64);
    let mut overflow: Vec<(&str, Vec<&CommandSpec>)> = Vec::new();
    let mut px = ribbon_left;
    for (panel, plan) in tab.panels.iter().zip(&plans) {
        let commands = registry.resolve(panel.commands);
        if plan.fit == Fit::More {
            overflow.push((panel.label, commands));
            continue;
        }
        let collapsed = plan.fit == Fit::Collapsed;
        let width = plan.width();
        let popup_id = ui.id().with(("panel_menu", &tab.id, panel.id));
        let drawn = if collapsed { 0 } else { plan.shown.min(commands.len()) };
        let buttons_width: f32 = plan.widths.iter().take(drawn).sum::<f32>() + drawn.saturating_sub(1) as f32 * BUTTON_GAP;
        let inset = if drawn == 0 { 0.0 } else { ((width - 8.0 - buttons_width) / 2.0).max(0.0).floor() };
        let bx = px + 4.0 + inset;
        let enabled_panel = !commands.is_empty();
        if collapsed {
            let br = Rect::from_min_size(pos2(px + 4.0, top + 2.0), vec2((width - 8.0).max(0.0), BUTTON_HEIGHT));
            let resp = ui.interact(br, ui.id().with(("panel_collapsed", panel.id)), Sense::click());
            if resp.hovered() && enabled_panel {
                painter.rect_filled(br, t.radius_sm, t.hover);
            }
            if let Some(command) = commands.first() {
                icons::paint_icon(&painter, Rect::from_center_size(br.center() - vec2(0.0, 3.0), vec2(26.0, 26.0)), command.icon, 1.0);
            }
            dropdown_arrow(&painter, pos2(br.center().x, br.bottom() - 3.0), t.text_dim);
            if resp.on_hover_text(tr(panel.label)).clicked() && enabled_panel && craft_ui::input::once_this_pass(ui.ctx(), "panel_collapsed") {
                toggle_panel_popup(ui.ctx(), popup_id);
            }
        } else {
            let mut button_x = bx;
            for (command, button_width) in commands.iter().zip(plan.widths.iter().copied()).take(drawn) {
                let whole = Rect::from_min_size(pos2(button_x, top + 2.0), vec2(button_width, BUTTON_HEIGHT));
                button_x += button_width + BUTTON_GAP;
                command_button(ui, &painter, whole, command, app, action);
            }
        }
        panel_label(ui, &painter, px, top, width, panel.label, tab.id, &commands, app, action, popup_id);
        craft_ui::scenario::publish_rect(
            &format!("panel:{}:{}", tab.id, panel.id),
            Rect::from_min_max(pos2(px, top + 2.0), pos2(px + width, r.bottom())),
        );
        px += width;
        painter.line_segment([pos2(px, top + 4.0), pos2(px, top + GROUP_HEIGHT - 3.0)], Stroke::new(1.0, t.border));
        px += PANEL_GAP;
    }
    if !overflow.is_empty() {
        let at = Rect::from_min_size(pos2(px.min(r.right() - 40.0), top + 4.0), vec2(38.0, BUTTON_HEIGHT));
        ui.scope_builder(egui::UiBuilder::new().max_rect(at), |ui| {
            let response = Menu::button(ui, "»", |ui| {
                for (panel, commands) in &overflow {
                    Menu::button(ui, tr(panel), |ui| {
                        for command in commands {
                            let mut item = MenuItem::new(command.id, command.label, command.icon);
                            item.shortcut = command.shortcut.unwrap_or("");
                            item.enabled = app.command_enabled(command.id);
                            if item.show(ui).clicked() && craft_ui::input::once_this_pass(ui.ctx(), "overflow_row") {
                                *action = Some(command.id);
                                ui.close();
                            }
                        }
                    });
                }
            });
            response.response.on_hover_text(tr("ui.more_panels"));
        });
    }
}

/// The workspace switcher: accent strip, artwork, the product name and a tab list.
fn workspace_switcher(
    ui: &mut Ui,
    painter: &egui::Painter,
    ws: Rect,
    floating: bool,
    title: &str,
    tabs: &[TabSpec],
    active: &str,
    switch_tab: &mut Option<String>,
) {
    let t = Tokens::get();
    let response = ui.interact(ws, ui.id().with("workspace_switcher"), Sense::click());
    if response.hovered() {
        painter.rect_filled(ws, if floating { t.radius_lg } else { 0 }, t.hover);
    }
    // Keep the accent straight while leaving the floating card's top corners round.
    let accent_inset = if floating { f32::from(t.radius_lg) } else { 0.0 };
    painter.rect_filled(
        Rect::from_min_size(ws.left_top() + vec2(accent_inset, 0.0), vec2((ws.width() - 2.0 * accent_inset).max(0.0), t.space_3)),
        egui::CornerRadius { nw: 0, ne: 0, sw: if floating { 2 } else { 0 }, se: if floating { 2 } else { 0 } },
        t.accent,
    );
    if !floating {
        painter.line_segment([ws.right_top(), ws.right_bottom()], Stroke::new(1.0, t.border));
    }
    icons::paint_icon(painter, Rect::from_center_size(pos2(ws.center().x, ws.top() + 39.0), vec2(30.0, 30.0)), "ws_design", 1.0);
    ribbon_text(ui, pos2(ws.center().x, ws.top() + 69.0), title, t.font_button, t.text, true);
    let label = tr("ui.workspace");
    let label_width = painter.layout_no_wrap(label.to_owned(), FontId::proportional(t.font_small), t.text_dim).size().x;
    ribbon_text(ui, pos2(ws.center().x - 6.0, ws.top() + 89.0), label, t.font_small, t.text_dim, false);
    dropdown_arrow(painter, pos2(ws.center().x - 6.0 + label_width / 2.0 + 7.0, ws.top() + 89.5), t.text_dim);
    let open = Id::new("craft_workspace_menu");
    if response.clicked() && craft_ui::input::once_this_pass(ui.ctx(), "workspace_menu") {
        ui.ctx().data_mut(|d| d.insert_temp(open, !d.get_temp::<bool>(open).unwrap_or(false)));
    }
    if !ui.ctx().data(|d| d.get_temp::<bool>(open).unwrap_or(false)) {
        return;
    }
    let popup = Container::new(ui.ctx(), Surface::Popup).area(open.with("area")).fixed_pos(ws.left_bottom()).order(egui::Order::Foreground).show(
        ui.ctx(),
        |ui| {
            Menu::frame().show(ui, |ui| {
                ui.set_width(190.0);
                for tab in tabs {
                    let mut item = MenuItem::new(tab.id, tab.label, "ws_design");
                    item.checked = Some(tab.id == active);
                    if item.show(ui).clicked() && craft_ui::input::once_this_pass(ui.ctx(), "workspace_row") {
                        *switch_tab = Some(tab.id.to_string());
                        ui.close();
                    }
                }
            });
        },
    );
    let outside = ui.ctx().input(|i| i.pointer.any_pressed()) && !popup.response.contains_pointer();
    if switch_tab.is_some() || outside || ui.ctx().input(|i| i.key_pressed(egui::Key::Escape)) {
        ui.ctx().data_mut(|d| d.insert_temp(open, false));
    }
}

/// The tab chips, then the reserved command-search field.
fn ribbon_tabs(ui: &mut Ui, painter: &egui::Painter, row: Rect, tabs: &[TabSpec], active: &str, floating: bool, switch_tab: &mut Option<String>) {
    let t = Tokens::get();
    let accent = AccentColors::current(ui.ctx());
    let search_width = CommandSearch::closed_width(ui.ctx());
    let inset = RIBBON_TAB_INSET;
    let corner = if floating { f32::from(t.radius_lg.saturating_sub(inset as u8)).max(2.0) } else { 5.0 };
    let search = Rect::from_min_size(pos2(row.right() - search_width - inset, row.top() + inset), vec2(search_width, TAB_ROW_HEIGHT - 2.0 * inset));
    // The palette draws the editor on the foreground layer, so it can grow over the ribbon.
    ui.ctx().data_mut(|d| d.insert_temp(Id::new(SEARCH_RECT), search));
    craft_ui::scenario::publish_rect("ribbon:search", search);
    let span = Rect::from_min_max(row.left_top() + vec2(inset, 0.0), pos2((search.left() - 6.0).max(row.left() + inset), row.bottom()));
    let widths: Vec<f32> = tabs
        .iter()
        .map(|tab| painter.layout_no_wrap(tr(tab.label).to_owned(), craft_ui::fonts::semibold(t.font_button), t.text).size().x + 24.0)
        .collect();
    ui.scope_builder(egui::UiBuilder::new().max_rect(span), |ui| {
        ui.set_clip_rect(span.intersect(ui.clip_rect()));
        egui::ScrollArea::horizontal()
            .id_salt("ribbon_tabs")
            .animated(false)
            .auto_shrink([false, false])
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
            .show(ui, |ui| {
                let width: f32 = widths.iter().sum::<f32>() + tabs.len().saturating_sub(1) as f32 * 2.0;
                let (all, _) = ui.allocate_exact_size(vec2(width, TAB_ROW_HEIGHT), Sense::hover());
                let mut x = all.left();
                for (tab, width) in tabs.iter().zip(widths) {
                    let chip = Rect::from_min_size(pos2(x, all.top() + inset), vec2(width, TAB_ROW_HEIGHT - 2.0 * inset));
                    let selected = tab.id == active;
                    let response = ui.interact(chip, ui.id().with(("tab", tab.id)), Sense::click());
                    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tr(tab.label)));
                    let fill = if selected {
                        accent.soft
                    } else if response.hovered() {
                        t.hover
                    } else {
                        Color32::TRANSPARENT
                    };
                    ui.painter().rect_filled(chip, corner, fill);
                    ribbon_text(ui, chip.center(), tr(tab.label), t.font_button, if selected { accent.ink } else { t.text }, selected);
                    if ui.clip_rect().contains(chip.center()) {
                        craft_ui::scenario::publish_target(&format!("ribbon:tab:{}", tab.id), chip);
                        craft_ui::scenario::publish_rect(&format!("toolbar_tab:{}", tab.id), chip);
                    }
                    if response.clicked() && craft_ui::input::once_this_pass(ui.ctx(), "ribbon_tab") {
                        *switch_tab = Some(tab.id.to_string());
                    }
                    x += width + 2.0;
                }
            });
    });
    painter.line_segment(
        [row.left_bottom() - vec2(0.0, 0.25), row.right_bottom() - vec2(0.0, 0.25)],
        Stroke::new(0.5, t.toolbar.lerp_to_gamma(t.border, 0.5)),
    );
}

/// One command button: icon over a label of at most two lines.
fn command_button<A: ShellApp>(ui: &mut Ui, painter: &egui::Painter, whole: Rect, command: &CommandSpec, app: &A, action: &mut Option<&'static str>) {
    let t = Tokens::get();
    let enabled = app.command_enabled(command.id);
    let response = ui.interact(whole, ui.id().with(("cmd", command.id)), if enabled { Sense::click() } else { Sense::hover() });
    if response.hovered() && enabled {
        painter.rect_filled(whole, t.radius_sm, t.hover);
    }
    let icon = Rect::from_center_size(pos2(whole.center().x, whole.top() + 16.0), vec2(26.0, 26.0));
    icons::paint_icon(painter, icon, command.icon, if enabled { 1.0 } else { 0.35 });
    let label = tr(command.label);
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, label));
    let ink = if enabled { t.text } else { t.text_dim };
    let galley = ribbon_button_galley(painter, label, whole.width() - BUTTON_LABEL_PADDING, ink);
    painter.galley(pos2(whole.center().x, whole.top() + 32.0), galley, ink);
    craft_ui::scenario::publish_target(&format!("ribbon:cmd:{}", command.id), whole);
    let tip = match command.shortcut {
        Some(key) => format!("{label} ({key})"),
        None => label.to_string(),
    };
    if response.on_hover_text(tip).clicked() && enabled && craft_ui::input::once_this_pass(ui.ctx(), "ribbon_cmd") {
        *action = Some(command.id);
    }
}

/// The 20 pt section label under a panel: its drop-down button.
#[allow(clippy::too_many_arguments)]
fn panel_label<A: ShellApp>(
    ui: &mut Ui,
    painter: &egui::Painter,
    px: f32,
    top: f32,
    width: f32,
    label: &str,
    tab: &str,
    commands: &[&CommandSpec],
    app: &A,
    action: &mut Option<&'static str>,
    popup_id: Id,
) {
    let t = Tokens::get();
    let accent = AccentColors::current(ui.ctx());
    let lr = Rect::from_min_size(
        pos2(px + RIBBON_TAB_INSET, top + GROUP_HEIGHT - SECTION_HEIGHT - (RIBBON_TAB_INSET - 1.0)),
        vec2((width - 2.0 * RIBBON_TAB_INSET).max(0.0), SECTION_HEIGHT),
    );
    let response = ui.interact(lr, ui.id().with(("panel", label)), Sense::click());
    craft_ui::scenario::publish_target(&format!("ribbon:section:{tab}:{label}"), lr);
    let open = ui.ctx().data(|d| d.get_temp::<Id>(Id::new("craft_panel_open"))) == Some(popup_id);
    let hot = response.hovered() && !commands.is_empty();
    if open || hot {
        painter.rect_filled(lr, 4.0, if open { accent.soft } else { t.hover });
    }
    let text = tr(label);
    let font = if open || hot { craft_ui::fonts::semibold(t.font_caption) } else { FontId::proportional(t.font_caption) };
    let label_width = painter.layout_no_wrap(text.to_owned(), font, t.text_dim).size().x;
    let ink = if open { accent.ink } else { t.text_dim };
    ribbon_text(ui, lr.center() - vec2(6.0, 0.0), text, t.font_caption, ink, open || hot);
    dropdown_arrow(painter, pos2(lr.center().x + label_width / 2.0 + 1.0, lr.center().y + 0.5), ink);
    if response.clicked() && !commands.is_empty() && craft_ui::input::once_this_pass(ui.ctx(), "panel_label") {
        toggle_panel_popup(ui.ctx(), popup_id);
    }
    if !open {
        return;
    }
    let area =
        Container::new(ui.ctx(), Surface::Popup).area(popup_id).fixed_pos(lr.left_bottom()).order(egui::Order::Foreground).show(ui.ctx(), |ui| {
            Menu::frame().show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                ui.set_width(240.0);
                let room = (ui.ctx().content_rect().bottom() - ui.cursor().top() - 64.0).max(t.control_height * 3.0);
                egui::ScrollArea::vertical().id_salt("panel_menu_rows").max_height(room).show(ui, |ui| {
                    for command in commands {
                        let mut item = MenuItem::new(command.id, command.label, command.icon);
                        item.shortcut = command.shortcut.unwrap_or("");
                        item.enabled = app.command_enabled(command.id);
                        if item.show(ui).clicked() && craft_ui::input::once_this_pass(ui.ctx(), "panel_row") {
                            *action = Some(command.id);
                            ui.ctx().data_mut(|d| d.remove::<Id>(Id::new("craft_panel_open")));
                            ui.close();
                        }
                    }
                });
            });
        });
    let outside = ui.ctx().input(|i| i.pointer.any_pressed())
        && !area.response.contains_pointer()
        && !lr.contains(ui.ctx().pointer_interact_pos().unwrap_or(egui::Pos2::ZERO));
    if outside || ui.ctx().input(|i| i.key_pressed(egui::Key::Escape)) {
        ui.ctx().data_mut(|d| d.remove::<Id>(Id::new("craft_panel_open")));
    }
}

fn toggle_panel_popup(ctx: &egui::Context, popup_id: Id) {
    ctx.data_mut(|d| {
        let key = Id::new("craft_panel_open");
        if d.get_temp::<Id>(key) == Some(popup_id) {
            d.remove::<Id>(key);
        } else {
            d.insert_temp(key, popup_id);
        }
    });
}

/// The dropdown chevron every ribbon menu shares.
fn dropdown_arrow(painter: &egui::Painter, at: egui::Pos2, ink: Color32) {
    let rect = Rect::from_center_size(at, vec2(7.0, 7.0));
    painter.line(
        vec![pos2(rect.left(), rect.top() + 1.5), pos2(rect.center().x, rect.bottom() - 1.0), pos2(rect.right(), rect.top() + 1.5)],
        Stroke::new(1.2, ink),
    );
}

/// Centred ribbon text, snapped to whole device pixels.
fn ribbon_text(ui: &Ui, at: egui::Pos2, text: &str, size: f32, color: Color32, bold: bool) {
    let font = if bold { craft_ui::fonts::semibold(size) } else { FontId::proportional(size) };
    let galley = ui.painter().layout_no_wrap(text.to_owned(), font, color);
    let origin = at - galley.size() / 2.0;
    let ppp = ui.ctx().pixels_per_point().max(0.5);
    let origin = pos2((origin.x * ppp).round() / ppp, (origin.y * ppp).round() / ppp);
    let painter = ui.painter().with_clip_rect(ui.clip_rect());
    painter.galley(origin, galley, color);
}

/// Choose at most two lines at spaces or after `/`, then truncate each line at Unicode character
/// boundaries. An oversized word stays on one line with an ellipsis.
fn ribbon_button_lines(painter: &egui::Painter, label: &str, width: f32) -> Vec<String> {
    let mut lines = ribbon_label_lines(painter, label, width);
    for line in &mut lines {
        if ribbon_label_width(painter, line) > width {
            while !line.is_empty() {
                line.pop();
                if ribbon_label_width(painter, &format!("{}…", line.trim_end())) <= width {
                    break;
                }
            }
            *line = format!("{}…", line.trim_end());
        }
    }
    lines
}

fn ribbon_label_width(painter: &egui::Painter, text: &str) -> f32 {
    let mut job = egui::text::LayoutJob::simple(text.to_owned(), FontId::proportional(Tokens::get().font_body), Color32::WHITE, f32::INFINITY);
    job.halign = egui::Align::Center;
    painter.layout_job(job).size().x
}

fn ribbon_button_width(painter: &egui::Painter, label: &str) -> f32 {
    let widest = ribbon_label_lines(painter, label, BUTTON_MAX_WIDTH - BUTTON_LABEL_PADDING)
        .iter()
        .map(|line| ribbon_label_width(painter, line))
        .fold(0.0_f32, f32::max);
    (widest + BUTTON_LABEL_PADDING).clamp(BUTTON_MIN_WIDTH, BUTTON_MAX_WIDTH)
}

fn ribbon_label_lines(painter: &egui::Painter, label: &str, width: f32) -> Vec<String> {
    let measure = |text: &str| ribbon_label_width(painter, text);
    let normalized = label.split_whitespace().collect::<Vec<_>>().join(" ");
    let label = normalized.as_str();
    let mut end = 0;
    for token in label.split_inclusive(|c: char| c.is_whitespace() || c == '/') {
        let next = end + token.len();
        if end > 0 && measure(label.get(..next).unwrap_or_default().trim()) > width {
            break;
        }
        end = next;
    }
    let (first, rest) = label.split_at(end);
    let mut lines = vec![first.trim().to_owned()];
    if !rest.trim().is_empty() {
        lines.push(rest.trim().to_owned());
    }
    lines
}

fn ribbon_button_galley(painter: &egui::Painter, label: &str, width: f32, color: Color32) -> std::sync::Arc<egui::Galley> {
    let lines = ribbon_button_lines(painter, label, width);
    // Explicit newlines only: egui must never split an oversized word.
    let mut job = egui::text::LayoutJob::simple(lines.join("\n"), FontId::proportional(Tokens::get().font_body), color, f32::INFINITY);
    job.halign = egui::Align::Center;
    painter.layout_job(job)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;
    use std::collections::BTreeSet;

    fn panel(count: usize) -> PanelSize {
        PanelSize { buttons: (0..count).map(|_| ButtonSize { width: BUTTON_MAX_WIDTH, min_width: BUTTON_MIN_WIDTH }).collect() }
    }

    #[test]
    fn every_command_stays_reachable_at_any_width() {
        let panels = vec![panel(4), panel(3), panel(2)];
        for room in [0.0, 40.0, 120.0, 240.0, 400.0, 700.0, 2000.0] {
            let plans = fit(&panels, room);
            assert_eq!(plans.len(), panels.len());
            let mut reachable: BTreeSet<usize> = BTreeSet::new();
            for (index, (plan, panel)) in plans.iter().zip(&panels).enumerate() {
                match plan.fit {
                    Fit::Buttons => {
                        for k in 0..plan.shown {
                            reachable.insert(index * 10 + k);
                        }
                        for k in plan.shown..panel.buttons.len() {
                            reachable.insert(index * 10 + k);
                        }
                    }
                    Fit::Collapsed | Fit::More => {
                        for k in 0..panel.buttons.len() {
                            reachable.insert(index * 10 + k);
                        }
                    }
                }
            }
            assert_eq!(reachable.len(), 9, "room {room}: every command is on a button, a drop-down or the » menu");
            assert!(used(&plans) <= room.max(0.0) + MORE_ROOM + 1.0, "room {room}: {plans:?}");
        }
    }

    #[test]
    fn fit_is_frame_independent() {
        let panels = vec![panel(3), panel(3)];
        let once = fit(&panels, 200.0);
        let twice = fit(&panels, 200.0);
        assert_eq!(once, twice);
        assert!(fit(&panels, f32::NAN).iter().all(|p| p.fit == Fit::More));
    }
}
