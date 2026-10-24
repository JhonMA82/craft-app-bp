//! Browser entry point: Trunk builds this crate for wasm32 and starts the same `CraftApp`.

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast as _;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub async fn wasm_start() -> Result<(), wasm_bindgen::JsValue> {
    eframe::WebLogger::init(log::LevelFilter::Info).ok();
    let document = web_sys::window().ok_or("no window")?.document().ok_or("no document")?;
    let canvas = document.get_element_by_id("craft_canvas").ok_or("no canvas")?.dyn_into::<web_sys::HtmlCanvasElement>()?;
    let options = eframe::WebOptions { renderer: eframe::Renderer::Glow, ..Default::default() };
    eframe::WebRunner::new()
        .start(
            canvas,
            options,
            Box::new(|cc| {
                craft_demo::craft_ui::fonts::setup_bundled_fonts(&cc.egui_ctx);
                let stored = cc.storage.and_then(|storage| storage.get_string(craft_demo::craft_shell::STORAGE_UI));
                let app = craft_demo::CraftApp::new(cc, stored.as_deref()).map_err(std::io::Error::other)?;
                Ok(Box::new(app))
            }),
        )
        .await
}

#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eprintln!("craft-demo-web only runs in the browser: build it with `trunk build --release`");
}
