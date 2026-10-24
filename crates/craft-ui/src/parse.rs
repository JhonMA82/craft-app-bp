//! Field value parsing shared by the kit's numeric editors.
//!
//! Adapted from SolidCraft (`crates/ui-egui/src/prefs.rs`), MIT OR Apache-2.0. See ATTRIBUTION.md.

/// A number typed into a field: whitespace and a Unicode minus are tolerated, and a single comma
/// is read as the decimal separator ("1,5" is 1.5). Not finite: `None`.
pub fn parse_number(text: &str) -> Option<f64> {
    let t: String = text.chars().filter(|c| !c.is_whitespace()).map(|c| if c == '\u{2212}' { '-' } else { c }).collect();
    let t = if !t.contains('.') && t.matches(',').count() == 1 { t.replacen(',', ".", 1) } else { t };
    let v = t.parse::<f64>().ok().filter(|v| v.is_finite())?;
    Some(v)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::parse_number;

    #[test]
    fn parses_plain_and_localised_numbers() {
        assert_eq!(parse_number("12"), Some(12.0));
        assert_eq!(parse_number(" 1,5 "), Some(1.5));
        assert_eq!(parse_number("−3"), Some(-3.0));
        assert_eq!(parse_number("1234.5"), Some(1234.5));
        assert_eq!(parse_number("abc"), None);
        assert_eq!(parse_number("NaN"), None);
        assert_eq!(parse_number("inf"), None);
    }
}
