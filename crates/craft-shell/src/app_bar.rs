//! The application bar: the window's own title bar. Brand, Home, the app's menus, the file
//! actions, the document tabs, the right cluster and — where the window has no OS decorations —
//! the caption buttons.
//!
//! Adapted from SolidCraft (`crates/ui-egui/src/toolbar.rs`: `app_bar`, `file_menu`,
//! `app_bar_separator`), MIT OR Apache-2.0. See ATTRIBUTION.md.

use std::collections::BTreeMap;

use craft_ui::i18n::tr;
use craft_ui::icons;
use craft_ui::theme::Tokens;
use craft_ui::ui_kit::{Container, Edge, Menu, MenuItem, Surface};
use egui::{Align2, FontId, Id, Pos2, Rect, Sense, Stroke, Ui, pos2, vec2};

use crate::commands::CommandSpec;
use crate::keymap;
use crate::titlebar;
use crate::{DocumentTab, MenuEntry, Shell, ShellApp, builtin};

pub const HEIGHT: f32 = 34.0;

/// The open application menu: which menu, and where its button's top-left corner is.
const MENU: &str = "craft_app_menu";
/// The pass the menu opened on, so the click that opened it is not also a click outside.
const MENU_OPENED: &str = "craft_app_menu_opened";
/// The narrowest bar that still shows the product name, the version and all three file actions.
const WIDE: f32 = 640.0;

/// What a document tab asked for, run after painting.
enum DocAction {
    Select(usize),
    Close(usize),
}
pub fn show<A: ShellApp>(ui: &mut Ui, shell: &mut Shell, app: &mut A) {
    let t = Tokens::get();
    let mut action: Option<&'static str> = None;
    let mut doc_action: Option<DocAction> = None;
    let mut toggle_menu: Option<(usize, Pos2)> = None;
    let custom_titlebar = shell.settings.custom_titlebar && !cfg!(target_arch = "wasm32");
    let container = Container::new(ui.ctx(), Surface::ApplicationBar);
    let frame = container.frame(ui.ctx(), t.app_bar, craft_ui::frame::Side::Left);
    let documents = app.documents();
    let active_document = app.active_document();
    let registry = app.commands();
    let row = container.edge("app_bar", Edge::Top).exact_size(HEIGHT).show_separator_line(false).frame(frame).show(ui, |ui| {
        let r = ui.max_rect();
        if custom_titlebar {
            titlebar::drag_area(ui, r);
        }
        let captions = if custom_titlebar { titlebar::WIDTH } else { 0.0 };
        let right_cluster = r.right() - captions;
        let left = r.left();
        let compact = r.right() - left < WIDE;
        // Brand mark: the design workspace artwork, at the same 20 pt height as a block.
        {
            let mark = Rect::from_center_size(pos2(left + 20.0, r.center().y), vec2(20.0, 20.0));
            icons::paint_icon(ui.painter(), mark, "ws_design", 1.0);
        }
        if !compact {
            ui.painter().text(
                pos2(left + 36.0, r.center().y),
                Align2::LEFT_CENTER,
                app.title(),
                craft_ui::fonts::semibold(t.font_heading),
                t.app_bar_text,
            );
        }
        let mut x = left + if compact { 56.0 } else { 128.0 };
        let home = Rect::from_center_size(pos2(x - 4.0, r.center().y), vec2(32.0, 32.0));
        let home_response = ui.interact(home, ui.id().with("appbar_home"), Sense::click());
        let home_tip = tr("ui.start_page");
        home_response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, home_tip));
        craft_ui::scenario::publish_handle("titlebar:home", home.center());
        craft_ui::scenario::publish_target("appbar:home", home);
        if home_response.hovered() || home_response.has_focus() {
            ui.painter().rect_filled(home.shrink(3.0), t.radius_sm, t.app_bar_hover_button);
        }
        icons::paint_app_bar(ui.painter(), home.shrink(7.0), "home", 1.0);
        if home_response.on_hover_text(home_tip).clicked() && craft_ui::input::once_this_pass(ui.ctx(), "app_bar_home") {
            action = Some(builtin::SHELL_HOME);
        }
        x = home.right() + 8.0;
        app_bar_separator(ui, r, x, "brand");
        x += 16.0;
        // One flat text button per application menu.
        let open_menu = ui.ctx().data(|d| d.get_temp::<(usize, Pos2)>(Id::new(MENU))).map(|(index, _)| index);
        for (index, menu) in app.menus().iter().enumerate() {
            let label = tr(menu.title).to_string();
            let galley = ui.painter().layout_no_wrap(label.clone(), FontId::proportional(t.font_button), t.app_bar_text);
            // Never let a menu button run under the right cluster.
            if x - 13.0 + galley.size().x + 16.0 > right_cluster - 120.0 {
                break;
            }
            let br = Rect::from_min_size(pos2(x - 13.0, r.center().y - 13.0), vec2(galley.size().x + 16.0, 26.0));
            let resp = ui.interact(br, ui.id().with(("ab", menu.title)), Sense::click());
            resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &label));
            craft_ui::scenario::publish_target(&format!("appbar:menu:{}", index), br);
            if resp.hovered() || resp.has_focus() || open_menu == Some(index) {
                ui.painter().rect_filled(br, t.radius_sm, t.app_bar_hover);
            }
            ui.painter().galley(pos2(br.left() + 8.0, br.center().y - galley.size().y / 2.0), galley, t.app_bar_text);
            if resp.clicked() && craft_ui::input::once_this_pass(ui.ctx(), "app_bar_menu") {
                // The menu hangs from the button's left edge, one bar height down.
                toggle_menu = Some((index, br.left_top() + vec2(0.0, HEIGHT)));
            }
            x = br.right() + 17.0;
        }
        // Save, Undo and Redo follow the menus, from the registry's own specs.
        let step = if compact { 22.0 } else { 34.0 };
        let mut first_action = true;
        let mut clicked_action: Option<&'static str> = None;
        for id in ["shell.save", "shell.undo", "shell.redo"] {
            let Some(spec) = registry.get(id) else { continue };
            if compact && !first_action {
                continue;
            }
            first_action = false;
            let shortcut = keymap::effective(registry, &shell.keymap.custom, id);
            if file_action(ui, &t, x, r.center().y, spec, shortcut.as_deref(), app.command_enabled(id), compact) {
                clicked_action = Some(spec.id);
            }
            x += step;
        }
        if let Some(id) = clicked_action {
            action = Some(id);
        }
        let separator_x = x - step + if compact { 19.0 } else { 24.0 };
        app_bar_separator(ui, r, separator_x, "documents");
        let tabs_start = separator_x + 10.0;
        let (doc, home) = document_tabs(ui, &t, r, tabs_start, right_cluster - 8.0, &documents, active_document, custom_titlebar);
        if let Some(doc) = doc {
            doc_action = Some(doc);
        }
        if home {
            action = Some(builtin::SHELL_HOME);
        }
        // Right cluster: Help, Settings, the version, then the caption buttons.
        if bar_button(ui, &t, pos2(right_cluster - 70.0, r.center().y), "help", tr("ui.help"), "appbar:help") {
            action = Some(builtin::SHELL_ABOUT);
        }
        if bar_button(ui, &t, pos2(right_cluster - 104.0, r.center().y), "settings", tr("menu.preferences"), "appbar:prefs") {
            action = Some(builtin::SHELL_PREFERENCES);
        }
        if r.width() >= WIDE {
            ui.painter().text(
                pos2(right_cluster - 12.0, r.center().y),
                Align2::RIGHT_CENTER,
                format!("v{}", env!("CARGO_PKG_VERSION")),
                FontId::proportional(t.font_small),
                t.app_bar_text,
            );
        }
        if custom_titlebar {
            titlebar::caption_buttons(ui, r);
        }
        ui.painter().hline(r.x_range(), r.bottom(), Stroke::new(1.0, t.app_bar_separator.gamma_multiply(0.35)));
    });
    craft_ui::scenario::publish_rect("appbar", row.response.rect);
    if let Some((index, at)) = toggle_menu {
        toggle_menu_at(ui.ctx(), index, at);
    }
    show_menu(ui.ctx(), app, &shell.keymap.custom, &mut action);
    if let Some(id) = action {
        shell.run(app, id);
    }
    match doc_action {
        Some(DocAction::Select(index)) => app.select_document(index),
        Some(DocAction::Close(index)) => app.close_document(index),
        None => {}
    }
}

/// A short divider between the brand, file actions and document tabs.
fn app_bar_separator(ui: &Ui, bar: Rect, x: f32, group: &str) {
    let rect = Rect::from_center_size(pos2(x, bar.center().y), vec2(1.0, 20.0));
    ui.painter().line_segment(
        [rect.center_top(), rect.center_bottom()],
        Stroke::new(1.0, Tokens::get().app_bar.lerp_to_gamma(Tokens::get().app_bar_separator, 0.4)),
    );
    craft_ui::scenario::publish_rect(&format!("titlebar:separator:{group}"), rect);
}

/// A flat application-bar icon button: 32 pt square, always dark, app-bar ink.
fn bar_button(ui: &mut Ui, t: &Tokens, center: Pos2, icon: &str, label: &str, handle: &str) -> bool {
    let br = Rect::from_center_size(center, vec2(32.0, 32.0));
    let resp = ui.interact(br, ui.id().with(("ab", icon)), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, label));
    craft_ui::scenario::publish_target(handle, br);
    if resp.hovered() || resp.has_focus() {
        ui.painter().rect_filled(br.shrink(3.0), t.radius_sm, t.app_bar_hover_button);
    }
    icons::paint_app_bar(ui.painter(), Rect::from_center_size(br.center(), vec2(18.0, 18.0)), icon, 1.0);
    resp.on_hover_text(label).clicked()
}

/// A file action from its registry spec: a disabled one is dimmed and ignores clicks.
fn file_action(ui: &mut Ui, t: &Tokens, x: f32, y: f32, spec: &CommandSpec, shortcut: Option<&str>, enabled: bool, compact: bool) -> bool {
    let width = if compact { 22.0 } else { 32.0 };
    let br = Rect::from_center_size(pos2(x, y), vec2(width, 32.0));
    let resp = ui.interact(br, ui.id().with(("ab", spec.id)), if enabled { Sense::click() } else { Sense::hover() });
    let label = tr(spec.label);
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, label));
    craft_ui::scenario::publish_target(&format!("appbar:{}", spec.id), br);
    craft_ui::scenario::publish_target(&format!("titlebar:{}", spec.id), br);
    if enabled && (resp.hovered() || resp.has_focus()) {
        ui.painter().rect_filled(br.shrink(3.0), t.radius_sm, t.app_bar_hover_button);
    }
    icons::paint_app_bar(ui.painter(), Rect::from_center_size(br.center(), vec2(18.0, 18.0)), spec.icon, if enabled { 1.0 } else { 0.35 });
    let tip = match shortcut {
        Some(key) if enabled => format!("{label} ({key})"),
        _ => label.to_string(),
    };
    enabled && resp.on_hover_text(tip).clicked() && craft_ui::input::once_this_pass(ui.ctx(), spec.id)
}

/// The document tabs, then a `+` that opens Home. Returns the action a click asked for.
fn document_tabs(
    ui: &mut Ui,
    t: &Tokens,
    r: Rect,
    x0: f32,
    x1: f32,
    documents: &[DocumentTab],
    active: usize,
    custom_titlebar: bool,
) -> (Option<DocAction>, bool) {
    let n = documents.len();
    if n == 0 {
        return (None, false);
    }
    const MAX_FIT: f32 = 200.0;
    let room = (x1 - x0).max(0.0);
    let natural: Vec<f32> = documents
        .iter()
        .map(|doc| {
            let label = tab_label(doc);
            let width = ui.painter().layout_no_wrap(label, craft_ui::fonts::semibold(t.font_button), t.app_bar_text).size().x;
            (width + 40.0).clamp(72.0, MAX_FIT)
        })
        .collect();
    let (first, shown, shared) = tab_layout(n, active, room);
    let fit_all = shown == n && natural.iter().sum::<f32>() <= room - 34.0;
    let mut x = x0;
    let mut action = None;
    for (index, doc) in documents.iter().enumerate().skip(first).take(shown) {
        let width = if fit_all { natural.get(index).copied().unwrap_or(shared) } else { shared };
        let tab_rect = Rect::from_min_size(pos2(x, r.top() + 4.0), vec2(width - 2.0, r.height() - 8.0));
        let is_active = index == active;
        let resp = ui.interact(tab_rect, ui.id().with(("doc_tab", index)), Sense::click());
        craft_ui::scenario::publish_target(&format!("tab:design:{index}"), tab_rect);
        if custom_titlebar {
            if resp.drag_started_by(egui::PointerButton::Primary) {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
            } else if resp.double_clicked() {
                titlebar::toggle_maximize(ui.ctx());
            }
        }
        let fill = if is_active {
            t.app_bar_tab_active
        } else if resp.hovered() {
            t.app_bar_hover
        } else {
            egui::Color32::TRANSPARENT
        };
        ui.painter().rect_filled(tab_rect, t.radius_md, fill);
        let label = tab_label(doc);
        let ink = if is_active { t.app_bar_text } else { t.app_bar_text.gamma_multiply(0.7) };
        let font = if is_active { craft_ui::fonts::semibold(t.font_button) } else { FontId::proportional(t.font_button) };
        let galley = ui.painter().layout_no_wrap(label, font, ink);
        let clip = Rect::from_min_size(tab_rect.min, vec2((tab_rect.width() - 20.0).max(10.0), tab_rect.height()));
        ui.painter().with_clip_rect(clip).galley(pos2(tab_rect.left() + 10.0, tab_rect.center().y - galley.size().y / 2.0), galley, ink);
        if n > 1 && (is_active || resp.hovered()) {
            let close = Rect::from_center_size(pos2(tab_rect.right() - 13.0, tab_rect.center().y), vec2(16.0, 16.0));
            let close_resp = ui.interact(close, ui.id().with(("doc_close", index)), Sense::click());
            craft_ui::scenario::publish_target(&format!("tab:close:{index}"), close);
            if close_resp.hovered() || close_resp.has_focus() {
                ui.painter().rect_filled(close, 3.0, t.app_bar_hover_strong);
            }
            icons::paint_app_bar(
                ui.painter(),
                Rect::from_center_size(close.center(), vec2(14.0, 14.0)),
                "close",
                if close_resp.hovered() { 1.0 } else { 0.55 },
            );
            if close_resp.on_hover_text(tr("ui.close")).clicked() && craft_ui::input::once_this_pass(ui.ctx(), "doc_close") {
                action = Some(DocAction::Close(index));
            }
        }
        if action.is_none() && resp.clicked() && craft_ui::input::once_this_pass(ui.ctx(), "doc_select") {
            action = Some(DocAction::Select(index));
        }
        x += width;
    }
    // More tabs than fit: a » button lists them all.
    if shown < n {
        let more = Rect::from_center_size(pos2(x + 12.0, r.center().y), vec2(20.0, 22.0));
        let resp = ui.interact(more, ui.id().with("doc_more"), Sense::click());
        craft_ui::scenario::publish_target("tab:overflow", more);
        if resp.hovered() {
            ui.painter().rect_filled(more, 4.0, t.app_bar_hover_button);
        }
        ui.painter().text(more.center(), Align2::CENTER_CENTER, format!("»{}", n - shown), FontId::proportional(t.font_body), t.app_bar_text);
        let open = Id::new("craft_doc_more");
        if resp.clicked() {
            ui.ctx().data_mut(|d| d.insert_temp(open, !d.get_temp::<bool>(open).unwrap_or(false)));
        }
        if ui.ctx().data(|d| d.get_temp::<bool>(open).unwrap_or(false)) {
            let popup = Container::new(ui.ctx(), Surface::Popup)
                .area(open.with("area"))
                .fixed_pos(pos2(more.left(), r.bottom()))
                .order(egui::Order::Foreground)
                .show(ui.ctx(), |ui| {
                    Menu::frame().show(ui, |ui| {
                        ui.set_width(190.0);
                        for (index, doc) in documents.iter().enumerate() {
                            let mut item = MenuItem::new("document", doc.title.as_str(), "box");
                            item.checked = Some(index == active);
                            if item.show(ui).clicked() {
                                action = Some(DocAction::Select(index));
                                ui.close();
                            }
                        }
                    });
                });
            let outside = ui.ctx().input(|i| i.pointer.any_pressed())
                && !popup.response.contains_pointer()
                && !more.contains(ui.ctx().pointer_interact_pos().unwrap_or(Pos2::ZERO));
            if action.is_some() || outside || ui.ctx().input(|i| i.key_pressed(egui::Key::Escape)) {
                ui.ctx().data_mut(|d| d.insert_temp(open, false));
            }
        }
        x += 26.0;
    }
    // The + opens Home.
    let plus = Rect::from_center_size(pos2(x + 14.0, r.center().y), vec2(22.0, 22.0));
    let resp = ui.interact(plus, ui.id().with("doc_new"), Sense::click());
    craft_ui::scenario::publish_target("tab:new", plus);
    if resp.hovered() {
        ui.painter().rect_filled(plus, 4.0, t.app_bar_hover_button);
    }
    icons::paint_app_bar(ui.painter(), Rect::from_center_size(plus.center(), vec2(16.0, 16.0)), "plus", if resp.hovered() { 1.0 } else { 0.55 });
    let home = resp.on_hover_text(tr("ui.start_page")).clicked() && craft_ui::input::once_this_pass(ui.ctx(), "doc_new");
    (action, home)
}

fn tab_label(doc: &DocumentTab) -> String {
    format!("{}{}", doc.title, if doc.dirty { " •" } else { "" })
}

/// Which tabs the strip shows in `room` points: (first, how many, tab width). Tabs shrink to a
/// 90 pt minimum; past that a window of tabs around the active one is shown.
fn tab_layout(n: usize, active: usize, room: f32) -> (usize, usize, f32) {
    const MIN_TAB: f32 = 90.0;
    let n = n.max(1);
    let avail = (room - 34.0).max(MIN_TAB);
    if avail / n as f32 >= MIN_TAB {
        return (0, n, (avail / n as f32).min(220.0));
    }
    let fit = (((avail - 26.0) / MIN_TAB).floor() as usize).max(1);
    let first = active.saturating_sub(fit - 1).min(n.saturating_sub(fit));
    (first, fit, MIN_TAB)
}

fn toggle_menu_at(ctx: &egui::Context, index: usize, at: Pos2) {
    // Read the pass before taking the data lock: a Context call inside the closure would wait on
    // the lock we already hold.
    let pass = ctx.cumulative_pass_nr();
    ctx.data_mut(|d| {
        let open = d.get_temp::<(usize, Pos2)>(Id::new(MENU));
        let next = if open.map(|(i, _)| i) == Some(index) { None } else { Some((index, at)) };
        match next {
            Some(open) => {
                d.insert_temp(Id::new(MENU), open);
                d.insert_temp(Id::new(MENU_OPENED), pass);
            }
            None => d.remove::<(usize, Pos2)>(Id::new(MENU)),
        }
    });
}

/// The open application menu, and the row a click chose.
fn show_menu<A: ShellApp>(ctx: &egui::Context, app: &A, custom: &BTreeMap<String, String>, action: &mut Option<&'static str>) {
    let Some((index, at)) = ctx.data(|d| d.get_temp::<(usize, Pos2)>(Id::new(MENU))) else { return };
    let Some(menu) = app.menus().get(index) else {
        ctx.data_mut(|d| d.remove::<(usize, Pos2)>(Id::new(MENU)));
        return;
    };
    let mut close = false;
    let resp =
        Container::new(ctx, Surface::Popup).area(Id::new("craft_app_menu_area")).fixed_pos(at).order(egui::Order::Foreground).show(ctx, |ui| {
            Menu::frame().show(ui, |ui| {
                ui.set_width(240.0);
                for entry in &menu.items {
                    match entry {
                        MenuEntry::Separator => Menu::separator(ui),
                        MenuEntry::Command(id) => {
                            let Some(spec) = app.commands().get(id) else { continue };
                            let mut item = MenuItem::new(spec.id, spec.label, spec.icon);
                            let shortcut = keymap::effective(app.commands(), custom, spec.id);
                            item.shortcut = shortcut.as_deref().unwrap_or("");
                            item.enabled = app.command_enabled(spec.id);
                            let response = item.show(ui);
                            if response.clicked() && craft_ui::input::once_this_pass(ui.ctx(), &format!("menu:{}", spec.id)) {
                                *action = Some(spec.id);
                                close = true;
                                ui.close();
                            }
                        }
                        MenuEntry::Preferences => {
                            let registered = app.commands().get(builtin::SHELL_PREFERENCES).is_some();
                            let shortcut = if registered {
                                keymap::effective(app.commands(), custom, builtin::SHELL_PREFERENCES).unwrap_or_default()
                            } else {
                                "Ctrl+Comma".to_string()
                            };
                            let mut item = MenuItem::new("menu:preferences", "menu.preferences", "settings");
                            item.shortcut = &shortcut;
                            let response = item.show(ui);
                            craft_ui::scenario::publish_target("menu:preferences", response.rect);
                            if response.clicked() && craft_ui::input::once_this_pass(ui.ctx(), "menu:preferences") {
                                *action = Some(builtin::SHELL_PREFERENCES);
                                close = true;
                                ui.close();
                            }
                        }
                        MenuEntry::About => {
                            let response = MenuItem::new("menu:about", "menu.about", "info").show(ui);
                            craft_ui::scenario::publish_target("menu:about", response.rect);
                            if response.clicked() && craft_ui::input::once_this_pass(ui.ctx(), "menu:about") {
                                *action = Some(builtin::SHELL_ABOUT);
                                close = true;
                                ui.close();
                            }
                        }
                        MenuEntry::Quit => {
                            let response = MenuItem::new("menu:quit", "menu.quit", "close").show(ui);
                            craft_ui::scenario::publish_target("menu:quit", response.rect);
                            if response.clicked() && craft_ui::input::once_this_pass(ui.ctx(), "menu:quit") {
                                *action = Some(builtin::SHELL_QUIT);
                                close = true;
                                ui.close();
                            }
                        }
                    }
                }
            });
        });
    let just_opened = ctx.data(|d| d.get_temp::<u64>(Id::new(MENU_OPENED))) == Some(ctx.cumulative_pass_nr());
    if close || (!just_opened && (resp.response.clicked_elsewhere() || ctx.input(|i| i.key_pressed(egui::Key::Escape)))) {
        ctx.data_mut(|d| d.remove::<(usize, Pos2)>(Id::new(MENU)));
    }
}
