//! A headless harness: the real shell and app drawn frame by frame into an egui context without a
//! window, driven by pointer, key and text events, with widgets found through accesskit.

use craft_shell::{Shell, ShellApp};
use egui::{Event, Key, Modifiers, PointerButton, Pos2, Rect, Vec2};

pub struct Harness<A: ShellApp> {
    pub app: A,
    pub shell: Shell,
    pub ctx: egui::Context,
    pub output: egui::PlatformOutput,
    time: f64,
    size: Vec2,
    events: Vec<Event>,
}

impl<A: ShellApp> Harness<A> {
    /// Build the app and shell, install the bundled fonts and settle three frames.
    pub fn new(build: impl FnOnce() -> (A, Shell)) -> Self {
        let ctx = egui::Context::default();
        craft_ui::fonts::setup_bundled_fonts(&ctx);
        // Layout must be settled before a test reads a widget's position and clicks it.
        ctx.global_style_mut(|style| style.animation_time = 0.0);
        ctx.enable_accesskit();
        let (app, shell) = build();
        let mut harness = Self { app, shell, ctx, output: Default::default(), time: 0.0, size: Vec2::new(1600.0, 1000.0), events: Vec::new() };
        harness.frames(3);
        harness
    }

    /// The window size the harness draws into.
    pub fn size(&mut self, size: Vec2) {
        self.size = size;
    }

    pub fn frame(&mut self) {
        self.time += 1.0 / 60.0;
        let raw = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, self.size)),
            time: Some(self.time),
            focused: true,
            events: std::mem::take(&mut self.events),
            ..Default::default()
        };
        let mut output = self.ctx.run_ui(raw, |ui| {
            self.shell.ui(&mut self.app, ui);
        });
        output.textures_delta.clear();
        self.output = output.platform_output;
    }

    pub fn frames(&mut self, n: usize) {
        for _ in 0..n {
            self.frame();
        }
    }

    /// Run frames until egui stops asking for another pass (bounded), so a widget's published
    /// position is the one a click will land on.
    pub fn settle(&mut self) {
        for _ in 0..8 {
            self.frame();
            if !self.ctx.has_requested_repaint() {
                break;
            }
        }
    }

    /// The centre of the visible widget whose label or value is `text` (the last drawn wins).
    pub fn widget(&self, text: &str) -> Option<[f64; 2]> {
        self.widgets(text).into_iter().next()
    }

    /// The widget showing `text` nearest the label `near` (the field in a label's row).
    pub fn widget_near(&self, text: &str, near: &str) -> Option<[f64; 2]> {
        let [lx, ly] = self.widgets(near).into_iter().next()?;
        self.widgets(text).into_iter().filter(|[x, _]| *x > lx).min_by(|a, b| {
            let d = |p: &[f64; 2]| (p[1] - ly).abs() * 4.0 + (p[0] - lx).abs();
            d(a).total_cmp(&d(b))
        })
    }

    fn widgets(&self, text: &str) -> Vec<[f64; 2]> {
        let Some(tree) = self.output.accesskit_update.as_ref() else { return Vec::new() };
        let centre =
            |node: &egui::accesskit::Node| node.bounds().filter(|b| b.x1 > b.x0 && b.y1 > b.y0).map(|b| [(b.x0 + b.x1) / 2.0, (b.y0 + b.y1) / 2.0]);
        let exact: Vec<[f64; 2]> = tree
            .nodes
            .iter()
            .rev()
            .filter(|(_, node)| node.label() == Some(text) || node.value() == Some(text))
            .filter_map(|(_, node)| centre(node))
            .collect();
        if !exact.is_empty() {
            return exact;
        }
        // A menu button's label ends with its shortcut ("Save Ctrl+S").
        tree.nodes
            .iter()
            .rev()
            .filter(|(_, node)| {
                node.role() == egui::accesskit::Role::Button
                    && node.label().is_some_and(|label| label.strip_prefix(text).is_some_and(|rest| rest.starts_with(' ')))
            })
            .filter_map(|(_, node)| centre(node))
            .collect()
    }

    /// Move the pointer there, press, release and settle.
    pub fn click(&mut self, at: [f64; 2]) {
        let pos = Pos2::new(at[0] as f32, at[1] as f32);
        self.events.push(Event::PointerMoved(pos));
        self.frame();
        self.events.push(Event::PointerButton { pos, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE });
        self.frame();
        self.events.push(Event::PointerButton { pos, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE });
        self.frames(2);
    }

    /// Click the widget labelled `text`; false when it is not on screen.
    pub fn click_widget(&mut self, text: &str) -> bool {
        // A window or section may still be resizing from the previous action; settle first so
        // the position read is the one the click lands on.
        self.settle();
        let Some(at) = self.widget(text) else { return false };
        self.click(at);
        true
    }

    /// Press and release a key (with modifiers).
    pub fn key(&mut self, key: Key, modifiers: Modifiers) {
        self.events.push(Event::Key { key, physical_key: None, pressed: true, repeat: false, modifiers });
        self.frame();
        self.events.push(Event::Key { key, physical_key: None, pressed: false, repeat: false, modifiers });
        self.frame();
    }

    /// Type text into the focused field.
    pub fn text(&mut self, text: &str) {
        self.events.push(Event::Text(text.to_owned()));
        self.frames(2);
    }

    /// Where a published scenario handle was drawn last frame.
    pub fn handle(&self, name: &str) -> Option<Pos2> {
        craft_ui::scenario::handle_at(name)
    }

    /// Where a published scenario rect was drawn last frame.
    pub fn rect(&self, name: &str) -> Option<Rect> {
        craft_ui::scenario::rect_of(name)
    }
}
