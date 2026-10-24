# Testing

## The harness

`craft_test::Harness<A: ShellApp>` draws the real shell and app into an egui context with no window:

```rust
use craft_test::Harness;
use egui::{Key, Modifiers};

let mut h = Harness::new(|| {
    let app = DemoApp::new().expect("valid registry and workspace");
    let shell = craft_shell::Shell::new(app.workspace.clone(), None);
    (app, shell)
});
h.key(Key::K, Modifiers::CTRL);   // open the ribbon search
h.text("ui kit");
h.key(Key::Enter, Modifiers::NONE);
assert_eq!(h.app.view, craft_demo::View::Gallery);
```

- `frame` / `frames(n)` run frames with the queued events; the window is 1600×1000 by default
  (`size(vec2)` changes it). The desktop demo asks for 1440×900 (the compositor may clamp it); the
  harness is larger so every card fits.
- `click(at)` sends the pointer move, press and release frames (ending with two more; call
  `settle()` when a layout change must finish). `click_widget("File")` settles first, then
  matches an accesskit label **exactly**; failing that, it matches a button whose label starts with
  the text before a space (menu rows append their shortcut, e.g. `"Save Ctrl+S"`).
- `widget` / `widget_near` return accesskit centres; `handle(name)` / `rect(name)` read last
  frame's published geometry.
- `settle()` runs frames until egui stops asking for another pass. Do it after opening a card or
  the Preferences dialog: their first frames size the body, so a handle read before it settles can
  point at where the row *was*.
- Prefer published handles to labels when a label is ambiguous: `"Interface"` also prefixes
  `"Interface language"`, so rail clicks use `prefs:page:prefs.interface`, not `click_widget`.

## Published geometry

Widgets and shell chrome publish handles and rects through `craft_ui::scenario`:

| Prefix | What |
|---|---|
| `appbar`, `appbar:home`, `appbar:shell.save`/`undo`/`redo`, `appbar:menu:<i>` | application bar and its controls |
| `tab:design:<i>`, `tab:close:<i>`, `tab:new`, `tab:overflow` | document tabs |
| `ribbon:card`, `ribbon:workspace`, `ribbon:tab:<id>`, `ribbon:search` | floating ribbon geometry |
| `ribbon:cmd:<id>`, `ribbon:section:<tab>:<panel>`, `panel:<tab>:<panel>` | command buttons and panels |
| `browser:card`, `browser:header`, `browser:rail` | browser card |
| `properties:card` | properties card (when open) |
| `history:card`, `history:<id>`, `history:first/back/forward/last` | history strip |
| `canvas:grid`, `axis_view:outer`, `axis_view:x/y/z`, `nav:<icon>` | workspace canvas |
| `status:chip` | transient status chip |
| `prefs:pages`, `prefs:page:<id>`, `prefs:theme:<id>` | Preferences rail and theme tiles (the published name uses the page/theme i18n id) |
| `home:page`, `home:rail`, `home:back`, `home:nav:<id>` | FileView start page |
| `menu:<id>` | rows of an open application menu |

Use them for layout assertions: the app bar is 34 pt, floating cards sit on a 7 pt gap, the
browser is 250 pt by default, the history strip is 40 pt and ends 7 pt above the workspace bottom.

```rust
let tab = h.handle("tab:design:1").expect("the second document tab");
h.click([f64::from(tab.x), f64::from(tab.y)]);
h.settle();
let browser = h.rect("browser:card").expect("the browser card");
assert!((browser.left() - 7.0).abs() < 0.5);
let history = h.rect("history:card").expect("the history strip");
assert!((browser.bottom() - (history.top() - 7.0)).abs() < 0.5);
```

## One click, one side effect

egui can run a second pass for a frame, and every pass sees the same pointer events. A mutating
click handler must be guarded with `craft_ui::input::once_this_pass(ctx, key)`; the shell does this
for menus, ribbon commands, panel toggles, canvas navigation and card gestures. A new clickable
surface that mutates app state needs the same guard, or the action can run twice in one frame.

## Tests worth writing

Consumer-visible behaviour: a command changes the model, a setting survives save/load, Cancel
restores the opening preferences, a rebind moves the shortcut, both themes draw the gallery, a
translated label appears after switching language, a geometry expectation for the shell's cards.
Keep unit tests for pure functions (`fit`, registry validation, theme contrast/round trip, key
parsing) beside the code.

Do not assert incidental wording or defaults; do not test that a draw call happened. When a test
only proves a copy of the implementation, it will fail for the wrong reason later.

## Gates

`cargo xtask ci` = `fmt --check`, `ui-containers`, `i18n --check`, `layers`, `clippy -D warnings`,
`test --workspace`. The container gate is the important one for UI code: only
`crates/craft-ui/src/ui_kit/container.rs` may name raw egui hosts. Run
`cargo test -p craft-demo --test ui` for the headless shell scenarios alone.
