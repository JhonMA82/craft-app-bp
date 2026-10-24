//! One click, one side effect. egui may run several passes of the same frame (a layout change
//! asks for a discard), and every pass sees the same input, so a response reports the same click
//! again. Mutating state from a click must be de-duplicated per pass.

/// Whether `key`'s side effect is the first one handled this pass. Call it after a response
/// reports a click or drag; the first caller for `key` in a pass gets `true`.
pub fn once_this_pass(ctx: &egui::Context, key: &str) -> bool {
    let pass = ctx.cumulative_pass_nr();
    let id = egui::Id::new(("craft_click_once", key));
    if ctx.data(|data| data.get_temp::<u64>(id)) == Some(pass) {
        return false;
    }
    ctx.data_mut(|data| data.insert_temp(id, pass));
    true
}
