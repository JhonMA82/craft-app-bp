//! The reference application built on `craft-shell`: a start page, generic panels, UI preferences
//! and the widget gallery. No application domain. The same code runs on desktop, in the browser
//! and inside the hot-reload host.

pub mod app;
pub mod gallery;

pub use app::{COMMANDS, CraftApp, DemoApp, View, workspace};
pub use craft_shell;
pub use craft_shell::{CommandRegistry, CommandSpec, PanelSpec, Shell, ShellApp, TabSpec, Workspace};
pub use craft_ui;
