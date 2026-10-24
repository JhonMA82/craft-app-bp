//! Desktop entry point: a native window with the demo application.

use craft_demo::CraftApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    // One decision for every platform: the desktop draws the frame unless it cannot
    // (GNOME/Wayland without server-side decorations), in which case the app bar is the titlebar.
    let choice = craft_window::Choice::probe(craft_window::Prefix::new("CRAFT"));
    let viewport =
        craft_window::decorate(egui::ViewportBuilder::default().with_inner_size([1440.0, 900.0]).with_min_inner_size([960.0, 600.0]), choice.frame());
    let app_frame = matches!(choice.frame(), craft_window::Frame::App(_));
    let options = eframe::NativeOptions { viewport, ..Default::default() };
    eframe::run_native(
        "Craft Demo",
        options,
        Box::new(move |cc| {
            craft_ui::fonts::setup_bundled_fonts(&cc.egui_ctx);
            let stored = cc.storage.and_then(|storage| storage.get_string(craft_shell::STORAGE_UI));
            let app = CraftApp::new_with_frame(cc, stored.as_deref(), Some(app_frame)).map_err(std::io::Error::other)?;
            Ok(Box::new(app))
        }),
    )
}
