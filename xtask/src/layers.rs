//! Dependency layering rules (docs/architecture.md).
//!
//! Adapted from SolidCraft (`xtask/src/layers.rs`), MIT OR Apache-2.0. See ATTRIBUTION.md.
//!
//! The rule engine works on a small, metadata-independent model so it can be
//! unit-tested; `from_metadata` builds that model from `cargo metadata`.

use serde_json::Value;

/// Where a workspace crate sits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    /// Regular layered crate.
    Layer(u8),
    /// Layer 0, and additionally may depend on no workspace crate at all.
    Standalone,
    /// Test tooling: may depend on anything up to L5 (it sits at L6 for rule
    /// purposes); other crates may use it only as a dev-dependency.
    Testkit,
    /// Binaries and build tooling: exempt from the rules.
    Exempt,
}

impl Class {
    fn layer(self) -> Option<u8> {
        match self {
            Class::Layer(l) => Some(l),
            Class::Standalone => Some(0),
            Class::Testkit => Some(6),
            Class::Exempt => None,
        }
    }
}

/// The layering table. Names are package names without the `solidcraft-`
/// prefix.
pub const TABLE: &[(&str, Class)] = &[
    // Design tokens and catalogues: no workspace dependencies.
    ("theme", Class::Standalone),
    ("i18n", Class::Standalone),
    // Fonts, icons, the container and the retained kit.
    ("ui", Class::Layer(1)),
    // Chrome and orchestration.
    ("shell", Class::Layer(2)),
    // The headless harness: other crates may use it only as a dev-dependency.
    ("test", Class::Testkit),
    // Apps and tooling.
    ("demo", Class::Exempt),
    ("demo-web", Class::Exempt),
    ("demo-hot", Class::Exempt),
    ("demo-hot-lib", Class::Exempt),
    ("xtask", Class::Exempt),
];

/// The design-token crate sits at layer 0 but names egui's colour and shadow value types, so the
/// UI-crate rule allows them in the foundation crates below layer 1.
pub const UI_FOUNDATION: &[&str] = &["theme"];

/// Explicit orderings *within* a layer (earlier may be used by later). None are needed yet.
pub const INTRA_LAYER_ORDER: &[&[&str]] = &[];

fn intra_layer_allowed(from: &str, to: &str) -> bool {
    let (from, to) = (short_name(from), short_name(to));
    INTRA_LAYER_ORDER.iter().any(|chain| match (chain.iter().position(|n| *n == from), chain.iter().position(|n| *n == to)) {
        (Some(f), Some(t)) => t < f,
        _ => false,
    })
}

/// External crates that constitute a UI toolkit / windowing dependency.
/// Entries ending in `*` are prefixes.
pub const UI_CRATES: &[&str] = &["egui", "eframe", "egui-wgpu", "wgpu", "winit", "egui_kittest", "rfd", "bevy*"];

/// First layer allowed to use UI crates.
pub const UI_MIN_LAYER: u8 = 1;

pub fn short_name(pkg: &str) -> &str {
    pkg.strip_prefix("craft-").unwrap_or(pkg)
}

pub fn classify(pkg: &str) -> Option<Class> {
    let s = short_name(pkg);
    TABLE.iter().find(|(n, _)| *n == s).map(|(_, c)| *c)
}

fn is_ui_crate(name: &str) -> bool {
    UI_CRATES.iter().any(|p| match p.strip_suffix('*') {
        Some(prefix) => name.starts_with(prefix),
        None => name == *p,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepKind {
    Normal,
    Dev,
    Build,
}

#[derive(Debug, Clone)]
pub struct Dep {
    pub name: String,
    pub kind: DepKind,
    /// `true` if the dependency is a workspace member.
    pub workspace: bool,
}

#[derive(Debug, Clone)]
pub struct Crate {
    pub name: String,
    pub deps: Vec<Dep>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Violation {
    Unregistered { krate: String },
    Upward { krate: String, dep: String, from: u8, to: u8, kind: DepKind },
    StandaloneHasWorkspaceDep { krate: String, dep: String },
    TestkitAsNormalDep { krate: String },
    UiBelowLayer { krate: String, dep: String, layer: u8 },
}

impl std::fmt::Display for Violation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Violation::Unregistered { krate } => {
                write!(f, "{krate}: unknown workspace crate; register it in xtask/src/layers.rs TABLE (see docs/architecture.md)")
            }
            Violation::Upward { krate, dep, from, to, kind } => {
                write!(f, "{krate} (L{from}) -> {dep} (L{to}) [{kind:?}]: may only depend on strictly lower layers")
            }
            Violation::StandaloneHasWorkspaceDep { krate, dep } => {
                write!(f, "{krate}: standalone crate must not depend on workspace crate {dep}")
            }
            Violation::TestkitAsNormalDep { krate } => {
                write!(f, "{krate}: solidcraft-testkit may only be a dev-dependency")
            }
            Violation::UiBelowLayer { krate, dep, layer } => {
                write!(f, "{krate} (L{layer}) depends on UI crate `{dep}`; UI toolkits are only allowed from L1 up")
            }
        }
    }
}

/// Check all rules. Returns violations sorted for stable output.
pub fn check(crates: &[Crate]) -> Vec<Violation> {
    let mut out = Vec::new();
    for c in crates {
        let Some(class) = classify(&c.name) else {
            out.push(Violation::Unregistered { krate: c.name.clone() });
            continue;
        };
        if class == Class::Exempt {
            continue;
        }
        let layer = class.layer().unwrap_or(0);
        for d in &c.deps {
            // Self dev-dependencies (e.g. to enable features in tests) are fine.
            if d.name == c.name {
                continue;
            }
            if d.workspace {
                if class == Class::Standalone {
                    out.push(Violation::StandaloneHasWorkspaceDep { krate: c.name.clone(), dep: d.name.clone() });
                    continue;
                }
                match classify(&d.name) {
                    // Unregistered deps are reported on their own entry.
                    None => {}
                    Some(Class::Testkit) if d.kind != DepKind::Dev => {
                        out.push(Violation::TestkitAsNormalDep { krate: c.name.clone() });
                    }
                    Some(Class::Testkit) => {}
                    Some(dc) => {
                        let to = dc.layer().unwrap_or(u8::MAX);
                        if to >= layer && !(to == layer && intra_layer_allowed(&c.name, &d.name)) {
                            out.push(Violation::Upward {
                                krate: c.name.clone(),
                                dep: d.name.clone(),
                                from: layer,
                                to: if to == u8::MAX { 7 } else { to },
                                kind: d.kind,
                            });
                        }
                    }
                }
            } else if layer < UI_MIN_LAYER && !UI_FOUNDATION.contains(&short_name(&c.name)) && is_ui_crate(&d.name) {
                out.push(Violation::UiBelowLayer { krate: c.name.clone(), dep: d.name.clone(), layer });
            }
        }
    }
    out.sort_by_key(|v| v.to_string());
    out.dedup();
    out
}

/// Build the model from `cargo metadata --format-version 1 --no-deps`.
pub fn from_metadata(meta: &Value) -> Result<Vec<Crate>, String> {
    let pkgs = meta["packages"].as_array().ok_or("metadata: no packages array")?;
    let members: Vec<&str> = pkgs.iter().filter_map(|p| p["name"].as_str()).collect();
    let mut out = Vec::new();
    for p in pkgs {
        let name = p["name"].as_str().ok_or("package without name")?.to_owned();
        let mut deps = Vec::new();
        for d in p["dependencies"].as_array().into_iter().flatten() {
            let dname = d["name"].as_str().unwrap_or_default().to_owned();
            let kind = match d["kind"].as_str() {
                Some("dev") => DepKind::Dev,
                Some("build") => DepKind::Build,
                _ => DepKind::Normal,
            };
            let workspace = members.contains(&dname.as_str()) || d["path"].is_string();
            deps.push(Dep { name: dname, kind, workspace });
        }
        out.push(Crate { name, deps });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    fn c(name: &str, deps: &[(&str, DepKind, bool)]) -> Crate {
        Crate { name: name.into(), deps: deps.iter().map(|(n, k, w)| Dep { name: (*n).into(), kind: *k, workspace: *w }).collect() }
    }
    use DepKind::*;

    #[test]
    fn the_workspace_layers_are_acyclic() {
        let g = [
            c("craft-theme", &[("egui", Normal, false), ("serde", Normal, false)]),
            c("craft-i18n", &[("serde_json", Normal, false)]),
            c("craft-ui", &[("craft-theme", Normal, true), ("craft-i18n", Normal, true), ("egui", Normal, false)]),
            c("craft-shell", &[("craft-ui", Normal, true), ("craft-theme", Normal, true), ("craft-i18n", Normal, true)]),
            c("craft-test", &[("craft-shell", Normal, true), ("craft-ui", Normal, true)]),
            c("craft-demo", &[("craft-shell", Normal, true), ("craft-test", Dev, true), ("eframe", Normal, false)]),
            c("craft-demo-web", &[("craft-demo", Normal, true), ("eframe", Normal, false)]),
            c("xtask", &[("syn", Normal, false)]),
        ];
        assert!(check(&g).is_empty(), "{:?}", check(&g));
    }

    #[test]
    fn upward_dependency_flagged() {
        // L0 crates are standalone: a workspace dependency is reported as such.
        let v = check(&[c("craft-theme", &[("craft-ui", Normal, true)])]);
        assert!(matches!(v[..], [Violation::StandaloneHasWorkspaceDep { .. }]));
        let v = check(&[c("craft-ui", &[("craft-shell", Normal, true)])]);
        assert!(matches!(v[..], [Violation::Upward { from: 1, to: 2, .. }]));
    }

    #[test]
    fn standalone_crates_have_no_workspace_dependencies() {
        let v = check(&[c("craft-i18n", &[("craft-theme", Normal, true)])]);
        assert!(matches!(v[..], [Violation::StandaloneHasWorkspaceDep { .. }]));
        assert!(check(&[c("craft-theme", &[("egui", Normal, false)])]).is_empty(), "the token crate may use egui's value types");
    }

    #[test]
    fn ui_crates_below_layer_one_flagged() {
        for dep in ["eframe", "winit", "egui_kittest"] {
            let v = check(&[c("craft-i18n", &[(dep, Normal, false)])]);
            assert!(matches!(v[..], [Violation::UiBelowLayer { layer: 0, .. }]), "{dep}");
        }
        assert!(check(&[c("craft-ui", &[("egui", Normal, false)])]).is_empty());
        assert!(check(&[c("craft-theme", &[("egui", Normal, false)])]).is_empty(), "the design tokens are the UI foundation");
    }

    #[test]
    fn testkit_only_as_a_dev_dependency() {
        let v = check(&[c("craft-ui", &[("craft-test", Normal, true)])]);
        assert!(matches!(v[..], [Violation::TestkitAsNormalDep { .. }]));
        assert!(check(&[c("craft-demo", &[("craft-test", Dev, true)])]).is_empty());
        assert!(check(&[c("craft-test", &[("craft-shell", Normal, true)])]).is_empty());
    }

    #[test]
    fn unregistered_crates_are_errors() {
        let v = check(&[c("craft-mystery", &[])]);
        assert!(matches!(&v[..], [Violation::Unregistered { krate }] if krate == "craft-mystery"));
        assert!(v[0].to_string().contains("register"));
    }

    #[test]
    fn apps_and_xtask_are_exempt() {
        for app in ["craft-demo", "craft-demo-web", "xtask"] {
            assert!(check(&[c(app, &[("egui", Normal, false), ("craft-ui", Normal, true)])]).is_empty());
        }
    }

    #[test]
    fn metadata_parsing() {
        let meta: Value = serde_json::from_str(
            r#"{"packages":[
                {"name":"craft-ui","dependencies":[
                    {"name":"craft-theme","kind":null,"path":"/x/crates/craft-theme"},
                    {"name":"serde","kind":null},
                    {"name":"proptest","kind":"dev"}]},
                {"name":"craft-theme","dependencies":[]}
            ]}"#,
        )
        .unwrap();
        let g = from_metadata(&meta).unwrap();
        assert_eq!(g.len(), 2);
        let ui = g.iter().find(|c| c.name == "craft-ui").unwrap();
        assert!(ui.deps[0].workspace && !ui.deps[1].workspace);
        assert_eq!(ui.deps[2].kind, Dev);
        assert!(check(&g).is_empty());
    }
}
