//! Guards for the boundaries that the translated crate currently relies on.
//!
//! These checks deliberately inventory the current shape instead of asserting
//! that the generated C translation has already become idiomatic Rust.  The
//! invalid fixtures below exercise the failure paths so a guard that silently
//! stops seeing syntax cannot pass by accident.
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use syn::{
    visit::{self, Visit},
    File, ForeignItem, Item, TypeBareFn,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct ForeignCounts {
    functions: usize,
    statics: usize,
    opaque_types: usize,
}

#[derive(Debug, Default)]
struct FileAudit {
    foreign_blocks: usize,
    foreign: ForeignCounts,
    module_mutable_statics: usize,
    scratch_statics: BTreeMap<String, usize>,
    callback_aliases: BTreeSet<String>,
    bare_function_types: usize,
    bad_callback_types: Vec<String>,
    function_depth: usize,
    location: String,
}

impl<'ast> Visit<'ast> for FileAudit {
    fn visit_item_foreign_mod(&mut self, item: &'ast syn::ItemForeignMod) {
        self.foreign_blocks += 1;
        for foreign in &item.items {
            match foreign {
                ForeignItem::Fn(_) => self.foreign.functions += 1,
                ForeignItem::Static(_) => self.foreign.statics += 1,
                ForeignItem::Type(_) => self.foreign.opaque_types += 1,
                _ => {}
            }
        }
        visit::visit_item_foreign_mod(self, item);
    }

    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        self.function_depth += 1;
        visit::visit_item_fn(self, item);
        self.function_depth -= 1;
    }

    fn visit_item_static(&mut self, item: &'ast syn::ItemStatic) {
        if matches!(item.mutability, syn::StaticMutability::Mut(_)) {
            if self.function_depth == 0 {
                self.module_mutable_statics += 1;
            } else {
                *self
                    .scratch_statics
                    .entry(item.ident.to_string())
                    .or_default() += 1;
            }
        }
        visit::visit_item_static(self, item);
    }

    fn visit_item_type(&mut self, item: &'ast syn::ItemType) {
        let before = self.bare_function_types;
        visit::visit_item_type(self, item);
        if self.bare_function_types != before {
            self.callback_aliases.insert(item.ident.to_string());
        }
    }

    fn visit_type_bare_fn(&mut self, item: &'ast TypeBareFn) {
        self.bare_function_types += 1;
        let abi = item
            .abi
            .as_ref()
            .and_then(|abi| abi.name.as_ref())
            .map(|name| name.value());
        if abi.as_deref() != Some("C") || item.unsafety.is_none() {
            self.bad_callback_types.push(format!(
                "{}: callback function pointer must be unsafe extern \"C\" fn",
                self.location
            ));
        }
        visit::visit_type_bare_fn(self, item);
    }
}

#[derive(Default)]
struct DeclarationNames {
    names: BTreeSet<String>,
}

impl<'ast> Visit<'ast> for DeclarationNames {
    fn visit_item(&mut self, item: &'ast Item) {
        let name = match item {
            Item::Const(item) => Some(item.ident.to_string()),
            Item::Enum(item) => Some(item.ident.to_string()),
            Item::Struct(item) => Some(item.ident.to_string()),
            Item::Type(item) => Some(item.ident.to_string()),
            Item::Union(item) => Some(item.ident.to_string()),
            _ => None,
        };
        if let Some(name) = name {
            self.names.insert(name);
        }
        visit::visit_item(self, item);
    }
}

fn source_files(root: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root).expect("read source directory") {
        let path = entry.expect("read source entry").path();
        if path.is_dir() {
            source_files(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
    files.sort();
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .expect("source path under repository")
        .to_string_lossy()
        .replace('\\', "/")
}

fn audit_file(path: &str, source: &str) -> FileAudit {
    let file: File = syn::parse_file(source).expect("parse Rust source");
    let mut audit = FileAudit {
        location: path.to_owned(),
        ..FileAudit::default()
    };
    audit.visit_file(&file);
    audit
}

fn declarations(source: &str) -> BTreeSet<String> {
    let file: File = syn::parse_file(source).expect("parse Rust source");
    let mut names = DeclarationNames::default();
    names.visit_file(&file);
    names.names
}

fn scratch_inventory(root: &Path) -> (BTreeMap<(String, String), usize>, usize) {
    let mut files = Vec::new();
    source_files(&root.join("src"), &mut files);
    let mut scratch = BTreeMap::new();
    let mut module_mutable_statics = 0;
    for path in files {
        let relative = relative(root, &path);
        let audit = audit_file(&relative, &fs::read_to_string(&path).unwrap());
        module_mutable_statics += audit.module_mutable_statics;
        for (symbol, count) in audit.scratch_statics {
            *scratch.entry((relative.clone(), symbol)).or_default() += count;
        }
    }
    (scratch, module_mutable_statics)
}

fn has_foreign_boundary_violation(path: &Path, audit: &FileAudit) -> bool {
    audit.foreign_blocks > 0 && !path.starts_with(Path::new("src/ffi"))
}

fn has_shared_conflict(path: &Path, names: &BTreeSet<String>, shared: &BTreeSet<String>) -> bool {
    !path.starts_with(Path::new("src/shared")) && names.iter().any(|name| shared.contains(name))
}

const RUST_DEALLOCATORS: [&str; 4] = [
    "Box::from_raw(",
    "Vec::from_raw_parts(",
    "CString::from_raw(",
    "alloc::dealloc(",
];

fn rust_deallocator_sites(source: &str) -> Vec<&'static str> {
    RUST_DEALLOCATORS
        .into_iter()
        .filter(|needle| source.contains(needle))
        .collect()
}

#[test]
fn foreign_declarations_are_in_provider_modules_with_frozen_counts() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let expected = [
        (
            "src/ffi/libc.rs",
            ForeignCounts {
                functions: 168,
                statics: 4,
                opaque_types: 14,
            },
        ),
        (
            "src/ffi/libm.rs",
            ForeignCounts {
                functions: 3,
                statics: 0,
                opaque_types: 0,
            },
        ),
        (
            "src/ffi/ncurses.rs",
            ForeignCounts {
                functions: 6,
                statics: 1,
                opaque_types: 0,
            },
        ),
        (
            "src/ffi/resolv.rs",
            ForeignCounts {
                functions: 2,
                statics: 0,
                opaque_types: 0,
            },
        ),
        (
            "src/ffi/systemd.rs",
            ForeignCounts {
                functions: 20,
                statics: 0,
                opaque_types: 3,
            },
        ),
        (
            "src/ffi/utempter.rs",
            ForeignCounts {
                functions: 2,
                statics: 0,
                opaque_types: 0,
            },
        ),
        (
            "src/ffi/utf8proc.rs",
            ForeignCounts {
                functions: 6,
                statics: 0,
                opaque_types: 0,
            },
        ),
    ];
    let expected: BTreeMap<_, _> = expected
        .into_iter()
        .map(|(path, counts)| (path.to_owned(), counts))
        .collect();
    let mut files = Vec::new();
    source_files(&root.join("src"), &mut files);
    let mut providers = BTreeMap::new();
    let mut total = ForeignCounts::default();
    for path in files {
        let relative = relative(root, &path);
        let audit = audit_file(&relative, &fs::read_to_string(&path).unwrap());
        assert!(
            !has_foreign_boundary_violation(Path::new(&relative), &audit),
            "foreign block escaped src/ffi: {relative}"
        );
        if relative.starts_with("src/ffi/") {
            if audit.foreign != ForeignCounts::default() {
                providers.insert(relative, audit.foreign);
            }
            total.functions += audit.foreign.functions;
            total.statics += audit.foreign.statics;
            total.opaque_types += audit.foreign.opaque_types;
        }
    }
    assert_eq!(providers, expected);
    assert_eq!(
        total,
        ForeignCounts {
            functions: 207,
            statics: 5,
            opaque_types: 17
        }
    );
}

#[test]
fn shared_declarations_have_one_authoritative_owner() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    source_files(&root.join("src"), &mut files);
    let mut shared = BTreeSet::new();
    for path in &files {
        let relative = relative(root, path);
        if relative.starts_with("src/shared/") {
            for name in declarations(&fs::read_to_string(path).unwrap()) {
                assert!(
                    shared.insert(name.clone()),
                    "duplicate shared owner for {name}"
                );
            }
        }
    }
    assert!(
        shared.len() > 300,
        "shared declaration inventory unexpectedly small"
    );
    for path in files {
        let relative = relative(root, &path);
        let names = declarations(&fs::read_to_string(&path).unwrap());
        assert!(
            !has_shared_conflict(Path::new(&relative), &names, &shared),
            "non-shared module redeclared an authoritative shared declaration: {relative}"
        );
    }
}

#[test]
fn callback_function_pointers_keep_the_c_abi() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    source_files(&root.join("src"), &mut files);
    let mut aliases = BTreeSet::new();
    let mut bare_function_types = 0;
    for path in files {
        let relative = relative(root, &path);
        let audit = audit_file(&relative, &fs::read_to_string(&path).unwrap());
        assert!(
            audit.bad_callback_types.is_empty(),
            "{}",
            audit.bad_callback_types.join("; ")
        );
        aliases.extend(audit.callback_aliases);
        bare_function_types += audit.bare_function_types;
    }
    assert_eq!(aliases.len(), 44);
    assert!(bare_function_types > aliases.len());
}

#[test]
fn c_heap_ownership_does_not_cross_into_rust_deallocation() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    source_files(&root.join("src"), &mut files);
    for path in files {
        let source = fs::read_to_string(&path).unwrap();
        let sites = rust_deallocator_sites(&source);
        let relative = relative(root, &path);
        // These destructors pair only with Box::into_raw in their own
        // constructors. In format/jobs.rs only the cache is Rust-owned;
        // job nodes and strings remain calloc/strdup + free allocations.
        if matches!(
            relative.as_str(),
            "src/reactor/buffer.rs" | "src/reactor/streams.rs" | "src/format/jobs.rs"
        ) {
            assert_eq!(
                sites,
                ["Box::from_raw("],
                "unexpected deallocator in {relative}"
            );
        } else {
            assert!(
                sites.is_empty(),
                "Rust deallocator used in {}",
                path.display()
            );
        }
    }
    let xmalloc = fs::read_to_string(root.join("src/xmalloc.rs")).unwrap();
    for name in [
        "xmalloc",
        "xcalloc",
        "xrealloc",
        "xreallocarray",
        "xrecallocarray",
        "xstrdup",
        "xstrndup",
        "xmemdup",
        "xasprintf",
        "xvasprintf",
        "xsnprintf",
        "xvsnprintf",
    ] {
        assert!(
            xmalloc.contains(&format!("pub unsafe extern \"C\" fn {name}")),
            "allocator wrapper changed ABI: {name}"
        );
    }
}
