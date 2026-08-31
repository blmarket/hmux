//! Coverage for [`crate::options`] — the pure-data option registry.
//!
//! `options_table.rs` is a single static array plus a small alias map. Both
//! are read-only for the whole run; the option and command code only walks
//! them. These tests pin the structural invariants that a regressing edit
//! would break — constants, array length, terminator, name uniqueness and
//! ordering, scope and type fields, the alias map, and the per-type
//! invariants on `choices`, `default_str`, `default_arr`, `separator` and
//! `pattern` — without touching the live option trees or spawning a server.

use crate::options::{
    OPTIONS_TABLE_CHOICE, OPTIONS_TABLE_COLOUR, OPTIONS_TABLE_COMMAND, OPTIONS_TABLE_FLAG,
    OPTIONS_TABLE_IS_ARRAY, OPTIONS_TABLE_IS_HOOK, OPTIONS_TABLE_IS_STYLE, OPTIONS_TABLE_KEY,
    OPTIONS_TABLE_NUMBER, OPTIONS_TABLE_PANE, OPTIONS_TABLE_SERVER, OPTIONS_TABLE_SESSION,
    OPTIONS_TABLE_STRING, OPTIONS_TABLE_WINDOW, options_other_names, options_table,
};
use ::core::ffi::CStr;
use ::std::collections::HashSet;

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

fn table_len() -> usize {
    options_table.len()
}

fn cstr_len(s: &CStr) -> usize {
    s.to_bytes().len()
}

fn seen(s: &CStr) -> String {
    s.to_string_lossy().into_owned()
}

// ---------------------------------------------------------------------------
// type / scope / flag constants are the documented values
// ---------------------------------------------------------------------------

#[test]
fn options_table_type_scope_and_flag_constants_match_header() {
    assert_eq!(OPTIONS_TABLE_STRING, 0);
    assert_eq!(OPTIONS_TABLE_NUMBER, 1);
    assert_eq!(OPTIONS_TABLE_KEY, 2);
    assert_eq!(OPTIONS_TABLE_COLOUR, 3);
    assert_eq!(OPTIONS_TABLE_FLAG, 4);
    assert_eq!(OPTIONS_TABLE_CHOICE, 5);
    assert_eq!(OPTIONS_TABLE_COMMAND, 6);

    assert_eq!(OPTIONS_TABLE_SERVER, 0x1);
    assert_eq!(OPTIONS_TABLE_SESSION, 0x2);
    assert_eq!(OPTIONS_TABLE_WINDOW, 0x4);
    assert_eq!(OPTIONS_TABLE_PANE, 0x8);

    assert_eq!(OPTIONS_TABLE_IS_ARRAY, 0x1);
    assert_eq!(OPTIONS_TABLE_IS_HOOK, 0x2);
    assert_eq!(OPTIONS_TABLE_IS_STYLE, 0x4);

    // distinct bits
    assert_eq!(OPTIONS_TABLE_SERVER & OPTIONS_TABLE_SESSION, 0);
    assert_eq!(OPTIONS_TABLE_WINDOW & OPTIONS_TABLE_PANE, 0);
    assert_eq!(OPTIONS_TABLE_IS_ARRAY & OPTIONS_TABLE_IS_STYLE, 0);
    assert_ne!(OPTIONS_TABLE_STRING, OPTIONS_TABLE_CHOICE);
    assert!(OPTIONS_TABLE_COMMAND > OPTIONS_TABLE_CHOICE);
}

// ---------------------------------------------------------------------------
// array length and terminator
// ---------------------------------------------------------------------------

#[test]
fn options_table_length_is_stable() {
    // tmux 3.7b ships 221 entries
    assert_eq!(options_table.len(), 221, "array len changed");
}

#[test]
fn options_table_names_are_unique_non_empty_and_nul_terminated() {
    let mut seen_names = HashSet::new();
    for e in &options_table {
        unsafe {
            let s = seen(e.name);
            assert!(!s.is_empty(), "empty name");
            assert!(!s.contains('\0'));
            assert!(s.len() < 64, "name too long: {s:?}");
            assert!(seen_names.insert(s.clone()), "duplicate name {s:?}");
            // CStr round-trip holds
            assert_eq!(cstr_len(e.name), s.len());
            // hook entries (after-*) have no text by tmux convention
            if let Some(text) = e.text {
                assert!(!seen(text).is_empty(), "empty text for {s:?}");
            } else {
                assert_ne!(
                    e.flags & OPTIONS_TABLE_IS_HOOK,
                    0,
                    "null text only for hooks {s:?}"
                );
            }
        }
        // scope is at least one of the four bits and no unknown bits
        let scope = e.scope;
        assert_ne!(scope & 0xF, 0, "no scope bit");
        assert_eq!(scope & !0xF, 0, "unknown scope bit {:#x}", scope);
        // type is in range
        assert!(e.type_0 <= OPTIONS_TABLE_COMMAND);
        // flags only use low 3 bits
        assert_eq!(e.flags & !0x7, 0, "unknown flag bit");
    }
    // spot-check ordering: server block first; word-separators exists
    unsafe {
        assert_eq!(seen(options_table[0].name), "backspace");
        assert!(
            find(c"word-separators").is_some(),
            "word-separators missing"
        );
        assert!(
            find(c"window-unlinked").is_some(),
            "window-unlinked missing"
        );
        assert!(!seen(options_table[table_len() - 1].name).is_empty());
    }
}

// ---------------------------------------------------------------------------
// alias map
// ---------------------------------------------------------------------------

#[test]
fn options_other_names_maps_american_spellings() {
    assert_eq!(options_other_names.len(), 6);
    for m in &options_other_names {
        let from = seen(m.from);
        let to = seen(m.to);
        assert!(!from.is_empty());
        assert!(!to.is_empty());
        assert!(
            from.contains("color") || from.contains("colors"),
            "unexpected from {from:?}"
        );
        assert!(
            to.contains("colour") || to.contains("colours"),
            "unexpected to {to:?}"
        );
    }
    // first mapping is stable
    assert_eq!(seen(options_other_names[0].from), "display-panes-color");
    assert_eq!(seen(options_other_names[0].to), "display-panes-colour");
    assert_eq!(seen(options_other_names[5].from), "pane-colors");
    assert_eq!(seen(options_other_names[5].to), "pane-colours");
}

// ---------------------------------------------------------------------------
// spot-check a handful of known entries
// ---------------------------------------------------------------------------

fn find(name: &CStr) -> Option<&'static crate::types::options_table_entry_t> {
    options_table
        .iter()
        .find(|&e| e.name == name)
        .map(|v| v as _)
}

#[test]
fn options_table_known_entries_have_expected_scope_type_and_defaults() {
    // server number with bounds
    let e = find(c"buffer-limit").expect("buffer-limit");
    assert_eq!(e.type_0, OPTIONS_TABLE_NUMBER);
    assert_eq!(e.scope, OPTIONS_TABLE_SERVER);
    assert_eq!(e.minimum, 1);
    assert_eq!(e.default_num, 50);
    assert!(e.choices.is_none());

    // session key with KEYC bits
    let e = find(c"prefix").expect("prefix");
    assert_eq!(e.type_0, OPTIONS_TABLE_KEY);
    assert_eq!(e.scope, OPTIONS_TABLE_SESSION);
    assert!(e.default_num > 0);

    let e2 = find(c"prefix2").expect("prefix2");
    assert_eq!(e2.type_0, OPTIONS_TABLE_KEY);
    assert_eq!(e2.default_num, 8589934592); // KEYC_NONE

    // colour option spans window+pane
    let e = find(c"cursor-colour").expect("cursor-colour");
    assert_eq!(e.type_0, OPTIONS_TABLE_COLOUR);
    assert_eq!(e.scope, OPTIONS_TABLE_WINDOW | OPTIONS_TABLE_PANE);
    assert_eq!(e.default_num, -1);

    // choice
    let e = find(c"status").expect("status");
    assert_eq!(e.type_0, OPTIONS_TABLE_CHOICE);
    assert_eq!(e.scope, OPTIONS_TABLE_SESSION);
    assert!(e.choices.is_some());
    assert_eq!(e.default_num, 1);

    // array with default_str and separator
    let e = find(c"command-alias").expect("command-alias");
    assert_eq!(e.type_0, OPTIONS_TABLE_STRING);
    assert_ne!(e.flags & OPTIONS_TABLE_IS_ARRAY, 0);
    assert!(seen(e.default_str.expect("a default")).contains("split-pane"));
    assert_eq!(seen(e.separator.expect("a separator")), ",");

    // pattern-bearing entry
    let e = find(c"default-size").expect("default-size");
    assert_eq!(seen(e.pattern.expect("a pattern")), "[0-9]*x[0-9]*");

    // style entries carry IS_STYLE and a separator
    let e = find(c"status-style").expect("status-style");
    assert_ne!(e.flags & OPTIONS_TABLE_IS_STYLE, 0);
    assert_eq!(seen(e.separator.expect("a separator")), ",");
}

// ---------------------------------------------------------------------------
// per-type invariants across the whole table
// ---------------------------------------------------------------------------

#[test]
fn options_table_choice_entries_have_non_null_choices_and_others_do_not() {
    for e in &options_table {
        if e.type_0 == OPTIONS_TABLE_CHOICE {
            let choices = e
                .choices
                .unwrap_or_else(|| panic!("choice {} lists no choices", seen(e.name)));
            for choice in choices {
                assert!(!choice.to_bytes().is_empty());
            }
            assert!(choices.len() < 16, "too many choices for {}", seen(e.name));
            assert!(
                choices.len() >= 2,
                "choice {} needs >=2 options",
                seen(e.name)
            );
        } else if e.type_0 == OPTIONS_TABLE_NUMBER
            || e.type_0 == OPTIONS_TABLE_FLAG
            || e.type_0 == OPTIONS_TABLE_KEY
            || e.type_0 == OPTIONS_TABLE_COLOUR
        {
            // not choices
            assert!(
                e.choices.is_none(),
                "non-choice {} has choices",
                seen(e.name)
            );
        }
    }
    // known choice list length
    let e = find(c"status-justify").expect("status-justify");
    assert_eq!(e.choices.expect("choices").len(), 4);
}

#[test]
fn options_table_array_and_style_flags_match_separator() {
    for e in &options_table {
        let is_array = e.flags & OPTIONS_TABLE_IS_ARRAY != 0;
        let is_style = e.flags & OPTIONS_TABLE_IS_STYLE != 0;
        {
            if is_array || is_style {
                let name = seen(e.name);
                // some arrays carry no separator: status-format (one entry per line),
                // update-environment (space-separated list handled elsewhere), and
                // pane-colours (array of colours). hooks use "" as separator.
                if name == "status-format" || name == "update-environment" || name == "pane-colours"
                {
                    assert!(e.separator.is_none(), "{name} expected no separator");
                } else if e.flags & OPTIONS_TABLE_IS_HOOK != 0 {
                    let sep = e
                        .separator
                        .unwrap_or_else(|| panic!("{name} hook needs separator"));
                    assert_eq!(seen(sep), "");
                } else {
                    let sep = e
                        .separator
                        .unwrap_or_else(|| panic!("{name} flagged but no separator"));
                    assert_eq!(seen(sep), ",");
                }
            }
            // separator implies string or command type (hooks are COMMAND with "" separator)
            if let Some(separator) = e.separator {
                let sep = seen(separator);
                if sep.is_empty() {
                    assert_eq!(
                        e.type_0,
                        OPTIONS_TABLE_COMMAND,
                        "empty sep on non-command {}",
                        seen(e.name)
                    );
                } else {
                    assert_eq!(sep, ",");
                    assert_eq!(
                        e.type_0,
                        OPTIONS_TABLE_STRING,
                        "',' sep on non-string {}",
                        seen(e.name)
                    );
                }
            }
        }
    }
}

#[test]
fn options_table_default_string_or_array_consistency() {
    // array default via default_arr (status-format) vs default_str (others)
    let e = find(c"status-format").expect("status-format");
    assert!(e.default_str.is_none());
    let default_arr = e.default_arr.expect("a default array");
    for value in default_arr {
        assert!(!value.to_bytes().is_empty());
    }
    assert_eq!(default_arr.len(), 3);
    // non-array string with empty default is still a non-null "" pointer
    let e = find(c"copy-command").expect("copy-command");
    assert_eq!(seen(e.default_str.expect("a default")), "");
    // number/flag/choice normally have null default_str, but COLOUR arrays
    // (pane-colours) carry "" as an empty palette and are allowed.
    for e in &options_table {
        if e.type_0 != OPTIONS_TABLE_STRING && e.type_0 != OPTIONS_TABLE_COMMAND {
            if e.type_0 == OPTIONS_TABLE_COLOUR && e.flags & OPTIONS_TABLE_IS_ARRAY != 0 {
                // exactly pane-colours in this table
                assert_eq!(seen(e.name), "pane-colours");
                assert_eq!(seen(e.default_str.expect("a default")), "");
            } else {
                assert!(
                    e.default_str.is_none(),
                    "default_str on {:?} {}",
                    e.type_0,
                    seen(e.name)
                );
            }
        }
    }
}
