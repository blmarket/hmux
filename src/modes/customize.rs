use super::widget::ModeTreeItemRef;
use crate::WindowPane;
use crate::args::RustArguments;

use crate::cmd::cmd_parse_from_string;
use crate::cmd::{cmd_find_copy_state, cmd_find_from_pane, cmd_find_valid_state};
use crate::compat::{tolower, toupper};
use crate::fmt_args;
use crate::format::format_true;
use crate::format::{format_add, format_create_from_state, format_expand};
use crate::grid::grid_default_cell;
use crate::key_bindings::{
    key_binding_cmdlist, key_binding_flags, key_binding_key, key_binding_note,
    key_binding_same_cmdlist, key_binding_set_cmdlist, key_binding_set_note,
    key_binding_toggle_repeat, key_bindings_remove, key_bindings_reset,
};
use crate::key_bindings::{key_bindings_get_table, key_tables};
use crate::options::{OptionsEngine, RustOptionsEngine};
use crate::prompt_history::PromptHistoryType;
#[cfg(test)]
use crate::screen::RustScreen;
use crate::screen::ScreenWriteCtx;

pub use crate::consts::{
    CMD_PARSE_ERROR, INT_MAX, KEY_BINDING_REPEAT, KEYC_NONE, KEYC_RIGHT, OPTIONS_TABLE_CHOICE,
    OPTIONS_TABLE_COLOUR, OPTIONS_TABLE_FLAG, OPTIONS_TABLE_IS_ARRAY, OPTIONS_TABLE_IS_HOOK,
    OPTIONS_TABLE_IS_STYLE, OPTIONS_TABLE_PANE, OPTIONS_TABLE_SERVER, OPTIONS_TABLE_SESSION,
    OPTIONS_TABLE_STRING, OPTIONS_TABLE_WINDOW, PANE_REDRAW, PROMPT_ACCEPT, PROMPT_NOFORMAT,
    PROMPT_SINGLE,
};
use crate::status::{status_message_set, status_prompt_set};
use crate::style::style_apply;
use crate::text::{KeyStringCodec, RustKeyStringCodec};
use crate::tmux::{global_options, global_s_options, global_w_options};
pub use crate::types::*;
use crate::window::RustWindowPaneWeak;
use crate::window::{window_pane_index};
use crate::xmalloc::xasprintf;
use ::core::ffi::CStr;
use ::std::ffi::CString;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct CustomizeOptionScope {
    scope: window_customize_scope,
    id: u_int,
}

impl CustomizeOptionScope {
    fn new(scope: window_customize_scope, fs: &cmd_find_state) -> Self {
        let id = {
            match scope {
                WINDOW_CUSTOMIZE_SESSION => {
                    fs.session().expect("a session option has a session").id()
                }
                WINDOW_CUSTOMIZE_WINDOW => fs
                    .window()
                    .expect("a window option has a window")
                    .window_id(),
                WINDOW_CUSTOMIZE_PANE => fs
                    .wp
                    .as_ref()
                    .map(|pane| pane.id())
                    .expect("a pane option has a pane"),
                _ => 0,
            }
        };
        Self { scope, id }
    }

    unsafe fn options(self) -> Option<RustOptionsRef> {
        unsafe {
            match self.scope {
                WINDOW_CUSTOMIZE_SERVER => global_options.get(),
                WINDOW_CUSTOMIZE_GLOBAL_SESSION => global_s_options.get(),
                WINDOW_CUSTOMIZE_GLOBAL_WINDOW => global_w_options.get(),
                WINDOW_CUSTOMIZE_SESSION => {
                    SessionRef::find_by_id(self.id).map(|session| session.options())
                }
                WINDOW_CUSTOMIZE_WINDOW => {
                    WindowRef::find_by_id(self.id).map(|window| window.options())
                }
                WINDOW_CUSTOMIZE_PANE => crate::window::window_pane_find_by_id(self.id)
                    .and_then(|pane| pane.get().map(|pane| pane.options_ref().clone())),
                _ => None,
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
enum CustomizeTag {
    OptionGroup(core::ffi::c_int),
    BuiltinOption(CString, core::ffi::c_int),
    UserOption(CustomizeOptionScope, CString, core::ffi::c_int),
    KeyTable(CString),
    KeyBinding(CString, key_code),
    KeyField(CString, key_code, u8),
}

impl CustomizeTag {
    fn is_live(&self) -> bool {
        match self {
            Self::OptionGroup(_) | Self::BuiltinOption(_, _) => true,
            Self::UserOption(scope, name, _) => unsafe { scope.options() }
                .is_some_and(|options| options.with_entry(name, true, |entry| entry.is_some())),
            Self::KeyTable(name) => {
                key_bindings_get_table(name, 0).is_some_and(|table| !table.is_empty())
            }
            Self::KeyBinding(name, key) | Self::KeyField(name, key, _) => {
                key_bindings_get_table(name, 0)
                    .is_some_and(|table| table.borrow().binding(*key).is_some())
            }
        }
    }
}
#[repr(C)]
pub struct window_customize_modedata {
    /// The pane the mode is running in.
    pub wp: Option<RustWindowPaneWeak>,
    pub(crate) data: Option<ModeTreeDataRef>,
    pub format: Option<CString>,
    pub hide_global: core::ffi::c_int,
    pub prompt_flags: core::ffi::c_int,
    pub item_list: Vec<std::rc::Rc<window_customize_itemdata>>,
    tags: std::collections::BTreeMap<CustomizeTag, uint64_t>,
    next_tag: uint64_t,
    pub fs: cmd_find_state,
    pub change: window_customize_change,
    pub(crate) owner: Option<WindowCustomizeModeDataWeak>,
}

impl window_customize_modedata {
    fn tag_for(&mut self, key: CustomizeTag) -> uint64_t {
        if let Some(tag) = self.tags.get(&key) {
            return *tag;
        }
        self.next_tag = self
            .next_tag
            .checked_add(1)
            .filter(|tag| *tag != uint64_t::MAX)
            .expect("customization selection tags exhausted");
        self.tags.insert(key, self.next_tag);
        self.next_tag
    }

    fn option_tag(
        &mut self,
        scope: window_customize_scope,
        fs: &cmd_find_state,
        option: &options_entry,
        index: core::ffi::c_int,
    ) -> uint64_t {
        let name = RustOptionsEngine.name(option).to_owned();
        let key = if RustOptionsEngine.definition(Some(option)).is_some() {
            CustomizeTag::BuiltinOption(name, index)
        } else {
            CustomizeTag::UserOption(CustomizeOptionScope::new(scope, fs), name, index)
        };
        self.tag_for(key)
    }

    /// The pane the mode is running in, retained directly until this reference is dropped.
    pub(crate) fn pane(&self) -> Option<RustWindowPaneWeak> {
        self.wp.as_ref().filter(|pane| pane.is_alive()).cloned()
    }

    /// The mode tree the mode is showing, as a handle. Only ever asked of a
    /// mode that is showing one.
    pub(crate) fn tree_ref(&self) -> ModeTreeDataRef {
        self.data
            .as_ref()
            .cloned()
            .expect("the mode is showing a tree")
    }
}

impl WindowCustomizeModeDataRef {
    fn tree_ref(&self) -> ModeTreeDataRef {
        self.borrow().tree_ref()
    }

    fn pane(&self) -> Option<RustWindowPaneWeak> {
        self.borrow().pane()
    }

    fn redraw_pane(&self) {
        if let Some(mut pane) = self.pane()
            && let Some(pane) = unsafe { pane.get_mut() }
        {
            *pane.flags_mut() |= PANE_REDRAW;
        }
    }
}

pub type window_customize_change = core::ffi::c_uint;
pub const WINDOW_CUSTOMIZE_RESET: window_customize_change = 1;
pub const WINDOW_CUSTOMIZE_UNSET: window_customize_change = 0;
#[derive(Default)]
#[repr(C)]
pub struct window_customize_itemdata {
    pub scope: window_customize_scope,
    pub table: Option<CString>,
    pub key: key_code,
    pub oo: Option<RustOptionsRef>,
    pub name: Option<CString>,
    pub idx: core::ffi::c_int,
    /// Reaches the customize mode from an item made only for a prompt, and
    /// finds nothing once that mode has closed.
    pub(crate) data: Option<WindowCustomizeModeDataWeak>,
}
impl window_customize_itemdata {
    /// The key table the row's binding is in, for a row that stands for a key
    /// rather than an option.
    pub(crate) fn table(&self) -> Option<&CStr> {
        self.table.as_deref()
    }
}
pub type window_customize_scope = core::ffi::c_uint;
pub const WINDOW_CUSTOMIZE_PANE: window_customize_scope = 7;
pub const WINDOW_CUSTOMIZE_WINDOW: window_customize_scope = 6;
pub const WINDOW_CUSTOMIZE_GLOBAL_WINDOW: window_customize_scope = 5;
pub const WINDOW_CUSTOMIZE_SESSION: window_customize_scope = 4;
pub const WINDOW_CUSTOMIZE_GLOBAL_SESSION: window_customize_scope = 3;
pub const WINDOW_CUSTOMIZE_SERVER: window_customize_scope = 2;
pub const WINDOW_CUSTOMIZE_KEY: window_customize_scope = 1;
pub const WINDOW_CUSTOMIZE_NONE: window_customize_scope = 0;

pub static WINDOW_CUSTOMIZE_DEFAULT_FORMAT: &CStr = c"#{?is_option,#{?option_is_global,,#[reverse](#{option_scope})#[default] }#[ignore]#{option_value}#{?option_unit, #{option_unit},},#{key}}";
static window_customize_menu_items: [menu_item<'static>; 8] = [
    menu_item {
        name: Some(c"Select"),
        key: '\r' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: Some(c"Expand"),
        key: KEYC_RIGHT as core::ffi::c_ulong as key_code,
        command: None,
    },
    menu_item {
        name: Some(c""),
        key: KEYC_NONE as core::ffi::c_ulong as key_code,
        command: None,
    },
    menu_item {
        name: Some(c"Tag"),
        key: 't' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: Some(c"Tag All"),
        key: '\u{14}' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: Some(c"Tag None"),
        key: 'T' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: Some(c""),
        key: KEYC_NONE as core::ffi::c_ulong as key_code,
        command: None,
    },
    menu_item {
        name: Some(c"Cancel"),
        key: 'q' as i32 as key_code,
        command: None,
    },
];
unsafe fn window_customize_get_tree(
    scope: window_customize_scope,
    fs: &cmd_find_state,
) -> Option<RustOptionsRef> {
    unsafe {
        match scope {
            WINDOW_CUSTOMIZE_NONE | WINDOW_CUSTOMIZE_KEY => {
                return None;
            }
            WINDOW_CUSTOMIZE_SERVER => return global_options.get(),
            WINDOW_CUSTOMIZE_GLOBAL_SESSION => return global_s_options.get(),
            WINDOW_CUSTOMIZE_SESSION => {
                return Some(
                    fs.session()
                        .expect("the state names a session")
                        .options()
                        .clone(),
                );
            }
            WINDOW_CUSTOMIZE_GLOBAL_WINDOW => return global_w_options.get(),
            WINDOW_CUSTOMIZE_WINDOW => {
                return Some(fs.window().expect("the state names a window").options());
            }
            WINDOW_CUSTOMIZE_PANE => {
                return fs
                    .pane_ref()
                    .and_then(|pane| pane.get().map(|pane| pane.options_ref().clone()));
            }
            _ => {}
        }
        None
    }
}

fn window_customize_get_key(
    item: &window_customize_itemdata,
) -> Option<(KeyTableRef, key_binding)> {
    let table = key_bindings_get_table(item.table()?, 0)?;
    let binding = table.borrow().binding(item.key).cloned()?;
    Some((table, binding))
}
unsafe fn window_customize_scope_text(
    scope: window_customize_scope,
    fs: &cmd_find_state,
) -> CString {
    unsafe {
        let idx: u_int;
        match scope {
            WINDOW_CUSTOMIZE_PANE => {
                let pane = fs.pane_ref().expect("the state names a pane");
                let window = pane.window().expect("the pane has a window");
                (_, idx) =
                    window_pane_index(&window.as_window(), pane.get().expect("the pane is live"));
                xasprintf(c"pane %u", fmt_args![idx])
            }
            WINDOW_CUSTOMIZE_SESSION => xasprintf(
                c"session %s",
                fmt_args![
                    fs.session()
                        .expect("the state names a session")
                        .name()
                        .as_deref()
                ],
            ),
            WINDOW_CUSTOMIZE_WINDOW => xasprintf(
                c"window %u",
                fmt_args![fs.wl.expect("the state names a window link")],
            ),
            _ => CString::default(),
        }
    }
}
fn window_customize_add_item(
    data: &mut window_customize_modedata,
    item: window_customize_itemdata,
) -> std::rc::Rc<window_customize_itemdata> {
    let item = std::rc::Rc::new(item);
    data.item_list.push(item.clone());
    item
}
unsafe fn window_customize_build_array(
    data: &mut window_customize_modedata,
    top: Option<&ModeTreeItemRef>,
    scope: window_customize_scope,
    o: &options_entry,
    ft: &mut format_tree,
    fs: &cmd_find_state,
) {
    unsafe {
        let oo = RustOptionsEngine.owner(o);

        let mut idx: u_int;
        let mut tag: uint64_t;
        for array_index in RustOptionsEngine.array_indices(o) {
            idx = array_index;
            let name = xasprintf(c"%s[%u]", fmt_args![RustOptionsEngine.name(o), idx]);
            format_add(ft, c"option_name", c"%s", fmt_args![name.as_c_str()]);
            let value =
                RustOptionsEngine.display(o, idx as core::ffi::c_int, 0 as core::ffi::c_int);
            format_add(ft, c"option_value", c"%s", fmt_args![value.as_c_str()]);
            let item = window_customize_add_item(
                data,
                window_customize_itemdata {
                    scope,
                    oo: Some(oo.clone()),
                    name: Some(RustOptionsEngine.name(o).to_owned()),
                    idx: idx as core::ffi::c_int,
                    ..Default::default()
                },
            );
            let text = format_expand(ft, data.format.as_deref().unwrap_or(c""));
            tag = data.option_tag(scope, fs, o, idx as core::ffi::c_int);
            (data.tree_ref())
                .add_item(
                    top,
                    ModeTreeItemData::Customize(item),
                    tag,
                    &name,
                    Some(&text),
                    -(1 as core::ffi::c_int),
                )
                .expect("the array parent belongs to this tree");
        }
    }
}
unsafe fn window_customize_build_option(
    data: &mut window_customize_modedata,
    top: Option<&ModeTreeItemRef>,
    scope: window_customize_scope,
    o: &options_entry,
    ft: &mut format_tree,
    filter: Option<&CStr>,
    fs: &cmd_find_state,
) {
    unsafe {
        let oe = RustOptionsEngine.definition(Some(o));
        let oo = RustOptionsEngine.owner(o);
        let name = RustOptionsEngine.name(o);
        let mut global: core::ffi::c_int = 0 as core::ffi::c_int;
        let mut array: core::ffi::c_int = 0 as core::ffi::c_int;

        if let Some(oe) = oe
            && oe.flags & OPTIONS_TABLE_IS_HOOK != 0
        {
            return;
        }
        if let Some(oe) = oe
            && oe.flags & OPTIONS_TABLE_IS_ARRAY != 0
        {
            array = 1 as core::ffi::c_int;
        }
        if scope as core::ffi::c_uint
            == WINDOW_CUSTOMIZE_SERVER as core::ffi::c_int as core::ffi::c_uint
            || scope as core::ffi::c_uint
                == WINDOW_CUSTOMIZE_GLOBAL_SESSION as core::ffi::c_int as core::ffi::c_uint
            || scope as core::ffi::c_uint
                == WINDOW_CUSTOMIZE_GLOBAL_WINDOW as core::ffi::c_int as core::ffi::c_uint
        {
            global = 1 as core::ffi::c_int;
        }
        if data.hide_global != 0 && global != 0 {
            return;
        }
        format_add(ft, c"option_name", c"%s", fmt_args![name]);
        format_add(ft, c"option_is_global", c"%d", fmt_args![global]);
        format_add(ft, c"option_is_array", c"%d", fmt_args![array]);
        let text = window_customize_scope_text(scope, fs);
        format_add(ft, c"option_scope", c"%s", fmt_args![text.as_c_str()]);
        if let Some(oe) = oe
            && let Some(unit) = oe.unit
        {
            format_add(ft, c"option_unit", c"%s", fmt_args![unit]);
        } else {
            format_add(ft, c"option_unit", c"%s", fmt_args![c""]);
        }
        if array == 0 {
            let value =
                RustOptionsEngine.display(o, -(1 as core::ffi::c_int), 0 as core::ffi::c_int);
            format_add(ft, c"option_value", c"%s", fmt_args![value.as_c_str()]);
        }
        if let Some(filter) = filter {
            let expanded = format_expand(ft, filter);
            if format_true(Some(&expanded)) == 0 {
                return;
            }
        }
        let item = window_customize_add_item(
            data,
            window_customize_itemdata {
                scope,
                oo: Some(oo.clone()),
                name: Some(name.to_owned()),
                idx: -1,
                ..Default::default()
            },
        );
        let text = match array {
            0 => Some(format_expand(ft, data.format.as_deref().unwrap_or(c""))),
            _ => None,
        };
        let tag: uint64_t = data.option_tag(scope, fs, o, -1);
        let tree = data.tree_ref();
        let parent = top.filter(|parent| parent.get().is_some());
        let top = tree
            .add_item(
                parent,
                ModeTreeItemData::Customize(item),
                tag,
                name,
                text.as_deref(),
                0 as core::ffi::c_int,
            )
            .expect("the option parent belongs to this tree");
        if array != 0 {
            window_customize_build_array(data, Some(&top), scope, o, ft, fs);
        }
    }
}
fn window_customize_find_user_options(oo: &RustOptionsRef, list: &mut Vec<CString>) {
    for name in oo.local_names() {
        if name.to_bytes().starts_with(b"@") && !list.contains(&name) {
            list.push(name);
        }
    }
}
#[allow(clippy::too_many_arguments)]
unsafe fn window_customize_build_options(
    data: &mut window_customize_modedata,
    title: &CStr,
    group: core::ffi::c_int,
    scope0: window_customize_scope,
    oo0: &RustOptionsRef,
    scope1: window_customize_scope,
    oo1: Option<&RustOptionsRef>,
    scope2: window_customize_scope,
    oo2: Option<&RustOptionsRef>,
    ft: &mut format_tree,
    filter: Option<&CStr>,
    fs: &cmd_find_state,
) {
    unsafe {
        let mut list: Vec<CString> = Vec::new();
        let top = (data.tree_ref())
            .add_item(
                None,
                ModeTreeItemData::None,
                data.tag_for(CustomizeTag::OptionGroup(group)),
                title,
                None,
                0 as core::ffi::c_int,
            )
            .expect("a root item can be inserted");
        top.no_tag().expect("the new option group is live");
        window_customize_find_user_options(oo0, &mut list);
        if let Some(oo1) = oo1 {
            window_customize_find_user_options(oo1, &mut list);
        }
        if let Some(oo2) = oo2 {
            window_customize_find_user_options(oo2, &mut list);
        }
        let mut build = |entry: &options_entry| {
            let owner = RustOptionsEngine.owner(entry);
            let scope = if oo2.is_some_and(|store| owner.ptr_eq(store)) {
                scope2
            } else if oo1.is_some_and(|store| owner.ptr_eq(store)) {
                scope1
            } else {
                scope0
            };
            window_customize_build_option(data, Some(&top), scope, entry, ft, filter, fs);
        };
        let mut selected: Option<(RustOptionsRef, CString)> = None;
        for name in list {
            if oo2.is_none()
                && let Some((owner, name)) = &selected
            {
                owner.with_entry(name, true, |entry| {
                    if let Some(entry) = entry {
                        build(entry);
                    }
                });
                continue;
            }
            for store in [oo2, oo1, Some(oo0)].into_iter().flatten() {
                let found = store.with_entry(&name, false, |entry| {
                    if let Some(entry) = entry {
                        selected = Some((RustOptionsEngine.owner(entry), name.clone()));
                        build(entry);
                        true
                    } else {
                        false
                    }
                });
                if found {
                    break;
                }
            }
        }
        for name in oo0.local_names() {
            if !name.to_bytes().starts_with(b"@") {
                oo2.or(oo1)
                    .unwrap_or(oo0)
                    .with_entry(&name, false, |entry| {
                        if let Some(entry) = entry {
                            build(entry);
                        }
                    });
            }
        }
    }
}
unsafe fn window_customize_build_keys(
    data: &mut window_customize_modedata,
    kt: &key_table,
    _ft: &mut format_tree,
    filter: Option<&CStr>,
    fs: &cmd_find_state,
) {
    unsafe {
        let mut flag: Option<&CStr>;
        let table_name = (kt).name().to_owned();
        let title = xasprintf(c"Key Table - %s", fmt_args![(kt).name()]);
        let top = (data.tree_ref())
            .add_item(
                None,
                ModeTreeItemData::None,
                data.tag_for(CustomizeTag::KeyTable(table_name.clone())),
                &title,
                None,
                0 as core::ffi::c_int,
            )
            .expect("a key table group can be inserted");
        top.no_tag().expect("the new key table group is live");
        let mut ft_box = format_create_from_state(None, None, fs);
        let ft: &mut format_tree = &mut ft_box;
        format_add(ft, c"is_option", c"0", fmt_args![]);
        format_add(ft, c"is_key", c"1", fmt_args![]);
        for bd in kt.bindings() {
            let key_name = RustKeyStringCodec.format_key(key_binding_key(bd), false);
            format_add(ft, c"key", c"%s", fmt_args![key_name.as_c_str()]);
            if let Some(note) = key_binding_note(bd) {
                format_add(ft, c"key_note", c"%s", fmt_args![note]);
            }
            if let Some(filter) = filter {
                let expanded = format_expand(ft, filter);
                if format_true(Some(&expanded)) == 0 {
                    continue;
                }
            }
            let item = window_customize_add_item(
                data,
                window_customize_itemdata {
                    scope: WINDOW_CUSTOMIZE_KEY,
                    table: Some((kt).name().to_owned()),
                    key: key_binding_key(bd),
                    name: Some(RustKeyStringCodec.format_key(key_binding_key(bd), false)),
                    idx: -1,
                    ..Default::default()
                },
            );
            let expanded = format_expand(ft, data.format.as_deref().unwrap_or(c""));
            let child = (data.tree_ref())
                .add_item(
                    Some(&top),
                    ModeTreeItemData::Customize(item.clone()),
                    data.tag_for(CustomizeTag::KeyBinding(
                        table_name.clone(),
                        key_binding_key(bd),
                    )),
                    &expanded,
                    None,
                    0 as core::ffi::c_int,
                )
                .expect("the key table group belongs to this tree");
            let tmp =
                key_binding_cmdlist(bd).map_or_else(CString::default, |cmdlist| cmdlist.print(0));
            let text = xasprintf(c"#[ignore]%s", fmt_args![tmp.as_c_str()]);
            let mti = (data.tree_ref())
                .add_item(
                    Some(&child),
                    ModeTreeItemData::Customize(item.clone()),
                    data.tag_for(CustomizeTag::KeyField(
                        table_name.clone(),
                        key_binding_key(bd),
                        0,
                    )),
                    c"Command",
                    Some(&text),
                    -(1 as core::ffi::c_int),
                )
                .expect("the key binding row belongs to this tree");
            mti.draw_as_parent().expect("the new key field is live");
            mti.no_tag().expect("the new key field is live");
            let text = if let Some(note) = key_binding_note(bd) {
                xasprintf(c"#[ignore]%s", fmt_args![note])
            } else {
                CString::default()
            };
            let mti = (data.tree_ref())
                .add_item(
                    Some(&child),
                    ModeTreeItemData::Customize(item.clone()),
                    data.tag_for(CustomizeTag::KeyField(
                        table_name.clone(),
                        key_binding_key(bd),
                        1,
                    )),
                    c"Note",
                    Some(&text),
                    -(1 as core::ffi::c_int),
                )
                .expect("the key binding row belongs to this tree");
            mti.draw_as_parent().expect("the new key field is live");
            mti.no_tag().expect("the new key field is live");
            if key_binding_flags(bd) & KEY_BINDING_REPEAT != 0 {
                flag = Some(c"on");
            } else {
                flag = Some(c"off");
            }
            let mti = (data.tree_ref())
                .add_item(
                    Some(&child),
                    ModeTreeItemData::Customize(item.clone()),
                    data.tag_for(CustomizeTag::KeyField(
                        table_name.clone(),
                        key_binding_key(bd),
                        2,
                    )),
                    c"Repeat",
                    flag,
                    -(1 as core::ffi::c_int),
                )
                .expect("the key binding row belongs to this tree");
            mti.draw_as_parent().expect("the new key field is live");
            mti.no_tag().expect("the new key field is live");
        }
    }
}

unsafe fn window_customize_draw_key(
    item: Option<&window_customize_itemdata>,
    writer: &mut impl ScreenWriteCtx,
    sx: u_int,
    sy: u_int,
) {
    unsafe {
        let (cx, cy) = writer.cursor_position();
        let Some(item) = item else {
            return;
        };
        let Some((kt, bd)) = window_customize_get_key(item) else {
            return;
        };
        let default_note = c"There is no note for this key.";
        let note = key_binding_note(&bd).unwrap_or(default_note);
        let period = match note.to_bytes().last() {
            None | Some(b'.') => c"",
            Some(_) => c".",
        };
        if writer.text(
            cx,
            sx,
            sy,
            0 as core::ffi::c_int,
            &grid_default_cell,
            c"%s%s",
            fmt_args![note, period],
        ) == 0
        {
            return;
        }
        writer.cursormove(
            cx as core::ffi::c_int,
            writer.cursor_position().1.wrapping_add(1 as u_int) as core::ffi::c_int,
            0 as core::ffi::c_int,
        );
        if writer.cursor_position().1 >= cy.wrapping_add(sy).wrapping_sub(1 as u_int) {
            return;
        }
        if writer.text(
            cx,
            sx,
            sy.wrapping_sub(writer.cursor_position().1.wrapping_sub(cy)),
            0 as core::ffi::c_int,
            &grid_default_cell,
            c"This key is in the %s table.",
            fmt_args![kt.name().as_c_str()],
        ) == 0
        {
            return;
        }
        if writer.text(
            cx,
            sx,
            sy.wrapping_sub(writer.cursor_position().1.wrapping_sub(cy)),
            0 as core::ffi::c_int,
            &grid_default_cell,
            c"This key %s repeat.",
            fmt_args![if key_binding_flags(&bd) & KEY_BINDING_REPEAT != 0 {
                c"does"
            } else {
                c"does not"
            }],
        ) == 0
        {
            return;
        }
        writer.cursormove(
            cx as core::ffi::c_int,
            writer.cursor_position().1.wrapping_add(1 as u_int) as core::ffi::c_int,
            0 as core::ffi::c_int,
        );
        if writer.cursor_position().1 >= cy.wrapping_add(sy).wrapping_sub(1 as u_int) {
            return;
        }
        let cmd =
            key_binding_cmdlist(&bd).map_or_else(CString::default, |cmdlist| cmdlist.print(0));
        if writer.text(
            cx,
            sx,
            sy.wrapping_sub(writer.cursor_position().1.wrapping_sub(cy)),
            0 as core::ffi::c_int,
            &grid_default_cell,
            c"Command: %s",
            fmt_args![cmd.as_c_str()],
        ) == 0
        {
            return;
        }
        let default_bd = kt.borrow().default_binding(key_binding_key(&bd)).cloned();
        if let Some(default_bd) = default_bd {
            let default_cmd = key_binding_cmdlist(&default_bd)
                .map_or_else(CString::default, |cmdlist| cmdlist.print(0));
            if cmd != default_cmd
                && writer.text(
                    cx,
                    sx,
                    sy.wrapping_sub(writer.cursor_position().1.wrapping_sub(cy)),
                    0 as core::ffi::c_int,
                    &grid_default_cell,
                    c"The default is: %s",
                    fmt_args![default_cmd.as_c_str()],
                ) == 0
            {}
        }
    }
}
/// The values `oe` accepts, as the comma-separated list the option preview
/// prints. The C grew this in a 256-byte buffer with `strlcat`, so a list that
/// would overrun it stops there, and the two characters it then takes back are
/// the separator the last value left behind.
fn window_customize_choice_list(oe: &options_table_entry_t) -> CString {
    let mut listed: Vec<u8> = Vec::new();
    for choice in oe.choices.unwrap_or(&[]) {
        listed.extend_from_slice(choice.to_bytes());
        listed.extend_from_slice(b", ");
    }
    listed.truncate(255);
    listed.truncate(listed.len().saturating_sub(2));
    CString::new(listed).expect("a choice holds no nul")
}

fn window_customize_height() -> u_int {
    12 as u_int
}
static window_customize_help_lines: [&CStr; 9] = [
    c"\r\x1B[1m   Enter, s \x1B[0m\x0Ex\x0F \x1B[0mSet %1 value\n",
    c"\r\x1B[1m          S \x1B[0m\x0Ex\x0F \x1B[0mSet global %1 value\n",
    c"\r\x1B[1m          w \x1B[0m\x0Ex\x0F \x1B[0mSet window %1 value\n",
    c"\r\x1B[1m          d \x1B[0m\x0Ex\x0F \x1B[0mSet to default value\n",
    c"\r\x1B[1m          D \x1B[0m\x0Ex\x0F \x1B[0mSet tagged %1s to default value\n",
    c"\r\x1B[1m          u \x1B[0m\x0Ex\x0F \x1B[0mUnset an %1\n",
    c"\r\x1B[1m          U \x1B[0m\x0Ex\x0F \x1B[0mUnset tagged %1s\n",
    c"\r\x1B[1m          f \x1B[0m\x0Ex\x0F \x1B[0mEnter a filter\n",
    c"\r\x1B[1m          v \x1B[0m\x0Ex\x0F \x1B[0mToggle information\n",
];
fn window_customize_help() -> (&'static [&'static CStr], u_int, &'static CStr) {
    (&window_customize_help_lines, 52 as u_int, c"option")
}
pub(crate) unsafe fn window_customize_init(
    wme: &mut window_mode_entry,
    mut pane: crate::window::RustWindowPaneWeak,
    fs: Option<&cmd_find_state>,
    args: Option<&RustArguments>,
) {
    unsafe {
        let data_ref = WindowCustomizeModeDataRef::new(window_customize_modedata {
            wp: Some(pane.clone()),
            data: None,
            format: None,
            hide_global: 0,
            prompt_flags: 0,
            item_list: Vec::new(),
            tags: Default::default(),
            next_tag: 0,
            fs: fs.expect("a choose mode opens from a target").clone(),
            change: WINDOW_CUSTOMIZE_UNSET,
            owner: None,
        });
        let mut data_guard = data_ref.borrow_mut();
        let data = &mut *data_guard;
        data.wp = Some(pane.clone());
        wme.state = WindowModeState::Customize(data_ref.clone());
        data.fs = fs.expect("a choose mode opens from a target").clone();
        data.format = Some(
            match args.and_then(|args| args.argument_flag_string(b'F')) {
                Some(value) => value.to_owned(),
                None => WINDOW_CUSTOMIZE_DEFAULT_FORMAT.to_owned(),
            },
        );
        if args.is_some_and(|args| args.argument_flag_count(b'y') != 0) {
            data.prompt_flags = PROMPT_ACCEPT;
        }
        drop(data_guard);
        let build_data = data_ref.downgrade();
        let menu_data = data_ref.downgrade();
        let draw_data = data_ref.downgrade();
        let mtd = ModeTreeDataRef::start(
            pane.get_mut().expect("the initializing pane still exists"),
            args,
            Some(std::rc::Rc::new(move |sort, tag, filter| {
                if let Some(data) = build_data.upgrade() {
                    data.build(sort, tag, filter);
                }
            })),
            Some(std::rc::Rc::new(move |itemdata, writer, sx, sy| {
                if let Some(data) = draw_data.upgrade() {
                    data.draw(itemdata, writer, sx, sy);
                }
            })),
            None,
            Some(std::rc::Rc::new(move |c, key| {
                if let Some(data) = menu_data.upgrade() {
                    data.menu(c, key);
                }
            })),
            Some(std::rc::Rc::new(window_customize_height)),
            None,
            None,
            None,
            Some(window_customize_help()),
            WindowModeData::Customize(data_ref.downgrade()),
            &window_customize_menu_items,
        );
        data_ref.borrow_mut().data = Some(mtd.clone());
        (data_ref.tree_ref()).zoom(args);
        (data_ref.tree_ref()).build();
        (data_ref.tree_ref()).draw();
    }
}
pub(crate) unsafe fn window_customize_free(wme: &mut window_mode_entry) {
    if let Some(data) = wme.state.customize() {
        let tree = data.borrow_mut().data.take().expect("the mode is showing a tree");
        unsafe { tree.close() };
    }
}
pub(crate) unsafe fn window_customize_resize(wme: &mut window_mode_entry, sx: u_int, sy: u_int) {
    let data = wme.state.customize().expect("customize mode state");
    unsafe { (data.tree_ref()).resize(sx, sy) };
}

pub(crate) unsafe fn window_customize_set_option_callback(
    c: &mut client,
    itemdata: &mut window_customize_itemdata,
    s: Option<&CStr>,
    _done: core::ffi::c_int,
) -> core::ffi::c_int {
    unsafe {
        let current_block: u64;
        let item = itemdata;
        let Some(owner) = item
            .data
            .as_ref()
            .and_then(WindowCustomizeModeDataWeak::upgrade)
        else {
            return 0 as core::ffi::c_int;
        };
        let data = &owner;
        let oo = item.oo.as_ref().expect("option row has a store");
        let name = item.name.as_deref().expect("option row has a name");
        let mut cause = None;
        let mut array_cause: Option<CString> = None;
        let mut idx: core::ffi::c_int = item.idx;
        let Some(s) = s.filter(|s| !s.is_empty()) else {
            return 0 as core::ffi::c_int;
        };
        if data.check_item(&*item, None) == 0 {
            return 0 as core::ffi::c_int;
        }
        let Some(oe) = oo.with_entry(name, false, |entry| {
            entry.map(|entry| RustOptionsEngine.definition(Some(entry)))
        }) else {
            return 0;
        };
        if let Some(oe) = oe
            && oe.flags & OPTIONS_TABLE_IS_ARRAY != 0
        {
            let result = oo.with_entry_mut(name, false, |entry| {
                let entry = entry.expect("option was found before editing");
                if idx == -1 {
                    idx = 0;
                    while idx < INT_MAX
                        && RustOptionsEngine.array_get(entry, idx as u_int).is_some()
                    {
                        idx += 1;
                    }
                }
                RustOptionsEngine.array_set(entry, idx as u_int, Some(s), 0, &mut array_cause)
            });
            if result != 0 {
                current_block = 9497928493432737774;
            } else {
                current_block = 5689001924483802034;
            }
        } else if (oo).set_from_string(oe, name, Some(s), 0 as core::ffi::c_int, &mut cause)
            != 0 as core::ffi::c_int
        {
            current_block = 9497928493432737774;
        } else {
            current_block = 5689001924483802034;
        }
        match current_block {
            9497928493432737774 => {
                if let Some(error) = array_cause.take() {
                    let mut bytes = error.into_bytes();
                    if let Some(first) = bytes.first_mut() {
                        *first = toupper(*first);
                    }
                    let error = CString::new(bytes).expect("error has no NUL");
                    status_message_set(
                        Some(c),
                        -(1 as core::ffi::c_int),
                        1 as core::ffi::c_int,
                        0 as core::ffi::c_int,
                        0 as core::ffi::c_int,
                        c"%s",
                        fmt_args![error.as_c_str()],
                    );
                    return 0 as core::ffi::c_int;
                }
                if let Some(mut cause) = cause.take() {
                    let mut bytes = cause.into_bytes();
                    if let Some(first) = bytes.first_mut() {
                        *first = toupper(*first);
                    }
                    cause = CString::new(bytes).expect("error has no NUL");
                    status_message_set(
                        Some(c),
                        -(1 as core::ffi::c_int),
                        1 as core::ffi::c_int,
                        0 as core::ffi::c_int,
                        0 as core::ffi::c_int,
                        c"%s",
                        fmt_args![cause.as_c_str()],
                    );
                }
                0 as core::ffi::c_int
            }
            _ => {
                RustOptionsEngine.push_changes(item.name.as_deref().unwrap());
                ((*data).tree_ref()).build();
                ((*data).tree_ref()).draw();
                data.redraw_pane();
                0 as core::ffi::c_int
            }
        }
    }
}

pub(crate) unsafe fn window_customize_set_command_callback(
    c: &mut client,
    item: &mut window_customize_itemdata,
    s: Option<&CStr>,
    _done: core::ffi::c_int,
) -> core::ffi::c_int {
    unsafe {
        let Some(owner) = item
            .data
            .as_ref()
            .and_then(WindowCustomizeModeDataWeak::upgrade)
        else {
            return 0 as core::ffi::c_int;
        };
        let data = &owner;
        let Some(s) = s.filter(|s| !s.is_empty()) else {
            return 0 as core::ffi::c_int;
        };
        let Some((kt, _bd)) = window_customize_get_key(item) else {
            return 0 as core::ffi::c_int;
        };
        let mut pr = cmd_parse_from_string(s, None);
        match pr.status {
            CMD_PARSE_ERROR => {
                let error = pr.error.take().unwrap();
                let mut bytes = error.into_bytes();
                if let Some(first) = bytes.first_mut() {
                    *first = toupper(*first);
                }
                let error = CString::new(bytes).expect("error has no NUL");
                status_message_set(
                    Some(c),
                    -(1 as core::ffi::c_int),
                    1 as core::ffi::c_int,
                    0 as core::ffi::c_int,
                    0 as core::ffi::c_int,
                    c"%s",
                    fmt_args![error.as_c_str()],
                );
                0 as core::ffi::c_int
            }
            _ => {
                if let Some(binding) = kt.borrow_mut().binding_mut(item.key) {
                    key_binding_set_cmdlist(binding, pr.cmdlist.take());
                }
                ((*data).tree_ref()).build();
                ((*data).tree_ref()).draw();
                data.redraw_pane();
                0 as core::ffi::c_int
            }
        }
    }
}
pub(crate) unsafe fn window_customize_set_note_callback(
    _c: &mut client,
    item: &mut window_customize_itemdata,
    s: Option<&CStr>,
    _done: core::ffi::c_int,
) -> core::ffi::c_int {
    unsafe {
        let Some(owner) = item
            .data
            .as_ref()
            .and_then(WindowCustomizeModeDataWeak::upgrade)
        else {
            return 0 as core::ffi::c_int;
        };
        let data = &owner;
        let Some(s) = s.filter(|s| !s.is_empty()) else {
            return 0 as core::ffi::c_int;
        };
        let Some((kt, _bd)) = window_customize_get_key(item) else {
            return 0 as core::ffi::c_int;
        };
        if let Some(binding) = kt.borrow_mut().binding_mut(item.key) {
            key_binding_set_note(binding, s);
        }
        ((*data).tree_ref()).build();
        ((*data).tree_ref()).draw();
        data.redraw_pane();
        0 as core::ffi::c_int
    }
}

unsafe fn window_customize_change_each(modedata: WindowModeData, itemdata: ModeTreeItemData) {
    unsafe {
        let held = modedata.customize().expect("the mode holds its state");
        let data = &held;
        let Some(item) = itemdata.customize() else {
            return;
        };
        let change = data.borrow().change;
        match change {
            WINDOW_CUSTOMIZE_UNSET => {
                if item.scope as core::ffi::c_uint
                    == WINDOW_CUSTOMIZE_KEY as core::ffi::c_int as core::ffi::c_uint
                {
                    data.unset_key(Some(&item));
                } else {
                    data.unset_option(Some(&item));
                }
            }
            WINDOW_CUSTOMIZE_RESET => {
                if item.scope as core::ffi::c_uint
                    == WINDOW_CUSTOMIZE_KEY as core::ffi::c_int as core::ffi::c_uint
                {
                    data.reset_key(Some(&item));
                } else {
                    data.reset_option(Some(&item));
                }
            }
            _ => {}
        }
        if item.scope as core::ffi::c_uint
            != WINDOW_CUSTOMIZE_KEY as core::ffi::c_int as core::ffi::c_uint
        {
            RustOptionsEngine.push_changes(item.name.as_deref().unwrap());
        }
    }
}

#[cfg(test)]
#[path = "customize_focused_tests.rs"]
mod focused_tests;

impl WindowCustomizeModeDataRef {
    unsafe fn check_item(
        &self,
        item: &window_customize_itemdata,
        fsp: Option<&mut cmd_find_state>,
    ) -> core::ffi::c_int {
        let data = self;

        unsafe {
            // A caller that wants the state the check found keeps its own; one
            // that only wants the answer gets a state that lives for the call.
            let mut fs = cmd_find_state::default();
            let fsp = fsp.unwrap_or(&mut fs);
            let target = data.borrow().fs.clone();
            if cmd_find_valid_state(&target) != 0 {
                cmd_find_copy_state(fsp, &target);
            } else {
                let Some(pane) = data.pane() else {
                    return 0;
                };
                let Some(pane) = pane.get() else {
                    return 0;
                };
                cmd_find_from_pane(fsp, pane, 0);
            }
            (item.oo == window_customize_get_tree(item.scope, fsp)) as core::ffi::c_int
        }
    }
    unsafe fn build(
        self,
        _sort_crit: &sort_criteria_t,
        _tag: &mut uint64_t,
        filter: Option<&CStr>,
    ) {
        let held = self;

        unsafe {
            let mut data_guard = held.borrow_mut();
            let data = &mut *data_guard;
            let mut fs = cmd_find_state::default();
            let mut i: u_int;
            data.item_list.clear();
            data.tags.retain(|key, _| key.is_live());
            if cmd_find_valid_state(&data.fs) != 0 {
                cmd_find_copy_state(&mut fs, &data.fs);
            } else {
                let Some(pane) = data.pane() else {
                    return;
                };
                let Some(pane) = pane.get() else {
                    return;
                };
                cmd_find_from_pane(&mut fs, pane, 0);
            }
            let mut ft = format_create_from_state(None, None, &fs);
            format_add(&mut ft, c"is_option", c"1", fmt_args![]);
            format_add(&mut ft, c"is_key", c"0", fmt_args![]);
            window_customize_build_options(
                &mut *data,
                c"Server Options",
                OPTIONS_TABLE_SERVER,
                WINDOW_CUSTOMIZE_SERVER,
                global_options
                    .get()
                    .as_ref()
                    .expect("global options are initialized"),
                WINDOW_CUSTOMIZE_NONE,
                None,
                WINDOW_CUSTOMIZE_NONE,
                None,
                &mut ft,
                filter,
                &fs,
            );
            window_customize_build_options(
                &mut *data,
                c"Session Options",
                OPTIONS_TABLE_SESSION,
                WINDOW_CUSTOMIZE_GLOBAL_SESSION,
                global_s_options
                    .get()
                    .as_ref()
                    .expect("global options are initialized"),
                WINDOW_CUSTOMIZE_SESSION,
                Some(&fs.session().expect("the state names a session").options()),
                WINDOW_CUSTOMIZE_NONE,
                None,
                &mut ft,
                filter,
                &fs,
            );
            let pane = fs.pane_ref().expect("the state names a pane");
            let pane_options = pane.get().expect("the pane is live").options_ref().clone();
            window_customize_build_options(
                &mut *data,
                c"Window & Pane Options",
                OPTIONS_TABLE_WINDOW,
                WINDOW_CUSTOMIZE_GLOBAL_WINDOW,
                global_w_options
                    .get()
                    .as_ref()
                    .expect("global options are initialized"),
                WINDOW_CUSTOMIZE_WINDOW,
                Some(&fs.window().expect("the state names a window").options()),
                WINDOW_CUSTOMIZE_PANE,
                Some(&pane_options),
                &mut ft,
                filter,
                &fs,
            );
            ft = format_create_from_state(None, None, &fs);
            i = 0 as u_int;
            key_tables.with_borrow(|tables| {
                for kt in tables.values() {
                    let table = kt.borrow();
                    if !table.is_empty() {
                        window_customize_build_keys(&mut *data, &table, &mut ft, filter, &fs);
                        i = i.wrapping_add(1);
                        if i == 256 as u_int {
                            break;
                        }
                    }
                }
            });
        }
    }
    unsafe fn draw_option(
        &self,
        item: &window_customize_itemdata,
        writer: &mut impl ScreenWriteCtx,
        sx: u_int,
        sy: u_int,
    ) {
        let data = self;

        unsafe {
            let (cx, cy) = writer.cursor_position();
            let mut fs = cmd_find_state::default();
            if data.check_item(item, Some(&mut fs)) == 0 {
                return;
            }
            let name = item.name.as_deref().expect("option row has a name");
            let store = item.oo.as_ref().expect("option row has a store");
            let idx = item.idx;
            let Some((oe, owner, value)) = store.with_entry(name, false, |entry| {
                entry.map(|entry| {
                    (
                        RustOptionsEngine.definition(Some(entry)),
                        RustOptionsEngine.owner(entry),
                        RustOptionsEngine.display(entry, idx, 0),
                    )
                })
            }) else {
                return;
            };
            let (space, unit) = oe
                .and_then(|oe| oe.unit)
                .map_or((c"", c""), |unit| (c" ", unit));
            let mut ft = format_create_from_state(None, None, &fs);
            let description = oe
                .and_then(|oe| oe.text)
                .unwrap_or(c"This option doesn't have a description.");
            if writer.text(
                cx,
                sx,
                sy,
                0,
                &grid_default_cell,
                c"%s",
                fmt_args![description],
            ) == 0
            {
                return;
            }
            writer.cursormove(
                cx as core::ffi::c_int,
                writer.cursor_position().1.wrapping_add(1) as core::ffi::c_int,
                0,
            );
            if writer.cursor_position().1 >= cy.wrapping_add(sy).wrapping_sub(1) {
                return;
            }
            let scope = match oe {
                Some(oe)
                    if oe.scope & (OPTIONS_TABLE_WINDOW | OPTIONS_TABLE_PANE)
                        == OPTIONS_TABLE_WINDOW | OPTIONS_TABLE_PANE =>
                {
                    c"window and pane"
                }
                Some(oe) if oe.scope & OPTIONS_TABLE_WINDOW != 0 => c"window",
                Some(oe) if oe.scope & OPTIONS_TABLE_SESSION != 0 => c"session",
                Some(_) => c"server",
                None => c"user",
            };
            if writer.text(
                cx,
                sx,
                sy.wrapping_sub(writer.cursor_position().1.wrapping_sub(cy)),
                0,
                &grid_default_cell,
                c"This is a %s option.",
                fmt_args![scope],
            ) == 0
            {
                return;
            }
            let is_array = oe.is_some_and(|oe| oe.flags & OPTIONS_TABLE_IS_ARRAY != 0);
            if is_array {
                let written = if idx != -1 {
                    writer.text(
                        cx,
                        sx,
                        sy.wrapping_sub(writer.cursor_position().1.wrapping_sub(cy)),
                        0,
                        &grid_default_cell,
                        c"This is an array option, index %u.",
                        fmt_args![idx],
                    )
                } else {
                    writer.text(
                        cx,
                        sx,
                        sy.wrapping_sub(writer.cursor_position().1.wrapping_sub(cy)),
                        0,
                        &grid_default_cell,
                        c"This is an array option.",
                        fmt_args![],
                    )
                };
                if written == 0 || idx == -1 {
                    return;
                }
            }
            writer.cursormove(
                cx as core::ffi::c_int,
                writer.cursor_position().1.wrapping_add(1) as core::ffi::c_int,
                0,
            );
            if writer.cursor_position().1 >= cy.wrapping_add(sy).wrapping_sub(1) {
                return;
            }
            let default_value = oe
                .filter(|_| idx == -1)
                .map(|oe| RustOptionsEngine.default_text(oe))
                .filter(|default_value| *default_value != value);
            if writer.text(
                cx,
                sx,
                sy.wrapping_sub(writer.cursor_position().1.wrapping_sub(cy)),
                0,
                &grid_default_cell,
                c"Option value: %s%s%s",
                fmt_args![value.as_c_str(), space, unit],
            ) == 0
            {
                return;
            }
            if oe.is_none_or(|oe| oe.type_0 == OPTIONS_TABLE_STRING) {
                let expanded = format_expand(&mut ft, &value);
                if expanded != value
                    && writer.text(
                        cx,
                        sx,
                        sy.wrapping_sub(writer.cursor_position().1.wrapping_sub(cy)),
                        0,
                        &grid_default_cell,
                        c"This expands to: %s",
                        fmt_args![expanded.as_c_str()],
                    ) == 0
                {
                    return;
                }
            }
            if let Some(oe) = oe
                && oe.type_0 == OPTIONS_TABLE_CHOICE
            {
                let listed = window_customize_choice_list(oe);
                if writer.text(
                    cx,
                    sx,
                    sy.wrapping_sub(writer.cursor_position().1.wrapping_sub(cy)),
                    0,
                    &grid_default_cell,
                    c"Available values are: %s",
                    fmt_args![listed.as_c_str()],
                ) == 0
                {
                    return;
                }
            }
            let mut gc = grid_default_cell;
            if oe.is_some_and(|oe| oe.type_0 == OPTIONS_TABLE_COLOUR) {
                if writer.text(
                    cx,
                    sx,
                    sy.wrapping_sub(writer.cursor_position().1.wrapping_sub(cy)),
                    1,
                    &grid_default_cell,
                    c"This is a colour option: ",
                    fmt_args![],
                ) == 0
                {
                    return;
                }
                gc.fg = (store).number(name) as core::ffi::c_int;
                if writer.text(
                    cx,
                    sx,
                    sy.wrapping_sub(writer.cursor_position().1.wrapping_sub(cy)),
                    0,
                    &gc,
                    c"EXAMPLE",
                    fmt_args![],
                ) == 0
                {
                    return;
                }
            }
            if oe.is_some_and(|oe| oe.flags & OPTIONS_TABLE_IS_STYLE != 0) {
                if writer.text(
                    cx,
                    sx,
                    sy.wrapping_sub(writer.cursor_position().1.wrapping_sub(cy)),
                    1,
                    &grid_default_cell,
                    c"This is a style option: ",
                    fmt_args![],
                ) == 0
                {
                    return;
                }
                style_apply(&mut gc, store, name, Some(&mut ft));
                if writer.text(
                    cx,
                    sx,
                    sy.wrapping_sub(writer.cursor_position().1.wrapping_sub(cy)),
                    0,
                    &gc,
                    c"EXAMPLE",
                    fmt_args![],
                ) == 0
                {
                    return;
                }
            }
            if let Some(default_value) = default_value
                && writer.text(
                    cx,
                    sx,
                    sy.wrapping_sub(writer.cursor_position().1.wrapping_sub(cy)),
                    0,
                    &grid_default_cell,
                    c"The default is: %s%s%s",
                    fmt_args![default_value.as_c_str(), space, unit],
                ) == 0
            {
                return;
            }
            writer.cursormove(
                cx as core::ffi::c_int,
                writer.cursor_position().1.wrapping_add(1) as core::ffi::c_int,
                0,
            );
            if writer.cursor_position().1 > cy.wrapping_add(sy).wrapping_sub(1) || is_array {
                return;
            }
            let (wo, go) = match item.scope {
                WINDOW_CUSTOMIZE_PANE => {
                    let wo = (store).parent();
                    let go = (wo.as_ref().expect("parent options exist")).parent();
                    (wo, go)
                }
                WINDOW_CUSTOMIZE_WINDOW | WINDOW_CUSTOMIZE_SESSION => (None, (store).parent()),
                _ => (None, None),
            };
            if let Some(wo) = wo.filter(|store| !owner.ptr_eq(store)) {
                let value = wo.with_entry(name, true, |entry| {
                    entry.map(|entry| RustOptionsEngine.display(entry, -1, 0))
                });
                if let Some(value) = value
                    && writer.text(
                        writer.cursor_position().0,
                        sx,
                        sy.wrapping_sub(writer.cursor_position().1.wrapping_sub(cy)),
                        0,
                        &grid_default_cell,
                        c"Window value (from window %u): %s%s%s",
                        fmt_args![
                            fs.wl.expect("the state names a window link"),
                            value.as_c_str(),
                            space,
                            unit
                        ],
                    ) == 0
                {
                    return;
                }
            }
            if let Some(go) = go.filter(|store| !owner.ptr_eq(store)) {
                let value = go.with_entry(name, true, |entry| {
                    entry.map(|entry| RustOptionsEngine.display(entry, -1, 0))
                });
                if let Some(value) = value {
                    writer.text(
                        writer.cursor_position().0,
                        sx,
                        sy.wrapping_sub(writer.cursor_position().1.wrapping_sub(cy)),
                        0,
                        &grid_default_cell,
                        c"Global value: %s%s%s",
                        fmt_args![value.as_c_str(), space, unit],
                    );
                }
            }
        }
    }
    unsafe fn draw(
        self,
        itemdata: ModeTreeItemData,
        writer: &mut impl ScreenWriteCtx,
        sx: u_int,
        sy: u_int,
    ) {
        let held = self;

        unsafe {
            let data = &held;
            let Some(item) = itemdata.customize() else {
                return;
            };
            if item.scope as core::ffi::c_uint
                == WINDOW_CUSTOMIZE_KEY as core::ffi::c_int as core::ffi::c_uint
            {
                window_customize_draw_key(Some(&item), writer, sx, sy);
            } else {
                data.draw_option(&item, writer, sx, sy);
            };
        }
    }
    unsafe fn menu(self, c: &mut client, key: key_code) {
        let held = self;

        unsafe {
            let Some(pane) = held.pane() else {
                return;
            };
            let current = pane.get().and_then(|pane| (pane).active_mode())
                .is_some_and(|mode| matches!(&mode.state, WindowModeState::Customize(data) if data.ptr_eq(&held)));
            if current {
                held.key(c, key, None);
            }
        }
    }
    fn owner(&self) -> Option<WindowCustomizeModeDataWeak> {
        let data = self;

        Some(data.downgrade())
    }
    unsafe fn set_option(
        &self,
        c: &mut client,
        item: Option<&window_customize_itemdata>,
        global: core::ffi::c_int,
        mut pane: core::ffi::c_int,
    ) {
        let data = self;

        unsafe {
            let oo: Option<RustOptionsRef>;
            let flag: core::ffi::c_int;
            let Some(item) = item else {
                return;
            };
            let idx: core::ffi::c_int = item.idx;
            let mut scope: window_customize_scope = WINDOW_CUSTOMIZE_NONE;
            let mut choice: u_int;
            let name = item.name.as_deref().expect("option row has a name");
            let mut space: &CStr = c"";
            let mut fs = cmd_find_state::default();
            if data.check_item(item, Some(&mut fs)) == 0 {
                return;
            }
            let store = item.oo.as_ref().expect("option row has a store");
            let Some(oe) = store.with_entry(name, false, |entry| {
                entry.map(|entry| RustOptionsEngine.definition(Some(entry)))
            }) else {
                return;
            };
            if let Some(oe) = oe
                && !oe.scope & OPTIONS_TABLE_PANE != 0
            {
                pane = 0 as core::ffi::c_int;
            }
            if let Some(oe) = oe
                && oe.flags & OPTIONS_TABLE_IS_ARRAY != 0
            {
                scope = item.scope;
                oo = item.oo.clone();
            } else {
                if global != 0 {
                    match item.scope {
                        WINDOW_CUSTOMIZE_NONE
                        | WINDOW_CUSTOMIZE_KEY
                        | WINDOW_CUSTOMIZE_SERVER
                        | WINDOW_CUSTOMIZE_GLOBAL_SESSION
                        | WINDOW_CUSTOMIZE_GLOBAL_WINDOW => {
                            scope = item.scope;
                        }
                        WINDOW_CUSTOMIZE_SESSION => {
                            scope = WINDOW_CUSTOMIZE_GLOBAL_SESSION;
                        }
                        WINDOW_CUSTOMIZE_WINDOW | WINDOW_CUSTOMIZE_PANE => {
                            scope = WINDOW_CUSTOMIZE_GLOBAL_WINDOW;
                        }
                        _ => {}
                    }
                } else {
                    match item.scope {
                        WINDOW_CUSTOMIZE_NONE
                        | WINDOW_CUSTOMIZE_KEY
                        | WINDOW_CUSTOMIZE_SERVER
                        | WINDOW_CUSTOMIZE_SESSION => {
                            scope = item.scope;
                        }
                        WINDOW_CUSTOMIZE_WINDOW | WINDOW_CUSTOMIZE_PANE => {
                            if pane != 0 {
                                scope = WINDOW_CUSTOMIZE_PANE;
                            } else {
                                scope = WINDOW_CUSTOMIZE_WINDOW;
                            }
                        }
                        WINDOW_CUSTOMIZE_GLOBAL_SESSION => {
                            scope = WINDOW_CUSTOMIZE_SESSION;
                        }
                        WINDOW_CUSTOMIZE_GLOBAL_WINDOW => {
                            if pane != 0 {
                                scope = WINDOW_CUSTOMIZE_PANE;
                            } else {
                                scope = WINDOW_CUSTOMIZE_WINDOW;
                            }
                        }
                        _ => {}
                    }
                }
                if scope as core::ffi::c_uint == item.scope as core::ffi::c_uint {
                    oo = item.oo.clone();
                } else {
                    oo = window_customize_get_tree(scope, &fs);
                }
            }
            if let Some(oe) = oe
                && oe.type_0 as core::ffi::c_uint
                    == OPTIONS_TABLE_FLAG as core::ffi::c_int as core::ffi::c_uint
            {
                flag = (oo.as_ref().expect("option scope was resolved")).number(name)
                    as core::ffi::c_int;
                (oo.as_ref().expect("option scope was resolved")).set_number(
                    name,
                    (flag == 0) as core::ffi::c_int as core::ffi::c_longlong,
                );
            } else if let Some(oe) = oe
                && oe.type_0 as core::ffi::c_uint
                    == OPTIONS_TABLE_CHOICE as core::ffi::c_int as core::ffi::c_uint
            {
                choice = (oo.as_ref().expect("option scope was resolved")).number(name) as u_int;
                let next = choice.wrapping_add(1 as u_int);
                if next as usize >= oe.choices.unwrap_or(&[]).len() {
                    choice = 0 as u_int;
                } else {
                    choice = next;
                }
                (oo.as_ref().expect("option scope was resolved"))
                    .set_number(name, choice as core::ffi::c_longlong);
            } else {
                let text = window_customize_scope_text(scope, &fs);
                if !text.as_bytes().is_empty() {
                    space = c", for ";
                } else if scope as core::ffi::c_uint
                    != WINDOW_CUSTOMIZE_SERVER as core::ffi::c_int as core::ffi::c_uint
                {
                    space = c", global";
                }
                let prompt = if let Some(oe) = oe
                    && oe.flags & OPTIONS_TABLE_IS_ARRAY != 0
                {
                    if idx == -(1 as core::ffi::c_int) {
                        xasprintf(c"(%s[+]%s%s) ", fmt_args![name, space, text.as_c_str()])
                    } else {
                        xasprintf(
                            c"(%s[%d]%s%s) ",
                            fmt_args![name, idx, space, text.as_c_str()],
                        )
                    }
                } else {
                    xasprintf(c"(%s%s%s) ", fmt_args![name, space, text.as_c_str()])
                };
                let value = store.with_entry(name, false, |entry| {
                    RustOptionsEngine.display(entry.expect("option row exists"), idx, 0)
                });
                let data_ref = data.owner().expect("window customize owner");
                let prompt_item = Box::new(window_customize_itemdata {
                    scope,
                    oo,
                    name: Some(name.to_owned()),
                    idx,
                    data: Some(data_ref),
                    ..Default::default()
                });
                status_prompt_set(
                    &mut *c,
                    None,
                    &prompt,
                    Some(&value),
                    Prompt::CustomizeSetOption,
                    PromptData::CustomizeSet(prompt_item),
                    PROMPT_NOFORMAT,
                    PromptHistoryType::Command,
                );
            };
        }
    }
    unsafe fn unset_option(&self, item: Option<&std::rc::Rc<window_customize_itemdata>>) {
        let data = self;

        unsafe {
            let Some(item) = item else {
                return;
            };
            if data.check_item(item, None) == 0 {
                return;
            }
            let name = item.name.as_deref().expect("option row has a name");
            let owner = item
                .oo
                .as_ref()
                .expect("option row has a store")
                .with_entry(name, false, |entry| {
                    entry.map(|entry| RustOptionsEngine.owner(entry))
                });
            let Some(owner) = owner else { return };
            if item.idx != -(1 as core::ffi::c_int)
                && (data.tree_ref())
                    .current_item()
                    .customize()
                    .as_ref()
                    .is_some_and(|current| std::rc::Rc::ptr_eq(item, current))
            {
                (data.tree_ref()).up(0 as core::ffi::c_int);
            }
            RustOptionsEngine.remove_or_default(&owner, name, item.idx, &mut None);
        }
    }
    unsafe fn reset_option(&self, item: Option<&window_customize_itemdata>) {
        let data = self;

        unsafe {
            let mut oo: Option<RustOptionsRef>;
            let Some(item) = item else {
                return;
            };
            if data.check_item(item, None) == 0 {
                return;
            }
            if item.idx != -(1 as core::ffi::c_int) {
                return;
            }
            oo = item.oo.clone();
            while oo.is_some() {
                RustOptionsEngine.remove_or_default(
                    item.oo.as_ref().expect("option row has a store"),
                    item.name.as_deref().expect("option row has a name"),
                    -1,
                    &mut None,
                );
                oo = (oo.as_ref().expect("option scope was resolved")).parent();
            }
        }
    }
    unsafe fn set_key(&self, c: &mut client, item: Option<&window_customize_itemdata>) {
        let data = self;

        unsafe {
            let Some(item) = item else {
                return;
            };
            let key: key_code = item.key;
            let Some((kt, bd)) = window_customize_get_key(item) else {
                return;
            };
            let s = (data.tree_ref()).current_name();
            if s.as_deref() == Some(c"Repeat") {
                if let Some(binding) = kt.borrow_mut().binding_mut(item.key) {
                    key_binding_toggle_repeat(binding);
                }
            } else if s.as_deref() == Some(c"Command") {
                let key_name = RustKeyStringCodec.format_key(key, false);
                let prompt = xasprintf(c"(%s) ", fmt_args![key_name.as_c_str()]);
                let value = key_binding_cmdlist(&bd)
                    .map_or_else(CString::default, |cmdlist| cmdlist.print(0));
                let data_ref = data.owner().expect("window customize owner");
                let prompt_item = Box::new(window_customize_itemdata {
                    scope: item.scope,
                    table: item.table.clone(),
                    key,
                    data: Some(data_ref),
                    ..Default::default()
                });
                status_prompt_set(
                    &mut *c,
                    None,
                    &prompt,
                    Some(&value),
                    Prompt::CustomizeSetCommand,
                    PromptData::CustomizeSet(prompt_item),
                    PROMPT_NOFORMAT,
                    PromptHistoryType::Command,
                );
            } else if s.as_deref() == Some(c"Note") {
                let key_name = RustKeyStringCodec.format_key(key, false);
                let prompt = xasprintf(c"(%s) ", fmt_args![key_name.as_c_str()]);
                let data_ref = data.owner().expect("window customize owner");
                let prompt_item = Box::new(window_customize_itemdata {
                    scope: item.scope,
                    table: item.table.clone(),
                    key,
                    data: Some(data_ref),
                    ..Default::default()
                });
                status_prompt_set(
                    &mut *c,
                    None,
                    &prompt,
                    Some(key_binding_note(&bd).unwrap_or(c"")),
                    Prompt::CustomizeSetNote,
                    PromptData::CustomizeSet(prompt_item),
                    PROMPT_NOFORMAT,
                    PromptHistoryType::Command,
                );
            }
        }
    }
    unsafe fn unset_key(&self, item: Option<&std::rc::Rc<window_customize_itemdata>>) {
        let data = self;

        unsafe {
            let Some(item) = item else {
                return;
            };
            let Some((kt, bd)) = window_customize_get_key(item) else {
                return;
            };
            if (data.tree_ref())
                .current_item()
                .customize()
                .as_ref()
                .is_some_and(|current| std::rc::Rc::ptr_eq(item, current))
            {
                (data.tree_ref()).collapse_current();
                (data.tree_ref()).up(0 as core::ffi::c_int);
            }
            let name = kt.name();
            key_bindings_remove(&name, key_binding_key(&bd));
        }
    }
    unsafe fn reset_key(&self, item: Option<&std::rc::Rc<window_customize_itemdata>>) {
        let data = self;

        unsafe {
            let Some(item) = item else {
                return;
            };
            let Some((kt, bd)) = window_customize_get_key(item) else {
                return;
            };
            let dd = kt.borrow().default_binding(key_binding_key(&bd)).cloned();
            if dd
                .as_ref()
                .is_some_and(|dd| key_binding_same_cmdlist(&bd, dd))
            {
                return;
            }
            if dd.is_none()
                && (data.tree_ref())
                    .current_item()
                    .customize()
                    .as_ref()
                    .is_some_and(|current| std::rc::Rc::ptr_eq(item, current))
            {
                (data.tree_ref()).collapse_current();
                (data.tree_ref()).up(0 as core::ffi::c_int);
            }
            let name = kt.name();
            key_bindings_reset(&name, key_binding_key(&bd));
        }
    }
    pub(crate) unsafe fn change_current(
        &self,
        _c: &mut client,
        s: Option<&CStr>,
        _done: core::ffi::c_int,
    ) -> core::ffi::c_int {
        let modedata = self;

        unsafe {
            let data = modedata;
            let Some(s) = s.filter(|s| !s.is_empty()) else {
                return 0 as core::ffi::c_int;
            };
            if s.to_bytes().len() != 1 || tolower(s.to_bytes()[0]) != b'y' {
                return 0 as core::ffi::c_int;
            }
            let Some(item) = (data.tree_ref()).current_item().customize() else {
                return 0;
            };
            let change = data.borrow().change;
            match change {
                WINDOW_CUSTOMIZE_UNSET => {
                    if item.scope as core::ffi::c_uint
                        == WINDOW_CUSTOMIZE_KEY as core::ffi::c_int as core::ffi::c_uint
                    {
                        data.unset_key(Some(&item));
                    } else {
                        data.unset_option(Some(&item));
                    }
                }
                WINDOW_CUSTOMIZE_RESET => {
                    if item.scope as core::ffi::c_uint
                        == WINDOW_CUSTOMIZE_KEY as core::ffi::c_int as core::ffi::c_uint
                    {
                        data.reset_key(Some(&item));
                    } else {
                        data.reset_option(Some(&item));
                    }
                }
                _ => {}
            }
            if item.scope as core::ffi::c_uint
                != WINDOW_CUSTOMIZE_KEY as core::ffi::c_int as core::ffi::c_uint
            {
                RustOptionsEngine.push_changes(item.name.as_deref().unwrap());
            }
            (data.tree_ref()).build();
            (data.tree_ref()).draw();
            data.redraw_pane();
            0 as core::ffi::c_int
        }
    }
    pub(crate) unsafe fn change_tagged(
        &self,
        _c: &mut client,
        s: Option<&CStr>,
        _done: core::ffi::c_int,
    ) -> core::ffi::c_int {
        let modedata = self;

        unsafe {
            let data = modedata;
            let Some(s) = s.filter(|s| !s.is_empty()) else {
                return 0 as core::ffi::c_int;
            };
            if s.to_bytes().len() != 1 || tolower(s.to_bytes()[0]) != b'y' {
                return 0 as core::ffi::c_int;
            }
            (data.tree_ref()).each_tagged(
                |modedata, itemdata| window_customize_change_each(modedata, itemdata),
                0 as core::ffi::c_int,
            );
            (data.tree_ref()).build();
            (data.tree_ref()).draw();
            data.redraw_pane();
            0 as core::ffi::c_int
        }
    }
    pub(crate) unsafe fn key(self, c: &mut client, mut key: key_code, m: Option<&mouse_event>) {
        let owner = self;

        unsafe {
            let data = &owner;
            let tree = data.tree_ref();
            let prompt_flags = data.borrow().prompt_flags;
            let finished: core::ffi::c_int;
            let idx: core::ffi::c_int;
            let tagged: u_int;
            (finished, _, _) = ((*data).tree_ref()).key(c, &mut key, m);
            let item = (data.tree_ref()).current_item().customize();
            match key {
                13 | 115 => {
                    if let Some(item) = item.as_ref() {
                        if item.scope as core::ffi::c_uint
                            == WINDOW_CUSTOMIZE_KEY as core::ffi::c_int as core::ffi::c_uint
                        {
                            data.set_key(c, Some(item));
                        } else {
                            data.set_option(
                                c,
                                Some(item),
                                0 as core::ffi::c_int,
                                1 as core::ffi::c_int,
                            );
                            RustOptionsEngine.push_changes(item.name.as_deref().unwrap());
                        }
                        ((*data).tree_ref()).build();
                    }
                }
                119 => {
                    if let Some(item) = item
                        .as_ref()
                        .filter(|item| item.scope != WINDOW_CUSTOMIZE_KEY)
                    {
                        data.set_option(
                            c,
                            Some(item),
                            0 as core::ffi::c_int,
                            0 as core::ffi::c_int,
                        );
                        RustOptionsEngine.push_changes(item.name.as_deref().unwrap());
                        ((*data).tree_ref()).build();
                    }
                }
                83 | 87 => {
                    if let Some(item) = item
                        .as_ref()
                        .filter(|item| item.scope != WINDOW_CUSTOMIZE_KEY)
                    {
                        data.set_option(
                            c,
                            Some(item),
                            1 as core::ffi::c_int,
                            0 as core::ffi::c_int,
                        );
                        RustOptionsEngine.push_changes(item.name.as_deref().unwrap());
                        ((*data).tree_ref()).build();
                    }
                }
                100 => {
                    if let Some(item) = item.as_ref().filter(|item| item.idx == -1) {
                        let prompt =
                            xasprintf(c"Reset %s to default? ", fmt_args![item.name.as_deref()]);
                        let data_ref = data.owner().expect("window customize owner");
                        data.borrow_mut().change = WINDOW_CUSTOMIZE_RESET;
                        status_prompt_set(
                            c,
                            None,
                            &prompt,
                            Some(c""),
                            Prompt::CustomizeChangeCurrent,
                            PromptData::CustomizeChange(Box::new(data_ref)),
                            PROMPT_SINGLE | PROMPT_NOFORMAT | prompt_flags,
                            PromptHistoryType::Command,
                        );
                    }
                }
                68 => {
                    tagged = ((*data).tree_ref()).count_tagged();
                    if !(tagged == 0 as u_int) {
                        let prompt = xasprintf(c"Reset %u tagged to default? ", fmt_args![tagged]);
                        let data_ref = data.owner().expect("window customize owner");
                        data.borrow_mut().change = WINDOW_CUSTOMIZE_RESET;
                        status_prompt_set(
                            c,
                            None,
                            &prompt,
                            Some(c""),
                            Prompt::CustomizeChangeTagged,
                            PromptData::CustomizeChange(Box::new(data_ref)),
                            PROMPT_SINGLE | PROMPT_NOFORMAT | prompt_flags,
                            PromptHistoryType::Command,
                        );
                    }
                }
                117 => {
                    if let Some(item) = item.as_ref() {
                        idx = item.idx;
                        let prompt = if idx != -(1 as core::ffi::c_int) {
                            xasprintf(c"Unset %s[%d]? ", fmt_args![item.name.as_deref(), idx])
                        } else {
                            xasprintf(c"Unset %s? ", fmt_args![item.name.as_deref()])
                        };
                        let data_ref = data.owner().expect("window customize owner");
                        data.borrow_mut().change = WINDOW_CUSTOMIZE_UNSET;
                        status_prompt_set(
                            c,
                            None,
                            &prompt,
                            Some(c""),
                            Prompt::CustomizeChangeCurrent,
                            PromptData::CustomizeChange(Box::new(data_ref)),
                            PROMPT_SINGLE | PROMPT_NOFORMAT | prompt_flags,
                            PromptHistoryType::Command,
                        );
                    }
                }
                85 => {
                    tagged = ((*data).tree_ref()).count_tagged();
                    if !(tagged == 0 as u_int) {
                        let prompt = xasprintf(c"Unset %u tagged? ", fmt_args![tagged]);
                        let data_ref = data.owner().expect("window customize owner");
                        data.borrow_mut().change = WINDOW_CUSTOMIZE_UNSET;
                        status_prompt_set(
                            c,
                            None,
                            &prompt,
                            Some(c""),
                            Prompt::CustomizeChangeTagged,
                            PromptData::CustomizeChange(Box::new(data_ref)),
                            PROMPT_SINGLE | PROMPT_NOFORMAT | prompt_flags,
                            PromptHistoryType::Command,
                        );
                    }
                }
                72 => {
                    {
                        let mut state = data.borrow_mut();
                        state.hide_global = (state.hide_global == 0) as core::ffi::c_int;
                    }
                    ((*data).tree_ref()).build();
                }
                _ => {}
            }
            if finished != 0 {
                let pane = tree.borrow().pane();
                if let Some(mut pane) = pane
                    && let Some(pane) = pane.get_mut()
                {
                    (pane).reset_mode();
                }
            } else {
                ((*data).tree_ref()).draw();
                let pane = tree.borrow().pane();
                if let Some(mut pane) = pane
                    && let Some(pane) = pane.get_mut()
                {
                    *pane.flags_mut() |= PANE_REDRAW;
                }
            };
        }
    }
}

#[cfg(test)]
pub use crate::consts::{KEYC_DOWN, KEYC_END, KEYC_HOME, KEYC_NPAGE, KEYC_PPAGE, KEYC_UP};
