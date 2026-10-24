//! Handles, rectangles and counts published while a frame is drawn: what headless tests click
//! and assert on. A handle is where an affordance was drawn last frame; a count is a number
//! about that frame's drawing.
//!
//! Adapted from SolidCraft (`crates/ui-egui/src/scenario.rs`), MIT OR Apache-2.0. See ATTRIBUTION.md.

use std::cell::RefCell;

thread_local! {
    static HANDLES: RefCell<Vec<(String, egui::Pos2)>> = const { RefCell::new(Vec::new()) };
    static RECTS: RefCell<Vec<(String, egui::Rect)>> = const { RefCell::new(Vec::new()) };
    static COUNTS: RefCell<Vec<(String, f64)>> = const { RefCell::new(Vec::new()) };
}

/// Publish an interaction handle drawn this frame at `at` (screen), by name.
pub fn publish_handle(name: &str, at: egui::Pos2) {
    // The latest wins (lookups search from the end), so publishing is cheap for rows.
    HANDLES.with(|h| h.borrow_mut().push((name.to_string(), at)));
}

/// Publish a rectangle and its centred interaction handle with the same identity.
pub fn publish_target(name: &str, rect: egui::Rect) {
    publish_target_as(name, name, rect);
}

/// Publish centred geometry with explicit, independently named handle and rectangle aliases.
pub fn publish_target_as(handle: &str, rect_name: &str, rect: egui::Rect) {
    publish_handle(handle, rect.center());
    publish_rect(rect_name, rect);
}

/// Where a handle was drawn last frame.
pub fn handle_at(name: &str) -> Option<egui::Pos2> {
    HANDLES.with(|h| h.borrow().iter().rfind(|(n, _)| n == name).map(|x| x.1))
}

/// Every handle drawn last frame whose name starts with `prefix`.
pub fn handles(prefix: &str) -> Vec<(String, egui::Pos2)> {
    HANDLES.with(|h| h.borrow().iter().filter(|(n, _)| n.starts_with(prefix)).cloned().collect())
}

/// A painted part's rect this frame, by name ("ribbon", "status").
pub fn publish_rect(name: &str, r: egui::Rect) {
    RECTS.with(|h| h.borrow_mut().push((name.to_string(), r)));
}

pub fn rect_of(name: &str) -> Option<egui::Rect> {
    RECTS.with(|h| h.borrow().iter().rfind(|(n, _)| n == name).map(|x| x.1))
}

/// A number about this frame's drawing, by name.
pub fn publish_count(name: &str, v: f64) {
    COUNTS.with(|h| h.borrow_mut().push((name.to_string(), v)));
}

pub fn count_of(name: &str) -> Option<f64> {
    COUNTS.with(|h| h.borrow().iter().rfind(|(n, _)| n == name).map(|x| x.1))
}

/// Forget last frame's handles, rects and counts (called as a frame starts).
pub fn clear_handles() {
    HANDLES.with(|h| h.borrow_mut().clear());
    RECTS.with(|h| h.borrow_mut().clear());
    COUNTS.with(|h| h.borrow_mut().clear());
}
