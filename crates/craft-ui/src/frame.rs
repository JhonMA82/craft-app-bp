//! Dock dragging and bounded anchors. Outer hosts and surfaces live in `ui_kit::Container`.
//!
//! Adapted from SolidCraft (`crates/ui-egui/src/frame.rs`), MIT OR Apache-2.0. See ATTRIBUTION.md.

use egui::{Pos2, Ui};

/// Width limits of a docked command dialog.
pub const DIALOG_MIN_W: f32 = 260.0;
pub const DIALOG_MAX_W: f32 = 380.0;

/// Which edge of the viewport a docked panel sits against.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Right,
    Left,
}

impl Side {
    /// The x of the viewport edge this side docks against.
    pub fn anchor_x(self, viewport: egui::Rect) -> f32 {
        match self {
            Side::Right => viewport.right(),
            Side::Left => viewport.left(),
        }
    }
}

/// How far the open docked dialog titled `title` was dragged from its docked place; zero for a
/// dialog that was not dragged, or once another dialog opens.
pub fn dock_offset(ctx: &egui::Context, title: &str) -> egui::Vec2 {
    ctx.data(|d| d.get_temp::<(String, egui::Vec2)>(egui::Id::new("craft_dock_offset")))
        .filter(|(t, _)| t == title)
        .map(|(_, v)| v)
        .unwrap_or(egui::Vec2::ZERO)
}

/// Forget a dragged dialog's place (it closed): the next one opens docked.
pub fn reset_dock_offset(ctx: &egui::Context) {
    ctx.data_mut(|d| d.remove::<(String, egui::Vec2)>(egui::Id::new("craft_dock_offset")));
}

/// A docked dialog's anchor moved by `offset`, kept so its docked corner stays in `vp` with room
/// for its title band.
pub fn offset_anchor(docked: Pos2, offset: egui::Vec2, vp: egui::Rect, side: Side) -> Pos2 {
    let p = docked + offset;
    let (lo, hi) = match side {
        Side::Right => (vp.left() + DIALOG_MIN_W, vp.right()),
        Side::Left => (vp.left(), vp.right() - DIALOG_MIN_W),
    };
    let x = if lo <= hi { p.x.clamp(lo, hi) } else { docked.x };
    let y = if vp.top() <= vp.bottom() - 40.0 { p.y.clamp(vp.top(), vp.bottom() - 40.0) } else { docked.y };
    egui::pos2(x, y)
}

/// Let `band` (a docked dialog's title band) drag the dialog: the drag adds to its offset `moved`
/// (as last frame left it; `applied` of it is where the dialog is now, after keeping it on
/// screen). Called before the band's buttons are added, so they keep their clicks.
pub fn drag_dock(ui: &Ui, band: egui::Rect, title: &str, moved: egui::Vec2, applied: egui::Vec2) {
    crate::scenario::publish_handle("dialog:title", band.center());
    let response = ui.interact(band, ui.id().with("craft_dock_drag"), egui::Sense::drag());
    // The drag runs from where the press started, so no movement is lost while egui decides it
    // is a drag, nor on the frame it ends.
    let base = egui::Id::new("craft_dock_drag_base");
    if response.drag_started()
        && let Some(origin) = ui.input(|i| i.pointer.press_origin())
    {
        ui.ctx().data_mut(|d| d.insert_temp(base, (applied, origin)));
    }
    if response.dragged() || response.drag_stopped() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
        let start = ui.ctx().data(|d| d.get_temp::<(egui::Vec2, Pos2)>(base));
        let to = match (start, ui.input(|i| i.pointer.latest_pos())) {
            (Some((start, from)), Some(now)) => start + (now - from),
            _ => applied + response.drag_delta(),
        };
        ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("craft_dock_offset"), (title.to_string(), to)));
    } else if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
    } else if moved != applied {
        // Kept on screen: remember where it is, not where it was dragged off to.
        ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("craft_dock_offset"), (title.to_string(), applied)));
    }
}
