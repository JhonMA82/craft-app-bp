#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::{ENGLISH_MESSAGES, Locale, SPANISH_MESSAGES, language, scope, tr};

#[test]
fn interpolation_substitutes_known_arguments_and_keeps_missing_ones() {
    let locale = Locale::for_language("en");
    assert_eq!(locale.format("about.version", &[("version", "3")]), "Version 3");
    assert_eq!(locale.format("about.version", &[]), "Version {version}");
    // A value that itself contains braces is literal.
    assert_eq!(locale.format("about.version", &[("version", "{x}")]), "Version {x}");
    // An unterminated placeholder stays visible.
    assert_eq!(super::interpolate("a {b", &[]), "a {b");
}

#[test]
fn scopes_nest_and_restore_the_previous_language() {
    assert_eq!(language(), "en");
    {
        let _outer = scope("es");
        assert_eq!(language(), "es");
        assert_eq!(tr("ui.close"), "Cerrar");
        {
            let _inner = scope("en");
            assert_eq!(language(), "en");
            assert_eq!(tr("ui.close"), "Close");
        }
        assert_eq!(language(), "es");
    }
    assert_eq!(language(), "en");
    // An unknown language falls back to English rather than failing.
    let _unknown = scope("xx");
    assert_eq!(tr("ui.close"), "Close");
}

#[test]
fn labels_translate_english_text_but_not_given_names() {
    assert_eq!(Locale::for_language("en").label("Close"), "Close");
    assert_eq!(Locale::for_language("es").label("Close"), "Cerrar");
    assert_eq!(Locale::for_language("es").label("ui.close"), "Cerrar");
    // A name someone typed is not a UI label and passes through unchanged.
    assert_eq!(Locale::for_language("es").label("My design"), "My design");
    assert_eq!(super::tr_name("My design"), "My design");
}

#[test]
fn every_english_key_exists_in_every_complete_language() {
    for id in super::COMPLETE {
        let catalog = super::LOCALES.iter().find(|c| c.id == *id).expect("a complete language is registered");
        let translated = catalog.messages();
        let missing: Vec<&str> = ENGLISH_MESSAGES.keys().filter(|k| !translated.contains_key(*k)).map(String::as_str).collect();
        assert!(missing.is_empty(), "{id} is missing {}", missing.join(", "));
    }
    let extra: Vec<&str> = SPANISH_MESSAGES.keys().filter(|k| !ENGLISH_MESSAGES.contains_key(*k)).map(String::as_str).collect();
    assert!(extra.is_empty(), "es has keys en lacks: {}", extra.join(", "));
}

#[test]
fn english_catalogue_has_no_empty_values() {
    for (id, text) in ENGLISH_MESSAGES.iter() {
        assert!(!text.trim().is_empty(), "{id} is empty");
    }
}
