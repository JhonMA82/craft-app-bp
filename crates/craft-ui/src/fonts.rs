//! Bundled IBM Plex interface faces and the weight families the kit draws with. The same
//! definitions are used on desktop and web.
//!
//! Adapted from SolidCraft (`crates/ui-egui/src/fonts.rs`), MIT OR Apache-2.0. The system font
//! scan, CJK/Hangul/Thai fallbacks and user font overrides were removed. See ATTRIBUTION.md.

use std::sync::Arc;

use egui::{FontDefinitions, FontFamily, FontId, RichText};

const PLEX_SANS: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/fonts/ibm-plex-sans/IBMPlexSans-Variable.ttf"));
const PLEX_MONO: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/fonts/ibm-plex-mono/IBMPlexMono-Regular.ttf"));
const WEIGHTS: [(&str, f32); 4] = [("plex-sans", 400.0), ("ui-medium", 500.0), ("ui-semibold", 600.0), ("ui-bold", 700.0)];

fn sans_families() -> [FontFamily; 4] {
    [FontFamily::Proportional, medium_family(), semibold_family(), bold_family()]
}

/// IBM Plex Sans at 400/500/600/700 and Plex Mono Regular, followed by egui's symbol/emoji
/// fallbacks. Each variable instance shares the embedded bytes and selects a real weight.
pub fn base_font_definitions() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    let fallbacks = fonts.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    for ((name, weight), family) in WEIGHTS.into_iter().zip(sans_families()) {
        let mut data = egui::FontData::from_static(PLEX_SANS);
        data.tweak.coords = egui::epaint::text::VariationCoords::new([(b"wght", weight)]);
        fonts.font_data.insert(name.to_owned(), Arc::new(data));
        fonts.families.insert(family, std::iter::once(name.to_owned()).chain(fallbacks.iter().cloned()).collect());
    }
    fonts.font_data.insert("plex-mono".to_owned(), Arc::new(egui::FontData::from_static(PLEX_MONO)));
    fonts.families.entry(FontFamily::Monospace).or_default().insert(0, "plex-mono".to_owned());
    with_symbol_fallback(fonts)
}

fn medium_family() -> FontFamily {
    FontFamily::Name("ui-medium".into())
}

fn semibold_family() -> FontFamily {
    FontFamily::Name("ui-semibold".into())
}

fn bold_family() -> FontFamily {
    FontFamily::Name("ui-bold".into())
}

/// Medium (500) interface text.
pub fn medium(size: f32) -> FontId {
    FontId::new(size, medium_family())
}

/// Semibold (600) interface text.
pub fn semibold(size: f32) -> FontId {
    FontId::new(size, semibold_family())
}

/// Bold (700) interface text.
pub fn bold(size: f32) -> FontId {
    FontId::new(size, bold_family())
}

/// Select a real interface weight without changing the text's size or colour.
pub trait RichTextExt {
    fn medium(self) -> Self;
    fn semibold(self) -> Self;
    fn bold_face(self) -> Self;
}

impl RichTextExt for RichText {
    fn medium(self) -> Self {
        self.family(medium_family())
    }
    fn semibold(self) -> Self {
        self.family(semibold_family())
    }
    fn bold_face(self) -> Self {
        self.family(bold_family())
    }
}

/// egui's built-in monospace face, also the last fallback of every sans family.
pub const SYMBOL_FALLBACK: &str = "Hack";

/// `fonts` with egui's built-in Hack last in every sans family: Ubuntu-Light, NotoEmoji and
/// emoji-icon-font lack common symbols such as ▲ ▼ ↑ ↓ → ⁻, which Hack has.
pub fn with_symbol_fallback(mut fonts: FontDefinitions) -> FontDefinitions {
    if fonts.font_data.contains_key(SYMBOL_FALLBACK) {
        for family in sans_families() {
            let list = fonts.families.entry(family).or_default();
            list.retain(|name| name != SYMBOL_FALLBACK);
            list.push(SYMBOL_FALLBACK.to_owned());
        }
    }
    fonts
}

fn installed_fonts_id() -> egui::Id {
    egui::Id::new("craft_interface_fonts")
}

fn install(ctx: &egui::Context, fonts: FontDefinitions) {
    let fonts = with_symbol_fallback(fonts);
    ctx.data_mut(|d| d.insert_temp(installed_fonts_id(), fonts.clone()));
    ctx.set_fonts(fonts);
    ctx.global_style_mut(|style| {
        if let Some(heading) = style.text_styles.get_mut(&egui::TextStyle::Heading) {
            heading.family = semibold(heading.size).family;
        }
    });
}

/// Install only the bundled definitions (also used by the headless test harness).
pub fn setup_bundled_fonts(ctx: &egui::Context) {
    install(ctx, base_font_definitions());
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    #[test]
    fn bundled_fonts_define_every_weight_and_mono() {
        let fonts = base_font_definitions();
        for (name, _) in WEIGHTS {
            assert!(fonts.font_data.contains_key(name), "{name} is bundled");
        }
        assert!(fonts.font_data.contains_key("plex-mono"));
        let sans = fonts.families.get(&FontFamily::Proportional).expect("sans family");
        assert_eq!(sans.first().map(String::as_str), Some("plex-sans"));
        assert_eq!(sans.last().map(String::as_str), Some(SYMBOL_FALLBACK));
    }
}
