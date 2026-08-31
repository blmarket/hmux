//! Unit tests for [`crate::cmd::cmd_show_options`] — the `show-options`,
//! `show-window-options` and `show-hooks` entries, their metadata and
//! constants, and the branches of [`cmd_show_options_exec`] the fixtures can
//! reach without a terminal or a live daemon.
//!
//! Exec runs through each entry's own function pointer over items whose
//! arguments come from the real command parser and whose target state is
//! resolved against a registered [`Target`]. Output is observed through a
//! control client, whose writes land verbatim in a buffer event; refusals on
//! client-less items are filed as config causes and read back the same way by
//! handing the cause list to `cfg_print_causes`. One shape stays out of reach:
//! the scope lookup cannot fail while a registered session, window and pane
//! exist, so the quiet no-scope branch is exercised with an untargeted item
//! instead.

use crate::client::CLIENT_CONTROL;
use crate::cmd::cmd_show_options::*;
use crate::cmd::{CMD_PARSE_ERROR, CMD_PARSE_SUCCESS, cmd_parse_from_string};
use crate::cmd::{cmd_find, cmd_table};
use crate::control::control_state;
use crate::fmt_args;
use crate::options::options_get_only_ptr;
use crate::options::{
    options_array_set, options_remove_or_default, options_set_number, options_set_parent,
    options_set_string,
};
use crate::session::session_options;
use crate::tests::test_fixtures::{Args, Clients, Item, StreamBuffer, Target, globals};
use ::core::ffi::{CStr, c_char};
use ::core::ptr::null_mut;
use ::std::ffi::CString;

/// A control client's write side: the state `control_write` reaches through
/// the client and the buffer event it writes into. Detaches itself from the
/// client when it goes.
struct ControlOut {
    c: *mut client,
    bev: StreamBuffer,
}

impl ControlOut {
    fn new(c: *mut client) -> ControlOut {
        let out = ControlOut {
            c,
            bev: StreamBuffer::new(),
        };
        unsafe {
            let state = (*c)
                .control_state
                .insert(Box::new(control_state::default()));
            state.write_event = out.bev.ptr();
            (*c).flags |= CLIENT_CONTROL as u64;
        }
        out
    }

    /// The complete lines written since the last time this was asked.
    fn take(&self) -> Vec<String> {
        let raw = self.bev.written();
        String::from_utf8_lossy(&raw)
            .lines()
            .map(|l| l.to_owned())
            .collect()
    }
}

impl Drop for ControlOut {
    fn drop(&mut self) {
        unsafe { (*self.c).control_state = None };
    }
}

/// Whether one of `lines` is exactly `want`.
fn has(lines: &[String], want: &str) -> bool {
    lines.iter().any(|l| l == want)
}

/// Whether none of `lines` starts with `prefix`.
fn lacks(lines: &[String], prefix: &str) -> bool {
    !lines.iter().any(|l| l.starts_with(prefix))
}

/// Runs the parsed command an item carries through its entry's exec hook, the
/// way the command queue calls it.
unsafe fn exec(item: &mut Item) -> cmd_retval {
    unsafe { ((*item.cmd()).entry.exec)(&*item.cmd(), item.ptr()) }
}

/// An item carrying `line`'s parsed arguments behind a control client, aimed
/// at a registered target: its target state names the current winlink and the
/// client doubles as the target client.
fn aimed(line: &'static CStr, who: *mut client, t: &mut Target) -> Item {
    let mut item = Item::with_client().with_args(line);
    item.set_client(who);
    item.targeting(t)
}

/// Makes `base-index` resolvable only through the global session options: the
/// local entry is removed and the set is pointed at the globals, which is how
/// a session that never overrode a default looks.
fn hide_base_index_behind_parent(t: &mut Target) {
    unsafe {
        let sess = session_options(t.session());
        let o = options_get_only_ptr(sess, c"base-index".as_ptr());
        assert!(!o.is_null());
        options_set_parent(sess, crate::tmux::global_s_options);
        let mut cause: Option<CString> = None;
        assert_eq!(options_remove_or_default(o, -1, &mut cause), 0);
        assert!(cause.is_none());
    }
}

#[test]
fn entry_metadata_matches_upstream() {
    unsafe {
        let e: *const cmd_entry = &raw const cmd_show_options_entry;
        assert_eq!((*e).name.to_string_lossy(), "show-options");
        assert_eq!(
            (*e).alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "show"
        );
        assert_eq!(
            (*e).usage.to_string_lossy(),
            "[-AgHpqsvw] [-t target-pane] [option]"
        );
        assert_eq!((*e).args.template.to_string_lossy(), "AgHpqst:vw");

        let w: *const cmd_entry = &raw const cmd_show_window_options_entry;
        assert_eq!((*w).name.to_string_lossy(), "show-window-options");
        assert_eq!(
            (*w).alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "showw"
        );
        assert_eq!(
            (*w).usage.to_string_lossy(),
            "[-gv] [-t target-window] [option]"
        );
        assert_eq!((*w).args.template.to_string_lossy(), "gvt:");

        let h: *const cmd_entry = &raw const cmd_show_hooks_entry;
        assert_eq!((*h).name.to_string_lossy(), "show-hooks");
        assert!((*h).alias.is_none(), "show-hooks has no alias");
        assert_eq!(
            (*h).usage.to_string_lossy(),
            "[-gpw] [-t target-pane] [hook]"
        );
        assert_eq!((*h).args.template.to_string_lossy(), "gpt:w");

        for entry in [e, w, h] {
            assert_eq!((*entry).args.lower, 0);
            assert_eq!((*entry).args.upper, 1);
            assert!((*entry).args.cb.is_none());

            assert_eq!((*entry).source.flag, 0);
            assert_eq!((*entry).source.type_0, CMD_FIND_PANE);
            assert_eq!((*entry).source.flags, 0);

            assert_eq!((*entry).target.flag, b't' as c_char);
            assert_eq!((*entry).target.flags, CMD_FIND_CANFAIL);

            assert_eq!((*entry).flags, CMD_AFTERHOOK);
        }

        assert_eq!((*e).target.type_0, CMD_FIND_PANE);
        assert_eq!((*w).target.type_0, CMD_FIND_WINDOW);
        assert_eq!((*h).target.type_0, CMD_FIND_PANE);

        assert_eq!((*e).exec as usize, (*w).exec as usize);
        assert_eq!((*e).exec as usize, (*h).exec as usize);
    }
}

#[test]
fn entries_are_registered_once_and_findable_by_name_and_alias() {
    let _guard = globals();
    unsafe {
        let entries = [
            &raw const cmd_show_options_entry,
            &raw const cmd_show_window_options_entry,
            &raw const cmd_show_hooks_entry,
        ];
        let counts = entries.map(|want| {
            cmd_table
                .iter()
                .filter(|slot| ::core::ptr::eq(**slot, want))
                .count()
        });
        assert!(
            counts.iter().all(|&count| count == 1),
            "{counts:?} over {} table slots",
            cmd_table.len()
        );

        let mut cause = None;
        assert_eq!(
            cmd_find(c"show-options".as_ptr(), &mut cause),
            &raw const cmd_show_options_entry
        );
        assert_eq!(
            cmd_find(c"show".as_ptr(), &mut cause),
            &raw const cmd_show_options_entry
        );
        assert_eq!(
            cmd_find(c"show-window-options".as_ptr(), &mut cause),
            &raw const cmd_show_window_options_entry
        );
        assert_eq!(
            cmd_find(c"showw".as_ptr(), &mut cause),
            &raw const cmd_show_window_options_entry
        );
        assert_eq!(
            cmd_find(c"show-hooks".as_ptr(), &mut cause),
            &raw const cmd_show_hooks_entry
        );
        assert!(cause.is_none());
    }
}

#[test]
fn argument_bounds_allow_at_most_one_option() {
    let _guard = globals();
    unsafe {
        for (line, name) in [
            (c"show-options", "show-options"),
            (c"show-options base-index", "show-options"),
            (c"show-window-options", "show-window-options"),
            (c"show-hooks -p", "show-hooks"),
        ] {
            let args = Args::parse(line);
            assert_eq!((*args.cmd()).entry.name.to_string_lossy(), name, "{line:?}");
        }

        for line in [c"show-options a b", c"showw a b"] {
            let mut pr = cmd_parse_from_string(line.as_ptr(), null_mut());
            assert_eq!(pr.status, CMD_PARSE_ERROR, "{line:?}");
            let err = pr.take_error();
            assert!(err.contains("too many arguments"), "{err}");
        }

        let mut pr = cmd_parse_from_string(c"show-hooks".as_ptr(), null_mut());
        assert_eq!(pr.status, CMD_PARSE_SUCCESS);
        let _ = pr.cmdlist.take();
    }
}

#[test]
fn listing_without_flags_shows_the_session_scope_with_user_options() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("session-scope", 80, 24);
    let out = ControlOut::new(c);
    unsafe {
        options_set_string(
            session_options(t.session()),
            c"@listed".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"hello".as_ptr()],
        );

        let mut item = aimed(c"show-options", c, &mut t);
        assert_eq!(exec(&mut item), CMD_RETURN_NORMAL);
        let lines = out.take();
        assert!(has(&lines, "base-index 0"), "{lines:?}");
        assert!(has(&lines, "status on"), "{lines:?}");
        assert!(has(&lines, "@listed hello"), "{lines:?}");
        assert!(lacks(&lines, "buffer-limit"));
        assert!(lacks(&lines, "allow-passthrough"));
        assert!(lacks(&lines, "after-new-window"));

        let mut bare = Item::new().with_args(c"show-options").targeting(&mut t);
        assert_eq!(exec(&mut bare), CMD_RETURN_NORMAL);
    }
}

#[test]
fn s_flag_selects_server_scope() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("server-scope", 80, 24);
    let out = ControlOut::new(c);
    unsafe {
        let mut item = aimed(c"show-options -s", c, &mut t);
        assert_eq!(exec(&mut item), CMD_RETURN_NORMAL);
        let lines = out.take();
        assert!(has(&lines, "buffer-limit 50"), "{lines:?}");
        assert!(lacks(&lines, "base-index"));
        assert!(lacks(&lines, "allow-passthrough"));
    }
}

#[test]
fn g_flag_reads_the_global_session_options_not_the_session_own() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("global-scope", 80, 24);
    let out = ControlOut::new(c);
    unsafe {
        options_set_number(session_options(t.session()), c"base-index".as_ptr(), 5);

        let mut item = aimed(c"show-options base-index", c, &mut t);
        assert_eq!(exec(&mut item), CMD_RETURN_NORMAL);
        assert_eq!(out.take(), ["base-index 5"]);

        let mut item = aimed(c"show-options -g base-index", c, &mut t);
        assert_eq!(exec(&mut item), CMD_RETURN_NORMAL);
        assert_eq!(out.take(), ["base-index 0"]);
    }
}

#[test]
fn w_flag_selects_the_window_scope_of_the_target() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("window-scope", 80, 24);
    let out = ControlOut::new(c);
    unsafe {
        let mut item = aimed(c"show-options -w", c, &mut t);
        assert_eq!(exec(&mut item), CMD_RETURN_NORMAL);
        let lines = out.take();
        assert!(has(&lines, "allow-passthrough off"), "{lines:?}");
        assert!(
            lines
                .iter()
                .any(|l| l.starts_with("window-status-separator")),
            "{lines:?}"
        );
        assert!(lacks(&lines, "base-index"));
    }
}

#[test]
fn p_flag_selects_the_pane_scope_of_the_target() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("pane-scope", 80, 24);
    let out = ControlOut::new(c);
    unsafe {
        let mut item = aimed(c"show-options -p", c, &mut t);
        assert_eq!(exec(&mut item), CMD_RETURN_NORMAL);
        let lines = out.take();
        assert!(has(&lines, "allow-passthrough off"), "{lines:?}");
        assert!(lacks(&lines, "base-index"));
        assert!(lacks(&lines, "buffer-limit"));
    }
}

#[test]
fn the_window_options_entry_defaults_to_the_window_scope() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("showw", 80, 24);
    let out = ControlOut::new(c);
    unsafe {
        let mut item = aimed(c"show-window-options", c, &mut t);
        assert_eq!(exec(&mut item), CMD_RETURN_NORMAL);
        let lines = out.take();
        assert!(has(&lines, "allow-passthrough off"), "{lines:?}");
        assert!(
            lines
                .iter()
                .any(|l| l.starts_with("window-status-separator")),
            "{lines:?}"
        );
        assert!(lacks(&lines, "base-index"));
        assert!(lacks(&lines, "buffer-limit"));
    }
}

#[test]
fn the_hooks_entry_lists_only_hooks_and_p_moves_to_the_pane() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("hooked", 80, 24);
    let out = ControlOut::new(c);
    unsafe {
        let mut item = aimed(c"show-hooks", c, &mut t);
        assert_eq!(exec(&mut item), CMD_RETURN_NORMAL);
        let lines = out.take();
        assert!(has(&lines, "after-new-window"), "{lines:?}");
        assert!(lacks(&lines, "base-index"));

        let mut item = aimed(c"show-hooks -p", c, &mut t);
        assert_eq!(exec(&mut item), CMD_RETURN_NORMAL);
        let lines = out.take();
        assert!(has(&lines, "pane-died"), "{lines:?}");
        assert!(lacks(&lines, "after-new-window"));
        assert!(lacks(&lines, "base-index"));
    }
}

#[test]
fn H_flag_adds_hooks_to_the_listing() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("with-hooks", 80, 24);
    let out = ControlOut::new(c);
    unsafe {
        let mut item = aimed(c"show-options -H", c, &mut t);
        assert_eq!(exec(&mut item), CMD_RETURN_NORMAL);
        let lines = out.take();
        assert!(has(&lines, "after-new-window"), "{lines:?}");
        assert!(has(&lines, "base-index 0"), "{lines:?}");
    }
}

#[test]
fn full_names_and_unique_prefixes_print_name_and_value() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("by-name", 80, 24);
    let out = ControlOut::new(c);
    unsafe {
        let mut item = aimed(c"show-options status", c, &mut t);
        assert_eq!(exec(&mut item), CMD_RETURN_NORMAL);
        assert_eq!(out.take(), ["status on"]);

        let mut item = aimed(c"show-options base-i", c, &mut t);
        assert_eq!(exec(&mut item), CMD_RETURN_NORMAL);
        assert_eq!(out.take(), ["base-index 0"]);

        let mut item = aimed(c"show-window-options allow-p", c, &mut t);
        assert_eq!(exec(&mut item), CMD_RETURN_NORMAL);
        assert_eq!(out.take(), ["allow-passthrough off"]);
    }
}

#[test]
fn v_flag_prints_only_values() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("values-only", 80, 24);
    let out = ControlOut::new(c);
    unsafe {
        options_set_string(
            session_options(t.session()),
            c"@listed".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"hello".as_ptr()],
        );

        for (line, want) in [
            (c"show-options -v base-index", "0"),
            (c"show-options -v status", "on"),
            (c"show-options -v @listed", "hello"),
        ] {
            let mut item = aimed(line, c, &mut t);
            assert_eq!(exec(&mut item), CMD_RETURN_NORMAL, "{line:?}");
            assert_eq!(out.take(), [want], "{line:?}");
        }
    }
}

#[test]
fn array_arguments_walk_every_index_and_an_explicit_index_selects_one() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("arrays", 80, 24);
    let out = ControlOut::new(c);
    unsafe {
        let sf = options_get_only_ptr(session_options(t.session()), c"status-format".as_ptr());
        assert!(!sf.is_null());
        assert_eq!(options_array_set(sf, 0, c"one".as_ptr(), 0, &mut None), 0);
        assert_eq!(options_array_set(sf, 1, c"two".as_ptr(), 0, &mut None), 0);
        assert_eq!(options_array_set(sf, 2, c"three".as_ptr(), 0, &mut None), 0);

        let mut item = aimed(c"show-options status-format", c, &mut t);
        assert_eq!(exec(&mut item), CMD_RETURN_NORMAL);
        assert_eq!(
            out.take(),
            [
                "status-format[0] one",
                "status-format[1] two",
                "status-format[2] three"
            ]
        );

        let mut item = aimed(c"show-options 'status-format[1]'", c, &mut t);
        assert_eq!(exec(&mut item), CMD_RETURN_NORMAL);
        assert_eq!(out.take(), ["status-format[1] two"]);

        let mut item = aimed(c"show-options -v 'status-format[0]'", c, &mut t);
        assert_eq!(exec(&mut item), CMD_RETURN_NORMAL);
        assert_eq!(out.take(), ["one"]);
    }
}

#[test]
fn an_empty_array_prints_its_bare_name_and_v_silences_it() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("empty-array", 80, 24);
    let out = ControlOut::new(c);
    unsafe {
        let mut item = aimed(c"show-options after-new-window", c, &mut t);
        assert_eq!(exec(&mut item), CMD_RETURN_NORMAL);
        assert_eq!(out.take(), ["after-new-window"]);

        let mut item = aimed(c"show-options -v after-new-window", c, &mut t);
        assert_eq!(exec(&mut item), CMD_RETURN_NORMAL);
        assert!(out.take().is_empty());
    }
}

#[test]
fn A_flag_marks_parent_values_in_the_listing_and_by_argument() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    hide_base_index_behind_parent(&mut t);
    let mut clients = Clients::new();
    let c = clients.add("parent-values", 80, 24);
    let out = ControlOut::new(c);
    unsafe {
        options_set_string(
            crate::tmux::global_s_options,
            c"@only-global".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"globalvalue".as_ptr()],
        );

        let mut item = aimed(c"show-options -A", c, &mut t);
        assert_eq!(exec(&mut item), CMD_RETURN_NORMAL);
        let lines = out.take();
        assert!(has(&lines, "base-index* 0"), "{lines:?}");

        let mut item = aimed(c"show-options -A @only-global", c, &mut t);
        assert_eq!(exec(&mut item), CMD_RETURN_NORMAL);
        assert_eq!(out.take(), ["@only-global* globalvalue"]);

        let mut item = aimed(c"show-options @only-global", c, &mut t);
        assert_eq!(exec(&mut item), CMD_RETURN_ERROR);
        assert_eq!(out.take(), ["invalid option: @only-global"]);
    }
}

#[test]
fn a_table_option_in_no_tree_answers_normal_silently() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    hide_base_index_behind_parent(&mut t);
    let mut clients = Clients::new();
    let c = clients.add("nowhere", 80, 24);
    let out = ControlOut::new(c);
    unsafe {
        let mut item = aimed(c"show-options base-index", c, &mut t);
        assert_eq!(exec(&mut item), CMD_RETURN_NORMAL);
        assert!(out.take().is_empty());
    }
}
