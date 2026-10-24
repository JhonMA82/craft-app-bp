# Attribution

Craft Boilerplate adapts UI code from **SolidCraft**
(<https://github.com/tangency-labs/solidcraft>), commit
`0db2abbeca4ae5c82d8d560dafd74feccb3bfbe4`, Copyright (c) 2026 SolidCraft contributors, licensed
under MIT OR Apache-2.0. The original licence texts are `LICENSE-MIT` and `LICENSE-APACHE`.

Every adapted file is listed below with what changed. Files or directories not listed are original
to this project.

| Source (SolidCraft) | Destination | Changes |
|---|---|---|
| `crates/ui-egui/src/theme.rs` | `crates/craft-theme/src/lib.rs`, `src/tokens.rs` | Kept the thread-local theme/vision state, `set_custom`, WCAG `contrast_ratio`, `Visuals` and `apply`; `TabAccent` became `AccentColors` with the seven-family `Accent`; CAD tokens and viewport/marker colours removed |
| `assets/styles/ui/palettes.rs` | `crates/craft-theme/src/palettes.rs` | Light and dark values unchanged for the retained tokens; CAD families renamed to `green`/`orange`/`yellow`/`purple`/`teal`/`pink`; the timeline, viewport, grid, axis and FileView colours and metrics use SolidCraft's values; only retained colours take part in colour-vision overrides |
| `crates/ui-egui/src/theme_files.rs` | `crates/craft-theme/src/files.rs` | `craft-theme/1` format, single-colour keys, validation and hostile-input caps kept; CAD keys and built-in theme files removed; `add` kept |
| `crates/ui-egui/src/i18n.rs` | `crates/craft-i18n/src/lib.rs` | Catalogues, locale scoping, interpolation and label lookup kept; CAD default-name translation and the engine adapter removed |
| `crates/ui-egui/src/fonts.rs` | `crates/craft-ui/src/fonts.rs` | Bundled IBM Plex weights, `RichTextExt`, symbol fallback and install kept; system-font scanning, CJK/Hangul/Thai fallbacks and user overrides removed |
| `crates/ui-egui/src/icons.rs` | `crates/craft-ui/src/icons.rs` | Texture cache and paint functions kept; glyph/marker/navigation/pointer sources and `audit` removed; danger artwork uses the pink accent; `paint_caption` ports the caption glyph from `assets/icons/ui/source/glyphs.rs` |
| `crates/ui-egui/src/icon_svg.rs` | `crates/craft-ui/src/icon_svg.rs` | Parser, renderer, palette roles and aliases kept; audit collection, CAD colour helpers and the SolidCraft artwork tests removed |
| `crates/ui-egui/build.rs` (`icon_data`) | `crates/craft-ui/build.rs` | Only icon data generation; git stamping, scenario tests and CAD accent constants removed; the master directory is flat |
| `crates/ui-egui/src/scenario.rs` | `crates/craft-ui/src/scenario.rs` | Handle/rect/count publication and lookup kept; the scenario runner and CAD checks removed |
| `crates/ui-egui/src/frame.rs` | `crates/craft-ui/src/frame.rs` | `Side`, dock offsets, anchors and `drag_dock` kept as-is; CAD test removed |
| `crates/ui-egui/src/prefs.rs` (`parse_number`) | `crates/craft-ui/src/parse.rs` | Exact function, moved out of the preferences module |
| `crates/ui-egui/src/shell.rs` (`LayoutMode`, `SHELL_GAP`) | `crates/craft-ui/src/layout.rs` | Same publish/current mechanism; `Floating` is the default and `Docked` the alternative, as in SolidCraft |
| `crates/ui-egui/src/shell.rs` (`Cards`, `paint_top_corners`) | `crates/craft-shell/src/geometry.rs` | Exact card geometry and the concave top-corner mesh; `WORKSPACE_WIDTH` comes from the ribbon |
| `crates/ui-egui/src/viewport.rs` | `crates/craft-ui/src/canvas.rs` | The gradient, isometric grid, origin triad, axis view, navigation bar geometry and middle-button pan; the 3D scene, picking, camera and wheel zoom are not ported |
| `crates/ui-egui/src/ui_kit/file_view.rs` | `crates/craft-ui/src/ui_kit/file_view.rs` | Page, content, title, sections, grid, recent list, rail, navigation, action tiles and sample cards kept; the recovery banner and review were removed |
| `crates/ui-egui/src/ui_kit/*` | `crates/craft-ui/src/ui_kit/*` | Retained modules made public; container is the single outer-host factory; `search`, `fields`, `color`, `choice`, `actions`, `confirm`, `menu` adapted to the token set and i18n ids; CAD-only modules (`assistant*`, `gallery`, `preferences_*` figures/navigation) not ported; `preferences.rs` was rewritten to the token/theme/layout compositions |
| `crates/ui-egui/src/ui_kit/history.rs` | `crates/craft-ui/src/ui_kit/history.rs` | Transport controls, the six-dot grip, the chip and the rollback marker kept; engine feature classification, error/warning badges and named-row styles removed |
| `crates/ui-egui/src/prefs.rs`, `prefs/general.rs`, `prefs/interface.rs` | `crates/craft-shell/src/prefs_window.rs` | The preferences dialog's painted page rail, OK/Apply/Cancel footer with Restore defaults, labelled language row, theme tiles, layout rows, scale specimen and colour-vision switch; CAD pages, controls, theme import and font files removed |
| `crates/ui-egui/src/keymap.rs` | `crates/craft-shell/src/keymap.rs` | Rebinding, capture, conflicts and reset kept; command lookup now uses `CommandRegistry`; shell-reserved keys reduced; page rebuilt on the kit |
| `crates/ui-egui/src/messages.rs` | `crates/craft-shell/src/messages.rs` | Message ring, dedupe and window kept; `SolveApp` coupling replaced by plain state |
| `crates/ui-egui/src/toolbar.rs` | `crates/craft-shell/src/ribbon.rs`, `src/app_bar.rs` | `toolbar`, `ribbon_tabs`, `app_bar`, `file_menu` and the label/button measuring helpers ported with their constants and painting order; `fit` kept pure; command lookup is `CommandRegistry`/`command_enabled`; CAD families, pinning, drag customisation and the report-a-bug pill removed |
| `crates/ui-egui/src/shortcut_box.rs` (`score`, command matching, `show`) | `crates/craft-shell/src/commands.rs`, `src/palette.rs` | Fuzzy scoring ported; `show` becomes the ribbon search: the expanding field, `CommandSearch` sheet, keyboard navigation and footer; ranking runs over `CommandRegistry`; typed command syntax, design rows and context menus removed |
| `crates/ui-egui/src/browser.rs` (`browser`, `browser_header`) | `crates/craft-shell/src/panels.rs` | Floating card and docked column geometry, header band, fold chevron and resize rail; the CAD tree, occurrence filtering and drag actions removed |
| `crates/ui-egui/src/timeline.rs` (`timeline`, `timeline_contents`) | `crates/craft-shell/src/panels.rs` | The 40 pt strip, grip, title, transport and chip layout; feature kinds, rollback editing and context menus removed |
| `crates/ui-egui/src/viewport.rs` (`paint_toast`) | `crates/craft-shell/src/status.rs` | The transient chip's geometry, dot/spinner and six-second life; a permanent status line is not drawn |
| `crates/ui-egui/src/titlebar.rs` | `crates/craft-shell/src/titlebar.rs`, `crates/craft-ui/src/titlebar.rs` | Window drag, caption buttons (46 pt each, full bar height), `resize_border_px`, `corner_size`, `zones` and `resize_zones` kept; the `Caption` enum moved to `craft-ui` with the painted glyph; Windows native hit-test and DPI helpers not ported |
| `xtask/src/ui_containers.rs` | `xtask/src/ui_containers.rs` | Unchanged scanner; the exemption path is `crates/craft-ui/src/ui_kit/container.rs` |
| `xtask/src/layers.rs` | `xtask/src/layers.rs` | Rule engine kept; layer table and UI foundations adapted to this workspace |
| `assets/icons/ui/design/palette.toml` | `assets/icons/ui/design/palette.toml` | Colour roles kept; CAD accents replaced by blue/green/orange/yellow/purple/teal/pink |
| `assets/fonts/ibm-plex-sans/`, `assets/fonts/ibm-plex-mono/` | `assets/fonts/` | IBM Plex Sans Variable and Plex Mono Regular with their SIL OFL 1.1 licence files, copied unchanged |

## Assets

- IBM Plex Sans and IBM Plex Mono: SIL Open Font License 1.1 (`assets/fonts/*/OFL.txt`).
- Most icon masters under `assets/icons/ui/design/svg` are drawn for this project on the 64-unit
  grid the renderer uses. `ws_design`, `box`, `help`, `pan`, `fit` and `chevron_right` are copied
  unchanged from SolidCraft's masters (MIT OR Apache-2.0, same provenance as the code above).
