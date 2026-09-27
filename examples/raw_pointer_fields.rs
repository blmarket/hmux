//! Inventory in-scope stored raw pointers, excluding foreign ABI/resource fields,
//! callback signatures and src/compat. Exclusions are recorded in the audit TSV.
//! Run with `cargo run --quiet --example raw_pointer_fields`.
use std::{fs, path::Path};
use syn::visit::{self, Visit};

#[derive(Default)]
struct Pointers(bool);
impl<'ast> Visit<'ast> for Pointers {
    fn visit_type_ptr(&mut self, _: &'ast syn::TypePtr) {
        self.0 = true;
    }
    // Parameters/results of function pointers are not stored data pointers.
    fn visit_type_bare_fn(&mut self, _: &'ast syn::TypeBareFn) {}
    fn visit_parenthesized_generic_arguments(
        &mut self,
        _: &'ast syn::ParenthesizedGenericArguments,
    ) {
    }
}

struct Fields<'a> {
    path: &'a Path,
    owner: String,
    rows: &'a mut Vec<String>,
}
impl<'ast> Visit<'ast> for Fields<'_> {
    fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
        let old = std::mem::replace(&mut self.owner, item.ident.to_string());
        self.fields(&item.fields);
        self.owner = old;
    }
    fn visit_item_union(&mut self, item: &'ast syn::ItemUnion) {
        let old = std::mem::replace(&mut self.owner, item.ident.to_string());
        for field in &item.fields.named {
            self.field(field, 0);
        }
        self.owner = old;
    }
    fn visit_item_enum(&mut self, item: &'ast syn::ItemEnum) {
        for variant in &item.variants {
            let old = std::mem::replace(
                &mut self.owner,
                format!("{}::{}", item.ident, variant.ident),
            );
            self.fields(&variant.fields);
            self.owner = old;
        }
    }
}
impl Fields<'_> {
    fn fields(&mut self, fields: &syn::Fields) {
        for (index, field) in fields.iter().enumerate() {
            self.field(field, index);
        }
    }
    fn field(&mut self, field: &syn::Field, index: usize) {
        let mut pointers = Pointers::default();
        pointers.visit_type(&field.ty);
        if pointers.0 {
            let name = field
                .ident
                .as_ref()
                .map_or_else(|| index.to_string(), ToString::to_string);
            self.rows
                .push(format!("{}\t{}\t{}", self.path.display(), self.owner, name));
        }
    }
}

fn scan(path: &Path, rows: &mut Vec<String>) {
    if path == Path::new("src/compat") {
        return;
    }
    if path.is_dir() {
        let mut entries: Vec<_> = fs::read_dir(path)
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        entries.sort();
        for entry in entries {
            scan(&entry, rows);
        }
    } else if path.extension().is_some_and(|ext| ext == "rs") {
        let source = fs::read_to_string(path).unwrap();
        let parsed = syn::parse_file(&source).unwrap();
        visit::visit_file(
            &mut Fields {
                path,
                owner: String::new(),
                rows,
            },
            &parsed,
        );
    }
}

fn main() {
    let mut rows = Vec::new();
    for root in [
        "src",
        "hmux-buffer",
        "hmux-cmdparse",
        "hmux-rt",
        "tests",
        "examples",
    ] {
        scan(Path::new(root), &mut rows);
    }
    let audit = fs::read_to_string("docs/raw-pointer-fields.tsv").unwrap();
    let mut expected = std::collections::BTreeSet::new();
    let mut excluded = std::collections::BTreeSet::new();
    for line in audit.lines().skip(1) {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 5, "invalid audit row");
        let key = fields[..3].join("\t");
        if fields[3].starts_with("Excluded;") {
            excluded.insert(key.clone());
            expected.insert(key);
        } else if fields[3].starts_with("Skip;") {
            expected.insert(key);
        }
    }
    if std::env::args().any(|arg| arg == "--check") {
        let actual: std::collections::BTreeSet<_> = rows.into_iter().collect();
        let missing: Vec<_> = actual.difference(&expected).collect();
        let stale: Vec<_> = expected.difference(&actual).collect();
        assert!(
            missing.is_empty() && stale.is_empty(),
            "unreviewed fields: {missing:?}\nstale dispositions: {stale:?}"
        );
        println!(
            "All {} in-scope raw-pointer fields have audit dispositions; {} external fields excluded.",
            actual.difference(&excluded).count(),
            excluded.len()
        );
    } else {
        for row in rows {
            if !excluded.contains(&row) {
                println!("{row}");
            }
        }
    }
}
