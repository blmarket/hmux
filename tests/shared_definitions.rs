//! Discover authoritative names automatically so new shared families are guarded too.
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

fn source_files(root: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root).expect("read source directory") {
        let path = entry.expect("read source entry").path();
        if path.is_dir() {
            source_files(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
}

// A small lexer, rather than line matching: visibility and wrapping must not
// let a duplicate through. Ignore comments and literals, including raw strings.
fn tokens(source: &str) -> Vec<String> {
    let b = source.as_bytes();
    let mut result = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if b[i..].starts_with(b"//") {
            while i < b.len() && b[i] != b'\n' {
                i += 1;
            }
        } else if b[i..].starts_with(b"/*") {
            i += 2;
            let mut depth = 1;
            while i < b.len() && depth != 0 {
                if b[i..].starts_with(b"/*") {
                    depth += 1;
                    i += 2;
                } else if b[i..].starts_with(b"*/") {
                    depth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
        } else if b[i] == b'r' && matches!(b.get(i + 1), Some(b'#' | b'"')) {
            let start = i;
            i += 1;
            while b.get(i) == Some(&b'#') {
                i += 1;
            }
            if b.get(i) == Some(&b'"') {
                let hashes = i - start - 1;
                i += 1;
                while i < b.len() {
                    if b[i] == b'"'
                        && b.get(i + 1..i + 1 + hashes)
                            .is_some_and(|tail| tail.iter().all(|c| *c == b'#'))
                    {
                        i += 1 + hashes;
                        break;
                    }
                    i += 1;
                }
            } else {
                // A raw identifier (r#name), not a string.
                i = start + 2;
            }
        } else if b[i] == b'\'' {
            // A character literal can contain a double quote. Lifetimes have
            // no closing apostrophe and must not consume the following code.
            let start = i + 1;
            let end = if b.get(start) == Some(&b'\\') {
                match b.get(start + 1) {
                    Some(b'u') => source[start..].find('}').map(|n| start + n + 1),
                    Some(b'x') => Some(start + 4),
                    Some(_) => Some(start + 2),
                    None => None,
                }
            } else {
                source[start..].chars().next().map(|c| start + c.len_utf8())
            };
            i = match end {
                Some(end) if b.get(end) == Some(&b'\'') => end + 1,
                _ => i + 1,
            };
        } else if b[i] == b'"' {
            i += 1;
            while i < b.len() {
                if b[i] == b'\\' {
                    i += 2;
                } else if b[i] == b'"' {
                    i += 1;
                    break;
                } else {
                    i += 1;
                }
            }
        } else if b[i].is_ascii_alphabetic() || b[i] == b'_' {
            let start = i;
            i += 1;
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                i += 1;
            }
            result.push(source[start..i].to_owned());
        } else {
            if !b[i].is_ascii_whitespace() {
                result.push((b[i] as char).to_string());
            }
            i += 1;
        }
    }
    result
}

fn declarations(source: &str) -> Vec<String> {
    let tokens = tokens(source);
    tokens
        .windows(2)
        .enumerate()
        .filter_map(|(i, pair)| {
            let kind = pair[0].as_str();
            let name = &pair[1];
            (matches!(kind, "struct" | "union" | "enum" | "type" | "const")
                && !(kind == "const" && i > 0 && matches!(tokens[i - 1].as_str(), "*" | "raw"))
                && name != "fn"
                && name != "_"
                && name
                    .as_bytes()
                    .first()
                    .is_some_and(|b| b.is_ascii_alphabetic() || *b == b'_'))
            .then(|| name.clone())
        })
        .collect()
}

#[test]
fn migrated_shared_declarations_are_unique_and_authoritative() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = vec![root.join("build.rs")];
    source_files(&root.join("src"), &mut files); // includes main.rs and future binary modules
    let mut locations: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();
    let mut authoritative = BTreeMap::new();
    for path in files {
        let relative = path.strip_prefix(root).unwrap().to_owned();
        for name in declarations(&fs::read_to_string(&path).unwrap()) {
            locations
                .entry(name.clone())
                .or_default()
                .push(relative.clone());
            if relative.starts_with("src/shared") {
                assert!(
                    authoritative
                        .insert(name.clone(), relative.clone())
                        .is_none(),
                    "{name} has multiple authoritative definitions"
                );
            }
        }
    }
    assert!(
        authoritative.len() > 300,
        "shared modules were not inventoried"
    );
    for (name, path) in authoritative {
        assert_eq!(
            locations[&name],
            [path],
            "{name} must be declared only in its authoritative subject module"
        );
    }
}

#[test]
fn guard_recognizes_visibility_wrapping_and_opaque_copies() {
    assert_eq!(
        declarations(
            r##"
        // pub struct ignored {}
        /* type ignored = (); /* nested */ */
        const TEXT: &str = r#"pub struct ignored {}"#;
        let quote = '"';
        let apostrophe = '\'';
        let unicode = '\u{22}';
        let hex = '\x22';
        pub(crate) struct
            Example<'a> { field: &'a u32, ptr: *const Opaque }
        type Alias = Example;
        extern "C" { pub type Opaque; }
        pub const VALUE: u32 = 1;
        let p = &raw const VALUE;
        const fn helper() {}
    "##
        ),
        ["TEXT", "Example", "Alias", "Opaque", "VALUE"]
    );
}

#[test]
fn no_unreviewed_named_duplicates_remain() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = vec![root.join("build.rs")];
    source_files(&root.join("src"), &mut files);
    let mut locations: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();
    for path in files {
        for name in declarations(&fs::read_to_string(&path).unwrap()) {
            // Generated anonymous names have translation-unit-local identity.
            // Their shared owning roles are checked by the authoritative guard.
            if !name.starts_with("C2RustUnnamed") {
                locations
                    .entry(name)
                    .or_default()
                    .push(path.strip_prefix(root).unwrap().to_owned());
            }
        }
    }
    for (name, mut paths) in locations {
        if paths.len() < 2 {
            continue;
        }
        assert!(
            matches!(name.as_str(), "NONE" | "LEFT" | "RIGHT" | "TOP" | "BOTTOM"),
            "unreviewed duplicated declaration {name}: {paths:?}"
        );
        paths.sort();
        let peer = match name.as_str() {
            "NONE" => "src/cmd/parse.rs",
            "LEFT" | "RIGHT" => "src/format_draw.rs",
            "TOP" | "BOTTOM" => "src/window_copy.rs",
            _ => unreachable!(),
        };
        let mut expected = vec![PathBuf::from("src/popup.rs"), PathBuf::from(peer)];
        expected.sort();
        assert_eq!(
            paths, expected,
            "private exception {name} must not spread to another module"
        );
    }
}

// These field sets identify candidates for the audited grid-storage roles, not
// C type identity. Flag candidates even if their types or generated names drift;
// any new lookalike needs an owning-role audit before it can be an exception.
fn grid_storage_candidates(source: &str) -> Vec<(String, &'static str)> {
    let words = tokens(source);
    let mut found = Vec::new();
    for (i, pair) in words.windows(2).enumerate() {
        if !matches!(pair[0].as_str(), "struct" | "union") {
            continue;
        }
        if words.get(i + 2).map(String::as_str) != Some("{") {
            continue;
        }
        let mut fields = Vec::new();
        let mut depth = 0;
        for j in i + 3..words.len() {
            match words[j].as_str() {
                "}" if depth == 0 => break,
                "{" | "(" | "[" | "<" => depth += 1,
                "}" | ")" | "]" | ">" => depth -= 1,
                ":" if depth == 0
                    && words[j - 1] != ":"
                    && words.get(j + 1).map(String::as_str) != Some(":") =>
                {
                    fields.push(words[j - 1].as_str());
                }
                _ => {}
            }
        }
        let role = match (pair[0].as_str(), fields.as_slice()) {
            ("union", ["offset", "data"]) => "grid_cell_entry_storage",
            ("struct", ["attr", "fg", "bg", "data"]) => "grid_cell_entry_data",
            _ => continue,
        };
        found.push((pair[1].clone(), role));
    }
    found
}

#[test]
fn grid_storage_roles_have_only_authoritative_definitions() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = vec![root.join("build.rs")];
    source_files(&root.join("src"), &mut files); // Includes binary code.
    let mut count = 0;
    for path in files {
        for (name, role) in grid_storage_candidates(&fs::read_to_string(&path).unwrap()) {
            assert_eq!(
                path.strip_prefix(root).unwrap(),
                Path::new("src/shared/grid.rs"),
                "redundant or unaudited grid-storage candidate {name} ({role})"
            );
            assert_eq!(name, role);
            count += 1;
        }
    }
    assert_eq!(
        count, 2,
        "both authoritative storage roles must be inventoried"
    );
}

#[test]
fn grid_role_guard_uses_fields_instead_of_generated_names() {
    assert_eq!(
        grid_storage_candidates(
            r#"
        /* union Ignored { offset: u32, data: Other } */
        const TEXT: &str = "struct Ignored { attr: u8, fg: u8, bg: u8, data: u8 }";
        pub(crate) union
        Renumbered { pub offset: u64, pub data: DifferentName }
        struct DifferentName { attr: u8, fg: u8, bg: u8, data: u8 }
        struct C2RustUnnamed { mask: u32, code: u32 }
    "#
        ),
        vec![
            ("Renumbered".into(), "grid_cell_entry_storage"),
            ("DifferentName".into(), "grid_cell_entry_data"),
        ]
    );
}

// These are audit candidates, not proof of C identity. The historical key enum
// used c_ulong; unrelated anonymous enum domains must still be reviewed on their
// own provenance. Compatibility paths should use `pub use`, never fresh aliases.
fn anonymous_key_enum_candidates(source: &str) -> Vec<String> {
    let words = tokens(source);
    let mut found = Vec::new();
    for (i, pair) in words.windows(2).enumerate() {
        if pair[0] != "type" || !pair[1].starts_with("C2RustUnnamed") {
            continue;
        }
        let rhs: Vec<_> = words[i + 2..].iter().take_while(|s| *s != ";").collect();
        if rhs
            .iter()
            .any(|s| matches!(s.as_str(), "c_ulong" | "key_code_enum"))
        {
            found.push(pair[1].clone());
        }
    }
    found
}

#[test]
fn key_enum_guard_handles_wrapping_and_unrelated_generated_domains() {
    assert_eq!(
        anonymous_key_enum_candidates(
            r#"
        // type C2RustUnnamedIgnored = c_ulong;
        const TEXT: &str = "type C2RustUnnamedIgnored = c_ulong;";
        pub(crate) type
            C2RustUnnamed_999 = ::core::ffi::c_ulong;
        type C2RustUnnamed_998 = crate::src::shared::key::key_code_enum;
        type C2RustUnnamed_35 = ::core::ffi::c_uint;
        struct C2RustUnnamed_38 { mask: u32, code: u32 }
        pub use crate::src::shared::key::key_code_enum as C2RustUnnamed_997;
    "#
        ),
        ["C2RustUnnamed_999", "C2RustUnnamed_998"]
    );
}
