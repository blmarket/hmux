use super::*;
use crate::WindowPane;
use crate::format::format_create;
use crate::options::{OptionsEngine, OptionsRef, RustOptionsEngine};

use crate::style::{COLOUR_FLAG_RGB, ColourEngine, RustColourEngine};
use crate::tests::test_fixtures::zeroed_term;
use crate::tests::test_fixtures::{
    Args, Clients, Options, Pane, Registry, Session, Window, globals, link, unlink,
};
use ::core::ffi::{CStr, c_int, c_longlong};
use ::std::ffi::CString;

pub const FORMAT_NOJOBS: c_int = 0x4;

/// The table entry for a named option.
fn entry_for(name: &CStr) -> &'static options_table_entry_t {
    RustOptionsEngine
        .table()
        .iter()
        .find(|oe| oe.name == name)
        .unwrap_or_else(|| panic!("{name:?} is not an option"))
}

/// A table entry of this module's own, so that the shapes the real option
/// table has none of can be reached. An option set holds a borrowed pointer
/// to the entry behind each of its options and reads it again while being
/// freed, so an entry built here belongs in a static of its own rather than
/// in anything the test owns.
const fn made_up(
    name: &'static CStr,
    type_0: options_table_type,
    flags: c_int,
    default_str: Option<&'static CStr>,
    default_num: c_longlong,
) -> options_table_entry_t {
    options_table_entry_t {
        name,
        alternative_name: None,
        type_0: type_0,
        scope: 0,
        flags,
        minimum: 0,
        maximum: 0,
        choices: None,
        default_str,
        default_num,
        default_arr: None,
        separator: None,
        pattern: None,
        text: None,
        unit: None,
    }
}

#[test]
fn command_defaults_can_read_aliases_from_their_own_store() {
    static COMMAND: options_table_entry_t = made_up(
        c"test-command-default",
        OPTIONS_TABLE_COMMAND,
        0,
        Some(c"test-default-alias"),
        0,
    );
    static COMMAND_ARRAY: options_table_entry_t = options_table_entry_t {
        name: c"test-command-array-default",
        flags: OPTIONS_TABLE_IS_ARRAY,
        default_arr: Some(&[c"test-default-alias", c"test-default-alias"]),
        ..COMMAND
    };
    let _guard = globals();
    unsafe {
        let store = global_options.get().unwrap();
        store.with_entry_mut(c"command-alias", true, |entry| {
            assert_eq!(
                RustOptionsEngine.array_set(
                    entry.unwrap(),
                    1000,
                    Some(c"test-default-alias=display-message initialized"),
                    0,
                    &mut None,
                ),
                0
            );
        });
        store.set_default(&COMMAND);
        store.with_entry(COMMAND.name, true, |entry| {
            assert_eq!(
                RustOptionsEngine.display(entry.unwrap(), -1, 0),
                c"display-message initialized"
            );
        });
        store.set_default(&COMMAND_ARRAY);
        store.with_entry(COMMAND_ARRAY.name, true, |entry| {
            let entry = entry.unwrap();
            assert_eq!(RustOptionsEngine.array_indices(entry), [0, 1]);
            for index in [0, 1] {
                assert_eq!(
                    RustOptionsEngine.display(entry, index, 0),
                    c"display-message initialized"
                );
            }
        });
    }
}

/// The shapes no option in the real table has: an array of numbers, a
/// command whose default is not a command line, and a flag that is on.
#[test]
fn the_shapes_the_option_table_has_none_of() {
    static NUMBERS: options_table_entry_t = made_up(
        c"@numbers",
        OPTIONS_TABLE_NUMBER,
        OPTIONS_TABLE_IS_ARRAY,
        None,
        0,
    );
    static COMMAND: options_table_entry_t = made_up(
        c"@command",
        OPTIONS_TABLE_COMMAND,
        0,
        Some(c"no-such-command"),
        0,
    );
    static FLAG: options_table_entry_t = made_up(c"@flag", OPTIONS_TABLE_FLAG, 0, None, 1);

    let _guard = globals();
    let oo = Options::empty(None);
    unsafe {
        oo.insert_empty(&NUMBERS);
        oo.with_entry_mut(NUMBERS.name, true, |entry| {
            let entry = entry.unwrap();
            let mut cause: Option<CString> = None;
            assert_eq!(
                RustOptionsEngine.array_set(entry, 0, Some(c"1"), 0, &mut cause),
                -1
            );
            assert_eq!(
                cause.as_ref().unwrap().to_string_lossy(),
                "wrong array type"
            );
            assert_eq!(
                RustOptionsEngine.array_set(entry, 0, Some(c"1"), 0, &mut None),
                -1
            );
        });
        oo.set_default(&COMMAND);
        oo.with_entry(COMMAND.name, true, |entry| {
            assert!(entry.unwrap().value.cmdlist().is_none());
        });

        assert_eq!(
            RustOptionsEngine.default_text(&FLAG).to_string_lossy(),
            "on"
        );
        oo.set_default(&FLAG);
        oo.with_entry(FLAG.name, true, |entry| {
            let entry = entry.unwrap();
            assert_eq!(
                RustOptionsEngine.display(entry, -1, 0).to_string_lossy(),
                "on"
            );
            assert_eq!(
                RustOptionsEngine.display(entry, -1, 1).to_string_lossy(),
                "1"
            );
        });
    }
}

/// What an option reads back as.
unsafe fn string_of(oo: &RustOptionsRef, name: &CStr) -> String {
    unsafe {
        oo.with_entry(name, false, |entry| {
            RustOptionsEngine
                .display(entry.expect("option is set"), -1, 0)
                .to_string_lossy()
                .into_owned()
        })
    }
}

/// Sets an option from a string, answering the reason it was turned down.
unsafe fn from_string(
    oo: &RustOptionsRef,
    name: &CStr,
    value: Option<&CStr>,
) -> Result<(), String> {
    unsafe {
        let mut cause = None;
        let oe = if name.to_bytes().first() == Some(&b'@') {
            None
        } else {
            Some(entry_for(name))
        };
        let answer = oo.set_from_string(oe, name, value, 0, &mut cause);
        if answer == 0 {
            assert!(cause.is_none(), "a value that was taken named a cause");
            return Ok(());
        }
        Err(cause.take().unwrap().to_string_lossy().into_owned())
    }
}

/// A great many options put into a set and taken out again in a different
/// order, which is what walks the tree's own rebalancing.
#[test]
fn an_option_set_stays_in_order_however_it_is_filled_and_emptied() {
    let _guard = globals();
    let oo = Options::empty(None);
    let mut seed = 0x2545f491u32;
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        seed
    };
    let mut names: Vec<CString> = (0..400)
        .map(|i| CString::new(format!("@o{:04}", next() % 10000 + i)).expect("no NUL"))
        .collect();
    names.sort();
    names.dedup();
    unsafe {
        for name in &names {
            oo.set_string(name, 0, c"%s", fmt_args![c"v"]);
        }
        let inorder = |oo: &RustOptionsRef| {
            oo.local_names()
                .iter()
                .map(|name| name.to_string_lossy().into_owned())
                .collect::<Vec<_>>()
        };
        let want: Vec<String> = names
            .iter()
            .map(|n| n.to_string_lossy().into_owned())
            .collect();
        assert_eq!(inorder(&*oo), want);

        let mut left = names.clone();
        while !left.is_empty() {
            let at = (next() as usize) % left.len();
            let name = left.remove(at);
            assert!(
                oo.with_entry(&name, true, |entry| entry.is_some()),
                "{name:?} is gone already"
            );
            assert_eq!(
                RustOptionsEngine.remove_or_default(&*oo, &name, -1, &mut None),
                0
            );
            let mut want: Vec<String> = left
                .iter()
                .map(|n| n.to_string_lossy().into_owned())
                .collect();
            want.sort();
            assert_eq!(inorder(&*oo), want);
        }
        assert!(oo.local_names().is_empty());
    }
}

/// The same for the tree an array option keeps its values in.
#[test]
fn an_array_stays_in_order_however_it_is_filled_and_emptied() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_SERVER);
    let mut seed = 0x9e3779b9u32;
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        seed
    };
    let mut indexes: Vec<u_int> = (0..400).map(|i| next() % 10000 + i).collect();
    indexes.sort();
    indexes.dedup();
    unsafe {
        oo.with_entry_mut(c"terminal-overrides", false, |entry| {
            let entry = entry.unwrap();
            RustOptionsEngine.array_clear(entry);
            let mut cause: Option<CString> = None;
            for index in &indexes {
                assert_eq!(
                    RustOptionsEngine.array_set(entry, *index, Some(c"v"), 0, &mut cause),
                    0
                );
            }
            let inorder = |entry: &options_entry| RustOptionsEngine.array_indices(entry);
            assert_eq!(inorder(entry), indexes);

            let mut left = indexes.clone();
            while !left.is_empty() {
                let at = (next() as usize) % left.len();
                let index = left.remove(at);
                assert!(RustOptionsEngine.array_get(entry, index).is_some());
                assert_eq!(
                    RustOptionsEngine.array_set(entry, index, None, 0, &mut cause),
                    0
                );
                let mut want = left.clone();
                want.sort();
                assert_eq!(inorder(entry), want);
            }
            assert!(RustOptionsEngine.array_indices(entry).is_empty());
        });
    }
}

#[test]
fn an_option_set_is_empty_until_something_is_put_in_it() {
    let _guard = globals();
    let oo = Options::empty(None);
    {
        assert!(oo.local_names().is_empty());
        assert!(oo.with_entry(c"status", true, |entry| entry.is_none()));
        assert!(oo.with_entry(c"status", false, |entry| entry.is_none()));
        assert!(oo.parent().is_none());
    }
}

#[test]
fn shared_options_keep_ancestors_alive_and_release_them_with_the_last_child() {
    let _guard = globals();
    let parent = RustOptionsEngine.create(None);
    let weak_parent = Rc::downgrade(&parent.0);
    unsafe {
        parent.set_string(c"@inherited", 0, c"before", &[]);
    }
    let child = RustOptionsEngine.create(Some(&parent));
    let grandchild = RustOptionsEngine.create(Some(&child));
    let weak_child = Rc::downgrade(&child.0);
    unsafe {
        parent.set_string(c"@inherited", 0, c"after", &[]);
    }
    drop(parent);
    drop(child);
    {
        assert_eq!(grandchild.string_ref(c"@inherited").as_ref(), c"after",);
        grandchild.with_entry(c"@inherited", false, |entry| {
            let entry = entry.unwrap();
            assert_eq!(
                RustOptionsEngine.owner(entry),
                RustOptionsRef(weak_parent.upgrade().unwrap())
            );
        });
    }
    assert!(weak_parent.upgrade().is_some());
    assert!(weak_child.upgrade().is_some());
    drop(grandchild);
    assert!(weak_child.upgrade().is_none());
    assert!(weak_parent.upgrade().is_none());
}

#[test]
fn shared_options_reparent_without_changing_local_entries() {
    let _guard = globals();
    let old_parent = RustOptionsEngine.create(None);
    let new_parent = RustOptionsEngine.create(None);
    let child = RustOptionsEngine.create(Some(&old_parent));
    let old_weak = Rc::downgrade(&old_parent.0);
    let new_weak = Rc::downgrade(&new_parent.0);
    unsafe {
        old_parent.set_string(c"@inherited", 0, c"old", &[]);
        new_parent.set_string(c"@inherited", 0, c"new", &[]);
        child.set_string(c"@local", 0, c"local", &[]);
        drop(old_parent);
        child.set_parent(Some(&new_parent));
        assert!(old_weak.upgrade().is_none());
        drop(new_parent);
        assert_eq!(child.string_ref(c"@inherited").as_ref(), c"new");
        assert_eq!(child.string_ref(c"@local").as_ref(), c"local");
        child.set_parent(None);
        assert!(new_weak.upgrade().is_none());
        assert!(child.with_entry(c"@inherited", false, |entry| entry.is_none()));
        assert_eq!(child.string_ref(c"@local").as_ref(), c"local");
    }
}

#[test]
fn shared_options_parent_handles_retain_the_store_independently() {
    let parent = RustOptionsEngine.create(None);
    let weak = Rc::downgrade(&parent.0);
    let child = RustOptionsEngine.create(Some(&parent));
    let retained = { child.parent().unwrap() };
    drop(parent);
    drop(child);
    assert!(weak.upgrade().is_some());
    drop(retained);
    assert!(weak.upgrade().is_none());
}

#[test]
fn shared_options_reject_parent_cycles_without_changing_the_chain() {
    let parent = RustOptionsEngine.create(None);
    let self_cycle = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        parent.set_parent(Some(&parent));
    }));
    assert!(self_cycle.is_err());
    let child = RustOptionsEngine.create(Some(&parent));
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        parent.set_parent(Some(&child));
    }));
    assert!(result.is_err());
    {
        assert!(parent.parent().is_none());
        assert_eq!(child.parent().unwrap(), parent);
    }
}

#[test]
fn shared_options_swapping_handles_preserves_the_store_graph() {
    let mut parent = RustOptionsEngine.create(None);
    let mut child = RustOptionsEngine.create(Some(&parent));
    let weak_parent = Rc::downgrade(&parent.0);
    let weak_child = Rc::downgrade(&child.0);
    std::mem::swap(&mut parent, &mut child);
    {
        assert_eq!(parent.parent(), Some(child.clone()));
        assert!(child.parent().is_none());
    }
    RustOptionsEngine.destroy(child);
    assert!(weak_parent.upgrade().is_some());
    RustOptionsEngine.destroy(parent);
    assert!(weak_parent.upgrade().is_none());
    assert!(weak_child.upgrade().is_none());
}

#[test]
fn shared_options_entry_iteration_and_removal_use_scoped_internal_borrows() {
    let _guard = globals();
    let store = RustOptionsEngine.create(None);
    let shared = store.clone();
    unsafe {
        for name in [c"@c", c"@a", c"@b"] {
            store.set_string(name, 0, c"value", &[]);
        }
        let names = store.local_names();
        for name in &names {
            store.with_entry(name, true, |entry| {
                assert_eq!(RustOptionsEngine.owner(entry.unwrap()), shared);
            });
            assert_eq!(
                RustOptionsEngine.remove_or_default(&shared, name, -1, &mut None),
                0
            );
        }
        assert_eq!(names, [c"@a", c"@b", c"@c"]);
        assert!(shared.local_names().is_empty());
    }
}

#[test]
fn an_option_set_falls_back_on_its_parent() {
    let _guard = globals();
    let parent = Options::defaults(OPTIONS_TABLE_SESSION);
    let child = Options::empty(None);
    {
        child.set_parent(Some(&*parent));
        assert_eq!(child.parent().unwrap(), *parent);
        assert!(child.with_entry(c"status", true, |entry| entry.is_none()));
        child.with_entry(c"status", false, |entry| {
            let entry = entry.unwrap();
            assert_eq!(RustOptionsEngine.owner(entry), *parent);
            assert!(core::ptr::eq(
                RustOptionsEngine.definition(Some(entry)).unwrap(),
                entry_for(c"status"),
            ));
        });
    }
}

#[test]
fn an_option_is_found_under_the_name_it_used_to_have() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_WINDOW);
    {
        assert_eq!(
            options_map_name(c"clock-mode-color").to_string_lossy(),
            "clock-mode-colour"
        );
        assert_eq!(options_map_name(c"status"), c"status");
        oo.with_entry(c"clock-mode-color", true, |entry| {
            assert_eq!(RustOptionsEngine.name(entry.unwrap()), c"clock-mode-colour");
        });
    }
}

#[test]
fn the_entries_of_a_set_come_back_in_name_order() {
    let _guard = globals();
    let oo = Options::empty(None);
    unsafe {
        oo.set_string(c"@b", 0, c"%s", fmt_args![c"2"]);
        oo.set_string(c"@a", 0, c"%s", fmt_args![c"1"]);
        oo.set_string(c"@c", 0, c"%s", fmt_args![c"3"]);
        let names: Vec<_> = oo
            .local_names()
            .iter()
            .map(|name| name.to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, vec!["@a", "@b", "@c"]);
    }
}

#[test]
fn a_user_option_is_a_string_of_whatever_is_put_in_it() {
    let _guard = globals();
    let oo = Options::empty(None);
    unsafe {
        oo.set_string(c"@u", 0, c"%s", fmt_args![c"one"]);
        oo.with_entry(c"@u", true, |entry| {
            let entry = entry.unwrap();
            assert_eq!(RustOptionsEngine.is_string(entry), 1);
            assert_eq!(RustOptionsEngine.is_array(entry), 0);
            assert!(RustOptionsEngine.definition(Some(entry)).is_none());
        });
        assert_eq!(string_of(&*oo, c"@u"), "one");
        oo.set_string(c"@u", 1, c"%s", fmt_args![c"two"]);
        assert_eq!(string_of(&*oo, c"@u"), "onetwo");
        oo.set_string(c"@u", 0, c"%s", fmt_args![c"three"]);
        assert_eq!(string_of(&*oo, c"@u"), "three");
    }
}

#[test]
fn a_string_option_appends_with_the_separator_the_table_names() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_SESSION);
    unsafe {
        oo.set_string(c"status-left", 0, c"%s", fmt_args![c"a"]);
        oo.set_string(c"status-left", 1, c"%s", fmt_args![c"b"]);
        assert_eq!(string_of(&*oo, c"status-left"), "ab");
    }
}

#[test]
fn an_option_the_set_does_not_carry_is_taken_from_the_table_first() {
    let _guard = globals();
    let parent = Options::defaults(OPTIONS_TABLE_SESSION);
    let child = Options::empty(Some(&parent));
    unsafe {
        assert!(child.with_entry(c"status-left", true, |entry| entry.is_none()));
        child.set_string(c"status-left", 0, c"%s", fmt_args![c"x"]);
        assert!(!child.with_entry(c"status-left", true, |entry| entry.is_none()));
        assert_eq!(string_of(&*child, c"status-left"), "x");
        child.set_number(c"status", 0);
        assert_eq!(child.number(c"status"), 0);
    }
}

#[test]
fn every_kind_of_option_reads_back_as_the_text_it_was_given() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_SESSION);
    unsafe {
        assert_eq!(string_of(&*oo, c"status-left"), "[#{session_name}] ");
        assert_eq!(string_of(&*oo, c"history-limit"), "2000");
        assert_eq!(string_of(&*oo, c"prefix"), "C-b");
        assert_eq!(
            string_of(&*oo, c"message-command-style"),
            "bg=black,fg=yellow,fill=black"
        );
        assert_eq!(string_of(&*oo, c"status"), "on");
        assert_eq!(string_of(&*oo, c"status-position"), "bottom");
        oo.with_entry(c"mouse", false, |entry| {
            let entry = entry.unwrap();
            assert_eq!(
                RustOptionsEngine.display(entry, -1, 0).to_string_lossy(),
                "off"
            );
            assert_eq!(
                RustOptionsEngine.display(entry, -1, 1).to_string_lossy(),
                "0"
            );
        });
    }
}

#[test]
fn a_command_option_reads_back_as_the_command_line_it_was_given() {
    let _guard = globals();
    let hook = Options::defaults(OPTIONS_TABLE_SESSION);
    unsafe {
        let mut cause: Option<CString> = None;
        hook.with_entry_mut(c"window-linked", false, |entry| {
            let entry = entry.unwrap();
            assert_eq!(RustOptionsEngine.is_array(entry), 1);
            assert_eq!(RustOptionsEngine.is_string(entry), 0);
            assert_eq!(
                RustOptionsEngine.array_set(entry, 0, Some(c"display-message hi"), 0, &mut cause),
                0
            );
            assert_eq!(
                RustOptionsEngine.display(entry, 0, 0).to_string_lossy(),
                "display-message hi"
            );
            assert_eq!(
                RustOptionsEngine.display(entry, -1, 0).to_string_lossy(),
                "display-message hi"
            );
            assert_eq!(RustOptionsEngine.display(entry, 5, 0).to_string_lossy(), "");
        });
    }
}

/// The default of one option of each kind reads back as the text a user
/// would have written for it.
#[test]
fn the_default_of_each_kind_of_option_reads_back() {
    let _guard = globals();
    {
        for (name, want) in [
            (c"status-left", "[#{session_name}] "),
            (c"history-limit", "2000"),
            (c"prefix", "C-b"),
            (c"clock-mode-colour", "blue"),
            (c"mouse", "off"),
            (c"status-position", "bottom"),
            (c"default-client-command", "new-session"),
        ] {
            assert_eq!(
                RustOptionsEngine
                    .default_text(entry_for(name))
                    .to_string_lossy(),
                want,
                "{name:?}"
            );
        }
    }
}

#[test]
fn an_array_option_takes_values_by_index_and_gives_them_back_in_order() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_SERVER);
    unsafe {
        oo.with_entry_mut(c"terminal-overrides", false, |entry| {
            let entry = entry.unwrap();
            RustOptionsEngine.array_clear(entry);
            assert!(RustOptionsEngine.array_indices(entry).is_empty());
            let mut cause: Option<CString> = None;
            assert_eq!(
                RustOptionsEngine.array_set(entry, 2, Some(c"two"), 0, &mut cause),
                0
            );
            assert_eq!(
                RustOptionsEngine.array_set(entry, 0, Some(c"zero"), 0, &mut cause),
                0
            );
            assert_eq!(
                RustOptionsEngine.array_set(entry, 0, Some(c"!"), 1, &mut cause),
                0
            );

            let mut items = Vec::new();
            for array_index in RustOptionsEngine.array_indices(entry) {
                items.push((
                    array_index,
                    RustOptionsEngine
                        .array_get(entry, array_index)
                        .unwrap()
                        .string()
                        .to_string_lossy()
                        .into_owned(),
                ));
            }
            assert_eq!(
                items,
                vec![(0, "zero!".to_string()), (2, "two".to_string())]
            );
            assert_eq!(
                RustOptionsEngine.array_get(entry, 2).unwrap().string(),
                c"two"
            );
            assert!(RustOptionsEngine.array_get(entry, 1).is_none());
            assert_eq!(
                RustOptionsEngine.array_set(entry, 2, None, 0, &mut cause),
                0
            );
            assert!(RustOptionsEngine.array_get(entry, 2).is_none());
            assert_eq!(
                RustOptionsEngine.display(entry, -1, 0).to_string_lossy(),
                "zero!"
            );
            RustOptionsEngine.array_clear(entry);
            assert_eq!(
                RustOptionsEngine.display(entry, -1, 0).to_string_lossy(),
                ""
            );
        });
    }
}

#[test]
fn an_option_that_is_not_an_array_turns_the_array_calls_down() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_SESSION);
    unsafe {
        oo.with_entry_mut(c"status-left", false, |entry| {
            let entry = entry.unwrap();
            assert!(RustOptionsEngine.array_get(entry, 0).is_none());
            assert!(RustOptionsEngine.array_indices(entry).is_empty());
            RustOptionsEngine.array_clear(entry);
            let mut cause: Option<CString> = None;
            assert_eq!(
                RustOptionsEngine.array_set(entry, 0, Some(c"x"), 0, &mut cause),
                -1
            );
            assert_eq!(cause.as_ref().unwrap().to_string_lossy(), "not an array");
            assert_eq!(
                RustOptionsEngine.array_set(entry, 0, Some(c"x"), 0, &mut None),
                -1
            );
        });
    }
}

#[test]
fn an_array_of_commands_takes_a_command_line_and_turns_down_what_is_not_one() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_SESSION);
    unsafe {
        oo.with_entry_mut(c"window-linked", false, |entry| {
            let entry = entry.unwrap();
            let mut cause: Option<CString> = None;
            assert_eq!(
                RustOptionsEngine.array_set(entry, 0, Some(c"display-message a"), 0, &mut cause),
                0
            );
            assert_eq!(
                RustOptionsEngine.array_set(entry, 0, Some(c"display-message b"), 0, &mut cause),
                0
            );
            assert_eq!(
                RustOptionsEngine.display(entry, 0, 0).to_string_lossy(),
                "display-message b"
            );
            assert_eq!(
                RustOptionsEngine.array_set(entry, 1, Some(c"no-such-command"), 0, &mut cause),
                -1
            );
            assert!(!cause.as_ref().unwrap().as_bytes().is_empty());
            assert_eq!(
                RustOptionsEngine.array_set(entry, 1, Some(c"no-such-command"), 0, &mut None),
                -1
            );
        });
    }
}

#[test]
fn an_array_of_colours_takes_a_colour_name_and_turns_down_what_is_not_one() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_PANE);
    unsafe {
        oo.with_entry_mut(c"pane-colours", false, |entry| {
            let entry = entry.unwrap();
            let mut cause: Option<CString> = None;
            assert_eq!(
                RustOptionsEngine.array_set(entry, 0, Some(c"red"), 0, &mut cause),
                0
            );
            assert_eq!(
                RustOptionsEngine.array_set(entry, 0, Some(c"blue"), 0, &mut cause),
                0
            );
            assert_eq!(RustOptionsEngine.array_get(entry, 0).unwrap().number(), 4);
            assert_eq!(
                RustOptionsEngine.array_set(entry, 1, Some(c"nonsense"), 0, &mut cause),
                -1
            );
            assert_eq!(
                cause.as_ref().unwrap().to_string_lossy(),
                "bad colour: nonsense"
            );
        });
    }
}

#[test]
fn an_array_option_takes_a_whole_list_at_once() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_SERVER);
    unsafe {
        oo.with_entry_mut(c"terminal-overrides", false, |entry| {
            let entry = entry.unwrap();
            RustOptionsEngine.array_clear(entry);
            let mut cause: Option<CString> = None;
            assert_eq!(
                RustOptionsEngine.array_assign(entry, Some(c"a,b c"), &mut cause),
                0
            );
            assert_eq!(
                RustOptionsEngine.display(entry, -1, 0).to_string_lossy(),
                "a b c"
            );
            assert_eq!(
                RustOptionsEngine.array_assign(entry, Some(c""), &mut cause),
                0
            );
            assert_eq!(
                RustOptionsEngine.display(entry, -1, 0).to_string_lossy(),
                "a b c"
            );
            assert_eq!(
                RustOptionsEngine.array_assign(entry, Some(c",,d"), &mut cause),
                0
            );
            assert_eq!(
                RustOptionsEngine.display(entry, -1, 0).to_string_lossy(),
                "a b c d"
            );
        });
    }
}

#[test]
fn an_array_with_no_separator_takes_the_whole_string_as_one() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_SESSION);
    unsafe {
        oo.with_entry_mut(c"window-linked", false, |entry| {
            let entry = entry.unwrap();
            let mut cause: Option<CString> = None;
            assert_eq!(
                RustOptionsEngine.array_assign(entry, Some(c""), &mut cause),
                0
            );
            assert!(RustOptionsEngine.array_indices(entry).is_empty());
            assert_eq!(
                RustOptionsEngine.array_assign(entry, Some(c"display-message ab"), &mut cause),
                0
            );
            assert_eq!(
                RustOptionsEngine.display(entry, -1, 0).to_string_lossy(),
                "display-message ab"
            );
            assert_eq!(
                RustOptionsEngine.array_assign(entry, Some(c"no-such-command"), &mut cause),
                -1
            );
            assert!(!cause.as_ref().unwrap().as_bytes().is_empty());
        });
    }
}

#[test]
fn an_array_list_stops_at_the_first_value_it_cannot_take() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_PANE);
    unsafe {
        oo.with_entry_mut(c"pane-colours", false, |entry| {
            let entry = entry.unwrap();
            let mut cause: Option<CString> = None;
            assert_eq!(
                RustOptionsEngine.array_assign(entry, Some(c"red,nonsense,blue"), &mut cause),
                -1
            );
            assert_eq!(
                cause.as_ref().unwrap().to_string_lossy(),
                "bad colour: nonsense"
            );
            assert_eq!(
                RustOptionsEngine.display(entry, -1, 0).to_string_lossy(),
                "red"
            );
        });
    }
}

#[test]
fn an_option_name_carries_an_index_in_brackets() {
    let _guard = globals();
    let mut idx = 0;
    assert_eq!(
        RustOptionsEngine
            .parse(c"status", &mut idx)
            .unwrap()
            .to_str()
            .unwrap(),
        "status"
    );
    assert_eq!(idx, -1);
    assert_eq!(
        RustOptionsEngine
            .parse(c"a[3]", &mut idx)
            .unwrap()
            .to_str()
            .unwrap(),
        "a"
    );
    assert_eq!(idx, 3);
    assert!(RustOptionsEngine.parse(c"", &mut idx).is_none());
    assert!(RustOptionsEngine.parse(c"a[", &mut idx).is_none());
    assert!(RustOptionsEngine.parse(c"a[]", &mut idx).is_none());
    assert!(RustOptionsEngine.parse(c"a[3]b", &mut idx).is_none());
    assert!(RustOptionsEngine.parse(c"a[-1]", &mut idx).is_none());
    assert!(RustOptionsEngine.parse(c"a[x]", &mut idx).is_none());
}

#[test]
fn an_option_is_looked_up_by_a_name_carrying_an_index() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_SESSION);
    let child = Options::empty(Some(&oo));
    {
        let mut idx = 0;
        let name = RustOptionsEngine
            .parse(c"status-left[2]", &mut idx)
            .unwrap();
        assert!(child.with_entry(&name, false, |entry| entry.is_some()));
        assert_eq!(idx, 2);
        let name = RustOptionsEngine.parse(c"status-left", &mut idx).unwrap();
        assert!(child.with_entry(&name, true, |entry| entry.is_none()));
        assert!(RustOptionsEngine.parse(c"", &mut idx).is_none());
    }
}

#[test]
fn an_option_name_can_be_shortened_until_it_is_ambiguous() {
    let _guard = globals();
    let mut idx = 0;
    let mut ambiguous = 0;
    assert_eq!(
        RustOptionsEngine
            .match_name(c"status-inter", &mut idx, &mut ambiguous)
            .unwrap()
            .to_string_lossy(),
        "status-interval"
    );
    assert_eq!(ambiguous, 0);
    assert!(
        RustOptionsEngine
            .match_name(c"status-l", &mut idx, &mut ambiguous)
            .is_none()
    );
    assert_eq!(ambiguous, 1);
    assert_eq!(
        RustOptionsEngine
            .match_name(c"status", &mut idx, &mut ambiguous)
            .unwrap()
            .to_string_lossy(),
        "status"
    );
    assert!(
        RustOptionsEngine
            .match_name(c"status-", &mut idx, &mut ambiguous)
            .is_none()
    );
    assert_eq!(ambiguous, 1);
    assert!(
        RustOptionsEngine
            .match_name(c"nonsense", &mut idx, &mut ambiguous)
            .is_none()
    );
    assert_eq!(ambiguous, 0);
    assert_eq!(
        RustOptionsEngine
            .match_name(c"@user", &mut idx, &mut ambiguous)
            .unwrap()
            .to_string_lossy(),
        "@user"
    );
    assert!(
        RustOptionsEngine
            .match_name(c"", &mut idx, &mut ambiguous)
            .is_none()
    );
    assert_eq!(
        RustOptionsEngine
            .match_name(c"clock-mode-color", &mut idx, &mut ambiguous)
            .unwrap()
            .to_string_lossy(),
        "clock-mode-colour"
    );
}

#[test]
fn an_option_is_looked_up_by_a_shortened_name() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_SESSION);
    {
        let mut idx = 0;
        let mut ambiguous = 0;
        let name = RustOptionsEngine
            .match_name(c"status-inter", &mut idx, &mut ambiguous)
            .unwrap();
        oo.with_entry(&name, true, |entry| {
            assert_eq!(RustOptionsEngine.name(entry.unwrap()), c"status-interval");
        });
        assert!(
            RustOptionsEngine
                .match_name(c"nonsense", &mut idx, &mut ambiguous)
                .is_none()
        );
        let child = Options::empty(Some(&oo));
        let name = RustOptionsEngine
            .match_name(c"status-inter", &mut idx, &mut ambiguous)
            .unwrap();
        child.with_entry(&name, false, |entry| {
            assert_eq!(RustOptionsEngine.owner(entry.unwrap()), *oo);
        });
    }
}

#[test]
fn a_string_value_is_checked_before_it_is_kept() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_SESSION);
    unsafe {
        assert_eq!(
            from_string(&*oo, c"default-shell", Some(c"/bin/sh")),
            Ok(())
        );
        assert_eq!(
            from_string(&*oo, c"default-shell", Some(c"/nowhere/at/all")),
            Err("not a suitable shell: /nowhere/at/all".to_string())
        );
        assert_eq!(string_of(&*oo, c"default-shell"), "/bin/sh");
    }
}

#[test]
fn a_value_that_does_not_match_the_pattern_is_turned_down() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_SESSION);
    unsafe {
        assert_eq!(from_string(&*oo, c"default-size", Some(c"80x24")), Ok(()));
        assert_eq!(
            from_string(&*oo, c"default-size", Some(c"wide")),
            Err("value is invalid: wide".to_string())
        );
    }
}

#[test]
fn a_style_value_is_parsed_before_it_is_kept() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_SESSION);
    unsafe {
        assert_eq!(from_string(&*oo, c"status-style", Some(c"fg=red")), Ok(()));
        assert_eq!(
            from_string(&*oo, c"status-style", Some(c"nonsense")),
            Err("invalid style: nonsense".to_string())
        );
        assert_eq!(from_string(&*oo, c"status-style", Some(c"#{a}")), Ok(()));
        assert_eq!(string_of(&*oo, c"status-style"), "#{a}");
    }
}

#[test]
fn every_kind_of_option_takes_a_value_from_a_string() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_SESSION);
    unsafe {
        assert_eq!(from_string(&*oo, c"history-limit", Some(c"100")), Ok(()));
        assert_eq!(string_of(&*oo, c"history-limit"), "100");
        assert_eq!(
            from_string(&*oo, c"history-limit", Some(c"nonsense")),
            Err("value is invalid: nonsense".to_string())
        );
        assert_eq!(from_string(&*oo, c"prefix", Some(c"C-a")), Ok(()));
        assert_eq!(string_of(&*oo, c"prefix"), "C-a");
        assert_eq!(
            from_string(&*oo, c"prefix", Some(c"nonsense")),
            Err("bad key: nonsense".to_string())
        );
        assert_eq!(
            from_string(&*oo, c"message-command-style", Some(c"fg=red")),
            Ok(())
        );

        let pane = Options::defaults(OPTIONS_TABLE_PANE);
        assert_eq!(
            from_string(&*pane, c"scroll-on-clear", Some(c"off")),
            Ok(())
        );
        assert_eq!(string_of(&*pane, c"scroll-on-clear"), "off");
    }
}

#[test]
fn a_colour_option_takes_a_colour_name() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_SERVER);
    unsafe {
        assert_eq!(from_string(&*oo, c"copy-command", Some(c"")), Ok(()));
        let window = Options::defaults(OPTIONS_TABLE_WINDOW);
        assert_eq!(
            from_string(&*window, c"clock-mode-colour", Some(c"red")),
            Ok(())
        );
        assert_eq!(string_of(&*window, c"clock-mode-colour"), "red");
        assert_eq!(
            from_string(&*window, c"clock-mode-colour", Some(c"nonsense")),
            Err("bad colour: nonsense".to_string())
        );
    }
}

#[test]
fn a_flag_option_takes_the_words_for_on_and_off_and_turns_over_with_none() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_SESSION);
    unsafe {
        for word in [c"on", c"yes", c"1"] {
            assert_eq!(from_string(&*oo, c"status-keys", Some(c"vi")), Ok(()));
            assert_eq!(from_string(&*oo, c"mouse", Some(word)), Ok(()));
            assert_eq!(oo.number(c"mouse"), 1);
        }
        for word in [c"off", c"no", c"0"] {
            assert_eq!(from_string(&*oo, c"mouse", Some(word)), Ok(()));
            assert_eq!(oo.number(c"mouse"), 0);
        }
        assert_eq!(from_string(&*oo, c"mouse", None), Ok(()));
        assert_eq!(oo.number(c"mouse"), 1);
        assert_eq!(from_string(&*oo, c"mouse", Some(c"")), Ok(()));
        assert_eq!(oo.number(c"mouse"), 0);
        assert_eq!(
            from_string(&*oo, c"mouse", Some(c"maybe")),
            Err("bad value: maybe".to_string())
        );
    }
}

#[test]
fn a_choice_option_takes_one_of_its_choices_and_turns_over_with_none() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_SESSION);
    unsafe {
        assert_eq!(from_string(&*oo, c"status-position", Some(c"top")), Ok(()));
        assert_eq!(oo.number(c"status-position"), 0);
        assert_eq!(
            from_string(&*oo, c"status-position", Some(c"middle")),
            Err("unknown value: middle".to_string())
        );
        assert_eq!(from_string(&*oo, c"status-position", None), Ok(()));
        assert_eq!(oo.number(c"status-position"), 1);
        assert_eq!(from_string(&*oo, c"status", Some(c"3")), Ok(()));
        assert_eq!(from_string(&*oo, c"status", None), Ok(()));
        assert_eq!(oo.number(c"status"), 3);

        let mut cause = None;
        assert_eq!(
            RustOptionsEngine.find_choice(entry_for(c"status-position"), c"top", &mut cause,),
            0
        );
        assert!(cause.is_none());
    }
}

#[test]
fn a_command_option_takes_a_command_line_and_turns_down_what_is_not_one() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_SERVER);
    unsafe {
        assert_eq!(
            from_string(
                &*oo,
                c"default-client-command",
                Some(c"display-message hello")
            ),
            Ok(())
        );
        assert_eq!(
            string_of(&*oo, c"default-client-command"),
            "display-message hello"
        );
        assert!(oo.command(c"default-client-command").is_some());
        assert!(
            !from_string(&*oo, c"default-client-command", Some(c"no-such-command"))
                .unwrap_err()
                .is_empty()
        );
        oo.set_command(c"default-client-command", None);
        assert!(oo.command(c"default-client-command").is_none());
    }
}

#[test]
fn an_option_with_no_value_and_a_bad_name_is_turned_down() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_SESSION);
    unsafe {
        assert_eq!(
            from_string(&*oo, c"status-left", None),
            Err("empty value".to_string())
        );
        let mut cause = None;
        assert_eq!(
            oo.set_from_string(None, c"nonsense", Some(c"x"), 0, &mut cause),
            -1
        );
        assert_eq!(cause.take().unwrap().to_string_lossy(), "bad option name");
        oo.set_string(c"@user", 0, c"%s", fmt_args![c"old"]);
        assert_eq!(from_string(&*oo, c"@user", Some(c"x")), Ok(()));
        assert_eq!(string_of(&*oo, c"@user"), "x");
    }
}

#[test]
fn a_style_option_is_read_once_and_kept_unless_it_is_a_format() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_SESSION);
    unsafe {
        let mut ft = format_create(None, None, 0, FORMAT_NOJOBS);
        let ft_ptr = &raw mut *ft;
        oo.set_string(c"status-style", 0, c"%s", fmt_args![c"fg=red"]);
        let sy = oo.style_value(c"status-style", Some(&mut *ft_ptr)).unwrap();
        assert_eq!(sy.gc.fg, 1);
        assert_eq!(
            oo.style_value(c"status-style", Some(&mut *ft_ptr))
                .unwrap()
                .gc
                .fg,
            sy.gc.fg
        );

        oo.set_string(c"status-style", 0, c"%s", fmt_args![c"nonsense"]);
        assert!(
            oo.style_value(c"status-style", Some(&mut *ft_ptr))
                .is_none()
        );

        oo.set_string(
            c"status-style",
            0,
            c"%s",
            fmt_args![c"#{?1,fg=red,fg=blue}"],
        );
        assert!(
            oo.style_value(c"status-style", Some(&mut *ft_ptr))
                .is_some()
        );
        oo.set_string(
            c"status-style",
            0,
            c"%s",
            fmt_args![c"#{?1,nonsense,nonsense}"],
        );
        assert!(
            oo.style_value(c"status-style", Some(&mut *ft_ptr))
                .is_none()
        );
        assert_eq!(sy.gc.fg, 1);
        oo.set_string(c"status-style", 0, c"%s", fmt_args![c"#{a}"]);
        assert!(oo.style_value(c"status-style", None).is_none());

        assert!(oo.style_value(c"status", Some(&mut *ft_ptr)).is_none());
        assert!(oo.style_value(c"nonsense", Some(&mut *ft_ptr)).is_none());
    }
}

#[test]
fn style_expansion_survives_replacement_of_its_option() {
    unsafe fn replace_style(_: &format_tree) -> Option<CString> {
        unsafe {
            let store = global_s_options.get().unwrap();
            store.set_default(entry_for(c"status-style"));
            store.set_string(c"status-style", 0, c"fg=blue", &[]);
        }
        Some(c"fg=red".to_owned())
    }
    let _guard = globals();
    unsafe {
        let store = global_s_options.get().unwrap();
        store.set_string(c"status-style", 0, c"#{replace_style}", &[]);
        let mut format = format_create(None, None, 0, FORMAT_NOJOBS);
        crate::format::format_add_cb(&mut format, c"replace_style", Some(replace_style));
        assert_eq!(
            store
                .style_value(c"status-style", Some(&mut format))
                .unwrap()
                .gc
                .fg,
            1
        );
        assert_eq!(store.string_ref(c"status-style").as_ref(), c"fg=blue");
        assert_eq!(store.style_value(c"status-style", None).unwrap().gc.fg, 4);
    }
}

#[test]
fn an_option_is_taken_away_or_put_back_to_its_default() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_SESSION);
    unsafe {
        oo.set_string(c"status-left", 0, c"%s", fmt_args![c"x"]);
        assert_eq!(
            RustOptionsEngine.remove_or_default(&*oo, c"status-left", -1, &mut None),
            0
        );
        assert!(oo.with_entry(c"status-left", true, |entry| entry.is_none()));

        let server = Options::defaults(OPTIONS_TABLE_SERVER);
        let mut cause: Option<CString> = None;
        server.with_entry_mut(c"terminal-overrides", false, |entry| {
            RustOptionsEngine.array_set(entry.unwrap(), 0, Some(c"x"), 0, &mut cause);
        });
        assert_eq!(
            RustOptionsEngine.remove_or_default(&*server, c"terminal-overrides", 0, &mut cause),
            0
        );
        server.with_entry(c"terminal-overrides", false, |entry| {
            assert!(RustOptionsEngine.array_get(entry.unwrap(), 0).is_none());
        });

        assert_eq!(
            RustOptionsEngine.remove_or_default(&*server, c"copy-command", 0, &mut cause),
            -1
        );
        assert_eq!(cause.as_ref().unwrap().to_string_lossy(), "not an array");
    }
}

#[test]
fn a_global_option_taken_away_goes_back_to_its_default() {
    let _guard = globals();
    unsafe {
        (global_s_options
            .get()
            .as_ref()
            .expect("global options are initialized"))
        .set_string(c"status-left", 0, c"%s", fmt_args![c"changed"]);
        assert_eq!(
            RustOptionsEngine.remove_or_default(
                global_s_options.get().as_ref().unwrap(),
                c"status-left",
                -1,
                &mut None
            ),
            0
        );
        assert_eq!(
            string_of(global_s_options.get().as_ref().unwrap(), c"status-left"),
            "[#{session_name}] "
        );
    }
}

#[test]
fn the_scope_of_an_option_is_worked_out_from_its_name() {
    let _guard = globals();
    let mut session = Session::new(1, "scope");
    let mut window = Window::new(1, "scope", 10, 2);
    let mut pane = Pane::new(1, 10, 2, 0);
    window.add_pane(&mut pane);
    let wl = link(&mut session, &mut window, 0);
    let args = Args::parse(c"set-option x");
    let mut fs = Box::new(cmd_find_state::default());
    unsafe { fs.set_session(session.ptr().as_ref()) };
    unsafe { fs.set_winlink(wl.as_ref()) };
    fs.set_window_ref(Some(window.handle()));
    unsafe { fs.set_pane(pane.ptr().as_ref()) };
    unsafe {
        let mut oo: Option<RustOptionsRef> = None;
        let mut cause = None;
        let scope = |name: &CStr, oo: &mut Option<RustOptionsRef>, cause: &mut Option<CString>| {
            RustOptionsEngine.scope_from_name(&args.borrow(), 0, name, &*fs, oo, cause)
        };
        assert_eq!(
            scope(c"copy-command", &mut oo, &mut cause),
            OPTIONS_TABLE_SERVER
        );
        assert_eq!(oo, global_options.get());
        assert_eq!(
            scope(c"status-left", &mut oo, &mut cause),
            OPTIONS_TABLE_SESSION
        );
        assert_eq!(oo, Some(session.options()));
        assert_eq!(
            scope(c"mode-keys", &mut oo, &mut cause),
            OPTIONS_TABLE_WINDOW
        );
        assert_eq!(oo, Some(window.options()));
        assert_eq!(scope(c"nonsense", &mut oo, &mut cause), OPTIONS_TABLE_NONE);
        assert_eq!(
            cause.take().unwrap().to_str().unwrap(),
            "unknown option: nonsense"
        );
    }
    unlink(&mut session, wl);
}

#[test]
fn the_scope_of_an_option_with_no_target_is_named_in_the_reason() {
    let _guard = globals();
    let args = Args::parse(c"set-option x");
    let targeted = Args::parse(c"set-option -t other x");
    let fs = Box::new(cmd_find_state::default());
    unsafe {
        let mut oo: Option<RustOptionsRef> = None;
        let mut cause = None;
        for (args, want) in [
            (&args, "no current session"),
            (&targeted, "no such session: other"),
        ] {
            assert_eq!(
                RustOptionsEngine.scope_from_name(
                    &args.borrow(),
                    0,
                    c"status-left",
                    &*fs,
                    &mut oo,
                    &mut cause
                ),
                OPTIONS_TABLE_NONE
            );
            assert_eq!(cause.take().unwrap().to_str().unwrap(), want);
        }
        for (args, want) in [
            (&args, "no current window"),
            (&targeted, "no such window: other"),
        ] {
            assert_eq!(
                RustOptionsEngine.scope_from_name(
                    &args.borrow(),
                    0,
                    c"mode-keys",
                    &*fs,
                    &mut oo,
                    &mut cause
                ),
                OPTIONS_TABLE_NONE
            );
            assert_eq!(cause.take().unwrap().to_str().unwrap(), want);
        }
    }
}

#[test]
fn the_scope_of_a_pane_option_follows_the_pane_flag() {
    let _guard = globals();
    let mut window = Window::new(1, "paneflag", 10, 2);
    let mut pane = Pane::new(1, 10, 2, 0);
    window.add_pane(&mut pane);
    let args = Args::parse(c"set-option -p x");
    let bare = Args::parse(c"set-option x");
    let targeted = Args::parse(c"set-option -p -t other x");
    let mut fs = Box::new(cmd_find_state::default());
    fs.set_window_ref(Some(window.handle()));
    unsafe {
        let mut oo: Option<RustOptionsRef> = None;
        let mut cause = None;
        assert_eq!(
            RustOptionsEngine.scope_from_name(
                &args.borrow(),
                0,
                c"remain-on-exit",
                &*fs,
                &mut oo,
                &mut cause
            ),
            OPTIONS_TABLE_NONE
        );
        assert_eq!(cause.take().unwrap().to_str().unwrap(), "no current pane");
        assert_eq!(
            RustOptionsEngine.scope_from_name(
                &targeted.borrow(),
                0,
                c"remain-on-exit",
                &*fs,
                &mut oo,
                &mut cause
            ),
            OPTIONS_TABLE_NONE
        );
        assert_eq!(
            cause.take().unwrap().to_str().unwrap(),
            "no such pane: other"
        );
        fs.set_pane(pane.ptr().as_ref());
        assert_eq!(
            RustOptionsEngine.scope_from_name(
                &args.borrow(),
                0,
                c"remain-on-exit",
                &*fs,
                &mut oo,
                &mut cause
            ),
            OPTIONS_TABLE_PANE
        );
        assert_eq!(oo, Some(pane.options()));
        assert_eq!(
            RustOptionsEngine.scope_from_name(
                &bare.borrow(),
                0,
                c"remain-on-exit",
                &*fs,
                &mut oo,
                &mut cause
            ),
            OPTIONS_TABLE_NONE
        );
        assert_eq!(cause.take().unwrap().to_str().unwrap(), "no current window");
    }
}

#[test]
fn a_global_flag_names_the_global_option_set() {
    let _guard = globals();
    let args = Args::parse(c"set-option -g x");
    let fs = Box::new(cmd_find_state::default());
    unsafe {
        let mut oo: Option<RustOptionsRef> = None;
        let mut cause = None;
        assert_eq!(
            RustOptionsEngine.scope_from_name(
                &args.borrow(),
                0,
                c"status-left",
                &*fs,
                &mut oo,
                &mut cause
            ),
            OPTIONS_TABLE_SESSION
        );
        assert_eq!(oo, global_s_options.get());
        assert_eq!(
            RustOptionsEngine.scope_from_name(
                &args.borrow(),
                0,
                c"mode-keys",
                &*fs,
                &mut oo,
                &mut cause
            ),
            OPTIONS_TABLE_WINDOW
        );
        assert_eq!(oo, global_w_options.get());
    }
}

#[test]
fn the_scope_of_a_user_option_is_worked_out_from_the_flags() {
    let _guard = globals();
    let mut session = Session::new(1, "flags");
    let mut window = Window::new(1, "flags", 10, 2);
    let mut pane = Pane::new(1, 10, 2, 0);
    window.add_pane(&mut pane);
    let wl = link(&mut session, &mut window, 0);
    let mut fs = Box::new(cmd_find_state::default());
    unsafe { fs.set_session(session.ptr().as_ref()) };
    unsafe { fs.set_winlink(wl.as_ref()) };
    fs.set_window_ref(Some(window.handle()));
    unsafe { fs.set_pane(pane.ptr().as_ref()) };
    unsafe {
        let mut oo: Option<RustOptionsRef> = None;
        let mut cause = None;
        for (line, window_flag, want, set) in [
            (
                c"set-option -s @u",
                0,
                OPTIONS_TABLE_SERVER,
                global_options.get(),
            ),
            (
                c"set-option -p @u",
                0,
                OPTIONS_TABLE_PANE,
                Some(pane.options()),
            ),
            (
                c"set-option -w @u",
                0,
                OPTIONS_TABLE_WINDOW,
                Some(window.options()),
            ),
            (
                c"set-option @u",
                1,
                OPTIONS_TABLE_WINDOW,
                Some(window.options()),
            ),
            (
                c"set-option -wg @u",
                0,
                OPTIONS_TABLE_WINDOW,
                global_w_options.get(),
            ),
            (
                c"set-option @u",
                0,
                OPTIONS_TABLE_SESSION,
                Some(session.options()),
            ),
            (
                c"set-option -g @u",
                0,
                OPTIONS_TABLE_SESSION,
                global_s_options.get(),
            ),
        ] {
            let args = Args::parse(line);
            assert_eq!(
                RustOptionsEngine.scope_from_name(
                    &args.borrow(),
                    window_flag,
                    c"@u",
                    &*fs,
                    &mut oo,
                    &mut cause
                ),
                want,
                "{line:?}"
            );
            assert_eq!(oo, set, "{line:?}");
        }
    }
    unlink(&mut session, wl);
}

#[test]
fn a_user_option_with_no_target_is_turned_down() {
    let _guard = globals();
    let fs = Box::new(cmd_find_state::default());
    unsafe {
        let mut oo: Option<RustOptionsRef> = None;
        let mut cause = None;
        for (line, window_flag, want) in [
            (c"set-option -p @u", 0, "no current pane"),
            (c"set-option -p -t x @u", 0, "no such pane: x"),
            (c"set-option -w @u", 0, "no current window"),
            (c"set-option -w -t x @u", 0, "no such window: x"),
            (c"set-option @u", 0, "no current session"),
            (c"set-option -t x @u", 0, "no such session: x"),
        ] {
            let args = Args::parse(line);
            assert_eq!(
                RustOptionsEngine.scope_from_flags(
                    &args.borrow(),
                    window_flag,
                    &*fs,
                    &mut oo,
                    &mut cause
                ),
                OPTIONS_TABLE_NONE,
                "{line:?}"
            );
            assert_eq!(cause.take().unwrap().to_str().unwrap(), want, "{line:?}");
        }
    }
}

#[test]
fn a_change_is_pushed_out_to_the_windows_panes_and_clients_it_reaches() {
    let _guard = globals();
    let mut registry = Registry::new();
    let mut session = Session::new(1, "push");
    let mut window = Window::new(1, "push", 10, 2);
    let mut pane = Pane::new(1, 10, 2, 0);
    window.add_pane(&mut pane);
    let wl = link(&mut session, &mut window, 0);
    registry.add_session(&mut session);
    registry.add_window(&mut window);
    let mut list = Clients::new();
    let client = list.add("push", 10, 2);
    unsafe {
        window.options().set_number(c"automatic-rename", 1);
        session
            .options()
            .set_parent(global_s_options.get().as_ref());
        window.options().set_parent(global_w_options.get().as_ref());
        pane.options().set_parent(Some(&window.options()));
        (*client).set_attached_session(Some(session.handle()));
        (*client).tty.term = Some(zeroed_term());
        (*client).tty.flags = TTY_OPENED;
        let table_ref = crate::key_bindings::key_bindings_get_table_ref(c"root", 1).unwrap();
        (*client).keytable = Some(table_ref);
        for name in [
            c"automatic-rename",
            c"cursor-colour",
            c"cursor-style",
            c"fill-character",
            c"key-table",
            c"user-keys",
            c"monitor-silence",
            c"window-style",
            c"window-active-style",
            c"@user",
            c"pane-colours",
            c"pane-border-status",
            c"pane-scrollbars",
            c"pane-scrollbars-position",
            c"pane-scrollbars-style",
            c"codepoint-widths",
            c"input-buffer-size",
            c"history-limit",
            c"nothing-in-particular",
        ] {
            RustOptionsEngine.push_changes(name);
        }
        assert_ne!(*(*pane.ptr()).flags() & PANE_STYLECHANGED, 0);
    }
    unlink(&mut session, wl);
}

/// The status timer is armed for every client, which needs a client with
/// no session behind it so that nothing is redrawn.
#[test]
fn a_status_change_starts_the_timers_again() {
    let _guard = globals();
    let list = Clients::new();
    unsafe {
        RustOptionsEngine.push_changes(c"status");
        RustOptionsEngine.push_changes(c"status-interval");
    }
    drop(list);
}

/// A standalone set holding just `pane-colours`, taken straight from the
/// options table so it has the right table entry.
fn pane_colours_options(values: &[(u_int, &CStr)]) -> Options {
    let oo = Options::empty(None);
    unsafe {
        oo.set_default(entry_for(c"pane-colours"));
        oo.with_entry_mut(c"pane-colours", true, |entry| {
            let entry = entry.unwrap();
            for &(n, value) in values {
                RustOptionsEngine.array_set(entry, n, Some(&value), 0, &mut None);
            }
        });
    }
    oo
}

#[test]
fn an_empty_pane_colours_option_reads_back_as_no_defaults() {
    let _guard = globals();
    let oo = pane_colours_options(&[]);
    {
        assert!(oo.pane_colours().is_none());
        let mut p = colour_palette {
            fg: 0,
            bg: 0,
            palette: None,
            default_palette: Some(std::sync::Arc::new([-1; 256])),
        };
        oo.load_pane_colours(Some(&mut p));
        assert!(p.default_palette.is_none());
    }
}

#[test]
fn pane_colours_read_back_by_index_and_load_into_a_palette() {
    let _guard = globals();
    let oo = pane_colours_options(&[(0, c"red"), (1, c"#00ff00"), (300, c"blue")]);
    {
        let def = oo.pane_colours().expect("the option holds entries");
        assert_eq!(def[0], 1);
        assert_eq!(def[1], 0x00ff00 | COLOUR_FLAG_RGB);
        assert_eq!(def[2], -1);

        let mut p = colour_palette {
            fg: 0,
            bg: 0,
            palette: None,
            default_palette: None,
        };
        oo.load_pane_colours(Some(&mut p));
        assert_eq!(RustColourEngine.get_palette(Some(&p), 0), 1);
        assert_eq!(
            RustColourEngine.get_palette(Some(&p), 1),
            0x00ff00 | COLOUR_FLAG_RGB
        );
        assert_eq!(RustColourEngine.get_palette(Some(&p), 2), -1);
        RustColourEngine.free_palette(Some(&mut p));
    }
}

/// Every spec the `codepoint-widths` option holds is handed over in array
/// order, and an option with none hands over nothing.
#[test]
fn codepoint_widths_are_read_back_in_array_order() {
    let _guard = globals();
    let oo = Options::empty(None);
    unsafe {
        oo.set_default(entry_for(c"codepoint-widths"));
        assert!(oo.codepoint_widths().is_empty());
        oo.with_entry_mut(c"codepoint-widths", true, |entry| {
            let entry = entry.unwrap();
            RustOptionsEngine.array_set(entry, 1, Some(c"U+E9=2"), 0, &mut None);
            RustOptionsEngine.array_set(entry, 0, Some(c"U+41=1"), 0, &mut None);
        });
        let specs = oo.codepoint_widths();
        assert_eq!(specs, vec![c"U+41=1".to_owned(), c"U+E9=2".to_owned()]);
    }
}

#[test]
fn pane_option_scopes_retain_physical_pane_options_and_reject_removed_targets() {
    let _guard = globals();
    let mut target = crate::tests::test_fixtures::Target::new(20, 6);
    let state = target.state();
    let args = Args::parse(c"set-option -p x");
    unsafe {
        let window = state.window().unwrap();
        let pane = crate::window::window_panes_take(
            &mut window.as_window_mut(),
            &crate::window::window_pane_find_by_id(
                state.wp.as_ref().map(|pane| pane.id()).unwrap(),
            )
            .expect("the pane exists"),
        )
        .unwrap();
        let observed = pane.downgrade();
        window.as_window_mut().panes.push(pane);
        let pane = observed;
        assert!(state.pane_ref().is_some());
        let options = pane.get().unwrap().options_ref().clone();
        options.set_number(c"remain-on-exit", 1);
        let mut selected = None;
        let mut cause = None;
        assert_eq!(
            RustOptionsEngine.scope_from_name(
                &args.borrow(),
                0,
                c"remain-on-exit",
                &state,
                &mut selected,
                &mut cause
            ),
            OPTIONS_TABLE_PANE
        );
        assert!(selected.as_ref().unwrap().ptr_eq(&options));
        assert_eq!(
            RustOptionsEngine.scope_from_flags(
                &args.borrow(),
                0,
                &state,
                &mut selected,
                &mut cause
            ),
            OPTIONS_TABLE_PANE
        );
        assert!(selected.as_ref().unwrap().ptr_eq(&options));
        window.remove_pane(
            &crate::window::window_pane_find_by_id(pane.id()).expect("the pane exists"),
        );
        assert!(pane.get().is_none());
        assert_eq!(
            RustOptionsEngine.scope_from_name(
                &args.borrow(),
                0,
                c"remain-on-exit",
                &state,
                &mut selected,
                &mut cause
            ),
            OPTIONS_TABLE_NONE
        );
        assert_eq!(cause.take().unwrap(), c"no current pane");
        assert_eq!(
            RustOptionsEngine.scope_from_flags(
                &args.borrow(),
                0,
                &state,
                &mut selected,
                &mut cause
            ),
            OPTIONS_TABLE_NONE
        );
        assert_eq!(cause.take().unwrap(), c"no current pane");
        assert!(selected.as_ref().unwrap().ptr_eq(&options));
        assert_eq!(selected.as_ref().unwrap().number(c"remain-on-exit"), 1);
        drop(target);
    }
}
