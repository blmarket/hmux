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
use crate::src::shared::mode_tree::{mode_tree_data, mode_tree_help_info, mode_tree_item};
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
use crate::src::window::{
    window_count_panes, window_has_pane, window_pane_find_by_id, window_pane_first,
    window_pane_index, window_pane_next, window_pane_reset_mode, window_winlinks_append,
    window_winlinks_remove, winlink_count, winlink_find_by_index, winlinks_minmax, winlinks_next,
};
use std::cell::UnsafeCell;
use std::ffi::{CStr, CString};
use std::rc::Rc;

#[repr(C)]
pub struct window_tree_modedata {
    pub wp: *mut window_pane,
    pub dead: ::core::ffi::c_int,
    pub data: *mut mode_tree_data,
    // Callback Rc references keep this record alive after mode shutdown.
    pub format: CString,
    pub key_format: CString,
    pub command: CString,
    pub squash_groups: ::core::ffi::c_int,
    pub hide_preview_this_pane: ::core::ffi::c_int,
    pub preview_is_info: ::core::ffi::c_int,
    pub prompt_flags: ::core::ffi::c_int,
    pub item_list: Vec<Box<window_tree_itemdata>>,
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
pub type window_tree_type = ::core::ffi::c_uint;
pub const WINDOW_TREE_PANE: window_tree_type = 3;
pub const WINDOW_TREE_WINDOW: window_tree_type = 2;
pub const WINDOW_TREE_SESSION: window_tree_type = 1;
pub const WINDOW_TREE_NONE: window_tree_type = 0;
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
pub const WINDOW_TREE_DEFAULT_FORMAT: [::core::ffi::c_char; 582] = unsafe {
    ::core::mem::transmute::<
        [u8; 582],
        [::core::ffi::c_char; 582],
    >(
        *b"#{?pane_format,#{?pane_marked,#[fg=thememagenta],}#{?pane_floating_flag,#[underscore],}#{pane_current_command}#[fg=themelightgrey]#{pane_flags}#{?#{&&:#{pane_title},#{!=:#{pane_title},#{host_short}}},: \"#{pane_title}\",},window_format,#{?window_marked_flag,#[fg=thememagenta],}#{window_name}#[fg=themelightgrey]#{window_flags}#{?#{&&:#{==:#{window_panes},1},#{&&:#{pane_title},#{!=:#{pane_title},#{host_short}}}},: \"#{pane_title}\",},#[fg=themelightgrey]#{session_windows} windows#{?session_grouped, (group #{session_group}: #{session_group_list}),}#{?session_attached, (attached),}}\0",
    )
};
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
pub static mut window_tree_mode: window_mode = {
    window_mode {
        name: c"tree-mode",
        default_format: WINDOW_TREE_DEFAULT_FORMAT.as_ptr(),
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
                    *mut client,
                    *mut session,
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
unsafe fn window_tree_pull_item(
    mut item: *mut window_tree_itemdata,
    mut sp: *mut *mut session,
    mut wlp: *mut *mut winlink,
    mut wp: *mut *mut window_pane,
) {
    *wp = ::core::ptr::null_mut::<window_pane>();
    *wlp = ::core::ptr::null_mut::<winlink>();
    *sp = session_find_by_id((*item).session as u_int);
    if (*sp).is_null() {
        return;
    }
    if (*item).type_0 as ::core::ffi::c_uint
        == WINDOW_TREE_SESSION as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        *wlp = (**sp).curw;
        *wp = (*(**wlp).window).active;
        return;
    }
    *wlp = winlink_find_by_index(&raw mut (**sp).windows, (*item).winlink);
    if (*wlp).is_null() {
        *sp = ::core::ptr::null_mut::<session>();
        return;
    }
    if (*item).type_0 as ::core::ffi::c_uint
        == WINDOW_TREE_WINDOW as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        *wp = (*(**wlp).window).active;
        return;
    }
    *wp = window_pane_find_by_id((*item).pane as u_int);
    if window_has_pane((**wlp).window, *wp) == 0 {
        *wp = ::core::ptr::null_mut::<window_pane>();
    }
    if (*wp).is_null() {
        *sp = ::core::ptr::null_mut::<session>();
        *wlp = ::core::ptr::null_mut::<winlink>();
        return;
    }
}
unsafe fn window_tree_add_item(mut data: *mut window_tree_modedata) -> *mut window_tree_itemdata {
    (*data).item_list.push(Box::new(window_tree_itemdata {
        type_0: 0,
        session: 0,
        winlink: 0,
        pane: 0,
    }));
    (*data).item_list.last_mut().unwrap().as_mut() as *mut window_tree_itemdata
}
unsafe fn window_tree_remove_last_item(
    data: *mut window_tree_modedata,
    item: *mut window_tree_itemdata,
    mti: *mut mode_tree_item,
) {
    debug_assert_eq!(
        (*data)
            .item_list
            .last()
            .map(|last| last.as_ref() as *const window_tree_itemdata),
        Some(item as *const window_tree_itemdata),
    );
    mode_tree_remove((*data).data, mti);
    (*data).item_list.pop();
}
unsafe fn window_tree_build_pane(
    mut s: *mut session,
    mut wl: *mut winlink,
    mut wp: *mut window_pane,
    mut modedata: *mut ::core::ffi::c_void,
    mut parent: *mut mode_tree_item,
) {
    let mut data: *mut window_tree_modedata = modedata as *mut window_tree_modedata;
    let mut item: *mut window_tree_itemdata = ::core::ptr::null_mut::<window_tree_itemdata>();
    let mut mti: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut idx: u_int = 0;
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    window_pane_index(wp, &raw mut idx);
    item = window_tree_add_item(data);
    (*item).type_0 = WINDOW_TREE_PANE;
    (*item).session = (*s).id as ::core::ffi::c_int;
    (*item).winlink = (*wl).idx;
    (*item).pane = (*wp).id as ::core::ffi::c_int;
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
    mti = mode_tree_add(
        (*data).data,
        parent,
        item as *mut ::core::ffi::c_void,
        wp as uint64_t,
        name.as_ptr(),
        text.as_ptr(),
        -(1 as ::core::ffi::c_int),
    ) as *mut mode_tree_item;
    mode_tree_align(mti);
}
unsafe fn window_tree_filter_pane(
    mut s: *mut session,
    mut wl: *mut winlink,
    mut wp: *mut window_pane,
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
        s,
        wl,
        wp,
    );
    result = format_true(cp.as_ptr());
    return result;
}
unsafe fn window_tree_build_window(
    mut s: *mut session,
    mut wl: *mut winlink,
    mut modedata: *mut ::core::ffi::c_void,
    mut sort_crit: *mut sort_criteria,
    mut parent: *mut mode_tree_item,
    mut filter: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut data: *mut window_tree_modedata = modedata as *mut window_tree_modedata;
    let mut item: *mut window_tree_itemdata = ::core::ptr::null_mut::<window_tree_itemdata>();
    let mut mti: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut i: u_int = 0;
    let mut found: u_int = 0;
    let mut expanded: ::core::ffi::c_int = 0;
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut tag: uint64_t = FORMAT_NONE as uint64_t;
    item = window_tree_add_item(data);
    (*item).type_0 = WINDOW_TREE_WINDOW;
    (*item).session = (*s).id as ::core::ffi::c_int;
    (*item).winlink = (*wl).idx;
    (*item).pane = -(1 as ::core::ffi::c_int);
    if !(*wl).window.is_null() && !(*(*wl).window).active.is_null() {
        tag = (FORMAT_PANE | (*(*(*wl).window).active).id) as uint64_t;
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
    mti = mode_tree_add(
        (*data).data,
        parent,
        item as *mut ::core::ffi::c_void,
        wl as uint64_t,
        name.as_ptr(),
        text.as_ptr(),
        expanded,
    ) as *mut mode_tree_item;
    mode_tree_align(mti);
    let l = sort_get_panes_window((*wl).window, sort_crit);
    let n = u_int::try_from(l.len()).expect("too many panes in window tree");
    found = 0 as u_int;
    i = 0 as u_int;
    while i < n {
        if !(window_tree_filter_pane(s, wl, l[i as usize], filter) == 0) {
            found = found.wrapping_add(1);
            if !((*data).hide_preview_this_pane != 0 && l[i as usize] == (*data).wp) {
                window_tree_build_pane(s, wl, l[i as usize], modedata, mti);
            }
        }
        i = i.wrapping_add(1);
    }
    if found == 0 as u_int {
        window_tree_remove_last_item(data, item, mti);
        return 0 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe fn window_tree_build_session(
    mut s: *mut session,
    mut modedata: *mut ::core::ffi::c_void,
    mut sort_crit: *mut sort_criteria,
    mut filter: *const ::core::ffi::c_char,
) {
    let mut data: *mut window_tree_modedata = modedata as *mut window_tree_modedata;
    let mut item: *mut window_tree_itemdata = ::core::ptr::null_mut::<window_tree_itemdata>();
    let mut mti: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut wl: *mut winlink = (*s).curw;
    let mut i: u_int = 0;
    let mut empty: u_int = 0;
    let mut expanded: ::core::ffi::c_int = 0;
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut tag: uint64_t = FORMAT_NONE as uint64_t;
    item = window_tree_add_item(data);
    (*item).type_0 = WINDOW_TREE_SESSION;
    (*item).session = (*s).id as ::core::ffi::c_int;
    (*item).winlink = -(1 as ::core::ffi::c_int);
    (*item).pane = -(1 as ::core::ffi::c_int);
    if !wl.is_null() && !(*wl).window.is_null() && !(*(*wl).window).active.is_null() {
        tag = (FORMAT_PANE | (*(*(*wl).window).active).id) as uint64_t;
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
    mti = mode_tree_add(
        (*data).data,
        ::core::ptr::null_mut::<mode_tree_item>(),
        item as *mut ::core::ffi::c_void,
        s as uint64_t,
        ((*s).name).as_ptr().cast_mut(),
        text.as_ptr(),
        expanded,
    ) as *mut mode_tree_item;
    let l = sort_get_winlinks_session(s, sort_crit);
    let n = u_int::try_from(l.len()).expect("too many winlinks in window tree");
    empty = 0 as u_int;
    i = 0 as u_int;
    while i < n {
        if window_tree_build_window(s, l[i as usize], modedata, sort_crit, mti, filter) == 0 {
            empty = empty.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    if empty == n {
        window_tree_remove_last_item(data, item, mti);
    }
}
unsafe fn window_tree_build(
    mut modedata: *mut ::core::ffi::c_void,
    mut sort_crit: *mut sort_criteria,
    tag: *mut uint64_t,
    mut filter: *const ::core::ffi::c_char,
) {
    let mut data: *mut window_tree_modedata = modedata as *mut window_tree_modedata;
    let mut squash_groups: ::core::ffi::c_int = (*data).squash_groups;
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    let mut current: *mut session_group = ::core::ptr::null_mut::<session_group>();
    let mut i: u_int = 0;
    current = session_group_contains((*data).fs.s);
    (*data).item_list.clear();
    let l = sort_get_sessions(sort_crit);
    let n = u_int::try_from(l.len()).expect("too many sessions for window tree");
    if n == 0 as u_int {
        return;
    }
    let mut current_block_12: u64;
    i = 0 as u_int;
    while i < n {
        s = l[i as usize];
        if squash_groups != 0 && {
            sg = session_group_contains(s);
            !sg.is_null()
        } {
            if sg == current && s != (*data).fs.s
                || sg != current
                    && Some(s)
                        != crate::src::session::session_group_members(sg)
                            .first()
                            .copied()
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
                window_tree_build_session(s, modedata, sort_crit, filter);
            }
            _ => {}
        }
        i = i.wrapping_add(1);
    }
    match (*data).type_0 as ::core::ffi::c_uint {
        1 => {
            if !(*data).fs.s.is_null() {
                *tag = (*data).fs.s as uint64_t;
            }
        }
        2 => {
            if !(*data).fs.s.is_null() && !(*data).fs.wl.is_null() {
                *tag = (*data).fs.wl as uint64_t;
            }
        }
        3 => {
            if window_count_panes((*(*data).fs.wl).window, 1 as ::core::ffi::c_int) == 1 as u_int {
                *tag = (*data).fs.wl as uint64_t;
            } else {
                *tag = (*data).fs.wp as uint64_t;
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
    mut data: *mut window_tree_modedata,
    mut s: *mut session,
    mut ctx: *mut screen_write_ctx,
    mut sx: u_int,
    mut sy: u_int,
) {
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
        (*(*(*data).wp).window).options,
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
            w = (*wl).window;
            oo = (*w).options;
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
    mut data: *mut window_tree_modedata,
    mut s: *mut session,
    mut wl: *mut winlink,
    mut ctx: *mut screen_write_ctx,
    mut sx: u_int,
    mut sy: u_int,
) {
    let mut w: *mut window = (*wl).window;
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
    total = window_count_panes(w, 1 as ::core::ffi::c_int);
    if (*data).hide_preview_this_pane != 0 && (*(*data).wp).window == w {
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
    wp = window_pane_first(w);
    while !wp.is_null() {
        if !((*data).hide_preview_this_pane != 0 && wp == (*data).wp) {
            if wp == (*w).active {
                break;
            }
            current = current.wrapping_add(1);
        }
        wp = window_pane_next(wp);
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
        (*(*(*data).wp).window).options,
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
    wp = window_pane_first(w);
    while !wp.is_null() {
        if !((*data).hide_preview_this_pane != 0 && wp == (*data).wp) {
            if loop_0 == end {
                break;
            }
            if loop_0 < start {
                loop_0 = loop_0.wrapping_add(1);
            } else {
                oo = (*wp).options;
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
        wp = window_pane_next(wp);
    }
}
unsafe fn window_tree_draw_info(
    mut data: *mut window_tree_modedata,
    mut itemdata: *mut ::core::ffi::c_void,
    mut ctx: *mut screen_write_ctx,
    mut sx: u_int,
    mut sy: u_int,
) {
    let mut item: *mut window_tree_itemdata = itemdata as *mut window_tree_itemdata;
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
    window_tree_pull_item(item, &raw mut sp, &raw mut wl, &raw mut wp);
    if sp.is_null() || wp.is_null() {
        return;
    }
    if (*item).type_0 as ::core::ffi::c_uint
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
    if (*item).type_0 as ::core::ffi::c_uint
        == WINDOW_TREE_PANE as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*item).type_0 as ::core::ffi::c_uint
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
                (*(*(*data).wp).window).options,
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
            (*(*(*data).wp).window).options,
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
    mut modedata: *mut ::core::ffi::c_void,
    mut itemdata: *mut ::core::ffi::c_void,
    mut ctx: *mut screen_write_ctx,
    mut sx: u_int,
    mut sy: u_int,
) {
    let mut data: *mut window_tree_modedata = modedata as *mut window_tree_modedata;
    let mut item: *mut window_tree_itemdata = itemdata as *mut window_tree_itemdata;
    let mut sp: *mut session = ::core::ptr::null_mut::<session>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    window_tree_pull_item(item, &raw mut sp, &raw mut wl, &raw mut wp);
    if wp.is_null() {
        return;
    }
    if (*data).preview_is_info != 0 {
        window_tree_draw_info(data, item as *mut ::core::ffi::c_void, ctx, sx, sy);
        return;
    }
    match (*item).type_0 as ::core::ffi::c_uint {
        1 => {
            window_tree_draw_session(modedata as *mut window_tree_modedata, sp, ctx, sx, sy);
        }
        2 => {
            window_tree_draw_window(modedata as *mut window_tree_modedata, sp, wl, ctx, sx, sy);
        }
        3 => {
            if (*data).hide_preview_this_pane == 0 || wp != (*data).wp {
                screen_write_preview(&mut *ctx, &(*wp).base, sx, sy);
            }
        }
        0 | _ => {}
    };
}
unsafe fn window_tree_search(
    mut itemdata: *mut ::core::ffi::c_void,
    mut ss: *const ::core::ffi::c_char,
    mut icase: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut item: *mut window_tree_itemdata = itemdata as *mut window_tree_itemdata;
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut retval: ::core::ffi::c_int = 0;
    window_tree_pull_item(item, &raw mut s, &raw mut wl, &raw mut wp);
    match (*item).type_0 as ::core::ffi::c_uint {
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
                return (strcasestr((*(*wl).window).name.as_ptr().cast_mut(), ss)
                    != NULL as *mut ::core::ffi::c_char)
                    as ::core::ffi::c_int;
            }
            return (strstr((*(*wl).window).name.as_ptr().cast_mut(), ss)
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
    mut modedata: *mut ::core::ffi::c_void,
    mut c: *mut client,
    mut key: key_code,
) {
    let mut data: *mut window_tree_modedata = modedata as *mut window_tree_modedata;
    let mut wp: *mut window_pane = (*data).wp;
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    wme = (*wp).modes.active;
    if wme.is_null() || (*wme).data != modedata {
        return;
    }
    window_tree_key(
        wme,
        c,
        ::core::ptr::null_mut::<session>(),
        ::core::ptr::null_mut::<winlink>(),
        key,
        ::core::ptr::null_mut::<mouse_event>(),
    );
}
unsafe fn window_tree_get_key(
    mut modedata: *mut ::core::ffi::c_void,
    mut itemdata: *mut ::core::ffi::c_void,
    mut line: u_int,
) -> key_code {
    let mut data: *mut window_tree_modedata = modedata as *mut window_tree_modedata;
    let mut item: *mut window_tree_itemdata = itemdata as *mut window_tree_itemdata;
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
    window_tree_pull_item(item, &raw mut s, &raw mut wl, &raw mut wp);
    if (*item).type_0 as ::core::ffi::c_uint
        == WINDOW_TREE_SESSION as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        format_defaults(
            ft,
            ::core::ptr::null_mut::<client>(),
            s,
            ::core::ptr::null_mut::<winlink>(),
            ::core::ptr::null_mut::<window_pane>(),
        );
    } else if (*item).type_0 as ::core::ffi::c_uint
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
    mut cur_itemdata: *mut ::core::ffi::c_void,
    mut other_itemdata: *mut ::core::ffi::c_void,
    mut sort_crit: *mut sort_criteria,
) -> ::core::ffi::c_int {
    let mut cur: *mut window_tree_itemdata = cur_itemdata as *mut window_tree_itemdata;
    let mut other: *mut window_tree_itemdata = other_itemdata as *mut window_tree_itemdata;
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
    window_tree_pull_item(
        cur,
        &raw mut cur_session,
        &raw mut cur_winlink,
        &raw mut cur_pane,
    );
    window_tree_pull_item(
        other,
        &raw mut other_session,
        &raw mut other_winlink,
        &raw mut other_pane,
    );
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
    other_window = (*other_winlink).window;
    cur_window = (*cur_winlink).window;
    window_winlinks_remove(other_window, other_winlink);
    window_winlinks_remove(cur_window, cur_winlink);
    (*other_winlink).window = cur_window;
    window_winlinks_append(cur_window, other_winlink);
    (*cur_winlink).window = other_window;
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
    let mut wp: *mut window_pane = (*wme).wp;
    let mut data: *mut window_tree_modedata = ::core::ptr::null_mut::<window_tree_modedata>();
    let mut s: *mut screen = ::core::ptr::null_mut::<screen>();
    let format = if args.is_null() || args_has(args, 'F' as i32 as u_char) == 0 {
        WINDOW_TREE_DEFAULT_FORMAT.as_ptr()
    } else {
        args_get(args, 'F' as i32 as u_char)
    };
    let key_format = if args.is_null() || args_has(args, 'K' as i32 as u_char) == 0 {
        WINDOW_TREE_DEFAULT_KEY_FORMAT.as_ptr()
    } else {
        args_get(args, 'K' as i32 as u_char)
    };
    let command = if args.is_null() || args_count(args) == 0 as u_int {
        WINDOW_TREE_DEFAULT_COMMAND.as_ptr()
    } else {
        args_string(args, 0 as u_int)
    };
    data = crate::src::shared::rc::new(window_tree_modedata {
        wp: ::core::ptr::null_mut(),
        dead: 0,
        data: ::core::ptr::null_mut(),
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
    });
    (*wme).data = data as *mut ::core::ffi::c_void;
    (*data).wp = wp;
    if args_has(args, 's' as i32 as u_char) != 0 {
        (*data).type_0 = WINDOW_TREE_SESSION;
    } else if args_has(args, 'w' as i32 as u_char) != 0 {
        (*data).type_0 = WINDOW_TREE_WINDOW;
    } else {
        (*data).type_0 = WINDOW_TREE_PANE;
    }
    memcpy(
        &raw mut (*data).fs as *mut ::core::ffi::c_void,
        fs as *const ::core::ffi::c_void,
        ::core::mem::size_of::<cmd_find_state>() as size_t,
    );
    (*data).squash_groups = (args_has(args, 'G' as i32 as u_char) == 0) as ::core::ffi::c_int;
    (*data).hide_preview_this_pane = args_has(args, 'h' as i32 as u_char);
    if args_has(args, 'y' as i32 as u_char) != 0 {
        (*data).prompt_flags = PROMPT_ACCEPT;
    }
    let data_handle = std::ptr::NonNull::new(data).expect("live tree mode data");
    (*data).data = mode_tree_start(
        wp,
        args,
        Some(Box::new(move |sort, tag, filter| {
            let mut selected = tag.unwrap_or(::core::primitive::u64::MAX as uint64_t);
            window_tree_build(
                data_handle.as_ptr().cast(),
                sort as *mut sort_criteria,
                &mut selected,
                filter.map_or(::core::ptr::null(), |value| value.as_ptr()),
            );
            (selected != ::core::primitive::u64::MAX as uint64_t).then_some(selected)
        })),
        Some(Box::new(move |itemdata, ctx, sx, sy| {
            window_tree_draw(
                data_handle.as_ptr().cast(),
                itemdata,
                ctx as *mut screen_write_ctx,
                sx,
                sy,
            )
        })),
        Some(Box::new(move |itemdata, search, icase| {
            window_tree_search(itemdata, search.as_ptr(), icase as ::core::ffi::c_int) != 0
        })),
        Some(Box::new(move |client, key| {
            window_tree_menu(
                data_handle.as_ptr().cast(),
                client.map_or(::core::ptr::null_mut(), std::ptr::NonNull::as_ptr),
                key,
            )
        })),
        None,
        Some(Box::new(move |itemdata, line| {
            window_tree_get_key(data_handle.as_ptr().cast(), itemdata, line)
        })),
        Some(Box::new(move |current, other, sort| {
            window_tree_swap(current, other, sort as *mut sort_criteria) != 0
        })),
        Some(window_tree_sort),
        Some(window_tree_help),
        &window_tree_menu_items,
        &raw mut s,
    );
    mode_tree_zoom((*data).data, args);
    mode_tree_view_name(
        (*data).data,
        b"preview\0" as *const u8 as *const ::core::ffi::c_char,
    );
    mode_tree_build((*data).data);
    mode_tree_draw((*data).data);
    (*data).type_0 = WINDOW_TREE_NONE;
    return s;
}
unsafe fn window_tree_destroy(mut data: *mut window_tree_modedata) {
    crate::src::shared::rc::release(data);
}
unsafe fn window_tree_free(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_tree_modedata = (*wme).data as *mut window_tree_modedata;
    if data.is_null() {
        return;
    }
    (*data).dead = 1 as ::core::ffi::c_int;
    mode_tree_free((*data).data);
    window_tree_destroy(data);
}
unsafe fn window_tree_resize(mut wme: *mut window_mode_entry, mut sx: u_int, mut sy: u_int) {
    let mut data: *mut window_tree_modedata = (*wme).data as *mut window_tree_modedata;
    mode_tree_resize((*data).data, sx, sy);
}
unsafe fn window_tree_update(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_tree_modedata = (*wme).data as *mut window_tree_modedata;
    mode_tree_build((*data).data);
    mode_tree_draw((*data).data);
    (*(*data).wp).flags |= PANE_REDRAW;
}
unsafe fn window_tree_get_target(
    mut item: *mut window_tree_itemdata,
    mut fs: *mut cmd_find_state,
) -> Option<CString> {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    window_tree_pull_item(item, &raw mut s, &raw mut wl, &raw mut wp);
    let target = match (*item).type_0 as ::core::ffi::c_uint {
        1 if !s.is_null() => {
            let mut bytes = Vec::from(b"=".as_slice());
            bytes.extend_from_slice((*s).name.as_bytes());
            bytes.push(b':');
            Some(CString::new(bytes).unwrap())
        }
        2 if !s.is_null() && !wl.is_null() => {
            let mut bytes = Vec::from(b"=".as_slice());
            bytes.extend_from_slice((*s).name.as_bytes());
            bytes.push(b':');
            bytes.extend_from_slice((*wl).idx.to_string().as_bytes());
            bytes.push(b'.');
            Some(CString::new(bytes).unwrap())
        }
        3 if !s.is_null() && !wl.is_null() && !wp.is_null() => {
            let mut bytes = Vec::from(b"=".as_slice());
            bytes.extend_from_slice((*s).name.as_bytes());
            bytes.push(b':');
            bytes.extend_from_slice((*wl).idx.to_string().as_bytes());
            bytes.extend_from_slice(b".%");
            bytes.extend_from_slice((*wp).id.to_string().as_bytes());
            Some(CString::new(bytes).unwrap())
        }
        _ => None,
    };
    if target.is_none() {
        cmd_find_clear_state(fs, 0 as ::core::ffi::c_int);
    } else {
        cmd_find_from_winlink_pane(fs, wl, wp, 0 as ::core::ffi::c_int);
    }
    target
}
unsafe fn window_tree_command_each(
    mut data: *mut window_tree_modedata,
    mut item: *mut window_tree_itemdata,
    mut c: *mut client,
) {
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    if let Some(name) = window_tree_get_target(item, &raw mut fs) {
        mode_tree_run_command(
            c,
            &raw mut fs,
            (*data)
                .entered
                .as_ref()
                .map_or(::core::ptr::null(), |entered| entered.as_ptr()),
            name.as_ptr(),
        );
    }
}
fn window_tree_command_done(mode: Rc<UnsafeCell<window_tree_modedata>>) -> cmdq_cb {
    Some(Box::new(move |_| unsafe {
        let data = crate::src::shared::rc::as_ptr(&mode);
        if (*data).dead == 0 {
            mode_tree_build((*data).data);
            mode_tree_draw((*data).data);
            (*(*data).wp).flags |= PANE_REDRAW;
        }
        CMD_RETURN_NORMAL
    }))
}
unsafe fn window_tree_enqueue_command_done(c: *mut client, data: *mut window_tree_modedata) {
    crate::src::shared::rc::retain(data);
    let mode = crate::src::shared::rc::take(data);
    let item = cmdq_get_callback_owned(
        c"window_tree_command_done".as_ptr(),
        window_tree_command_done(mode),
    );
    cmdq_append(c, item);
}
unsafe fn window_tree_command_callback(
    mut c: *mut client,
    mut data: *mut window_tree_modedata,
    s: Option<&CStr>,
    _key: prompt_key_result,
) -> prompt_result {
    let Some(s) = s.filter(|text| !text.to_bytes().is_empty()) else {
        return PROMPT_CLOSE;
    };
    if (*data).dead != 0 {
        return PROMPT_CLOSE;
    }
    (*data).entered = Some(s.to_owned());
    mode_tree_each_tagged(
        (*data).data,
        |row, c, _| unsafe { window_tree_command_each(data, (*row).itemdata.cast(), c) },
        c,
        KEYC_NONE as ::core::ffi::c_ulong as key_code,
        1 as ::core::ffi::c_int,
    );
    (*data).entered = None;
    window_tree_enqueue_command_done(c, data);
    return PROMPT_CLOSE;
}
unsafe fn window_tree_command_free(mut data: *mut window_tree_modedata) {
    window_tree_destroy(data);
}
unsafe fn window_tree_kill_each(mut item: *mut window_tree_itemdata) {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    window_tree_pull_item(item, &raw mut s, &raw mut wl, &raw mut wp);
    match (*item).type_0 as ::core::ffi::c_uint {
        1 => {
            if !s.is_null() {
                server_destroy_session(s);
                session_destroy(
                    s,
                    1 as ::core::ffi::c_int,
                    b"window_tree_kill_each\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
        }
        2 => {
            if !wl.is_null() {
                server_kill_window((*wl).window, 0 as ::core::ffi::c_int);
            }
        }
        3 => {
            if !wp.is_null() {
                server_kill_pane(wp);
            }
        }
        0 | _ => {}
    };
}
unsafe fn window_tree_kill_current_callback(
    mut c: *mut client,
    mut data: *mut window_tree_modedata,
    s: Option<&CStr>,
    _key: prompt_key_result,
) -> prompt_result {
    let mut mtd: *mut mode_tree_data = (*data).data;
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
    window_tree_kill_each(mode_tree_get_current(mtd) as *mut window_tree_itemdata);
    server_renumber_all();
    window_tree_enqueue_command_done(c, data);
    return PROMPT_CLOSE;
}
unsafe fn window_tree_kill_tagged_callback(
    mut c: *mut client,
    mut data: *mut window_tree_modedata,
    s: Option<&CStr>,
    _key: prompt_key_result,
) -> prompt_result {
    let mut mtd: *mut mode_tree_data = (*data).data;
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
        mtd,
        |row, _, _| unsafe { window_tree_kill_each((*row).itemdata.cast()) },
        c,
        KEYC_NONE as ::core::ffi::c_ulong as key_code,
        1 as ::core::ffi::c_int,
    );
    server_renumber_all();
    window_tree_enqueue_command_done(c, data);
    return PROMPT_CLOSE;
}
unsafe fn window_tree_mouse(
    mut data: *mut window_tree_modedata,
    mut key: key_code,
    mut x: u_int,
    mut item: *mut window_tree_itemdata,
) -> key_code {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut loop_0: u_int = 0;
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
    window_tree_pull_item(item, &raw mut s, &raw mut wl, &raw mut wp);
    if (*item).type_0 as ::core::ffi::c_uint
        == WINDOW_TREE_SESSION as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if s.is_null() {
            return KEYC_NONE as ::core::ffi::c_ulong as key_code;
        }
        mode_tree_expand_current((*data).data);
        loop_0 = 0 as u_int;
        wl = winlinks_minmax(&(*s).windows, RB_NEGINF);
        while !wl.is_null() {
            if loop_0 == (*data).start.wrapping_add(x) {
                break;
            }
            loop_0 = loop_0.wrapping_add(1);
            wl = winlinks_next(&*wl);
        }
        if !wl.is_null() {
            mode_tree_set_current((*data).data, wl as uint64_t);
        }
        return '\r' as i32 as key_code;
    }
    if (*item).type_0 as ::core::ffi::c_uint
        == WINDOW_TREE_WINDOW as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if wl.is_null() {
            return KEYC_NONE as ::core::ffi::c_ulong as key_code;
        }
        mode_tree_expand_current((*data).data);
        loop_0 = 0 as u_int;
        wp = window_pane_first((*wl).window);
        while !wp.is_null() {
            if loop_0 == (*data).start.wrapping_add(x) {
                break;
            }
            loop_0 = loop_0.wrapping_add(1);
            wp = window_pane_next(wp);
        }
        if !wp.is_null() {
            mode_tree_set_current((*data).data, wp as uint64_t);
        }
        return '\r' as i32 as key_code;
    }
    return KEYC_NONE as ::core::ffi::c_ulong as key_code;
}
unsafe fn window_tree_key(
    mut wme: *mut window_mode_entry,
    mut c: *mut client,
    _s: *mut session,
    _wl: *mut winlink,
    mut key: key_code,
    mut m: *mut mouse_event,
) {
    let mut wp: *mut window_pane = (*wme).wp;
    let mut data: *mut window_tree_modedata = (*wme).data as *mut window_tree_modedata;
    let mut item: *mut window_tree_itemdata = ::core::ptr::null_mut::<window_tree_itemdata>();
    let mut new_item: *mut window_tree_itemdata = ::core::ptr::null_mut::<window_tree_itemdata>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
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
    let mode = crate::src::shared::rc::downgrade(data);
    item = mode_tree_get_current((*data).data) as *mut window_tree_itemdata;
    finished = mode_tree_key((*data).data, c, &raw mut key, m, &raw mut x, &raw mut y);
    let Some(_mode_owner) = mode.upgrade() else {
        return;
    };
    if (*data).dead != 0 {
        return;
    }
    loop {
        new_item = mode_tree_get_current((*data).data) as *mut window_tree_itemdata;
        if item != new_item {
            item = new_item;
            (*data).offset = 0 as ::core::ffi::c_int;
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
        key = window_tree_mouse(data, key, x, item);
    }
    match key {
        60 => {
            (*data).offset -= 1;
        }
        62 => {
            (*data).offset += 1;
        }
        72 => {
            mode_tree_expand((*data).data, (*fsp).s as uint64_t);
            mode_tree_expand((*data).data, (*fsp).wl as uint64_t);
            if mode_tree_set_current((*data).data, (*wme).wp as uint64_t) == 0 {
                mode_tree_set_current((*data).data, (*fsp).wl as uint64_t);
            }
        }
        109 => {
            window_tree_pull_item(item, &raw mut ns, &raw mut nwl, &raw mut nwp);
            server_set_marked(ns, nwl, nwp);
            mode_tree_build((*data).data);
        }
        77 => {
            server_clear_marked();
            mode_tree_build((*data).data);
        }
        105 => {
            (*data).preview_is_info = ((*data).preview_is_info == 0) as ::core::ffi::c_int;
            if (*data).preview_is_info != 0 {
                mode_tree_view_name(
                    (*data).data,
                    b"info\0" as *const u8 as *const ::core::ffi::c_char,
                );
            } else {
                mode_tree_view_name(
                    (*data).data,
                    b"preview\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
        }
        120 => {
            window_tree_pull_item(item, &raw mut ns, &raw mut nwl, &raw mut nwp);
            let prompt = match (*item).type_0 as ::core::ffi::c_uint {
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
                        || window_pane_index(nwp, &raw mut idx) != 0 as ::core::ffi::c_int)
                    {
                        Some(CString::new(format!("Kill pane {idx}? ")).unwrap())
                    } else {
                        None
                    }
                }
                0 | _ => None,
            };
            if let Some(prompt) = prompt {
                crate::src::shared::rc::retain(data);
                mode_tree_set_prompt(
                    (*data).data,
                    c,
                    prompt.as_ptr(),
                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                    PROMPT_TYPE_COMMAND,
                    PROMPT_SINGLE | PROMPT_NOFORMAT | (*data).prompt_flags,
                    Some(Box::new(move |c, s, key| unsafe {
                        window_tree_kill_current_callback(
                            c.map_or(::core::ptr::null_mut(), std::ptr::NonNull::as_ptr),
                            data,
                            s,
                            key,
                        )
                    })),
                    Some(Box::new(move || unsafe { window_tree_command_free(data) })),
                );
            }
        }
        88 => {
            tagged = mode_tree_count_tagged((*data).data);
            if !(tagged == 0 as u_int) {
                let prompt = CString::new(format!("Kill {tagged} tagged? ")).unwrap();
                crate::src::shared::rc::retain(data);
                mode_tree_set_prompt(
                    (*data).data,
                    c,
                    prompt.as_ptr(),
                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                    PROMPT_TYPE_COMMAND,
                    PROMPT_SINGLE | PROMPT_NOFORMAT | (*data).prompt_flags,
                    Some(Box::new(move |c, s, key| unsafe {
                        window_tree_kill_tagged_callback(
                            c.map_or(::core::ptr::null_mut(), std::ptr::NonNull::as_ptr),
                            data,
                            s,
                            key,
                        )
                    })),
                    Some(Box::new(move || unsafe { window_tree_command_free(data) })),
                );
            }
        }
        58 => {
            tagged = mode_tree_count_tagged((*data).data);
            let prompt = if tagged != 0 as u_int {
                CString::new(format!("({tagged} tagged) ")).unwrap()
            } else {
                CString::new("(current) ").unwrap()
            };
            crate::src::shared::rc::retain(data);
            mode_tree_set_prompt(
                (*data).data,
                c,
                prompt.as_ptr(),
                b"\0" as *const u8 as *const ::core::ffi::c_char,
                PROMPT_TYPE_COMMAND,
                PROMPT_NOFORMAT,
                Some(Box::new(move |c, s, key| unsafe {
                    window_tree_command_callback(
                        c.map_or(::core::ptr::null_mut(), std::ptr::NonNull::as_ptr),
                        data,
                        s,
                        key,
                    )
                })),
                Some(Box::new(move || unsafe { window_tree_command_free(data) })),
            );
        }
        13 => {
            if let Some(name) = window_tree_get_target(item, &raw mut fs) {
                mode_tree_run_command(
                    c,
                    ::core::ptr::null_mut::<cmd_find_state>(),
                    (*data).command.as_ptr(),
                    name.as_ptr(),
                );
            }
            finished = 1 as ::core::ffi::c_int;
        }
        _ => {}
    }
    if finished != 0 {
        window_pane_reset_mode(wp);
    } else {
        mode_tree_draw((*data).data);
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
    fn queued_refresh_releases_closed_mode_when_fired_or_cancelled() {
        for fire in [false, true] {
            unsafe {
                let mode = rc::take(rc::new(window_tree_modedata {
                    wp: std::ptr::null_mut(),
                    dead: 1,
                    data: std::ptr::null_mut(),
                    format: c"row format".to_owned(),
                    key_format: c"key format".to_owned(),
                    command: c"display-message".to_owned(),
                    squash_groups: 0,
                    hide_preview_this_pane: 0,
                    preview_is_info: 0,
                    prompt_flags: 0,
                    item_list: vec![Box::new(window_tree_itemdata {
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
                let item = cmdq_get_callback_owned(
                    c"test-tree-refresh".as_ptr(),
                    window_tree_command_done(mode),
                );
                assert!(observed.upgrade().is_some());
                if fire {
                    (*item).flags |= CMDQ_FIRED;
                    let callback = (*item).cb.take().unwrap();
                    // A closed mode has already released its pane and tree.
                    assert_eq!(callback(NonNull::new(item).unwrap()), CMD_RETURN_NORMAL);
                    assert!(observed.upgrade().is_none());
                }
                cmdq_free_detached(item);
                assert!(observed.upgrade().is_none());
            }
        }
    }
}
