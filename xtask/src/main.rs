//! Repository gates. Run with `cargo xtask <command>`.
//!
//! Adapted from SolidCraft's xtask (MIT OR Apache-2.0). See ATTRIBUTION.md.

use std::path::{Path, PathBuf};
use std::process::Command;

mod i18n;
mod layers;
mod ui_containers;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().map(Path::to_path_buf).unwrap_or_else(|| PathBuf::from("."))
}

fn cargo(root: &Path, args: &[&str]) -> Result<(), String> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    println!("$ cargo {}", args.join(" "));
    let status = Command::new(cargo).args(args).current_dir(root).status().map_err(|e| format!("cargo {}: {e}", args.join(" ")))?;
    if status.success() { Ok(()) } else { Err(format!("cargo {} failed ({status})", args.join(" "))) }
}

fn layers(root: &Path) -> Result<(), String> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let output = Command::new(cargo)
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .current_dir(root)
        .output()
        .map_err(|e| format!("cargo metadata: {e}"))?;
    if !output.status.success() {
        return Err(format!("cargo metadata failed ({})", output.status));
    }
    let meta: serde_json::Value = serde_json::from_slice(&output.stdout).map_err(|e| format!("cargo metadata: {e}"))?;
    let crates = layers::from_metadata(&meta)?;
    let violations = layers::check(&crates);
    if !violations.is_empty() {
        return Err(format!("crate layers:\n{}", violations.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n")));
    }
    println!("layers: ok");
    Ok(())
}

fn ci(root: &Path) -> Result<(), String> {
    cargo(root, &["fmt", "--check"])?;
    ui_containers::run(root)?;
    i18n::run(root)?;
    layers(root)?;
    cargo(root, &["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"])?;
    cargo(root, &["test", "--workspace"])?;
    println!("ci: ok");
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let root = root();
    let command = args.first().map(String::as_str).unwrap_or("");
    let result = match command {
        "ui-containers" => ui_containers::run(&root),
        "i18n" => {
            if args.iter().any(|argument| argument == "--check") {
                i18n::run(&root)
            } else {
                Err("i18n: pass --check".into())
            }
        }
        "layers" => layers(&root),
        "ci" => ci(&root),
        _ => Err("usage: cargo xtask <ui-containers | i18n --check | layers | ci>".into()),
    };
    if let Err(message) = result {
        eprintln!("{message}");
        std::process::exit(1);
    }
}
