//! Colours and widget style: a dark theme (the default, a blue-grey in the spirit of desktop
//! tools) and a light one, each with colour vision modes, and theme files that set any of their
//! colours (`files`). Every colour the UI uses comes from [`Tokens`].
//!
//! Adapted from SolidCraft (`crates/ui-egui/src/theme.rs`), MIT OR Apache-2.0. See ATTRIBUTION.md.

use std::cell::Cell;

mod palettes;

pub mod files;
pub mod tokens;

pub use files::{Base, MAX_COLORS, MAX_CUSTOM, ThemeFile};
pub use tokens::{Accent, AccentColors, Tokens, contrast_ratio};

use egui::Stroke;
use serde::{Deserialize, Serialize};

thread_local! {
    /// Theme state belongs to the UI thread drawing the frame. Concurrent app/test renderers
    /// must not change one another's palette while a frame or icon texture is being built.
    static DARK: Cell<bool> = const { Cell::new(true) };
    /// The colour vision mode, per thread like the rest.
    static VISION: Cell<ColorVision> = const { Cell::new(ColorVision::Off) };
    /// The palette of a theme file (`files`) in place of the built-in one of its base.
    static CUSTOM: Cell<Option<Custom>> = const { Cell::new(None) };
}

/// A theme file's palette and whether it is dark.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Custom {
    pub dark: bool,
    pub tokens: Tokens,
}

/// Use a theme file's palette for its base, or the built-in palettes (`None`). Returns whether
/// that changed anything (the visuals follow on the next [`apply`]).
pub fn set_custom(custom: Option<Custom>) -> bool {
    CUSTOM.with(|c| c.replace(custom) != custom)
}

/// Is the dark theme on?
pub fn is_dark() -> bool {
    DARK.get()
}

/// Switch themes (the visuals follow on the next [`apply`]).
pub fn set_dark(dark: bool) {
    DARK.set(dark);
}

/// Colour vision modes: a second set of values for the colours that carry a meaning, for a weak
/// or missing red (protan), green (deutan) or blue (tritan) sense. Not a filter: Off gives back
/// the theme's colours unchanged. Protan and Deutan share one red-green table.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ColorVision {
    #[default]
    Off,
    Protan,
    Deutan,
    Tritan,
}

impl ColorVision {
    pub const ALL: [ColorVision; 4] = [ColorVision::Off, ColorVision::Protan, ColorVision::Deutan, ColorVision::Tritan];

    /// The saved value.
    pub fn id(self) -> &'static str {
        match self {
            ColorVision::Off => "off",
            ColorVision::Protan => "protan",
            ColorVision::Deutan => "deutan",
            ColorVision::Tritan => "tritan",
        }
    }

    pub fn from_id(id: &str) -> Option<ColorVision> {
        ColorVision::ALL.into_iter().find(|m| m.id() == id)
    }
}

/// From saved preferences: a value this version doesn't know is Off rather than an error, which
/// would lose every other saved preference with it.
impl<'de> Deserialize<'de> for ColorVision {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v = serde_json::Value::deserialize(d)?;
        Ok(v.as_str().and_then(ColorVision::from_id).unwrap_or_default())
    }
}

/// The colour vision mode on now (on this thread).
pub fn vision() -> ColorVision {
    VISION.with(Cell::get)
}

/// Switch the colour vision mode (the visuals follow on the next [`apply`]).
pub fn set_vision(mode: ColorVision) {
    VISION.with(|v| v.set(mode));
}

/// An sRGB colour given as 0..1, for drawing in the UI.
pub fn srgb(c: [f32; 3]) -> egui::Color32 {
    let byte = |x: f32| (x.clamp(0.0, 1.0) * 255.0).round() as u8;
    egui::Color32::from_rgb(byte(c[0]), byte(c[1]), byte(c[2]))
}

impl Tokens {
    /// Subtle shell elevation, derived from the semantic application-bar colour.
    pub fn shell_shadow(self) -> egui::Shadow {
        egui::Shadow { offset: [0, self.space_2 as i8], blur: self.space_8 as u8, spread: 0, color: self.app_bar.gamma_multiply(0.18) }
    }

    /// The tokens of the current theme and colour vision mode.
    pub fn get() -> Tokens {
        Tokens::themed(is_dark(), vision())
    }

    /// The tokens of a theme and colour vision mode: a theme file's palette when one is on for
    /// that base ([`set_custom`]), else the built-in one; the colour vision mode on top.
    pub fn themed(dark: bool, mode: ColorVision) -> Tokens {
        let custom = CUSTOM.get().filter(|c| c.dark == dark).map(|c| c.tokens);
        custom.unwrap_or(if dark { Tokens::dark() } else { Tokens::light() }).with_vision(dark, mode)
    }
}

fn visuals(dark: bool) -> egui::Visuals {
    let t = Tokens::themed(dark, vision());
    let mut v = if dark { egui::Visuals::dark() } else { egui::Visuals::light() };
    v.panel_fill = t.panel;
    v.window_fill = t.panel;
    v.window_stroke = Stroke::new(1.0, t.border);
    v.extreme_bg_color = t.field;
    v.faint_bg_color = t.panel_header;
    v.override_text_color = Some(t.text);
    v.widgets.inactive.weak_bg_fill = t.panel_header;
    v.widgets.inactive.bg_fill = t.panel_header;
    v.widgets.active.weak_bg_fill = t.accent_soft;
    v.selection.bg_fill = t.accent_soft;
    v.selection.stroke = Stroke::new(1.0, t.accent);
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, t.border);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, t.text);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, t.text);
    v.widgets.hovered.weak_bg_fill = t.hover;
    v.widgets.hovered.bg_fill = t.hover;
    // egui uses the open-state fill for a movable window's active title bar.
    v.widgets.open.weak_bg_fill = t.panel_header;
    v.window_corner_radius = egui::CornerRadius::same(t.radius_md);
    v
}

/// Apply the current theme, vision mode and widget spacing to `ctx`.
pub fn apply(ctx: &egui::Context) {
    let dark = is_dark();
    ctx.set_theme(if dark { egui::Theme::Dark } else { egui::Theme::Light });
    ctx.set_visuals(visuals(dark));
    let t = Tokens::get();
    ctx.global_style_mut(|s| {
        s.spacing.item_spacing = egui::vec2(t.space_6, t.space_4);
        s.spacing.button_padding = egui::vec2(t.space_6, t.space_3);
    });
}

#[cfg(test)]
mod tests;
