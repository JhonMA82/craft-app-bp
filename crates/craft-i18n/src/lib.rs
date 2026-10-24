//! Embedded offline message catalogues. UI messages use stable ids (`ui.close`, `menu.file`,
//! `cmd.shell.preferences`); [`tr`] returns the active language's text, falling back to English and
//! then to the id itself.
//!
//! Adapted from SolidCraft (`crates/ui-egui/src/i18n.rs`), MIT OR Apache-2.0. See ATTRIBUTION.md.

use std::cell::Cell;
use std::collections::BTreeMap;
use std::sync::LazyLock;

type Messages = BTreeMap<String, String>;

/// A language. Register one here after adding its catalogue; missing keys use English.
pub struct Catalog {
    pub id: &'static str,
    pub name: &'static str,
    messages: &'static LazyLock<Messages>,
}

impl Catalog {
    const fn new(id: &'static str, name: &'static str, messages: &'static LazyLock<Messages>) -> Self {
        Self { id, name, messages }
    }

    fn messages(&'static self) -> &'static Messages {
        self.messages
    }
}

fn parse(source: &str) -> Messages {
    serde_json::from_str(source).unwrap_or_default()
}

static ENGLISH_MESSAGES: LazyLock<Messages> =
    LazyLock::new(|| parse(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/locales/en.json"))));
static SPANISH_MESSAGES: LazyLock<Messages> =
    LazyLock::new(|| parse(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/locales/es.json"))));

static ENGLISH: Catalog = Catalog::new("en", "English", &ENGLISH_MESSAGES);
static SPANISH: Catalog = Catalog::new("es", "Español (España)", &SPANISH_MESSAGES);

/// English first (the fallback), then by language id.
pub const LOCALES: &[&Catalog] = &[&ENGLISH, &SPANISH];

/// Languages the catalogue must be complete in: `cargo xtask i18n --check` fails if one of them
/// misses a key English has.
pub const COMPLETE: &[&str] = &["es"];

pub fn is_supported(id: &str) -> bool {
    LOCALES.iter().any(|catalog| catalog.id == id)
}

/// An immutable locale reference. Catalogues parse once; lookups are logarithmic.
#[derive(Clone, Copy)]
pub struct Locale(&'static Catalog);

impl Locale {
    pub fn for_language(id: &str) -> Self {
        Self(LOCALES.iter().copied().find(|catalog| catalog.id == id).unwrap_or(&ENGLISH))
    }

    pub fn name(self) -> &'static str {
        self.0.name
    }

    /// Stable message id → translation → English → id.
    pub fn text(self, id: &str) -> &str {
        self.0.messages().get(id).or_else(|| ENGLISH.messages().get(id)).map_or(id, String::as_str)
    }

    /// Translate source labels from tables that still carry English text, preserving unknown
    /// labels. Also accepts stable ids.
    pub fn label(self, label: &str) -> &str {
        static LABEL_IDS: LazyLock<BTreeMap<&'static str, &'static str>> = LazyLock::new(|| {
            ENGLISH.messages().iter().filter(|(_, text)| !text.contains('{')).map(|(id, text)| (text.as_str(), id.as_str())).collect()
        });
        let ids = &*LABEL_IDS;
        ids.get(label).map_or_else(|| self.text(label), |id| self.text(id))
    }

    pub fn format(self, id: &str, args: &[(&str, &str)]) -> String {
        interpolate(self.text(id), args)
    }
}

/// One-pass substitution: argument values are literal, even when they contain braces.
/// Missing arguments remain visible rather than being silently removed.
fn interpolate(template: &str, args: &[(&str, &str)]) -> String {
    let mut output = String::with_capacity(template.len());
    let mut remaining = template;
    while let Some((before, after)) = remaining.split_once('{') {
        output.push_str(before);
        let Some((name, rest)) = after.split_once('}') else {
            output.push('{');
            output.push_str(after);
            return output;
        };
        if let Some((_, value)) = args.iter().find(|(key, _)| *key == name) {
            output.push_str(value);
        } else {
            output.push('{');
            output.push_str(name);
            output.push('}');
        }
        remaining = rest;
    }
    output.push_str(remaining);
    output
}

thread_local! {
    // Helpers without an app reference share a render scope. Nested renders restore their
    // caller's locale; no owned language strings or persistent global setting.
    static CURRENT: Cell<Locale> = const { Cell::new(Locale(&ENGLISH)) };
}

#[must_use]
pub struct LanguageScope {
    previous: Locale,
    // Drop on the thread where the scope was entered.
    _thread: std::marker::PhantomData<std::rc::Rc<()>>,
}

/// Draw with `id` active for the rest of this scope; the previous language is restored on drop.
pub fn scope(id: &str) -> LanguageScope {
    let previous = CURRENT.with(|current| current.replace(Locale::for_language(id)));
    LanguageScope { previous, _thread: std::marker::PhantomData }
}

impl Drop for LanguageScope {
    fn drop(&mut self) {
        CURRENT.with(|current| current.set(self.previous));
    }
}

/// This thread's language, to carry into a worker thread with [`scope`].
pub fn language() -> &'static str {
    CURRENT.with(|current| current.get().0.id)
}

pub fn tr(id: &str) -> &str {
    CURRENT.with(|current| current.get().text(id))
}

/// Translate a label from data or a generated table, preserving unknown labels.
pub fn tr_label(label: &str) -> &str {
    CURRENT.with(|current| current.get().label(label))
}

pub fn format(id: &str, args: &[(&str, &str)]) -> String {
    CURRENT.with(|current| current.get().format(id, args))
}

/// Translate generated label text (falls back to the text itself).
pub fn tr_dyn(text: &str) -> String {
    tr_label(text).to_string()
}

/// A name someone gave: shown exactly as typed, never translated.
pub fn tr_name(text: &str) -> String {
    text.to_string()
}

#[cfg(test)]
mod tests;
