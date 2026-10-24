//! The hot-reloadable half of `craft-demo-hot`: the host owns the `CraftApp` state, this library
//! draws it. Rebuild this crate (only) while the host runs and the new UI appears on the next
//! frame; the shell state and the window survive.
//!
//! UI code reached from [`render`] — the whole shell and the demo — lives in this dylib, so edits
//! to `craft-shell`, `craft-ui` or `craft-demo` are picked up without restarting the app.
//!
//! Limits: the signature of [`render`] is fixed, and changing the layout of `CraftApp`/`Shell`/
//! `DemoApp` requires restarting the host (the host holds the state with the old layout).

pub use craft_demo::CraftApp;
pub use craft_demo::craft_shell::STORAGE_UI;
pub use craft_demo::craft_ui;

/// Draw one frame. The host calls the newest compiled copy of this function through
/// `hot-lib-reloader`.
#[unsafe(no_mangle)]
pub fn render(state: &mut CraftApp, ui: &mut egui::Ui) {
    state.draw(ui);
}
