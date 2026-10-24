# Hot reload

`craft-demo-hot` runs the demo in an eframe window whose entire UI is drawn by a reloadable
library, `craft-demo-hot-lib`. Rebuild **only the library** while the app runs and the next frame
draws the new code — no host compile, no window restart, and the shell state survives.

```
crates/craft-shell, craft-ui, craft-theme, craft-i18n
        │ (linked into the dylib)
apps/craft-demo-hot-lib   ← rebuild this while the host runs
        │
apps/craft-demo-hot       ← host: owns CraftApp, the window and the eframe loop
```

The host owns the `CraftApp` (shell + demo app). The library exports one function:

```rust
#[unsafe(no_mangle)]
pub fn render(state: &mut CraftApp, ui: &mut egui::Ui) {
    state.draw(ui);
}
```

`hot-lib-reloader` loads `target/debug/libcraft_demo_hot_lib.so`, watches it, and swaps it when it
changes. A subscription thread asks egui to repaint. The host's `hot_module!` generates the
forwarding wrapper from the library source (`hot_functions_from_file!`), so keep `render`'s
signature and the `CraftApp` layout stable:

- Changing drawing code in `craft-shell`, `craft-ui` or `craft-demo` hot-reloads.
- Changing the signature of `render`, or the fields of `CraftApp`/`Shell`/`DemoApp`/`DemoSettings`,
  requires restarting the host (the old state in the host has the old layout). `CraftApp` now owns
  the desktop `SettingsHost`, so its layout is part of the ABI.
- Global state inside the library resets on reload. This app keeps its state in the host, so it
  does not.

## Running

Terminal 1 — rebuild the library on every change (install once: `cargo install cargo-watch`):

```sh
cargo watch -w crates -w apps/craft-demo/src -w apps/craft-demo-hot-lib \
    -x 'build -p craft-demo-hot-lib'
```

Terminal 2 — the host. Build it once; after that it only loads new libraries:

```sh
CRAFT_DEMO_CONFIG_DIR=target/hot-config cargo run -p craft-demo-hot
```

`CRAFT_DEMO_CONFIG_DIR` keeps the hot host's `config.toml` out of the real profile.

Keep the watcher and host in one command if you prefer:

```sh
parallel --line-buffer --halt now,fail=1 ::: \
  "cargo watch -w crates -w apps/craft-demo/src -w apps/craft-demo-hot-lib -x 'build -p craft-demo-hot-lib'" \
  "cargo run -p craft-demo-hot"
```

Edit a label, spacing or paint call, save, and watch the running window update. `cargo watch`
builds the library and its dependencies; the host is not recompiled.

## Notes

- The host looks for the dylib next to its own executable. That is the workspace's shared
  `target/`, and keeps working with `CARGO_TARGET_DIR`, `--target` and release profiles; the
  loader's default (`<crate>/target/debug`) does not exist for a workspace member.
- Debug builds only. The library is a dylib; release builds of the normal demo do not include any
  of this.
- Linux and Windows work out of the box; on macOS the library is codesigned automatically by
  `hot-lib-reloader` (Xcode command line tools).
- The web build (`craft-demo-web`) does not depend on these crates and cannot hot-reload; rebuild
  Trunk as usual.
- `apps/craft-demo-hot-lib` allows `unsafe_code` for the single `#[unsafe(no_mangle)]` export. It
  contains no `unsafe` blocks; every other lint matches the workspace.
