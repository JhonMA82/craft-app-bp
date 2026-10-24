//! The i18n gate: every English key exists in each complete language, every literal `tr("…")` /
//! `format("…")` id exists in English, and no language invents keys English lacks.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn catalogs(root: &Path) -> Result<Vec<(String, BTreeSet<String>)>, String> {
    let dir = root.join("assets/locales");
    let mut out = Vec::new();
    let mut files: Vec<PathBuf> =
        std::fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))?.filter_map(Result::ok).map(|entry| entry.path()).collect();
    files.sort();
    for path in files.iter().filter(|path| path.extension().is_some_and(|e| e == "json")) {
        let id = path.file_stem().and_then(|s| s.to_str()).ok_or("bad locale file name")?.to_string();
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let value: serde_json::Value = serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        let object = value.as_object().ok_or_else(|| format!("{}: not an object of ids", path.display()))?;
        for (key, value) in object {
            if !value.is_string() {
                return Err(format!("{}: `{key}` is not a string", path.display()));
            }
        }
        out.push((id, object.keys().cloned().collect()));
    }
    Ok(out)
}

/// Ids the source passes to `tr`/`format` as a literal.
fn used_ids(root: &Path) -> Result<BTreeSet<String>, String> {
    let mut ids = BTreeSet::new();
    let mut files = Vec::new();
    for dir in ["crates", "apps"] {
        collect(&root.join(dir), &mut files)?;
    }
    for path in files {
        let source = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        for line in source.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") || trimmed.starts_with('*') {
                continue;
            }
            for needle in ["tr(\"", "format(\""] {
                let mut rest = line;
                while let Some(index) = rest.find(needle) {
                    // Only a whole identifier: `push_str("…")` is not `tr("…")`.
                    let whole = index == 0 || !line.as_bytes().get(index - 1).is_some_and(|c| c.is_ascii_alphanumeric() || *c == b'_');
                    rest = &rest[index + needle.len()..];
                    let Some(end) = rest.find('"') else { break };
                    if whole {
                        ids.insert(rest[..end].to_string());
                    }
                    rest = &rest[end + 1..];
                }
            }
        }
    }
    Ok(ids)
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let Ok(entries) = std::fs::read_dir(dir) else { return Ok(()) };
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            if !matches!(entry.file_name().to_str(), Some("target" | ".git")) {
                collect(&path, out)?;
            }
        } else if path.extension().is_some_and(|extension| extension == "rs") && path.components().any(|component| component.as_os_str() == "src") {
            out.push(path);
        }
    }
    Ok(())
}

pub fn run(root: &Path) -> Result<(), String> {
    let catalogs = catalogs(root)?;
    let english = catalogs.iter().find(|(id, _)| id == "en").map(|(_, keys)| keys).ok_or("assets/locales/en.json is missing")?;
    let mut failures = Vec::new();
    for (id, keys) in &catalogs {
        if id == "en" {
            continue;
        }
        let missing: Vec<&str> = english.difference(keys).map(String::as_str).collect();
        let extra: Vec<&str> = keys.difference(english).map(String::as_str).collect();
        if !missing.is_empty() {
            failures.push(format!("{id}: missing {} ({})", missing.len(), missing.join(", ")));
        }
        if !extra.is_empty() {
            failures.push(format!("{id}: not in English ({})", extra.join(", ")));
        }
    }
    let used = used_ids(root)?;
    let unknown: Vec<&str> = used.difference(english).map(String::as_str).collect();
    if !unknown.is_empty() {
        failures.push(format!("used in code but not in en.json: {}", unknown.join(", ")));
    }
    if !failures.is_empty() {
        return Err(format!("i18n catalogues disagree:\n{}", failures.join("\n")));
    }
    println!("i18n: en/es complete");
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    #[test]
    fn the_workspace_catalogues_are_complete() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        run(root).unwrap();
    }

    #[test]
    fn a_missing_translation_or_an_unknown_id_fails() {
        let unique = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let root = std::env::temp_dir().join(format!("craft-i18n-gate-{}-{unique}", std::process::id()));
        std::fs::create_dir_all(root.join("assets/locales")).unwrap();
        std::fs::create_dir_all(root.join("crates/x/src")).unwrap();
        std::fs::write(root.join("assets/locales/en.json"), r#"{"ui.ok":"OK","ui.close":"Close"}"#).unwrap();
        std::fs::write(root.join("assets/locales/es.json"), r#"{"ui.ok":"Aceptar"}"#).unwrap();
        std::fs::write(root.join("crates/x/src/lib.rs"), "fn f() { tr(\"ui.ok\"); tr(\"ui.gone\"); }\n").unwrap();
        let error = run(&root).unwrap_err();
        assert!(error.contains("missing 1 (ui.close)"), "{error}");
        assert!(error.contains("ui.gone"), "{error}");
        std::fs::write(root.join("assets/locales/es.json"), r#"{"ui.ok":"Aceptar","ui.close":"Cerrar"}"#).unwrap();
        std::fs::write(root.join("crates/x/src/lib.rs"), "fn f() { tr(\"ui.ok\"); }\n").unwrap();
        run(&root).unwrap();
        std::fs::remove_dir_all(root).unwrap();
    }
}
