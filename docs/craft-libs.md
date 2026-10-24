# Shared Craft infrastructure

`craft-app-bp` consumes a small, pinned set of crates from
[`craft-libs`](https://github.com/storytold/craft-libs). They are infrastructure, not a second UI
framework: the shell and the retained `craft_ui::ui_kit` stay in this repository.

## Pinned revision

Every crate comes from the same commit:

```text
53837472d6d4dca583df434e66bbfe01d89a4b49
```

The dependencies are declared once in the root `Cargo.toml` under `[workspace.dependencies]`.
Update all of them together; mixing revisions duplicates crates and types. Do not use `path`
dependencies: a copied app must still build from the repository.

## What is adopted

| Crate | Used for | Where |
|---|---|---|
| `craft-quit` | Window close on macOS and the optional native Quit menu | `ShellApp::can_close`, `CraftApp::logic` |
| `craft-window` | Decides whether the desktop or the app draws the window frame | `apps/craft-demo/src/main.rs`, `apps/craft-demo-hot/src/main.rs` |
| `craft-settings` | `config.toml`, backup, damaged-file recovery and schema migration | `apps/craft-demo/src/settings.rs` |
| `craft-atomic` | Crash-safe file replacement under `craft-settings` | transitive |
| `craft-platform` | Native dialog lifecycle, deferred close, Linux appearance and web downloads | re-exported by `craft-shell` |
| `craft-docpath` | Refuses stored paths outside the document's own local/network location | re-exported by `craft-shell` |

`craft-shell` re-exports `craft_quit`, `craft_platform` and `craft_docpath` so an app can use them
without a second dependency declaration. `craft-platform` is feature-gated: `dialogs`, `egui` and
`appearance` on native; `browser` on wasm.

## What is deliberately not adopted

- `craft-ui` and `craft-i18n` have the same package names as the local crates. A wholesale adoption
  would need an alias (`package = "craft-ui"`) or a rename, and their models differ:
  - the local `craft-ui` owns the shell's `Container` policy, the icon palette and the retained kit;
  - the shared `craft-ui` is a smaller component set plus docking; its `DockArea` constructs its own
    egui areas/popups and therefore bypasses the `Container` gate.
  - the local `craft-i18n` uses stable ids and embedded JSON; the shared crate uses source-text
    catalogs, TSV build input and ICU4X data. Migrating is a separate ADR, not a dependency bump.
- `craft-pdf`, `craft-media`, `craft-cms`, `craft-print` and `craft-ai-core` are domain libraries.
  They stay in the apps that need them. In particular, `craft-pdf` relies on vendored
  `krilla`/`hayro` and workspace `[patch]` entries that a consuming workspace does not inherit.

The decision to keep the local UI/i18n and adopt only infrastructure is recorded in
[ADR 0001](adr/0001-shared-ui-and-i18n.md).

## Settings and UI state

Desktop keeps two stores on purpose:

- `config.toml` via `craft-settings`: theme, language, layout, scale, colour vision, title-bar
  choice and keymap. It is atomic, backed up, recoverable and migratable. `DemoSettings` is the
  reference type in `apps/craft-demo/src/settings.rs`; `<APP>_CONFIG_DIR` (for the demo,
  `CRAFT_DEMO_CONFIG_DIR`) isolates a profile.
- eframe storage for transient UI state: active tab, palette history and the host's window state.
  The shell's `SavedShell` is the current format; apps may add their own eframe keys for canvas or
  window geometry. This is the split recommended by `craft-libs`' settings policy.

Web has no `craft-settings` store. `craft-demo-web` keeps the shell JSON in eframe/local storage.
Any app that needs cross-platform settings must put a small storage trait in front of both
backends; do not call `SettingsStore` from shared code that compiles for wasm.

`SettingsHost` calls `SettingsStore::tick` every frame and uses `next_deadline` to request a repaint,
so an idle app still saves after the debounce. `eframe::App::save` flushes the store on exit.

## Close and quit

The shell no longer closes the window itself. `shell.quit` sets a request; the host consumes it:

1. `ShellApp::can_close` decides whether the app may close.
2. If not, the host sends `CancelClose` and calls `ShellApp::close_denied` (show the save prompt).
3. Otherwise the host sends `ViewportCommand::Close`.
4. `craft_quit::on_close_request(ctx)` runs last, after the app's decision and after
   `craft_platform::dialogs::DeferredClose` while a picker or async save is pending.
5. On macOS, `craft_quit` cancels the close and quits through `NSApplication`; `install_app_menu`
   replaces winit's default Quit so Cmd+Q asks the app first.

## Window frame

`craft_window::Choice::probe(Prefix::new("CRAFT"))` runs once in `main`, before `run_native`, and
`craft_window::decorate` applies the decision. The app passes `Some(app_frame)` to
`CraftApp::new_with_frame` so the shell knows whether to draw caption buttons. `CRAFT_TITLEBAR=app`
or `=system` overrides the probe. On platforms where the desktop draws the frame, the app bar must
not draw minimize/maximize/close.

## Updating the pin

1. Pick one new revision and change every `craft-*` workspace dependency to it.
2. Read the upstream README/CHANGELOG for the crates you use; this suite has no tags.
3. Run `cargo xtask ci` and `cargo xtask web`.
4. Run the desktop app and check the frame/close paths; macOS quit and native dialogs still need
   platform acceptance.
5. Update `ATTRIBUTION.md`/`NOTICE` if the dependency set or its licences changed.

Do not add a second `craft-ui` or `craft-i18n` to the workspace; see the alias rule above.
