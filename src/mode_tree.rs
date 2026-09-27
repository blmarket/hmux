use crate::src::arguments::{args_get, args_has};
use crate::src::cmd::parse::{cmd_parse_and_append, cmd_parse_error_uppercase_first};
use crate::src::cmd::queue::{
    cmdq_append, cmdq_free_state, cmdq_get_callback_owned, cmdq_get_client, cmdq_new_state,
    cmdq_set_cancel_callback,
};
use crate::src::cmd::{cmd_mouse_at, cmd_template_replace_cstring};
use crate::src::ffi::libc::{
    __ctype_tolower_loc, __ctype_toupper_loc, memcpy, memset, strcasestr, strlen, strstr,
};
use crate::src::format::bytes::write_cstr;
use crate::src::format::{format_add, format_create_defaults, format_expand_cstring, format_free};
use crate::src::format_draw::{format_draw, format_width};
use crate::src::grid::grid_default_cell;
use crate::src::key_string::key_string_format;
use crate::src::log::{log_cstr, log_debug};
use crate::src::menu::{menu_add_items, menu_create, menu_display, menu_free};
use crate::src::options::options_get_number;
use crate::src::prompt::{
    prompt_closed, prompt_create, prompt_draw, prompt_free, prompt_key, prompt_mouse,
    prompt_set_options,
};
use crate::src::screen::{screen_free, screen_init, screen_resize};
use crate::src::screen_write::{
    screen_write_box, screen_write_clearcharacter, screen_write_clearendofline,
    screen_write_clearscreen, screen_write_cursormove, screen_write_puts, screen_write_start,
    screen_write_stop,
};
use crate::src::server_fn::{server_redraw_window, server_unzoom_window};
use crate::src::shared::abi::__int32_t;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::args;
use crate::src::shared::client::client;
use crate::src::shared::colour::{COLOUR_FLAG_THEME, COLOUR_THEME_CYAN};
use crate::src::shared::command::cmd_parse_input;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd_find_state, cmdq_item, cmdq_state};
use crate::src::shared::display::*;
use crate::src::shared::format::format_tree;
use crate::src::shared::grid::*;
use crate::src::shared::key::key_event;
use crate::src::shared::key::*;
use crate::src::shared::layout::*;
use crate::src::shared::menu::{menu, menu_item, MenuSelection};
use crate::src::shared::mode_tree::{
    mode_tree_build_cb, mode_tree_data, mode_tree_draw_cb, mode_tree_height_cb, mode_tree_help_cb,
    mode_tree_help_info, mode_tree_item, mode_tree_key_cb, mode_tree_line, mode_tree_list,
    mode_tree_menu_cb, mode_tree_prompt, mode_tree_prompt_input_cb, mode_tree_search_cb,
    mode_tree_search_dir, mode_tree_sort_cb, mode_tree_swap_cb,
};
use crate::src::shared::mouse::{mouse_event, MOUSE_BUTTON_1, MOUSE_MASK_BUTTONS, MOUSE_MASK_DRAG};
use crate::src::shared::options::options;
use crate::src::shared::pane::window_pane;
use crate::src::shared::pane::PANE_REDRAW;
use crate::src::shared::prompt::prompt;
use crate::src::shared::prompt::*;
use crate::src::shared::prompt::{prompt_create_data, prompt_draw_data};
use crate::src::shared::prompt::{
    prompt_free_cb, prompt_result, PROMPT_ACCEPT, PROMPT_CLOSE, PROMPT_CONTINUE, PROMPT_ISMODE,
    PROMPT_NOFORMAT, PROMPT_SINGLE,
};
use crate::src::shared::screen::{screen, MODE_CURSOR};
use crate::src::shared::screen_write::screen_write_ctx;
use crate::src::shared::session::session;
use crate::src::shared::sort::sort_criteria;
use crate::src::shared::style::*;
use crate::src::shared::window::WINDOW_ZOOMED;
use crate::src::shared::window::{window, winlink};
use crate::src::sort::{sort_next_order, sort_order_from_string, sort_order_to_string};
use crate::src::status::status_message_set;
use crate::src::style::style_apply;
use crate::src::tmux::global_s_options;
use crate::src::window::window_zoom;
use std::ffi::{CStr, CString};

pub const MODE_TREE_SEARCH_BACKWARD: mode_tree_search_dir = 1;
pub const MODE_TREE_SEARCH_FORWARD: mode_tree_search_dir = 0;

impl mode_tree_item {
    unsafe fn from_item(item: *mut mode_tree_item) -> *mut Self {
        item.cast()
    }

    fn set_keystr(&mut self, keystr: Option<CString>) {
        self.keystr = keystr;

        self.keylen = self.keystr.as_ref().map_or(0, |s| s.as_bytes().len());
    }
}
pub type mode_tree_preview = ::core::ffi::c_uint;
pub const MODE_TREE_PREVIEW_BIG: mode_tree_preview = 2;
pub const MODE_TREE_PREVIEW_NORMAL: mode_tree_preview = 1;
pub const MODE_TREE_PREVIEW_OFF: mode_tree_preview = 0;
#[inline]
unsafe fn tolower(mut __c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    return if __c >= -(128 as ::core::ffi::c_int) && __c < 256 as ::core::ffi::c_int {
        *(*__ctype_tolower_loc()).offset(__c as isize) as ::core::ffi::c_int
    } else {
        __c
    };
}
pub const UINT64_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
static mut mode_tree_menu_items: [menu_item; 5] = [
    menu_item {
        name: b"Scroll Left\0" as *const u8 as *const ::core::ffi::c_char,
        key: '<' as i32 as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"Scroll Right\0" as *const u8 as *const ::core::ffi::c_char,
        key: '>' as i32 as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_NONE as ::core::ffi::c_ulong as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"Cancel\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'q' as i32 as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        key: KEYC_NONE as ::core::ffi::c_ulong as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
];
static mode_tree_help_start: &[&'static CStr] = &[
    c"#[fg=themelightgrey]      Up, k #[#{E:tree-mode-border-style},acs]x#[default] Move cursor up",
    c"#[fg=themelightgrey]    Down, j #[#{E:tree-mode-border-style},acs]x#[default] Move cursor down",
    c"#[fg=themelightgrey]          g #[#{E:tree-mode-border-style},acs]x#[default] Go to top",
    c"#[fg=themelightgrey]          G #[#{E:tree-mode-border-style},acs]x#[default] Go to bottom",
    c"#[fg=themelightgrey] PPage, C-b #[#{E:tree-mode-border-style},acs]x#[default] Page up",
    c"#[fg=themelightgrey] NPage, C-f #[#{E:tree-mode-border-style},acs]x#[default] Page down",
    c"#[fg=themelightgrey]    Left, h #[#{E:tree-mode-border-style},acs]x#[default] Collapse %1",
    c"#[fg=themelightgrey]   Right, l #[#{E:tree-mode-border-style},acs]x#[default] Expand %1",
    c"#[fg=themelightgrey]        M-- #[#{E:tree-mode-border-style},acs]x#[default] Collapse all %1s",
    c"#[fg=themelightgrey]        M-+ #[#{E:tree-mode-border-style},acs]x#[default] Expand all %1s",
    c"#[fg=themelightgrey]          t #[#{E:tree-mode-border-style},acs]x#[default] Toggle %1 tag",
    c"#[fg=themelightgrey]          T #[#{E:tree-mode-border-style},acs]x#[default] Untag all %1s",
    c"#[fg=themelightgrey]        C-t #[#{E:tree-mode-border-style},acs]x#[default] Tag all %1s",
    c"#[fg=themelightgrey]        C-s #[#{E:tree-mode-border-style},acs]x#[default] Search forward",
    c"#[fg=themelightgrey]          n #[#{E:tree-mode-border-style},acs]x#[default] Repeat search forward",
    c"#[fg=themelightgrey]          N #[#{E:tree-mode-border-style},acs]x#[default] Repeat search backward",
    c"#[fg=themelightgrey]          f #[#{E:tree-mode-border-style},acs]x#[default] Filter %1s",
    c"#[fg=themelightgrey]          O #[#{E:tree-mode-border-style},acs]x#[default] Change sort order",
    c"#[fg=themelightgrey]          r #[#{E:tree-mode-border-style},acs]x#[default] Reverse sort order",
    c"#[fg=themelightgrey]          v #[#{E:tree-mode-border-style},acs]x#[default] Toggle preview",
];
static mode_tree_help_end: &[&'static CStr] =
    &[c"#[fg=themelightgrey]  q, Escape #[#{E:tree-mode-border-style},acs]x#[default] Exit mode"];
pub const MODE_TREE_HELP_DEFAULT_WIDTH: ::core::ffi::c_int = 39 as ::core::ffi::c_int;
unsafe fn mode_tree_is_lowercase(mut ptr: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    while *ptr as ::core::ffi::c_int != '\0' as i32 {
        if *ptr as ::core::ffi::c_int
            != ({
                let mut __res: ::core::ffi::c_int = 0;
                if ::core::mem::size_of::<u_char>() as usize > 1 as usize {
                    if 0 != 0 {
                        let mut __c: ::core::ffi::c_int = *ptr as u_char as ::core::ffi::c_int;
                        __res = (if __c < -(128 as ::core::ffi::c_int)
                            || __c > 255 as ::core::ffi::c_int
                        {
                            __c as __int32_t
                        } else {
                            *(*__ctype_tolower_loc()).offset(__c as isize)
                        }) as ::core::ffi::c_int;
                    } else {
                        __res = tolower(*ptr as u_char as ::core::ffi::c_int);
                    }
                } else {
                    __res = *(*__ctype_tolower_loc())
                        .offset(*ptr as u_char as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int;
                }
                __res
            })
        {
            return 0 as ::core::ffi::c_int;
        }
        ptr = ptr.offset(1);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe fn mode_tree_find_item(mut mtl: *mut mode_tree_list, tag: uint64_t) -> *mut mode_tree_item {
    let mut child: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    for mti in (*mtl).pointers() {
        if (*mti).tag == tag {
            return mti;
        }
        child = mode_tree_find_item(&raw mut (*mti).children, tag);
        if !child.is_null() {
            return child;
        }
    }
    return ::core::ptr::null_mut::<mode_tree_item>();
}
unsafe fn mode_tree_free_items(mtl: *mut mode_tree_list) {
    (*mtl).items.clear();
}

unsafe fn mode_tree_siblings(
    mtd: *mut mode_tree_data,
    mti: *mut mode_tree_item,
) -> *mut mode_tree_list {
    let parent = (*mti).parent;
    if parent.is_null() {
        &raw mut (*mtd).children
    } else {
        &raw mut (*parent).children
    }
}

unsafe fn mode_tree_check_selected(mut mtd: *mut mode_tree_data) {
    if (*mtd).current > (*mtd).height.wrapping_sub(1 as u_int) {
        (*mtd).offset = (*mtd)
            .current
            .wrapping_sub((*mtd).height)
            .wrapping_add(1 as u_int);
    }
}
unsafe fn mode_tree_alloc_data() -> *mut mode_tree_data {
    crate::src::shared::rc::new(mode_tree_data::default())
}

#[inline]
unsafe fn mode_tree_line_count(mtd: &mode_tree_data) -> u_int {
    mtd.lines.len() as u_int
}

unsafe fn mode_tree_clear_lines(mut mtd: *mut mode_tree_data) {
    (*mtd).lines = Vec::new();
}
unsafe fn mode_tree_build_lines(
    mut mtd: *mut mode_tree_data,
    mut mtl: *mut mode_tree_list,
    mut depth: u_int,
) {
    let mut line: *mut mode_tree_line = ::core::ptr::null_mut::<mode_tree_line>();
    let mut i: u_int = 0;
    let mut flat: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    (*mtd).depth = depth;
    if depth > (*mtd).maxdepth {
        (*mtd).maxdepth = depth;
    }
    for mti in (*mtl).pointers() {
        (*mtd).lines.push(mode_tree_line {
            item: mti,
            depth,
            last: (mti == (*mtl).last()) as ::core::ffi::c_int,
            flat: 0,
        });
        (*mti).line = mode_tree_line_count(&*mtd).wrapping_sub(1 as u_int);
        if !(*mti).children.first().is_null() {
            flat = 0 as ::core::ffi::c_int;
        }
        if (*mti).expanded != 0 {
            mode_tree_build_lines(
                mtd,
                &raw mut (*mti).children,
                depth.wrapping_add(1 as u_int),
            );
        }
        if (*mtd).keycb.is_some() {
            (*mti).key =
                (*mtd).keycb.as_mut().expect("non-null key callback")((*mti).itemdata, (*mti).line);
            if (*mti).key == KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code {
                (*mti).key = KEYC_NONE as ::core::ffi::c_ulong as key_code;
            }
        } else if (*mti).line < 10 as u_int {
            (*mti).key = ('0' as i32 as u_int).wrapping_add((*mti).line) as key_code;
        } else if (*mti).line < 36 as u_int {
            (*mti).key = (KEYC_META
                | ('a' as i32 as u_int)
                    .wrapping_add((*mti).line)
                    .wrapping_sub(10 as u_int) as ::core::ffi::c_ulonglong)
                as key_code;
        } else {
            (*mti).key = KEYC_NONE as ::core::ffi::c_ulong as key_code;
        }
        let keystr = ((*mti).key != KEYC_NONE as ::core::ffi::c_ulong as key_code)
            .then(|| key_string_format((*mti).key, false));
        (*mode_tree_item::from_item(mti)).set_keystr(keystr);
    }
    for mti in (*mtl).pointers() {
        i = 0 as u_int;
        while i < mode_tree_line_count(&*mtd) {
            line = (*mtd).lines.as_mut_ptr().offset(i as isize) as *mut mode_tree_line
                as *mut mode_tree_line;
            if (*line).item == mti {
                (*line).flat = flat;
            }
            i = i.wrapping_add(1);
        }
    }
}
unsafe fn mode_tree_clear_tagged(mut mtl: *mut mode_tree_list) {
    for mti in (*mtl).pointers() {
        (*mti).tagged = 0 as ::core::ffi::c_int;
        mode_tree_clear_tagged(&raw mut (*mti).children);
    }
}
pub unsafe fn mode_tree_up(mut mtd: *mut mode_tree_data, mut wrap: ::core::ffi::c_int) {
    if mode_tree_line_count(&*mtd) == 0 as u_int {
        return;
    }
    if (*mtd).current == 0 as u_int {
        if wrap != 0 {
            (*mtd).current = mode_tree_line_count(&*mtd).wrapping_sub(1 as u_int);
            if mode_tree_line_count(&*mtd) >= (*mtd).height {
                (*mtd).offset = mode_tree_line_count(&*mtd).wrapping_sub((*mtd).height);
            }
        }
    } else {
        (*mtd).current = (*mtd).current.wrapping_sub(1);
        if (*mtd).current < (*mtd).offset {
            (*mtd).offset = (*mtd).offset.wrapping_sub(1);
        }
    };
}
pub unsafe fn mode_tree_down(
    mut mtd: *mut mode_tree_data,
    mut wrap: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if mode_tree_line_count(&*mtd) == 0 as u_int {
        return 0 as ::core::ffi::c_int;
    }
    if (*mtd).current == mode_tree_line_count(&*mtd).wrapping_sub(1 as u_int) {
        if wrap != 0 {
            (*mtd).current = 0 as u_int;
            (*mtd).offset = 0 as u_int;
        } else {
            return 0 as ::core::ffi::c_int;
        }
    } else {
        (*mtd).current = (*mtd).current.wrapping_add(1);
        if (*mtd).current
            > (*mtd)
                .offset
                .wrapping_add((*mtd).height)
                .wrapping_sub(1 as u_int)
        {
            (*mtd).offset = (*mtd).offset.wrapping_add(1);
        }
    }
    return 1 as ::core::ffi::c_int;
}
unsafe fn mode_tree_swap(mut mtd: *mut mode_tree_data, mut direction: ::core::ffi::c_int) {
    let mut current_depth: u_int =
        (*(*mtd).lines.as_mut_ptr().offset((*mtd).current as isize)).depth;
    let mut swap_with: u_int = 0;
    let mut swap_with_depth: u_int = 0;
    if (*mtd).swapcb.is_none() {
        return;
    }
    swap_with = (*mtd).current;
    loop {
        if direction < 0 as ::core::ffi::c_int && swap_with < -direction as u_int {
            return;
        }
        if direction > 0 as ::core::ffi::c_int
            && swap_with.wrapping_add(direction as u_int) >= mode_tree_line_count(&*mtd)
        {
            return;
        }
        swap_with = swap_with.wrapping_add(direction as u_int);
        swap_with_depth = (*(*mtd).lines.as_mut_ptr().offset(swap_with as isize)).depth;
        if !(swap_with_depth > current_depth) {
            break;
        }
    }
    if swap_with_depth != current_depth {
        return;
    }
    let current = (*(*mtd).lines.as_mut_ptr().offset((*mtd).current as isize)).item;
    let other = (*(*mtd).lines.as_mut_ptr().offset(swap_with as isize)).item;
    if (*mtd).swapcb.as_mut().expect("non-null swap callback")(
        (*current).itemdata,
        (*other).itemdata,
        &mut (*mtd).sort_crit,
    ) {
        (*mtd).current = swap_with;
        mode_tree_build(mtd);
    }
}
pub unsafe fn mode_tree_get_current(mut mtd: *mut mode_tree_data) -> *mut ::core::ffi::c_void {
    if mode_tree_line_count(&*mtd) == 0 as u_int {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    return (*(*(*mtd).lines.as_mut_ptr().offset((*mtd).current as isize)).item).itemdata;
}
pub unsafe fn mode_tree_get_current_name(
    mut mtd: *mut mode_tree_data,
) -> *const ::core::ffi::c_char {
    return ((*(*(*mtd).lines.as_mut_ptr().offset((*mtd).current as isize)).item).name)
        .as_ptr()
        .cast_mut();
}
pub unsafe fn mode_tree_expand_current(mut mtd: *mut mode_tree_data) {
    if (*(*(*mtd).lines.as_mut_ptr().offset((*mtd).current as isize)).item).expanded == 0 {
        (*(*(*mtd).lines.as_mut_ptr().offset((*mtd).current as isize)).item).expanded =
            1 as ::core::ffi::c_int;
        mode_tree_build(mtd);
    }
}
unsafe fn mode_tree_get_tag(
    mut mtd: *mut mode_tree_data,
    tag: uint64_t,
    mut found: *mut u_int,
) -> ::core::ffi::c_int {
    let mut i: u_int = 0;
    i = 0 as u_int;
    while i < mode_tree_line_count(&*mtd) {
        if (*(*(*mtd).lines.as_mut_ptr().offset(i as isize)).item).tag == tag {
            break;
        }
        i = i.wrapping_add(1);
    }
    if i != mode_tree_line_count(&*mtd) {
        *found = i;
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn mode_tree_expand(mut mtd: *mut mode_tree_data, mut tag: uint64_t) {
    let mut found: u_int = 0;
    if mode_tree_get_tag(mtd, tag, &raw mut found) == 0 {
        return;
    }
    if (*(*(*mtd).lines.as_mut_ptr().offset(found as isize)).item).expanded == 0 {
        (*(*(*mtd).lines.as_mut_ptr().offset(found as isize)).item).expanded =
            1 as ::core::ffi::c_int;
        mode_tree_build(mtd);
    }
}
pub unsafe fn mode_tree_set_current(
    mut mtd: *mut mode_tree_data,
    mut tag: uint64_t,
) -> ::core::ffi::c_int {
    let mut found: u_int = 0;
    if mode_tree_get_tag(mtd, tag, &raw mut found) != 0 {
        (*mtd).current = found;
        if (*mtd).current > (*mtd).height.wrapping_sub(1 as u_int) {
            (*mtd).offset = (*mtd)
                .current
                .wrapping_sub((*mtd).height)
                .wrapping_add(1 as u_int);
        } else {
            (*mtd).offset = 0 as u_int;
        }
        return 1 as ::core::ffi::c_int;
    }
    if (*mtd).current >= mode_tree_line_count(&*mtd) {
        if mode_tree_line_count(&*mtd) == 0 as u_int {
            return 0 as ::core::ffi::c_int;
        }
        (*mtd).current = mode_tree_line_count(&*mtd).wrapping_sub(1 as u_int);
        if (*mtd).current > (*mtd).height.wrapping_sub(1 as u_int) {
            (*mtd).offset = (*mtd)
                .current
                .wrapping_sub((*mtd).height)
                .wrapping_add(1 as u_int);
        } else {
            (*mtd).offset = 0 as u_int;
        }
    }
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn mode_tree_count_tagged(mut mtd: *mut mode_tree_data) -> u_int {
    let mut mti: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut i: u_int = 0;
    let mut tagged: u_int = 0;
    tagged = 0 as u_int;
    i = 0 as u_int;
    while i < mode_tree_line_count(&*mtd) {
        mti = (*(*mtd).lines.as_mut_ptr().offset(i as isize)).item;
        if (*mti).tagged != 0 {
            tagged = tagged.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    return tagged;
}
pub unsafe fn mode_tree_each_tagged(
    mut mtd: *mut mode_tree_data,
    mut cb: impl FnMut(*mut mode_tree_item, *mut client, key_code),
    mut c: *mut client,
    mut key: key_code,
    mut current: ::core::ffi::c_int,
) {
    let mut mti: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut i: u_int = 0;
    let mut fired: ::core::ffi::c_int = 0;
    fired = 0 as ::core::ffi::c_int;
    i = 0 as u_int;
    while i < mode_tree_line_count(&*mtd) {
        mti = (*(*mtd).lines.as_mut_ptr().offset(i as isize)).item;
        if (*mti).tagged != 0 {
            fired = 1 as ::core::ffi::c_int;
            cb(mti, c, key);
        }
        i = i.wrapping_add(1);
    }
    if fired == 0 && current != 0 {
        mti = (*(*mtd).lines.as_mut_ptr().offset((*mtd).current as isize)).item;
        cb(mti, c, key);
    }
}
pub unsafe fn mode_tree_start(
    wp: *mut window_pane,
    args: *mut args,
    buildcb: mode_tree_build_cb,
    drawcb: mode_tree_draw_cb,
    searchcb: mode_tree_search_cb,
    menucb: mode_tree_menu_cb,
    heightcb: mode_tree_height_cb,
    keycb: mode_tree_key_cb,
    swapcb: mode_tree_swap_cb,
    sortcb: mode_tree_sort_cb,
    helpcb: mode_tree_help_cb,
    menu: *const menu_item,
    s: *mut *mut screen,
) -> *mut mode_tree_data {
    let mut mtd: *mut mode_tree_data = ::core::ptr::null_mut::<mode_tree_data>();
    mtd = mode_tree_alloc_data();
    (*mtd).wp = wp;
    (*mtd).menu = menu;
    if drawcb.is_none() {
        (*mtd).preview = MODE_TREE_PREVIEW_OFF as ::core::ffi::c_int;
    } else if args_has(args, 'N' as i32 as u_char) > 1 as ::core::ffi::c_int {
        (*mtd).preview = MODE_TREE_PREVIEW_BIG as ::core::ffi::c_int;
    } else if args_has(args, 'N' as i32 as u_char) != 0 {
        (*mtd).preview = MODE_TREE_PREVIEW_OFF as ::core::ffi::c_int;
    } else {
        (*mtd).preview = MODE_TREE_PREVIEW_NORMAL as ::core::ffi::c_int;
    }
    (*mtd).sort_crit.order = sort_order_from_string(args_get(args, 'O' as i32 as u_char));
    (*mtd).sort_crit.reversed = args_has(args, 'r' as i32 as u_char);
    if args_has(args, 'f' as i32 as u_char) != 0 {
        (*mtd).filter = Some(CStr::from_ptr(args_get(args, 'f' as i32 as u_char)).to_owned());
    } else {
        (*mtd).filter = None;
    }
    (*mtd).buildcb = buildcb;
    (*mtd).drawcb = drawcb;
    (*mtd).searchcb = searchcb;
    (*mtd).menucb = menucb;
    (*mtd).heightcb = heightcb;
    (*mtd).keycb = keycb;
    (*mtd).swapcb = swapcb;
    (*mtd).sortcb = sortcb;
    (*mtd).helpcb = helpcb;
    *s = &raw mut (*mtd).screen;
    screen_init(*s, (*wp).base.grid().sx, (*wp).base.grid().sy, 0 as u_int);
    (**s).mode &= !MODE_CURSOR;
    return mtd;
}
pub unsafe fn mode_tree_zoom(mut mtd: *mut mode_tree_data, mut args: *mut args) {
    let mut wp: *mut window_pane = (*mtd).wp;
    if args_has(args, 'Z' as i32 as u_char) != 0 {
        (*mtd).zoomed = (*(*wp).window).flags & WINDOW_ZOOMED;
        if (*mtd).zoomed == 0 && window_zoom(wp) == 0 as ::core::ffi::c_int {
            server_redraw_window((*wp).window as *mut window);
        }
    } else {
        (*mtd).zoomed = -(1 as ::core::ffi::c_int);
    };
}
unsafe fn mode_tree_set_height(mut mtd: *mut mode_tree_data) {
    let mut s: *mut screen = &raw mut (*mtd).screen;
    let mut height: u_int = 0;
    if (*mtd).heightcb.is_some() {
        height = (*mtd).heightcb.as_mut().expect("non-null height callback")((*s).grid().sy);
        if height < (*s).grid().sy {
            (*mtd).height = (*s).grid().sy.wrapping_sub(height);
        }
    } else if (*mtd).preview == MODE_TREE_PREVIEW_NORMAL as ::core::ffi::c_int {
        (*mtd).height = (*s).grid()
            .sy
            .wrapping_div(3 as u_int)
            .wrapping_mul(2 as u_int);
        if (*mtd).height > mode_tree_line_count(&*mtd) {
            (*mtd).height = (*s).grid().sy.wrapping_div(2 as u_int);
        }
        if (*mtd).height < 10 as u_int {
            (*mtd).height = (*s).grid().sy;
        }
    } else if (*mtd).preview == MODE_TREE_PREVIEW_BIG as ::core::ffi::c_int {
        (*mtd).height = (*s).grid().sy.wrapping_div(4 as u_int);
        if (*mtd).height > mode_tree_line_count(&*mtd) {
            (*mtd).height = mode_tree_line_count(&*mtd);
        }
        if (*mtd).height < 2 as u_int {
            (*mtd).height = 2 as u_int;
        }
    } else {
        (*mtd).height = (*s).grid().sy;
    }
    if (*s).grid().sy.wrapping_sub((*mtd).height) < 2 as u_int {
        (*mtd).height = (*s).grid().sy;
    }
}
pub unsafe fn mode_tree_build(mut mtd: *mut mode_tree_data) {
    let mut s: *mut screen = &raw mut (*mtd).screen;
    let mut tag: Option<uint64_t>;
    if !(*mtd).lines.is_empty() {
        tag = Some((*(*(*mtd).lines.as_mut_ptr().offset((*mtd).current as isize)).item).tag);
    } else {
        tag = None;
    }
    (*mtd).saved.items.append(&mut (*mtd).children.items);
    if (*mtd).sortcb.is_some() {
        (*mtd).sortcb.expect("non-null sort callback")(&mut (*mtd).sort_crit);
    }
    tag = (*mtd).buildcb.as_mut().expect("non-null build callback")(
        &mut (*mtd).sort_crit,
        tag,
        (*mtd).filter.as_deref(),
    );
    (*mtd).no_matches = ((*mtd).children.first()
        == ::core::ptr::null_mut::<::core::ffi::c_void>() as *mut mode_tree_item)
        as ::core::ffi::c_int;
    if (*mtd).no_matches != 0 {
        tag = (*mtd).buildcb.as_mut().expect("non-null build callback")(
            &mut (*mtd).sort_crit,
            tag,
            None,
        );
    }
    mode_tree_free_items(&raw mut (*mtd).saved);
    mode_tree_clear_lines(mtd);
    (*mtd).maxdepth = 0 as u_int;
    mode_tree_build_lines(mtd, &raw mut (*mtd).children, 0 as u_int);
    if !(*mtd).lines.is_empty() && tag.is_none() {
        tag = Some((*(*(*mtd).lines.as_mut_ptr().offset((*mtd).current as isize)).item).tag);
    }
    mode_tree_set_current(mtd, tag.unwrap_or(UINT64_MAX as uint64_t));
    (*mtd).width = (*s).grid().sx;
    if (*mtd).preview != MODE_TREE_PREVIEW_OFF as ::core::ffi::c_int {
        mode_tree_set_height(mtd);
    } else {
        (*mtd).height = (*s).grid().sy;
    }
    mode_tree_check_selected(mtd);
}

unsafe fn mode_tree_remove_ref(mut mtd: *mut mode_tree_data) {
    crate::src::shared::rc::release(mtd);
}
pub unsafe fn mode_tree_free(mut mtd: *mut mode_tree_data) {
    let mut wp: *mut window_pane = (*mtd).wp;
    if (*mtd).zoomed == 0 as ::core::ffi::c_int {
        server_unzoom_window((*wp).window as *mut window);
    }
    mode_tree_clear_prompt(mtd);
    mode_tree_free_items(&raw mut (*mtd).children);
    mode_tree_clear_lines(mtd);
    screen_free(&raw mut (*mtd).screen);
    (*mtd).search = None;
    (*mtd).filter = None;
    (*mtd).dead = 1 as ::core::ffi::c_int;
    mode_tree_remove_ref(mtd);
}
pub unsafe fn mode_tree_resize(mut mtd: *mut mode_tree_data, mut sx: u_int, mut sy: u_int) {
    let mut s: *mut screen = &raw mut (*mtd).screen;
    screen_resize(s, sx, sy, 0 as ::core::ffi::c_int);
    mode_tree_build(mtd);
    mode_tree_draw(mtd);
    (*(*mtd).wp).flags |= PANE_REDRAW;
}
pub unsafe fn mode_tree_add(
    mut mtd: *mut mode_tree_data,
    mut parent: *mut mode_tree_item,
    mut itemdata: *mut ::core::ffi::c_void,
    mut tag: uint64_t,
    mut name: *const ::core::ffi::c_char,
    mut text: *const ::core::ffi::c_char,
    mut expanded: ::core::ffi::c_int,
) -> *mut mode_tree_item {
    let mut mti: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut saved: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    log_debug(format_args!(
        "{}: {}, {} {}",
        "mode_tree_add",
        tag as ::core::ffi::c_ulonglong,
        log_cstr((name) as *const _),
        log_cstr(
            (if text.is_null() {
                b"\0" as *const u8 as *const ::core::ffi::c_char
            } else {
                text
            }) as *const _
        )
    ));
    let name = CStr::from_ptr(name).to_owned();
    let text = (!text.is_null()).then(|| CStr::from_ptr(text).to_owned());
    let mut owner = Box::new(mode_tree_item {
        name: name,
        text: text,
        keystr: None,
        ..mode_tree_item::empty()
    });
    owner.tag = tag;

    mti = &mut *owner;
    (*mti).parent = parent;
    (*mti).itemdata = itemdata;
    saved = mode_tree_find_item(&raw mut (*mtd).saved, tag);
    if !saved.is_null() {
        if parent.is_null() || (*parent).expanded != 0 {
            (*mti).tagged = (*saved).tagged;
        }
        (*mti).expanded = (*saved).expanded;
    } else if expanded == -(1 as ::core::ffi::c_int) {
        (*mti).expanded = 1 as ::core::ffi::c_int;
    } else {
        (*mti).expanded = expanded;
    }
    (*mode_tree_siblings(mtd, mti)).items.push(owner);
    return mti as *mut mode_tree_item;
}
pub unsafe fn mode_tree_view_name(
    mut mtd: *mut mode_tree_data,
    mut name: *const ::core::ffi::c_char,
) {
    (*mtd).view_name = if name.is_null() {
        None
    } else {
        Some(CStr::from_ptr(name))
    };
}
pub unsafe fn mode_tree_draw_as_parent(mut mti: *mut mode_tree_item) {
    (*mti).draw_as_parent = 1 as ::core::ffi::c_int;
}
pub unsafe fn mode_tree_no_tag(mut mti: *mut mode_tree_item) {
    (*mti).no_tag = 1 as ::core::ffi::c_int;
}
pub unsafe fn mode_tree_align(mut mti: *mut mode_tree_item) {
    (*mti).align = 1;
}
pub unsafe fn mode_tree_remove(mut mtd: *mut mode_tree_data, mut mti: *mut mode_tree_item) {
    let siblings = &mut *mode_tree_siblings(mtd, mti);
    let position = siblings.position(mti);
    siblings.items.remove(position);
}
fn mode_tree_append_printf_string(bytes: &mut Vec<u8>, value: Option<&CStr>) {
    if let Some(value) = value {
        bytes.extend_from_slice(value.to_bytes());
    } else {
        bytes.extend_from_slice(b"(null)");
    }
}
pub unsafe fn mode_tree_draw(mut mtd: *mut mode_tree_data) {
    let mut wp: *mut window_pane = (*mtd).wp;
    let mut s: *mut screen = &raw mut (*mtd).screen;
    let mut line: *mut mode_tree_line = ::core::ptr::null_mut::<mode_tree_line>();
    let mut mti: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut oo: *mut options = (*(*wp).window).options;
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: ::core::ptr::null_mut::<window_pane>(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        item: None,
        scrolled: 0,
        bg: 0,
    };
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut gc0: grid_cell = grid_cell {
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
    let mut box_gc: grid_cell = grid_cell {
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
    let mut w: u_int = 0;
    let mut h: u_int = 0;
    let mut i: u_int = 0;
    let mut sy: u_int = 0;
    let mut box_x: u_int = 0;
    let mut box_y: u_int = 0;
    let mut width: u_int = 0;
    let mut text_width: u_int = 0;
    let mut prefix_width: u_int = 0;
    let mut left: u_int = 0;
    let mut tag: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut separator: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut n: size_t = 0;
    let mut keylen: ::core::ffi::c_int = 0;
    let vla = (*mtd).maxdepth.wrapping_add(1 as u_int) as usize;
    let mut alignlen: Vec<::core::ffi::c_int> = ::std::vec::from_elem(0, vla);
    let mut dfg: ::core::ffi::c_int = 0;
    let mut dfg0: ::core::ffi::c_int = 0;
    if mode_tree_line_count(&*mtd) == 0 as u_int {
        return;
    }
    w = (*mtd).width;
    h = (*mtd).height;
    if w == 0 as u_int || h == 0 as u_int {
        return;
    }
    memcpy(
        &raw mut gc0 as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    memcpy(
        &raw mut gc as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    style_apply(
        &raw mut gc,
        oo,
        b"tree-mode-selection-style\0" as *const u8 as *const ::core::ffi::c_char,
        ::core::ptr::null_mut::<format_tree>(),
    );
    memcpy(
        &raw mut box_gc as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    style_apply(
        &raw mut box_gc,
        oo,
        b"tree-mode-border-style\0" as *const u8 as *const ::core::ffi::c_char,
        ::core::ptr::null_mut::<format_tree>(),
    );
    dfg = gc.fg;
    dfg0 = gc0.fg;
    screen_write_start(&raw mut ctx, s);
    screen_write_clearscreen(&raw mut ctx, 8 as u_int);
    ft = format_create_defaults(
        ::core::ptr::null_mut::<cmdq_item>(),
        ::core::ptr::null_mut::<client>(),
        ::core::ptr::null_mut::<session>(),
        ::core::ptr::null_mut::<winlink>(),
        wp,
    );
    keylen = 0 as ::core::ffi::c_int;
    i = 0 as u_int;
    while i < mode_tree_line_count(&*mtd) {
        mti = (*(*mtd).lines.as_mut_ptr().offset(i as isize)).item;
        if !((*mti).key == KEYC_NONE as ::core::ffi::c_ulong as key_code) {
            if (*mti).keylen as ::core::ffi::c_int + 3 as ::core::ffi::c_int > keylen {
                keylen = (*mti).keylen.wrapping_add(3 as size_t) as ::core::ffi::c_int;
            }
        }
        i = i.wrapping_add(1);
    }
    i = 0 as u_int;
    while i < (*mtd).maxdepth.wrapping_add(1 as u_int) {
        *alignlen.as_mut_ptr().offset(i as isize) = 0 as ::core::ffi::c_int;
        i = i.wrapping_add(1);
    }
    i = 0 as u_int;
    while i < mode_tree_line_count(&*mtd) {
        line = (*mtd).lines.as_mut_ptr().offset(i as isize) as *mut mode_tree_line
            as *mut mode_tree_line;
        mti = (*line).item;
        if (*mti).align != 0
            && strlen(((*mti).name).as_ptr().cast_mut()) as ::core::ffi::c_int
                > *alignlen.as_mut_ptr().offset((*line).depth as isize)
        {
            *alignlen.as_mut_ptr().offset((*line).depth as isize) =
                strlen(((*mti).name).as_ptr().cast_mut()) as ::core::ffi::c_int;
        }
        i = i.wrapping_add(1);
    }
    i = 0 as u_int;
    while i < mode_tree_line_count(&*mtd) {
        if !(i < (*mtd).offset) {
            if i > (*mtd).offset.wrapping_add(h).wrapping_sub(1 as u_int) {
                break;
            }
            line = (*mtd).lines.as_mut_ptr().offset(i as isize) as *mut mode_tree_line
                as *mut mode_tree_line;
            mti = (*line).item;
            screen_write_cursormove(
                &raw mut ctx,
                0 as ::core::ffi::c_int,
                i.wrapping_sub((*mtd).offset) as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            if (*mti).key != KEYC_NONE as ::core::ffi::c_ulong as key_code {
                format_add(
                    ft,
                    b"mode_tree_key\0" as *const u8 as *const ::core::ffi::c_char,
                    |out| {
                        write_cstr(
                            out,
                            ((*mti).keystr)
                                .as_ref()
                                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                        )
                    },
                );
            } else {
                format_add(
                    ft,
                    b"mode_tree_key\0" as *const u8 as *const ::core::ffi::c_char,
                    |out| write_cstr(out, b"\0" as *const u8 as *const ::core::ffi::c_char),
                );
            }
            format_add(
                ft,
                b"mode_tree_key_width\0" as *const u8 as *const ::core::ffi::c_char,
                |out| write!(out, "{}", (keylen) as i32),
            );
            format_add(
                ft,
                b"mode_tree_selected\0" as *const u8 as *const ::core::ffi::c_char,
                |out| {
                    write!(
                        out,
                        "{}",
                        ((i == (*mtd).current) as ::core::ffi::c_int) as i32
                    )
                },
            );
            if (*line).depth == 0 as u_int {
                format_add(
                    ft,
                    b"mode_tree_repeat\0" as *const u8 as *const ::core::ffi::c_char,
                    |out| write!(out, "{}", (0 as ::core::ffi::c_int) as u32),
                );
                format_add(
                    ft,
                    b"mode_tree_branch\0" as *const u8 as *const ::core::ffi::c_char,
                    |out| out.write_all(b"0"),
                );
                format_add(
                    ft,
                    b"mode_tree_parent_last\0" as *const u8 as *const ::core::ffi::c_char,
                    |out| out.write_all(b"0"),
                );
            } else {
                format_add(
                    ft,
                    b"mode_tree_repeat\0" as *const u8 as *const ::core::ffi::c_char,
                    |out| write!(out, "{}", ((*line).depth.wrapping_sub(1 as u_int)) as u32),
                );
                format_add(
                    ft,
                    b"mode_tree_branch\0" as *const u8 as *const ::core::ffi::c_char,
                    |out| out.write_all(b"1"),
                );
                if !(*mti).parent.is_null()
                    && (*(*mtd)
                        .lines
                        .as_mut_ptr()
                        .offset((*(*mti).parent).line as isize))
                    .last
                        != 0
                {
                    format_add(
                        ft,
                        b"mode_tree_parent_last\0" as *const u8 as *const ::core::ffi::c_char,
                        |out| out.write_all(b"1"),
                    );
                } else {
                    format_add(
                        ft,
                        b"mode_tree_parent_last\0" as *const u8 as *const ::core::ffi::c_char,
                        |out| out.write_all(b"0"),
                    );
                }
            }
            if (*mti).children.first().is_null() {
                format_add(
                    ft,
                    b"mode_tree_has_children\0" as *const u8 as *const ::core::ffi::c_char,
                    |out| out.write_all(b"0"),
                );
            } else {
                format_add(
                    ft,
                    b"mode_tree_has_children\0" as *const u8 as *const ::core::ffi::c_char,
                    |out| out.write_all(b"1"),
                );
            }
            format_add(
                ft,
                b"mode_tree_last\0" as *const u8 as *const ::core::ffi::c_char,
                |out| write!(out, "{}", ((*line).last) as i32),
            );
            format_add(
                ft,
                b"mode_tree_expanded\0" as *const u8 as *const ::core::ffi::c_char,
                |out| write!(out, "{}", ((*mti).expanded) as i32),
            );
            format_add(
                ft,
                b"mode_tree_flat\0" as *const u8 as *const ::core::ffi::c_char,
                |out| write!(out, "{}", ((*line).flat) as i32),
            );
            let prefix = format_expand_cstring(
                ft,
                b"#[fg=themelightgrey]#[bg=default]#[noacs]#{p/#{mode_tree_key_width}:#{?#{!=:#{mode_tree_key},},(#{mode_tree_key}),}}#{R:#{?mode_tree_parent_last,    ,#[acs]x#[fg=themelightgrey]#[bg=default]#[noacs]   },#{mode_tree_repeat}}#{?mode_tree_branch,#[acs]#{?mode_tree_last,mq,tq}+#[fg=themelightgrey]#[bg=default]#[noacs] ,}#{?mode_tree_has_children,#{?mode_tree_expanded,#[fg=themered]-#[fg=themelightgrey]#[bg=default]#[noacs] ,#[fg=themegreen]+#[fg=themelightgrey]#[bg=default]#[noacs] },#{?mode_tree_flat,,  }}\0"
                    as *const u8 as *const ::core::ffi::c_char,
            );
            prefix_width = format_width(prefix.as_ptr());
            if prefix_width > w {
                prefix_width = w;
            }
            if (*mti).tagged != 0 {
                tag = b"*\0" as *const u8 as *const ::core::ffi::c_char;
            } else {
                tag = b"\0" as *const u8 as *const ::core::ffi::c_char;
            }
            if !(*mti).text.is_none() {
                separator = b"#[fg=themelightgrey]: #[default]\0" as *const u8
                    as *const ::core::ffi::c_char;
            } else {
                separator = b"\0" as *const u8 as *const ::core::ffi::c_char;
            }
            let field_width = (*mti).align * *alignlen.as_mut_ptr().offset((*line).depth as isize);
            let mut name = Vec::new();
            mode_tree_append_printf_string(&mut name, Some((*mti).name.as_c_str()));
            let padding = (field_width.unsigned_abs() as usize).saturating_sub(name.len());
            let mut row = Vec::with_capacity(
                name.len()
                    + padding
                    + CStr::from_ptr(tag).to_bytes().len()
                    + CStr::from_ptr(separator).to_bytes().len(),
            );
            if field_width >= 0 {
                row.extend(std::iter::repeat_n(b' ', padding));
            }
            row.extend_from_slice(&name);
            if field_width < 0 {
                row.extend(std::iter::repeat_n(b' ', padding));
            }
            row.extend_from_slice(CStr::from_ptr(tag).to_bytes());
            row.extend_from_slice(CStr::from_ptr(separator).to_bytes());
            let text = CString::new(row).expect("mode-tree row contains no NUL");
            text_width = format_width(text.as_ptr());
            left = if prefix_width < w {
                w.wrapping_sub(prefix_width)
            } else {
                0 as u_int
            };
            if text_width > left {
                text_width = left;
            }
            width = prefix_width.wrapping_add(text_width);
            if (*mti).tagged != 0 {
                gc.fg = COLOUR_THEME_CYAN as ::core::ffi::c_int | COLOUR_FLAG_THEME;
                gc0.fg = COLOUR_THEME_CYAN as ::core::ffi::c_int | COLOUR_FLAG_THEME;
            }
            if i != (*mtd).current {
                screen_write_clearendofline(&raw mut ctx, 8 as u_int);
                format_draw(
                    &raw mut ctx,
                    &raw const grid_default_cell,
                    prefix_width,
                    prefix.as_ptr(),
                    ::core::ptr::null_mut::<style_ranges>(),
                    0 as ::core::ffi::c_int,
                );
                if left != 0 as u_int {
                    screen_write_cursormove(
                        &raw mut ctx,
                        prefix_width as ::core::ffi::c_int,
                        i.wrapping_sub((*mtd).offset) as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    );
                    format_draw(
                        &raw mut ctx,
                        &raw mut gc0,
                        left,
                        text.as_ptr(),
                        ::core::ptr::null_mut::<style_ranges>(),
                        0 as ::core::ffi::c_int,
                    );
                    if !(*mti).text.is_none() && width < w {
                        screen_write_cursormove(
                            &raw mut ctx,
                            width as ::core::ffi::c_int,
                            i.wrapping_sub((*mtd).offset) as ::core::ffi::c_int,
                            0 as ::core::ffi::c_int,
                        );
                        format_draw(
                            &raw mut ctx,
                            &raw mut gc0,
                            w.wrapping_sub(width),
                            ((*mti).text)
                                .as_ref()
                                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                            ::core::ptr::null_mut::<style_ranges>(),
                            0 as ::core::ffi::c_int,
                        );
                    }
                }
            } else {
                screen_write_clearendofline(&raw mut ctx, gc.bg as u_int);
                format_draw(
                    &raw mut ctx,
                    &raw mut gc,
                    prefix_width,
                    prefix.as_ptr(),
                    ::core::ptr::null_mut::<style_ranges>(),
                    1 as ::core::ffi::c_int,
                );
                if left != 0 as u_int {
                    screen_write_cursormove(
                        &raw mut ctx,
                        prefix_width as ::core::ffi::c_int,
                        i.wrapping_sub((*mtd).offset) as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    );
                    format_draw(
                        &raw mut ctx,
                        &raw mut gc,
                        left,
                        text.as_ptr(),
                        ::core::ptr::null_mut::<style_ranges>(),
                        1 as ::core::ffi::c_int,
                    );
                    if !(*mti).text.is_none() && width < w {
                        screen_write_cursormove(
                            &raw mut ctx,
                            width as ::core::ffi::c_int,
                            i.wrapping_sub((*mtd).offset) as ::core::ffi::c_int,
                            0 as ::core::ffi::c_int,
                        );
                        format_draw(
                            &raw mut ctx,
                            &raw mut gc,
                            w.wrapping_sub(width),
                            ((*mti).text)
                                .as_ref()
                                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                            ::core::ptr::null_mut::<style_ranges>(),
                            1 as ::core::ffi::c_int,
                        );
                    }
                }
            }
            drop(text);
            if (*mti).tagged != 0 {
                gc.fg = dfg;
                gc0.fg = dfg0;
            }
        }
        i = i.wrapping_add(1);
    }
    format_free(ft);
    if !((*mtd).preview == MODE_TREE_PREVIEW_OFF as ::core::ffi::c_int) {
        sy = (*s).grid().sy;
        if !(sy <= 4 as u_int
            || h < 2 as u_int
            || sy.wrapping_sub(h) <= 4 as u_int
            || w <= 4 as u_int)
        {
            line = (*mtd).lines.as_mut_ptr().offset((*mtd).current as isize) as *mut mode_tree_line
                as *mut mode_tree_line;
            mti = (*line).item;
            if (*mti).draw_as_parent != 0 {
                mti = (*mti).parent;
            }
            screen_write_cursormove(
                &raw mut ctx,
                0 as ::core::ffi::c_int,
                h as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            screen_write_box(
                &raw mut ctx,
                w,
                sy.wrapping_sub(h),
                BOX_LINES_DEFAULT,
                &raw mut box_gc,
                ::core::ptr::null::<::core::ffi::c_char>(),
            );
            let mut label_bytes = b" ".to_vec();
            mode_tree_append_printf_string(&mut label_bytes, Some((*mti).name.as_c_str()));
            if !(&(*mtd).sort_crit.order_seq).is_empty() {
                label_bytes.extend_from_slice(b" (sort: ");
                let order = sort_order_to_string((*mtd).sort_crit.order);
                let order = if order.is_null() {
                    None
                } else {
                    Some(CStr::from_ptr(order))
                };
                mode_tree_append_printf_string(&mut label_bytes, order);
                if (*mtd).sort_crit.reversed != 0 {
                    label_bytes.extend_from_slice(b", reversed");
                }
                label_bytes.push(b')');
                if let Some(view_name) = (*mtd).view_name {
                    label_bytes.extend_from_slice(b" (view: ");
                    mode_tree_append_printf_string(&mut label_bytes, Some(view_name));
                    label_bytes.push(b')');
                }
            }
            let label = CString::new(label_bytes).expect("preview label contains no NUL");
            let label_len = label.as_bytes().len() as size_t;
            if w.wrapping_sub(2 as u_int) as size_t >= label_len {
                screen_write_cursormove(
                    &raw mut ctx,
                    1 as ::core::ffi::c_int,
                    h as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
                screen_write_puts(&raw mut ctx, &raw mut box_gc, |out| {
                    write_cstr(out, label.as_ptr())
                });
                if (*mtd).no_matches != 0 {
                    n = (::core::mem::size_of::<[::core::ffi::c_char; 11]>() as usize)
                        .wrapping_sub(1 as usize) as size_t;
                } else {
                    n = (::core::mem::size_of::<[::core::ffi::c_char; 7]>() as usize)
                        .wrapping_sub(1 as usize) as size_t;
                }
                if (*mtd).filter.is_some()
                    && w.wrapping_sub(2 as u_int) as size_t
                        >= label_len
                            .wrapping_add(10 as size_t)
                            .wrapping_add(n)
                            .wrapping_add(2 as size_t)
                {
                    screen_write_puts(&raw mut ctx, &raw mut box_gc, |out| {
                        out.write_all(b" (filter: ")
                    });
                    if (*mtd).no_matches != 0 {
                        screen_write_puts(&raw mut ctx, &raw mut box_gc, |out| {
                            out.write_all(b"no matches")
                        });
                    } else {
                        screen_write_puts(&raw mut ctx, &raw mut box_gc, |out| {
                            out.write_all(b"active")
                        });
                    }
                    screen_write_puts(&raw mut ctx, &raw mut box_gc, |out| out.write_all(b") "));
                } else {
                    screen_write_puts(&raw mut ctx, &raw mut box_gc, |out| out.write_all(b" "));
                }
            }
            drop(label);
            box_x = w.wrapping_sub(4 as u_int);
            box_y = sy.wrapping_sub(h).wrapping_sub(2 as u_int);
            if box_x != 0 as u_int && box_y != 0 as u_int {
                screen_write_cursormove(
                    &raw mut ctx,
                    2 as ::core::ffi::c_int,
                    h.wrapping_add(1 as u_int) as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
                (*mtd).drawcb.as_mut().expect("non-null draw callback")(
                    (*mti).itemdata,
                    &mut ctx,
                    box_x,
                    box_y,
                );
            }
        }
    }
    if (*mtd).help != 0 {
        mode_tree_draw_help(mtd, &raw mut ctx);
    }
    if !(*mtd).prompt.is_null() {
        mode_tree_draw_prompt(mtd, &raw mut ctx);
    } else {
        (*s).mode &= !MODE_CURSOR;
        screen_write_cursormove(
            &raw mut ctx,
            0 as ::core::ffi::c_int,
            (*mtd).current.wrapping_sub((*mtd).offset) as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
    }
    screen_write_stop(&raw mut ctx);
}
unsafe fn mode_tree_draw_prompt(mut mtd: *mut mode_tree_data, mut ctx: *mut screen_write_ctx) {
    let mut s: *mut screen = &raw mut (*mtd).screen;
    let mut pdd: prompt_draw_data = prompt_draw_data {
        ctx: ::core::ptr::null_mut::<screen_write_ctx>(),
        cursor_x: ::core::ptr::null_mut::<u_int>(),
        area_x: 0,
        area_width: 0,
        prompt_line: 0,
    };
    let mut sx: u_int = (*s).grid().sx;
    let mut sy: u_int = (*s).grid().sy;
    let mut py: u_int = 0;
    if sx == 0 as u_int || sy == 0 as u_int {
        return;
    }
    if (*mtd).prompt_top != 0 {
        py = 0 as u_int;
    } else {
        py = sy.wrapping_sub(1 as u_int);
    }
    pdd.ctx = ctx;
    pdd.cursor_x = &raw mut (*mtd).prompt_cx;
    pdd.area_x = 0 as u_int;
    pdd.area_width = sx;
    pdd.prompt_line = py;
    (*s).mode |= MODE_CURSOR;
    prompt_draw((*mtd).prompt, &raw mut pdd);
    screen_write_cursormove(
        ctx,
        (*mtd).prompt_cx as ::core::ffi::c_int,
        py as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
}
pub unsafe fn mode_tree_clear_prompt(mut mtd: *mut mode_tree_data) {
    let mut prompt: *mut prompt = (*mtd).prompt;
    if !(*mtd).prompt.is_null() {
        (*mtd).prompt = ::core::ptr::null_mut::<prompt>();
        prompt_free(prompt);
        (*mtd).screen.mode &= !MODE_CURSOR;
    }
}
unsafe fn mode_tree_prompt_accept(
    mut item: *mut cmdq_item,
    mut mtd: *mut mode_tree_data,
) -> cmd_retval {
    let mut c: *mut client = cmdq_get_client(item);
    let mut key: key_code = 'y' as i32 as key_code;
    if !(*mtd).prompt.is_null() && !c.is_null() {
        mode_tree_key(
            mtd,
            c,
            &raw mut key,
            ::core::ptr::null_mut::<mouse_event>(),
            ::core::ptr::null_mut::<u_int>(),
            ::core::ptr::null_mut::<u_int>(),
        );
    }
    mode_tree_remove_ref(mtd);
    return CMD_RETURN_NORMAL;
}
unsafe fn mode_tree_cancel_prompt_accept(mtd: *mut mode_tree_data) {
    mode_tree_remove_ref(mtd);
}
unsafe fn mode_tree_prompt_input_callback(
    mut mtp: *mut mode_tree_prompt,
    s: Option<&CStr>,
    mut key: prompt_key_result,
) -> prompt_result {
    if let Some(inputcb) = (*mtp).inputcb.as_mut() {
        return inputcb(std::ptr::NonNull::new((*mtp).c), s, key);
    }
    return PROMPT_CLOSE;
}
unsafe fn mode_tree_prompt_free_callback(mtp_ptr: *mut mode_tree_prompt) {
    let mut mtp = Box::from_raw(mtp_ptr);
    if (*mtp.mtd).prompt_data == mtp_ptr {
        (*mtp.mtd).prompt_data = ::core::ptr::null_mut::<mode_tree_prompt>();
    }
    if let Some(freecb) = mtp.freecb.take() {
        freecb();
    }
    mode_tree_remove_ref(mtp.mtd);
}
pub unsafe fn mode_tree_set_prompt(
    mut mtd: *mut mode_tree_data,
    mut c: *mut client,
    mut prompt: *const ::core::ffi::c_char,
    mut input: *const ::core::ffi::c_char,
    mut type_0: prompt_type,
    mut flags: ::core::ffi::c_int,
    mut inputcb: mode_tree_prompt_input_cb,
    mut freecb: prompt_free_cb,
) {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    let mut pd = prompt_create_data::default();
    let mut mtp: *mut mode_tree_prompt = ::core::ptr::null_mut::<mode_tree_prompt>();
    if !c.is_null() && !(*c).session.is_null() {
        s = (*c).session;
        oo = (*s).options;
    } else {
        s = ::core::ptr::null_mut::<session>();
        oo = global_s_options;
    }
    mode_tree_clear_prompt(mtd);
    mtp = Box::into_raw(Box::new(mode_tree_prompt {
        mtd,
        c,
        inputcb,
        freecb,
    }));
    crate::src::shared::rc::retain(mtd);
    (*mtd).prompt_top = (options_get_number(
        oo,
        b"status-position\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_longlong) as ::core::ffi::c_int;
    prompt_set_options(&raw mut pd, s);
    pd.prompt = prompt;
    pd.input = input;
    pd.type_0 = type_0;
    pd.flags = flags | PROMPT_ISMODE;
    pd.inputcb = Some(Box::new(move |s, key| unsafe {
        mode_tree_prompt_input_callback(mtp, s, key)
    }));
    pd.freecb = Some(Box::new(move || unsafe {
        mode_tree_prompt_free_callback(mtp)
    }));
    (*mtd).prompt = prompt_create(&raw mut pd);
    (*mtd).prompt_data = mtp;
    mode_tree_draw(mtd);
    (*(*mtd).wp).flags |= PANE_REDRAW;
    if flags & PROMPT_SINGLE != 0 && flags & PROMPT_ACCEPT != 0 && !c.is_null() {
        crate::src::shared::rc::retain(mtd);
        let item = cmdq_get_callback_owned(
            b"mode_tree_prompt_accept\0" as *const u8 as *const ::core::ffi::c_char,
            Some(Box::new(move |item| unsafe {
                mode_tree_prompt_accept(item.as_ptr(), mtd)
            })),
        );
        cmdq_set_cancel_callback(
            &mut *item,
            Box::new(move || unsafe { mode_tree_cancel_prompt_accept(mtd) }),
        );
        cmdq_append(c, item);
    }
}
unsafe fn mode_tree_search_backward(mut mtd: *mut mode_tree_data) -> *mut mode_tree_item {
    let mut mti: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut last: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut prev: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut icase: ::core::ffi::c_int = (*mtd).search_icase;
    let Some(search) = (*mtd).search.as_ref() else {
        return ::core::ptr::null_mut::<mode_tree_item>();
    };
    let search = search.as_ptr();
    last = (*(*mtd).lines.as_mut_ptr().offset((*mtd).current as isize)).item;
    mti = last;
    loop {
        prev = (*mode_tree_siblings(mtd, mti)).previous(mti);
        if !prev.is_null() {
            while !(*prev).children.first().is_null() {
                prev = (*prev).children.last();
            }
            mti = prev;
        } else {
            mti = (*mti).parent;
        }
        if mti.is_null() {
            prev = (*mtd).children.last();
            while !(*prev).children.first().is_null() {
                prev = (*prev).children.last();
            }
            mti = prev;
        }
        if mti == last {
            break;
        }
        if (*mtd).searchcb.is_none() {
            if icase == 0 && !strstr(((*mti).name).as_ptr().cast_mut(), search.cast_mut()).is_null()
            {
                return mti;
            }
            if icase != 0
                && !strcasestr(((*mti).name).as_ptr().cast_mut(), search.cast_mut()).is_null()
            {
                return mti;
            }
        } else if (*mtd).searchcb.as_mut().expect("non-null search callback")(
            (*mti).itemdata,
            CStr::from_ptr(search),
            icase != 0,
        ) {
            return mti;
        }
    }
    return ::core::ptr::null_mut::<mode_tree_item>();
}
unsafe fn mode_tree_search_forward(mut mtd: *mut mode_tree_data) -> *mut mode_tree_item {
    let mut mti: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut last: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut next: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut icase: ::core::ffi::c_int = (*mtd).search_icase;
    let Some(search) = (*mtd).search.as_ref() else {
        return ::core::ptr::null_mut::<mode_tree_item>();
    };
    let search = search.as_ptr();
    last = (*(*mtd).lines.as_mut_ptr().offset((*mtd).current as isize)).item;
    mti = last;
    loop {
        if !(*mti).children.first().is_null() {
            mti = (*mti).children.first();
        } else {
            next = (*mode_tree_siblings(mtd, mti)).next(mti);
            if !next.is_null() {
                mti = next;
            } else {
                loop {
                    mti = (*mti).parent;
                    if mti.is_null() {
                        break;
                    }
                    next = (*mode_tree_siblings(mtd, mti)).next(mti);
                    if next.is_null() {
                        continue;
                    }
                    mti = next;
                    break;
                }
            }
        }
        if mti.is_null() {
            mti = (*mtd).children.first();
        }
        if mti == last {
            break;
        }
        if (*mtd).searchcb.is_none() {
            if icase == 0 && !strstr(((*mti).name).as_ptr().cast_mut(), search.cast_mut()).is_null()
            {
                return mti;
            }
            if icase != 0
                && !strcasestr(((*mti).name).as_ptr().cast_mut(), search.cast_mut()).is_null()
            {
                return mti;
            }
        } else if (*mtd).searchcb.as_mut().expect("non-null search callback")(
            (*mti).itemdata,
            CStr::from_ptr(search),
            icase != 0,
        ) {
            return mti;
        }
    }
    return ::core::ptr::null_mut::<mode_tree_item>();
}
unsafe fn mode_tree_search_set(mut mtd: *mut mode_tree_data) {
    let mut mti: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut loop_0: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut tag: uint64_t = 0;
    if (*mtd).search_dir as ::core::ffi::c_uint
        == MODE_TREE_SEARCH_FORWARD as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        mti = mode_tree_search_forward(mtd);
    } else {
        mti = mode_tree_search_backward(mtd);
    }
    if mti.is_null() {
        return;
    }
    tag = (*mti).tag;
    loop_0 = (*mti).parent;
    while !loop_0.is_null() {
        (*loop_0).expanded = 1 as ::core::ffi::c_int;
        loop_0 = (*loop_0).parent;
    }
    mode_tree_build(mtd);
    mode_tree_set_current(mtd, tag);
    mode_tree_draw(mtd);
    (*(*mtd).wp).flags |= PANE_REDRAW;
}
unsafe fn mode_tree_search_callback(
    mut mtd: *mut mode_tree_data,
    s: Option<&CStr>,
    mut key: prompt_key_result,
) -> prompt_result {
    if (*mtd).dead != 0 {
        return PROMPT_CLOSE;
    }
    let replacement = s
        .filter(|text| !text.to_bytes().is_empty())
        .map(CStr::to_owned);
    (*mtd).search = replacement;
    if let Some(search) = (*mtd).search.as_ref() {
        (*mtd).search_icase = mode_tree_is_lowercase(search.as_ptr());
        mode_tree_search_set(mtd);
    }
    if key as ::core::ffi::c_uint == PROMPT_KEY_HANDLED as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return PROMPT_CONTINUE;
    }
    return PROMPT_CLOSE;
}
unsafe fn mode_tree_filter_callback(
    mut mtd: *mut mode_tree_data,
    s: Option<&CStr>,
    mut key: prompt_key_result,
) -> prompt_result {
    if (*mtd).dead != 0 {
        return PROMPT_CLOSE;
    }
    let replacement = s
        .filter(|text| !text.to_bytes().is_empty())
        .map(CStr::to_owned);
    (*mtd).filter = replacement;
    mode_tree_build(mtd);
    mode_tree_draw(mtd);
    (*(*mtd).wp).flags |= PANE_REDRAW;
    if key as ::core::ffi::c_uint == PROMPT_KEY_HANDLED as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return PROMPT_CONTINUE;
    }
    return PROMPT_CLOSE;
}
unsafe fn mode_tree_clear_filter(mut mtd: *mut mode_tree_data) {
    (*mtd).filter = None;
    mode_tree_build(mtd);
    mode_tree_draw(mtd);
    (*(*mtd).wp).flags |= PANE_REDRAW;
}
unsafe fn mode_tree_display_menu(
    mut mtd: *mut mode_tree_data,
    mut c: *mut client,
    mut x: u_int,
    mut y: u_int,
    mut outside: ::core::ffi::c_int,
) {
    let mut mti: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut menu: *mut menu = ::core::ptr::null_mut::<menu>();
    let mut items: *const menu_item = ::core::ptr::null::<menu_item>();
    let mut line: u_int = 0;
    if (*mtd).offset.wrapping_add(y) > mode_tree_line_count(&*mtd).wrapping_sub(1 as u_int) {
        line = (*mtd).current;
    } else {
        line = (*mtd).offset.wrapping_add(y);
    }
    mti = (*(*mtd).lines.as_mut_ptr().offset(line as isize)).item;
    let title = if outside == 0 {
        items = (*mtd).menu;
        let mut bytes = b"#[align=centre]".to_vec();
        bytes.extend_from_slice((*mti).name.as_bytes());
        CString::new(bytes).expect("mode tree item names contain no NUL")
    } else {
        items = &raw const mode_tree_menu_items as *const menu_item;
        c"".to_owned()
    };
    menu = menu_create(title.as_ptr());
    menu_add_items(menu, items, c);
    drop(title);
    crate::src::shared::rc::retain(mtd);
    if x >= (*menu)
        .width
        .wrapping_add(4 as u_int)
        .wrapping_div(2 as u_int)
    {
        x = x.wrapping_sub(
            (*menu)
                .width
                .wrapping_add(4 as u_int)
                .wrapping_div(2 as u_int),
        );
    } else {
        x = 0 as u_int;
    }
    x = x.wrapping_add((*(*mtd).wp).xoff as u_int);
    y = y.wrapping_add((*(*mtd).wp).yoff as u_int);
    if menu_display(
        menu,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        ::core::ptr::null_mut::<cmdq_item>(),
        x,
        y,
        c,
        BOX_LINES_DEFAULT,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<cmd_find_state>(),
        Some(Box::new(move |selection| {
            if let MenuSelection::Selected { key, .. } = selection {
                if !((*mtd).dead != 0 || key == KEYC_NONE as ::core::ffi::c_ulong as key_code)
                    && line < mode_tree_line_count(&*mtd)
                {
                    (*mtd).current = line;
                    if let Some(callback) = (*mtd).menucb.as_mut() {
                        callback(std::ptr::NonNull::new(c), key);
                    }
                }
            }
            mode_tree_remove_ref(mtd);
        })),
    ) != 0 as ::core::ffi::c_int
    {
        mode_tree_remove_ref(mtd);
        menu_free(menu);
    }
}
unsafe fn mode_tree_draw_help_line(
    mut ctx: *mut screen_write_ctx,
    mut gc: *const grid_cell,
    mut ft: *mut format_tree,
    line: &CStr,
    item: &CStr,
    mut x: u_int,
    mut y: u_int,
    mut w: u_int,
) {
    let replaced = cmd_template_replace_cstring(line, item, 1 as ::core::ffi::c_int);
    let expanded = format_expand_cstring(ft, replaced.as_ptr());
    drop(replaced);
    screen_write_cursormove(
        ctx,
        x as ::core::ffi::c_int,
        y as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    screen_write_clearcharacter(ctx, w, (*gc).bg as u_int);
    screen_write_cursormove(
        ctx,
        x as ::core::ffi::c_int,
        y as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    format_draw(
        ctx,
        gc,
        w,
        expanded.as_ptr(),
        ::core::ptr::null_mut::<style_ranges>(),
        0 as ::core::ffi::c_int,
    );
}
unsafe fn mode_tree_draw_help(mut mtd: *mut mode_tree_data, mut ctx: *mut screen_write_ctx) {
    let mut s: *mut screen = &raw mut (*mtd).screen;
    let mut oo: *mut options = (*(*(*mtd).wp).window).options;
    let mut box_gc: grid_cell = grid_cell {
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
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let help = (*mtd).helpcb.map(|callback| callback());
    let lines: &[&'static CStr] = help.map_or(&[], |info| info.lines);
    let item = help.map_or(c"item", |info| info.item);
    let mut sx: u_int = (*s).grid().sx;
    let mut sy: u_int = (*s).grid().sy;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut w = help
        .map_or(MODE_TREE_HELP_DEFAULT_WIDTH as u_int, |info| info.width)
        .max(MODE_TREE_HELP_DEFAULT_WIDTH as u_int);
    let h = (mode_tree_help_start.len() + lines.len() + mode_tree_help_end.len()) as u_int;
    let mut box_w: u_int = 0;
    let mut box_h: u_int = 0;
    box_w = w.wrapping_add(2 as u_int);
    box_h = h.wrapping_add(2 as u_int);
    if sx < box_w || sy < box_h {
        return;
    }
    x = sx.wrapping_sub(box_w).wrapping_div(2 as u_int);
    y = sy.wrapping_sub(box_h).wrapping_div(2 as u_int);
    memcpy(
        &raw mut box_gc as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    style_apply(
        &raw mut box_gc,
        oo,
        b"tree-mode-border-style\0" as *const u8 as *const ::core::ffi::c_char,
        ::core::ptr::null_mut::<format_tree>(),
    );
    memcpy(
        &raw mut gc as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    ft = format_create_defaults(
        ::core::ptr::null_mut::<cmdq_item>(),
        ::core::ptr::null_mut::<client>(),
        ::core::ptr::null_mut::<session>(),
        ::core::ptr::null_mut::<winlink>(),
        (*mtd).wp,
    );
    screen_write_cursormove(
        ctx,
        x as ::core::ffi::c_int,
        y as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    screen_write_box(
        ctx,
        box_w,
        box_h,
        BOX_LINES_DEFAULT,
        &raw mut box_gc,
        ::core::ptr::null::<::core::ffi::c_char>(),
    );
    y = y.wrapping_add(1);
    x = x.wrapping_add(1);
    for line in mode_tree_help_start.iter().copied() {
        mode_tree_draw_help_line(ctx, &gc, ft, line, item, x, y, w);
        y = y.wrapping_add(1);
    }
    for line in lines.iter().copied() {
        mode_tree_draw_help_line(ctx, &gc, ft, line, item, x, y, w);
        y = y.wrapping_add(1);
    }
    for line in mode_tree_help_end.iter().copied() {
        mode_tree_draw_help_line(ctx, &gc, ft, line, item, x, y, w);
        y = y.wrapping_add(1);
    }
    format_free(ft);
}
unsafe fn mode_tree_display_help(mut mtd: *mut mode_tree_data) {
    (*mtd).help = 1 as ::core::ffi::c_int;
    mode_tree_draw(mtd);
}
pub unsafe fn mode_tree_key(
    mut mtd: *mut mode_tree_data,
    mut c: *mut client,
    mut key: *mut key_code,
    mut m: *mut mouse_event,
    mut xp: *mut u_int,
    mut yp: *mut u_int,
) -> ::core::ffi::c_int {
    let mut line: *mut mode_tree_line = ::core::ptr::null_mut::<mode_tree_line>();
    let mut current: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut parent: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let _mti: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut i: u_int = 0;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut py: u_int = 0;
    let mut sx: u_int = 0;
    let mut choice: ::core::ffi::c_int = 0;
    let mut preview: ::core::ffi::c_int = 0;
    let mut result: prompt_key_result = PROMPT_KEY_NOT_HANDLED;
    let mut redraw: ::core::ffi::c_int = 0;
    let mut prompt: *mut prompt = ::core::ptr::null_mut::<prompt>();
    let mut mtp: *mut mode_tree_prompt = ::core::ptr::null_mut::<mode_tree_prompt>();
    if mode_tree_line_count(&*mtd) == 0 as u_int {
        *key = KEYC_NONE as ::core::ffi::c_ulong as key_code;
        return 1 as ::core::ffi::c_int;
    }
    if !(*mtd).prompt.is_null() {
        redraw = 0 as ::core::ffi::c_int;
        prompt = (*mtd).prompt;
        mtp = (*mtd).prompt_data;
        if !mtp.is_null() {
            (*mtp).c = c;
        }
        if *key & KEYC_MASK_KEY == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
            || *key & KEYC_MASK_TYPE
                >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int
                && *key & KEYC_MASK_TYPE
                    <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                        << 32 as ::core::ffi::c_int
        {
            if m.is_null()
                || (*m).b & MOUSE_MASK_BUTTONS as u_int != MOUSE_BUTTON_1 as u_int
                || (*m).b & MOUSE_MASK_DRAG as u_int != 0
                || (*m).b & MOUSE_MASK_BUTTONS as u_int == 3 as u_int
                || cmd_mouse_at(
                    (*mtd).wp,
                    m,
                    &raw mut x,
                    &raw mut y,
                    0 as ::core::ffi::c_int,
                ) != 0 as ::core::ffi::c_int
            {
                result = PROMPT_KEY_NOT_HANDLED;
            } else {
                sx = (*mtd).screen.grid().sx;
                if (*mtd).prompt_top != 0 {
                    py = 0 as u_int;
                } else {
                    py = (*mtd).screen.grid().sy.wrapping_sub(1 as u_int);
                }
                if y == py {
                    result = prompt_mouse(prompt, x, 0 as u_int, sx, &raw mut redraw);
                } else {
                    result = PROMPT_KEY_NOT_HANDLED;
                }
            }
        } else {
            result = prompt_key(prompt, *key, &raw mut redraw);
        }
        if (*mtd).prompt_data == mtp && !mtp.is_null() {
            (*mtp).c = ::core::ptr::null_mut::<client>();
        }
        if (*mtd).prompt == prompt
            && (result as ::core::ffi::c_uint
                == PROMPT_KEY_CLOSE as ::core::ffi::c_int as ::core::ffi::c_uint
                || prompt_closed(prompt) != 0)
        {
            mode_tree_clear_prompt(mtd);
        }
        if redraw != 0 || (*mtd).prompt != prompt {
            mode_tree_draw(mtd);
            (*(*mtd).wp).flags |= PANE_REDRAW;
        }
        if result as ::core::ffi::c_uint
            != PROMPT_KEY_NOT_HANDLED as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            *key = KEYC_NONE as ::core::ffi::c_ulong as key_code;
            return 0 as ::core::ffi::c_int;
        }
    }
    if (*mtd).help != 0 {
        if *key & KEYC_MASK_KEY == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
            || *key & KEYC_MASK_TYPE
                >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int
                && *key & KEYC_MASK_TYPE
                    <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                        << 32 as ::core::ffi::c_int
        {
            *key = KEYC_NONE as ::core::ffi::c_ulong as key_code;
            return 0 as ::core::ffi::c_int;
        }
        if *key == KEYC_FOCUS_IN as ::core::ffi::c_ulong as key_code
            || *key == KEYC_FOCUS_OUT as ::core::ffi::c_ulong as key_code
        {
            *key = KEYC_NONE as ::core::ffi::c_ulong as key_code;
            return 0 as ::core::ffi::c_int;
        }
        (*mtd).help = 0 as ::core::ffi::c_int;
        mode_tree_draw(mtd);
        *key = KEYC_NONE as ::core::ffi::c_ulong as key_code;
        return 0 as ::core::ffi::c_int;
    }
    if (*key & KEYC_MASK_KEY == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
        || *key & KEYC_MASK_TYPE
            >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                << 32 as ::core::ffi::c_int
            && *key & KEYC_MASK_TYPE
                <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int)
        && !m.is_null()
    {
        if cmd_mouse_at(
            (*mtd).wp,
            m,
            &raw mut x,
            &raw mut y,
            0 as ::core::ffi::c_int,
        ) != 0 as ::core::ffi::c_int
        {
            *key = KEYC_NONE as ::core::ffi::c_ulong as key_code;
            return 0 as ::core::ffi::c_int;
        }
        if !xp.is_null() {
            *xp = x;
        }
        if !yp.is_null() {
            *yp = y;
        }
        if x > (*mtd).width || y > (*mtd).height {
            preview = (*mtd).preview;
            if *key == KEYC_MOUSEDOWN3_PANE as ::core::ffi::c_ulong as key_code {
                mode_tree_display_menu(mtd, c, x, y, 1 as ::core::ffi::c_int);
            }
            if preview == MODE_TREE_PREVIEW_OFF as ::core::ffi::c_int {
                *key = KEYC_NONE as ::core::ffi::c_ulong as key_code;
            }
            return 0 as ::core::ffi::c_int;
        }
        if (*mtd).offset.wrapping_add(y) < mode_tree_line_count(&*mtd) {
            if *key == KEYC_MOUSEDOWN1_PANE as ::core::ffi::c_ulong as key_code
                || *key == KEYC_MOUSEDOWN3_PANE as ::core::ffi::c_ulong as key_code
                || *key == KEYC_DOUBLECLICK1_PANE as ::core::ffi::c_ulong as key_code
            {
                (*mtd).current = (*mtd).offset.wrapping_add(y);
            }
            if *key == KEYC_DOUBLECLICK1_PANE as ::core::ffi::c_ulong as key_code {
                *key = '\r' as i32 as key_code;
            } else {
                if *key == KEYC_MOUSEDOWN3_PANE as ::core::ffi::c_ulong as key_code {
                    mode_tree_display_menu(mtd, c, x, y, 0 as ::core::ffi::c_int);
                }
                *key = KEYC_NONE as ::core::ffi::c_ulong as key_code;
            }
        } else {
            if *key == KEYC_MOUSEDOWN3_PANE as ::core::ffi::c_ulong as key_code {
                mode_tree_display_menu(mtd, c, x, y, 0 as ::core::ffi::c_int);
            }
            *key = KEYC_NONE as ::core::ffi::c_ulong as key_code;
        }
        return 0 as ::core::ffi::c_int;
    }
    line = (*mtd).lines.as_mut_ptr().offset((*mtd).current as isize) as *mut mode_tree_line
        as *mut mode_tree_line;
    current = (*line).item;
    choice = -(1 as ::core::ffi::c_int);
    i = 0 as u_int;
    while i < mode_tree_line_count(&*mtd) {
        if *key == (*(*(*mtd).lines.as_mut_ptr().offset(i as isize)).item).key {
            choice = i as ::core::ffi::c_int;
            break;
        } else {
            i = i.wrapping_add(1);
        }
    }
    if choice != -(1 as ::core::ffi::c_int) {
        if choice as u_int > mode_tree_line_count(&*mtd).wrapping_sub(1 as u_int) {
            *key = KEYC_NONE as ::core::ffi::c_ulong as key_code;
            return 0 as ::core::ffi::c_int;
        }
        (*mtd).current = choice as u_int;
        *key = '\r' as i32 as key_code;
        return 0 as ::core::ffi::c_int;
    }
    match *key {
        113 | 27 | 35184372088923 | 35184372088935 => return 1 as ::core::ffi::c_int,
        8589934600 | 35184372088936 => {
            mode_tree_display_help(mtd);
        }
        8589934619 | 107 | 38654705664 | 35184372088944 => {
            mode_tree_up(mtd, 1 as ::core::ffi::c_int);
        }
        8589934620 | 106 | 34359738368 | 35184372088942 => {
            mode_tree_down(mtd, 1 as ::core::ffi::c_int);
        }
        70377334112283 | 75 => {
            mode_tree_swap(mtd, -(1 as ::core::ffi::c_int));
        }
        70377334112284 | 74 => {
            mode_tree_swap(mtd, 1 as ::core::ffi::c_int);
        }
        8589934617 | 35184372088930 => {
            i = 0 as u_int;
            while i < (*mtd).height {
                if (*mtd).current == 0 as u_int {
                    break;
                }
                mode_tree_up(mtd, 1 as ::core::ffi::c_int);
                i = i.wrapping_add(1);
            }
        }
        8589934616 | 35184372088934 => {
            i = 0 as u_int;
            while i < (*mtd).height {
                if (*mtd).current == mode_tree_line_count(&*mtd).wrapping_sub(1 as u_int) {
                    break;
                }
                mode_tree_down(mtd, 1 as ::core::ffi::c_int);
                i = i.wrapping_add(1);
            }
        }
        103 | 8589934614 => {
            (*mtd).current = 0 as u_int;
            (*mtd).offset = 0 as u_int;
        }
        71 | 8589934615 => {
            (*mtd).current = mode_tree_line_count(&*mtd).wrapping_sub(1 as u_int);
            if (*mtd).current > (*mtd).height.wrapping_sub(1 as u_int) {
                (*mtd).offset = (*mtd)
                    .current
                    .wrapping_sub((*mtd).height)
                    .wrapping_add(1 as u_int);
            } else {
                (*mtd).offset = 0 as u_int;
            }
        }
        116 => {
            if !((*current).no_tag != 0) {
                if (*current).tagged == 0 {
                    parent = (*current).parent;
                    while !parent.is_null() {
                        (*parent).tagged = 0 as ::core::ffi::c_int;
                        parent = (*parent).parent;
                    }
                    mode_tree_clear_tagged(&raw mut (*current).children);
                    (*current).tagged = 1 as ::core::ffi::c_int;
                } else {
                    (*current).tagged = 0 as ::core::ffi::c_int;
                }
                if !m.is_null() {
                    mode_tree_down(mtd, 0 as ::core::ffi::c_int);
                }
            }
        }
        84 => {
            i = 0 as u_int;
            while i < mode_tree_line_count(&*mtd) {
                (*(*(*mtd).lines.as_mut_ptr().offset(i as isize)).item).tagged =
                    0 as ::core::ffi::c_int;
                i = i.wrapping_add(1);
            }
        }
        35184372088948 => {
            i = 0 as u_int;
            while i < mode_tree_line_count(&*mtd) {
                if (*(*(*mtd).lines.as_mut_ptr().offset(i as isize)).item)
                    .parent
                    .is_null()
                    && (*(*(*mtd).lines.as_mut_ptr().offset(i as isize)).item).no_tag == 0
                    || !(*(*(*mtd).lines.as_mut_ptr().offset(i as isize)).item)
                        .parent
                        .is_null()
                        && (*(*(*(*mtd).lines.as_mut_ptr().offset(i as isize)).item).parent).no_tag
                            != 0
                {
                    (*(*(*mtd).lines.as_mut_ptr().offset(i as isize)).item).tagged =
                        1 as ::core::ffi::c_int;
                } else {
                    (*(*(*mtd).lines.as_mut_ptr().offset(i as isize)).item).tagged =
                        0 as ::core::ffi::c_int;
                }
                i = i.wrapping_add(1);
            }
        }
        79 => {
            sort_next_order(&raw mut (*mtd).sort_crit);
            mode_tree_build(mtd);
        }
        114 => {
            (*mtd).sort_crit.reversed = ((*mtd).sort_crit.reversed == 0) as ::core::ffi::c_int;
            mode_tree_build(mtd);
        }
        8589934621 | 104 | 45 => {
            if (*line).flat != 0 || (*current).expanded == 0 {
                current = (*current).parent;
            }
            if current.is_null() {
                mode_tree_up(mtd, 0 as ::core::ffi::c_int);
            } else {
                (*current).expanded = 0 as ::core::ffi::c_int;
                (*mtd).current = (*current).line;
                mode_tree_build(mtd);
            }
        }
        8589934622 | 108 | 43 => {
            if (*line).flat != 0 || (*current).expanded != 0 {
                mode_tree_down(mtd, 0 as ::core::ffi::c_int);
            } else if (*line).flat == 0 {
                (*current).expanded = 1 as ::core::ffi::c_int;
                mode_tree_build(mtd);
            }
        }
        17592186044461 => {
            for mti in (*mtd).children.pointers() {
                (*mti).expanded = 0 as ::core::ffi::c_int;
            }
            mode_tree_build(mtd);
        }
        17592186044459 => {
            for mti in (*mtd).children.pointers() {
                (*mti).expanded = 1 as ::core::ffi::c_int;
            }
            mode_tree_build(mtd);
        }
        63 | 47 | 35184372088947 => {
            (*mtd).search_dir = MODE_TREE_SEARCH_FORWARD;
            mode_tree_set_prompt(
                mtd,
                c,
                b"(search) \0" as *const u8 as *const ::core::ffi::c_char,
                b"\0" as *const u8 as *const ::core::ffi::c_char,
                PROMPT_TYPE_SEARCH,
                PROMPT_NOFORMAT,
                Some(Box::new(move |_, s, key| unsafe {
                    mode_tree_search_callback(mtd, s, key)
                })),
                None,
            );
        }
        110 => {
            (*mtd).search_dir = MODE_TREE_SEARCH_FORWARD;
            mode_tree_search_set(mtd);
        }
        78 => {
            (*mtd).search_dir = MODE_TREE_SEARCH_BACKWARD;
            mode_tree_search_set(mtd);
        }
        102 => {
            mode_tree_set_prompt(
                mtd,
                c,
                b"(filter) \0" as *const u8 as *const ::core::ffi::c_char,
                (*mtd)
                    .filter
                    .as_ref()
                    .map_or(b"\0" as *const u8 as *const ::core::ffi::c_char, |filter| {
                        filter.as_ptr()
                    }),
                PROMPT_TYPE_SEARCH,
                PROMPT_NOFORMAT,
                Some(Box::new(move |_, s, key| unsafe {
                    mode_tree_filter_callback(mtd, s, key)
                })),
                None,
            );
        }
        99 => {
            mode_tree_clear_prompt(mtd);
            mode_tree_clear_filter(mtd);
        }
        118 => {
            match (*mtd).preview {
                0 => {
                    (*mtd).preview = MODE_TREE_PREVIEW_BIG as ::core::ffi::c_int;
                }
                1 => {
                    (*mtd).preview = MODE_TREE_PREVIEW_OFF as ::core::ffi::c_int;
                }
                2 => {
                    (*mtd).preview = MODE_TREE_PREVIEW_NORMAL as ::core::ffi::c_int;
                }
                _ => {}
            }
            mode_tree_build(mtd);
            if (*mtd).preview != MODE_TREE_PREVIEW_OFF as ::core::ffi::c_int {
                mode_tree_check_selected(mtd);
            }
        }
        _ => {}
    }
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn mode_tree_run_command(
    mut c: *mut client,
    mut fs: *mut cmd_find_state,
    mut template: *const ::core::ffi::c_char,
    mut name: *const ::core::ffi::c_char,
) {
    let mut state: *mut cmdq_state = ::core::ptr::null_mut::<cmdq_state>();
    let command = cmd_template_replace_cstring(
        CStr::from_ptr(template),
        CStr::from_ptr(name),
        1 as ::core::ffi::c_int,
    );
    if !command.as_bytes().is_empty() {
        state = cmdq_new_state(
            fs,
            ::core::ptr::null_mut::<key_event>(),
            0 as ::core::ffi::c_int,
        );
        if let Err(mut error) = cmd_parse_and_append(command.as_c_str(), c, state) {
            if !c.is_null() {
                cmd_parse_error_uppercase_first(&mut error);
                status_message_set(
                    c,
                    -(1 as ::core::ffi::c_int),
                    1 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    |out| {
                        write_cstr(
                            out,
                            error
                                .as_ref()
                                .map_or(::core::ptr::null(), |cause| cause.as_ptr()),
                        )
                    },
                );
            }
        }
        cmdq_free_state(state);
    }
}

#[cfg(test)]
mod mode_tree_tests {
    use super::*;
    use crate::src::shared::sort::sort_criteria;

    #[test]
    fn child_owners_preserve_search_order_and_survive_removal() {
        unsafe {
            let mtd = mode_tree_alloc_data();
            let add = move |parent, tag, name: &CStr| {
                mode_tree_add(
                    mtd,
                    parent,
                    std::ptr::null_mut(),
                    tag,
                    name.as_ptr(),
                    std::ptr::null(),
                    1,
                )
            };
            let first = add(std::ptr::null_mut(), 1, c"match first");
            let branch = add(std::ptr::null_mut(), 2, c"branch");
            let child = add(branch, 3, c"match child");
            let tail = add(std::ptr::null_mut(), 4, c"match tail");
            // Force growth while existing row pointers remain live.
            for tag in 5..100 {
                add(std::ptr::null_mut(), tag, c"other");
            }
            (*branch).expanded = 0;
            mode_tree_build_lines(mtd, &raw mut (*mtd).children, 0);
            (*mtd).search = Some(c"match".to_owned());
            (*mtd).current = (*first).line;
            assert_eq!(mode_tree_search_forward(mtd), child);
            assert_eq!(mode_tree_search_backward(mtd), tail);
            (*mtd).current = (*tail).line;
            assert_eq!(mode_tree_search_backward(mtd), child);
            assert_eq!(mode_tree_search_forward(mtd), first);
            (*mtd).search = Some(c"missing".to_owned());
            assert!(mode_tree_search_forward(mtd).is_null());
            assert!(mode_tree_search_backward(mtd).is_null());

            mode_tree_remove(mtd, child);
            assert!((*branch).children.items.is_empty());
            add(branch, 100, c"replacement child");
            mode_tree_remove(mtd, branch);
            mode_tree_remove(mtd, first);
            let last = (*mtd).children.last();
            mode_tree_remove(mtd, last);
            assert_eq!((*mtd).children.first(), tail);
            assert!(mode_tree_find_item(&raw mut (*mtd).children, 100).is_null());
            mode_tree_clear_lines(mtd);
            mode_tree_build_lines(mtd, &raw mut (*mtd).children, 0);
            assert_eq!((*mtd).lines.len(), 95);
            assert_eq!((*mtd).lines.last().unwrap().last, 1);
            mode_tree_remove_ref(mtd);
        }
    }

    #[test]
    fn item_key_label_tracks_repeated_line_builds() {
        unsafe {
            let mtd = mode_tree_alloc_data();
            let mut key = b'x' as key_code;
            let key_ptr = &raw const key;
            (*mtd).keycb = Some(Box::new(move |_, _| *key_ptr));
            let item = mode_tree_add(
                mtd,
                ::core::ptr::null_mut(),
                ::core::ptr::null_mut(),
                1,
                c"row".as_ptr(),
                ::core::ptr::null(),
                1,
            );
            for (next, expected) in [
                (b'x' as key_code, Some(b"x".as_slice())),
                (KEYC_NONE, None),
                (b'y' as key_code, Some(b"y".as_slice())),
            ] {
                key = next;
                mode_tree_clear_lines(mtd);
                mode_tree_build_lines(mtd, &raw mut (*mtd).children, 0);
                match expected {
                    Some(expected) => {
                        assert_eq!(
                            CStr::from_ptr(
                                ((*item).keystr)
                                    .as_ref()
                                    .map_or(::core::ptr::null_mut(), |value| value
                                        .as_ptr()
                                        .cast_mut())
                            )
                            .to_bytes(),
                            expected
                        );
                        assert_eq!((*item).keylen, expected.len());
                    }
                    None => {
                        assert!((*item).keystr.is_none());
                        assert_eq!((*item).keylen, 0);
                    }
                }
            }
            mode_tree_free_items(&raw mut (*mtd).children);
            mode_tree_clear_lines(mtd);
            mode_tree_remove_ref(mtd);
        }
    }

    struct NestedBuildState {
        mtd: *mut mode_tree_data,
        empty: bool,
    }

    unsafe fn nested_build(state: &mut NestedBuildState) {
        if state.empty {
            return;
        }
        let parent = mode_tree_add(
            state.mtd,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            1,
            c"parent".as_ptr(),
            std::ptr::null(),
            1,
        );
        for id in 0..64 {
            mode_tree_add(
                state.mtd,
                parent,
                std::ptr::null_mut(),
                id as u64 + 2,
                c"child".as_ptr(),
                std::ptr::null(),
                1,
            );
        }
    }

    #[test]
    fn nested_lines_survive_growth_and_clear_on_empty_rebuild() {
        unsafe {
            let mtd = mode_tree_alloc_data();
            (*mtd).preview = MODE_TREE_PREVIEW_OFF as ::core::ffi::c_int;
            let state = Box::into_raw(Box::new(NestedBuildState { mtd, empty: false }));
            (*mtd).buildcb = Some(Box::new(move |_, _, _| {
                nested_build(&mut *state);
                None
            }));
            (*mtd).screen.grid = Some(crate::src::grid::grid_create(80, 24, 0));

            mode_tree_build(mtd);
            assert_eq!((*mtd).lines.len(), 65);
            assert_eq!((&(*mtd).lines)[0].depth, 0);
            for i in 1..65 {
                assert_eq!((&(*mtd).lines)[i].depth, 1);
                assert_eq!((*(&(*mtd).lines)[i].item).line, i as u_int);
            }
            assert_eq!(mode_tree_set_current(mtd, 65), 1);
            assert_eq!((*mtd).current, 64);

            (*state).empty = true;
            mode_tree_build(mtd);
            assert!((*mtd).lines.is_empty());
            mode_tree_free_items(&raw mut (*mtd).children);
            drop((*mtd).screen.grid.take());
            mode_tree_remove_ref(mtd);
            drop(Box::from_raw(state));
        }
    }

    #[test]
    fn numeric_tags_restore_saved_row_state() {
        unsafe {
            let mtd = mode_tree_alloc_data();
            let prior = mode_tree_add(
                mtd,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                7,
                c"row".as_ptr(),
                std::ptr::null(),
                1,
            );
            (*prior).tagged = 1;
            (*prior).expanded = 0;
            (*mtd).saved = std::mem::take(&mut (*mtd).children);

            let restored = mode_tree_add(
                mtd,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                7,
                c"row".as_ptr(),
                std::ptr::null(),
                1,
            );
            let different = mode_tree_add(
                mtd,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                8,
                c"other".as_ptr(),
                std::ptr::null(),
                1,
            );
            assert_eq!(((*restored).tagged, (*restored).expanded), (1, 0));
            assert_eq!(((*different).tagged, (*different).expanded), (0, 1));

            mode_tree_free_items(&raw mut (*mtd).children);
            mode_tree_free_items(&raw mut (*mtd).saved);
            mode_tree_remove_ref(mtd);
        }
    }
}
