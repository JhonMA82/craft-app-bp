//! Stable UI rendering boundary for the icon artwork.
//!
//! Icons are the SVG masters in `assets/icons/ui/design/svg`, rendered by [`crate::icon_svg`]:
//! colours come from `palette.toml` (light and dark, and the accents), never from the caller.
//! An icon with no master draws the placeholder.
//!
//! Adapted from SolidCraft (`crates/ui-egui/src/icons.rs`), MIT OR Apache-2.0. The glyph,
//! marker, navigation and pointer sources were removed. See ATTRIBUTION.md.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use egui::{Color32, Painter, Rect, TextureHandle, TextureOptions, pos2};

use crate::icon_svg::{self, Look};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Finish {
    Artwork,
    Ink(Color32),
    Danger,
}

/// Textures already rasterised: one per (master, look, pixel size, finish).
#[derive(Clone, Default)]
struct Cache(Arc<Mutex<HashMap<(&'static str, Look, u32, Finish), TextureHandle>>>);

/// Draw palette-owned icon artwork with an explicit opacity (0 to 1).
pub fn paint_icon(p: &Painter, r: Rect, name: &str, opacity: f32) {
    paint_on(p, r, name, None, opacity);
}

/// Draw the icon `name` for a surface that is always dark (the application bar), whatever the
/// theme: `dark: Some(true)`. `None` follows the theme.
pub fn paint_on(p: &Painter, r: Rect, name: &str, dark: Option<bool>, opacity: f32) {
    paint_with_ink(p, r, name, dark, opacity, None, Mirror::None);
}

/// Monochrome flat controls on the application bar use its text token exactly.
pub fn paint_app_bar(p: &Painter, r: Rect, name: &str, opacity: f32) {
    paint_with_ink(p, r, name, Some(true), opacity, Some(crate::theme::Tokens::get().app_bar_text), Mirror::None);
}

/// Paint a single-colour icon in `ink`.
pub fn paint_ink(p: &Painter, r: Rect, name: &str, ink: Color32) {
    paint_with_ink(p, r, name, Some(true), 1.0, Some(ink), Mirror::None);
}

/// Paint a single-colour icon in `ink`, mirrored top to bottom (a chevron pointing up).
pub fn paint_ink_flipped(p: &Painter, r: Rect, name: &str, ink: Color32) {
    paint_with_ink(p, r, name, Some(true), 1.0, Some(ink), Mirror::Vertical);
}

/// Paint a single-colour icon mirrored left to right, reusing the original master.
pub fn paint_ink_mirrored(p: &Painter, r: Rect, name: &str, ink: Color32) {
    paint_with_ink(p, r, name, Some(true), 1.0, Some(ink), Mirror::Horizontal);
}

#[derive(Clone, Copy)]
enum Mirror {
    None,
    Vertical,
    Horizontal,
}

fn paint_with_ink(p: &Painter, r: Rect, name: &str, dark: Option<bool>, opacity: f32, ink: Option<Color32>, mirror: Mirror) {
    let ctx = p.ctx();
    let resolved = icon_svg::resolve(name);
    let look = Look { dark: dark.unwrap_or_else(|| ctx.global_style().visuals.dark_mode), accent: resolved.accent, vision: crate::theme::vision() };
    paint_resolved(p, r, resolved, look, opacity, ink.map_or(Finish::Artwork, Finish::Ink), mirror);
}

/// Draw with a named palette ramp, preserving outlines, passive faces and transparency.
/// `None` explicitly selects the base blue; an unknown name keeps the icon's own accent.
pub fn paint_accented(p: &Painter, r: Rect, name: &str, accent: Option<&str>, opacity: f32) {
    let resolved = icon_svg::resolve(name);
    let accent = match accent {
        Some(name) => icon_svg::accent_index(name).or(resolved.accent),
        None => None,
    };
    let look = Look { dark: p.ctx().global_style().visuals.dark_mode, accent, vision: crate::theme::vision() };
    paint_resolved(p, r, resolved, look, opacity, Finish::Artwork, Mirror::None);
}

/// Shared destructive-action artwork: red shaded faces, normal outlines and cutouts.
pub fn paint_danger(p: &Painter, r: Rect, name: &str, opacity: f32) {
    let resolved = icon_svg::resolve(name);
    let look = Look { dark: p.ctx().global_style().visuals.dark_mode, accent: icon_svg::accent_index("pink"), vision: crate::theme::vision() };
    paint_resolved(p, r, resolved, look, opacity, Finish::Danger, Mirror::None);
}

/// Paint a window caption's glyph centred at `at`, in `stroke`'s ink and width. A maximized
/// window's maximize caption becomes Restore: a front square with the corner of the one behind.
pub fn paint_caption(ui: &egui::Ui, caption: crate::titlebar::Caption, maximized: bool, at: egui::Pos2, stroke: egui::Stroke) {
    use crate::titlebar::Caption;
    let p = ui.painter();
    let g = Rect::from_center_size(at, egui::vec2(10.0, 10.0));
    match caption {
        Caption::Minimize => {
            p.line_segment([g.left_center(), g.right_center()], stroke);
        }
        Caption::Maximize if maximized => {
            let front = Rect::from_min_max(g.min + egui::vec2(0.0, 2.0), g.max - egui::vec2(2.0, 0.0));
            p.rect_stroke(front, 0.0, stroke, egui::StrokeKind::Middle);
            p.line(
                vec![
                    g.min + egui::vec2(2.0, 2.0),
                    g.min + egui::vec2(2.0, 0.0),
                    g.right_top(),
                    g.max - egui::vec2(0.0, 2.0),
                    g.max - egui::vec2(2.0, 2.0),
                ],
                stroke,
            );
        }
        Caption::Maximize => {
            p.rect_stroke(g, 0.0, stroke, egui::StrokeKind::Middle);
        }
        Caption::Close => {
            p.line_segment([g.left_top(), g.right_bottom()], stroke);
            p.line_segment([g.right_top(), g.left_bottom()], stroke);
        }
    }
}

fn paint_resolved(p: &Painter, r: Rect, resolved: icon_svg::Resolved, look: Look, opacity: f32, finish: Finish, mirror: Mirror) {
    let ctx = p.ctx();
    let ppp = ctx.pixels_per_point().max(0.5);
    let side = r.width().min(r.height());
    let px = (side * ppp).round().clamp(8.0, 512.0) as u32;
    let cache = ctx.data_mut(|d| d.get_temp_mut_or_insert_with(egui::Id::new("craft_icon_cache"), Cache::default).clone());
    let Ok(mut map) = cache.0.lock() else { return };
    let key = (resolved.master, look, px, finish);
    if let std::collections::hash_map::Entry::Vacant(slot) = map.entry(key) {
        let Some(doc) = icon_svg::doc_for(resolved.master, px) else { return };
        let pm = if finish == Finish::Danger { icon_svg::render_tinted(doc, look, px) } else { icon_svg::render(doc, look, px) };
        let Some(pm) = pm else { return };
        let mut image = egui::ColorImage::from_rgba_premultiplied([px as usize, px as usize], pm.data());
        if let Finish::Ink(ink) = finish {
            for pixel in &mut image.pixels {
                *pixel = ink.gamma_multiply(f32::from(pixel.a()) / 255.0);
            }
        }
        slot.insert(ctx.load_texture(format!("icon:{}:{px}", resolved.master), image, TextureOptions::LINEAR));
    }
    let Some(tex) = map.get(&key) else { return };
    // Snap the image to whole physical pixels so it stays crisp.
    let size = px as f32 / ppp;
    let min = pos2(((r.center().x - size / 2.0) * ppp).round() / ppp, ((r.center().y - size / 2.0) * ppp).round() / ppp);
    let tint = Color32::from_white_alpha((opacity.clamp(0.0, 1.0) * 255.0).round() as u8);
    let uv = match mirror {
        Mirror::None => Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
        Mirror::Vertical => Rect::from_min_max(pos2(0.0, 1.0), pos2(1.0, 0.0)),
        Mirror::Horizontal => Rect::from_min_max(pos2(1.0, 0.0), pos2(0.0, 1.0)),
    };
    p.image(tex.id(), Rect::from_min_size(min, egui::vec2(size, size)), uv, tint);
}
