---
name: craft-bp-review
description: "Review a Craft Boilerplate (craft-app-bp) change against repo-specific rules: crate boundaries, Container-only hosts, Tokens, i18n, no production unwrap/expect/panic/todo, serde defaults/sanitize, persistence/keymap, docs and attribution, tests, wasm and accessibility. Trigger: review this PR, code review, check the diff, is this ready, audit a change, quality pass, pre-merge review, find regressions, review my patch."
---

# Repo-specific review

Review the diff, not the whole repository. Read `AGENTS.md`, `docs/architecture.md` and the doc
for the changed area first. Report findings as `severity — file:line — problem — why it matters`,
strongest first. Do not edit unless the task also asks for fixes; do not restate the diff.

## 1. Boundaries and layering

- `crates/craft-theme` and `crates/craft-i18n` are L0: no workspace dependencies (theme may use
  egui colour/shadow value types only).
- `craft-ui` is L1; `craft-shell` L2; `craft-test` is a dev-dependency only; apps/xtask are exempt.
- Domain code must not enter `craft-theme`/`craft-i18n`/`craft-ui`/`craft-shell`. App commands,
  models and business rules stay in the app crate.
- `cargo xtask layers` is the proof. A new workspace crate must be in
  `xtask/src/layers.rs` `TABLE`; flag a gate exemption added to code.
- No path dependencies to a craft-libs checkout; shared crates use one pinned git `rev`.
- `craft-ui` package-name collisions: a second `craft-ui` (git vs local) cannot coexist; aliases
  must be explicit (`package = "craft-ui"`).

## 2. Container and UI composition

- Only `crates/craft-ui/src/ui_kit/container.rs` may construct outer hosts (`Area`, `Window`,
  `Modal`, `Panel`, `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Popup`, `SubMenu`, `Frame`
  hosts). Any other file names them → finding (`cargo xtask ui-containers`).
- Apps must not copy host constructors into a "helper" to dodge the gate; macros/aliases are
  scanned.
- `DockArea` from shared `craft-ui` is the one documented exception, isolated behind the adapter;
  it is not present in this repo and is not a precedent for other raw hosts.
- Adaptive surfaces choose `Container::area` (floating) vs `Container::edge` (docked) from
  `LayoutMode`; presentation must not be inferred from a panel name.
- Reuse kit widgets/compositions instead of a second visual language.

## 3. Tokens and theme

- Every colour/size/radius/spacing in UI code comes from `craft_theme::Tokens`. Literals are a
  finding unless they are genuinely non-visual (e.g. arithmetic constants in geometry tests).
- New token: field in `tokens.rs`, value in **both** palettes, key in `theme_keys!` (colour) or
  exhaustive destructuring (non-colour), test passes.
- Both themes and colour-vision modes still meet the contrast test; no test was weakened.
- Icons use palette roles/aliases, not caller colours; unknown ids silently draw the placeholder,
  so verify a new id exists. `palette.toml`/`aliases.toml` changes need a forced `craft-ui`
  rebuild (build-script watch gap).

## 4. i18n

- No hard-coded user-visible strings; stable ids in `CommandSpec`/`TabSpec`/`PanelSpec`/
  `MenuSpec`/`PageSpec`.
- Every new id exists in **both** `assets/locales/en.json` and `es.json`; no extra Spanish keys.
- Correct lookup: `tr` for fixed UI, `format` for placeholders, `tr_label` for data/generated
  labels, `tr_name` for person names, `tr_dyn` only for genuinely dynamic text.
- Plurals use `_one`/`_other`; placeholders keep identical names across languages; no sentence
  assembly with `format!`.
- A language switch is covered by a test when the change is visible.
- Do not migrate the local `craft-i18n` to a shared crate without a decision/ADR.

## 5. Production lints and error handling

- No `unwrap`, `expect`, `panic`, `todo`, `unimplemented`, `unreachable` in production. Tests may
  use them only with the module-level allow.
- No `#[allow(clippy::unwrap_used)]` etc. added to production to bypass the workspace lint.
- Fallible paths return `Result`/`Option` or a user-visible message; serialization uses the
  existing never-fail fallback (`unwrap_or_default` is acceptable where the contract is "save is
  best effort").
- Async/close paths must not unwrap a failed save.
- `unsafe` is forbidden except the documented hot-reload `#[unsafe(no_mangle)]` export.

## 6. Persistence, keymap and preferences

- `UiSettings`, `Panels` and `SavedShell` keep `#[serde(default)]`; fields are added, not renamed.
  New persisted values are sanitized (`SavedShell::sanitize`) and tested for out-of-range input.
- `Shell::save` must serialize the live keymap (`ui.keymap = keymap.custom.clone()`), and `load`
  must restore both `settings` and `keymap.custom`.
- The Preferences snapshot includes `UiSettings`, `Panels` **and** `keymap.custom`; Apply
  re-baselines, OK keeps, Cancel/Esc/× restores. App-owned pages own their own revert state.
- Shortcuts displayed/executed use `keymap::effective(registry, custom, id)`, not
  `CommandSpec.shortcut`. Reserved keys are only keys the shell actually handles.
- `STORAGE_UI`/custom storage keys match across desktop/web/hot hosts.
- `craft-settings` (if used) is native-only; wasm has no store. Settings save via the debounced
  tick + `request_repaint_after` and flush in `on_exit`.

## 7. Commands and input

- New command: `CommandSpec` + `Workspace` placement (or deliberate omission) + `run_command` arm
  + surface + en/es + test + docs. Registry rejects duplicate ids/shortcuts and non-`cmd.*` labels.
- Shell-owned ids are not handled in the app; app ids are not handled by the shell.
- Mutating click/drag handlers call `once_this_pass`; egui may run multiple passes.
- Disabled state uses `command_enabled`, not a second disabled command.
- Changes that mutate while painting (instead of queueing after) are a finding.

## 8. Close, quit, accessibility

- Close decision in `eframe::App::logic`, not `ui`; `CancelClose` before any quit; `craft_quit`
  last; shell must not send `ViewportCommand::Close` itself while recording requests; deferred
  close waits for async saves; cleanup in `on_exit`.
- Icon-only controls have labels/tooltips; accesskit labels are not ambiguous duplicates; focus
  and keyboard traversal still work.
- No blocking native dialog in a macOS quit callback.

## 9. Tests and docs

- Consumer-visible tests: model change, save/load round trip, Cancel restore, rebind, both themes,
  language switch, geometry. No "a draw call happened" assertions, no copies of implementation.
- Test modules carry the allow only where needed; tests do not write outside temp dirs.
- Docs are part of the deliverable: shell/surface/tokens/handles/settings changes update the
  matching `docs/` file (`building-an-app`, `architecture`, `ui-kit`, `theming`,
  `adding-a-command`, `testing`, `i18n`, `hot-reload`).
- Adapted SolidCraft code has an `ATTRIBUTION.md` row; `NOTICE`/licence files remain truthful.
  Flag claims of generated licence bundles, packaging scripts or tools that do not exist.

## 10. Verification and wasm

- `cargo xtask ci` covers fmt, container, i18n, layers, clippy, workspace tests.
- `cargo xtask ci` does **not** compile wasm. A change to `apps/*-web`, web-only code or a shared
  crate's browser path needs `cargo xtask web` (and ideally a real browser smoke).
- Check `#[cfg(target_arch = "wasm32")]` branches separately; native tests do not compile them.
- Report which checks you ran, and which you did not (e.g. macOS quit, real browser).

## Useful read-only commands

```sh
git diff --stat
git diff
cargo xtask ui-containers
cargo xtask i18n --check
cargo xtask layers
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p craft-demo --test ui
```

Grep for smells rather than reading everything:
`rg "unwrap\(|expect\(|panic!|todo!|unimplemented!|unreachable!" crates apps`,
`rg "egui::(Window|Area|Panel|CentralPanel|Modal|Popup)" crates apps`,
`rg "Color32::|vec2\(|from_rgb" crates`,
`rg "tr_label|tr_dyn|tr_name" <changed files>`.

## Do not

- Do not rewrite the change to your preferred design or add scope while reviewing.
- Do not flag intentional, documented exceptions as violations.
- Do not treat a passing build as proof of behaviour or a passing native test as wasm proof.
- Do not claim a check ran if you only read code.
- Do not report style nits ahead of correctness, boundary or data-loss findings.
