---
name: craft-bp-wasm-web
description: "Build and debug the Craft Boilerplate browser app: wasm32-unknown-unknown, Trunk, eframe WebRunner, wasm-bindgen-futures, JsValue error mapping, browser storage, web downloads/dialogs, and the cargo xtask web gate. Trigger: web build, wasm compile error, browser app, Trunk, WebRunner, wasm-bindgen, localStorage, browser download, web smoke test, cargo xtask web, wasm32, web persistence."
---

# The browser build

Crate: `apps/craft-demo-web` (Trunk bootstrap). It compiles `craft-demo` for
`wasm32-unknown-unknown` and runs the same `CraftApp` as desktop.

## Build and run

```sh
rustup target add wasm32-unknown-unknown
cargo xtask web                                  # fast type-check
cd apps/craft-demo-web && trunk serve            # http://127.0.0.1:8782
cd apps/craft-demo-web && trunk build --release  # output in ../../dist/web
```

`Trunk.toml` pins `target = "index.html"`, `dist = "../../dist/web"`, address `127.0.0.1`, port
`8782`, `open = false`. `index.html` declares the Rust entry (`data-wasm-opt="z"`), copies
`manifest.webmanifest`, and provides `<canvas id="craft_canvas">`.

The gate is:

```sh
cargo xtask web
# equivalent to: cargo check -p craft-demo-web --target wasm32-unknown-unknown
```

It checks the wasm **target**, not a served page. `cargo xtask ci` does not run it.

## Entry point

`apps/craft-demo-web/src/main.rs`:

```rust
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub async fn wasm_start() -> Result<(), wasm_bindgen::JsValue> {
    eframe::WebLogger::init(log::LevelFilter::Info).ok();
    let document = web_sys::window().ok_or("no window")?.document().ok_or("no document")?;
    let canvas = document.get_element_by_id("craft_canvas").ok_or("no canvas")?
        .dyn_into::<web_sys::HtmlCanvasElement>()?;
    let options = eframe::WebOptions { renderer: eframe::Renderer::Glow, ..Default::default() };
    eframe::WebRunner::new()
        .start(canvas, options, Box::new(|cc| {
            craft_demo::craft_ui::fonts::setup_bundled_fonts(&cc.egui_ctx);
            let stored = cc.storage.and_then(|s| s.get_string(craft_demo::craft_shell::STORAGE_UI));
            let app = craft_demo::CraftApp::new(cc, stored.as_deref())
                .map_err(std::io::Error::other)?;   // not `?` on the String error
            Ok(Box::new(app))
        }))
        .await
}

#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eprintln!("craft-demo-web only runs in the browser: build it with `trunk build --release`");
}
```

Why this shape:

- `eframe::WebRunner::start` returns a future; the `wasm-bindgen-futures` dependency is what lets
  the async entry point run. Keep `wasm-bindgen-futures = "0.4"` under
  `[target.'cfg(target_arch = "wasm32")'.dependencies]` in `apps/craft-demo-web/Cargo.toml`.
- The app creator's error type is `Box<dyn std::error::Error + Send + Sync>`. `JsValue` does not
  implement that conversion, so map every app-level error (here `CraftApp::new` returns
  `Result<_, String>`) with `std::io::Error::other(e)`. `wasm_start` itself may keep returning
  `JsValue`; `?` on web-sys/JsValue is fine inside it.
- `WebLogger` turns `log` macros into the browser console. Use `log::info!`, not stdout.
- The canvas id must match `index.html`; a missing element is a startup error, not a blank app.

## Browser storage

- eframe persistence works in the browser through `cc.storage` (local storage). The same
  `STORAGE_UI` key and `Shell::save`/`load` path as desktop; no filesystem.
- Persistence is per origin. `http://127.0.0.1:8782` and another port are different stores.
- `craft-settings` has **no store on wasm** (only its trait/document/error types). Do not call
  `SettingsStore::open` from shared code; use eframe storage or a server.
- `craft-atomic`, `dirs`, native config paths and file writes do not exist on wasm. Guard them
  with `#[cfg(not(target_arch = "wasm32"))]`.
- Clear local storage to test first-run behaviour; private windows may not persist.

## Time, threads and platform APIs

- `std::time::SystemTime`/`Instant` can panic or misbehave on `wasm32-unknown-unknown`. The
  workspace already uses `web-time` for `craft_shell::now_ms`; use it instead.
- No OS threads, `std::process`, blocking I/O, native file pickers or `rfd`. Use browser APIs.
- No native window decorations: `custom_titlebar` is false on wasm (default function in
  `UiSettings`), and `craft-window` probing is native-only.
- Keep `unsafe` out; the workspace forbids it.

## Downloads and file picking

- Browser downloads: create an object URL / `<a download>` with `web-sys` (or the shared
  `craft-platform` browser path) and revoke the URL after the click. There is no path to choose.
- Upload/pick: an `<input type="file">` flow or the File System Access API where supported; fall
  back to download/upload. Do not call `rfd`/native dialogs from shared code.
- For a save, trigger the download from a user gesture; browsers block programmatic downloads
  without one.
- Test the actual bytes (filename, MIME type) in a real browser, not only a Rust unit test.

## Verification

1. `cargo xtask web` — type-check the wasm target.
2. `cd apps/craft-demo-web && trunk serve` — open `http://127.0.0.1:8782`.
3. Real browser smoke:
   - canvas renders (not a blank page); console has no panic/wasm error;
   - Home/FileView and the workspace draw; resize the window;
   - theme switch and Preferences work;
   - a command runs from the ribbon/menu/palette; keyboard shortcuts work;
   - reload the page and confirm persisted settings survive (then clear storage and confirm
     defaults);
   - any download produces the expected file.
4. Check the browser console for 404s (manifest, fonts, icons) and wasm MIME errors.
5. For a release candidate: `trunk build --release` and serve `dist/web` with a static server.

## Traps

- **Missing `wasm-bindgen-futures`:** async `wasm_start`/`WebRunner::start` fails to compile or
  never runs. It belongs in the wasm32 target dependencies.
- **`?` on the wrong error:** `JsValue` is not `Send + Sync`; the WebRunner creator cannot box it.
  Map app errors with `std::io::Error::other`.
- **`cargo check` without the target:** native `cargo check` never sees wasm-only code. Use
  `cargo xtask web`; install the target if the gate says it is missing.
- **CI gap:** `cargo xtask ci` passing says nothing about the browser build.
- **Storage surprise:** different port/origin = different data; phone/desktop layouts are not
  tested by the desktop harness.
- **`std::time`:** compiles but panics at runtime on wasm; use `web-time`.
- **Trunk version:** `data-wasm-opt` needs a Trunk/wasm-opt toolchain; a missing wasm-opt can fail
  a release build. `trunk serve` is usually enough for UI checks.
- **Feature unification:** eframe features (`persistence`, `glow`) are workspace-wide; a native
  feature can leak into wasm. Check `cargo tree -e features -p craft-demo-web`.
- **No real browser in CI:** the wasm gate is only a type-check; state that browser smoke was not
  run if you did not run it.

## Do not

- Do not add a second web host or a custom winit loop.
- Do not use `std::fs`, native dialogs, `dirs`, `craft-atomic` or `craft-settings::SettingsStore`
  in shared wasm code.
- Do not put the app creator's fallible logic behind `unwrap`/`expect`; map errors.
- Do not claim the web app works from `cargo xtask web` alone.
- Do not commit `dist/web` output.
