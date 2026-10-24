# Craft Boilerplate

A reusable desktop **and** web application shell built on [egui](https://github.com/emilk/egui) 0.36:
design tokens, a retained ui-kit, an application bar with brand, Home, menus, file actions, document
tabs, caption buttons and a Help/Settings cluster; a two-card ribbon with a workspace switcher,
fitted command panels and an always-visible command search; floating or docked Browser, Properties
and History cards over a pannable canvas; a FileView start page; preferences with OK/Apply/Cancel,
theme tiles and colour vision; keyboard shortcuts, messages and toasts, icons and i18n — with no
domain attached.

Start an app by implementing `craft_shell::ShellApp`: six required methods (title, commands,
`run_command` and the two card bodies plus `center`); every other surface (documents, history, start
page, preferences pages) is an override. The reference
implementation uses them all. Read **[Building an application](docs/building-an-app.md)** first.

## Quickstart

```sh
cargo run -p craft-demo                  # the desktop reference application
cargo test -p craft-demo --test ui       # headless shell scenarios
cd apps/craft-demo-web && trunk serve    # the same app in a browser (http://127.0.0.1:8782)
cargo xtask ci                           # fmt, containers, i18n, layers, clippy, tests
cargo run -p craft-demo-hot              # the UI in a hot-reloaded library (docs/hot-reload.md)
```

The desktop binary persists its shell state under the OS data directory; the web build uses local
storage. Both run the identical `CraftApp` from `craft-demo`. The default view is Home (the FileView
start page); the workspace opens in the floating layout with the Browser and History cards shown.

## Layout

| Path | What it is |
|---|---|
| `crates/craft-theme` | `Tokens`, light/dark palettes, colour-vision modes, theme files |
| `crates/craft-i18n` | Message catalogues, stable ids, locale scoping |
| `crates/craft-ui` | Fonts, icon pipeline, scenario geometry, canvas, caption glyphs, `ui_kit` (the only outer-host factory) |
| `crates/craft-shell` | `ShellApp` contract, commands, app bar, ribbon, cards, canvas wiring, search, preferences, keymap, messages |
| `crates/craft-test` | Headless harness driving the real shell frame by frame |
| `apps/craft-demo` | Reference app: FileView Home, document tabs, browser tree, history strip, gallery over the canvas |
| `apps/craft-demo-hot` | Development host that hot-reloads the UI library |
| `apps/craft-demo-hot-lib` | Reloadable UI library (dylib) used by the hot host |
| `apps/craft-demo-web` | Trunk bootstrap for wasm32 |
| `xtask` | Gates: `ui-containers`, `i18n --check`, `layers`, `ci` |

See `docs/` for [building an app](docs/building-an-app.md), [architecture](docs/architecture.md),
the [ui-kit](docs/ui-kit.md), [theming](docs/theming.md), [commands](docs/adding-a-command.md),
[testing](docs/testing.md), [i18n](docs/i18n.md) and [hot reload](docs/hot-reload.md).

## Built on patterns from SolidCraft

The shell's UI layer is adapted from [SolidCraft](https://github.com/tangency-labs/solidcraft)
(MIT OR Apache-2.0), with the CAD domain removed: tokens and palettes, the container policy, the
retained kit, the icon renderer, the keymap, messages and the xtask guards. `ATTRIBUTION.md` lists
every adapted file and the changes made. Fonts are IBM Plex (SIL OFL 1.1). This project is
independent of SolidCraft.
