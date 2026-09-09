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

use crate::cmd::CmdListRef;
use crate::cmd::{CMD_PARSE_SUCCESS, cmd_parse_from_string};
use crate::fmt_args;
use crate::input::{KEYC_LITERAL, KEYC_META};
use crate::key_bindings::{
    KEY_BINDING_REPEAT, KEYC_MASK_FLAGS, key_binding_cmdlist_ref, key_binding_flags,
    key_binding_key, key_binding_note, key_binding_tablename, key_bindings_add, key_bindings_get,
    key_bindings_get_default, key_bindings_get_table, key_bindings_get_table_ref,
    key_bindings_has_repeat, key_bindings_remove, key_bindings_remove_table, key_bindings_reset,
    key_bindings_reset_table, key_tables,
};

use crate::server::server_client_set_key_table;
use crate::tests::test_fixtures::{Clients, Session, globals};
use crate::types::*;
use ::core::ffi::{CStr, c_int};
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
    fn take(&mut self, name: &'static CStr) -> KeyTableRef {
        self.track(name);
        key_bindings_get_table(name, 1).unwrap()
    }
}

impl Drop for Tables {
    fn drop(&mut self) {
        for name in &self.0 {
            unsafe { key_bindings_remove_table(name) };
        }
    }
}

/// Parses one command line and hands back the list it built, ready to be
/// handed over to a binding.
unsafe fn parsed_list(s: &'static CStr) -> Option<CmdListRef> {
    unsafe {
        let mut pr = cmd_parse_from_string(s, None);
        assert_eq!(pr.status, CMD_PARSE_SUCCESS, "{s:?} did not parse");
        pr.cmdlist.take()
    }
}

/// Puts a binding into a table's default tree and nowhere else, the way the
/// server does: the binding is made, the table takes what it holds as its
/// defaults, and the live binding is then removed again.
unsafe fn install_default(
    name: &'static CStr,
    table: &KeyTableRef,
    key: key_code,
    note: Option<&'static CStr>,
    flags: c_int,
    list: Option<CmdListRef>,
) {
    unsafe {
        key_bindings_add(
            name,
            key,
            note,
            (flags & KEY_BINDING_REPEAT != 0) as c_int,
            list,
        );
        table.take_defaults();
        key_bindings_remove(name, key);
    }
}

/// The keys of a table's live bindings, in table order.
unsafe fn binding_keys(table: &KeyTableRef) -> Vec<key_code> {
    table.borrow().bindings().map(key_binding_key).collect()
}

/// The names of every table in the server, in the order the tree walks them.
fn table_names() -> Vec<String> {
    key_tables.with_borrow(|tables| {
        tables
            .keys()
            .map(|name| name.to_string_lossy().into_owned())
            .collect()
    })
}

#[test]
fn key_bindings_get_table_looks_up_without_and_with_creation() {
    let _guard: MutexGuard<()> = globals();
    let mut ts = Tables::new();
    let name = c"kb-get-table";
    unsafe {
        assert!(key_bindings_get_table(name, 0).is_none());

        let table = ts.take(name);
        assert_eq!(table.name().as_c_str(), c"kb-get-table");
        assert!(table.is_empty());
        assert!(!table.has_defaults());

        assert_eq!(key_bindings_get_table(name, 1), Some(table.clone()));
        assert_eq!(key_bindings_get_table(name, 0), Some(table.clone()));

        key_bindings_remove_table(name);
        assert!(key_bindings_get_table(name, 0).is_none());
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
            name,
            b'a' as key_code,
            Some(c"first note"),
            0,
            first.clone(),
        );

        let bd = key_bindings_get(&table.borrow(), b'a' as key_code);
        assert!(!bd.is_none());
        assert_eq!(key_binding_key(bd.as_ref().unwrap()), b'a' as key_code);
        assert_eq!(key_binding_tablename(bd.as_ref().unwrap()), Some(c"kb-add"));
        assert_eq!(key_binding_note(bd.as_ref().unwrap()), Some(c"first note"));
        assert_eq!(key_binding_flags(bd.as_ref().unwrap()), 0);
        assert_eq!(key_binding_cmdlist_ref(bd.as_ref().unwrap()), first);

        let second = parsed_list(c"display-message two");
        key_bindings_add(name, b'a' as key_code, None, 0, second.clone());
        let bd = key_bindings_get(&table.borrow(), b'a' as key_code);
        assert_eq!(key_binding_cmdlist_ref(bd.as_ref().unwrap()), second);
        assert!(key_binding_note(bd.as_ref().unwrap()).is_none());

        assert!(key_bindings_get_default(&table.borrow(), b'a' as key_code).is_none());
        assert!(key_bindings_get(&table.borrow(), b'b' as key_code).is_none());
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
        key_bindings_add(name, b'a' as key_code, Some(c"before"), 0, list.clone());

        key_bindings_add(name, b'a' as key_code, None, 0, None);
        let bd = key_bindings_get(&table.borrow(), b'a' as key_code);
        assert_eq!(key_binding_note(bd.as_ref().unwrap()), Some(c"before"));
        assert_eq!(
            key_binding_flags(bd.as_ref().unwrap()) & KEY_BINDING_REPEAT,
            0
        );
        assert_eq!(key_binding_cmdlist_ref(bd.as_ref().unwrap()), list);

        key_bindings_add(name, b'a' as key_code, Some(c"after"), 0, None);
        let bd = key_bindings_get(&table.borrow(), b'a' as key_code);
        assert_eq!(key_binding_note(bd.as_ref().unwrap()), Some(c"after"));
        assert_eq!(
            key_binding_flags(bd.as_ref().unwrap()) & KEY_BINDING_REPEAT,
            0
        );
        assert_eq!(key_binding_cmdlist_ref(bd.as_ref().unwrap()), list);

        key_bindings_add(name, b'a' as key_code, None, 1, None);
        let bd = key_bindings_get(&table.borrow(), b'a' as key_code);
        assert_eq!(
            key_binding_flags(bd.as_ref().unwrap()) & KEY_BINDING_REPEAT,
            KEY_BINDING_REPEAT
        );
        assert_eq!(key_binding_note(bd.as_ref().unwrap()), Some(c"after"));
        assert_eq!(key_binding_cmdlist_ref(bd.as_ref().unwrap()), list);

        key_bindings_add(name, b'a' as key_code, Some(c"later"), 0, None);
        let bd = key_bindings_get(&table.borrow(), b'a' as key_code);
        assert_eq!(key_binding_note(bd.as_ref().unwrap()), Some(c"later"));
        assert_eq!(
            key_binding_flags(bd.as_ref().unwrap()) & KEY_BINDING_REPEAT,
            KEY_BINDING_REPEAT
        );

        let ghost = c"kb-update-empty";
        ts.track(ghost);
        key_bindings_add(ghost, b'z' as key_code, Some(c"ghost"), 0, None);
        let empty = key_bindings_get_table(ghost, 0).unwrap();
        assert!(empty.borrow().bindings().next().is_none());
        assert!(key_bindings_get(&empty.borrow(), b'z' as key_code).is_none());
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
            name,
            flagged,
            Some(c"masked"),
            0,
            parsed_list(c"display-message masked"),
        );

        let bd = key_bindings_get(&table.borrow(), b'a' as key_code);
        assert!(!bd.is_none());
        assert_eq!(key_binding_key(bd.as_ref().unwrap()), b'a' as key_code);
        assert_eq!(key_binding_key(bd.as_ref().unwrap()) & KEYC_MASK_FLAGS, 0);

        assert!(key_bindings_get(&table.borrow(), flagged).is_none());

        let meta = b'b' as key_code | KEYC_META;
        key_bindings_add(name, meta, None, 0, parsed_list(c"display-message meta"));
        let bd = key_bindings_get(&table.borrow(), meta);
        assert!(!bd.is_none());
        assert_eq!(key_binding_key(bd.as_ref().unwrap()), meta);

        key_bindings_remove(name, flagged);
        assert!(key_bindings_get(&table.borrow(), b'a' as key_code).is_none());
    }
}

#[test]
fn borrowed_key_bindings_walk_a_table_in_key_order() {
    let _guard = globals();
    let mut ts = Tables::new();
    let name = c"kb-walk";
    unsafe {
        let table = ts.take(name);
        assert!(binding_keys(&table).is_empty());

        for key in [300 as key_code, 100, 400, 200] {
            key_bindings_add(name, key, None, 0, parsed_list(c"display-message walk"));
        }
        assert_eq!(binding_keys(&table), vec![100 as key_code, 200, 300, 400]);
    }
}

#[test]
fn key_bindings_remove_forgets_a_binding_and_then_the_empty_table() {
    let _guard = globals();
    let mut ts = Tables::new();
    unsafe {
        let missing = c"kb-remove-nowhere";
        key_bindings_remove(missing, b'a' as key_code);
        assert!(key_bindings_get_table(missing, 0).is_none());

        let name = c"kb-remove";
        let table = ts.take(name);
        for key in [b'a' as key_code, b'b' as key_code] {
            key_bindings_add(name, key, None, 0, parsed_list(c"display-message gone"));
        }

        key_bindings_remove(name, b'q' as key_code);
        assert_eq!(
            binding_keys(&table),
            vec![b'a' as key_code, b'b' as key_code]
        );

        key_bindings_remove(name, b'a' as key_code);
        assert_eq!(binding_keys(&table), vec![b'b' as key_code]);
        assert_eq!(key_bindings_get_table(name, 0), Some(table.clone()));

        key_bindings_remove(name, b'b' as key_code);
        assert!(key_bindings_get_table(name, 0).is_none());
    }
}

#[test]
fn key_bindings_reset_without_a_default_removes_the_binding_instead() {
    let _guard = globals();
    let mut ts = Tables::new();
    unsafe {
        let missing = c"kb-reset-nowhere";
        key_bindings_reset(missing, b'a' as key_code);
        assert!(key_bindings_get_table(missing, 0).is_none());

        let name = c"kb-reset";
        let table = ts.take(name);
        for key in [b'a' as key_code, b'b' as key_code] {
            key_bindings_add(name, key, None, 0, parsed_list(c"display-message custom"));
        }

        key_bindings_reset(name, b'q' as key_code);
        assert_eq!(
            binding_keys(&table),
            vec![b'a' as key_code, b'b' as key_code]
        );

        key_bindings_reset(name, b'a' as key_code);
        assert_eq!(binding_keys(&table), vec![b'b' as key_code]);
        assert_eq!(key_bindings_get_table(name, 0), Some(table.clone()));

        key_bindings_reset(name, b'b' as key_code);
        assert!(key_bindings_get_table(name, 0).is_none());
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
            &table,
            b'a' as key_code,
            Some(c"default note"),
            KEY_BINDING_REPEAT,
            original.clone(),
        );
        let dd = key_bindings_get_default(&table.borrow(), b'a' as key_code);
        assert_eq!(key_binding_cmdlist_ref(dd.as_ref().unwrap()), original);
        assert!(key_bindings_get_default(&table.borrow(), b'b' as key_code).is_none());

        key_bindings_add(
            noted,
            b'a' as key_code,
            Some(c"custom"),
            0,
            parsed_list(c"display-message custom"),
        );
        key_bindings_reset(noted, b'a' as key_code);

        let bd = key_bindings_get(&table.borrow(), b'a' as key_code);
        assert_eq!(key_binding_cmdlist_ref(bd.as_ref().unwrap()), original);
        assert_eq!(
            key_binding_note(bd.as_ref().unwrap()),
            Some(c"default note")
        );
        assert_eq!(key_binding_flags(bd.as_ref().unwrap()), KEY_BINDING_REPEAT);

        let plain = c"kb-default-plain";
        let table = ts.take(plain);
        let bare = parsed_list(c"display-message plain");
        install_default(plain, &table, b'x' as key_code, None, 0, bare.clone());
        key_bindings_add(
            plain,
            b'x' as key_code,
            Some(c"temporary"),
            1,
            parsed_list(c"display-message temporary"),
        );
        key_bindings_reset(plain, b'x' as key_code);

        let bd = key_bindings_get(&table.borrow(), b'x' as key_code);
        assert_eq!(key_binding_cmdlist_ref(bd.as_ref().unwrap()), bare);
        assert!(key_binding_note(bd.as_ref().unwrap()).is_none());
        assert_eq!(key_binding_flags(bd.as_ref().unwrap()), 0);
    }
}

#[test]
fn key_table_registry_orders_tables_by_name() {
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

        key_bindings_remove_table(mike);
        let names = table_names();
        let alpha_at = names.iter().position(|n| n == "kb-alpha-table");
        let zulu_at = names.iter().position(|n| n == "kb-zulu-table");
        assert!(alpha_at.is_some() && zulu_at.is_some());
        assert!(alpha_at < zulu_at);
        assert!(!names.iter().any(|n| n == "kb-mike-table"));
        assert!(names.len() >= 2);

        assert!(key_tables.with_borrow(|tables| tables.last_key_value().is_some()));
    }
}

#[test]
fn key_bindings_registry_and_client_walk_are_independent() {
    let _guard = globals();
    let mut ts = Tables::new();
    let name = c"kb-unref";
    unsafe {
        let table_ref = key_bindings_get_table_ref(name, 1).unwrap();
        let table = table_ref.clone();
        ts.track(name);
        drop(table_ref);
        assert_eq!(key_bindings_get_table(name, 0), Some(table.clone()));
        key_bindings_remove_table(name);
        assert!(key_bindings_get_table(name, 0).is_none());
    }
}

#[test]
fn key_bindings_remove_table_rebinds_attached_clients() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut ts = Tables::new();
    unsafe {
        let home = c"kb-client-home";
        let s = Session::new(1, "kb-session");
        s.options()
            .set_string(c"key-table", 0, c"%s", fmt_args![home.as_ptr()]);

        let c = clients.add("kb-client", 80, 24);
        (*c).set_attached_session(Some(s.handle()));
        let home_table_ref = key_bindings_get_table_ref(home, 1).unwrap();
        let home_table = home_table_ref.clone();
        (*c).keytable_ref = Some(home_table_ref.clone());

        let name = c"kb-detach";
        let table = ts.take(name);
        let table_name = table.name().as_c_str().to_owned();
        server_client_set_key_table(&mut *c, Some(&table_name));
        assert_eq!((*c).keytable(), Some(table.clone()));

        key_bindings_remove_table(name);
        assert!(key_bindings_get_table(name, 0).is_none());
        assert_eq!((*c).keytable(), Some(home_table.clone()));

        (*c).keytable_ref = None;
        drop(home_table_ref);
        key_bindings_remove_table(home);
        assert!(key_bindings_get_table(home, 0).is_none());
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
                bare_name,
                key,
                None,
                0,
                parsed_list(c"display-message bare"),
            );
        }
        assert_eq!(binding_keys(&table).len(), 2);
        key_bindings_reset_table(bare_name);
        assert!(key_bindings_get_table(bare_name, 0).is_none());

        let mixed_name = c"kb-reset-table-mixed";
        let table = ts.take(mixed_name);
        let dflt = parsed_list(c"display-message dflt");
        install_default(mixed_name, &table, b'd' as key_code, None, 0, dflt.clone());
        key_bindings_add(
            mixed_name,
            b'c' as key_code,
            None,
            0,
            parsed_list(c"display-message stray"),
        );
        key_bindings_add(
            mixed_name,
            b'd' as key_code,
            None,
            0,
            parsed_list(c"display-message overridden"),
        );

        key_bindings_reset_table(mixed_name);

        assert!(key_bindings_get(&table.borrow(), b'c' as key_code).is_none());
        let bd = key_bindings_get(&table.borrow(), b'd' as key_code);
        assert_eq!(key_binding_cmdlist_ref(bd.as_ref().unwrap()), dflt);
        assert_eq!(key_bindings_get_table(mixed_name, 0), Some(table.clone()));
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
            name,
            b'a' as key_code,
            None,
            0,
            parsed_list(c"display-message plain"),
        );
        key_bindings_add(
            name,
            b'b' as key_code,
            None,
            1,
            parsed_list(c"display-message repeat"),
        );
        let ba = key_bindings_get(&table.borrow(), b'a' as key_code).unwrap();
        let bb = key_bindings_get(&table.borrow(), b'b' as key_code).unwrap();

        let plains = [ba.clone(), ba.clone()];
        assert_eq!(key_bindings_has_repeat(&plains[..0]), 0);
        assert_eq!(key_bindings_has_repeat(&plains[..1]), 0);

        let leading = [bb.clone(), ba.clone()];
        assert_eq!(key_bindings_has_repeat(&leading[..1]), 1);
        let trailing = [ba, bb];
        assert_eq!(key_bindings_has_repeat(&trailing[..1]), 0);
        assert_eq!(key_bindings_has_repeat(&trailing[..2]), 1);
    }
}
