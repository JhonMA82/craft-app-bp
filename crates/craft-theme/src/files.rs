//! Colour themes as files: a `craft-theme/1` JSON file names a base palette (the built-in dark or
//! light) and sets any of its [`Tokens`] colours by name, so a theme can be a handful of lines.
//! Colour vision modes still apply on top of a theme ([`Tokens::themed`]).
//!
//! ```json
//! {"format": "craft-theme/1", "name": "Mine", "base": "dark",
//!  "colors": {"panel": "#282828", "accent": "#fb4934"}}
//! ```
//!
//! Adapted from SolidCraft (`crates/ui-egui/src/theme_files.rs`), MIT OR Apache-2.0.
//! See ATTRIBUTION.md.

use std::collections::BTreeMap;

use egui::Color32;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::tokens::Tokens;

/// The `format` tag of a theme file.
pub const FORMAT: &str = "craft-theme/1";
/// Hostile-input caps: file size, colours per file, name length, imported themes kept.
const MAX_BYTES: usize = 256 * 1024;
pub const MAX_COLORS: usize = 512;
const MAX_NAME: usize = 80;
pub const MAX_CUSTOM: usize = 64;

/// The palette a theme starts from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Base {
    #[default]
    Dark,
    Light,
}

impl Base {
    pub fn is_dark(self) -> bool {
        self == Base::Dark
    }
}

/// A theme: its id (a stable slug of the name), name, base and colour overrides, each a
/// `#rrggbb` / `#rrggbbaa` string keyed by [`keys`].
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ThemeFile {
    pub id: String,
    pub name: String,
    pub base: Base,
    pub colors: BTreeMap<String, Value>,
}

/// The settable colours: a key, and where it lives in [`Tokens`]. The destructuring below fails
/// to build when a colour token is added without a key here, so every colour stays themeable.
macro_rules! theme_keys {
    (one: [$($one:ident),* $(,)?] $(,)?) => {
        /// Every key a theme file may set, in the order an export writes them.
        pub fn keys() -> Vec<&'static str> {
            vec![$(stringify!($one),)*]
        }

        fn slot<'a>(t: &'a mut Tokens, key: &str) -> Option<&'a mut [Color32]> {
            match key {
                $(stringify!($one) => Some(std::slice::from_mut(&mut t.$one)),)*
                _ => None,
            }
        }

        #[allow(dead_code)]
        fn every_colour_has_a_key(t: &Tokens) {
            // Not colours: spacing, radii, type sizes and form metrics.
            let Tokens {
                $($one: _,)*
                space_2: _, space_3: _, space_4: _, space_6: _, space_8: _, space_10: _,
                space_12: _, space_14: _, space_16: _, space_24: _,
                radius_sm: _, radius_md: _, radius_lg: _, radius_xl: _,
                font_badge: _, font_micro: _, font_panel: _, font_small: _, font_caption: _,
                font_body: _, font_label: _, font_button: _, font_heading: _, font_subtitle: _,
                font_intro: _, font_title: _, font_section_tracking: _,
                control_height: _, control_radius: _, form_label_width: _, form_gap_y: _,
                form_footer_height: _, form_header_height: _, form_card_width: _, form_icon_size: _,
                form_footer_icon_size: _, form_close_size: _, form_inset: _, form_icon_inset: _,
                form_section_icon_size: _, form_checkbox_height: _, form_button_height: _,
                mode_size: _, mode_radius: _, mode_tint: _,
                file_action_height: _, file_back_height: _, file_grid_gap: _, file_nav_width: _,
                file_nav_height: _, file_tile_height: _, file_tile_icon: _, file_recent_height: _,
                file_sample_image_height: _, file_content_inset_x: _, file_content_inset_y: _,
                file_section_gap: _, font_start_title: _,
            } = t;
        }
    };
}

theme_keys! {
    one: [
        app_bar, app_bar_text, app_bar_separator, app_bar_hover, app_bar_hover_subtle,
        app_bar_hover_button, app_bar_hover_strong, app_bar_tab_active,
        toolbar, panel, panel_header, border, text, text_dim, icon, hover,
        dialog_body, dialog_bg, dialog_footer, dialog_rule, dialog_border, dialog_label,
        control_fill, control_border, field, selected_bg, selected_ink,
        accent, accent_soft, on_accent, tab_on_fill_dark,
        form_ink, form_dim, form_primary, form_error, form_error_fill, form_success, form_success_fill,
        error, warning, chip, section_cap, overlay,
        timeline, timeline_item, timeline_marker,
        viewport_top, viewport_bottom, grid, grid_major, axis_x, axis_y, axis_z,
        file_background, file_surface, file_border, file_image_tint,
        file_rail, file_rail_ink, file_rail_hover,
        file_primary, file_primary_ink, file_primary_border, file_dim,
        green, green_ink, green_soft, orange, orange_ink, orange_soft,
        yellow, yellow_ink, yellow_soft, purple, purple_ink, purple_soft,
        teal, teal_ink, teal_soft, pink, pink_ink, pink_soft,
    ],
}

/// `#rrggbb` or `#rrggbbaa` (alpha not premultiplied).
fn parse_color(s: &str) -> Option<Color32> {
    let hex = s.trim().strip_prefix('#')?;
    if !(hex.len() == 6 || hex.len() == 8) || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let byte = |i: usize| hex.get(i..i + 2).and_then(|h| u8::from_str_radix(h, 16).ok());
    let a = if hex.len() == 8 { byte(6)? } else { 255 };
    Some(Color32::from_rgba_unmultiplied(byte(0)?, byte(2)?, byte(4)?, a))
}

fn format_color(c: Color32) -> String {
    let [r, g, b, a] = c.to_srgba_unmultiplied();
    if a == 255 { format!("#{r:02x}{g:02x}{b:02x}") } else { format!("#{r:02x}{g:02x}{b:02x}{a:02x}") }
}

/// The colours a value gives for a slot of `len` colours, or why not.
fn colors_of(v: &Value, len: usize) -> Result<Color32, String> {
    if len != 1 {
        return Err("is not a single colour token".into());
    }
    v.as_str().and_then(parse_color).ok_or_else(|| format!("`{v}` is not a colour such as \"#282828\""))
}

/// A theme file's text, checked: the format tag, a name, a base and only known keys with valid
/// colours. Nothing is taken from a file with any error; the message names every bad key.
pub fn parse(text: &str) -> Result<ThemeFile, String> {
    if text.len() > MAX_BYTES {
        return Err(format!("a theme file is at most {} KiB", MAX_BYTES / 1024));
    }
    let v: Value = serde_json::from_str(text).map_err(|e| format!("not a theme file: {e}"))?;
    from_value(&v)
}

/// [`parse`] for JSON already read.
pub fn from_value(v: &Value) -> Result<ThemeFile, String> {
    if v.get("format").and_then(Value::as_str) != Some(FORMAT) {
        return Err(format!("not a theme file: `format` must be \"{FORMAT}\""));
    }
    let name = v.get("name").and_then(Value::as_str).map(str::trim).unwrap_or_default();
    if name.is_empty() || name.chars().count() > MAX_NAME {
        return Err(format!("`name` must be 1 to {MAX_NAME} characters"));
    }
    let base = match v.get("base").and_then(Value::as_str) {
        Some("dark") => Base::Dark,
        Some("light") => Base::Light,
        _ => return Err("`base` must be \"dark\" or \"light\"".into()),
    };
    let src = match v.get("colors") {
        None => serde_json::Map::new(),
        Some(Value::Object(o)) => o.clone(),
        Some(_) => return Err("`colors` must be an object of colour names and values".into()),
    };
    if src.len() > MAX_COLORS {
        return Err(format!("`colors` has more than {MAX_COLORS} entries"));
    }
    let mut probe = Tokens::light();
    let mut bad = Vec::new();
    for (k, c) in &src {
        match slot(&mut probe, k) {
            None => bad.push(format!("`{k}` is not a theme colour")),
            Some(s) => {
                if let Err(e) = colors_of(c, s.len()) {
                    bad.push(format!("`{k}` {e}"));
                }
            }
        }
    }
    if !bad.is_empty() {
        bad.truncate(8);
        return Err(bad.join("; "));
    }
    Ok(ThemeFile { id: slug(name), name: name.to_string(), base, colors: src.into_iter().collect() })
}

/// An id from a name: lower-case letters, digits and dashes.
fn slug(name: &str) -> String {
    let mut s = String::new();
    for c in name.chars().flat_map(char::to_lowercase) {
        if c.is_ascii_alphanumeric() {
            s.push(c);
        } else if !s.ends_with('-') && !s.is_empty() {
            s.push('-');
        }
    }
    let s = s.trim_end_matches('-').to_string();
    if s.is_empty() { "theme".into() } else { s }
}

/// The theme's palette: its base with its colours on top. A colour the saved preferences hold
/// that this version can't read (a hand edit) is skipped, never an error.
pub fn resolve(theme: &ThemeFile) -> Tokens {
    let mut t = if theme.base.is_dark() { Tokens::dark() } else { Tokens::light() };
    for (k, v) in &theme.colors {
        if let Some(s) = slot(&mut t, k)
            && let Ok(c) = colors_of(v, s.len())
        {
            s[0] = c;
        }
    }
    t
}

/// A complete theme file of a palette: every key, so it is a starting point for a new theme.
pub fn export(name: &str, base: Base, t: &Tokens) -> Value {
    let mut t = *t;
    let mut colors = serde_json::Map::new();
    for k in keys() {
        if let Some(s) = slot(&mut t, k) {
            colors.insert(k.to_string(), json!(format_color(s[0])));
        }
    }
    json!({"format": FORMAT, "name": name, "base": base, "colors": colors})
}

/// Add an imported theme to `custom`, replacing one with the same id (importing a file again
/// updates it). Its id never takes a base mode's (dark, light, system). Returns the id it is kept
/// under.
pub fn add(custom: &mut Vec<ThemeFile>, mut theme: ThemeFile) -> Result<String, String> {
    if matches!(theme.id.as_str(), "dark" | "light" | "system") {
        theme.id = format!("{}-1", theme.id);
    }
    if custom.iter().all(|t| t.id != theme.id) && custom.len() >= MAX_CUSTOM {
        return Err(format!("at most {MAX_CUSTOM} imported themes are kept"));
    }
    let id = theme.id.clone();
    match custom.iter_mut().find(|t| t.id == id) {
        Some(slot) => *slot = theme,
        None => custom.push(theme),
    }
    Ok(id)
}
