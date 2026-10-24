---
name: craft-bp-ui-edit
description: "Edit UI in the Craft Boilerplate safely: compose only with craft_ui::ui_kit::Container, use craft_theme::Tokens, de-duplicate mutating clicks with once_this_pass, publish scenario handles, and verify floating/docked layouts, both themes and accessibility. Trigger: change a widget, add a dialog/card/toolbar, style a surface, fix a UI bug, add a floating window, touch ui_kit, raw egui host, accesskit, layout mode, theme switch."
---

# Edit the UI layer

Read `docs/ui-kit.md`, `docs/theming.md` and `docs/architecture.md` before changing a surface.
The kit is retained-mode code over egui 0.36; apps supply content, the kit owns presentation.

## Hard boundaries

1. **Outer hosts only through `Container`.** `crates/craft-ui/src/ui_kit/container.rs` is the
   only file allowed to name `Area`, `Window`, `Modal`, `Panel`, `SidePanel`, `TopBottomPanel`,
   `CentralPanel`, `Popup`, `SubMenu` or a `Frame` host constructor. Use:
   `Container::new(ctx, Surface::…)` then `.area(id)`, `.edge(id, Edge::…)`, `.window(title)`,
   `.central()`, `.modal(id)`, `.popup_frame(...)`, `.submenu()`.
   `cargo xtask ui-containers` is source/AST-based; aliases, macros and `.menu_button` shorthands
   are caught too.
2. **Tokens only.** Every colour, size, radius, spacing, font metric and inset comes from
   `craft_theme::Tokens::get()` or the token struct passed to you. No `Color32::from_rgb`,
   `vec2(250.0, …)`, `12.0` spacing literals in widgets. Semantic text uses
   `craft_ui::ui_kit::text`/`Text` variants (`Text::Error`, `Text::Note`).
3. **No domain in the UI crates.** `craft-ui` and `craft-theme` stay generic. App-specific copy,
   commands and model reads stay in the app crate.
4. **No raw eframe/egui window creation in apps.** Build dialogs with `Container::window` /
   `Container::modal` or the kit's `Window`/`Confirm` compositions.

## Adding a new token

1. Add the field to `crates/craft-theme/src/tokens.rs`.
2. Set it in `Tokens::light()` and `Tokens::dark()` in `palettes.rs`.
3. Colour → add to `theme_keys!` in `files.rs`; non-colour → add it to the exhaustive
   destructuring in `every_colour_has_a_key`.
4. Use it through `Tokens::get()`.
5. `cargo test -p craft-theme` must pass (theme-file round trip catches a missing key).

## Clicking and dragging

egui may run several passes for one frame, and every pass sees the same input. A mutating handler
(button click, drag, toggle, menu action) must call
`craft_ui::input::once_this_pass(ctx, "stable-unique-key")` before changing state.
The shell already does this for menus, ribbon commands, panel toggles, canvas navigation and card
gestures; new app surfaces need their own guard. Painting is idempotent and needs no guard.

## Geometry and scenario handles

Publish state that tests and layout code consume:

```rust
craft_ui::scenario::publish_rect("myapp:card", rect);
craft_ui::scenario::publish_target("myapp:button", response.rect);
let at = craft_ui::scenario::handle_at("myapp:button");
```

Use stable prefixes modeled on the existing ones (`appbar:*`, `ribbon:*`, `browser:*`,
`prefs:*`, `home:*`, `menu:*`). The harness reads these after a frame; call `h.settle()` after a
layout change before clicking. Prefer handles over labels when a label prefixes another
(`"Interface"` also prefixes `"Interface language"`).

## Floating and docked

Presentation comes from `LayoutMode`, never from a panel's name:

- `LayoutMode::Floating`: adaptive surfaces are `Container::area` cards on the middle layer,
  offset by `SHELL_GAP` (7 pt), with elevation; they reserve no central space.
- `LayoutMode::Docked`: adaptive surfaces use `Container::edge`, consuming their edge before the
  central panel.

Every UI change that affects a card, ribbon or canvas must be checked in both modes. Use
`docs/testing.md` geometry defaults: app bar 34 pt, browser 250 pt, history 40 pt, 7 pt gaps.

## Accessibility

The harness finds widgets through accesskit (`ctx.enable_accesskit()` is already on). Give icon-only
buttons a label/tooltip from the catalogue; do not ship unlabeled interactive controls. Use
`WidgetText`/`RichText` with semantic roles where the kit offers them. Verify with
`click_widget("<exact label>")` or a published handle.

## Verification

1. `cargo xtask ui-containers` — raw hosts and popup shorthands.
2. `cargo test -p craft-theme` — tokens, contrast, theme round trip.
3. `cargo test -p craft-demo --test ui` — shell scenarios.
4. `cargo run -p craft-demo` — inspect the changed surface in light and dark, floating and docked;
   resize to the minimum (960×600) and check no card overlaps or clips.
5. If the change reaches wasm-only code, `cargo xtask web`.
6. Run the rest of `skill://craft-bp-verify` before reporting done.

## Traps

- **DockArea bypasses Container.** The shared `craft-ui` docking renderer from `craft-libs`
  constructs its own areas/panels. In this repo it is not present; if an app adopts it, document
  the single exception beside the call and keep every other outer host in `Container`. Do not
  copy `DockArea` internals into app code to dodge the gate.
- **Status and toasts repaint.** `Shell::notify`, status chips and toasts request frames; don't
  add several notifications per pass. Deduplicate with `once_this_pass` and avoid notifying while
  merely painting.
- **Harness size.** Default 1600×1000; the desktop demo is 1440×900. A layout that only fits at
  1600 can hide a real clipping bug. Test the smallest supported window too.
- **Menu labels append shortcuts.** `click_widget("Save")` also matches `"Save Ctrl+S"`; use the
  exact label when several rows start with the same word.
- **Animation.** The harness sets `animation_time = 0`; hover/expansion states may differ in the
  real app. Do not rely on animation completion for a test.
- **Conditional compilation.** `#[cfg(target_arch = "wasm32")]` branches are not compiled by the
  native test suite; they still need `cargo xtask web`.

## Do not

- Do not add a raw egui `Window`/`Area` "just for this one dialog".
- Do not add a literal colour/metric to a widget; add a token.
- Do not mutate app state from a draw closure without `once_this_pass`.
- Do not test that a paint call happened; test the visible behaviour/geometry.
- Do not edit `docs/` files outside the area you changed; keep the matching surface doc current.
