//! The original light and dark palette values, compiled into [`Tokens`].
//!
//! Adapted from SolidCraft (`assets/styles/ui/palettes.rs`), MIT OR Apache-2.0. The CAD-domain
//! fields were dropped; the values kept here are unchanged. See ATTRIBUTION.md.

use crate::ColorVision;
use crate::tokens::Tokens;
use egui::Color32;

impl Tokens {
    pub const fn light() -> Tokens {
        Tokens {
            space_2: 2.0,
            space_3: 3.0,
            space_4: 4.0,
            space_6: 6.0,
            space_8: 8.0,
            space_10: 10.0,
            space_12: 12.0,
            space_14: 14.0,
            space_16: 16.0,
            space_24: 24.0,
            radius_sm: 4,
            radius_md: 6,
            radius_lg: 8,
            radius_xl: 10,
            font_badge: 9.5,
            font_micro: 10.0,
            font_panel: 10.5,
            font_small: 11.0,
            font_caption: 11.5,
            font_body: 12.0,
            font_label: 12.5,
            font_button: 13.0,
            font_heading: 14.0,
            font_subtitle: 15.0,
            font_intro: 16.0,
            font_title: 22.0,
            font_section_tracking: 0.04,
            control_height: 28.0,
            control_radius: 6,
            form_label_width: 70.0,
            form_gap_y: 5.0,
            form_footer_height: 48.0,
            form_header_height: 40.0,
            form_card_width: 264.0,
            form_icon_size: 19.0,
            form_footer_icon_size: 15.0,
            form_close_size: 26.0,
            form_inset: 9.0,
            form_icon_inset: 4.5,
            form_section_icon_size: 11.0,
            form_checkbox_height: 22.0,
            form_button_height: 28.0,
            mode_size: 36.0,
            mode_radius: 7,
            mode_tint: 0.12,
            file_action_height: 32.0,
            file_back_height: 40.0,
            file_grid_gap: 14.0,
            file_nav_width: 248.0,
            file_nav_height: 38.0,
            file_tile_height: 84.0,
            file_tile_icon: 44.0,
            file_recent_height: 52.0,
            file_sample_image_height: 84.0,
            file_content_inset_x: 40.0,
            file_content_inset_y: 32.0,
            file_section_gap: 28.0,
            font_start_title: 28.0,
            app_bar: Color32::from_rgb(52, 56, 64),
            app_bar_text: Color32::from_rgb(230, 234, 242),
            app_bar_separator: Color32::from_rgb(136, 145, 160),
            app_bar_hover: Color32::from_rgba_unmultiplied_const(255, 255, 255, 28),
            app_bar_hover_subtle: Color32::from_rgba_unmultiplied_const(255, 255, 255, 18),
            app_bar_hover_button: Color32::from_rgba_unmultiplied_const(255, 255, 255, 30),
            app_bar_hover_strong: Color32::from_rgba_unmultiplied_const(255, 255, 255, 40),
            app_bar_tab_active: Color32::from_rgb(53, 59, 71),
            toolbar: Color32::from_rgb(246, 247, 249),
            panel: Color32::from_rgb(250, 251, 252),
            panel_header: Color32::from_rgb(236, 238, 242),
            border: Color32::from_rgb(208, 212, 220),
            text: Color32::from_rgb(38, 42, 50),
            text_dim: Color32::from_rgb(110, 116, 128),
            icon: Color32::from_rgb(70, 78, 92),
            hover: Color32::from_rgb(226, 234, 246),
            dialog_body: Color32::from_rgb(255, 255, 255),
            dialog_bg: Color32::from_rgb(250, 251, 252),
            dialog_footer: Color32::from_rgb(244, 246, 249),
            dialog_rule: Color32::from_rgb(230, 234, 240),
            dialog_border: Color32::from_rgb(213, 219, 227),
            dialog_label: Color32::from_rgb(58, 70, 86),
            control_fill: Color32::from_rgb(255, 255, 255),
            control_border: Color32::from_rgb(195, 203, 214),
            field: Color32::WHITE,
            selected_bg: Color32::from_rgb(232, 241, 250),
            selected_ink: Color32::from_rgb(26, 80, 127),
            accent: Color32::from_rgb(38, 120, 218),
            accent_soft: Color32::from_rgb(214, 230, 250),
            on_accent: Color32::WHITE,
            tab_on_fill_dark: Color32::BLACK,
            form_ink: Color32::from_rgb(27, 36, 48),
            form_dim: Color32::from_rgb(74, 88, 104),
            form_primary: Color32::from_rgb(35, 110, 173),
            form_error: Color32::from_rgb(161, 52, 44),
            form_error_fill: Color32::from_rgb(252, 239, 237),
            form_success: Color32::from_rgb(39, 107, 44),
            form_success_fill: Color32::from_rgb(229, 242, 230),
            error: Color32::from_rgb(214, 54, 54),
            warning: Color32::from_rgb(222, 150, 30),
            chip: Color32::from_rgb(38, 120, 218),
            section_cap: Color32::from_rgb(214, 128, 72),
            overlay: Color32::from_rgba_unmultiplied_const(255, 255, 255, 235),
            timeline: Color32::from_rgb(232, 235, 240),
            timeline_item: Color32::WHITE,
            timeline_marker: Color32::from_rgb(70, 76, 88),
            viewport_top: Color32::from_rgb(240, 243, 247),
            viewport_bottom: Color32::from_rgb(203, 210, 221),
            grid: Color32::from_rgb(196, 202, 212),
            grid_major: Color32::from_rgb(170, 178, 190),
            axis_x: Color32::from_rgb(255, 0, 0),
            axis_y: Color32::from_rgb(0, 255, 0),
            axis_z: Color32::from_rgb(2, 2, 248),
            file_background: Color32::from_rgb(247, 249, 251),
            file_surface: Color32::WHITE,
            file_border: Color32::from_rgb(224, 229, 236),
            file_image_tint: Color32::WHITE,
            file_rail: Color32::from_rgb(35, 110, 173),
            file_rail_ink: Color32::WHITE,
            file_rail_hover: Color32::from_rgb(43, 118, 180),
            file_primary: Color32::from_rgb(227, 240, 251),
            file_primary_ink: Color32::from_rgb(26, 80, 127),
            file_primary_border: Color32::from_rgb(50, 139, 211),
            file_dim: Color32::from_rgb(58, 70, 86),
            green: Color32::from_rgb(5, 127, 69),
            green_ink: Color32::from_rgb(5, 127, 69),
            green_soft: Color32::from_rgb(237, 247, 241),
            orange: Color32::from_rgb(164, 80, 0),
            orange_ink: Color32::from_rgb(164, 80, 0),
            orange_soft: Color32::from_rgb(251, 235, 216),
            yellow: Color32::from_rgb(224, 178, 0),
            yellow_ink: Color32::from_rgb(138, 109, 0),
            yellow_soft: Color32::from_rgb(255, 248, 220),
            purple: Color32::from_rgb(115, 73, 205),
            purple_ink: Color32::from_rgb(115, 73, 205),
            purple_soft: Color32::from_rgb(238, 232, 250),
            teal: Color32::from_rgb(30, 111, 150),
            teal_ink: Color32::from_rgb(30, 111, 150),
            teal_soft: Color32::from_rgb(225, 240, 247),
            pink: Color32::from_rgb(168, 50, 110),
            pink_ink: Color32::from_rgb(168, 50, 110),
            pink_soft: Color32::from_rgb(248, 230, 239),
        }
    }

    pub const fn dark() -> Tokens {
        let mut t = Tokens::light();
        t.app_bar = Color32::from_rgb(28, 31, 38);
        t.toolbar = Color32::from_rgb(39, 43, 52);
        t.panel = Color32::from_rgb(39, 43, 52);
        t.panel_header = Color32::from_rgb(47, 52, 62);
        t.border = Color32::from_rgb(63, 69, 82);
        t.text = Color32::from_rgb(222, 226, 232);
        t.text_dim = Color32::from_rgb(148, 155, 168);
        t.accent = Color32::from_rgb(58, 136, 238);
        t.accent_soft = Color32::from_rgb(42, 70, 110);
        t.green = Color32::from_rgb(18, 141, 69);
        t.green_ink = Color32::from_rgb(70, 190, 120);
        t.green_soft = Color32::from_rgb(31, 59, 43);
        t.orange = Color32::from_rgb(178, 106, 20);
        t.orange_ink = Color32::from_rgb(224, 145, 74);
        t.orange_soft = Color32::from_rgb(69, 51, 30);
        t.yellow = Color32::from_rgb(198, 168, 0);
        t.yellow_ink = Color32::from_rgb(212, 183, 38);
        t.yellow_soft = Color32::from_rgb(65, 57, 26);
        t.purple = Color32::from_rgb(138, 111, 204);
        t.purple_ink = Color32::from_rgb(179, 155, 238);
        t.purple_soft = Color32::from_rgb(55, 47, 79);
        t.teal = Color32::from_rgb(42, 127, 168);
        t.teal_ink = Color32::from_rgb(79, 179, 217);
        t.teal_soft = Color32::from_rgb(31, 58, 71);
        t.pink = Color32::from_rgb(200, 88, 143);
        t.pink_ink = Color32::from_rgb(224, 116, 174);
        t.pink_soft = Color32::from_rgb(64, 35, 54);
        t.dialog_body = Color32::from_rgb(39, 43, 52);
        t.dialog_bg = Color32::from_rgb(39, 43, 52);
        t.dialog_footer = Color32::from_rgb(47, 52, 62);
        t.dialog_rule = Color32::from_rgb(53, 59, 71);
        t.dialog_border = Color32::from_rgb(63, 69, 82);
        t.control_fill = Color32::from_rgb(28, 31, 38);
        t.control_border = Color32::from_rgb(74, 82, 98);
        t.dialog_label = t.text;
        t.selected_ink = Color32::from_rgb(156, 199, 245);
        t.selected_bg = Color32::from_rgb(42, 70, 110);
        t.error = Color32::from_rgb(240, 100, 96);
        t.warning = Color32::from_rgb(236, 170, 70);
        t.icon = Color32::from_rgb(208, 214, 226);
        t.hover = Color32::from_rgb(54, 61, 74);
        t.chip = Color32::from_rgb(59, 101, 168);
        t.overlay = Color32::from_rgba_unmultiplied_const(34, 38, 46, 240);
        t.timeline = Color32::from_rgb(33, 37, 45);
        t.timeline_item = Color32::from_rgb(52, 58, 70);
        t.timeline_marker = Color32::from_rgb(170, 178, 192);
        t.viewport_top = Color32::from_rgb(46, 52, 64);
        t.viewport_bottom = Color32::from_rgb(46, 52, 64);
        t.grid = Color32::from_rgb(59, 65, 76);
        t.grid_major = Color32::from_rgb(73, 79, 90);
        t.file_background = Color32::from_rgb(31, 35, 43);
        t.file_surface = Color32::from_rgb(39, 43, 52);
        t.file_border = Color32::from_rgb(63, 69, 82);
        t.file_rail = Color32::from_rgb(30, 80, 128);
        t.file_rail_hover = Color32::from_rgb(46, 99, 149);
        t.file_primary = Color32::from_rgb(27, 48, 72);
        t.file_primary_ink = Color32::from_rgb(156, 199, 245);
        t.file_primary_border = Color32::from_rgb(58, 136, 238);
        t.file_dim = Color32::from_rgb(180, 187, 200);
        t.field = Color32::from_rgb(30, 34, 41);
        t.form_ink = t.text;
        t.form_dim = Color32::from_rgb(180, 187, 200);
        t.form_primary = t.accent;
        t.form_error = t.error;
        t.form_error_fill = Color32::from_rgb(110, 50, 54);
        t.form_success = Color32::from_rgb(143, 219, 154);
        t.form_success_fill = Color32::from_rgb(44, 74, 51);
        t
    }

    /// These tokens with a colour vision mode's values on top; Off gives them back unchanged.
    pub(crate) const fn with_vision(self, dark: bool, mode: ColorVision) -> Tokens {
        match mode {
            ColorVision::Off => self,
            ColorVision::Protan | ColorVision::Deutan => self.red_green(dark),
            ColorVision::Tritan => self.blue_yellow(dark),
        }
    }

    /// Red-green (Protan and Deutan share it): values of the theme's colours that a weak red or
    /// green sense still tells apart. Only the colours this crate keeps are overridden.
    const fn red_green(self, dark: bool) -> Tokens {
        let mut t = self;
        // Keep validation badges separate from red errors under both red-green simulations.
        t.form_success_fill = if dark { Color32::from_rgb(44, 57, 67) } else { Color32::from_rgb(227, 227, 246) };
        if !dark {
            t.form_success = Color32::from_rgb(39, 88, 78);
        }
        t
    }

    /// Blue-yellow (Tritan): none of the colours kept here carry a red-green meaning, so the
    /// palette is unchanged.
    const fn blue_yellow(self, _dark: bool) -> Tokens {
        self
    }
}
