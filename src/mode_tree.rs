pub use crate::src::shared::prompt::{prompt_create_data, prompt_draw_data};
pub use crate::src::shared::mode_tree::{
    mode_tree_build_cb, mode_tree_data, mode_tree_draw_cb, mode_tree_each_cb,
    mode_tree_height_cb, mode_tree_help_cb, mode_tree_item, mode_tree_item_entry,
    mode_tree_key_cb, mode_tree_line, mode_tree_list, mode_tree_menu_cb, mode_tree_prompt,
    mode_tree_prompt_input_cb, mode_tree_search_cb, mode_tree_search_dir, mode_tree_sort_cb,
    mode_tree_swap_cb,
};
pub use crate::src::shared::command::{cmd_parse_input};
pub use crate::src::shared::arguments::{args};
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
pub use crate::src::shared::command::{
    cmd_find_state, cmd_list, cmdq_cb, cmdq_item, cmdq_list, cmdq_state, cmds,
};
pub use crate::src::shared::control::{control_state};
pub use crate::src::shared::format::{format_job_tree, format_tree};
pub use crate::src::shared::input::{input_ctx, input_request, input_requests};
pub use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
pub use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
pub use crate::src::shared::menu::{menu_choice_cb, menu_data};
pub use crate::src::shared::options::{options};
pub use crate::src::shared::pane::{
    window_pane, window_pane_entry, window_pane_modes, window_pane_prompt, window_pane_sentry,
    window_pane_tree_entry, window_pane_zentry, window_panes,
};
pub use crate::src::shared::process::{tmuxpeer};
pub use crate::src::shared::prompt::{prompt};
pub use crate::src::shared::redraw::{redraw_scene};
pub use crate::src::shared::screen_write::{screen_write_ctx, screen_write_init_ctx_cb};
pub use crate::src::shared::session::{session, session_entry, session_gentry};
pub use crate::src::shared::spawn::{spawn_editor_state};
pub use crate::src::shared::status::{status_line};
pub use crate::src::shared::tty::{
    tty, tty_code, tty_ctx, tty_ctx_c2rust_unnamed, tty_ctx_c2rust_unnamed_data,
    tty_ctx_c2rust_unnamed_sel, tty_ctx_redraw_cb, tty_ctx_set_client_cb, tty_key,
    tty_style_ctx, tty_term, tty_term_entry,
};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
pub use crate::src::shared::environment::{environ};
pub use crate::src::shared::window::{WINDOW_ZOOMED};
pub use crate::src::shared::pane::{
    PANE_REDRAW, window_pane_offset, window_pane_resize, window_pane_resize_entry,
    window_pane_resizes,
};
pub use crate::src::shared::screen::{MODE_CURSOR, screen, screen_sel, screen_titles};
pub use crate::src::shared::menu::{menu, menu_item};
pub use crate::src::shared::prompt::{
    PROMPT_ACCEPT, PROMPT_CLOSE, PROMPT_CONTINUE, PROMPT_ISMODE, PROMPT_NOFORMAT, PROMPT_SINGLE,
    prompt_free_cb, prompt_input_cb, prompt_result,
};
pub use crate::src::shared::abi::{__int32_t};
pub use crate::src::shared::screen_write::{screen_write_citem, screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::colour::{
    COLOUR_FLAG_THEME, COLOUR_THEME_BLACK, COLOUR_THEME_BLUE, COLOUR_THEME_CYAN,
    COLOUR_THEME_DARK_GREY, COLOUR_THEME_GREEN, COLOUR_THEME_LIGHT_GREY, COLOUR_THEME_MAGENTA,
    COLOUR_THEME_RED, COLOUR_THEME_WHITE, COLOUR_THEME_YELLOW, colour_theme,
};
pub use crate::src::shared::sort::{sort_criteria};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{
    MOUSE_BUTTON_1, MOUSE_MASK_BUTTONS, MOUSE_MASK_DRAG, mouse_event,
};
use crate::src::shared::client::*;
use crate::src::shared::layout::*;
use crate::src::shared::prompt::*;
use crate::src::shared::command::*;
use crate::src::shared::sort::*;
use crate::src::shared::terminal::*;
use crate::src::shared::event::*;
use crate::src::shared::display::*;
use crate::src::shared::layout::*;
use crate::src::shared::message::*;
use crate::src::shared::abi::*;
use crate::src::shared::colour::*;
use crate::src::shared::command::*;
use crate::src::shared::grid::*;
use crate::src::shared::key::*;
use crate::src::shared::style::*;
extern "C" {

    fn __ctype_tolower_loc() -> *mut *const __int32_t;
    fn __ctype_toupper_loc() -> *mut *const __int32_t;
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strstr(
        __haystack: *const ::core::ffi::c_char,
        __needle: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strcasestr(
        __haystack: *const ::core::ffi::c_char,
        __needle: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xmalloc(_: size_t) -> *mut ::core::ffi::c_void;
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn xreallocarray(_: *mut ::core::ffi::c_void, _: size_t, _: size_t)
        -> *mut ::core::ffi::c_void;
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn xasprintf(
        _: *mut *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    static mut global_s_options: *mut options;
    fn sort_next_order(_: *mut sort_criteria);
    fn sort_order_from_string(_: *const ::core::ffi::c_char) -> sort_order;
    fn sort_order_to_string(_: sort_order) -> *const ::core::ffi::c_char;
    fn format_free(_: *mut format_tree);
    fn format_add(
        _: *mut format_tree,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    );
    fn format_expand(
        _: *mut format_tree,
        _: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn format_create_defaults(
        _: *mut cmdq_item,
        _: *mut client,
        _: *mut session,
        _: *mut winlink,
        _: *mut window_pane,
    ) -> *mut format_tree;
    fn format_draw(
        _: *mut screen_write_ctx,
        _: *const grid_cell,
        _: u_int,
        _: *const ::core::ffi::c_char,
        _: *mut style_ranges,
        _: ::core::ffi::c_int,
    );
    fn format_width(_: *const ::core::ffi::c_char) -> u_int;
    fn options_get_number(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn args_has(_: *mut args, _: u_char) -> ::core::ffi::c_int;
    fn args_get(_: *mut args, _: u_char) -> *const ::core::ffi::c_char;
    fn cmd_mouse_at(
        _: *mut window_pane,
        _: *mut mouse_event,
        _: *mut u_int,
        _: *mut u_int,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn cmd_template_replace(
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn cmd_parse_and_append(
        _: *const ::core::ffi::c_char,
        _: *mut cmd_parse_input,
        _: *mut client,
        _: *mut cmdq_state,
        _: *mut *mut ::core::ffi::c_char,
    ) -> cmd_parse_status;
    fn cmdq_new_state(
        _: *mut cmd_find_state,
        _: *mut key_event,
        _: ::core::ffi::c_int,
    ) -> *mut cmdq_state;
    fn cmdq_free_state(_: *mut cmdq_state);
    fn cmdq_get_client(_: *mut cmdq_item) -> *mut client;
    fn cmdq_get_callback1(
        _: *const ::core::ffi::c_char,
        _: cmdq_cb,
        _: *mut ::core::ffi::c_void,
    ) -> *mut cmdq_item;
    fn cmdq_append(_: *mut client, _: *mut cmdq_item) -> *mut cmdq_item;
    fn key_string_lookup_key(_: key_code, _: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn server_redraw_window(_: *mut window);
    fn server_unzoom_window(_: *mut window);
    fn status_message_set(
        _: *mut client,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: *const ::core::ffi::c_char,
        ...
    );
    fn prompt_set_options(_: *mut prompt_create_data, _: *mut session);
    fn prompt_create(_: *const prompt_create_data) -> *mut prompt;
    fn prompt_free(_: *mut prompt);
    fn prompt_draw(_: *mut prompt, _: *mut prompt_draw_data);
    fn prompt_key(_: *mut prompt, _: key_code, _: *mut ::core::ffi::c_int) -> prompt_key_result;
    fn prompt_mouse(
        _: *mut prompt,
        _: u_int,
        _: u_int,
        _: u_int,
        _: *mut ::core::ffi::c_int,
    ) -> prompt_key_result;
    fn prompt_closed(_: *mut prompt) -> ::core::ffi::c_int;
    static grid_default_cell: grid_cell;
    fn screen_write_start(_: *mut screen_write_ctx, _: *mut screen);
    fn screen_write_stop(_: *mut screen_write_ctx);
    fn screen_write_puts(
        _: *mut screen_write_ctx,
        _: *const grid_cell,
        _: *const ::core::ffi::c_char,
        ...
    );
    fn screen_write_box(
        _: *mut screen_write_ctx,
        _: u_int,
        _: u_int,
        _: box_lines,
        _: *const grid_cell,
        _: *const ::core::ffi::c_char,
    );
    fn screen_write_clearcharacter(_: *mut screen_write_ctx, _: u_int, _: u_int);
    fn screen_write_clearendofline(_: *mut screen_write_ctx, _: u_int);
    fn screen_write_cursormove(
        _: *mut screen_write_ctx,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    );
    fn screen_write_clearscreen(_: *mut screen_write_ctx, _: u_int);
    fn screen_init(_: *mut screen, _: u_int, _: u_int, _: u_int);
    fn screen_free(_: *mut screen);
    fn screen_resize(_: *mut screen, _: u_int, _: u_int, _: ::core::ffi::c_int);
    fn window_zoom(_: *mut window_pane) -> ::core::ffi::c_int;
    fn log_debug(_: *const ::core::ffi::c_char, ...);
    fn menu_create(_: *const ::core::ffi::c_char) -> *mut menu;
    fn menu_add_items(
        _: *mut menu,
        _: *const menu_item,
        _: *mut cmdq_item,
        _: *mut client,
        _: *mut cmd_find_state,
    );
    fn menu_free(_: *mut menu);
    fn menu_display(
        _: *mut menu,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: *mut cmdq_item,
        _: u_int,
        _: u_int,
        _: *mut client,
        _: box_lines,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: *mut cmd_find_state,
        _: menu_choice_cb,
        _: *mut ::core::ffi::c_void,
    ) -> ::core::ffi::c_int;
    fn style_apply(
        _: *mut grid_cell,
        _: *mut options,
        _: *const ::core::ffi::c_char,
        _: *mut format_tree,
    );
}

pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;
pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;

pub const MODE_TREE_SEARCH_BACKWARD: mode_tree_search_dir = 1;
pub const MODE_TREE_SEARCH_FORWARD: mode_tree_search_dir = 0;

pub type C2RustUnnamed_39 = ::core::ffi::c_ulong;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct mode_tree_menu {
    pub data: *mut mode_tree_data,
    pub c: *mut client,
    pub line: u_int,
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
    mut tag: uint64_t,
) -> *mut mode_tree_item {
    let mut mti: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut child: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    mti = (*mtl).tqh_first;
    while !mti.is_null() {
        if (*mti).tag == tag {
            return mti;
        }
        child = mode_tree_find_item(&raw mut (*mti).children, tag);
        if !child.is_null() {
            return child;
        }
        mti = (*mti).entry.tqe_next;
    }
    return ::core::ptr::null_mut::<mode_tree_item>();
}
unsafe extern "C" fn mode_tree_free_item(mut mti: *mut mode_tree_item) {
    mode_tree_free_items(&raw mut (*mti).children);
    free((*mti).name as *mut ::core::ffi::c_void);
    free((*mti).text as *mut ::core::ffi::c_void);
    free((*mti).keystr as *mut ::core::ffi::c_void);
    free(mti as *mut ::core::ffi::c_void);
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
unsafe extern "C" fn mode_tree_clear_lines(mut mtd: *mut mode_tree_data) {
    free((*mtd).line_list as *mut ::core::ffi::c_void);
    (*mtd).line_list = ::core::ptr::null_mut::<mode_tree_line>();
    (*mtd).line_size = 0 as u_int;
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
        (*mtd).line_list = xreallocarray(
            (*mtd).line_list as *mut ::core::ffi::c_void,
            (*mtd).line_size.wrapping_add(1 as u_int) as size_t,
            ::core::mem::size_of::<mode_tree_line>() as size_t,
        ) as *mut mode_tree_line;
        let fresh0 = (*mtd).line_size;
        (*mtd).line_size = (*mtd).line_size.wrapping_add(1);
        line =
            (*mtd).line_list.offset(fresh0 as isize) as *mut mode_tree_line as *mut mode_tree_line;
        (*line).item = mti;
        (*line).depth = depth;
        (*line).last =
            (mti == *(*((*mtl).tqh_last as *mut mode_tree_list)).tqh_last) as ::core::ffi::c_int;
        (*mti).line = (*mtd).line_size.wrapping_sub(1 as u_int);
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
        if (*mti).key != KEYC_NONE as ::core::ffi::c_ulong as key_code {
            (*mti).keystr = xstrdup(key_string_lookup_key((*mti).key, 0 as ::core::ffi::c_int));
            (*mti).keylen = strlen((*mti).keystr);
        } else {
            (*mti).keystr = ::core::ptr::null::<::core::ffi::c_char>();
            (*mti).keylen = 0 as size_t;
        }
        mti = (*mti).entry.tqe_next;
    }
    mti = (*mtl).tqh_first;
    while !mti.is_null() {
        i = 0 as u_int;
        while i < (*mtd).line_size {
            line =
                (*mtd).line_list.offset(i as isize) as *mut mode_tree_line as *mut mode_tree_line;
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
    if (*mtd).line_size == 0 as u_int {
        return;
    }
    if (*mtd).current == 0 as u_int {
        if wrap != 0 {
            (*mtd).current = (*mtd).line_size.wrapping_sub(1 as u_int);
            if (*mtd).line_size >= (*mtd).height {
                (*mtd).offset = (*mtd).line_size.wrapping_sub((*mtd).height);
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
    if (*mtd).line_size == 0 as u_int {
        return 0 as ::core::ffi::c_int;
    }
    if (*mtd).current == (*mtd).line_size.wrapping_sub(1 as u_int) {
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
    let mut current_depth: u_int = (*(*mtd).line_list.offset((*mtd).current as isize)).depth;
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
            && swap_with.wrapping_add(direction as u_int) >= (*mtd).line_size
        {
            return;
        }
        swap_with = swap_with.wrapping_add(direction as u_int);
        swap_with_depth = (*(*mtd).line_list.offset(swap_with as isize)).depth;
        if !(swap_with_depth > current_depth) {
            break;
        }
    }
    if swap_with_depth != current_depth {
        return;
    }
    if (*mtd).swapcb.expect("non-null function pointer")(
        (*(*(*mtd).line_list.offset((*mtd).current as isize)).item).itemdata,
        (*(*(*mtd).line_list.offset(swap_with as isize)).item).itemdata,
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
    if (*mtd).line_size == 0 as u_int {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    return (*(*(*mtd).line_list.offset((*mtd).current as isize)).item).itemdata;
}
#[no_mangle]
pub unsafe extern "C" fn mode_tree_get_current_name(
    mut mtd: *mut mode_tree_data,
) -> *const ::core::ffi::c_char {
    return (*(*(*mtd).line_list.offset((*mtd).current as isize)).item).name;
}
#[no_mangle]
pub unsafe extern "C" fn mode_tree_select_top(mut mtd: *mut mode_tree_data) {
    (*mtd).current = 0 as u_int;
    (*mtd).offset = 0 as u_int;
}
#[no_mangle]
pub unsafe extern "C" fn mode_tree_expand_current(mut mtd: *mut mode_tree_data) {
    if (*(*(*mtd).line_list.offset((*mtd).current as isize)).item).expanded == 0 {
        (*(*(*mtd).line_list.offset((*mtd).current as isize)).item).expanded =
            1 as ::core::ffi::c_int;
        mode_tree_build(mtd);
    }
}
#[no_mangle]
pub unsafe extern "C" fn mode_tree_collapse_current(mut mtd: *mut mode_tree_data) {
    if (*(*(*mtd).line_list.offset((*mtd).current as isize)).item).expanded != 0 {
        (*(*(*mtd).line_list.offset((*mtd).current as isize)).item).expanded =
            0 as ::core::ffi::c_int;
        mode_tree_build(mtd);
    }
}
unsafe extern "C" fn mode_tree_get_tag(
    mut mtd: *mut mode_tree_data,
    mut tag: uint64_t,
    mut found: *mut u_int,
) -> ::core::ffi::c_int {
    let mut i: u_int = 0;
    i = 0 as u_int;
    while i < (*mtd).line_size {
        if (*(*(*mtd).line_list.offset(i as isize)).item).tag == tag {
            break;
        }
        i = i.wrapping_add(1);
    }
    if i != (*mtd).line_size {
        *found = i;
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn mode_tree_expand(mut mtd: *mut mode_tree_data, mut tag: uint64_t) {
    let mut found: u_int = 0;
    if mode_tree_get_tag(mtd, tag, &raw mut found) == 0 {
        return;
    }
    if (*(*(*mtd).line_list.offset(found as isize)).item).expanded == 0 {
        (*(*(*mtd).line_list.offset(found as isize)).item).expanded = 1 as ::core::ffi::c_int;
        mode_tree_build(mtd);
    }
}
#[no_mangle]
pub unsafe extern "C" fn mode_tree_set_current(
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
    if (*mtd).current >= (*mtd).line_size {
        if (*mtd).line_size == 0 as u_int {
            return 0 as ::core::ffi::c_int;
        }
        (*mtd).current = (*mtd).line_size.wrapping_sub(1 as u_int);
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
    while i < (*mtd).line_size {
        mti = (*(*mtd).line_list.offset(i as isize)).item;
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
    while i < (*mtd).line_size {
        mti = (*(*mtd).line_list.offset(i as isize)).item;
        if (*mti).tagged != 0 {
            fired = 1 as ::core::ffi::c_int;
            cb.expect("non-null function pointer")((*mtd).modedata, (*mti).itemdata, c, key);
        }
        i = i.wrapping_add(1);
    }
    if fired == 0 && current != 0 {
        mti = (*(*mtd).line_list.offset((*mtd).current as isize)).item;
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
    mtd = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<mode_tree_data>() as size_t,
    ) as *mut mode_tree_data;
    (*mtd).references = 1 as u_int;
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
        (*mtd).filter = xstrdup(args_get(args, 'f' as i32 as u_char));
    } else {
        (*mtd).filter = ::core::ptr::null_mut::<::core::ffi::c_char>();
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
        if (*mtd).height > (*mtd).line_size {
            (*mtd).height = (*(*s).grid).sy.wrapping_div(2 as u_int);
        }
        if (*mtd).height < 10 as u_int {
            (*mtd).height = (*(*s).grid).sy;
        }
    } else if (*mtd).preview == MODE_TREE_PREVIEW_BIG as ::core::ffi::c_int {
        (*mtd).height = (*(*s).grid).sy.wrapping_div(4 as u_int);
        if (*mtd).height > (*mtd).line_size {
            (*mtd).height = (*mtd).line_size;
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
    if !(*mtd).line_list.is_null() {
        tag = (*(*(*mtd).line_list.offset((*mtd).current as isize)).item).tag;
    } else {
        tag = UINT64_MAX as uint64_t;
    }
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
    mode_tree_free_items(&raw mut (*mtd).saved);
    (*mtd).saved.tqh_first = ::core::ptr::null_mut::<mode_tree_item>();
    (*mtd).saved.tqh_last = &raw mut (*mtd).saved.tqh_first;
    mode_tree_clear_lines(mtd);
    (*mtd).maxdepth = 0 as u_int;
    mode_tree_build_lines(mtd, &raw mut (*mtd).children, 0 as u_int);
    if !(*mtd).line_list.is_null() && tag == UINT64_MAX as uint64_t {
        tag = (*(*(*mtd).line_list.offset((*mtd).current as isize)).item).tag;
    }
    mode_tree_set_current(mtd, tag);
    (*mtd).width = (*(*s).grid).sx;
    if (*mtd).preview != MODE_TREE_PREVIEW_OFF as ::core::ffi::c_int {
        mode_tree_set_height(mtd);
    } else {
        (*mtd).height = (*(*s).grid).sy;
    }
    mode_tree_check_selected(mtd);
}
unsafe extern "C" fn mode_tree_remove_ref(mut mtd: *mut mode_tree_data) {
    (*mtd).references = (*mtd).references.wrapping_sub(1);
    if (*mtd).references == 0 as u_int {
        free(mtd as *mut ::core::ffi::c_void);
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
    free((*mtd).search as *mut ::core::ffi::c_void);
    free((*mtd).filter as *mut ::core::ffi::c_void);
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
    let mut mti: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut saved: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    log_debug(
        b"%s: %llu, %s %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"mode_tree_add\0" as *const u8 as *const ::core::ffi::c_char,
        tag as ::core::ffi::c_ulonglong,
        name,
        if text.is_null() {
            b"\0" as *const u8 as *const ::core::ffi::c_char
        } else {
            text
        },
    );
    mti = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<mode_tree_item>() as size_t,
    ) as *mut mode_tree_item;
    (*mti).parent = parent;
    (*mti).itemdata = itemdata;
    (*mti).tag = tag;
    (*mti).name = xstrdup(name);
    if !text.is_null() {
        (*mti).text = xstrdup(text);
    }
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
    let mut text: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut prefix: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut tag: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut separator: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut n: size_t = 0;
    let mut keylen: ::core::ffi::c_int = 0;
    let vla = (*mtd).maxdepth.wrapping_add(1 as u_int) as usize;
    let mut alignlen: Vec<::core::ffi::c_int> = ::std::vec::from_elem(0, vla);
    let mut dfg: ::core::ffi::c_int = 0;
    let mut dfg0: ::core::ffi::c_int = 0;
    if (*mtd).line_size == 0 as u_int {
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
    while i < (*mtd).line_size {
        mti = (*(*mtd).line_list.offset(i as isize)).item;
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
    while i < (*mtd).line_size {
        line = (*mtd).line_list.offset(i as isize) as *mut mode_tree_line as *mut mode_tree_line;
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
    while i < (*mtd).line_size {
        if !(i < (*mtd).offset) {
            if i > (*mtd).offset.wrapping_add(h).wrapping_sub(1 as u_int) {
                break;
            }
            line =
                (*mtd).line_list.offset(i as isize) as *mut mode_tree_line as *mut mode_tree_line;
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
                    && (*(*mtd).line_list.offset((*(*mti).parent).line as isize)).last != 0
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
            xasprintf(
                &raw mut text,
                b"%*s%s%s\0" as *const u8 as *const ::core::ffi::c_char,
                (*mti).align * *alignlen.as_mut_ptr().offset((*line).depth as isize),
                (*mti).name,
                tag,
                separator,
            );
            text_width = format_width(text);
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
                        text,
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
                        text,
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
            free(text as *mut ::core::ffi::c_void);
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
            line = (*mtd).line_list.offset((*mtd).current as isize) as *mut mode_tree_line
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
            if !(*mtd).sort_crit.order_seq.is_null() {
                xasprintf(
                    &raw mut text,
                    b" %s (sort: %s%s)%s%s%s\0" as *const u8 as *const ::core::ffi::c_char,
                    (*mti).name,
                    sort_order_to_string((*mtd).sort_crit.order),
                    if (*mtd).sort_crit.reversed != 0 {
                        b", reversed\0" as *const u8 as *const ::core::ffi::c_char
                    } else {
                        b"\0" as *const u8 as *const ::core::ffi::c_char
                    },
                    if (*mtd).view_name.is_null() {
                        b"\0" as *const u8 as *const ::core::ffi::c_char
                    } else {
                        b" (view: \0" as *const u8 as *const ::core::ffi::c_char
                    },
                    if (*mtd).view_name.is_null() {
                        b"\0" as *const u8 as *const ::core::ffi::c_char
                    } else {
                        (*mtd).view_name
                    },
                    if (*mtd).view_name.is_null() {
                        b"\0" as *const u8 as *const ::core::ffi::c_char
                    } else {
                        b")\0" as *const u8 as *const ::core::ffi::c_char
                    },
                );
            } else {
                xasprintf(
                    &raw mut text,
                    b" %s\0" as *const u8 as *const ::core::ffi::c_char,
                    (*mti).name,
                );
            }
            if w.wrapping_sub(2 as u_int) as size_t >= strlen(text) {
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
                    text,
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
                        >= strlen(text)
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
            free(text as *mut ::core::ffi::c_void);
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
unsafe extern "C" fn mode_tree_prompt_free_callback(mut data: *mut ::core::ffi::c_void) {
    let mut mtp: *mut mode_tree_prompt = data as *mut mode_tree_prompt;
    if (*(*mtp).mtd).prompt_data == mtp {
        (*(*mtp).mtd).prompt_data = ::core::ptr::null_mut::<mode_tree_prompt>();
    }
    if (*mtp).freecb.is_some() {
        (*mtp).freecb.expect("non-null function pointer")((*mtp).data);
    }
    mode_tree_remove_ref((*mtp).mtd);
    free(mtp as *mut ::core::ffi::c_void);
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
    mtp = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<mode_tree_prompt>() as size_t,
    ) as *mut mode_tree_prompt;
    (*mtp).mtd = mtd;
    (*mtp).c = c;
    (*mtp).inputcb = inputcb;
    (*mtp).freecb = freecb;
    (*mtp).data = data;
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
    last = (*(*mtd).line_list.offset((*mtd).current as isize)).item;
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
    last = (*(*mtd).line_list.offset((*mtd).current as isize)).item;
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
    free((*mtd).search as *mut ::core::ffi::c_void);
    if s.is_null() || *s as ::core::ffi::c_int == '\0' as i32 {
        (*mtd).search = ::core::ptr::null_mut::<::core::ffi::c_char>();
    } else {
        (*mtd).search = xstrdup(s);
        (*mtd).search_icase = mode_tree_is_lowercase(s);
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
    if !(*mtd).filter.is_null() {
        free((*mtd).filter as *mut ::core::ffi::c_void);
    }
    if s.is_null() || *s as ::core::ffi::c_int == '\0' as i32 {
        (*mtd).filter = ::core::ptr::null_mut::<::core::ffi::c_char>();
    } else {
        (*mtd).filter = xstrdup(s);
    }
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
    free((*mtd).filter as *mut ::core::ffi::c_void);
    (*mtd).filter = ::core::ptr::null_mut::<::core::ffi::c_char>();
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
    let mut mtm: *mut mode_tree_menu = data as *mut mode_tree_menu;
    let mut mtd: *mut mode_tree_data = (*mtm).data;
    if !((*mtd).dead != 0 || key == KEYC_NONE as ::core::ffi::c_ulong as key_code) {
        if !((*mtm).line >= (*mtd).line_size) {
            (*mtd).current = (*mtm).line;
            (*mtd).menucb.expect("non-null function pointer")((*mtd).modedata, (*mtm).c, key);
        }
    }
    mode_tree_remove_ref(mtd);
    free(mtm as *mut ::core::ffi::c_void);
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
    let mut title: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut line: u_int = 0;
    if (*mtd).offset.wrapping_add(y) > (*mtd).line_size.wrapping_sub(1 as u_int) {
        line = (*mtd).current;
    } else {
        line = (*mtd).offset.wrapping_add(y);
    }
    mti = (*(*mtd).line_list.offset(line as isize)).item;
    if outside == 0 {
        items = (*mtd).menu;
        xasprintf(
            &raw mut title,
            b"#[align=centre]%s\0" as *const u8 as *const ::core::ffi::c_char,
            (*mti).name,
        );
    } else {
        items = &raw const mode_tree_menu_items as *const menu_item;
        title = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    }
    menu = menu_create(title);
    menu_add_items(
        menu,
        items,
        ::core::ptr::null_mut::<cmdq_item>(),
        c,
        ::core::ptr::null_mut::<cmd_find_state>(),
    );
    free(title as *mut ::core::ffi::c_void);
    mtm = xmalloc(::core::mem::size_of::<mode_tree_menu>() as size_t) as *mut mode_tree_menu;
    (*mtm).data = mtd;
    (*mtm).c = c;
    (*mtm).line = line;
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
        free(mtm as *mut ::core::ffi::c_void);
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
    if (*mtd).line_size == 0 as u_int {
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
        if (*mtd).offset.wrapping_add(y) < (*mtd).line_size {
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
    line = (*mtd).line_list.offset((*mtd).current as isize) as *mut mode_tree_line
        as *mut mode_tree_line;
    current = (*line).item;
    choice = -(1 as ::core::ffi::c_int);
    i = 0 as u_int;
    while i < (*mtd).line_size {
        if *key == (*(*(*mtd).line_list.offset(i as isize)).item).key {
            choice = i as ::core::ffi::c_int;
            break;
        } else {
            i = i.wrapping_add(1);
        }
    }
    if choice != -(1 as ::core::ffi::c_int) {
        if choice as u_int > (*mtd).line_size.wrapping_sub(1 as u_int) {
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
                if (*mtd).current == (*mtd).line_size.wrapping_sub(1 as u_int) {
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
            (*mtd).current = (*mtd).line_size.wrapping_sub(1 as u_int);
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
            while i < (*mtd).line_size {
                (*(*(*mtd).line_list.offset(i as isize)).item).tagged = 0 as ::core::ffi::c_int;
                i = i.wrapping_add(1);
            }
        }
        35184372088948 => {
            i = 0 as u_int;
            while i < (*mtd).line_size {
                if (*(*(*mtd).line_list.offset(i as isize)).item)
                    .parent
                    .is_null()
                    && (*(*(*mtd).line_list.offset(i as isize)).item).no_tag == 0
                    || !(*(*(*mtd).line_list.offset(i as isize)).item)
                        .parent
                        .is_null()
                        && (*(*(*(*mtd).line_list.offset(i as isize)).item).parent).no_tag != 0
                {
                    (*(*(*mtd).line_list.offset(i as isize)).item).tagged = 1 as ::core::ffi::c_int;
                } else {
                    (*(*(*mtd).line_list.offset(i as isize)).item).tagged = 0 as ::core::ffi::c_int;
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
