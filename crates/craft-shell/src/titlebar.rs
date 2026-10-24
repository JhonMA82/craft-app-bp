//! The window's own title bar where the window has no OS decorations
//! (`UiSettings::custom_titlebar`): empty space in the application bar drags the window and a
//! double-click there maximizes or restores it, caption buttons at the far right minimize,
//! maximize/restore and close, and thin invisible zones along the window's edges resize it.
//!
//! Adapted from SolidCraft (`crates/ui-egui/src/titlebar.rs`), MIT OR Apache-2.0. The Windows
//! native hit-test hook and DPI helpers are not ported. See ATTRIBUTION.md.

use craft_ui::icons;
use craft_ui::theme::Tokens;
use egui::{CursorIcon, Id, LayerId, Order, PointerButton, Rect, ResizeDirection, Sense, Stroke, Ui, ViewportCommand, pos2, vec2};

pub use craft_ui::titlebar::{CAPTIONS, Caption};

/// One caption button: as wide as the native Windows ones.
pub const BUTTON_WIDTH: f32 = 46.0;
/// The three caption buttons together.
pub const WIDTH: f32 = 3.0 * BUTTON_WIDTH;
/// Windows' resize border at 100 % (96 DPI): `SM_CXSIZEFRAME` + `SM_CXPADDEDBORDER`, 4 + 4 px.
pub const RESIZE_BORDER_AT_96_DPI: f32 = 8.0;

/// Caption button ids (stable, so tests and agents can find them).
pub fn caption_id(caption: Caption) -> Id {
    caption.id()
}

fn maximized(ctx: &egui::Context) -> bool {
    ctx.input(|i| i.viewport().maximized.unwrap_or(false))
}

pub fn toggle_maximize(ctx: &egui::Context) {
    ctx.send_viewport_cmd(ViewportCommand::Maximized(!maximized(ctx)));
}

/// The bar's background: a press-and-drag on empty space moves the window, a double-click
/// toggles maximize. Call it before laying out the bar so its buttons, registered later, take
/// their clicks.
pub fn drag_area(ui: &Ui, rect: Rect) {
    let ctx = ui.ctx();
    let response = ui.interact(rect, Id::new("craft_titlebar_drag"), Sense::click_and_drag());
    // egui hands a drag that starts on a click-only widget to the drag-sensing background below
    // it, so remember whether the press itself landed on bare background.
    if ctx.input(|i| i.pointer.any_pressed()) {
        let bare = ctx.viewport(|v| v.hits.click.is_some_and(|w| w.id == response.id));
        ctx.data_mut(|d| d.insert_temp(response.id, bare));
    }
    if response.double_clicked() {
        toggle_maximize(ctx);
    } else if response.drag_started_by(PointerButton::Primary) && ctx.data(|d| d.get_temp(response.id).unwrap_or(false)) {
        ctx.send_viewport_cmd(ViewportCommand::StartDrag);
    }
}

/// Minimize, Maximize/Restore and Close at the right end of `bar` (the application bar's rect),
/// full bar height, flush with the window's edge so a maximized window's corner hits Close.
pub fn caption_buttons(ui: &mut Ui, bar: Rect) {
    let t = Tokens::get();
    let max = maximized(ui.ctx());
    let mut clicked = None;
    for (index, caption) in CAPTIONS.into_iter().enumerate() {
        let r = Rect::from_min_size(pos2(bar.right() - WIDTH + index as f32 * BUTTON_WIDTH, bar.top()), vec2(BUTTON_WIDTH, bar.height()));
        let response = ui.interact(r, caption_id(caption), Sense::click());
        let fill = match (response.is_pointer_button_down_on(), response.hovered()) {
            (true, _) => t.app_bar_hover_subtle,
            (_, true) => t.app_bar_hover_button,
            _ if response.has_focus() => t.app_bar_hover_button,
            _ => egui::Color32::TRANSPARENT,
        };
        ui.painter().rect_filled(r, 0.0, fill);
        icons::paint_caption(ui, caption, max, r.center(), Stroke::new(1.0, t.app_bar_text));
        let tip = match caption {
            Caption::Minimize => craft_ui::i18n::tr("ui.minimize"),
            Caption::Maximize if max => craft_ui::i18n::tr("ui.restore"),
            Caption::Maximize => craft_ui::i18n::tr("ui.maximize"),
            Caption::Close => craft_ui::i18n::tr("ui.close"),
        };
        craft_ui::scenario::publish_target(
            match caption {
                Caption::Minimize => "titlebar:minimize",
                Caption::Maximize => "titlebar:maximize",
                Caption::Close => "titlebar:close",
            },
            r,
        );
        if response.on_hover_text(tip).clicked() {
            clicked = Some(caption);
        }
    }
    if let Some(caption) = clicked {
        let ctx = ui.ctx();
        match caption {
            Caption::Minimize => ctx.send_viewport_cmd(ViewportCommand::Minimized(true)),
            Caption::Maximize => toggle_maximize(ctx),
            Caption::Close => ctx.send_viewport_cmd(ViewportCommand::Close),
        }
    }
}

/// The resize border in physical pixels at `scale` (1.0 = 96 DPI): Windows' default, scaled and
/// rounded, never less than a pixel.
pub fn resize_border_px(scale: f32) -> f32 {
    let s = if scale.is_finite() && scale > 0.0 { scale } else { 1.0 };
    (RESIZE_BORDER_AT_96_DPI * s).round().max(1.0)
}

/// The side of a corner zone for an edge `edge` wide: twice the edge, so a corner is easy to hit.
pub fn corner_size(edge: f32) -> f32 {
    2.0 * edge
}

/// The edge and corner zones of a window with rect `w`, edges `edge` wide: (zone, direction,
/// cursor). Corners come last so they win where they overlap the edges.
pub fn zones(w: Rect, edge: f32) -> [(Rect, ResizeDirection, CursorIcon); 8] {
    use ResizeDirection::*;
    let (e, c) = (edge, corner_size(edge));
    [
        (Rect::from_min_max(w.left_top(), pos2(w.right(), w.top() + e)), North, CursorIcon::ResizeVertical),
        (Rect::from_min_max(pos2(w.left(), w.bottom() - e), w.right_bottom()), South, CursorIcon::ResizeVertical),
        (Rect::from_min_max(w.left_top(), pos2(w.left() + e, w.bottom())), West, CursorIcon::ResizeHorizontal),
        (Rect::from_min_max(pos2(w.right() - e, w.top()), w.right_bottom()), East, CursorIcon::ResizeHorizontal),
        (Rect::from_min_size(w.left_top(), vec2(c, c)), NorthWest, CursorIcon::ResizeNwSe),
        (Rect::from_min_size(w.right_top() - vec2(c, 0.0), vec2(c, c)), NorthEast, CursorIcon::ResizeNeSw),
        (Rect::from_min_size(w.left_bottom() - vec2(0.0, c), vec2(c, c)), SouthWest, CursorIcon::ResizeNeSw),
        (Rect::from_min_size(w.right_bottom() - vec2(c, c), vec2(c, c)), SouthEast, CursorIcon::ResizeNwSe),
    ]
}

/// Invisible resize zones along the window's edges, above everything else; a press in one
/// starts an OS resize in that direction. None while the window is maximized or fullscreen.
pub fn resize_zones(ui: &mut Ui) {
    let ctx = ui.ctx().clone();
    if maximized(&ctx) || ctx.input(|i| i.viewport().fullscreen.unwrap_or(false)) {
        return;
    }
    let w = ctx.content_rect();
    let ppp = ctx.pixels_per_point();
    let edge = resize_border_px(ctx.input(|i| i.viewport().native_pixels_per_point).unwrap_or(ppp)) / ppp;
    let layer = LayerId::new(Order::Foreground, Id::new("craft_titlebar_resize"));
    let zui = ui.new_child(egui::UiBuilder::new().layer_id(layer).max_rect(w));
    for (i, (r, dir, cursor)) in zones(w, edge).into_iter().enumerate() {
        let resp = zui.interact(r, Id::new(("craft_titlebar_resize", i)), Sense::drag()).on_hover_cursor(cursor);
        if resp.drag_started_by(PointerButton::Primary) {
            ctx.send_viewport_cmd(ViewportCommand::BeginResize(dir));
        }
    }
}
