//! Unit tests for [`crate::cmd::cmd_unbind_key`], the `unbind-key` command:
//! the metadata its [`cmd_entry`] publishes and every deterministic branch of
//! its exec routine — whole-table removal through `-a`, table selection by
//! `-T`, `-n` or the `prefix` default, and single-key removal, including each
//! of the ways a request is refused.
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
//! absence of side effects. Each refusal is exercised with and without `-q`,
//! so both sides of every `quiet` branch run even though the message itself is
//! not observed.

use crate::cmd::cmd_unbind_key::{
    CMD_AFTERHOOK, CMD_FIND_PANE, CMD_RETURN_ERROR, CMD_RETURN_NORMAL, KEYC_F5,
    cmd_unbind_key_entry,
};
use crate::cmd::{CMD_PARSE_SUCCESS, cmd_parse_from_string};
use crate::input::KEYC_CTRL;
use crate::key_bindings::{
    key_binding_key, key_bindings_add, key_bindings_first, key_bindings_get_table,
    key_bindings_next, key_bindings_remove_table,
};
use crate::tests::test_fixtures::{Item, globals};
use crate::text::key_string_lookup_string;
use crate::types::*;
use ::core::ffi::{CStr, c_char};
use ::core::ptr::{null, null_mut};

/// Where the tests' items claim to come from, which is what `cmdq_error`
/// reports them under.
const FILE: &CStr = c"test-coverage-cmd-unbind-key.conf";

/// The tables unbind-key can reach by itself.
const REACHABLE_TABLES: [&CStr; 4] = [c"prefix", c"root", c"uk-T-flag", c"uk-a-flag"];

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

    /// What every table unbind-key could reach holds right now.
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

/// Runs `unbind-key`'s exec through the entry's own function pointer, the way
/// the command queue calls it, and answers what it answers.
unsafe fn exec_unbind_key(item: &mut Item) -> cmd_retval {
    unsafe {
        let entry = &raw const cmd_unbind_key_entry;
        let exec = (*entry).exec;
        exec(&*item.cmd(), item.ptr())
    }
}

/// An item carrying a parsed `unbind-key` command line, sourced from [`FILE`].
fn unbind_item(line: &'static CStr, number: u_int) -> Item {
    Item::new().from_file(FILE, number).with_args(line)
}

/// Binds `key` to a fresh one-command list in `name`, which creates the table
/// if need be, the way `key_bindings_add` is used everywhere else. The binding
/// owns the list, so taking it down later frees everything again.
unsafe fn bind_one(name: &CStr, key: key_code) {
    unsafe {
        let mut pr =
            cmd_parse_from_string(c"display-panes".as_ptr(), null_mut::<cmd_parse_input>());
        assert_eq!(pr.status, CMD_PARSE_SUCCESS);
        key_bindings_add(name.as_ptr(), key, null::<c_char>(), 0, pr.cmdlist.take());
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
fn the_unbind_key_entry_describes_the_unbind_key_command() {
    let _guard = globals();
    unsafe {
        let entry = &raw const cmd_unbind_key_entry;
        assert_eq!((*entry).name.to_string_lossy(), "unbind-key");
        assert_eq!(
            (*entry)
                .alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "unbind"
        );
        assert_eq!(
            (*entry).usage.to_string_lossy(),
            "[-anq] [-T key-table] key"
        );
        assert_eq!((*entry).args.template.to_string_lossy(), "anqT:");
        assert_eq!((*entry).args.lower, 0);
        assert_eq!((*entry).args.upper, 1);
        assert!((*entry).args.cb.is_none());

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
fn unbinding_all_removes_the_whole_prefix_table_by_default() {
    let _guard = globals();
    let mut ts = Tables::new();
    ts.clear(c"prefix");
    ts.clear(c"root");
    unsafe { bind_one(c"prefix", 'a' as key_code) };
    unsafe { bind_one(c"prefix", 'b' as key_code) };
    let before = ts.snapshot();
    unsafe {
        let mut item = unbind_item(c"unbind-key -a", 1);
        assert_eq!(exec_unbind_key(&mut item), CMD_RETURN_NORMAL);
        assert!(key_bindings_get_table(c"prefix".as_ptr(), 0).is_null());
        let mut others = ts.snapshot();
        others[0] = before[0].clone();
        assert_ne!(before[0], None, "nothing was bound to begin with");
        assert_eq!(others, before, "a table other than prefix changed");

        let mut again = unbind_item(c"unbind-key -a", 2);
        assert_eq!(exec_unbind_key(&mut again), CMD_RETURN_ERROR);
    }
}

#[test]
fn unbinding_all_with_n_removes_the_root_table_instead() {
    let _guard = globals();
    let mut ts = Tables::new();
    ts.clear(c"root");
    ts.clear(c"prefix");
    unsafe { bind_one(c"root", 'c' as key_code | KEYC_CTRL) };
    let before = ts.snapshot();
    unsafe {
        let mut item = unbind_item(c"unbind-key -n -a", 1);
        assert_eq!(exec_unbind_key(&mut item), CMD_RETURN_NORMAL);
        assert!(key_bindings_get_table(c"root".as_ptr(), 0).is_null());
        let mut others = ts.snapshot();
        others[1] = before[1].clone();
        assert_ne!(before[1], None, "nothing was bound to begin with");
        assert_eq!(others, before, "a table other than root changed");
    }
}

#[test]
fn unbinding_all_names_the_T_table_and_wins_over_the_n_flag() {
    let _guard = globals();
    let mut ts = Tables::new();
    ts.clear(c"uk-a-flag");
    ts.clear(c"prefix");
    ts.clear(c"root");
    unsafe { bind_one(c"uk-a-flag", KEYC_F5) };
    let before = ts.snapshot();
    unsafe {
        let mut item = unbind_item(c"unbind-key -n -T uk-a-flag -a", 1);
        assert_eq!(exec_unbind_key(&mut item), CMD_RETURN_NORMAL);
        assert!(key_bindings_get_table(c"uk-a-flag".as_ptr(), 0).is_null());
        let mut others = ts.snapshot();
        others[3] = before[3].clone();
        assert_ne!(before[3], None, "nothing was bound to begin with");
        assert_eq!(others, before, "a table other than uk-a-flag changed");

        let mut missing = unbind_item(c"unbind-key -T uk-a-flag -a", 2);
        assert_eq!(exec_unbind_key(&mut missing), CMD_RETURN_ERROR);
    }
}

#[test]
fn a_plain_word_unbinds_from_the_prefix_table_and_leaves_the_rest() {
    let _guard = globals();
    let mut ts = Tables::new();
    ts.clear(c"prefix");
    unsafe { bind_one(c"prefix", key_string_lookup_string(c"0x41".as_ptr())) };
    unsafe { bind_one(c"prefix", 'b' as key_code) };
    let before = ts.snapshot();
    unsafe {
        let mut item = unbind_item(c"unbind-key 0x41", 1);
        assert_eq!(exec_unbind_key(&mut item), CMD_RETURN_NORMAL);
        assert_eq!(bound_keys(c"prefix"), Some(vec!['b' as key_code]));

        let mut last = unbind_item(c"unbind-key b", 2);
        assert_eq!(exec_unbind_key(&mut last), CMD_RETURN_NORMAL);
        assert!(key_bindings_get_table(c"prefix".as_ptr(), 0).is_null());

        let mut unknown = unbind_item(c"unbind-key NoSuchKey", 3);
        assert_eq!(exec_unbind_key(&mut unknown), CMD_RETURN_ERROR);
        assert_eq!(ts.snapshot()[1..], before[1..], "other tables changed");
        assert!(key_bindings_get_table(c"prefix".as_ptr(), 0).is_null());
    }
}

#[test]
fn the_n_flag_unbinds_from_the_root_table_instead_of_prefix() {
    let _guard = globals();
    let mut ts = Tables::new();
    ts.clear(c"root");
    ts.clear(c"prefix");
    unsafe { bind_one(c"root", 'c' as key_code | KEYC_CTRL) };
    unsafe { bind_one(c"prefix", 'd' as key_code) };
    let before = ts.snapshot();
    unsafe {
        let mut item = unbind_item(c"unbind-key -n C-c", 1);
        assert_eq!(exec_unbind_key(&mut item), CMD_RETURN_NORMAL);
        assert!(key_bindings_get_table(c"root".as_ptr(), 0).is_null());
        let mut others = ts.snapshot();
        others[1] = before[1].clone();
        assert_eq!(others, before, "a table other than root changed");
    }
}

#[test]
fn the_T_flag_unbinds_from_the_named_table_alone() {
    let _guard = globals();
    let mut ts = Tables::new();
    ts.clear(c"uk-T-flag");
    ts.clear(c"prefix");
    ts.clear(c"root");
    unsafe { bind_one(c"uk-T-flag", KEYC_F5) };
    unsafe { bind_one(c"uk-T-flag", 'e' as key_code) };
    let before = ts.snapshot();
    unsafe {
        let mut item = unbind_item(c"unbind-key -T uk-T-flag F5", 1);
        assert_eq!(exec_unbind_key(&mut item), CMD_RETURN_NORMAL);
        assert_eq!(bound_keys(c"uk-T-flag"), Some(vec!['e' as key_code]));
        let mut others = ts.snapshot();
        others[2] = before[2].clone();
        assert_eq!(others, before, "a table other than uk-T-flag changed");
        let mid = ts.snapshot();

        let mut missing = unbind_item(c"unbind-key -T uk-missing F5", 2);
        assert_eq!(exec_unbind_key(&mut missing), CMD_RETURN_ERROR);
        assert!(key_bindings_get_table(c"uk-missing".as_ptr(), 0).is_null());
        assert_eq!(
            ts.snapshot(),
            mid,
            "a refused unbind created or changed a table"
        );
    }
}
