---
name: craft-bp-verify
description: "Run and interpret the Craft Boilerplate (craft-app-bp) verification gates: rustfmt, the Container host gate, i18n catalogues, crate layers, clippy, the focused headless demo test, the workspace test suite and the wasm32 web check. Trigger: before claiming a change works, verify my change, run the gates, cargo xtask ci, cargo xtask web, CI is red, pre-push check, check the wasm build, which tests should I run."
---

# Verify a Craft Boilerplate change

The repo root is `craft-app-bp`. Run every command from there unless a step says otherwise.
Report only checks you actually ran. Never turn a skipped check into a pass.

## First: reproduce, then narrow

1. Reproduce the reported failure on its own before running anything broad, for example:
   `cargo test -p craft-demo --test ui -- <test_name>` or `cargo run -p craft-demo`.
2. Inspect the actual diff (`git diff --stat`, then the touched files). Do not "verify" unrelated
   or half-finished work from another agent; stay in your assigned files.
3. If the change touches `apps/craft-demo-web`, anything under `#[cfg(target_arch = "wasm32")]`,
   or a crate the web host pulls in, add the wasm gate in step 7.
4. Only then run the full sequence. A full gate before a repro hides which check is relevant.

## Gate sequence

| # | Gate | Command | What it proves |
|---|---|---|---|
| 1 | Format | `cargo fmt --check` | Rust matches `rustfmt.toml`. |
| 2 | Container | `cargo xtask ui-containers` | No raw egui host outside `crates/craft-ui/src/ui_kit/container.rs`. |
| 3 | i18n | `cargo xtask i18n --check` | `en.json`/`es.json` agree and literal `tr`/`format` ids exist. |
| 4 | Layers | `cargo xtask layers` | No upward or forbidden crate dependencies. |
| 5 | Clippy | `cargo clippy --workspace --all-targets -- -D warnings` | No warnings; production `unwrap`/`expect`/`panic` are denied by workspace lints. |
| 6 | Focused UI | `cargo test -p craft-demo --test ui` | The real shell + demo in the headless harness. |
| 7 | Workspace | `cargo test --workspace` | Unit and integration tests everywhere. |
| 8 | wasm | `cargo xtask web` | `cargo check -p craft-demo-web --target wasm32-unknown-unknown`. |

`cargo xtask ci` runs fmt, containers, i18n, layers, clippy and the workspace tests (1–5 and 7);
it does not run the focused UI test separately, and it deliberately does **not** run the wasm gate.
`ci: ok` alone does not mean the browser build compiles. CI installs `wasm32-unknown-unknown` and
runs both `cargo xtask ci` and `cargo xtask web`.

If the host is missing its Rust target, `cargo xtask web` exits with an install hint. Install with:

```sh
rustup target add wasm32-unknown-unknown
```

## Reading the gates

- **fmt**: `rustfmt.toml` sets `max_width = 150` and `use_small_heuristics = "Max"`. Do not
  reformat by hand to a 120-column style. Run `cargo fmt` only when you intend to write fixes;
  `cargo fmt --check` is the verification form.
- **ui-containers**: parses Rust with `syn`, including aliases, macros and `Frame` shorthands.
  Allowed constructors live only in `crates/craft-ui/src/ui_kit/container.rs`. Use
  `craft_ui::ui_kit::Container` (`area`, `edge`, `window`, `central`, `modal`, `popup`, `submenu`)
  in shell, apps and tests instead of weakening the gate.
- **i18n**: English is the source of truth. `es.json` must have exactly the same keys. The scanner
  only sees literal `tr("…")` and `format("…")` in `crates/**/src/**/*.rs` and
  `apps/**/src/**/*.rs`; ids stored in `CommandSpec`/`PanelSpec`/`TabSpec` are still required in
  both catalogues.
- **layers**: an unregistered workspace crate fails with
  `unknown workspace crate; register it in xtask/src/layers.rs TABLE`. Add new library crates to
  `xtask/src/layers.rs` (`TABLE`) and to the dependency rule tests there; apps are
  `Class::Exempt`.
- **clippy**: the workspace denies `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`
  and `unreachable`. Tests that need them must carry
  `#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]` at the test-module top.
  Do not add a production `#[allow]` to get past this gate.
- **focused UI**: use it while editing shell surfaces; it is faster than the workspace suite and
  fails with the frame-by-frame harness state.
- **workspace test**: includes xtask unit tests (including the i18n and layer rule tests) and
  `craft-theme`/`craft-ui` tests.

## Known gaps and traps

- `cargo xtask ci` has no wasm step; the only wasm proof is `cargo xtask web`.
- `cargo test -p craft-demo --test ui` is the shell scenario suite, not every app test. Tests for
  commands/persistence belong in that file or beside the code.
- A clean clippy does not validate runtime layout. For visible UI changes also run
  `cargo run -p craft-demo`, check both themes and both `LayoutMode` values, and inspect the
  surface you changed.
- CI/profile settings live in the root `Cargo.toml` (`profile.ci`). Do not add a second lint
  configuration to a member crate.
- Concurrent agents may have unrelated modified files. `cargo fmt --check` or clippy failures in
  files outside your assignment are still real failures, but report them as such rather than
  silently formatting or editing them.

## Evidence to report

For each gate: command, result (`pass`/`fail`), and for a failure the first actionable error.
Also state what you could not run (for example "no browser, so no manual wasm smoke test";
"macOS-only quit path not exercised"). Do not claim a browser or macOS check from a Linux
`cargo check`.

## Do not

- Do not run the full gate before reproducing a known failure.
- Do not run `cargo fmt` (write mode) and call that verification; run `cargo fmt --check`.
- Do not rerun a check the user already reported as failing to "confirm" it.
- Do not report `cargo xtask ci` as covering wasm.
- Do not add `#[allow(clippy::...)]` in production code or weaken `xtask` gates to pass.
- Do not commit or push from this workflow; the parent agent owns integration.
