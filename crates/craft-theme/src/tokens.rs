//! The design tokens every widget paints from: spacing, radii, type sizes, form metrics and the
//! semantic colours of the shell. `palettes` fills in the light and dark values; theme files
//! (`files`) override any colour by name.
//!
//! Adapted from SolidCraft (`crates/ui-egui/src/theme.rs`), MIT OR Apache-2.0. See ATTRIBUTION.md.

use serde::{Deserialize, Serialize};

use egui::Color32;

/// A named accent family: the tab colour, its ink and its soft surface tint.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Accent {
    Blue,
    Green,
    Orange,
    Yellow,
    Purple,
    Teal,
    Pink,
}

impl Accent {
    pub const ALL: [Accent; 7] = [Accent::Blue, Accent::Green, Accent::Orange, Accent::Yellow, Accent::Purple, Accent::Teal, Accent::Pink];

    /// The stable id used by settings and theme files.
    pub fn id(self) -> &'static str {
        match self {
            Accent::Blue => "blue",
            Accent::Green => "green",
            Accent::Orange => "orange",
            Accent::Yellow => "yellow",
            Accent::Purple => "purple",
            Accent::Teal => "teal",
            Accent::Pink => "pink",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Accent::ALL.into_iter().find(|a| a.id() == id)
    }

    /// The three colours of the family under `t`.
    pub fn colors(self, t: &Tokens) -> AccentColors {
        match self {
            Accent::Blue => AccentColors { color: t.accent, ink: t.selected_ink, soft: t.accent_soft },
            Accent::Green => AccentColors { color: t.green, ink: t.green_ink, soft: t.green_soft },
            Accent::Orange => AccentColors { color: t.orange, ink: t.orange_ink, soft: t.orange_soft },
            Accent::Yellow => AccentColors { color: t.yellow, ink: t.yellow_ink, soft: t.yellow_soft },
            Accent::Purple => AccentColors { color: t.purple, ink: t.purple_ink, soft: t.purple_soft },
            Accent::Teal => AccentColors { color: t.teal, ink: t.teal_ink, soft: t.teal_soft },
            Accent::Pink => AccentColors { color: t.pink, ink: t.pink_ink, soft: t.pink_soft },
        }
    }
}

/// A tab's selection colours. Deliberately separate from the global widget style: chrome,
/// checkboxes, status colours and dialog actions keep their own tokens.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AccentColors {
    pub color: Color32,
    pub ink: Color32,
    pub soft: Color32,
}

impl AccentColors {
    /// Small text on a tab's solid pill. Brighter fills need dark ink in either theme.
    pub fn on(self, t: &Tokens) -> Color32 {
        if contrast_ratio(t.on_accent, self.color) >= 4.5 { t.on_accent } else { t.tab_on_fill_dark }
    }

    /// Store the active tab's accent for the frame, so widgets drawn after the ribbon agree.
    pub fn publish(ctx: &egui::Context, accent: Accent) {
        let colors = accent.colors(&Tokens::get());
        ctx.data_mut(|d| d.insert_temp(egui::Id::new("craft_tab_accent"), colors));
    }

    pub fn current(ctx: &egui::Context) -> AccentColors {
        ctx.data(|d| d.get_temp::<AccentColors>(egui::Id::new("craft_tab_accent"))).unwrap_or_else(|| Accent::Blue.colors(&Tokens::get()))
    }
}

/// WCAG contrast of two opaque sRGB colours.
pub fn contrast_ratio(a: Color32, b: Color32) -> f64 {
    let luminance = |c: Color32| {
        let linear = |v: u8| {
            let v = f64::from(v) / 255.0;
            if v <= 0.04045 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
        };
        0.2126 * linear(c.r()) + 0.7152 * linear(c.g()) + 0.0722 * linear(c.b())
    };
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tokens {
    // Spacing.
    pub space_2: f32,
    pub space_3: f32,
    pub space_4: f32,
    pub space_6: f32,
    pub space_8: f32,
    pub space_10: f32,
    pub space_12: f32,
    pub space_14: f32,
    pub space_16: f32,
    pub space_24: f32,
    // Radii.
    pub radius_sm: u8,
    pub radius_md: u8,
    pub radius_lg: u8,
    pub radius_xl: u8,
    // Typography.
    pub font_badge: f32,
    pub font_micro: f32,
    pub font_panel: f32,
    pub font_small: f32,
    pub font_caption: f32,
    pub font_body: f32,
    pub font_label: f32,
    pub font_button: f32,
    pub font_heading: f32,
    pub font_subtitle: f32,
    pub font_intro: f32,
    pub font_title: f32,
    pub font_section_tracking: f32,
    // Form metrics.
    pub control_height: f32,
    pub control_radius: u8,
    pub form_label_width: f32,
    pub form_gap_y: f32,
    pub form_footer_height: f32,
    pub form_header_height: f32,
    pub form_card_width: f32,
    pub form_icon_size: f32,
    pub form_footer_icon_size: f32,
    pub form_close_size: f32,
    pub form_inset: f32,
    pub form_icon_inset: f32,
    pub form_section_icon_size: f32,
    pub form_checkbox_height: f32,
    pub form_button_height: f32,
    pub mode_size: f32,
    pub mode_radius: u8,
    pub mode_tint: f32,
    // File/start page metrics.
    pub file_action_height: f32,
    pub file_back_height: f32,
    pub file_grid_gap: f32,
    pub file_nav_width: f32,
    pub file_nav_height: f32,
    pub file_tile_height: f32,
    pub file_tile_icon: f32,
    pub file_recent_height: f32,
    pub file_sample_image_height: f32,
    pub file_content_inset_x: f32,
    pub file_content_inset_y: f32,
    pub file_section_gap: f32,
    pub font_start_title: f32,
    // Chrome.
    pub app_bar: Color32,
    pub app_bar_text: Color32,
    pub app_bar_separator: Color32,
    pub app_bar_hover: Color32,
    pub app_bar_hover_subtle: Color32,
    pub app_bar_hover_button: Color32,
    pub app_bar_hover_strong: Color32,
    pub app_bar_tab_active: Color32,
    pub toolbar: Color32,
    pub panel: Color32,
    pub panel_header: Color32,
    pub border: Color32,
    pub text: Color32,
    pub text_dim: Color32,
    pub icon: Color32,
    pub hover: Color32,
    // Dialogs and forms.
    pub dialog_body: Color32,
    pub dialog_bg: Color32,
    pub dialog_footer: Color32,
    pub dialog_rule: Color32,
    pub dialog_border: Color32,
    pub dialog_label: Color32,
    pub control_fill: Color32,
    pub control_border: Color32,
    pub field: Color32,
    pub selected_bg: Color32,
    pub selected_ink: Color32,
    pub accent: Color32,
    pub accent_soft: Color32,
    pub on_accent: Color32,
    pub tab_on_fill_dark: Color32,
    pub form_ink: Color32,
    pub form_dim: Color32,
    pub form_primary: Color32,
    pub form_error: Color32,
    pub form_error_fill: Color32,
    pub form_success: Color32,
    pub form_success_fill: Color32,
    pub error: Color32,
    pub warning: Color32,
    pub chip: Color32,
    pub section_cap: Color32,
    pub overlay: Color32,
    // Timeline history, viewport and the File/start page.
    pub timeline: Color32,
    pub timeline_item: Color32,
    pub timeline_marker: Color32,
    pub viewport_top: Color32,
    pub viewport_bottom: Color32,
    pub grid: Color32,
    pub grid_major: Color32,
    pub axis_x: Color32,
    pub axis_y: Color32,
    pub axis_z: Color32,
    pub file_background: Color32,
    pub file_surface: Color32,
    pub file_border: Color32,
    pub file_image_tint: Color32,
    pub file_rail: Color32,
    pub file_rail_ink: Color32,
    pub file_rail_hover: Color32,
    pub file_primary: Color32,
    pub file_primary_ink: Color32,
    pub file_primary_border: Color32,
    pub file_dim: Color32,
    // Accent families.
    pub green: Color32,
    pub green_ink: Color32,
    pub green_soft: Color32,
    pub orange: Color32,
    pub orange_ink: Color32,
    pub orange_soft: Color32,
    pub yellow: Color32,
    pub yellow_ink: Color32,
    pub yellow_soft: Color32,
    pub purple: Color32,
    pub purple_ink: Color32,
    pub purple_soft: Color32,
    pub teal: Color32,
    pub teal_ink: Color32,
    pub teal_soft: Color32,
    pub pink: Color32,
    pub pink_ink: Color32,
    pub pink_soft: Color32,
}
