//! The command palette, drawn as SolidCraft's ribbon search: a field that sits in the ribbon at
//! rest and expands in place into a results sheet. Ctrl+K (or S outside a text field) opens it;
//! Enter runs the selection.
//!
//! Adapted from SolidCraft (`crates/ui-egui/src/shortcut_box.rs`: `show`), MIT OR Apache-2.0.
//! The typed command syntax, design-row search and context menus are not ported. See
//! ATTRIBUTION.md.

use craft_ui::i18n::tr;
use craft_ui::layout::LayoutMode;
use craft_ui::theme::Tokens;
use craft_ui::ui_kit::{CommandSearch, Container, MenuItem, SearchFieldGeometry, Surface};
use egui::{Id, Key, Rect, Sense, Ui, pos2, vec2};

use crate::commands::CommandSpec;
use crate::ribbon::SEARCH_RECT;
use crate::{Shell, ShellApp};

const ROWS: usize = 12;
const RECENT: usize = 12;
const MAX_QUERY: usize = 200;
/// The search editor's focus id.
pub const FIELD_ID: &str = "craft_sbox_text";

/// Draw the palette. Returns a command id to run after painting.
pub fn show<A: ShellApp>(ui: &mut Ui, shell: &mut Shell, app: &mut A) -> Option<&'static str> {
    let t = Tokens::get();
    let ctx = ui.ctx().clone();
    let screen = ctx.content_rect();
    let open = shell.palette.open;
    let ribbon = ctx.data(|d| d.get_temp::<Rect>(Id::new(SEARCH_RECT)));
    if !open && ribbon.is_none() {
        return None;
    }
    // Respect egui's instant/reduced animation policy, with a 120 ms upper bound.
    let progress = ctx.animate_bool_with_time_and_easing(Id::new("craft_search_expansion"), open, 0.12, egui::emath::easing::cubic_out);
    let focused = ctx.memory(|m| m.focused());
    if open && focused.is_none() {
        ctx.memory_mut(|m| m.request_focus(Id::new(FIELD_ID)));
    }
    let (up, down, enter, escape) = ctx.input_mut(|i| {
        (
            i.consume_key(egui::Modifiers::NONE, Key::ArrowUp),
            i.consume_key(egui::Modifiers::NONE, Key::ArrowDown),
            i.consume_key(egui::Modifiers::NONE, Key::Enter),
            i.consume_key(egui::Modifiers::NONE, Key::Escape),
        )
    });
    if escape && open {
        shell.palette.open = false;
        return None;
    }
    let width = CommandSearch::width().min((screen.width() - t.space_8).max(t.space_24));
    // Without the ribbon (Home) the field anchors under the app bar's centre.
    let fallback = pos2(screen.center().x - width / 2.0, screen.top() + t.space_24);
    let closed = ribbon.unwrap_or_else(|| Rect::from_min_size(fallback, vec2(width, t.control_height)));
    let field_size = egui::lerp(closed.size()..=vec2(width, t.control_height + t.space_12), progress);
    let pos = pos2(closed.right() - field_size.x, closed.top());
    let height = (screen.bottom() - pos.y - field_size.y - t.space_8).max(t.space_24 * 4.0);
    let floating = LayoutMode::current(&ctx) == LayoutMode::Floating;
    let mut query = if open { bounded(&shell.palette.query) } else { String::new() };
    let results = ranked(app.commands(), shell);
    let enabled: Vec<bool> = results.iter().map(|command| app.command_enabled(command.id)).collect();
    if up {
        step_selection(&mut shell.palette.selected, &enabled, false);
    }
    if down {
        step_selection(&mut shell.palette.selected, &enabled, true);
    }
    shell.palette.selected = shell.palette.selected.min(enabled.len().saturating_sub(1));
    if !enabled.get(shell.palette.selected).copied().unwrap_or(false) {
        shell.palette.selected = enabled.iter().position(|on| *on).unwrap_or(0);
    }
    let selected = shell.palette.selected;
    let mut run: Option<&'static str> = None;
    let mut open_requested = false;
    Container::new(&ctx, Surface::Popup)
        .area(Id::new("craft_sbox"))
        .fixed_pos(pos)
        .order(egui::Order::Foreground)
        .fade_in(false)
        .constrain(false)
        .show(&ctx, |ui| {
            // Area rounds its origin; place the child at the exact interpolated bounds so the
            // field's right edge does not drift during expansion.
            ui.scope_builder(egui::UiBuilder::new().max_rect(Rect::from_min_size(pos, vec2(field_size.x, height + field_size.y))), |ui| {
                ui.set_width(field_size.x);
                ui.spacing_mut().item_spacing.y = 0.0;
                let field = CommandSearch::field(
                    ui,
                    &mut query,
                    false,
                    open && focused.is_none(),
                    open,
                    SearchFieldGeometry { height: field_size.y, radius: CommandSearch::field_radius(progress, floating), progress },
                    MAX_QUERY,
                );
                if field.changed() {
                    shell.palette.selected = 0;
                }
                if !open && progress == 0.0 {
                    let hit =
                        field.clicked() || ui.interact(Rect::from_min_size(pos, field_size), ui.id().with("search_open"), Sense::click()).clicked();
                    if hit && craft_ui::input::once_this_pass(&ctx, "palette_open") {
                        open_requested = true;
                        ctx.request_repaint();
                    }
                    return;
                }
                ui.add_space(t.space_6 * progress);
                ui.add_enabled_ui(open, |ui| {
                    CommandSearch::sheet(ui, field_size.x, progress, false, |ui| {
                        egui::ScrollArea::vertical().max_height((height - t.control_height * 4.0).max(t.control_height)).show(ui, |ui| {
                            if results.is_empty() {
                                ui.add_space(t.space_8);
                                ui.label(egui::RichText::new(tr("palette.no_results")).size(t.font_body).color(t.text_dim));
                                ui.add_space(t.space_8);
                            }
                            for (index, command) in results.iter().enumerate() {
                                let mut item = MenuItem::new(command.id, tr(command.label), command.icon);
                                item.enabled = enabled.get(index).copied().unwrap_or(false);
                                item.highlighted = index == selected;
                                item.shortcut = command.shortcut.unwrap_or("");
                                let path = path_of(&shell.workspace, command.id);
                                let response = item.search_row(ui, &query, &path, None);
                                if item.highlighted && (up || down) {
                                    ui.scroll_to_rect(response.rect, None);
                                }
                                if response.clicked() && item.enabled && craft_ui::input::once_this_pass(ui.ctx(), "palette_row") {
                                    run = Some(command.id);
                                }
                            }
                        });
                    });
                });
            });
        });
    if !open {
        if open_requested {
            shell.palette.open = true;
            shell.palette.query.clear();
            shell.palette.selected = 0;
            ctx.memory_mut(|m| m.request_focus(Id::new(FIELD_ID)));
        }
        return None;
    }
    if query.len() > MAX_QUERY {
        query.truncate(MAX_QUERY);
    }
    shell.palette.query = query;
    if enter
        && run.is_none()
        && let Some(command) = results.get(selected)
    {
        run = Some(command.id);
    }
    if let Some(id) = run {
        remember_pick(shell, id);
        shell.palette.open = false;
    }
    run
}

fn bounded(text: &str) -> String {
    if text.chars().count() <= MAX_QUERY { text.to_string() } else { text.chars().take(MAX_QUERY).collect() }
}

/// Move the selection by one, skipping disabled rows and wrapping at either end.
fn step_selection(selected: &mut usize, enabled: &[bool], forward: bool) {
    let n = enabled.len();
    if n == 0 {
        *selected = 0;
        return;
    }
    let mut index = *selected;
    for _ in 0..n {
        index = if forward { (index + 1) % n } else { (index + n - 1) % n };
        if enabled.get(index).copied().unwrap_or(false) {
            *selected = index;
            return;
        }
    }
}

fn remember_pick(shell: &mut Shell, id: &str) {
    shell.palette.recent.retain(|recent| recent != id);
    shell.palette.recent.insert(0, id.to_string());
    shell.palette.recent.truncate(RECENT);
}

/// Pinned first, then recent, then the best matches.
fn ranked<'a>(registry: &'a crate::commands::CommandRegistry, shell: &Shell) -> Vec<&'a CommandSpec> {
    let mut out: Vec<&CommandSpec> = Vec::new();
    for id in shell.palette.pinned.iter().chain(&shell.palette.recent) {
        if let Some(spec) = registry.get(id)
            && !out.iter().any(|o| o.id == spec.id)
        {
            out.push(spec);
        }
    }
    for spec in registry.search(&shell.palette.query) {
        if !out.iter().any(|o| o.id == spec.id) {
            out.push(spec);
        }
        if out.len() >= ROWS {
            break;
        }
    }
    out.truncate(ROWS);
    out
}

/// Where a command lives ("Workspace › Create"), for the row's second line.
fn path_of(workspace: &crate::commands::Workspace, id: &str) -> String {
    for tab in &workspace.tabs {
        for panel in &tab.panels {
            if panel.commands.contains(&id) {
                return format!("{} › {}", tr(tab.label), tr(panel.label));
            }
        }
    }
    String::new()
}
