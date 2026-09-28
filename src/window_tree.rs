use crate::src::options::options_owner_ptr;
use crate::src::arguments::{args_count, args_get, args_has, args_string};
use crate::src::cmd::find::{cmd_find_clear_state, cmd_find_from_winlink_pane};
use crate::src::cmd::queue::{cmdq_append, cmdq_get_callback_owned};
use crate::src::ffi::libc::{__ctype_tolower_loc, memcpy, strcasestr, strstr};
use crate::src::format::{
    format_add, format_create, format_defaults, format_expand_cstring, format_free,
    format_single_cstring, format_true,
};
use crate::src::format_draw::{format_draw, format_trim_left_bytes, format_width};
use crate::src::grid::grid_default_cell;
use crate::src::key_string::key_string_parse_cstr;
use crate::src::mode_tree::{
    mode_tree_add, mode_tree_align, mode_tree_build, mode_tree_count_tagged, mode_tree_draw,
    mode_tree_each_tagged, mode_tree_expand, mode_tree_expand_current, mode_tree_free,
    mode_tree_get_current, mode_tree_key, mode_tree_remove, mode_tree_resize,
    mode_tree_run_command, mode_tree_set_current, mode_tree_set_prompt, mode_tree_start,
    mode_tree_view_name, mode_tree_zoom,
};
use crate::src::options::options_get_string;
use crate::src::osdep_linux::osdep_get_name_cstring;
use crate::src::resize::recalculate_sizes;
use crate::src::screen_write::{
    screen_write_box, screen_write_clearcharacter, screen_write_cursormove, screen_write_hline,
    screen_write_preview, screen_write_putc, screen_write_puts, screen_write_vline,
};
use crate::src::server::{server_clear_marked, server_set_marked};
use crate::src::server_fn::{
    server_destroy_session, server_kill_pane, server_kill_window, server_redraw_session_group,
    server_renumber_all,
};
use crate::src::session::{
    session_destroy, session_find_by_id, session_group_contains, session_group_synchronize_from,
    session_set_current,
};
use crate::src::shared::abi::__int32_t;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::args;
use crate::src::shared::client::client;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd_find_state, cmdq_item};
use crate::src::shared::format::format_tree;
use crate::src::shared::format::{FORMAT_NONE, FORMAT_PANE, FORMAT_WINDOW};
use crate::src::shared::grid::*;
use crate::src::shared::key::*;
use crate::src::shared::layout::*;
use crate::src::shared::menu::menu_item;
use crate::src::shared::mode_tree::{
    mode_tree_data, mode_tree_help_info, ModeTreeItemData, ModeTreeItemRef,
};
use crate::src::shared::mouse::mouse_event;
use crate::src::shared::options::options;
use crate::src::shared::pane::window_pane;
use crate::src::shared::pane::PANE_REDRAW;
use crate::src::shared::prompt::*;
use crate::src::shared::prompt::{
    prompt_result, PROMPT_ACCEPT, PROMPT_CLOSE, PROMPT_NOFORMAT, PROMPT_SINGLE,
};
use crate::src::shared::screen::screen;
use crate::src::shared::screen_write::screen_write_ctx;
use crate::src::shared::session::session;
use crate::src::shared::session::session_group;
use crate::src::shared::sort::sort_criteria;
use crate::src::shared::sort::*;
use crate::src::shared::style::*;
use crate::src::shared::tree::RB_NEGINF;
use crate::src::shared::window::{window, window_mode, window_mode_entry, winlink};
use crate::src::sort::{
    sort_get_panes_window, sort_get_sessions, sort_get_winlinks_session,
    sort_would_window_tree_swap,
};
use crate::src::style::style_apply;
use crate::src::window::window_pane_upgrade;
use crate::src::window::{
    window_count_panes, window_has_pane, window_pane_find_by_id, window_pane_first,
    window_pane_index, window_pane_next, window_pane_reset_mode, window_winlinks_append,
    window_winlinks_remove, winlink_count, winlink_find_by_index, winlinks_minmax, winlinks_next,
};
use std::cell::UnsafeCell;
use std::ffi::{CStr, CString};
use std::rc::{Rc, Weak};

#[repr(C)]
pub struct window_tree_modedata {
    /// Nonowning allocation observer for callbacks receiving borrowed pointers.
    pub(crate) observer: Weak<UnsafeCell<window_tree_modedata>>,
    pub wp: Weak<UnsafeCell<window_pane>>,
    pub dead: ::core::ffi::c_int,
    pub data: Option<std::rc::Rc<std::cell::UnsafeCell<mode_tree_data>>>,
    // Callback Rc references keep this record alive after mode shutdown.
    pub format: CString,
    pub key_format: CString,
    pub command: CString,
    pub squash_groups: ::core::ffi::c_int,
    pub hide_preview_this_pane: ::core::ffi::c_int,
    pub preview_is_info: ::core::ffi::c_int,
    pub prompt_flags: ::core::ffi::c_int,
    pub item_list: Vec<refbox::RefBox<window_tree_itemdata>>,
    pub entered: Option<::std::ffi::CString>,
    pub fs: cmd_find_state,
    pub type_0: window_tree_type,
    pub offset: ::core::ffi::c_int,
    pub left: ::core::ffi::c_int,
    pub right: ::core::ffi::c_int,
    pub start: u_int,
    pub end: u_int,
    pub each: u_int,
}

impl window_tree_modedata {
    fn data_ptr(&self) -> *mut mode_tree_data {
        self.data.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr)
    }
}

pub type window_tree_type = ::core::ffi::c_uint;
pub const WINDOW_TREE_PANE: window_tree_type = 3;
pub const WINDOW_TREE_WINDOW: window_tree_type = 2;
pub const WINDOW_TREE_SESSION: window_tree_type = 1;
pub const WINDOW_TREE_NONE: window_tree_type = 0;
/// Immutable row identity; actions resolve these IDs against the live model.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_tree_itemdata {
    pub type_0: window_tree_type,
    pub session: ::core::ffi::c_int,
    pub winlink: ::core::ffi::c_int,
    pub pane: ::core::ffi::c_int,
}
#[inline]
unsafe fn tolower(mut __c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    return if __c >= -(128 as ::core::ffi::c_int) && __c < 256 as ::core::ffi::c_int {
        *(*__ctype_tolower_loc()).offset(__c as isize) as ::core::ffi::c_int
    } else {
        __c
    };
}

pub const WINDOW_TREE_DEFAULT_COMMAND: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"switch-client -Zt '%%'\0")
};
pub const WINDOW_TREE_DEFAULT_FORMAT: &CStr = c"#{?pane_format,#{?pane_marked,#[fg=thememagenta],}#{?pane_floating_flag,#[underscore],}#{pane_current_command}#[fg=themelightgrey]#{pane_flags}#{?#{&&:#{pane_title},#{!=:#{pane_title},#{host_short}}},: \"#{pane_title}\",},window_format,#{?window_marked_flag,#[fg=thememagenta],}#{window_name}#[fg=themelightgrey]#{window_flags}#{?#{&&:#{==:#{window_panes},1},#{&&:#{pane_title},#{!=:#{pane_title},#{host_short}}}},: \"#{pane_title}\",},#[fg=themelightgrey]#{session_windows} windows#{?session_grouped, (group #{session_group}: #{session_group_list}),}#{?session_attached, (attached),}}";
pub const WINDOW_TREE_DEFAULT_KEY_FORMAT: [::core::ffi::c_char; 83] = unsafe {
    ::core::mem::transmute::<[u8; 83], [::core::ffi::c_char; 83]>(
        *b"#{?#{e|<:#{line},10},#{line},#{e|<:#{line},36},M-#{a:#{e|+:97,#{e|-:#{line},10}}}}\0",
    )
};
static window_tree_menu_items: [menu_item<'static>; 12] = [
    menu_item {
        name: c"Select",
        key: '\r' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: c"Expand",
        key: KEYC_RIGHT as ::core::ffi::c_ulong as key_code,
        command: None,
    },
    menu_item {
        name: c"Mark",
        key: 'm' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: c"",
        key: KEYC_NONE as ::core::ffi::c_ulong as key_code,
        command: None,
    },
    menu_item {
        name: c"Tag",
        key: 't' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: c"Tag All",
        key: '\u{14}' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: c"Tag None",
        key: 'T' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: c"",
        key: KEYC_NONE as ::core::ffi::c_ulong as key_code,
        command: None,
    },
    menu_item {
        name: c"Kill",
        key: 'x' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: c"Kill Tagged",
        key: 'X' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: c"",
        key: KEYC_NONE as ::core::ffi::c_ulong as key_code,
        command: None,
    },
    menu_item {
        name: c"Cancel",
        key: 'q' as i32 as key_code,
        command: None,
    },
];
pub static window_tree_mode: window_mode = {
    window_mode {
        name: c"tree-mode",
        default_format: Some(WINDOW_TREE_DEFAULT_FORMAT),
        flags: 0,
        init: Some(
            window_tree_init
                as unsafe fn(
                    *mut window_mode_entry,
                    *mut cmdq_item,
                    *mut cmd_find_state,
                    *mut args,
                ) -> *mut screen,
        ),
        free: Some(window_tree_free as unsafe fn(*mut window_mode_entry) -> ()),
        resize: Some(window_tree_resize as unsafe fn(*mut window_mode_entry, u_int, u_int) -> ()),
        update: Some(window_tree_update as unsafe fn(*mut window_mode_entry) -> ()),
        style_changed: None,
        key: Some(
            window_tree_key
                as unsafe fn(
                    *mut window_mode_entry,
                    &std::rc::Rc<std::cell::UnsafeCell<client>>,
                    *mut winlink,
                    key_code,
                    *mut mouse_event,
                ) -> (),
        ),
        key_table: None,
        command: None,
        formats: None,
        get_screen: None,
    }
};
static window_tree_order_seq: [sort_order; 4] = [SORT_INDEX, SORT_NAME, SORT_ACTIVITY, SORT_Z];
static mut window_tree_pane_info_lines: [*const ::core::ffi::c_char; 8] = [
    b"#[fg=themelightgrey]Pane          #[#{E:tree-mode-border-style},acs]x#[default] #{pane_index} #[fg=themelightgrey](#{pane_id})#[default]\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Title         #[#{E:tree-mode-border-style},acs]x#[default] #{pane_title}\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Command       #[#{E:tree-mode-border-style},acs]x#[default] #{pane_current_command} #[fg=themelightgrey](PID #{pane_pid})#[default]\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Path          #[#{E:tree-mode-border-style},acs]x#[default] #{pane_current_path}\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]TTY           #[#{E:tree-mode-border-style},acs]x#[default] #{pane_tty}\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Position      #[#{E:tree-mode-border-style},acs]x#[default] #{pane_x},#{pane_y} #{pane_width}x#{pane_height}\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Mode          #[#{E:tree-mode-border-style},acs]x#[default] #{?pane_in_mode,#{pane_mode},none}\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Flags         #[#{E:tree-mode-border-style},acs]x#[default] #{?pane_active,#[fg=themegreen],#[fg=themelightgrey]}active#[default] #{?window_zoomed_flag,#[fg=themegreen],#[fg=themelightgrey]}zoomed#[default] #{?pane_marked,#[fg=themegreen],#[fg=themelightgrey]}marked#[default] #{?pane_synchronized,#[fg=themegreen],#[fg=themelightgrey]}sync#[default] #{?pane_dead,#[fg=themegreen],#[fg=themelightgrey]}dead#[default] #{?pane_pipe,#[fg=themegreen],#[fg=themelightgrey]}piped#[default]\0"
        as *const u8 as *const ::core::ffi::c_char,
];
static mut window_tree_window_info_lines: [*const ::core::ffi::c_char; 6] = [
    b"#[fg=themelightgrey]Window        #[#{E:tree-mode-border-style},acs]x#[default] #{window_index}: #{window_name} #[fg=themelightgrey](#{window_id})#[default]\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Size          #[#{E:tree-mode-border-style},acs]x#[default] #{window_width}x#{window_height}\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Panes         #[#{E:tree-mode-border-style},acs]x#[default] #{window_panes}\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Activity Time #[#{E:tree-mode-border-style},acs]x#[default] #{t:window_activity} #[fg=themelightgrey](#{t/r:window_activity})#[default]\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Sessions      #[#{E:tree-mode-border-style},acs]x#[default] #{s/,/ /:window_linked_sessions_list}\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Flags         #[#{E:tree-mode-border-style},acs]x#[default] #{?window_active,#[fg=themegreen],#[fg=themelightgrey]}active#[default] #{?window_last_flag,#[fg=themegreen],#[fg=themelightgrey]}last#[default] #{?window_bell_flag,#[fg=themegreen],#[fg=themelightgrey]}bell#[default] #{?window_activity_flag,#[fg=themegreen],#[fg=themelightgrey]}activity#[default] #{?window_silence_flag,#[fg=themegreen],#[fg=themelightgrey]}silence#[default] #{?window_zoomed_flag,#[fg=themegreen],#[fg=themelightgrey]}zoomed#[default] #{?window_marked_flag,#[fg=themegreen],#[fg=themelightgrey]}marked#[default]\0"
        as *const u8 as *const ::core::ffi::c_char,
];
static mut window_tree_session_info_lines: [*const ::core::ffi::c_char; 9] = [
    b"#[fg=themelightgrey]Session       #[#{E:tree-mode-border-style},acs]x#[default] #{session_name} #[fg=themelightgrey](#{session_id})#[default]\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Created Time  #[#{E:tree-mode-border-style},acs]x#[default] #{t:session_created} #[fg=themelightgrey](#{t/r:session_created})#[default]\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Activity Time #[#{E:tree-mode-border-style},acs]x#[default] #{t:session_activity} #[fg=themelightgrey](#{t/r:session_activity})#[default]\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Attached Time #[#{E:tree-mode-border-style},acs]x#[default] #{?#{t:session_last_attached},#{t:session_last_attached} #[fg=themelightgrey](#{t/r:session_last_attached})#[default],never}\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Clients       #[#{E:tree-mode-border-style},acs]x#[default] #{s/,/ /:session_attached_list}\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Windows       #[#{E:tree-mode-border-style},acs]x#[default] #{session_windows}\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Path          #[#{E:tree-mode-border-style},acs]x#[default] #{session_path}\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Group         #[#{E:tree-mode-border-style},acs]x#[default] #{?session_grouped,#{session_group} (#{session_group_size}),none}\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Flags         #[#{E:tree-mode-border-style},acs]x#[default] #{?session_attached,#[fg=themegreen],#[fg=themelightgrey]}attached#[default] #{?session_grouped,#[fg=themegreen],#[fg=themelightgrey]}grouped#[default] #{?session_marked,#[fg=themegreen],#[fg=themelightgrey]}marked#[default] #{?session_bell_flag,#[fg=themegreen],#[fg=themelightgrey]}bell#[default] #{?session_activity_flag,#[fg=themegreen],#[fg=themelightgrey]}activity#[default] #{?session_silence_flag,#[fg=themegreen],#[fg=themelightgrey]}silence#[default]\0"
        as *const u8 as *const ::core::ffi::c_char,
];
#[must_use = "retain the resolved session and pane while using the output pointers"]
#[derive(Default)]
struct WindowTreeTarget {
    session: Option<Rc<UnsafeCell<session>>>,
    winlink: refbox::Weak<winlink>,
    pane: Option<Rc<UnsafeCell<window_pane>>>,
}

unsafe fn window_tree_pull_item(item: &window_tree_itemdata) -> WindowTreeTarget {
    let Some(session) = session_find_by_id(item.session as u_int) else {
        return WindowTreeTarget::default();
    };
    let s = session.get();
    let wl = if item.type_0 == WINDOW_TREE_SESSION {
        (*s).curw
    } else {
        winlink_find_by_index(&raw mut (*s).windows, item.winlink)
    };
    let Some(link) = wl.as_ref() else { return WindowTreeTarget::default(); };
    let pane = if item.type_0 == WINDOW_TREE_SESSION || item.type_0 == WINDOW_TREE_WINDOW {
        (*link.window_ptr()).active.as_ref().and_then(|pane| pane.observer.upgrade())
    } else {
        let pane = window_pane_find_by_id(item.pane as u_int);
        if !pane.as_ref().is_some_and(|owner| window_has_pane(&*link.window_ptr(), &Rc::downgrade(owner))) {
            return WindowTreeTarget::default();
        }
        pane
    };
    WindowTreeTarget { session: Some(session), winlink: link.observer.clone(), pane }
}
fn window_tree_add_item(
    items: &mut Vec<refbox::RefBox<window_tree_itemdata>>,
    value: window_tree_itemdata,
) -> refbox::Weak<window_tree_itemdata> {
    let item = refbox::RefBox::new(value);
    let handle = item.downgrade();
    items.push(item);
    handle
}
fn window_tree_remove_last_item(
    items: &mut Vec<refbox::RefBox<window_tree_itemdata>>,
    tree: &mut mode_tree_data,
    item: &refbox::Weak<window_tree_itemdata>,
    mti: &ModeTreeItemRef,
) {
    debug_assert!(items.last().is_some_and(|last| item.is(last)));
    mode_tree_remove(tree, mti);
    items.pop();
}
unsafe fn window_tree_build_pane(
    session_owner: &Rc<UnsafeCell<session>>,
    mut wl: *mut winlink,
    pane_owner: &Rc<UnsafeCell<window_pane>>,
    mode_owner: &Rc<UnsafeCell<window_tree_modedata>>,
    parent: &ModeTreeItemRef,
) {
    let s = session_owner.get();
    let wp = pane_owner.get();
    let data = mode_owner.get();
    let mut idx: u_int = 0;
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    idx = window_pane_index(&*wp).expect("pane belongs to window ordering");
    let item_owner = window_tree_add_item(
        &mut (*data).item_list,
        window_tree_itemdata {
            type_0: WINDOW_TREE_PANE,
            session: (*s).id as ::core::ffi::c_int,
            winlink: (*wl).idx,
            pane: (*wp).id as ::core::ffi::c_int,
        },
    );
    ft = format_create(
        ::core::ptr::null_mut::<client>(),
        ::core::ptr::null_mut::<cmdq_item>(),
        (FORMAT_PANE | (*wp).id) as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    format_defaults(ft, ::core::ptr::null_mut::<client>(), s, wl, wp);
    let text = format_expand_cstring(ft, (*data).format.as_ptr());
    let name = CString::new(idx.to_string()).expect("pane index contains NUL");
    format_free(ft);
    let mti = mode_tree_add(
        &mut *(*data).data_ptr(),
        Some(parent),
        ModeTreeItemData::Tree(item_owner.clone()),
        wp as uint64_t,
        &name,
        Some(&text),
        -(1 as ::core::ffi::c_int),
    );
    mode_tree_align(&mti);
}
unsafe fn window_tree_filter_pane(
    session_owner: &Rc<UnsafeCell<session>>,
    mut wl: *mut winlink,
    pane_owner: &Rc<UnsafeCell<window_pane>>,
    mut filter: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut result: ::core::ffi::c_int = 0;
    if filter.is_null() {
        return 1 as ::core::ffi::c_int;
    }
    let cp = format_single_cstring(
        ::core::ptr::null_mut::<cmdq_item>(),
        filter,
        ::core::ptr::null_mut::<client>(),
        session_owner.get(),
        wl,
        pane_owner.get(),
    );
    result = format_true(cp.as_ptr());
    return result;
}
unsafe fn window_tree_build_window(
    session_owner: &Rc<UnsafeCell<session>>,
    mut wl: *mut winlink,
    mode_owner: &Rc<UnsafeCell<window_tree_modedata>>,
    mut sort_crit: *mut sort_criteria,
    parent: &ModeTreeItemRef,
    mut filter: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let s = session_owner.get();
    let data = mode_owner.get();
    let Some(mode_pane_owner) = window_pane_upgrade(&(*data).wp) else {
        return 0;
    };
    let mode_pane = crate::src::shared::rc::as_ptr(&mode_pane_owner);
    let mut i: u_int = 0;
    let mut found: u_int = 0;
    let mut expanded: ::core::ffi::c_int = 0;
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut tag: uint64_t = FORMAT_NONE as uint64_t;
    let item_owner = window_tree_add_item(
        &mut (*data).item_list,
        window_tree_itemdata {
            type_0: WINDOW_TREE_WINDOW,
            session: (*s).id as ::core::ffi::c_int,
            winlink: (*wl).idx,
            pane: -(1 as ::core::ffi::c_int),
        },
    );
    if !(*wl).window_ptr().is_null() && !(*(*wl).window_ptr()).active.is_null() {
        tag = (FORMAT_PANE | (*(*(*wl).window_ptr()).active).id) as uint64_t;
    }
    ft = format_create(
        ::core::ptr::null_mut::<client>(),
        ::core::ptr::null_mut::<cmdq_item>(),
        tag as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    format_defaults(
        ft,
        ::core::ptr::null_mut::<client>(),
        s,
        wl,
        ::core::ptr::null_mut::<window_pane>(),
    );
    let text = format_expand_cstring(ft, (*data).format.as_ptr());
    let name = CString::new(((*wl).idx as u_int).to_string()).expect("window index contains NUL");
    format_free(ft);
    if (*data).type_0 as ::core::ffi::c_uint
        == WINDOW_TREE_SESSION as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*data).type_0 as ::core::ffi::c_uint
            == WINDOW_TREE_WINDOW as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        expanded = 0 as ::core::ffi::c_int;
    } else {
        expanded = 1 as ::core::ffi::c_int;
    }
    let mti = mode_tree_add(
        &mut *(*data).data_ptr(),
        Some(parent),
        ModeTreeItemData::Tree(item_owner.clone()),
        wl as uint64_t,
        &name,
        Some(&text),
        expanded,
    );
    mode_tree_align(&mti);
    let l = sort_get_panes_window(&*(*wl).window_ptr(), &*sort_crit);
    let n = u_int::try_from(l.len()).expect("too many panes in window tree");
    found = 0 as u_int;
    i = 0 as u_int;
    while i < n {
        if !(window_tree_filter_pane(session_owner, wl, &l[i as usize], filter) == 0) {
            found = found.wrapping_add(1);
            if !((*data).hide_preview_this_pane != 0 && l[i as usize].get() == mode_pane) {
                window_tree_build_pane(session_owner, wl, &l[i as usize], mode_owner, &mti);
            }
        }
        i = i.wrapping_add(1);
    }
    if found == 0 as u_int {
        let tree_owner = (*data).data.as_ref().expect("mode tree owner").clone();
        window_tree_remove_last_item(
            &mut (*data).item_list, &mut *tree_owner.get(), &item_owner, &mti,
        );
        return 0 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe fn window_tree_build_session(
    session_owner: &Rc<UnsafeCell<session>>,
    mode_owner: &Rc<UnsafeCell<window_tree_modedata>>,
    mut sort_crit: *mut sort_criteria,
    mut filter: *const ::core::ffi::c_char,
) {
    let s = session_owner.get();
    let data = mode_owner.get();
    let mut wl: *mut winlink = (*s).curw;
    let mut i: u_int = 0;
    let mut empty: u_int = 0;
    let mut expanded: ::core::ffi::c_int = 0;
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut tag: uint64_t = FORMAT_NONE as uint64_t;
    let item_owner = window_tree_add_item(
        &mut (*data).item_list,
        window_tree_itemdata {
            type_0: WINDOW_TREE_SESSION,
            session: (*s).id as ::core::ffi::c_int,
            winlink: -(1 as ::core::ffi::c_int),
            pane: -(1 as ::core::ffi::c_int),
        },
    );
    if !wl.is_null() && !(*wl).window_ptr().is_null() && !(*(*wl).window_ptr()).active.is_null() {
        tag = (FORMAT_PANE | (*(*(*wl).window_ptr()).active).id) as uint64_t;
    }
    ft = format_create(
        ::core::ptr::null_mut::<client>(),
        ::core::ptr::null_mut::<cmdq_item>(),
        tag as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    format_defaults(
        ft,
        ::core::ptr::null_mut::<client>(),
        s,
        ::core::ptr::null_mut::<winlink>(),
        ::core::ptr::null_mut::<window_pane>(),
    );
    let text = format_expand_cstring(ft, (*data).format.as_ptr());
    format_free(ft);
    if (*data).type_0 as ::core::ffi::c_uint
        == WINDOW_TREE_SESSION as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        expanded = 0 as ::core::ffi::c_int;
    } else {
        expanded = 1 as ::core::ffi::c_int;
    }
    let mti = mode_tree_add(
        &mut *(*data).data_ptr(),
        None,
        ModeTreeItemData::Tree(item_owner.clone()),
        s as uint64_t,
        &(*s).name,
        Some(&text),
        expanded,
    );
    let l = sort_get_winlinks_session(s, sort_crit);
    let n = u_int::try_from(l.len()).expect("too many winlinks in window tree");
    empty = 0 as u_int;
    i = 0 as u_int;
    while i < n {
        if window_tree_build_window(session_owner, l[i as usize], mode_owner, sort_crit, &mti, filter) == 0 {
            empty = empty.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    if empty == n {
        let tree_owner = (*data).data.as_ref().expect("mode tree owner").clone();
        window_tree_remove_last_item(
            &mut (*data).item_list, &mut *tree_owner.get(), &item_owner, &mti,
        );
    }
}
unsafe fn window_tree_live_mode(
    observer: &std::rc::Weak<UnsafeCell<window_tree_modedata>>,
) -> Option<Rc<UnsafeCell<window_tree_modedata>>> {
    let owner = observer.upgrade()?;
    if (*owner.get()).dead != 0 { return None; }
    Some(owner)
}

unsafe fn window_tree_build(
    mode_owner: &Rc<UnsafeCell<window_tree_modedata>>,
    mut sort_crit: *mut sort_criteria,
    tag: *mut uint64_t,
    mut filter: *const ::core::ffi::c_char,
) {
    let data = mode_owner.get();
    let mut squash_groups: ::core::ffi::c_int = (*data).squash_groups;
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    let mut current: *mut session_group = ::core::ptr::null_mut::<session_group>();
    let mut i: u_int = 0;
    current = session_group_contains((*data).fs.s_ptr());
    (*data).item_list.clear();
    let l = sort_get_sessions(&*sort_crit);
    let n = u_int::try_from(l.len()).expect("too many sessions for window tree");
    if n == 0 as u_int {
        return;
    }
    let mut current_block_12: u64;
    i = 0 as u_int;
    while i < n {
        s = l[i as usize].get();
        if squash_groups != 0 && {
            sg = session_group_contains(s);
            !sg.is_null()
        } {
            if sg == current && s != (*data).fs.s_ptr()
                || sg != current
                    && Some(s)
                        != crate::src::session::session_group_members(sg)
                            .first()
                            .map(|owner| owner.get())
            {
                current_block_12 = 3640593987805443782;
            } else {
                current_block_12 = 4166486009154926805;
            }
        } else {
            current_block_12 = 4166486009154926805;
        }
        match current_block_12 {
            4166486009154926805 => {
                window_tree_build_session(&l[i as usize], mode_owner, sort_crit, filter);
            }
            _ => {}
        }
        i = i.wrapping_add(1);
    }
    match (*data).type_0 as ::core::ffi::c_uint {
        1 => {
            if !(*data).fs.s_ptr().is_null() {
                *tag = (*data).fs.s_ptr() as uint64_t;
            }
        }
        2 => {
            if !(*data).fs.s_ptr().is_null() && !(*data).fs.wl_ptr().is_null() {
                *tag = (*data).fs.wl_ptr() as uint64_t;
            }
        }
        3 => {
            if window_count_panes(&*(*(*data).fs.wl_ptr()).window_ptr(), 1 as ::core::ffi::c_int) == 1 as u_int {
                *tag = (*data).fs.wl_ptr() as uint64_t;
            } else {
                *tag = (*data).fs.wp_ptr() as uint64_t;
            }
        }
        0 | _ => {}
    };
}
unsafe fn window_tree_draw_label(
    mut ctx: *mut screen_write_ctx,
    mut px: u_int,
    mut py: u_int,
    mut sx: u_int,
    mut sy: u_int,
    mut border_gc: *const grid_cell,
    mut label_gc: *const grid_cell,
    mut label: *const ::core::ffi::c_char,
) {
    let mut width: u_int = 0;
    let mut ox: u_int = 0;
    let mut oy: u_int = 0;
    if sx < 5 as u_int || sy < 3 as u_int {
        return;
    }
    width = format_width(label);
    let trimmed_label = if width > sx.wrapping_sub(4 as u_int) {
        Some(
            CString::new(format_trim_left_bytes(
                CStr::from_ptr(label),
                sx.wrapping_sub(4 as u_int),
            ))
            .expect("trimmed preview label contains no NUL"),
        )
    } else {
        None
    };
    if let Some(trimmed_label) = trimmed_label.as_ref() {
        label = trimmed_label.as_ptr();
        width = format_width(label);
    }
    if width == 0 as u_int {
        return;
    }
    ox = sx
        .wrapping_sub(width)
        .wrapping_add(1 as u_int)
        .wrapping_div(2 as u_int);
    oy = sy.wrapping_add(1 as u_int).wrapping_div(2 as u_int);
    screen_write_cursormove(
        &mut *ctx,
        px.wrapping_add(ox).wrapping_sub(2 as u_int) as ::core::ffi::c_int,
        py.wrapping_add(oy).wrapping_sub(1 as u_int) as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    screen_write_box(
        &mut *ctx,
        width.wrapping_add(4 as u_int),
        3 as u_int,
        BOX_LINES_DEFAULT,
        border_gc.as_ref(),
        None,
    );
    screen_write_cursormove(
        &mut *ctx,
        px.wrapping_add(ox).wrapping_sub(1 as u_int) as ::core::ffi::c_int,
        py.wrapping_add(oy) as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    screen_write_clearcharacter(
        &mut *ctx,
        width.wrapping_add(2 as u_int),
        (*border_gc).bg as u_int,
    );
    screen_write_cursormove(
        &mut *ctx,
        px.wrapping_add(ox) as ::core::ffi::c_int,
        py.wrapping_add(oy) as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    format_draw(
        ctx,
        label_gc,
        width,
        label,
        ::core::ptr::null_mut::<style_ranges>(),
        0 as ::core::ffi::c_int,
    );
}
unsafe fn window_tree_border_cell(
    mut gc: *mut grid_cell,
    mut oo: *mut options,
    mut ft: *mut format_tree,
) {
    memcpy(
        gc as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    style_apply(
        gc,
        oo,
        b"tree-mode-border-style\0" as *const u8 as *const ::core::ffi::c_char,
        ft,
    );
}
unsafe fn window_tree_draw_session(
    mode_owner: &Rc<UnsafeCell<window_tree_modedata>>,
    session_owner: &Rc<UnsafeCell<session>>,
    mut ctx: *mut screen_write_ctx,
    mut sx: u_int,
    mut sy: u_int,
) {
    let data = mode_owner.get();
    let s = session_owner.get();
    let Some(mode_pane_owner) = window_pane_upgrade(&(*data).wp) else {
        return;
    };
    let mode_pane = crate::src::shared::rc::as_ptr(&mode_pane_owner);
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut cx: u_int = (*(*ctx).s).cx;
    let mut cy: u_int = (*(*ctx).s).cy;
    let mut loop_0: u_int = 0;
    let mut total: u_int = 0;
    let mut visible: u_int = 0;
    let mut each: u_int = 0;
    let mut width: u_int = 0;
    let mut offset: u_int = 0;
    let mut current: u_int = 0;
    let mut start: u_int = 0;
    let mut end: u_int = 0;
    let mut remaining: u_int = 0;
    let mut i: u_int = 0;
    let mut gc: grid_cell = grid_cell {
        data: utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        },
        attr: 0,
        flags: 0,
        fg: 0,
        bg: 0,
        us: 0,
        link: 0,
    };
    let mut label_gc: grid_cell = grid_cell {
        data: utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        },
        attr: 0,
        flags: 0,
        fg: 0,
        bg: 0,
        us: 0,
        link: 0,
    };
    let mut left: ::core::ffi::c_int = 0;
    let mut right: ::core::ffi::c_int = 0;
    let mut format: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    total = winlink_count(&raw mut (*s).windows);
    if sx.wrapping_div(total) < 24 as u_int {
        visible = sx.wrapping_div(24 as u_int);
        if visible == 0 as u_int {
            visible = 1 as u_int;
        }
    } else {
        visible = total;
    }
    current = 0 as u_int;
    wl = winlinks_minmax(&(*s).windows, RB_NEGINF);
    while !wl.is_null() {
        if wl == (*s).curw {
            break;
        }
        current = current.wrapping_add(1);
        wl = winlinks_next(&*wl);
    }
    if current < visible {
        start = 0 as u_int;
        end = visible;
    } else if current >= total.wrapping_sub(visible) {
        start = total.wrapping_sub(visible);
        end = total;
    } else {
        start = current.wrapping_sub(visible.wrapping_div(2 as u_int));
        end = start.wrapping_add(visible);
    }
    if (*data).offset < -(start as ::core::ffi::c_int) {
        (*data).offset = -(start as ::core::ffi::c_int);
    }
    if (*data).offset > total.wrapping_sub(end) as ::core::ffi::c_int {
        (*data).offset = total.wrapping_sub(end) as ::core::ffi::c_int;
    }
    start = start.wrapping_add((*data).offset as u_int);
    end = end.wrapping_add((*data).offset as u_int);
    left = (start != 0 as u_int) as ::core::ffi::c_int;
    right = (end != total) as ::core::ffi::c_int;
    if left != 0 && right != 0 && sx <= 6 as u_int || (left != 0 || right != 0) && sx <= 3 as u_int
    {
        right = 0 as ::core::ffi::c_int;
        left = right;
    }
    if left != 0 && right != 0 {
        each = sx.wrapping_sub(6 as u_int).wrapping_div(visible);
        remaining = sx
            .wrapping_sub(6 as u_int)
            .wrapping_sub(visible.wrapping_mul(each));
    } else if left != 0 || right != 0 {
        each = sx.wrapping_sub(3 as u_int).wrapping_div(visible);
        remaining = sx
            .wrapping_sub(3 as u_int)
            .wrapping_sub(visible.wrapping_mul(each));
    } else {
        each = sx.wrapping_div(visible);
        remaining = sx.wrapping_sub(visible.wrapping_mul(each));
    }
    if each == 0 as u_int {
        return;
    }
    window_tree_border_cell(
        &raw mut gc,
        options_owner_ptr(&mut (*(*mode_pane).window).options).map_or(std::ptr::null_mut(), |options| options),
        ::core::ptr::null_mut::<format_tree>(),
    );
    if left != 0 {
        (*data).left = cx.wrapping_add(2 as u_int) as ::core::ffi::c_int;
        screen_write_cursormove(
            &mut *ctx,
            cx.wrapping_add(2 as u_int) as ::core::ffi::c_int,
            cy as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_vline(&mut *ctx, sy, Some(&gc));
        screen_write_cursormove(
            &mut *ctx,
            cx as ::core::ffi::c_int,
            cy.wrapping_add(sy.wrapping_div(2 as u_int)) as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_puts(&mut *ctx, &gc, |out| out.write_all(b"<"));
    } else {
        (*data).left = -(1 as ::core::ffi::c_int);
    }
    if right != 0 {
        (*data).right = cx.wrapping_add(sx).wrapping_sub(3 as u_int) as ::core::ffi::c_int;
        screen_write_cursormove(
            &mut *ctx,
            cx.wrapping_add(sx).wrapping_sub(3 as u_int) as ::core::ffi::c_int,
            cy as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_vline(&mut *ctx, sy, Some(&gc));
        screen_write_cursormove(
            &mut *ctx,
            cx.wrapping_add(sx).wrapping_sub(1 as u_int) as ::core::ffi::c_int,
            cy.wrapping_add(sy.wrapping_div(2 as u_int)) as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_puts(&mut *ctx, &gc, |out| out.write_all(b">"));
    } else {
        (*data).right = -(1 as ::core::ffi::c_int);
    }
    (*data).start = start;
    (*data).end = end;
    (*data).each = each;
    loop_0 = 0 as u_int;
    i = loop_0;
    wl = winlinks_minmax(&(*s).windows, RB_NEGINF);
    while !wl.is_null() {
        if loop_0 == end {
            break;
        }
        if loop_0 < start {
            loop_0 = loop_0.wrapping_add(1);
        } else {
            w = (*wl).window_ptr();
            oo = options_owner_ptr(&mut (*w).options).map_or(std::ptr::null_mut(), |options| options);
            ft = format_create(
                ::core::ptr::null_mut::<client>(),
                ::core::ptr::null_mut::<cmdq_item>(),
                (FORMAT_WINDOW | (*w).id) as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            format_defaults(
                ft,
                ::core::ptr::null_mut::<client>(),
                s,
                wl,
                ::core::ptr::null_mut::<window_pane>(),
            );
            window_tree_border_cell(&raw mut gc, oo, ft);
            memcpy(
                &raw mut label_gc as *mut ::core::ffi::c_void,
                &raw const grid_default_cell as *const ::core::ffi::c_void,
                ::core::mem::size_of::<grid_cell>() as size_t,
            );
            style_apply(
                &raw mut label_gc,
                oo,
                b"tree-mode-preview-style\0" as *const u8 as *const ::core::ffi::c_char,
                ft,
            );
            label_gc.bg = gc.bg;
            if left != 0 {
                offset = (3 as u_int).wrapping_add(i.wrapping_mul(each));
            } else {
                offset = i.wrapping_mul(each);
            }
            if loop_0 == end.wrapping_sub(1 as u_int) {
                width = each.wrapping_add(remaining);
            } else {
                width = each.wrapping_sub(1 as u_int);
            }
            screen_write_cursormove(
                &mut *ctx,
                cx.wrapping_add(offset) as ::core::ffi::c_int,
                cy as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            screen_write_preview(&mut *ctx, &(*(*w).active).base, width, sy);
            format = options_get_string(
                oo,
                b"tree-mode-preview-format\0" as *const u8 as *const ::core::ffi::c_char,
            );
            if *format as ::core::ffi::c_int != '\0' as i32 {
                let label = format_expand_cstring(ft, format);
                if !label.as_bytes().is_empty() {
                    window_tree_draw_label(
                        ctx,
                        cx.wrapping_add(offset),
                        cy,
                        width,
                        sy,
                        &raw mut gc,
                        &raw mut label_gc,
                        label.as_ptr(),
                    );
                }
            }
            format_free(ft);
            if loop_0 != end.wrapping_sub(1 as u_int) {
                screen_write_cursormove(
                    &mut *ctx,
                    cx.wrapping_add(offset).wrapping_add(width) as ::core::ffi::c_int,
                    cy as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
                screen_write_vline(&mut *ctx, sy, Some(&gc));
            }
            loop_0 = loop_0.wrapping_add(1);
            i = i.wrapping_add(1);
        }
        wl = winlinks_next(&*wl);
    }
}
unsafe fn window_tree_draw_window(
    mode_owner: &Rc<UnsafeCell<window_tree_modedata>>,
    session_owner: &Rc<UnsafeCell<session>>,
    mut wl: *mut winlink,
    mut ctx: *mut screen_write_ctx,
    mut sx: u_int,
    mut sy: u_int,
) {
    let data = mode_owner.get();
    let s = session_owner.get();
    let Some(mode_pane_owner) = window_pane_upgrade(&(*data).wp) else {
        return;
    };
    let mode_pane = crate::src::shared::rc::as_ptr(&mode_pane_owner);
    let mut w: *mut window = (*wl).window_ptr();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut cx: u_int = (*(*ctx).s).cx;
    let mut cy: u_int = (*(*ctx).s).cy;
    let mut loop_0: u_int = 0;
    let mut total: u_int = 0;
    let mut visible: u_int = 0;
    let mut each: u_int = 0;
    let mut width: u_int = 0;
    let mut offset: u_int = 0;
    let mut current: u_int = 0;
    let mut start: u_int = 0;
    let mut end: u_int = 0;
    let mut remaining: u_int = 0;
    let mut i: u_int = 0;
    let mut gc: grid_cell = grid_cell {
        data: utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        },
        attr: 0,
        flags: 0,
        fg: 0,
        bg: 0,
        us: 0,
        link: 0,
    };
    let mut label_gc: grid_cell = grid_cell {
        data: utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        },
        attr: 0,
        flags: 0,
        fg: 0,
        bg: 0,
        us: 0,
        link: 0,
    };
    let mut left: ::core::ffi::c_int = 0;
    let mut right: ::core::ffi::c_int = 0;
    let mut format: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    total = window_count_panes(&*w, 1 as ::core::ffi::c_int);
    if (*data).hide_preview_this_pane != 0 && (*mode_pane).window == w {
        total = total.wrapping_sub(1);
    }
    if total == 0 as u_int {
        return;
    }
    if sx.wrapping_div(total) < 24 as u_int {
        visible = sx.wrapping_div(24 as u_int);
        if visible == 0 as u_int {
            visible = 1 as u_int;
        }
    } else {
        visible = total;
    }
    current = 0 as u_int;
    wp = window_pane_first(w).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    while !wp.is_null() {
        if !((*data).hide_preview_this_pane != 0 && wp == mode_pane) {
            if wp == (*w).active {
                break;
            }
            current = current.wrapping_add(1);
        }
        wp = window_pane_next(wp).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    }
    if current < visible {
        start = 0 as u_int;
        end = visible;
    } else if current >= total.wrapping_sub(visible) {
        start = total.wrapping_sub(visible);
        end = total;
    } else {
        start = current.wrapping_sub(visible.wrapping_div(2 as u_int));
        end = start.wrapping_add(visible);
    }
    if (*data).offset < -(start as ::core::ffi::c_int) {
        (*data).offset = -(start as ::core::ffi::c_int);
    }
    if (*data).offset > total.wrapping_sub(end) as ::core::ffi::c_int {
        (*data).offset = total.wrapping_sub(end) as ::core::ffi::c_int;
    }
    start = start.wrapping_add((*data).offset as u_int);
    end = end.wrapping_add((*data).offset as u_int);
    left = (start != 0 as u_int) as ::core::ffi::c_int;
    right = (end != total) as ::core::ffi::c_int;
    if left != 0 && right != 0 && sx <= 6 as u_int || (left != 0 || right != 0) && sx <= 3 as u_int
    {
        right = 0 as ::core::ffi::c_int;
        left = right;
    }
    if left != 0 && right != 0 {
        each = sx.wrapping_sub(6 as u_int).wrapping_div(visible);
        remaining = sx
            .wrapping_sub(6 as u_int)
            .wrapping_sub(visible.wrapping_mul(each));
    } else if left != 0 || right != 0 {
        each = sx.wrapping_sub(3 as u_int).wrapping_div(visible);
        remaining = sx
            .wrapping_sub(3 as u_int)
            .wrapping_sub(visible.wrapping_mul(each));
    } else {
        each = sx.wrapping_div(visible);
        remaining = sx.wrapping_sub(visible.wrapping_mul(each));
    }
    if each == 0 as u_int {
        return;
    }
    window_tree_border_cell(
        &raw mut gc,
        options_owner_ptr(&mut (*(*mode_pane).window).options).map_or(std::ptr::null_mut(), |options| options),
        ::core::ptr::null_mut::<format_tree>(),
    );
    if left != 0 {
        (*data).left = cx.wrapping_add(2 as u_int) as ::core::ffi::c_int;
        screen_write_cursormove(
            &mut *ctx,
            cx.wrapping_add(2 as u_int) as ::core::ffi::c_int,
            cy as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_vline(&mut *ctx, sy, Some(&gc));
        screen_write_cursormove(
            &mut *ctx,
            cx as ::core::ffi::c_int,
            cy.wrapping_add(sy.wrapping_div(2 as u_int)) as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_puts(&mut *ctx, &gc, |out| out.write_all(b"<"));
    } else {
        (*data).left = -(1 as ::core::ffi::c_int);
    }
    if right != 0 {
        (*data).right = cx.wrapping_add(sx).wrapping_sub(3 as u_int) as ::core::ffi::c_int;
        screen_write_cursormove(
            &mut *ctx,
            cx.wrapping_add(sx).wrapping_sub(3 as u_int) as ::core::ffi::c_int,
            cy as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_vline(&mut *ctx, sy, Some(&gc));
        screen_write_cursormove(
            &mut *ctx,
            cx.wrapping_add(sx).wrapping_sub(1 as u_int) as ::core::ffi::c_int,
            cy.wrapping_add(sy.wrapping_div(2 as u_int)) as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_puts(&mut *ctx, &gc, |out| out.write_all(b">"));
    } else {
        (*data).right = -(1 as ::core::ffi::c_int);
    }
    (*data).start = start;
    (*data).end = end;
    (*data).each = each;
    loop_0 = 0 as u_int;
    i = loop_0;
    wp = window_pane_first(w).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    while !wp.is_null() {
        if !((*data).hide_preview_this_pane != 0 && wp == mode_pane) {
            if loop_0 == end {
                break;
            }
            if loop_0 < start {
                loop_0 = loop_0.wrapping_add(1);
            } else {
                oo = options_owner_ptr(&mut (*wp).options).map_or(std::ptr::null_mut(), |options| options);
                ft = format_create(
                    ::core::ptr::null_mut::<client>(),
                    ::core::ptr::null_mut::<cmdq_item>(),
                    (FORMAT_PANE | (*wp).id) as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
                format_defaults(ft, ::core::ptr::null_mut::<client>(), s, wl, wp);
                window_tree_border_cell(&raw mut gc, oo, ft);
                memcpy(
                    &raw mut label_gc as *mut ::core::ffi::c_void,
                    &raw const grid_default_cell as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<grid_cell>() as size_t,
                );
                style_apply(
                    &raw mut label_gc,
                    oo,
                    b"tree-mode-preview-style\0" as *const u8 as *const ::core::ffi::c_char,
                    ft,
                );
                label_gc.bg = gc.bg;
                if left != 0 {
                    offset = (3 as u_int).wrapping_add(i.wrapping_mul(each));
                } else {
                    offset = i.wrapping_mul(each);
                }
                if loop_0 == end.wrapping_sub(1 as u_int) {
                    width = each.wrapping_add(remaining);
                } else {
                    width = each.wrapping_sub(1 as u_int);
                }
                screen_write_cursormove(
                    &mut *ctx,
                    cx.wrapping_add(offset) as ::core::ffi::c_int,
                    cy as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
                screen_write_preview(&mut *ctx, &(*wp).base, width, sy);
                format = options_get_string(
                    oo,
                    b"tree-mode-preview-format\0" as *const u8 as *const ::core::ffi::c_char,
                );
                if *format as ::core::ffi::c_int != '\0' as i32 {
                    let label = format_expand_cstring(ft, format);
                    if !label.as_bytes().is_empty() {
                        window_tree_draw_label(
                            ctx,
                            cx.wrapping_add(offset),
                            cy,
                            width,
                            sy,
                            &raw mut gc,
                            &raw mut label_gc,
                            label.as_ptr(),
                        );
                    }
                }
                format_free(ft);
                if loop_0 != end.wrapping_sub(1 as u_int) {
                    screen_write_cursormove(
                        &mut *ctx,
                        cx.wrapping_add(offset).wrapping_add(width) as ::core::ffi::c_int,
                        cy as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    );
                    screen_write_vline(&mut *ctx, sy, Some(&gc));
                }
                loop_0 = loop_0.wrapping_add(1);
                i = i.wrapping_add(1);
            }
        }
        wp = window_pane_next(wp).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    }
}
unsafe fn window_tree_draw_info(
    mode_owner: &Rc<UnsafeCell<window_tree_modedata>>,
    item: &window_tree_itemdata,
    ctx: &mut screen_write_ctx,
    mut sx: u_int,
    mut sy: u_int,
) {
    let data = mode_owner.get();
    let Some(mode_pane_owner) = window_pane_upgrade(&(*data).wp) else {
        return;
    };
    let mode_pane = crate::src::shared::rc::as_ptr(&mode_pane_owner);
    let mut s: *mut screen = (*ctx).s;
    let mut sp: *mut session = ::core::ptr::null_mut::<session>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut gc: grid_cell = grid_cell {
        data: utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        },
        attr: 0,
        flags: 0,
        fg: 0,
        bg: 0,
        us: 0,
        link: 0,
    };
    let mut cx: u_int = (*s).cx;
    let mut cy: u_int = (*s).cy;
    let mut i: u_int = 0;
    let mut j: u_int = 0;
    let mut k: u_int = 0;
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut lines: [*const *const ::core::ffi::c_char; 3] =
        [::core::ptr::null::<*const ::core::ffi::c_char>(); 3];
    let mut count: [u_int; 3] = [0; 3];
    let mut n: u_int = 0 as u_int;
    let _target_owners_1 = window_tree_pull_item(item);
    sp = _target_owners_1.session.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    wl = if _target_owners_1.winlink.is_alive() { _target_owners_1.winlink.as_ptr().cast_mut() } else { std::ptr::null_mut() };
    wp = _target_owners_1.pane.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    if sp.is_null() || wp.is_null() {
        return;
    }
    if item.type_0 as ::core::ffi::c_uint
        == WINDOW_TREE_PANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        lines[n as usize] = &raw mut window_tree_pane_info_lines as *mut *const ::core::ffi::c_char;
        let fresh0 = n;
        n = n.wrapping_add(1);
        count[fresh0 as usize] = (::core::mem::size_of::<[*const ::core::ffi::c_char; 8]>()
            as usize)
            .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>() as usize)
            as u_int;
    }
    if item.type_0 as ::core::ffi::c_uint
        == WINDOW_TREE_PANE as ::core::ffi::c_int as ::core::ffi::c_uint
        || item.type_0 as ::core::ffi::c_uint
            == WINDOW_TREE_WINDOW as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        lines[n as usize] =
            &raw mut window_tree_window_info_lines as *mut *const ::core::ffi::c_char;
        let fresh1 = n;
        n = n.wrapping_add(1);
        count[fresh1 as usize] = (::core::mem::size_of::<[*const ::core::ffi::c_char; 6]>()
            as usize)
            .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>() as usize)
            as u_int;
    }
    lines[n as usize] = &raw mut window_tree_session_info_lines as *mut *const ::core::ffi::c_char;
    let fresh2 = n;
    n = n.wrapping_add(1);
    count[fresh2 as usize] = (::core::mem::size_of::<[*const ::core::ffi::c_char; 9]>() as usize)
        .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>() as usize)
        as u_int;
    ft = format_create(
        ::core::ptr::null_mut::<client>(),
        ::core::ptr::null_mut::<cmdq_item>(),
        FORMAT_NONE,
        0 as ::core::ffi::c_int,
    );
    format_defaults(ft, ::core::ptr::null_mut::<client>(), sp, wl, wp);
    i = 0 as u_int;
    j = 0 as u_int;
    while j < n {
        if j != 0 as u_int {
            if i == sy {
                break;
            }
            window_tree_border_cell(
                &raw mut gc,
                options_owner_ptr(&mut (*(*mode_pane).window).options).map_or(std::ptr::null_mut(), |options| options),
                ::core::ptr::null_mut::<format_tree>(),
            );
            screen_write_cursormove(
                &mut *ctx,
                cx as ::core::ffi::c_int,
                cy.wrapping_add(i) as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            screen_write_hline(
                &mut *ctx,
                sx,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                BOX_LINES_DEFAULT,
                Some(&gc),
            );
            if sx > 14 as u_int {
                gc.attr = (gc.attr as ::core::ffi::c_int | GRID_ATTR_CHARSET) as u_short;
                screen_write_cursormove(
                    &mut *ctx,
                    cx.wrapping_add(14 as u_int) as ::core::ffi::c_int,
                    cy.wrapping_add(i) as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
                screen_write_putc(&mut *ctx, &gc, 'n' as i32 as u_char);
            }
            i = i.wrapping_add(1);
        }
        k = 0 as u_int;
        while k < count[j as usize] {
            if i == sy {
                break;
            }
            let expanded = format_expand_cstring(ft, *lines[j as usize].offset(k as isize));
            screen_write_cursormove(
                &mut *ctx,
                cx as ::core::ffi::c_int,
                cy.wrapping_add(i) as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            format_draw(
                ctx,
                &raw const grid_default_cell,
                sx,
                expanded.as_ptr(),
                ::core::ptr::null_mut::<style_ranges>(),
                0 as ::core::ffi::c_int,
            );
            i = i.wrapping_add(1);
            k = k.wrapping_add(1);
        }
        if i == sy {
            break;
        }
        j = j.wrapping_add(1);
    }
    if sx > 14 as u_int && i < sy {
        window_tree_border_cell(
            &raw mut gc,
            options_owner_ptr(&mut (*(*mode_pane).window).options).map_or(std::ptr::null_mut(), |options| options),
            ::core::ptr::null_mut::<format_tree>(),
        );
        screen_write_cursormove(
            &mut *ctx,
            cx.wrapping_add(14 as u_int) as ::core::ffi::c_int,
            cy.wrapping_add(i) as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_vline(&mut *ctx, sy.wrapping_sub(i), Some(&gc));
    }
    format_free(ft);
}
unsafe fn window_tree_draw(
    mode_owner: &Rc<UnsafeCell<window_tree_modedata>>,
    item: &window_tree_itemdata,
    ctx: &mut screen_write_ctx,
    mut sx: u_int,
    mut sy: u_int,
) {
    let data = mode_owner.get();
    let Some(mode_pane_owner) = window_pane_upgrade(&(*data).wp) else {
        return;
    };
    let mode_pane = crate::src::shared::rc::as_ptr(&mode_pane_owner);
    let mut sp: *mut session = ::core::ptr::null_mut::<session>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let target_owners = window_tree_pull_item(item);
    sp = target_owners.session.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    wl = if target_owners.winlink.is_alive() { target_owners.winlink.as_ptr().cast_mut() } else { std::ptr::null_mut() };
    wp = target_owners.pane.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    if wp.is_null() {
        return;
    }
    if (*data).preview_is_info != 0 {
        window_tree_draw_info(mode_owner, item, ctx, sx, sy);
        return;
    }
    match item.type_0 as ::core::ffi::c_uint {
        1 => {
            window_tree_draw_session(mode_owner, target_owners.session.as_ref().expect("resolved preview session"), ctx, sx, sy);
        }
        2 => {
            window_tree_draw_window(mode_owner, target_owners.session.as_ref().expect("resolved preview session"), wl, ctx, sx, sy);
        }
        3 => {
            if (*data).hide_preview_this_pane == 0 || wp != mode_pane {
                screen_write_preview(&mut *ctx, &(*wp).base, sx, sy);
            }
        }
        0 | _ => {}
    };
}
unsafe fn window_tree_search(
    item: &window_tree_itemdata,
    search: &CStr,
    mut icase: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let ss = search.as_ptr();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut retval: ::core::ffi::c_int = 0;
    let _target_owners_3 = window_tree_pull_item(item);
    s = _target_owners_3.session.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    wl = if _target_owners_3.winlink.is_alive() { _target_owners_3.winlink.as_ptr().cast_mut() } else { std::ptr::null_mut() };
    wp = _target_owners_3.pane.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    match item.type_0 as ::core::ffi::c_uint {
        0 => return 0 as ::core::ffi::c_int,
        1 => {
            if s.is_null() {
                return 0 as ::core::ffi::c_int;
            }
            if icase != 0 {
                return (strcasestr(((*s).name).as_ptr().cast_mut(), ss)
                    != NULL as *mut ::core::ffi::c_char)
                    as ::core::ffi::c_int;
            }
            return (strstr(((*s).name).as_ptr().cast_mut(), ss) != NULL as *mut ::core::ffi::c_char)
                as ::core::ffi::c_int;
        }
        2 => {
            if s.is_null() || wl.is_null() {
                return 0 as ::core::ffi::c_int;
            }
            if icase != 0 {
                return (strcasestr((*(*wl).window_ptr()).name.as_ptr().cast_mut(), ss)
                    != NULL as *mut ::core::ffi::c_char)
                    as ::core::ffi::c_int;
            }
            return (strstr((*(*wl).window_ptr()).name.as_ptr().cast_mut(), ss)
                != NULL as *mut ::core::ffi::c_char) as ::core::ffi::c_int;
        }
        3 => {
            if !(s.is_null() || wl.is_null() || wp.is_null()) {
                let Some(cmd) = osdep_get_name_cstring((*wp).fd) else {
                    return 0 as ::core::ffi::c_int;
                };
                if icase != 0 {
                    retval = (strcasestr(cmd.as_ptr(), ss) != NULL as *mut ::core::ffi::c_char)
                        as ::core::ffi::c_int;
                } else {
                    retval = (strstr(cmd.as_ptr(), ss) != NULL as *mut ::core::ffi::c_char)
                        as ::core::ffi::c_int;
                }
                return retval;
            }
        }
        _ => {}
    }
    return 0 as ::core::ffi::c_int;
}
unsafe fn window_tree_menu(
    mode_owner: &Rc<UnsafeCell<window_tree_modedata>>,
    c: &std::rc::Rc<std::cell::UnsafeCell<client>>,
    mut key: key_code,
) {
    let data = mode_owner.get();
    let Some(mode_pane_owner) = window_pane_upgrade(&(*data).wp) else {
        return;
    };
    let mode_pane = crate::src::shared::rc::as_ptr(&mode_pane_owner);
    let mut wp: *mut window_pane = mode_pane;
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    wme = (*wp).modes.active;
    if wme.is_null() || (*wme).data != data.cast() {
        return;
    }
    window_tree_key(
        wme,
        c,
        ::core::ptr::null_mut::<winlink>(),
        key,
        ::core::ptr::null_mut::<mouse_event>(),
    );
}
unsafe fn window_tree_get_key(
    mode_owner: &Rc<UnsafeCell<window_tree_modedata>>,
    item: &window_tree_itemdata,
    mut line: u_int,
) -> key_code {
    let data = mode_owner.get();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut key: key_code = 0;
    ft = format_create(
        ::core::ptr::null_mut::<client>(),
        ::core::ptr::null_mut::<cmdq_item>(),
        FORMAT_NONE,
        0 as ::core::ffi::c_int,
    );
    let _target_owners_4 = window_tree_pull_item(item);
    s = _target_owners_4.session.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    wl = if _target_owners_4.winlink.is_alive() { _target_owners_4.winlink.as_ptr().cast_mut() } else { std::ptr::null_mut() };
    wp = _target_owners_4.pane.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    if item.type_0 as ::core::ffi::c_uint
        == WINDOW_TREE_SESSION as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        format_defaults(
            ft,
            ::core::ptr::null_mut::<client>(),
            s,
            ::core::ptr::null_mut::<winlink>(),
            ::core::ptr::null_mut::<window_pane>(),
        );
    } else if item.type_0 as ::core::ffi::c_uint
        == WINDOW_TREE_WINDOW as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        format_defaults(
            ft,
            ::core::ptr::null_mut::<client>(),
            s,
            wl,
            ::core::ptr::null_mut::<window_pane>(),
        );
    } else {
        format_defaults(ft, ::core::ptr::null_mut::<client>(), s, wl, wp);
    }
    format_add(
        ft,
        b"line\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write!(out, "{}", (line) as u32),
    );
    let expanded = format_expand_cstring(ft, (*data).key_format.as_ptr());
    key = key_string_parse_cstr(expanded.as_c_str()).unwrap_or(KEYC_UNKNOWN);
    format_free(ft);
    return key;
}
unsafe fn window_tree_swap(
    cur: &window_tree_itemdata,
    other: &window_tree_itemdata,
    sort_crit: &mut sort_criteria,
) -> ::core::ffi::c_int {
    let mut cur_session: *mut session = ::core::ptr::null_mut::<session>();
    let mut other_session: *mut session = ::core::ptr::null_mut::<session>();
    let mut cur_winlink: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut other_winlink: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut cur_window: *mut window = ::core::ptr::null_mut::<window>();
    let mut other_window: *mut window = ::core::ptr::null_mut::<window>();
    let mut cur_pane: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut other_pane: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    if (*cur).type_0 as ::core::ffi::c_uint != (*other).type_0 as ::core::ffi::c_uint {
        return 0 as ::core::ffi::c_int;
    }
    if (*cur).type_0 as ::core::ffi::c_uint
        != WINDOW_TREE_WINDOW as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return 0 as ::core::ffi::c_int;
    }
    let _target_owners_5 = window_tree_pull_item(cur);
    cur_session = _target_owners_5.session.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    cur_winlink = if _target_owners_5.winlink.is_alive() { _target_owners_5.winlink.as_ptr().cast_mut() } else { std::ptr::null_mut() };
    cur_pane = _target_owners_5.pane.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    let _target_owners_6 = window_tree_pull_item(other);
    other_session = _target_owners_6.session.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    other_winlink = if _target_owners_6.winlink.is_alive() { _target_owners_6.winlink.as_ptr().cast_mut() } else { std::ptr::null_mut() };
    other_pane = _target_owners_6.pane.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    if cur_session.is_null() || cur_winlink.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if other_session.is_null() || other_winlink.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if cur_session != other_session {
        return 0 as ::core::ffi::c_int;
    }
    if sort_would_window_tree_swap(sort_crit, cur_winlink, other_winlink) != 0 {
        return 0 as ::core::ffi::c_int;
    }
    other_window = (*other_winlink).window_ptr();
    cur_window = (*cur_winlink).window_ptr();
    window_winlinks_remove(other_window, other_winlink);
    window_winlinks_remove(cur_window, cur_winlink);
    if other_winlink != cur_winlink {
        std::mem::swap(&mut (*other_winlink).window_owner, &mut (*cur_winlink).window_owner);
    }
    window_winlinks_append(cur_window, other_winlink);
    window_winlinks_append(other_window, cur_winlink);
    if (*cur_session).curw == cur_winlink {
        session_set_current(cur_session, other_winlink);
    } else if (*cur_session).curw == other_winlink {
        session_set_current(cur_session, cur_winlink);
    }
    session_group_synchronize_from(cur_session);
    server_redraw_session_group(cur_session);
    recalculate_sizes();
    return 1 as ::core::ffi::c_int;
}
fn window_tree_sort(sort_crit: &mut sort_criteria) {
    unsafe {
        sort_crit.order_seq = &window_tree_order_seq;
        if sort_crit.order as ::core::ffi::c_uint
            == SORT_END as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            sort_crit.order = sort_crit.order_seq[0];
        }
    }
}
static window_tree_help_lines: &[&'static CStr] = &[
    c"#[fg=themelightgrey]      Enter #[#{E:tree-mode-border-style},acs]x#[default] Choose selected item",
    c"#[fg=themelightgrey]       S-Up #[#{E:tree-mode-border-style},acs]x#[default] Swap current and previous window",
    c"#[fg=themelightgrey]     S-Down #[#{E:tree-mode-border-style},acs]x#[default] Swap current and next window",
    c"#[fg=themelightgrey]          x #[#{E:tree-mode-border-style},acs]x#[default] Kill selected item",
    c"#[fg=themelightgrey]          X #[#{E:tree-mode-border-style},acs]x#[default] Kill tagged items",
    c"#[fg=themelightgrey]          < #[#{E:tree-mode-border-style},acs]x#[default] Scroll previews left",
    c"#[fg=themelightgrey]          > #[#{E:tree-mode-border-style},acs]x#[default] Scroll previews right",
    c"#[fg=themelightgrey]          m #[#{E:tree-mode-border-style},acs]x#[default] Set the marked pane",
    c"#[fg=themelightgrey]          M #[#{E:tree-mode-border-style},acs]x#[default] Clear the marked pane",
    c"#[fg=themelightgrey]          i #[#{E:tree-mode-border-style},acs]x#[default] Toggle session, window and pane information",
    c"#[fg=themelightgrey]          : #[#{E:tree-mode-border-style},acs]x#[default] Run a command for each tagged item",
    c"#[fg=themelightgrey]          f #[#{E:tree-mode-border-style},acs]x#[default] Enter a format",
    c"#[fg=themelightgrey]          H #[#{E:tree-mode-border-style},acs]x#[default] Jump to the starting pane",
];
fn window_tree_help() -> mode_tree_help_info {
    mode_tree_help_info {
        width: 51 as u_int,
        item: c"item",
        lines: window_tree_help_lines,
    }
}
unsafe fn window_tree_init(
    mut wme: *mut window_mode_entry,
    _item: *mut cmdq_item,
    mut fs: *mut cmd_find_state,
    mut args: *mut args,
) -> *mut screen {
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut wp: *mut window_pane = mode_pane;
    let mut data: *mut window_tree_modedata = ::core::ptr::null_mut::<window_tree_modedata>();
    let mut s: *mut screen = ::core::ptr::null_mut::<screen>();
    let format = if args.is_null() || args_has(args, 'F' as i32 as u_char) == 0 {
        WINDOW_TREE_DEFAULT_FORMAT.as_ptr()
    } else {
        args_get(&*(args), 'F' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr())
    };
    let key_format = if args.is_null() || args_has(args, 'K' as i32 as u_char) == 0 {
        WINDOW_TREE_DEFAULT_KEY_FORMAT.as_ptr()
    } else {
        args_get(&*(args), 'K' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr())
    };
    let command = if args.is_null() || args_count(args) == 0 as u_int {
        WINDOW_TREE_DEFAULT_COMMAND.as_ptr()
    } else {
        args_string(&mut *(args), 0 as u_int).map_or(std::ptr::null(), |value| value.as_ptr())
    };
    let owner = Rc::new_cyclic(|observer| UnsafeCell::new(window_tree_modedata {
        observer: observer.clone(),
        wp: Weak::new(),
        dead: 0,
        data: None,
        format: CStr::from_ptr(format).to_owned(),
        key_format: CStr::from_ptr(key_format).to_owned(),
        command: CStr::from_ptr(command).to_owned(),
        squash_groups: 0,
        hide_preview_this_pane: 0,
        preview_is_info: 0,
        prompt_flags: 0,
        item_list: Vec::new(),
        entered: None,
        fs: Default::default(),
        type_0: WINDOW_TREE_NONE,
        offset: 0,
        left: 0,
        right: 0,
        start: 0,
        end: 0,
        each: 0,
    }));
    data = crate::src::shared::rc::as_ptr(&owner);
    (*wme).data_owner = Some(owner);
    (*wme).data = data as *mut ::core::ffi::c_void;
    (*data).wp = std::rc::Rc::downgrade(&mode_pane_owner);
    if args_has(args, 's' as i32 as u_char) != 0 {
        (*data).type_0 = WINDOW_TREE_SESSION;
    } else if args_has(args, 'w' as i32 as u_char) != 0 {
        (*data).type_0 = WINDOW_TREE_WINDOW;
    } else {
        (*data).type_0 = WINDOW_TREE_PANE;
    }
    (*data).fs = (*fs).clone();
    (*data).squash_groups = (args_has(args, 'G' as i32 as u_char) == 0) as ::core::ffi::c_int;
    (*data).hide_preview_this_pane = args_has(args, 'h' as i32 as u_char);
    if args_has(args, 'y' as i32 as u_char) != 0 {
        (*data).prompt_flags = PROMPT_ACCEPT;
    }
    let build_mode = (*data).observer.clone();
    let draw_mode = build_mode.clone();
    let menu_mode = build_mode.clone();
    let key_mode = build_mode.clone();
    (*data).data = Some(mode_tree_start(
        &mode_pane_owner,
        args,
        Some(Box::new(move |sort, tag, filter| {
            let Some(mode) = window_tree_live_mode(&build_mode) else { return tag; };
            let mut selected = tag.unwrap_or(::core::primitive::u64::MAX as uint64_t);
            window_tree_build(
                &mode,
                sort as *mut sort_criteria,
                &mut selected,
                filter.map_or(::core::ptr::null(), |value| value.as_ptr()),
            );
            (selected != ::core::primitive::u64::MAX as uint64_t).then_some(selected)
        })),
        Some(Box::new(move |itemdata, ctx, sx, sy| {
            let Some(mode) = window_tree_live_mode(&draw_mode) else { return; };
            let item = itemdata.as_tree().expect("tree row payload");
            window_tree_draw(&mode, &item, ctx, sx, sy)
        })),
        Some(Box::new(move |itemdata, search, icase| {
            let item = itemdata.as_tree().expect("tree row payload");
            window_tree_search(&item, search, icase as ::core::ffi::c_int) != 0
        })),
        Some(Box::new(move |client, key| {
            let Some(mode) = window_tree_live_mode(&menu_mode) else { return; };
            window_tree_menu(&mode, client, key)
        })),
        None,
        Some(Box::new(move |itemdata, line| {
            let Some(mode) = window_tree_live_mode(&key_mode) else { return KEYC_NONE; };
            let item = itemdata.as_tree().expect("tree row payload");
            window_tree_get_key(&mode, &item, line)
        })),
        Some(Box::new(move |current, other, sort| {
            window_tree_swap(
                &current.as_tree().expect("tree row payload"),
                &other.as_tree().expect("tree row payload"),
                sort,
            ) != 0
        })),
        Some(window_tree_sort),
        Some(window_tree_help),
        &window_tree_menu_items,
        &raw mut s,
    ));
    mode_tree_zoom((*data).data.clone().as_ref().expect("mode tree owner"), args);
    mode_tree_view_name(&mut *(*data).data_ptr(), Some(c"preview"));
    mode_tree_build((*data).data.clone().as_ref().expect("mode tree owner"));
    mode_tree_draw((*data).data.clone().as_ref().expect("mode tree owner"));
    (*data).type_0 = WINDOW_TREE_NONE;
    return s;
}
unsafe fn window_tree_free(mut wme: *mut window_mode_entry) {
    let Some(mode_owner) = (*wme).retained_data::<UnsafeCell<window_tree_modedata>>() else {
        return;
    };
    let data = mode_owner.get();
    (*data).dead = 1 as ::core::ffi::c_int;
    mode_tree_free((*data).data.take().expect("mode tree owner"));
    drop((*wme).data_owner.take());
    (*wme).data = std::ptr::null_mut();
}
unsafe fn window_tree_resize(mut wme: *mut window_mode_entry, mut sx: u_int, mut sy: u_int) {
    let Some(mode_owner) = (*wme).retained_data::<UnsafeCell<window_tree_modedata>>() else {
        return;
    };
    let data = mode_owner.get();
    if (*data).dead != 0 { return; }
    mode_tree_resize((*data).data.clone().as_ref().expect("mode tree owner"), sx, sy);
}
unsafe fn window_tree_update(mut wme: *mut window_mode_entry) {
    let Some(mode_owner) = (*wme).retained_data::<UnsafeCell<window_tree_modedata>>() else {
        return;
    };
    let data = mode_owner.get();
    if (*data).dead != 0 { return; }
    let Some(mode_pane_owner) = window_pane_upgrade(&(*data).wp) else {
        return;
    };
    let mode_pane = crate::src::shared::rc::as_ptr(&mode_pane_owner);
    mode_tree_build((*data).data.clone().as_ref().expect("mode tree owner"));
    if (*data).dead != 0 { return; }
    mode_tree_draw((*data).data.clone().as_ref().expect("mode tree owner"));
    (*mode_pane).flags |= PANE_REDRAW;
}
unsafe fn window_tree_get_target(
    item: &window_tree_itemdata,
    fs: &mut cmd_find_state,
) -> Option<CString> {
    let resolved = window_tree_pull_item(item);
    let link_index = match resolved.winlink.try_borrow_mut() {
        Ok(link) => Some(link.idx),
        Err(refbox::BorrowError::Dropped) => None,
        Err(refbox::BorrowError::Borrowed) => panic!("tree target winlink already borrowed"),
    };
    let target = resolved.session.as_ref().and_then(|session_owner| {
        let mut bytes = Vec::from(b"=".as_slice());
        bytes.extend_from_slice((*session_owner.get()).name.as_bytes());
        bytes.push(b':');
        match item.type_0 {
            WINDOW_TREE_SESSION => {},
            WINDOW_TREE_WINDOW => {
                bytes.extend_from_slice(link_index?.to_string().as_bytes());
                bytes.push(b'.');
            }
            WINDOW_TREE_PANE => {
                let pane_owner = resolved.pane.as_ref()?;
                bytes.extend_from_slice(link_index?.to_string().as_bytes());
                bytes.extend_from_slice(b".%");
                bytes.extend_from_slice((*pane_owner.get()).id.to_string().as_bytes());
            }
            _ => return None,
        }
        Some(CString::new(bytes).expect("tree target contains no NUL"))
    });
    if target.is_none() {
        cmd_find_clear_state(fs, 0);
    } else {
        cmd_find_from_winlink_pane(
            fs,
            resolved.winlink.as_ptr().cast_mut(),
            resolved.pane.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get()),
            0,
        );
    }
    target
}
unsafe fn window_tree_command_each(
    mode_owner: &Rc<UnsafeCell<window_tree_modedata>>,
    item: &window_tree_itemdata,
    client_owner: Option<&Rc<UnsafeCell<client>>>,
) {
    let data = mode_owner.get();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: Default::default(),
        wl: Default::default(),
        w: Default::default(),
        wp: Default::default(),
        idx: 0,
    };
    if let Some(name) = window_tree_get_target(item, &mut fs) {
        mode_tree_run_command(
            client_owner,
            Some(&fs),
            (*data).entered.as_deref().expect("entered tree command"),
            &name,
        );
    }
}
fn window_tree_command_done(mode: Rc<UnsafeCell<window_tree_modedata>>) -> cmdq_cb {
    Some(Box::new(move |_| unsafe {
        let data = crate::src::shared::rc::as_ptr(&mode);
        if (*data).dead == 0 {
            let Some(mode_pane_owner) = window_pane_upgrade(&(*data).wp) else {
                return CMD_RETURN_NORMAL;
            };
            let mode_pane = crate::src::shared::rc::as_ptr(&mode_pane_owner);
            mode_tree_build((*data).data.clone().as_ref().expect("mode tree owner"));
            mode_tree_draw((*data).data.clone().as_ref().expect("mode tree owner"));
            (*mode_pane).flags |= PANE_REDRAW;
        }
        CMD_RETURN_NORMAL
    }))
}
unsafe fn window_tree_enqueue_command_done(client_owner: Option<&Rc<UnsafeCell<client>>>, mode: &Rc<UnsafeCell<window_tree_modedata>>) {
    let item = cmdq_get_callback_owned(
        c"window_tree_command_done".as_ptr(),
        window_tree_command_done(mode.clone()),
    );
    cmdq_append(client_owner, item);
}
fn window_tree_prompt_callbacks(
    mode: Rc<UnsafeCell<window_tree_modedata>>,
    callback: unsafe fn(Option<&Rc<UnsafeCell<client>>>, &Rc<UnsafeCell<window_tree_modedata>>, Option<&CStr>, prompt_key_result) -> prompt_result,
) -> (crate::src::shared::mode_tree::mode_tree_prompt_input_cb, crate::src::shared::prompt::prompt_free_cb) {
    let observer = Rc::downgrade(&mode);
    let input: crate::src::shared::mode_tree::mode_tree_prompt_input_cb = Some(Box::new(move |client, text, key| unsafe {
        let Some(owner) = window_tree_live_mode(&observer) else { return PROMPT_CLOSE; };
        callback(client, &owner, text, key)
    }));
    (input, Some(Box::new(move || drop(mode))))
}

unsafe fn window_tree_command_callback(
    client_owner: Option<&Rc<UnsafeCell<client>>>,
    mode_owner: &Rc<UnsafeCell<window_tree_modedata>>,
    s: Option<&CStr>,
    _key: prompt_key_result,
) -> prompt_result {
    let data = mode_owner.get();
    let Some(s) = s.filter(|text| !text.to_bytes().is_empty()) else {
        return PROMPT_CLOSE;
    };
    if (*data).dead != 0 {
        return PROMPT_CLOSE;
    }
    (*data).entered = Some(s.to_owned());
    mode_tree_each_tagged(
        (*data).data.clone().as_ref().expect("live mode tree"),
        |row, _| unsafe {
            let itemdata = row.borrow().itemdata.clone();
            window_tree_command_each(mode_owner, &itemdata.as_tree().expect("tree row payload"), client_owner)
        },
        KEYC_NONE as ::core::ffi::c_ulong as key_code,
        1 as ::core::ffi::c_int,
    );
    (*data).entered = None;
    window_tree_enqueue_command_done(client_owner, mode_owner);
    return PROMPT_CLOSE;
}
unsafe fn window_tree_kill_each(item: &window_tree_itemdata) {
    let target = window_tree_pull_item(item);
    match item.type_0 {
        WINDOW_TREE_SESSION => {
            if let Some(session_owner) = target.session.as_ref() {
                server_destroy_session(session_owner);
                session_destroy(session_owner.get(), 1, c"window_tree_kill_each".as_ptr());
            }
        }
        WINDOW_TREE_WINDOW => {
            // Release the winlink borrow before destruction can unlink it.
            let window_owner = match target.winlink.try_borrow_mut() {
                Ok(link) => link.window_owner.as_ref().map(|owner| {
                    crate::src::shared::window::WindowOwner::adopt(
                        owner.as_rc().clone(),
                    )
                }),
                Err(refbox::BorrowError::Dropped) => None,
                Err(refbox::BorrowError::Borrowed) => panic!("tree target winlink already borrowed"),
            };
            if let Some(window_owner) = window_owner {
                server_kill_window(window_owner.as_rc().clone(), 0);
            }
        }
        WINDOW_TREE_PANE => {
            if let Some(pane_owner) = target.pane.as_ref() {
                server_kill_pane(pane_owner);
            }
        }
        _ => {},
    }
}
unsafe fn window_tree_kill_current_callback(
    client_owner: Option<&Rc<UnsafeCell<client>>>,
    mode_owner: &Rc<UnsafeCell<window_tree_modedata>>,
    s: Option<&CStr>,
    _key: prompt_key_result,
) -> prompt_result {
    let data = mode_owner.get();
    let Some(s) = s.filter(|text| !text.to_bytes().is_empty()) else {
        return PROMPT_CLOSE;
    };
    if (*data).dead != 0 {
        return PROMPT_CLOSE;
    }
    let s = s.as_ptr();
    if ({
        let mut __res: ::core::ffi::c_int = 0;
        if ::core::mem::size_of::<u_char>() as usize > 1 as usize {
            if 0 != 0 {
                let mut __c: ::core::ffi::c_int =
                    *s.offset(0 as ::core::ffi::c_int as isize) as u_char as ::core::ffi::c_int;
                __res = (if __c < -(128 as ::core::ffi::c_int) || __c > 255 as ::core::ffi::c_int {
                    __c as __int32_t
                } else {
                    *(*__ctype_tolower_loc()).offset(__c as isize)
                }) as ::core::ffi::c_int;
            } else {
                __res = tolower(
                    *s.offset(0 as ::core::ffi::c_int as isize) as u_char as ::core::ffi::c_int
                );
            }
        } else {
            __res = *(*__ctype_tolower_loc()).offset(*s.offset(0 as ::core::ffi::c_int as isize)
                as u_char as ::core::ffi::c_int
                as isize) as ::core::ffi::c_int;
        }
        __res
    }) != 'y' as i32
        || *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\0' as i32
    {
        return PROMPT_CLOSE;
    }
    let tree_owner = (*data).data.as_ref().expect("mode tree owner");
    let item_owner = mode_tree_get_current(&*tree_owner.get());
    if let Some(item) = item_owner.as_tree() {
        window_tree_kill_each(&item);
    }
    server_renumber_all();
    window_tree_enqueue_command_done(client_owner, mode_owner);
    return PROMPT_CLOSE;
}
unsafe fn window_tree_kill_tagged_callback(
    client_owner: Option<&Rc<UnsafeCell<client>>>,
    mode_owner: &Rc<UnsafeCell<window_tree_modedata>>,
    s: Option<&CStr>,
    _key: prompt_key_result,
) -> prompt_result {
    let data = mode_owner.get();
    let Some(s) = s.filter(|text| !text.to_bytes().is_empty()) else {
        return PROMPT_CLOSE;
    };
    if (*data).dead != 0 {
        return PROMPT_CLOSE;
    }
    let s = s.as_ptr();
    if ({
        let mut __res: ::core::ffi::c_int = 0;
        if ::core::mem::size_of::<u_char>() as usize > 1 as usize {
            if 0 != 0 {
                let mut __c: ::core::ffi::c_int =
                    *s.offset(0 as ::core::ffi::c_int as isize) as u_char as ::core::ffi::c_int;
                __res = (if __c < -(128 as ::core::ffi::c_int) || __c > 255 as ::core::ffi::c_int {
                    __c as __int32_t
                } else {
                    *(*__ctype_tolower_loc()).offset(__c as isize)
                }) as ::core::ffi::c_int;
            } else {
                __res = tolower(
                    *s.offset(0 as ::core::ffi::c_int as isize) as u_char as ::core::ffi::c_int
                );
            }
        } else {
            __res = *(*__ctype_tolower_loc()).offset(*s.offset(0 as ::core::ffi::c_int as isize)
                as u_char as ::core::ffi::c_int
                as isize) as ::core::ffi::c_int;
        }
        __res
    }) != 'y' as i32
        || *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\0' as i32
    {
        return PROMPT_CLOSE;
    }
    mode_tree_each_tagged(
        (*data).data.clone().as_ref().expect("live mode tree"),
        |row, _| unsafe {
            let itemdata = row.borrow().itemdata.clone();
            window_tree_kill_each(&itemdata.as_tree().expect("tree row payload"))
        },
        KEYC_NONE as ::core::ffi::c_ulong as key_code,
        1 as ::core::ffi::c_int,
    );
    server_renumber_all();
    window_tree_enqueue_command_done(client_owner, mode_owner);
    return PROMPT_CLOSE;
}
unsafe fn window_tree_mouse(
    mode_owner: &Rc<UnsafeCell<window_tree_modedata>>,
    mut key: key_code,
    mut x: u_int,
    item: &window_tree_itemdata,
) -> key_code {
    let data = mode_owner.get();
    if key != KEYC_MOUSEDOWN1_PANE as ::core::ffi::c_ulong as key_code {
        return KEYC_NONE as ::core::ffi::c_ulong as key_code;
    }
    if (*data).left != -(1 as ::core::ffi::c_int) && x <= (*data).left as u_int {
        return '<' as i32 as key_code;
    }
    if (*data).right != -(1 as ::core::ffi::c_int) && x >= (*data).right as u_int {
        return '>' as i32 as key_code;
    }
    if (*data).left != -(1 as ::core::ffi::c_int) {
        x = x.wrapping_sub((*data).left as u_int);
    } else if x != 0 as u_int {
        x = x.wrapping_sub(1);
    }
    if x == 0 as u_int || (*data).end == 0 as u_int {
        x = 0 as u_int;
    } else {
        x = x.wrapping_div((*data).each);
        if (*data).start.wrapping_add(x) >= (*data).end {
            x = (*data).end.wrapping_sub(1 as u_int);
        }
    }
    let target = window_tree_pull_item(item);
    if item.type_0 as ::core::ffi::c_uint
        == WINDOW_TREE_SESSION as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let Some(session_owner) = target.session.as_ref() else {
            return KEYC_NONE;
        };
        mode_tree_expand_current((*data).data.clone().as_ref().expect("mode tree owner"));
        if (*data).dead != 0 {
            return KEYC_NONE;
        }
        let mut loop_0 = 0 as u_int;
        let mut wl = winlinks_minmax(&(*session_owner.get()).windows, RB_NEGINF);
        while !wl.is_null() {
            if loop_0 == (*data).start.wrapping_add(x) {
                break;
            }
            loop_0 = loop_0.wrapping_add(1);
            wl = winlinks_next(&*wl);
        }
        if !wl.is_null() {
            mode_tree_set_current(&mut *(*data).data_ptr(), wl as uint64_t);
        }
        return '\r' as i32 as key_code;
    }
    if item.type_0 as ::core::ffi::c_uint
        == WINDOW_TREE_WINDOW as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if !target.winlink.is_alive() {
            return KEYC_NONE;
        }
        mode_tree_expand_current((*data).data.clone().as_ref().expect("mode tree owner"));
        if (*data).dead != 0 || !target.winlink.is_alive() {
            return KEYC_NONE;
        }
        let wl = target.winlink.as_ptr();
        let mut pane = window_pane_first((*wl).window_ptr());
        for _ in 0..(*data).start.wrapping_add(x) {
            pane = pane.as_ref().and_then(|owner| window_pane_next(owner.get()));
            if pane.is_none() {
                break;
            }
        }
        if let Some(pane_owner) = pane {
            mode_tree_set_current(&mut *(*data).data_ptr(), pane_owner.get() as uint64_t);
        }
        return '\r' as i32 as key_code;
    }
    return KEYC_NONE as ::core::ffi::c_ulong as key_code;
}
unsafe fn window_tree_key(
    mut wme: *mut window_mode_entry,
    client_owner: &std::rc::Rc<std::cell::UnsafeCell<client>>,
    _wl: *mut winlink,
    mut key: key_code,
    mut m: *mut mouse_event,
) {
    let c = client_owner.get();
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut wp: *mut window_pane = mode_pane;
    let mode_owner = (*wme).retained_data::<UnsafeCell<window_tree_modedata>>()
        .expect("live mode payload");
    let data = mode_owner.get();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: Default::default(),
        wl: Default::default(),
        w: Default::default(),
        wp: Default::default(),
        idx: 0,
    };
    let mut fsp: *mut cmd_find_state = &raw mut (*data).fs;
    let mut finished: ::core::ffi::c_int = 0;
    let mut tagged: u_int = 0;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut idx: u_int = 0;
    let mut ns: *mut session = ::core::ptr::null_mut::<session>();
    let mut nwl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut nwp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut selection = mode_tree_get_current(&*(*data).data_ptr());
    finished = mode_tree_key((*data).data.as_ref().expect("mode tree owner").clone(), Some(client_owner), &raw mut key, m, &raw mut x, &raw mut y);
    if (*data).dead != 0 {
        return;
    }
    loop {
        let next_selection = mode_tree_get_current(&*(*data).data_ptr());
        let same_item = selection.same_identity(&next_selection);
        selection = next_selection;
        let item = selection.as_tree();
        if !same_item {
            (*data).offset = 0;
        }
        if !((key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
            == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
            || key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int
                && key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                    <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                        << 32 as ::core::ffi::c_int)
            && !m.is_null())
        {
            break;
        }
        key = item
            .as_deref()
            .map_or(KEYC_NONE, |item| window_tree_mouse(&mode_owner, key, x, item));
        if (*data).dead != 0 {
            return;
        }
    }
    let item = selection.as_tree();
    match key {
        60 => {
            (*data).offset -= 1;
        }
        62 => {
            (*data).offset += 1;
        }
        72 => {
            mode_tree_expand((*data).data.clone().as_ref().expect("mode tree owner"), (*fsp).s_ptr() as uint64_t);
            mode_tree_expand((*data).data.clone().as_ref().expect("mode tree owner"), (*fsp).wl_ptr() as uint64_t);
            if mode_tree_set_current(&mut *(*data).data_ptr(), mode_pane as uint64_t) == 0 {
                mode_tree_set_current(&mut *(*data).data_ptr(), (*fsp).wl_ptr() as uint64_t);
            }
        }
        109 if item.is_some() => {
            let item = item.as_deref().unwrap();
            let _target_owners_10 = window_tree_pull_item(item);
            ns = _target_owners_10.session.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
            nwl = if _target_owners_10.winlink.is_alive() { _target_owners_10.winlink.as_ptr().cast_mut() } else { std::ptr::null_mut() };
            nwp = _target_owners_10.pane.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
            server_set_marked(ns, nwl, nwp);
            mode_tree_build((*data).data.clone().as_ref().expect("mode tree owner"));
        }
        77 => {
            server_clear_marked();
            mode_tree_build((*data).data.clone().as_ref().expect("mode tree owner"));
        }
        105 => {
            (*data).preview_is_info = ((*data).preview_is_info == 0) as ::core::ffi::c_int;
            if (*data).preview_is_info != 0 {
                mode_tree_view_name(&mut *(*data).data_ptr(), Some(c"info"));
            } else {
                mode_tree_view_name(&mut *(*data).data_ptr(), Some(c"preview"));
            }
        }
        120 if item.is_some() => {
            let item = item.as_deref().unwrap();
            let _target_owners_11 = window_tree_pull_item(item);
            ns = _target_owners_11.session.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
            nwl = if _target_owners_11.winlink.is_alive() { _target_owners_11.winlink.as_ptr().cast_mut() } else { std::ptr::null_mut() };
            nwp = _target_owners_11.pane.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
            let prompt = match item.type_0 as ::core::ffi::c_uint {
                1 => {
                    if !ns.is_null() {
                        let mut bytes = b"Kill session ".to_vec();
                        bytes.extend_from_slice((*ns).name.as_bytes());
                        bytes.extend_from_slice(b"? ");
                        Some(CString::new(bytes).expect("session name contains no NUL"))
                    } else {
                        None
                    }
                }
                2 => {
                    if !nwl.is_null() {
                        Some(
                            CString::new(format!("Kill window {}? ", (*nwl).idx as u32))
                                .expect("window index contains no NUL"),
                        )
                    } else {
                        None
                    }
                }
                3 => {
                    if !(nwp.is_null()
                        || !window_pane_index(&*nwp).map(|value| { idx = value; }).is_some())
                    {
                        Some(CString::new(format!("Kill pane {idx}? ")).unwrap())
                    } else {
                        None
                    }
                }
                0 | _ => None,
            };
            if let Some(prompt) = prompt {
                let mode = mode_owner.clone();
                let (inputcb, freecb) = window_tree_prompt_callbacks(mode, window_tree_kill_current_callback);
                mode_tree_set_prompt(
                    (*data).data.as_ref().expect("mode tree owner").clone(),
                    Some(client_owner),
                    &prompt,
                    Some(c""),
                    PROMPT_TYPE_COMMAND,
                    PROMPT_SINGLE | PROMPT_NOFORMAT | (*data).prompt_flags,
                    inputcb,
                    freecb,
                );
            }
        }
        88 => {
            tagged = mode_tree_count_tagged(&*(*data).data_ptr());
            if !(tagged == 0 as u_int) {
                let prompt = CString::new(format!("Kill {tagged} tagged? ")).unwrap();
                let mode = mode_owner.clone();
                let (inputcb, freecb) = window_tree_prompt_callbacks(mode, window_tree_kill_tagged_callback);
                mode_tree_set_prompt(
                    (*data).data.as_ref().expect("mode tree owner").clone(),
                    Some(client_owner),
                    &prompt,
                    Some(c""),
                    PROMPT_TYPE_COMMAND,
                    PROMPT_SINGLE | PROMPT_NOFORMAT | (*data).prompt_flags,
                    inputcb,
                    freecb,
                );
            }
        }
        58 => {
            tagged = mode_tree_count_tagged(&*(*data).data_ptr());
            let prompt = if tagged != 0 as u_int {
                CString::new(format!("({tagged} tagged) ")).unwrap()
            } else {
                CString::new("(current) ").unwrap()
            };
            let mode = mode_owner.clone();
            let (inputcb, freecb) = window_tree_prompt_callbacks(mode, window_tree_command_callback);
            mode_tree_set_prompt(
                (*data).data.as_ref().expect("mode tree owner").clone(),
                Some(client_owner),
                &prompt,
                Some(c""),
                PROMPT_TYPE_COMMAND,
                PROMPT_NOFORMAT,
                inputcb,
                freecb,
            );
        }
        13 => {
            if let Some(name) = item
                .as_deref()
                .and_then(|item| window_tree_get_target(item, &mut fs))
            {
                mode_tree_run_command(Some(client_owner), None, &(*data).command, &name);
            }
            finished = 1 as ::core::ffi::c_int;
        }
        _ => {}
    }
    if finished != 0 {
        window_pane_reset_mode(&mode_pane_owner);
    } else {
        mode_tree_draw((*data).data.clone().as_ref().expect("mode tree owner"));
        (*wp).flags |= PANE_REDRAW;
    };
}

#[cfg(test)]
mod queued_refresh_tests {
    use super::*;
    use crate::src::cmd::queue::cmdq_free_detached;
    use crate::src::shared::rc;
    use std::ptr::NonNull;

    #[test]
    fn queued_refresh_releases_closed_or_orphaned_mode_when_fired_or_cancelled() {
        unsafe fn read_mode(
            _: Option<&Rc<UnsafeCell<client>>>,
            mode: &Rc<UnsafeCell<window_tree_modedata>>,
            _: Option<&CStr>,
            _: prompt_key_result,
        ) -> prompt_result {
            assert_eq!((*mode.get()).dead, 0);
            assert_eq!(Rc::strong_count(mode), 3);
            PROMPT_CONTINUE
        }
        for (fire, dead) in [(false, 0), (true, 0), (false, 1), (true, 1)] {
            unsafe {
                let mode = Rc::new_cyclic(|observer| UnsafeCell::new(window_tree_modedata {
                    observer: observer.clone(),
                    wp: Weak::new(),
                    dead,
                    data: None,
                    format: c"row format".to_owned(),
                    key_format: c"key format".to_owned(),
                    command: c"display-message".to_owned(),
                    squash_groups: 0,
                    hide_preview_this_pane: 0,
                    preview_is_info: 0,
                    prompt_flags: 0,
                    item_list: vec![refbox::RefBox::new(window_tree_itemdata {
                        type_0: WINDOW_TREE_NONE,
                        session: -1,
                        winlink: -1,
                        pane: -1,
                    })],
                    entered: Some(c"entered command".to_owned()),
                    fs: Default::default(),
                    type_0: WINDOW_TREE_NONE,
                    offset: 0,
                    left: 0,
                    right: 0,
                    start: 0,
                    end: 0,
                    each: 0,
                }));
                let observed = Rc::downgrade(&mode);
                let live = window_tree_live_mode(&observed);
                assert_eq!(live.is_some(), dead == 0);
                if let Some(live) = &live {
                    assert!(Rc::ptr_eq(live, &mode));
                    assert_eq!(Rc::strong_count(&mode), 2);
                }
                drop(live);
                assert_eq!(Rc::strong_count(&mode), 1);
                let (mut input, cleanup) = window_tree_prompt_callbacks(mode.clone(), read_mode);
                assert_eq!(Rc::strong_count(&mode), 2);
                assert_eq!(
                    input.as_mut().unwrap()(None, None, PROMPT_KEY_HANDLED),
                    if dead == 0 { PROMPT_CONTINUE } else { PROMPT_CLOSE },
                );
                cleanup.unwrap()();
                assert_eq!(Rc::strong_count(&mode), 1);
                let selected_observer = (&(*mode.get()).item_list)[0].downgrade();
                let selected = ModeTreeItemData::Tree(selected_observer.clone()).as_tree().unwrap();
                let item = cmdq_get_callback_owned(
                    c"test-tree-refresh".as_ptr(),
                    window_tree_command_done(mode),
                );
                assert!(observed.upgrade().is_some());
                if fire {
                    (*item).flags |= CMDQ_FIRED;
                    let callback = (*item).cb.take().unwrap();
                    // Closed modes and modes with expired parents must not
                    // access the absent pane or tree.
                    assert_eq!(callback(NonNull::new(item).unwrap()), CMD_RETURN_NORMAL);
                    assert!(observed.upgrade().is_none());
                }
                cmdq_free_detached(item);
                assert!(observed.upgrade().is_none());
                assert!(window_tree_live_mode(&observed).is_none());
                assert_eq!(input.as_mut().unwrap()(None, None, PROMPT_KEY_HANDLED), PROMPT_CLOSE);
                assert_eq!(selected.type_0, WINDOW_TREE_NONE);
                assert_eq!(
                    (selected.session, selected.winlink, selected.pane),
                    (-1, -1, -1)
                );
                assert!(!selected_observer.is_alive());
                drop(selected);
                assert!(!selected_observer.is_alive());
            }
        }
    }
}
