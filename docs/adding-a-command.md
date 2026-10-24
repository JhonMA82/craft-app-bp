# Adding a command

1. **Declare it** in the app's static spec list:

   ```rust
   CommandSpec { id: "demo.share", label: "cmd.demo.share", icon: "copy", shortcut: Some("Ctrl+Shift+S") }
   ```

   `CommandRegistry::new` rejects empty ids, duplicate ids, duplicate default shortcuts and labels
   that are not `cmd.*` ids. `icon` is a master id from `assets/icons/ui/design/svg`; an unknown id
   draws the placeholder.

2. **Place it** in the workspace (`TabSpec` → `PanelSpec::commands`). `Workspace::validate` rejects
   unknown ids. A command may appear on several panels, or on none: the palette and the keymap
   still see every registered command, and menus reach it through `MenuEntry::Command`.

3. **Handle it** in `ShellApp::run_command`, returning `Ok(Some(message))` for a success notice,
   `Ok(None)` for silence, or `Err(reason)` (the shell records it in Messages and a toast, and shows
   the label on the transient status chip). Shell-owned ids (`shell.home`, `shell.preferences`,
   `shell.theme.toggle`, `shell.about`, `shell.messages`, `shell.quit`, `shell.shortcuts`,
   `shell.palette`, `shell.properties`) never reach the app; anything else, including
   `shell.save`/`shell.undo`/`shell.redo`, is yours.

4. **Gate availability** with `ShellApp::command_enabled`; disabled ribbon rows and menu rows stay
   visible but dimmed and ignore clicks. Prefer this over adding disabled command variants.

5. **Reach it from the app bar** with `MenuSpec { title, items }`; entries are
   `MenuEntry::Command(id)`, `Separator`, `Preferences`, `About` or `Quit`. The row shows the spec's
   icon, translated label, shortcut and enabled state.

6. **Add the text** to `assets/locales/en.json` (`cmd.demo.share`) and `es.json`, plus `panel.*`,
   `tab.*` and `menu.*` ids for anything new on the ribbon. `cargo xtask i18n --check` fails on a
   missing key or an unknown id used in code.

7. **Shortcuts**: the spec default is used until the user rebinds it in Preferences › Shortcuts.
   Only keys the shell handles itself — `Escape`, `Ctrl+K` and `Ctrl+Shift+P` (`keymap::RESERVED`) —
   plus whatever `ShellApp::reserved_keys` adds for your content cannot be bound. `Ctrl+S`, `Ctrl+N`
   and `Ctrl+O` are ordinary app keys: bind them freely.

8. **Document tabs and history** are model state, not commands: wire them through
   `documents`/`select_document`/`close_document` and `history`/`history_selected`/
   `history_transport` as `docs/building-an-app.md` describes.

9. **Test** with the headless harness: run the command through `Shell::run`, drive it from the
   palette (`Ctrl+K`, query, Enter) or click its published handle (`ribbon:cmd:<id>`,
   `menu:<id>`), then assert the model. A click can arrive on more than one egui pass; mutating
   handlers need `craft_ui::input::once_this_pass`.

10. **Attribute** any code adapted from SolidCraft with a row in `ATTRIBUTION.md`.
