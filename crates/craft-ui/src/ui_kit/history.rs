//! Shared history chips, the six-dot panel grip and the rollback transport controls.
//!
//! Adapted from SolidCraft (`crates/ui-egui/src/ui_kit/history.rs`), MIT OR Apache-2.0. The
//! engine-specific feature classification, error badges and named-row styles were removed.
//! See ATTRIBUTION.md.

use crate::theme::{AccentColors, Tokens};
use crate::{fonts, icons, scenario};
use egui::{Color32, FontId, Painter, Rect, Stroke, Vec2, pos2, vec2};

/// Browser and history rows share the same density.
pub const TREE_ROW_HEIGHT: f32 = 26.0;

/// The four rollback controls: to the oldest state, back one, forward one, to the newest.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HistoryTransport {
    First,
    Back,
    Forward,
    Last,
}

impl HistoryTransport {
    pub const ALL: [Self; 4] = [Self::First, Self::Back, Self::Forward, Self::Last];

    pub fn id(self) -> &'static str {
        match self {
            Self::First => "first",
            Self::Back => "back",
            Self::Forward => "forward",
            Self::Last => "last",
        }
    }

    fn label(self) -> &'static str {
        crate::i18n::tr(match self {
            Self::First => "ui.history_first",
            Self::Back => "ui.history_back",
            Self::Forward => "ui.history_forward",
            Self::Last => "ui.history_last",
        })
    }

    pub fn show(self, ui: &mut egui::Ui, rect: Rect) -> egui::Response {
        let t = Tokens::get();
        let response = ui.interact(rect, ui.id().with(("history_transport", self.id())), egui::Sense::click());
        scenario::publish_target(&format!("history:{}", self.id()), rect);
        if response.hovered() {
            ui.painter().rect_filled(rect, t.radius_sm, t.hover);
        }
        transport_glyph(ui.painter(), rect.center(), self.id(), t.icon);
        response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), self.label()));
        response.on_hover_cursor(egui::CursorIcon::PointingHand).on_hover_text(self.label())
    }
}

/// The four controls' width: 24 pt buttons, 2 pt apart.
pub fn history_transport_width() -> f32 {
    let t = Tokens::get();
    t.space_24 * HistoryTransport::ALL.len() as f32 + t.space_2 * (HistoryTransport::ALL.len() - 1) as f32
}

fn transport_glyph(p: &Painter, c: egui::Pos2, what: &str, col: Color32) {
    let tri = |dir: f32, dx: f32| {
        let pts = vec![pos2(c.x + dx - 4.0 * dir, c.y - 5.0), pos2(c.x + dx + 4.0 * dir, c.y), pos2(c.x + dx - 4.0 * dir, c.y + 5.0)];
        p.add(egui::Shape::convex_polygon(pts, col, Stroke::NONE));
    };
    let bar = |dx: f32| p.line_segment([pos2(c.x + dx, c.y - 5.0), pos2(c.x + dx, c.y + 5.0)], Stroke::new(1.6, col));
    match what {
        "first" => {
            bar(-5.0);
            tri(-1.0, 1.0);
        }
        "back" => tri(-1.0, 0.0),
        "forward" => tri(1.0, 0.0),
        _ => {
            tri(1.0, -1.0);
            bar(5.0);
        }
    }
}

/// The six-dot grip every floating card's header carries.
pub fn panel_grip(ui: &mut egui::Ui) -> egui::Response {
    let t = Tokens::get();
    let (rect, response) = ui.allocate_exact_size(vec2(t.space_10, t.space_16 + t.space_2), egui::Sense::hover());
    for row in 0..3 {
        for col in 0..2 {
            ui.painter().circle_filled(rect.left_top() + vec2(2.5 + col as f32 * 5.0, 4.0 + row as f32 * 5.0), 1.1, t.text_dim);
        }
    }
    response
}

/// A history strip's chip: icon, name, and the selected/hovered states in the tab's accent.
pub struct HistoryChip<'a> {
    pub icon: &'a str,
    pub name: &'a str,
    pub selected: bool,
    pub hovered: bool,
    pub accent: AccentColors,
}

impl HistoryChip<'_> {
    pub fn paint(&self, painter: &Painter, rect: Rect) {
        let t = Tokens::get();
        let fill = if self.selected {
            self.accent.soft
        } else if self.hovered {
            t.hover
        } else {
            t.timeline_item
        };
        let edge = if self.selected || self.hovered { self.accent.color } else { t.border };
        painter.rect(rect, t.radius_md, fill, Stroke::new(1.0, edge), egui::StrokeKind::Inside);
        let side = 15.0;
        let icon = Rect::from_center_size(pos2(rect.left() + t.space_8 + side / 2.0, rect.center().y), Vec2::splat(side));
        icons::paint_accented(painter, icon, self.icon, None, 1.0);
        let ink = if self.selected { self.accent.ink } else { t.text };
        let font = if self.selected { fonts::semibold(t.font_body) } else { FontId::proportional(t.font_body) };
        let text_rect = Rect::from_min_max(pos2(icon.right() + t.space_4, rect.top()), pos2(rect.right() - t.space_8, rect.bottom()));
        let galley = painter.layout_no_wrap(self.name.to_owned(), font, ink);
        painter.with_clip_rect(text_rect.intersect(painter.clip_rect())).galley(
            pos2(text_rect.left(), rect.center().y - galley.size().y / 2.0),
            galley,
            ink,
        );
    }
}

/// The rollback marker in a horizontal strip: a line from the top with a hollow end circle.
pub fn history_marker(p: &Painter, rect: Rect, accent: AccentColors) {
    let t = Tokens::get();
    let outer_radius = t.space_10 / 2.0;
    let centre = pos2(rect.center().x, rect.bottom() - outer_radius);
    let start = rect.center_top();
    // The line reaches one point inside the circle's outer edge, as on the reference board.
    let end = centre - vec2(0.0, outer_radius - 1.0);
    p.line_segment([start, end], Stroke::new(t.space_2, accent.color));
    p.circle(centre, outer_radius - t.space_2 / 2.0, t.timeline, Stroke::new(t.space_2, accent.color));
}
