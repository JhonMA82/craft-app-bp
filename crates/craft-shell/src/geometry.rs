//! Transient shell geometry shared by the ribbon, the cards and the viewport corners.
//!
//! Adapted from SolidCraft (`crates/ui-egui/src/shell.rs`), MIT OR Apache-2.0. See ATTRIBUTION.md.

use craft_ui::layout::LayoutMode;
use craft_ui::ui_kit::{Container, Surface};
use egui::{Color32, Painter, Rect, pos2, vec2};

use crate::ribbon::WORKSPACE_WIDTH;

/// The two floating cards at the top of the workspace: the workspace switcher and the ribbon.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Cards {
    pub workspace: Rect,
    pub ribbon: Rect,
    /// The card column's bottom edge, including the gap below it.
    pub bottom: f32,
}

impl Cards {
    pub(crate) fn new(available: Rect, mode: LayoutMode, ribbon_height: f32) -> Self {
        let margin = Container::for_layout(mode, Surface::Chrome).gap();
        let gap = margin;
        let left = available.left() + margin;
        let top = available.top() + margin;
        let right = (available.right() - margin).max(left);
        let floating = Container::for_layout(mode, Surface::Chrome).floating();
        let workspace = Rect::from_min_size(
            pos2(left, top),
            vec2(WORKSPACE_WIDTH.min((right - left).max(0.0)), ribbon_height - if floating { 0.0 } else { 1.0 }),
        );
        let ribbon = Rect::from_min_max(pos2((workspace.right() + gap).min(right), top), pos2(right, top + ribbon_height));
        Self { workspace, ribbon, bottom: ribbon.bottom() + margin }
    }
}

/// Fill the two concave corners where the application bar flows into the viewport's top edge, so
/// the viewport reads as a rounded surface under the bar.
pub(crate) fn paint_top_corners(painter: &Painter, viewport: Rect, color: Color32) {
    let r = craft_ui::layout::corner_radius().min(viewport.width() / 2.0).min(viewport.height() / 2.0);
    if !(r.is_finite() && r > 0.5) {
        return;
    }
    const STEPS: usize = 32;
    for left in [true, false] {
        let (corner, centre, dir) = if left {
            (viewport.left_top(), pos2(viewport.left() + r, viewport.top() + r), -1.0)
        } else {
            (viewport.right_top(), pos2(viewport.right() - r, viewport.top() + r), 1.0)
        };
        let mut mesh = egui::Mesh::default();
        mesh.colored_vertex(corner, color);
        for i in 0..=STEPS {
            // From the top tangent point round to the side tangent point.
            let a = std::f32::consts::FRAC_PI_2 * i as f32 / STEPS as f32;
            mesh.colored_vertex(pos2(centre.x + dir * r * a.sin(), centre.y - r * a.cos()), color);
        }
        for i in 0..STEPS as u32 {
            mesh.add_triangle(0, i + 1, i + 2);
        }
        painter.add(egui::Shape::mesh(mesh));
    }
}
