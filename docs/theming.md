# Theming

## Tokens

`craft_theme::Tokens` is a plain `Copy` struct: spacing, radii, type sizes, form metrics, semantic
chrome colours (app bar, panels, dialogs, fields, selection), the workspace canvas (timeline strip,
viewport gradient, grid, axis triad), the FileView start page (rail, surfaces, borders, primary
tile, dim text, its heights and insets) and seven accent families (`green`, `orange`, `yellow`,
`purple`, `teal`, `pink`, plus the base `accent` blue). `Tokens::light()` and `Tokens::dark()` are
`const fn`s in `palettes.rs`; the values come from SolidCraft's palettes.

Where the workspace, canvas and FileView tokens are used:

- `viewport_top`/`viewport_bottom`: the canvas background mesh (top and bottom vertex colours).
- `grid`/`grid_major`: isometric grid, minor and every fifth line.
- `axis_x`/`axis_y`/`axis_z`: origin triad and axis view circles.
- `timeline`, `timeline_item`, `timeline_marker`: the history card, chips and marker.
- `file_*`: FileView page, rail, nav rows, action tiles, recent list and sample cards;
  `file_action_height`, `file_back_height`, `file_grid_gap`, `file_nav_width`, `file_nav_height`,
  `file_tile_height`, `file_tile_icon`, `file_recent_height`, `file_sample_image_height`,
  `file_content_inset_x`, `file_content_inset_y`, `file_section_gap` and `font_start_title` are
  their metrics.

Thread-locals hold the current theme, colour-vision mode and custom palette, so a widget never
receives them. `craft_theme::apply(ctx)` installs the egui `Visuals` derived from the tokens and
`Tokens::get()` returns the active pair.

Colour-vision modes (`Off`, `Protan`, `Deutan`, `Tritan`) override only the retained colours that
carry meaning: the success badge under red-green. Contrast is covered by
`text_keeps_wcag_contrast_in_every_vision_mode`. The active mode lives in
`UiSettings::color_vision`; `Shell::apply_appearance` calls `theme::set_vision` and
`theme::apply` when it changes, and Preferences › Interface exposes it.

## Icons and caption glyphs

Icon artwork is rendered from `assets/icons/ui/design/svg` through `palette.toml` roles, never from
caller colours: `paint_icon` follows the theme, `paint_app_bar` forces the app-bar ink,
`paint_ink`/`paint_accented` take an ink or a ramp. Window caption glyphs are painted from tokens
(`icons::paint_caption`), not from artwork. Unknown icon ids draw the placeholder.

## Accents

`Accent::ALL` names the seven families; `Accent::colors(&tokens)` returns
`AccentColors { color, ink, soft }`. The active tab's trio is published into `ctx.data` every frame
(`AccentColors::publish` / `current`), so later widgets agree without a second preference.
`AccentColors::on(&tokens)` picks readable text for a solid pill by WCAG contrast.

## Theme files

`craft-theme/1` JSON sets any colour by name on a dark or light base:

```json
{"format": "craft-theme/1", "name": "Mine", "base": "dark",
 "colors": {"panel": "#282828", "accent": "#fb4934"}}
```

`files::parse` checks the format, name, base and every key; `files::resolve` builds the `Tokens`.
`files::export(name, base, &tokens)` writes every key, so it can seed a new theme. Caps: 256 KiB,
512 colours, 80-character names, 64 imported themes.

## Adding a token

1. Add the field to `Tokens` (`tokens.rs`).
2. Give it a value in `Tokens::light()` and, if different, `Tokens::dark()`.
3. If it is a colour, add it to `theme_keys! one: [...]`; if not, add it to the exhaustive
   destructuring in `every_colour_has_a_key` (the build fails until you do).
4. Use it through `Tokens::get()`; never write the literal.
5. `cargo test -p craft-theme` — the theme-file round trip fails if a colour is missing a key.
