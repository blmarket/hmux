use super::*;
use crate::format::format_create;
use crate::options::{options_get_only_ptr, options_get_ptr};
use crate::style::{COLOUR_FLAG_RGB, colour_palette_free, colour_palette_get};
use crate::tests::test_fixtures::zeroed_term;
use crate::tests::test_fixtures::{
    Args, Clients, Options, Pane, Registry, Session, Window, globals, link, unlink,
};
use ::core::ffi::{CStr, c_int, c_longlong};
use ::core::ptr::{null, null_mut};
use ::std::ffi::CString;

pub const FORMAT_NOJOBS: c_int = 0x4;

/// The table entry for a named option.
fn entry_for(name: &CStr) -> &'static options_table_entry_t {
    options_table
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
        type_0,
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
    let oo = Options::empty(null_mut());
    unsafe {
        let o = options_empty(oo.ptr(), &NUMBERS);
        let mut cause: Option<CString> = None;
        assert_eq!(options_array_set(o, 0, c"1".as_ptr(), 0, &mut cause), -1);
        assert_eq!(
            cause.as_ref().unwrap().to_string_lossy(),
            "wrong array type"
        );
        assert_eq!(options_array_set(o, 0, c"1".as_ptr(), 0, &mut None), -1);

        let o = options_default(oo.ptr(), &COMMAND);
        assert!((*o).value.cmdlist().is_none());

        assert_eq!(options_default_to_string(&FLAG).to_string_lossy(), "on");
        let o = options_default(oo.ptr(), &FLAG);
        assert_eq!(options_to_string(o, -1, 0).to_string_lossy(), "on");
        assert_eq!(options_to_string(o, -1, 1).to_string_lossy(), "1");
    }
}

/// What an option reads back as.
unsafe fn string_of(oo: *mut options, name: &CStr) -> String {
    unsafe {
        let o = options_get_ptr(oo, name.as_ptr());
        assert!(!o.is_null(), "{name:?} is not set");
        options_to_string(o, -1, 0).to_string_lossy().into_owned()
    }
}

/// Sets an option from a string, answering the reason it was turned down.
unsafe fn from_string(oo: *mut options, name: &CStr, value: Option<&CStr>) -> Result<(), String> {
    unsafe {
        let mut cause = None;
        let oe = if name.to_bytes().first() == Some(&b'@') {
            None
        } else {
            Some(entry_for(name))
        };
        let answer = options_from_string(
            oo,
            oe,
            name.as_ptr(),
            value.map_or(null(), |v| v.as_ptr()),
            0,
            &mut cause,
        );
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
    let oo = Options::empty(null_mut());
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
            options_set_string(
                oo.ptr(),
                name.as_ptr(),
                0,
                c"%s".as_ptr(),
                fmt_args![c"v".as_ptr()],
            );
        }
        let inorder = |oo: *mut options| {
            let mut out = Vec::new();
            let mut o = options_first(oo);
            while !o.is_null() {
                out.push(options_name(o).to_string_lossy().into_owned());
                o = options_next(o);
            }
            out
        };
        let want: Vec<String> = names
            .iter()
            .map(|n| n.to_string_lossy().into_owned())
            .collect();
        assert_eq!(inorder(oo.ptr()), want);

        let mut left = names.clone();
        while !left.is_empty() {
            let at = (next() as usize) % left.len();
            let name = left.remove(at);
            let o = options_get_only_ptr(oo.ptr(), name.as_ptr());
            assert!(!o.is_null(), "{name:?} is gone already");
            assert_eq!(options_remove_or_default(o, -1, &mut None), 0);
            let mut want: Vec<String> = left
                .iter()
                .map(|n| n.to_string_lossy().into_owned())
                .collect();
            want.sort();
            assert_eq!(inorder(oo.ptr()), want);
        }
        assert!(options_first(oo.ptr()).is_null());
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
        let o = options_get_ptr(oo.ptr(), c"terminal-overrides".as_ptr());
        options_array_clear(o);
        let mut cause: Option<CString> = None;
        for index in &indexes {
            assert_eq!(
                options_array_set(o, *index, c"v".as_ptr(), 0, &mut cause),
                0
            );
        }
        let inorder = |o: *mut options_entry| {
            let mut out = Vec::new();
            let mut a = options_array_first(o);
            while !a.is_null() {
                out.push(options_array_item_index(a));
                a = options_array_next(o, a);
            }
            out
        };
        assert_eq!(inorder(o), indexes);

        let mut left = indexes.clone();
        while !left.is_empty() {
            let at = (next() as usize) % left.len();
            let index = left.remove(at);
            assert!(!options_array_get(o, index).is_null());
            assert_eq!(options_array_set(o, index, null(), 0, &mut cause), 0);
            let mut want = left.clone();
            want.sort();
            assert_eq!(inorder(o), want);
        }
        assert!(options_array_first(o).is_null());
    }
}

#[test]
fn an_option_set_is_empty_until_something_is_put_in_it() {
    let _guard = globals();
    let oo = Options::empty(null_mut());
    unsafe {
        assert!(options_first(oo.ptr()).is_null());
        assert!(options_get_only_ptr(oo.ptr(), c"status".as_ptr()).is_null());
        assert!(options_get_ptr(oo.ptr(), c"status".as_ptr()).is_null());
        assert!(options_get_parent(oo.ptr()).is_null());
    }
}

#[test]
fn an_option_set_falls_back_on_its_parent() {
    let _guard = globals();
    let parent = Options::defaults(OPTIONS_TABLE_SESSION);
    let child = Options::empty(null_mut());
    unsafe {
        options_set_parent(child.ptr(), parent.ptr());
        assert_eq!(options_get_parent(child.ptr()), parent.ptr());
        assert!(options_get_only_ptr(child.ptr(), c"status".as_ptr()).is_null());
        let o = options_get_ptr(child.ptr(), c"status".as_ptr());
        assert!(!o.is_null());
        assert_eq!(options_owner(o), parent.ptr());
        assert!(::core::ptr::eq(
            options_table_entry(o).unwrap(),
            entry_for(c"status"),
        ));
    }
}

#[test]
fn an_option_is_found_under_the_name_it_used_to_have() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_WINDOW);
    unsafe {
        assert_eq!(
            options_map_name(c"clock-mode-color").to_string_lossy(),
            "clock-mode-colour"
        );
        assert_eq!(options_map_name(c"status"), c"status");
        let o = options_get_only_ptr(oo.ptr(), c"clock-mode-color".as_ptr());
        assert!(!o.is_null());
        assert_eq!(options_name(o), c"clock-mode-colour");
    }
}

#[test]
fn the_entries_of_a_set_come_back_in_name_order() {
    let _guard = globals();
    let oo = Options::empty(null_mut());
    unsafe {
        options_set_string(
            oo.ptr(),
            c"@b".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"2".as_ptr()],
        );
        options_set_string(
            oo.ptr(),
            c"@a".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"1".as_ptr()],
        );
        options_set_string(
            oo.ptr(),
            c"@c".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"3".as_ptr()],
        );
        let mut names = Vec::new();
        let mut o = options_first(oo.ptr());
        while !o.is_null() {
            names.push(options_name(o).to_string_lossy().into_owned());
            o = options_next(o);
        }
        assert_eq!(names, vec!["@a", "@b", "@c"]);
    }
}

#[test]
fn a_user_option_is_a_string_of_whatever_is_put_in_it() {
    let _guard = globals();
    let oo = Options::empty(null_mut());
    unsafe {
        let o = options_set_string(
            oo.ptr(),
            c"@u".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"one".as_ptr()],
        );
        assert_eq!(options_is_string(o), 1);
        assert_eq!(options_is_array(o), 0);
        assert!(options_table_entry(o).is_none());
        assert_eq!(string_of(oo.ptr(), c"@u"), "one");
        options_set_string(
            oo.ptr(),
            c"@u".as_ptr(),
            1,
            c"%s".as_ptr(),
            fmt_args![c"two".as_ptr()],
        );
        assert_eq!(string_of(oo.ptr(), c"@u"), "onetwo");
        options_set_string(
            oo.ptr(),
            c"@u".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"three".as_ptr()],
        );
        assert_eq!(string_of(oo.ptr(), c"@u"), "three");
    }
}

#[test]
fn a_string_option_appends_with_the_separator_the_table_names() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_SESSION);
    unsafe {
        options_set_string(
            oo.ptr(),
            c"status-left".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"a".as_ptr()],
        );
        options_set_string(
            oo.ptr(),
            c"status-left".as_ptr(),
            1,
            c"%s".as_ptr(),
            fmt_args![c"b".as_ptr()],
        );
        assert_eq!(string_of(oo.ptr(), c"status-left"), "ab");
    }
}

#[test]
fn an_option_the_set_does_not_carry_is_taken_from_the_table_first() {
    let _guard = globals();
    let parent = Options::defaults(OPTIONS_TABLE_SESSION);
    let child = Options::empty(parent.ptr());
    unsafe {
        assert!(options_get_only_ptr(child.ptr(), c"status-left".as_ptr()).is_null());
        options_set_string(
            child.ptr(),
            c"status-left".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"x".as_ptr()],
        );
        assert!(!options_get_only_ptr(child.ptr(), c"status-left".as_ptr()).is_null());
        assert_eq!(string_of(child.ptr(), c"status-left"), "x");
        options_set_number(child.ptr(), c"status".as_ptr(), 0);
        assert_eq!(options_get_number(child.ptr(), c"status".as_ptr()), 0);
    }
}

#[test]
fn every_kind_of_option_reads_back_as_the_text_it_was_given() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_SESSION);
    unsafe {
        assert_eq!(string_of(oo.ptr(), c"status-left"), "[#{session_name}] ");
        assert_eq!(string_of(oo.ptr(), c"history-limit"), "2000");
        assert_eq!(string_of(oo.ptr(), c"prefix"), "C-b");
        assert_eq!(
            string_of(oo.ptr(), c"message-command-style"),
            "bg=black,fg=yellow,fill=black"
        );
        assert_eq!(string_of(oo.ptr(), c"status"), "on");
        assert_eq!(string_of(oo.ptr(), c"status-position"), "bottom");
        let o = options_get_ptr(oo.ptr(), c"mouse".as_ptr());
        assert_eq!(options_to_string(o, -1, 0).to_string_lossy(), "off");
        assert_eq!(options_to_string(o, -1, 1).to_string_lossy(), "0");
    }
}

#[test]
fn a_command_option_reads_back_as_the_command_line_it_was_given() {
    let _guard = globals();
    let hook = Options::defaults(OPTIONS_TABLE_SESSION);
    unsafe {
        let mut cause: Option<CString> = None;
        let h = options_get_ptr(hook.ptr(), c"window-linked".as_ptr());
        assert_eq!(options_is_array(h), 1);
        assert_eq!(options_is_string(h), 0);
        assert_eq!(
            options_array_set(h, 0, c"display-message hi".as_ptr(), 0, &mut cause),
            0
        );
        assert_eq!(
            options_to_string(h, 0, 0).to_string_lossy(),
            "display-message hi"
        );
        assert_eq!(
            options_to_string(h, -1, 0).to_string_lossy(),
            "display-message hi"
        );
        assert_eq!(options_to_string(h, 5, 0).to_string_lossy(), "");
    }
}

/// The default of one option of each kind reads back as the text a user
/// would have written for it.
#[test]
fn the_default_of_each_kind_of_option_reads_back() {
    let _guard = globals();
    unsafe {
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
                options_default_to_string(entry_for(name)).to_string_lossy(),
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
        let o = options_get_ptr(oo.ptr(), c"terminal-overrides".as_ptr());
        options_array_clear(o);
        assert!(options_array_first(o).is_null());
        let mut cause: Option<CString> = None;
        assert_eq!(options_array_set(o, 2, c"two".as_ptr(), 0, &mut cause), 0);
        assert_eq!(options_array_set(o, 0, c"zero".as_ptr(), 0, &mut cause), 0);
        assert_eq!(options_array_set(o, 0, c"!".as_ptr(), 1, &mut cause), 0);

        let mut items = Vec::new();
        let mut a = options_array_first(o);
        while !a.is_null() {
            items.push((
                options_array_item_index(a),
                (*options_array_item_value(a))
                    .string()
                    .to_string_lossy()
                    .into_owned(),
            ));
            a = options_array_next(o, a);
        }
        assert_eq!(
            items,
            vec![(0, "zero!".to_string()), (2, "two".to_string())]
        );
        assert_eq!((*options_array_get(o, 2)).string(), c"two");
        assert!(options_array_get(o, 1).is_null());
        assert_eq!(options_array_set(o, 2, null(), 0, &mut cause), 0);
        assert!(options_array_get(o, 2).is_null());
        assert_eq!(options_to_string(o, -1, 0).to_string_lossy(), "zero!");
        options_array_clear(o);
        assert_eq!(options_to_string(o, -1, 0).to_string_lossy(), "");
    }
}

#[test]
fn an_option_that_is_not_an_array_turns_the_array_calls_down() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_SESSION);
    unsafe {
        let o = options_get_ptr(oo.ptr(), c"status-left".as_ptr());
        assert!(options_array_get(o, 0).is_null());
        assert!(options_array_first(o).is_null());
        options_array_clear(o);
        let mut cause: Option<CString> = None;
        assert_eq!(options_array_set(o, 0, c"x".as_ptr(), 0, &mut cause), -1);
        assert_eq!(cause.as_ref().unwrap().to_string_lossy(), "not an array");
        assert_eq!(options_array_set(o, 0, c"x".as_ptr(), 0, &mut None), -1);
    }
}

#[test]
fn an_array_of_commands_takes_a_command_line_and_turns_down_what_is_not_one() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_SESSION);
    unsafe {
        let o = options_get_ptr(oo.ptr(), c"window-linked".as_ptr());
        let mut cause: Option<CString> = None;
        assert_eq!(
            options_array_set(o, 0, c"display-message a".as_ptr(), 0, &mut cause),
            0
        );
        assert_eq!(
            options_array_set(o, 0, c"display-message b".as_ptr(), 0, &mut cause),
            0
        );
        assert_eq!(
            options_to_string(o, 0, 0).to_string_lossy(),
            "display-message b"
        );
        assert_eq!(
            options_array_set(o, 1, c"no-such-command".as_ptr(), 0, &mut cause),
            -1
        );
        assert!(!cause.as_ref().unwrap().as_bytes().is_empty());
        assert_eq!(
            options_array_set(o, 1, c"no-such-command".as_ptr(), 0, &mut None),
            -1
        );
    }
}

#[test]
fn an_array_of_colours_takes_a_colour_name_and_turns_down_what_is_not_one() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_PANE);
    unsafe {
        let o = options_get_ptr(oo.ptr(), c"pane-colours".as_ptr());
        let mut cause: Option<CString> = None;
        assert_eq!(options_array_set(o, 0, c"red".as_ptr(), 0, &mut cause), 0);
        assert_eq!(options_array_set(o, 0, c"blue".as_ptr(), 0, &mut cause), 0);
        assert_eq!((*options_array_get(o, 0)).number(), 4);
        assert_eq!(
            options_array_set(o, 1, c"nonsense".as_ptr(), 0, &mut cause),
            -1
        );
        assert_eq!(
            cause.as_ref().unwrap().to_string_lossy(),
            "bad colour: nonsense"
        );
    }
}

#[test]
fn an_array_option_takes_a_whole_list_at_once() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_SERVER);
    unsafe {
        let o = options_get_ptr(oo.ptr(), c"terminal-overrides".as_ptr());
        options_array_clear(o);
        let mut cause: Option<CString> = None;
        assert_eq!(options_array_assign(o, Some(c"a,b c"), &mut cause), 0);
        assert_eq!(options_to_string(o, -1, 0).to_string_lossy(), "a b c");
        assert_eq!(options_array_assign(o, Some(c""), &mut cause), 0);
        assert_eq!(options_to_string(o, -1, 0).to_string_lossy(), "a b c");
        assert_eq!(options_array_assign(o, Some(c",,d"), &mut cause), 0);
        assert_eq!(options_to_string(o, -1, 0).to_string_lossy(), "a b c d");
    }
}

#[test]
fn an_array_with_no_separator_takes_the_whole_string_as_one() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_SESSION);
    unsafe {
        let o = options_get_ptr(oo.ptr(), c"window-linked".as_ptr());
        let mut cause: Option<CString> = None;
        assert_eq!(options_array_assign(o, Some(c""), &mut cause), 0);
        assert!(options_array_first(o).is_null());
        assert_eq!(
            options_array_assign(o, Some(c"display-message ab"), &mut cause),
            0
        );
        assert_eq!(
            options_to_string(o, -1, 0).to_string_lossy(),
            "display-message ab"
        );
        assert_eq!(
            options_array_assign(o, Some(c"no-such-command"), &mut cause),
            -1
        );
        assert!(!cause.as_ref().unwrap().as_bytes().is_empty());
    }
}

#[test]
fn an_array_list_stops_at_the_first_value_it_cannot_take() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_PANE);
    unsafe {
        let o = options_get_ptr(oo.ptr(), c"pane-colours".as_ptr());
        let mut cause: Option<CString> = None;
        assert_eq!(
            options_array_assign(o, Some(c"red,nonsense,blue"), &mut cause),
            -1
        );
        assert_eq!(
            cause.as_ref().unwrap().to_string_lossy(),
            "bad colour: nonsense"
        );
        assert_eq!(options_to_string(o, -1, 0).to_string_lossy(), "red");
    }
}

#[test]
fn an_option_name_carries_an_index_in_brackets() {
    let _guard = globals();
    let mut idx = 0;
    assert_eq!(
        options_parse(c"status", &mut idx)
            .unwrap()
            .to_str()
            .unwrap(),
        "status"
    );
    assert_eq!(idx, -1);
    assert_eq!(
        options_parse(c"a[3]", &mut idx)
            .unwrap()
            .to_str()
            .unwrap(),
        "a"
    );
    assert_eq!(idx, 3);
    assert!(options_parse(c"", &mut idx).is_none());
    assert!(options_parse(c"a[", &mut idx).is_none());
    assert!(options_parse(c"a[]", &mut idx).is_none());
    assert!(options_parse(c"a[3]b", &mut idx).is_none());
    assert!(options_parse(c"a[-1]", &mut idx).is_none());
    assert!(options_parse(c"a[x]", &mut idx).is_none());
}

#[test]
fn an_option_is_looked_up_by_a_name_carrying_an_index() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_SESSION);
    let child = Options::empty(oo.ptr());
    unsafe {
        let mut idx = 0;
        let o = options_parse_get(child.ptr(), c"status-left[2]", &mut idx, 0);
        assert!(!o.is_null());
        assert_eq!(idx, 2);
        assert!(options_parse_get(child.ptr(), c"status-left", &mut idx, 1).is_null());
        assert!(options_parse_get(child.ptr(), c"", &mut idx, 0).is_null());
    }
}

#[test]
fn an_option_name_can_be_shortened_until_it_is_ambiguous() {
    let _guard = globals();
    let mut idx = 0;
    let mut ambiguous = 0;
    assert_eq!(
        options_match(c"status-inter", &mut idx, &mut ambiguous)
            .unwrap()
            .to_string_lossy(),
        "status-interval"
    );
    assert_eq!(ambiguous, 0);
    assert!(options_match(c"status-l", &mut idx, &mut ambiguous).is_none());
    assert_eq!(ambiguous, 1);
    assert_eq!(
        options_match(c"status", &mut idx, &mut ambiguous)
            .unwrap()
            .to_string_lossy(),
        "status"
    );
    assert!(options_match(c"status-", &mut idx, &mut ambiguous).is_none());
    assert_eq!(ambiguous, 1);
    assert!(options_match(c"nonsense", &mut idx, &mut ambiguous).is_none());
    assert_eq!(ambiguous, 0);
    assert_eq!(
        options_match(c"@user", &mut idx, &mut ambiguous)
            .unwrap()
            .to_string_lossy(),
        "@user"
    );
    assert!(options_match(c"", &mut idx, &mut ambiguous).is_none());
    assert_eq!(
        options_match(c"clock-mode-color", &mut idx, &mut ambiguous)
            .unwrap()
            .to_string_lossy(),
        "clock-mode-colour"
    );
}

#[test]
fn an_option_is_looked_up_by_a_shortened_name() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_SESSION);
    unsafe {
        let mut idx = 0;
        let mut ambiguous = 0;
        let o = options_match_get(
            oo.ptr(),
            c"status-inter",
            &mut idx,
            1,
            &mut ambiguous,
        );
        assert_eq!(options_name(o), c"status-interval");
        assert!(
            options_match_get(oo.ptr(), c"nonsense", &mut idx, 0, &mut ambiguous).is_null()
        );
        let child = Options::empty(oo.ptr());
        let o = options_match_get(
            child.ptr(),
            c"status-inter",
            &mut idx,
            0,
            &mut ambiguous,
        );
        assert_eq!(options_owner(o), oo.ptr());
    }
}

#[test]
fn a_string_value_is_checked_before_it_is_kept() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_SESSION);
    unsafe {
        assert_eq!(
            from_string(oo.ptr(), c"default-shell", Some(c"/bin/sh")),
            Ok(())
        );
        assert_eq!(
            from_string(oo.ptr(), c"default-shell", Some(c"/nowhere/at/all")),
            Err("not a suitable shell: /nowhere/at/all".to_string())
        );
        assert_eq!(string_of(oo.ptr(), c"default-shell"), "/bin/sh");
    }
}

#[test]
fn a_value_that_does_not_match_the_pattern_is_turned_down() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_SESSION);
    unsafe {
        assert_eq!(
            from_string(oo.ptr(), c"default-size", Some(c"80x24")),
            Ok(())
        );
        assert_eq!(
            from_string(oo.ptr(), c"default-size", Some(c"wide")),
            Err("value is invalid: wide".to_string())
        );
    }
}

#[test]
fn a_style_value_is_parsed_before_it_is_kept() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_SESSION);
    unsafe {
        assert_eq!(
            from_string(oo.ptr(), c"status-style", Some(c"fg=red")),
            Ok(())
        );
        assert_eq!(
            from_string(oo.ptr(), c"status-style", Some(c"nonsense")),
            Err("invalid style: nonsense".to_string())
        );
        assert_eq!(
            from_string(oo.ptr(), c"status-style", Some(c"#{a}")),
            Ok(())
        );
        assert_eq!(string_of(oo.ptr(), c"status-style"), "#{a}");
    }
}

#[test]
fn every_kind_of_option_takes_a_value_from_a_string() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_SESSION);
    unsafe {
        assert_eq!(
            from_string(oo.ptr(), c"history-limit", Some(c"100")),
            Ok(())
        );
        assert_eq!(string_of(oo.ptr(), c"history-limit"), "100");
        assert_eq!(
            from_string(oo.ptr(), c"history-limit", Some(c"nonsense")),
            Err("value is invalid: nonsense".to_string())
        );
        assert_eq!(from_string(oo.ptr(), c"prefix", Some(c"C-a")), Ok(()));
        assert_eq!(string_of(oo.ptr(), c"prefix"), "C-a");
        assert_eq!(
            from_string(oo.ptr(), c"prefix", Some(c"nonsense")),
            Err("bad key: nonsense".to_string())
        );
        assert_eq!(
            from_string(oo.ptr(), c"message-command-style", Some(c"fg=red")),
            Ok(())
        );

        let pane = Options::defaults(OPTIONS_TABLE_PANE);
        assert_eq!(
            from_string(pane.ptr(), c"scroll-on-clear", Some(c"off")),
            Ok(())
        );
        assert_eq!(string_of(pane.ptr(), c"scroll-on-clear"), "off");
    }
}

#[test]
fn a_colour_option_takes_a_colour_name() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_SERVER);
    unsafe {
        assert_eq!(from_string(oo.ptr(), c"copy-command", Some(c"")), Ok(()));
        let window = Options::defaults(OPTIONS_TABLE_WINDOW);
        assert_eq!(
            from_string(window.ptr(), c"clock-mode-colour", Some(c"red")),
            Ok(())
        );
        assert_eq!(string_of(window.ptr(), c"clock-mode-colour"), "red");
        assert_eq!(
            from_string(window.ptr(), c"clock-mode-colour", Some(c"nonsense")),
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
            assert_eq!(from_string(oo.ptr(), c"status-keys", Some(c"vi")), Ok(()));
            assert_eq!(from_string(oo.ptr(), c"mouse", Some(word)), Ok(()));
            assert_eq!(options_get_number(oo.ptr(), c"mouse".as_ptr()), 1);
        }
        for word in [c"off", c"no", c"0"] {
            assert_eq!(from_string(oo.ptr(), c"mouse", Some(word)), Ok(()));
            assert_eq!(options_get_number(oo.ptr(), c"mouse".as_ptr()), 0);
        }
        assert_eq!(from_string(oo.ptr(), c"mouse", None), Ok(()));
        assert_eq!(options_get_number(oo.ptr(), c"mouse".as_ptr()), 1);
        assert_eq!(from_string(oo.ptr(), c"mouse", Some(c"")), Ok(()));
        assert_eq!(options_get_number(oo.ptr(), c"mouse".as_ptr()), 0);
        assert_eq!(
            from_string(oo.ptr(), c"mouse", Some(c"maybe")),
            Err("bad value: maybe".to_string())
        );
    }
}

#[test]
fn a_choice_option_takes_one_of_its_choices_and_turns_over_with_none() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_SESSION);
    unsafe {
        assert_eq!(
            from_string(oo.ptr(), c"status-position", Some(c"top")),
            Ok(())
        );
        assert_eq!(options_get_number(oo.ptr(), c"status-position".as_ptr()), 0);
        assert_eq!(
            from_string(oo.ptr(), c"status-position", Some(c"middle")),
            Err("unknown value: middle".to_string())
        );
        assert_eq!(from_string(oo.ptr(), c"status-position", None), Ok(()));
        assert_eq!(options_get_number(oo.ptr(), c"status-position".as_ptr()), 1);
        assert_eq!(from_string(oo.ptr(), c"status", Some(c"3")), Ok(()));
        assert_eq!(from_string(oo.ptr(), c"status", None), Ok(()));
        assert_eq!(options_get_number(oo.ptr(), c"status".as_ptr()), 3);

        let mut cause = None;
        assert_eq!(
            options_find_choice(
                entry_for(c"status-position"),
                c"top",
                &mut cause,
            ),
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
                oo.ptr(),
                c"default-client-command",
                Some(c"display-message hello")
            ),
            Ok(())
        );
        assert_eq!(
            string_of(oo.ptr(), c"default-client-command"),
            "display-message hello"
        );
        assert!(options_get_command(oo.ptr(), c"default-client-command".as_ptr()).is_some());
        assert!(
            !from_string(
                oo.ptr(),
                c"default-client-command",
                Some(c"no-such-command")
            )
            .unwrap_err()
            .is_empty()
        );
        options_set_command(oo.ptr(), c"default-client-command".as_ptr(), None);
        assert!(options_get_command(oo.ptr(), c"default-client-command".as_ptr()).is_none());
    }
}

#[test]
fn an_option_with_no_value_and_a_bad_name_is_turned_down() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_SESSION);
    unsafe {
        assert_eq!(
            from_string(oo.ptr(), c"status-left", None),
            Err("empty value".to_string())
        );
        let mut cause = None;
        assert_eq!(
            options_from_string(
                oo.ptr(),
                None,
                c"nonsense".as_ptr(),
                c"x".as_ptr(),
                0,
                &mut cause
            ),
            -1
        );
        assert_eq!(cause.take().unwrap().to_string_lossy(), "bad option name");
        options_set_string(
            oo.ptr(),
            c"@user".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"old".as_ptr()],
        );
        assert_eq!(from_string(oo.ptr(), c"@user", Some(c"x")), Ok(()));
        assert_eq!(string_of(oo.ptr(), c"@user"), "x");
    }
}

#[test]
fn a_style_option_is_read_once_and_kept_unless_it_is_a_format() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_SESSION);
    unsafe {
        let mut ft = format_create(null_mut(), null_mut(), 0, FORMAT_NOJOBS);
        let ft_ptr = &raw mut *ft;
        options_set_string(
            oo.ptr(),
            c"status-style".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"fg=red".as_ptr()],
        );
        let sy = options_string_to_style(oo.ptr(), c"status-style".as_ptr(), Some(&mut *ft_ptr));
        assert!(!sy.is_null());
        assert_eq!((*sy).gc.fg, 1);
        assert_eq!(
            options_string_to_style(oo.ptr(), c"status-style".as_ptr(), Some(&mut *ft_ptr)),
            sy
        );

        options_set_string(
            oo.ptr(),
            c"status-style".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"nonsense".as_ptr()],
        );
        assert!(
            options_string_to_style(oo.ptr(), c"status-style".as_ptr(), Some(&mut *ft_ptr))
                .is_null()
        );

        options_set_string(
            oo.ptr(),
            c"status-style".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"#{?1,fg=red,fg=blue}".as_ptr()],
        );
        assert!(
            !options_string_to_style(oo.ptr(), c"status-style".as_ptr(), Some(&mut *ft_ptr))
                .is_null()
        );
        options_set_string(
            oo.ptr(),
            c"status-style".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"#{?1,nonsense,nonsense}".as_ptr()],
        );
        assert!(
            options_string_to_style(oo.ptr(), c"status-style".as_ptr(), Some(&mut *ft_ptr))
                .is_null()
        );
        options_set_string(
            oo.ptr(),
            c"status-style".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"#{a}".as_ptr()],
        );
        assert!(options_string_to_style(oo.ptr(), c"status-style".as_ptr(), None).is_null());

        assert!(
            options_string_to_style(oo.ptr(), c"status".as_ptr(), Some(&mut *ft_ptr)).is_null()
        );
        assert!(
            options_string_to_style(oo.ptr(), c"nonsense".as_ptr(), Some(&mut *ft_ptr)).is_null()
        );
    }
}

#[test]
fn an_option_is_taken_away_or_put_back_to_its_default() {
    let _guard = globals();
    let oo = Options::defaults(OPTIONS_TABLE_SESSION);
    unsafe {
        options_set_string(
            oo.ptr(),
            c"status-left".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"x".as_ptr()],
        );
        let o = options_get_only_ptr(oo.ptr(), c"status-left".as_ptr());
        assert_eq!(options_remove_or_default(o, -1, &mut None), 0);
        assert!(options_get_only_ptr(oo.ptr(), c"status-left".as_ptr()).is_null());

        let server = Options::defaults(OPTIONS_TABLE_SERVER);
        let o = options_get_ptr(server.ptr(), c"terminal-overrides".as_ptr());
        let mut cause: Option<CString> = None;
        options_array_set(o, 0, c"x".as_ptr(), 0, &mut cause);
        assert_eq!(options_remove_or_default(o, 0, &mut cause), 0);
        assert!(options_array_get(o, 0).is_null());

        let single = options_get_ptr(server.ptr(), c"copy-command".as_ptr());
        assert_eq!(options_remove_or_default(single, 0, &mut cause), -1);
        assert_eq!(cause.as_ref().unwrap().to_string_lossy(), "not an array");
    }
}

#[test]
fn a_global_option_taken_away_goes_back_to_its_default() {
    let _guard = globals();
    unsafe {
        options_set_string(
            global_s_options,
            c"status-left".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"changed".as_ptr()],
        );
        let o = options_get_only_ptr(global_s_options, c"status-left".as_ptr());
        assert_eq!(options_remove_or_default(o, -1, &mut None), 0);
        assert_eq!(
            string_of(global_s_options, c"status-left"),
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
    fs.set_session(session.ptr());
    unsafe { fs.set_winlink(wl) };
    fs.set_window(window.ptr());
    unsafe { fs.set_pane(pane.ptr()) };
    unsafe {
        let mut oo = null_mut::<options>();
        let mut cause = None;
        let mut scope = |name: &CStr, oo: &mut *mut options, cause: &mut Option<CString>| {
            options_scope_from_name(&*args.ptr(), 0, name, &raw mut *fs, oo, cause)
        };
        assert_eq!(
            scope(c"copy-command", &mut oo, &mut cause),
            OPTIONS_TABLE_SERVER
        );
        assert_eq!(oo, global_options);
        assert_eq!(
            scope(c"status-left", &mut oo, &mut cause),
            OPTIONS_TABLE_SESSION
        );
        assert_eq!(oo, session.options());
        assert_eq!(
            scope(c"mode-keys", &mut oo, &mut cause),
            OPTIONS_TABLE_WINDOW
        );
        assert_eq!(oo, window.options());
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
    let mut fs = Box::new(cmd_find_state::default());
    unsafe {
        let mut oo = null_mut::<options>();
        let mut cause = None;
        for (args, want) in [
            (&args, "no current session"),
            (&targeted, "no such session: other"),
        ] {
            assert_eq!(
                options_scope_from_name(
                    &*args.ptr(),
                    0,
                    c"status-left",
                    &raw mut *fs,
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
                options_scope_from_name(
                    &*args.ptr(),
                    0,
                    c"mode-keys",
                    &raw mut *fs,
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
    fs.set_window(window.ptr());
    unsafe {
        let mut oo = null_mut::<options>();
        let mut cause = None;
        assert_eq!(
            options_scope_from_name(
                &*args.ptr(),
                0,
                c"remain-on-exit",
                &raw mut *fs,
                &mut oo,
                &mut cause
            ),
            OPTIONS_TABLE_NONE
        );
        assert_eq!(cause.take().unwrap().to_str().unwrap(), "no current pane");
        assert_eq!(
            options_scope_from_name(
                &*targeted.ptr(),
                0,
                c"remain-on-exit",
                &raw mut *fs,
                &mut oo,
                &mut cause
            ),
            OPTIONS_TABLE_NONE
        );
        assert_eq!(
            cause.take().unwrap().to_str().unwrap(),
            "no such pane: other"
        );
        fs.set_pane(pane.ptr());
        assert_eq!(
            options_scope_from_name(
                &*args.ptr(),
                0,
                c"remain-on-exit",
                &raw mut *fs,
                &mut oo,
                &mut cause
            ),
            OPTIONS_TABLE_PANE
        );
        assert_eq!(oo, pane.options());
        assert_eq!(
            options_scope_from_name(
                &*bare.ptr(),
                0,
                c"remain-on-exit",
                &raw mut *fs,
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
    let mut fs = Box::new(cmd_find_state::default());
    unsafe {
        let mut oo = null_mut::<options>();
        let mut cause = None;
        assert_eq!(
            options_scope_from_name(
                &*args.ptr(),
                0,
                c"status-left",
                &raw mut *fs,
                &mut oo,
                &mut cause
            ),
            OPTIONS_TABLE_SESSION
        );
        assert_eq!(oo, global_s_options);
        assert_eq!(
            options_scope_from_name(
                &*args.ptr(),
                0,
                c"mode-keys",
                &raw mut *fs,
                &mut oo,
                &mut cause
            ),
            OPTIONS_TABLE_WINDOW
        );
        assert_eq!(oo, global_w_options);
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
    fs.set_session(session.ptr());
    unsafe { fs.set_winlink(wl) };
    fs.set_window(window.ptr());
    unsafe { fs.set_pane(pane.ptr()) };
    unsafe {
        let mut oo = null_mut::<options>();
        let mut cause = None;
        for (line, window_flag, want, set) in [
            (c"set-option -s @u", 0, OPTIONS_TABLE_SERVER, global_options),
            (c"set-option -p @u", 0, OPTIONS_TABLE_PANE, pane.options()),
            (
                c"set-option -w @u",
                0,
                OPTIONS_TABLE_WINDOW,
                window.options(),
            ),
            (c"set-option @u", 1, OPTIONS_TABLE_WINDOW, window.options()),
            (
                c"set-option -wg @u",
                0,
                OPTIONS_TABLE_WINDOW,
                global_w_options,
            ),
            (
                c"set-option @u",
                0,
                OPTIONS_TABLE_SESSION,
                session.options(),
            ),
            (
                c"set-option -g @u",
                0,
                OPTIONS_TABLE_SESSION,
                global_s_options,
            ),
        ] {
            let args = Args::parse(line);
            assert_eq!(
                options_scope_from_name(
                    &*args.ptr(),
                    window_flag,
                    c"@u",
                    &raw mut *fs,
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
    let mut fs = Box::new(cmd_find_state::default());
    unsafe {
        let mut oo = null_mut::<options>();
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
                options_scope_from_flags(
                    &*args.ptr(),
                    window_flag,
                    &raw mut *fs,
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
    registry.add_pane(&mut pane);
    let mut list = Clients::new();
    let client = list.add("push", 10, 2);
    unsafe {
        options_set_number(window.options(), c"automatic-rename".as_ptr(), 1);
        options_set_parent(session.options(), global_s_options);
        options_set_parent(window.options(), global_w_options);
        options_set_parent(pane.options(), window.options());
        (*client).session = session.ptr();
        (*client).tty.term = Some(zeroed_term());
        (*client).tty.flags = TTY_OPENED;
        let table_ref =
            crate::key_bindings::key_bindings_get_table_ref(c"root".as_ptr(), 1).unwrap();
        (*client).keytable_ref = Some(table_ref);
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
            options_push_changes(name);
        }
        assert_ne!((*pane.ptr()).flags & PANE_STYLECHANGED, 0);
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
        options_push_changes(c"status");
        options_push_changes(c"status-interval");
    }
    drop(list);
}

/// A standalone set holding just `pane-colours`, taken straight from the
/// options table so it has the right table entry.
fn pane_colours_options(values: &[(u_int, &CStr)]) -> Options {
    let oo = Options::empty(null_mut());
    unsafe {
        let o = options_default(oo.ptr(), entry_for(c"pane-colours"));
        for &(n, value) in values {
            options_array_set(o, n, value.as_ptr(), 0, &mut None);
        }
    }
    oo
}

#[test]
fn an_empty_pane_colours_option_reads_back_as_no_defaults() {
    let _guard = globals();
    let oo = pane_colours_options(&[]);
    unsafe {
        assert!(options_pane_colours(oo.ptr()).is_none());
        let mut p = colour_palette {
            fg: 0,
            bg: 0,
            palette: None,
            default_palette: Some(Box::new([-1; 256])),
        };
        options_load_pane_colours(oo.ptr(), Some(&mut p));
        assert!(p.default_palette.is_none());
    }
}

#[test]
fn pane_colours_read_back_by_index_and_load_into_a_palette() {
    let _guard = globals();
    let oo = pane_colours_options(&[(0, c"red"), (1, c"#00ff00"), (300, c"blue")]);
    unsafe {
        let def = options_pane_colours(oo.ptr()).expect("the option holds entries");
        assert_eq!(def[0], 1);
        assert_eq!(def[1], 0x00ff00 | COLOUR_FLAG_RGB);
        assert_eq!(def[2], -1);

        let mut p = colour_palette {
            fg: 0,
            bg: 0,
            palette: None,
            default_palette: None,
        };
        options_load_pane_colours(oo.ptr(), Some(&mut p));
        assert_eq!(colour_palette_get(Some(&p), 0), 1);
        assert_eq!(colour_palette_get(Some(&p), 1), 0x00ff00 | COLOUR_FLAG_RGB);
        assert_eq!(colour_palette_get(Some(&p), 2), -1);
        colour_palette_free(Some(&mut p));
    }
}

/// Every spec the `codepoint-widths` option holds is handed over in array
/// order, and an option with none hands over nothing.
#[test]
fn codepoint_widths_are_read_back_in_array_order() {
    let _guard = globals();
    let oo = Options::empty(null_mut());
    unsafe {
        let o = options_default(oo.ptr(), entry_for(c"codepoint-widths"));
        assert!(options_codepoint_widths(oo.ptr()).is_empty());
        options_array_set(o, 1, c"U+E9=2".as_ptr(), 0, &mut None);
        options_array_set(o, 0, c"U+41=1".as_ptr(), 0, &mut None);
        let specs = options_codepoint_widths(oo.ptr());
        assert_eq!(specs, vec![c"U+41=1".to_owned(), c"U+E9=2".to_owned()]);
    }
}
