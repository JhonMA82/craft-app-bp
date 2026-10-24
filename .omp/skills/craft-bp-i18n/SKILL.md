---
name: craft-bp-i18n
description: "Add or change translated text in the Craft Boilerplate: stable catalogue ids, en.json as source of truth, es.json parity, tr versus tr_label/tr_dyn/tr_name, plurals, format placeholders, the i18n gate and the language-switch harness test. Trigger: add a label, translate a string, missing translation, Spanish text, i18n gate, tr key, menu text, error message, plural, language switch, locale."
---

# Internationalisation

Two catalogues, one source of truth:

- `assets/locales/en.json` — canonical ids (`id → text`).
- `assets/locales/es.json` — must have exactly the same key set.
- `crates/craft-i18n` implements catalogues, scoping, interpolation and lookup.
- `crate::i18n` / `craft_ui::i18n` re-export it; app code imports `craft_ui::i18n::{self, tr}` or
  `craft_shell::i18n`.

## Ids

Stable, lowercase, dotted, namespaced: `ui.close`, `menu.file`, `prefs.theme`,
`cmd.demo.share`, `panel.configure`, `tab.home`, `home.new`, `status.saved`, `keymap.reserved`.
Never store displayed English text in `CommandSpec`, `TabSpec`, `PanelSpec`, `MenuSpec` or
`PageSpec`; store the id and resolve it at paint time.

## Which lookup

| Function | Use |
|---|---|
| `tr("ui.close")` | Fixed UI strings. Returns active language, then English, then the id. |
| `format("about.version", &[("version", "3")])` | Placeholders `{version}`. Unknown placeholders stay visible. |
| `tr_label(label)` | Data or generated labels: translates a known English string, otherwise passes it through. |
| `tr_dyn(text)` | Translate when a key or known English text is found, otherwise return as-is. |
| `tr_name(text)` | A person's typed name; never translated. |

Use `tr` for every literal shipped in the UI. Do not use `tr_label` to avoid adding a catalogue
entry. Use `tr_name` for document/user names, not for product strings.

Plurals use `_one` / `_other` suffixes and `{count}`:

```json
"ui.browser_count_one": "{count} body",
"ui.browser_count_other": "{count} bodies"
```

```rust
i18n::format(if n == 1 { "ui.browser_count_one" } else { "ui.browser_count_other" },
             &[("count", &n.to_string())])
```

Do not build translated sentences with `format!`; word order differs. Keep whole sentences in the
catalogue.

## Workflow

1. Add the id to **both** `en.json` and `es.json` (same key, same placeholders).
2. Replace the literal in code with `tr("…")` / `format("…")`.
3. For data-driven labels, pass the id through the spec/menu/page structures; the shell translates.
4. Run `cargo xtask i18n --check`.
5. Add a test when the change is consumer-visible (see below).
6. If adding a language: add `assets/locales/<id>.json`, a `Catalog` static, an entry in `LOCALES`
   and, once complete, `COMPLETE` in `crates/craft-i18n`. Every complete language must match
   English exactly.
7. Update `docs/i18n.md` if the rules or the language list change.

## What the gate checks

`xtask/src/i18n.rs`:

- Every non-English catalogue has exactly the English key set (missing **and** extra keys fail).
- Every literal `tr("…")` / `format("…")` in `crates/**/src/**/*.rs` and `apps/**/src/**/*.rs`
  exists in English.
- It skips comment lines and only matches a whole identifier (`push_str("…")` is not `tr("…")`).

It does **not** see:
- ids stored in `CommandSpec`/`PanelSpec`/`TabSpec`/`MenuSpec`/`PageSpec`,
- `tr_label`/`tr_dyn`/`tr_name` arguments,
- dynamic ids built with variables.

Those still need both catalogue entries and a test.

## Language-switch test

Use the harness and drive real UI:

```rust
let mut h = Harness::new(/* clean app + shell */);
h.key(Key::K, Modifiers::CTRL);       // palette, or open Preferences
h.text("preferences");
h.key(Key::Enter, Modifiers::NONE);
h.settle();
// click the language row / Spanish option, Apply, then assert the visible label
h.settle();
assert!(/* catalogued label is Spanish, e.g. "Preferencias" */);
```

Or set `h.shell.settings.language = "es".into()`, run a frame, and assert the translated text is
published/visible. Assert a stable consumer-visible string, not an incidental default. Switching
back must restore English.

## Traps

- **Only one catalogue:** `cargo xtask i18n --check` fails; add both.
- **Extra Spanish key:** also fails ("not in English"). Remove or add the English id.
- **Literal ids in specs:** the gate misses them, so a typo surfaces at runtime as the raw id.
- **`tr_label` for fixed UI:** hides a missing translation behind pass-through English.
- **`tr_dyn` everywhere:** masks missing ids; prefer `tr` when the id is known statically.
- **Format placeholders:** a translation that renames `{count}` leaves the literal visible; keep
  placeholder names identical across languages.
- **Sentence assembly:** concatenating translated fragments breaks grammar; keep full strings.
- **Language scope:** the shell enters `i18n::scope(language)` once per frame; code that reads
  strings before `Shell::ui` (startup, background threads) sees the default. Translate at paint
  time or re-read after the scope.
- **Migrating to the shared crate:** this repo already has a local `craft-i18n`. Do not swap it for
  a craft-libs variant (or introduce a second catalogue system) without an ADR/decision; the
  package name may collide and the IDs/catalogue format must move together.
- **Unused English keys:** allowed by the gate, but they rot. Remove keys with the feature.

## Do not

- Do not hard-code user-visible English in app/shell UI code.
- Do not use `format!` to interpolate catalogue text; use `i18n::format`.
- Do not translate person names (`tr_name`) or internal ids.
- Do not add a locale without updating `LOCALES`/`COMPLETE` and the docs.
- Do not claim a language works without running the switch test or checking both catalogues.
