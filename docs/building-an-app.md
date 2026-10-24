# Building an application on the boilerplate

The boilerplate is a shell you fill in, not a framework you inherit from. Your app owns the model;
`craft-shell` owns the window chrome, the workspace, persistence and the settings dialog. This
guide is the whole path from an empty crate to a scaled app; the other `docs/` files go deeper on
one area each.

Reference implementation: `apps/craft-demo/src/app.rs` exercises most of the extension points
described here (documents, history, Home, cards, menus, file actions and the gallery card).

## 1. The contract: `ShellApp`

Implement `craft_shell::ShellApp` on your app type. Six methods are required — `title`,
`commands`, `run_command`, `left_panel`, `right_panel` and `center` (an app without a card can
leave the two panel bodies empty); everything else has a no-op default, so a minimal app stays
minimal and grows by override.

| Method | Required | What the shell does with it |
|---|---|---|
| `title` | yes | Product name in the app bar (apps usually reuse it in their Home rail) |
| `commands` | yes | `CommandRegistry` for the ribbon, menus, palette and keymap |
| `run_command` | yes | Dispatches app command ids; `Ok(Some(text))` posts a status notice, `Err` records a failure |
| `command_enabled` | no | Dims and disables ribbon rows/menu rows; default `true` |
| `menus` | no | One flat app-bar button per `MenuSpec` |
| `left_panel_title`, `left_panel_subtitle`, `left_panel` | `left_panel` yes | Browser card title, right-aligned count, and body |
| `right_panel_title`, `right_panel` | `right_panel` yes | Properties card title and body |
| `center` | yes | The workspace content drawn over the canvas, under the floating cards |
| `documents`, `active_document`, `select_document`, `close_document` | no | Application-bar document tabs; `home_open` decides which tab is active |
| `home_open`, `show_home`, `home` | no | The FileView start page (shown instead of the ribbon, cards and canvas) |
| `history`, `history_selected`, `history_transport` | no | The 40 pt history strip and its transport buttons; `None` hides it |
| `status` | no | The transient status chip (text + optional `fraction` for a running task) |
| `preference_pages`, `draw_preferences_page` | no | Extra pages in the Preferences rail |
| `reserved_keys` | no | Shortcuts the keymap must not steal from your content |

Start with the six required methods. Add the rest when your app has a browser, documents, history
or a start page.

## 2. A minimal application

```rust
use craft_shell::commands::{CommandRegistry, CommandSpec, PanelSpec, TabSpec, Workspace};
use craft_shell::{Shell, ShellApp};
use craft_theme::Accent;

static COMMANDS: &[CommandSpec] = &[
    CommandSpec { id: "notes.add", label: "cmd.notes.add", icon: "plus", shortcut: Some("Ctrl+N") },
];

pub fn workspace() -> Workspace {
    Workspace {
        tabs: vec![TabSpec {
            id: "home",
            label: "tab.home",
            accent: Accent::Blue,
            panels: vec![PanelSpec { id: "notes", label: "panel.notes", commands: &["notes.add"] }],
        }],
    }
}

pub struct App {
    pub registry: CommandRegistry,
    pub workspace: Workspace,
    pub notes: Vec<String>,
}

impl App {
    pub fn new() -> Result<Self, String> {
        let registry = CommandRegistry::new(COMMANDS.to_vec())?;
        let workspace = workspace();
        workspace.validate(&registry)?;
        Ok(Self { registry, workspace, notes: Vec::new() })
    }
}

impl ShellApp for App {
    fn title(&self) -> &str {
        "Notes"
    }
    fn commands(&self) -> &CommandRegistry {
        &self.registry
    }
    fn run_command(&mut self, id: &str) -> Result<Option<String>, String> {
        match id {
            "notes.add" => {
                self.notes.push(format!("Note {}", self.notes.len() + 1));
                Ok(Some(format!("{} notes", self.notes.len())))
            }
            other => Err(format!("unknown command `{other}`")),
        }
    }
    fn left_panel(&mut self, _ui: &mut egui::Ui) {}
    fn right_panel(&mut self, _ui: &mut egui::Ui) {}
    fn center(&mut self, ui: &mut egui::Ui) {
        ui.label(format!("{} notes", self.notes.len()));
    }
}
```

### The host

One `Shell` per window. Persist with `Shell::save` under `STORAGE_UI`:

```rust
struct Host {
    shell: Shell,
    app: App,
}

eframe::run_native(
    "Notes",
    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1440.0, 900.0]),
        ..Default::default()
    },
    Box::new(|cc| {
        craft_ui::fonts::setup_bundled_fonts(&cc.egui_ctx);
        let stored = cc.storage.and_then(|storage| storage.get_string(craft_shell::STORAGE_UI));
        let app = App::new().map_err(std::io::Error::other)?;
        let shell = Shell::new(app.workspace.clone(), stored.as_deref());
        Ok(Box::new(Host { shell, app }))
    }),
)
```

```rust
impl eframe::App for Host {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.shell.ui(&mut self.app, ui);
    }
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string(craft_shell::STORAGE_UI, self.shell.save());
    }
}
```

The same host works on wasm (`eframe::WebRunner`, see `apps/craft-demo-web`). Where the window has
no OS decorations (desktop, non-macOS), pass `.with_decorations(false)`: `UiSettings::sanitize`
then keeps `custom_titlebar` on so the app bar's caption buttons can close the window.

### Or copy the reference app

For a real application, copy the reference crate rather than the skeleton:

```sh
cp -r apps/craft-demo apps/my-app
```

1. Rename `name = "craft-demo"` to `my-app` in `apps/my-app/Cargo.toml`; the workspace already
   includes `apps/*`.
2. **Register the crate** in `xtask/src/layers.rs`: add `("my-app", Class::Exempt)` to `TABLE`.
   An unregistered workspace crate fails `cargo xtask layers`.
3. Keep `app.rs`'s host shape (`CraftApp { shell, demo }`, `draw`, `eframe::App`, `save`) and
   replace `DemoApp` and `gallery.rs` with your model and surfaces.
4. For a browser or hot-reload host, copy `apps/craft-demo-web` / `craft-demo-hot` /
   `craft-demo-hot-lib` and repoint their `craft-demo` dependency and imports at `my-app`.
5. Delete the demo specimens you do not need; keep one File menu, the required `ShellApp` methods
   and the i18n keys you actually use.

## 3. Choose the surfaces

### Home and the workspace

`home_open() == true` makes the shell draw `home(ui)` full-bleed and hide the ribbon, cards,
canvas and history. That is what the FileView rails in `craft_ui::ui_kit::FileView` are for. Home
is not a separate window: `documents()` keeps the tab strip visible and `show_home` is dispatched by
the Home button and the `+` tab.

Typical wiring (the demo does exactly this):

- one `View` enum with `Start` and your workspace view,
- `home_open` → `view == Start`, `show_home` → `set_view(Start)`,
- `documents` → one `DocumentTab` per view, `active_document` → view index,
- `select_document` → `set_view`, `close_document` → the other view.

### Document tabs

Tabs come from `documents()` as `DocumentTab { title, dirty }`; their widths shrink to 90 pt and
overflow into a `»` list automatically. A `dirty: true` tab gets the `•` suffix. The close button only appears while more
than one tab exists; `close_document(index)` owns what closing means. The `+` button and the Home
icon dispatch `shell.home`.

### Browser (left card)

`left_panel_title` defaults to `ui.browser_title`; `left_panel_subtitle` is right-aligned before
the fold chevron (use it for a count or scope). `left_panel(ui)` runs inside the card's scroll area.
The floating card is 250 pt wide by default (resizable 170–620 by its right rail; the docked
column keeps its fixed width) and its chevron folds the body through `Panels::left.collapsed` (the
floating card keeps its column height). Rows are yours; `TREE_ROW_HEIGHT` (26 pt) matches the
shell's density. Nothing in the card is a tree widget: the demo draws rows with
`allocate_exact_size` and an icon.

### Properties (right card)

Closed by default. `right_panel_title` defaults to `panel.properties`. `right_panel(ui)` runs in a
scrolling body under the shared card header; the card's close button sets `Panels::right.open =
false`. Open it from code with `shell.run(app, builtin::SHELL_PROPERTIES)` or from Preferences.

### History (bottom strip)

`history()` returns `HistoryView { items: Vec<HistoryItem>, marker }` or `None` (no strip, no
reserved space). Each `HistoryItem { id, label, icon }` is the chip's stable id, translated label
and master icon. The
strip is 40 pt high and floats over the bottom of the workspace; the canvas's navigation bar lifts
above it automatically. `marker` is the index of the current step (0-based); the marker line is
drawn before that chip and that chip is highlighted. A marker at `items.len()` draws the line after
the last chip. Clicking a chip calls `history_selected(id)`; the four transport buttons call
`history_transport(HistoryTransport::{First,Back,Forward,Last})`. Map those four to your own
undo/redo semantics.

### Canvas and centre

The workspace paints a domain-free `craft_ui::canvas` behind the app: gradient, isometric grid,
origin triad, axis view, navigation bar. Middle-button drag pans; the wheel is deliberately not
read, so scrollable content over the canvas keeps its scroll. Zoom/Fit/Home/Settings/Messages come
back as
`CanvasAction` and are handled by the shell (`Fit`/`Home` reset pan and zoom). `center(ui)` is
drawn after the canvas, so it can paint a card over the grid; it is **under** the floating Browser
and Properties cards, which is why a centred card should leave their columns free or be started
with `home`-style full-bleed content.

### App-owned windows and overlays

The shell hosts Preferences, About, Messages and confirmations. An app that needs its own floating
tool, picker or blocking dialog builds it through the same container, so it keeps the surface
policy and the gates:

```rust
use craft_ui::ui_kit::{Container, Surface};

Container::new(ui.ctx(), Surface::Window)
    .window(craft_ui::i18n::tr("tool.title"))
    .open(&mut self.tool_open)
    .show(ui.ctx(), |ui| {
        // tool body
    });
```

`Surface::Popup` is for anchored menus (`Container::new(ctx, Surface::Popup).area(id)`),
`Surface::Window` for movable tools, and `Container::new(ctx, Surface::Window).modal(id)` for a
blocking dialog. Never construct `Area`/`Window`/`Modal` yourself: `cargo xtask ui-containers`
rejects it outside the container.

### Status, toasts and messages

There is no permanent status line. `status()` returns `Some(Status { text, fraction: Some(f) })` for
a running task (the chip draws a spinner) or `None`. `Shell::notify(text, error)` posts a six-second
chip, a toast and a Messages entry; `Shell::confirm(ConfirmRequest)` queues a modal whose answer
runs after it closes.

## 4. Commands, menus and the ribbon

1. **Declare** a `CommandSpec` (static slice preferred): `id`, a `cmd.*` i18n label, a master icon
   id and an optional default shortcut. `CommandRegistry::new` rejects empty ids, non-`cmd.*`
   labels, duplicate ids and duplicate default shortcuts.
2. **Place** it in a `Workspace`: tabs (label + `Accent`) and panels (label + command ids).
   `Workspace::validate` rejects unknown commands. A command may appear on several panels or on
   none (menus, palette and keymap still see it).
3. **Handle** it in `run_command`. Return `Ok(None)` for silence, `Ok(Some(text))` for a notice,
   `Err(reason)` for a failure (Messages + toast + chip).
4. **Gate** it with `command_enabled`; disabled rows stay visible, dimmed.
5. **Add text** for `cmd.*`, `panel.*`, `tab.*` and `menu.*` to both catalogues
   (`cargo xtask i18n --check`).

The ribbon fits automatically: buttons narrow to 54 pt, then panels collapse to one drop-down
button, then the tail moves into a `»` menu; nothing becomes unreachable. Section labels are the
panel labels and open the panel's drop-down. The search field on the tab row is the palette; it is
always visible in the workspace.

Menus are `MenuSpec { title, items }` with `MenuEntry::{Command, Separator, Preferences, About,
Quit}`. A single File menu plus save/undo/redo file actions is the reference app bar; menu buttons
are hidden before the right cluster when the bar gets narrow.

Shell-owned commands (do not reimplement): `shell.home`, `shell.preferences`, `shell.theme.toggle`,
`shell.about`, `shell.messages`, `shell.quit`, `shell.shortcuts`, `shell.palette`,
`shell.properties`. Any other id — including `shell.save`, `shell.undo`, `shell.redo` — falls
through to your `run_command`, which is how the demo binds them.

## 5. Preferences and settings

Built-in pages: General (language, version), Interface (theme tiles, layout rows, scale, font,
colour vision), Shortcuts. The footer is **Restore defaults · Cancel · Apply · OK**:

- edits apply live;
- **Apply** makes the current values the new baseline;
- **OK** keeps them and closes;
- **Cancel**, Esc and the header’s × put back the `UiSettings`/`Panels` from when the dialog
  opened. Your own state is not reverted by the shell: if an app page edits app data, snapshot it
  yourself or read it back from `UiSettings`.

`UiSettings` fields: `theme` (`"dark"`/`"light"`), `language`, `layout` (`Floating` default,
`Docked`), `scale`, `color_vision`, `custom_titlebar`, `keymap`. All are `#[serde(default)]`, so old
saved state keeps working. Defaults: left card open 250 pt, right card closed 264 pt, history open
40 pt, floating layout.

Add a page with `PageSpec { id, label, icon }` in `preference_pages()` and draw it in
`draw_preferences_page(ui, id)`. The rail groups your pages under `prefs.group_app`. Use the kit's
`settings_heading`, `settings_section`, `settings_pair`, `settings_form`/`row`,
`dropdown_menu`/`list_picker`, `segmented`, `checkbox`, `theme_tile` and `theme_rail` to look like
the built-in pages.

## 6. Scaling the app

- **One `Shell` per window.** Several windows mean several shells and separate storage keys; the
  shell itself is not shareable across threads.
- **Keep the layers clean.** Domain models never enter `craft-ui`/`craft-theme`; generic widgets go
  in `craft-ui::ui_kit` (new module + re-export), app-specific composables stay in your app. The
  `layers` gate rejects upward dependencies, and `ui-containers` rejects raw egui hosts outside
  `Container`.
- **Grow by feature slices.** A slice is: `CommandSpec` rows, a `PanelSpec` (and tab if new),
  `run_command` arms, one draw function for its card/centre surface, i18n keys, and tests. Slices
  share the registry, the Workspace and the panels; they never add a second layout or settings.
- **Long work:** report `fraction` through `status()` while it runs, `notify` the outcome, keep the
  UI thread free, and `ctx.request_repaint()` from the worker. The chip disappears six seconds
  after the last notice.
- **Many commands:** ribbon fitting, palette ranking and the keymap scale with the registry; use
  `command_enabled` instead of adding disabled variants, and `reserved_keys` for content keys.
- **New icons:** drop an SVG master in `assets/icons/ui/design/svg` (flat directory; the build
  script scans only that) on the 64-unit grid, using `palette.toml` roles for colour. Unknown ids
  draw the placeholder. Command and history icons are ids in the registry/data.
- **New locale:** JSON catalogue + `Catalog` entry in `craft-i18n` + both `LOCALES`/`COMPLETE`
  lists as `docs/i18n.md` describes.
- **Tests:** drive the real shell with `craft_test::Harness` and assert on published geometry and
  app state; see `docs/testing.md`. A click can be delivered on several egui passes, so guard
  mutating click handlers with `craft_ui::input::once_this_pass`.
- **Attribution:** code adapted from SolidCraft gets a row in `ATTRIBUTION.md`; keep `NOTICE` and
  the licence files untouched.

## 7. Feature checklist

- [ ] `CommandSpec` added (unique id, `cmd.*` label, icon master, unique shortcut).
- [ ] Command placed on a `TabSpec`/`PanelSpec` (or deliberately palette/menu-only).
- [ ] `run_command` arm; `command_enabled` if it can be unavailable.
- [ ] Menu entry if users should reach it from the app bar.
- [ ] Surface changed (`center`, `home`, `left_panel`, `right_panel`, `history`, `documents`).
- [ ] `en.json` + `es.json` keys; `cargo xtask i18n --check`.
- [ ] Preferences page/row if it has settings; defaults behave under Cancel/Apply/OK.
- [ ] Headless test for the consumer-visible behaviour and, if it is chrome, a geometry assertion.
- [ ] `cargo xtask ci`.
