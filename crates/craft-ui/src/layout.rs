//! Where the shell's panels sit: floating cards over the canvas, or docked to the window edges.
//!
//! Adapted from SolidCraft (`crates/ui-egui/src/shell.rs`), MIT OR Apache-2.0. See ATTRIBUTION.md.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LayoutMode {
    #[default]
    Floating,
    Docked,
}

impl LayoutMode {
    pub fn publish(self, ctx: &egui::Context) {
        ctx.data_mut(|data| data.insert_temp(egui::Id::new("craft_container_layout"), self));
    }

    pub fn current(ctx: &egui::Context) -> Self {
        ctx.data(|data| data.get_temp(egui::Id::new("craft_container_layout"))).unwrap_or_default()
    }
}

/// The gap from the viewport edge to the floating cards and between the cards.
pub const SHELL_GAP: f32 = 7.0;

/// The viewport's top corners are rounded concentrically with the cards: the card margin plus
/// the card radius.
pub fn corner_radius() -> f32 {
    SHELL_GAP + f32::from(craft_theme::Tokens::get().radius_lg)
}
