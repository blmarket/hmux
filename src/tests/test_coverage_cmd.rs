//! Unit tests for [`crate::cmd`] — the command hub every other `cmd_*`
//! module hangs off: the table of command entries, the lookup that turns a
//! name on a command line into one of them, the parse that builds a `cmd` from
//! a row of argument values, the argv helpers the client protocol packs and
//! unpacks with, the command-list plumbing behind `;` and `;;`, the mouse
//! resolvers and the `%1`/`%%` template expansion `run-shell` and
//! `confirm-before` build their command lines with.
//!
//! These tests close the gaps left by the argv and accessor tests in
//! [`crate::tests::test_coverage_beta`], and they characterize the module as it
//! stands before its conversion — quirks included. Four are worth naming.
//! [`cmd_find`] reports an ambiguous prefix by walking the table a *second*
//! time and listing every name that starts with what was typed — names only,
//! never aliases — while an alias matched exactly on the first walk clears an
//! ambiguity already gathered and wins outright. An empty name is a prefix of
//! everything, so it lists the whole table. [`cmd_get_alias`]
//! reads `command-alias` out of the global options each time and simply skips
//! an entry carrying no `=`. And the template expansion treats `%%` as "put
//! the argument here, once": a second `%%` in the same template is copied out
//! literally, while `%1` may repeat as often as it likes.
//!
//! Everything here runs without a server. The command table, the parser and
//! the option trees are process-wide, so every test that reaches them holds
//! the [`globals`] guard; the mouse tests take their session, window and pane
//! out of [`Target`], which registers server-free fixtures in the trees
//! `session_find_by_id`, `window_find_by_id` and `window_pane_find_by_id`
//! walk. The one test that needs `command-alias` to say something else puts
//! back exactly what it found rather than a default of its own.

use crate::cmd::CMD_AFTERHOOK;
use crate::cmd::cmd_attach_session::{CMD_STARTSERVER, cmd_attach_session_entry};
use crate::cmd::{
    CMD_LIST_PRINT_ESCAPED, CMD_LIST_PRINT_NO_GROUPS, cmd_copy, cmd_find, cmd_free, cmd_get_alias,
    cmd_get_parse_flags, cmd_get_source, cmd_list_all_have, cmd_list_any_have, cmd_list_append_all,
    cmd_list_copy, cmd_list_move, cmd_list_new, cmd_list_print, cmd_mouse_at, cmd_mouse_pane,
    cmd_mouse_window, cmd_parse, cmd_print, cmd_table, cmd_template_replace,
};
use crate::cmd::{CMD_PARSE_SUCCESS, cmd_parse_from_string};
use crate::options::options_get_only_ptr;
use crate::options::{options_array_get, options_array_set, options_create_boxed, options_free};
use crate::session::session_get_curw;
use crate::tests::test_fixtures::{Target, globals, seen, zeroed_pane};
use crate::types::*;
use ::core::ffi::{CStr, c_int};
use ::core::ptr::{null, null_mut};

//
// helpers
//

/// Hands `words` to [`cmd_parse`] the way the command parser hands a command
/// line over — one string value per word — and answers the command it built,
/// or the reason it refused. `cmd_parse` copies whatever it keeps, so the
/// words stay the caller's.
unsafe fn parse_words(
    words: &[&CStr],
    file: Option<&CStr>,
    line: u_int,
    parse_flags: c_int,
) -> Result<Box<cmd>, String> {
    unsafe {
        let mut values: Vec<args_value_t> = words
            .iter()
            .map(|word| {
                let mut value = args_value_t::default();
                value.value = ArgsValue::String((*word).to_owned());
                value
            })
            .collect();
        cmd_parse(
            values.as_mut_ptr(),
            words.len() as u_int,
            file,
            line,
            parse_flags,
        )
        .map_err(|cause| cause.into_string().unwrap())
    }
}

/// What [`cmd_find`] says about `name` — either the entry it settled on or the
/// reason it would not.
unsafe fn find(name: &CStr) -> Result<&'static CStr, String> {
    unsafe {
        let mut cause = None;
        let entry = cmd_find(name.as_ptr(), &mut cause);
        if entry.is_null() {
            Err(cause.unwrap().into_string().unwrap())
        } else {
            assert!(cause.is_none(), "a found command left a reason behind");
            Ok((*entry).name)
        }
    }
}

/// A command list parsed from `s`, owned by the caller.
unsafe fn commands(s: &CStr) -> CmdListRef {
    unsafe {
        let mut pr = cmd_parse_from_string(s.as_ptr(), null_mut::<cmd_parse_input>());
        assert_eq!(pr.status, CMD_PARSE_SUCCESS, "{s:?} did not parse");
        pr.cmdlist.take().unwrap()
    }
}

unsafe fn printed(cmdlist: *mut cmd_list, flags: c_int) -> String {
    unsafe {
        cmd_list_print(cmdlist, flags)
            .to_string_lossy()
            .into_owned()
    }
}

/// What `template` becomes with `s` put in for argument `idx`.
unsafe fn expand(template: &CStr, s: &CStr, idx: c_int) -> String {
    unsafe {
        cmd_template_replace(template.as_ptr(), s.as_ptr(), idx)
            .to_string_lossy()
            .into_owned()
    }
}

/// Every command name in the table, in table order.
fn table_names() -> Vec<String> {
    cmd_table
        .iter()
        .map(|entry| entry.name.to_string_lossy().into_owned())
        .collect()
}

//
// the command table
//

#[test]
fn the_command_table_is_a_run_of_entries_in_name_order() {
    let names = table_names();
    assert_eq!(names.len(), cmd_table.len());
    assert_eq!(names.first().map(String::as_str), Some("attach-session"));
    assert_eq!(names.last().map(String::as_str), Some("wait-for"));

    // The lookup below reports an ambiguous prefix by listing the names in
    // the order they sit here, which is alphabetical apart from the two
    // display commands, whose entries are declared in one file.
    let mut sorted = names.clone();
    sorted.sort();
    let swapped = ["display-panes".to_string(), "display-popup".to_string()];
    assert_eq!(
        names
            .iter()
            .filter(|n| !swapped.contains(n))
            .collect::<Vec<_>>(),
        sorted
            .iter()
            .filter(|n| !swapped.contains(n))
            .collect::<Vec<_>>()
    );

    // Entries are held by reference to the modules' own statics, whose
    // addresses the crate compares against all over the place.
    assert!(::core::ptr::eq(
        cmd_table[0],
        &raw const cmd_attach_session_entry
    ));
}

//
// cmd_find — name lookup, aliases and ambiguity
//

#[test]
fn a_command_is_found_by_its_full_name_its_alias_or_an_unambiguous_prefix() {
    let _guard = globals();
    unsafe {
        assert_eq!(find(c"list-buffers"), Ok(c"list-buffers"));
        assert_eq!(find(c"lsb"), Ok(c"list-buffers"));

        // A prefix only one command answers to is that command, however short.
        assert_eq!(find(c"attach"), Ok(c"attach-session"));
        assert_eq!(find(c"a"), Ok(c"attach-session"));
        assert_eq!(find(c"list-b"), Ok(c"list-buffers"));
        assert_eq!(find(c"list-s"), Ok(c"list-sessions"));

        // An alias is tested for equality, so a prefix of one is not it: "ls"
        // is the alias of list-sessions and finds it outright, while "l" is a
        // prefix of several names and none of their aliases.
        assert_eq!(find(c"ls"), Ok(c"list-sessions"));
        assert!(find(c"l").is_err());

        assert_eq!(
            find(c"no-such-command"),
            Err("unknown command: no-such-command".to_string())
        );
    }
}

#[test]
fn an_ambiguous_prefix_is_reported_with_every_name_it_could_be() {
    let _guard = globals();
    unsafe {
        // Two or more matches, and the reason lists them in table order with
        // the trailing ", " cut back off.
        assert_eq!(
            find(c"list-c"),
            Err("ambiguous command: list-c, could be: list-clients, list-commands".to_string())
        );
        assert_eq!(
            find(c"kill-s"),
            Err("ambiguous command: kill-s, could be: kill-server, kill-session".to_string())
        );

        // The list is of names; an alias is never offered as one of the
        // things a prefix could be, even where one exists — "lsc" and "lscm"
        // are the two above.

        // An alias clears an ambiguity already gathered. "next" is a prefix of
        // both next-layout and next-window, and by the time the walk reaches
        // next-window it has already settled on next-layout — but next-window's
        // alias is "next" exactly, which puts the ambiguity back to none and
        // ends the walk there.
        assert_eq!(find(c"next"), Ok(c"next-window"));
        assert_eq!(find(c"nextl"), Ok(c"next-layout"));
    }
}

#[test]
fn an_empty_name_is_a_prefix_of_every_command_in_the_table() {
    let _guard = globals();
    unsafe {
        let names = table_names();
        let expected = format!("ambiguous command: , could be: {}", names.join(", "));
        assert_eq!(find(c""), Err(expected));

        // Everything the table holds fits inside the 8192-byte buffer the
        // reason is built in, with room to spare, which is why the two
        // truncation guards in the middle of that walk never fire.
        let listed: usize = names.iter().map(|n| n.len() + 2).sum();
        assert!(listed < 8192, "the whole table lists as {listed} bytes");
    }
}

//
// cmd_parse — building a command from argument values
//

#[test]
fn parsing_keeps_the_parse_flags_and_where_the_command_came_from() {
    let _guard = globals();
    unsafe {
        let mut cmd = parse_words(&[c"list-buffers"], Some(c"/etc/tmux.conf"), 17, 0x3)
            .expect("list-buffers parses");
        let cmd_ptr = &raw mut *cmd;

        assert_eq!(cmd_get_parse_flags(&*cmd_ptr), 0x3);
        let (file, line) = cmd_get_source(&*cmd_ptr);
        assert_eq!(seen(file), "/etc/tmux.conf");
        assert_eq!(line, 17);

        // A copy carries the entry, the file and the line over, and gets
        // arguments of its own. The parse flags are not copied.
        let mut copy = cmd_copy(&cmd, &[]);
        let copy_ptr = &raw mut *copy;
        let (copied_file, copied_line) = cmd_get_source(&*copy_ptr);
        assert_eq!(seen(copied_file), "/etc/tmux.conf");
        assert_ne!(copied_file, file, "the copy has a file name of its own");
        assert_eq!(copied_line, 17);
        assert_eq!(cmd_get_parse_flags(&*copy_ptr), 0);
        assert_eq!(cmd_print(copy_ptr).to_string_lossy(), "list-buffers");

        cmd_free(copy);
        cmd_free(cmd);
    }
}

#[test]
fn parsing_refuses_a_row_of_values_that_names_no_command() {
    let _guard = globals();
    unsafe {
        assert_eq!(
            parse_words(&[], None, 0, 0).err(),
            Some("no command".to_string())
        );

        // A first value that is a brace-enclosed command list rather than a
        // word is turned down the same way, without being looked at.
        let mut value = args_value_t::default();
        value.value = ArgsValue::Commands {
            cmdlist: None,
            cached: None,
        };
        let cause = cmd_parse(&raw mut value, 1, None, 0, 0)
            .err()
            .expect("a command list value names no command");
        assert_eq!(cause.to_str().unwrap(), "no command");
    }
}

#[test]
fn parsing_reports_a_bad_flag_as_the_commands_usage_or_as_the_parsers_reason() {
    let _guard = globals();
    unsafe {
        // `-?` is how the argument parser says "show the usage": it fails
        // without a reason of its own, and the command's own usage line is
        // what comes back.
        assert_eq!(
            parse_words(&[c"kill-window", c"-?"], None, 0, 0).err(),
            Some("usage: kill-window [-a] [-t target-window]".to_string())
        );

        // A flag the command does not know is reported by the argument parser,
        // and the command's name is put in front of what it said.
        assert_eq!(
            parse_words(&[c"kill-window", c"-Q"], None, 0, 0).err(),
            Some("command kill-window: unknown flag -Q".to_string())
        );

        // A name that is no command at all never reaches the argument parser.
        assert_eq!(
            parse_words(&[c"no-such-command"], None, 0, 0).err(),
            Some("unknown command: no-such-command".to_string())
        );
    }
}

//
// cmd_get_alias — the command-alias option
//

#[test]
fn an_alias_entry_without_an_equals_sign_is_skipped() {
    let _guard = globals();
    unsafe {
        let o = options_get_only_ptr(crate::tmux::global_options, c"command-alias".as_ptr());
        assert!(!o.is_null());

        // Put an entry carrying no `=` at the first index the array has none
        // at, and take exactly that index away again afterwards.
        let mut idx: u_int = 0;
        while !options_array_get(o, idx).is_null() {
            idx += 1;
        }
        assert_eq!(
            options_array_set(o, idx, c"bare-entry".as_ptr(), 0, &mut None),
            0
        );

        assert!(cmd_get_alias(c"bare-entry".as_ptr()).is_none());
        assert!(cmd_get_alias(c"bare".as_ptr()).is_none());
        // The entries around it still answer.
        assert_eq!(
            cmd_get_alias(c"splitp".as_ptr()).as_deref(),
            Some(c"split-window")
        );

        assert_eq!(options_array_set(o, idx, null(), 0, &mut None), 0);
        assert!(options_array_get(o, idx).is_null());
    }
}

#[test]
fn there_is_no_alias_at_all_when_the_option_is_not_there() {
    let _guard = globals();
    unsafe {
        // An option set that has never been given a default carries no
        // `command-alias`, and the lookup answers nothing rather than walking
        // an array it has not got. The global set is put back as it was.
        let saved = crate::tmux::global_options;
        let empty = Box::into_raw(options_create_boxed(null_mut::<options>()));
        crate::tmux::global_options = empty;
        let answer = cmd_get_alias(c"splitp".as_ptr());
        crate::tmux::global_options = saved;
        options_free(Box::from_raw(empty));

        assert!(answer.is_none());
        assert_eq!(
            cmd_get_alias(c"splitp".as_ptr()).as_deref(),
            Some(c"split-window")
        );
    }
}

//
// cmd_list_print — separators, escaping and groups
//

#[test]
fn a_command_list_prints_its_groups_with_double_separators() {
    let _guard = globals();
    unsafe {
        let first = commands(c"list-buffers ; list-clients");
        let second = commands(c"list-panes");
        cmd_list_move(first.as_ptr(), second.as_ptr());

        // Commands of one group are joined by a single separator and the step
        // between groups by a double one.
        assert_eq!(
            printed(first.as_ptr(), 0),
            "list-buffers ; list-clients ;; list-panes"
        );

        // Escaped, the separators come back the way a shell would have to
        // spell them.
        assert_eq!(
            printed(first.as_ptr(), CMD_LIST_PRINT_ESCAPED),
            "list-buffers \\; list-clients \\;\\; list-panes"
        );

        // Asked to forget the groups, every step is a single separator.
        assert_eq!(
            printed(first.as_ptr(), CMD_LIST_PRINT_NO_GROUPS),
            "list-buffers ; list-clients ; list-panes"
        );
        assert_eq!(
            printed(
                first.as_ptr(),
                CMD_LIST_PRINT_ESCAPED | CMD_LIST_PRINT_NO_GROUPS
            ),
            "list-buffers \\; list-clients \\; list-panes"
        );
    }
}

#[test]
fn moving_an_empty_list_onto_another_leaves_it_as_it_was() {
    let _guard = globals();
    unsafe {
        let first = commands(c"list-buffers");
        let empty = cmd_list_new();

        cmd_list_move(first.as_ptr(), empty.as_ptr());
        assert_eq!(printed(first.as_ptr(), 0), "list-buffers");

        // The same the other way about: what a move takes over keeps its
        // order, and the list it came from is left empty rather than freed.
        let second = commands(c"list-panes");
        cmd_list_append_all(empty.as_ptr(), second.as_ptr());
        assert_eq!(printed(empty.as_ptr(), 0), "list-panes");
        assert_eq!(printed(second.as_ptr(), 0), "");
        cmd_list_append_all(empty.as_ptr(), second.as_ptr());
        assert_eq!(printed(empty.as_ptr(), 0), "list-panes");
    }
}

#[test]
fn copying_a_command_list_keeps_its_groups_apart() {
    let _guard = globals();
    unsafe {
        // Every command of one group stays in one group in the copy.
        let one = commands(c"list-buffers ; list-clients");
        let copy = cmd_list_copy(one.as_ptr(), &[]);
        assert_eq!(printed(copy.as_ptr(), 0), "list-buffers ; list-clients");

        // A copy of a list holding two groups takes a fresh group number at
        // each step, so the step is still there afterwards.
        let two = commands(c"list-panes");
        cmd_list_move(one.as_ptr(), two.as_ptr());
        let copy = cmd_list_copy(one.as_ptr(), &[]);
        assert_eq!(
            printed(copy.as_ptr(), 0),
            "list-buffers ; list-clients ;; list-panes"
        );
    }
}

//
// cmd_list_all_have / cmd_list_any_have
//

#[test]
fn a_command_lists_flags_are_read_across_every_command_in_it() {
    let _guard = globals();
    unsafe {
        // Every command in this one runs its after hook.
        let all = commands(c"list-windows ; rename-window name");
        assert_eq!(cmd_list_all_have(all.as_ptr(), CMD_AFTERHOOK), 1);
        assert_eq!(cmd_list_any_have(all.as_ptr(), CMD_AFTERHOOK), 1);

        // One of these does and one does not.
        let some = commands(c"list-windows ; kill-window");
        assert_eq!(cmd_list_all_have(some.as_ptr(), CMD_AFTERHOOK), 0);
        assert_eq!(cmd_list_any_have(some.as_ptr(), CMD_AFTERHOOK), 1);

        // Neither of these does.
        let none = commands(c"kill-window ; kill-session");
        assert_eq!(cmd_list_all_have(none.as_ptr(), CMD_AFTERHOOK), 0);
        assert_eq!(cmd_list_any_have(none.as_ptr(), CMD_AFTERHOOK), 0);
        assert_eq!(cmd_list_any_have(none.as_ptr(), CMD_STARTSERVER), 0);

        // An empty list has every flag and none of them.
        let empty = cmd_list_new();
        assert_eq!(cmd_list_all_have(empty.as_ptr(), CMD_AFTERHOOK), 1);
        assert_eq!(cmd_list_any_have(empty.as_ptr(), CMD_AFTERHOOK), 0);
    }
}

//
// cmd_mouse_at — where in a pane a mouse event landed
//

#[test]
fn a_mouse_event_lands_at_a_pane_offset_or_outside_the_pane_altogether() {
    unsafe {
        let mut wp = zeroed_pane();
        wp.xoff = 10;
        wp.yoff = 5;
        wp.sx = 20;
        wp.sy = 8;
        let wp = &raw mut *wp;

        let mut m = *Box::new(mouse_event::default());
        m.x = 12;
        m.y = 6;
        assert_eq!(cmd_mouse_at(wp, &raw mut m, 0), Some((2, 1)));

        // The offsets a scrolled window carries are added on first.
        m.ox = 3;
        m.oy = 2;
        assert_eq!(cmd_mouse_at(wp, &raw mut m, 0), Some((5, 3)));

        // With `last` set it is the previous position that is read.
        m.lx = 11;
        m.ly = 5;
        assert_eq!(cmd_mouse_at(wp, &raw mut m, 1), Some((4, 2)));

        // A status line at the top pushes the rows down by its height.
        m.ox = 0;
        m.oy = 0;
        m.statusat = 0;
        m.statuslines = 2;
        m.y = 8;
        assert_eq!(cmd_mouse_at(wp, &raw mut m, 0), Some((2, 1)));

        // A status line anywhere else does not, and neither does one the event
        // landed above.
        m.statusat = 1;
        m.y = 6;
        assert_eq!(cmd_mouse_at(wp, &raw mut m, 0), Some((2, 1)));
        m.statusat = 0;
        m.statuslines = 8;
        assert_eq!(cmd_mouse_at(wp, &raw mut m, 0), Some((2, 1)));

        m.statuslines = 0;
        assert_eq!(cmd_mouse_at(wp, &raw mut m, 0), Some((2, 1)));

        // Outside the pane, in either direction on either axis.
        m.statuslines = 0;
        m.x = 9;
        m.y = 6;
        assert_eq!(cmd_mouse_at(wp, &raw mut m, 0), None);
        m.x = 30;
        assert_eq!(cmd_mouse_at(wp, &raw mut m, 0), None);
        m.x = 12;
        m.y = 4;
        assert_eq!(cmd_mouse_at(wp, &raw mut m, 0), None);
        m.y = 13;
        assert_eq!(cmd_mouse_at(wp, &raw mut m, 0), None);
    }
}

//
// cmd_mouse_window / cmd_mouse_pane — what a mouse event points at
//

#[test]
fn a_mouse_event_resolves_to_the_window_its_ids_name() {
    let _guard = globals();
    unsafe {
        let mut target = Target::new(80, 24);
        target.add_window(1, 80, 24);

        let mut m = *Box::new(mouse_event::default());

        // An event nothing filled in points nowhere.
        assert!(cmd_mouse_window(&raw mut m).is_none());

        // Nor does a valid one carrying no session.
        m.valid = 1;
        m.s = -1;
        m.w = -1;
        assert!(cmd_mouse_window(&raw mut m).is_none());

        // Nor one naming a session that has gone.
        m.s = 99;
        assert!(cmd_mouse_window(&raw mut m).is_none());

        // No window means the session's current one, and the session comes
        // back alongside it.
        m.s = 0;
        assert_eq!(
            cmd_mouse_window(&raw mut m),
            Some((target.session(), session_get_curw(target.session())))
        );

        // A window id is looked up in the server's tree and then found among
        // the session's links.
        m.w = 1;
        assert_eq!(
            cmd_mouse_window(&raw mut m),
            Some((target.session(), target.winlink(1)))
        );

        // A window that has gone points nowhere.
        m.w = 99;
        assert!(cmd_mouse_window(&raw mut m).is_none());
    }
}

#[test]
fn a_mouse_event_resolves_to_a_pane_inside_the_window_it_names() {
    let _guard = globals();
    unsafe {
        let mut target = Target::new(80, 24);
        target.add_window(1, 80, 24);

        let mut m = *Box::new(mouse_event::default());

        // Nothing to resolve the window to is nothing to resolve the pane to.
        assert!(cmd_mouse_pane(&raw mut m).is_none());

        // No pane id means the window's active pane, and the session and
        // window link come back with it.
        m.valid = 1;
        m.s = 0;
        m.w = -1;
        m.wp = -1;
        assert_eq!(
            cmd_mouse_pane(&raw mut m),
            Some((
                target.session(),
                session_get_curw(target.session()),
                target.pane(0)
            ))
        );

        // A pane id is looked up in the server's tree and has to be in the
        // window the event named.
        m.wp = 0;
        assert_eq!(
            cmd_mouse_pane(&raw mut m).map(|(_, _, wp)| wp),
            Some(target.pane(0))
        );

        // The second window's pane is a real pane, but not this window's.
        m.wp = 1;
        assert!(cmd_mouse_pane(&raw mut m).is_none());

        // A pane that has gone points nowhere.
        m.wp = 99;
        assert!(cmd_mouse_pane(&raw mut m).is_none());
    }
}

//
// cmd_template_replace — the %1 and %% expansions
//

#[test]
fn a_template_with_no_percent_in_it_comes_back_as_it_was() {
    unsafe {
        assert_eq!(expand(c"nothing to do", c"arg", 1), "nothing to do");
        assert_eq!(expand(c"", c"arg", 1), "");
    }
}

#[test]
fn a_numbered_placeholder_is_replaced_only_when_its_number_matches() {
    unsafe {
        assert_eq!(expand(c"echo %1", c"hello", 1), "echo hello");
        assert_eq!(expand(c"echo %2", c"hello", 1), "echo %2");
        assert_eq!(expand(c"echo %2", c"hello", 2), "echo hello");

        // A numbered placeholder may be used as often as it likes.
        assert_eq!(expand(c"%1 and %1", c"x", 1), "x and x");

        // Only the digits 1 to 9 are placeholders; anything else after the
        // percent is copied out with it.
        assert_eq!(expand(c"%0 %a %", c"x", 1), "%0 %a %");
    }
}

#[test]
fn a_bare_double_percent_is_replaced_once_and_copied_out_after_that() {
    unsafe {
        assert_eq!(expand(c"echo %%", c"hello", 1), "echo hello");

        // The second one in the same template is literal, because the flag
        // that says a replacement has happened is never cleared.
        assert_eq!(expand(c"%% and %%", c"x", 1), "x and %%");

        // A numbered placeholder does not set that flag, so a `%%` after one
        // still replaces.
        assert_eq!(expand(c"%1 and %%", c"x", 1), "x and x");
    }
}

#[test]
fn a_placeholder_followed_by_a_percent_escapes_what_it_puts_in() {
    unsafe {
        // The quoted forms are `%1%` and `%%%`; each escapes the five
        // characters a command line would otherwise read.
        assert_eq!(expand(c"echo %1%", c"a\"b", 1), "echo a\\\"b");
        assert_eq!(expand(c"echo %1%", c"a\\b", 1), "echo a\\\\b");
        assert_eq!(expand(c"echo %1%", c"a$b", 1), "echo a\\$b");
        assert_eq!(expand(c"echo %1%", c"a;b", 1), "echo a\\;b");
        assert_eq!(expand(c"echo %1%", c"a~b", 1), "echo a\\~b");
        assert_eq!(expand(c"echo %%%", c"a;b", 1), "echo a\\;b");

        // Nothing else is escaped, and the unquoted form escapes nothing at
        // all.
        assert_eq!(expand(c"%1%", c"a'b c", 1), "a'b c");
        assert_eq!(expand(c"%1", c"a\"b;c", 1), "a\"b;c");

        // Every character of the argument can want escaping, which is what the
        // three-times-the-length the buffer grows by is for.
        assert_eq!(expand(c"%1%", c"\"\\$;~", 1), "\\\"\\\\\\$\\;\\~");

        // An empty argument leaves the placeholder standing for nothing.
        assert_eq!(expand(c"[%1%]", c"", 1), "[]");
    }
}
