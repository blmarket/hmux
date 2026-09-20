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
    let mut files = vec![root.join("lib.rs"), root.join("build.rs")];
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
    let mut files = vec![root.join("lib.rs"), root.join("build.rs")];
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
        // These are separate implementation enums, with distinct domains.
        // See the declaration-specific exceptions in docs/shared-declarations.md.
        assert!(
            matches!(name.as_str(), "NONE" | "LEFT" | "RIGHT" | "TOP" | "BOTTOM"),
            "unreviewed duplicated declaration {name}: {paths:?}"
        );
        paths.sort();
        let peer = match name.as_str() {
            "NONE" => "src/cmd_parse.rs",
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
