---
name: craft-bp-persistence
description: "Persist Craft Boilerplate shell state correctly: serde defaults and sanitize, Shell save/load under STORAGE_UI, keymap custom bindings, Preferences Apply/OK/Cancel snapshots, and the craft-settings config.toml versus eframe ui.ron split on desktop and web. Trigger: settings do not survive restart, save/load state, add a persisted field, rebindings lost, Cancel does not revert, add settings storage, craft-settings, config.toml, ui.ron, keymap persistence."
---

# Persistence in the Craft Boilerplate

There are two persistence layers, and they answer different questions:

| Layer | What | Where | Code |
|---|---|---|---|
| Desktop user settings | theme, language, layout, scale, colour vision, title-bar choice, keymap | `config.toml` via `craft-settings` (atomic, backup, recovery, migrations) | `apps/craft-demo/src/settings.rs` (`DemoSettings`, `SettingsHost`) |
| Transient UI state | active tab, palette history, window state; the shell JSON | eframe storage under `craft_shell::STORAGE_UI` (`"craft.ui"`); canvas/app keys are app-owned | `crates/craft-shell/src/settings.rs` |
| Web | no `craft-settings` store; shell JSON in local storage | eframe storage | `CraftApp::save` wasm branch |

The suite policy is: readable settings in `config.toml`; frequently changing window/UI state in
eframe storage. Do not put transient UI state in `config.toml`, and do not call `SettingsStore`
from code that compiles for wasm.

## Shell state: the contract

`crates/craft-shell/src/settings.rs`:

- `UiSettings` and `Panels` are `#[serde(default)]`. **Add fields, never rename existing ones.**
- `SavedShell { ui, panels, active_tab, palette_recent, palette_pinned }` is the on-disk JSON.
- `SavedShell::sanitize` clamps values (`theme`, `language`, `scale` 0.75–2.0, panel sizes,
  non-finite numbers, keymap strings ≤ 40 chars).
- `Shell::load` never fails: invalid JSON returns to defaults, unknown keys are ignored, an unknown
  active tab falls back to the first.
- `Shell::new(workspace, stored)` loads once; `eframe::App::save` calls `shell.save()`.

## One keymap source (implemented)

`UiSettings.keymap` and `Shell.keymap.custom` both represent rebindings. `Shell::load` sets both;
`Shell::save` copies the live map before serializing, so a rebind survives restart even before
Apply. `SettingsHost::sync` does the same for `config.toml`. The contract is:

```rust
pub fn save(&self) -> String {
    let mut ui = self.settings.clone();
    ui.keymap = self.keymap.custom.clone();
    let saved = SavedShell {
        ui,
        panels: self.panels,
        active_tab: self.ribbon.active_tab.clone(),
        palette_recent: self.palette.recent.clone(),
        palette_pinned: self.palette.pinned.clone(),
    };
    serde_json::to_string(&saved).unwrap_or_default()
}
```

Keep `load` assigning `self.keymap.custom = saved.ui.keymap.clone()` **before** `self.settings =
saved.ui`. The spec's `CommandSpec.shortcut` is only the fallback; the effective key is
`keymap::effective(&registry, &custom, id)`. Do not add a third shortcut store.

`UiSettings` should derive `PartialEq` (it is useful for snapshots/tests and cheap). All fields are
plain data.

## Preferences Cancel/Apply/OK

The dialog edits live state and snapshots on open. The snapshot must include the keymap, otherwise
Cancel leaves rebindings applied. Use a private struct rather than a growing tuple:

```rust
#[derive(Clone)]
struct Snapshot {
    settings: UiSettings,
    panels: Panels,
    keymap: BTreeMap<String, String>,
}
```

- `remember(shell, ctx)` stores `settings.clone()`, `panels`, `keymap.custom.clone()`.
- `restore(shell, ctx)` restores all three and calls `appearance_changed`.
- Apply → re-baseline (`remember`); OK → `close` (keep + drop snapshot); Cancel, Esc, header × →
  `restore` then close.
- `page_defaults("shortcuts")` clears `shell.keymap.custom`.
- App-provided preference pages own their revert state; the shell snapshot cannot know app data.

Keep `SNAPSHOT` as a private `egui::Id` key. Do not snapshot by serializing JSON.

## craft-settings (adopted in the demo)

Reference: `apps/craft-demo/src/settings.rs`; dependency is desktop-only in
`apps/craft-demo/Cargo.toml`.

- `DemoSettings` implements `craft_settings::Settings` (`APP_ID = "craft-demo"`,
  `SCHEMA_VERSION = 1`). Put `#[serde(default)]` on every struct and do **not** use
  `deny_unknown_fields`; unknown keys survive saves on purpose.
- `SettingsHost::open` calls `SettingsStore::<DemoSettings>::open(Location::Auto)`. It never fails;
  inspect `LoadReport` (`read_only`, `broken_moved_to`, `section_errors`) and log or show it.
- If the store reports `Source::Defaults`, import the old eframe JSON; otherwise apply
  `store.get()` to the shell. `SettingsHost::sync` copies live shell settings/keymap on change.
- Call `SettingsHost::tick(ctx)` every frame: it calls `store.tick(Instant::now())` and
  `request_repaint_after(next_deadline)`, so an idle app still writes after the debounce.
  `CraftApp::save` flushes the store; if you add `on_exit`, flush there too.
- `<APP>_CONFIG_DIR` (`CRAFT_DEMO_CONFIG_DIR`) isolates a profile; `Location::Memory` is for tests.
- `open_at(dir)` for settings that travel with a project; `Location::Memory` for tests/demos.
- `config.toml` is atomic (via `craft-atomic`), keeps comments/key order, keeps a `.bak`, moves
  broken TOML aside and refuses to overwrite a newer schema.
- No UI: the app renders errors.

**wasm gap:** `craft-settings` only provides the trait, document helpers and error types on
`wasm32`; there is no `dirs`-based store and no filesystem. On the web, use eframe/browser storage
(JSON in localStorage) or a server, and keep the native settings type separate. Do not call
`SettingsStore::open` from shared wasm code.

## Desktop/web split

- Native: `eframe` storage is the OS data directory (`NativeOptions.persistence` default in this
  repo's feature set); the demo saves `craft.ui` there.
- Web: eframe uses browser local storage; the same host code calls `cc.storage`. The web build has
  no `craft-settings` store and no atomic file writes.
- The harness (`craft_test::Harness`) has no storage; call `Shell::new(..., Some(json))` or
  `shell.load(json)` directly.
- Hot reload: the host owns `CraftApp`; persistence stays in the host's `save`, not in the dylib.

## Tests worth writing

1. **Round trip:** create a shell, set a setting/rebind, `let json = shell.save()`, build a new
   `Shell::new(workspace, Some(&json))`, assert both `shell.settings.<field>` and
   `shell.keymap.custom` contain the value.
2. **Keymap:** bind `shell.preferences` to `Ctrl+G`, save, reload, assert
   `custom.get("shell.preferences") == Some("Ctrl+G")` and `effective(...)` is `Ctrl+G`.
3. **Cancel:** open Preferences, change theme + rebind + panel size, press Esc/Cancel, assert all
   three are restored (including `keymap.custom`).
4. **Apply:** change, Apply, change again, Cancel → the first change survives (re-baselined).
5. **Sanitize:** load out-of-range JSON (scale 99, unknown theme, unknown language) and assert
   clamped defaults without a panic.
6. **Corrupt input:** `shell.load("{")` leaves the previous state/ default state intact.

Run `cargo test -p craft-shell` (if present) and
`cargo test -p craft-demo --test ui`; the headless harness in `docs/testing.md` is the right driver
for dialog tests.

## Traps

- **Current keymap bug:** if `save` does not copy `keymap.custom` into a cloned `UiSettings`,
  rebinds appear to work until restart. Fix the source, not each caller.
- **Cancel snapshot:** a `(UiSettings, Panels)` tuple silently drops keymap edits. Add the map.
- **sanitize is not validation:** it clamps known fields; it does not reject unknown command ids or
  malformed shortcut strings. Filter against the registry when applying loaded custom bindings if
  stale ids are possible (or accept and ignore them via `effective`).
- **Renaming fields:** `#[serde(default)]` only helps missing keys. Rename a field and old state
  silently resets to default; add an alias or keep the field.
- **Storage key drift:** desktop/web/hot hosts must use the same key. A copy-pasted host with a
  different constant looks like "settings don't persist".
- **wasm:** no filesystem, no `std::time::Instant` for the settings tick unless via `web-time`.
- **Settings tick:** forgetting `request_repaint_after` means the debounced save never runs in an
  idle window.

## Do not

- Do not persist the app's model in the shell's `STORAGE_UI`; the shell owns chrome state only.
- Do not store API keys in plain TOML/JSON; use the OS keychain when the app needs secrets.
- Do not write settings on every keystroke; use the debounce/tick.
- Do not use `unwrap` on serialization or storage; `save` returns a string and already falls back.
- Do not claim persistence works from a unit test alone when the change affects browser storage;
  smoke-test the browser build.
