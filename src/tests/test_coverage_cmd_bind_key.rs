//! Unit tests for [`crate::cmd::cmd_bind_key`], the `bind-key` command: the
//! metadata its [`cmd_entry`] publishes, the argument-parse callback it hands
//! the parser, and every deterministic branch of its exec routine — key
//! validation, key-table selection, the repeat flag and note, and each of the
//! ways a bound command list is built.
//!
//! Exec is reached through the entry's own function pointer, exactly as the
//! command queue calls it, over items whose arguments come from the real
//! command parser. Bindings land in the process-wide `key_tables` tree and the
//! parser keeps state of its own in statics, so every test holds [`globals`]
//! and takes a [`Tables`] guard that puts the tree back the way it was found,
//! even through a failed assertion. Other suites leave tables such as `root`
//! behind on purpose, so nothing here assumes the tree is empty: what a test
//! must not touch is checked by taking a [`bound_keys`] snapshot before and
//! comparing it afterwards. No fatal path is touched either: the error
//! branches report through `cmdq_error` onto a client-less item, which only
//! records a config cause, and they are checked through return values and the
//! absence of side effects.

use crate::arguments::args_create;
use crate::arguments::{args_count, args_value};
use crate::cmd::cmd_bind_key::{
    ARGS_PARSE_COMMANDS_OR_STRING, CMD_AFTERHOOK, CMD_FIND_PANE, CMD_RETURN_ERROR,
    CMD_RETURN_NORMAL, KEYC_F5, KEYC_UNKNOWN, cmd_bind_key_entry,
};
use crate::cmd::{cmd_get_args_ptr, cmd_list_all};
use crate::input::KEYC_CTRL;
use crate::key_bindings::{
    KEY_BINDING_REPEAT, key_binding_cmdlist_ref, key_binding_flags, key_binding_key,
    key_binding_note, key_binding_tablename, key_bindings_first, key_bindings_get,
    key_bindings_get_table, key_bindings_next, key_bindings_remove, key_bindings_remove_table,
};
use crate::tests::test_fixtures::{Item, globals};
use crate::text::key_string_lookup_string;
use crate::types::*;
use ::core::ffi::CStr;

/// Where the tests' items claim to come from, which is what `cmdq_error`
/// reports them under.
const FILE: &CStr = c"test-coverage-cmd-bind-key.conf";

/// The tables bind-key can reach by itself.
const REACHABLE_TABLES: [&CStr; 3] = [c"prefix", c"root", c"bk-T-flag"];

/// The tables a test touched, taken back down again when the guard goes away —
/// even through a failed assertion — so the global tree is left as found.
struct Tables(Vec<&'static CStr>);

impl Tables {
    fn new() -> Tables {
        Tables(Vec::new())
    }

    /// Remembers `name` for cleanup without creating anything.
    fn track(&mut self, name: &'static CStr) {
        self.0.push(name);
    }

    /// Makes sure `name` does not exist now, remembering it so that it stays
    /// gone whatever the rest of the test does.
    fn clear(&mut self, name: &'static CStr) {
        self.track(name);
        unsafe { key_bindings_remove_table(name.as_ptr()) };
    }

    /// What every table bind-key could reach holds right now.
    fn snapshot(&self) -> Vec<Option<Vec<key_code>>> {
        REACHABLE_TABLES
            .map(|name| unsafe { bound_keys(name) })
            .to_vec()
    }
}

impl Drop for Tables {
    fn drop(&mut self) {
        for name in &self.0 {
            unsafe { key_bindings_remove_table(name.as_ptr()) };
        }
    }
}

/// Runs `bind-key`'s exec through the entry's own function pointer, the way
/// the command queue calls it, and answers what it answers.
unsafe fn exec_bind_key(item: &mut Item) -> cmd_retval {
    unsafe {
        let entry = &raw const cmd_bind_key_entry;
        let exec = (*entry).exec;
        let cmd = item.cmd();
        let qitem = item.ptr();
        exec(&*cmd, qitem)
    }
}

/// An item carrying a parsed `bind-key` command line, sourced from [`FILE`].
fn bind_item(line: &'static CStr, number: u_int) -> Item {
    Item::new().from_file(FILE, number).with_args(line)
}

/// The names of the commands of a bound list, in list order.
unsafe fn command_names(list: &Option<CmdListRef>) -> Vec<String> {
    unsafe {
        let list = list.as_ref().expect("a binding has a command list");
        cmd_list_all(list.as_ptr())
            .into_iter()
            .map(|cmd| (*cmd).entry.name.to_string_lossy().into_owned())
            .collect()
    }
}

/// The table named `name`, asserting the test itself created it.
unsafe fn table_named(name: &CStr) -> *mut key_table {
    unsafe {
        let table = key_bindings_get_table(name.as_ptr(), 0);
        assert!(!table.is_null(), "no {name:?} table exists");
        table
    }
}

/// Every key bound in `name`, or `None` when there is no such table.
unsafe fn bound_keys(name: &CStr) -> Option<Vec<key_code>> {
    unsafe {
        let table = key_bindings_get_table(name.as_ptr(), 0);
        if table.is_null() {
            return None;
        }
        let mut keys = Vec::new();
        let mut bd = key_bindings_first(table);
        while !bd.is_null() {
            keys.push(key_binding_key(bd));
            bd = key_bindings_next(table, bd);
        }
        Some(keys)
    }
}

#[test]
fn the_bind_key_entry_describes_the_bind_key_command() {
    let _guard = globals();
    unsafe {
        let entry = &raw const cmd_bind_key_entry;
        assert_eq!((*entry).name.to_string_lossy(), "bind-key");
        assert_eq!(
            (*entry)
                .alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "bind"
        );
        assert_eq!(
            (*entry).usage.to_string_lossy(),
            "[-nr] [-T key-table] [-N note] key [command [argument ...]]"
        );
        assert_eq!((*entry).args.template.to_string_lossy(), "nrN:T:");
        assert_eq!((*entry).args.lower, 1);
        assert_eq!((*entry).args.upper, -1);
        assert!((*entry).args.cb.is_some());

        assert_eq!((*entry).source.flag, 0);
        assert_eq!((*entry).source.type_0, CMD_FIND_PANE);
        assert_eq!((*entry).source.flags, 0);
        assert_eq!((*entry).target.flag, 0);
        assert_eq!((*entry).target.type_0, CMD_FIND_PANE);
        assert_eq!((*entry).target.flags, 0);

        assert_eq!((*entry).flags, CMD_AFTERHOOK);
    }
}

#[test]
fn the_argument_callback_accepts_commands_or_strings_wherever_it_is_asked() {
    let _guard = globals();
    unsafe {
        let entry = &raw const cmd_bind_key_entry;
        let cb = (*entry).args.cb.expect("bind-key has an args callback");
        let mut cause = None;
        assert_eq!(
            cb(&args_create(), 0, &mut cause),
            ARGS_PARSE_COMMANDS_OR_STRING
        );
        assert_eq!(
            cb(&args_create(), 7, &mut cause),
            ARGS_PARSE_COMMANDS_OR_STRING
        );
    }
}

#[test]
fn an_unknown_key_is_reported_and_never_touches_a_table() {
    let _guard = globals();
    let mut ts = Tables::new();
    let before = ts.snapshot();
    unsafe {
        for (i, line) in [
            c"bind-key None display-panes",
            c"bind-key Unknown a",
            c"bind-key NoSuchKey display-panes",
            c"bind-key None",
        ]
        .into_iter()
        .enumerate()
        {
            let mut item = bind_item(line, i as u_int + 1);
            assert_eq!(exec_bind_key(&mut item), CMD_RETURN_ERROR, "{line:?}");
        }
        assert_eq!(ts.snapshot(), before, "the tables changed");
    }
}

#[test]
fn a_plain_word_command_is_reparsed_into_the_prefix_table_by_default() {
    let _guard = globals();
    let mut ts = Tables::new();
    ts.clear(c"prefix");
    let root_before = unsafe { bound_keys(c"root") };
    unsafe {
        let mut item = bind_item(c"bind-key 0x41 display-panes", 1);
        assert_eq!(exec_bind_key(&mut item), CMD_RETURN_NORMAL);

        let table = table_named(c"prefix");

        let key = key_string_lookup_string(c"0x41".as_ptr());
        assert_ne!(key, KEYC_UNKNOWN);
        let bd = key_bindings_get(table, key);
        assert!(!bd.is_null());
        assert_eq!(key_binding_key(bd), key);
        assert_eq!(key_binding_tablename(bd), Some(c"prefix"));
        assert!(key_binding_note(bd).is_none());
        assert_eq!(key_binding_flags(bd) & KEY_BINDING_REPEAT, 0);

        assert_eq!(
            command_names(&key_binding_cmdlist_ref(bd)),
            vec!["display-panes"]
        );

        assert_eq!(bound_keys(c"root"), root_before, "root changed");
    }
}

#[test]
fn the_n_flag_binds_into_the_root_table_instead_of_prefix() {
    let _guard = globals();
    let mut ts = Tables::new();
    ts.clear(c"root");
    let prefix_before = unsafe { bound_keys(c"prefix") };
    unsafe {
        let mut item = bind_item(c"bind-key -n C-c display-panes", 1);
        assert_eq!(exec_bind_key(&mut item), CMD_RETURN_NORMAL);

        let table = table_named(c"root");
        let bd = key_bindings_get(table, 'c' as key_code | KEYC_CTRL);
        assert!(!bd.is_null());
        assert_eq!(key_binding_key(bd), 'c' as key_code | KEYC_CTRL);
        assert_eq!(key_binding_tablename(bd), Some(c"root"));
        assert_eq!(
            command_names(&key_binding_cmdlist_ref(bd)),
            vec!["display-panes"]
        );

        assert_eq!(bound_keys(c"prefix"), prefix_before, "prefix changed");
    }
}

#[test]
fn the_T_flag_names_the_table_and_wins_over_the_n_flag() {
    let _guard = globals();
    let mut ts = Tables::new();
    ts.clear(c"bk-T-flag");
    let before = ts.snapshot();
    unsafe {
        let mut by_name = bind_item(c"bind-key -T bk-T-flag F5 display-panes", 1);
        assert_eq!(exec_bind_key(&mut by_name), CMD_RETURN_NORMAL);
        let table = table_named(c"bk-T-flag");
        let bd = key_bindings_get(table, KEYC_F5);
        assert!(!bd.is_null());
        assert_eq!(key_binding_key(bd), KEYC_F5);
        assert_eq!(key_binding_tablename(bd), Some(c"bk-T-flag"));

        let mut with_n_too = bind_item(c"bind-key -n -T bk-T-flag a display-panes", 2);
        assert_eq!(exec_bind_key(&mut with_n_too), CMD_RETURN_NORMAL);
        assert!(!key_bindings_get(table, 'a' as key_code).is_null());
        assert_eq!(bound_keys(c"root"), before[1].clone(), "root changed");
        assert_eq!(bound_keys(c"prefix"), before[0].clone(), "prefix changed");
    }
}

#[test]
fn the_r_flag_marks_the_binding_repeatable_and_the_N_flag_notes_it() {
    let _guard = globals();
    let mut ts = Tables::new();
    ts.clear(c"prefix");
    unsafe {
        let mut noted = bind_item(c"bind-key -r -N home b display-panes", 1);
        assert_eq!(exec_bind_key(&mut noted), CMD_RETURN_NORMAL);
        let table = table_named(c"prefix");
        let bd = key_bindings_get(table, 'b' as key_code);
        assert!(!bd.is_null());
        assert_eq!(
            key_binding_flags(bd) & KEY_BINDING_REPEAT,
            KEY_BINDING_REPEAT
        );
        assert_eq!(key_binding_note(bd), Some(c"home"));

        let mut plain = bind_item(c"bind-key c display-panes", 2);
        assert_eq!(exec_bind_key(&mut plain), CMD_RETURN_NORMAL);
        let bd = key_bindings_get(table, 'c' as key_code);
        assert!(!bd.is_null());
        assert_eq!(key_binding_flags(bd) & KEY_BINDING_REPEAT, 0);
        assert!(key_binding_note(bd).is_none());
    }
}

#[test]
fn a_key_without_a_command_keeps_the_binding_but_updates_note_and_repeat() {
    let _guard = globals();
    let mut ts = Tables::new();
    ts.clear(c"prefix");
    unsafe {
        let mut bare = bind_item(c"bind-key d", 1);
        assert_eq!(exec_bind_key(&mut bare), CMD_RETURN_NORMAL);
        let table = table_named(c"prefix");
        assert!(
            key_bindings_first(table).is_null(),
            "a bare key must not create a binding"
        );

        let mut bound = bind_item(c"bind-key d { display-panes }", 2);
        assert_eq!(exec_bind_key(&mut bound), CMD_RETURN_NORMAL);
        let bd = key_bindings_get(table, 'd' as key_code);
        assert!(!bd.is_null());
        let list = key_binding_cmdlist_ref(bd);
        assert!(key_binding_note(bd).is_none());
        assert_eq!(key_binding_flags(bd) & KEY_BINDING_REPEAT, 0);

        let mut update = bind_item(c"bind-key -r -N touch d", 3);
        assert_eq!(exec_bind_key(&mut update), CMD_RETURN_NORMAL);
        let bd = key_bindings_get(table, 'd' as key_code);
        assert_eq!(key_binding_cmdlist_ref(bd), list);
        assert_eq!(
            key_binding_flags(bd) & KEY_BINDING_REPEAT,
            KEY_BINDING_REPEAT
        );
        assert_eq!(key_binding_note(bd), Some(c"touch"));

        key_bindings_remove(c"prefix".as_ptr(), 'd' as key_code);
    }
}

#[test]
fn braced_commands_are_attached_directly_and_take_an_extra_reference() {
    let _guard = globals();
    let mut ts = Tables::new();
    ts.clear(c"prefix");
    unsafe {
        let mut item = bind_item(c"bind-key e { display-panes }", 1);
        assert_eq!(exec_bind_key(&mut item), CMD_RETURN_NORMAL);

        let args = cmd_get_args_ptr(&*item.cmd());
        assert_eq!(args_count(&*args), 2);
        let value = args_value(args, 1);
        let ArgsValue::Commands { cmdlist: list, .. } = &(*value).value else {
            panic!("bind-key command value is not a command list");
        };
        let list = list.clone();

        let table = table_named(c"prefix");
        let bd = key_bindings_get(table, 'e' as key_code);
        assert!(!bd.is_null());
        assert_eq!(key_binding_cmdlist_ref(bd), list);
        assert_eq!(command_names(&list), vec!["display-panes"]);

        key_bindings_remove(c"prefix".as_ptr(), 'e' as key_code);
    }
}

#[test]
fn a_word_that_cannot_be_parsed_as_a_command_is_an_error_and_binds_nothing() {
    let _guard = globals();
    let mut ts = Tables::new();
    let before = ts.snapshot();
    unsafe {
        let mut item = bind_item(c"bind-key f badcmd", 1);
        assert_eq!(exec_bind_key(&mut item), CMD_RETURN_ERROR);
        assert_eq!(ts.snapshot(), before, "the tables changed");
    }
}

#[test]
fn several_words_are_reparsed_together_into_one_bound_command_list() {
    let _guard = globals();
    let mut ts = Tables::new();
    ts.clear(c"prefix");
    unsafe {
        let mut good = bind_item(c"bind-key g display-panes \\; display-message hi", 1);
        assert_eq!(exec_bind_key(&mut good), CMD_RETURN_NORMAL);
        let table = table_named(c"prefix");
        let bd = key_bindings_get(table, 'g' as key_code);
        assert!(!bd.is_null());
        let list = key_binding_cmdlist_ref(bd);
        assert_eq!(
            command_names(&list),
            vec!["display-panes", "display-message"]
        );

        ts.clear(c"prefix");
        let before = ts.snapshot();
        let mut bad = bind_item(c"bind-key h badcmd more", 2);
        assert_eq!(exec_bind_key(&mut bad), CMD_RETURN_ERROR);
        assert_eq!(ts.snapshot(), before, "the tables changed");
    }
}

#[test]
fn rebinding_the_same_key_replaces_the_old_command_list() {
    let _guard = globals();
    let mut ts = Tables::new();
    ts.clear(c"prefix");
    unsafe {
        let mut first = bind_item(c"bind-key i { display-panes }", 1);
        assert_eq!(exec_bind_key(&mut first), CMD_RETURN_NORMAL);
        let old =
            key_binding_cmdlist_ref(key_bindings_get(table_named(c"prefix"), 'i' as key_code));

        let mut second = bind_item(c"bind-key i display-message replaced", 2);
        assert_eq!(exec_bind_key(&mut second), CMD_RETURN_NORMAL);
        let bd = key_bindings_get(table_named(c"prefix"), 'i' as key_code);
        let new = key_binding_cmdlist_ref(bd);
        assert_ne!(new, old);
        assert_eq!(command_names(&new), vec!["display-message"]);
        assert!(key_binding_note(bd).is_none());
        assert_eq!(key_binding_flags(bd) & KEY_BINDING_REPEAT, 0);
    }
}

#[test]
fn the_unknown_key_check_runs_before_anything_else_is_touched() {
    let _guard = globals();
    let mut ts = Tables::new();
    let before = ts.snapshot();
    unsafe {
        let mut item = bind_item(c"bind-key -T bk-T-flag -r -N note Unknown x y z", 1);
        assert_eq!(exec_bind_key(&mut item), CMD_RETURN_ERROR);
        assert_eq!(ts.snapshot(), before, "the tables changed");
    }
}

#[test]
fn every_reachable_table_is_left_as_it_was_found_after_a_full_bind() {
    let _guard = globals();
    let mut ts = Tables::new();
    for name in REACHABLE_TABLES {
        ts.clear(name);
    }
    let before = ts.snapshot();
    unsafe {
        let mut item = bind_item(
            c"bind-key -r -N note -T bk-T-flag C-c display-panes \\; display-message done",
            1,
        );
        assert_eq!(exec_bind_key(&mut item), CMD_RETURN_NORMAL);

        let after = ts.snapshot();
        assert_ne!(after, before, "nothing was bound at all");
        assert_eq!(
            after[2].clone(),
            Some(vec!['c' as key_code | KEYC_CTRL]),
            "-T's table holds exactly the new binding"
        );
        let mut others = after;
        others[2] = before[2].clone();
        assert_eq!(others, before, "tables other than -T's changed");

        let bd = key_bindings_get(table_named(c"bk-T-flag"), 'c' as key_code | KEYC_CTRL);
        assert!(!bd.is_null());
        assert_eq!(
            key_binding_flags(bd) & KEY_BINDING_REPEAT,
            KEY_BINDING_REPEAT
        );
        assert_eq!(key_binding_note(bd), Some(c"note"));
        assert_eq!(
            command_names(&key_binding_cmdlist_ref(bd)),
            vec!["display-panes", "display-message"]
        );
    }
}
