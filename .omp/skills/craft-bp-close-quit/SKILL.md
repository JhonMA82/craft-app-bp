---
name: craft-bp-close-quit
description: "Implement window close and application quit correctly in a Craft Boilerplate eframe app: close_requested, CancelClose, unsaved-change prompts, the shell quit request, craft-quit on_close_request last, deferred close after async saves, and on_exit persistence. Trigger: close the window, quit does not work, app crashes on quit, macOS Cmd+Q, unsaved changes prompt, CancelClose, close button, on_exit, save before exit, quit request, DeferredClose."
---

# Close and quit

The close path has three independent inputs that must converge:

1. the OS/window close button or Cmd+Q (`ctx.input(|i| i.viewport().close_requested())`),
2. the app's own Quit command (`shell.quit` → a quit request),
3. a platform-specific quit (macOS must leave through `-[NSApplication terminate:]`).

`craft-quit` (from `craft-libs`) owns (3). The app owns the decision. The shell must not.

## The rule

- The shell **records** a quit request; it does not send `ViewportCommand::Close` itself.
- The app's `eframe::App::logic` consumes the request and decides. If the app can close, it sends
  `ViewportCommand::Close`; the next `logic` frame sees `close_requested()` and handles it.
- The app's own close decision runs before `craft_quit::on_close_request(ctx)`, which must be the
  **last** call in `logic`. It sees a `CancelClose` the app sent this frame and does nothing.
- `logic` runs even when the window is minimized or hidden, where `ui` does not. Keep the decision
  and any unsaved check in `logic`, never only in `ui`.
- Persist in `eframe::App::save`/`on_exit`, not after `run_native`; the process exits inside the
  loop on macOS and code after `run_native` never runs. The demo flushes `config.toml` in `save`.

## Current shape in this repo

The contract is already implemented:

- `Shell::ui` does **not** send `ViewportCommand::Close`; it only records the request.
- `Shell::take_quit_request()` consumes it once (`std::mem::take`).
- `ShellApp::can_close`/`close_denied` are the app's decision hooks.
- `CraftApp::logic` runs the OS close check, consumes the shell request, then calls
  `craft_quit::on_close_request` last.
- `install_app_menu` is called from `CraftApp::new_with_frame` (no-op outside macOS).

Keep that shape. In `CraftApp::logic` (or the app's host):

```rust
pub fn logic(&mut self, ctx: &egui::Context) {
    if ctx.input(|i| i.viewport().close_requested()) && !self.can_close() {
        ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        self.show_save_prompt = true;     // rendered in ui(), app state
    }
    if self.shell.take_quit_request() {
        if self.can_close() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        } else {
            self.show_save_prompt = true;
        }
    }
    craft_quit::on_close_request(ctx);    // last
}
```

Do not reset the prompt flag before the user answers. Reset it on Cancel, a cancelled file picker,
a failed save, or when a new document is opened.

## Unsaved changes and deferred close

`DeferredClose` means: the user asked to close; an async save is still running; do not send
`Close` until it resolves.

- Keep `PendingClose { requested: bool }` in app state (not egui memory).
- On close request: `CancelClose`, show the prompt or start the save, and keep the flag.
- When the save future completes successfully, clear the dirty flag and send `Close` from the next
  `logic` (eframe commands are queued for the next frame).
- If the save fails or is cancelled, clear the pending flag and keep the window open.
- If the window is hidden/minimized while the prompt is pending, bring it back before showing the
  prompt; otherwise the user cannot answer and the app appears stuck.
- Multiple rapid close requests must be idempotent: do not spawn a save per frame.

## macOS specifics

- Never exit by only closing the window on macOS (AppKit Touch Bar KVO crash on newer systems).
  `craft_quit::on_close_request` cancels the close and schedules
  `-[NSApplication terminate:]`.
- `craft_quit::install_app_menu(ctx, app_name)` must be called once from the eframe app creator
  (before winit replaces the menu). It replaces the whole native menu and sets muda's process-wide
  handler; an app with its own muda menu must instead give Quit `craft_quit::QUIT_MENU_ID` and
  forward events to `craft_quit::on_menu_event`.
- `on_menu_event` sends `ViewportCommand::Close` to the **root** viewport, so the app's unsaved
  check runs, even when Quit came from a child viewport.
- Code after `eframe::run_native` does not run; move cleanup/save into `on_exit`.

## Platforms without a dedicated quit

On Linux/Windows `craft_quit::quit()` returns `false`; `on_close_request` leaves the close alone
and eframe exits normally. The app still runs its unsaved check and sends `CancelClose` when
needed.

## Verification

1. Unit-test the decision helper with egui's `Context` and `RawInput`, not a real window:
   push `ViewportEvent::Close` into `input.viewports` for `ViewportId::ROOT`, run frames, and
   inspect `FullOutput.viewport_output[ROOT].commands`.
   Assert: dirty + close → `CancelClose` and no exit; clean + close → one `Close`/takeover;
   shell quit request alone → no command from `Shell::ui`.
2. Headless shell test: run `shell.run(app, "shell.quit")`, assert `shell.take_quit_request()` is
   true exactly once and the shell issued no `ViewportCommand::Close`.
3. Manual: close button, Cmd+Q/Alt+F4, the Quit command; dirty and clean; Cancel on the prompt;
   minimize while the prompt is open.
4. Manual macOS (cannot be proven in Linux CI): Cmd+Q with unsaved work prompts and exits without
   a crash; the app menu's Quit asks first.
5. Run `cargo xtask ci`; wasm is unaffected unless you changed `apps/*-web` (then `cargo xtask web`).

## Traps

- **Two Close senders:** shell + app both sending Close. Pick one (the app).
- **CancelClose suppression:** `craft_quit::on_close_request` sees `CancelClose` in
  `viewport(|v| v.commands)`; if you send it after the hook, the hook may quit despite the prompt.
  Order matters: app first, craft-quit last.
- **Logic vs ui:** a close check in `ui` is skipped while minimized and runs after the quit
  decision. Move it to `logic`.
- **Prompt in a closure:** store prompt state in the app; do not hold a closure across frames.
- **Async save:** sending `Close` before the save resolves loses data; a failed save must not quit.
- **Child viewport:** closing a child is not quitting the app; only the root close/quit ends it.
- **`on_exit` panic:** `on_exit` must not unwrap a failing save; report and let the process exit.
- **Harness limitation:** `craft_test::Harness` drives `Shell::ui`, not `eframe::App::logic`; test
  the host logic separately with an egui `Context` if needed.

## Do not

- Do not call `std::process::exit` from a frame or from a menu handler.
- Do not use `NSApplication terminate` directly in app code; call `craft_quit`.
- Do not show a blocking native dialog in the same callback that initiates quit on macOS.
- Do not put the unsaved-changes check in `ui` only, or after `craft_quit`.
- Do not add a second quit flag in the shell and the app without a single consumer.
