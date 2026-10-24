//! The hot-reload host: an eframe window whose whole UI is drawn by `craft-demo-hot-lib`.
//!
//! Run the host once, then rebuild **only** the library while it runs:
//!
//! ```sh
//! cargo watch -w crates -w apps/craft-demo/src -w apps/craft-demo-hot-lib \
//!     -x 'build -p craft-demo-hot-lib'
//! cargo run -p craft-demo-hot
//! ```
//!
//! `hot-lib-reloader` swaps the library when the file changes; the subscription thread asks egui
//! to repaint, and the next frame calls the new `render`. The state and the open window stay.
//! See `docs/hot-reload.md`.

use craft_demo_hot_lib::{CraftApp, STORAGE_UI, craft_ui};
use eframe::egui;

/// Where `cargo` left `libcraft_demo_hot_lib`: next to this executable. The loader's default is
/// `<crate>/target/debug`, which a workspace member does not have (the workspace shares `target/`).
fn lib_dir() -> std::path::PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(std::path::Path::to_path_buf))
        .unwrap_or_else(|| std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")))
}

// Generated forwarding functions read the `#[unsafe(no_mangle)]` exports from the library source.
// `CraftApp` is imported here again because the generated signature names it.
#[allow(clippy::all)]
#[hot_lib_reloader::hot_module(dylib = "craft_demo_hot_lib", lib_dir = crate::lib_dir())]
mod hot {
    pub use craft_demo_hot_lib::CraftApp;
    use eframe::egui;

    hot_functions_from_file!("apps/craft-demo-hot-lib/src/lib.rs");

    #[lib_change_subscription]
    pub fn subscribe() -> hot_lib_reloader::LibReloadObserver {}
}

/// The host state: the app (shell + project) lives here, so reloading only swaps functions.
struct HotApp {
    app: CraftApp,
}

impl eframe::App for HotApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.app.logic(ctx);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        hot::render(&mut self.app, ui);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::App::save(&mut self.app, storage);
    }
}

fn main() -> eframe::Result<()> {
    let choice = craft_window::Choice::probe(craft_window::Prefix::new("CRAFT"));
    let viewport =
        craft_window::decorate(egui::ViewportBuilder::default().with_inner_size([1440.0, 900.0]).with_min_inner_size([960.0, 600.0]), choice.frame());
    let app_frame = matches!(choice.frame(), craft_window::Frame::App(_));
    let options = eframe::NativeOptions { viewport, ..Default::default() };
    eframe::run_native(
        "Craft Demo (hot)",
        options,
        Box::new(move |cc| {
            craft_ui::fonts::setup_bundled_fonts(&cc.egui_ctx);
            let stored = cc.storage.and_then(|storage| storage.get_string(STORAGE_UI));
            let app = CraftApp::new_with_frame(cc, stored.as_deref(), Some(app_frame)).map_err(std::io::Error::other)?;
            let ctx = cc.egui_ctx.clone();
            std::thread::spawn(move || {
                loop {
                    hot::subscribe().wait_for_reload();
                    ctx.request_repaint();
                }
            });
            Ok(Box::new(HotApp { app }))
        }),
    )
}
