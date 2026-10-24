//! Transient notifications: at most four on screen for four seconds, newest at the bottom.

use std::collections::VecDeque;
use std::time::Duration;

use craft_ui::theme::Tokens;
use craft_ui::ui_kit::{Container, Surface};
use egui::{Align2, RichText, vec2};

const KEEP: usize = 4;
const LIFETIME_MS: f64 = 4000.0;

#[derive(Clone, Debug)]
pub struct Toast {
    pub text: String,
    pub error: bool,
    pub at: f64,
}

#[derive(Default)]
pub struct Toasts {
    queue: VecDeque<Toast>,
}

impl Toasts {
    /// Add a toast, keeping at most [`KEEP`] (the oldest go).
    pub fn push(&mut self, text: impl Into<String>, error: bool, at: f64) {
        self.queue.push_back(Toast { text: text.into(), error, at });
        while self.queue.len() > KEEP {
            self.queue.pop_front();
        }
    }

    pub fn error(&mut self, text: impl Into<String>, at: f64) {
        self.push(text, true, at);
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Drop expired toasts and draw the rest above the status bar's right corner.
    pub fn show(&mut self, ctx: &egui::Context, now: f64) {
        self.queue.retain(|toast| now - toast.at < LIFETIME_MS);
        let Some(left) = self.queue.iter().map(|toast| (LIFETIME_MS - (now - toast.at)).max(0.0)).reduce(f64::min) else {
            return;
        };
        // eframe does not repaint an idle window; without this the cards stay after their life.
        ctx.request_repaint_after(Duration::from_millis(left as u64 + 1));
        let t = Tokens::get();
        let screen = ctx.content_rect();
        Container::new(ctx, Surface::Toast)
            .area(egui::Id::new("toasts"))
            .fixed_pos(screen.right_bottom() - vec2(2.0, 2.0))
            .pivot(Align2::RIGHT_BOTTOM)
            .show(ctx, |ui| {
                ui.set_max_width((t.form_card_width * 2.0).min(screen.width() - t.space_24));
                ui.spacing_mut().item_spacing.y = t.space_4;
                let mut dismissed = None;
                for (index, toast) in self.queue.iter().enumerate() {
                    let ink = if toast.error { t.error } else { t.text };
                    let response =
                        ui.add(egui::Label::new(RichText::new(&toast.text).size(t.font_body).color(ink)).wrap().sense(egui::Sense::click()));
                    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, true, &toast.text));
                    if response.clicked() {
                        dismissed = Some(index);
                    }
                }
                if let Some(index) = dismissed
                    && let Some(toast) = self.queue.remove(index)
                {
                    log::debug!("toast dismissed: {}", toast.text);
                }
            });
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::{KEEP, Toasts};

    #[test]
    fn the_queue_keeps_the_newest_four() {
        let mut toasts = Toasts::default();
        for i in 0..KEEP + 2 {
            toasts.push(format!("toast {i}"), false, 0.0);
        }
        assert_eq!(toasts.len(), KEEP);
        assert_eq!(toasts.queue.front().map(|t| t.text.as_str()), Some("toast 2"));
    }
}
