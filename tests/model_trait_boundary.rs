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
        if self.consumer && call.method == "get" && call.args.is_empty() && !payload_map {
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
fn migrated_consumers_do_not_project_model_storage() {
    for path in [
        "src/alerts.rs",
        "src/control_notify.rs",
        "src/events_payload.rs",
        "src/status.rs",
        "src/tty_acs.rs",
        "src/tty_features.rs",
        "src/tty_term.rs",
        "src/tty/output.rs",
        "src/tty/input.rs",
        "src/tty_draw.rs",
        "src/tty_keys.rs",
        "src/format/jobs.rs",
    ] {
        let source =
            std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap();
        assert_eq!(audit(&source, true), Vec::<String>::new(), "{path}");
    }
}

#[test]
fn format_callbacks_do_not_retain_cache_or_entry_pointers() {
    let source = std::fs::read_to_string("src/format/jobs.rs").unwrap();
    let syntax = syn::parse_file(&source).unwrap();
    struct CachePointers;
    impl<'ast> Visit<'ast> for CachePointers {
        fn visit_type_ptr(&mut self, ty: &'ast syn::TypePtr) {
            if let Type::Path(path) = &*ty.elem {
                assert!(
                    !path.path.segments.last().is_some_and(|segment| matches!(
                        segment.ident.to_string().as_str(),
                        "format_job" | "format_job_tree"
                    )),
                    "format callbacks must reacquire cache entries after Client borrows end"
                );
            }
            visit::visit_type_ptr(self, ty);
        }
    }
    CachePointers.visit_file(&syntax);
}

#[test]
fn overlay_mode_callback_returns_owned_screen_state() {
    let source = std::fs::read_to_string("src/shared/client.rs").unwrap();
    let syntax = syn::parse_file(&source).unwrap();
    let callback = syntax
        .items
        .iter()
        .find_map(|item| match item {
            Item::Type(alias) if alias.ident == "overlay_mode_cb" => Some(alias),
            _ => None,
        })
        .expect("overlay mode callback declaration");
    struct OwnedResult;
    impl<'ast> Visit<'ast> for OwnedResult {
        fn visit_return_type(&mut self, output: &'ast syn::ReturnType) {
            struct NoBorrow;
            impl<'ast> Visit<'ast> for NoBorrow {
                fn visit_type_ptr(&mut self, _: &'ast syn::TypePtr) {
                    panic!("overlay callback returned a component pointer");
                }
                fn visit_type_reference(&mut self, _: &'ast syn::TypeReference) {
                    panic!("overlay callback returned a borrowed component");
                }
                fn visit_type_path(&mut self, ty: &'ast syn::TypePath) {
                    assert!(
                        !ty.path.segments.iter().any(|part| part.ident == "NonNull"),
                        "overlay callback returned a NonNull component"
                    );
                    visit::visit_type_path(self, ty);
                }
            }
            NoBorrow.visit_return_type(output);
        }
    }
    OwnedResult.visit_item_type(callback);
    // The returned display snapshot itself must own only copied scalar state.
    let source = std::fs::read_to_string("src/shared/screen.rs").unwrap();
    let syntax = syn::parse_file(&source).unwrap();
    let snapshot = syntax
        .items
        .iter()
        .find_map(|item| match item {
            Item::Struct(item) if item.ident == "ScreenMode" => Some(item),
            _ => None,
        })
        .expect("owned screen mode snapshot");
    for field in &snapshot.fields {
        let Type::Path(path) = &field.ty else {
            panic!("borrowed screen mode field")
        };
        assert!(matches!(
            path.path
                .segments
                .last()
                .unwrap()
                .ident
                .to_string()
                .as_str(),
            "u_int" | "c_int" | "screen_cursor_style"
        ));
    }
}

#[test]
fn migrated_terminal_helpers_do_not_reach_through_client_storage() {
    let syntax = syn::parse_file(&std::fs::read_to_string("src/tty.rs").unwrap()).unwrap();
    for item in &syntax.items {
        let Item::Fn(function) = item else { continue };
        // These remaining projections belong to the Window/Pane migration.
        // Terminal orchestration and output have no Client storage exemption.
        if matches!(
            function.sig.ident.to_string().as_str(),
            "tty_update_window_offset" | "tty_style_changed" | "tty_default_colours"
        ) {
            continue;
        }
        let mut check = Audit {
            consumer: true,
            ..Default::default()
        };
        check.visit_item_fn(function);
        assert!(
            check.findings.is_empty(),
            "{}: {:?}",
            function.sig.ident,
            check.findings
        );
    }
    for (path, expected) in [
        (
            "src/tty.rs",
            vec![
                "tty_init",
                "tty_initialize_component",
                "tty_window_bigger",
                "tty_window_offset1",
                "tty_update_client_offset",
                "tty_set_selection",
                "tty_selection_sequence",
            ],
        ),
        (
            "src/tty_keys.rs",
            vec![
                "tty_keys_next1",
                "tty_keys_extended_key",
                "tty_keys_mouse",
                "tty_keys_colours",
                "tty_keys_update_focus",
                "tty_keys_clipboard",
                "tty_keys_palette",
            ],
        ),
        (
            "src/input.rs",
            vec![
                "input_request_reply",
                "input_request_matches",
                "input_complete_request",
                "input_free_request",
            ],
        ),
    ] {
        let source =
            std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap();
        let syntax = syn::parse_file(&source).unwrap();
        let mut checked = 0;
        for item in syntax.items {
            let Item::Fn(function) = item else { continue };
            if !expected.contains(&function.sig.ident.to_string().as_str()) {
                continue;
            }
            let mut check = Audit {
                consumer: true,
                ..Default::default()
            };
            check.visit_item_fn(&function);
            assert!(
                check.findings.is_empty(),
                "{path}::{}: {:?}",
                function.sig.ident,
                check.findings
            );
            checked += 1;
        }
        assert_eq!(
            checked,
            expected.len(),
            "{path}: migrated helpers must remain checked"
        );
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

#[test]
fn client_fields_remain_private_to_the_implementation_module() {
    let source = std::fs::read_to_string("src/server_client/model.rs").unwrap();
    let syntax = syn::parse_file(&source).unwrap();
    let model = syntax
        .items
        .iter()
        .find_map(|item| match item {
            Item::Struct(model) if model.ident == "client" => Some(model),
            _ => None,
        })
        .unwrap();
    for field in &model.fields {
        assert!(
            matches!(&field.vis, syn::Visibility::Restricted(vis)
            if vis.path.is_ident("super")),
            "Client field visibility widened: {:?}",
            field.ident
        );
    }
    let syntax =
        syn::parse_file(&std::fs::read_to_string("src/server_client/mod.rs").unwrap()).unwrap();
    struct ClientStorage;
    impl<'ast> Visit<'ast> for ClientStorage {
        fn visit_type_reference(&mut self, ty: &'ast syn::TypeReference) {
            assert!(
                !matches!(&*ty.elem, Type::Path(path) if path.path.is_ident("client")),
                "exported helper must not accept or return a whole Client borrow"
            );
            visit::visit_type_reference(self, ty);
        }
        fn visit_type_ptr(&mut self, ty: &'ast syn::TypePtr) {
            assert!(
                !matches!(&*ty.elem, Type::Path(path) if path.path.is_ident("client")),
                "exported helper must not expose raw Client storage"
            );
            visit::visit_type_ptr(self, ty);
        }
    }
    for item in &syntax.items {
        let Item::Fn(function) = item else { continue };
        if !matches!(function.vis, syn::Visibility::Inherited) {
            ClientStorage.visit_signature(&function.sig);
        }
    }
}

#[test]
fn queued_items_do_not_retain_raw_queue_component_pointers() {
    let source = std::fs::read_to_string("src/cmd/queue.rs").unwrap();
    let syntax = syn::parse_file(&source).unwrap();
    struct QueuePointers;
    impl<'ast> Visit<'ast> for QueuePointers {
        fn visit_type_ptr(&mut self, ty: &'ast syn::TypePtr) {
            if let Type::Path(path) = &*ty.elem {
                assert!(
                    !path
                        .path
                        .segments
                        .last()
                        .is_some_and(|s| s.ident == "cmdq_list"),
                    "queue pointers must not outlive the Client component borrow"
                );
            }
            visit::visit_type_ptr(self, ty);
        }
    }
    QueuePointers.visit_file(&syntax);
    assert!(syntax.items.iter().any(|item| matches!(item,
        Item::Enum(target) if target.ident == "QueueTarget")));
}

#[test]
fn session_state_visibility_cannot_widen_silently() {
    let source = std::fs::read_to_string("src/session/model.rs").unwrap();
    let syntax = syn::parse_file(&source).unwrap();
    let model = syntax
        .items
        .iter()
        .find_map(|item| match item {
            Item::Struct(model) if model.ident == "session" => Some(model),
            _ => None,
        })
        .unwrap();
    for field in &model.fields {
        let name = field.ident.as_ref().unwrap();
        // Explicit unfinished migration: option selectors and customization
        // retain component pointers. The inventory must continue reporting them.
        let expected = if name == "options" { "crate" } else { "super" };
        assert!(
            matches!(&field.vis, syn::Visibility::Restricted(vis)
            if vis.path.is_ident(expected)),
            "Session::{name} visibility widened"
        );
    }
    let syntax = syn::parse_file(&std::fs::read_to_string("src/session/mod.rs").unwrap()).unwrap();
    for item in &syntax.items {
        let Item::Fn(function) = item else { continue };
        if matches!(function.vis, syn::Visibility::Inherited) {
            continue;
        }
        struct References;
        impl<'ast> Visit<'ast> for References {
            fn visit_type_reference(&mut self, ty: &'ast syn::TypeReference) {
                if let Type::Path(path) = &*ty.elem {
                    assert!(
                        !path.path.is_ident("session"),
                        "public helper bypasses Session"
                    );
                }
                visit::visit_type_reference(self, ty);
            }
            fn visit_type_ptr(&mut self, ty: &'ast syn::TypePtr) {
                if let Type::Path(path) = &*ty.elem {
                    assert!(
                        !path.path.is_ident("session"),
                        "public helper exposes Session pointer"
                    );
                }
                visit::visit_type_ptr(self, ty);
            }
        }
        References.visit_signature(&function.sig);
    }
}

fn rust_sources(directory: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files.extend(rust_sources(&path));
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
    files
}

#[test]
fn external_client_types_use_holders_without_exposing_model_storage() {
    struct ClientStorage<'a>(&'a Path);
    impl<'ast> Visit<'ast> for ClientStorage<'_> {
        fn visit_type_path(&mut self, ty: &'ast syn::TypePath) {
            assert!(
                !ty.path
                    .segments
                    .last()
                    .is_some_and(|part| part.ident == "client"),
                "{}: external types must use ClientRef/ClientWeak or component guards",
                self.0.display()
            );
            visit::visit_type_path(self, ty);
        }
    }
    for path in rust_sources(Path::new("src")) {
        if path.starts_with("src/server_client") || path == Path::new("src/shared/client.rs") {
            continue;
        }
        let syntax = syn::parse_file(&std::fs::read_to_string(&path).unwrap()).unwrap();
        ClientStorage(&path).visit_file(&syntax);
    }
}

#[test]
fn terminal_output_cannot_recover_a_client_from_the_component() {
    struct ComponentOnly;
    impl<'ast> Visit<'ast> for ComponentOnly {
        fn visit_item(&mut self, item: &'ast Item) {
            if matches!(item, Item::Mod(module) if module.attrs.iter().any(|attr|
                attr.path().is_ident("cfg") && attr.parse_args::<syn::Path>()
                    .is_ok_and(|path| path.is_ident("test"))))
            {
                return;
            }
            visit::visit_item(self, item);
        }
        fn visit_expr_field(&mut self, field: &'ast syn::ExprField) {
            assert!(
                !matches!(&field.member, syn::Member::Named(name) if name == "client"),
                "terminal output must not upgrade the Client back reference under a borrow"
            );
            visit::visit_expr_field(self, field);
        }
        fn visit_type_path(&mut self, ty: &'ast syn::TypePath) {
            assert!(
                !ty.path.segments.iter().any(|part| matches!(
                    part.ident.to_string().as_str(),
                    "client" | "ClientRef" | "ClientWeak"
                )),
                "terminal output must receive component data and copied metadata"
            );
            visit::visit_type_path(self, ty);
        }
    }
    for path in ["src/tty/output.rs", "src/tty_draw.rs"] {
        ComponentOnly
            .visit_file(&syn::parse_file(&std::fs::read_to_string(path).unwrap()).unwrap());
    }
}

#[test]
fn terminal_registry_and_clipping_results_do_not_retain_component_pointers() {
    struct Owned;
    impl<'ast> Visit<'ast> for Owned {
        fn visit_type_ptr(&mut self, _: &'ast syn::TypePtr) {
            panic!("terminal observation must reacquire components through its Client holder");
        }
        fn visit_type_reference(&mut self, _: &'ast syn::TypeReference) {
            panic!("terminal observation must own its result");
        }
        fn visit_type_path(&mut self, ty: &'ast syn::TypePath) {
            assert!(!ty.path.segments.iter().any(|part| part.ident == "NonNull"));
            visit::visit_type_path(self, ty);
        }
    }
    let syntax = syn::parse_file(&std::fs::read_to_string("src/tty_term.rs").unwrap()).unwrap();
    let mut checked = 0;
    for item in &syntax.items {
        if let Item::Struct(item) = item {
            if matches!(
                item.ident.to_string().as_str(),
                "TerminalRegistration" | "TerminalDescription"
            ) {
                Owned.visit_fields(&item.fields);
                checked += 1;
            }
        }
    }
    assert_eq!(checked, 2);
    let syntax = syn::parse_file(&std::fs::read_to_string("src/tty.rs").unwrap()).unwrap();
    let function = syntax
        .items
        .iter()
        .find_map(|item| match item {
            Item::Fn(function) if function.sig.ident == "tty_check_overlay_range" => Some(function),
            _ => None,
        })
        .unwrap();
    Owned.visit_return_type(&function.sig.output);
}
