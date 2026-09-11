use super::*;
use crate::WindowPane;
use crate::options::{OptionsEngine, OptionsRef, RustOptionsEngine};

use crate::tests::test_fixtures::{KeyTable, Target, globals, zeroed_client};
use core::ffi::c_int;

struct Chain {
    pane: RustOptionsRef,
    window: RustOptionsRef,
    session: RustOptionsRef,
}

impl Chain {
    unsafe fn new(target: &mut Target) -> Self {
        unsafe {
            let fs = target.state();
            let window = fs.window().unwrap();
            let pane = window
                .pane_by_id(fs.wp.as_ref().map(|pane| pane.id()).unwrap())
                .unwrap();
            let pane = pane.get().unwrap().options_ref().clone();
            let window = window.options();
            let session = fs.session().unwrap().options();
            pane.set_parent(Some(&window));
            window.set_parent(global_w_options.get().as_ref());
            session.set_parent(global_s_options.get().as_ref());
            Self {
                pane,
                window,
                session,
            }
        }
    }
}

impl Drop for Chain {
    fn drop(&mut self) {
        self.pane.set_parent(None);
        self.window.set_parent(None);
        self.session.set_parent(None);
    }
}

fn data(target: &mut Target) -> WindowCustomizeModeDataRef {
    WindowCustomizeModeDataRef::new(window_customize_modedata {
        wp: target.state().pane_ref(),
        data: None,
        format: Some(WINDOW_CUSTOMIZE_DEFAULT_FORMAT.to_owned()),
        hide_global: 0,
        prompt_flags: 0,
        item_list: Vec::new(),
        tags: Default::default(),
        next_tag: 0,
        fs: target.state(),
        change: WINDOW_CUSTOMIZE_UNSET,
        owner: None,
    })
}

fn option_item(
    scope: window_customize_scope,
    oo: &RustOptionsRef,
    name: &CStr,
    idx: c_int,
) -> window_customize_itemdata {
    window_customize_itemdata {
        scope,
        oo: Some(oo.clone()),
        name: Some(name.to_owned()),
        idx,
        ..Default::default()
    }
}

#[test]
fn case_conversion_and_static_metadata_cover_boundaries() {
    assert_eq!(tolower(b'A'), b'a');
    assert_eq!(tolower(b'z'), b'z');
    assert_eq!(tolower(0), 0);
    assert_eq!(toupper(b'a'), b'A');
    assert_eq!(toupper(b'Z'), b'Z');
    assert_eq!(toupper(0xff), 0xff);
    let (lines, width, noun) = window_customize_help();
    assert_eq!(lines.len(), 9);
    assert_eq!(width, 52);
    assert_eq!(noun, c"option");
    let choice = RustOptionsEngine
        .table()
        .iter()
        .find(|entry| entry.choices.is_some())
        .unwrap();
    assert!(!window_customize_choice_list(choice).as_bytes().is_empty());
}

#[test]
fn scope_tree_and_text_map_to_the_target_owners() {
    let _guard = globals();
    let mut target = Target::new(80, 24);
    let fs = target.state();
    unsafe {
        assert!(window_customize_get_tree(WINDOW_CUSTOMIZE_NONE, &fs).is_none());
        assert!(window_customize_get_tree(WINDOW_CUSTOMIZE_KEY, &fs).is_none());
        assert!(window_customize_get_tree(WINDOW_CUSTOMIZE_SERVER, &fs) == global_options.get());
        assert!(
            window_customize_get_tree(WINDOW_CUSTOMIZE_GLOBAL_SESSION, &fs)
                == global_s_options.get()
        );
        assert!(
            window_customize_get_tree(WINDOW_CUSTOMIZE_GLOBAL_WINDOW, &fs)
                == global_w_options.get()
        );
        assert!(
            window_customize_get_tree(WINDOW_CUSTOMIZE_SESSION, &fs)
                == Some(fs.session().unwrap().options())
        );
        assert!(
            window_customize_get_tree(WINDOW_CUSTOMIZE_WINDOW, &fs)
                == Some((*target.window(0)).options_ref().clone())
        );
        assert!(
            window_customize_get_tree(WINDOW_CUSTOMIZE_PANE, &fs)
                == Some((*target.pane(0)).options_ref().clone())
        );
        assert!(window_customize_get_tree(99, &fs).is_none());
        assert_eq!(
            window_customize_scope_text(WINDOW_CUSTOMIZE_SERVER, &fs),
            c""
        );
        assert!(
            window_customize_scope_text(WINDOW_CUSTOMIZE_SESSION, &fs)
                .to_bytes()
                .starts_with(b"session ")
        );
        assert_eq!(
            window_customize_scope_text(WINDOW_CUSTOMIZE_WINDOW, &fs),
            c"window 0"
        );
        assert_eq!(
            window_customize_scope_text(WINDOW_CUSTOMIZE_PANE, &fs),
            c"pane 0"
        );
    }
}

#[test]
fn item_allocation_tags_and_validity_are_deterministic() {
    let _guard = globals();
    let mut target = Target::new(80, 24);
    let data = data(&mut target);
    unsafe {
        let item = window_customize_add_item(
            &mut data.borrow_mut(),
            window_customize_itemdata {
                scope: WINDOW_CUSTOMIZE_WINDOW,
                oo: Some((*target.window(0)).options_ref().clone()),
                ..Default::default()
            },
        );
        assert_eq!(data.check_item(&item, None), 1);
        let item = window_customize_add_item(
            &mut data.borrow_mut(),
            window_customize_itemdata {
                scope: WINDOW_CUSTOMIZE_WINDOW,
                oo: global_w_options.get(),
                ..Default::default()
            },
        );
        assert_eq!(data.check_item(&item, None), 0);

        let fs = target.state();
        global_w_options
            .get()
            .as_ref()
            .unwrap()
            .with_entry(c"automatic-rename", false, |entry| {
                let entry = entry.unwrap();
                let mut data = data.borrow_mut();
                let first = data.option_tag(WINDOW_CUSTOMIZE_GLOBAL_WINDOW, &fs, entry, -1);
                assert_eq!(
                    first,
                    data.option_tag(WINDOW_CUSTOMIZE_WINDOW, &fs, entry, -1)
                );
                assert_ne!(
                    first,
                    data.option_tag(WINDOW_CUSTOMIZE_WINDOW, &fs, entry, 7)
                );
            });
    }
}

#[test]
fn user_option_discovery_deduplicates_and_ignores_builtins() {
    let _guard = globals();
    let mut target = Target::new(80, 24);
    unsafe {
        let oo = &*(*target.window(0)).options_ref();
        (oo).set_string(c"@alpha", 0, c"one", &[]);
        (oo).set_string(c"@beta", 0, c"two", &[]);
        let mut list = Vec::new();
        window_customize_find_user_options(oo, &mut list);
        window_customize_find_user_options(oo, &mut list);
        assert_eq!(list.len(), 2);
        let names: Vec<_> = list.iter().map(|name| name.as_bytes()).collect();
        assert!(names.contains(&b"@alpha".as_slice()));
        assert!(names.contains(&b"@beta".as_slice()));
    }
}

#[test]
fn option_unset_and_reset_mutate_only_matching_live_owners() {
    let _guard = globals();
    let mut target = Target::new(80, 24);
    let data = data(&mut target);
    unsafe {
        let oo = (*target.window(0)).options_ref();
        (*oo).set_string(c"@temporary", 0, c"value", &[]);
        let item = std::rc::Rc::new(option_item(WINDOW_CUSTOMIZE_WINDOW, oo, c"@temporary", -1));
        data.unset_option(Some(&item));
        assert!((*oo).with_entry(c"@temporary", true, |entry| entry.is_none()));

        (*oo).set_string(c"@temporary", 0, c"again", &[]);
        data.reset_option(Some(&item));
        assert!((*oo).with_entry(c"@temporary", true, |entry| entry.is_none()));

        let wrong = option_item(
            WINDOW_CUSTOMIZE_WINDOW,
            global_w_options.get().as_ref().unwrap(),
            c"@temporary",
            -1,
        );
        let wrong = std::rc::Rc::new(wrong);
        data.unset_option(Some(&wrong));
        data.reset_option(Some(&wrong));
        data.unset_option(None);
        data.reset_option(None);
    }
}

#[test]
fn key_lookup_and_prompt_callbacks_reject_stale_items() {
    let _guard = globals();
    let mut table = KeyTable::new("customize-focused");
    table.bind(b'x' as key_code, c"display-message x", Some(c"note"));
    let target = Target::new(80, 24);
    let mut client = zeroed_client();
    unsafe { client.set_attached_session(Some(target.session_handle())) };
    let mut item = window_customize_itemdata {
        scope: WINDOW_CUSTOMIZE_KEY,
        table: Some(c"customize-focused".to_owned()),
        key: b'x' as key_code,
        ..Default::default()
    };
    unsafe {
        let (kt, bd) = window_customize_get_key(&item).expect("binding exists");
        assert_eq!(kt, table.handle());
        assert_eq!(key_binding_key(&bd), b'x' as key_code);
        assert_eq!(
            window_customize_set_command_callback(
                client.as_client_mut(),
                &mut item,
                Some(c"new"),
                1
            ),
            0
        );
        assert_eq!(
            window_customize_set_note_callback(client.as_client_mut(), &mut item, Some(c"new"), 1),
            0
        );
        assert_eq!(
            window_customize_set_option_callback(
                client.as_client_mut(),
                &mut item,
                Some(c"new"),
                1
            ),
            0
        );
        item.table = Some(c"missing-table".to_owned());
        assert!(window_customize_get_key(&item).is_none());
        item.table = None;
        assert!(window_customize_get_key(&item).is_none());
    }
}

#[test]
fn full_mode_rebuilds_with_filters_and_hide_global_toggles() {
    let _guard = globals();
    let mut table = KeyTable::new("retained-customize");
    table.bind(b'x' as key_code, c"display-message x", Some(c"note"));
    let mut target = Target::new(80, 24);
    let _chain = unsafe { Chain::new(&mut target) };
    let fs = target.state();
    let mut client = zeroed_client();
    unsafe { client.set_attached_session(Some(target.session_handle())) };
    unsafe {
        let wp = target.pane(0);
        assert_eq!(
            (&mut *wp).set_mode( None, WindowMode::Customize, Some(&fs), None),
            0
        );
        let wme = (&mut *wp).active_mode_mut().map(|mode| mode.into_entry()).expect("pane is in a mode");
        let data = wme.state.customize().unwrap();
        assert!(!data.borrow().item_list.is_empty());
        let original = {
            let tree = data.borrow().tree_ref();
            let tree_state = tree.borrow();
            let group = tree_state
                .children
                .iter()
                .find(|row| row.name.as_deref() == Some(c"Key Table - retained-customize"))
                .unwrap();
            let binding = &group.children[0];
            let original = binding.itemdata.clone().customize().unwrap();
            assert_eq!(binding.children.len(), 3);
            for field in &binding.children {
                let metadata = field.itemdata.clone().customize().unwrap();
                assert!(std::rc::Rc::ptr_eq(&original, &metadata));
            }
            original
        };
        let weak = std::rc::Rc::downgrade(&original);

        let owner = data
            .borrow()
            .owner
            .clone()
            .expect("mode owns its data")
            .upgrade()
            .expect("live mode data");
        let mut tag = 0;
        (owner.clone()).build(&sort_criteria_t::default(), &mut tag, Some(c"0"));
        assert!(data.borrow().item_list.is_empty());
        owner.build(&sort_criteria_t::default(), &mut tag, Some(c"1"));
        assert!(!data.borrow().item_list.is_empty());

        window_customize_resize(wme, 60, 18);
        (wme.state.customize().unwrap()).key(client.as_client_mut(), b'H' as key_code, None);
        assert_eq!(data.borrow().hide_global, 1);
        (wme.state.customize().unwrap()).key(client.as_client_mut(), b'H' as key_code, None);
        assert_eq!(data.borrow().hide_global, 0);
        {
            let state = data.borrow();
            let replacement = state
                .item_list
                .iter()
                .find(|row| row.table == original.table && row.key == original.key)
                .unwrap();
            assert!(!std::rc::Rc::ptr_eq(&original, replacement));
        }

        (&mut *wp).reset_modes();
        assert!((*wp).active_mode().is_none());
        assert_eq!(original.table(), Some(c"retained-customize"));
        assert!(window_customize_get_key(&original).is_some());
        drop(original);
        assert!(weak.upgrade().is_none());
    }
}

#[test]
fn confirmation_callbacks_ignore_empty_and_negative_answers() {
    let _guard = globals();
    let mut target = Target::new(80, 24);
    let mode_data = data(&mut target);
    let mut client = zeroed_client();
    unsafe {
        assert_eq!(mode_data.change_current(client.as_client_mut(), None, 0), 0);
        assert_eq!(
            mode_data.change_current(client.as_client_mut(), Some(c"no"), 0),
            0
        );
        assert_eq!(mode_data.change_tagged(client.as_client_mut(), None, 0), 0);
        assert_eq!(
            mode_data.change_tagged(client.as_client_mut(), Some(c"n"), 0),
            0
        );
    }
}

#[test]
fn option_actions_toggle_flags_cycle_choices_and_map_requested_scopes() {
    let _guard = globals();
    let mut target = Target::new(80, 24);
    let _chain = unsafe { Chain::new(&mut target) };
    let data = data(&mut target);
    let mut client = zeroed_client();
    unsafe { client.set_attached_session(Some(target.session_handle())) };
    unsafe {
        let window = (*target.window(0)).options_ref();
        let automatic = option_item(WINDOW_CUSTOMIZE_WINDOW, window, c"automatic-rename", -1);
        let before = (&*window).number(c"automatic-rename");
        data.set_option(client.as_client_mut(), Some(&automatic), 0, 0);
        assert_eq!((*window).number(c"automatic-rename"), (before == 0) as i64);
        data.set_option(client.as_client_mut(), Some(&automatic), 0, 1);
        assert_eq!((*window).number(c"automatic-rename"), before);

        let session = (&mut *target.session()).options_ref().clone();
        let position = option_item(WINDOW_CUSTOMIZE_SESSION, &session, c"status-position", -1);
        let old_position = session.number(c"status-position");
        data.set_option(client.as_client_mut(), Some(&position), 0, 0);
        assert_ne!(session.number(c"status-position"), old_position);
        data.set_option(client.as_client_mut(), Some(&position), 0, 0);
        assert_eq!(session.number(c"status-position"), old_position);

        let global_before = (global_w_options
            .get()
            .as_ref()
            .expect("global options are initialized"))
        .number(c"automatic-rename");
        data.set_option(client.as_client_mut(), Some(&automatic), 1, 0);
        assert_eq!(
            (global_w_options
                .get()
                .as_ref()
                .expect("global options are initialized"))
            .number(c"automatic-rename"),
            (global_before == 0) as i64
        );
        (global_w_options
            .get()
            .as_ref()
            .expect("global options are initialized"))
        .set_number(c"automatic-rename", global_before);

        data.set_option(client.as_client_mut(), None, 0, 0);
        let stale = option_item(
            WINDOW_CUSTOMIZE_WINDOW,
            global_w_options.get().as_ref().unwrap(),
            c"automatic-rename",
            -1,
        );
        data.set_option(client.as_client_mut(), Some(&stale), 0, 0);
    }
}

#[test]
fn rebuilding_tracks_user_option_precedence_filters_and_global_visibility() {
    let _guard = globals();
    let mut target = Target::new(80, 24);
    let _chain = unsafe { Chain::new(&mut target) };
    unsafe {
        (global_w_options
            .get()
            .as_ref()
            .expect("global options are initialized"))
        .set_string(c"@focused-global", 0, c"global", &[]);
        (*(*target.window(0)).options_ref()).set_string(c"@focused-local", 0, c"window", &[]);
        (*(*target.pane(0)).options_ref()).set_string(c"@focused-pane", 0, c"pane", &[]);
        let fs = target.state();
        let wp = target.pane(0);
        assert_eq!(
            (&mut *wp).set_mode( None, WindowMode::Customize, Some(&fs), None),
            0
        );
        let wme = (&*wp).active_mode().expect("pane is in a mode");
        let data = wme.state.customize().unwrap();
        assert!(
            data.borrow()
                .item_list
                .iter()
                .any(|item| item.name.as_deref() == Some(c"@focused-local"))
        );
        assert!(
            data.borrow()
                .item_list
                .iter()
                .any(|item| item.name.as_deref() == Some(c"@focused-pane"))
        );

        let owner = data.borrow().owner.clone().unwrap().upgrade().unwrap();
        let mut tag = 0;
        (owner.clone()).build(
            &sort_criteria_t::default(),
            &mut tag,
            Some(c"#{m:*focused*,#{option_name}}"),
        );
        assert!(!data.borrow().item_list.is_empty());
        data.borrow_mut().hide_global = 1;
        owner.build(&sort_criteria_t::default(), &mut tag, Some(c"1"));
        assert!(data.borrow().item_list.iter().all(|item| !matches!(
            item.scope,
            WINDOW_CUSTOMIZE_SERVER
                | WINDOW_CUSTOMIZE_GLOBAL_SESSION
                | WINDOW_CUSTOMIZE_GLOBAL_WINDOW
        )));
        (&mut *wp).reset_modes();
        RustOptionsEngine.remove_or_default(
            global_w_options
                .get()
                .as_ref()
                .expect("global options are initialized"),
            c"@focused-global",
            -1,
            &mut None,
        );
    }
}

#[test]
fn mode_tree_navigation_tags_information_and_boundaries_without_prompts() {
    let _guard = globals();
    let mut target = Target::new(80, 24);
    let _chain = unsafe { Chain::new(&mut target) };
    let fs = target.state();
    let mut client = zeroed_client();
    unsafe { client.set_attached_session(Some(target.session_handle())) };
    unsafe {
        let wp = target.pane(0);
        assert_eq!(
            (&mut *wp).set_mode( None, WindowMode::Customize, Some(&fs), None),
            0
        );
        let wme = (&mut *wp).active_mode_mut().map(|mode| mode.into_entry()).expect("pane is in a mode");
        let data = wme.state.customize().unwrap();
        for key in [
            KEYC_DOWN,
            KEYC_UP,
            KEYC_NPAGE,
            KEYC_PPAGE,
            KEYC_HOME,
            KEYC_END,
            b't' as key_code,
            b'T' as key_code,
            b'v' as key_code,
        ] {
            (wme.state.customize().unwrap()).key(client.as_client_mut(), key, None);
        }
        assert!(!data.borrow().item_list.is_empty());
        assert!(*(*wp).flags() & PANE_REDRAW != 0);
        (&mut *wp).reset_modes();
    }
}

#[test]
fn key_preview_preserves_empty_notes_and_sentence_punctuation() {
    let _guard = globals();
    let mut table = KeyTable::new("customize-note-preview");
    for (index, (note, expected)) in [
        (None, "There is no note for this key."),
        (Some(c""), ""),
        (Some(c"A note"), "A note."),
        (Some(c"A note."), "A note."),
    ]
    .into_iter()
    .enumerate()
    {
        let key = b'a' as key_code + index as key_code;
        table.bind(key, c"display-message x", note);
        let item = window_customize_itemdata {
            scope: WINDOW_CUSTOMIZE_KEY,
            table: Some(c"customize-note-preview".to_owned()),
            key,
            ..Default::default()
        };
        let mut output = crate::tests::test_fixtures::Screen::new(60, 12, 0);
        unsafe {
            let mut writer = crate::screen::screen_write_ctx_on_screen(&mut output);
            window_customize_draw_key(Some(&item), &mut writer, 60, 12);
        }
        let grid = RustScreen::grid(&output);
        let row = crate::grid::grid_string_cells(grid, 0, grid.hsize, grid.sx, None, 0, None);
        assert_eq!(row.to_string_lossy().trim_end(), expected);
    }
}

#[test]
fn key_selection_and_expansion_survive_inserting_earlier_tables_and_bindings() {
    let _guard = globals();
    let mut table = KeyTable::new("zz-stable-tags");
    table.bind(b'z' as key_code, c"display-message z", Some(c"chosen note"));
    let mut target = Target::new(80, 24);
    let _chain = unsafe { Chain::new(&mut target) };
    unsafe {
        let fs = target.state();
        let mut pane = fs
            .window()
            .unwrap()
            .pane_by_id(fs.wp.as_ref().map(|pane| pane.id()).unwrap())
            .unwrap();
        assert_eq!(
            (pane.get_mut().unwrap()).set_mode(
                None,
                WindowMode::Customize,
                Some(&fs),
                None
            ),
            0
        );
        let data = (pane.get().unwrap()).active_mode()
            .unwrap()
            .state
            .customize()
            .unwrap();
        let tree = data.borrow().tree_ref();
        let (group_tag, binding_tag, note_tag) = {
            let tree_state = tree.borrow();
            let group = tree_state
                .children
                .iter()
                .find(|row| row.name.as_deref() == Some(c"Key Table - zz-stable-tags"))
                .unwrap();
            let binding = &group.children[0];
            (group.tag, binding.tag, binding.children[1].tag)
        };
        tree.expand(group_tag);
        tree.expand(binding_tag);
        assert_eq!(tree.set_current(note_tag), 1);
        {
            let mut tree_state = tree.borrow_mut();
            let group = tree_state
                .children
                .iter_mut()
                .find(|row| row.tag == group_tag)
                .unwrap();
            group.children[0].tagged = 1;
        }
        let mut earlier_table = KeyTable::new("aa-stable-tags");
        earlier_table.bind(b'a' as key_code, c"display-message a", None);
        for key in b'a'..=b'z' {
            table.bind(key as key_code, c"display-message inserted", None);
        }
        tree.build();
        let selected = tree.current_item().customize().unwrap();
        assert_eq!(selected.table(), Some(c"zz-stable-tags"));
        assert_eq!(selected.key, b'z' as key_code);
        assert_eq!(tree.current_name().as_deref(), Some(c"Note"));
        {
            let tree_state = tree.borrow();
            let group = tree_state
                .children
                .iter()
                .find(|row| row.tag == group_tag)
                .unwrap();
            assert_eq!(group.expanded, 1);
            let binding = group
                .children
                .iter()
                .find(|row| row.tag == binding_tag)
                .unwrap();
            assert_eq!(binding.expanded, 1);
            assert_eq!(binding.tagged, 1);
            assert_eq!(binding.children[1].tag, note_tag);
        }
        let key = CustomizeTag::KeyBinding(c"zz-stable-tags".to_owned(), b'z' as key_code);
        key_bindings_remove(c"zz-stable-tags", b'z' as key_code);
        tree.build();
        assert!(!data.borrow().tags.contains_key(&key));
        table.bind(b'z' as key_code, c"display-message replacement", None);
        tree.build();
        assert_ne!(data.borrow().tags[&key], binding_tag);
        (pane.get_mut().unwrap()).reset_modes();
    }
}

#[test]
fn user_option_tags_follow_scope_and_prune_removed_options_without_reusing_tags() {
    let _guard = globals();
    let mut target = Target::new(80, 24);
    let data = data(&mut target);
    let fs = target.state();
    unsafe {
        let window = fs.window().unwrap().options();
        let session = fs.session().unwrap().options();
        let mut tags = Vec::new();
        for (scope, options) in [
            (WINDOW_CUSTOMIZE_WINDOW, &window),
            (WINDOW_CUSTOMIZE_SESSION, &session),
        ] {
            options.set_string(c"@stable-tag", 0, c"one", &[]);
            let tag = options.with_entry(c"@stable-tag", true, |entry| {
                data.borrow_mut().option_tag(scope, &fs, entry.unwrap(), -1)
            });
            options.set_string(c"@stable-tag", 0, c"two", &[]);
            assert_eq!(
                tag,
                options.with_entry(c"@stable-tag", true, |entry| data.borrow_mut().option_tag(
                    scope,
                    &fs,
                    entry.unwrap(),
                    -1
                ))
            );
            tags.push(tag);
        }
        assert_ne!(tags[0], tags[1]);
        RustOptionsEngine.remove_or_default(&window, c"@stable-tag", -1, &mut None);
        data.borrow_mut().tags.retain(|key, _| key.is_live());
        assert_eq!(data.borrow().tags.len(), 1);
        window.set_string(c"@stable-tag", 0, c"new", &[]);
        let replacement = window.with_entry(c"@stable-tag", true, |entry| {
            data.borrow_mut()
                .option_tag(WINDOW_CUSTOMIZE_WINDOW, &fs, entry.unwrap(), -1)
        });
        assert_ne!(replacement, tags[0]);
        assert_ne!(replacement, tags[1]);
    }
}

#[test]
fn retained_mode_state_stops_observing_a_removed_pane() {
    let _guard = globals();
    let mut target = Target::new(80, 24);
    let data = data(&mut target);
    let observed = data.pane().unwrap();
    drop(target);
    unsafe {
        assert!(observed.get().is_none());
        assert!(data.pane().is_none());
        let item = window_customize_itemdata::default();
        assert_eq!(data.check_item(&item, None), 0);
        let mut client = zeroed_client();
        (data.clone()).menu(client.as_client_mut(), b'H' as key_code);
        (data.clone()).build(&sort_criteria_t::default(), &mut 0, None);
        data.redraw_pane();
        assert!(data.borrow().item_list.is_empty());
    }
}
