//! Headless scenarios: the real shell and demo drawn frame by frame, driven like a person.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use craft_demo::{DemoApp, View};
use craft_shell::Shell;
use craft_test::Harness;
use egui::{Key, Modifiers};

fn harness() -> Harness<DemoApp> {
    Harness::new(|| {
        let demo = DemoApp::new().expect("the demo's registry and workspace are valid");
        let shell = Shell::new(demo.workspace.clone(), None);
        (demo, shell)
    })
}

#[test]
fn palette_runs_a_command() {
    let mut h = harness();
    assert_eq!(h.app.view, View::Start, "the app opens on the start page");
    h.key(Key::K, Modifiers::CTRL);
    h.frames(2);
    assert!(h.shell.palette.open, "Ctrl+K opens the palette");
    h.text("ui kit");
    h.frames(2);
    h.key(Key::Enter, Modifiers::NONE);
    h.frames(2);
    assert_eq!(h.app.view, View::Gallery, "Enter runs the first match");
    assert!(!h.shell.palette.open, "running a command closes the palette");
}

#[test]
fn theme_toggle_repaints_from_tokens() {
    let mut h = harness();
    let before = craft_ui::theme::Tokens::get().panel;
    h.shell.run(&mut h.app, craft_shell::builtin::SHELL_THEME_TOGGLE);
    h.frames(3);
    let after = craft_ui::theme::Tokens::get().panel;
    assert_ne!(before, after, "the panel fill follows the theme tokens");
    h.shell.run(&mut h.app, craft_shell::builtin::SHELL_THEME_TOGGLE);
    h.frames(3);
    assert_eq!(craft_ui::theme::Tokens::get().panel, before, "and back");
}

#[test]
fn preferences_open_on_general_and_interface() {
    let mut h = harness();
    h.shell.run(&mut h.app, craft_shell::builtin::SHELL_PREFERENCES);
    h.frames(8);
    h.settle();
    assert!(h.widget("English").is_some(), "General holds the language");
    assert!(h.click_widget("Interface"), "the interface page is in the rail");
    h.frames(8);
    h.settle();
    assert!(h.widget("Docked").is_some(), "Interface holds the layout");
    assert!(h.widget("75%").is_some(), "Interface holds the scale presets");
}

#[test]
fn language_switch_translates_chrome() {
    let mut h = harness();
    assert!(h.widget("File").is_some(), "the File menu starts in English");
    h.shell.run(&mut h.app, craft_shell::builtin::SHELL_PREFERENCES);
    h.frames(8);
    h.settle();
    assert!(h.click_widget("English"), "the language dropdown shows the current language");
    h.settle();
    assert!(h.click_widget("Español (España)"), "the Spanish catalogue is listed");
    h.settle();
    assert_eq!(h.shell.settings.language, "es");
    assert!(h.widget("Archivo").is_some(), "the File menu is translated");
    assert!(h.widget("File").is_none(), "the English label is gone");
}

#[test]
fn panels_collapse_and_persist() {
    let mut h = harness();
    h.shell.panels.left.open = false;
    h.frames(2);
    let saved = h.shell.save();
    let mut reloaded = Shell::new(h.app.workspace.clone(), Some(&saved));
    assert!(!reloaded.panels.left.open, "a closed panel stays closed");
    // A stored tab that no longer exists falls back to the first one.
    let bogus = saved.replace("\"home\"", "\"gone\"");
    reloaded = Shell::new(h.app.workspace.clone(), Some(&bogus));
    assert_eq!(reloaded.workspace().tabs.first().map(|tab| tab.id), Some("home"));
}

#[test]
fn keymap_rebind_moves_the_shortcut() {
    let mut h = harness();
    let registry = h.app.registry.clone();
    let mut custom = h.shell.keymap.custom.clone();
    craft_shell::keymap::bind(&registry, &mut custom, "shell.preferences", "Ctrl+G", true, &[]).expect("Ctrl+G is free");
    h.shell.keymap.custom = custom;
    h.key(Key::G, Modifiers::CTRL);
    h.frames(2);
    assert!(h.shell.prefs.open, "the new key opens preferences");
    h.shell.prefs.open = false;
    h.frames(2);
    h.key(Key::Comma, Modifiers::CTRL);
    h.frames(2);
    assert!(!h.shell.prefs.open, "the old key no longer opens preferences");
}

#[test]
fn gallery_renders_both_themes() {
    let mut h = harness();
    h.shell.run(&mut h.app, "app.gallery");
    h.frames(3);
    let dark = craft_ui::scenario::handles("gallery:").len();
    assert!(dark >= 15, "the gallery publishes every specimen: {dark}");
    h.shell.run(&mut h.app, craft_shell::builtin::SHELL_THEME_TOGGLE);
    h.frames(3);
    let light = craft_ui::scenario::handles("gallery:").len();
    assert!(light >= 15, "the gallery draws in the light theme too: {light}");
}

/// The floating cards, the app bar and the start page land where the SolidCraft shell puts
/// them, at the harness's 1600 × 1000 window.
#[test]
fn shell_geometry_matches_the_floating_reference() {
    let mut h = harness();
    h.settle();
    let appbar = h.rect("appbar").expect("the application bar is drawn");
    assert!((appbar.height() - 34.0).abs() < 0.5, "app bar is 34 pt: {appbar:?}");
    let home = h.rect("home:page").expect("the app opens on Home");
    assert!(h.rect("ribbon:card").is_none(), "Home shows no ribbon");
    let rail = h.rect("home:rail").expect("the start page has its rail");
    assert!((rail.width() - 248.0).abs() < 0.5, "the rail is 248 pt: {rail:?}");
    assert!((home.left() - rail.left()).abs() < 0.5, "the rail sits at the page's left edge");
}

#[test]
fn workspace_cards_are_flush_with_the_shell_gaps() {
    let mut h = harness();
    h.settle();
    let tab = h.handle("tab:design:1").expect("the UI kit document tab");
    h.click([f64::from(tab.x), f64::from(tab.y)]);
    h.settle();
    assert!(h.rect("home:page").is_none(), "the workspace replaces Home");
    let workspace = h.rect("ribbon:workspace").expect("the workspace switcher");
    assert!((workspace.left() - 7.0).abs() < 0.5, "{workspace:?}");
    assert!((workspace.top() - 41.0).abs() < 0.5, "{workspace:?}");
    assert!((workspace.width() - 88.0).abs() < 0.5, "{workspace:?}");
    assert!((workspace.height() - 117.0).abs() < 0.5, "{workspace:?}");
    let ribbon = h.rect("ribbon:card").expect("the ribbon card");
    assert!((ribbon.left() - 102.0).abs() < 0.5, "{ribbon:?}");
    assert!((ribbon.right() - 1593.0).abs() < 0.5, "{ribbon:?}");
    assert!((ribbon.height() - 117.0).abs() < 0.5, "{ribbon:?}");
    let browser = h.rect("browser:card").expect("the browser card");
    assert!((browser.left() - 7.0).abs() < 0.5, "{browser:?}");
    assert!((browser.width() - 250.0).abs() < 0.5, "{browser:?}");
    let history = h.rect("history:card").expect("the history card");
    assert!((history.height() - 40.0).abs() < 0.5, "{history:?}");
    assert!((history.bottom() - 993.0).abs() < 0.5, "{history:?}");
    assert!((browser.bottom() - (history.top() - 7.0)).abs() < 0.5, "{browser:?} over {history:?}");
    assert!(h.rect("canvas:grid").is_some(), "the canvas fills the rest");
    // Home brings the start page back.
    let home = h.handle("appbar:home").expect("the Home button");
    h.click([f64::from(home.x), f64::from(home.y)]);
    h.settle();
    assert!(h.rect("home:page").is_some());
    assert!(h.rect("ribbon:card").is_none());
}

#[test]
fn the_ribbon_search_sits_where_the_palette_expands() {
    let mut h = harness();
    let tab = h.handle("tab:design:1").expect("the UI kit document tab");
    h.click([f64::from(tab.x), f64::from(tab.y)]);
    h.settle();
    let search = h.rect("ribbon:search").expect("the ribbon reserves the search field");
    assert!(search.right() <= 1600.0 && search.right() > 1400.0, "{search:?}");
    h.key(Key::K, Modifiers::CTRL);
    h.settle();
    assert!(h.shell.palette.open, "Ctrl+K opens the ribbon search");
    h.text("ui kit");
    h.settle();
    h.key(Key::Enter, Modifiers::NONE);
    h.settle();
    assert_eq!(h.app.view, View::Gallery, "the search runs the UI kit");
}

#[test]
fn the_file_menu_opens_preferences() {
    let mut h = harness();
    assert!(h.click_widget("File"), "the File menu is in the app bar");
    h.settle();
    assert!(h.widget("Preferences").is_some(), "File lists Preferences");
    let preferences = h.handle("menu:shell.preferences").expect("the Preferences row");
    h.click([f64::from(preferences.x), f64::from(preferences.y)]);
    h.settle();
    assert!(h.shell.prefs.open);
}

/// Docked, the same cards are flush edges: the ribbon is a strip and the browser joins the left.
#[test]
fn docked_layout_flushes_the_cards_to_the_edges() {
    let mut h = harness();
    h.shell.settings.layout = craft_ui::layout::LayoutMode::Docked;
    h.settle();
    let tab = h.handle("tab:design:1").expect("the UI kit document tab");
    h.click([f64::from(tab.x), f64::from(tab.y)]);
    h.settle();
    assert!(h.rect("ribbon:card").is_none(), "docked has no floating ribbon card");
    let ribbon = h.rect("toolbar").expect("the docked ribbon strip");
    assert!(ribbon.top() >= 34.0 && ribbon.bottom() <= 34.0 + 117.0 + 0.5, "the strip is 117 pt: {ribbon:?}");
    let browser = h.rect("browser:card").expect("the docked browser");
    assert!(browser.left() <= 0.5, "the browser joins the left edge: {browser:?}");
    assert!(h.rect("canvas:grid").is_some());
}

/// The preferences footer keeps the dialog's opening settings to Cancel, and Apply/OK keep them.
#[test]
fn preferences_cancel_reverts_and_ok_keeps() {
    let mut h = harness();
    h.shell.run(&mut h.app, craft_shell::builtin::SHELL_PREFERENCES);
    h.frames(8);
    h.settle();
    let page = h.handle("prefs:page:prefs.interface").expect("the interface page in the rail");
    h.click([f64::from(page.x), f64::from(page.y)]);
    h.frames(4);
    h.settle();
    let light = h.handle("prefs:theme:prefs.theme_light").expect("the light theme tile");
    h.click([f64::from(light.x), f64::from(light.y)]);
    h.settle();
    assert_eq!(h.shell.settings.theme, "light", "the tile switches the theme at once");
    assert!(h.click_widget("Cancel"));
    h.settle();
    assert_eq!(h.shell.settings.theme, "dark", "Cancel puts the opening settings back");
    assert!(!h.shell.prefs.open);

    h.shell.run(&mut h.app, craft_shell::builtin::SHELL_PREFERENCES);
    h.frames(8);
    h.settle();
    let page = h.handle("prefs:page:prefs.interface").expect("the interface page in the rail");
    h.click([f64::from(page.x), f64::from(page.y)]);
    h.frames(4);
    h.settle();
    let light = h.handle("prefs:theme:prefs.theme_light").expect("the light theme tile");
    h.click([f64::from(light.x), f64::from(light.y)]);
    h.settle();
    assert_eq!(h.shell.settings.theme, "light");
    assert!(h.click_widget("OK"));
    h.settle();
    assert!(!h.shell.prefs.open);
    assert_eq!(h.shell.settings.theme, "light", "OK keeps the change");

    // Restore defaults resets the open page and keeps the dialog open.
    h.shell.run(&mut h.app, craft_shell::builtin::SHELL_PREFERENCES);
    h.frames(8);
    h.settle();
    let page = h.handle("prefs:page:prefs.interface").expect("the interface page in the rail");
    h.click([f64::from(page.x), f64::from(page.y)]);
    h.frames(4);
    h.settle();
    assert!(h.click_widget("Restore defaults"));
    h.settle();
    assert!(h.shell.prefs.open, "Restore defaults keeps the dialog open");
    assert_eq!(h.shell.settings.theme, "dark", "the interface page returns to its defaults");
}
