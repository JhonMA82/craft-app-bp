//! Every status message of this run, kept so it can be read and copied after the status line has
//! moved on. The status chip and the Messages command open the window.
//!
//! Adapted from SolidCraft (`crates/ui-egui/src/messages.rs`), MIT OR Apache-2.0. See ATTRIBUTION.md.

use std::collections::VecDeque;
use std::time::Duration;

use craft_ui::i18n::{format, tr};
use craft_ui::theme::Tokens;
use craft_ui::ui_kit::{self, Window};
use egui::{RichText, vec2};

/// Messages kept (the oldest go first).
pub const KEEP: usize = 200;
/// The longest message kept, in characters (a longer one is cut, saying so).
const MAX_CHARS: usize = 4000;

/// One status message.
#[derive(Clone, Debug, PartialEq)]
pub struct Message {
    pub text: String,
    pub error: bool,
    /// When it last came (ms since the Unix epoch).
    pub at: f64,
    /// How many times in a row it came.
    pub count: u32,
}

#[derive(Default)]
pub struct Messages {
    list: VecDeque<Message>,
    pub open: bool,
}

impl Messages {
    /// Keep a message: the same one again in a row counts up instead.
    pub fn push(&mut self, text: &str, error: bool, at: f64) {
        if let Some(last) = self.list.back_mut()
            && last.text == text
            && last.error == error
        {
            last.count = last.count.saturating_add(1);
            last.at = at;
            return;
        }
        let text =
            if text.chars().count() > MAX_CHARS { format!("{}…", text.chars().take(MAX_CHARS).collect::<String>()) } else { text.to_string() };
        self.list.push_back(Message { text, error, at, count: 1 });
        while self.list.len() > KEEP {
            self.list.pop_front();
        }
    }

    /// Oldest first.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &Message> + ExactSizeIterator {
        self.list.iter()
    }

    pub fn clear(&mut self) {
        self.list.clear();
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    pub fn errors(&self) -> usize {
        self.list.iter().filter(|m| m.error).count()
    }

    /// The messages as text, oldest first, one per line: how long ago, `error:` for errors, the
    /// text and the repeat count.
    pub fn as_text(&self, now: f64) -> String {
        self.list.iter().map(|m| format!("{}\t{}", ago(now, m.at), line(m))).collect::<Vec<_>>().join("\n")
    }

    /// The Messages window. Returns true when it closed this frame.
    pub fn show(&mut self, ui: &mut egui::Ui) -> bool {
        if !self.open {
            return false;
        }
        let ctx = ui.ctx().clone();
        let t = Tokens::get();
        let now = crate::now_ms();
        let closed = Window::new(egui::Id::new("craft_messages"), "command", tr("status.messages"))
            .cards(3.0)
            .actions("", "", None)
            .keep_open_on_escape()
            .show_body(&ctx, |ui| {
                ui.horizontal(|ui| {
                    let copy = ui_kit::button(ui, tr("ui.copy"), ui_kit::Button::Enabled(!self.list.is_empty()));
                    if copy.clicked() {
                        ctx.copy_text(self.as_text(now));
                    }
                    let clear = ui_kit::button(ui, tr("status.clear"), ui_kit::Button::Enabled(!self.list.is_empty()));
                    if clear.clicked() {
                        self.clear();
                    }
                });
                ui.separator();
                if self.list.is_empty() {
                    ui.label(RichText::new(tr("ui.none")).color(t.text_dim));
                    return;
                }
                egui::ScrollArea::vertical().max_height(t.form_card_width * 2.0).auto_shrink([false, true]).show(ui, |ui| {
                    for message in self.list.iter().rev() {
                        ui.horizontal_top(|ui| {
                            ui.add_sized(vec2(70.0, 14.0), egui::Label::new(RichText::new(ago(now, message.at)).small().color(t.text_dim)));
                            let text = RichText::new(line(message)).color(if message.error { t.error } else { t.text });
                            ui.add(egui::Label::new(text).wrap().selectable(true));
                        });
                    }
                });
            });
        if closed {
            self.open = false;
        }
        // The ages tick on while it is open.
        ctx.request_repaint_after(Duration::from_secs(1));
        closed
    }
}

/// A message as one line: `error:` for errors, and `(×n)` when it came more than once.
fn line(m: &Message) -> String {
    let repeats = if m.count > 1 { format!(" (×{})", m.count) } else { String::new() };
    format!("{}{}{repeats}", if m.error { "error: " } else { "" }, m.text)
}

/// How long ago `at` was, at `now` (both ms): seconds, minutes or hours.
fn ago(now: f64, at: f64) -> String {
    let s = ((now - at) / 1000.0).max(0.0).floor();
    let (id, n) = if s < 60.0 {
        ("status.ago_s", s)
    } else if s < 3600.0 {
        ("status.ago_min", (s / 60.0).floor())
    } else {
        ("status.ago_h", (s / 3600.0).floor())
    };
    format(id, &[("n", &format!("{n:.0}"))])
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::{KEEP, Messages};

    #[test]
    fn repeats_count_up_and_the_oldest_go_past_the_limit() {
        let mut m = Messages::default();
        m.push("Note added", false, 1000.0);
        m.push("Note added", false, 2000.0);
        m.push("That failed", true, 3000.0);
        let got: Vec<(&str, bool, u32)> = m.iter().map(|x| (x.text.as_str(), x.error, x.count)).collect();
        assert_eq!(got, [("Note added", false, 2), ("That failed", true, 1)]);
        for i in 0..KEEP + 5 {
            m.push(&format!("message {i}"), false, 0.0);
        }
        assert_eq!(m.iter().len(), KEEP);
        assert_eq!(m.iter().next().map(|x| x.text.as_str()), Some("message 5"), "the oldest went first");
    }

    #[test]
    fn a_very_long_message_is_cut() {
        let mut m = Messages::default();
        m.push(&"x".repeat(10_000), true, 0.0);
        assert_eq!(m.iter().next().map(|x| x.text.chars().count()), Some(4001));
    }
}
