# Architecture

## Layers

| Layer | Crate | May depend on |
|---|---|---|
| L0 | `craft-theme`, `craft-i18n` | no workspace crate (egui value types only in the theme) |
| L1 | `craft-ui` | theme, i18n, egui, tiny-skia |
| L2 | `craft-shell` | ui, theme, i18n, egui |
| testkit | `craft-test` | anything below; only as a dev-dependency |
| exempt | `craft-demo`, `craft-demo-web`, `craft-demo-hot`, `craft-demo-hot-lib`, `xtask` | — |

`cargo xtask layers` reads `cargo metadata` and rejects upward dependencies, workspace
dependencies in the L0 crates, the testkit as a normal dependency, and UI toolkits below L1.

## Frame order

`Shell::ui` draws, in this order, once per frame:

1. the language scope and the appearance (theme, vision, scale);
2. scenario handles cleared, layout and active accent published;
3. the application bar (34 pt: brand, Home, menus, file actions, document tabs, Help, Settings,
   version and — where the window has no OS chrome — the caption buttons);
4. Home: the app's start page fills the window, and the ribbon's search rect is removed;
5. workspace: the ribbon (workspace card, tab row, search field, fitted command panels), the
   Browser / Properties / History cards, then the canvas and the app's centre through
   `Container::central()`;
6. the transient status chip over the workspace;
7. overlays: the ribbon search, preferences, about, messages, confirmations, toasts;
8. shortcuts: Esc closes overlays and menus, Ctrl+K / S open the search, then command bindings;
9. a pending quit sends `ViewportCommand::Close`.

Floating cards are `Container::area` hosts on the middle layer: they reserve no central space and
paint above the canvas. Docked cards use `Container::edge`, so they consume their edge before the
central panel is created.

Commands are collected while a surface paints (`&'static str` ids) and dispatched after it, so a
component never holds a borrow of the app across `Shell::run`.

egui may run several passes for one frame (a discard requests a second pass, and every pass sees
the same input events). Any click or drag handler that mutates state must be deduplicated per pass
with `craft_ui::input::once_this_pass(ctx, key)`; otherwise a menu toggles twice, Undo runs twice
or the canvas zooms twice. Painting is idempotent and needs no guard.

## The app boundary

`ShellApp` (in `crates/craft-shell/src/lib.rs`) is the only coupling between the shell and an
application. `title`, `commands`, `run_command`, `left_panel`, `right_panel` and `center` are
required; every other method has a no-op default so an app opts into surfaces as it grows. The
shell owns the chrome state and `UiSettings`; the app owns the model. `docs/building-an-app.md`
maps each method to its surface.

Persistence is `Shell::save` / `Shell::load` behind `STORAGE_UI`: JSON with `ui`, `panels`,
`active_tab`, `palette_recent` and `palette_pinned`; every field is `#[serde(default)]` and
`SavedShell::sanitize` clamps values, so old state keeps loading. The shell never persists the
app's model — an app that needs persistence writes its own keys through the same `eframe::Storage`.

Defaults: floating layout, browser open (250 pt), properties closed (264 pt), history open
(40 pt). On desktop, non-macOS `custom_titlebar` is forced on (an undecorated window needs the
app bar's captions); the Preferences checkbox is hidden there.

## Preferences semantics

The dialog edits `UiSettings` and `Panels` live. A snapshot of both is taken when it opens:
**Apply** re-baselines, **OK** keeps and closes, **Cancel**, Esc and the header's × restore the
snapshot and close, **Restore defaults** resets the open built-in page. App-provided pages are
drawn by the app: their own data is not in the shell snapshot, so an app page that edits app state
must snapshot it itself or derive it from `UiSettings`.

## Why `Container`

Outer hosts (`Area`, `Window`, `Modal`, `Panel`, `CentralPanel`, `Popup`, `SubMenu`) are constructed
only by `craft_ui::ui_kit::Container`. The container owns the surface policy: adaptive surfaces are
docked edges in `LayoutMode::Docked` and floating cards with the shell gap and elevation in
`LayoutMode::Floating`; transient surfaces keep their anchoring; the input shield never inherits
domain input blocking. Apps build their own floating tools and dialogs through the same container
(`Container::window`, `Container::modal`, `Container::area`), never with raw egui hosts.
`cargo xtask ui-containers` parses the source and fails any raw constructor, alias, type alias,
macro or frame override outside `crates/craft-ui/src/ui_kit/container.rs`.

## Tokens

Every colour, size, radius and spacing comes from `craft_theme::Tokens`. Widgets may not write
literals; new metrics are added to `Tokens`, both palettes and the theme-file key macro (which
destructures the non-colour fields exhaustively, so a new token fails to compile until it is
documented).
