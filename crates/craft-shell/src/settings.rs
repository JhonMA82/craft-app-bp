//! Persisted shell state. `UiSettings` is the live appearance; `SavedShell` is the JSON written to
//! storage and read back tolerantly (unknown keys and out-of-range values are ignored).

use std::collections::BTreeMap;

use craft_ui::layout::LayoutMode;
use craft_ui::theme::ColorVision;
use serde::{Deserialize, Serialize};

use crate::panels::Panels;

/// The storage key the demo app uses for the shell state.
pub const STORAGE_UI: &str = "craft.ui";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct UiSettings {
    /// `"dark"` or `"light"`.
    pub theme: String,
    /// A catalogue id (`"en"`, `"es"`).
    pub language: String,
    pub layout: LayoutMode,
    pub scale: f32,
    /// The semantic-colour mode; `Off` keeps the palette as drawn.
    pub color_vision: ColorVision,
    /// Draw our own title bar and window buttons (the OS keeps its decorations otherwise).
    pub custom_titlebar: bool,
    /// Command id → key; only the commands the user changed.
    pub keymap: BTreeMap<String, String>,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            theme: "dark".into(),
            language: "en".into(),
            layout: LayoutMode::Floating,
            scale: 1.0,
            color_vision: ColorVision::Off,
            custom_titlebar: default_custom_titlebar(),
            keymap: BTreeMap::new(),
        }
    }
}

/// Our own title bar is only possible on a window we create without OS decorations.
fn default_custom_titlebar() -> bool {
    cfg!(all(not(target_arch = "wasm32"), not(target_os = "macos")))
}

impl UiSettings {
    pub fn dark(&self) -> bool {
        self.theme != "light"
    }

    /// Values outside what the shell can draw are clamped; an unknown category is the default.
    pub fn sanitize(&mut self) {
        if self.theme != "light" && self.theme != "dark" {
            self.theme = "dark".into();
        }
        if !craft_ui::i18n::is_supported(&self.language) {
            self.language = "en".into();
        }
        if !self.scale.is_finite() {
            self.scale = 1.0;
        }
        self.scale = self.scale.clamp(0.75, 2.0);
        // An undecorated window without our caption buttons cannot be closed, so the custom
        // title bar is mandatory wherever the window has no OS chrome.
        if default_custom_titlebar() {
            self.custom_titlebar = true;
        }
        self.keymap.retain(|_, key| key.chars().count() <= 40);
    }
}

/// Everything the shell keeps between runs.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SavedShell {
    pub ui: UiSettings,
    pub panels: Panels,
    pub active_tab: String,
    pub palette_recent: Vec<String>,
    pub palette_pinned: Vec<String>,
}

impl SavedShell {
    pub fn sanitize(&mut self) {
        self.ui.sanitize();
        self.panels.sanitize();
    }
}
