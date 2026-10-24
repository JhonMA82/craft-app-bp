---
name: craft-bp-docking
description: "Add panel docking to a Craft app with craft_ui::docking: Layout/Node/Action transactions, stable panel IDs, Permissions, validated persistence, DropTarget previews, and the DockArea exception to the Container rule. Trigger: movable panels, dock panels, split view, floating panel groups, panel layout persistence, DockArea, docking model, tabs reorder, panel permissions, craft-ui docking, customize workspace."
---

# Docking

`craft_ui::docking` (craft-libs) owns panel placement and drag/drop transactions. The application
owns panel contents, command dispatch, document state and persistence policy. The suite design
doc is `docs/ui-docking.md` in `craft-libs`; read it before changing an integration.

Package note: this boilerplate's **local** `crates/craft-ui` does not contain `docking`. The
shared crate is also named package `craft-ui`, so a consuming app aliases it (for example
`craft-ui-libs = { package = "craft-ui", git = "...", rev = "..." }`) and imports
`craft_ui_libs::docking`. See `skill://craft-bp-craft-libs`.

## Model

```rust
use craft_ui::docking::{Action, Floating, Layout, Node, Placement};

let mut layout = Layout {
    root: Some(Node::Tabs { panels: vec!["layers", "properties"], active: 0 }),
    floating: vec![Floating {
        panels: vec!["brushes"], active: 0, rect: [40.0, 80.0, 240.0, 300.0],
    }],
};
layout.apply(Action::Move {
    panel: "brushes", anchor: "layers",
    placement: Placement::Tab { before: Some("properties") },
})?;
```

- `Node::Tabs`: ordered identities plus one active index.
- `Node::Split`: horizontal/vertical children, ratio or fixed sizes.
- `Node::Stack`: accordion entries with open state and optional content height.
- `Floating`: tab groups in one viewport.

Panel IDs must be stable, nonlocalized, `Clone + Eq + Hash` (plus `Debug + Send + Sync + 'static`
to render), and occur at most once in the layout. Keep transient document/sequence tabs separate
from the stable panel they live in.

## Transactions

- Every `Action` validates the original layout, clones/mutates, validates the result, then
  replaces the original. On error the source panel is untouched.
- `DockError` covers missing/self targets, duplicate IDs, invalid active indices, depth/count and
  geometry. Do not ignore it; surface the action failure.
- `apply_with_permissions` protects a panel (`Permissions::PROTECTED`: no move/close/merge, side
  splits around it allowed). An app that requires a permanently docked panel must also check the
  policy before accepting a transaction.
- `OpenAt` adds a currently absent registered panel at a tab/side destination; `Open` reveals a
  group. The shared model cannot know whether an id names a real panel: the app's registry check
  stays app-owned.
- An empty root is valid when all panels float.

## Rendering

- Existing renderers can convert to this model for transactions and keep their own chrome.
- `DropTarget` describes body/tab rectangles and whether a group accepts tabs or splits. Supply
  targets back-to-front, including floating layer order. `resolve_drop` returns the insertion,
  placement and preview rectangle. The frontmost containing group blocks targets underneath even
  when it rejects the drop.
- `DockArea` (optional) renders tab fitting/overflow, drag/drop previews, Escape/focus-loss
  cancellation, split/accordion dividers and floating resize grips. It renders a snapshot and
  returns `DockOutput.actions`; apply them through the app's normal command path.
- `DockStyle` supplies colors, font, spacing and localized action labels; use theme tokens for
  those values. `show_with_limits` adds per-panel min/max; visible geometry clamps to the
  viewport while persisted sizes stay unchanged.
- `begin_drag` carries an existing tab/header gesture into `DockArea`.
- `show_customized` takes a `DockContent<P>` adapter with body, panel-menu, group-header and label
  callbacks. Queue commands in callbacks and apply them after rendering the read-only snapshot.
  `label` shapes visual text; the logical label callback supplies accessibility text; keep them
  consistent. `action_label` localizes stock Float/Move/Close.
- `Action::Activate` is applied by the host, then `DockArea::raise_panel(panel)` on the next frame
  raises the floating layer/focuses the tab without changing the serialized layout.
- `DockOutput::group_drag` / `group_drop` expose group identity/release position for app-owned
  atomic group policies.
- Supply current overlay rectangles through `blocked_rects`; otherwise a modal can be dropped
  through. `DockArea::chrome_has_focus` lets hosts reserve Enter/Space/Tab/Escape/arrows while
  dock chrome has focus.

## The Container rule

Every app-designed outer host still goes through `craft_ui::ui_kit::Container`. Panel bodies,
dialogs and floating tools you author use `Container`/the kit.

`DockArea` is the **single documented exception**: it constructs its own areas/panels as the
shared renderer. Keep it behind the aliased shared crate, document the call site, and do not copy
its raw host code into the app. Any other raw egui `Area`/`Window`/`Panel` fails
`cargo xtask ui-containers`.

## Persistence

- Enable the shared crate's `serde` feature to serialize `Action` and the layout, or keep an
  existing schema through an adapter.
- Validate before committing loaded state: known panel IDs, bounds, duplicate IDs, schema version
  and payload/document size. Bound recursive deserialization in the app; `Layout::validate` cannot
  protect an unbounded parser.
- The serialization feature does not choose a file location or write preferences. Use the app's
  existing shell/settings storage; do not add a second store.
- Hiding a panel: `Layout::location` then close; keep the `Location` with visibility state.
  `restore` uses neighbor IDs and recreated groups. If a neighbor is gone it returns the layout
  unchanged plus `MissingTarget`; choose an explicit fallback.
- Restoring multiple panels closed from the same group in reverse close order preserves neighbors.
- Panel content state (tabs/documents) stays outside transient egui widget memory.

## Writing an adapter (craftcad pattern)

`craftcad`-style apps keep their existing chrome and:
1. convert their layout into `Layout<P>` for transactions,
2. implement `DockContent<P>` (body, menu, group header, label) or use `show_with_limits`,
3. queue ids from callbacks and apply them after `show`/`show_customized`,
4. persist through their own adapter with validation,
5. render panel bodies with `Container`/the app's kit.

Do this rather than copying the docking renderer into the app.

## Verification

- Model: lossless moves, tab reorder, stale targets, atomic errors, protected canvases, accordion
  state, fixed sizing, hidden-panel restoration, bounded malformed layouts, a long move sequence.
- Pointer: drag a floating panel into the tab strip, cancel a drag, redock; assert retained widget
  values and identity.
- Render at two widths, two display scales and both themes. Shared captures:
  `CRAFT_UI_DOCK_VISUAL_DIR=/abs/path cargo test -p craft-ui --test docking_workflows --all-features`
  (inside craft-libs).
- App tests: command path, content, workspace reload, theme and input using your adapter.
- Gates: `cargo xtask ui-containers`, `cargo xtask ci`, `cargo xtask web` if the app is web.

## Traps

- **DockArea bypass:** it is a renderer exception, not a license for other raw hosts.
- **Unstable IDs:** localized titles or indices as IDs break persistence after a rename/reorder.
- **Duplicate panel identity:** rejected; documents belong inside a panel, not as stable panel IDs.
- **Ignoring `DockError`:** a rejected action must leave the old layout; do not half-apply UI state.
- **Persistence without validation:** a stale/oversized layout can panic or hang; validate and cap.
- **Overlay drops:** missing `blocked_rects` lets a drag drop through a modal that appeared mid-drag.
- **Multi-viewport:** floating groups are egui areas in one viewport; native child windows,
  cross-viewport transport and OS ownership remain the host's job.
- **Protected canvas only in the renderer:** also enforce policy in the app command path.
- **Theme literals in `DockStyle`:** use `Tokens`; the renderer is shared, the palette is not.

## Do not

- Do not use `DockArea` in this boilerplate unless the app has adopted the shared crate and the
  exception is documented.
- Do not add a second layout model or parallel drag/drop state.
- Do not persist on every pointer move; apply committed actions and save debounced.
- Do not let the shared model decide app registry/validity or file locations.
