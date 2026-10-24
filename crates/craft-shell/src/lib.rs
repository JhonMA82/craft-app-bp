//! The application shell: app bar, ribbon, side panels, status line, command palette, preferences,
//! messages, toasts and keyboard shortcuts. An app supplies the model, its commands and its
//! panels through [`ShellApp`].
//!
//! Adapted from SolidCraft (`crates/ui-egui/src/shell.rs`, `toolbar.rs`, `shortcut_box.rs`),
//! MIT OR Apache-2.0. See ATTRIBUTION.md.

use std::collections::VecDeque;

use craft_ui::layout::LayoutMode;
use craft_ui::theme::{AccentColors, ColorVision};
use craft_ui::ui_kit::{Action, Confirm, Container, Surface, Tone};
use egui::Ui;
use serde::{Deserialize, Serialize};

pub use craft_i18n as i18n;
pub use craft_ui::{fonts, icons, scenario, theme, ui_kit};

mod about;
mod app_bar;
pub mod commands;
mod geometry;
pub mod keymap;
mod messages;
mod palette;
mod panels;
mod prefs_window;
mod ribbon;
mod settings;
mod status;
mod titlebar;
mod toasts;

pub use commands::{CommandRegistry, CommandSpec, PanelSpec, TabSpec, Workspace};
pub use keymap::Keymap;
pub use messages::{Message, Messages};
pub use panels::{PanelState, Panels};
pub use settings::{STORAGE_UI, SavedShell, UiSettings};
pub use toasts::Toasts;

use craft_ui::i18n::tr;

/// Milliseconds since the Unix epoch, on desktop and in the browser.
pub fn now_ms() -> f64 {
    web_time::SystemTime::now().duration_since(web_time::UNIX_EPOCH).map(|d| d.as_millis() as f64).unwrap_or(0.0)
}

/// Command ids the shell answers itself, before the app sees them.
pub mod builtin {
    pub const SHELL_PREFERENCES: &str = "shell.preferences";
    pub const SHELL_THEME_TOGGLE: &str = "shell.theme.toggle";
    pub const SHELL_ABOUT: &str = "shell.about";
    pub const SHELL_MESSAGES: &str = "shell.messages";
    pub const SHELL_QUIT: &str = "shell.quit";
    pub const SHELL_SHORTCUTS: &str = "shell.shortcuts";
    pub const SHELL_PALETTE: &str = "shell.palette";
    pub const SHELL_HOME: &str = "shell.home";
    pub const SHELL_PROPERTIES: &str = "shell.properties";
}

/// The status line's content: a message and an optional progress fraction.
#[derive(Clone, Debug, PartialEq)]
pub struct Status {
    pub text: String,
    pub fraction: Option<f32>,
}

/// A preferences page the app adds after the built-in ones.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageSpec {
    pub id: &'static str,
    pub label: &'static str,
    pub icon: &'static str,
}

/// One application menu: a title and its entries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuSpec {
    pub title: &'static str,
    pub items: Vec<MenuEntry>,
}

/// A menu row: an app command, a separator, or one of the shell's own windows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuEntry {
    Command(&'static str),
    Separator,
    Preferences,
    About,
    Quit,
}

/// One open document in the application bar's tab strip.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentTab {
    pub title: String,
    pub dirty: bool,
}

/// One step in the application's history strip.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HistoryItem {
    pub id: String,
    pub label: String,
    pub icon: &'static str,
}

/// The history strip's steps and the marked step (the one the document is rolled back to).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HistoryView {
    pub items: Vec<HistoryItem>,
    pub marker: usize,
}

/// What the host application provides to the shell.
pub trait ShellApp {
    fn title(&self) -> &str;
    fn commands(&self) -> &CommandRegistry;
    /// Run one app command; `Ok(Some(message))` is a success notice.
    fn run_command(&mut self, id: &str) -> Result<Option<String>, String>;
    fn command_enabled(&self, _id: &str) -> bool {
        true
    }
    fn menus(&self) -> &[MenuSpec] {
        &[]
    }
    /// Panel headers; the shell uses its own labels when the app returns `None`.
    fn left_panel_title(&self) -> Option<&str> {
        None
    }
    /// A count or scope shown right-aligned in the browser card's header.
    fn left_panel_subtitle(&self) -> Option<String> {
        None
    }
    fn right_panel_title(&self) -> Option<&str> {
        None
    }
    fn left_panel(&mut self, ui: &mut egui::Ui);
    fn right_panel(&mut self, ui: &mut egui::Ui);
    fn center(&mut self, ui: &mut egui::Ui);
    /// The application bar's document tabs, left to right.
    fn documents(&self) -> Vec<DocumentTab> {
        Vec::new()
    }
    fn active_document(&self) -> usize {
        0
    }
    fn select_document(&mut self, index: usize) {
        let _ = index;
    }
    fn close_document(&mut self, index: usize) {
        let _ = index;
    }
    /// Whether the start page is showing instead of the workspace.
    fn home_open(&self) -> bool {
        false
    }
    fn show_home(&mut self) {}
    /// The start page's content; the shell hosts it full-bleed.
    fn home(&mut self, ui: &mut egui::Ui) {
        self.center(ui);
    }
    /// The history strip's steps; `None` hides the strip entirely.
    fn history(&self) -> Option<HistoryView> {
        None
    }
    fn history_selected(&mut self, id: &str) {
        let _ = id;
    }
    fn history_transport(&mut self, action: craft_ui::ui_kit::HistoryTransport) {
        let _ = action;
    }
    fn status(&self) -> Option<Status> {
        None
    }
    fn preference_pages(&self) -> &[PageSpec] {
        &[]
    }
    fn draw_preferences_page(&mut self, _ui: &mut egui::Ui, _page: &str) {}
    fn reserved_keys(&self) -> &'static [&'static str] {
        &[]
    }
}

/// The active ribbon tab.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RibbonState {
    pub active_tab: String,
}

/// The Preferences window's open state and selected page.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PrefsWindow {
    pub open: bool,
    pub page: usize,
}

/// The command palette's open state and its short history.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Palette {
    pub open: bool,
    pub query: String,
    pub selected: usize,
    pub recent: Vec<String>,
    pub pinned: Vec<String>,
}

/// A confirmation awaiting its answer. Answers run after the modal closes.
pub struct ConfirmRequest {
    pub tone: Tone,
    pub title: String,
    pub message: String,
    pub detail: Option<String>,
    pub ok: String,
    pub cancel: String,
    pub on_answer: Box<dyn FnOnce(bool)>,
}

/// The shell's state for one window.
pub struct Shell {
    workspace: Workspace,
    pub settings: UiSettings,
    last_theme: Option<(bool, ColorVision)>,
    last_scale: f32,
    pub ribbon: RibbonState,
    pub panels: Panels,
    pub palette: Palette,
    pub prefs: PrefsWindow,
    pub keymap: Keymap,
    pub messages: Messages,
    pub toasts: Toasts,
    pub about_open: bool,
    pub quit_requested: bool,
    pub confirm: VecDeque<ConfirmRequest>,
    /// The workspace canvas's pan and zoom.
    pub canvas: craft_ui::canvas::CanvasState,
    /// A transient status line message: text, when it was set (ms), and whether it is an error.
    pub status: Option<(String, f64, bool)>,
}

impl Shell {
    /// `stored` is the JSON [`Shell::save`] wrote, or `None` on a first run.
    pub fn new(workspace: Workspace, stored: Option<&str>) -> Self {
        let active_tab = workspace.tabs.first().map(|tab| tab.id.to_string()).unwrap_or_default();
        let mut shell = Self {
            workspace,
            settings: UiSettings::default(),
            last_theme: None,
            last_scale: f32::NAN,
            ribbon: RibbonState { active_tab },
            panels: Panels::default(),
            palette: Palette::default(),
            prefs: PrefsWindow::default(),
            keymap: Keymap::default(),
            messages: Messages::default(),
            toasts: Toasts::default(),
            about_open: false,
            quit_requested: false,
            confirm: VecDeque::new(),
            canvas: craft_ui::canvas::CanvasState::default(),
            status: None,
        };
        if let Some(text) = stored {
            shell.load(text);
        }
        shell
    }

    pub fn workspace(&self) -> &Workspace {
        &self.workspace
    }

    /// One frame: chrome, cards, the app's centre, overlays and shortcuts, in that order.
    pub fn ui<A: ShellApp>(&mut self, app: &mut A, ui: &mut egui::Ui) {
        let _language = i18n::scope(&self.settings.language);
        self.apply_appearance(ui.ctx());
        scenario::clear_handles();
        self.settings.layout.publish(ui.ctx());
        let accent = self.active_accent();
        AccentColors::publish(ui.ctx(), accent);
        let layout = self.settings.layout;
        app_bar::show(ui, self, app);
        // The content under the app bar: Home fills it, the workspace is the ribbon plus cards.
        let mut content = ui.available_rect_before_wrap();
        if app.home_open() {
            app.home(ui);
            ui.ctx().data_mut(|d| d.remove::<egui::Rect>(egui::Id::new(ribbon::SEARCH_RECT)));
        } else {
            ribbon::show(ui, self, app);
            content = ui.available_rect_before_wrap();
            if layout == LayoutMode::Floating {
                geometry::paint_top_corners(ui.painter(), content, craft_ui::theme::Tokens::get().app_bar);
            }
            panels::show_browser(ui, self, app);
            panels::show_properties(ui, self, app);
            panels::show_history(ui, self, app);
            // Bottom-anchored canvas controls stay clear of the history card.
            let lift = if layout == LayoutMode::Floating { panels::history_inset(self, app) } else { 0.0 };
            ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new(craft_ui::canvas::BOTTOM_INSET), lift));
            let mut canvas_action = None;
            let viewport = Container::new(ui.ctx(), Surface::Viewport);
            viewport.central().frame(viewport.edge_frame(egui::Color32::TRANSPARENT)).show(ui, |ui| {
                let rect = ui.max_rect();
                canvas_action = craft_ui::canvas::show(ui, rect, &mut self.canvas);
                app.center(ui);
            });
            match canvas_action {
                Some(craft_ui::canvas::CanvasAction::ZoomIn) => self.canvas.zoom = (self.canvas.zoom * 1.25).min(8.0),
                Some(craft_ui::canvas::CanvasAction::ZoomOut) => self.canvas.zoom = (self.canvas.zoom * 0.8).max(0.1),
                Some(craft_ui::canvas::CanvasAction::Fit | craft_ui::canvas::CanvasAction::Home) => {
                    self.canvas.pan = egui::Vec2::ZERO;
                    self.canvas.zoom = 1.0;
                }
                Some(craft_ui::canvas::CanvasAction::Settings) => self.run(app, builtin::SHELL_PREFERENCES),
                Some(craft_ui::canvas::CanvasAction::Messages) => self.run(app, builtin::SHELL_MESSAGES),
                None => {}
            }
        }
        if let Some(id) = palette::show(ui, self, app) {
            self.run(app, id);
        }
        prefs_window::show(ui, self, app);
        about::show(ui, self);
        self.messages.show(ui);
        self.show_confirm(ui.ctx());
        self.toasts.show(ui.ctx(), now_ms());
        status::show(ui, self, app, content);
        self.shortcuts(ui, app);
        if self.quit_requested {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if self.settings.custom_titlebar && !cfg!(target_arch = "wasm32") {
            titlebar::resize_zones(ui);
        }
    }

    /// The persisted shell state as JSON.
    pub fn save(&self) -> String {
        let saved = SavedShell {
            ui: self.settings.clone(),
            panels: self.panels,
            active_tab: self.ribbon.active_tab.clone(),
            palette_recent: self.palette.recent.clone(),
            palette_pinned: self.palette.pinned.clone(),
        };
        serde_json::to_string(&saved).unwrap_or_default()
    }

    /// Read [`Shell::save`]'s output. Never fails: unknown keys and out-of-range values are
    /// ignored, and a tab that no longer exists falls back to the first one.
    pub fn load(&mut self, text: &str) {
        let Ok(mut saved) = serde_json::from_str::<SavedShell>(text) else { return };
        saved.sanitize();
        if !saved.active_tab.is_empty() && self.workspace.tabs.iter().any(|tab| tab.id == saved.active_tab) {
            self.ribbon.active_tab = saved.active_tab;
        }
        self.panels = saved.panels;
        self.palette.recent = saved.palette_recent;
        self.palette.pinned = saved.palette_pinned;
        self.keymap.custom = saved.ui.keymap.clone();
        self.settings = saved.ui;
        self.last_theme = None;
        self.last_scale = f32::NAN;
    }

    /// Put a message on the status line and in the Messages window.
    pub fn notify(&mut self, text: impl Into<String>, error: bool) {
        let text = text.into();
        let now = now_ms();
        self.status = Some((text.clone(), now, error));
        self.messages.push(&text, error, now);
        if error {
            self.toasts.error(text, now);
        } else {
            self.toasts.push(text, false, now);
        }
    }

    /// Queue a confirmation; `on_answer` runs once with `true` for OK and `false` for cancel.
    pub fn confirm(&mut self, request: ConfirmRequest) {
        self.confirm.push_back(request);
    }

    /// Run a command: the shell's own first, then the app's.
    pub fn run<A: ShellApp>(&mut self, app: &mut A, id: &str) {
        match id {
            builtin::SHELL_PREFERENCES => {
                self.prefs.open = true;
                self.prefs.page = crate::prefs_window::GENERAL_PAGE;
            }
            builtin::SHELL_THEME_TOGGLE => {
                self.settings.theme = if self.settings.dark() { "light".into() } else { "dark".into() };
            }
            builtin::SHELL_ABOUT => self.about_open = true,
            builtin::SHELL_MESSAGES => self.messages.open = true,
            builtin::SHELL_QUIT => self.quit_requested = true,
            builtin::SHELL_SHORTCUTS => {
                self.prefs.open = true;
                self.prefs.page = crate::prefs_window::SHORTCUTS_PAGE;
            }
            builtin::SHELL_PALETTE => self.palette.open = true,
            builtin::SHELL_HOME => app.show_home(),
            builtin::SHELL_PROPERTIES => {
                self.panels.right.open = true;
                self.panels.right.collapsed = false;
            }
            _ => match app.run_command(id) {
                Ok(Some(message)) => self.notify(message, false),
                Ok(None) => {}
                Err(error) => {
                    let label = app.commands().get(id).map(|spec| i18n::tr(spec.label)).unwrap_or(id).to_string();
                    let text = format!("{label}: {error}");
                    let now = now_ms();
                    self.status = Some((format!("{}: {error}", tr("error.command_failed")), now, true));
                    self.messages.push(&text, true, now);
                    self.toasts.error(text, now);
                }
            },
        }
    }

    pub fn status_text(&self) -> Option<&str> {
        self.status.as_ref().map(|(text, _, _)| text.as_str())
    }

    fn active_accent(&self) -> craft_theme::Accent {
        self.workspace
            .tabs
            .iter()
            .find(|tab| tab.id == self.ribbon.active_tab)
            .or_else(|| self.workspace.tabs.first())
            .map(|tab| tab.accent)
            .unwrap_or(craft_theme::Accent::Blue)
    }

    fn apply_appearance(&mut self, ctx: &egui::Context) {
        let appearance = (self.settings.dark(), self.settings.color_vision);
        if self.last_theme != Some(appearance) {
            theme::set_dark(appearance.0);
            theme::set_vision(appearance.1);
            theme::apply(ctx);
            self.last_theme = Some(appearance);
        }
        if !(self.last_scale.is_finite() && (self.last_scale - self.settings.scale).abs() < f32::EPSILON) {
            ctx.set_zoom_factor(self.settings.scale);
            self.last_scale = self.settings.scale;
        }
    }

    fn show_confirm(&mut self, ctx: &egui::Context) {
        let Some(request) = self.confirm.front() else { return };
        let detail = request.detail.as_deref().unwrap_or("");
        let (closed, action) = Confirm::new(egui::Id::new("craft_confirm"), "warning", &request.title)
            .tone(request.tone)
            .actions(&request.ok, &request.cancel, None)
            .detail(detail)
            .show(ctx, true, |ui| {
                ui.label(&request.message);
            });
        let answer = if action == Action::Ok {
            Some(true)
        } else if closed || action == Action::Cancel {
            Some(false)
        } else {
            None
        };
        if let Some(ok) = answer
            && let Some(request) = self.confirm.pop_front()
        {
            (request.on_answer)(ok);
        }
    }

    fn shortcuts<A: ShellApp>(&mut self, ui: &mut Ui, app: &mut A) {
        let ctx = ui.ctx().clone();
        // System keys first.
        let toggle = ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::K) || i.consume_key(egui::Modifiers::COMMAND, egui::Key::K));
        let toggle_p = ctx.input_mut(|i| {
            i.consume_key(egui::Modifiers::CTRL | egui::Modifiers::SHIFT, egui::Key::P)
                || i.consume_key(egui::Modifiers::COMMAND | egui::Modifiers::SHIFT, egui::Key::P)
        });
        let escape = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
        if escape {
            self.palette.open = false;
            crate::prefs_window::cancel(self, &ctx);
            self.about_open = false;
            self.messages.open = false;
            ctx.data_mut(|d| {
                d.remove::<(usize, egui::Pos2)>(egui::Id::new("craft_app_menu"));
                d.remove::<egui::Id>(egui::Id::new("craft_panel_open"));
            });
        }
        if toggle || toggle_p {
            self.palette.open = !self.palette.open;
            self.palette.query.clear();
            self.palette.selected = 0;
            ctx.memory_mut(|m| m.request_focus(egui::Id::new(palette::FIELD_ID)));
        }
        let overlay = self.palette.open || self.prefs.open || self.about_open || self.messages.open || !self.confirm.is_empty();
        if overlay {
            return;
        }
        let focused = ctx.memory(|m| m.focused().is_some());
        let pressed = ctx.input(|i| {
            i.events
                .iter()
                .filter_map(|event| match event {
                    egui::Event::Key { key, pressed: true, modifiers, repeat: false, .. } => Some((*key, *modifiers)),
                    _ => None,
                })
                .collect::<Vec<_>>()
        });
        for (key, modifiers) in pressed {
            if (modifiers == egui::Modifiers::NONE || modifiers == egui::Modifiers::SHIFT) && key == egui::Key::S && !focused {
                self.palette.open = true;
                self.palette.query.clear();
                ctx.memory_mut(|m| m.request_focus(egui::Id::new(palette::FIELD_ID)));
                return;
            }
            // Unmodified keys belong to the focused text field, not to commands.
            if focused && !(modifiers.command || modifiers.ctrl || modifiers.alt) {
                continue;
            }
            let name = keymap::key_name(key, modifiers);
            if let Some(id) = keymap::press(app.commands(), &self.keymap.custom, app.reserved_keys(), &name)
                && app.command_enabled(id)
            {
                self.run(app, id);
                return;
            }
        }
    }
}
