---
name: craft-bp-feature-slice
description: "Add one complete application feature slice to the Craft Boilerplate shell: CommandSpec, Workspace placement, ShellApp::run_command, the visible surface, en/es strings, a headless test and docs. Trigger: add a command, wire a new action into the ribbon/menu/palette, implement run_command, add a ribbon panel, make a shortcut, feature slice, new toolbar button."
---

# Add a feature slice

A slice is not complete until all seven parts exist. Reference implementation:
`apps/craft-demo/src/app.rs` (`COMMANDS`, `workspace()`, `DemoApp::execute`, `menus`).

## 1. Declare the command

In the app's static `COMMANDS` (usually `apps/<app>/src/app.rs`):

```rust
CommandSpec {
    id: "notes.add",                 // stable, lowercase, unique
    label: "cmd.notes.add",          // must start with cmd. and exist in both catalogues
    icon: "plus",                    // master id from assets/icons/ui/design/svg
    shortcut: Some("Ctrl+Alt+N"),    // optional default; conflicts are rejected by the registry
}
```

`CommandRegistry::new` rejects empty/duplicate ids, duplicate default shortcuts and labels that are
not `cmd.*`. An unknown icon id silently draws the placeholder, so verify the file exists.

## 2. Place it in the workspace

Add the id to a `PanelSpec::commands` in `workspace()` (`TabSpec` → `PanelSpec`). A command may
appear on several panels or none; the palette and keymap still see every registered command.
`Workspace::validate(&registry)` rejects unknown ids and empty tab/panel ids.

## 3. Handle it

Add an arm to `ShellApp::run_command` (in the demo: `DemoApp::execute`):

- `Ok(Some(text))` posts a status notice, message and toast.
- `Ok(None)` is silent.
- `Err(reason)` records an error in Messages, a toast and the status chip.

Do not handle shell-owned ids in the app; `Shell::run` intercepts
`shell.home`, `shell.preferences`, `shell.theme.toggle`, `shell.about`, `shell.messages`,
`shell.quit`, `shell.shortcuts`, `shell.palette`, `shell.properties`.
`shell.save`, `shell.undo` and `shell.redo` are app commands in the demo.

Use `command_enabled(id)` for availability instead of registering disabled variants. Disabled
ribbon/menu rows stay visible and ignore clicks.

## 4. Expose the surface

- Ribbon: the `Workspace` placement is enough; the shell draws it and publishes
  `ribbon:cmd:<id>`.
- App bar menu: add `MenuEntry::Command("notes.add")` to a `MenuSpec` in `menus()`.
- Palette: automatic for every registered command; no extra wiring.
- App bar save/undo/redo: the shell publishes `appbar:shell.save` etc. only when the app registers
  commands with those ids.
- Canvas/content: queue `&'static str` ids while painting and dispatch after the frame, as the demo
  does with `pending`; do not hold a `&mut App` borrow from a surface.

## 5. Add the strings

Add the label and every new panel/tab/menu/status id to **both**
`assets/locales/en.json` and `assets/locales/es.json`. English is the source of truth.
`cargo xtask i18n --check` fails on a missing translation or an unknown literal id.

## 6. Test the consumer-visible behaviour

In `apps/craft-demo/tests/ui.rs` (or the app's matching integration test):

```rust
let mut h = Harness::new(/* app + shell */);
h.key(Key::K, Modifiers::CTRL);       // palette
h.text("notes.add");
h.key(Key::Enter, Modifiers::NONE);
assert!(/* model changed */);
```

Prefer real input paths (shortcut, palette, published handle click) over calling `run_command`
directly. Assert the model/status/translation, not that a paint happened. For a new mutating
click surface, the handler must call `craft_ui::input::once_this_pass(ctx, "unique-key")`, because
egui can run more than one pass per frame.

## 7. Document and attribute

- Update `docs/adding-a-command.md` when the workflow or rules change.
- Update `docs/building-an-app.md` if the surface/method table changes.
- If code is adapted from SolidCraft, add the file to `ATTRIBUTION.md` and keep `NOTICE` truthful.
- Run the gates in `skill://craft-bp-verify`; at minimum `cargo xtask i18n --check`,
  `cargo test -p craft-demo --test ui`, `cargo clippy --workspace --all-targets -- -D warnings`.

## Traps

- **Reserved shortcuts:** `keymap::RESERVED` and `ShellApp::reserved_keys` block binding and
  dispatch. Only keys the shell actually handles should be reserved. If the shell records a quit
  request instead of acting on a key, do not add that key to `RESERVED`. (Ctrl+S/N/O are
  app-bindable; the shell handles Escape, Ctrl+K and Ctrl+Shift+P.)
- **Default vs effective shortcut:** every surface must show
  `keymap::effective(registry, &custom, id)`, not `CommandSpec.shortcut`. Otherwise a rebind is
  invisible in menus/tooltips/palette until restart.
- **Duplicate shortcut:** `CommandRegistry::new` rejects two commands sharing a default. Pick a
  different default or leave `None`.
- **Paint-time mutation:** never mutate app state directly inside a ribbon/menu row closure; queue
  the id and dispatch after painting.
- **Domain leakage:** generic widgets belong in `craft-ui::ui_kit`; commands and model types stay
  in the app crate. `craft-theme`/`craft-i18n`/`craft-ui` must not learn the app's domain.
- **Missing surface placement:** a command that is never placed is still reachable via palette and
  keymap, but it will not appear in the ribbon. That is allowed; document it if intentional.
- **Docs are part of done:** `AGENTS.md` requires the matching `docs/` file to be updated.

## Do not

- Do not store displayed text in specs; store ids.
- Do not add a second command registry or a parallel shortcut table.
- Do not bypass `Workspace::validate` or `CommandRegistry::new`.
- Do not add a shell command arm for an app command id.
