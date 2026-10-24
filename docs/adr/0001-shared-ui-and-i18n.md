# ADR 0001: keep the local UI kit and i18n; adopt craft-libs infrastructure

Status: accepted for the current boilerplate.

## Context

`craft-libs` provides a shared `craft-ui` (icons, buttons, tabs, splits, docking, printing) and a
shared `craft-i18n` (source-text catalogs, plurals, RTL, numbers, system locale). This repository
already has crates with the same package names:

- `crates/craft-ui` owns the `Container` policy, the retained shell kit, the FileView, preferences
  and the icon palette renderer.
- `crates/craft-i18n` owns stable ids (`ui.*`, `cmd.*`), embedded JSON catalogs and the xtask gate.

The shared `craft-ui::docking::DockArea` also constructs `egui::Area`/`Popup` directly, which
bypasses this repository's `Container` gate and surface policy.

## Decision

1. Keep the local `craft-ui`, `craft-theme` and `craft-i18n` as the UI and string boundaries.
2. Adopt only the non-colliding `craft-libs` infrastructure: `craft-quit`, `craft-window`,
   `craft-settings`, `craft-atomic`, `craft-platform` and `craft-docpath`.
3. Do not add a second `craft-ui`/`craft-i18n` package to the workspace.
4. Use the `craft_ui::docking` model (`Layout`, `Action`, `resolve_drop`) only in a future optional
   adapter that renders through `Container`; do not adopt `DockArea` wholesale.
5. Do not migrate the stable id catalog to the source-text `craft-i18n` model without a separate
   migration plan. Adding plurals, RTL, system locale or number formatting may be done as focused
   features on the local crate.

## Consequences

- The shell keeps one surface policy and one i18n gate.
- The shared infrastructure improves close/quit, window frames, persistence and platform services
  without duplicating UI code.
- Docking and richer i18n remain explicit future work; they are not half-adopted.
- A future consolidation into one suite crate is possible, but it must start by renaming or
  aliasing the local packages and moving the `Container` policy, not by adding a duplicate crate.

## Revisit when

- a second app needs customizable docked panels;
- a locale needs plural rules, RTL or regional number formatting;
- the local UI kit and the shared UI components have converged enough to share a package name.
