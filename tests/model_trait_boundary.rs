//! Syntax checks complement the compiler-backed field inventory in tools/.
//! They deliberately do not claim to type-resolve arbitrary Rust expressions.
use std::collections::HashSet;
use std::path::Path;
use syn::{
    visit::{self, Visit},
    GenericArgument, Item, Type,
};

fn model_name(name: &str) -> bool {
    matches!(name, "session" | "window" | "window_pane" | "client")
}

fn direct_model(ty: &Type, aliases: &HashSet<String>) -> bool {
    match ty {
        Type::Path(path) => path.path.segments.last().is_some_and(|s| {
            model_name(&s.ident.to_string()) || aliases.contains(&s.ident.to_string())
        }),
        Type::Paren(ty) => direct_model(&ty.elem, aliases),
        Type::Group(ty) => direct_model(&ty.elem, aliases),
        _ => false,
    }
}

fn model_cell(ty: &Type, aliases: &HashSet<String>) -> bool {
    let Type::Path(path) = ty else { return false };
    let Some(last) = path.path.segments.last() else {
        return false;
    };
    if last.ident != "UnsafeCell" {
        return false;
    }
    let syn::PathArguments::AngleBracketed(args) = &last.arguments else {
        return false;
    };
    args.args
        .iter()
        .any(|arg| matches!(arg, GenericArgument::Type(ty) if direct_model(ty, aliases)))
}

#[derive(Default)]
struct Audit {
    aliases: HashSet<String>,
    consumer: bool,
    findings: Vec<String>,
}

impl<'ast> Visit<'ast> for Audit {
    fn visit_item(&mut self, item: &'ast Item) {
        if let Item::Mod(module) = item {
            if module.attrs.iter().any(|attr| {
                attr.path().is_ident("cfg")
                    && attr
                        .parse_args::<syn::Path>()
                        .is_ok_and(|p| p.is_ident("test"))
            }) {
                if self.consumer {
                    return; // Test fixtures may initialize owner state directly.
                }
            }
        }
        visit::visit_item(self, item);
    }
    fn visit_use_rename(&mut self, rename: &'ast syn::UseRename) {
        if model_name(&rename.ident.to_string()) {
            self.aliases.insert(rename.rename.to_string());
        }
        visit::visit_use_rename(self, rename);
    }
    fn visit_item_type(&mut self, alias: &'ast syn::ItemType) {
        if direct_model(&alias.ty, &self.aliases) {
            self.aliases.insert(alias.ident.to_string());
        }
        visit::visit_item_type(self, alias);
    }
    fn visit_type_reference(&mut self, ty: &'ast syn::TypeReference) {
        if self.consumer && direct_model(&ty.elem, &self.aliases) {
            self.findings
                .push("borrowed model passed around the trait".into());
        }
        visit::visit_type_reference(self, ty);
    }
    fn visit_type_ptr(&mut self, ty: &'ast syn::TypePtr) {
        if self.consumer && direct_model(&ty.elem, &self.aliases) {
            self.findings.push("raw model pointer".into());
        }
        visit::visit_type_ptr(self, ty);
    }
    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        // The event payload's own BTreeMap is unrelated to model storage.
        let payload_map = matches!(&*call.receiver, syn::Expr::Field(field)
            if matches!(&*field.base, syn::Expr::Path(base) if base.path.is_ident("ep"))
                && matches!(&field.member, syn::Member::Named(name) if name == "items"));
        if self.consumer && call.method == "get" && !payload_map {
            self.findings
                .push("storage get() in migrated consumer".into());
        }
        if call.method == "cast" {
            if let Some(args) = &call.turbofish {
                for arg in &args.args {
                    if let GenericArgument::Type(ty) = arg {
                        if model_cell(ty, &self.aliases) || direct_model(ty, &self.aliases) {
                            self.findings.push("model representation cast".into());
                        }
                    }
                }
            }
        }
        visit::visit_expr_method_call(self, call);
    }
    fn visit_expr_cast(&mut self, cast: &'ast syn::ExprCast) {
        if let Type::Ptr(ty) = &*cast.ty {
            // `Rc::as_ptr(owner) as *mut Model` also assumes the cell and model
            // have interchangeable layouts. Follow chains through integer casts.
            struct PointerSource(bool);
            impl<'ast> Visit<'ast> for PointerSource {
                fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
                    self.0 |= call.method == "as_ptr";
                    visit::visit_expr_method_call(self, call);
                }
                fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
                    if let syn::Expr::Path(path) = &*call.func {
                        self.0 |= path
                            .path
                            .segments
                            .last()
                            .is_some_and(|s| s.ident == "as_ptr");
                    }
                    visit::visit_expr_call(self, call);
                }
            }
            let mut source = PointerSource(false);
            source.visit_expr(&cast.expr);
            if model_cell(&ty.elem, &self.aliases)
                || (direct_model(&ty.elem, &self.aliases) && source.0)
            {
                self.findings.push("model representation cast".into());
            }
        }
        visit::visit_expr_cast(self, cast);
    }
    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        if self.consumer {
            if let syn::Expr::Path(path) = &*call.func {
                if path
                    .path
                    .segments
                    .last()
                    .is_some_and(|s| s.ident == "as_ptr")
                {
                    self.findings
                        .push("storage pointer helper in migrated consumer".into());
                }
            }
        }
        visit::visit_expr_call(self, call);
    }
}

fn audit(source: &str, consumer: bool) -> Vec<String> {
    let syntax = syn::parse_file(source).expect("Rust source");
    // Collect import aliases before visiting uses, independent of item order.
    let mut aliases = Audit::default();
    aliases.visit_file(&syntax);
    let mut audit = Audit {
        aliases: aliases.aliases,
        consumer,
        ..Default::default()
    };
    audit.visit_file(&syntax);
    audit.findings
}

#[test]
fn rejects_projection_references_pointer_helpers_and_representation_casts() {
    for source in [
        "fn f(s: &Rc<UnsafeCell<session>>) { let s = s.get(); unsafe { use_it(&*s); } }",
        "fn f(s: &session) { helper(s); }",
        "fn f(s: *mut window) { unsafe { (*s).id = 1; } }",
        "fn f(s: &Rc<UnsafeCell<client>>) { let p = rc::as_ptr(s); }",
        "fn f(p: *mut window_pane) { let _ = p.cast::<UnsafeCell<window_pane>>(); }",
        "fn f(p: *mut u8) { let _ = p.cast::<Record>(); } use x::client as Record;",
        "fn f(p: *mut u8) { let _ = p as *mut UnsafeCell<session>; }",
    ] {
        assert!(!audit(source, true).is_empty(), "missed {source}");
    }
    for source in [
        "fn f(s: &Rc<UnsafeCell<session>>) { let _ = Rc::as_ptr(s) as *mut session; }",
        "fn f(s: &Weak<UnsafeCell<window>>) { let _ = s.as_ptr() as usize as *const window; }",
        "type Record = client; fn f(p: *mut u8) { let _ = p.cast::<Record>(); }",
    ] {
        assert!(
            !audit(source, false).is_empty(),
            "missed representation cast: {source}"
        );
    }
    assert!(audit(
        "fn f(s: &Rc<UnsafeCell<session>>) { s.name(); s.with_winlinks(|links| lookup(links)); }",
        true
    )
    .is_empty());
}

#[test]
fn migrated_notification_consumers_do_not_project_model_storage() {
    for path in [
        "src/alerts.rs",
        "src/control_notify.rs",
        "src/events_payload.rs",
    ] {
        let source =
            std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap();
        assert_eq!(audit(&source, true), Vec::<String>::new(), "{path}");
    }
}

#[test]
fn notification_helpers_do_not_project_client_storage() {
    let source =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/control.rs"))
            .unwrap();
    let syntax = syn::parse_file(&source).unwrap();
    let expected = [
        "control_write_line",
        "control_flush_deferred",
        "control_write",
        "control_write_guard",
        "control_notify_write",
        "control_discard_pane_output",
    ];
    let mut checked = 0;
    for item in syntax.items {
        let Item::Fn(function) = item else { continue };
        if expected.contains(&function.sig.ident.to_string().as_str()) {
            let mut check = Audit {
                consumer: true,
                ..Default::default()
            };
            check.visit_item_fn(&function);
            assert!(
                check.findings.is_empty(),
                "{}: {:?}",
                function.sig.ident,
                check.findings
            );
            checked += 1;
        }
    }
    assert_eq!(
        checked,
        expected.len(),
        "migrated helpers must remain checked"
    );
}

#[test]
fn no_model_representation_casts_anywhere_in_application() {
    fn check(directory: &Path) {
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                check(&path);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                let source = std::fs::read_to_string(&path).unwrap();
                assert!(audit(&source, false).is_empty(), "{}", path.display());
            }
        }
    }
    check(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src"));
}

#[test]
fn model_traits_do_not_return_raw_components_or_whole_models() {
    for path in [
        "src/session/api.rs",
        "src/window/api.rs",
        "src/window/pane_api.rs",
        "src/server_client/api.rs",
    ] {
        let source =
            std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap();
        let syntax = syn::parse_file(&source).unwrap();
        for item in syntax.items {
            let Item::Trait(interface) = item else {
                continue;
            };
            for item in interface.items {
                let syn::TraitItem::Fn(method) = item else {
                    continue;
                };
                struct Output;
                impl<'ast> Visit<'ast> for Output {
                    fn visit_type_ptr(&mut self, _: &'ast syn::TypePtr) {
                        panic!("raw component in model trait result");
                    }
                    fn visit_type_reference(&mut self, ty: &'ast syn::TypeReference) {
                        assert!(
                            !direct_model(&ty.elem, &HashSet::new()),
                            "whole model borrow"
                        );
                        visit::visit_type_reference(self, ty);
                    }
                }
                Output.visit_return_type(&method.sig.output);
            }
        }
    }
}
