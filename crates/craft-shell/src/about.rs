//! Help › About: what this build is, its licence and where its UI comes from.

use craft_ui::i18n::{format, tr};
use craft_ui::theme::Tokens;
use craft_ui::ui_kit::{self, Action, Text, Window};
use egui::{Align2, Id, Ui, Vec2};

use crate::Shell;

pub fn show(ui: &mut Ui, shell: &mut Shell, title: &str) {
    if !shell.about_open {
        return;
    }
    let t = Tokens::get();
    let ctx = ui.ctx().clone();
    let commit = option_env!("CRAFT_COMMIT").unwrap_or("unknown");
    let (closed, action) = Window::new(Id::new("craft_about"), "info", tr("menu.about"))
        .cards(3.0)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .actions(tr("ui.close"), "", None)
        .show(&ctx, true, |ui| {
            ui.set_width(t.form_card_width + t.space_24 * 2.0);
            ui.label(egui::RichText::new(title).font(craft_ui::fonts::semibold(t.font_title)).color(t.form_ink));
            ui_kit::text(ui, &format("about.version", &[("version", env!("CARGO_PKG_VERSION"))]), Text::Note("about_version"));
            ui_kit::text(ui, &format("about.commit", &[("commit", commit)]), Text::Note("about_commit"));
            ui_kit::text(ui, tr("about.license"), Text::Note("about_license"));
            ui.add_space(t.space_8);
            ui_kit::text(ui, tr("about.attribution"), Text::Hint("about_attribution"));
        });
    if closed || matches!(action, Action::Ok) {
        shell.about_open = false;
    }
}
