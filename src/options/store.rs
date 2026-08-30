use super::table::{options_other_names, options_table};
use crate::alerts::alerts_reset_all;
use crate::arguments::{args_get, args_has};
use crate::cmd::cmd_list_print;
use crate::cmd::{CMD_PARSE_SUCCESS, cmd_parse_from_string};
use crate::compat::strtonum;
use crate::ffi::{fnmatch, sscanf, strstr};
use crate::fmt_args;
use crate::fmt_engine::{FmtArg, format_alloc};
use crate::format::format_expand;
use crate::grid::grid_default_cell;
use crate::input::input_set_buffer_size;
use crate::layout::layout_fix_panes;
use crate::log::{fatalx, log_debug};
use crate::resize::recalculate_sizes;
use crate::server::client_walk;
use crate::server::server_client_set_key_table;
use crate::server::server_redraw_client;
use crate::session::{session_options, session_update_history, sessions_after, sessions_first};
use crate::status::{status_timer_start_all, status_update_cache};
use crate::style::{colour_fromstring, colour_palette_from_defaults, colour_tostring};
use crate::style::{style_parse, style_set, style_set_scrollbar_style_from_option};
use crate::text::utf8_update_width_cache;
use crate::text::{KEYC_UNKNOWN, key_string_lookup_key, key_string_lookup_string};
use crate::tmux::{checkshell, global_options, global_s_options, global_w_options};
use crate::tty::tty_keys_build;
pub use crate::types::*;
use crate::window::pane_walk;
use crate::window::window_get_active;
use crate::window::{
    window_find_by_id_ref, window_pane_default_cursor, window_set_fill_character, windows,
};
use crate::xmalloc::xasprintf;
use ::core::ffi::{CStr, c_char, c_int, c_longlong};
use ::core::ops::Bound;
use ::core::ptr::{null, null_mut};
use ::std::ffi::CString;

pub const RB_BLACK: c_int = 0;
pub const RB_RED: c_int = 1;
pub const RB_NEGINF: c_int = -1;
pub const UINT_MAX: u_int = u_int::MAX;
pub const TTY_OPENED: c_int = 0x20;
pub const PANE_CHANGED: c_int = 0x80;
pub const PANE_STYLECHANGED: c_int = 0x1000;
pub const PANE_THEMECHANGED: c_int = 0x2000;

pub const OPTIONS_TABLE_STRING: options_table_type = 0;
pub const OPTIONS_TABLE_NUMBER: options_table_type = 1;
pub const OPTIONS_TABLE_KEY: options_table_type = 2;
pub const OPTIONS_TABLE_COLOUR: options_table_type = 3;
pub const OPTIONS_TABLE_FLAG: options_table_type = 4;
pub const OPTIONS_TABLE_CHOICE: options_table_type = 5;
pub const OPTIONS_TABLE_COMMAND: options_table_type = 6;

pub const OPTIONS_TABLE_NONE: c_int = 0;
pub const OPTIONS_TABLE_SERVER: c_int = 0x1;
pub const OPTIONS_TABLE_SESSION: c_int = 0x2;
pub const OPTIONS_TABLE_WINDOW: c_int = 0x4;
pub const OPTIONS_TABLE_PANE: c_int = 0x8;
pub const OPTIONS_TABLE_IS_ARRAY: c_int = 0x1;
pub const OPTIONS_TABLE_IS_STYLE: c_int = 0x4;

/// A set of options and the set it falls back on when it has none of its own.
#[repr(C)]
pub struct options {
    pub tree: options_tree,
    /// The set this one falls back to when it has no entry of its own. A
    /// borrow, not an owning edge: a set's parent is one of the three the
    /// server holds for as long as it runs, so it never goes first.
    pub parent: *mut options,
}

/// The options of one set, by name.
pub type options_tree = ::std::collections::BTreeMap<CString, Box<options_entry>>;

/// One option: its name, the table entry that says what kind of value it
/// holds, the value itself, and the style that value was last read as. An
/// array option keeps its values in `array` rather than in `value`, which a
/// union cannot hold.
#[repr(C)]
pub struct options_entry {
    /// The set that holds this entry. A borrow, not an owning edge: the set
    /// holds its entries by value, so an entry never outlives it.
    pub owner: *mut options,
    pub name: Option<CString>,
    pub(crate) tableentry: Option<&'static options_table_entry_t>,
    pub value: options_value,
    pub array: options_array,
    pub cached: c_int,
    pub style: style,
}

/// One value of an array option.
#[repr(C)]
pub struct options_array_item_t {
    pub index: u_int,
    pub value: options_value,
}

/// The entries of the option table.
fn table() -> &'static [options_table_entry_t] {
    &options_table
}

/// The choices a choice option accepts. Only ever asked of one that has them.
unsafe fn choices_of(oe: *const options_table_entry_t) -> &'static [&'static CStr] {
    unsafe { (*oe).choices.expect("a choice option lists its choices") }
}

/// A table string as the raw pointer the older calls still take.
fn cstr_or_null(value: Option<&'static CStr>) -> *const c_char {
    value.map_or(null(), CStr::as_ptr)
}

/// The name an option used to be spelled, mapped to the one it has now.
unsafe fn options_map_name(name: *const c_char) -> *const c_char {
    unsafe {
        let wanted = CStr::from_ptr(name);
        options_other_names
            .iter()
            .find(|map| map.from == wanted)
            .map_or(name, |map| map.to.as_ptr())
    }
}

/// The table entry the parent set has for `s`, which is what an option added
/// to a set below it is made from.
unsafe fn options_parent_table_entry(
    oo: *mut options,
    s: *const c_char,
) -> *const options_table_entry_t {
    unsafe {
        if (*oo).parent.is_null() {
            fatalx(c"no parent options for %s".as_ptr(), fmt_args![s]);
        }
        let Some(o) = options_get((*oo).parent, s) else {
            fatalx(c"%s not in parent options".as_ptr(), fmt_args![s]);
        };
        o.tableentry
            .map_or(::core::ptr::null(), |oe| oe as *const options_table_entry_t)
    }
}

/// Whether an option holds a string, which a user option does too.
fn is_string(o: &options_entry) -> bool {
    {
        match o.tableentry {
            None => true,
            Some(oe) => oe.type_0 == OPTIONS_TABLE_STRING,
        }
    }
}

/// Whether an option holds a number, which the key, colour, flag and choice
/// kinds all do.
fn is_number(o: &options_entry) -> bool {
    {
        o.tableentry.is_some_and(|oe| {
            matches!(
                oe.type_0,
                OPTIONS_TABLE_NUMBER
                    | OPTIONS_TABLE_KEY
                    | OPTIONS_TABLE_COLOUR
                    | OPTIONS_TABLE_FLAG
                    | OPTIONS_TABLE_CHOICE
            )
        })
    }
}

/// Whether an option holds a command list.
fn is_command(o: &options_entry) -> bool {
    o.tableentry
        .is_some_and(|oe| oe.type_0 == OPTIONS_TABLE_COMMAND)
}

/// Gives up whatever one value of `o` was holding.
unsafe fn options_value_free(o: *mut options_entry, ov: *mut options_value) {
    unsafe {
        if is_string(&*o) || is_command(&*o) {
            *ov = options_value::None;
        }
    }
}

unsafe fn options_array_value_free(o: *mut options_entry, a: *mut options_array_item_t) {
    unsafe { options_value_free(o, &raw mut (*a).value) }
}

/// One value of `o` as the text a user would have written for it. `numeric`
/// asks for a flag as its number rather than as `on` or `off`.
unsafe fn options_value_to_string(
    o: *mut options_entry,
    ov: *mut options_value,
    numeric: c_int,
) -> CString {
    unsafe {
        if is_command(&*o) {
            return cmd_list_print((*ov).cmdlist(), 0);
        }
        if is_number(&*o) {
            return match table_of(o).type_0 {
                OPTIONS_TABLE_NUMBER => xasprintf(c"%lld".as_ptr(), fmt_args![(*ov).number()]),
                OPTIONS_TABLE_KEY => {
                    CStr::from_ptr(key_string_lookup_key((*ov).number() as key_code, 0)).to_owned()
                }
                OPTIONS_TABLE_COLOUR => colour_tostring((*ov).number() as c_int),
                OPTIONS_TABLE_FLAG if numeric != 0 => {
                    xasprintf(c"%lld".as_ptr(), fmt_args![(*ov).number()])
                }
                OPTIONS_TABLE_FLAG if (*ov).number() != 0 => c"on".to_owned(),
                OPTIONS_TABLE_FLAG => c"off".to_owned(),
                _ => choices_of(table_of(o))[(*ov).number() as usize].to_owned(),
            };
        }
        CStr::from_ptr((*ov).string()).to_owned()
    }
}

/// A new, empty set of options below `parent`.
/// The option set a struct carries, or null if it carries none.
pub fn options_ptr(oo: &Option<Box<options>>) -> *mut options {
    match oo {
        Some(oo) => &raw const **oo as *mut options,
        None => ::core::ptr::null_mut::<options>(),
    }
}

pub fn options_create_boxed(parent: *mut options) -> Box<options> {
    Box::new(options {
        tree: options_tree::new(),
        parent,
    })
}

/// Gives up a set and everything in it.
pub unsafe fn options_free(mut oo: Box<options>) {
    unsafe {
        let entries: Vec<*mut options_entry> = oo.tree.values_mut().map(|o| &raw mut **o).collect();
        for o in entries {
            options_remove(o);
        }
        drop(oo);
    }
}

pub unsafe fn options_get_parent(oo: *mut options) -> *mut options {
    unsafe { (*oo).parent }
}

pub unsafe fn options_set_parent(oo: *mut options, parent: *mut options) {
    unsafe { (*oo).parent = parent };
}

/// The first option of a set, in name order.
pub unsafe fn options_first(oo: *mut options) -> *mut options_entry {
    unsafe {
        (*oo)
            .tree
            .values()
            .next()
            .map(|o| &raw const **o as *mut options_entry)
            .unwrap_or(null_mut::<options_entry>())
    }
}

pub unsafe fn options_next(o: *mut options_entry) -> *mut options_entry {
    unsafe {
        (*(*o).owner)
            .tree
            .range::<CStr, _>((
                Bound::Excluded(CStr::from_ptr(cstr_ptr(&(*o).name))),
                Bound::Unbounded,
            ))
            .next()
            .map(|(_, o)| &raw const **o as *mut options_entry)
            .unwrap_or(null_mut::<options_entry>())
    }
}

/// The option `name` in this set alone, looked for under the name it used to
/// have if it is not there under the one asked for.
pub unsafe fn options_get_only<'a>(
    oo: *mut options,
    name: *const c_char,
) -> Option<&'a mut options_entry> {
    unsafe { options_get_only_ptr(oo, name).as_mut() }
}

/// [`options_get_only`] as the raw view a caller takes when it wants to keep
/// the entry across a call that writes to the same set.
pub unsafe fn options_get_only_ptr(oo: *mut options, name: *const c_char) -> *mut options_entry {
    unsafe {
        if let Some(found) = (*oo).tree.get(CStr::from_ptr(name)) {
            return &raw const **found as *mut options_entry;
        }
        (*oo)
            .tree
            .get(CStr::from_ptr(options_map_name(name)))
            .map(|o| &raw const **o as *mut options_entry)
            .unwrap_or(null_mut::<options_entry>())
    }
}

/// The option `name`, from this set or the first set above it that has one.
pub unsafe fn options_get<'a>(
    oo: *mut options,
    name: *const c_char,
) -> Option<&'a mut options_entry> {
    unsafe { options_get_ptr(oo, name).as_mut() }
}

/// [`options_get`] as the raw view a caller takes when it wants to keep the
/// entry across a call that writes to the same set.
pub unsafe fn options_get_ptr(mut oo: *mut options, name: *const c_char) -> *mut options_entry {
    unsafe {
        let mut o = options_get_only_ptr(oo, name);
        while o.is_null() {
            oo = (*oo).parent;
            if oo.is_null() {
                break;
            }
            o = options_get_only_ptr(oo, name);
        }
        o
    }
}

/// Adds `oe` to the set with no value in it yet.
pub unsafe fn options_empty(
    oo: *mut options,
    oe: *const options_table_entry_t,
) -> *mut options_entry {
    unsafe {
        let o = options_add(oo, (*oe).name.as_ptr());
        (*o).tableentry = oe.as_ref();
        o
    }
}

/// Adds `oe` to the set with the value the table gives it.
pub unsafe fn options_default(
    oo: *mut options,
    oe: *const options_table_entry_t,
) -> *mut options_entry {
    unsafe {
        let o = options_empty(oo, oe);
        let ov = &raw mut (*o).value;
        if (*oe).flags & OPTIONS_TABLE_IS_ARRAY != 0 {
            let Some(default_arr) = (*oe).default_arr else {
                options_array_assign(o, cstr_or_null((*oe).default_str), &mut None);
                return o;
            };
            for (i, value) in default_arr.iter().enumerate() {
                options_array_set(o, i as u_int, value.as_ptr(), 0, &mut None);
            }
            return o;
        }
        match (*oe).type_0 {
            OPTIONS_TABLE_STRING => {
                *ov = options_value::String((*oe).default_str.unwrap_or(c"").to_owned());
            }
            OPTIONS_TABLE_COMMAND => {
                let mut pr = cmd_parse_from_string(
                    cstr_or_null((*oe).default_str),
                    null_mut::<cmd_parse_input>(),
                );
                if pr.status == CMD_PARSE_SUCCESS {
                    *ov = match pr.cmdlist.take() {
                        Some(cmdlist) => options_value::Commands(cmdlist),
                        None => options_value::None,
                    };
                } else {
                    let _ = pr.error.take();
                }
            }
            _ => {
                *ov = options_value::Number((*oe).default_num);
            }
        }
        o
    }
}

/// The value the table gives `oe`, as the text a user would have written.
pub unsafe fn options_default_to_string(oe: *const options_table_entry_t) -> CString {
    unsafe {
        match (*oe).type_0 {
            OPTIONS_TABLE_STRING | OPTIONS_TABLE_COMMAND => {
                (*oe).default_str.unwrap_or(c"").to_owned()
            }
            OPTIONS_TABLE_NUMBER => xasprintf(c"%lld".as_ptr(), fmt_args![(*oe).default_num]),
            OPTIONS_TABLE_KEY => {
                CStr::from_ptr(key_string_lookup_key((*oe).default_num as key_code, 0)).to_owned()
            }
            OPTIONS_TABLE_COLOUR => colour_tostring((*oe).default_num as c_int),
            OPTIONS_TABLE_FLAG => {
                if (*oe).default_num != 0 {
                    c"on".to_owned()
                } else {
                    c"off".to_owned()
                }
            }
            OPTIONS_TABLE_CHOICE => choices_of(oe)[(*oe).default_num as usize].to_owned(),
            _ => fatalx(c"unknown option type".as_ptr(), fmt_args![]),
        }
    }
}

/// Adds an option named `name` to the set, taking away whatever was there.
unsafe fn options_add(oo: *mut options, name: *const c_char) -> *mut options_entry {
    unsafe {
        let o = options_get_only_ptr(oo, name);
        if !o.is_null() {
            options_remove(o);
        }
        let mut o = Box::new(options_entry {
            owner: oo,
            name: Some(CStr::from_ptr(name).to_owned()),
            tableentry: None,
            value: options_value::None,
            array: options_array::new(),
            cached: 0,
            style: style::default(),
        });
        let ptr = &mut *o as *mut options_entry;
        (*oo).tree.insert(CStr::from_ptr(name).to_owned(), o);
        ptr
    }
}

/// Takes an option out of its set and gives up what it held.
unsafe fn options_remove(o: *mut options_entry) {
    unsafe {
        let oo = (*o).owner;
        if options_is_array(o) != 0 {
            options_array_clear(o);
        } else {
            options_value_free(o, &raw mut (*o).value);
        }
        (*oo).tree.remove(CStr::from_ptr(cstr_ptr(&(*o).name)));
    }
}

pub unsafe fn options_name(o: *mut options_entry) -> *const c_char {
    unsafe { cstr_ptr(&(*o).name) }
}

pub unsafe fn options_owner(o: *mut options_entry) -> *mut options {
    unsafe { (*o).owner }
}

pub unsafe fn options_table_entry(o: *mut options_entry) -> *const options_table_entry_t {
    unsafe {
        (*o).tableentry
            .map_or(::core::ptr::null(), |oe| oe as *const options_table_entry_t)
    }
}

/// The table entry an option was made from. Only ever asked of one that has
/// it, which the `is_*` tests above have already established.
unsafe fn table_of(o: *mut options_entry) -> &'static options_table_entry_t {
    unsafe { (*o).tableentry.expect("the option comes from the table") }
}

/// The value of an array option at `idx`, if it has one.
unsafe fn options_array_item(o: *mut options_entry, idx: u_int) -> *mut options_array_item_t {
    unsafe {
        (*o).array
            .get_mut(&idx)
            .map(|a| &raw mut *a)
            .unwrap_or(null_mut::<options_array_item_t>())
    }
}

/// Makes room for a value of an array option at `idx`.
unsafe fn options_array_new(o: *mut options_entry, idx: u_int) -> *mut options_array_item_t {
    unsafe {
        (*o).array.insert(
            idx,
            options_array_item_t {
                index: idx,
                value: ::core::mem::zeroed(),
            },
        );
        options_array_item(o, idx)
    }
}

/// The value an array option is to keep at `idx`, made room for if it has none
/// there yet and given up if it had one.
unsafe fn options_array_slot(o: *mut options_entry, idx: u_int) -> *mut options_array_item_t {
    unsafe {
        let a = options_array_item(o, idx);
        if a.is_null() {
            return options_array_new(o, idx);
        }
        options_array_value_free(o, a);
        a
    }
}

unsafe fn options_array_free(o: *mut options_entry, a: *mut options_array_item_t) {
    unsafe {
        options_array_value_free(o, a);
        (*o).array.remove(&(*a).index);
    }
}

/// Empties an array option.
pub unsafe fn options_array_clear(o: *mut options_entry) {
    unsafe {
        if options_is_array(o) == 0 {
            return;
        }
        for a in (*o).array.values_mut() {
            options_array_value_free(o, a as *mut options_array_item_t);
        }
        (*o).array.clear();
    }
}

pub unsafe fn options_array_get(o: *mut options_entry, idx: u_int) -> *mut options_value {
    unsafe {
        if options_is_array(o) == 0 {
            return null_mut::<options_value>();
        }
        let a = options_array_item(o, idx);
        if a.is_null() {
            return null_mut::<options_value>();
        }
        &raw mut (*a).value
    }
}

/// Puts `value` at `idx` of an array option, or takes that index away when
/// there is no value. `append` adds to a string already there.
pub unsafe fn options_array_set(
    o: *mut options_entry,
    idx: u_int,
    value: *const c_char,
    append: c_int,
    cause: &mut Option<CString>,
) -> c_int {
    unsafe {
        if options_is_array(o) == 0 {
            *cause = Some(c"not an array".to_owned());
            return -1;
        }
        if value.is_null() {
            let a = options_array_item(o, idx);
            if !a.is_null() {
                options_array_free(o, a);
            }
            return 0;
        }

        if is_command(&*o) {
            let mut pr = cmd_parse_from_string(value, null_mut::<cmd_parse_input>());
            if pr.status != CMD_PARSE_SUCCESS {
                if let Some(error) = pr.error.take() {
                    *cause = Some(error);
                }
                return -1;
            }
            let slot = options_array_slot(o, idx);
            (*slot).value = match pr.cmdlist.take() {
                Some(cmdlist) => options_value::Commands(cmdlist),
                None => options_value::None,
            };
            return 0;
        }
        if is_string(&*o) {
            let a = options_array_item(o, idx);
            let new = if !a.is_null() && append != 0 {
                xasprintf(c"%s%s".as_ptr(), fmt_args![(*a).value.string(), value])
            } else {
                CStr::from_ptr(value).to_owned()
            };
            let slot = options_array_slot(o, idx);
            (*slot).value = options_value::String(new);
            return 0;
        }
        if table_of(o).type_0 == OPTIONS_TABLE_COLOUR {
            let number = colour_fromstring(value) as c_longlong;
            if number == -1 {
                *cause = Some(xasprintf(c"bad colour: %s".as_ptr(), fmt_args![value]));
                return -1;
            }
            (*options_array_slot(o, idx)).value = options_value::Number(number);
            return 0;
        }

        *cause = Some(c"wrong array type".to_owned());
        -1
    }
}

/// Puts a whole list into an array option, one value per separator the table
/// names. An option whose separator is empty takes the string as one value.
pub unsafe fn options_array_assign(
    o: *mut options_entry,
    s: *const c_char,
    cause: &mut Option<CString>,
) -> c_int {
    unsafe {
        let separators = table_of(o).separator.unwrap_or(c" ,").to_bytes();
        let bytes = CStr::from_ptr(s).to_bytes();
        if bytes.is_empty() {
            return 0;
        }

        /*
         * A value goes at the first index the array has none at, which is why
         * assigning twice adds rather than replaces.
         */
        let mut first_free = |o: *mut options_entry| {
            let mut i = 0;
            while !options_array_item(o, i).is_null() {
                i += 1;
            }
            i
        };

        if separators.is_empty() {
            let i = first_free(o);
            return options_array_set(o, i, s, 0, cause);
        }

        for next in bytes.split(|byte| separators.contains(byte)) {
            if next.is_empty() {
                continue;
            }
            let i = first_free(o);
            let value = CString::new(next).expect("no NUL inside");
            if options_array_set(o, i, value.as_ptr(), 0, cause) != 0 {
                return -1;
            }
        }
        0
    }
}

pub unsafe fn options_array_first(o: *mut options_entry) -> *mut options_array_item_t {
    unsafe {
        if options_is_array(o) == 0 {
            return null_mut::<options_array_item_t>();
        }
        (*o).array
            .values()
            .next()
            .map(|a| &raw const *a as *mut options_array_item_t)
            .unwrap_or(null_mut::<options_array_item_t>())
    }
}

pub unsafe fn options_array_next(
    o: *mut options_entry,
    a: *mut options_array_item_t,
) -> *mut options_array_item_t {
    unsafe {
        (*o).array
            .range((Bound::Excluded((*a).index), Bound::Unbounded))
            .next()
            .map(|(_, a)| &raw const *a as *mut options_array_item_t)
            .unwrap_or(null_mut::<options_array_item_t>())
    }
}

pub unsafe fn options_array_item_index(a: *mut options_array_item_t) -> u_int {
    unsafe { (*a).index }
}

pub unsafe fn options_array_item_value(a: *mut options_array_item_t) -> *mut options_value {
    unsafe { &raw mut (*a).value }
}

pub(crate) unsafe fn options_array_item_command(
    a: *mut options_array_item_t,
) -> Option<CmdListRef> {
    unsafe { (*a).value.commands() }
}

/// The `codepoint-widths` specs held in `oo`, in array order, for
/// [`utf8_update_width_cache`] to apply. The pointers stay valid only until
/// the option is next changed.
pub unsafe fn options_codepoint_widths(oo: *mut options) -> Vec<*const c_char> {
    unsafe {
        let o = options_get_ptr(oo, c"codepoint-widths".as_ptr());
        let mut a = options_array_first(o);
        let mut specs = Vec::new();
        while !a.is_null() {
            specs.push((*options_array_item_value(a)).string());
            a = options_array_next(o, a);
        }
        specs
    }
}

/// The `pane-colours` option of `oo` as a default palette, or `None` when the
/// option holds no entries at all.
pub unsafe fn options_pane_colours(oo: *mut options) -> Option<[c_int; 256]> {
    unsafe {
        let o = options_get_ptr(oo, c"pane-colours".as_ptr());
        let mut a = options_array_first(o);
        if a.is_null() {
            return None;
        }
        let mut def = [-1; 256];
        while !a.is_null() {
            let n = options_array_item_index(a) as usize;
            if n < 256 {
                def[n] = (*options_array_item_value(a)).number() as c_int;
            }
            a = options_array_next(o, a);
        }
        Some(def)
    }
}

/// Points the palette's default table at what the `pane-colours` option of
/// `oo` holds.
pub unsafe fn options_load_pane_colours(oo: *mut options, p: *mut colour_palette) {
    unsafe { colour_palette_from_defaults(p, options_pane_colours(oo).as_ref()) }
}

pub unsafe fn options_is_array(o: *mut options_entry) -> c_int {
    unsafe {
        (*o).tableentry
            .is_some_and(|oe| oe.flags & OPTIONS_TABLE_IS_ARRAY != 0) as c_int
    }
}

pub unsafe fn options_is_string(o: *mut options_entry) -> c_int {
    unsafe { is_string(&*o) as c_int }
}

/// An option as the text a user would have written for it. `idx` of -1 asks
/// for the whole of an array, its values separated by spaces.
pub unsafe fn options_to_string(o: *mut options_entry, idx: c_int, numeric: c_int) -> CString {
    unsafe {
        if options_is_array(o) == 0 {
            return options_value_to_string(o, &raw mut (*o).value, numeric);
        }
        if idx != -1 {
            let a = options_array_item(o, idx as u_int);
            if a.is_null() {
                return c"".to_owned();
            }
            return options_value_to_string(o, &raw mut (*a).value, numeric);
        }

        let mut result = Vec::<u8>::new();
        for a in (*o)
            .array
            .values_mut()
            .map(|a| &raw mut *a)
            .collect::<Vec<_>>()
        {
            if !result.is_empty() {
                result.push(b' ');
            }
            let next = options_value_to_string(o, &raw mut (*a).value, numeric);
            result.extend_from_slice(next.as_bytes());
        }
        CString::new(result).expect("option values contain no NUL")
    }
}

/// The option name out of `name`, with the index in brackets after it read
/// into `idx`, which is -1 when there is none. Null if it is not a name.
pub unsafe fn options_parse(name: *const c_char, idx: *mut c_int) -> Option<CString> {
    unsafe {
        let bytes = CStr::from_ptr(name).to_bytes();
        if bytes.is_empty() {
            return None;
        }
        let Some(open) = bytes.iter().position(|&byte| byte == b'[') else {
            if !idx.is_null() {
                *idx = -1;
            }
            return CString::new(bytes).ok();
        };
        let close = bytes[open + 1..]
            .iter()
            .position(|&byte| byte == b']')
            .map(|at| open + 1 + at);
        let Some(close) = close else {
            return None;
        };
        if close + 1 != bytes.len() || !bytes[close - 1].is_ascii_digit() {
            return None;
        }
        let mut parsed_idx: c_int = 0;
        if sscanf(name.add(open), c"[%d]".as_ptr(), &raw mut parsed_idx) != 1 || parsed_idx < 0 {
            return None;
        }
        if !idx.is_null() {
            *idx = parsed_idx;
        }
        CString::new(&bytes[..open]).ok()
    }
}

/// The option `s` names, with its index read into `idx`. `only` looks in this
/// set alone rather than in the sets above it too.
pub unsafe fn options_parse_get(
    oo: *mut options,
    s: *const c_char,
    idx: *mut c_int,
    only: c_int,
) -> *mut options_entry {
    unsafe {
        let Some(name) = options_parse(s, idx) else {
            return null_mut::<options_entry>();
        };
        if only != 0 {
            options_get_only_ptr(oo, name.as_ptr())
        } else {
            options_get_ptr(oo, name.as_ptr())
        }
    }
}

/// The whole name of the option `s` is the start of, with its index read into
/// `idx`. Null if no option matches, or if more than one does, which sets
/// `ambiguous`. A user option is its own whole name.
pub unsafe fn options_match(
    s: *const c_char,
    idx: *mut c_int,
    ambiguous: *mut c_int,
) -> Option<CString> {
    unsafe {
        let parsed = options_parse(s, idx)?;
        if parsed.as_bytes().first() == Some(&b'@') {
            *ambiguous = 0;
            return Some(parsed);
        }

        let name = CStr::from_ptr(options_map_name(parsed.as_ptr()))
            .to_bytes()
            .to_vec();

        let mut found = None;
        for oe in table() {
            let entry = oe.name.to_bytes();
            if entry == name {
                found = Some(oe);
                break;
            }
            if entry.starts_with(&name) {
                if found.is_some() {
                    *ambiguous = 1;
                    return None;
                }
                found = Some(oe);
            }
        }
        match found {
            Some(oe) => Some(oe.name.to_owned()),
            None => {
                *ambiguous = 0;
                None
            }
        }
    }
}

/// The option whose whole name `s` is the start of, the way [`options_match`]
/// works it out.
pub unsafe fn options_match_get(
    oo: *mut options,
    s: *const c_char,
    idx: *mut c_int,
    only: c_int,
    ambiguous: *mut c_int,
) -> *mut options_entry {
    unsafe {
        let Some(name) = options_match(s, idx, ambiguous) else {
            return null_mut::<options_entry>();
        };
        *ambiguous = 0;

        if only != 0 {
            options_get_only_ptr(oo, name.as_ptr())
        } else {
            options_get_ptr(oo, name.as_ptr())
        }
    }
}

pub unsafe fn options_get_string(oo: *mut options, name: *const c_char) -> *const c_char {
    unsafe {
        let Some(o) = options_get(oo, name) else {
            fatalx(c"missing option %s".as_ptr(), fmt_args![name]);
        };
        if !is_string(&*o) {
            fatalx(c"option %s is not a string".as_ptr(), fmt_args![name]);
        }
        o.value.string()
    }
}

pub unsafe fn options_get_number(oo: *mut options, name: *const c_char) -> c_longlong {
    unsafe {
        let Some(o) = options_get(oo, name) else {
            fatalx(c"missing option %s".as_ptr(), fmt_args![name]);
        };
        if !is_number(&*o) {
            fatalx(c"option %s is not a number".as_ptr(), fmt_args![name]);
        }
        o.value.number()
    }
}

pub(crate) unsafe fn options_get_command(
    oo: *mut options,
    name: *const c_char,
) -> Option<CmdListRef> {
    unsafe {
        let Some(o) = options_get(oo, name) else {
            fatalx(c"missing option %s".as_ptr(), fmt_args![name]);
        };
        if !is_command(&*o) {
            fatalx(c"option %s is not a command".as_ptr(), fmt_args![name]);
        }
        o.value.commands()
    }
}

/// The option `name` in `oo`, added from the parent set's table entry if this
/// set has none of its own. `options_default` always answers an option, so the
/// C's check for one it did not is gone.
unsafe fn options_own(oo: *mut options, name: *const c_char) -> *mut options_entry {
    unsafe {
        let o = options_get_only_ptr(oo, name);
        if !o.is_null() {
            return o;
        }
        options_default(oo, options_parent_table_entry(oo, name))
    }
}

pub unsafe fn options_set_string(
    oo: *mut options,
    name: *const c_char,
    append: c_int,
    fmt: *const c_char,
    args: &[FmtArg],
) -> *mut options_entry {
    unsafe {
        let s = format_alloc(fmt, args);

        let mut o = options_get_only_ptr(oo, name);
        let value = if !o.is_null() && append != 0 && is_string(&*o) {
            /*
             * A user option has no table entry to name a separator, and the
             * one a table entry names is empty when it has none.
             */
            let mut separator = c"".as_ptr();
            if *name != b'@' as c_char {
                separator = cstr_or_null(table_of(o).separator);
                if separator.is_null() {
                    separator = c"".as_ptr();
                }
            }

            xasprintf(
                c"%s%s%s".as_ptr(),
                fmt_args![(*o).value.string(), separator, s.as_ptr()],
            )
        } else {
            s
        };

        if o.is_null() {
            o = if *name == b'@' as c_char {
                options_add(oo, name)
            } else {
                options_own(oo, name)
            };
        }
        if !is_string(&*o) {
            fatalx(c"option %s is not a string".as_ptr(), fmt_args![name]);
        }
        (*o).value = options_value::String(value);
        (*o).cached = 0;
        o
    }
}

pub unsafe fn options_set_number(
    oo: *mut options,
    name: *const c_char,
    value: c_longlong,
) -> *mut options_entry {
    unsafe {
        if *name == b'@' as c_char {
            fatalx(c"user option %s must be a string".as_ptr(), fmt_args![name]);
        }
        let o = options_own(oo, name);
        if !is_number(&*o) {
            fatalx(c"option %s is not a number".as_ptr(), fmt_args![name]);
        }
        (*o).value = options_value::Number(value);
        o
    }
}

pub(crate) unsafe fn options_set_command(
    oo: *mut options,
    name: *const c_char,
    value: Option<CmdListRef>,
) -> *mut options_entry {
    unsafe {
        if *name == b'@' as c_char {
            fatalx(c"user option %s must be a string".as_ptr(), fmt_args![name]);
        }
        let o = options_own(oo, name);
        if !is_command(&*o) {
            fatalx(c"option %s is not a command".as_ptr(), fmt_args![name]);
        }
        (*o).value = match value {
            Some(cmdlist) => options_value::Commands(cmdlist),
            None => options_value::None,
        };
        o
    }
}

/// The set a window option is to be read from or written to, which the
/// `-g` flag, the target and the current window between them decide.
unsafe fn options_window_scope(
    args: &args,
    fs: *mut cmd_find_state,
    oo: *mut *mut options,
    cause: &mut Option<CString>,
) -> c_int {
    unsafe {
        if args_has(args, b'g') != 0 {
            *oo = global_w_options;
            return OPTIONS_TABLE_WINDOW;
        }
        let wl = (*fs).winlink();
        if wl.is_null() {
            let target = args_get(args, b't');
            if target.is_null() {
                *cause = Some(xasprintf(c"no current window".as_ptr(), fmt_args![]));
            } else {
                *cause = Some(xasprintf(c"no such window: %s".as_ptr(), fmt_args![target]));
            }
            return OPTIONS_TABLE_NONE;
        }
        *oo = options_ptr(&(*(*wl).window()).options);
        OPTIONS_TABLE_WINDOW
    }
}

/// The set the option `name` belongs to, worked out from the scope the table
/// gives that name. A user option has no scope of its own, so the command
/// flags decide it instead.
///
/// Every option in the table is of the server, session, window, or window and
/// pane scope, so the C's arm for any other is gone.
pub unsafe fn options_scope_from_name(
    args: &args,
    window: c_int,
    name: *const c_char,
    fs: *mut cmd_find_state,
    oo: *mut *mut options,
    cause: &mut Option<CString>,
) -> c_int {
    unsafe {
        if *name == b'@' as c_char {
            return options_scope_from_flags(args, window, fs, oo, cause);
        }
        let wanted = CStr::from_ptr(name);
        let Some(oe) = table().iter().find(|oe| oe.name == wanted) else {
            *cause = Some(xasprintf(c"unknown option: %s".as_ptr(), fmt_args![name]));
            return OPTIONS_TABLE_NONE;
        };

        let target = args_get(args, b't');
        match oe.scope {
            OPTIONS_TABLE_SERVER => {
                *oo = global_options;
                OPTIONS_TABLE_SERVER
            }
            OPTIONS_TABLE_SESSION => {
                if args_has(args, b'g') != 0 {
                    *oo = global_s_options;
                    return OPTIONS_TABLE_SESSION;
                }
                let s = (*fs).session();
                if s.is_null() {
                    if target.is_null() {
                        *cause = Some(xasprintf(c"no current session".as_ptr(), fmt_args![]));
                    } else {
                        *cause = Some(xasprintf(
                            c"no such session: %s".as_ptr(),
                            fmt_args![target],
                        ));
                    }
                    return OPTIONS_TABLE_NONE;
                }
                *oo = session_options(s);
                OPTIONS_TABLE_SESSION
            }
            scope
                if scope == OPTIONS_TABLE_WINDOW | OPTIONS_TABLE_PANE
                    && args_has(args, b'p') != 0 =>
            {
                let wp = (*fs).pane();
                if wp.is_null() {
                    if target.is_null() {
                        *cause = Some(xasprintf(c"no current pane".as_ptr(), fmt_args![]));
                    } else {
                        *cause = Some(xasprintf(c"no such pane: %s".as_ptr(), fmt_args![target]));
                    }
                    return OPTIONS_TABLE_NONE;
                }
                *oo = options_ptr(&(*wp).options);
                OPTIONS_TABLE_PANE
            }
            _ => options_window_scope(args, fs, oo, cause),
        }
    }
}

/// The set a user option belongs to, worked out from the command flags alone.
pub unsafe fn options_scope_from_flags(
    args: &args,
    window: c_int,
    fs: *mut cmd_find_state,
    oo: *mut *mut options,
    cause: &mut Option<CString>,
) -> c_int {
    unsafe {
        let target = args_get(args, b't');
        if args_has(args, b's') != 0 {
            *oo = global_options;
            return OPTIONS_TABLE_SERVER;
        }
        if args_has(args, b'p') != 0 {
            let wp = (*fs).pane();
            if wp.is_null() {
                if target.is_null() {
                    *cause = Some(xasprintf(c"no current pane".as_ptr(), fmt_args![]));
                } else {
                    *cause = Some(xasprintf(c"no such pane: %s".as_ptr(), fmt_args![target]));
                }
                return OPTIONS_TABLE_NONE;
            }
            *oo = options_ptr(&(*wp).options);
            return OPTIONS_TABLE_PANE;
        }
        if window != 0 || args_has(args, b'w') != 0 {
            return options_window_scope(args, fs, oo, cause);
        }
        if args_has(args, b'g') != 0 {
            *oo = global_s_options;
            return OPTIONS_TABLE_SESSION;
        }
        let s = (*fs).session();
        if s.is_null() {
            if target.is_null() {
                *cause = Some(xasprintf(c"no current session".as_ptr(), fmt_args![]));
            } else {
                *cause = Some(xasprintf(
                    c"no such session: %s".as_ptr(),
                    fmt_args![target],
                ));
            }
            return OPTIONS_TABLE_NONE;
        }
        *oo = session_options(s);
        OPTIONS_TABLE_SESSION
    }
}

/// The style the option `name` holds, read once and kept unless the string is
/// a format, which is expanded against `ft` every time.
pub unsafe fn options_string_to_style(
    oo: *mut options,
    name: *const c_char,
    ft: Option<&mut format_tree>,
) -> *mut style {
    unsafe {
        let o = options_get_ptr(oo, name);
        if o.is_null() || !is_string(&*o) {
            return null_mut::<style>();
        }
        if (*o).cached != 0 {
            return &raw mut (*o).style;
        }

        let s = (*o).value.string();
        log_debug(
            c"%s: %s is '%s'".as_ptr(),
            fmt_args![c"options_string_to_style".as_ptr(), name, s],
        );
        style_set(&mut (*o).style, &grid_default_cell);
        (*o).cached = strstr(s, c"#{".as_ptr()).is_null() as c_int;

        if let Some(ft) = ft
            && (*o).cached == 0
        {
            let expanded = format_expand(ft, CStr::from_ptr(s));
            let answer = style_parse(&mut (*o).style, &grid_default_cell, expanded.as_bytes());
            if answer != 0 {
                return null_mut::<style>();
            }
        } else if style_parse(
            &mut (*o).style,
            &grid_default_cell,
            CStr::from_ptr(s).to_bytes(),
        ) != 0
        {
            return null_mut::<style>();
        }
        &raw mut (*o).style
    }
}

/// Whether a value is one the table would let the option hold: a shell that
/// can be run, a value the entry's pattern matches, and a style that parses.
unsafe fn options_from_string_check(
    oe: *const options_table_entry_t,
    value: *const c_char,
    cause: &mut Option<CString>,
) -> c_int {
    unsafe {
        if oe.is_null() {
            return 0;
        }
        if (*oe).name == c"default-shell" && checkshell(value) == 0 {
            *cause = Some(xasprintf(
                c"not a suitable shell: %s".as_ptr(),
                fmt_args![value],
            ));
            return -1;
        }
        if let Some(pattern) = (*oe).pattern
            && fnmatch(pattern.as_ptr(), value, 0) != 0
        {
            *cause = Some(xasprintf(
                c"value is invalid: %s".as_ptr(),
                fmt_args![value],
            ));
            return -1;
        }
        if (*oe).flags & OPTIONS_TABLE_IS_STYLE != 0 && strstr(value, c"#{".as_ptr()).is_null() {
            let mut sy = style::default();
            if style_parse(
                &mut sy,
                &grid_default_cell,
                CStr::from_ptr(value).to_bytes(),
            ) != 0
            {
                *cause = Some(xasprintf(c"invalid style: %s".as_ptr(), fmt_args![value]));
                return -1;
            }
        }
        0
    }
}

/// Sets a flag option from the words for on and off. No value at all turns it
/// over.
unsafe fn options_from_string_flag(
    oo: *mut options,
    name: *const c_char,
    value: *const c_char,
    cause: &mut Option<CString>,
) -> c_int {
    unsafe {
        let flag = if value.is_null() || *value == 0 {
            (options_get_number(oo, name) == 0) as c_int
        } else {
            let word = CStr::from_ptr(value).to_bytes();
            if word == b"1" || word.eq_ignore_ascii_case(b"on") || word.eq_ignore_ascii_case(b"yes")
            {
                1
            } else if word == b"0"
                || word.eq_ignore_ascii_case(b"off")
                || word.eq_ignore_ascii_case(b"no")
            {
                0
            } else {
                *cause = Some(xasprintf(c"bad value: %s".as_ptr(), fmt_args![value]));
                return -1;
            }
        };
        options_set_number(oo, name, flag as c_longlong);
        0
    }
}

/// Which of an option's choices `value` is, or -1 if it is none of them.
pub unsafe fn options_find_choice(
    oe: *const options_table_entry_t,
    value: *const c_char,
    cause: &mut Option<CString>,
) -> c_int {
    unsafe {
        let wanted = CStr::from_ptr(value);
        let mut choice = -1;
        for (n, name) in choices_of(oe).iter().enumerate() {
            if *name == wanted {
                choice = n as c_int;
            }
        }
        if choice == -1 {
            *cause = Some(xasprintf(c"unknown value: %s".as_ptr(), fmt_args![value]));
            return -1;
        }
        choice
    }
}

/// Sets a choice option. No value at all turns over the first two choices and
/// leaves any other where it is.
unsafe fn options_from_string_choice(
    oe: *const options_table_entry_t,
    oo: *mut options,
    name: *const c_char,
    value: *const c_char,
    cause: &mut Option<CString>,
) -> c_int {
    unsafe {
        let choice = if value.is_null() {
            let choice = options_get_number(oo, name) as c_int;
            if choice < 2 {
                (choice == 0) as c_int
            } else {
                choice
            }
        } else {
            let choice = options_find_choice(oe, value, cause);
            if choice < 0 {
                return -1;
            }
            choice
        };
        options_set_number(oo, name, choice as c_longlong);
        0
    }
}

/// Sets the option `name` from the text a user wrote for it, answering -1 and
/// a reason for a value the option cannot hold. A string that is turned down
/// leaves the option with what it had.
pub unsafe fn options_from_string(
    oo: *mut options,
    oe: *const options_table_entry_t,
    name: *const c_char,
    value: *const c_char,
    append: c_int,
    cause: &mut Option<CString>,
) -> c_int {
    unsafe {
        let type_0 = if oe.is_null() {
            if *name != b'@' as c_char {
                *cause = Some(xasprintf(c"bad option name".as_ptr(), fmt_args![]));
                return -1;
            }
            OPTIONS_TABLE_STRING
        } else {
            if value.is_null()
                && (*oe).type_0 != OPTIONS_TABLE_FLAG
                && (*oe).type_0 != OPTIONS_TABLE_CHOICE
            {
                *cause = Some(xasprintf(c"empty value".as_ptr(), fmt_args![]));
                return -1;
            }
            (*oe).type_0
        };

        match type_0 {
            OPTIONS_TABLE_STRING => {
                let old = CStr::from_ptr(options_get_string(oo, name)).to_owned();
                options_set_string(oo, name, append, c"%s".as_ptr(), fmt_args![value]);
                let new = options_get_string(oo, name);
                if options_from_string_check(oe, new, cause) != 0 {
                    options_set_string(oo, name, 0, c"%s".as_ptr(), fmt_args![old.as_ptr()]);
                    return -1;
                }
                0
            }
            OPTIONS_TABLE_NUMBER => {
                let Ok(number) = strtonum(
                    value,
                    (*oe).minimum as c_longlong,
                    (*oe).maximum as c_longlong,
                )
                .inspect_err(|errstr| {
                    *cause = Some(xasprintf(
                        c"value is %s: %s".as_ptr(),
                        fmt_args![errstr.as_ptr(), value],
                    ));
                }) else {
                    return -1;
                };
                options_set_number(oo, name, number);
                0
            }
            OPTIONS_TABLE_KEY => {
                let key = key_string_lookup_string(value);
                if key == KEYC_UNKNOWN as key_code {
                    *cause = Some(xasprintf(c"bad key: %s".as_ptr(), fmt_args![value]));
                    return -1;
                }
                options_set_number(oo, name, key as c_longlong);
                0
            }
            OPTIONS_TABLE_COLOUR => {
                let number = colour_fromstring(value) as c_longlong;
                if number == -1 {
                    *cause = Some(xasprintf(c"bad colour: %s".as_ptr(), fmt_args![value]));
                    return -1;
                }
                options_set_number(oo, name, number);
                0
            }
            OPTIONS_TABLE_FLAG => options_from_string_flag(oo, name, value, cause),
            OPTIONS_TABLE_CHOICE => options_from_string_choice(oe, oo, name, value, cause),
            _ => {
                let mut pr = cmd_parse_from_string(value, null_mut::<cmd_parse_input>());
                if pr.status != CMD_PARSE_SUCCESS {
                    *cause = pr.error.take();
                    return -1;
                }
                options_set_command(oo, name, pr.cmdlist.take());
                0
            }
        }
    }
}

/// The windows the server has, in id order.
fn each_window() -> impl Iterator<Item = WindowRef> {
    let ids: Vec<u_int> = windows.map().keys().copied().collect();
    ids.into_iter().filter_map(window_find_by_id_ref)
}

/// The sessions the server has, in name order.
fn each_session() -> impl Iterator<Item = *mut session> {
    let mut current = null_mut::<session>();
    let mut started = false;
    ::core::iter::from_fn(move || unsafe {
        current = if started {
            sessions_after(current)
        } else {
            started = true;
            sessions_first()
        };
        (!current.is_null()).then_some(current)
    })
}

/// Tells whatever an option reaches that it has changed. Every option ends by
/// having the status caches, the window sizes and the attached clients brought
/// up to date, whether or not it is one of the names below.
pub unsafe fn options_push_changes(name: *const c_char) {
    unsafe {
        log_debug(
            c"%s: %s".as_ptr(),
            fmt_args![c"options_push_changes".as_ptr(), name],
        );
        let named = CStr::from_ptr(name);

        if named == c"automatic-rename" {
            for w_ref in each_window() {
                let w = w_ref.as_ptr();
                if !window_get_active(w).is_null()
                    && options_get_number(options_ptr(&(*w).options), name) != 0
                {
                    (*window_get_active(w)).flags |= PANE_CHANGED;
                }
            }
        }
        if named == c"cursor-colour" || named == c"cursor-style" {
            for wp in pane_walk() {
                window_pane_default_cursor(wp);
            }
        }
        if named == c"fill-character" {
            for w_ref in each_window() {
                let w = w_ref.as_ptr();
                window_set_fill_character(w);
            }
        }
        if named == c"key-table" {
            for c in client_walk() {
                server_client_set_key_table(c, null::<c_char>());
            }
        }
        if named == c"user-keys" {
            for c in client_walk() {
                if (*c).tty.flags & TTY_OPENED != 0 {
                    tty_keys_build(&raw mut (*c).tty);
                }
            }
        }
        if named == c"status" || named == c"status-interval" {
            status_timer_start_all();
        }
        if named == c"monitor-silence" {
            alerts_reset_all();
        }
        if named == c"window-style" || named == c"window-active-style" {
            for wp in pane_walk() {
                (*wp).flags |= PANE_STYLECHANGED | PANE_THEMECHANGED;
            }
        }
        if *name == b'@' as c_char {
            for wp in pane_walk() {
                (*wp).flags |= PANE_STYLECHANGED;
            }
        }
        if named == c"pane-colours" {
            for wp in pane_walk() {
                options_load_pane_colours(options_ptr(&(*wp).options), &raw mut (*wp).palette);
            }
        }
        if named == c"pane-border-status"
            || named == c"pane-scrollbars"
            || named == c"pane-scrollbars-position"
        {
            for w_ref in each_window() {
                let w = w_ref.as_ptr();
                (*w).sb =
                    options_get_number(options_ptr(&(*w).options), c"pane-scrollbars".as_ptr())
                        as c_int;
                (*w).sb_pos = options_get_number(
                    options_ptr(&(*w).options),
                    c"pane-scrollbars-position".as_ptr(),
                ) as c_int;
                layout_fix_panes(w, null_mut::<window_pane>());
            }
        }
        if named == c"pane-scrollbars-style" {
            for wp in pane_walk() {
                style_set_scrollbar_style_from_option(
                    &mut (*wp).scrollbar_style,
                    options_ptr(&(*wp).options),
                );
            }
            for w_ref in each_window() {
                let w = w_ref.as_ptr();
                layout_fix_panes(w, null_mut::<window_pane>());
            }
        }
        if named == c"codepoint-widths" {
            utf8_update_width_cache(options_codepoint_widths(global_options));
        }
        if named == c"input-buffer-size" {
            input_set_buffer_size(options_get_number(global_options, name) as size_t);
        }
        if named == c"history-limit" {
            for s in each_session() {
                session_update_history(s);
            }
        }

        for s in each_session() {
            status_update_cache(s);
        }
        recalculate_sizes();
        for c in client_walk() {
            if !(*c).session.is_null() {
                server_redraw_client(c);
            }
        }
    }
}

/// Takes an option away, or puts it back to what the table gives it when the
/// set is one of the global ones. `idx` of anything but -1 takes one value of
/// an array away instead.
pub unsafe fn options_remove_or_default(
    o: *mut options_entry,
    idx: c_int,
    cause: &mut Option<CString>,
) -> c_int {
    unsafe {
        let oo = (*o).owner;
        if idx != -1 {
            if options_array_set(o, idx as u_int, null::<c_char>(), 0, cause) != 0 {
                return -1;
            }
            return 0;
        }
        if (*o).tableentry.is_some()
            && (oo == global_options || oo == global_s_options || oo == global_w_options)
        {
            options_default(oo, table_of(o));
        } else {
            options_remove(o);
        }
        0
    }
}

#[cfg(test)]
#[path = "../tests/test_options.rs"]
mod tests;
