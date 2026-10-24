---
name: craft-bp-craft-libs
description: "Consume the shared craft-libs crates from a Craft Boilerplate app: pin one git revision for every craft-* dependency, alias colliding craft-ui/craft-i18n packages, gate features by target, avoid path dependencies and domain crates, and keep licence/attribution files current. Trigger: add craft-settings, add craft-quit, add craft-window, upgrade craft-libs, git dependency, package name collision, cargo patch, workspace dependency, add shared crate, NOTICE attribution."
---

# Using shared craft-libs

Read `docs/craft-libs.md` and `docs/adr/0001-shared-ui-and-i18n.md` first. Shared infrastructure
lives in <https://github.com/storytold/craft-libs> and is consumed as git dependencies. Current
consumers in this repo: `craft-atomic`, `craft-settings`, `craft-quit`
(workspace feature `egui`; the demo adds `menu` on desktop only), `craft-window`,
`craft-platform` (workspace features `dialogs`, `egui`; native adds `appearance`; wasm adds
`browser`) and `craft-docpath`.
All are declared in the root `Cargo.toml` under `[workspace.dependencies]`; `craft-shell`
re-exports `craft-quit`, `craft-platform` and `craft-docpath`.

## Pin one revision

Every craft-libs crate must use the **same** `rev`:

```toml
[workspace.dependencies]
craft-atomic = { git = "https://github.com/storytold/craft-libs", rev = "<40-hex-rev>" }
craft-settings = { git = "https://github.com/storytold/craft-libs", rev = "<40-hex-rev>" }
craft-quit = { git = "https://github.com/storytold/craft-libs", rev = "<40-hex-rev>", features = ["egui"] }
# The desktop app adds "menu" only in its native target dependency:
# craft-quit = { workspace = true, features = ["menu"] }
```

Rules:

- Use `rev`, not a branch. The repo has no tags to rely on. A branch silently moves every build.
- Change the revision in all entries together, then regenerate `Cargo.lock` and commit it.
- Members depend with `crate.workspace = true`; do not repeat the git URL/rev in member manifests.
- Pin the same rev as the app suite (or the current root manifest) unless the task explicitly
  upgrades; a mixed rev yields duplicate types across crate boundaries.
- Verify with `cargo metadata --format-version 1 | jq` or `cargo tree -i craft-settings`; the
  source must be a single `git+…?rev=…#…`.

## Package-name collisions

`craft-libs` contains a crate whose package name is also **`craft-ui`**, and this repo has a local
`crates/craft-ui`. A direct dependency on both is ambiguous in code and Cargo needs an explicit
alias; this workspace does not add the shared one. Similarly, a shared `craft-i18n` would collide
with the local `crates/craft-i18n`.

- Do not add a second `craft-ui` dependency to this workspace.
- In a consuming app that needs the libraries' `craft-ui` (docking/controls) instead of the local
  one, alias it in the manifest and import by the alias:
  `craft-ui-libs = { package = "craft-ui", git = "...", rev = "..." }`.
- If both are genuinely needed in one workspace, the local crate must be renamed (and all uses
  updated) or the shared one aliased everywhere; there is no automatic disambiguation.
- Alias `craft-i18n` the same way (`craft-i18n-libs`) if the shared catalogue crate is adopted.
- The alias changes the Rust crate name (`craft_ui_libs`), not the package name in `Cargo.lock`.

## Feature and target gates

- `craft-quit`: `egui` enables `on_close_request`; `menu` adds the macOS app menu (pulls `muda`).
  On non-macOS these are effectively no-ops but the features still compile.
- `craft-platform`: `default-features = false` plus only what the app uses (`dialogs`, `egui`);
  native dialogs and browser downloads are target-specific.
- `craft-settings`: no store/filesystem on `wasm32`; only the trait, document helpers and errors
  exist. Guard native-only calls with `#[cfg(not(target_arch = "wasm32"))]`.
- `craft-window`: window decorations/hit-testing are native; keep the web host free of it.
- `craft-atomic`: filesystem replacement; not for wasm.
- eframe features are workspace-wide; adding a feature is fine, removing one can break a host.

Check both target surfaces after any dependency change:

```sh
cargo check --workspace
cargo check -p craft-demo-web --target wasm32-unknown-unknown   # or cargo xtask web
```

## No path dependencies

Do not point at a local checkout:

```toml
# Wrong for committed manifests
craft-settings = { path = "../craft-libs/crates/settings" }
```

Path dependencies make CI and other machines depend on a sibling directory. Use the pinned git
rev. If you need a local patch while developing, use a temporary `[patch]` locally, never commit
it.

## Patches are not transitive

A `[patch]` in an app workspace does not apply to another workspace that consumes the app. If a
dependency must be patched for everyone, the fix belongs in `craft-libs` at the pinned rev (or a
new rev), not in a leaf manifest. Document any unavoidable patch in the root `Cargo.toml` comment
and remove it when the upstream rev includes the fix.

## Domains stay out

The boilerplate must not pull CAD/imaging/printing domain crates (`craft-cms`, `craft-print`,
`craft-pdf`, `craft-ai-core`) into `craft-theme`, `craft-i18n`, `craft-ui`, `craft-shell` or the
demo. Apps may opt into a shared crate only when the feature needs it, and the layer gate must
still pass. `craft-ui`/`craft-theme` stay domain-free.

## Licences and attribution

- craft-libs is MIT OR Apache-2.0. Keep `NOTICE`/`ATTRIBUTION.md` truthful when adding shared code
  that is copied rather than depended on.
- Dependencies keep their own licences; `Cargo.lock` records the versions. There is no generated
  third-party licence bundle in this repo, so do not claim one.
- If you copy source out of craft-libs, add the source path and changes to `ATTRIBUTION.md`; a
  version bump alone does not require a new row.

## Traps

- **No tags:** `tag = "v1"` will fail or drift; use the full commit `rev`.
- **Checkout mismatch:** a cached `~/.cargo/git/checkouts/craft-libs-*` may contain several revs.
  `cargo tree`/`Cargo.lock` is the source of truth, not the newest directory timestamp.
- **Feature unification:** a dev-dependency can enable an extra feature for normal builds (e.g.
  serde) and hide a target-specific compile error. Check `cargo tree -e features`.
- **Package collision:** adding `craft-ui` from git while `crates/craft-ui` exists fails with
  "two packages named `craft-ui`"; alias or rename.
- **DockArea:** the shared `craft-ui` renderer constructs its own egui areas/panels, so it is the
  only documented bypass of `Container`. Isolate it behind the aliased crate, document the call
  site and keep it out of the local `crates/craft-ui`; do not treat it as a pattern for app code.
- **Patch leakage:** `[patch."https://github.com/storytold/craft-libs"]` affects every craft-libs
  dependency in that workspace; keep it minimal and temporary.
- **Offline CI:** git dependencies require network on a cold cache. Do not add one for a trivial
  helper; add local code instead.
- **Rev upgrade:** read the upstream changelog/docs and run the full gate, including wasm. A new
  crate may be native-only.

## Verification

```sh
cargo metadata --format-version 1 >/dev/null
cargo tree -i craft-settings
cargo check --workspace
cargo xtask layers
cargo xtask ci
cargo xtask web
```

Also confirm `Cargo.lock` changed intentionally (one rev, not several) and that no member manifest
contains a `path` or a second `rev`.

## Do not

- Do not vendor craft-libs into this repo unless a task explicitly asks.
- Do not add a path dependency, a branch, or two revisions.
- Do not put shared UI/domain code into `craft-shell`; extend craft-libs instead.
- Do not add a licence bundle or a `cargo xtask licences` step; neither exists here.
