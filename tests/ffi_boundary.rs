//! Parse Rust syntax so comments, wrapping, visibility, and symbol aliases cannot
//! hide a Rust definition behind a foreign declaration.
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
use syn::{
    visit::{self, Visit},
    Attribute, Expr, ForeignItem, Item, Lit, Meta,
};

fn symbol(attrs: &[Attribute], attr: &str, fallback: &str) -> String {
    for a in attrs {
        // Both #[export_name = "..."] and #[unsafe(export_name = "...")].
        let meta = if a.path().is_ident("unsafe") {
            a.parse_args::<Meta>().expect("unsafe attribute")
        } else {
            a.meta.clone()
        };
        if let Meta::NameValue(nv) = meta {
            if nv.path.is_ident(attr) {
                if let Expr::Lit(expr) = nv.value {
                    if let Lit::Str(value) = expr.lit {
                        return value.value();
                    }
                }
            }
        }
    }
    fallback.to_owned()
}

#[derive(Default)]
struct Symbols {
    defined: Vec<String>,
    foreign: Vec<String>,
    blocks: usize,
    exported: Vec<String>,
}
impl<'ast> Visit<'ast> for Symbols {
    fn visit_item(&mut self, item: &'ast Item) {
        match item {
            Item::Fn(f) => {
                self.defined
                    .push(symbol(&f.attrs, "export_name", &f.sig.ident.to_string()))
            }
            Item::Static(s) => {
                self.defined
                    .push(symbol(&s.attrs, "export_name", &s.ident.to_string()))
            }
            Item::ForeignMod(_) => self.blocks += 1,
            // Fail closed for unparsed syntax in this inventory.
            Item::Verbatim(tokens) => panic!("unparsed item: {tokens}"),
            _ => {}
        }
        let export = match item {
            Item::Fn(f) => Some((&f.attrs, f.sig.ident.to_string())),
            Item::Static(s) => Some((&s.attrs, s.ident.to_string())),
            _ => None,
        };
        if let Some((attrs, name)) = export {
            if attrs.iter().any(|a| {
                let meta = if a.path().is_ident("unsafe") {
                    a.parse_args::<Meta>().expect("unsafe attribute")
                } else {
                    a.meta.clone()
                };
                meta.path().is_ident("no_mangle") || meta.path().is_ident("export_name")
            }) {
                self.exported.push(symbol(attrs, "export_name", &name));
            }
        }
        visit::visit_item(self, item);
    }
    fn visit_foreign_item(&mut self, item: &'ast ForeignItem) {
        match item {
            ForeignItem::Fn(f) => {
                self.foreign
                    .push(symbol(&f.attrs, "link_name", &f.sig.ident.to_string()))
            }
            ForeignItem::Static(s) => {
                self.foreign
                    .push(symbol(&s.attrs, "link_name", &s.ident.to_string()))
            }
            ForeignItem::Verbatim(tokens) => panic!("unparsed foreign item: {tokens}"),
            _ => {}
        }
        visit::visit_foreign_item(self, item);
    }
}
fn scan(source: &str) -> Symbols {
    let mut symbols = Symbols::default();
    symbols.visit_file(&syn::parse_file(source).expect("parse Rust source"));
    symbols
}
fn source_files(dir: &Path, paths: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let p = entry.unwrap().path();
        if p.is_dir() {
            source_files(&p, paths);
        } else if p.extension().is_some_and(|ext| ext == "rs") {
            paths.push(p);
        }
    }
}

#[test]
fn rust_symbols_are_never_foreign_and_foreign_items_stay_at_the_boundary() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    source_files(&root.join("src"), &mut files);
    let mut definitions = BTreeMap::new();
    let mut foreign = Vec::new();
    let mut exported = std::collections::BTreeSet::new();
    for path in files {
        let s = scan(&fs::read_to_string(&path).unwrap());
        assert!(
            s.blocks == 0 || path.starts_with(root.join("src/ffi")),
            "foreign block outside provider boundary: {}",
            path.display()
        );
        exported.extend(s.exported);
        for name in s.defined {
            definitions.entry(name).or_insert_with(|| path.clone());
        }
        for name in s.foreign {
            foreign.push((name, path.clone()));
        }
    }
    assert!(
        definitions.len() > 2000,
        "definition inventory unexpectedly small"
    );
    assert!(foreign.len() > 200, "foreign inventory unexpectedly small");
    for row in include_str!("../docs/required-exports.tsv").lines().skip(1) {
        let name = row.split('\t').next().unwrap();
        assert!(exported.contains(name), "required C export lost: {name}");
    }
    let mut providers = BTreeMap::new();
    for (name, path) in foreign {
        assert!(
            !definitions.contains_key(&name),
            "{name} defined at {:?} redeclared as foreign at {}",
            definitions.get(&name),
            path.display()
        );
        assert!(
            providers.insert(name.clone(), path.clone()).is_none(),
            "{name} has multiple foreign declarations (including {})",
            path.display()
        );
    }
}

#[test]
fn inventory_handles_aliases_callbacks_variadics_and_misleading_text() {
    let s = scan(
        r###"
        // extern "C" { fn ignored(); }
        const TEXT: &str = r#"extern "C" { fn also_ignored(); }"#;
        #[unsafe(export_name = "real_fn")]
        pub(crate) unsafe extern "C" fn renamed(_: i32, mut ap: ...) {}
        #[export_name = "real_global"] static mut VALUE: i32 = 0;
        mod nested {
            extern "C" {
                #[link_name = "real_fn"] pub fn alias(_: Option<unsafe extern "C" fn(i32)>, ...);
                #[link_name = "real_global"] static mut OTHER: i32;
                pub type opaque;
            }
        }
    "###,
    );
    assert_eq!(s.defined, ["real_fn", "real_global"]);
    assert_eq!(s.exported, ["real_fn", "real_global"]);
    assert_eq!(s.foreign, ["real_fn", "real_global"]);
    assert_eq!(s.blocks, 1);
    let s = scan("fn local() {} static G: i32 = 0; extern \"C\" { fn local(); static G: i32; }");
    assert_eq!(s.defined, s.foreign);
}
