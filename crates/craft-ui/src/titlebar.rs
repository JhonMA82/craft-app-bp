//! The window-caption glyph vocabulary. The shell owns the buttons and their actions; the
//! artwork lives here with the rest of the drawn chrome.
//!
//! Adapted from SolidCraft (`crates/ui-egui/src/titlebar.rs` and
//! `assets/icons/ui/source/glyphs.rs`), MIT OR Apache-2.0. See ATTRIBUTION.md.

/// One caption button, left to right as the window draws them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Caption {
    Minimize,
    Maximize,
    Close,
}

/// The three captions in drawing order.
pub const CAPTIONS: [Caption; 3] = [Caption::Minimize, Caption::Maximize, Caption::Close];

impl Caption {
    /// The stable id a caption's interaction response is created with.
    pub fn id(self) -> egui::Id {
        egui::Id::new(("craft_titlebar_caption", self as u8))
    }
}
