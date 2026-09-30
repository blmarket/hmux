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
        "fn f(window: &WindowRef) { let model = window.get(); }",
        "fn f(window: &WindowWeak) { let model = window.upgrade().unwrap().get(); }",
        "fn f(window: &WindowRef) { let model = rc::as_ptr(window); }",
        "fn f(session: &SessionRef) { let model = session.get(); }",
        "fn f(session: &SessionWeak) { let model = session.upgrade().unwrap().get(); }",
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
        "fn f(window: &WindowRef) { let _ = Rc::as_ptr(window) as *mut window; }",
        "fn f(window: &WindowWeak) { let _ = window.as_ptr() as usize as *const window; }",
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
        "src/names.rs",
        "src/menu.rs",
        "src/resize.rs",
        "src/status.rs",
        "src/tty_acs.rs",
        "src/tty_features.rs",
        "src/tty_term.rs",
        "src/tty/output.rs",
        "src/tty/input.rs",
        "src/tty_draw.rs",
        "src/tty_keys.rs",
        "src/format/jobs.rs",
        "tests/pane_order_borrows.rs",
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
                "tty_update_window_offset",
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
fn migrated_name_and_window_notification_helpers_use_holders() {
    for (path, expected) in [
        (
            "src/window_border.rs",
            &[
                "window_render_fill_cell",
                "window_set_fill_cells",
                "window_get_fill_cell",
            ][..],
        ),
        (
            "src/layout/core.rs",
            &[
                "layout_resize_check_with_policy",
                "layout_split_check_space_with_policy",
                "layout_new_pane_size",
                "layout_set_size_check",
                "layout_resize_child_cells",
                "layout_resize_pane_grow",
                "layout_resize_pane_shrink",
                "layout_destroy_cell_with_policy",
                "layout_remove_tile_with_policy",
                "layout_resize_layout",
                "layout_resize_pane",
                "layout_replace_with_node",
                "layout_floating_pane",
                "layout_assign_pane",
                "layout_get_tiled_cell",
                "layout_get_floating_cell",
                "layout_split_floating_cell",
                "layout_resize_adjust_with_policy",
                "layout_spread_cell_with_policy",
                "layout_spread_out",
                "layout_float_pane",
                "layout_tile_pane",
                "layout_insert_tile_with_policy",
                "layout_resize_set_size_with_policy",
                "layout_fix_offsets",
                "layout_init",
                "layout_free",
                "layout_resize",
                "layout_floating_args_parse",
            ][..],
        ),
        (
            "src/layout/custom.rs",
            &["layout_assign_from_ctx", "layout_parse_apply_ctx"][..],
        ),
        (
            "src/format/callbacks.rs",
            &[
                "format_cb_window_layout",
                "format_cb_window_visible_layout",
                "format_cb_pane_modal_flag",
                "format_cb_pane_at_bottom",
                "format_cb_pane_unzoomed_width",
                "format_cb_pane_unzoomed_height",
                "format_cb_window_linked_sessions",
                "format_cb_window_modal_pane",
                "format_cb_window_name",
                "format_cb_window_zoomed_flag",
            ][..],
        ),
        (
            "src/format/expression.rs",
            &["format_add_window_neighbour"][..],
        ),
        ("src/input.rs", &["input_exit_rename"][..]),
        (
            "src/cmd/entries/break_pane.rs",
            &["cmd_break_pane_float"][..],
        ),
        (
            "src/cmd/entries/resize_pane.rs",
            &["cmd_resize_pane_mouse_resize_tiled"][..],
        ),
        (
            "src/window_tree.rs",
            &["window_tree_pull_item", "window_tree_border_cell"][..],
        ),
        (
            "src/window_panes.rs",
            &[
                "window_panes_pane_geometry",
                "window_panes_pane_floating",
                "window_panes_pane_visible",
                "window_panes_scaled_geometry",
                "window_panes_get_geometry",
                "window_panes_get_border_cell",
                "window_panes_mark_pane_status_borders",
                "window_panes_get_floating_borders",
                "window_panes_clip_floating_pane",
                "window_panes_draw_borders",
                "window_panes_draw_floating_border",
                "window_panes_clear_floating_area",
                "window_panes_draw_format",
            ][..],
        ),
        ("src/spawn.rs", &["initialize_spawned_window"][..]),
        (
            "src/cmd/entries/join_pane.rs",
            &["cmd_join_pane_finish", "cmd_join_pane_tile"][..],
        ),
        ("src/session/mod.rs", &["session_is_linked"][..]),
        ("src/monitor.rs", &["monitor_check_window"][..]),
        (
            "src/cmd/entries/select_pane.rs",
            &["cmd_select_pane_redraw"][..],
        ),
        (
            "src/screen_redraw.rs",
            &[
                "redraw_set_context",
                "redraw_reset_cell",
                "redraw_build_cells",
                "redraw_get_default_border_style",
                "redraw_make_scene",
                "redraw_get_scene",
                "redraw_invalidate_all_scenes",
            ][..],
        ),
        (
            "src/server_fn.rs",
            &[
                "server_redraw_window",
                "server_redraw_window_borders",
                "server_status_window",
            ][..],
        ),
    ] {
        let source =
            std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap();
        let syntax = syn::parse_file(&source).unwrap();
        let mut checked = 0;
        for item in &syntax.items {
            let Item::Fn(function) = item else { continue };
            if expected.contains(&function.sig.ident.to_string().as_str()) {
                let mut check = Audit {
                    consumer: true,
                    ..Default::default()
                };
                check.visit_item_fn(function);
                assert!(
                    check.findings.is_empty(),
                    "{path} {}: {:?}",
                    function.sig.ident,
                    check.findings
                );
                checked += 1;
            }
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
fn layout_snapshot_serialization_does_not_project_model_storage() {
    let source = std::fs::read_to_string("src/layout/custom.rs").unwrap();
    let syntax = syn::parse_file(&source).unwrap();
    let mut checked = false;
    for item in &syntax.items {
        let Item::Impl(implementation) = item else {
            continue;
        };
        if matches!(&*implementation.self_ty, Type::Path(path) if path.path.is_ident("LayoutSnapshot"))
        {
            let mut check = Audit {
                consumer: true,
                ..Default::default()
            };
            check.visit_item_impl(implementation);
            assert!(
                check.findings.is_empty(),
                "snapshot serializer: {:?}",
                check.findings
            );
            checked = true;
        }
    }
    assert!(checked, "snapshot serializer must remain audited");
}

#[test]
fn model_traits_do_not_return_raw_components_or_whole_models() {
    for path in [
        "src/session/api.rs",
        "src/window/api.rs",
        "src/window_pane/api.rs",
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
        assert!(
            matches!(&field.vis, syn::Visibility::Restricted(vis)
            if vis.path.is_ident("super")),
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

#[test]
fn window_state_is_private_and_pane_implementation_is_a_sibling_module() {
    let source = std::fs::read_to_string("src/window/model.rs").unwrap();
    let syntax = syn::parse_file(&source).unwrap();
    let model = syntax
        .items
        .iter()
        .find_map(|item| match item {
            Item::Struct(model) if model.ident == "window" => Some(model),
            _ => None,
        })
        .unwrap();
    for field in &model.fields {
        assert!(
            matches!(&field.vis, syn::Visibility::Restricted(vis)
            if vis.path.is_ident("super")),
            "Window field visibility widened: {:?}",
            field.ident
        );
    }
    let syntax = syn::parse_file(&std::fs::read_to_string("src/window/mod.rs").unwrap()).unwrap();
    for item in &syntax.items {
        if let Item::Mod(module) = item {
            assert!(
                !module.ident.to_string().contains("pane"),
                "Pane implementation must not inherit access to Window's private state"
            );
        }
    }
    for item in &syntax.items {
        let Item::Fn(function) = item else { continue };
        let name = function.sig.ident.to_string();
        let order_edit = [
            "window_pane_list_",
            "window_pane_z_insert_",
            "window_pane_z_remove",
        ]
        .iter()
        .any(|prefix| name.starts_with(prefix));
        let whole_model_query = matches!(
            name.as_str(),
            "windows_find"
                | "windows_next"
                | "window_winlinks_first"
                | "window_winlinks_next"
                | "window_has_pane"
                | "window_zoomed_pane"
                | "window_count_panes"
                | "window_pane_first"
                | "window_pane_last"
                | "window_pane_z_first"
                | "window_pane_z_last"
                | "window_pane_stack_first"
                | "window_pane_stack_next"
                | "window_get_pane_lines"
                | "window_get_pane_status"
        );
        if order_edit || whole_model_query || name == "window_replace_name" {
            assert!(
                matches!(function.vis, syn::Visibility::Inherited),
                "{name} must remain private; consumers use Window operations/component borrows"
            );
        }
        assert!(
            !matches!(
                name.as_str(),
                "window_pane_swap_order" | "window_pane_z_swap_order"
            ),
            "whole-model swap helpers must not be reintroduced"
        );
    }
    assert!(Path::new("src/window_pane/mod.rs").exists());
    assert!(Path::new("src/window_pane/api.rs").exists());
    assert!(Path::new("src/winlink.rs").exists());
}

// Exported helpers must use holder identities, even when an old whole-model
// helper is reintroduced under a new name. Pane signatures are a separate phase.
fn exported_window_storage_types(source: &str) -> Vec<String> {
    model_storage_types(source, "window", true)
}

fn model_storage_types(source: &str, model: &str, exported_only: bool) -> Vec<String> {
    let syntax = syn::parse_file(source).unwrap();
    struct ModelStorage {
        names: HashSet<String>,
        findings: Vec<String>,
    }
    impl<'ast> Visit<'ast> for ModelStorage {
        fn visit_use_rename(&mut self, rename: &'ast syn::UseRename) {
            if self.names.contains(&rename.ident.to_string()) {
                self.names.insert(rename.rename.to_string());
            }
        }
        fn visit_item_type(&mut self, alias: &'ast syn::ItemType) {
            if matches!(&*alias.ty, Type::Path(path)
                if path.path.segments.last().is_some_and(|part|
                    self.names.contains(&part.ident.to_string())))
            {
                self.names.insert(alias.ident.to_string());
            }
        }
        fn visit_type_path(&mut self, ty: &'ast syn::TypePath) {
            if ty
                .path
                .segments
                .last()
                .is_some_and(|part| self.names.contains(&part.ident.to_string()))
            {
                self.findings.push("type exposes model storage".into());
            }
            visit::visit_type_path(self, ty);
        }
    }
    let mut check = ModelStorage {
        names: HashSet::from([model.into()]),
        findings: Vec::new(),
    };
    check.visit_file(&syntax);
    check.findings.clear();
    if !exported_only {
        check.visit_file(&syntax);
        return check.findings;
    }
    for item in &syntax.items {
        if let Item::Fn(function) = item {
            if !matches!(function.vis, syn::Visibility::Inherited) {
                check.visit_signature(&function.sig);
            }
        }
    }
    check.findings
}

#[test]
fn session_consumers_and_exported_helpers_use_holder_identities() {
    // This covers the group module too. Its BTreeMap OccupiedEntry::get() is
    // legitimate; the separate compiler storage probe distinguishes it from
    // SessionRef::get(), including unused or inferred model projections.
    for source in [
        "fn f(state: &session) {}",
        "fn f(state: *mut session) {}",
        "fn f(state: &Rc<UnsafeCell<session>>) {}",
        "use model::session as State; fn f(state: &State) {}",
        "type State = session; fn f(state: &State) {}",
    ] {
        assert!(
            !model_storage_types(source, "session", false).is_empty(),
            "missed {source}"
        );
    }
    assert!(model_storage_types(
        "fn f(owner: &SessionRef, weak: SessionWeak) { let tag = Rc::as_ptr(owner) as u64; }",
        "session",
        false,
    )
    .is_empty());
    for path in rust_sources(Path::new("src")) {
        if path.starts_with("src/session") || path == Path::new("src/shared/session.rs") {
            continue;
        }
        let source = std::fs::read_to_string(&path).unwrap();
        assert!(
            model_storage_types(&source, "session", false).is_empty(),
            "{}: external Session types must use holders",
            path.display()
        );
    }
    let source = std::fs::read_to_string("src/session/mod.rs").unwrap();
    assert!(model_storage_types(&source, "session", true).is_empty());
    assert!(
        Path::new("src/session_group.rs").exists(),
        "groups have a separate owner boundary"
    );
}

#[test]
fn exported_window_helpers_do_not_lend_or_project_whole_models() {
    for source in [
        "pub fn f(window: &window) {}",
        "pub(crate) fn f(window: *mut window) {}",
        "pub fn f() -> Option<&'static window> { todo!() }",
        "use model::window as State; pub fn f(state: &State) {}",
        "type State = window; pub fn f(state: &State) {}",
        "pub fn f(state: &Rc<UnsafeCell<window>>) {}",
    ] {
        assert!(
            !exported_window_storage_types(source).is_empty(),
            "missed {source}"
        );
    }
    assert!(exported_window_storage_types(
        "fn private(window: &window) {} pub fn f(owner: &WindowRef, weak: WindowWeak) {}"
    )
    .is_empty());
    let source = std::fs::read_to_string("src/window/mod.rs").unwrap();
    assert_eq!(exported_window_storage_types(&source), Vec::<String>::new());
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

fn optional_layout_cell_id(ty: &Type) -> bool {
    let Type::Path(path) = ty else { return false };
    let Some(option) = path.path.segments.last() else {
        return false;
    };
    if option.ident != "Option" {
        return false;
    }
    let syn::PathArguments::AngleBracketed(arguments) = &option.arguments else {
        return false;
    };
    arguments.args.len() == 1
        && matches!(arguments.args.first(), Some(GenericArgument::Type(Type::Path(id)))
            if id.path.segments.last().is_some_and(|segment|
                segment.ident == "LayoutCellId" && matches!(segment.arguments, syn::PathArguments::None)))
}

#[test]
fn pane_layout_observers_are_optional_cell_identities() {
    for ty in [
        "*mut layout_cell",
        "*const layout_cell",
        "Option<NonNull<layout_cell>>",
        "Option<&'static layout_cell>",
        "Option<Rc<layout_cell>>",
        "Option<usize>",
        "Option<CellPointerAlias>",
        "LayoutCellId",
    ] {
        assert!(
            !optional_layout_cell_id(&syn::parse_str(ty).unwrap()),
            "missed {ty}"
        );
    }
    assert!(optional_layout_cell_id(
        &syn::parse_str("Option<LayoutCellId>").unwrap()
    ));
    let syntax =
        syn::parse_file(&std::fs::read_to_string("src/window_pane/model.rs").unwrap()).unwrap();
    let pane = syntax
        .items
        .iter()
        .find_map(|item| match item {
            Item::Struct(model) if model.ident == "window_pane" => Some(model),
            _ => None,
        })
        .expect("Pane model");
    let mut checked = 0;
    for field in &pane.fields {
        if field
            .ident
            .as_ref()
            .is_some_and(|name| name == "layout_cell" || name == "saved_layout_cell")
        {
            assert!(
                optional_layout_cell_id(&field.ty),
                "{:?} must retain only Option<LayoutCellId>; resolve through a Window guard",
                field.ident
            );
            checked += 1;
        }
    }
    assert_eq!(
        checked, 2,
        "both visible and saved layout identities must remain audited"
    );
}

fn layout_command_pointer_types(source: &str) -> Vec<String> {
    struct Check {
        cell_names: HashSet<String>,
        findings: Vec<String>,
    }
    impl<'ast> Visit<'ast> for Check {
        fn visit_item(&mut self, item: &'ast Item) {
            if matches!(item, Item::Mod(module) if module.attrs.iter().any(|attr|
                attr.path().is_ident("cfg") && attr.parse_args::<syn::Path>()
                    .is_ok_and(|path| path.is_ident("test"))))
            {
                return;
            }
            visit::visit_item(self, item);
        }
        fn visit_use_rename(&mut self, rename: &'ast syn::UseRename) {
            if self.cell_names.contains(&rename.ident.to_string()) {
                self.cell_names.insert(rename.rename.to_string());
            }
        }
        fn visit_type_path(&mut self, ty: &'ast syn::TypePath) {
            if ty
                .path
                .segments
                .last()
                .is_some_and(|segment| self.cell_names.contains(&segment.ident.to_string()))
            {
                self.findings
                    .push("command names a layout cell storage type".into());
            }
            visit::visit_type_path(self, ty);
        }
    }
    let syntax = syn::parse_file(source).unwrap();
    let mut check = Check {
        cell_names: HashSet::from(["layout_cell".into()]),
        findings: Vec::new(),
    };
    // Import aliases may appear after the use being audited.
    check.visit_file(&syntax);
    check.findings.clear();
    check.visit_file(&syntax);
    check.findings
}

#[test]
fn swap_and_rotation_cannot_reintroduce_layout_cell_pointer_types() {
    for source in [
        "fn f() { let cell: *mut layout_cell = todo!(); }",
        "fn f() { let cell = value as *const layout_cell; }",
        "fn f() { let cell = value.cast::<layout_cell>(); }",
        "fn f() -> Option<&'static layout_cell> { todo!() }",
        "type Cell = *mut layout_cell; fn f(cell: Cell) {}",
        "fn f(cell: *mut Cell) {} use model::layout_cell as Cell;",
    ] {
        assert!(
            !layout_command_pointer_types(source).is_empty(),
            "missed {source}"
        );
    }
    assert!(layout_command_pointer_types(
        "fn f(window: &WindowRef, id: LayoutCellId) { let cell = window.borrow_layout_cell_mut(id); }"
    ).is_empty());
    for path in [
        "src/cmd/entries/swap_pane.rs",
        "src/cmd/entries/rotate_window.rs",
    ] {
        assert_eq!(
            layout_command_pointer_types(&std::fs::read_to_string(path).unwrap()),
            Vec::<String>::new(),
            "{path}: keep IDs across command steps and callbacks"
        );
    }
}

// A deliberately bounded syntax check: component guard bindings in these files
// live to their lexical block end. This catches model queries and callbacks added
// to those scopes; runtime callback tests and storage probes cover other paths.
fn layout_queries_under_guards(source: &str) -> Vec<String> {
    fn creates_guard(expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::MethodCall(call)
                if matches!(
                    call.method.to_string().as_str(),
                    "borrow_layout_root"
                        | "borrow_layout_root_mut"
                        | "borrow_layout_cell"
                        | "borrow_layout_cell_mut"
                ) =>
            {
                true
            }
            syn::Expr::MethodCall(call)
                if matches!(call.method.to_string().as_str(), "unwrap" | "expect") =>
            {
                creates_guard(&call.receiver)
            }
            syn::Expr::Paren(expression) => creates_guard(&expression.expr),
            _ => false,
        }
    }
    struct Names<'a> {
        names: &'a HashSet<String>,
        found: bool,
    }
    impl<'ast> Visit<'ast> for Names<'_> {
        fn visit_expr_path(&mut self, path: &'ast syn::ExprPath) {
            self.found |= path
                .path
                .get_ident()
                .is_some_and(|name| self.names.contains(&name.to_string()));
        }
    }
    struct Check {
        windows: HashSet<String>,
        guarded: bool,
        findings: Vec<String>,
    }
    impl Check {
        fn uses_window(&self, expression: &syn::Expr) -> bool {
            let mut names = Names {
                names: &self.windows,
                found: false,
            };
            names.visit_expr(expression);
            names.found
        }
    }
    impl<'ast> Visit<'ast> for Check {
        fn visit_block(&mut self, block: &'ast syn::Block) {
            let outer = self.guarded;
            for statement in &block.stmts {
                self.visit_stmt(statement);
                if matches!(statement, syn::Stmt::Local(local)
                    if local.init.as_ref().is_some_and(|init| creates_guard(&init.expr)))
                {
                    self.guarded = true;
                }
            }
            self.guarded = outer;
        }
        fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
            if self.guarded
                && ((self.uses_window(&call.receiver) && call.method != "clone")
                    || matches!(
                        call.method.to_string().as_str(),
                        "is_floating"
                            | "border_status"
                            | "outer_geometry"
                            | "window_handle"
                            | "with_options_mut"
                            | "resize"
                            | "select_pane"
                    ))
            {
                self.findings.push(format!(
                    "{} may reenter a model under a layout guard",
                    call.method
                ));
            }
            visit::visit_expr_method_call(self, call);
        }
        fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
            let name = match &*call.func {
                syn::Expr::Path(path) => {
                    path.path.segments.last().map(|part| part.ident.to_string())
                }
                _ => None,
            };
            if self.guarded
                && name.as_deref().is_some_and(|name| {
                    matches!(
                        name,
                        "layout_fix_panes"
                            | "layout_fix_offsets"
                            | "window_resize"
                            | "window_pane_resize"
                            | "window_set_active_pane"
                            | "window_pane_is_floating"
                            | "server_redraw_window"
                            | "events_fire_window"
                            | "recalculate_sizes"
                            | "layout_parse_apply_ctx"
                    ) || (!matches!(name, "clone" | "downgrade")
                        && call.args.iter().any(|arg| self.uses_window(arg)))
                })
            {
                self.findings.push(format!(
                    "{} may dispatch or borrow the Window under a layout guard",
                    name.unwrap()
                ));
            }
            visit::visit_expr_call(self, call);
        }
    }
    let syntax = syn::parse_file(source).unwrap();
    let mut findings = Vec::new();
    for item in &syntax.items {
        let Item::Fn(function) = item else { continue };
        let mut check = Check {
            windows: HashSet::new(),
            guarded: false,
            findings: Vec::new(),
        };
        for argument in &function.sig.inputs {
            let syn::FnArg::Typed(argument) = argument else {
                continue;
            };
            let syn::Pat::Ident(name) = &*argument.pat else {
                continue;
            };
            if matches!(&*argument.ty, Type::Reference(reference)
                if matches!(&*reference.elem, Type::Path(path)
                    if path.path.segments.last().is_some_and(|segment| segment.ident == "WindowRef")))
            {
                check.windows.insert(name.ident.to_string());
            }
        }
        check.visit_block(&function.block);
        findings.extend(
            check
                .findings
                .into_iter()
                .map(|finding| format!("{}: {finding}", function.sig.ident)),
        );
    }
    findings
}

#[test]
fn layout_borrow_scopes_end_before_model_queries_and_callbacks() {
    for source in [
        "fn f(window: &WindowRef) { let tree = window.borrow_layout_root_mut(); window.size(); }",
        "fn f(window: &WindowRef, pane: &PaneRef) { let tree = window.borrow_layout_root_mut(); pane.is_floating(); }",
        "fn f(window: &WindowRef) { let tree = window.borrow_layout_root_mut(); layout_fix_panes(window, None); }",
        "fn f(window: &WindowRef) { let tree = window.borrow_layout_root_mut(); helper(window); }",
        "fn f(window: &WindowRef, id: LayoutCellId) { let Some(cell) = window.borrow_layout_cell_mut(id) else { return }; events_fire_window(c\"event\", Rc::clone(window)); }",
    ] {
        assert!(!layout_queries_under_guards(source).is_empty(), "missed {source}");
    }
    assert!(layout_queries_under_guards(
        "fn f(window: &WindowRef) { let size = window.size(); { let tree = window.borrow_layout_root_mut(); edit(&mut tree); } layout_fix_panes(window, None); }"
    ).is_empty());
    for path in [
        "src/layout/set.rs",
        "src/layout/custom.rs",
        "src/cmd/entries/swap_pane.rs",
        "src/cmd/entries/rotate_window.rs",
    ] {
        assert_eq!(
            layout_queries_under_guards(&std::fs::read_to_string(path).unwrap()),
            Vec::<String>::new(),
            "{path}"
        );
    }
}

#[test]
fn layout_restore_state_cannot_carry_cells_or_own_panes_across_callbacks() {
    let source =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/layout/custom.rs"))
            .unwrap();
    let syntax = syn::parse_file(&source).unwrap();
    struct RestoreTypes;
    impl<'ast> Visit<'ast> for RestoreTypes {
        fn visit_type_ptr(&mut self, _: &'ast syn::TypePtr) {
            panic!("layout restoration must not retain raw component pointers");
        }
        fn visit_type_reference(&mut self, _: &'ast syn::TypeReference) {
            panic!("restoration records must not retain component borrows");
        }
        fn visit_type_path(&mut self, ty: &'ast syn::TypePath) {
            for segment in &ty.path.segments {
                assert!(
                    !matches!(
                        segment.ident.to_string().as_str(),
                        "layout_cell" | "layout_parse_ctx" | "Rc" | "NonNull"
                    ),
                    "restoration records must contain values and weak identities"
                );
            }
            visit::visit_type_path(self, ty);
        }
    }
    let record = syntax
        .items
        .iter()
        .find_map(|item| match item {
            Item::Struct(record) if record.ident == "LayoutPaneRestore" => Some(record),
            _ => None,
        })
        .expect("typed restoration record");
    RestoreTypes.visit_item_struct(record);
}

#[test]
fn layout_reservations_do_not_pass_cell_pointers_across_callbacks() {
    struct NoCellPointer;
    impl<'ast> Visit<'ast> for NoCellPointer {
        fn visit_type_ptr(&mut self, _: &'ast syn::TypePtr) {
            panic!("layout reservations must carry an ID, not a component pointer");
        }
        fn visit_type_reference(&mut self, _: &'ast syn::TypeReference) {
            panic!("layout reservations cannot retain a component borrow");
        }
        fn visit_type_path(&mut self, ty: &'ast syn::TypePath) {
            assert!(!ty
                .path
                .segments
                .iter()
                .any(|s| matches!(s.ident.to_string().as_str(), "layout_cell" | "NonNull")));
            visit::visit_type_path(self, ty);
        }
    }
    struct CellArgumentTypes;
    impl<'ast> Visit<'ast> for CellArgumentTypes {
        fn visit_type_ptr(&mut self, _: &'ast syn::TypePtr) {
            panic!("layout operations must resolve identities under a tree borrow");
        }
        fn visit_type_path(&mut self, ty: &'ast syn::TypePath) {
            assert!(!ty
                .path
                .segments
                .iter()
                .any(|s| matches!(s.ident.to_string().as_str(), "layout_cell" | "NonNull")));
            visit::visit_type_path(self, ty);
        }
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let spawn =
        syn::parse_file(&std::fs::read_to_string(root.join("src/shared/spawn.rs")).unwrap())
            .unwrap();
    let context = spawn
        .items
        .iter()
        .find_map(|item| match item {
            Item::Struct(item) if item.ident == "spawn_context" => Some(item),
            _ => None,
        })
        .unwrap();
    NoCellPointer.visit_item_struct(context);
    let core = syn::parse_file(&std::fs::read_to_string(root.join("src/layout/core.rs")).unwrap())
        .unwrap();
    let expected = [
        "layout_split_pane",
        "layout_floating_pane",
        "layout_get_tiled_cell",
        "layout_get_floating_cell",
    ];
    let mut checked = 0;
    for item in &core.items {
        if let Item::Fn(function) = item {
            if expected.contains(&function.sig.ident.to_string().as_str()) {
                NoCellPointer.visit_return_type(&function.sig.output);
                checked += 1;
            }
        }
    }
    let removed_pointer_wrappers = [
        "layout_insert_tile",
        "layout_resize_check",
        "layout_resize_adjust",
        "layout_resize_set_size",
        "layout_split_check_space",
    ];
    let mut consumers = 0;
    for item in &core.items {
        if let Item::Fn(function) = item {
            assert!(
                !removed_pointer_wrappers.contains(&function.sig.ident.to_string().as_str()),
                "removed Window/cell pointer wrapper must not return: {}",
                function.sig.ident,
            );
            if matches!(
                function.sig.ident.to_string().as_str(),
                "layout_assign_pane"
                    | "layout_resize_layout"
                    | "layout_float_pane"
                    | "layout_tile_pane"
                    | "layout_spread_out"
            ) {
                CellArgumentTypes.visit_signature(&function.sig);
                consumers += 1;
            }
        }
    }
    assert_eq!(consumers, 5);
    assert_eq!(checked, expected.len());
}

#[test]
fn option_inheritance_retains_owner_identity_without_component_pointers() {
    fn path_name(ty: &Type, expected: &str) -> bool {
        matches!(ty, Type::Path(path) if path.path.segments.last().is_some_and(|s| s.ident == expected))
    }
    fn only_argument<'a>(ty: &'a Type, expected: &str) -> Option<&'a Type> {
        let Type::Path(path) = ty else { return None };
        let last = path.path.segments.last()?;
        if last.ident != expected {
            return None;
        }
        let syn::PathArguments::AngleBracketed(arguments) = &last.arguments else {
            return None;
        };
        if arguments.args.len() != 1 {
            return None;
        }
        match arguments.args.first()? {
            GenericArgument::Type(inner) => Some(inner),
            _ => None,
        }
    }
    fn model_observer(ty: &Type, model: &str, alias: &str) -> bool {
        if path_name(ty, alias) {
            return true;
        }
        let Some(cell) = only_argument(ty, "Weak") else {
            return false;
        };
        ["UnsafeCell", "RefCell"].iter().any(|storage| {
            only_argument(cell, storage).is_some_and(|inner| path_name(inner, model))
        })
    }
    fn optional_scope(ty: &Type) -> bool {
        only_argument(ty, "Option").is_some_and(|inner| path_name(inner, "OptionsScope"))
    }

    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let scope =
        syn::parse_file(&std::fs::read_to_string(root.join("src/options/scope.rs")).unwrap())
            .unwrap();
    let identity = scope
        .items
        .iter()
        .find_map(|item| match item {
            Item::Enum(item) if item.ident == "OptionsScope" => Some(item),
            _ => None,
        })
        .expect("persistent option scope identity");
    assert_eq!(identity.variants.len(), 6);
    for variant in &identity.variants {
        let model = match variant.ident.to_string().as_str() {
            "GlobalServer" | "GlobalSession" | "GlobalWindow" => {
                assert!(matches!(variant.fields, syn::Fields::Unit));
                continue;
            }
            "Session" => ("session", "SessionWeak"),
            "Window" => ("window", "WindowWeak"),
            "Pane" => ("window_pane", "PaneWeak"),
            other => panic!("option scope must name a known owner: {other}"),
        };
        let syn::Fields::Unnamed(fields) = &variant.fields else {
            panic!("model option scope must carry one Weak owner");
        };
        assert_eq!(fields.unnamed.len(), 1);
        assert!(
            model_observer(&fields.unnamed[0].ty, model.0, model.1),
            "{} scope cannot store a component reference, pointer or strong owner",
            variant.ident,
        );
    }
    let mut audit = Audit {
        consumer: true,
        ..Default::default()
    };
    audit.visit_file(&scope);
    assert!(audit.findings.is_empty(), "{:?}", audit.findings);

    let shared =
        syn::parse_file(&std::fs::read_to_string(root.join("src/shared/options.rs")).unwrap())
            .unwrap();
    let table = shared
        .items
        .iter()
        .find_map(|item| match item {
            Item::Struct(item) if item.ident == "options" => Some(item),
            _ => None,
        })
        .expect("option table");
    let parent = table
        .fields
        .iter()
        .find(|field| field.ident.as_ref().is_some_and(|name| name == "parent"))
        .expect("inheritance parent");
    assert!(
        optional_scope(&parent.ty),
        "option parent must retain only its scope identity"
    );

    let functions =
        syn::parse_file(&std::fs::read_to_string(root.join("src/options.rs")).unwrap()).unwrap();
    let constructors = [
        "options_create",
        "options_create_owned",
        "options_set_parent",
    ];
    let mut checked = 0;
    for item in &functions.items {
        let Item::Fn(function) = item else { continue };
        assert_ne!(
            function.sig.ident, "options_get",
            "inherited entries cannot be returned as unguarded observers"
        );
        if !constructors.contains(&function.sig.ident.to_string().as_str()) {
            continue;
        }
        let parent = function.sig.inputs.iter().find_map(|argument| match argument {
            syn::FnArg::Typed(argument) if matches!(&*argument.pat, syn::Pat::Ident(name) if name.ident == "parent") => Some(&*argument.ty),
            _ => None,
        }).expect("typed inheritance argument");
        assert!(
            optional_scope(parent),
            "{} must accept an owner identity",
            function.sig.ident
        );
        checked += 1;
    }
    assert_eq!(checked, constructors.len());
}

#[test]
fn hook_monitor_callbacks_carry_scopes_and_generations_without_component_observers() {
    struct NoMonitorObserver;
    impl<'ast> Visit<'ast> for NoMonitorObserver {
        fn visit_type_path(&mut self, ty: &'ast syn::TypePath) {
            assert!(
                !ty.path.segments.iter().any(|segment| matches!(
                    segment.ident.to_string().as_str(),
                    "options" | "options_entry" | "hooks_monitor" | "NonNull"
                )),
                "hook callback arguments must carry owner identity, not component observers",
            );
            visit::visit_type_path(self, ty);
        }
        fn visit_type_ptr(&mut self, _: &'ast syn::TypePtr) {
            panic!("hook callback state cannot contain raw pointers");
        }
    }
    struct NoMonitorAddress;
    impl<'ast> Visit<'ast> for NoMonitorAddress {
        fn visit_expr_raw_addr(&mut self, _: &'ast syn::ExprRawAddr) {
            panic!("monitor registration cannot retain the monitor's component address");
        }
        fn visit_expr_cast(&mut self, cast: &'ast syn::ExprCast) {
            if let Type::Path(path) = &*cast.ty {
                assert!(
                    !path.path.segments.last().is_some_and(|segment| matches!(
                        segment.ident.to_string().as_str(),
                        "usize" | "isize" | "u64"
                    )),
                    "hook monitor identities must not be encoded pointer addresses",
                );
            }
            visit::visit_expr_cast(self, cast);
        }
    }
    let syntax = syn::parse_file(
        &std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/hooks.rs"))
            .unwrap(),
    )
    .unwrap();
    let data = syntax
        .items
        .iter()
        .find_map(|item| match item {
            Item::Struct(item) if item.ident == "hooks_data" => Some(item),
            _ => None,
        })
        .expect("hook dispatch data");
    let scope = data
        .fields
        .iter()
        .find(|field| field.ident.as_ref().is_some_and(|name| name == "oo"))
        .expect("hook dispatch option scope");
    NoMonitorObserver.visit_type(&scope.ty);
    let Type::Path(path) = &scope.ty else {
        panic!("hook option scope must be owned")
    };
    let option = path.path.segments.last().unwrap();
    assert_eq!(option.ident, "Option");
    let syn::PathArguments::AngleBracketed(arguments) = &option.arguments else {
        panic!("optional owner scope")
    };
    assert!(
        matches!(arguments.args.first(), Some(GenericArgument::Type(Type::Path(scope))) if scope.path.segments.last().is_some_and(|segment| segment.ident == "OptionsScope"))
    );

    let callbacks = ["hooks_monitor_cb", "hooks_monitor_hook_cb"];
    let mut checked = 0;
    let mut registrations = 0;
    for item in &syntax.items {
        let Item::Fn(function) = item else { continue };
        if callbacks.contains(&function.sig.ident.to_string().as_str()) {
            NoMonitorObserver.visit_signature(&function.sig);
            checked += 1;
        }
        if function.sig.ident == "hooks_monitor_add" {
            NoMonitorAddress.visit_block(&function.block);
            registrations += 1;
        }
    }
    assert_eq!(checked, callbacks.len());
    assert_eq!(registrations, 1);
}

#[test]
fn show_options_rows_are_owned_before_delivery_to_formatting() {
    struct OwnedRow;
    impl<'ast> Visit<'ast> for OwnedRow {
        fn visit_type_ptr(&mut self, _: &'ast syn::TypePtr) {
            panic!("option display rows cannot retain component pointers");
        }
        fn visit_type_reference(&mut self, _: &'ast syn::TypeReference) {
            panic!("option display rows cannot retain component borrows");
        }
        fn visit_type_path(&mut self, ty: &'ast syn::TypePath) {
            assert!(
                !ty.path.segments.iter().any(|segment| matches!(
                    segment.ident.to_string().as_str(),
                    "options"
                        | "options_entry"
                        | "options_array_item"
                        | "hooks_monitor"
                        | "NonNull"
                        | "Rc"
                        | "Weak"
                        | "Ref"
                        | "RefMut"
                        | "UnsafeCell"
                        | "RefCell"
                )),
                "option display rows must contain copied text and scalar metadata"
            );
            visit::visit_type_path(self, ty);
        }
    }
    let syntax = syn::parse_file(
        &std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("src/cmd/entries/show_options.rs"),
        )
        .unwrap(),
    )
    .unwrap();
    let records = ["ShowOptionRecord", "ShowOptionRows", "ShowMonitorRecord"];
    let mut checked = 0;
    let mut deliveries = 0;
    for item in &syntax.items {
        match item {
            Item::Struct(record) if records.contains(&record.ident.to_string().as_str()) => {
                OwnedRow.visit_item_struct(record);
                checked += 1;
            }
            Item::Enum(record) if records.contains(&record.ident.to_string().as_str()) => {
                OwnedRow.visit_item_enum(record);
                checked += 1;
            }
            Item::Fn(function)
                if matches!(
                    function.sig.ident.to_string().as_str(),
                    "cmd_show_options_print_record" | "cmd_show_hooks_print_monitor"
                ) =>
            {
                let record = function.sig.inputs.iter().find_map(|argument| match argument {
                    syn::FnArg::Typed(argument) if matches!(&*argument.pat, syn::Pat::Ident(name) if name.ident == "record") => Some(&*argument.ty),
                    _ => None,
                }).expect("owned display record argument");
                OwnedRow.visit_type(record);
                assert!(
                    matches!(record, Type::Path(path) if path.path.segments.last().is_some_and(|segment| matches!(segment.ident.to_string().as_str(), "ShowOptionRecord" | "ShowMonitorRecord")))
                );
                deliveries += 1;
            }
            _ => {}
        }
    }
    assert_eq!(checked, records.len());
    assert_eq!(deliveries, 2);
}

#[test]
fn option_style_adapters_return_values_without_lending_cached_style_storage() {
    struct OwnedStyle;
    impl<'ast> Visit<'ast> for OwnedStyle {
        fn visit_type_ptr(&mut self, _: &'ast syn::TypePtr) {
            panic!("style resolution must copy its result out of the option cache");
        }
        fn visit_type_reference(&mut self, _: &'ast syn::TypeReference) {
            panic!("style resolution cannot lend an inherited model component");
        }
        fn visit_type_path(&mut self, ty: &'ast syn::TypePath) {
            assert!(!ty.path.segments.iter().any(|segment| matches!(
                segment.ident.to_string().as_str(),
                "options" | "options_entry" | "NonNull" | "Ref" | "RefMut"
            )));
            visit::visit_type_path(self, ty);
        }
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for (path, name, optional) in [
        ("src/options.rs", "options_string_to_style", true),
        ("src/style/parsing.rs", "style_add", false),
        ("src/style/scoped.rs", "style_resolve_with_options", true),
    ] {
        let syntax = syn::parse_file(&std::fs::read_to_string(root.join(path)).unwrap()).unwrap();
        let function = syntax
            .items
            .iter()
            .find_map(|item| match item {
                Item::Fn(function) if function.sig.ident == name => Some(function),
                _ => None,
            })
            .expect("style value adapter");
        OwnedStyle.visit_return_type(&function.sig.output);
        let syn::ReturnType::Type(_, value) = &function.sig.output else {
            panic!("{name} must return the copied style");
        };
        let Type::Path(path) = &**value else {
            panic!("owned style value")
        };
        let value = path.path.segments.last().unwrap();
        if optional {
            assert_eq!(value.ident, "Option");
            let syn::PathArguments::AngleBracketed(arguments) = &value.arguments else {
                panic!("optional copied style");
            };
            assert!(
                matches!(arguments.args.first(), Some(GenericArgument::Type(Type::Path(style))) if style.path.segments.last().is_some_and(|segment| segment.ident == "style"))
            );
        } else {
            assert_eq!(value.ident, "style");
        }
    }
}

// Monitors are independently retained components: a raw observer into a Client
// or options table would evade the opaque Window storage compile probe.
fn monitor_storage_boundary(source: &str) -> Vec<String> {
    fn named(ty: &Type, name: &str) -> bool {
        matches!(ty, Type::Path(path) if path.path.segments.last().is_some_and(|part| part.ident == name))
    }
    fn optional_holder(ty: &Type) -> bool {
        let Type::Path(path) = ty else { return false };
        let Some(segment) = path.path.segments.last() else {
            return false;
        };
        let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
            return false;
        };
        segment.ident == "Option"
            && args.args.len() == 1
            && matches!(args.args.first(), Some(GenericArgument::Type(ty)) if named(ty, "MonitorRef"))
    }
    #[derive(Default)]
    struct Exposure {
        state: bool,
        pointer: bool,
    }
    impl<'ast> Visit<'ast> for Exposure {
        fn visit_type_path(&mut self, ty: &'ast syn::TypePath) {
            self.state |= ty
                .path
                .segments
                .last()
                .is_some_and(|part| part.ident == "MonitorState");
            visit::visit_type_path(self, ty);
        }
        fn visit_type_ptr(&mut self, ty: &'ast syn::TypePtr) {
            self.pointer = true;
            visit::visit_type_ptr(self, ty);
        }
    }
    #[derive(Default)]
    struct Boundary(Vec<String>);
    impl<'ast> Visit<'ast> for Boundary {
        fn visit_item_struct(&mut self, model: &'ast syn::ItemStruct) {
            let name = model.ident.to_string();
            if matches!(name.as_str(), "MonitorRef" | "MonitorWeak" | "MonitorState") {
                if model
                    .fields
                    .iter()
                    .any(|field| !matches!(field.vis, syn::Visibility::Inherited))
                {
                    self.0.push(format!("{name} exposes monitor storage"));
                }
                if name == "MonitorState" && !matches!(model.vis, syn::Visibility::Inherited) {
                    self.0.push("MonitorState must remain private".into());
                }
            }
            let owner_field = match name.as_str() {
                "control_state" => Some("subs"),
                "hooks_monitor" => Some("set"),
                _ => None,
            };
            if let Some(name) = owner_field {
                if !model.fields.iter().any(|field| {
                    field.ident.as_ref().is_some_and(|id| id == name) && optional_holder(&field.ty)
                }) {
                    self.0.push(format!(
                        "{}::{name} must own Option<MonitorRef>",
                        model.ident
                    ));
                }
            }
            if name == "monitor_set" {
                self.0.push("obsolete raw monitor_set declaration".into());
            }
            visit::visit_item_struct(self, model);
        }
        fn visit_item_type(&mut self, alias: &'ast syn::ItemType) {
            if matches!(
                alias.ident.to_string().as_str(),
                "MonitorRef" | "MonitorWeak" | "monitor_set"
            ) {
                self.0.push("monitor holders must be opaque structs".into());
            }
            visit::visit_item_type(self, alias);
        }
        fn visit_type_path(&mut self, ty: &'ast syn::TypePath) {
            if ty
                .path
                .segments
                .last()
                .is_some_and(|part| part.ident == "monitor_set")
            {
                self.0.push("obsolete raw monitor_set type".into());
            }
            visit::visit_type_path(self, ty);
        }
        fn visit_item_fn(&mut self, function: &'ast syn::ItemFn) {
            if function.sig.ident == "monitor_timer" {
                let retained = function.sig.inputs.len() == 1
                    && matches!(function.sig.inputs.first(),
                    Some(syn::FnArg::Typed(arg)) if matches!(&*arg.ty,
                        Type::Reference(reference) if named(&reference.elem, "MonitorRef")));
                if !retained {
                    self.0
                        .push("monitor timer must receive a retained holder reference".into());
                }
            }
            if !matches!(function.vis, syn::Visibility::Inherited) {
                let mut exposure = Exposure::default();
                exposure.visit_signature(&function.sig);
                if exposure.state {
                    self.0
                        .push("exported monitor helper exposes private state".into());
                }
            }
            visit::visit_item_fn(self, function);
        }
        fn visit_item_impl(&mut self, implementation: &'ast syn::ItemImpl) {
            if named(&implementation.self_ty, "MonitorRef")
                || named(&implementation.self_ty, "MonitorWeak")
            {
                if implementation.trait_.is_some() {
                    let mut exposure = Exposure::default();
                    exposure.visit_item_impl(implementation);
                    if exposure.state {
                        self.0
                            .push("holder trait projects private monitor state".into());
                    }
                }
                for item in &implementation.items {
                    let syn::ImplItem::Fn(method) = item else {
                        continue;
                    };
                    if !matches!(method.vis, syn::Visibility::Inherited) {
                        let mut signature = Exposure::default();
                        signature.visit_signature(&method.sig);
                        let mut output = Exposure::default();
                        output.visit_return_type(&method.sig.output);
                        if signature.state || output.pointer {
                            self.0.push("holder method exposes monitor storage".into());
                        }
                    }
                }
            }
            visit::visit_item_impl(self, implementation);
        }
    }
    let mut boundary = Boundary::default();
    boundary.visit_file(&syn::parse_file(source).expect("Rust monitor source"));
    boundary.0
}

#[test]
fn monitor_boundary_rejects_raw_observers_and_public_storage() {
    for source in [
        "pub struct MonitorRef(pub Rc<UnsafeCell<MonitorState>>);",
        "pub struct MonitorWeak(pub(crate) Weak<UnsafeCell<MonitorState>>);",
        "pub struct MonitorState { items: monitor_items }",
        "struct MonitorState { pub(crate) items: monitor_items }",
        "pub type MonitorRef = Rc<UnsafeCell<MonitorState>>;",
        "unsafe fn monitor_timer(set: *mut monitor_set) {}",
        "unsafe fn monitor_timer(set: &MonitorState) {}",
        "struct State { monitor: *mut monitor_set }",
        "struct control_state { subs: Option<Box<MonitorState>> }",
        "struct hooks_monitor { set: *mut MonitorState }",
        "impl MonitorRef { pub fn raw(&self) -> *mut core::ffi::c_void { todo!() } }",
        "impl MonitorRef { pub fn state(&self) -> &MonitorState { todo!() } }",
        "impl Deref for MonitorRef { type Target = MonitorState; }",
    ] {
        assert!(
            !monitor_storage_boundary(source).is_empty(),
            "missed {source}"
        );
    }
    assert!(monitor_storage_boundary(
        "pub struct MonitorRef(Rc<RefCell<MonitorState>>); pub struct MonitorWeak(Weak<RefCell<MonitorState>>); struct MonitorState { items: monitor_items } unsafe fn monitor_timer(owner: &MonitorRef) {} struct control_state { subs: Option<MonitorRef> } struct hooks_monitor { set: Option<MonitorRef> } fn child_index(item: *mut monitor_item) {}"
    ).is_empty());
}

#[test]
fn monitors_keep_independent_opaque_holders_across_callbacks() {
    for path in rust_sources(Path::new("src")) {
        let source = std::fs::read_to_string(&path).unwrap();
        assert_eq!(
            monitor_storage_boundary(&source),
            Vec::<String>::new(),
            "{}",
            path.display()
        );
    }
    let monitor = syn::parse_file(&std::fs::read_to_string("src/monitor.rs").unwrap()).unwrap();
    for name in ["MonitorRef", "MonitorWeak", "MonitorState"] {
        assert!(
            monitor
                .items
                .iter()
                .any(|item| matches!(item, Item::Struct(model) if model.ident == name)),
            "missing opaque monitor declaration {name}"
        );
    }
    assert!(monitor
        .items
        .iter()
        .any(|item| matches!(item, Item::Fn(function) if function.sig.ident == "monitor_timer")));
}
