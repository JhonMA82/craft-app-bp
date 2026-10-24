//! Desktop entry point: a native window with the demo application.

use craft_demo::CraftApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([960.0, 600.0])
            // The application bar is the window's title bar where the OS draws no chrome.
            .with_decorations(!cfg!(all(not(target_arch = "wasm32"), not(target_os = "macos")))),
        ..Default::default()
    };
    eframe::run_native(
        "Craft Demo",
        options,
        Box::new(|cc| {
            craft_ui::fonts::setup_bundled_fonts(&cc.egui_ctx);
            let stored = cc.storage.and_then(|storage| storage.get_string(craft_shell::STORAGE_UI));
            let app = CraftApp::new(cc, stored.as_deref()).map_err(std::io::Error::other)?;
            Ok(Box::new(app))
        }),
    )
}
