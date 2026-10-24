# Repository rules

## Boundaries

- Only `craft_ui::ui_kit::Container` constructs outer hosts (`Area`, `Window`, `Modal`, `Panel`,
  `CentralPanel`, `Popup`, `SubMenu`). `cargo xtask ui-containers` enforces it.
- Layers: `craft-theme`/`craft-i18n` (L0) → `craft-ui` (L1) → `craft-shell` (L2). Apps are exempt;
  `craft-test` is a dev-dependency only. `cargo xtask layers` enforces it.
- Shared infrastructure is pinned from `craft-libs` at one git revision (`craft-quit`,
  `craft-window`, `craft-settings`, `craft-atomic`, `craft-platform`, `craft-docpath`); see
  `docs/craft-libs.md`. The local `craft-ui`/`craft-i18n` are not the craft-libs crates.
- Every colour, size, radius and spacing comes from `craft_theme::Tokens`; no literals in widgets.
- No `unwrap`, `expect`, `panic`, `todo`, `unimplemented` or `unreachable` in production code. Test
  modules that need them add `#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]`.
- UI strings use stable catalog ids (`ui.*`, `menu.*`, `cmd.*`); `tr_label` is for data and
  generated labels only. `cargo xtask i18n --check` enforces the catalogues.
- `#![forbid(unsafe_code)]` comes from the workspace lints.

## Building an app

- `craft_shell::ShellApp` (in `crates/craft-shell/src/lib.rs`) is the only app boundary: required
  `title`, `commands`, `run_command`, `left_panel`, `right_panel` and `center`; every other surface
  is an override with a no-op default. `docs/building-an-app.md` maps each method to the surface it drives and is the place to
  start for any new app or feature; `apps/craft-demo/src/app.rs` is the reference
  implementation.
- One `Shell` per window. Desktop keeps user-editable settings in `config.toml` through
  `craft-settings`; transient UI state uses `Shell::save`/`Shell::load` under `STORAGE_UI`.
  `UiSettings` and `Panels` are `#[serde(default)]`, so add fields, never rename existing ones.
- Defaults: floating layout, Browser open (250 pt), Properties closed (264 pt), History open
  (40 pt). Home is the app's `home()` (FileView) and hides the ribbon, cards and canvas.
- Surfaces: application bar (title, Home, menus, file actions, document tabs, Help/Settings,
  captions), ribbon (workspace switcher, tabs, fitted panels, search), Browser/Properties/History
  cards, canvas (grid/triad/axis/nav), `center`, transient status chip, Preferences pages.
- Preferences edits apply live; OK/Apply/Cancel snapshot `UiSettings` + `Panels` (Cancel and Esc
  restore, Apply re-baselines). App-provided pages own their own revert state.
- A feature slice is: `CommandSpec` + `Workspace` placement + `run_command` arm + its surface +
  `en`/`es` keys + a harness test (+ a `MenuSpec` entry when the app bar should reach it). Generic
  widgets go into `craft-ui::ui_kit`; the domain stays in the app crate.

## UI rules

- Mutating click/drag handlers call `craft_ui::input::once_this_pass(ctx, key)`: egui can run
  several passes for one frame and would otherwise run the action twice.
- Floating cards are `Container::area` on the middle layer; docked cards are `Container::edge`.
  Presentation comes from `LayoutMode`, never from a panel's name.
- Port SolidCraft's UI code and constants rather than inventing a second visual language; list each
  adapted file in `ATTRIBUTION.md`.
- New icon ids go into `assets/icons/ui/design/svg` (flat directory, 64-unit grid, colours through
  `palette.toml`); an unknown id silently draws the placeholder.

## Documentation

Docs are part of the deliverable: changing the shell, its surfaces, tokens, handles or settings
means updating the matching file in `docs/` (`building-an-app`, `architecture`, `ui-kit`,
`theming`, `adding-a-command`, `testing`, `i18n`, `hot-reload`) so the next agent can build and
scale an app from the text alone.

## Commands

```sh
cargo xtask ci                    # fmt, containers, i18n, layers, clippy, tests
cargo xtask web                   # wasm32 check for craft-demo-web (ci does not cover it)
cargo test -p craft-demo --test ui # headless shell scenarios
cargo run -p craft-demo           # the reference app (check the surface you changed)
cd apps/craft-demo-web && trunk serve
```

## Attribution

Portions of the UI layer are adapted from SolidCraft (MIT OR Apache-2.0). Keep `NOTICE`,
`ATTRIBUTION.md` and the licence files in sync when adding adapted code.
