---
name: craft-bp-theme-icons
description: "Change theme tokens, palettes, theme files or SVG icons in the Craft Boilerplate: add Tokens through palettes and theme_keys!, keep WCAG contrast and colour-vision overrides, draw icons from the 64-unit SVG masters through palette.toml/aliases.toml, and avoid the build-script watch gap. Trigger: change a colour, add a token, new palette entry, light/dark contrast, colour vision, theme file, add an icon, icon looks wrong, palette.toml, aliases.toml, SVG icon, placeholder icon, theming."
---

# Theme and icons

Two foundations:

- `crates/craft-theme`: `Tokens`, light/dark palettes, colour-vision modes, theme-file parse/export,
  WCAG contrast. No domain, no widgets.
- `crates/craft-ui`: fonts, SVG icon pipeline, caption glyphs. Icons are palette-owned artwork,
  never caller-coloured.

Read `docs/theming.md` first.

## Tokens

Every colour, size, radius, spacing and type metric used by UI code comes from
`craft_theme::Tokens::get()` (or a `Tokens` value already in scope). There are no literal colours
or metrics in widgets.

Add a token:

1. Field in `crates/craft-theme/src/tokens.rs`.
2. Value in `Tokens::light()` and `Tokens::dark()` in `crates/craft-theme/src/palettes.rs`.
3. Colour → add the key to `theme_keys!` in `crates/craft-theme/src/files.rs`; non-colour → add it
   to the exhaustive destructuring in `every_colour_has_a_key` (compile error until you do).
4. Use it through `Tokens::get()`.
5. `cargo test -p craft-theme`.

Never rename an existing public token field without updating every caller and the theme-file keys;
exported theme files reference key names.

## Theme files

Format `craft-theme/1`:

```json
{"format": "craft-theme/1", "name": "Mine", "base": "dark",
 "colors": {"panel": "#282828", "accent": "#fb4934"}}
```

`files::parse` validates format/name/base/every key; `files::resolve` builds `Tokens`;
`files::export` writes every key. Caps: 256 KiB, 512 colours, 80-character names, 64 imported
themes. A new colour without a key means the round-trip test fails; fix the macro, not the test.

## Colour vision and contrast

- Modes: `Off`, `Protan`, `Deutan`, `Tritan`; `craft_theme::set_vision` + `theme::apply` are driven
  by `UiSettings::color_vision` in `Shell::apply_appearance`.
- Only retained meaningful colours are overridden; the canvas/artwork palette stays as drawn.
- `text_keeps_wcag_contrast_in_every_vision_mode` and `contrast_ratio` are the contract. If you
  add a semantic ink/surface pair, add/adjust the test rather than eyeballing it.
- `AccentColors::on(&tokens)` picks readable text on a solid accent; use it for pills, not a
  hard-coded white/black.
- Check both themes (and ideally all four vision modes) for any new surface.

## Icons

Masters: `assets/icons/ui/design/svg/<id>.svg` (flat directory). Requirements:

- 64-unit grid, consistent stroke/geometry, `currentColor` or one of the stable source hex values
  from `palette.toml`.
- Colours are **roles**, not final display values: the build maps role → light/dark (and accent
  ramps) and the renderer resolves the active theme.
- Add display colours for a new role in `assets/icons/ui/design/palette.toml` (`[roles.<name>]`
  with `source`, `light`, `dark`). Accent ramps live under `[accents.*]`; default icon accents
  under `[icon-accents]`.
- Aliases go in `assets/icons/ui/design/aliases.toml` (`[alias] name = "master"` or a table with
  `master` + `accent`). An alias to a missing master fails the build.
- Use the id in `CommandSpec.icon` or `icons::paint_*`. An unknown id silently draws the
  placeholder, so a "wrong icon" often means a typo, not a renderer bug.
- `paint_icon` follows the theme; `paint_app_bar` forces app-bar ink; `paint_ink`/`paint_accented`
  take an explicit ink/ramp. Do not pass a caller colour to palette-owned artwork.
- Window caption glyphs use `icons::paint_caption`, not SVG masters.

**Build-script watch gap:** `crates/craft-ui/build.rs` prints `rerun-if-changed` for `build.rs`,
the `svg/` directory and each SVG file, but **not** for `palette.toml` or `aliases.toml`. After
editing those two, force a rebuild (touch `build.rs` or `cargo clean -p craft-ui`) or the generated
`icon_data.rs` keeps the old colours/aliases.

## Verification

1. `cargo test -p craft-theme` — contrast, accent round trip, theme-file round trip.
2. `cargo test -p craft-ui` — icon/parser/kit tests.
3. `cargo test -p craft-demo --test ui` — surfaces using the token/icon.
4. `cargo run -p craft-demo` — switch light/dark (Preferences › Interface or the theme command),
   cycle colour vision, open the Preferences theme tiles, and inspect the icon at both sizes.
5. `cargo xtask ui-containers` if the change touched layout composition.
6. If the renderer changed, `cargo xtask web` (tiny-skia/icon code compiles for wasm).

## Traps

- **Literal drift:** a new widget using `Color32::WHITE` or `vec2(12.0, 12.0)` compiles and looks
  fine in one theme, then fails in the other. Use tokens.
- **Palette watch:** editing `palette.toml` without forcing the build shows old icon colours.
- **Unknown icon:** placeholder is silent by design; check the file stem and aliases.
- **Alias cycles:** aliases must point at masters, not other aliases; the build validates.
- **Accent mismatch:** an icon's default accent and the tab's accent can differ conceptually. Pass
  an explicit ramp only when the surface owns the accent (e.g. the active tab).
- **Theme file keys:** adding a field to `Tokens` but not `theme_keys!` fails the round trip only
  if it is a colour; non-colours are caught by the exhaustive destructuring.
- **Both palettes:** adding only to `light()` leaves dark at the old/zero value; compile may still
  pass for non-colours. Set both.
- **Contrast:** a pretty colour can fail WCAG for the ink it is paired with; run the test.

## Do not

- Do not add a second colour module, palette constant or shadow style outside `craft-theme`.
- Do not let apps pass arbitrary colours into icon painters; fix the role/palette instead.
- Do not edit generated `icon_data.rs` in `target/`; edit the masters/tables.
- Do not claim a theme works from a screenshot in one mode; check both themes and the vision
  modes affected.
- Do not remove the contrast/round-trip tests to land a colour.
