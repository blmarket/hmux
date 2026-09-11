use crate::screen::Screen as _;
use super::*;
use crate::options::OptionsRef;
use crate::server::client_ref_of;
use crate::tests::test_fixtures::{Clients, Target, ensure_reactor, globals, prompt_answers_clear};

struct PromptFixture {
    _target: Target,
    _clients: Clients,
    c: *mut client,
}

impl PromptFixture {
    fn new() -> Self {
        let target = Target::new(80, 24);
        let mut clients = Clients::new();
        let c = clients.add("status-focused", 80, 24);
        unsafe {
            ensure_reactor();
            (*c).set_attached_session(Some(target.session_handle()));
            status_init(&mut *c);
        }
        Self {
            _target: target,
            _clients: clients,
            c,
        }
    }

    unsafe fn open(&mut self, input: &CStr) {
        unsafe {
            prompt_answers_clear();
            status_prompt_set(
                &mut *self.c,
                None,
                c":",
                Some(input),
                Prompt::Recorder,
                PromptData::None,
                0,
                PromptHistoryType::Command,
            );
        }
    }
}

#[test]
fn status_rendering_covers_initial_unchanged_resized_forced_and_disabled_paths() {
    let _guard = globals();
    let fixture = PromptFixture::new();
    unsafe {
        let c = &mut *fixture.c;
        (c.attached_session().unwrap()).update_status_cache();
        assert_eq!(status_redraw(c), 1);
        assert_eq!(status_redraw(c), 0);
        c.flags |= CLIENT_STATUSFORCE as u64;
        assert_eq!(status_redraw(c), 0);
        c.flags &= !(CLIENT_STATUSFORCE as u64);
        c.tty.sx = 43;
        assert_eq!(status_redraw(c), 1);
        c.tty.sy = 0;
        assert_eq!(status_redraw(c), 1);
        c.tty.sy = 24;
        (c.attached_session().unwrap().options()).set_number(c"status", 0);
        (c.attached_session().unwrap()).update_status_cache();
        assert_eq!(status_redraw(c), 1);
    }
}

#[test]
fn message_overlay_renders_styles_widths_reuse_and_clear_paths() {
    let _guard = globals();
    let fixture = PromptFixture::new();
    unsafe {
        let c = &mut *fixture.c;
        (c.attached_session().unwrap()).update_status_cache();
        status_redraw(c);
        for style in [
            c"align=left,width=20",
            c"align=centre,width=50%",
            c"align=right,width=200",
        ] {
            (c.attached_session().unwrap().options()).set_string(
                c"message-style",
                0,
                c"%s",
                fmt_args![style.as_ptr()],
            );
            status_message_set(
                Some(c),
                0,
                0,
                0,
                0,
                c"hello #[bold]world ## %d",
                fmt_args![7],
            );
            assert_eq!(status_message_redraw(c), 1);
            assert_eq!(status_message_redraw(c), 0);
            status_message_clear(c);
            status_message_clear(c);
        }
        status_message_set(Some(c), 25, 1, 1, 1, c"timed", &[]);
        assert!(c.message_timer.is_armed());
        status_message_callback(c);
        assert!(c.message_string.is_none());
    }
}

#[test]
fn prompt_redraw_covers_entry_command_scrolling_quote_and_zero_terminal() {
    let _guard = globals();
    let mut fixture = PromptFixture::new();
    unsafe {
        let c = &mut *fixture.c;
        (c.attached_session().unwrap()).update_status_cache();
        status_redraw(c);
        fixture.open(c"abcdefghijklmnopqrstuvwxyz0123456789");
        c.prompt_string = Some(c"prompt: ".to_owned());
        c.tty.sx = 18;
        c.prompt_index = c.prompt_buffer.len();
        assert_eq!(status_prompt_redraw(c), 1);
        assert_eq!(status_prompt_redraw(c), 0);
        c.prompt_flags |= PROMPT_QUOTENEXT;
        assert!(status_prompt_redraw(c) <= 1);
        c.prompt_mode = PROMPT_COMMAND;
        c.prompt_index = 0;
        assert_eq!(status_prompt_redraw(c), 1);
        c.prompt_buffer = utf8_fromcstr(c"a\x01\x7fz");
        c.prompt_index = 2;
        assert_eq!(status_prompt_redraw(c), 1);
        c.tty.sx = 0;
        assert_eq!(status_prompt_redraw(c), 0);
        c.tty.sx = 18;
        c.tty.sy = 0;
        assert_eq!(status_prompt_redraw(c), 0);
        status_prompt_clear(c);
    }
}

#[test]
fn prompt_update_incremental_single_numeric_and_noformat_paths_are_initialized() {
    let _guard = globals();
    let fixture = PromptFixture::new();
    unsafe {
        let c = &mut *fixture.c;
        status_prompt_set(
            c,
            None,
            c"#{session_name}: ",
            Some(c"#{session_name}"),
            Prompt::Recorder,
            PromptData::None,
            PROMPT_INCREMENTAL,
            PromptHistoryType::Search,
        );
        assert!(c.prompt_last.is_some());
        status_prompt_update(c, c"updated: ", Some(c"new"));
        assert_eq!(c.prompt_index, 3);
        status_prompt_clear(c);
        status_prompt_set(
            c,
            None,
            c"literal",
            Some(c"#{session_name}"),
            Prompt::Recorder,
            PromptData::None,
            PROMPT_NOFORMAT | PROMPT_NOFREEZE | PROMPT_NUMERIC | PROMPT_SINGLE,
            PromptHistoryType::Command,
        );
        assert_eq!(utf8_vec_tocstr(&c.prompt_buffer), c"#{session_name}");
        status_prompt_clear(c);
    }
}

#[test]
fn timer_callback_handles_detached_attached_hidden_message_and_prompt_states() {
    let _guard = globals();
    let fixture = PromptFixture::new();
    unsafe {
        let c = &mut *fixture.c;
        let session = c.attached_session().unwrap();
        c.set_attached_session(None);
        status_timer_callback(c);
        c.set_attached_session(Some(&session));
        (session.options()).set_number(c"status", 1);
        (session.options()).set_number(c"status-interval", 1);
        c.flags &= !(CLIENT_REDRAWSTATUS as u64);
        status_timer_start(c);
        assert!(c.status.timer.is_armed());
        assert_ne!(c.flags & CLIENT_REDRAWSTATUS as u64, 0);
        c.message_string = Some(c"message".to_owned());
        c.flags &= !(CLIENT_REDRAWSTATUS as u64);
        status_timer_callback(c);
        assert_eq!(c.flags & CLIENT_REDRAWSTATUS as u64, 0);
        c.message_string = None;
        c.prompt_string = Some(c"prompt".to_owned());
        status_timer_callback(c);
        c.prompt_string = None;
        (session.options()).set_number(c"status-interval", 0);
        status_timer_callback(c);
        assert!(!c.status.timer.is_armed());
    }
}

#[test]
fn completion_helpers_cover_duplicates_prefixes_and_edges() {
    let _guard = globals();
    unsafe {
        let mut list = Vec::new();
        status_prompt_add_list(&mut list, c"display-message");
        status_prompt_add_list(&mut list, c"display-message");
        status_prompt_add_list(&mut list, c"display-menu");
        assert_eq!(list.len(), 2);
        assert_eq!(status_prompt_complete_prefix(&list).unwrap(), c"display-me");
        assert!(status_prompt_complete_prefix(&[]).is_none());
        let commands = status_prompt_complete_list(c"new-", 1);
        assert!(commands.iter().any(|s| s == c"new-session"));
        let options = status_prompt_complete_list(c"status-", 0);
        assert!(!options.is_empty());
        let layouts = status_prompt_complete_list(c"main-", 0);
        assert!(layouts.iter().any(|s| s == c"main-horizontal"));

    }
}

#[test]
fn every_emacs_prompt_dispatch_arm_accepts_representative_state() {
    let _guard = globals();
    let mut fixture = PromptFixture::new();
    let keys = [
        8589934621,
        35184372088930,
        8589934622,
        35184372088934,
        8589934614,
        35184372088929,
        8589934615,
        35184372088933,
        9,
        8589934599,
        35184372088936,
        8589934613,
        35184372088932,
        35184372088949,
        35184372088939,
        35184372088951,
        35192962023454,
        17592186044518,
        9007199254741061,
        9007199254741093,
        9007199254741079,
        9007199254741111,
        9007199254741058,
        35192962023453,
        17592186044514,
        8589934619,
        35184372088944,
        8589934620,
        35184372088942,
        35184372088953,
        35184372088948,
        13,
        10,
        27,
        35184372088923,
        35184372088931,
        35184372088935,
        35184372088946,
        35184372088947,
        35184372088950,
    ];
    unsafe {
        for key in keys {
            fixture.open(c"one two");
            (*fixture.c).prompt_index = 4;
            status_prompt_key(&mut *fixture.c, key);
            if (*fixture.c).prompt_string.is_some() {
                status_prompt_clear(&mut *fixture.c);
            }
        }
    }
}

#[test]
fn keypad_translation_covers_digits_operators_enter_and_modifiers() {
    let keypad = [
        (8589934623, b'/' as key_code),
        (8589934624, b'*' as key_code),
        (8589934625, b'-' as key_code),
        (8589934626, b'7' as key_code),
        (8589934627, b'8' as key_code),
        (8589934628, b'9' as key_code),
        (8589934629, b'+' as key_code),
        (8589934630, b'4' as key_code),
        (8589934631, b'5' as key_code),
        (8589934632, b'6' as key_code),
        (8589934633, b'1' as key_code),
        (8589934634, b'2' as key_code),
        (8589934635, b'3' as key_code),
        (8589934636, b'\r' as key_code),
        (8589934637, b'0' as key_code),
        (8589934638, b'.' as key_code),
    ];
    for (key, expected) in keypad {
        assert_eq!(status_prompt_keypad_key(key), expected);
    }
    assert_eq!(
        status_prompt_keypad_key(8589934626 | KEYC_CTRL),
        8589934626 | KEYC_CTRL
    );
    assert_eq!(status_prompt_keypad_key(b'z' as key_code), b'z' as key_code);
}

#[test]
fn vi_translation_covers_entry_and_command_maps() {
    let _guard = globals();
    let mut fixture = PromptFixture::new();
    unsafe {
        fixture.open(c"abc");
        ((*fixture.c).attached_session().unwrap().options()).set_number(
            c"status-keys",
            MODEKEY_VI as i64,
        );
        let mut translated = 0;
        (*fixture.c).prompt_mode = PROMPT_ENTRY;
        for key in [
            35184372088929,
            35184372088931,
            35184372088933,
            35184372088935,
            35184372088936,
            9,
            35184372088939,
            35184372088942,
            35184372088944,
            35184372088948,
            35184372088949,
            35184372088950,
            35184372088951,
            35184372088953,
            10,
            13,
            35192962023453,
            35192962023454,
            8589934599,
            8589934613,
            8589934620,
            8589934615,
            8589934614,
            8589934621,
            8589934622,
            8589934619,
        ] {
            assert_eq!(
                status_prompt_translate_key(&mut *fixture.c, key, &mut translated),
                1
            );
        }
        assert_eq!(
            status_prompt_translate_key(&mut *fixture.c, b'z' as u64, &mut translated),
            2
        );
        (*fixture.c).prompt_mode = PROMPT_COMMAND;
        for key in [
            8589934599,
            b'A' as u64,
            b'I' as u64,
            b'C' as u64,
            b's' as u64,
            b'a' as u64,
            b'S' as u64,
            b'i' as u64,
            b'$' as u64,
            b'0' as u64,
            b'^' as u64,
            b'D' as u64,
            b'X' as u64,
            b'b' as u64,
            b'B' as u64,
            b'd' as u64,
            b'e' as u64,
            b'E' as u64,
            b'w' as u64,
            b'W' as u64,
            b'p' as u64,
            b'q' as u64,
            b'x' as u64,
            8589934613,
            8589934620,
            b'j' as u64,
            8589934621,
            b'h' as u64,
            8589934622,
            b'l' as u64,
            8589934619,
            b'k' as u64,
            10,
            13,
        ] {
            let result = status_prompt_translate_key(&mut *fixture.c, key, &mut translated);
            assert!(result <= 1);
            (*fixture.c).prompt_mode = PROMPT_COMMAND;
        }
        status_prompt_clear(&mut *fixture.c);
    }
}

#[test]
fn prompt_escape_and_word_helpers_cover_boundaries() {
    assert_eq!(status_prompt_escape(c"plain"), c"plain");
    assert_eq!(status_prompt_escape(c"#a##"), c"##a####");
    let _guard = globals();
    let mut fixture = PromptFixture::new();
    unsafe {
        fixture.open(c"  one::two three  ");
        let c = &mut *fixture.c;
        let size = c.prompt_buffer.len();
        c.prompt_index = 0;
        status_prompt_forward_word(c, size, 0, c":");
        assert_eq!(c.prompt_index, 5);
        status_prompt_forward_word(c, size, 1, c":");
        assert_eq!(c.prompt_index, 7);
        status_prompt_end_word(c, size, c":");
        assert!(c.prompt_index >= 6);
        c.prompt_index = size;
        status_prompt_backward_word(c, c":");
        assert!(c.prompt_index < size);
        status_prompt_delete_range(c, c.prompt_index, size);
        assert!(c.prompt_saved.is_some());
        status_prompt_clear(c);
    }
}

#[test]
fn status_geometry_covers_top_bottom_disabled_control_detached_and_multiline_clamps() {
    let _guard = globals();
    let fixture = PromptFixture::new();
    unsafe {
        let c = &mut *fixture.c;
        let session = c.attached_session().unwrap();
        let options = session.options();
        options.set_number(c"status", 3);
        options.set_number(c"status-position", 0);
        options.set_number(c"message-line", 99);
        session.update_status_cache();
        assert_eq!(status_line_size(c), 3);
        assert_eq!(status_at_line(c), 0);
        assert_eq!(status_prompt_line_at(c), 2);
        assert_eq!(status_redraw(c), 1);

        options.set_number(c"status-position", 1);
        session.update_status_cache();
        assert_eq!(status_at_line(c), 21);
        c.flags |= CLIENT_STATUSOFF as u64;
        assert_eq!(status_line_size(c), 0);
        assert_eq!(status_at_line(c), -1);
        c.flags &= !(CLIENT_STATUSOFF as u64);
        c.flags |= CLIENT_CONTROL as u64;
        assert_eq!(status_line_size(c), 0);
        c.flags &= !(CLIENT_CONTROL as u64);

        c.set_attached_session(None);
        assert_eq ! (status_line_size (c) , (global_s_options . get () . as_ref () . expect ("global options are initialized")) . number (c"status") as u_int);
        assert_eq!(status_at_line(c), -1);
        assert_eq!(status_prompt_line_at(c), 0);
        c.flags |= CLIENT_STATUSOFF as u64;
        assert_eq!(status_at_line(c), -1);
        assert_eq!(status_line_size(c), 0);
        assert_eq!(status_prompt_line_at(c), 0);
        c.flags &= !(CLIENT_STATUSOFF as u64);
        c.set_attached_session(Some(&session));
        options.set_number(c"status", 0);
        session.update_status_cache();
        assert_eq!(status_prompt_line_at(c), 0);
    }
}

#[test]
fn overlay_screen_ownership_is_shared_between_message_prompt_and_restored_on_last_clear() {
    let _guard = globals();
    let fixture = PromptFixture::new();
    unsafe {
        let c = &mut *fixture.c;
        (c.attached_session().unwrap()).update_status_cache();
        status_redraw(c);
        status_message_set(Some(c), 0, 0, 0, 1, c"message", &[]);
        assert!(!c.status.is_own());
        status_prompt_set(
            c,
            None,
            c"prompt",
            Some(c"input"),
            Prompt::Recorder,
            PromptData::None,
            PROMPT_NOFREEZE,
            PromptHistoryType::Command,
        );
        assert!(!c.status.is_own());
        status_message_clear(c);
        assert!(!c.status.is_own());
        status_prompt_clear(c);
        assert!(c.status.is_own());

        let first = status_push_screen(c);
        c.message_overlay = Some(first.clone());
        let second = status_push_screen(c);
        assert!(first.ptr_eq(&second));
        c.message_overlay = None;
        drop(second);
        status_pop_screen(&mut c.status, Some(first));
        assert!(c.status.is_own());
    }
}

#[test]
fn message_and_prompt_area_cover_alignment_percentage_absolute_and_line_preservation() {
    let _guard = globals();
    let fixture = PromptFixture::new();
    unsafe {
        let c = &mut *fixture.c;
        (c.attached_session().unwrap()).update_status_cache();
        status_redraw(c);
        for (style, expected) in [
            (c"align=left,width=12" as &CStr, (0, 12)),
            (c"align=centre,width=50%", (20, 40)),
            (c"align=right,width=10", (70, 10)),
            (c"align=left,width=0", (0, 80)),
            (c"align=left,width=999", (0, 80)),
        ] {
            (c.attached_session().unwrap().options()).set_string(
                c"message-style",
                0,
                c"%s",
                fmt_args![style.as_ptr()],
            );
            assert_eq!(status_prompt_area(c), expected);
            status_message_set(Some(c), 0, 1, 1, 1, c"#[red]literal # text", &[]);
            assert_eq!(status_message_redraw(c), 1);
            assert_eq!(status_message_redraw(c), 0);
            status_message_clear(c);
        }
        assert!(status_get_range(c, 0, 5).is_none());
        assert!(status_get_range(c, u_int::MAX, 0).is_none());
    }
}

#[test]
fn status_free_handles_initialized_reinitialized_and_already_freed_clients() {
    let _guard = globals();
    let fixture = PromptFixture::new();
    unsafe {
        let c = &mut *fixture.c;
        (c.attached_session().unwrap()).update_status_cache();
        status_redraw(c);
        c.status.entries[0].expanded = Some(c"cached".to_owned());
        status_free(c);
        assert!(!c.status.screen.is_initialized());
        assert!(c.status.entries[0].expanded.is_none());
        status_free(c);
        status_init(c);
        assert!(c.status.screen.is_initialized());
        assert!(c.status.is_own());
    }
}

#[test]
fn prompt_paste_and_explicit_completion_replace_at_start_middle_and_end() {
    let _guard = globals();
    let mut fixture = PromptFixture::new();
    unsafe {
        fixture.open(c"alpha gamma");
        let c = &mut *fixture.c;
        c.prompt_saved = Some(utf8_fromcstr(c"BETA"));
        c.prompt_index = 6;
        assert_eq!(status_prompt_paste(c), 1);
        assert_eq!(utf8_vec_tocstr(&c.prompt_buffer), c"alpha BETAgamma");
        assert_eq!(c.prompt_index, 10);

        c.prompt_index = 2;
        assert_eq!(status_prompt_replace_complete(c, Some(c"first")), 1);
        assert!(
            utf8_vec_tocstr(&c.prompt_buffer)
                .to_bytes()
                .starts_with(b"first")
        );
        c.prompt_index = c.prompt_buffer.len();
        assert_eq!(status_prompt_replace_complete(c, Some(c"tail")), 1);
        assert!(
            utf8_vec_tocstr(&c.prompt_buffer)
                .to_bytes()
                .ends_with(b"tail")
        );

        c.prompt_buffer = utf8_fromcstr(c"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
        c.prompt_index = c.prompt_buffer.len();
        assert_eq!(status_prompt_replace_complete(c, None), 0);
        status_prompt_clear(c);
    }
}

#[test]
fn completion_routes_cover_commands_sessions_targets_windows_and_flags() {
    let _guard = globals();
    let mut fixture = PromptFixture::new();
    unsafe {
        fixture.open(c"");
        let c = &mut *fixture.c;
        c.prompt_type = PromptHistoryType::Command;
        assert!(
            status_prompt_complete(c, c"new-sess", 0)
                .unwrap()
                .as_bytes()
                .starts_with(b"new-session")
        );
        assert!(status_prompt_complete(c, c"definitely-no-command", 0).is_none());

        let mut sessions = Vec::new();
        let prefix = status_prompt_complete_session(&mut sessions, c"0", 0).unwrap();
        assert!(prefix.as_bytes().starts_with(b"0:"));
        let flagged = status_prompt_complete_session(&mut Vec::new(), c"0", b't' as i8).unwrap();
        assert!(flagged.as_bytes().starts_with(b"-t"));

        c.prompt_type = PromptHistoryType::Target;
        assert!(status_prompt_complete(c, c"0", 0).is_some());
        assert!(status_prompt_complete(c, c"missing:", 0).is_none());
        c.prompt_type = PromptHistoryType::WindowTarget;
        let session = c.attached_session().unwrap();
        assert_eq!(
            status_prompt_complete_window_menu(c, &session, c"0", 0, 0).unwrap(),
            c"0"
        );
        assert!(status_prompt_complete(c, c"0", 0).is_none());
        status_prompt_clear(c);
    }
}

#[test]
fn completion_menu_callback_replaces_words_targets_and_ignores_cancel() {
    let _guard = globals();
    let mut fixture = PromptFixture::new();
    unsafe {
        fixture.open(c"before old after");
        let c = &mut *fixture.c;
        c.prompt_index = 10;
        let weak = client_ref_of(c).unwrap().downgrade();
        status_prompt_menu_callback(
            0,
            b'0' as key_code,
            status_prompt_menu {
                c: weak.clone(),
                start: 1,
                list: vec![c"zero".to_owned(), c"chosen".to_owned()],
                flag: 0,
            },
        );
        assert!(
            utf8_vec_tocstr(&c.prompt_buffer)
                .to_bytes()
                .windows(6)
                .any(|v| v == b"chosen")
        );

        c.prompt_type = PromptHistoryType::WindowTarget;
        status_prompt_menu_callback(
            0,
            b'0' as key_code,
            status_prompt_menu {
                c: weak.clone(),
                start: 0,
                list: vec![c"3".to_owned()],
                flag: 0,
            },
        );
        assert_eq!(utf8_vec_tocstr(&c.prompt_buffer), c"3");
        status_prompt_menu_callback(
            0,
            KEYC_NONE,
            status_prompt_menu {
                c: weak,
                start: 0,
                list: vec![c"ignored".to_owned()],
                flag: b't' as i8,
            },
        );
        assert_eq!(utf8_vec_tocstr(&c.prompt_buffer), c"3");
        status_prompt_clear(c);
    }
}

#[test]
fn prompt_separators_preserve_the_c_terminator_and_single_column_rule() {
    let mut cell = utf8_data::default();
    cell.size = 1;
    cell.width = 1;
    assert_eq!(status_prompt_in_list(c"", &cell), 1);
    assert_eq!(status_prompt_space(&cell), 0);
    cell.data[0] = b' ';
    assert_eq!(status_prompt_in_list(c" -", &cell), 1);
    assert_eq!(status_prompt_space(&cell), 1);
    cell.width = 2;
    assert_eq!(status_prompt_in_list(c" -", &cell), 0);
    assert_eq!(status_prompt_space(&cell), 0);
}

#[test]
fn prompt_paste_keeps_complete_utf8_and_stops_at_controls_or_invalid_sequences() {
    let _guard = globals();
    let mut fixture = PromptFixture::new();
    for (input, accepted) in [
        (b"".as_slice(), c""),
        (b"ab\ncd".as_slice(), c"ab"),
        (b"ab\0cd".as_slice(), c"ab"),
        (b"ab\x7fcd".as_slice(), c"ab"),
        (b"ab\xffcd".as_slice(), c"ab"),
        (b"ab\xc3".as_slice(), c"ab"),
        (b"ab\xc3x".as_slice(), c"ab"),
        (b"ab\xc3\xa9Z".as_slice(), c"abéZ"),
    ] {
        unsafe {
            fixture.open(c"[]");
            crate::paste::with_paste_buffers_mut(|buffers| {
                buffers.add_automatic(None, input.to_vec(), 1);
            });
            let c = &mut *fixture.c;
            c.prompt_index = 1;
            status_prompt_paste(c);
            let expected = CString::new([b"[", accepted.to_bytes(), b"]"].concat()).unwrap();
            assert_eq!(utf8_vec_tocstr(&c.prompt_buffer), expected, "{input:?}");
            assert_eq!(c.prompt_index, 1 + utf8_fromcstr(accepted).len());
            status_prompt_clear(c);
        }
    }
}

#[test]
fn completion_reads_to_an_embedded_nul_but_replaces_the_whole_prompt_word() {
    let _guard = globals();
    let mut fixture = PromptFixture::new();
    unsafe {
        fixture.open(c"new-sess");
        let c = &mut *fixture.c;
        let mut nul = utf8_data::default();
        utf8_set(&mut nul, 0);
        c.prompt_buffer.push(nul);
        c.prompt_buffer.extend(utf8_fromcstr(c"ignored"));
        c.prompt_index = c.prompt_buffer.len();
        assert_eq!(status_prompt_replace_complete(c, None), 1);
        assert_eq!(utf8_vec_tocstr(&c.prompt_buffer), c"new-session ");
        assert_eq!(c.prompt_index, c.prompt_buffer.len());
        status_prompt_clear(c);
    }
}

#[test]
fn session_completion_prefers_matching_names_before_dollar_id_aliases() {
    let _guard = globals();
    let fixture = PromptFixture::new();
    unsafe {
        let owner = (*fixture.c).attached_session().unwrap();
        let id = CString::new(format!("${}", owner.id())).unwrap();
        let alias = CString::new(format!("{}:", id.to_string_lossy())).unwrap();
        assert_eq!(
            status_prompt_complete_session(&mut Vec::new(), &id, 0),
            Some(alias)
        );
        let name = CString::new(format!("{}name", id.to_string_lossy())).unwrap();
        owner.rename(name.clone());
        let expected = CString::new(format!("-t{}:", name.to_string_lossy())).unwrap();
        assert_eq!(
            status_prompt_complete_session(&mut Vec::new(), &id, b't' as i8),
            Some(expected)
        );
    }
}

#[test]
fn window_completions_filter_numeric_prefixes_in_index_order_and_obey_menu_height() {
    let _guard = globals();
    let mut fixture = PromptFixture::new();
    fixture._target.add_window(20, 80, 24);
    fixture._target.add_window(10, 80, 24);
    fixture._target.add_window(2, 80, 24);
    unsafe {
        fixture.open(c"");
        let c = &mut *fixture.c;
        let session = c.attached_session().unwrap();
        c.prompt_type = PromptHistoryType::WindowTarget;
        assert_eq!(
            status_prompt_complete_window_menu(c, &session, c"1", 0, 0),
            Some(c"10".to_owned())
        );
        c.tty.sy = status_line_size(c) + 3;
        assert_eq!(
            status_prompt_complete_window_menu(c, &session, c"2", 0, b't' as i8),
            Some(c"-t2".to_owned())
        );
        assert!(status_prompt_complete_window_menu(c, &session, c"99", 0, 0).is_none());
        status_prompt_clear(c);
    }
}

#[test]
fn target_completion_preserves_flags_current_session_and_pane_suffix_rules() {
    let _guard = globals();
    let mut fixture = PromptFixture::new();
    unsafe {
        fixture.open(c"");
        let c = &mut *fixture.c;
        c.prompt_type = PromptHistoryType::Command;
        for (word, expected) in [(c"-t0:", c"-t0:0"), (c"-s:", c"-s0:0")] {
            assert_eq!(status_prompt_complete(c, word, 0).as_deref(), Some(expected));
        }
        assert!(status_prompt_complete(c, c"-t0:0", 0).is_none());
        c.prompt_type = PromptHistoryType::Target;
        assert_eq!(status_prompt_complete(c, c":0", 0).as_deref(), Some(c"0:0"));
        assert!(status_prompt_complete(c, c"0:0.1", 0).is_none());
        assert!(status_prompt_complete(c, c"missing:", 0).is_none());
        status_prompt_clear(c);
    }
}

pub(crate) fn status_prompt_type(type_0: &CStr) -> prompt_type {
    PromptHistoryType::parse(type_0).map_or(PROMPT_TYPE_INVALID, |kind| kind as prompt_type)
}

pub(crate) fn status_prompt_type_string(type_0: u_int) -> &'static CStr {
    PromptHistoryType::ALL
        .get(type_0 as usize)
        .copied()
        .map_or(c"invalid", PromptHistoryType::name)
}
