//! Keyboard shortcuts: every command's default key comes from its [`CommandSpec`]; the user can
//! rebind or clear any of them in Preferences › Shortcuts. A new key that another command already
//! uses is reported and only taken over on request; keys the shell handles itself can't be bound.
//!
//! Adapted from SolidCraft (`crates/ui-egui/src/keymap.rs`), MIT OR Apache-2.0. See ATTRIBUTION.md.

use std::collections::BTreeMap;

use craft_ui::i18n::{self, tr};
use craft_ui::theme::Tokens;
use craft_ui::ui_kit::{self, Button, Text};
use egui::Ui;

use crate::commands::{CommandRegistry, CommandSpec};

/// Keys the shell handles before any command shortcut.
pub const RESERVED: &[&str] = &["Escape", "Ctrl+S", "Ctrl+N", "Ctrl+O", "Ctrl+K", "Ctrl+Shift+P"];

/// What the Shortcuts page asks the shell to do after painting.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum KeymapAction {
    #[default]
    None,
    Reset(String),
    ResetAll,
    Close,
}

/// Rebindings kept between runs. Only the commands the user changed are listed; a command
/// mapped to `""` has no key.
#[derive(Clone, Debug, Default)]
pub struct Keymap {
    pub custom: BTreeMap<String, String>,
    pub open: bool,
    pub filter: String,
    capturing: Option<String>,
    conflict: Option<(String, String, Vec<String>)>,
    message: Option<String>,
}

/// A key press as a shortcut name: `E`, `Shift+E`, `Ctrl+Alt+K`, `F5`, `Delete`.
pub fn key_name(key: egui::Key, mods: egui::Modifiers) -> String {
    let mut s = String::new();
    if mods.command || mods.ctrl {
        s.push_str("Ctrl+");
    }
    if mods.alt {
        s.push_str("Alt+");
    }
    if mods.shift {
        s.push_str("Shift+");
    }
    s.push_str(key.name());
    s
}

/// The key a command runs from now.
pub fn effective(registry: &CommandRegistry, custom: &BTreeMap<String, String>, id: &str) -> Option<String> {
    match custom.get(id) {
        Some(k) if k.is_empty() => None,
        Some(k) => Some(k.clone()),
        None => registry.get(id).and_then(|c| c.shortcut).map(str::to_string),
    }
}

/// Commands bound to `key` now.
pub fn commands_for(registry: &CommandRegistry, custom: &BTreeMap<String, String>, key: &str) -> Vec<&'static str> {
    registry.specs().iter().filter(|c| effective(registry, custom, c.id).as_deref() == Some(key)).map(|c| c.id).collect()
}

/// The command a key press runs, if any.
pub fn command_for(registry: &CommandRegistry, custom: &BTreeMap<String, String>, key: &str) -> Option<&'static str> {
    commands_for(registry, custom, key).into_iter().next()
}

/// Bind `key` to a command (`""` clears it). Refused for reserved keys, and when other commands
/// use the key unless `replace` (they lose it).
pub fn bind(
    registry: &CommandRegistry,
    custom: &mut BTreeMap<String, String>,
    id: &str,
    key: &str,
    replace: bool,
    reserved: &[&str],
) -> Result<(), String> {
    let spec = registry.get(id).ok_or_else(|| format!("no command `{id}`"))?;
    if reserved.contains(&key) || RESERVED.contains(&key) {
        return Err(i18n::format("keymap.reserved", &[("key", key)]));
    }
    if !key.is_empty() {
        let others: Vec<&'static str> = commands_for(registry, custom, key).into_iter().filter(|o| *o != id).collect();
        if !others.is_empty() && !replace {
            let names: Vec<&str> = others.iter().map(|o| registry.get(o).map(|c| i18n::tr(c.label)).unwrap_or(o)).collect();
            return Err(i18n::format("keymap.conflict", &[("key", key), ("command", &names.join(", "))]));
        }
        for other in others {
            custom.insert(other.to_string(), String::new());
        }
    }
    if spec.shortcut.unwrap_or("") == key {
        custom.remove(id);
    } else {
        custom.insert(id.to_string(), key.to_string());
    }
    Ok(())
}

pub fn reset(custom: &mut BTreeMap<String, String>, id: Option<&str>) {
    match id {
        Some(id) => drop(custom.remove(id)),
        None => custom.clear(),
    }
}

/// The command a key press runs, unless the shell reserved the key.
pub fn press(registry: &CommandRegistry, custom: &BTreeMap<String, String>, reserved: &[&str], key: &str) -> Option<&'static str> {
    if reserved.contains(&key) || RESERVED.contains(&key) {
        return None;
    }
    command_for(registry, custom, key)
}

/// The Shortcuts page: filter, capture a new key, resolve a conflict, reset.
pub fn page(ui: &mut Ui, keymap: &mut Keymap, registry: &CommandRegistry, reserved: &[&str]) -> KeymapAction {
    // A key press while waiting for one binds it (Esc cancels).
    if let Some(id) = keymap.capturing.clone() {
        let pressed = ui.input(|i| {
            i.events.iter().find_map(|e| match e {
                egui::Event::Key { key, pressed: true, modifiers, .. } => Some((*key, *modifiers)),
                _ => None,
            })
        });
        if let Some((key, modifiers)) = pressed {
            keymap.capturing = None;
            if key != egui::Key::Escape {
                let name = key_name(key, modifiers);
                match bind(registry, &mut keymap.custom, &id, &name, false, reserved) {
                    Ok(()) => keymap.message = None,
                    Err(e) => {
                        let others = commands_for(registry, &keymap.custom, &name).iter().map(|s| s.to_string()).collect();
                        keymap.conflict = Some((id.clone(), name, others));
                        keymap.message = Some(e);
                    }
                }
            }
        }
    }
    let mut action = KeymapAction::None;
    let t = Tokens::get();
    ui_kit::settings_scroll(ui, "keymap_scroll", |ui| {
        ui.horizontal(|ui| {
            let (_, changed) = ui_kit::text_field(ui, &mut keymap.filter, tr("keymap.filter"));
            if changed {
                keymap.conflict = None;
            }
            if ui_kit::button(ui, tr("prefs.reset_all"), Button::Normal).clicked() {
                action = KeymapAction::ResetAll;
            }
        });
        ui.add_space(t.space_4);
        if let Some(message) = &keymap.message {
            ui_kit::text(ui, message, Text::Report("keymap_message", true));
        }
        if let Some((id, key, _)) = keymap.conflict.clone() {
            ui.horizontal_wrapped(|ui| {
                let replace = ui_kit::button(ui, tr("keymap.replace"), Button::Normal);
                if replace.clicked() {
                    match bind(registry, &mut keymap.custom, &id, &key, true, reserved) {
                        Ok(()) => {
                            keymap.conflict = None;
                            keymap.message = None;
                        }
                        Err(e) => keymap.message = Some(e),
                    }
                }
                if ui_kit::button(ui, tr("ui.cancel"), Button::Normal).clicked() {
                    keymap.conflict = None;
                    keymap.message = None;
                }
            });
        }
        let query = keymap.filter.to_lowercase();
        let matching: Vec<&CommandSpec> = registry
            .specs()
            .iter()
            .filter(|spec| query.is_empty() || i18n::tr(spec.label).to_lowercase().contains(&query) || spec.id.to_lowercase().contains(&query))
            .collect();
        if matching.is_empty() {
            ui_kit::text(ui, tr("ui.no_results"), Text::Note("keymap_none"));
        }
        for spec in matching {
            let current = effective(registry, &keymap.custom, spec.id);
            let capturing = keymap.capturing.as_deref() == Some(spec.id);
            ui.horizontal(|ui| {
                ui.set_min_width((ui.available_width() - t.form_card_width).max(t.form_card_width));
                ui.label(egui::RichText::new(i18n::tr(spec.label)).size(t.font_label));
                ui.add_space(t.space_8);
                let label = match (&current, capturing) {
                    (_, true) => tr("keymap.press").to_string(),
                    (Some(key), false) => key.clone(),
                    (None, false) => tr("ui.none").to_string(),
                };
                if ui_kit::button(ui, label, Button::Normal).clicked() {
                    keymap.capturing = Some(spec.id.to_string());
                    keymap.message = None;
                }
                let changed = keymap.custom.contains_key(spec.id);
                if changed && ui_kit::button(ui, tr("keymap.reset"), Button::Normal).clicked() {
                    action = KeymapAction::Reset(spec.id.to_string());
                }
                craft_ui::scenario::publish_target(&format!("keymap:{}", spec.id), ui.min_rect());
            });
        }
    });
    action
}
