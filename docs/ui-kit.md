# The ui-kit

`craft_ui::ui_kit` is a retained-mode kit over egui. Geometry, colour and type come from
`craft_theme::Tokens`.

## Modules

| Module | Contents |
|---|---|
| `container` | `Container`, `Surface`, `Edge`, `DockEdges` — the only outer-host factory |
| `metrics` | `row_height`, `header_height`, `dimension`, `ContentWidth` |
| `layout` | `row`, `sized_row`, `label`, `section`, `list_row`, `divider`, `text` (`Text::Error`, `Text::Note`, …) |
| `actions` | `Button`, `footer`, `icon_button_at`, `command_menu_button` |
| `checks` | `checkbox`, `checkbox_indeterminate`, check glyph painter |
| `choice` | `segmented`, `Segments`, `options`, `color_swatch`, `swatch_grid` |
| `fields` | `field`, `text_field`, `cell_editor`, `dropdown`, `slider`, `text_area`, `search_field` |
| `disclosure`, `table`, `status`, `menu`, `panel`, `confirm` | their egui hosts and rows |
| `search` | `CommandSearch`, `SearchFieldGeometry`, `SearchBadge`, `MenuItem::search_row` |
| `shortcuts`, `reports`, `color`, `radial`, `preferences`, `preferences_small` | grooves, colour popover, radial menu, settings compositions |
| `file_view` | `FileView` (`page`, `content`, `title`, `subtitle`, `section`, `grid`, `rail`, `navigation`, `nav_row`, `back_to`, `action_tile`, `recent_list`, `recent_row`, `sample_grid`), `RecentRow`, `SampleCard` |
| `history` | `HistoryTransport` (`ALL`, `id`, `show`), `HistoryChip`, `history_marker`, `panel_grip`, `history_transport_width`, `TREE_ROW_HEIGHT` |

`panel::Window` is the shared dialog/modal window: title band, cards, footer actions, escape policy.
`Confirm` is the one confirmation dialog. Preferences uses `Window` with `show_with_left` (a painted
page rail, then `settings_scroll` per page), `preferences::theme_rail`/`theme_tile`/`ThemeSwatch`
for theme choices, and `settings_form` + `row` for labelled control rows.

Outside `ui_kit`:

- `craft_ui::canvas` — the workspace canvas (gradient, grid, triad, axis view, navigation bar,
  middle-drag pan). Domain-free; the shell handles its `CanvasAction`s.
- `craft_ui::titlebar::Caption` + `icons::paint_caption` — the window-caption vocabulary.
- `craft_ui::input::once_this_pass` — per-pass de-duplication for mutating click/drag handlers
  (egui may run several passes of one frame).

## Rules

- Compose with the kit, never with raw egui hosts. `Container` is the only factory (the gate checks
  this).
- Pass i18n ids (`ui.close`, `demo.name`) to labels the kit translates with `tr_label`; use `tr("…")`
  for fixed strings. `MenuItem::search_row` paints its label raw: pass `tr(id)` there.
- Mutating click handlers call `input::once_this_pass(ctx, key)` before changing state.
- No literal colours or metrics in widgets.
- Use `ui_kit::text`'s variants for semantic colours: `Text::Error` paints `form_error_fill`.

## Example

```rust
use craft_ui::ui_kit::{self, Button, Text};

ui_kit::settings_heading(ui, craft_ui::i18n::tr("prefs.interface"), "");
ui_kit::settings_section(ui, "theme", craft_ui::i18n::tr("prefs.theme"), |ui| {
    ui_kit::checkbox(ui, craft_ui::i18n::tr("ui.enabled"), &mut enabled);
});
if ui_kit::button(ui, craft_ui::i18n::tr("ui.apply"), Button::Primary).clicked() {
    ui_kit::text(ui, "applied", Text::Note("status"));
}
```
