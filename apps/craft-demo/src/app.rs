//! The reference application: a SolidCraft-shaped shell with a FileView start page, the UI-kit
//! workspace and generic panels. The same code runs on desktop, in the browser and inside the
//! hot-reload host.

use craft_shell::commands::{CommandRegistry, CommandSpec, PanelSpec, TabSpec, Workspace};
use craft_shell::{DocumentTab, HistoryItem, HistoryView, MenuEntry, MenuSpec, Shell, ShellApp, Status};
use craft_theme::Accent;
use craft_ui::i18n::{self, tr};
use craft_ui::theme::Tokens;
use craft_ui::ui_kit::{self, Button, Container, FileView, RecentRow, SampleCard, Surface, Text};
use egui::{Align2, FontId, Rect, RichText, Sense, Ui, UiBuilder, pos2, vec2};

use crate::gallery;

/// The commands the demo registers: the app's own and the shell's (so menus and the palette have
/// their metadata in one place).
pub static COMMANDS: &[CommandSpec] = &[
    CommandSpec { id: "app.start", label: "cmd.app.start", icon: "home", shortcut: None },
    CommandSpec { id: "app.gallery", label: "cmd.app.gallery", icon: "panel_bottom", shortcut: None },
    CommandSpec { id: "shell.save", label: "cmd.shell.save", icon: "save", shortcut: Some("Ctrl+S") },
    CommandSpec { id: "shell.undo", label: "cmd.shell.undo", icon: "undo", shortcut: Some("Ctrl+Z") },
    CommandSpec { id: "shell.redo", label: "cmd.shell.redo", icon: "redo", shortcut: Some("Ctrl+Y") },
    CommandSpec { id: "shell.theme.toggle", label: "cmd.shell.theme_toggle", icon: "palette", shortcut: None },
    CommandSpec { id: "shell.preferences", label: "cmd.shell.preferences", icon: "settings", shortcut: Some("Ctrl+Comma") },
    CommandSpec { id: "shell.shortcuts", label: "cmd.shell.shortcuts", icon: "keyboard", shortcut: None },
    CommandSpec { id: "shell.palette", label: "cmd.shell.palette", icon: "command", shortcut: None },
    CommandSpec { id: "shell.about", label: "cmd.shell.about", icon: "info", shortcut: None },
    CommandSpec { id: "shell.messages", label: "cmd.shell.messages", icon: "info", shortcut: None },
    CommandSpec { id: "shell.quit", label: "cmd.shell.quit", icon: "close", shortcut: None },
];

/// The tabs and panels the ribbon draws. Only shell/UI commands: no application domain.
pub fn workspace() -> Workspace {
    Workspace {
        tabs: vec![
            TabSpec {
                id: "home",
                label: "tab.home",
                accent: Accent::Blue,
                panels: vec![
                    PanelSpec { id: "configure", label: "panel.configure", commands: &["shell.preferences", "shell.shortcuts", "shell.palette"] },
                    PanelSpec { id: "appearance", label: "panel.appearance", commands: &["shell.theme.toggle", "app.gallery"] },
                ],
            },
            TabSpec {
                id: "help",
                label: "tab.help",
                accent: Accent::Green,
                panels: vec![PanelSpec { id: "information", label: "panel.information", commands: &["app.start", "shell.about", "shell.messages"] }],
            },
        ],
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum View {
    #[default]
    Start,
    Gallery,
}

impl View {
    /// The start page is the first document tab, the workspace the second.
    fn index(self) -> usize {
        match self {
            View::Start => 0,
            View::Gallery => 1,
        }
    }

    fn from_index(index: usize) -> Self {
        if index == 0 { View::Start } else { View::Gallery }
    }
}

pub struct DemoApp {
    pub registry: CommandRegistry,
    pub workspace: Workspace,
    pub menus: Vec<MenuSpec>,
    pub view: View,
    /// Views the application bar's Undo and Redo walk.
    pub undo: Vec<View>,
    pub redo: Vec<View>,
    /// The history strip's marker: 0 rolls back to Start, 1 sits at the workspace.
    pub history_marker: usize,
    /// The browser's tree root, folded or open.
    pub tree_open: bool,
    /// A mirror of the shell's live settings, for the Properties card.
    pub settings: craft_shell::UiSettings,
    /// The gallery's floating-window specimen.
    pub gallery_open: bool,
    /// A command clicked in the app's own content; the host runs it after painting.
    pub pending: Option<&'static str>,
    /// Queued for the host shell after this frame.
    pub pending_theme: Option<bool>,
    pub pending_toast: Option<String>,
    pub pending_confirm: Option<String>,
    /// The last message from a panel-owned button, shown on the status chip.
    pub notice: Option<String>,
}

impl DemoApp {
    pub fn new() -> Result<Self, String> {
        let registry = CommandRegistry::new(COMMANDS.to_vec())?;
        let workspace = workspace();
        workspace.validate(&registry)?;
        let menus = vec![MenuSpec {
            title: "menu.file",
            items: vec![
                MenuEntry::Command("app.gallery"),
                MenuEntry::Separator,
                MenuEntry::Command("shell.theme.toggle"),
                MenuEntry::Command("shell.palette"),
                MenuEntry::Command("shell.preferences"),
                MenuEntry::Command("shell.shortcuts"),
                MenuEntry::Separator,
                MenuEntry::Command("shell.about"),
                MenuEntry::Command("shell.messages"),
                MenuEntry::Separator,
                MenuEntry::Quit,
            ],
        }];
        Ok(Self {
            registry,
            workspace,
            menus,
            view: View::Start,
            undo: Vec::new(),
            redo: Vec::new(),
            history_marker: 1,
            tree_open: true,
            settings: craft_shell::UiSettings::default(),
            gallery_open: false,
            pending: None,
            pending_theme: None,
            pending_toast: None,
            pending_confirm: None,
            notice: None,
        })
    }

    /// Switch views, keeping the application bar's undo/redo stacks.
    fn set_view(&mut self, view: View) {
        if self.view == view {
            return;
        }
        self.undo.push(self.view);
        self.redo.clear();
        self.view = view;
        self.history_marker = view.index();
    }

    fn undo_view(&mut self) {
        if let Some(previous) = self.undo.pop() {
            self.redo.push(self.view);
            self.view = previous;
            self.history_marker = previous.index();
        }
    }

    fn redo_view(&mut self) {
        if let Some(next) = self.redo.pop() {
            self.undo.push(self.view);
            self.view = next;
            self.history_marker = next.index();
        }
    }

    /// Run one app command. Shell commands are handled by `Shell::run` before this is called.
    pub fn execute(&mut self, id: &str) -> Result<Option<String>, String> {
        match id {
            "app.start" => {
                self.set_view(View::Start);
                Ok(None)
            }
            "app.gallery" => {
                self.set_view(View::Gallery);
                Ok(None)
            }
            "shell.save" => Ok(Some(tr("status.saved").to_string())),
            "shell.undo" => {
                self.undo_view();
                Ok(None)
            }
            "shell.redo" => {
                self.redo_view();
                Ok(None)
            }
            other => Err(format!("unknown command `{other}`")),
        }
    }

    /// The start page's content: the FileView rail and sections.
    fn home_page(&mut self, ui: &mut Ui) {
        let t = Tokens::get();
        let mut picked: Option<&'static str> = None;
        FileView::page(ui, |ui| {
            let full = ui.max_rect();
            craft_ui::scenario::publish_rect("home:page", full);
            let rail = Rect::from_min_size(full.min, vec2(t.file_nav_width, full.height()));
            craft_ui::scenario::publish_rect("home:rail", rail);
            FileView::rail(ui, rail, |ui| {
                let back = FileView::back_to(ui, "home:back", tr("home.back"), self.title(), "Esc");
                craft_ui::scenario::publish_target("home:back", back.rect);
                if back.clicked() {
                    picked = Some("app.gallery");
                }
                let mut nav = |ui: &mut Ui, id: &str, label: &str, shortcut: &str, selected: bool, command: &'static str| {
                    let response = FileView::nav_row(ui, id, label, shortcut, selected, true);
                    craft_ui::scenario::publish_target(id, response.rect);
                    if response.clicked() {
                        picked = Some(command);
                    }
                };
                nav(ui, "home:nav:start", tr("cmd.app.start"), "", self.view == View::Start, "app.start");
                nav(ui, "home:nav:gallery", tr("cmd.app.gallery"), "", self.view == View::Gallery, "app.gallery");
                nav(ui, "home:nav:properties", tr("panel.properties"), "", false, craft_shell::builtin::SHELL_PROPERTIES);
                FileView::separator(ui);
                nav(ui, "home:nav:preferences", tr("cmd.shell.preferences"), "Ctrl+,", false, "shell.preferences");
                nav(ui, "home:nav:shortcuts", tr("cmd.shell.shortcuts"), "", false, "shell.shortcuts");
                nav(ui, "home:nav:about", tr("cmd.shell.about"), "", false, "shell.about");
                nav(ui, "home:nav:messages", tr("cmd.shell.messages"), "", false, "shell.messages");
            });
            let content = Rect::from_min_max(pos2(rail.right(), full.top()), full.max);
            ui.scope_builder(UiBuilder::new().max_rect(content), |ui| {
                FileView::content(ui, false, |ui| {
                    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                        FileView::title(ui, tr("start.title"));
                        ui.add_space(t.space_6);
                        FileView::subtitle(ui, tr("start.note"));
                        ui.add_space(t.space_4);
                        FileView::detail(ui, tr("home.start_hint"));
                        ui.add_space(t.file_section_gap);
                        let mut clicked: Option<&'static str> = None;
                        FileView::section(ui, tr("home.new"), t.space_14, |ui| {
                            FileView::grid(ui, "home_actions", FileView::tile_min_width(), 3, |ui, index, width| {
                                let (id, icon, label, subtitle, primary) = match index {
                                    0 => ("app.gallery", "box", tr("cmd.app.gallery"), tr("start.gallery_desc"), true),
                                    1 => ("shell.preferences", "settings", tr("start.preferences"), tr("start.preferences_desc"), false),
                                    _ => ("shell.theme.toggle", "palette", tr("start.theme"), tr("start.theme_desc"), false),
                                };
                                if FileView::action_tile(ui, &format!("home:action:{id}"), width, icon, label, subtitle, primary).clicked() {
                                    clicked = Some(id);
                                }
                            });
                        });
                        if let Some(id) = clicked {
                            picked = Some(id);
                        }
                        ui.add_space(t.file_section_gap);
                        FileView::section(ui, tr("home.recent"), t.space_14, |ui| {
                            FileView::recent_list(ui, |ui| {
                                if FileView::recent_row(
                                    ui,
                                    "home:recent:gallery",
                                    RecentRow {
                                        title: tr("cmd.app.gallery"),
                                        path: "app.gallery",
                                        when: "",
                                        workspace: self.title(),
                                        thumbnail: None,
                                    },
                                )
                                .clicked()
                                {
                                    picked = Some("app.gallery");
                                }
                            });
                            FileView::detail(ui, tr("home.recent_empty"));
                        });
                        ui.add_space(t.file_section_gap);
                        FileView::section(ui, tr("home.samples"), t.space_14, |ui| {
                            let samples = [
                                SampleCard { title: tr("start.title"), description: tr("start.note"), icon: "home", thumbnail: None },
                                SampleCard { title: tr("cmd.app.gallery"), description: tr("start.gallery_desc"), icon: "box", thumbnail: None },
                                SampleCard {
                                    title: tr("start.preferences"),
                                    description: tr("start.preferences_desc"),
                                    icon: "settings",
                                    thumbnail: None,
                                },
                            ];
                            let ids = ["app.start", "app.gallery", "shell.preferences"];
                            for (response, id) in FileView::sample_grid(ui, "home_samples", &samples).into_iter().zip(ids) {
                                if response.clicked() {
                                    picked = Some(id);
                                }
                            }
                        });
                        ui.add_space(t.file_section_gap);
                    });
                });
            });
        });
        if let Some(id) = picked {
            self.pending = Some(id);
        }
    }

    /// The workspace: the UI kit on a card over the canvas grid.
    fn gallery_page(&mut self, ui: &mut Ui) {
        let t = Tokens::get();
        let available = ui.max_rect();
        // Clear the 40 pt history card and its gaps.
        let width = (available.width() - 48.0).min(760.0);
        let height = available.height() - 24.0 - 60.0;
        if !(width > 0.0 && height > 0.0) {
            return;
        }
        let rect = Rect::from_min_size(pos2(available.center().x - width / 2.0, available.top() + 24.0), vec2(width, height));
        craft_ui::scenario::publish_rect("gallery:card", rect);
        let container = Container::new(ui.ctx(), Surface::Feature);
        let frame = container.frame(ui.ctx(), t.dialog_body, craft_ui::frame::Side::Right).inner_margin(t.space_12 as i8);
        ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
            frame.show(ui, |ui| {
                ui.set_width(rect.width() - frame.total_margin().sum().x);
                ui.set_max_height(rect.height() - frame.total_margin().sum().y);
                ui.label(RichText::new(tr("cmd.app.gallery")).font(craft_ui::fonts::semibold(t.font_heading)).color(t.form_ink));
                ui_kit::text(ui, tr("start.gallery_desc"), Text::Note("gallery_card_note"));
                ui.add_space(t.space_8);
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| gallery::show(self, ui));
            });
        });
    }

    fn properties(&mut self, ui: &mut Ui) {
        let t = Tokens::get();
        ui.label(RichText::new(tr("panel.properties")).font(craft_ui::fonts::semibold(t.font_heading)).color(t.form_ink));
        ui.add_space(t.space_6);
        // The shell's live appearance, read-only here; Preferences edits it.
        let settings = self.settings.clone();
        for (label, value) in [
            (tr("prefs.theme"), tr(if settings.dark() { "prefs.theme_dark" } else { "prefs.theme_light" }).to_string()),
            (
                tr("prefs.layout"),
                tr(if settings.layout == craft_ui::layout::LayoutMode::Floating { "prefs.layout_floating" } else { "prefs.layout_docked" })
                    .to_string(),
            ),
            (tr("prefs.scale_text"), format!("{:.0}%", settings.scale * 100.0)),
        ] {
            ui.horizontal(|ui| {
                ui.label(RichText::new(label).size(t.font_body).color(t.text_dim));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(value).size(t.font_body).color(t.form_ink));
                });
            });
        }
        ui.add_space(t.space_8);
        ui_kit::text(ui, tr("panel.properties_note"), Text::Note("panel_properties_note"));
        ui.add_space(t.space_8);
        if ui_kit::button(ui, tr("cmd.shell.preferences"), Button::Normal).clicked() {
            self.pending = Some("shell.preferences");
        }
    }

    /// The browser's rows: the document tree.
    fn browser(&mut self, ui: &mut Ui) {
        let mut picked: Option<&'static str> = None;
        let root = tree_row(ui, "tree_root", "chevron", tr("demo.title"), 0, false, Some(true));
        if root.clicked() {
            self.tree_open = !self.tree_open;
        }
        if self.tree_open {
            if tree_row(ui, "tree_start", "home", tr("cmd.app.start"), 1, self.view == View::Start, None).clicked() {
                picked = Some("app.start");
            }
            if tree_row(ui, "tree_gallery", "panel_bottom", tr("cmd.app.gallery"), 1, self.view == View::Gallery, None).clicked() {
                picked = Some("app.gallery");
            }
            if tree_row(ui, "tree_prefs", "settings", tr("cmd.shell.preferences"), 1, false, None).clicked() {
                picked = Some("shell.preferences");
            }
        }
        if let Some(id) = picked {
            self.pending = Some(id);
        }
    }
}

/// One browser tree row: indent per level, icon, label, optional chevron. Returns its response.
fn tree_row(ui: &mut Ui, name: &str, icon: &str, label: &str, level: usize, selected: bool, chevron: Option<bool>) -> egui::Response {
    let t = Tokens::get();
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), craft_ui::ui_kit::TREE_ROW_HEIGHT), Sense::hover());
    let response = ui.interact(rect, ui.make_persistent_id(name), Sense::click());
    if selected {
        ui.painter().rect_filled(rect, t.radius_sm, t.selected_bg);
    } else if response.hovered() {
        ui.painter().rect_filled(rect, t.radius_sm, t.hover);
    }
    let mut x = rect.left() + t.space_4 + level as f32 * 18.0;
    if let Some(open) = chevron {
        let at = pos2(x + 6.0, rect.center().y);
        ui.painter().add(egui::Shape::line(
            if open {
                vec![pos2(at.x - 3.5, at.y - 1.5), pos2(at.x, at.y + 2.0), pos2(at.x + 3.5, at.y - 1.5)]
            } else {
                vec![pos2(at.x - 1.5, at.y - 3.5), pos2(at.x + 2.0, at.y), pos2(at.x - 1.5, at.y + 3.5)]
            },
            egui::Stroke::new(1.2, t.text_dim),
        ));
    }
    x += 18.0;
    let icon_rect = Rect::from_center_size(pos2(x + 7.0, rect.center().y), vec2(t.space_14, t.space_14));
    craft_ui::icons::paint_icon(ui.painter(), icon_rect, icon, 1.0);
    let ink = if selected { t.selected_ink } else { t.form_ink };
    ui.painter().text(pos2(icon_rect.right() + t.space_6, rect.center().y), Align2::LEFT_CENTER, label, FontId::proportional(t.font_body), ink);
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, label));
    craft_ui::scenario::publish_target(&format!("browser:{name}"), rect);
    response
}

impl ShellApp for DemoApp {
    fn title(&self) -> &str {
        tr("demo.title")
    }

    fn commands(&self) -> &CommandRegistry {
        &self.registry
    }

    fn run_command(&mut self, id: &str) -> Result<Option<String>, String> {
        self.execute(id)
    }

    fn command_enabled(&self, id: &str) -> bool {
        match id {
            "shell.save" => self.view == View::Gallery,
            "shell.undo" => !self.undo.is_empty(),
            "shell.redo" => !self.redo.is_empty(),
            _ => true,
        }
    }

    fn menus(&self) -> &[MenuSpec] {
        &self.menus
    }

    fn left_panel_title(&self) -> Option<&str> {
        Some(tr("ui.browser_title"))
    }

    fn left_panel_subtitle(&self) -> Option<String> {
        Some(i18n::format("ui.browser_count_other", &[("count", "2")]))
    }

    fn right_panel_title(&self) -> Option<&str> {
        Some(tr("panel.properties"))
    }

    fn left_panel(&mut self, ui: &mut Ui) {
        self.browser(ui);
    }

    fn right_panel(&mut self, ui: &mut Ui) {
        self.properties(ui);
    }

    fn center(&mut self, ui: &mut Ui) {
        self.gallery_page(ui);
    }

    fn documents(&self) -> Vec<DocumentTab> {
        vec![
            DocumentTab { title: tr("start.title").to_string(), dirty: false },
            DocumentTab { title: tr("cmd.app.gallery").to_string(), dirty: false },
        ]
    }

    fn active_document(&self) -> usize {
        self.view.index()
    }

    fn select_document(&mut self, index: usize) {
        self.set_view(View::from_index(index));
    }

    fn close_document(&mut self, index: usize) {
        // Two views: closing one shows the other.
        self.set_view(View::from_index(usize::from(index == 0)));
    }

    fn home_open(&self) -> bool {
        self.view == View::Start
    }

    fn show_home(&mut self) {
        self.set_view(View::Start);
    }

    fn home(&mut self, ui: &mut Ui) {
        self.home_page(ui);
    }

    fn history(&self) -> Option<HistoryView> {
        Some(HistoryView {
            items: vec![
                HistoryItem { id: "start".into(), label: tr("cmd.app.start").into(), icon: "home" },
                HistoryItem { id: "gallery".into(), label: tr("cmd.app.gallery").into(), icon: "panel_bottom" },
            ],
            marker: self.history_marker,
        })
    }

    fn history_selected(&mut self, id: &str) {
        match id {
            "start" => {
                self.history_marker = 0;
                self.set_view(View::Start);
            }
            _ => {
                self.history_marker = 1;
                self.set_view(View::Gallery);
            }
        }
    }

    fn history_transport(&mut self, action: craft_ui::ui_kit::HistoryTransport) {
        use craft_ui::ui_kit::HistoryTransport;
        match action {
            HistoryTransport::First | HistoryTransport::Back => {
                self.history_marker = 0;
                self.set_view(View::Start);
            }
            HistoryTransport::Forward | HistoryTransport::Last => {
                self.history_marker = 1;
                self.set_view(View::Gallery);
            }
        }
    }

    fn status(&self) -> Option<Status> {
        self.notice.as_ref().map(|text| Status { text: text.clone(), fraction: None })
    }
}

/// One frame of the demo, without any windowing. The desktop and web bootstraps both call it.
pub struct CraftApp {
    pub shell: Shell,
    pub demo: DemoApp,
}

impl CraftApp {
    /// `stored` is the shell JSON; a first run uses the shell's own defaults (browser and history
    /// showing, the floating layout).
    pub fn new(_cc: &eframe::CreationContext<'_>, stored: Option<&str>) -> Result<Self, String> {
        let demo = DemoApp::new()?;
        let shell = Shell::new(demo.workspace.clone(), stored);
        Ok(Self { shell, demo })
    }
}

impl CraftApp {
    /// One frame of the whole shell and demo. Kept separate from `eframe::App` so the hot-reload
    /// host can call it through a reloadable library function.
    pub fn draw(&mut self, ui: &mut Ui) {
        if let Some(id) = self.demo.pending.take() {
            self.shell.run(&mut self.demo, id);
        }
        if let Some(dark) = self.demo.pending_theme.take() {
            self.shell.settings.theme = if dark { "dark".into() } else { "light".into() };
        }
        if let Some(text) = self.demo.pending_toast.take() {
            self.shell.notify(text, false);
        }
        if let Some(message) = self.demo.pending_confirm.take() {
            let title = tr("gallery.confirm_title").to_string();
            self.shell.confirm(craft_shell::ConfirmRequest {
                tone: craft_ui::ui_kit::Tone::Warning,
                title,
                message,
                detail: None,
                ok: tr("ui.ok").to_string(),
                cancel: tr("ui.cancel").to_string(),
                on_answer: Box::new(|ok| log::info!("gallery confirm answered {ok}")),
            });
        }
        // The demo's Properties card mirrors the live appearance; the shell owns the settings.
        self.demo.settings = self.shell.settings.clone();
        self.shell.ui(&mut self.demo, ui);
    }
}

impl eframe::App for CraftApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.draw(ui);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string(craft_shell::STORAGE_UI, self.shell.save());
    }
}
