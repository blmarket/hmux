use crate::args::RustArguments;
use crate::cmd::CmdListRef;
use crate::window_scrollbar::WindowScrollbarState;

use super::table::{options_other_names, options_table};
use crate::WindowPane;
use crate::alerts::alerts_reset_all;
use crate::cmd::{CMD_PARSE_SUCCESS, cmd_parse_from_string};
use crate::compat::strtonum;
use crate::ffi::fnmatch;
use crate::fmt_args;
use crate::fmt_engine::{FmtArg, format_alloc};
use crate::format::format_expand;
use crate::grid::grid_default_cell;
use crate::input::input_set_buffer_size;

use crate::log::{fatalx, log_debug};
use crate::pane_scrollbar_style::PaneScrollbarStyleState;
use crate::resize::recalculate_sizes;
use crate::server::client_walk;
use crate::server::server_client_set_key_table;
use crate::server::server_redraw_client;
use crate::session::SESSIONS_FIELD;
use crate::status::status_timer_start_all;
use crate::style::{ColourEngine, RustColourEngine};
use crate::style::{RustStyleCodec, StyleCodec, pane_scrollbar_style_from_option};
use crate::text::utf8_update_width_cache;
use crate::text::{KEYC_UNKNOWN, KeyStringCodec, RustKeyStringCodec};
use crate::tmux::{checkshell, global_options, global_s_options, global_w_options};
use crate::tty::tty_keys_build;
pub use crate::types::*;
use crate::window::pane_walk;
use crate::window::{WINDOWS, window_pane_default_cursor, window_set_fill_character};
use crate::xmalloc::xasprintf;
use ::core::ffi::{CStr, c_int, c_longlong};
use ::std::ffi::CString;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub use crate::consts::{
    OPTIONS_TABLE_CHOICE, OPTIONS_TABLE_COLOUR, OPTIONS_TABLE_COMMAND, OPTIONS_TABLE_FLAG,
    OPTIONS_TABLE_IS_ARRAY, OPTIONS_TABLE_IS_STYLE, OPTIONS_TABLE_KEY, OPTIONS_TABLE_NONE,
    OPTIONS_TABLE_NUMBER, OPTIONS_TABLE_PANE, OPTIONS_TABLE_SERVER, OPTIONS_TABLE_SESSION,
    OPTIONS_TABLE_STRING, OPTIONS_TABLE_WINDOW, PANE_CHANGED, PANE_STYLECHANGED, PANE_THEMECHANGED,
    TTY_OPENED,
};

/// An opaque shared handle to an option store.
#[derive(Clone)]
pub struct RustOptionsRef(Rc<RefCell<RustOptions>>);

impl RustOptionsRef {
    /// Whether both handles refer to the same option store.
    pub fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl PartialEq for RustOptionsRef {
    fn eq(&self, other: &Self) -> bool {
        self.ptr_eq(other)
    }
}

impl Eq for RustOptionsRef {}

impl std::fmt::Debug for RustOptionsRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("RustOptionsRef")
            .field(&Rc::as_ptr(&self.0))
            .finish()
    }
}

struct RustOptions {
    tree: options_tree,
    parent: Option<RustOptionsRef>,
}

/// The options of one set, by name.
type options_tree = std::collections::BTreeMap<CString, Box<options_entry>>;

/// One option: its name, the table entry that says what kind of value it
/// holds, the value itself, and the style that value was last read as. An
/// array option stores its elements in the array variant of `value`.
#[repr(C)]
pub struct options_entry {
    /// The owning store, held weakly to avoid an ownership cycle.
    owner: Weak<RefCell<RustOptions>>,
    name: CString,
    tableentry: Option<&'static options_table_entry_t>,
    value: options_value,
    cached: c_int,
    style: style,
}

/// One value of an array option.
#[repr(C)]
pub struct options_array_item_t {
    index: u_int,
    value: options_value,
}

/// The entries of the option table.
fn table() -> &'static [options_table_entry_t] {
    &options_table
}

/// The choices a choice option accepts. Only ever asked of one that has them.
fn choices_of(oe: &options_table_entry_t) -> &'static [&'static CStr] {
    oe.choices.expect("a choice option lists its choices")
}

/// The name an option used to be spelled, mapped to the one it has now.
fn options_map_name(name: &CStr) -> &CStr {
    options_other_names
        .iter()
        .find(|map| map.from == name)
        .map_or(name, |map| map.to)
}

/// The table entry the parent set has for `s`, which is what an option added
/// to a set below it is made from.
fn options_parent_table_entry(oo: &RustOptionsRef, s: &CStr) -> &'static options_table_entry_t {
    {
        let Some(parent) = options_get_parent(oo) else {
            fatalx(c"no parent options for %s", fmt_args![s.as_ptr()]);
        };
        with_entry(&parent, s, false, |entry| {
            let Some(entry) = entry else {
                fatalx(c"%s not in parent options", fmt_args![s]);
            };
            entry.tableentry.expect("parent option has a table entry")
        })
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

/// One value of `o` as the text a user would have written for it. `numeric`
/// asks for a flag as its number rather than as `on` or `off`.
unsafe fn options_value_to_string(
    o: &options_entry,
    ov: &options_value,
    numeric: c_int,
) -> CString {
    unsafe {
        if is_command(o) {
            return ov
                .cmdlist()
                .map_or_else(CString::default, |cmdlist| cmdlist.print(0));
        }
        if is_number(o) {
            return match table_of(o).type_0 {
                OPTIONS_TABLE_NUMBER => xasprintf(c"%lld", fmt_args![ov.number()]),
                OPTIONS_TABLE_KEY => RustKeyStringCodec.format_key(ov.number() as key_code, false),
                OPTIONS_TABLE_COLOUR => RustColourEngine.to_string(ov.number() as c_int),
                OPTIONS_TABLE_FLAG if numeric != 0 => xasprintf(c"%lld", fmt_args![ov.number()]),
                OPTIONS_TABLE_FLAG if ov.number() != 0 => c"on".to_owned(),
                OPTIONS_TABLE_FLAG => c"off".to_owned(),
                _ => choices_of(table_of(o))[ov.number() as usize].to_owned(),
            };
        }
        ov.string().to_owned()
    }
}

/// A new, empty set retaining its fallback parent.
pub(super) fn options_create(parent: Option<&RustOptionsRef>) -> RustOptionsRef {
    RustOptionsRef(Rc::new(RefCell::new(RustOptions {
        tree: options_tree::new(),
        parent: parent.cloned(),
    })))
}

pub(super) fn options_get_parent(oo: &RustOptionsRef) -> Option<RustOptionsRef> {
    oo.0.borrow().parent.clone()
}

pub(super) fn options_set_parent(oo: &RustOptionsRef, parent: Option<&RustOptionsRef>) {
    let mut walk = parent.cloned();
    while let Some(ancestor) = walk {
        assert!(!ancestor.ptr_eq(oo), "cyclic option parent");
        walk = options_get_parent(&ancestor);
    }
    oo.0.borrow_mut().parent = parent.cloned();
}

pub(super) fn local_names(oo: &RustOptionsRef) -> Vec<CString> {
    oo.0.borrow().tree.keys().cloned().collect()
}

/// Adds `oe` to the set with no value in it yet.
pub(super) fn options_empty(oo: &RustOptionsRef, oe: &'static options_table_entry_t) {
    let mut entry = new_entry(oo, oe.name);
    entry.tableentry = Some(oe);
    replace_entry(oo, entry);
}

/// Initializes a default outside the store borrow so command parsing can read options.
pub(super) unsafe fn options_default(oo: &RustOptionsRef, oe: &'static options_table_entry_t) {
    unsafe {
        options_empty(oo, oe);
        let mut initialized = new_entry(oo, oe.name);
        initialized.tableentry = Some(oe);
        if oe.flags & OPTIONS_TABLE_IS_ARRAY != 0 {
            if let Some(default_arr) = oe.default_arr {
                for (i, value) in default_arr.iter().enumerate() {
                    options_array_set(&mut initialized, i as u_int, Some(value), 0, &mut None);
                }
            } else {
                options_array_assign(&mut initialized, oe.default_str, &mut None);
            }
        } else {
            initialized.value = match oe.type_0 {
                OPTIONS_TABLE_STRING => {
                    options_value::String(oe.default_str.unwrap_or(c"").to_owned())
                }
                OPTIONS_TABLE_COMMAND => {
                    let mut parsed = cmd_parse_from_string(oe.default_str.unwrap_or(c""), None);
                    if parsed.status == CMD_PARSE_SUCCESS {
                        parsed
                            .cmdlist
                            .take()
                            .map_or(options_value::None, options_value::Commands)
                    } else {
                        options_value::None
                    }
                }
                _ => options_value::Number(oe.default_num),
            };
        }
        with_entry_mut(oo, oe.name, true, |entry| {
            let entry = entry.expect("default entry was initialized");
            entry.value = initialized.value;
        });
    }
}

/// The value the table gives `oe`, as the text a user would have written.
pub(super) fn options_default_to_string(oe: &options_table_entry_t) -> CString {
    {
        match oe.type_0 {
            OPTIONS_TABLE_STRING | OPTIONS_TABLE_COMMAND => {
                oe.default_str.unwrap_or(c"").to_owned()
            }
            OPTIONS_TABLE_NUMBER => xasprintf(c"%lld", fmt_args![oe.default_num]),
            OPTIONS_TABLE_KEY => RustKeyStringCodec.format_key(oe.default_num as key_code, false),
            OPTIONS_TABLE_COLOUR => RustColourEngine.to_string(oe.default_num as c_int),
            OPTIONS_TABLE_FLAG => {
                if oe.default_num != 0 {
                    c"on".to_owned()
                } else {
                    c"off".to_owned()
                }
            }
            OPTIONS_TABLE_CHOICE => choices_of(oe)[oe.default_num as usize].to_owned(),
            _ => fatalx(c"unknown option type", fmt_args![]),
        }
    }
}

fn new_entry(oo: &RustOptionsRef, name: &CStr) -> options_entry {
    options_entry {
        owner: Rc::downgrade(&oo.0),
        name: name.to_owned(),
        tableentry: None,
        value: options_value::None,
        cached: 0,
        style: style::default(),
    }
}

fn replace_entry(oo: &RustOptionsRef, entry: options_entry) {
    let mut store = oo.0.borrow_mut();
    if !store.tree.contains_key(entry.name.as_c_str()) {
        store.tree.remove(options_map_name(&entry.name));
    }
    store.tree.insert(entry.name.clone(), Box::new(entry));
}

/// Adds an option named `name` to the set, taking away whatever was there.
fn options_add(oo: &RustOptionsRef, name: &CStr) {
    replace_entry(oo, new_entry(oo, name));
}

/// The name the entry is filed under.
pub(super) fn options_name(o: &options_entry) -> &CStr {
    &o.name
}

pub(super) fn options_owner(o: &options_entry) -> RustOptionsRef {
    RustOptionsRef(o.owner.upgrade().expect("entry store is owned"))
}

pub(super) fn options_table_entry(
    o: Option<&options_entry>,
) -> Option<&'static options_table_entry_t> {
    o.and_then(|o| o.tableentry)
}

/// The table entry an option was made from. Only ever asked of one that has
/// it, which the `is_*` tests above have already established.
fn table_of(o: &options_entry) -> &'static options_table_entry_t {
    o.tableentry.expect("the option comes from the table")
}

/// The value at `idx`, inserting an empty value when the slot is absent.
fn options_array_slot(o: &mut options_entry, idx: u_int) -> &mut options_array_item_t {
    o.value.array_mut().entry(idx).or_insert(options_array_item_t {
        index: idx,
        value: options_value::default(),
    })
}

/// Empties an array option.
pub(super) fn options_array_clear(o: &mut options_entry) {
    {
        if options_is_array(o) == 0 {
            return;
        }
        o.value.array_mut().clear();
    }
}

pub(super) fn array_indices(o: &options_entry) -> Vec<u_int> {
    o.value.array().into_iter().flat_map(|array| array.keys().copied()).collect()
}

pub(super) fn array_value_at(o: &options_entry, idx: u_int) -> Option<&options_value> {
    o.value.array()?.get(&idx).map(|item| &item.value)
}

pub(super) fn array_item_value(item: &options_array_item_t) -> &options_value {
    &item.value
}

/// Puts `value` at `idx` of an array option, or takes that index away when
/// there is no value. `append` adds to a string already there.
pub(super) unsafe fn options_array_set(
    o: &mut options_entry,
    idx: u_int,
    value: Option<&CStr>,
    append: c_int,
    cause: &mut Option<CString>,
) -> c_int {
    unsafe {
        if options_is_array(o) == 0 {
            *cause = Some(c"not an array".to_owned());
            return -1;
        }
        let Some(value) = value else {
            o.value.array_mut().remove(&idx);
            return 0;
        };
        if is_command(o) {
            let mut pr = cmd_parse_from_string(value, None);
            if pr.status != CMD_PARSE_SUCCESS {
                if let Some(error) = pr.error.take() {
                    *cause = Some(error);
                }
                return -1;
            }
            let slot = options_array_slot(o, idx);
            slot.value = match pr.cmdlist.take() {
                Some(cmdlist) => options_value::Commands(cmdlist),
                None => options_value::None,
            };
            return 0;
        }
        if is_string(o) {
            let new = if append != 0
                && let Some(item) = o.value.array().and_then(|array| array.get(&idx))
            {
                xasprintf(c"%s%s", fmt_args![item.value.string(), value])
            } else {
                value.to_owned()
            };
            let slot = options_array_slot(o, idx);
            slot.value = options_value::String(new);
            return 0;
        }
        if table_of(o).type_0 == OPTIONS_TABLE_COLOUR {
            let number = RustColourEngine.from_string(value) as c_longlong;
            if number == -1 {
                *cause = Some(xasprintf(c"bad colour: %s", fmt_args![value]));
                return -1;
            }
            options_array_slot(o, idx).value = options_value::Number(number);
            return 0;
        }

        *cause = Some(c"wrong array type".to_owned());
        -1
    }
}

/// Puts a whole list into an array option, one value per separator the table
/// names. An option whose separator is empty takes the string as one value.
pub(super) unsafe fn options_array_assign(
    o: &mut options_entry,
    s: Option<&CStr>,
    cause: &mut Option<CString>,
) -> c_int {
    unsafe {
        let separators = table_of(&*o).separator.unwrap_or(c" ,").to_bytes();
        let Some(s) = s else {
            return 0;
        };
        let bytes = s.to_bytes();
        if bytes.is_empty() {
            return 0;
        }

        let first_free = |o: &options_entry| {
            let mut i = 0;
            while o.value.array().is_some_and(|array| array.contains_key(&i)) {
                i += 1;
            }
            i
        };

        if separators.is_empty() {
            let i = first_free(o);
            return options_array_set(&mut *o, i, Some(s), 0, cause);
        }

        for next in bytes.split(|byte| separators.contains(byte)) {
            if next.is_empty() {
                continue;
            }
            let i = first_free(o);
            let value = CString::new(next).expect("no NUL inside");
            if options_array_set(&mut *o, i, Some(&value), 0, cause) != 0 {
                return -1;
            }
        }
        0
    }
}

pub(super) fn options_array_item_index(a: &options_array_item_t) -> u_int {
    a.index
}

pub(super) fn options_array_item_command(a: &options_array_item_t) -> Option<CmdListRef> {
    a.value.commands()
}

/// The `codepoint-widths` specs held in `oo`, in array order, for
/// [`utf8_update_width_cache`] to apply.
pub(super) fn options_codepoint_widths(oo: &RustOptionsRef) -> Vec<CString> {
    with_entry(oo, c"codepoint-widths", false, |entry| {
        let entry = entry.expect("codepoint-widths is initialized");
        if options_is_array(entry) == 0 {
            return Vec::new();
        }
        entry.value.array().into_iter().flat_map(|array| array.values())
            .map(|item| item.value.string().to_owned())
            .collect()
    })
}

/// The `pane-colours` option of `oo` as a default palette, or `None` when the
/// option holds no entries at all.
pub(super) fn options_pane_colours(oo: &RustOptionsRef) -> Option<[c_int; 256]> {
    with_entry(oo, c"pane-colours", false, |entry| {
        let entry = entry.expect("pane-colours is initialized");
        if options_is_array(entry) == 0 || entry.value.array().is_none_or(|array| array.is_empty()) {
            return None;
        }
        let mut colours = [-1; 256];
        for item in entry.value.array().into_iter().flat_map(|array| array.values()) {
            if let Some(colour) = colours.get_mut(item.index as usize) {
                *colour = item.value.number() as c_int;
            }
        }
        Some(colours)
    })
}

/// Points the palette's default table at what the `pane-colours` option of
/// `oo` holds.
pub(super) fn options_load_pane_colours(oo: &RustOptionsRef, p: Option<&mut colour_palette>) {
    RustColourEngine.set_palette_defaults(p, options_pane_colours(oo).as_ref())
}

pub(super) fn options_is_array(o: &options_entry) -> c_int {
    {
        o.tableentry
            .is_some_and(|oe| oe.flags & OPTIONS_TABLE_IS_ARRAY != 0) as c_int
    }
}

pub(super) fn options_is_string(o: &options_entry) -> c_int {
    is_string(o) as c_int
}

/// An option as the text a user would have written for it. `idx` of -1 asks
/// for the whole of an array, its values separated by spaces.
pub(super) unsafe fn options_to_string(o: &options_entry, idx: c_int, numeric: c_int) -> CString {
    unsafe {
        if options_is_array(o) == 0 {
            return options_value_to_string(o, &o.value, numeric);
        }
        if idx != -1 {
            return o.value.array().and_then(|array| array.get(&(idx as u_int)))
                .map_or_else(CString::default, |item| {
                    options_value_to_string(o, &item.value, numeric)
                });
        }
        let mut result = Vec::new();
        for item in o.value.array().into_iter().flat_map(|array| array.values()) {
            if !result.is_empty() {
                result.push(b' ');
            }
            let next = options_value_to_string(o, &item.value, numeric);
            result.extend_from_slice(next.as_bytes());
        }
        CString::new(result).expect("option values contain no NUL")
    }
}

/// The option name out of `name`, with the index in brackets after it read
/// into `idx`, which is -1 when there is none. Null if it is not a name.
pub(super) fn options_parse(name: &CStr, idx: &mut c_int) -> Option<CString> {
    let bytes = name.to_bytes();
    if bytes.is_empty() {
        return None;
    }
    let Some(open) = bytes.iter().position(|&byte| byte == b'[') else {
        *idx = -1;
        return CString::new(bytes).ok();
    };
    let close = bytes[open + 1..]
        .iter()
        .position(|&byte| byte == b']')
        .map(|at| open + 1 + at);
    let close = close?;
    if close + 1 != bytes.len() || !bytes[close - 1].is_ascii_digit() {
        return None;
    }
    let index_str = core::str::from_utf8(&bytes[open + 1..close]).ok()?;
    let parsed_idx = index_str.parse::<c_int>().ok()?;
    if parsed_idx < 0 {
        return None;
    }
    *idx = parsed_idx;
    CString::new(&bytes[..open]).ok()
}

/// The whole name of the option `s` is the start of, with its index read into
/// `idx`. Null if no option matches, or if more than one does, which sets
/// `ambiguous`. A user option is its own whole name.
pub(super) fn options_match(s: &CStr, idx: &mut c_int, ambiguous: &mut c_int) -> Option<CString> {
    let parsed = options_parse(s, idx)?;
    if parsed.as_bytes().first() == Some(&b'@') {
        *ambiguous = 0;
        return Some(parsed);
    }

    let name = options_map_name(&parsed).to_bytes().to_vec();

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

pub(super) fn with_entry<R>(
    oo: &RustOptionsRef,
    name: &CStr,
    local: bool,
    read: impl FnOnce(Option<&options_entry>) -> R,
) -> R {
    let mut current = Some(oo.clone());
    while let Some(owner) = current {
        let store = owner.0.borrow();
        if let Some(entry) = store
            .tree
            .get(name)
            .or_else(|| store.tree.get(options_map_name(name)))
        {
            return read(Some(entry));
        }
        current = if local { None } else { store.parent.clone() };
    }
    read(None)
}

pub(super) fn with_entry_mut<R>(
    oo: &RustOptionsRef,
    name: &CStr,
    local: bool,
    write: impl FnOnce(Option<&mut options_entry>) -> R,
) -> R {
    let mut current = Some(oo.clone());
    while let Some(owner) = current {
        let mut store = owner.0.borrow_mut();
        let key = if store.tree.contains_key(name) {
            name
        } else {
            options_map_name(name)
        };
        if let Some(entry) = store.tree.get_mut(key) {
            return write(Some(entry));
        }
        current = if local { None } else { store.parent.clone() };
    }
    write(None)
}

pub(super) fn options_string_ref(oo: &RustOptionsRef, name: &CStr) -> Rc<CStr> {
    with_entry(oo, name, false, |entry| {
        let Some(entry) = entry else {
            fatalx(c"missing option %s", fmt_args![name]);
        };
        if !is_string(entry) {
            fatalx(c"option %s is not a string", fmt_args![name]);
        }
        match &entry.value.0 {
            OptionValue::String(value) => value.clone(),
            _ => panic!("not a string option"),
        }
    })
}

pub(super) fn options_get_number(oo: &RustOptionsRef, name: &CStr) -> c_longlong {
    with_entry(oo, name, false, |entry| {
        let Some(entry) = entry else {
            fatalx(c"missing option %s", fmt_args![name]);
        };
        if !is_number(entry) {
            fatalx(c"option %s is not a number", fmt_args![name]);
        }
        entry.value.number()
    })
}

pub(super) fn options_get_command(oo: &RustOptionsRef, name: &CStr) -> Option<CmdListRef> {
    with_entry(oo, name, false, |entry| {
        let Some(entry) = entry else {
            fatalx(c"missing option %s", fmt_args![name]);
        };
        if !is_command(entry) {
            fatalx(c"option %s is not a command", fmt_args![name]);
        }
        entry.value.commands()
    })
}

/// Materializes a local entry from its inherited definition when absent.
unsafe fn options_own(oo: &RustOptionsRef, name: &CStr) {
    if with_entry(oo, name, true, |entry| entry.is_some()) {
        return;
    }
    unsafe {
        let entry = options_parent_table_entry(oo, name);
        options_default(oo, entry);
    }
}

pub(super) unsafe fn options_set_string(
    oo: &RustOptionsRef,
    name: &CStr,
    append: c_int,
    fmt: &CStr,
    args: &[FmtArg],
) {
    unsafe {
        let s = format_alloc(fmt, args);

        let value = with_entry(oo, name, true, |entry| {
            let entry = entry.filter(|entry| append != 0 && is_string(entry))?;
            let separator = if name.to_bytes().first() == Some(&b'@') {
                c""
            } else {
                table_of(entry).separator.unwrap_or(c"")
            };
            Some(xasprintf(
                c"%s%s%s",
                fmt_args![entry.value.string(), separator, s.as_c_str()],
            ))
        })
        .unwrap_or(s);
        if name.to_bytes().first() == Some(&b'@') {
            if !with_entry(oo, name, true, |entry| entry.is_some()) {
                options_add(oo, name);
            }
        } else {
            options_own(oo, name);
        }
        with_entry_mut(oo, name, true, |entry| {
            let entry = entry.expect("local string option was initialized");
            if !is_string(entry) {
                fatalx(c"option %s is not a string", fmt_args![name]);
            }
            entry.value = options_value::String(value);
            entry.cached = 0;
        });
    }
}

pub(super) unsafe fn options_set_number(oo: &RustOptionsRef, name: &CStr, value: c_longlong) {
    unsafe {
        if name.to_bytes().first() == Some(&b'@') {
            fatalx(c"user option %s must be a string", fmt_args![name]);
        }
        options_own(oo, name);
        with_entry_mut(oo, name, true, |entry| {
            let entry = entry.expect("local number option was initialized");
            if !is_number(entry) {
                fatalx(c"option %s is not a number", fmt_args![name]);
            }
            entry.value = options_value::Number(value);
        });
    }
}

pub(super) unsafe fn options_set_command(
    oo: &RustOptionsRef,
    name: &CStr,
    value: Option<CmdListRef>,
) {
    unsafe {
        if name.to_bytes().first() == Some(&b'@') {
            fatalx(c"user option %s must be a string", fmt_args![name]);
        }
        options_own(oo, name);
        with_entry_mut(oo, name, true, |entry| {
            let entry = entry.expect("local command option was initialized");
            if !is_command(entry) {
                fatalx(c"option %s is not a command", fmt_args![name]);
            }
            entry.value = match value {
                Some(cmdlist) => options_value::Commands(cmdlist),
                None => options_value::None,
            };
        });
    }
}

/// The set a window option is to be read from or written to, which the
/// `-g` flag, the target and the current window between them decide.
unsafe fn options_window_scope(
    args: &RustArguments,
    fs: &cmd_find_state,
    oo: &mut Option<RustOptionsRef>,
    cause: &mut Option<CString>,
) -> c_int {
    unsafe {
        if args.argument_flag_count(b'g') != 0 {
            *oo = global_w_options.get();
            return OPTIONS_TABLE_WINDOW;
        }
        let window = fs.session().and_then(|session| {
            session
                .as_session()
                .windows
                .get(&fs.wl?)?
                .window_handle()
                .cloned()
        });
        let Some(window) = window else {
            let target = args.argument_flag_string(b't');
            if let Some(target) = target {
                *cause = Some(xasprintf(c"no such window: %s", fmt_args![target]));
            } else {
                *cause = Some(xasprintf(c"no current window", fmt_args![]));
            }
            return OPTIONS_TABLE_NONE;
        };
        *oo = Some(window.options());
        OPTIONS_TABLE_WINDOW
    }
}

/// The set the option `name` belongs to, worked out from the scope the table
/// gives that name. A user option has no scope of its own, so the command
/// flags decide it instead.
///
/// Every option in the table is of the server, session, window, or window and
/// pane scope, so the C's arm for any other is gone.
pub(super) unsafe fn options_scope_from_name(
    args: &RustArguments,
    window: c_int,
    name: &CStr,
    fs: &cmd_find_state,
    oo: &mut Option<RustOptionsRef>,
    cause: &mut Option<CString>,
) -> c_int {
    unsafe {
        if name.to_bytes().first() == Some(&b'@') {
            return options_scope_from_flags(args, window, fs, oo, cause);
        }
        let Some(oe) = table().iter().find(|oe| oe.name == name) else {
            *cause = Some(xasprintf(c"unknown option: %s", fmt_args![name.as_ptr()]));
            return OPTIONS_TABLE_NONE;
        };

        let target = args.argument_flag_string(b't');
        match oe.scope {
            OPTIONS_TABLE_SERVER => {
                *oo = global_options.get();
                OPTIONS_TABLE_SERVER
            }
            OPTIONS_TABLE_SESSION => {
                if args.argument_flag_count(b'g') != 0 {
                    *oo = global_s_options.get();
                    return OPTIONS_TABLE_SESSION;
                }
                let Some(session) = fs.session() else {
                    if let Some(target) = target {
                        *cause = Some(xasprintf(c"no such session: %s", fmt_args![target]));
                    } else {
                        *cause = Some(xasprintf(c"no current session", fmt_args![]));
                    }
                    return OPTIONS_TABLE_NONE;
                };
                *oo = Some(session.options());
                OPTIONS_TABLE_SESSION
            }
            scope
                if scope == OPTIONS_TABLE_WINDOW | OPTIONS_TABLE_PANE
                    && args.argument_flag_count(b'p') != 0 =>
            {
                let Some(options) = fs
                    .pane_list_ref()
                    .and_then(|pane| pane.get().map(|pane| pane.options_ref().clone()))
                else {
                    if let Some(target) = target {
                        *cause = Some(xasprintf(c"no such pane: %s", fmt_args![target]));
                    } else {
                        *cause = Some(xasprintf(c"no current pane", fmt_args![]));
                    }
                    return OPTIONS_TABLE_NONE;
                };
                *oo = Some(options);
                OPTIONS_TABLE_PANE
            }
            _ => options_window_scope(args, fs, oo, cause),
        }
    }
}

/// The set a user option belongs to, worked out from the command flags alone.
pub(super) unsafe fn options_scope_from_flags(
    args: &RustArguments,
    window: c_int,
    fs: &cmd_find_state,
    oo: &mut Option<RustOptionsRef>,
    cause: &mut Option<CString>,
) -> c_int {
    unsafe {
        let target = args.argument_flag_string(b't');
        if args.argument_flag_count(b's') != 0 {
            *oo = global_options.get();
            return OPTIONS_TABLE_SERVER;
        }
        if args.argument_flag_count(b'p') != 0 {
            let Some(options) = fs
                .pane_list_ref()
                .and_then(|pane| pane.get().map(|pane| pane.options_ref().clone()))
            else {
                if let Some(target) = target {
                    *cause = Some(xasprintf(c"no such pane: %s", fmt_args![target]));
                } else {
                    *cause = Some(xasprintf(c"no current pane", fmt_args![]));
                }
                return OPTIONS_TABLE_NONE;
            };
            *oo = Some(options);
            return OPTIONS_TABLE_PANE;
        }
        if window != 0 || args.argument_flag_count(b'w') != 0 {
            return options_window_scope(args, fs, oo, cause);
        }
        if args.argument_flag_count(b'g') != 0 {
            *oo = global_s_options.get();
            return OPTIONS_TABLE_SESSION;
        }
        let Some(session) = fs.session() else {
            if target.is_none() {
                *cause = Some(xasprintf(c"no current session", fmt_args![]));
            } else {
                *cause = Some(xasprintf(c"no such session: %s", fmt_args![target]));
            }
            return OPTIONS_TABLE_NONE;
        };
        *oo = Some(session.options());
        OPTIONS_TABLE_SESSION
    }
}

/// The style the option `name` holds, read once and kept unless the string is
/// a format, which is expanded against `ft` every time.
pub(super) unsafe fn options_string_to_style(
    oo: &RustOptionsRef,
    name: &CStr,
    ft: Option<&mut format_tree>,
) -> Option<style> {
    unsafe {
        let source = with_entry(oo, name, false, |entry| {
            let entry = entry.filter(|entry| is_string(entry))?;
            Some(if entry.cached != 0 {
                Ok(entry.style)
            } else {
                Err((
                    options_owner(entry),
                    entry.name.clone(),
                    value_string_ref(&entry.value),
                ))
            })
        })?;
        let (owner, canonical_name, text) = match source {
            Ok(style) => return Some(style),
            Err(source) => source,
        };
        log_debug(
            c"%s: %s is '%s'",
            fmt_args![c"options_string_to_style", name, text.as_ref()],
        );
        let cached = !text.to_bytes().windows(2).any(|pair| pair == b"#{") as c_int;
        let save = |style| {
            with_entry_mut(&owner, &canonical_name, true, |entry| {
                if let Some(entry) = entry
                    && let OptionValue::String(current) = &entry.value.0
                    && Rc::ptr_eq(current, &text)
                {
                    entry.style = style;
                    entry.cached = cached;
                }
            })
        };
        let mut parsed = style::default();
        RustStyleCodec.set(&mut parsed, &grid_default_cell);
        save(parsed);
        let valid = if let Some(ft) = ft
            && cached == 0
        {
            let expanded = format_expand(ft, &text);
            RustStyleCodec
                .parse(&mut parsed, &grid_default_cell, expanded.as_bytes())
                .is_ok()
        } else {
            RustStyleCodec
                .parse(&mut parsed, &grid_default_cell, text.to_bytes())
                .is_ok()
        };
        save(parsed);
        valid.then_some(parsed)
    }
}

/// Whether a value is one the table would let the option hold: a shell that
/// can be run, a value the entry's pattern matches, and a style that parses.
unsafe fn options_from_string_check(
    oe: Option<&options_table_entry_t>,
    value: Option<&CStr>,
    cause: &mut Option<CString>,
) -> c_int {
    unsafe {
        let Some(oe) = oe else {
            return 0;
        };
        if oe.name == c"default-shell" && checkshell(value) == 0 {
            *cause = Some(xasprintf(
                c"not a suitable shell: %s",
                fmt_args![value.map_or(c"".as_ptr(), CStr::as_ptr)],
            ));
            return -1;
        }
        if let Some(pattern) = oe.pattern
            && fnmatch(
                pattern.as_ptr(),
                value.map_or(c"".as_ptr(), CStr::as_ptr),
                0,
            ) != 0
        {
            *cause = Some(xasprintf(
                c"value is invalid: %s",
                fmt_args![value.map_or(c"".as_ptr(), CStr::as_ptr)],
            ));
            return -1;
        }
        if oe.flags & OPTIONS_TABLE_IS_STYLE != 0
            && !value.is_some_and(|value| crate::types::cstr_has(value, c"#{"))
        {
            let mut sy = style::default();
            if RustStyleCodec
                .parse(
                    &mut sy,
                    &grid_default_cell,
                    value.map_or(b"" as &[u8], CStr::to_bytes),
                )
                .is_err()
            {
                *cause = Some(xasprintf(
                    c"invalid style: %s",
                    fmt_args![value.map_or(c"".as_ptr(), CStr::as_ptr)],
                ));
                return -1;
            }
        }
        0
    }
}

/// Sets a flag option from the words for on and off. No value at all turns it
/// over.
unsafe fn options_from_string_flag(
    oo: &RustOptionsRef,
    name: &CStr,
    value: Option<&CStr>,
    cause: &mut Option<CString>,
) -> c_int {
    unsafe {
        let flag = if value.is_none_or(|value| value.is_empty()) {
            (options_get_number(oo, name) == 0) as c_int
        } else {
            let word = value.expect("the flag has a value").to_bytes();
            if word == b"1" || word.eq_ignore_ascii_case(b"on") || word.eq_ignore_ascii_case(b"yes")
            {
                1
            } else if word == b"0"
                || word.eq_ignore_ascii_case(b"off")
                || word.eq_ignore_ascii_case(b"no")
            {
                0
            } else {
                *cause = Some(xasprintf(
                    c"bad value: %s",
                    fmt_args![value.map_or(c"".as_ptr(), CStr::as_ptr)],
                ));
                return -1;
            }
        };
        options_set_number(oo, name, flag as c_longlong);
        0
    }
}

/// Which of an option's choices `value` is, or -1 if it is none of them.
pub(super) fn options_find_choice(
    oe: &options_table_entry_t,
    value: &CStr,
    cause: &mut Option<CString>,
) -> c_int {
    let mut choice = -1;
    for (n, name) in choices_of(oe).iter().enumerate() {
        if *name == value {
            choice = n as c_int;
        }
    }
    if choice == -1 {
        *cause = Some(xasprintf(c"unknown value: %s", fmt_args![value.as_ptr()]));
        return -1;
    }
    choice
}

/// Sets a choice option. No value at all turns over the first two choices and
/// leaves any other where it is.
unsafe fn options_from_string_choice(
    oe: &options_table_entry_t,
    oo: &RustOptionsRef,
    name: &CStr,
    value: Option<&CStr>,
    cause: &mut Option<CString>,
) -> c_int {
    unsafe {
        let choice = if let Some(value) = value {
            let choice = options_find_choice(oe, value, cause);
            if choice < 0 {
                return -1;
            }
            choice
        } else {
            let choice = options_get_number(oo, name) as c_int;
            if choice < 2 {
                (choice == 0) as c_int
            } else {
                choice
            }
        };
        options_set_number(oo, name, choice as c_longlong);
        0
    }
}

/// Sets the option `name` from the text a user wrote for it, answering -1 and
/// a reason for a value the option cannot hold. A string that is turned down
/// leaves the option with what it had.
pub(super) unsafe fn options_from_string(
    oo: &RustOptionsRef,
    oe: Option<&options_table_entry_t>,
    name: &CStr,
    value: Option<&CStr>,
    append: c_int,
    cause: &mut Option<CString>,
) -> c_int {
    unsafe {
        let type_0 = if let Some(oe) = oe {
            if value.is_none()
                && oe.type_0 != OPTIONS_TABLE_FLAG
                && oe.type_0 != OPTIONS_TABLE_CHOICE
            {
                *cause = Some(xasprintf(c"empty value", fmt_args![]));
                return -1;
            }
            oe.type_0
        } else {
            if name.to_bytes().first() != Some(&b'@') {
                *cause = Some(xasprintf(c"bad option name", fmt_args![]));
                return -1;
            }
            OPTIONS_TABLE_STRING
        };

        match type_0 {
            OPTIONS_TABLE_STRING => {
                let old = options_string_ref(oo, name);
                options_set_string(
                    oo,
                    name,
                    append,
                    c"%s",
                    fmt_args![value.map_or(c"".as_ptr(), CStr::as_ptr)],
                );
                let new = options_string_ref(oo, name);
                if options_from_string_check(oe, Some(&new), cause) != 0 {
                    options_set_string(oo, name, 0, c"%s", fmt_args![old.as_ptr()]);
                    return -1;
                }
                0
            }
            OPTIONS_TABLE_NUMBER => {
                let oe = oe.unwrap();
                let Ok(number) = strtonum(
                    value.unwrap_or(c""),
                    oe.minimum as c_longlong,
                    oe.maximum as c_longlong,
                )
                .inspect_err(|errstr| {
                    *cause = Some(xasprintf(
                        c"value is %s: %s",
                        fmt_args![errstr.as_ptr(), value],
                    ));
                }) else {
                    return -1;
                };
                options_set_number(oo, name, number);
                0
            }
            OPTIONS_TABLE_KEY => {
                let key = RustKeyStringCodec.parse_key(value.unwrap_or(c""));
                if key == KEYC_UNKNOWN as key_code {
                    *cause = Some(xasprintf(
                        c"bad key: %s",
                        fmt_args![value.map_or(c"".as_ptr(), CStr::as_ptr)],
                    ));
                    return -1;
                }
                options_set_number(oo, name, key as c_longlong);
                0
            }
            OPTIONS_TABLE_COLOUR => {
                let number = RustColourEngine.from_string(value.unwrap_or(c"")) as c_longlong;
                if number == -1 {
                    *cause = Some(xasprintf(
                        c"bad colour: %s",
                        fmt_args![value.map_or(c"".as_ptr(), CStr::as_ptr)],
                    ));
                    return -1;
                }
                options_set_number(oo, name, number);
                0
            }
            OPTIONS_TABLE_FLAG => options_from_string_flag(oo, name, value, cause),
            OPTIONS_TABLE_CHOICE => options_from_string_choice(oe.unwrap(), oo, name, value, cause),
            _ => {
                let mut pr = cmd_parse_from_string(value.unwrap_or(c""), None);
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

/// Tells whatever an option reaches that it has changed. Every option ends by
/// having the status caches, the window sizes and the attached clients brought
/// up to date, whether or not it is one of the names below.
pub(super) unsafe fn options_push_changes(name: &CStr) {
    let SESSIONS = SESSIONS_FIELD.get();

    WINDOWS.with(|windows| unsafe {
        log_debug(
            c"%s: %s",
            fmt_args![c"options_push_changes".as_ptr(), name.as_ptr()],
        );

        if name == c"automatic-rename" {
            for owner in windows.iter().filter_map(|(_, window)| window.upgrade()) {
                let mut window = owner.as_window_mut();
                if let Some(index) = window
                    .panes
                    .iter()
                    .position(|pane| Some(pane.pane_id()) == window.active_pane_id())
                    && options_get_number(window.options_ref(), name) != 0
                {
                    *window.panes[index].as_pane_mut().flags_mut() |= PANE_CHANGED;
                }
            }
        }
        if name == c"cursor-colour" || name == c"cursor-style" {
            for mut pane in pane_walk() {
                let wp = pane.get_mut().expect("registered pane");
                window_pane_default_cursor(wp);
            }
        }
        if name == c"fill-character" {
            for owner in windows.iter().filter_map(|(_, window)| window.upgrade()) {
                window_set_fill_character(&mut owner.as_window_mut());
            }
        }
        if name == c"key-table" {
            for mut c in client_walk() {
                server_client_set_key_table(c.as_client_mut(), None);
            }
        }
        if name == c"user-keys" {
            for mut c in client_walk() {
                if c.as_tty().flags & TTY_OPENED != 0 {
                    tty_keys_build(c.as_tty_mut());
                }
            }
        }
        if name == c"status" || name == c"status-interval" {
            status_timer_start_all();
        }
        if name == c"monitor-silence" {
            alerts_reset_all();
        }
        if name == c"window-style" || name == c"window-active-style" {
            for mut pane in pane_walk() {
                let wp = pane.get_mut().expect("registered pane");
                *wp.flags_mut() |= PANE_STYLECHANGED | PANE_THEMECHANGED;
            }
        }
        if name.to_bytes().first() == Some(&b'@') {
            for mut pane in pane_walk() {
                let wp = pane.get_mut().expect("registered pane");
                *wp.flags_mut() |= PANE_STYLECHANGED;
            }
        }
        if name == c"pane-colours" {
            for mut pane in pane_walk() {
                let wp = pane.get_mut().expect("registered pane");
                let options = wp.options_ref().clone();
                options_load_pane_colours(&options, Some(wp.palette_mut()));
            }
        }
        if name == c"pane-border-status"
            || name == c"pane-scrollbars"
            || name == c"pane-scrollbars-position"
        {
            for owner in windows.iter().filter_map(|(_, window)| window.upgrade()) {
                let mut window = owner.as_window_mut();
                let mode = options_get_number(window.options_ref(), c"pane-scrollbars") as c_int;
                let position =
                    options_get_number(window.options_ref(), c"pane-scrollbars-position") as c_int;
                window.set_scrollbar_settings(crate::window_scrollbar::WindowScrollbarSettings {
                    sb: mode,
                    sb_pos: position,
                });
                drop(window);
                owner.fix_layout_panes(None);
            }
        }
        if name == c"pane-scrollbars-style" {
            for mut pane in pane_walk() {
                let wp = pane.get_mut().expect("registered pane");
                let scrollbar_style = pane_scrollbar_style_from_option(wp.options_ref());
                wp.set_scrollbar_style(scrollbar_style);
            }
            for owner in windows.iter().filter_map(|(_, window)| window.upgrade()) {
                owner.fix_layout_panes(None);
            }
        }
        if name == c"codepoint-widths" {
            utf8_update_width_cache(options_codepoint_widths(
                global_options
                    .get()
                    .as_ref()
                    .expect("global options are initialized"),
            ));
        }
        if name == c"input-buffer-size" {
            input_set_buffer_size(options_get_number(
                global_options
                    .get()
                    .as_ref()
                    .expect("global options are initialized"),
                name,
            ) as size_t);
        }
        if name == c"history-limit" {
            for s_ref in SESSIONS.read().values() {
                s_ref.update_history();
            }
        }

        for s_ref in SESSIONS.read().values() {
            s_ref.update_status_cache();
        }
        recalculate_sizes();
        for mut c in client_walk() {
            if !c.attached_session().is_none() {
                server_redraw_client(c.as_client_mut());
            }
        }
    });
}

/// Takes an option away, or puts it back to what the table gives it when the
/// set is one of the global ones. `idx` of anything but -1 takes one value of
/// an array away instead.
pub(super) unsafe fn options_remove_or_default(
    owner: &RustOptionsRef,
    name: &CStr,
    idx: c_int,
    cause: &mut Option<CString>,
) -> c_int {
    unsafe {
        if idx != -1 {
            return with_entry_mut(owner, name, true, |entry| {
                entry.map_or(0, |entry| {
                    options_array_set(entry, idx as u_int, None, 0, cause)
                })
            });
        }
        let Some((name, definition)) = with_entry(owner, name, true, |entry| {
            entry.map(|entry| (entry.name.clone(), entry.tableentry))
        }) else {
            return 0;
        };
        if let Some(definition) = definition
            && [
                &global_options.get(),
                &global_s_options.get(),
                &global_w_options.get(),
            ]
            .into_iter()
            .any(|global| global.as_ref().is_some_and(|global| owner.ptr_eq(global)))
        {
            options_default(owner, definition);
        } else {
            owner.0.borrow_mut().tree.remove(&name);
        }
        0
    }
}

#[cfg(test)]
#[path = "../tests/test_options.rs"]
mod tests;

pub(super) fn entry_value(entry: &options_entry) -> &options_value {
    &entry.value
}
pub(super) fn value_number(value: &options_value) -> c_longlong {
    value.number()
}
pub(super) fn value_string(value: &options_value) -> &CStr {
    value.string()
}
pub(super) fn value_string_ref(value: &options_value) -> Rc<CStr> {
    match &value.0 {
        OptionValue::String(string) => string.clone(),
        _ => panic!("not a string option"),
    }
}
pub(super) fn value_command(value: &options_value) -> Option<CmdListRef> {
    value.commands()
}
/// What one value of an option holds. The kind the option's table entry
/// names is what it is set to; nothing else is ever read out of it.
#[derive(Default)]
#[repr(C)]
enum OptionValue {
    /// A value the option has not been given yet.
    #[default]
    None,
    Number(core::ffi::c_longlong),
    String(Rc<CStr>),
    Commands(CmdListRef),
    Array(options_array),
}

impl options_value {
    fn array(&self) -> Option<&options_array> {
        match &self.0 { OptionValue::Array(array) => Some(array), _ => None }
    }

    fn array_mut(&mut self) -> &mut options_array {
        if !matches!(self.0, OptionValue::Array(_)) {
            self.0 = OptionValue::Array(options_array::new());
        }
        let OptionValue::Array(array) = &mut self.0 else { unreachable!() };
        array
    }
}

impl options_value {
    /// The number the value holds, or zero when it holds something else.
    fn number(&self) -> core::ffi::c_longlong {
        match &self.0 {
            OptionValue::Number(number) => *number,
            _ => 0,
        }
    }

    /// The string the value holds, which every caller of this has already
    /// established it is.
    fn string(&self) -> &core::ffi::CStr {
        match &self.0 {
            OptionValue::String(string) => string,
            _ => panic!("not a string option"),
        }
    }

    /// The command list the value holds, or nothing when it holds something
    /// else.
    fn cmdlist(&self) -> Option<&CmdListRef> {
        match &self.0 {
            OptionValue::Commands(cmdlist) => Some(cmdlist),
            _ => None,
        }
    }

    /// The command list the value holds as a handle, so a caller can keep it.
    fn commands(&self) -> Option<CmdListRef> {
        match &self.0 {
            OptionValue::Commands(cmdlist) => Some(cmdlist.clone()),
            _ => None,
        }
    }
}

/// An opaque option value, observed through the option engine.
pub struct options_value(OptionValue);

#[allow(non_snake_case, non_upper_case_globals)]
impl options_value {
    fn default() -> Self {
        Self::None
    }
    const None: Self = Self(OptionValue::None);
    fn Number(value: c_longlong) -> Self {
        Self(OptionValue::Number(value))
    }
    fn String(value: CString) -> Self {
        Self(OptionValue::String(value.into()))
    }
    fn Commands(value: CmdListRef) -> Self {
        Self(OptionValue::Commands(value))
    }
}

type options_array = std::collections::BTreeMap<u_int, options_array_item_t>;
