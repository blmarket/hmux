use super::*;
use crate::WindowPane;
use crate::pane_exit::PaneExitState;
use crate::tests::test_fixtures::{Clients, Format, Pane, Target, Window, ascii, globals};

fn text(value: CString) -> String {
    value.to_string_lossy().into_owned()
}

fn expand(ft: &Format, value: &CStr) -> String {
    ft.expand(value)
}

unsafe fn add(ft: &Format, key: &CStr, value: &CStr) {
    unsafe { format_add(&mut *ft.ptr(), key, value, &[]) }
}

fn state() -> format_expand_state {
    format_expand_state {
        start_time: get_timer(),
        ..Default::default()
    }
}

fn clear_cached_jobs(_ft: &format_tree) -> Option<CString> {
    let format_jobs = format_jobs_FIELD.get();

    format_jobs.map().clear();
    Some(c"rendered".to_owned())
}

#[test]
fn cached_job_output_can_clear_the_cache_during_expansion() {
    let format_jobs = format_jobs_FIELD.get();

    let _guard = globals();
    let mut ft = Format::new();
    format_add_cb(ft.tree(), c"clear_cache", Some(clear_cached_jobs));
    let key = (ft.tree().tag, c"unused".to_owned());
    format_jobs.map().insert(
        key,
        Box::new(format_job {
            client: None,
            tag: ft.tree().tag,
            cmd: c"unused".to_owned(),
            expanded: Some(c"unused".to_owned()),
            last: 0,
            out: Some(c"#{clear_cache}".to_owned()),
            updated: 1,
            job: Some(u_int::MAX),
            status: 0,
        }),
    );
    let output = unsafe { format_job_get(ft.tree(), &mut state(), c"unused") };
    assert_eq!(output, c"rendered");
    assert!(format_jobs.map().is_empty());
}

#[test]
fn a_job_removed_during_command_expansion_is_not_reused() {
    let format_jobs = format_jobs_FIELD.get();

    let _guard = globals();
    let mut ft = Format::new();
    format_add_cb(ft.tree(), c"clear_cache", Some(clear_cached_jobs));
    let output = unsafe { format_job_get(ft.tree(), &mut state(), c"#{clear_cache}") };
    assert!(output.is_empty());
    assert!(format_jobs.map().is_empty());
}

#[test]
fn list_values_preserve_bytes_and_stop_at_the_first_nul() {
    assert_eq!(format_list_value(&mut ByteBuffer::new()), None);
    for (bytes, expected) in [
        (&b"one,two"[..], &b"one,two"[..]),
        (&b"\xff\xfe"[..], &b"\xff\xfe"[..]),
        (&b"one\0two"[..], &b"one"[..]),
        (&b"\0two"[..], &b""[..]),
    ] {
        let mut buffer = ByteBuffer::from(bytes.to_vec());
        assert_eq!(format_list_value(&mut buffer).unwrap().as_bytes(), expected);
        assert_eq!(buffer.as_slice(), bytes);
    }
}

#[test]
fn quote_unescape_strip_skip_and_choose_cover_nested_delimiters() {
    let _guard = globals();
    let ft = Format::new();
    unsafe {
        assert_eq!(text(format_quote_shell(c"a b$c'x")), "a\\ b\\$c\\'x");
        assert_eq!(text(format_quote_style(c"#[fg=red]#x")), "##[fg=red]##x");
        let mut es = state();
        assert_eq!(
            text(format_unescape(&mut *ft.ptr(), &mut es, c"a#,b##c#{x#,y}")),
            "a,b#c#{x#,y}"
        );
        assert_eq!(
            text(format_strip(&mut *ft.ptr(), &mut es, c"a#,b##c#{x#,y}")),
            "a,b#c#{x#,y}"
        );

        let source = c"one#{x,y},two";
        let found = format_skip1(Some((&mut *ft.ptr(), &mut es)), source, c",");
        assert_eq!(found, Some(9));
        assert_eq!(&source.to_bytes()[found.unwrap() + 1..], b"two");
        assert!(format_skip1(Some((&mut *ft.ptr(), &mut es)), c"no delimiter", c",").is_none());
        let (left, right) = format_choose(&mut *ft.ptr(), &mut es, c"#{value},tail", 0).unwrap();
        assert_eq!((left.as_c_str(), right.as_c_str()), (c"#{value}", c"tail"));
        assert!(format_choose(&mut *ft.ptr(), &mut es, c"only", 1).is_none());
    }
}

#[test]
fn modifier_builder_recognizes_flags_operators_and_delimited_arguments() {
    let _guard = globals();
    let ft = Format::new();
    unsafe {
        let mut es = state();
        let input = c"l;b;||;m/ri/:key";
        let (list, consumed) = format_build_modifiers(&mut *ft.ptr(), &mut es, input).unwrap();
        assert_eq!(list.len(), 4);
        assert_eq!(list[0].format_modifier_name(), c"l");
        assert_eq!(list[1].format_modifier_name(), c"b");
        assert_eq!(list[2].format_modifier_name(), c"||");
        assert_eq!(list[3].format_modifier_name(), c"m");
        assert_eq!(list[3].argv[0], c"ri");
        assert_eq!(&input.to_bytes()[consumed..], b"key");

        for bad in [c"unknown:key", c"l"] {
            assert!(format_build_modifiers(&mut *ft.ptr(), &mut es, bad).is_none());
        }
    }
}

#[test]
fn glob_regex_and_substitution_helpers_cover_matches_misses_and_flags() {
    {
        let plain = format_modifier {
            modifier: [b'm', 0, 0],
            size: 1,
            argv: vec![],
        };
        let ignore = format_modifier {
            modifier: [b'm', 0, 0],
            size: 1,
            argv: vec![c"i".to_owned()],
        };
        let regex = format_modifier {
            modifier: [b'm', 0, 0],
            size: 1,
            argv: vec![c"r".to_owned()],
        };
        assert_eq!(format_match(&plain, c"a*", c"abc"), c"1");
        assert_eq!(format_match(&plain, c"a*", c"xyz"), c"0");
        assert_eq!(format_match(&ignore, c"ABC", c"abc"), c"1");
        assert_eq!(format_match(&regex, c"^a.+c$", c"abc"), c"1");
        assert_eq!(format_match(&regex, c"[", c"abc"), c"0");

        let insensitive = format_modifier {
            modifier: [b's', 0, 0],
            size: 1,
            argv: vec![c"x".to_owned(), c"y".to_owned(), c"i".to_owned()],
        };
        assert_eq!(format_sub(&insensitive, c"ABCabc", c"abc", c"x"), c"xx");
        assert_eq!(format_sub(&plain, c"unchanged", c"[", c"x"), c"unchanged");
    }
}

#[test]
fn boolean_helpers_short_circuit_and_expand_nested_operands() {
    let _guard = globals();
    let ft = Format::new();
    unsafe {
        add(&ft, c"yes", c"1");
        add(&ft, c"no", c"0");
        let mut es = state();
        assert_eq!(
            format_bool_op_1(&mut *ft.ptr(), &mut es, c"#{yes}", 0),
            c"1"
        );
        assert_eq!(
            format_bool_op_1(&mut *ft.ptr(), &mut es, c"#{yes}", 1),
            c"0"
        );
        assert_eq!(
            format_bool_op_n(&mut *ft.ptr(), &mut es, c"#{yes},#{no}", 1),
            c"0"
        );
        assert_eq!(
            format_bool_op_n(&mut *ft.ptr(), &mut es, c"#{no},#{yes}", 0),
            c"1"
        );
        assert_eq!(
            format_bool_op_n(&mut *ft.ptr(), &mut es, c"#{yes},#{yes}", 1),
            c"1"
        );
        assert_eq!(
            format_bool_op_n(&mut *ft.ptr(), &mut es, c"#{no},#{no}", 0),
            c"0"
        );
    }
}

#[test]
fn expansion_byte_cursors_preserve_malformed_endings_and_style_boundaries() {
    let _guard = globals();
    let mut ft = Format::new();
    ft.tree().flags |= FORMAT_NOJOBS;
    for (source, expected) in [
        (c"head#", "head"),
        (c"head#{x", "head"),
        (c"head#(x", "head"),
        (c"head###", "head#"),
        (c"head####", "head##"),
        (c"head#(outer(inner)tail)end", "headend"),
        (c"head#(outer(inner)end", "head"),
        (c"head#()end", "headend"),
    ] {
        let mut es = state();
        assert_eq!(
            unsafe { format_expand1(ft.tree(), &mut es, source) }.to_bytes(),
            expected.as_bytes()
        );
        assert_eq!(es.r#loop, 0);
    }
    let host = ft.expand(c"#H");
    for (source, expected) in [
        (c"#[#H]#H", format!("#[#H]{host}")),
        (c"##[#H]#H", format!("##[#H]{host}")),
        (c"#[#H", format!("#[{host}")),
        (c"#[#{l:#H}]#H", format!("#[#H]{host}")),
    ] {
        assert_eq!(ft.expand(source), expected);
    }
}

#[test]
fn expansion_literals_conditionals_boolean_and_comparison_operators() {
    let _guard = globals();
    let ft = Format::new();
    unsafe {
        add(&ft, c"yes", c"1");
        add(&ft, c"no", c"0");
        add(&ft, c"word", c"Alpha");
    }
    for (source, expected) in [
        (c"plain ## #, #} unknown=#?", "plain # , } unknown=#?"),
        (c"#{?yes,left,right}", "left"),
        (c"#{?no,left,right}", "right"),
        (c"#{?#{==:a,a},yes,no}", "yes"),
        (c"#{==:a,a}", "1"),
        (c"#{!=:a,b}", "1"),
        (c"#{<:1,2}", "1"),
        (c"#{>:2,1}", "1"),
        (c"#{<=:2,2}", "1"),
        (c"#{>=:2,2}", "1"),
        (c"#{!!:yes}", "1"),
        (c"#{!:no}", "0"),
        (c"#{&&:#{yes},#{yes},#{no}}", "0"),
        (c"#{||:#{no},#{no},#{yes}}", "1"),
        (c"#{m/i:alpha,#{word}}", "1"),
        (c"#{m/r:^Al.*,#{word}}", "1"),
    ] {
        assert_eq!(expand(&ft, source), expected, "{source:?}");
    }
}

#[test]
fn expansion_transformers_cover_case_path_quote_width_padding_and_substitution() {
    let _guard = globals();
    let ft = Format::new();
    unsafe {
        add(&ft, c"path", c"/tmp/Some File.txt");
        add(&ft, c"hash", c"a#b");
        add(&ft, c"long", c"abcdef");
    }
    for (source, expected) in [
        (c"#{b:path}", "Some File.txt"),
        (c"#{d:path}", "/tmp"),
        (c"#{l:literal}", "literal"),
        (c"#{q:path}", "/tmp/Some\\ File.txt"),
        (c"#{q|h:hash}", "a##b"),
        (c"#{=3:long}", "abc"),
        (c"#{=-3:long}", "def"),
        (c"#{p10:long}", "abcdef    "),
        (c"#{s/abc/XYZ/:long}", "XYZdef"),
        (c"#{s/ABC/xyz/i:long}", "xyzdef"),
    ] {
        assert_eq!(expand(&ft, source), expected, "{source:?}");
    }
}

#[test]
fn comparisons_preserve_unsigned_bytes_and_prefix_order() {
    let _guard = globals();
    let ft = Format::new();
    for (source, expected) in [
        (c"#{<:\x7f,\x80}", "1"),
        (c"#{>:\xff,\x80}", "1"),
        (c"#{<:a,aa}", "1"),
        (c"#{==:\xff,\xff}", "1"),
        (c"#{!=:\xff,\xff}", "0"),
    ] {
        assert_eq!(expand(&ft, source), expected);
    }
}

#[test]
fn expression_operator_covers_integer_float_comparison_and_error_paths() {
    let _guard = globals();
    let ft = Format::new();
    for (source, expected) in [
        (c"#{e|+|:7,5}", "12"),
        (c"#{e|-|:7,5}", "2"),
        (c"#{e|*|:7,5}", "35"),
        (c"#{e|/|:7,2}", "3"),
        (c"#{e|%|:7,4}", "3"),
        (c"#{e|==|:2,2}", "1"),
        (c"#{e|!=|:2,3}", "1"),
        (c"#{e|>|:3,2}", "1"),
        (c"#{e|<|:2,3}", "1"),
        (c"#{e|>=|:3,3}", "1"),
        (c"#{e|<=|:3,3}", "1"),
        (c"#{e|+|f|3:1.25,2.5}", "3.750"),
        (c"#{e|+|f|2:0x1.8p2, 2}", "8.00"),
        (c"#{e|+|:,2}", "2"),
        (c"#{e|+|:2,}", "2"),
        (c"x#{e|+|:2 ,1}y", "xy"),
        (c"x#{e|+|:1,2x}y", "xy"),
        (c"x#{e|bogus|:1,2}y", "xy"),
        (c"x#{e|+|:bad,2}y", "xy"),
        (c"x#{e|+|:1}y", "xy"),
    ] {
        assert_eq!(expand(&ft, source), expected, "{source:?}");
    }
}

fn lazy_value(_ft: &format_tree) -> Option<CString> {
    Some(c"lazy".to_owned())
}

#[test]
fn entries_replace_values_fill_callbacks_store_times_and_iterate() {
    let _guard = globals();
    let mut ft = Format::new();
    unsafe {
        add(&ft, c"key", c"first");
        add(&ft, c"key", c"second");
        format_add_cb(ft.tree(), c"lazy_key", Some(lazy_value));
        format_add_tv(
            ft.tree(),
            c"time_key",
            &timeval {
                tv_sec: 123,
                tv_usec: 0,
            },
        );
        assert_eq!(
            expand(&ft, c"#{key}:#{lazy_key}:#{time_key}"),
            "second:lazy:123"
        );
        assert!(!expand(&ft, c"#{t:time_key}").is_empty());

        let mut values = Vec::new();
        format_each(ft.tree(), |key, value| {
            values.push((key.to_owned(), value.to_owned()));
        });
        assert!(values.iter().any(|(k, v)| k == c"key" && v == c"second"));
        assert!(values.iter().any(|(k, v)| k == c"lazy_key" && v == c"lazy"));
        assert!(values.iter().any(|(k, v)| k == c"time_key" && v == c"123"));
    }
}

#[test]
fn time_expansion_and_loop_guards_cover_empty_percent_and_limits() {
    let _guard = globals();
    let ft = Format::new();
    unsafe {
        assert_eq!(text(format_expand(&mut *ft.ptr(), c"")), "");
        assert_eq!(text(format_expand_time(&mut *ft.ptr(), c"%%")), "%");
        assert_eq!(text(format_expand_time(&mut *ft.ptr(), c"%Y")).len(), 4);
        let mut es = state();
        es.r#loop = FORMAT_LOOP_LIMIT as u_int;
        assert_eq!(format_expand1(&mut *ft.ptr(), &mut es, c"text"), c"");
        es.r#loop = 0;
        es.start_time = get_timer().wrapping_sub(FORMAT_TIME_LIMIT as u64 + 1);
        assert_eq!(format_check_time(&mut *ft.ptr(), &mut es), 0);
    }
}

#[test]
fn format_search_rechecks_its_pane_after_expanding_the_search_text() {
    unsafe fn remove_search_pane(ft: &format_tree) -> Option<CString> {
        unsafe {
            let pane = ft.pane_handle()?;
            let window = pane.window().unwrap();
            window.remove_pane(
                &crate::window::window_pane_find_by_id(pane.id()).expect("the pane exists"),
            );
            Some(c"needle".to_owned())
        }
    }

    let _guard = globals();
    let mut window = Window::new(100, "search", 20, 3);
    let mut pane = Pane::new(100, 20, 3, 0);
    window.add_pane(&mut pane);
    let mut ft = Format::new();
    unsafe {
        let wp = &mut *pane.ptr();
        for (x, byte) in b"needle".iter().copied().enumerate() {
            crate::grid::grid_set_cell(
                RustScreen::grid_mut(wp.base_mut()),
                x as u_int,
                0,
                &ascii(byte),
            );
        }
        ft.tree().set_window(Some(&window.reference()));
        ft.tree().set_pane(Some(wp));
        assert_eq!(ft.expand(c"#{C:needle}"), "1");
        format_add_cb(ft.tree(), c"remove_search_pane", Some(remove_search_pane));
        assert_eq!(ft.expand(c"#{C:#{remove_search_pane}}"), "0");
        assert!(ft.tree().pane_handle().is_none());
    }
}

#[test]
fn window_name_search_observes_changes_made_by_its_expansion_callback() {
    fn rename_window(ft: &format_tree) -> Option<CString> {
        let window = ft.window()?;
        window.set_window_name(Some(c"renamed"));
        Some(c"renamed".to_owned())
    }

    let _guard = globals();
    let mut target = Target::new(20, 3);
    let mut ft = Format::new();
    unsafe {
        ft.tree().set_session(Some(&*target.session()));
        let link = &*target.winlink(0);
        ft.tree().set_window(Some(&link.window_handle().unwrap()));
        format_add_cb(ft.tree(), c"rename_window", Some(rename_window));
        assert_eq!(
            format_window_name(ft.tree(), &mut state(), c"renamed").as_deref(),
            Some(c"0")
        );
        assert_eq!(
            format_window_name(ft.tree(), &mut state(), c"#{rename_window}").as_deref(),
            Some(c"1")
        );
        assert_eq!(
            format_window_name(ft.tree(), &mut state(), c"renamed").as_deref(),
            Some(c"1")
        );
    }
}

#[test]
fn session_alert_formats_preserve_index_order_and_first_seen_marks() {
    let _guard = globals();
    let mut target = Target::new(20, 3);
    target.add_window(5, 20, 3);
    target.add_window(2, 20, 3);
    let mut ft = Format::new();
    unsafe {
        let session = &mut *target.session();
        session.windows.get_mut(&0).unwrap().flags = WINLINK_BELL | WINLINK_SILENCE;
        session.windows.get_mut(&2).unwrap().flags = WINLINK_ACTIVITY | WINLINK_BELL;
        session.windows.get_mut(&5).unwrap().flags = WINLINK_SILENCE;
        session.lastw = vec![5, 2];
        ft.tree().set_session(Some(session));
        assert_eq!(format_cb_session_alert(ft.tree()).as_deref(), Some(c"!~#"));
        assert_eq!(
            format_cb_session_alerts(ft.tree()).as_deref(),
            Some(c"0!~,2#!,5~")
        );
        assert_eq!(
            format_cb_session_stack(ft.tree()).as_deref(),
            Some(c"0,5,2")
        );
    }
}

#[test]
fn retained_format_panes_preserve_their_reference_across_window_changes() {
    let _guard = globals();
    let mut target = Target::new(80, 24);
    let mut ft = Format::new();
    assert!(ft.tree().pane_handle().is_none());
    unsafe {
        ft.tree().set_pane(Some(&*target.pane(0)));
        let pane = ft.tree().pane_handle().unwrap();
        let window = pane.window().unwrap();
        ft.tree().set_window(Some(&window));
        assert_eq!(pane.get().unwrap().pane_id(), pane.id());
        window.as_window_mut().last_panes.insert(0, pane.clone());
        assert_eq!(format_cb_pane_active(ft.tree()).as_deref(), Some(c"1"));
        assert_eq!(format_cb_pane_last(ft.tree()).as_deref(), Some(c"1"));
        let removed = crate::window::window_panes_take(
            &mut window.as_window_mut(),
            &crate::window::window_pane_find_by_id(pane.id()).expect("the pane exists"),
        )
        .unwrap();
        assert!(pane.ptr_eq(&removed.downgrade()));
        assert!(pane.get().is_some());
        assert!(ft.tree().pane_handle().unwrap().ptr_eq(&pane));
        assert_eq!(format_cb_pane_active(ft.tree()), None);
        assert_eq!(format_cb_pane_last(ft.tree()), None);
        crate::window::window_panes_insert_head(&mut window.as_window_mut(), removed);
        assert_eq!(ft.tree().pane_handle().unwrap().id(), pane.id());
        target.add_window(2, 60, 18);
        let other = crate::window::window_pane_find_by_id((*target.pane(1)).pane_id()).unwrap();
        ft.tree().set_window(Some(&other.window().unwrap()));
        assert!(ft.tree().pane_handle().unwrap().ptr_eq(&pane));
        ft.tree().set_window(None);
        assert_eq!(ft.tree().pane_handle().unwrap().id(), pane.id());
    }
}

#[test]
fn retained_format_links_use_checked_lookup_after_removal() {
    let _guard = globals();
    let mut target = Target::new(80, 24);
    let mut ft = Format::new();
    unsafe { ft.tree().set_winlink(Some(&*target.winlink(0))) };
    let link = ft.tree().winlink().unwrap();
    assert_eq!(link.get().unwrap().idx, 0);
    let removed = unsafe { (*target.session()).windows.remove(&0).unwrap() };
    assert!(link.get().is_none());
    assert!(ft.tree().winlink().is_none());
    unsafe { (*target.session()).windows.insert(0, removed) };
    assert_eq!(link.get().unwrap().idx, 0);
}

#[test]
fn registered_target_defaults_expand_dense_session_window_and_pane_callback_matrix() {
    let _guard = globals();
    let mut target = Target::new(80, 24);
    target.add_window(2, 60, 18);
    unsafe {
        let s = target.session();
        let wl = target.winlink(0);
        let wp = target.pane(0);
        let mut ft = format_create(None, None, FORMAT_NONE, 0);
        format_defaults(&mut ft, None, Some(&*s), Some(&*wl), Some(&*wp));
        let output = format_expand(
            &mut ft,
            c"#{session_id}|#{session_name}|#{session_windows}|#{session_attached}|#{session_grouped}|#{session_many_attached}|#{window_id}|#{window_index}|#{window_name}|#{window_width}|#{window_height}|#{window_panes}|#{window_linked}|#{window_zoomed_flag}|#{pane_id}|#{pane_index}|#{pane_width}|#{pane_height}|#{pane_left}|#{pane_right}|#{pane_top}|#{pane_bottom}|#{pane_active}|#{pane_dead}|#{pane_in_mode}|#{history_size}|#{history_limit}|#{cursor_x}|#{cursor_y}|#{cursor_flag}|#{alternate_on}|#{bracket_paste_flag}|#{origin_flag}|#{wrap_flag}|#{insert_flag}|#{keypad_flag}|#{synchronized_output_flag}",
        );
        let fields: Vec<_> = output.to_bytes().split(|b| *b == b'|').collect();
        assert_eq!(fields.len(), 37);
        assert_eq!(fields[0], b"$0");
        assert_eq!(fields[1], b"0");
        assert_eq!(fields[6], b"@0");
        assert_eq!(fields[14], b"%0");

        *(*wp).flags_mut() |= crate::window::PANE_EXITED | PANE_UNSEENCHANGES;
        (*wp).record_process_exit(7);
        (*wp).base_mut().set_cursor(4, 3);
        let changed = format_expand(
            &mut ft,
            c"#{pane_dead}:#{pane_dead_status}:#{pane_unseen_changes}:#{cursor_x}:#{cursor_y}",
        );
        assert_eq!(changed.to_bytes().split(|b| *b == b':').count(), 5);
    }
}

#[test]
fn active_window_formats_skip_sessions_without_a_current_link() {
    let _guard = globals();
    let mut target = Target::new(20, 3);
    let mut clients = Clients::new();
    let mut ft = Format::new();
    unsafe {
        let client = &mut *clients.add("active-format-client", 20, 3);
        client.set_attached_session(Some(target.session_handle()));
        ft.tree()
            .set_session(Some(target.session_handle().as_session()));
        ft.tree().set_winlink(Some(&*target.winlink(0)));
        assert_eq!(
            format_cb_window_active_clients(ft.tree()).as_deref(),
            Some(c"1")
        );
        assert_eq!(
            format_cb_window_active_clients_list(ft.tree()).as_deref(),
            Some(c"active-format-client")
        );
        assert_eq!(
            format_cb_window_active_sessions(ft.tree()).as_deref(),
            Some(c"1")
        );
        let mut session = ft.tree().session().unwrap();
        let current = session.as_session_mut().curw.take();
        assert_eq!(
            format_cb_window_active_clients(ft.tree()).as_deref(),
            Some(c"0")
        );
        assert_eq!(format_cb_window_active_clients_list(ft.tree()), None);
        assert_eq!(
            format_cb_window_active_sessions(ft.tree()).as_deref(),
            Some(c"0")
        );
        assert_eq!(format_cb_window_active_sessions_list(ft.tree()), None);
        session.as_session_mut().curw = current;
    }
}

#[test]
fn registered_client_defaults_expand_client_callbacks_and_absent_optional_values() {
    let _guard = globals();
    let mut target = Target::new(80, 24);
    let mut clients = Clients::new();
    unsafe {
        let c = &mut *clients.add("format-client", 90, 30);
        c.set_attached_session(Some(target.session_handle()));
        c.pid = 4321;
        c.flags |= CLIENT_UTF8 as u64 | CLIENT_READONLY as u64 | CLIENT_CONTROL as u64;
        c.term_name = Some(c"xterm-256color".to_owned());
        c.ttyname = Some(c"/dev/pts/fmt".to_owned());
        let mut ft = format_create(Some(c), None, FORMAT_NONE, 0);
        let session = c.attached_session().unwrap();
        format_defaults(
            &mut ft,
            Some(c),
            Some(session.as_session()),
            Some(&*target.winlink(0)),
            Some(&*target.pane(0)),
        );
        let output = format_expand(
            &mut ft,
            c"#{client_name}|#{client_pid}|#{client_width}|#{client_height}|#{client_session}|#{client_tty}|#{client_termname}|#{client_utf8}|#{client_readonly}|#{client_control_mode}|#{client_flags}|#{client_cell_width}|#{client_cell_height}|#{client_theme}|#{client_discarded}|#{client_written}|#{client_last_session}",
        );
        let fields: Vec<_> = output.to_bytes().split(|b| *b == b'|').collect();
        assert_eq!(fields.len(), 17);
        assert_eq!(fields[0], b"format-client");
        assert_eq!(fields[1], b"4321");
    }
}

#[test]
fn loops_nested_conditionals_padding_truncation_and_arithmetic_compose_in_one_tree() {
    let _guard = globals();
    let mut target = Target::new(80, 24);
    target.add_window(2, 60, 18);
    unsafe {
        let mut ft = format_create(None, None, FORMAT_NONE, 0);
        format_defaults(
            &mut ft,
            None,
            Some(&*target.session()),
            Some(&*target.winlink(0)),
            Some(&*target.pane(0)),
        );
        let sessions = format_expand(&mut ft, c"#{S:#{session_name},#{session_id}}");
        assert!(!sessions.to_bytes().is_empty());
        let windows = format_expand(&mut ft, c"#{W:#{window_index}=#{window_name},#{window_id}}");
        assert!(windows.to_bytes().contains(&b'='));
        assert_eq!(
            format_expand(
                &mut ft,
                c"#{window_index}|#{W:#{window_index}:#{P:#{window_index}.#{pane_index};}}|#{window_index}",
            ),
            c"0|0:0.0;2:2.0;|0"
        );
        let panes = format_expand(
            &mut ft,
            c"#{P:#{pane_index}:#{pane_width}x#{pane_height},none}",
        );
        assert!(!panes.to_bytes().contains(&0));
        let nested = format_expand(&mut ft, c"#{?#{&&:#{==:#{e|+|:2,3},5},#{m/r:^tar.*,#{window_name}}},#{p12:#{=6:abcdefghi}},bad}");
        assert_eq!(nested, c"            ");
        assert!(!format_expand(&mut ft, c"#{e|/|:1,0}").to_bytes().is_empty());
        assert!(!format_expand(&mut ft, c"#{e|%|:1,0}").to_bytes().is_empty());
        assert_eq!(format_expand(&mut ft, c"#{s/a/A/g:banana}"), c"");
    }
}

#[test]
fn cleared_session_environment_suppresses_global_format_fallback() {
    use crate::environ::{EnvironmentStore, with_global_environment_mut};

    let _guard = globals();
    let mut target = Target::new(80, 24);
    let ft = Format::from_target(&mut target);
    with_global_environment_mut(|env| env.set(c"HMUX_FORMAT_ENV", 0, c"global"));
    assert_eq!(expand(&ft, c"#{HMUX_FORMAT_ENV}"), "global");
    unsafe {
        (&mut *target.session())
            .environ_mut()
            .set(c"HMUX_FORMAT_ENV", 0, c"session");
    }
    assert_eq!(expand(&ft, c"#{HMUX_FORMAT_ENV}"), "session");
    unsafe {
        (&mut *target.session())
            .environ_mut()
            .clear(c"HMUX_FORMAT_ENV");
    }
    assert_eq!(expand(&ft, c"#{HMUX_FORMAT_ENV}"), "");
    unsafe {
        (&mut *target.session())
            .environ_mut()
            .unset(c"HMUX_FORMAT_ENV");
    }
    assert_eq!(expand(&ft, c"#{HMUX_FORMAT_ENV}"), "global");
}

#[test]
fn defaults_collect_mode_fields_while_its_screen_is_shared() {
    let _guard = globals();
    let mut target = Target::new(20, 6);
    unsafe {
        let state = target.state();
        let mut pane = state.pane_ref().unwrap();
        let session = state.session().unwrap();
        let link = state.winlink_ref().unwrap();
        assert_eq!(
            (pane.get_mut().unwrap()).set_mode(
                None,
                WindowMode::View,
                None,
                None
            ),
            0
        );
        let wp = pane.get().unwrap();
        let mode = wp.active_mode().unwrap();
        let data = mode.state.copy_mode_data_ref().unwrap();
        let screen = data.screen.borrow();
        let cursor = screen.cursor();
        for selected in [Some(wp), None] {
            let mut ft = format_create_defaults(
                None,
                None,
                Some(session.as_session()),
                link.get(),
                selected,
            );
            assert_eq!(
                format_expand(
                    &mut ft,
                    c"#{scroll_position}|#{copy_cursor_x}|#{selection_present}"
                )
                .as_c_str(),
                c"0|0|0"
            );
            assert_eq!(
                ft.type_0,
                if selected.is_some() {
                    FORMAT_TYPE_PANE
                } else {
                    FORMAT_TYPE_WINDOW
                }
            );
            assert_eq!(screen.cursor(), cursor);
        }
    }
}

#[test]
fn an_unattached_client_uses_the_inactive_session_loop_format() {
    let _guard = globals();
    let _target = Target::new(80, 24);
    let client = crate::tests::test_fixtures::zeroed_client();
    unsafe {
        let mut ft = format_create(Some(client.as_client()), None, FORMAT_NONE, 0);
        format_defaults(
            &mut ft,
            Some(client.as_client()),
            None,
            None,
            None::<&dyn crate::WindowPane>,
        );
        assert_eq!(format_expand(&mut ft, c"#{S:inactive,active}"), c"inactive");
        assert_eq!(format_expand(&mut ft, c"#{client_session}"), c"");
    }
}

#[test]
fn format_window_observation_neither_borrows_nor_retains_window() {
    let _guard = globals();
    let fixture = Window::new(42, "observed", 80, 24);
    let owner = fixture.handle();
    let mut ft = Format::new();
    let payload = owner.as_window_mut();
    ft.tree().set_window(Some(&owner));
    assert!(ft.tree().window().unwrap().ptr_eq(&owner));
    drop(payload);
    drop(fixture);
    assert!(ft.tree().window().is_none());
}
