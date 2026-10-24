//! Compiles the icon artwork into `$OUT_DIR/icon_data.rs`: the SVG masters, `palette.toml`
//! (every colour role in light and dark, and the accents) and `aliases.toml`. A missing hex, a
//! role an accent does not know or an alias to a missing master fails the build.
//!
//! Adapted from SolidCraft (`crates/ui-egui/build.rs`), MIT OR Apache-2.0. See ATTRIBUTION.md.

use std::fmt::Write as _;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if let Err(e) = icon_data() {
        eprintln!("icon data: {e}");
        std::process::exit(1);
    }
}

fn icon_data() -> Result<(), String> {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").map_err(|e| e.to_string())?;
    let root = std::path::Path::new(&manifest).join("../../assets/icons/ui/design");
    println!("cargo:rerun-if-changed={}", root.join("palette.toml").display());
    println!("cargo:rerun-if-changed={}", root.join("aliases.toml").display());
    let read = |name: &str| std::fs::read_to_string(root.join(name)).map_err(|e| format!("{name}: {e}"));
    let table = |name: &str| read(name)?.parse::<toml::Table>().map_err(|e| format!("{name}: {e}"));
    let hex = |v: Option<&toml::Value>, what: &str| -> Result<String, String> {
        let s = v.and_then(toml::Value::as_str).ok_or_else(|| format!("{what}: missing hex"))?;
        let ok = s.len() == 7 && s.starts_with('#') && s[1..].bytes().all(|b| b.is_ascii_hexdigit());
        if ok { Ok(s.to_ascii_lowercase()) } else { Err(format!("{what}: `{s}` is not #rrggbb")) }
    };
    let rgb = |h: &str| format!("[0x{}, 0x{}, 0x{}]", &h[1..3], &h[3..5], &h[5..7]);

    let palette = table("palette.toml")?;
    let roles = palette.get("roles").and_then(toml::Value::as_table).ok_or("palette.toml: no [roles]")?;
    let names: Vec<&String> = roles.keys().collect();
    let mut out = String::new();
    let _ = writeln!(out, "pub const ROLE_NAMES: &[&str] = &{:?};", names);
    let (mut lights, mut darks, mut light_hex) = (Vec::new(), Vec::new(), Vec::new());
    for (name, role) in roles {
        let l = hex(role.get("light"), &format!("role {name} light"))?;
        let d = hex(role.get("dark"), &format!("role {name} dark"))?;
        lights.push(rgb(&l));
        darks.push(rgb(&d));
        light_hex.push(hex(role.get("source").or_else(|| role.get("light")), &format!("role {name} source"))?);
    }
    let _ = writeln!(out, "pub const ROLE_SOURCE_HEX: &[&str] = &{:?};", light_hex);
    let _ = writeln!(out, "pub const PALETTE: [&[[u8; 3]]; 2] = [&[{}], &[{}]];", lights.join(", "), darks.join(", "));

    let mut accent_names = Vec::new();
    let mut accents = Vec::new();
    if let Some(table) = palette.get("accents").and_then(toml::Value::as_table) {
        for (acc, roles_of) in table {
            let roles_of = roles_of.as_table().ok_or_else(|| format!("accent {acc}: not a table"))?;
            let mut items = Vec::new();
            for (role, v) in roles_of {
                let idx = names.iter().position(|n| *n == role).ok_or_else(|| format!("accent {acc}: unknown role {role}"))?;
                let l = hex(v.get("light"), &format!("accent {acc} {role} light"))?;
                let d = hex(v.get("dark"), &format!("accent {acc} {role} dark"))?;
                items.push(format!("({idx}, {}, {})", rgb(&l), rgb(&d)));
            }
            accent_names.push(acc.clone());
            accents.push(format!("&[{}]", items.join(", ")));
        }
    }
    let _ = writeln!(out, "pub const ACCENT_NAMES: &[&str] = &{:?};", accent_names);
    let _ = writeln!(out, "pub const ACCENTS: &[&[(usize, [u8; 3], [u8; 3])]] = &[{}];", accents.join(", "));
    // Colour vision variants are not used by the boilerplate artwork; the table stays so the
    // renderer keeps one shape if a project adds them to palette.toml.
    let _ = writeln!(out, "pub const ACCENT_VARIANTS: &[(&str, usize, usize, [u8; 3], [u8; 3])] = &[];");

    let mut svgs = Vec::new();
    let mut ids = std::collections::BTreeSet::new();
    let svg_dir = root.join("svg");
    println!("cargo:rerun-if-changed={}", svg_dir.display());
    let mut files: Vec<_> =
        std::fs::read_dir(&svg_dir).map_err(|e| format!("{}: {e}", svg_dir.display()))?.filter_map(Result::ok).map(|e| e.path()).collect();
    files.sort();
    for f in files.iter().filter(|f| f.extension().is_some_and(|e| e == "svg")) {
        let id = f.file_stem().and_then(|s| s.to_str()).ok_or("bad file name")?;
        ids.insert(id.to_owned());
        let abs = f.canonicalize().map_err(|e| e.to_string())?;
        println!("cargo:rerun-if-changed={}", abs.display());
        svgs.push(format!("({id:?}, \"model\", include_str!({:?}))", abs.display().to_string()));
    }
    let _ = writeln!(out, "pub static SVGS: &[(&str, &str, &str)] = &[{}];", svgs.join(", "));

    let aliases = table("aliases.toml")?;
    let mut al = Vec::new();
    if let Some(t) = aliases.get("alias").and_then(toml::Value::as_table) {
        for (alias, v) in t {
            let (master, accent) = match v {
                toml::Value::String(m) => (m.clone(), String::new()),
                toml::Value::Table(t) => (
                    t.get("master").and_then(toml::Value::as_str).ok_or_else(|| format!("alias {alias}: no master"))?.to_string(),
                    t.get("accent").and_then(toml::Value::as_str).unwrap_or("").to_string(),
                ),
                _ => return Err(format!("alias {alias}: expected a string or a table")),
            };
            if !accent.is_empty() && !accent_names.contains(&accent) {
                return Err(format!("alias {alias}: unknown accent {accent}"));
            }
            if !ids.contains(&master) {
                return Err(format!("alias {alias}: unknown master {master}"));
            }
            al.push(format!("({alias:?}, {master:?}, {accent:?})"));
        }
    }
    let _ = writeln!(out, "pub const ALIASES: &[(&str, &str, &str)] = &[{}];", al.join(", "));
    if let Some(t) = aliases.get("alias").and_then(toml::Value::as_table) {
        ids.extend(t.keys().cloned());
    }

    let mut defaults = std::collections::BTreeMap::new();
    if let Some(groups) = palette.get("icon-accents").and_then(toml::Value::as_table) {
        for (accent, icons) in groups {
            if !accent_names.contains(accent) {
                return Err(format!("icon-accents: unknown accent {accent}"));
            }
            for icon in icons.as_array().ok_or("icon-accents: expected icon array")? {
                let icon = icon.as_str().ok_or("icon-accents: expected icon id")?;
                if !ids.contains(icon) || defaults.insert(icon, accent).is_some() {
                    return Err(format!("icon-accents: unknown or repeated icon {icon}"));
                }
            }
        }
    }
    let defaults: Vec<_> = defaults.iter().map(|(id, accent)| format!("({id:?}, {accent:?})")).collect();
    let _ = writeln!(out, "pub const ICON_ACCENTS: &[(&str, &str)] = &[{}];", defaults.join(", "));

    let dest = std::path::Path::new(&std::env::var("OUT_DIR").map_err(|e| e.to_string())?).join("icon_data.rs");
    std::fs::write(dest, out).map_err(|e| e.to_string())
}
