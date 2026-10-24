//! A domain-free 2D canvas: the workspace stand-in for SolidCraft's viewport. It paints the
//! gradient background, an isometric grid, the origin triad, the axis view and the navigation
//! bar, and pans on a middle-button drag. Zooming is through the navigation bar, never the wheel,
//! so scrollable content over the canvas keeps its scroll.
//!
//! Adapted from SolidCraft (`crates/ui-egui/src/viewport.rs`), MIT OR Apache-2.0. The 3D scene,
//! picking and camera are not ported. See ATTRIBUTION.md.

use crate::theme::Tokens;
use crate::ui_kit::{Container, Surface};
use crate::{fonts, icons, scenario};
use egui::{Align2, Mesh, Painter, Pos2, Rect, Sense, Shape, Stroke, Ui, pos2, vec2};

/// Space the axis view occupies at the canvas's top-right corner, reused by the right cards.
pub const AXIS_VIEW_CLEARANCE: f32 = 128.0;

/// The axis view's square, and its inset from the canvas edge.
const AXIS_VIEW_SIZE: f32 = 112.0;
const AXIS_VIEW_INSET: f32 = 8.0;
/// The grid's spacing in points at zoom 1.
const GRID_SPACING: f32 = 32.0;
/// The half-range of drawn grid lines, in cells.
const GRID_HALF_RANGE: i32 = 40;
/// The origin triad's arm length.
const TRIAD_LENGTH: f32 = 64.0;
/// The navigation bar's height and its buttons' side.
const NAV_HEIGHT: f32 = 30.0;
const NAV_BUTTON: f32 = 30.0;
const NAV_INSET: f32 = 8.0;

#[derive(Clone, Copy, Debug)]
pub struct CanvasState {
    pub pan: egui::Vec2,
    pub zoom: f32,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self { pan: egui::Vec2::ZERO, zoom: 1.0 }
    }
}

/// A navigation-bar control the caller acts on after painting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CanvasAction {
    ZoomIn,
    ZoomOut,
    Fit,
    Home,
    Settings,
    Messages,
}

/// The navigation bar's buttons, left to right.
const NAV_ICONS: [&str; 6] = ["pan", "zoom", "fit", "home", "settings", "info"];

/// Space the shell reserves under the canvas for a bottom card, so the navigation bar stays clear.
pub const BOTTOM_INSET: &str = "craft_canvas_bottom_inset";

/// Draw the canvas over `rect` and report the navigation action the user chose.
pub fn show(ui: &mut Ui, rect: Rect, state: &mut CanvasState) -> Option<CanvasAction> {
    if !(rect.is_finite() && rect.width() > 0.0 && rect.height() > 0.0) {
        return None;
    }
    let t = Tokens::get();
    background(ui.painter(), rect, &t);
    let origin = rect.center() + state.pan;
    let spacing = GRID_SPACING * state.zoom;
    grid(ui.painter(), rect, origin, spacing, &t);
    triad(ui.painter(), origin, &t);
    axis_view(ui, rect, &t);
    scenario::publish_rect("canvas:grid", rect);
    pan_input(ui, rect, state);
    navigation(ui, rect, state, &t)
}

fn background(painter: &Painter, rect: Rect, t: &Tokens) {
    let mut mesh = Mesh::default();
    let top = rect.left_top();
    let bottom = rect.left_bottom();
    mesh.colored_vertex(top, t.viewport_top);
    mesh.colored_vertex(rect.right_top(), t.viewport_top);
    mesh.colored_vertex(bottom, t.viewport_bottom);
    mesh.colored_vertex(rect.right_bottom(), t.viewport_bottom);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(1, 3, 2);
    painter.add(Shape::mesh(mesh));
}

/// The isometric ground: two line families along the projected baselines.
fn grid(painter: &Painter, rect: Rect, origin: Pos2, spacing: f32, t: &Tokens) {
    if !(spacing.is_finite() && spacing >= 0.5) {
        return;
    }
    let painter = painter.with_clip_rect(rect.intersect(painter.clip_rect()));
    let dx = vec2(0.866, 0.5);
    let dy = vec2(-0.866, 0.5);
    let minor = t.grid.gamma_multiply(0.6);
    let major = t.grid_major.gamma_multiply(0.9);
    let far = |index: i32| index as f32 * spacing;
    let point = |i: i32, j: i32| origin + (dx * far(i) + dy * far(j));
    for i in -GRID_HALF_RANGE..=GRID_HALF_RANGE {
        if i == 0 {
            continue;
        }
        let stroke = if i % 5 == 0 { major } else { minor };
        let stroke = Stroke::new(1.0, stroke);
        painter.line_segment([point(i, -GRID_HALF_RANGE), point(i, GRID_HALF_RANGE)], stroke);
        painter.line_segment([point(-GRID_HALF_RANGE, i), point(GRID_HALF_RANGE, i)], stroke);
    }
}

/// The three axis arms from the origin, with their letters at the tips.
fn triad(painter: &Painter, origin: Pos2, t: &Tokens) {
    let font = fonts::semibold(t.font_micro);
    for (dir, colour, letter) in [(vec2(0.866, 0.5), t.axis_x, "X"), (vec2(-0.866, 0.5), t.axis_y, "Y"), (vec2(0.0, -1.0), t.axis_z, "Z")] {
        let tip = origin + dir * TRIAD_LENGTH;
        painter.line_segment([origin, tip], Stroke::new(1.5, colour));
        painter.circle_filled(tip, 3.0, colour);
        painter.text(tip + dir * 10.0, Align2::CENTER_CENTER, letter, font.clone(), colour);
    }
}

/// The compass at the canvas's top-right corner: the projected axes and a Home mark.
fn axis_view(ui: &mut Ui, rect: Rect, t: &Tokens) {
    let square = Rect::from_min_size(
        pos2(rect.right() - AXIS_VIEW_INSET - AXIS_VIEW_SIZE, rect.top() + AXIS_VIEW_INSET),
        vec2(AXIS_VIEW_SIZE, AXIS_VIEW_SIZE),
    );
    if !(square.is_finite() && square.is_positive()) {
        return;
    }
    scenario::publish_rect("axis_view:outer", square);
    let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
    let center = square.center();
    let font = fonts::semibold(t.font_micro);
    for (name, dir, colour) in [("x", vec2(0.866, 0.5), t.axis_x), ("y", vec2(-0.866, 0.5), t.axis_y), ("z", vec2(0.0, -1.0), t.axis_z)] {
        let at = center + dir * 26.0;
        painter.circle_filled(at, 8.0, colour);
        let letter = name.to_uppercase();
        painter.text(at, Align2::CENTER_CENTER, &letter, font.clone(), t.on_accent);
        scenario::publish_target(&format!("axis_view:{name}"), Rect::from_center_size(at, vec2(16.0, 16.0)));
    }
    let home = Rect::from_center_size(center + vec2(-38.0, -38.0), vec2(16.0, 16.0));
    icons::paint_icon(&painter, home, "home", 1.0);
}

/// A middle-button drag pans the canvas.
fn pan_input(ui: &Ui, rect: Rect, state: &mut CanvasState) {
    let response = ui.interact(rect, ui.id().with("canvas_pan"), Sense::drag());
    if response.dragged_by(egui::PointerButton::Middle) && crate::input::once_this_pass(ui.ctx(), "canvas_pan") {
        let delta = ui.input(|input| input.pointer.delta());
        if delta.is_finite() {
            state.pan += delta;
        }
    }
}

/// The floating navigation bar at the bottom centre of the canvas. It is an Area on the middle
/// layer, so the app's own content never covers it.
fn navigation(ui: &mut Ui, rect: Rect, state: &mut CanvasState, t: &Tokens) -> Option<CanvasAction> {
    let width = NAV_BUTTON * NAV_ICONS.len() as f32 + t.space_16;
    let lift = ui.ctx().data(|d| d.get_temp::<f32>(egui::Id::new(BOTTOM_INSET))).unwrap_or(0.0);
    let bottom = rect.bottom() - if lift.is_finite() { lift.max(0.0) } else { 0.0 };
    let bar = Rect::from_center_size(pos2(rect.center().x, bottom - NAV_INSET - NAV_HEIGHT / 2.0), vec2(width, NAV_HEIGHT));
    if !(bar.is_finite() && bar.is_positive() && rect.contains_rect(bar)) {
        return None;
    }
    let frame = Container::new(ui.ctx(), Surface::Navigation).frame(ui.ctx(), t.panel, crate::frame::Side::Left);
    let mut action = None;
    Container::new(ui.ctx(), Surface::Navigation).area(egui::Id::new("craft_canvas_nav")).fixed_pos(bar.min).order(egui::Order::Middle).show(
        ui.ctx(),
        |ui| {
            frame.show(ui, |ui| {
                ui.set_width(bar.width());
                ui.set_height(bar.height());
                ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
                ui.horizontal_centered(|ui| {
                    ui.add_space(t.space_8);
                    for icon in NAV_ICONS {
                        let (button, response) = ui.allocate_exact_size(vec2(NAV_BUTTON, NAV_BUTTON), Sense::click());
                        if response.hovered() || response.is_pointer_button_down_on() {
                            ui.painter().rect_filled(button.shrink(t.space_2), t.radius_sm, t.hover);
                        }
                        let glyph = Rect::from_center_size(button.center(), vec2(t.space_16, t.space_16));
                        icons::paint_icon(ui.painter(), glyph, icon, 1.0);
                        response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, icon));
                        scenario::publish_target(&format!("nav:{icon}"), button);
                        if (response.clicked() || response.secondary_clicked()) && crate::input::once_this_pass(ui.ctx(), "canvas_nav") {
                            match icon {
                                "pan" => state.pan = egui::Vec2::ZERO,
                                "zoom" => {
                                    let out = response.secondary_clicked() || ui.input(|input| input.modifiers.shift);
                                    action = Some(if out { CanvasAction::ZoomOut } else { CanvasAction::ZoomIn });
                                }
                                "fit" => action = Some(CanvasAction::Fit),
                                "home" => action = Some(CanvasAction::Home),
                                "settings" => action = Some(CanvasAction::Settings),
                                _ => action = Some(CanvasAction::Messages),
                            }
                        }
                    }
                });
            });
        },
    );
    if matches!(action, Some(CanvasAction::Fit | CanvasAction::Home)) {
        state.pan = egui::Vec2::ZERO;
        state.zoom = 1.0;
    }
    action
}
