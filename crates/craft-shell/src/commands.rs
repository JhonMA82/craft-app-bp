//! The command registry and the workspace: the metadata every surface (ribbon, menus, palette,
//! keymap) searches, and the only source of where a command appears.
//!
//! Adapted from SolidCraft (`crates/ui-egui/src/shortcut_box.rs` scoring, engine command specs),
//! MIT OR Apache-2.0. See ATTRIBUTION.md.

use std::collections::{BTreeMap, BTreeSet};

use craft_theme::Accent;
use craft_ui::i18n;

/// One command: a stable id, an i18n label id (`cmd.*`), an icon master and an optional default
/// shortcut. Where it appears is the [`Workspace`]'s business, not the spec's.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandSpec {
    pub id: &'static str,
    pub label: &'static str,
    pub icon: &'static str,
    pub shortcut: Option<&'static str>,
}

/// A ribbon panel: its title and the commands it lists, in order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PanelSpec {
    pub id: &'static str,
    pub label: &'static str,
    pub commands: &'static [&'static str],
}

/// A ribbon tab and its panels.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TabSpec {
    pub id: &'static str,
    pub label: &'static str,
    pub accent: Accent,
    pub panels: Vec<PanelSpec>,
}

/// The tabs and panels the ribbon draws. Commands may appear on several panels.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Workspace {
    pub tabs: Vec<TabSpec>,
}

impl Workspace {
    /// Every panel command exists in `registry`, and every tab and panel has an id and a label.
    pub fn validate(&self, registry: &CommandRegistry) -> Result<(), String> {
        if self.tabs.is_empty() {
            return Err("the workspace has no tabs".into());
        }
        let mut ids = BTreeSet::new();
        for tab in &self.tabs {
            if tab.id.is_empty() || tab.label.is_empty() {
                return Err(format!("tab `{}` needs an id and a label", tab.id));
            }
            if !ids.insert(tab.id) {
                return Err(format!("duplicate tab `{}`", tab.id));
            }
            let mut panels = BTreeSet::new();
            for panel in &tab.panels {
                if panel.id.is_empty() || panel.label.is_empty() {
                    return Err(format!("panel `{}` of tab `{}` needs an id and a label", panel.id, tab.id));
                }
                if !panels.insert(panel.id) {
                    return Err(format!("duplicate panel `{}` in tab `{}`", panel.id, tab.id));
                }
                for id in panel.commands {
                    if registry.get(id).is_none() {
                        return Err(format!("panel `{}` lists unknown command `{id}`", panel.id));
                    }
                }
            }
        }
        Ok(())
    }
}

/// The commands an app offers, indexed by id.
#[derive(Clone, Debug)]
pub struct CommandRegistry {
    specs: Vec<CommandSpec>,
    by_id: BTreeMap<&'static str, usize>,
}

impl CommandRegistry {
    /// Check the specs once at startup: ids are unique and non-empty, every label is a `cmd.*`
    /// i18n id, and no two commands default to the same shortcut.
    pub fn new(specs: Vec<CommandSpec>) -> Result<Self, String> {
        let mut by_id = BTreeMap::new();
        let mut shortcuts = BTreeSet::new();
        for (index, spec) in specs.iter().enumerate() {
            if spec.id.is_empty() {
                return Err("a command has an empty id".into());
            }
            if !spec.label.starts_with("cmd.") {
                return Err(format!("command `{}` label `{}` is not a cmd.* id", spec.id, spec.label));
            }
            if by_id.insert(spec.id, index).is_some() {
                return Err(format!("duplicate command `{}`", spec.id));
            }
            if let Some(key) = spec.shortcut
                && !shortcuts.insert(key)
            {
                return Err(format!("shortcut `{key}` is the default of more than one command"));
            }
        }
        Ok(Self { specs, by_id })
    }

    pub fn get(&self, id: &str) -> Option<&CommandSpec> {
        self.by_id.get(id).map(|index| &self.specs[*index])
    }

    pub fn specs(&self) -> &[CommandSpec] {
        &self.specs
    }

    /// The command ids on a tab, in panel order, without repeats.
    pub fn tab_commands(&self, tab: &TabSpec) -> Vec<&'static str> {
        let mut seen = BTreeSet::new();
        tab.panels.iter().flat_map(|p| p.commands.iter().copied()).filter(|id| seen.insert(*id)).collect()
    }

    /// Commands matching `query`, best first; an empty query lists everything by label.
    pub fn search(&self, query: &str) -> Vec<&CommandSpec> {
        let q = query.trim().to_lowercase();
        let mut scored: Vec<(u32, &CommandSpec)> = self
            .specs
            .iter()
            .filter_map(|spec| {
                let label = i18n::tr(spec.label);
                let score = score(&q, label).max(score(&q, spec.id) / 2);
                (score > 0).then_some((score, spec))
            })
            .collect();
        scored.sort_by(|(a_score, a), (b_score, b)| b_score.cmp(a_score).then_with(|| a.id.cmp(b.id)));
        scored.into_iter().map(|(_, spec)| spec).collect()
    }

    /// The specs a panel lists, in its order. Ids the registry doesn't know are skipped (the
    /// workspace validation rejects them first).
    pub fn resolve(&self, ids: &'static [&'static str]) -> Vec<&CommandSpec> {
        ids.iter().filter_map(|id| self.get(id)).collect()
    }
}

/// How well `q` matches `s` (0 = not at all): a prefix beats a word start beats a substring
/// beats the letters in order with gaps.
pub fn score(q: &str, s: &str) -> u32 {
    if q.is_empty() {
        return 1;
    }
    let s = s.to_lowercase();
    if s.starts_with(q) {
        return 1000 - s.len().min(500) as u32;
    }
    if s.split(|c: char| !c.is_alphanumeric()).any(|w| w.starts_with(q)) {
        return 800;
    }
    if s.contains(q) {
        return 600;
    }
    // Letters in order: fewer gaps score higher.
    let mut gaps = 0u32;
    let mut it = s.chars();
    let mut last_hit = true;
    for qc in q.chars() {
        let mut found = false;
        for c in it.by_ref() {
            if c == qc {
                found = true;
                break;
            }
            if last_hit {
                gaps += 1;
            }
            last_hit = false;
        }
        if !found {
            return 0;
        }
        last_hit = true;
    }
    300u32.saturating_sub(gaps * 20).max(10)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    fn spec(id: &'static str, label: &'static str, shortcut: Option<&'static str>) -> CommandSpec {
        CommandSpec { id, label, icon: "command", shortcut }
    }

    fn registry() -> CommandRegistry {
        CommandRegistry::new(vec![
            spec("app.alpha", "cmd.app.alpha", None),
            spec("app.delete", "cmd.app.delete", Some("Delete")),
            spec("app.undo", "cmd.app.undo", Some("Ctrl+Z")),
        ])
        .expect("valid registry")
    }

    #[test]
    fn duplicate_ids_and_shortcuts_are_rejected() {
        let dup = CommandRegistry::new(vec![spec("app.a", "cmd.app.a", None), spec("app.a", "cmd.app.b", None)]);
        assert!(dup.unwrap_err().contains("duplicate command"));
        let keys = CommandRegistry::new(vec![spec("app.a", "cmd.app.a", Some("Ctrl+Z")), spec("app.b", "cmd.app.b", Some("Ctrl+Z"))]);
        assert!(keys.unwrap_err().contains("Ctrl+Z"));
        let label = CommandRegistry::new(vec![spec("app.a", "Not a catalog id", None)]);
        assert!(label.unwrap_err().contains("cmd.*"));
    }

    #[test]
    fn search_finds_a_command_by_label_words_and_ids() {
        let registry = registry();
        let found: Vec<&str> = registry.search("alpha").iter().map(|s| s.id).collect();
        assert_eq!(found, ["app.alpha"]);
        let found: Vec<&str> = registry.search("app.delete").iter().map(|s| s.id).collect();
        assert_eq!(found.first(), Some(&"app.delete"));
        assert!(registry.search("zzz").is_empty());
        // Results are stable for an empty query.
        assert_eq!(registry.search(""), registry.search(" "));
    }

    #[test]
    fn workspace_validation_rejects_unknown_commands() {
        let registry = registry();
        let workspace = Workspace {
            tabs: vec![TabSpec {
                id: "workspace",
                label: "tab.workspace",
                accent: Accent::Blue,
                panels: vec![PanelSpec { id: "edit", label: "panel.edit", commands: &["app.missing"] }],
            }],
        };
        assert!(workspace.validate(&registry).unwrap_err().contains("app.missing"));
        assert!(Workspace::default().validate(&registry).unwrap_err().contains("no tabs"));
    }

    #[test]
    fn resolve_keeps_panel_order() {
        let registry = registry();
        let specs = registry.resolve(&["app.undo", "app.alpha"]);
        assert_eq!(specs.iter().map(|s| s.id).collect::<Vec<_>>(), ["app.undo", "app.alpha"]);
    }
}
