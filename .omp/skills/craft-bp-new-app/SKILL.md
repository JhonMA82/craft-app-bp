---
name: craft-bp-new-app
description: "Start a new application from the Craft Boilerplate: copy apps/craft-demo, rename crates and storage keys, register the crate in the xtask layer table, wire desktop/web (and optional hot) hosts, strip demo content, update docs and run the full gates including wasm. Trigger: create a new app, bootstrap a project, copy the demo, rename craft-demo, new desktop app, new web app, scaffold an app, add an application crate."
---

# Create a new app from the boilerplate

The demo (`apps/craft-demo`) is the reference implementation. Copy it, then remove everything that
belongs to the demo's subject while keeping the shell wiring.

## 1. Copy and rename

```sh
cp -R apps/craft-demo apps/craft-notes
```

Then change, in `apps/craft-notes/Cargo.toml`:

- `package.name = "craft-notes"` (workspace dependency entries and host imports follow).
- `description`.
- Keep `edition.workspace`, `rust-version.workspace`, `license.workspace` and `[lints] workspace = true`.

`Cargo.toml` at the root already has `members = ["crates/*", "apps/*", "xtask"]`, so a directory
under `apps/` joins the workspace automatically. Do not add it to `members` manually.

Add a `craft-notes` entry to `[workspace.dependencies]` if another workspace member needs it, and
change the `craft-demo` path dependency if the web/hot hosts are copied too.

## 2. Register the layer table

`cargo xtask layers` fails with
`unknown workspace crate; register it in xtask/src/layers.rs TABLE` until the app is known.
In `xtask/src/layers.rs`, add the short name (the rule strips the `craft-` prefix) as
`Class::Exempt`, next to `("demo", Class::Exempt)`, `("demo-web", …)`:

```rust
("notes", Class::Exempt),
("notes-web", Class::Exempt),
```

Apps are exempt from the L0/L1/L2 rules, but the metadata must be known. Add the dependency
expectations to the tests in that file when the app is part of a gate fixture.

## 3. Own the storage key

`craft_shell::STORAGE_UI` currently exports `"craft.ui"`. Each eframe app has its own storage
namespace, so keeping the key is valid; if you want a distinct key (or copy the host into a shared
origin), define your own constant and use it in **every** host:

- desktop: `apps/craft-notes/src/main.rs`
- web: `apps/craft-notes-web/src/main.rs`
- hot: `apps/craft-notes-hot/src/main.rs` and the re-export in `craft-notes-hot-lib`

Keep the `save`/`load` pair: `storage.set_string(KEY, shell.save())` and
`Shell::new(workspace, stored.as_deref())`. Desktop also has `apps/craft-demo/src/settings.rs`:
rename `DemoSettings`, set a stable `APP_ID`/`SCHEMA_VERSION`, and keep the shell's `SettingsHost`
in the host so `config.toml` is atomic, backed up and migrated. On wasm there is no settings-file
layer; eframe/local storage is the only store. `CRAFT_DEMO_CONFIG_DIR` (rename with the app ID)
isolates a profile.

## 4. Wire the hosts

- Desktop `main.rs`: `craft_window::Choice::probe(...)` → `decorate(...)` → `run_native`; create
  `CraftApp::new_with_frame`, call `craft_ui::fonts::setup_bundled_fonts`, and forward
  `eframe::App::{logic, ui, save}`. Keep `logic` for the close/quit policy.
- Web `main.rs`: `eframe::WebRunner::start` with `wasm-bindgen-futures`, a `Start`/async entry,
  canvas id `craft_canvas`; map the app constructor error with `std::io::Error::other`, never `?`
  on `JsValue`.
- Hot host (optional): copy `apps/craft-demo-hot` + `craft-demo-hot-lib`; keep the state in the
  host and the `#[unsafe(no_mangle)] pub fn render(&mut CraftApp, &mut egui::Ui)` signature stable.
  See `skill://egui-hot-reload` and `docs/hot-reload.md`.
- `index.html`, `Trunk.toml`, `manifest.webmanifest`: rename the title and keep
  `target = "index.html"`, `dist = "../../dist/web"`, port 8782 unless a real conflict.

## 5. Rename the app boundary content

- Replace `DemoApp`/`View`/`gallery.rs`/demo commands with your app's model.
- Keep the `ShellApp` contract: required `title`, `commands`, `run_command`, `left_panel`,
  `right_panel`, `center`; everything else is an opt-in override.
- Replace the demo's `COMMANDS`, `workspace()`, menus, `app.start`/`app.gallery` and Home content.
- Keep or rewrite `apps/craft-notes/tests/ui.rs`; do not leave tests asserting demo behaviour.
- Remove unused `cmd.*`, `home.*`, `gallery.*`, `panel.*` keys from **both**
  `assets/locales/en.json` and `es.json`, or leave them only if genuinely used. The i18n gate
  rejects language keys missing from English, not unused English keys.
- If the app has its own icon needs, add masters under `assets/icons/ui/design/svg`; unknown ids
  draw the placeholder.

## 6. Docs and attribution

- Update `README.md` only where command lists/paths for the new app are wrong; do not duplicate the
  guide.
- `docs/building-an-app.md` remains the generic guide; add an app-specific note only if the app
  establishes a new pattern.
- Ported code must be added to `ATTRIBUTION.md`; keep `NOTICE` truthful. Do not invent license
  statements for copied assets.

## 7. Gates

Run from the repo root:

```sh
cargo fmt --check
cargo xtask ui-containers
cargo xtask i18n --check
cargo xtask layers
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p craft-notes --test ui
cargo test --workspace
cargo xtask web                     # if the app has a web host
cargo run -p craft-notes            # check the window and the Home/workspace surfaces
```

`cargo xtask ci` covers all but the wasm gate; it does not compile `apps/craft-notes-web`.
See `skill://craft-bp-verify`.

## Traps

- **Auto-member:** `apps/*` is a workspace member glob. A copied directory with a stale package
  name can collide with `craft-demo`; change the name before building.
- **Unregistered crate:** the layer gate fails until the short name is in `TABLE`. This is a
  feature, not a reason to add an exemption in the gate code.
- **Web build:** the native tests cannot see wasm-only errors (`JsValue`, `std::time`, web APIs).
  Always run `cargo xtask web`.
- **Duplicate storage key:** several apps used from the same browser origin share localStorage.
  Use a unique key or app origin.
- **Leftover demo domain:** files, commands, menu entries and translations referencing the demo
  make a "new" app look like a second demo. Grep for `craft-demo`, `DemoApp`, `gallery`, `app.` ids.
- **Hot host ABI:** renaming `CraftApp`/`Shell` fields while a hot host runs needs a restart, not a
  reload. Re-export the new types in the hot-lib and update `hot_functions_from_file!` paths.
- **Docs drift:** command lists in `README.md` and `docs/testing.md` mention `craft-demo`; update
  only the places that are now wrong for your app.

## Do not

- Do not add path dependencies to a sibling checkout of `craft-libs`.
- Do not copy the demo's business logic as a placeholder; either implement the new model or leave
  the required methods intentionally empty.
- Do not create a second shell or a second command registry.
- Do not commit generated `dist/` output.
