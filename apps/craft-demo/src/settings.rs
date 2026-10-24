//! Crash-safe desktop settings for the reference app.
//!
//! The shell state people can edit (`UiSettings`, `Panels`) lives in `config.toml`; transient UI
//! state (active tab, palette history and the host's window state) stays in the shell's eframe
//! storage. `craft-settings` owns the file, backup, recovery and migration; `craft-atomic` is
//! underneath it.
//!
//! On wasm there is no settings store: the web host keeps using the shell JSON in local storage.

use std::time::Instant;

use craft_settings::{Location, SaveStatus, Settings, SettingsStore, Source};
use craft_shell::{Panels, Shell, UiSettings};
use serde::{Deserialize, Serialize};

/// What `config.toml` keeps. `#[serde(default)]` on both levels means older files keep loading.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DemoSettings {
    pub ui: UiSettings,
    pub panels: Panels,
}

impl Settings for DemoSettings {
    const APP_ID: &'static str = "craft-demo";
    const SCHEMA_VERSION: u32 = 1;
}

/// The live store and the shell it mirrors.
pub struct SettingsHost {
    store: SettingsStore<DemoSettings>,
}

impl SettingsHost {
    /// Open `config.toml`, or import the old eframe JSON on the first run that has no file.
    ///
    /// `legacy` is the shell JSON the host read from eframe storage. It is adopted only when the
    /// store reports defaults, so a user-edited `config.toml` always wins.
    pub fn open(shell: &mut Shell, legacy: Option<&str>) -> Self {
        let (mut store, report) = SettingsStore::<DemoSettings>::open(Location::Auto);
        if let Some(reason) = &report.read_only {
            log::warn!("settings are read-only: {reason}");
        }
        if let Some(broken) = &report.broken_moved_to {
            log::warn!("damaged settings moved to {}", broken.display());
        }
        if report.source == Source::Defaults {
            if let Some(text) = legacy {
                shell.load(text);
            }
            store.set(DemoSettings { ui: shell.settings.clone(), panels: shell.panels });
        } else {
            let saved = store.get().clone();
            shell.settings = saved.ui;
            shell.panels = saved.panels;
            shell.keymap.custom = shell.settings.keymap.clone();
        }
        Self { store }
    }

    /// Copy the shell's live settings into the store. `update` only marks it dirty on a change.
    pub fn sync(&mut self, shell: &Shell) {
        let saved = self.store.get();
        if saved.ui == shell.settings && saved.panels == shell.panels && saved.ui.keymap == shell.keymap.custom {
            return;
        }
        let mut ui = shell.settings.clone();
        ui.keymap = shell.keymap.custom.clone();
        let panels = shell.panels;
        self.store.update(move |saved| {
            saved.ui = ui;
            saved.panels = panels;
        });
    }

    /// Save after the debounce and ask egui for a frame at the retry deadline, so an idle app
    /// still saves. A failure is logged and retried by the store.
    pub fn tick(&mut self, ctx: &egui::Context) {
        let now = Instant::now();
        let tick = self.store.tick(now);
        if let Some(deadline) = tick.next_deadline {
            ctx.request_repaint_after(deadline.saturating_duration_since(now));
        }
        if let SaveStatus::Failed(error) = tick.status {
            log::error!("settings: {error}");
        }
    }

    /// Flush on exit. A failure is logged; the previous file remains intact.
    pub fn flush(&mut self) {
        if let Err(error) = self.store.flush() {
            log::error!("settings: {error}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use craft_settings::Location;

    #[test]
    fn memory_store_round_trips_the_shell_keymap() {
        let (mut store, report) = SettingsStore::<DemoSettings>::open(Location::Memory);
        assert_eq!(report.source, Source::Defaults);
        let mut settings = DemoSettings::default();
        settings.ui.keymap.insert("shell.save".into(), "Ctrl+Shift+S".into());
        assert!(store.set(settings.clone()));
        assert_eq!(store.get(), &settings);
        assert!(store.flush().is_ok());
    }
}
