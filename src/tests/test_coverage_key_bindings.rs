//! Unit tests for [`crate::key_bindings`], the bind-key engine behind
//! the key tables.
//!
//! Every function here works on the process-wide `key_tables` tree, so each
//! test takes [`globals`] and brings a [`Tables`] guard along: it takes the
//! tables the test created back down again even through a failed assertion,
//! so the tree is left exactly as it was found. Command lists come from the
//! real parser rather than hand-rolled internals; the one thing tests do
//! build by hand is a single-node default tree, which is the shape
//! `key_bindings_init_done` leaves for a table with one default key.

use crate::cmd::{CMD_PARSE_SUCCESS, cmd_parse_from_string};
use crate::fmt_args;
use crate::input::{KEYC_LITERAL, KEYC_META};
use crate::key_bindings::{
    KEY_BINDING_REPEAT, KEYC_MASK_FLAGS, key_binding_cmdlist_ref, key_binding_flags,
    key_binding_key, key_binding_note, key_binding_tablename, key_bindings_add, key_bindings_first,
    key_bindings_first_table, key_bindings_get, key_bindings_get_default, key_bindings_get_table,
    key_bindings_get_table_ref, key_bindings_has_repeat, key_bindings_next,
    key_bindings_next_table, key_bindings_remove, key_bindings_remove_table, key_bindings_reset,
    key_bindings_reset_table, key_bindings_take_defaults, key_table_has_defaults,
    key_table_is_empty, key_table_name,
};
use crate::options::options_set_string;
use crate::server::server_client_set_key_table;
use crate::tests::test_fixtures::{Clients, Session, globals};
use crate::types::*;
use ::core::ffi::{CStr, c_int};
use ::core::ptr::{null, null_mut};
use ::std::sync::MutexGuard;

/// The tables a test created, taken back down again when the guard goes away —
/// even through a failed assertion — so the global tree is left as found.
struct Tables(Vec<&'static CStr>);

impl Tables {
    fn new() -> Tables {
        Tables(Vec::new())
    }

    /// Records a name for cleanup without creating anything, for a table some
    /// call under test is about to create itself.
    fn track(&mut self, name: &'static CStr) {
        self.0.push(name);
    }

    /// Creates the named table and remembers it for cleanup.
    fn take(&mut self, name: &'static CStr) -> *mut key_table {
        self.track(name);
        unsafe { key_bindings_get_table(name.as_ptr(), 1) }
    }
}

impl Drop for Tables {
    fn drop(&mut self) {
        for name in &self.0 {
            unsafe { key_bindings_remove_table(name.as_ptr()) };
        }
    }
}

/// Parses one command line and hands back the list it built, ready to be
/// handed over to a binding.
unsafe fn parsed_list(s: &'static CStr) -> Option<CmdListRef> {
    unsafe {
        let mut pr = cmd_parse_from_string(s.as_ptr(), null_mut::<cmd_parse_input>());
        assert_eq!(pr.status, CMD_PARSE_SUCCESS, "{s:?} did not parse");
        pr.cmdlist.take()
    }
}

/// Puts a binding into a table's default tree and nowhere else, the way the
/// server does: the binding is made, the table takes what it holds as its
/// defaults, and the live binding is then removed again.
unsafe fn install_default(
    name: &'static CStr,
    table: *mut key_table,
    key: key_code,
    note: Option<&'static CStr>,
    flags: c_int,
    list: Option<CmdListRef>,
) {
    unsafe {
        key_bindings_add(
            name.as_ptr(),
            key,
            note.map_or(null(), CStr::as_ptr),
            (flags & KEY_BINDING_REPEAT != 0) as c_int,
            list,
        );
        key_bindings_take_defaults(table);
        key_bindings_remove(name.as_ptr(), key);
    }
}

/// The keys of a table's live bindings, in the order first/next visits them.
unsafe fn binding_keys(table: *mut key_table) -> Vec<key_code> {
    unsafe {
        let mut keys = Vec::new();
        let mut bd = key_bindings_first(table);
        while !bd.is_null() {
            keys.push(key_binding_key(bd));
            bd = key_bindings_next(table, bd);
        }
        keys
    }
}

/// The names of every table in the server, in the order the tree walks them.
unsafe fn table_names() -> Vec<String> {
    unsafe {
        let mut names = Vec::new();
        let mut table = key_bindings_first_table();
        while !table.is_null() {
            names.push(key_table_name(table).to_string_lossy().into_owned());
            table = key_bindings_next_table(table);
        }
        names
    }
}

#[test]
fn key_bindings_get_table_looks_up_without_and_with_creation() {
    let _guard: MutexGuard<()> = globals();
    let mut ts = Tables::new();
    let name = c"kb-get-table";
    unsafe {
        assert!(key_bindings_get_table(name.as_ptr(), 0).is_null());

        let table = ts.take(name);
        assert_eq!(key_table_name(table), c"kb-get-table");
        assert!(key_table_is_empty(table));
        assert!(!key_table_has_defaults(table));

        assert_eq!(key_bindings_get_table(name.as_ptr(), 1), table);
        assert_eq!(key_bindings_get_table(name.as_ptr(), 0), table);

        key_bindings_remove_table(name.as_ptr());
        assert!(key_bindings_get_table(name.as_ptr(), 0).is_null());
    }
}

#[test]
fn key_bindings_add_stores_a_parsed_command_list_and_replaces_it() {
    let _guard = globals();
    let mut ts = Tables::new();
    let name = c"kb-add";
    unsafe {
        let table = ts.take(name);
        let first = parsed_list(c"display-message one");
        key_bindings_add(
            name.as_ptr(),
            b'a' as key_code,
            c"first note".as_ptr(),
            0,
            first.clone(),
        );

        let bd = key_bindings_get(table, b'a' as key_code);
        assert!(!bd.is_null());
        assert_eq!(key_binding_key(bd), b'a' as key_code);
        assert_eq!(key_binding_tablename(bd), Some(c"kb-add"));
        assert_eq!(key_binding_note(bd), Some(c"first note"));
        assert_eq!(key_binding_flags(bd), 0);
        assert_eq!(key_binding_cmdlist_ref(bd), first);

        let second = parsed_list(c"display-message two");
        key_bindings_add(name.as_ptr(), b'a' as key_code, null(), 0, second.clone());
        let bd = key_bindings_get(table, b'a' as key_code);
        assert_eq!(key_binding_cmdlist_ref(bd), second);
        assert!(key_binding_note(bd).is_none());

        assert!(key_bindings_get_default(table, b'a' as key_code).is_null());
        assert!(key_bindings_get(table, b'b' as key_code).is_null());
    }
}

#[test]
fn key_bindings_add_updates_note_and_repeat_without_a_command_list() {
    let _guard = globals();
    let mut ts = Tables::new();
    let name = c"kb-update";
    unsafe {
        let table = ts.take(name);
        let list = parsed_list(c"display-message keep");
        key_bindings_add(
            name.as_ptr(),
            b'a' as key_code,
            c"before".as_ptr(),
            0,
            list.clone(),
        );

        key_bindings_add(name.as_ptr(), b'a' as key_code, null(), 0, None);
        let bd = key_bindings_get(table, b'a' as key_code);
        assert_eq!(key_binding_note(bd), Some(c"before"));
        assert_eq!(key_binding_flags(bd) & KEY_BINDING_REPEAT, 0);
        assert_eq!(key_binding_cmdlist_ref(bd), list);

        key_bindings_add(name.as_ptr(), b'a' as key_code, c"after".as_ptr(), 0, None);
        let bd = key_bindings_get(table, b'a' as key_code);
        assert_eq!(key_binding_note(bd), Some(c"after"));
        assert_eq!(key_binding_flags(bd) & KEY_BINDING_REPEAT, 0);
        assert_eq!(key_binding_cmdlist_ref(bd), list);

        key_bindings_add(name.as_ptr(), b'a' as key_code, null(), 1, None);
        let bd = key_bindings_get(table, b'a' as key_code);
        assert_eq!(
            key_binding_flags(bd) & KEY_BINDING_REPEAT,
            KEY_BINDING_REPEAT
        );
        assert_eq!(key_binding_note(bd), Some(c"after"));
        assert_eq!(key_binding_cmdlist_ref(bd), list);

        key_bindings_add(name.as_ptr(), b'a' as key_code, c"later".as_ptr(), 0, None);
        let bd = key_bindings_get(table, b'a' as key_code);
        assert_eq!(key_binding_note(bd), Some(c"later"));
        assert_eq!(
            key_binding_flags(bd) & KEY_BINDING_REPEAT,
            KEY_BINDING_REPEAT
        );

        let ghost = c"kb-update-empty";
        ts.track(ghost);
        key_bindings_add(ghost.as_ptr(), b'z' as key_code, c"ghost".as_ptr(), 0, None);
        let empty = key_bindings_get_table(ghost.as_ptr(), 0);
        assert!(!empty.is_null());
        assert!(key_bindings_first(empty).is_null());
        assert!(key_bindings_get(empty, b'z' as key_code).is_null());
    }
}

#[test]
fn key_bindings_add_and_remove_mask_the_flag_bits_off_the_key() {
    let _guard = globals();
    let mut ts = Tables::new();
    let name = c"kb-mask";
    unsafe {
        let table = ts.take(name);
        let flagged = b'a' as key_code | KEYC_LITERAL;
        key_bindings_add(
            name.as_ptr(),
            flagged,
            c"masked".as_ptr(),
            0,
            parsed_list(c"display-message masked"),
        );

        let bd = key_bindings_get(table, b'a' as key_code);
        assert!(!bd.is_null());
        assert_eq!(key_binding_key(bd), b'a' as key_code);
        assert_eq!(key_binding_key(bd) & KEYC_MASK_FLAGS, 0);

        assert!(key_bindings_get(table, flagged).is_null());

        let meta = b'b' as key_code | KEYC_META;
        key_bindings_add(
            name.as_ptr(),
            meta,
            null(),
            0,
            parsed_list(c"display-message meta"),
        );
        let bd = key_bindings_get(table, meta);
        assert!(!bd.is_null());
        assert_eq!(key_binding_key(bd), meta);

        key_bindings_remove(name.as_ptr(), flagged);
        assert!(key_bindings_get(table, b'a' as key_code).is_null());
    }
}

#[test]
fn key_bindings_first_and_next_walk_a_table_in_key_order() {
    let _guard = globals();
    let mut ts = Tables::new();
    let name = c"kb-walk";
    unsafe {
        let table = ts.take(name);
        assert!(key_bindings_first(table).is_null());
        assert!(binding_keys(table).is_empty());

        for key in [300 as key_code, 100, 400, 200] {
            key_bindings_add(
                name.as_ptr(),
                key,
                null(),
                0,
                parsed_list(c"display-message walk"),
            );
        }
        assert_eq!(binding_keys(table), vec![100 as key_code, 200, 300, 400]);

        let last = key_bindings_get(table, 400 as key_code);
        assert!(key_bindings_next(table, last).is_null());
    }
}

#[test]
fn key_bindings_remove_forgets_a_binding_and_then_the_empty_table() {
    let _guard = globals();
    let mut ts = Tables::new();
    unsafe {
        let missing = c"kb-remove-nowhere";
        key_bindings_remove(missing.as_ptr(), b'a' as key_code);
        assert!(key_bindings_get_table(missing.as_ptr(), 0).is_null());

        let name = c"kb-remove";
        let table = ts.take(name);
        for key in [b'a' as key_code, b'b' as key_code] {
            key_bindings_add(
                name.as_ptr(),
                key,
                null(),
                0,
                parsed_list(c"display-message gone"),
            );
        }

        key_bindings_remove(name.as_ptr(), b'q' as key_code);
        assert_eq!(
            binding_keys(table),
            vec![b'a' as key_code, b'b' as key_code]
        );

        key_bindings_remove(name.as_ptr(), b'a' as key_code);
        assert_eq!(binding_keys(table), vec![b'b' as key_code]);
        assert_eq!(key_bindings_get_table(name.as_ptr(), 0), table);

        key_bindings_remove(name.as_ptr(), b'b' as key_code);
        assert!(key_bindings_get_table(name.as_ptr(), 0).is_null());
    }
}

#[test]
fn key_bindings_reset_without_a_default_removes_the_binding_instead() {
    let _guard = globals();
    let mut ts = Tables::new();
    unsafe {
        let missing = c"kb-reset-nowhere";
        key_bindings_reset(missing.as_ptr(), b'a' as key_code);
        assert!(key_bindings_get_table(missing.as_ptr(), 0).is_null());

        let name = c"kb-reset";
        let table = ts.take(name);
        for key in [b'a' as key_code, b'b' as key_code] {
            key_bindings_add(
                name.as_ptr(),
                key,
                null(),
                0,
                parsed_list(c"display-message custom"),
            );
        }

        key_bindings_reset(name.as_ptr(), b'q' as key_code);
        assert_eq!(
            binding_keys(table),
            vec![b'a' as key_code, b'b' as key_code]
        );

        key_bindings_reset(name.as_ptr(), b'a' as key_code);
        assert_eq!(binding_keys(table), vec![b'b' as key_code]);
        assert_eq!(key_bindings_get_table(name.as_ptr(), 0), table);

        key_bindings_reset(name.as_ptr(), b'b' as key_code);
        assert!(key_bindings_get_table(name.as_ptr(), 0).is_null());
    }
}

#[test]
fn key_bindings_get_default_and_reset_restore_the_default_binding() {
    let _guard = globals();
    let mut ts = Tables::new();
    unsafe {
        let noted = c"kb-default-note";
        let table = ts.take(noted);
        let original = parsed_list(c"display-message original");
        install_default(
            noted,
            table,
            b'a' as key_code,
            Some(c"default note"),
            KEY_BINDING_REPEAT,
            original.clone(),
        );
        let dd = key_bindings_get_default(table, b'a' as key_code);
        assert_eq!(key_binding_cmdlist_ref(dd), original);
        assert!(key_bindings_get_default(table, b'b' as key_code).is_null());

        key_bindings_add(
            noted.as_ptr(),
            b'a' as key_code,
            c"custom".as_ptr(),
            0,
            parsed_list(c"display-message custom"),
        );
        key_bindings_reset(noted.as_ptr(), b'a' as key_code);

        let bd = key_bindings_get(table, b'a' as key_code);
        assert_eq!(key_binding_cmdlist_ref(bd), original);
        assert_eq!(key_binding_note(bd), Some(c"default note"));
        assert_eq!(key_binding_flags(bd), KEY_BINDING_REPEAT);

        let plain = c"kb-default-plain";
        let table = ts.take(plain);
        let bare = parsed_list(c"display-message plain");
        install_default(plain, table, b'x' as key_code, None, 0, bare.clone());
        key_bindings_add(
            plain.as_ptr(),
            b'x' as key_code,
            c"temporary".as_ptr(),
            1,
            parsed_list(c"display-message temporary"),
        );
        key_bindings_reset(plain.as_ptr(), b'x' as key_code);

        let bd = key_bindings_get(table, b'x' as key_code);
        assert_eq!(key_binding_cmdlist_ref(bd), bare);
        assert!(key_binding_note(bd).is_none());
        assert_eq!(key_binding_flags(bd), 0);
    }
}

#[test]
fn key_bindings_first_table_and_next_table_walk_the_tables_by_name() {
    let _guard = globals();
    let mut ts = Tables::new();
    unsafe {
        let alpha = c"kb-alpha-table";
        let mike = c"kb-mike-table";
        let zulu = c"kb-zulu-table";
        for name in [zulu, mike, alpha] {
            ts.take(name);
        }

        let names = table_names();
        let at = |want: &str| names.iter().position(|n| n == want).expect(want);
        assert!(at("kb-alpha-table") < at("kb-mike-table"));
        assert!(at("kb-mike-table") < at("kb-zulu-table"));

        key_bindings_remove_table(mike.as_ptr());
        let names = table_names();
        let alpha_at = names.iter().position(|n| n == "kb-alpha-table");
        let zulu_at = names.iter().position(|n| n == "kb-zulu-table");
        assert!(alpha_at.is_some() && zulu_at.is_some());
        assert!(alpha_at < zulu_at);
        assert!(!names.iter().any(|n| n == "kb-mike-table"));
        assert!(names.len() >= 2);

        let mut last = key_bindings_first_table();
        let mut following = key_bindings_next_table(last);
        while !following.is_null() {
            last = following;
            following = key_bindings_next_table(last);
        }
        assert!(!last.is_null());
        assert!(key_bindings_next_table(last).is_null());
    }
}

#[test]
fn key_bindings_registry_and_client_owners_are_independent() {
    let _guard = globals();
    let mut ts = Tables::new();
    let name = c"kb-unref";
    unsafe {
        let table_ref = key_bindings_get_table_ref(name.as_ptr(), 1).unwrap();
        let table = table_ref.as_ptr();
        ts.track(name);
        drop(table_ref);
        assert_eq!(key_bindings_get_table(name.as_ptr(), 0), table);
        key_bindings_remove_table(name.as_ptr());
        assert!(key_bindings_get_table(name.as_ptr(), 0).is_null());
    }
}

#[test]
fn key_bindings_remove_table_rebinds_attached_clients() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut ts = Tables::new();
    unsafe {
        let home = c"kb-client-home";
        let mut s = Session::new(1, "kb-session");
        options_set_string(
            s.options(),
            c"key-table".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![home.as_ptr()],
        );

        let c = clients.add("kb-client", 80, 24);
        (*c).session = s.ptr();
        let home_table_ref = key_bindings_get_table_ref(home.as_ptr(), 1).unwrap();
        let home_table = home_table_ref.as_ptr();
        (*c).keytable_ref = Some(home_table_ref.clone());

        let name = c"kb-detach";
        let table = ts.take(name);
        server_client_set_key_table(c, key_table_name(table).as_ptr());
        assert_eq!((*c).keytable(), table);

        key_bindings_remove_table(name.as_ptr());
        assert!(key_bindings_get_table(name.as_ptr(), 0).is_null());
        assert_eq!((*c).keytable(), home_table);

        (*c).keytable_ref = None;
        drop(home_table_ref);
        key_bindings_remove_table(home.as_ptr());
        assert!(key_bindings_get_table(home.as_ptr(), 0).is_null());
    }
}

#[test]
fn key_bindings_reset_table_restores_each_key_or_drops_the_table() {
    let _guard = globals();
    let mut ts = Tables::new();
    unsafe {
        let bare_name = c"kb-reset-table-bare";
        let table = ts.take(bare_name);
        for key in [b'p' as key_code, b'q' as key_code] {
            key_bindings_add(
                bare_name.as_ptr(),
                key,
                null(),
                0,
                parsed_list(c"display-message bare"),
            );
        }
        assert_eq!(binding_keys(table).len(), 2);
        key_bindings_reset_table(bare_name.as_ptr());
        assert!(key_bindings_get_table(bare_name.as_ptr(), 0).is_null());

        let mixed_name = c"kb-reset-table-mixed";
        let table = ts.take(mixed_name);
        let dflt = parsed_list(c"display-message dflt");
        install_default(mixed_name, table, b'd' as key_code, None, 0, dflt.clone());
        key_bindings_add(
            mixed_name.as_ptr(),
            b'c' as key_code,
            null(),
            0,
            parsed_list(c"display-message stray"),
        );
        key_bindings_add(
            mixed_name.as_ptr(),
            b'd' as key_code,
            null(),
            0,
            parsed_list(c"display-message overridden"),
        );

        key_bindings_reset_table(mixed_name.as_ptr());

        assert!(key_bindings_get(table, b'c' as key_code).is_null());
        let bd = key_bindings_get(table, b'd' as key_code);
        assert_eq!(key_binding_cmdlist_ref(bd), dflt);
        assert_eq!(key_bindings_get_table(mixed_name.as_ptr(), 0), table);
    }
}

#[test]
fn key_bindings_has_repeat_scans_only_the_given_bindings() {
    let _guard = globals();
    let mut ts = Tables::new();
    let name = c"kb-has-repeat";
    unsafe {
        let table = ts.take(name);
        key_bindings_add(
            name.as_ptr(),
            b'a' as key_code,
            null(),
            0,
            parsed_list(c"display-message plain"),
        );
        key_bindings_add(
            name.as_ptr(),
            b'b' as key_code,
            null(),
            1,
            parsed_list(c"display-message repeat"),
        );
        let ba = key_bindings_get(table, b'a' as key_code);
        let bb = key_bindings_get(table, b'b' as key_code);

        let plains = [ba, ba];
        assert_eq!(key_bindings_has_repeat(&plains[..0]), 0);
        assert_eq!(key_bindings_has_repeat(&plains[..1]), 0);

        let leading = [bb, ba];
        assert_eq!(key_bindings_has_repeat(&leading[..1]), 1);
        let trailing = [ba, bb];
        assert_eq!(key_bindings_has_repeat(&trailing[..1]), 0);
        assert_eq!(key_bindings_has_repeat(&trailing[..2]), 1);
    }
}
