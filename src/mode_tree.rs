use crate::src::arguments::{args_get, args_has};
use crate::src::cmd::{cmd_mouse_at, cmd_template_replace};
use crate::src::cmd_parse::cmd_parse_and_append;
use crate::src::cmd_queue::{
    cmdq_append, cmdq_free_state, cmdq_get_callback1, cmdq_get_client, cmdq_new_state,
};
use crate::src::ffi::libc::{
    __ctype_tolower_loc, __ctype_toupper_loc, free, memcpy, memset, strcasestr, strlen, strstr,
};
use crate::src::format::{format_add, format_create_defaults, format_expand, format_free};
use crate::src::format_draw::{format_draw, format_width};
use crate::src::grid::grid_default_cell;
use crate::src::key_string::key_string_format;
use crate::src::log::log_debug;
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
pub use crate::src::shared::abi::__int32_t;
use crate::src::shared::abi::*;
pub use crate::src::shared::arguments::args;
use crate::src::shared::client::*;
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
use crate::src::shared::colour::*;
pub use crate::src::shared::colour::{
    colour_theme, COLOUR_FLAG_THEME, COLOUR_THEME_BLACK, COLOUR_THEME_BLUE, COLOUR_THEME_CYAN,
    COLOUR_THEME_DARK_GREY, COLOUR_THEME_GREEN, COLOUR_THEME_LIGHT_GREY, COLOUR_THEME_MAGENTA,
    COLOUR_THEME_RED, COLOUR_THEME_WHITE, COLOUR_THEME_YELLOW,
};
pub use crate::src::shared::command::cmd_parse_input;
use crate::src::shared::command::*;
use crate::src::shared::command::*;
pub use crate::src::shared::command::{
    cmd_find_state, cmd_list, cmdq_cb, cmdq_item, cmdq_list, cmdq_state, cmds,
};
pub use crate::src::shared::control::control_state;
use crate::src::shared::display::*;
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::environment::environ;
use crate::src::shared::event::*;
pub use crate::src::shared::format::{format_job_tree, format_tree};
use crate::src::shared::grid::*;
pub use crate::src::shared::hyperlinks::hyperlinks;
pub use crate::src::shared::input::{input_ctx, input_request, input_requests};
use crate::src::shared::key::*;
pub use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
pub use crate::src::shared::layout::layout_geometry;
use crate::src::shared::layout::*;
use crate::src::shared::layout::*;
pub use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
pub use crate::src::shared::menu::{menu, menu_item};
pub use crate::src::shared::menu::{menu_choice_cb, menu_data};
use crate::src::shared::message::*;
pub use crate::src::shared::mode_tree::{
    mode_tree_build_cb, mode_tree_data, mode_tree_draw_cb, mode_tree_each_cb, mode_tree_height_cb,
    mode_tree_help_cb, mode_tree_item, mode_tree_item_entry, mode_tree_key_cb, mode_tree_line,
    mode_tree_list, mode_tree_menu_cb, mode_tree_prompt, mode_tree_prompt_input_cb,
    mode_tree_search_cb, mode_tree_search_dir, mode_tree_sort_cb, mode_tree_swap_cb,
    ModeTreeIdentity,
};
pub use crate::src::shared::mouse::{
    mouse_event, MOUSE_BUTTON_1, MOUSE_MASK_BUTTONS, MOUSE_MASK_DRAG,
};
pub use crate::src::shared::options::options;
pub use crate::src::shared::pane::{
    window_pane, window_pane_entry, window_pane_modes, window_pane_prompt, window_pane_sentry,
    window_pane_tree_entry, window_pane_zentry, window_panes,
};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
    PANE_REDRAW,
};
pub use crate::src::shared::process::tmuxpeer;
pub use crate::src::shared::prompt::prompt;
use crate::src::shared::prompt::*;
pub use crate::src::shared::prompt::{prompt_create_data, prompt_draw_data};
pub use crate::src::shared::prompt::{
    prompt_free_cb, prompt_input_cb, prompt_result, PROMPT_ACCEPT, PROMPT_CLOSE, PROMPT_CONTINUE,
    PROMPT_ISMODE, PROMPT_NOFORMAT, PROMPT_SINGLE,
};
pub use crate::src::shared::redraw::redraw_scene;
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles, MODE_CURSOR};
pub use crate::src::shared::screen_write::{screen_write_citem, screen_write_cline};
pub use crate::src::shared::screen_write::{screen_write_ctx, screen_write_init_ctx_cb};
pub use crate::src::shared::session::{session, session_entry, session_gentry};
pub use crate::src::shared::sort::sort_criteria;
pub use crate::src::shared::spawn::spawn_editor_state;
pub use crate::src::shared::status::status_line;
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
pub use crate::src::shared::tty::{
    tty, tty_code, tty_ctx, tty_ctx_c2rust_unnamed, tty_ctx_c2rust_unnamed_data,
    tty_ctx_c2rust_unnamed_sel, tty_ctx_redraw_cb, tty_ctx_set_client_cb, tty_key, tty_style_ctx,
    tty_term, tty_term_entry,
};
pub use crate::src::shared::window::WINDOW_ZOOMED;
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
use crate::src::sort::{sort_next_order, sort_order_from_string, sort_order_to_string};
use crate::src::status::status_message_set;
use crate::src::style::style_apply;
use crate::src::tmux::global_s_options;
use crate::src::window::window_zoom;
use crate::src::xmalloc::{xasprintf, xstrdup};
use std::ffi::{CStr, CString};

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

pub const MODE_TREE_SEARCH_BACKWARD: mode_tree_search_dir = 1;
pub const MODE_TREE_SEARCH_FORWARD: mode_tree_search_dir = 0;

pub use crate::src::shared::key::key_code_enum as C2RustUnnamed_39;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct mode_tree_menu {
    pub data: *mut mode_tree_data,
    pub c: *mut client,
    pub line: u_int,
}

// The C-facing item stays at offset zero because its intrusive links and
// callbacks retain its address until mode_tree_free_item removes it.
#[repr(C)]
struct ModeTreeItemOwner {
    item: mode_tree_item,
    identity_name: Option<CString>,
    identity_detail: Option<CString>,
    name: CString,
    text: Option<CString>,
    keystr: Option<CString>,
}
const _: () = assert!(std::mem::offset_of!(ModeTreeItemOwner, item) == 0);

impl ModeTreeItemOwner {
    unsafe fn from_item(item: *mut mode_tree_item) -> *mut Self {
        item.cast()
    }

    fn set_keystr(&mut self, keystr: Option<CString>) {
        self.keystr = keystr;
        self.item.keystr = self
            .keystr
            .as_ref()
            .map_or(::core::ptr::null(), |s| s.as_ptr());
        self.item.keylen = self.keystr.as_ref().map_or(0, |s| s.as_bytes().len());
    }
}
pub type mode_tree_preview = ::core::ffi::c_uint;
pub const MODE_TREE_PREVIEW_BIG: mode_tree_preview = 2;
pub const MODE_TREE_PREVIEW_NORMAL: mode_tree_preview = 1;
pub const MODE_TREE_PREVIEW_OFF: mode_tree_preview = 0;
#[inline]
unsafe extern "C" fn tolower(mut __c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    return if __c >= -(128 as ::core::ffi::c_int) && __c < 256 as ::core::ffi::c_int {
        *(*__ctype_tolower_loc()).offset(__c as isize) as ::core::ffi::c_int
    } else {
        __c
    };
}
#[inline]
unsafe extern "C" fn toupper(mut __c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    return if __c >= -(128 as ::core::ffi::c_int) && __c < 256 as ::core::ffi::c_int {
        *(*__ctype_toupper_loc()).offset(__c as isize) as ::core::ffi::c_int
    } else {
        __c
    };
}
pub const UINT64_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;

#[repr(C)]
struct ModeTreeOwner {
    // Exported mode-tree callbacks receive a pointer to this prefix.
    data: mode_tree_data,
    search: Option<CString>,
    filter: Option<CString>,
}
const _: () = assert!(::core::mem::offset_of!(ModeTreeOwner, data) == 0);

unsafe fn mode_tree_set_search(mtd: *mut mode_tree_data, search: Option<CString>) {
    let owner = mtd.cast::<ModeTreeOwner>();
    (*owner).search = search;
    (*mtd).search = (*owner)
        .search
        .as_ref()
        .map_or(::core::ptr::null_mut(), |s| s.as_ptr().cast_mut());
}

unsafe fn mode_tree_set_filter(mtd: *mut mode_tree_data, filter: Option<CString>) {
    let owner = mtd.cast::<ModeTreeOwner>();
    (*owner).filter = filter;
    (*mtd).filter = (*owner)
        .filter
        .as_ref()
        .map_or(::core::ptr::null_mut(), |s| s.as_ptr().cast_mut());
}
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
static mut mode_tree_help_start: [*const ::core::ffi::c_char; 21] = [
    b"#[fg=themelightgrey]      Up, k #[#{E:tree-mode-border-style},acs]x#[default] Move cursor up\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]    Down, j #[#{E:tree-mode-border-style},acs]x#[default] Move cursor down\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]          g #[#{E:tree-mode-border-style},acs]x#[default] Go to top\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]          G #[#{E:tree-mode-border-style},acs]x#[default] Go to bottom\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey] PPage, C-b #[#{E:tree-mode-border-style},acs]x#[default] Page up\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey] NPage, C-f #[#{E:tree-mode-border-style},acs]x#[default] Page down\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]    Left, h #[#{E:tree-mode-border-style},acs]x#[default] Collapse %1\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]   Right, l #[#{E:tree-mode-border-style},acs]x#[default] Expand %1\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]        M-- #[#{E:tree-mode-border-style},acs]x#[default] Collapse all %1s\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]        M-+ #[#{E:tree-mode-border-style},acs]x#[default] Expand all %1s\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]          t #[#{E:tree-mode-border-style},acs]x#[default] Toggle %1 tag\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]          T #[#{E:tree-mode-border-style},acs]x#[default] Untag all %1s\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]        C-t #[#{E:tree-mode-border-style},acs]x#[default] Tag all %1s\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]        C-s #[#{E:tree-mode-border-style},acs]x#[default] Search forward\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]          n #[#{E:tree-mode-border-style},acs]x#[default] Repeat search forward\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]          N #[#{E:tree-mode-border-style},acs]x#[default] Repeat search backward\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]          f #[#{E:tree-mode-border-style},acs]x#[default] Filter %1s\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]          O #[#{E:tree-mode-border-style},acs]x#[default] Change sort order\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]          r #[#{E:tree-mode-border-style},acs]x#[default] Reverse sort order\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]          v #[#{E:tree-mode-border-style},acs]x#[default] Toggle preview\0"
        as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut mode_tree_help_end: [*const ::core::ffi::c_char; 2] = [
    b"#[fg=themelightgrey]  q, Escape #[#{E:tree-mode-border-style},acs]x#[default] Exit mode\0"
        as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
pub const MODE_TREE_HELP_DEFAULT_WIDTH: ::core::ffi::c_int = 39 as ::core::ffi::c_int;
unsafe extern "C" fn mode_tree_is_lowercase(
    mut ptr: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
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
unsafe extern "C" fn mode_tree_find_item(
    mut mtl: *mut mode_tree_list,
    identity: ModeTreeIdentity,
) -> *mut mode_tree_item {
    let mut mti: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut child: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    mti = (*mtl).tqh_first;
    while !mti.is_null() {
        if (*mti).identity == identity {
            return mti;
        }
        child = mode_tree_find_item(&raw mut (*mti).children, identity);
        if !child.is_null() {
            return child;
        }
        mti = (*mti).entry.tqe_next;
    }
    return ::core::ptr::null_mut::<mode_tree_item>();
}
unsafe extern "C" fn mode_tree_free_item(mut mti: *mut mode_tree_item) {
    mode_tree_free_items(&raw mut (*mti).children);
    drop(Box::from_raw(ModeTreeItemOwner::from_item(mti)));
}
unsafe extern "C" fn mode_tree_free_items(mut mtl: *mut mode_tree_list) {
    let mut mti: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut mti1: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    mti = (*mtl).tqh_first;
    while !mti.is_null() && {
        mti1 = (*mti).entry.tqe_next;
        1 as ::core::ffi::c_int != 0
    } {
        if !(*mti).entry.tqe_next.is_null() {
            (*(*mti).entry.tqe_next).entry.tqe_prev = (*mti).entry.tqe_prev;
        } else {
            (*mtl).tqh_last = (*mti).entry.tqe_prev;
        }
        *(*mti).entry.tqe_prev = (*mti).entry.tqe_next;
        mode_tree_free_item(mti);
        mti = mti1;
    }
}
unsafe extern "C" fn mode_tree_check_selected(mut mtd: *mut mode_tree_data) {
    if (*mtd).current > (*mtd).height.wrapping_sub(1 as u_int) {
        (*mtd).offset = (*mtd)
            .current
            .wrapping_sub((*mtd).height)
            .wrapping_add(1 as u_int);
    }
}
unsafe fn mode_tree_alloc_data() -> *mut mode_tree_data {
    let mut allocation = Box::<ModeTreeOwner>::new_uninit();
    let owner = allocation.as_mut_ptr();
    let mtd = &raw mut (*owner).data;
    // Keep the translated C zero state. Initialize every Rust owner before
    // treating this allocation as a ModeTreeOwner value.
    owner.write_bytes(0, 1);
    ::core::ptr::write(&raw mut (*mtd).lines, Vec::new());
    ::core::ptr::write(&raw mut (*owner).search, None);
    ::core::ptr::write(&raw mut (*owner).filter, None);
    let mtd = Box::into_raw(allocation.assume_init()).cast::<mode_tree_data>();
    (*mtd).references = 1;
    mtd
}

#[inline]
unsafe fn mode_tree_line_count(mtd: *mut mode_tree_data) -> u_int {
    (*mtd).lines.len() as u_int
}

unsafe extern "C" fn mode_tree_clear_lines(mut mtd: *mut mode_tree_data) {
    (*mtd).lines = Vec::new();
}
unsafe extern "C" fn mode_tree_build_lines(
    mut mtd: *mut mode_tree_data,
    mut mtl: *mut mode_tree_list,
    mut depth: u_int,
) {
    let mut mti: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut line: *mut mode_tree_line = ::core::ptr::null_mut::<mode_tree_line>();
    let mut i: u_int = 0;
    let mut flat: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    (*mtd).depth = depth;
    if depth > (*mtd).maxdepth {
        (*mtd).maxdepth = depth;
    }
    mti = (*mtl).tqh_first;
    while !mti.is_null() {
        (*mtd).lines.push(mode_tree_line {
            item: mti,
            depth,
            last: (mti == *(*((*mtl).tqh_last as *mut mode_tree_list)).tqh_last)
                as ::core::ffi::c_int,
            flat: 0,
        });
        (*mti).line = mode_tree_line_count(mtd).wrapping_sub(1 as u_int);
        if !(*mti).children.tqh_first.is_null() {
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
            (*mti).key = (*mtd).keycb.expect("non-null function pointer")(
                (*mtd).modedata,
                (*mti).itemdata,
                (*mti).line,
            );
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
        (*ModeTreeItemOwner::from_item(mti)).set_keystr(keystr);
        mti = (*mti).entry.tqe_next;
    }
    mti = (*mtl).tqh_first;
    while !mti.is_null() {
        i = 0 as u_int;
        while i < mode_tree_line_count(mtd) {
            line = (*mtd).lines.as_mut_ptr().offset(i as isize) as *mut mode_tree_line
                as *mut mode_tree_line;
            if (*line).item == mti {
                (*line).flat = flat;
            }
            i = i.wrapping_add(1);
        }
        mti = (*mti).entry.tqe_next;
    }
}
unsafe extern "C" fn mode_tree_clear_tagged(mut mtl: *mut mode_tree_list) {
    let mut mti: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    mti = (*mtl).tqh_first;
    while !mti.is_null() {
        (*mti).tagged = 0 as ::core::ffi::c_int;
        mode_tree_clear_tagged(&raw mut (*mti).children);
        mti = (*mti).entry.tqe_next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn mode_tree_up(mut mtd: *mut mode_tree_data, mut wrap: ::core::ffi::c_int) {
    if mode_tree_line_count(mtd) == 0 as u_int {
        return;
    }
    if (*mtd).current == 0 as u_int {
        if wrap != 0 {
            (*mtd).current = mode_tree_line_count(mtd).wrapping_sub(1 as u_int);
            if mode_tree_line_count(mtd) >= (*mtd).height {
                (*mtd).offset = mode_tree_line_count(mtd).wrapping_sub((*mtd).height);
            }
        }
    } else {
        (*mtd).current = (*mtd).current.wrapping_sub(1);
        if (*mtd).current < (*mtd).offset {
            (*mtd).offset = (*mtd).offset.wrapping_sub(1);
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn mode_tree_down(
    mut mtd: *mut mode_tree_data,
    mut wrap: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if mode_tree_line_count(mtd) == 0 as u_int {
        return 0 as ::core::ffi::c_int;
    }
    if (*mtd).current == mode_tree_line_count(mtd).wrapping_sub(1 as u_int) {
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
unsafe extern "C" fn mode_tree_swap(
    mut mtd: *mut mode_tree_data,
    mut direction: ::core::ffi::c_int,
) {
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
            && swap_with.wrapping_add(direction as u_int) >= mode_tree_line_count(mtd)
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
    if (*mtd).swapcb.expect("non-null function pointer")(
        (*(*(*mtd).lines.as_mut_ptr().offset((*mtd).current as isize)).item).itemdata,
        (*(*(*mtd).lines.as_mut_ptr().offset(swap_with as isize)).item).itemdata,
        &raw mut (*mtd).sort_crit,
    ) != 0
    {
        (*mtd).current = swap_with;
        mode_tree_build(mtd);
    }
}
#[no_mangle]
pub unsafe extern "C" fn mode_tree_get_current(
    mut mtd: *mut mode_tree_data,
) -> *mut ::core::ffi::c_void {
    if mode_tree_line_count(mtd) == 0 as u_int {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    return (*(*(*mtd).lines.as_mut_ptr().offset((*mtd).current as isize)).item).itemdata;
}
#[no_mangle]
pub unsafe extern "C" fn mode_tree_get_current_name(
    mut mtd: *mut mode_tree_data,
) -> *const ::core::ffi::c_char {
    return (*(*(*mtd).lines.as_mut_ptr().offset((*mtd).current as isize)).item).name;
}
#[no_mangle]
pub unsafe extern "C" fn mode_tree_select_top(mut mtd: *mut mode_tree_data) {
    (*mtd).current = 0 as u_int;
    (*mtd).offset = 0 as u_int;
}
#[no_mangle]
pub unsafe extern "C" fn mode_tree_expand_current(mut mtd: *mut mode_tree_data) {
    if (*(*(*mtd).lines.as_mut_ptr().offset((*mtd).current as isize)).item).expanded == 0 {
        (*(*(*mtd).lines.as_mut_ptr().offset((*mtd).current as isize)).item).expanded =
            1 as ::core::ffi::c_int;
        mode_tree_build(mtd);
    }
}
#[no_mangle]
pub unsafe extern "C" fn mode_tree_collapse_current(mut mtd: *mut mode_tree_data) {
    if (*(*(*mtd).lines.as_mut_ptr().offset((*mtd).current as isize)).item).expanded != 0 {
        (*(*(*mtd).lines.as_mut_ptr().offset((*mtd).current as isize)).item).expanded =
            0 as ::core::ffi::c_int;
        mode_tree_build(mtd);
    }
}
unsafe extern "C" fn mode_tree_get_tag(
    mut mtd: *mut mode_tree_data,
    identity: ModeTreeIdentity,
    mut found: *mut u_int,
) -> ::core::ffi::c_int {
    let mut i: u_int = 0;
    i = 0 as u_int;
    while i < mode_tree_line_count(mtd) {
        if (*(*(*mtd).lines.as_mut_ptr().offset(i as isize)).item).identity == identity {
            break;
        }
        i = i.wrapping_add(1);
    }
    if i != mode_tree_line_count(mtd) {
        *found = i;
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn mode_tree_expand(mut mtd: *mut mode_tree_data, mut tag: uint64_t) {
    mode_tree_expand_identity(mtd, ModeTreeIdentity::legacy(tag));
}

pub unsafe fn mode_tree_expand_identity(mtd: *mut mode_tree_data, identity: ModeTreeIdentity) {
    let mut found: u_int = 0;
    if mode_tree_get_tag(mtd, identity, &raw mut found) == 0 {
        return;
    }
    if (*(*(*mtd).lines.as_mut_ptr().offset(found as isize)).item).expanded == 0 {
        (*(*(*mtd).lines.as_mut_ptr().offset(found as isize)).item).expanded =
            1 as ::core::ffi::c_int;
        mode_tree_build(mtd);
    }
}
#[no_mangle]
pub unsafe extern "C" fn mode_tree_set_current(
    mut mtd: *mut mode_tree_data,
    mut tag: uint64_t,
) -> ::core::ffi::c_int {
    mode_tree_set_current_identity(mtd, ModeTreeIdentity::legacy(tag))
}

pub unsafe fn mode_tree_set_current_identity(
    mtd: *mut mode_tree_data,
    identity: ModeTreeIdentity,
) -> ::core::ffi::c_int {
    let mut found: u_int = 0;
    if mode_tree_get_tag(mtd, identity, &raw mut found) != 0 {
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
    if (*mtd).current >= mode_tree_line_count(mtd) {
        if mode_tree_line_count(mtd) == 0 as u_int {
            return 0 as ::core::ffi::c_int;
        }
        (*mtd).current = mode_tree_line_count(mtd).wrapping_sub(1 as u_int);
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
#[no_mangle]
pub unsafe extern "C" fn mode_tree_count_tagged(mut mtd: *mut mode_tree_data) -> u_int {
    let mut mti: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut i: u_int = 0;
    let mut tagged: u_int = 0;
    tagged = 0 as u_int;
    i = 0 as u_int;
    while i < mode_tree_line_count(mtd) {
        mti = (*(*mtd).lines.as_mut_ptr().offset(i as isize)).item;
        if (*mti).tagged != 0 {
            tagged = tagged.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    return tagged;
}
#[no_mangle]
pub unsafe extern "C" fn mode_tree_each_tagged(
    mut mtd: *mut mode_tree_data,
    mut cb: mode_tree_each_cb,
    mut c: *mut client,
    mut key: key_code,
    mut current: ::core::ffi::c_int,
) {
    let mut mti: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut i: u_int = 0;
    let mut fired: ::core::ffi::c_int = 0;
    fired = 0 as ::core::ffi::c_int;
    i = 0 as u_int;
    while i < mode_tree_line_count(mtd) {
        mti = (*(*mtd).lines.as_mut_ptr().offset(i as isize)).item;
        if (*mti).tagged != 0 {
            fired = 1 as ::core::ffi::c_int;
            cb.expect("non-null function pointer")((*mtd).modedata, (*mti).itemdata, c, key);
        }
        i = i.wrapping_add(1);
    }
    if fired == 0 && current != 0 {
        mti = (*(*mtd).lines.as_mut_ptr().offset((*mtd).current as isize)).item;
        cb.expect("non-null function pointer")((*mtd).modedata, (*mti).itemdata, c, key);
    }
}
#[no_mangle]
pub unsafe extern "C" fn mode_tree_start(
    mut wp: *mut window_pane,
    mut args: *mut args,
    mut buildcb: mode_tree_build_cb,
    mut drawcb: mode_tree_draw_cb,
    mut searchcb: mode_tree_search_cb,
    mut menucb: mode_tree_menu_cb,
    mut heightcb: mode_tree_height_cb,
    mut keycb: mode_tree_key_cb,
    mut swapcb: mode_tree_swap_cb,
    mut sortcb: mode_tree_sort_cb,
    mut helpcb: mode_tree_help_cb,
    mut modedata: *mut ::core::ffi::c_void,
    mut menu: *const menu_item,
    mut s: *mut *mut screen,
) -> *mut mode_tree_data {
    let mut mtd: *mut mode_tree_data = ::core::ptr::null_mut::<mode_tree_data>();
    mtd = mode_tree_alloc_data();
    (*mtd).wp = wp;
    (*mtd).modedata = modedata;
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
        mode_tree_set_filter(
            mtd,
            Some(CStr::from_ptr(args_get(args, 'f' as i32 as u_char)).to_owned()),
        );
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
    (*mtd).children.tqh_first = ::core::ptr::null_mut::<mode_tree_item>();
    (*mtd).children.tqh_last = &raw mut (*mtd).children.tqh_first;
    *s = &raw mut (*mtd).screen;
    screen_init(*s, (*(*wp).base.grid).sx, (*(*wp).base.grid).sy, 0 as u_int);
    (**s).mode &= !MODE_CURSOR;
    return mtd;
}
#[no_mangle]
pub unsafe extern "C" fn mode_tree_zoom(mut mtd: *mut mode_tree_data, mut args: *mut args) {
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
unsafe extern "C" fn mode_tree_set_height(mut mtd: *mut mode_tree_data) {
    let mut s: *mut screen = &raw mut (*mtd).screen;
    let mut height: u_int = 0;
    if (*mtd).heightcb.is_some() {
        height = (*mtd).heightcb.expect("non-null function pointer")(
            mtd as *mut ::core::ffi::c_void,
            (*(*s).grid).sy,
        );
        if height < (*(*s).grid).sy {
            (*mtd).height = (*(*s).grid).sy.wrapping_sub(height);
        }
    } else if (*mtd).preview == MODE_TREE_PREVIEW_NORMAL as ::core::ffi::c_int {
        (*mtd).height = (*(*s).grid)
            .sy
            .wrapping_div(3 as u_int)
            .wrapping_mul(2 as u_int);
        if (*mtd).height > mode_tree_line_count(mtd) {
            (*mtd).height = (*(*s).grid).sy.wrapping_div(2 as u_int);
        }
        if (*mtd).height < 10 as u_int {
            (*mtd).height = (*(*s).grid).sy;
        }
    } else if (*mtd).preview == MODE_TREE_PREVIEW_BIG as ::core::ffi::c_int {
        (*mtd).height = (*(*s).grid).sy.wrapping_div(4 as u_int);
        if (*mtd).height > mode_tree_line_count(mtd) {
            (*mtd).height = mode_tree_line_count(mtd);
        }
        if (*mtd).height < 2 as u_int {
            (*mtd).height = 2 as u_int;
        }
    } else {
        (*mtd).height = (*(*s).grid).sy;
    }
    if (*(*s).grid).sy.wrapping_sub((*mtd).height) < 2 as u_int {
        (*mtd).height = (*(*s).grid).sy;
    }
}
#[no_mangle]
pub unsafe extern "C" fn mode_tree_build(mut mtd: *mut mode_tree_data) {
    let mut s: *mut screen = &raw mut (*mtd).screen;
    let mut tag: uint64_t = 0;
    let mut identity = ModeTreeIdentity::legacy(UINT64_MAX as uint64_t);
    if !(*mtd).lines.is_empty() {
        identity = (*(*(*mtd).lines.as_mut_ptr().offset((*mtd).current as isize)).item).identity;
        // The callback tag is only used by legacy modes. A typed identity is
        // retained separately and never narrowed to a numeric tag.
        if identity.kind == 0 {
            tag = identity.second;
        } else {
            tag = UINT64_MAX as uint64_t;
        }
    } else {
        tag = UINT64_MAX as uint64_t;
    }
    (*mtd).has_build_identity = 0;
    if !(*mtd).children.tqh_first.is_null() {
        *(*mtd).saved.tqh_last = (*mtd).children.tqh_first;
        (*(*mtd).children.tqh_first).entry.tqe_prev = (*mtd).saved.tqh_last;
        (*mtd).saved.tqh_last = (*mtd).children.tqh_last;
        (*mtd).children.tqh_first = ::core::ptr::null_mut::<mode_tree_item>();
        (*mtd).children.tqh_last = &raw mut (*mtd).children.tqh_first;
    }
    (*mtd).children.tqh_first = ::core::ptr::null_mut::<mode_tree_item>();
    (*mtd).children.tqh_last = &raw mut (*mtd).children.tqh_first;
    if (*mtd).sortcb.is_some() {
        (*mtd).sortcb.expect("non-null function pointer")(&raw mut (*mtd).sort_crit);
    }
    (*mtd).buildcb.expect("non-null function pointer")(
        (*mtd).modedata,
        &raw mut (*mtd).sort_crit,
        &raw mut tag,
        (*mtd).filter,
    );
    (*mtd).no_matches = ((*mtd).children.tqh_first
        == ::core::ptr::null_mut::<::core::ffi::c_void>() as *mut mode_tree_item)
        as ::core::ffi::c_int;
    if (*mtd).no_matches != 0 {
        (*mtd).buildcb.expect("non-null function pointer")(
            (*mtd).modedata,
            &raw mut (*mtd).sort_crit,
            &raw mut tag,
            ::core::ptr::null::<::core::ffi::c_char>(),
        );
    }
    if (*mtd).has_build_identity != 0 {
        identity = (*mtd).build_identity;
    } else if tag != UINT64_MAX as uint64_t {
        identity = ModeTreeIdentity::legacy(tag);
    }
    // The old rows are freed below. Keep their string key alive through the
    // selection lookup, including when a build callback chooses a new row.
    let name = (!identity.name.is_null()).then(|| CStr::from_ptr(identity.name).to_owned());
    let detail = (!identity.detail.is_null()).then(|| CStr::from_ptr(identity.detail).to_owned());
    identity.name = name.as_ref().map_or(::core::ptr::null(), |s| s.as_ptr());
    identity.detail = detail.as_ref().map_or(::core::ptr::null(), |s| s.as_ptr());
    mode_tree_free_items(&raw mut (*mtd).saved);
    (*mtd).saved.tqh_first = ::core::ptr::null_mut::<mode_tree_item>();
    (*mtd).saved.tqh_last = &raw mut (*mtd).saved.tqh_first;
    mode_tree_clear_lines(mtd);
    (*mtd).maxdepth = 0 as u_int;
    mode_tree_build_lines(mtd, &raw mut (*mtd).children, 0 as u_int);
    if !(*mtd).lines.is_empty() && tag == UINT64_MAX as uint64_t {
        if identity == ModeTreeIdentity::legacy(UINT64_MAX as uint64_t) {
            identity =
                (*(*(*mtd).lines.as_mut_ptr().offset((*mtd).current as isize)).item).identity;
        }
    }
    mode_tree_set_current_identity(mtd, identity);
    (*mtd).width = (*(*s).grid).sx;
    if (*mtd).preview != MODE_TREE_PREVIEW_OFF as ::core::ffi::c_int {
        mode_tree_set_height(mtd);
    } else {
        (*mtd).height = (*(*s).grid).sy;
    }
    mode_tree_check_selected(mtd);
}

pub unsafe fn mode_tree_set_build_identity(mtd: *mut mode_tree_data, identity: ModeTreeIdentity) {
    (*mtd).build_identity = identity;
    (*mtd).has_build_identity = 1;
}
unsafe extern "C" fn mode_tree_remove_ref(mut mtd: *mut mode_tree_data) {
    (*mtd).references = (*mtd).references.wrapping_sub(1);
    if (*mtd).references == 0 as u_int {
        drop(Box::from_raw(mtd.cast::<ModeTreeOwner>()));
    }
}
#[no_mangle]
pub unsafe extern "C" fn mode_tree_free(mut mtd: *mut mode_tree_data) {
    let mut wp: *mut window_pane = (*mtd).wp;
    if (*mtd).zoomed == 0 as ::core::ffi::c_int {
        server_unzoom_window((*wp).window as *mut window);
    }
    mode_tree_clear_prompt(mtd);
    mode_tree_free_items(&raw mut (*mtd).children);
    mode_tree_clear_lines(mtd);
    screen_free(&raw mut (*mtd).screen);
    mode_tree_set_search(mtd, None);
    mode_tree_set_filter(mtd, None);
    (*mtd).dead = 1 as ::core::ffi::c_int;
    mode_tree_remove_ref(mtd);
}
#[no_mangle]
pub unsafe extern "C" fn mode_tree_resize(
    mut mtd: *mut mode_tree_data,
    mut sx: u_int,
    mut sy: u_int,
) {
    let mut s: *mut screen = &raw mut (*mtd).screen;
    screen_resize(s, sx, sy, 0 as ::core::ffi::c_int);
    mode_tree_build(mtd);
    mode_tree_draw(mtd);
    (*(*mtd).wp).flags |= PANE_REDRAW;
}
#[no_mangle]
pub unsafe extern "C" fn mode_tree_add(
    mut mtd: *mut mode_tree_data,
    mut parent: *mut mode_tree_item,
    mut itemdata: *mut ::core::ffi::c_void,
    mut tag: uint64_t,
    mut name: *const ::core::ffi::c_char,
    mut text: *const ::core::ffi::c_char,
    mut expanded: ::core::ffi::c_int,
) -> *mut mode_tree_item {
    mode_tree_add_identity(
        mtd,
        parent,
        itemdata,
        ModeTreeIdentity::legacy(tag),
        name,
        text,
        expanded,
    )
}

pub unsafe fn mode_tree_add_identity(
    mtd: *mut mode_tree_data,
    parent: *mut mode_tree_item,
    itemdata: *mut ::core::ffi::c_void,
    identity: ModeTreeIdentity,
    name: *const ::core::ffi::c_char,
    text: *const ::core::ffi::c_char,
    expanded: ::core::ffi::c_int,
) -> *mut mode_tree_item {
    let mut mti: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut saved: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    if identity.kind == 0 {
        log_debug(
            b"%s: %llu, %s %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"mode_tree_add\0" as *const u8 as *const ::core::ffi::c_char,
            identity.second as ::core::ffi::c_ulonglong,
            name,
            if text.is_null() {
                b"\0" as *const u8 as *const ::core::ffi::c_char
            } else {
                text
            },
        );
    } else {
        log_debug(
            b"%s: %llu:%llu:%llu, %s %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"mode_tree_add\0" as *const u8 as *const ::core::ffi::c_char,
            identity.kind as ::core::ffi::c_ulonglong,
            identity.first as ::core::ffi::c_ulonglong,
            identity.second as ::core::ffi::c_ulonglong,
            name,
            if text.is_null() {
                b"\0" as *const u8 as *const ::core::ffi::c_char
            } else {
                text
            },
        );
    }
    let identity_name =
        (!identity.name.is_null()).then(|| CStr::from_ptr(identity.name).to_owned());
    let identity_detail =
        (!identity.detail.is_null()).then(|| CStr::from_ptr(identity.detail).to_owned());
    let name = CStr::from_ptr(name).to_owned();
    let text = (!text.is_null()).then(|| CStr::from_ptr(text).to_owned());
    let mut owner = Box::new(ModeTreeItemOwner {
        item: ::core::mem::zeroed(),
        identity_name,
        identity_detail,
        name,
        text,
        keystr: None,
    });
    owner.item.identity = identity;
    owner.item.identity.name = owner
        .identity_name
        .as_ref()
        .map_or(::core::ptr::null(), |s| s.as_ptr());
    owner.item.identity.detail = owner
        .identity_detail
        .as_ref()
        .map_or(::core::ptr::null(), |s| s.as_ptr());
    owner.item.name = owner.name.as_ptr();
    owner.item.text = owner
        .text
        .as_ref()
        .map_or(::core::ptr::null(), |s| s.as_ptr());
    mti = Box::into_raw(owner).cast::<mode_tree_item>();
    (*mti).parent = parent;
    (*mti).itemdata = itemdata;
    saved = mode_tree_find_item(&raw mut (*mtd).saved, identity);
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
    (*mti).children.tqh_first = ::core::ptr::null_mut::<mode_tree_item>();
    (*mti).children.tqh_last = &raw mut (*mti).children.tqh_first;
    if !parent.is_null() {
        (*mti).entry.tqe_next = ::core::ptr::null_mut::<mode_tree_item>();
        (*mti).entry.tqe_prev = (*parent).children.tqh_last;
        *(*parent).children.tqh_last = mti;
        (*parent).children.tqh_last = &raw mut (*mti).entry.tqe_next;
    } else {
        (*mti).entry.tqe_next = ::core::ptr::null_mut::<mode_tree_item>();
        (*mti).entry.tqe_prev = (*mtd).children.tqh_last;
        *(*mtd).children.tqh_last = mti;
        (*mtd).children.tqh_last = &raw mut (*mti).entry.tqe_next;
    }
    return mti as *mut mode_tree_item;
}
#[no_mangle]
pub unsafe extern "C" fn mode_tree_view_name(
    mut mtd: *mut mode_tree_data,
    mut name: *const ::core::ffi::c_char,
) {
    (*mtd).view_name = name;
}
#[no_mangle]
pub unsafe extern "C" fn mode_tree_draw_as_parent(mut mti: *mut mode_tree_item) {
    (*mti).draw_as_parent = 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn mode_tree_no_tag(mut mti: *mut mode_tree_item) {
    (*mti).no_tag = 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn mode_tree_align(
    mut mti: *mut mode_tree_item,
    mut align: ::core::ffi::c_int,
) {
    (*mti).align = align;
}
#[no_mangle]
pub unsafe extern "C" fn mode_tree_remove(
    mut mtd: *mut mode_tree_data,
    mut mti: *mut mode_tree_item,
) {
    let mut parent: *mut mode_tree_item = (*mti).parent;
    if !parent.is_null() {
        if !(*mti).entry.tqe_next.is_null() {
            (*(*mti).entry.tqe_next).entry.tqe_prev = (*mti).entry.tqe_prev;
        } else {
            (*parent).children.tqh_last = (*mti).entry.tqe_prev;
        }
        *(*mti).entry.tqe_prev = (*mti).entry.tqe_next;
    } else {
        if !(*mti).entry.tqe_next.is_null() {
            (*(*mti).entry.tqe_next).entry.tqe_prev = (*mti).entry.tqe_prev;
        } else {
            (*mtd).children.tqh_last = (*mti).entry.tqe_prev;
        }
        *(*mti).entry.tqe_prev = (*mti).entry.tqe_next;
    }
    mode_tree_free_item(mti);
}
unsafe fn mode_tree_append_printf_string(bytes: &mut Vec<u8>, value: *const ::core::ffi::c_char) {
    if value.is_null() {
        bytes.extend_from_slice(b"(null)");
    } else {
        bytes.extend_from_slice(CStr::from_ptr(value).to_bytes());
    }
}

#[no_mangle]
pub unsafe extern "C" fn mode_tree_draw(mut mtd: *mut mode_tree_data) {
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
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        item: ::core::ptr::null_mut::<screen_write_citem>(),
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
    let mut prefix: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut tag: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut separator: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut n: size_t = 0;
    let mut keylen: ::core::ffi::c_int = 0;
    let vla = (*mtd).maxdepth.wrapping_add(1 as u_int) as usize;
    let mut alignlen: Vec<::core::ffi::c_int> = ::std::vec::from_elem(0, vla);
    let mut dfg: ::core::ffi::c_int = 0;
    let mut dfg0: ::core::ffi::c_int = 0;
    if mode_tree_line_count(mtd) == 0 as u_int {
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
    while i < mode_tree_line_count(mtd) {
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
    while i < mode_tree_line_count(mtd) {
        line = (*mtd).lines.as_mut_ptr().offset(i as isize) as *mut mode_tree_line
            as *mut mode_tree_line;
        mti = (*line).item;
        if (*mti).align != 0
            && strlen((*mti).name) as ::core::ffi::c_int
                > *alignlen.as_mut_ptr().offset((*line).depth as isize)
        {
            *alignlen.as_mut_ptr().offset((*line).depth as isize) =
                strlen((*mti).name) as ::core::ffi::c_int;
        }
        i = i.wrapping_add(1);
    }
    i = 0 as u_int;
    while i < mode_tree_line_count(mtd) {
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
                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                    (*mti).keystr,
                );
            } else {
                format_add(
                    ft,
                    b"mode_tree_key\0" as *const u8 as *const ::core::ffi::c_char,
                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            format_add(
                ft,
                b"mode_tree_key_width\0" as *const u8 as *const ::core::ffi::c_char,
                b"%d\0" as *const u8 as *const ::core::ffi::c_char,
                keylen,
            );
            format_add(
                ft,
                b"mode_tree_selected\0" as *const u8 as *const ::core::ffi::c_char,
                b"%d\0" as *const u8 as *const ::core::ffi::c_char,
                (i == (*mtd).current) as ::core::ffi::c_int,
            );
            if (*line).depth == 0 as u_int {
                format_add(
                    ft,
                    b"mode_tree_repeat\0" as *const u8 as *const ::core::ffi::c_char,
                    b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                    0 as ::core::ffi::c_int,
                );
                format_add(
                    ft,
                    b"mode_tree_branch\0" as *const u8 as *const ::core::ffi::c_char,
                    b"0\0" as *const u8 as *const ::core::ffi::c_char,
                );
                format_add(
                    ft,
                    b"mode_tree_parent_last\0" as *const u8 as *const ::core::ffi::c_char,
                    b"0\0" as *const u8 as *const ::core::ffi::c_char,
                );
            } else {
                format_add(
                    ft,
                    b"mode_tree_repeat\0" as *const u8 as *const ::core::ffi::c_char,
                    b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                    (*line).depth.wrapping_sub(1 as u_int),
                );
                format_add(
                    ft,
                    b"mode_tree_branch\0" as *const u8 as *const ::core::ffi::c_char,
                    b"1\0" as *const u8 as *const ::core::ffi::c_char,
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
                        b"1\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                } else {
                    format_add(
                        ft,
                        b"mode_tree_parent_last\0" as *const u8 as *const ::core::ffi::c_char,
                        b"0\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                }
            }
            if (*mti).children.tqh_first.is_null() {
                format_add(
                    ft,
                    b"mode_tree_has_children\0" as *const u8 as *const ::core::ffi::c_char,
                    b"0\0" as *const u8 as *const ::core::ffi::c_char,
                );
            } else {
                format_add(
                    ft,
                    b"mode_tree_has_children\0" as *const u8 as *const ::core::ffi::c_char,
                    b"1\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            format_add(
                ft,
                b"mode_tree_last\0" as *const u8 as *const ::core::ffi::c_char,
                b"%d\0" as *const u8 as *const ::core::ffi::c_char,
                (*line).last,
            );
            format_add(
                ft,
                b"mode_tree_expanded\0" as *const u8 as *const ::core::ffi::c_char,
                b"%d\0" as *const u8 as *const ::core::ffi::c_char,
                (*mti).expanded,
            );
            format_add(
                ft,
                b"mode_tree_flat\0" as *const u8 as *const ::core::ffi::c_char,
                b"%d\0" as *const u8 as *const ::core::ffi::c_char,
                (*line).flat,
            );
            prefix = format_expand(
                ft,
                b"#[fg=themelightgrey]#[bg=default]#[noacs]#{p/#{mode_tree_key_width}:#{?#{!=:#{mode_tree_key},},(#{mode_tree_key}),}}#{R:#{?mode_tree_parent_last,    ,#[acs]x#[fg=themelightgrey]#[bg=default]#[noacs]   },#{mode_tree_repeat}}#{?mode_tree_branch,#[acs]#{?mode_tree_last,mq,tq}+#[fg=themelightgrey]#[bg=default]#[noacs] ,}#{?mode_tree_has_children,#{?mode_tree_expanded,#[fg=themered]-#[fg=themelightgrey]#[bg=default]#[noacs] ,#[fg=themegreen]+#[fg=themelightgrey]#[bg=default]#[noacs] },#{?mode_tree_flat,,  }}\0"
                    as *const u8 as *const ::core::ffi::c_char,
            );
            prefix_width = format_width(prefix);
            if prefix_width > w {
                prefix_width = w;
            }
            if (*mti).tagged != 0 {
                tag = b"*\0" as *const u8 as *const ::core::ffi::c_char;
            } else {
                tag = b"\0" as *const u8 as *const ::core::ffi::c_char;
            }
            if !(*mti).text.is_null() {
                separator = b"#[fg=themelightgrey]: #[default]\0" as *const u8
                    as *const ::core::ffi::c_char;
            } else {
                separator = b"\0" as *const u8 as *const ::core::ffi::c_char;
            }
            let field_width = (*mti).align * *alignlen.as_mut_ptr().offset((*line).depth as isize);
            let mut name = Vec::new();
            mode_tree_append_printf_string(&mut name, (*mti).name);
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
                    prefix,
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
                    if !(*mti).text.is_null() && width < w {
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
                            (*mti).text,
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
                    prefix,
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
                    if !(*mti).text.is_null() && width < w {
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
                            (*mti).text,
                            ::core::ptr::null_mut::<style_ranges>(),
                            1 as ::core::ffi::c_int,
                        );
                    }
                }
            }
            drop(text);
            free(prefix as *mut ::core::ffi::c_void);
            if (*mti).tagged != 0 {
                gc.fg = dfg;
                gc0.fg = dfg0;
            }
        }
        i = i.wrapping_add(1);
    }
    format_free(ft);
    if !((*mtd).preview == MODE_TREE_PREVIEW_OFF as ::core::ffi::c_int) {
        sy = (*(*s).grid).sy;
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
            mode_tree_append_printf_string(&mut label_bytes, (*mti).name);
            if !(*mtd).sort_crit.order_seq.is_null() {
                label_bytes.extend_from_slice(b" (sort: ");
                mode_tree_append_printf_string(
                    &mut label_bytes,
                    sort_order_to_string((*mtd).sort_crit.order),
                );
                if (*mtd).sort_crit.reversed != 0 {
                    label_bytes.extend_from_slice(b", reversed");
                }
                label_bytes.push(b')');
                if !(*mtd).view_name.is_null() {
                    label_bytes.extend_from_slice(b" (view: ");
                    mode_tree_append_printf_string(&mut label_bytes, (*mtd).view_name);
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
                screen_write_puts(
                    &raw mut ctx,
                    &raw mut box_gc,
                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                    label.as_ptr(),
                );
                if (*mtd).no_matches != 0 {
                    n = (::core::mem::size_of::<[::core::ffi::c_char; 11]>() as usize)
                        .wrapping_sub(1 as usize) as size_t;
                } else {
                    n = (::core::mem::size_of::<[::core::ffi::c_char; 7]>() as usize)
                        .wrapping_sub(1 as usize) as size_t;
                }
                if !(*mtd).filter.is_null()
                    && w.wrapping_sub(2 as u_int) as size_t
                        >= label_len
                            .wrapping_add(10 as size_t)
                            .wrapping_add(n)
                            .wrapping_add(2 as size_t)
                {
                    screen_write_puts(
                        &raw mut ctx,
                        &raw mut box_gc,
                        b" (filter: \0" as *const u8 as *const ::core::ffi::c_char,
                    );
                    if (*mtd).no_matches != 0 {
                        screen_write_puts(
                            &raw mut ctx,
                            &raw mut box_gc,
                            b"no matches\0" as *const u8 as *const ::core::ffi::c_char,
                        );
                    } else {
                        screen_write_puts(
                            &raw mut ctx,
                            &raw mut box_gc,
                            b"active\0" as *const u8 as *const ::core::ffi::c_char,
                        );
                    }
                    screen_write_puts(
                        &raw mut ctx,
                        &raw mut box_gc,
                        b") \0" as *const u8 as *const ::core::ffi::c_char,
                    );
                } else {
                    screen_write_puts(
                        &raw mut ctx,
                        &raw mut box_gc,
                        b" \0" as *const u8 as *const ::core::ffi::c_char,
                    );
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
                (*mtd).drawcb.expect("non-null function pointer")(
                    (*mtd).modedata,
                    (*mti).itemdata,
                    &raw mut ctx,
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
unsafe extern "C" fn mode_tree_draw_prompt(
    mut mtd: *mut mode_tree_data,
    mut ctx: *mut screen_write_ctx,
) {
    let mut s: *mut screen = &raw mut (*mtd).screen;
    let mut pdd: prompt_draw_data = prompt_draw_data {
        ctx: ::core::ptr::null_mut::<screen_write_ctx>(),
        cursor_x: ::core::ptr::null_mut::<u_int>(),
        area_x: 0,
        area_width: 0,
        prompt_line: 0,
    };
    let mut sx: u_int = (*(*s).grid).sx;
    let mut sy: u_int = (*(*s).grid).sy;
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
#[no_mangle]
pub unsafe extern "C" fn mode_tree_clear_prompt(mut mtd: *mut mode_tree_data) {
    let mut prompt: *mut prompt = (*mtd).prompt;
    if !(*mtd).prompt.is_null() {
        (*mtd).prompt = ::core::ptr::null_mut::<prompt>();
        prompt_free(prompt);
        (*mtd).screen.mode &= !MODE_CURSOR;
    }
}
#[no_mangle]
pub unsafe extern "C" fn mode_tree_has_prompt(mut mtd: *mut mode_tree_data) -> ::core::ffi::c_int {
    return ((*mtd).prompt != NULL as *mut prompt) as ::core::ffi::c_int;
}
unsafe extern "C" fn mode_tree_prompt_accept(
    mut item: *mut cmdq_item,
    mut data: *mut ::core::ffi::c_void,
) -> cmd_retval {
    let mut mtd: *mut mode_tree_data = data as *mut mode_tree_data;
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
unsafe extern "C" fn mode_tree_prompt_input_callback(
    mut data: *mut ::core::ffi::c_void,
    mut s: *const ::core::ffi::c_char,
    mut key: prompt_key_result,
) -> prompt_result {
    let mut mtp: *mut mode_tree_prompt = data as *mut mode_tree_prompt;
    if (*mtp).inputcb.is_some() {
        return (*mtp).inputcb.expect("non-null function pointer")((*mtp).c, (*mtp).data, s, key);
    }
    return PROMPT_CLOSE;
}
unsafe extern "C" fn mode_tree_prompt_free_callback(data: *mut ::core::ffi::c_void) {
    let mtp_ptr = data as *mut mode_tree_prompt;
    let mtp = Box::from_raw(mtp_ptr);
    if (*mtp.mtd).prompt_data == mtp_ptr {
        (*mtp.mtd).prompt_data = ::core::ptr::null_mut::<mode_tree_prompt>();
    }
    if mtp.freecb.is_some() {
        mtp.freecb.expect("non-null function pointer")(mtp.data);
    }
    mode_tree_remove_ref(mtp.mtd);
}
#[no_mangle]
pub unsafe extern "C" fn mode_tree_set_prompt(
    mut mtd: *mut mode_tree_data,
    mut c: *mut client,
    mut prompt: *const ::core::ffi::c_char,
    mut input: *const ::core::ffi::c_char,
    mut type_0: prompt_type,
    mut flags: ::core::ffi::c_int,
    mut inputcb: mode_tree_prompt_input_cb,
    mut freecb: prompt_free_cb,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    let mut pd: prompt_create_data = prompt_create_data {
        fs: ::core::ptr::null_mut::<cmd_find_state>(),
        prompt: ::core::ptr::null::<::core::ffi::c_char>(),
        input: ::core::ptr::null::<::core::ffi::c_char>(),
        type_0: PROMPT_TYPE_COMMAND,
        flags: 0,
        style: grid_cell {
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
        },
        command_style: grid_cell {
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
        },
        style_str: ::core::ptr::null::<::core::ffi::c_char>(),
        command_style_str: ::core::ptr::null::<::core::ffi::c_char>(),
        cstyle: SCREEN_CURSOR_DEFAULT,
        command_cstyle: SCREEN_CURSOR_DEFAULT,
        ccolour: 0,
        command_ccolour: 0,
        cmode: 0,
        command_cmode: 0,
        message_format: ::core::ptr::null::<::core::ffi::c_char>(),
        keys: 0,
        word_separators: ::core::ptr::null::<::core::ffi::c_char>(),
        inputcb: None,
        freecb: None,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
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
        data,
    }));
    (*mtd).references = (*mtd).references.wrapping_add(1);
    (*mtd).prompt_top = (options_get_number(
        oo,
        b"status-position\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_longlong) as ::core::ffi::c_int;
    memset(
        &raw mut pd as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<prompt_create_data>() as size_t,
    );
    prompt_set_options(&raw mut pd, s);
    pd.prompt = prompt;
    pd.input = input;
    pd.type_0 = type_0;
    pd.flags = flags | PROMPT_ISMODE;
    pd.inputcb = Some(
        mode_tree_prompt_input_callback
            as unsafe extern "C" fn(
                *mut ::core::ffi::c_void,
                *const ::core::ffi::c_char,
                prompt_key_result,
            ) -> prompt_result,
    ) as prompt_input_cb;
    pd.freecb = Some(
        mode_tree_prompt_free_callback as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> (),
    ) as prompt_free_cb;
    pd.data = mtp as *mut ::core::ffi::c_void;
    (*mtd).prompt = prompt_create(&raw mut pd);
    (*mtd).prompt_data = mtp;
    mode_tree_draw(mtd);
    (*(*mtd).wp).flags |= PANE_REDRAW;
    if flags & PROMPT_SINGLE != 0 && flags & PROMPT_ACCEPT != 0 && !c.is_null() {
        (*mtd).references = (*mtd).references.wrapping_add(1);
        cmdq_append(
            c,
            cmdq_get_callback1(
                b"mode_tree_prompt_accept\0" as *const u8 as *const ::core::ffi::c_char,
                Some(
                    mode_tree_prompt_accept
                        as unsafe extern "C" fn(
                            *mut cmdq_item,
                            *mut ::core::ffi::c_void,
                        ) -> cmd_retval,
                ),
                mtd as *mut ::core::ffi::c_void,
            ),
        );
    }
}
unsafe extern "C" fn mode_tree_search_backward(
    mut mtd: *mut mode_tree_data,
) -> *mut mode_tree_item {
    let mut mti: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut last: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut prev: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut icase: ::core::ffi::c_int = (*mtd).search_icase;
    if (*mtd).search.is_null() {
        return ::core::ptr::null_mut::<mode_tree_item>();
    }
    last = (*(*mtd).lines.as_mut_ptr().offset((*mtd).current as isize)).item;
    mti = last;
    loop {
        prev = *(*((*mti).entry.tqe_prev as *mut mode_tree_list)).tqh_last;
        if !prev.is_null() {
            while !(*prev).children.tqh_first.is_null() {
                prev = *(*((*prev).children.tqh_last as *mut mode_tree_list)).tqh_last;
            }
            mti = prev;
        } else {
            mti = (*mti).parent;
        }
        if mti.is_null() {
            prev = *(*((*mtd).children.tqh_last as *mut mode_tree_list)).tqh_last;
            while !(*prev).children.tqh_first.is_null() {
                prev = *(*((*prev).children.tqh_last as *mut mode_tree_list)).tqh_last;
            }
            mti = prev;
        }
        if mti == last {
            break;
        }
        if (*mtd).searchcb.is_none() {
            if icase == 0 && !strstr((*mti).name, (*mtd).search).is_null() {
                return mti;
            }
            if icase != 0 && !strcasestr((*mti).name, (*mtd).search).is_null() {
                return mti;
            }
        } else if (*mtd).searchcb.expect("non-null function pointer")(
            (*mtd).modedata,
            (*mti).itemdata,
            (*mtd).search,
            icase,
        ) != 0
        {
            return mti;
        }
    }
    return ::core::ptr::null_mut::<mode_tree_item>();
}
unsafe extern "C" fn mode_tree_search_forward(mut mtd: *mut mode_tree_data) -> *mut mode_tree_item {
    let mut mti: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut last: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut next: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut icase: ::core::ffi::c_int = (*mtd).search_icase;
    if (*mtd).search.is_null() {
        return ::core::ptr::null_mut::<mode_tree_item>();
    }
    last = (*(*mtd).lines.as_mut_ptr().offset((*mtd).current as isize)).item;
    mti = last;
    loop {
        if !(*mti).children.tqh_first.is_null() {
            mti = (*mti).children.tqh_first;
        } else {
            next = (*mti).entry.tqe_next;
            if !next.is_null() {
                mti = next;
            } else {
                loop {
                    mti = (*mti).parent;
                    if mti.is_null() {
                        break;
                    }
                    next = (*mti).entry.tqe_next;
                    if next.is_null() {
                        continue;
                    }
                    mti = next;
                    break;
                }
            }
        }
        if mti.is_null() {
            mti = (*mtd).children.tqh_first;
        }
        if mti == last {
            break;
        }
        if (*mtd).searchcb.is_none() {
            if icase == 0 && !strstr((*mti).name, (*mtd).search).is_null() {
                return mti;
            }
            if icase != 0 && !strcasestr((*mti).name, (*mtd).search).is_null() {
                return mti;
            }
        } else if (*mtd).searchcb.expect("non-null function pointer")(
            (*mtd).modedata,
            (*mti).itemdata,
            (*mtd).search,
            icase,
        ) != 0
        {
            return mti;
        }
    }
    return ::core::ptr::null_mut::<mode_tree_item>();
}
unsafe extern "C" fn mode_tree_search_set(mut mtd: *mut mode_tree_data) {
    let mut mti: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut loop_0: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut identity = ModeTreeIdentity::legacy(0);
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
    identity = (*mti).identity;
    let name = (!identity.name.is_null()).then(|| CStr::from_ptr(identity.name).to_owned());
    let detail = (!identity.detail.is_null()).then(|| CStr::from_ptr(identity.detail).to_owned());
    identity.name = name.as_ref().map_or(::core::ptr::null(), |s| s.as_ptr());
    identity.detail = detail.as_ref().map_or(::core::ptr::null(), |s| s.as_ptr());
    loop_0 = (*mti).parent;
    while !loop_0.is_null() {
        (*loop_0).expanded = 1 as ::core::ffi::c_int;
        loop_0 = (*loop_0).parent;
    }
    mode_tree_build(mtd);
    mode_tree_set_current_identity(mtd, identity);
    mode_tree_draw(mtd);
    (*(*mtd).wp).flags |= PANE_REDRAW;
}
unsafe extern "C" fn mode_tree_search_callback(
    mut c: *mut client,
    mut data: *mut ::core::ffi::c_void,
    mut s: *const ::core::ffi::c_char,
    mut key: prompt_key_result,
) -> prompt_result {
    let mut mtd: *mut mode_tree_data = data as *mut mode_tree_data;
    if (*mtd).dead != 0 {
        return PROMPT_CLOSE;
    }
    let search = if s.is_null() || *s == 0 {
        None
    } else {
        Some(CStr::from_ptr(s).to_owned())
    };
    mode_tree_set_search(mtd, search);
    if !(*mtd).search.is_null() {
        (*mtd).search_icase = mode_tree_is_lowercase((*mtd).search);
        mode_tree_search_set(mtd);
    }
    if key as ::core::ffi::c_uint == PROMPT_KEY_HANDLED as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return PROMPT_CONTINUE;
    }
    return PROMPT_CLOSE;
}
unsafe extern "C" fn mode_tree_filter_callback(
    mut c: *mut client,
    mut data: *mut ::core::ffi::c_void,
    mut s: *const ::core::ffi::c_char,
    mut key: prompt_key_result,
) -> prompt_result {
    let mut mtd: *mut mode_tree_data = data as *mut mode_tree_data;
    if (*mtd).dead != 0 {
        return PROMPT_CLOSE;
    }
    let filter = if s.is_null() || *s == 0 {
        None
    } else {
        Some(CStr::from_ptr(s).to_owned())
    };
    mode_tree_set_filter(mtd, filter);
    mode_tree_build(mtd);
    mode_tree_draw(mtd);
    (*(*mtd).wp).flags |= PANE_REDRAW;
    if key as ::core::ffi::c_uint == PROMPT_KEY_HANDLED as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return PROMPT_CONTINUE;
    }
    return PROMPT_CLOSE;
}
unsafe extern "C" fn mode_tree_clear_filter(mut mtd: *mut mode_tree_data) {
    mode_tree_set_filter(mtd, None);
    mode_tree_build(mtd);
    mode_tree_draw(mtd);
    (*(*mtd).wp).flags |= PANE_REDRAW;
}
unsafe extern "C" fn mode_tree_menu_callback(
    mut menu: *mut menu,
    mut idx: u_int,
    mut key: key_code,
    mut data: *mut ::core::ffi::c_void,
) {
    let mtm = Box::from_raw(data as *mut mode_tree_menu);
    let mtd = mtm.data;
    if !((*mtd).dead != 0 || key == KEYC_NONE as ::core::ffi::c_ulong as key_code) {
        if !(mtm.line >= mode_tree_line_count(mtd)) {
            (*mtd).current = mtm.line;
            (*mtd).menucb.expect("non-null function pointer")((*mtd).modedata, mtm.c, key);
        }
    }
    mode_tree_remove_ref(mtd);
}
unsafe extern "C" fn mode_tree_display_menu(
    mut mtd: *mut mode_tree_data,
    mut c: *mut client,
    mut x: u_int,
    mut y: u_int,
    mut outside: ::core::ffi::c_int,
) {
    let mut mti: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut menu: *mut menu = ::core::ptr::null_mut::<menu>();
    let mut items: *const menu_item = ::core::ptr::null::<menu_item>();
    let mut mtm: *mut mode_tree_menu = ::core::ptr::null_mut::<mode_tree_menu>();
    let mut line: u_int = 0;
    if (*mtd).offset.wrapping_add(y) > mode_tree_line_count(mtd).wrapping_sub(1 as u_int) {
        line = (*mtd).current;
    } else {
        line = (*mtd).offset.wrapping_add(y);
    }
    mti = (*(*mtd).lines.as_mut_ptr().offset(line as isize)).item;
    let title = if outside == 0 {
        items = (*mtd).menu;
        let mut bytes = b"#[align=centre]".to_vec();
        bytes.extend_from_slice(CStr::from_ptr((*mti).name).to_bytes());
        CString::new(bytes).expect("mode tree item names contain no NUL")
    } else {
        items = &raw const mode_tree_menu_items as *const menu_item;
        c"".to_owned()
    };
    menu = menu_create(title.as_ptr());
    menu_add_items(
        menu,
        items,
        ::core::ptr::null_mut::<cmdq_item>(),
        c,
        ::core::ptr::null_mut::<cmd_find_state>(),
    );
    drop(title);
    mtm = Box::into_raw(Box::new(mode_tree_menu { data: mtd, c, line }));
    (*mtd).references = (*mtd).references.wrapping_add(1);
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
        Some(
            mode_tree_menu_callback
                as unsafe extern "C" fn(*mut menu, u_int, key_code, *mut ::core::ffi::c_void) -> (),
        ),
        mtm as *mut ::core::ffi::c_void,
    ) != 0 as ::core::ffi::c_int
    {
        mode_tree_remove_ref(mtd);
        drop(Box::from_raw(mtm));
        menu_free(menu);
    }
}
unsafe extern "C" fn mode_tree_draw_help_line(
    mut ctx: *mut screen_write_ctx,
    mut gc: *const grid_cell,
    mut ft: *mut format_tree,
    mut line: *const ::core::ffi::c_char,
    mut item: *const ::core::ffi::c_char,
    mut x: u_int,
    mut y: u_int,
    mut w: u_int,
) {
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut replaced: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    replaced = cmd_template_replace(line, item, 1 as ::core::ffi::c_int);
    expanded = format_expand(ft, replaced);
    free(replaced as *mut ::core::ffi::c_void);
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
        expanded,
        ::core::ptr::null_mut::<style_ranges>(),
        0 as ::core::ffi::c_int,
    );
    free(expanded as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn mode_tree_draw_help(
    mut mtd: *mut mode_tree_data,
    mut ctx: *mut screen_write_ctx,
) {
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
    let mut line: *mut *const ::core::ffi::c_char =
        ::core::ptr::null_mut::<*const ::core::ffi::c_char>();
    let mut lines: *mut *const ::core::ffi::c_char =
        ::core::ptr::null_mut::<*const ::core::ffi::c_char>();
    let mut item: *const ::core::ffi::c_char = b"item\0" as *const u8 as *const ::core::ffi::c_char;
    let mut sx: u_int = (*(*s).grid).sx;
    let mut sy: u_int = (*(*s).grid).sy;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut w: u_int = 0;
    let mut h: u_int = 0 as u_int;
    let mut box_w: u_int = 0;
    let mut box_h: u_int = 0;
    if (*mtd).helpcb.is_none() {
        w = MODE_TREE_HELP_DEFAULT_WIDTH as u_int;
    } else {
        lines = (*mtd).helpcb.expect("non-null function pointer")(&raw mut w, &raw mut item);
        if w < MODE_TREE_HELP_DEFAULT_WIDTH as u_int {
            w = MODE_TREE_HELP_DEFAULT_WIDTH as u_int;
        }
    }
    line = &raw mut mode_tree_help_start as *mut *const ::core::ffi::c_char;
    while !(*line).is_null() {
        h = h.wrapping_add(1);
        line = line.offset(1);
    }
    line = lines;
    while !line.is_null() && !(*line).is_null() {
        h = h.wrapping_add(1);
        line = line.offset(1);
    }
    line = &raw mut mode_tree_help_end as *mut *const ::core::ffi::c_char;
    while !(*line).is_null() {
        h = h.wrapping_add(1);
        line = line.offset(1);
    }
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
    line = &raw mut mode_tree_help_start as *mut *const ::core::ffi::c_char;
    while !(*line).is_null() {
        mode_tree_draw_help_line(ctx, &raw mut gc, ft, *line, item, x, y, w);
        line = line.offset(1);
        y = y.wrapping_add(1);
    }
    line = lines;
    while !line.is_null() && !(*line).is_null() {
        mode_tree_draw_help_line(ctx, &raw mut gc, ft, *line, item, x, y, w);
        line = line.offset(1);
        y = y.wrapping_add(1);
    }
    line = &raw mut mode_tree_help_end as *mut *const ::core::ffi::c_char;
    while !(*line).is_null() {
        mode_tree_draw_help_line(ctx, &raw mut gc, ft, *line, item, x, y, w);
        line = line.offset(1);
        y = y.wrapping_add(1);
    }
    format_free(ft);
}
unsafe extern "C" fn mode_tree_display_help(mut mtd: *mut mode_tree_data) {
    (*mtd).help = 1 as ::core::ffi::c_int;
    mode_tree_draw(mtd);
}
#[no_mangle]
pub unsafe extern "C" fn mode_tree_key(
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
    let mut mti: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
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
    if mode_tree_line_count(mtd) == 0 as u_int {
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
                sx = (*(*mtd).screen.grid).sx;
                if (*mtd).prompt_top != 0 {
                    py = 0 as u_int;
                } else {
                    py = (*(*mtd).screen.grid).sy.wrapping_sub(1 as u_int);
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
        if (*mtd).offset.wrapping_add(y) < mode_tree_line_count(mtd) {
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
    while i < mode_tree_line_count(mtd) {
        if *key == (*(*(*mtd).lines.as_mut_ptr().offset(i as isize)).item).key {
            choice = i as ::core::ffi::c_int;
            break;
        } else {
            i = i.wrapping_add(1);
        }
    }
    if choice != -(1 as ::core::ffi::c_int) {
        if choice as u_int > mode_tree_line_count(mtd).wrapping_sub(1 as u_int) {
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
                if (*mtd).current == mode_tree_line_count(mtd).wrapping_sub(1 as u_int) {
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
            (*mtd).current = mode_tree_line_count(mtd).wrapping_sub(1 as u_int);
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
            while i < mode_tree_line_count(mtd) {
                (*(*(*mtd).lines.as_mut_ptr().offset(i as isize)).item).tagged =
                    0 as ::core::ffi::c_int;
                i = i.wrapping_add(1);
            }
        }
        35184372088948 => {
            i = 0 as u_int;
            while i < mode_tree_line_count(mtd) {
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
            mti = (*mtd).children.tqh_first;
            while !mti.is_null() {
                (*mti).expanded = 0 as ::core::ffi::c_int;
                mti = (*mti).entry.tqe_next;
            }
            mode_tree_build(mtd);
        }
        17592186044459 => {
            mti = (*mtd).children.tqh_first;
            while !mti.is_null() {
                (*mti).expanded = 1 as ::core::ffi::c_int;
                mti = (*mti).entry.tqe_next;
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
                Some(
                    mode_tree_search_callback
                        as unsafe extern "C" fn(
                            *mut client,
                            *mut ::core::ffi::c_void,
                            *const ::core::ffi::c_char,
                            prompt_key_result,
                        ) -> prompt_result,
                ),
                None,
                mtd as *mut ::core::ffi::c_void,
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
                (*mtd).filter,
                PROMPT_TYPE_SEARCH,
                PROMPT_NOFORMAT,
                Some(
                    mode_tree_filter_callback
                        as unsafe extern "C" fn(
                            *mut client,
                            *mut ::core::ffi::c_void,
                            *const ::core::ffi::c_char,
                            prompt_key_result,
                        ) -> prompt_result,
                ),
                None,
                mtd as *mut ::core::ffi::c_void,
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
#[no_mangle]
pub unsafe extern "C" fn mode_tree_run_command(
    mut c: *mut client,
    mut fs: *mut cmd_find_state,
    mut template: *const ::core::ffi::c_char,
    mut name: *const ::core::ffi::c_char,
) {
    let mut state: *mut cmdq_state = ::core::ptr::null_mut::<cmdq_state>();
    let mut command: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut error: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut status: cmd_parse_status = CMD_PARSE_ERROR;
    command = cmd_template_replace(template, name, 1 as ::core::ffi::c_int);
    if !command.is_null() && *command as ::core::ffi::c_int != '\0' as i32 {
        state = cmdq_new_state(
            fs,
            ::core::ptr::null_mut::<key_event>(),
            0 as ::core::ffi::c_int,
        );
        status = cmd_parse_and_append(
            command,
            ::core::ptr::null_mut::<cmd_parse_input>(),
            c,
            state,
            &raw mut error,
        );
        if status as ::core::ffi::c_uint
            == CMD_PARSE_ERROR as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if !c.is_null() {
                *error = ({
                    let mut __res: ::core::ffi::c_int = 0;
                    if ::core::mem::size_of::<u_char>() as usize > 1 as usize {
                        if 0 != 0 {
                            let mut __c: ::core::ffi::c_int =
                                *error as u_char as ::core::ffi::c_int;
                            __res = (if __c < -(128 as ::core::ffi::c_int)
                                || __c > 255 as ::core::ffi::c_int
                            {
                                __c as __int32_t
                            } else {
                                *(*__ctype_toupper_loc()).offset(__c as isize)
                            }) as ::core::ffi::c_int;
                        } else {
                            __res = toupper(*error as u_char as ::core::ffi::c_int);
                        }
                    } else {
                        __res = *(*__ctype_toupper_loc())
                            .offset(*error as u_char as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int;
                    }
                    __res
                }) as ::core::ffi::c_char;
                status_message_set(
                    c,
                    -(1 as ::core::ffi::c_int),
                    1 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                    error,
                );
            }
            free(error as *mut ::core::ffi::c_void);
        }
        cmdq_free_state(state);
    }
    free(command as *mut ::core::ffi::c_void);
}

#[cfg(test)]
mod identity_tests {
    use super::*;
    use std::ffi::CString;

    unsafe extern "C" fn changing_key(
        data: *mut ::core::ffi::c_void,
        _: *mut ::core::ffi::c_void,
        _: u_int,
    ) -> key_code {
        *(data as *const key_code)
    }

    #[test]
    fn item_key_label_tracks_repeated_line_builds() {
        unsafe {
            let mtd = mode_tree_alloc_data();
            (*mtd).children.tqh_last = &raw mut (*mtd).children.tqh_first;
            let mut key = b'x' as key_code;
            (*mtd).keycb = Some(changing_key);
            (*mtd).modedata = (&raw mut key).cast();
            let item = mode_tree_add_identity(
                mtd,
                ::core::ptr::null_mut(),
                ::core::ptr::null_mut(),
                ModeTreeIdentity::legacy(1),
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
                        assert_eq!(CStr::from_ptr((*item).keystr).to_bytes(), expected);
                        assert_eq!((*item).keylen, expected.len());
                    }
                    None => {
                        assert!((*item).keystr.is_null());
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

    unsafe extern "C" fn nested_build(
        data: *mut ::core::ffi::c_void,
        _: *mut sort_criteria,
        _: *mut uint64_t,
        _: *const ::core::ffi::c_char,
    ) {
        let state = &mut *(data as *mut NestedBuildState);
        if state.empty {
            return;
        }
        let parent = mode_tree_add_identity(
            state.mtd,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            ModeTreeIdentity::session(1),
            c"parent".as_ptr(),
            std::ptr::null(),
            1,
        );
        for id in 0..64 {
            mode_tree_add_identity(
                state.mtd,
                parent,
                std::ptr::null_mut(),
                ModeTreeIdentity::pane(id),
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
            (*mtd).children.tqh_last = &raw mut (*mtd).children.tqh_first;
            (*mtd).saved.tqh_last = &raw mut (*mtd).saved.tqh_first;
            (*mtd).preview = MODE_TREE_PREVIEW_OFF as ::core::ffi::c_int;
            (*mtd).buildcb = Some(nested_build);
            let mut state = NestedBuildState { mtd, empty: false };
            (*mtd).modedata = (&raw mut state).cast();
            (*mtd).screen.grid = Box::into_raw(Box::new(std::mem::zeroed::<grid>()));
            (*(*mtd).screen.grid).sx = 80;
            (*(*mtd).screen.grid).sy = 24;

            mode_tree_build(mtd);
            assert_eq!((*mtd).lines.len(), 65);
            assert_eq!((&(*mtd).lines)[0].depth, 0);
            for i in 1..65 {
                assert_eq!((&(*mtd).lines)[i].depth, 1);
                assert_eq!((*(&(*mtd).lines)[i].item).line, i as u_int);
            }
            assert_eq!(
                mode_tree_set_current_identity(mtd, ModeTreeIdentity::pane(63)),
                1
            );
            assert_eq!((*mtd).current, 64);

            state.empty = true;
            mode_tree_build(mtd);
            assert!((*mtd).lines.is_empty());
            mode_tree_free_items(&raw mut (*mtd).children);
            drop(Box::from_raw((*mtd).screen.grid));
            mode_tree_remove_ref(mtd);
        }
    }

    unsafe extern "C" fn named_build(
        data: *mut ::core::ffi::c_void,
        _: *mut sort_criteria,
        _: *mut uint64_t,
        _: *const ::core::ffi::c_char,
    ) {
        let mtd = data as *mut mode_tree_data;
        for key in ["one", "two"] {
            let name = CString::new("@row").unwrap();
            let detail = CString::new(key).unwrap();
            mode_tree_add_identity(
                mtd,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                ModeTreeIdentity::named(5, 2, 0, name.as_ptr(), detail.as_ptr()),
                detail.as_ptr(),
                std::ptr::null(),
                1,
            );
        }
    }

    #[test]
    fn named_selection_and_tag_survive_rebuild_after_source_names_drop() {
        unsafe {
            let mtd = mode_tree_alloc_data();
            (*mtd).children.tqh_last = &raw mut (*mtd).children.tqh_first;
            (*mtd).saved.tqh_last = &raw mut (*mtd).saved.tqh_first;
            (*mtd).preview = MODE_TREE_PREVIEW_OFF as ::core::ffi::c_int;
            (*mtd).buildcb = Some(named_build);
            (*mtd).modedata = mtd.cast();
            (*mtd).screen.grid = Box::into_raw(Box::new(std::mem::zeroed::<grid>()));
            (*(*mtd).screen.grid).sx = 80;
            (*(*mtd).screen.grid).sy = 24;
            mode_tree_build(mtd);
            assert_eq!(mode_tree_line_count(mtd), 2);
            (*mtd).current = 1;
            let chosen = (*(*mtd).lines.as_mut_ptr().add(1)).item;
            (*chosen).tagged = 1;
            (*chosen).expanded = 0;
            mode_tree_build(mtd);
            assert_eq!((*mtd).current, 1);
            let restored = (*(*mtd).lines.as_mut_ptr().add(1)).item;
            assert_eq!((*restored).tagged, 1);
            assert_eq!((*restored).expanded, 0);
            assert_eq!(
                CStr::from_ptr((*restored).identity.detail).to_bytes(),
                b"two"
            );
            mode_tree_free_items(&raw mut (*mtd).children);
            mode_tree_clear_lines(mtd);
            drop(Box::from_raw((*mtd).screen.grid));
            mode_tree_remove_ref(mtd);
        }
    }

    #[test]
    fn named_keys_own_bytes_and_restore_only_exact_rows() {
        unsafe {
            let mtd = mode_tree_alloc_data();
            (*mtd).children.tqh_last = &raw mut (*mtd).children.tqh_first;
            (*mtd).saved.tqh_last = &raw mut (*mtd).saved.tqh_first;
            let name = CString::new(vec![b'@', 0xff, b'x']).unwrap().into_raw();
            let array_key = CString::new("7").unwrap().into_raw();
            let identity = ModeTreeIdentity::named(5, 2, 0, name, array_key);
            let prior = mode_tree_add_identity(
                mtd,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                identity,
                b"row\0".as_ptr().cast(),
                std::ptr::null(),
                1,
            );
            (*prior).tagged = 1;
            (*prior).expanded = 0;
            *name.add(1) = b'z' as ::core::ffi::c_char;
            *array_key = b'8' as ::core::ffi::c_char;
            assert_eq!(CStr::from_ptr((*prior).identity.name).to_bytes(), b"@\xffx");
            assert_eq!(CStr::from_ptr((*prior).identity.detail).to_bytes(), b"7");
            drop(CString::from_raw(name));
            drop(CString::from_raw(array_key));

            (*mtd).saved.tqh_first = prior;
            (*mtd).saved.tqh_last = (*mtd).children.tqh_last;
            (*prior).entry.tqe_prev = &raw mut (*mtd).saved.tqh_first;
            (*mtd).children.tqh_first = std::ptr::null_mut();
            (*mtd).children.tqh_last = &raw mut (*mtd).children.tqh_first;
            let original_name = CString::new(vec![b'@', 0xff, b'x']).unwrap();
            let original_key = CString::new("7").unwrap();
            let restored = mode_tree_add_identity(
                mtd,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                ModeTreeIdentity::named(5, 2, 0, original_name.as_ptr(), original_key.as_ptr()),
                b"row\0".as_ptr().cast(),
                std::ptr::null(),
                1,
            );
            assert_eq!((*restored).tagged, 1);
            assert_eq!((*restored).expanded, 0);
            for (kind, group, detail) in [
                (5, 3, b"7\0".as_ptr()),
                (5, 2, b"8\0".as_ptr()),
                (8, 2, b"7\0".as_ptr()),
            ] {
                let different = mode_tree_add_identity(
                    mtd,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    ModeTreeIdentity::named(kind, group, 0, original_name.as_ptr(), detail.cast()),
                    b"row\0".as_ptr().cast(),
                    std::ptr::null(),
                    1,
                );
                assert_eq!((*different).tagged, 0);
                assert_eq!((*different).expanded, 1);
            }
            mode_tree_free_items(&raw mut (*mtd).children);
            mode_tree_free_items(&raw mut (*mtd).saved);
            mode_tree_remove_ref(mtd);
        }
    }

    #[test]
    fn semantic_keys_restore_tags_expansion_and_selection_without_collisions() {
        assert_ne!(ModeTreeIdentity::session(7), ModeTreeIdentity::pane(7));
        assert_ne!(ModeTreeIdentity::session(7), ModeTreeIdentity::legacy(7));
        assert_ne!(
            ModeTreeIdentity::winlink(7, 3),
            ModeTreeIdentity::winlink(8, 3)
        );
        assert_ne!(
            ModeTreeIdentity::winlink(7, 3),
            ModeTreeIdentity::winlink(7, 4)
        );
        assert_ne!(
            ModeTreeIdentity::winlink(u_int::MAX, -1),
            ModeTreeIdentity::winlink(u_int::MAX, i32::MAX)
        );
        unsafe {
            let mtd = mode_tree_alloc_data();
            (*mtd).children.tqh_last = &raw mut (*mtd).children.tqh_first;
            (*mtd).saved.tqh_last = &raw mut (*mtd).saved.tqh_first;
            let name = b"item\0".as_ptr().cast();
            let prior = mode_tree_add_identity(
                mtd,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                ModeTreeIdentity::winlink(7, 3),
                name,
                std::ptr::null(),
                1,
            );
            (*prior).tagged = 1;
            (*prior).expanded = 0;
            (*mtd).saved.tqh_first = prior;
            (*mtd).saved.tqh_last = (*mtd).children.tqh_last;
            (*prior).entry.tqe_prev = &raw mut (*mtd).saved.tqh_first;
            (*mtd).children.tqh_first = std::ptr::null_mut();
            (*mtd).children.tqh_last = &raw mut (*mtd).children.tqh_first;

            let restored = mode_tree_add_identity(
                mtd,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                ModeTreeIdentity::winlink(7, 3),
                name,
                std::ptr::null(),
                1,
            );
            assert_eq!((*restored).tagged, 1);
            assert_eq!((*restored).expanded, 0);

            let different_session = mode_tree_add_identity(
                mtd,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                ModeTreeIdentity::winlink(8, 3),
                name,
                std::ptr::null(),
                1,
            );
            assert_eq!((*different_session).tagged, 0);
            assert_eq!((*different_session).expanded, 1);

            let lines = [
                mode_tree_line {
                    item: restored,
                    depth: 0,
                    last: 0,
                    flat: 0,
                },
                mode_tree_line {
                    item: different_session,
                    depth: 0,
                    last: 0,
                    flat: 0,
                },
            ];
            (*mtd).lines.extend_from_slice(&lines);
            (*mtd).height = 20;
            assert_eq!(
                mode_tree_set_current_identity(mtd, ModeTreeIdentity::winlink(8, 3)),
                1
            );
            assert_eq!((*mtd).current, 1);
            assert_eq!(
                mode_tree_set_current_identity(mtd, ModeTreeIdentity::winlink(7, 3)),
                1
            );
            assert_eq!((*mtd).current, 0);
            assert_eq!(
                mode_tree_set_current_identity(mtd, ModeTreeIdentity::pane(7)),
                0
            );
            assert_eq!(
                mode_tree_set_current_identity(mtd, ModeTreeIdentity::session(7)),
                0
            );
            assert_eq!(
                mode_tree_set_current_identity(mtd, ModeTreeIdentity::legacy(7)),
                0
            );

            mode_tree_free_items(&raw mut (*mtd).children);
            mode_tree_free_items(&raw mut (*mtd).saved);
            mode_tree_remove_ref(mtd);
        }
    }
}
