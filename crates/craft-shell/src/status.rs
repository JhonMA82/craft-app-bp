//! The transient status chip. SolidCraft has no permanent status line: a notice (or a running
//! task's progress) appears as a small card at the bottom-left of the workspace for six seconds
//! and then goes. A click opens the Messages window.
//!
//! Adapted from SolidCraft (`crates/ui-egui/src/viewport.rs`: `paint_toast`), MIT OR Apache-2.0.
//! See ATTRIBUTION.md.

use craft_ui::theme::Tokens;
use craft_ui::ui_kit::{Container, Surface};
use egui::{Rect, Sense, Ui, pos2, vec2};

use crate::{Shell, ShellApp};

/// How long a notice stays on screen.
const LIFETIME_MS: f64 = 6000.0;
/// The chip's inset from the content's bottom-left corner.
const INSET: f32 = 12.0;
/// The chip's widest text column.
const MAX_TEXT_W: f32 = 440.0;

/// The transient chip over `content` (the workspace below the app bar or the ribbon).
pub fn show<A: ShellApp>(ui: &mut Ui, shell: &mut Shell, app: &A, content: Rect) {
    let t = Tokens::get();
    let transient = shell.status.as_ref().filter(|(_, at, _)| crate::now_ms() - at < LIFETIME_MS);
    let status = app.status();
    let fraction = status.as_ref().and_then(|status| status.fraction).filter(|fraction| fraction.is_finite());
    let (text, error) = match (transient, &status) {
        (Some((text, _, error)), _) => (text.clone(), *error),
        (None, Some(status)) if fraction.is_some() => (status.text.clone(), false),
        _ => return,
    };
    if !(content.is_finite() && content.is_positive()) {
        return;
    }
    let painter = ui.painter().with_clip_rect(content);
    let galley = painter.layout(text.clone(), craft_ui::fonts::medium(t.font_label), t.text, (content.width() - 40.0).min(MAX_TEXT_W));
    let size = galley.size() + vec2(40.0, 16.0);
    if !(size.is_finite() && size.x <= content.width() && size.y <= content.height()) {
        return;
    }
    // Above the history card when it is showing, so the two never overlap.
    let history = if shell.panels.bottom.open && app.history().is_some() { crate::panels::HISTORY_H + craft_ui::layout::SHELL_GAP } else { 0.0 };
    let r = Rect::from_min_size(pos2(content.left() + INSET, content.bottom() - history - size.y - INSET), size);
    let frame = Container::new(ui.ctx(), Surface::Toast).frame(ui.ctx(), t.panel, craft_ui::frame::Side::Left);
    ui.painter().add(frame.paint(r.shrink(frame.stroke.width)));
    let dot = pos2(r.left() + 14.0, r.center().y);
    if fraction.is_some() {
        egui::Spinner::new().size(12.0).color(t.text).paint_at(ui, Rect::from_center_size(dot, vec2(12.0, 12.0)));
    } else {
        painter.circle_filled(dot, 3.0, if error { t.error } else { t.text });
    }
    painter.galley(r.min + vec2(28.0, 8.0), galley, t.text);
    let response = ui.interact(r, ui.id().with("status_chip"), Sense::click());
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &text));
    craft_ui::scenario::publish_target("status:chip", r);
    if response.on_hover_text(craft_ui::i18n::tr("status.messages")).clicked() {
        shell.messages.open = true;
    }
}
