//! The UI layer: bundled fonts, the icon pipeline, published scenario geometry and the retained
//! ui-kit every host is built from. Domain-free by design: apps supply the model.
//!
//! Adapted from SolidCraft (`crates/ui-egui`), MIT OR Apache-2.0. See ATTRIBUTION.md.

pub use craft_i18n as i18n;
pub use craft_theme as theme;

pub mod canvas;
pub mod fonts;
pub mod frame;
pub mod icon_svg;
pub mod icons;
pub mod input;
pub mod layout;
pub mod parse;
pub mod scenario;
pub mod titlebar;
pub mod ui_kit;

#[cfg(test)]
mod ui_kit_tests;
