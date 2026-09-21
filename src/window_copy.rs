use crate::src::arguments::{args_count, args_free, args_has, args_string, args_values};
pub use crate::src::arguments::args_parse;
use crate::src::cmd::{cmd_mouse_at, cmd_mouse_pane};
use crate::src::compat::strtonum::strtonum;
use crate::src::events::events_fire_pane;
use crate::src::ffi::libc::{
    __ctype_tolower_loc, abs, free, llabs, memcmp, memcpy, memmove, memset, regcomp, regexec,
    regfree, strcasecmp, strchr, strcmp, strcspn, strlen, strncmp, vasprintf,
};
use crate::src::ffi::libevent::{bufferevent_write, event_add, event_del, event_set};
use crate::src::format::{
    format_add, format_add_cb, format_create_defaults, format_expand, format_free,
    format_get_pane, format_grid_hyperlink, format_grid_line, format_grid_word, format_single,
};
use crate::src::format_draw::format_draw;
use crate::src::grid::{
    grid_default_cell, grid_duplicate_lines, grid_free_lines, grid_get_cell, grid_get_line,
    grid_in_set, grid_line_length, grid_line_limit, grid_line_time, grid_peek_line,
    grid_unwrap_position, grid_wrap_position,
};
use crate::src::grid_reader::{
    grid_reader_cursor_back_to_indentation, grid_reader_cursor_end_of_line,
    grid_reader_cursor_jump, grid_reader_cursor_jump_back, grid_reader_cursor_left,
    grid_reader_cursor_next_word, grid_reader_cursor_next_word_end,
    grid_reader_cursor_previous_word, grid_reader_cursor_right,
    grid_reader_cursor_start_of_line, grid_reader_get_cursor, grid_reader_in_set,
    grid_reader_start,
};
use crate::src::hyperlinks::{hyperlinks_copy, hyperlinks_free};
use crate::src::input::{input_free, input_init, input_parse_screen};
use crate::src::job::{job_get_event, job_run};
use crate::src::log::{fatalx, log_debug};
use crate::src::options::{options_get_number, options_get_string};
use crate::src::paste::{paste_add, paste_buffer_data, paste_get_top, paste_set};
use crate::src::screen::{
    screen_check_selection, screen_clear_selection, screen_free, screen_hide_selection,
    screen_init, screen_resize, screen_resize_cursor, screen_set_default_cursor,
    screen_set_selection,
};
use crate::src::screen_write::{
    screen_write_carriagereturn, screen_write_cell, screen_write_cursormove,
    screen_write_deleteline, screen_write_insertline, screen_write_linefeed, screen_write_nputs,
    screen_write_putc, screen_write_setselection, screen_write_start, screen_write_start_pane,
    screen_write_stop, screen_write_strlen, screen_write_vnputs,
};
use crate::src::status::status_message_set;
use crate::src::style::style_apply;
use crate::src::tmux::{get_timer, global_options, global_w_options};
use crate::src::tty::tty_window_offset;
use crate::src::tty_acs::tty_acs_get;
use crate::src::utf8::{utf8_copy, utf8_fromcstr, utf8_set, utf8_to_data};
use crate::src::window::{
    window_pane_reset_mode, window_pane_scrollbar_overlay_visible, window_pane_scrollbar_redraw,
    window_pane_scrollbar_show, window_set_active_pane,
};
use crate::src::xmalloc::{xcalloc, xmalloc, xrealloc, xreallocarray, xstrdup};
pub use crate::src::shared::arguments::{
    args, args_parse_cb, args_value, args_value_c2rust_unnamed, args_value_entry,
};
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
pub use crate::src::shared::command::{cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds};
pub use crate::src::shared::control::{control_state};
pub use crate::src::shared::format::{format_cb, format_job_tree, format_tree};
pub use crate::src::shared::input::{input_ctx, input_request, input_requests};
pub use crate::src::shared::job::{job, job_complete_cb, job_free_cb, job_update_cb};
pub use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
pub use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
pub use crate::src::shared::menu::{menu_data};
pub use crate::src::shared::options::{options};
pub use crate::src::shared::pane::{
    window_pane, window_pane_entry, window_pane_modes, window_pane_prompt, window_pane_sentry,
    window_pane_tree_entry, window_pane_zentry, window_panes,
};
pub use crate::src::shared::paste::{paste_buffer, paste_buffer_name_entry, paste_buffer_time_entry};
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
pub use crate::src::shared::grid::{WHITESPACE};
pub use crate::src::shared::key::{MODEKEY_EMACS, MODEKEY_VI};
pub use crate::src::shared::job::{JOB_NOWAIT};
pub use crate::src::shared::limits::{__INT_MAX__, __SCHAR_MAX__, INT_MAX, UCHAR_MAX, UINT_MAX};
pub use crate::src::shared::regex::{
    __re_long_size_t, re_dfa_t, re_pattern_buffer, reg_syntax_t, regex_t, regmatch_t, regoff_t,
    REG_EXTENDED, REG_ICASE,
};
pub use crate::src::shared::pane::{
    PANE_REDRAW, PANE_REDRAWSCROLLBAR, PANE_UNSEENCHANGES, window_pane_offset,
    window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::variadic::{__builtin_va_list, __va_list_tag, va_list};
pub use crate::src::shared::abi::{__int32_t, ssize_t};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_citem, screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::client::{CLIENT_READONLY};
pub use crate::src::shared::grid::{grid_reader};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{
    MOUSE_MASK_BUTTONS, MOUSE_WHEEL_DOWN, MOUSE_WHEEL_UP, mouse_event,
};
use crate::src::shared::client::*;
use crate::src::shared::arguments::*;
use crate::src::shared::terminal::*;
use crate::src::shared::event::*;
use crate::src::shared::display::*;
use crate::src::shared::layout::*;
use crate::src::shared::message::*;
use crate::src::shared::abi::*;
use crate::src::shared::colour::*;
use crate::src::shared::grid::*;
use crate::src::shared::key::*;
use crate::src::shared::style::*;

pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;
pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_copy_mode_data {
    pub screen: screen,
    pub backing: *mut screen,
    pub backing_written: ::core::ffi::c_int,
    pub ictx: *mut input_ctx,
    pub sync_added: u_int,
    pub sync_collected: u_int,
    pub sync_generation: u_int,
    pub viewmode: ::core::ffi::c_int,
    pub oy: u_int,
    pub selx: u_int,
    pub sely: u_int,
    pub endselx: u_int,
    pub endsely: u_int,
    pub cursordrag: C2RustUnnamed_45,
    pub modekeys: ::core::ffi::c_int,
    pub lineflag: C2RustUnnamed_44,
    pub rectflag: ::core::ffi::c_int,
    pub scroll_exit: ::core::ffi::c_int,
    pub hide_position: ::core::ffi::c_int,
    pub line_numbers: ::core::ffi::c_int,
    pub selflag: C2RustUnnamed_43,
    pub recentre_state: C2RustUnnamed_42,
    pub recentre_line: u_int,
    pub separators: *const ::core::ffi::c_char,
    pub dx: u_int,
    pub dy: u_int,
    pub selrx: u_int,
    pub selry: u_int,
    pub endselrx: u_int,
    pub endselry: u_int,
    pub cx: u_int,
    pub cy: u_int,
    pub lastcx: u_int,
    pub lastsx: u_int,
    pub mx: u_int,
    pub my: u_int,
    pub showmark: ::core::ffi::c_int,
    pub searchtype: ::core::ffi::c_int,
    pub searchdirection: ::core::ffi::c_int,
    pub searchregex: ::core::ffi::c_int,
    pub searchstr: *mut ::core::ffi::c_char,
    pub searchmark: *mut u_char,
    pub searchcount: ::core::ffi::c_int,
    pub searchmore: ::core::ffi::c_int,
    pub searchall: ::core::ffi::c_int,
    pub searchx: ::core::ffi::c_int,
    pub searchy: ::core::ffi::c_int,
    pub searcho: ::core::ffi::c_int,
    pub searchgen: u_char,
    pub timeout: ::core::ffi::c_int,
    pub jumptype: ::core::ffi::c_int,
    pub jumpchar: *mut utf8_data,
    pub dragtimer: event,
    pub refresh_timer: event,
    pub refresh_active: ::core::ffi::c_int,
}
pub type C2RustUnnamed_42 = ::core::ffi::c_uint;
pub const RECENTRE_BOTTOM: C2RustUnnamed_42 = 2;
pub const RECENTRE_MIDDLE: C2RustUnnamed_42 = 1;
pub const RECENTRE_TOP: C2RustUnnamed_42 = 0;
pub type C2RustUnnamed_43 = ::core::ffi::c_uint;
pub const SEL_LINE: C2RustUnnamed_43 = 2;
pub const SEL_WORD: C2RustUnnamed_43 = 1;
pub const SEL_CHAR: C2RustUnnamed_43 = 0;
pub type C2RustUnnamed_44 = ::core::ffi::c_uint;
pub const LINE_SEL_RIGHT_LEFT: C2RustUnnamed_44 = 2;
pub const LINE_SEL_LEFT_RIGHT: C2RustUnnamed_44 = 1;
pub const LINE_SEL_NONE: C2RustUnnamed_44 = 0;
pub type C2RustUnnamed_45 = ::core::ffi::c_uint;
pub const CURSORDRAG_SEL: C2RustUnnamed_45 = 2;
pub const CURSORDRAG_ENDSEL: C2RustUnnamed_45 = 1;
pub const CURSORDRAG_NONE: C2RustUnnamed_45 = 0;
pub const WINDOW_COPY_LINE_NUMBERS_OFF: window_copy_line_numbers = 0;
pub const WINDOW_COPY_LINE_NUMBERS_DEFAULT: window_copy_line_numbers = 1;
pub const WINDOW_COPY_LINE_NUMBERS_HYBRID: window_copy_line_numbers = 4;
pub const WINDOW_COPY_LINE_NUMBERS_RELATIVE: window_copy_line_numbers = 3;
pub const WINDOW_COPY_LINE_NUMBERS_ABSOLUTE: window_copy_line_numbers = 2;
pub const WINDOW_COPY_CMD_MOVE: window_copy_cmd_action = 1;
pub type window_copy_cmd_action = ::core::ffi::c_uint;
pub const WINDOW_COPY_CMD_CANCEL: window_copy_cmd_action = 3;
pub const WINDOW_COPY_CMD_REDRAW: window_copy_cmd_action = 2;
pub const WINDOW_COPY_CMD_NOTHING: window_copy_cmd_action = 0;
pub const WINDOW_COPY_CMD_CLEAR_NEVER: window_copy_cmd_clear = 1;
pub type window_copy_cmd_clear = ::core::ffi::c_uint;
pub const WINDOW_COPY_CMD_CLEAR_EMACS_ONLY: window_copy_cmd_clear = 2;
pub const WINDOW_COPY_CMD_CLEAR_ALWAYS: window_copy_cmd_clear = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_copy_cmd_state {
    pub wme: *mut window_mode_entry,
    pub args: *mut args,
    pub wargs: *mut args,
    pub m: *mut mouse_event,
    pub c: *mut client,
    pub s: *mut session,
    pub wl: *mut winlink,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_46 {
    pub command: *const ::core::ffi::c_char,
    pub minargs: u_int,
    pub maxargs: u_int,
    pub args: args_parse,
    pub flags: ::core::ffi::c_int,
    pub clear: window_copy_cmd_clear,
    pub f: Option<unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action>,
}
pub const WINDOW_COPY_REL_POS_ON_SCREEN: C2RustUnnamed_50 = 1;
pub const WINDOW_COPY_REL_POS_BELOW: C2RustUnnamed_50 = 2;
pub const WINDOW_COPY_REL_POS_ABOVE: C2RustUnnamed_50 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_47 {
    pub d: *const ::core::ffi::c_char,
    pub dlen: size_t,
    pub allocated: ::core::ffi::c_int,
}
pub const WINDOW_COPY_SEARCHDOWN: C2RustUnnamed_49 = 2;
pub const WINDOW_COPY_SEARCHUP: C2RustUnnamed_49 = 1;
pub const BOTTOM: C2RustUnnamed_48 = 2;
pub const TOP: C2RustUnnamed_48 = 1;
pub const MIDDLE: C2RustUnnamed_48 = 0;
pub type C2RustUnnamed_48 = ::core::ffi::c_uint;
pub const WINDOW_COPY_JUMPTOFORWARD: C2RustUnnamed_49 = 5;
pub const WINDOW_COPY_JUMPTOBACKWARD: C2RustUnnamed_49 = 6;
pub const WINDOW_COPY_JUMPBACKWARD: C2RustUnnamed_49 = 4;
pub const WINDOW_COPY_JUMPFORWARD: C2RustUnnamed_49 = 3;
pub const WINDOW_COPY_OFF: C2RustUnnamed_49 = 0;
pub type C2RustUnnamed_49 = ::core::ffi::c_uint;
pub type C2RustUnnamed_50 = ::core::ffi::c_uint;
pub type window_copy_line_numbers = ::core::ffi::c_uint;
#[inline]
unsafe extern "C" fn tolower(mut __c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    return if __c >= -(128 as ::core::ffi::c_int) && __c < 256 as ::core::ffi::c_int {
        *(*__ctype_tolower_loc()).offset(__c as isize) as ::core::ffi::c_int
    } else {
        __c
    };
}

pub const REG_NOTBOL: ::core::ffi::c_int = 1 as ::core::ffi::c_int;

#[no_mangle]
pub static mut window_copy_mode: window_mode = unsafe {
    window_mode {
        name: b"copy-mode\0" as *const u8 as *const ::core::ffi::c_char,
        default_format: ::core::ptr::null::<::core::ffi::c_char>(),
        flags: 0,
        init: Some(
            window_copy_init
                as unsafe extern "C" fn(
                    *mut window_mode_entry,
                    *mut cmdq_item,
                    *mut cmd_find_state,
                    *mut args,
                ) -> *mut screen,
        ),
        free: Some(window_copy_free as unsafe extern "C" fn(*mut window_mode_entry) -> ()),
        resize: Some(
            window_copy_resize as unsafe extern "C" fn(*mut window_mode_entry, u_int, u_int) -> (),
        ),
        update: None,
        style_changed: Some(
            window_copy_style_changed as unsafe extern "C" fn(*mut window_mode_entry) -> (),
        ),
        key: None,
        key_table: Some(
            window_copy_key_table
                as unsafe extern "C" fn(*mut window_mode_entry) -> *const ::core::ffi::c_char,
        ),
        command: Some(
            window_copy_command
                as unsafe extern "C" fn(
                    *mut window_mode_entry,
                    *mut client,
                    *mut session,
                    *mut winlink,
                    *mut args,
                    *mut mouse_event,
                ) -> (),
        ),
        formats: Some(
            window_copy_formats
                as unsafe extern "C" fn(*mut window_mode_entry, *mut format_tree) -> (),
        ),
        get_screen: Some(
            window_copy_get_screen as unsafe extern "C" fn(*mut window_mode_entry) -> *mut screen,
        ),
    }
};
#[no_mangle]
pub static mut window_view_mode: window_mode = unsafe {
    window_mode {
        name: b"view-mode\0" as *const u8 as *const ::core::ffi::c_char,
        default_format: ::core::ptr::null::<::core::ffi::c_char>(),
        flags: 0,
        init: Some(
            window_copy_view_init
                as unsafe extern "C" fn(
                    *mut window_mode_entry,
                    *mut cmdq_item,
                    *mut cmd_find_state,
                    *mut args,
                ) -> *mut screen,
        ),
        free: Some(window_copy_free as unsafe extern "C" fn(*mut window_mode_entry) -> ()),
        resize: Some(
            window_copy_resize as unsafe extern "C" fn(*mut window_mode_entry, u_int, u_int) -> (),
        ),
        update: None,
        style_changed: Some(
            window_copy_style_changed as unsafe extern "C" fn(*mut window_mode_entry) -> (),
        ),
        key: None,
        key_table: Some(
            window_copy_key_table
                as unsafe extern "C" fn(*mut window_mode_entry) -> *const ::core::ffi::c_char,
        ),
        command: Some(
            window_copy_command
                as unsafe extern "C" fn(
                    *mut window_mode_entry,
                    *mut client,
                    *mut session,
                    *mut winlink,
                    *mut args,
                    *mut mouse_event,
                ) -> (),
        ),
        formats: Some(
            window_copy_formats
                as unsafe extern "C" fn(*mut window_mode_entry, *mut format_tree) -> (),
        ),
        get_screen: Some(
            window_copy_get_screen as unsafe extern "C" fn(*mut window_mode_entry) -> *mut screen,
        ),
    }
};
pub const WINDOW_COPY_SEARCH_TIMEOUT: ::core::ffi::c_int = 10000 as ::core::ffi::c_int;
pub const WINDOW_COPY_SEARCH_ALL_TIMEOUT: ::core::ffi::c_int = 200 as ::core::ffi::c_int;
pub const WINDOW_COPY_SEARCH_MAX_LINE: ::core::ffi::c_int = 2000 as ::core::ffi::c_int;
pub const WINDOW_COPY_DRAG_REPEAT_TIME: ::core::ffi::c_int = 50000 as ::core::ffi::c_int;
pub const WINDOW_COPY_REFRESH_INTERVAL: ::core::ffi::c_int = 50000 as ::core::ffi::c_int;
unsafe extern "C" fn window_copy_scroll_timer(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut wme: *mut window_mode_entry = arg as *mut window_mode_entry;
    let mut wp: *mut window_pane = (*wme).wp;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: WINDOW_COPY_DRAG_REPEAT_TIME as __suseconds_t,
    };
    event_del(&raw mut (*data).dragtimer);
    if (*wp).modes.tqh_first != wme {
        return;
    }
    if (*data).cy == 0 as u_int {
        event_add(&raw mut (*data).dragtimer, &raw mut tv);
        window_copy_cursor_up(wme, 1 as ::core::ffi::c_int);
    } else if (*data).cy == (*(*data).screen.grid).sy.wrapping_sub(1 as u_int) {
        event_add(&raw mut (*data).dragtimer, &raw mut tv);
        window_copy_cursor_down(wme, 1 as ::core::ffi::c_int);
    }
}
unsafe extern "C" fn window_copy_clone_screen(
    mut src: *mut screen,
    mut hint: *mut screen,
    mut cx: *mut u_int,
    mut cy: *mut u_int,
    mut trim: ::core::ffi::c_int,
) -> *mut screen {
    let mut dst: *mut screen = ::core::ptr::null_mut::<screen>();
    let mut gl: *const grid_line = ::core::ptr::null::<grid_line>();
    let mut sy: u_int = 0;
    let mut wx: u_int = 0;
    let mut wy: u_int = 0;
    let mut reflow: ::core::ffi::c_int = 0;
    dst = xcalloc(1 as size_t, ::core::mem::size_of::<screen>() as size_t) as *mut screen;
    sy = (*(*src).grid).hsize.wrapping_add((*(*src).grid).sy);
    if trim != 0 {
        while sy > (*(*src).grid).hsize {
            gl = grid_peek_line((*src).grid, sy.wrapping_sub(1 as u_int));
            if gl.is_null() || (*gl).cellused as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
                break;
            }
            sy = sy.wrapping_sub(1);
        }
    }
    log_debug(
        b"%s: target screen is %ux%u, source %ux%u\0" as *const u8 as *const ::core::ffi::c_char,
        b"window_copy_clone_screen\0" as *const u8 as *const ::core::ffi::c_char,
        (*(*src).grid).sx,
        sy,
        (*(*hint).grid).sx,
        (*(*src).grid).hsize.wrapping_add((*(*src).grid).sy),
    );
    screen_init(dst, (*(*src).grid).sx, sy, (*(*src).grid).hlimit);
    (*(*dst).grid).flags |= GRID_HISTORY;
    grid_duplicate_lines((*dst).grid, 0 as u_int, (*src).grid, 0 as u_int, sy);
    (*(*dst).grid).sy = sy.wrapping_sub((*(*src).grid).hsize);
    (*(*dst).grid).hsize = (*(*src).grid).hsize;
    (*(*dst).grid).hscrolled = (*(*src).grid).hscrolled;
    if (*src).cy > (*(*dst).grid).sy.wrapping_sub(1 as u_int) {
        (*dst).cx = 0 as u_int;
        (*dst).cy = (*(*dst).grid).sy.wrapping_sub(1 as u_int);
    } else {
        (*dst).cx = (*src).cx;
        (*dst).cy = (*src).cy;
    }
    if !cx.is_null() && !cy.is_null() {
        *cx = (*dst).cx;
        *cy = (*(*dst).grid).hsize.wrapping_add((*dst).cy);
        reflow = ((*(*hint).grid).sx != (*(*dst).grid).sx) as ::core::ffi::c_int;
    } else {
        reflow = 0 as ::core::ffi::c_int;
    }
    if reflow != 0 {
        grid_wrap_position((*dst).grid, *cx, *cy, &raw mut wx, &raw mut wy);
    }
    screen_resize_cursor(
        dst,
        (*(*hint).grid).sx,
        (*(*hint).grid).sy,
        1 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    if reflow != 0 {
        grid_unwrap_position((*dst).grid, cx, cy, wx, wy);
    }
    return dst;
}
unsafe extern "C" fn window_copy_sync_snapshot(
    mut data: *mut window_copy_mode_data,
    mut src: *mut grid,
) {
    (*data).sync_added = (*src).scroll_added;
    (*data).sync_collected = (*src).scroll_collected;
    (*data).sync_generation = (*src).scroll_generation;
}
unsafe extern "C" fn window_copy_sync_backing(
    mut wme: *mut window_mode_entry,
) -> ::core::ffi::c_int {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut wp: *mut window_pane = (*wme).swp;
    let mut src: *mut screen = &raw mut (*wp).base;
    let mut dst: *mut screen = (*data).backing;
    let mut sg: *mut grid = (*src).grid;
    let mut dg: *mut grid = (*dst).grid;
    let mut sy: u_int = (*sg).sy;
    let mut old_hsize: u_int = (*dg).hsize;
    let mut new_hsize: u_int = (*sg).hsize;
    let mut added: u_int = 0;
    let mut collected: u_int = 0;
    let mut kept: u_int = 0;
    if (*data).viewmode != 0 || (*wme).swp != (*wme).wp {
        return 0 as ::core::ffi::c_int;
    }
    if (*sg).sx != (*dg).sx
        || (*sg).sy != (*dg).sy
        || (*sg).scroll_generation != (*data).sync_generation
    {
        return 0 as ::core::ffi::c_int;
    }
    added = (*sg).scroll_added.wrapping_sub((*data).sync_added);
    collected = (*sg).scroll_collected.wrapping_sub((*data).sync_collected);
    if added > INT_MAX as u_int
        || collected > INT_MAX as u_int
        || collected > old_hsize
        || old_hsize.wrapping_add(added) < collected
        || old_hsize.wrapping_add(added).wrapping_sub(collected) != new_hsize
    {
        return 0 as ::core::ffi::c_int;
    }
    kept = old_hsize.wrapping_sub(collected);
    if added == 0 as u_int && collected == 0 as u_int {
        grid_duplicate_lines(dg, (*dg).hsize, sg, (*sg).hsize, sy);
    } else {
        if collected > 0 as u_int {
            grid_free_lines(dg, 0 as u_int, collected);
            memmove(
                (*dg).linedata.offset(0 as ::core::ffi::c_int as isize) as *mut grid_line
                    as *mut ::core::ffi::c_void,
                (*dg).linedata.offset(collected as isize) as *mut grid_line
                    as *const ::core::ffi::c_void,
                (old_hsize.wrapping_add(sy).wrapping_sub(collected) as size_t)
                    .wrapping_mul(::core::mem::size_of::<grid_line>() as size_t),
            );
            memset(
                (*dg)
                    .linedata
                    .offset(old_hsize.wrapping_add(sy).wrapping_sub(collected) as isize)
                    as *mut grid_line as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                (collected as size_t).wrapping_mul(::core::mem::size_of::<grid_line>() as size_t),
            );
        }
        if new_hsize.wrapping_add(sy) != old_hsize.wrapping_add(sy).wrapping_sub(collected) {
            (*dg).linedata = xreallocarray(
                (*dg).linedata as *mut ::core::ffi::c_void,
                new_hsize.wrapping_add(sy) as size_t,
                ::core::mem::size_of::<grid_line>() as size_t,
            ) as *mut grid_line;
            memset(
                (*dg)
                    .linedata
                    .offset(old_hsize.wrapping_add(sy).wrapping_sub(collected) as isize)
                    as *mut grid_line as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                (new_hsize.wrapping_sub(kept) as size_t)
                    .wrapping_mul(::core::mem::size_of::<grid_line>() as size_t),
            );
        }
        (*dg).hsize = new_hsize;
        if added > 0 as u_int {
            grid_duplicate_lines(dg, kept, sg, kept, added);
        }
        grid_duplicate_lines(dg, new_hsize, sg, new_hsize, sy);
    }
    (*dg).hscrolled = (*sg).hscrolled;
    if (*src).cy > (*dg).sy.wrapping_sub(1 as u_int) {
        (*dst).cx = 0 as u_int;
        (*dst).cy = (*dg).sy.wrapping_sub(1 as u_int);
    } else {
        (*dst).cx = (*src).cx;
        (*dst).cy = (*src).cy;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn window_copy_common_init(
    mut wme: *mut window_mode_entry,
) -> *mut window_copy_mode_data {
    let mut wp: *mut window_pane = (*wme).wp;
    let mut data: *mut window_copy_mode_data = ::core::ptr::null_mut::<window_copy_mode_data>();
    let mut base: *mut screen = &raw mut (*wp).base;
    data = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<window_copy_mode_data>() as size_t,
    ) as *mut window_copy_mode_data;
    (*wme).data = data as *mut ::core::ffi::c_void;
    (*data).cursordrag = CURSORDRAG_NONE;
    (*data).lineflag = LINE_SEL_NONE;
    (*data).selflag = SEL_CHAR;
    if !(*wp).searchstr.is_null() {
        (*data).searchtype = WINDOW_COPY_SEARCHUP as ::core::ffi::c_int;
        (*data).searchregex = (*wp).searchregex;
        (*data).searchstr = xstrdup((*wp).searchstr);
    } else {
        (*data).searchtype = WINDOW_COPY_OFF as ::core::ffi::c_int;
        (*data).searchregex = 0 as ::core::ffi::c_int;
        (*data).searchstr = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    (*data).searcho = -(1 as ::core::ffi::c_int);
    (*data).searchy = (*data).searcho;
    (*data).searchx = (*data).searchy;
    (*data).searchall = 1 as ::core::ffi::c_int;
    (*data).jumptype = WINDOW_COPY_OFF as ::core::ffi::c_int;
    (*data).jumpchar = ::core::ptr::null_mut::<utf8_data>();
    (*data).line_numbers = 1 as ::core::ffi::c_int;
    screen_init(
        &raw mut (*data).screen,
        (*(*base).grid).sx,
        (*(*base).grid).sy,
        0 as u_int,
    );
    screen_set_default_cursor(&raw mut (*data).screen, global_w_options);
    (*data).modekeys = options_get_number(
        (*(*wp).window).options,
        b"mode-keys\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    event_set(
        &raw mut (*data).dragtimer,
        -(1 as ::core::ffi::c_int),
        0 as ::core::ffi::c_short,
        Some(
            window_copy_scroll_timer
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    ::core::ffi::c_short,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        wme as *mut ::core::ffi::c_void,
    );
    event_set(
        &raw mut (*data).refresh_timer,
        -(1 as ::core::ffi::c_int),
        0 as ::core::ffi::c_short,
        Some(
            window_copy_refresh_timer
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    ::core::ffi::c_short,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        wme as *mut ::core::ffi::c_void,
    );
    return data;
}
unsafe extern "C" fn window_copy_init(
    mut wme: *mut window_mode_entry,
    mut item: *mut cmdq_item,
    mut fs: *mut cmd_find_state,
    mut args: *mut args,
) -> *mut screen {
    let mut wp: *mut window_pane = (*wme).swp;
    let mut data: *mut window_copy_mode_data = ::core::ptr::null_mut::<window_copy_mode_data>();
    let mut base: *mut screen = &raw mut (*wp).base;
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
    let mut i: u_int = 0;
    let mut cx: u_int = 0;
    let mut cy: u_int = 0;
    data = window_copy_common_init(wme);
    (*data).backing = window_copy_clone_screen(
        base,
        &raw mut (*data).screen,
        &raw mut cx,
        &raw mut cy,
        ((*wme).swp != (*wme).wp) as ::core::ffi::c_int,
    );
    window_copy_sync_snapshot(data, (*base).grid);
    (*data).cx = cx;
    if cy < (*(*(*data).backing).grid).hsize {
        (*data).cy = 0 as u_int;
        (*data).oy = (*(*(*data).backing).grid).hsize.wrapping_sub(cy);
    } else {
        (*data).cy = cy.wrapping_sub((*(*(*data).backing).grid).hsize);
        (*data).oy = 0 as u_int;
    }
    (*data).scroll_exit = args_has(args, 'e' as i32 as u_char);
    (*data).hide_position = args_has(args, 'H' as i32 as u_char);
    if !(*base).hyperlinks.is_null() {
        hyperlinks_free((*data).screen.hyperlinks);
        (*data).screen.hyperlinks = hyperlinks_copy((*base).hyperlinks);
    }
    (*data).screen.cx = window_copy_cursor_offset(wme, (*data).cx, (*(*data).screen.grid).sx);
    (*data).screen.cy = (*data).cy;
    (*data).mx = (*data).cx;
    (*data).my = (*(*(*data).backing).grid)
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    (*data).showmark = 0 as ::core::ffi::c_int;
    screen_write_start(&raw mut ctx, &raw mut (*data).screen);
    i = 0 as u_int;
    while i < (*(*data).screen.grid).sy {
        window_copy_write_line(wme, &raw mut ctx, i);
        i = i.wrapping_add(1);
    }
    screen_write_cursormove(
        &raw mut ctx,
        window_copy_cursor_offset(wme, (*data).cx, (*(*data).screen.grid).sx) as ::core::ffi::c_int,
        (*data).cy as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    screen_write_stop(&raw mut ctx);
    (*data).recentre_state = RECENTRE_MIDDLE;
    (*data).recentre_line = 0 as u_int;
    return &raw mut (*data).screen;
}
unsafe extern "C" fn window_copy_view_init(
    mut wme: *mut window_mode_entry,
    mut item: *mut cmdq_item,
    mut fs: *mut cmd_find_state,
    mut args: *mut args,
) -> *mut screen {
    let mut wp: *mut window_pane = (*wme).wp;
    let mut data: *mut window_copy_mode_data = ::core::ptr::null_mut::<window_copy_mode_data>();
    let mut base: *mut screen = &raw mut (*wp).base;
    let mut sx: u_int = (*(*base).grid).sx;
    data = window_copy_common_init(wme);
    (*data).viewmode = 1 as ::core::ffi::c_int;
    (*data).line_numbers = 0 as ::core::ffi::c_int;
    (*data).backing = xmalloc(::core::mem::size_of::<screen>() as size_t) as *mut screen;
    screen_init((*data).backing, sx, (*(*base).grid).sy, UINT_MAX);
    (*data).ictx = input_init(
        ::core::ptr::null_mut::<window_pane>(),
        ::core::ptr::null_mut::<bufferevent>(),
        ::core::ptr::null_mut::<colour_palette>(),
        ::core::ptr::null_mut::<client>(),
    );
    (*data).mx = (*data).cx;
    (*data).my = (*(*(*data).backing).grid)
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    (*data).showmark = 0 as ::core::ffi::c_int;
    return &raw mut (*data).screen;
}
unsafe extern "C" fn window_copy_free(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    event_del(&raw mut (*data).dragtimer);
    event_del(&raw mut (*data).refresh_timer);
    free((*data).searchmark as *mut ::core::ffi::c_void);
    free((*data).searchstr as *mut ::core::ffi::c_void);
    free((*data).jumpchar as *mut ::core::ffi::c_void);
    if !(*data).ictx.is_null() {
        input_free((*data).ictx);
    }
    screen_free((*data).backing);
    free((*data).backing as *mut ::core::ffi::c_void);
    screen_free(&raw mut (*data).screen);
    free(data as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn window_copy_add(
    mut wp: *mut window_pane,
    mut parse: ::core::ffi::c_int,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut ap: ::core::ffi::VaList;
    ap = args.clone();
    window_copy_vadd(wp, parse, fmt, ap);
}
unsafe extern "C" fn window_copy_init_ctx_cb(
    mut ctx: *mut screen_write_ctx,
    mut ttyctx: *mut tty_ctx,
) {
}
#[no_mangle]
pub unsafe extern "C" fn window_copy_vadd(
    mut wp: *mut window_pane,
    mut parse: ::core::ffi::c_int,
    mut fmt: *const ::core::ffi::c_char,
    mut ap: ::core::ffi::VaList,
) {
    let mut wme: *mut window_mode_entry = (*wp).modes.tqh_first;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut backing: *mut screen = (*data).backing;
    let mut backing_ctx: screen_write_ctx = screen_write_ctx {
        wp: ::core::ptr::null_mut::<window_pane>(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        item: ::core::ptr::null_mut::<screen_write_citem>(),
        scrolled: 0,
        bg: 0,
    };
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
    let mut old_hsize: u_int = 0;
    let mut old_cy: u_int = 0;
    let mut text: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    old_hsize = (*(*(*data).backing).grid).hsize;
    screen_write_start(&raw mut backing_ctx, backing);
    if (*data).backing_written != 0 {
        screen_write_carriagereturn(&raw mut backing_ctx);
        screen_write_linefeed(&raw mut backing_ctx, 0 as ::core::ffi::c_int, 8 as u_int);
    } else {
        (*data).backing_written = 1 as ::core::ffi::c_int;
    }
    old_cy = (*backing).cy;
    if parse != 0 {
        vasprintf(&raw mut text, fmt, ap);
        input_parse_screen(
            (*data).ictx,
            backing,
            Some(
                window_copy_init_ctx_cb
                    as unsafe extern "C" fn(*mut screen_write_ctx, *mut tty_ctx) -> (),
            ),
            data as *mut ::core::ffi::c_void,
            text as *const u_char,
            strlen(text),
        );
        free(text as *mut ::core::ffi::c_void);
    } else {
        memcpy(
            &raw mut gc as *mut ::core::ffi::c_void,
            &raw const grid_default_cell as *const ::core::ffi::c_void,
            ::core::mem::size_of::<grid_cell>() as size_t,
        );
        screen_write_vnputs(
            &raw mut backing_ctx,
            0 as ssize_t,
            &raw mut gc,
            fmt,
            ap,
        );
    }
    screen_write_stop(&raw mut backing_ctx);
    (*data).oy = (*data)
        .oy
        .wrapping_add((*(*(*data).backing).grid).hsize.wrapping_sub(old_hsize));
    screen_write_start_pane(&raw mut ctx, wp, &raw mut (*data).screen);
    if (*(*(*data).backing).grid).hsize != 0 {
        window_copy_redraw_lines(wme, 0 as u_int, 1 as u_int);
    }
    window_copy_redraw_lines(
        wme,
        old_cy,
        (*backing).cy.wrapping_sub(old_cy).wrapping_add(1 as u_int),
    );
    screen_write_stop(&raw mut ctx);
}
#[no_mangle]
pub unsafe extern "C" fn window_copy_scroll(
    mut wp: *mut window_pane,
    mut sl_mpos: ::core::ffi::c_int,
    mut my: u_int,
    mut tty_oy: u_int,
    mut scroll_exit: ::core::ffi::c_int,
) {
    let mut wme: *mut window_mode_entry = (*wp).modes.tqh_first;
    if !wme.is_null() {
        window_set_active_pane((*wp).window as *mut window, wp, 0 as ::core::ffi::c_int);
        window_copy_scroll1(wme, wp, sl_mpos, my, tty_oy, scroll_exit);
    }
}
unsafe extern "C" fn window_copy_scroll1(
    mut wme: *mut window_mode_entry,
    mut wp: *mut window_pane,
    mut sl_mpos: ::core::ffi::c_int,
    mut my: u_int,
    mut tty_oy: u_int,
    mut scroll_exit: ::core::ffi::c_int,
) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut ox: u_int = 0;
    let mut oy: u_int = 0;
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut n: u_int = 0;
    let mut offset: u_int = 0;
    let mut size: u_int = 0;
    let mut new_offset: u_int = 0;
    let mut slider_height: u_int = (*wp).sb_slider_h;
    let mut sb_height: u_int = (*wp).sy;
    let mut sb_top: u_int = (*wp).yoff as u_int;
    let mut sy: u_int = (*(*(*data).backing).grid).sy;
    let mut my_w: u_int = 0;
    let mut new_slider_y: ::core::ffi::c_int = 0;
    let mut delta: ::core::ffi::c_int = 0;
    my_w = my.wrapping_add(tty_oy);
    if my_w <= sb_top.wrapping_add(sl_mpos as u_int) {
        new_slider_y = sb_top.wrapping_sub((*wp).yoff as u_int) as ::core::ffi::c_int;
    } else if my_w.wrapping_sub(sl_mpos as u_int)
        > sb_top.wrapping_add(sb_height).wrapping_sub(slider_height)
    {
        new_slider_y = sb_top
            .wrapping_sub((*wp).yoff as u_int)
            .wrapping_add(sb_height.wrapping_sub(slider_height))
            as ::core::ffi::c_int;
    } else {
        new_slider_y = my_w
            .wrapping_sub((*wp).yoff as u_int)
            .wrapping_sub(sl_mpos as u_int) as ::core::ffi::c_int;
    }
    if (*wp).modes.tqh_first.is_null()
        || window_copy_get_current_offset(wp, &raw mut offset, &raw mut size)
            == 0 as ::core::ffi::c_int
    {
        return;
    }
    new_offset = (new_slider_y as ::core::ffi::c_float
        * (size.wrapping_add(sb_height) as ::core::ffi::c_float
            / sb_height as ::core::ffi::c_float)) as u_int;
    delta = (offset as ::core::ffi::c_int as u_int).wrapping_sub(new_offset) as ::core::ffi::c_int;
    oy = (*(*(*data).backing).grid)
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    ox = window_copy_find_length(wme, oy);
    if (*data).cx != ox {
        (*data).lastcx = (*data).cx;
        (*data).lastsx = ox;
    }
    (*data).cx = (*data).lastcx;
    if delta >= 0 as ::core::ffi::c_int {
        n = delta as u_int;
        if (*data).oy.wrapping_add(n) > (*(*(*data).backing).grid).hsize {
            (*data).oy = (*(*(*data).backing).grid).hsize;
            if (*data).cy < n {
                (*data).cy = 0 as u_int;
            } else {
                (*data).cy = (*data).cy.wrapping_sub(n);
            }
        } else {
            (*data).oy = (*data).oy.wrapping_add(n);
        }
    } else {
        n = -delta as u_int;
        if (*data).oy < n {
            (*data).oy = 0 as u_int;
            if (*data).cy.wrapping_add(n.wrapping_sub((*data).oy)) >= sy {
                (*data).cy = sy.wrapping_sub(1 as u_int);
            } else {
                (*data).cy = (*data).cy.wrapping_add(n.wrapping_sub((*data).oy));
            }
        } else {
            (*data).oy = (*data).oy.wrapping_sub(n);
        }
    }
    (*data).cursordrag = CURSORDRAG_NONE;
    if (*data).screen.sel.is_null() || (*data).rectflag == 0 {
        py = (*(*(*data).backing).grid)
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        px = window_copy_find_length(wme, py);
        if (*data).cx >= (*data).lastsx && (*data).cx != px || (*data).cx > px {
            window_copy_cursor_end_of_line(wme);
        }
    }
    if scroll_exit != 0 && (*data).oy == 0 as u_int && (*data).screen.sel.is_null() {
        window_pane_reset_mode(wp);
        return;
    }
    if !(*data).searchmark.is_null() && (*data).timeout == 0 {
        window_copy_search_marks(
            wme,
            ::core::ptr::null_mut::<screen>(),
            (*data).searchregex,
            1 as ::core::ffi::c_int,
        );
    }
    window_copy_update_selection_view(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    window_pane_scrollbar_show(wp, 1 as ::core::ffi::c_int);
    window_copy_redraw_screen(wme);
}
#[no_mangle]
pub unsafe extern "C" fn window_copy_pageup(
    mut wp: *mut window_pane,
    mut half_page: ::core::ffi::c_int,
) {
    window_copy_pageup1((*wp).modes.tqh_first, half_page);
}
unsafe extern "C" fn window_copy_pageup1(
    mut wme: *mut window_mode_entry,
    mut half_page: ::core::ffi::c_int,
) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut s: *mut screen = &raw mut (*data).screen;
    let mut n: u_int = 0;
    let mut ox: u_int = 0;
    let mut oy: u_int = 0;
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    oy = (*(*(*data).backing).grid)
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    ox = window_copy_find_length(wme, oy);
    if (*data).cx != ox {
        (*data).lastcx = (*data).cx;
        (*data).lastsx = ox;
    }
    (*data).cx = (*data).lastcx;
    n = 1 as u_int;
    if (*(*s).grid).sy > 2 as u_int {
        if half_page != 0 {
            n = (*(*s).grid).sy.wrapping_div(2 as u_int);
        } else {
            n = (*(*s).grid).sy.wrapping_sub(2 as u_int);
        }
    }
    if (*data).oy.wrapping_add(n) > (*(*(*data).backing).grid).hsize {
        (*data).oy = (*(*(*data).backing).grid).hsize;
        if (*data).cy < n {
            (*data).cy = 0 as u_int;
        } else {
            (*data).cy = (*data).cy.wrapping_sub(n);
        }
    } else {
        (*data).oy = (*data).oy.wrapping_add(n);
    }
    if (*data).screen.sel.is_null() || (*data).rectflag == 0 {
        py = (*(*(*data).backing).grid)
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        px = window_copy_find_length(wme, py);
        if (*data).cx >= (*data).lastsx && (*data).cx != px || (*data).cx > px {
            window_copy_cursor_end_of_line(wme);
        }
    }
    if !(*data).searchmark.is_null() && (*data).timeout == 0 {
        window_copy_search_marks(
            wme,
            ::core::ptr::null_mut::<screen>(),
            (*data).searchregex,
            1 as ::core::ffi::c_int,
        );
    }
    window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    window_pane_scrollbar_show((*wme).wp, 1 as ::core::ffi::c_int);
    window_copy_redraw_screen(wme);
}
#[no_mangle]
pub unsafe extern "C" fn window_copy_pagedown(
    mut wp: *mut window_pane,
    mut half_page: ::core::ffi::c_int,
    mut scroll_exit: ::core::ffi::c_int,
) {
    if window_copy_pagedown1((*wp).modes.tqh_first, half_page, scroll_exit) != 0 {
        window_pane_reset_mode(wp);
        return;
    }
}
unsafe extern "C" fn window_copy_pagedown1(
    mut wme: *mut window_mode_entry,
    mut half_page: ::core::ffi::c_int,
    mut scroll_exit: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut s: *mut screen = &raw mut (*data).screen;
    let mut n: u_int = 0;
    let mut ox: u_int = 0;
    let mut oy: u_int = 0;
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    oy = (*(*(*data).backing).grid)
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    ox = window_copy_find_length(wme, oy);
    if (*data).cx != ox {
        (*data).lastcx = (*data).cx;
        (*data).lastsx = ox;
    }
    (*data).cx = (*data).lastcx;
    n = 1 as u_int;
    if (*(*s).grid).sy > 2 as u_int {
        if half_page != 0 {
            n = (*(*s).grid).sy.wrapping_div(2 as u_int);
        } else {
            n = (*(*s).grid).sy.wrapping_sub(2 as u_int);
        }
    }
    if (*data).oy < n {
        (*data).oy = 0 as u_int;
        if (*data).cy.wrapping_add(n.wrapping_sub((*data).oy)) >= (*(*(*data).backing).grid).sy {
            (*data).cy = (*(*(*data).backing).grid).sy.wrapping_sub(1 as u_int);
        } else {
            (*data).cy = (*data).cy.wrapping_add(n.wrapping_sub((*data).oy));
        }
    } else {
        (*data).oy = (*data).oy.wrapping_sub(n);
    }
    if (*data).screen.sel.is_null() || (*data).rectflag == 0 {
        py = (*(*(*data).backing).grid)
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        px = window_copy_find_length(wme, py);
        if (*data).cx >= (*data).lastsx && (*data).cx != px || (*data).cx > px {
            window_copy_cursor_end_of_line(wme);
        }
    }
    if scroll_exit != 0 && (*data).oy == 0 as u_int && (*data).screen.sel.is_null() {
        return 1 as ::core::ffi::c_int;
    }
    if !(*data).searchmark.is_null() && (*data).timeout == 0 {
        window_copy_search_marks(
            wme,
            ::core::ptr::null_mut::<screen>(),
            (*data).searchregex,
            1 as ::core::ffi::c_int,
        );
    }
    window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    window_pane_scrollbar_show((*wme).wp, 1 as ::core::ffi::c_int);
    window_copy_redraw_screen(wme);
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn window_copy_previous_paragraph(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut oy: u_int = 0;
    oy = (*(*(*data).backing).grid)
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    while oy > 0 as u_int && window_copy_find_length(wme, oy) == 0 as u_int {
        oy = oy.wrapping_sub(1);
    }
    while oy > 0 as u_int && window_copy_find_length(wme, oy) > 0 as u_int {
        oy = oy.wrapping_sub(1);
    }
    window_copy_scroll_to(wme, 0 as u_int, oy, 0 as ::core::ffi::c_int);
}
unsafe extern "C" fn window_copy_next_paragraph(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut s: *mut screen = &raw mut (*data).screen;
    let mut maxy: u_int = 0;
    let mut ox: u_int = 0;
    let mut oy: u_int = 0;
    oy = (*(*(*data).backing).grid)
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    maxy = (*(*(*data).backing).grid)
        .hsize
        .wrapping_add((*(*s).grid).sy)
        .wrapping_sub(1 as u_int);
    while oy < maxy && window_copy_find_length(wme, oy) == 0 as u_int {
        oy = oy.wrapping_add(1);
    }
    while oy < maxy && window_copy_find_length(wme, oy) > 0 as u_int {
        oy = oy.wrapping_add(1);
    }
    ox = window_copy_find_length(wme, oy);
    window_copy_scroll_to(wme, ox, oy, 0 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn window_copy_get_word(
    mut wp: *mut window_pane,
    mut x: u_int,
    mut y: u_int,
) -> *mut ::core::ffi::c_char {
    let mut wme: *mut window_mode_entry = (*wp).modes.tqh_first;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut gd: *mut grid = (*(*data).backing).grid;
    return format_grid_word(gd, x, (*gd).hsize.wrapping_add(y).wrapping_sub((*data).oy));
}
#[no_mangle]
pub unsafe extern "C" fn window_copy_get_line(
    mut wp: *mut window_pane,
    mut y: u_int,
) -> *mut ::core::ffi::c_char {
    let mut wme: *mut window_mode_entry = (*wp).modes.tqh_first;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut gd: *mut grid = (*(*data).backing).grid;
    return format_grid_line(gd, (*gd).hsize.wrapping_add(y).wrapping_sub((*data).oy));
}
#[no_mangle]
pub unsafe extern "C" fn window_copy_get_hyperlink(
    mut wp: *mut window_pane,
    mut x: u_int,
    mut y: u_int,
) -> *mut ::core::ffi::c_char {
    let mut wme: *mut window_mode_entry = (*wp).modes.tqh_first;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut gd: *mut grid = (*data).screen.grid;
    return format_grid_hyperlink(gd, x, (*gd).hsize.wrapping_add(y), (*wp).screen);
}
unsafe extern "C" fn window_copy_cursor_hyperlink_cb(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = format_get_pane(ft);
    let mut wme: *mut window_mode_entry = (*wp).modes.tqh_first;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut gd: *mut grid = (*data).screen.grid;
    return format_grid_hyperlink(
        gd,
        (*data).cx,
        (*gd).hsize.wrapping_add((*data).cy),
        &raw mut (*data).screen,
    ) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn window_copy_cursor_word_cb(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = format_get_pane(ft);
    let mut wme: *mut window_mode_entry = (*wp).modes.tqh_first;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    return window_copy_get_word(wp, (*data).cx, (*data).cy) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn window_copy_cursor_line_cb(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = format_get_pane(ft);
    let mut wme: *mut window_mode_entry = (*wp).modes.tqh_first;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    return window_copy_get_line(wp, (*data).cy) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn window_copy_search_match_cb(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = format_get_pane(ft);
    let mut wme: *mut window_mode_entry = (*wp).modes.tqh_first;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    return window_copy_match_at_cursor(data) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn window_copy_formats(
    mut wme: *mut window_mode_entry,
    mut ft: *mut format_tree,
) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut hsize: u_int = (*(*(*data).backing).grid).hsize;
    let mut position: u_int = 0;
    let mut limit: u_int = 0;
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut t: time_t = 0;
    gl = grid_get_line((*(*data).backing).grid, hsize.wrapping_sub((*data).oy));
    t = grid_line_time(gl);
    format_add(
        ft,
        b"top_line_time\0" as *const u8 as *const ::core::ffi::c_char,
        b"%llu\0" as *const u8 as *const ::core::ffi::c_char,
        t as ::core::ffi::c_ulonglong,
    );
    format_add(
        ft,
        b"scroll_position\0" as *const u8 as *const ::core::ffi::c_char,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        (*data).oy,
    );
    if window_copy_line_number_is_absolute(wme) != 0 {
        position = hsize.wrapping_sub((*data).oy).wrapping_add(1 as u_int);
        limit = hsize.wrapping_add((*(*(*data).backing).grid).sy);
    } else {
        position = (*data).oy;
        limit = hsize;
    }
    format_add(
        ft,
        b"copy_position\0" as *const u8 as *const ::core::ffi::c_char,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        position,
    );
    format_add(
        ft,
        b"copy_position_limit\0" as *const u8 as *const ::core::ffi::c_char,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        limit,
    );
    format_add(
        ft,
        b"copy_line_numbers\0" as *const u8 as *const ::core::ffi::c_char,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        window_copy_line_numbers_active(wme),
    );
    format_add(
        ft,
        b"refresh_active\0" as *const u8 as *const ::core::ffi::c_char,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        (*data).refresh_active,
    );
    format_add(
        ft,
        b"rectangle_toggle\0" as *const u8 as *const ::core::ffi::c_char,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        (*data).rectflag,
    );
    format_add(
        ft,
        b"copy_cursor_x\0" as *const u8 as *const ::core::ffi::c_char,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        (*data).cx,
    );
    format_add(
        ft,
        b"copy_cursor_y\0" as *const u8 as *const ::core::ffi::c_char,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        (*data).cy,
    );
    if !(*data).screen.sel.is_null() {
        format_add(
            ft,
            b"selection_start_x\0" as *const u8 as *const ::core::ffi::c_char,
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            (*data).selx,
        );
        format_add(
            ft,
            b"selection_start_y\0" as *const u8 as *const ::core::ffi::c_char,
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            (*data).sely,
        );
        format_add(
            ft,
            b"selection_end_x\0" as *const u8 as *const ::core::ffi::c_char,
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            (*data).endselx,
        );
        format_add(
            ft,
            b"selection_end_y\0" as *const u8 as *const ::core::ffi::c_char,
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            (*data).endsely,
        );
        if (*data).cursordrag as ::core::ffi::c_uint
            != CURSORDRAG_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            format_add(
                ft,
                b"selection_active\0" as *const u8 as *const ::core::ffi::c_char,
                b"1\0" as *const u8 as *const ::core::ffi::c_char,
            );
        } else {
            format_add(
                ft,
                b"selection_active\0" as *const u8 as *const ::core::ffi::c_char,
                b"0\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        if (*data).endselx != (*data).selx || (*data).endsely != (*data).sely {
            format_add(
                ft,
                b"selection_present\0" as *const u8 as *const ::core::ffi::c_char,
                b"1\0" as *const u8 as *const ::core::ffi::c_char,
            );
        } else {
            format_add(
                ft,
                b"selection_present\0" as *const u8 as *const ::core::ffi::c_char,
                b"0\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
    } else {
        format_add(
            ft,
            b"selection_active\0" as *const u8 as *const ::core::ffi::c_char,
            b"0\0" as *const u8 as *const ::core::ffi::c_char,
        );
        format_add(
            ft,
            b"selection_present\0" as *const u8 as *const ::core::ffi::c_char,
            b"0\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    match (*data).selflag as ::core::ffi::c_uint {
        0 => {
            format_add(
                ft,
                b"selection_mode\0" as *const u8 as *const ::core::ffi::c_char,
                b"char\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        1 => {
            format_add(
                ft,
                b"selection_mode\0" as *const u8 as *const ::core::ffi::c_char,
                b"word\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        2 => {
            format_add(
                ft,
                b"selection_mode\0" as *const u8 as *const ::core::ffi::c_char,
                b"line\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        _ => {}
    }
    format_add(
        ft,
        b"search_present\0" as *const u8 as *const ::core::ffi::c_char,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        ((*data).searchmark != NULL as *mut u_char) as ::core::ffi::c_int,
    );
    format_add(
        ft,
        b"search_timed_out\0" as *const u8 as *const ::core::ffi::c_char,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        (*data).timeout,
    );
    if (*data).searchcount != -(1 as ::core::ffi::c_int) {
        format_add(
            ft,
            b"search_count\0" as *const u8 as *const ::core::ffi::c_char,
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            (*data).searchcount,
        );
        format_add(
            ft,
            b"search_count_partial\0" as *const u8 as *const ::core::ffi::c_char,
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            (*data).searchmore,
        );
    }
    format_add_cb(
        ft,
        b"search_match\0" as *const u8 as *const ::core::ffi::c_char,
        Some(
            window_copy_search_match_cb
                as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
        ),
    );
    format_add_cb(
        ft,
        b"copy_cursor_word\0" as *const u8 as *const ::core::ffi::c_char,
        Some(
            window_copy_cursor_word_cb
                as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
        ),
    );
    format_add_cb(
        ft,
        b"copy_cursor_line\0" as *const u8 as *const ::core::ffi::c_char,
        Some(
            window_copy_cursor_line_cb
                as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
        ),
    );
    format_add_cb(
        ft,
        b"copy_cursor_hyperlink\0" as *const u8 as *const ::core::ffi::c_char,
        Some(
            window_copy_cursor_hyperlink_cb
                as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
        ),
    );
}
unsafe extern "C" fn window_copy_get_screen(mut wme: *mut window_mode_entry) -> *mut screen {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    return (*data).backing;
}
unsafe extern "C" fn window_copy_size_changed(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut s: *mut screen = &raw mut (*data).screen;
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
    let mut search: ::core::ffi::c_int =
        ((*data).searchmark != NULL as *mut u_char) as ::core::ffi::c_int;
    window_copy_clear_selection(wme);
    window_copy_clear_marks(wme);
    screen_write_start(&raw mut ctx, s);
    window_copy_write_lines(wme, &raw mut ctx, 0 as u_int, (*(*s).grid).sy);
    screen_write_stop(&raw mut ctx);
    if search != 0 && (*data).timeout == 0 {
        window_copy_search_marks(
            wme,
            ::core::ptr::null_mut::<screen>(),
            (*data).searchregex,
            0 as ::core::ffi::c_int,
        );
    }
    (*data).searchx = (*data).cx as ::core::ffi::c_int;
    (*data).searchy = (*data).cy as ::core::ffi::c_int;
    (*data).searcho = (*data).oy as ::core::ffi::c_int;
}
unsafe extern "C" fn window_copy_resize(
    mut wme: *mut window_mode_entry,
    mut sx: u_int,
    mut sy: u_int,
) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut s: *mut screen = &raw mut (*data).screen;
    let mut gd: *mut grid = (*(*data).backing).grid;
    let mut cx: u_int = 0;
    let mut cy: u_int = 0;
    let mut wx: u_int = 0;
    let mut wy: u_int = 0;
    let mut reflow: ::core::ffi::c_int = 0;
    screen_resize(s, sx, sy, 0 as ::core::ffi::c_int);
    cx = (*data).cx;
    if (*data).oy > (*gd).hsize.wrapping_add((*data).cy) {
        (*data).oy = (*gd).hsize.wrapping_add((*data).cy);
    }
    cy = (*gd)
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    reflow = ((*gd).sx != sx) as ::core::ffi::c_int;
    if reflow != 0 {
        grid_wrap_position(gd, cx, cy, &raw mut wx, &raw mut wy);
    }
    screen_resize_cursor(
        (*data).backing,
        sx,
        sy,
        1 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    if reflow != 0 {
        grid_unwrap_position(gd, &raw mut cx, &raw mut cy, wx, wy);
    }
    (*data).cx = cx;
    if cy < (*gd).hsize {
        (*data).cy = 0 as u_int;
        (*data).oy = (*gd).hsize.wrapping_sub(cy);
    } else {
        (*data).cy = cy.wrapping_sub((*gd).hsize);
        (*data).oy = 0 as u_int;
    }
    window_copy_size_changed(wme);
    window_copy_redraw_screen(wme);
}
unsafe extern "C" fn window_copy_key_table(
    mut wme: *mut window_mode_entry,
) -> *const ::core::ffi::c_char {
    let mut wp: *mut window_pane = (*wme).wp;
    if options_get_number(
        (*(*wp).window).options,
        b"mode-keys\0" as *const u8 as *const ::core::ffi::c_char,
    ) == MODEKEY_VI as ::core::ffi::c_longlong
    {
        return b"copy-mode-vi\0" as *const u8 as *const ::core::ffi::c_char;
    }
    return b"copy-mode\0" as *const u8 as *const ::core::ffi::c_char;
}
unsafe extern "C" fn window_copy_expand_search_string(
    mut cs: *mut window_copy_cmd_state,
) -> ::core::ffi::c_int {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut ss: *const ::core::ffi::c_char = args_string((*cs).wargs, 0 as u_int);
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if ss.is_null() || *ss as ::core::ffi::c_int == '\0' as i32 {
        return 0 as ::core::ffi::c_int;
    }
    if args_has((*cs).args, 'F' as i32 as u_char) != 0 {
        expanded = format_single(
            ::core::ptr::null_mut::<cmdq_item>(),
            ss,
            ::core::ptr::null_mut::<client>(),
            ::core::ptr::null_mut::<session>(),
            ::core::ptr::null_mut::<winlink>(),
            (*wme).wp,
        );
        if *expanded as ::core::ffi::c_int == '\0' as i32 {
            free(expanded as *mut ::core::ffi::c_void);
            return 0 as ::core::ffi::c_int;
        }
        free((*data).searchstr as *mut ::core::ffi::c_void);
        (*data).searchstr = expanded;
    } else {
        free((*data).searchstr as *mut ::core::ffi::c_void);
        (*data).searchstr = xstrdup(ss);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn window_copy_cmd_append_selection(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut s: *mut session = (*cs).s;
    if !s.is_null() {
        window_copy_append_selection(wme);
    }
    window_copy_clear_selection(wme);
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe extern "C" fn window_copy_cmd_append_selection_and_cancel(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut s: *mut session = (*cs).s;
    if !s.is_null() {
        window_copy_append_selection(wme);
    }
    window_copy_clear_selection(wme);
    return WINDOW_COPY_CMD_CANCEL;
}
unsafe extern "C" fn window_copy_cmd_back_to_indentation(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    window_copy_cursor_back_to_indentation(wme);
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_begin_selection(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut c: *mut client = (*cs).c;
    let mut m: *mut mouse_event = (*cs).m;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    if !m.is_null() {
        window_copy_start_drag(c, m);
        return WINDOW_COPY_CMD_MOVE;
    }
    (*data).lineflag = LINE_SEL_NONE;
    (*data).selflag = SEL_CHAR;
    window_copy_start_selection(wme);
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe extern "C" fn window_copy_cmd_stop_selection(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    (*data).cursordrag = CURSORDRAG_NONE;
    (*data).lineflag = LINE_SEL_NONE;
    (*data).selflag = SEL_CHAR;
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_bottom_line(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    (*data).cx = 0 as u_int;
    (*data).cy = (*(*data).screen.grid).sy.wrapping_sub(1 as u_int);
    window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe extern "C" fn window_copy_cmd_cancel(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    return WINDOW_COPY_CMD_CANCEL;
}
unsafe extern "C" fn window_copy_cmd_clear_selection(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    window_copy_clear_selection(wme);
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe extern "C" fn window_copy_do_copy_end_of_line(
    mut cs: *mut window_copy_cmd_state,
    mut pipe: ::core::ffi::c_int,
    mut cancel: ::core::ffi::c_int,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut c: *mut client = (*cs).c;
    let mut s: *mut session = (*cs).s;
    let mut wl: *mut winlink = (*cs).wl;
    let mut wp: *mut window_pane = (*wme).wp;
    let mut count: u_int = args_count((*cs).wargs);
    let mut np: u_int = (*wme).prefix;
    let mut ocx: u_int = 0;
    let mut ocy: u_int = 0;
    let mut ooy: u_int = 0;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut prefix: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut command: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut arg0: *const ::core::ffi::c_char = args_string((*cs).wargs, 0 as u_int);
    let mut arg1: *const ::core::ffi::c_char = args_string((*cs).wargs, 1 as u_int);
    let mut set_paste: ::core::ffi::c_int =
        (args_has((*cs).wargs, 'P' as i32 as u_char) == 0) as ::core::ffi::c_int;
    let mut set_clip: ::core::ffi::c_int =
        (args_has((*cs).wargs, 'C' as i32 as u_char) == 0) as ::core::ffi::c_int;
    if pipe != 0 {
        if count == 2 as u_int {
            prefix = format_single(::core::ptr::null_mut::<cmdq_item>(), arg1, c, s, wl, wp);
        }
        if !s.is_null() && count > 0 as u_int && *arg0 as ::core::ffi::c_int != '\0' as i32 {
            command = format_single(::core::ptr::null_mut::<cmdq_item>(), arg0, c, s, wl, wp);
        }
    } else if count == 1 as u_int {
        prefix = format_single(::core::ptr::null_mut::<cmdq_item>(), arg0, c, s, wl, wp);
    }
    ocx = (*data).cx;
    ocy = (*data).cy;
    ooy = (*data).oy;
    window_copy_start_selection(wme);
    while np > 1 as u_int {
        window_copy_cursor_down(wme, 0 as ::core::ffi::c_int);
        np = np.wrapping_sub(1);
    }
    window_copy_cursor_end_of_line(wme);
    if !s.is_null() {
        if pipe != 0 {
            window_copy_copy_pipe(wme, s, prefix, command, set_paste, set_clip);
        } else {
            window_copy_copy_selection(wme, prefix, set_paste, set_clip);
        }
        if cancel != 0 {
            free(prefix as *mut ::core::ffi::c_void);
            free(command as *mut ::core::ffi::c_void);
            return WINDOW_COPY_CMD_CANCEL;
        }
    }
    window_copy_clear_selection(wme);
    (*data).cx = ocx;
    (*data).cy = ocy;
    (*data).oy = ooy;
    free(prefix as *mut ::core::ffi::c_void);
    free(command as *mut ::core::ffi::c_void);
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe extern "C" fn window_copy_cmd_copy_end_of_line(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    return window_copy_do_copy_end_of_line(cs, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
}
unsafe extern "C" fn window_copy_cmd_copy_end_of_line_and_cancel(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    return window_copy_do_copy_end_of_line(cs, 0 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
}
unsafe extern "C" fn window_copy_cmd_copy_pipe_end_of_line(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    return window_copy_do_copy_end_of_line(cs, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
}
unsafe extern "C" fn window_copy_cmd_copy_pipe_end_of_line_and_cancel(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    return window_copy_do_copy_end_of_line(cs, 1 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
}
unsafe extern "C" fn window_copy_do_copy_line(
    mut cs: *mut window_copy_cmd_state,
    mut pipe: ::core::ffi::c_int,
    mut cancel: ::core::ffi::c_int,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut c: *mut client = (*cs).c;
    let mut s: *mut session = (*cs).s;
    let mut wl: *mut winlink = (*cs).wl;
    let mut wp: *mut window_pane = (*wme).wp;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut count: u_int = args_count((*cs).wargs);
    let mut np: u_int = (*wme).prefix;
    let mut ocx: u_int = 0;
    let mut ocy: u_int = 0;
    let mut ooy: u_int = 0;
    let mut prefix: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut command: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut arg0: *const ::core::ffi::c_char = args_string((*cs).wargs, 0 as u_int);
    let mut arg1: *const ::core::ffi::c_char = args_string((*cs).wargs, 1 as u_int);
    let mut set_paste: ::core::ffi::c_int =
        (args_has((*cs).wargs, 'P' as i32 as u_char) == 0) as ::core::ffi::c_int;
    let mut set_clip: ::core::ffi::c_int =
        (args_has((*cs).wargs, 'C' as i32 as u_char) == 0) as ::core::ffi::c_int;
    if pipe != 0 {
        if count == 2 as u_int {
            prefix = format_single(::core::ptr::null_mut::<cmdq_item>(), arg1, c, s, wl, wp);
        }
        if !s.is_null() && count > 0 as u_int && *arg0 as ::core::ffi::c_int != '\0' as i32 {
            command = format_single(::core::ptr::null_mut::<cmdq_item>(), arg0, c, s, wl, wp);
        }
    } else if count == 1 as u_int {
        prefix = format_single(::core::ptr::null_mut::<cmdq_item>(), arg0, c, s, wl, wp);
    }
    ocx = (*data).cx;
    ocy = (*data).cy;
    ooy = (*data).oy;
    (*data).selflag = SEL_CHAR;
    window_copy_cursor_start_of_line(wme);
    window_copy_start_selection(wme);
    while np > 1 as u_int {
        window_copy_cursor_down(wme, 0 as ::core::ffi::c_int);
        np = np.wrapping_sub(1);
    }
    window_copy_cursor_end_of_line(wme);
    if !s.is_null() {
        if pipe != 0 {
            window_copy_copy_pipe(wme, s, prefix, command, set_paste, set_clip);
        } else {
            window_copy_copy_selection(wme, prefix, set_paste, set_clip);
        }
        if cancel != 0 {
            free(prefix as *mut ::core::ffi::c_void);
            free(command as *mut ::core::ffi::c_void);
            return WINDOW_COPY_CMD_CANCEL;
        }
    }
    window_copy_clear_selection(wme);
    (*data).cx = ocx;
    (*data).cy = ocy;
    (*data).oy = ooy;
    free(prefix as *mut ::core::ffi::c_void);
    free(command as *mut ::core::ffi::c_void);
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe extern "C" fn window_copy_cmd_copy_line(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    return window_copy_do_copy_line(cs, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
}
unsafe extern "C" fn window_copy_cmd_copy_line_and_cancel(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    return window_copy_do_copy_line(cs, 0 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
}
unsafe extern "C" fn window_copy_cmd_copy_pipe_line(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    return window_copy_do_copy_line(cs, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
}
unsafe extern "C" fn window_copy_cmd_copy_pipe_line_and_cancel(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    return window_copy_do_copy_line(cs, 1 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
}
unsafe extern "C" fn window_copy_cmd_copy_selection_no_clear(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut c: *mut client = (*cs).c;
    let mut s: *mut session = (*cs).s;
    let mut wl: *mut winlink = (*cs).wl;
    let mut wp: *mut window_pane = (*wme).wp;
    let mut prefix: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut arg0: *const ::core::ffi::c_char = args_string((*cs).wargs, 0 as u_int);
    let mut set_paste: ::core::ffi::c_int =
        (args_has((*cs).wargs, 'P' as i32 as u_char) == 0) as ::core::ffi::c_int;
    let mut set_clip: ::core::ffi::c_int =
        (args_has((*cs).wargs, 'C' as i32 as u_char) == 0) as ::core::ffi::c_int;
    if !arg0.is_null() {
        prefix = format_single(::core::ptr::null_mut::<cmdq_item>(), arg0, c, s, wl, wp);
    }
    if !s.is_null() {
        window_copy_copy_selection(wme, prefix, set_paste, set_clip);
    }
    free(prefix as *mut ::core::ffi::c_void);
    return WINDOW_COPY_CMD_NOTHING;
}
unsafe extern "C" fn window_copy_cmd_copy_selection(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    window_copy_cmd_copy_selection_no_clear(cs);
    window_copy_clear_selection(wme);
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe extern "C" fn window_copy_cmd_copy_selection_and_cancel(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    window_copy_cmd_copy_selection_no_clear(cs);
    window_copy_clear_selection(wme);
    return WINDOW_COPY_CMD_CANCEL;
}
unsafe extern "C" fn window_copy_cmd_cursor_down(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut np: u_int = (*wme).prefix;
    while np != 0 as u_int {
        window_copy_cursor_down(wme, 0 as ::core::ffi::c_int);
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_cursor_down_and_cancel(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut np: u_int = (*wme).prefix;
    let mut cy: u_int = 0;
    cy = (*data).cy;
    while np != 0 as u_int {
        window_copy_cursor_down(wme, 0 as ::core::ffi::c_int);
        np = np.wrapping_sub(1);
    }
    if cy == (*data).cy && (*data).oy == 0 as u_int {
        return WINDOW_COPY_CMD_CANCEL;
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_cursor_left(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut np: u_int = (*wme).prefix;
    while np != 0 as u_int {
        window_copy_cursor_left(wme);
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_cursor_right(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut np: u_int = (*wme).prefix;
    while np != 0 as u_int {
        window_copy_cursor_right(
            wme,
            (!(*data).screen.sel.is_null() && (*data).rectflag != 0) as ::core::ffi::c_int,
        );
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_scroll_to(
    mut cs: *mut window_copy_cmd_state,
    mut to: u_int,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut oy: u_int = 0;
    let mut delta: u_int = 0;
    let mut scroll_up: ::core::ffi::c_int = 0;
    scroll_up = (*data).cy.wrapping_sub(to) as ::core::ffi::c_int;
    delta = abs(scroll_up) as u_int;
    oy = (*(*(*data).backing).grid).hsize.wrapping_sub((*data).oy);
    if scroll_up > 0 as ::core::ffi::c_int && (*data).oy >= delta {
        window_copy_scroll_up(wme, delta);
        (*data).cy = (*data).cy.wrapping_sub(delta);
    } else if scroll_up < 0 as ::core::ffi::c_int && oy >= delta {
        window_copy_scroll_down(wme, delta);
        (*data).cy = (*data).cy.wrapping_add(delta);
    }
    window_copy_update_selection_view(wme, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe extern "C" fn window_copy_cmd_scroll_bottom(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut data: *mut window_copy_mode_data = (*(*cs).wme).data as *mut window_copy_mode_data;
    let mut bottom: u_int = 0;
    bottom = (*(*data).screen.grid).sy.wrapping_sub(1 as u_int);
    return window_copy_cmd_scroll_to(cs, bottom);
}
unsafe extern "C" fn window_copy_cmd_scroll_middle(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut data: *mut window_copy_mode_data = (*(*cs).wme).data as *mut window_copy_mode_data;
    let mut mid_value: u_int = 0;
    mid_value = (*(*data).screen.grid)
        .sy
        .wrapping_sub(1 as u_int)
        .wrapping_div(2 as u_int);
    return window_copy_cmd_scroll_to(cs, mid_value);
}
unsafe extern "C" fn window_copy_cmd_scroll_to_mouse(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut wp: *mut window_pane = (*wme).wp;
    let mut c: *mut client = (*cs).c;
    let mut m: *mut mouse_event = (*cs).m;
    let mut scroll_exit: ::core::ffi::c_int = args_has((*cs).wargs, 'e' as i32 as u_char);
    let mut tty_ox: u_int = 0;
    let mut tty_oy: u_int = 0;
    let mut tty_sx: u_int = 0;
    let mut tty_sy: u_int = 0;
    tty_window_offset(
        &raw mut (*c).tty,
        &raw mut tty_ox,
        &raw mut tty_oy,
        &raw mut tty_sx,
        &raw mut tty_sy,
    );
    window_copy_scroll(wp, (*c).tty.mouse_slider_mpos, (*m).y, tty_oy, scroll_exit);
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_scroll_top(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    return window_copy_cmd_scroll_to(cs, 0 as u_int);
}
unsafe extern "C" fn window_copy_cmd_cursor_up(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut np: u_int = (*wme).prefix;
    while np != 0 as u_int {
        window_copy_cursor_up(wme, 0 as ::core::ffi::c_int);
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_centre_vertical(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    window_copy_update_cursor(wme, (*data).cx, (*(*wme).wp).sy.wrapping_div(2 as u_int));
    window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe extern "C" fn window_copy_cmd_centre_horizontal(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    window_copy_update_cursor(wme, (*(*wme).wp).sx.wrapping_div(2 as u_int), (*data).cy);
    window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe extern "C" fn window_copy_cmd_end_of_line(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    window_copy_cursor_end_of_line(wme);
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_halfpage_down(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut np: u_int = (*wme).prefix;
    while np != 0 as u_int {
        if window_copy_pagedown1(wme, 1 as ::core::ffi::c_int, (*data).scroll_exit) != 0 {
            return WINDOW_COPY_CMD_CANCEL;
        }
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_halfpage_down_and_cancel(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut np: u_int = (*wme).prefix;
    while np != 0 as u_int {
        if window_copy_pagedown1(wme, 1 as ::core::ffi::c_int, 1 as ::core::ffi::c_int) != 0 {
            return WINDOW_COPY_CMD_CANCEL;
        }
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_halfpage_up(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut np: u_int = (*wme).prefix;
    while np != 0 as u_int {
        window_copy_pageup1(wme, 1 as ::core::ffi::c_int);
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_toggle_position(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    (*data).hide_position = ((*data).hide_position == 0) as ::core::ffi::c_int;
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe extern "C" fn window_copy_cmd_history_bottom(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut s: *mut screen = (*data).backing;
    let mut oy: u_int = 0;
    let mut old_oy: u_int = (*data).oy;
    oy = (*(*s).grid)
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    if (*data).lineflag as ::core::ffi::c_uint
        == LINE_SEL_RIGHT_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
        && oy == (*data).endsely
    {
        window_copy_other_end(wme);
    }
    (*data).cy = (*(*data).screen.grid).sy.wrapping_sub(1 as u_int);
    (*data).cx = window_copy_cursor_limit(
        wme,
        (*(*s).grid).hsize.wrapping_add((*data).cy),
        0 as ::core::ffi::c_int,
    );
    (*data).oy = 0 as u_int;
    if !(*data).searchmark.is_null() && (*data).timeout == 0 {
        window_copy_search_marks(
            wme,
            ::core::ptr::null_mut::<screen>(),
            (*data).searchregex,
            1 as ::core::ffi::c_int,
        );
    }
    window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    if (*data).oy != old_oy {
        window_pane_scrollbar_show((*wme).wp, 1 as ::core::ffi::c_int);
    }
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe extern "C" fn window_copy_cmd_history_top(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut oy: u_int = 0;
    let mut old_oy: u_int = (*data).oy;
    oy = (*(*(*data).backing).grid)
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    if (*data).lineflag as ::core::ffi::c_uint
        == LINE_SEL_LEFT_RIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
        && oy == (*data).sely
    {
        window_copy_other_end(wme);
    }
    (*data).cy = 0 as u_int;
    (*data).cx = 0 as u_int;
    (*data).oy = (*(*(*data).backing).grid).hsize;
    if !(*data).searchmark.is_null() && (*data).timeout == 0 {
        window_copy_search_marks(
            wme,
            ::core::ptr::null_mut::<screen>(),
            (*data).searchregex,
            1 as ::core::ffi::c_int,
        );
    }
    window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    if (*data).oy != old_oy {
        window_pane_scrollbar_show((*wme).wp, 1 as ::core::ffi::c_int);
    }
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe extern "C" fn window_copy_cmd_jump_again(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut np: u_int = (*wme).prefix;
    match (*data).jumptype {
        3 => {
            while np != 0 as u_int {
                window_copy_cursor_jump(wme);
                np = np.wrapping_sub(1);
            }
        }
        4 => {
            while np != 0 as u_int {
                window_copy_cursor_jump_back(wme);
                np = np.wrapping_sub(1);
            }
        }
        5 => {
            while np != 0 as u_int {
                window_copy_cursor_jump_to(wme);
                np = np.wrapping_sub(1);
            }
        }
        6 => {
            while np != 0 as u_int {
                window_copy_cursor_jump_to_back(wme);
                np = np.wrapping_sub(1);
            }
        }
        _ => {}
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_jump_reverse(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut np: u_int = (*wme).prefix;
    match (*data).jumptype {
        3 => {
            while np != 0 as u_int {
                window_copy_cursor_jump_back(wme);
                np = np.wrapping_sub(1);
            }
        }
        4 => {
            while np != 0 as u_int {
                window_copy_cursor_jump(wme);
                np = np.wrapping_sub(1);
            }
        }
        5 => {
            while np != 0 as u_int {
                window_copy_cursor_jump_to_back(wme);
                np = np.wrapping_sub(1);
            }
        }
        6 => {
            while np != 0 as u_int {
                window_copy_cursor_jump_to(wme);
                np = np.wrapping_sub(1);
            }
        }
        _ => {}
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_middle_line(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    (*data).cx = 0 as u_int;
    (*data).cy = (*(*data).screen.grid)
        .sy
        .wrapping_sub(1 as u_int)
        .wrapping_div(2 as u_int);
    window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe extern "C" fn window_copy_cmd_previous_matching_bracket(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut np: u_int = (*wme).prefix;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut s: *mut screen = (*data).backing;
    let mut open: [::core::ffi::c_char; 4] =
        ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"{[(\0");
    let mut close: [::core::ffi::c_char; 4] =
        ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"}])\0");
    let mut tried: ::core::ffi::c_char = 0;
    let mut found: ::core::ffi::c_char = 0;
    let mut start: ::core::ffi::c_char = 0;
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut xx: u_int = 0;
    let mut n: u_int = 0;
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
    let mut failed: ::core::ffi::c_int = 0;
    while np != 0 as u_int {
        px = (*data).cx;
        py = (*(*s).grid)
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        xx = window_copy_find_length(wme, py);
        if xx == 0 as u_int {
            break;
        }
        tried = 0 as ::core::ffi::c_char;
        loop {
            grid_get_cell((*s).grid, px, py, &raw mut gc);
            if gc.data.size as ::core::ffi::c_int != 1 as ::core::ffi::c_int
                || gc.flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0
            {
                cp = ::core::ptr::null_mut::<::core::ffi::c_char>();
            } else {
                found = *(&raw mut gc.data.data as *mut u_char) as ::core::ffi::c_char;
                cp = strchr(
                    &raw mut close as *mut ::core::ffi::c_char,
                    found as ::core::ffi::c_int,
                );
            }
            if cp.is_null() {
                if !((*data).modekeys == MODEKEY_EMACS) {
                    break;
                }
                if tried == 0 && px > 0 as u_int {
                    px = px.wrapping_sub(1);
                    tried = 1 as ::core::ffi::c_char;
                } else {
                    window_copy_cursor_previous_word(
                        wme,
                        &raw mut close as *mut ::core::ffi::c_char,
                        1 as ::core::ffi::c_int,
                    );
                    break;
                }
            } else {
                start = open[cp.offset_from(&raw mut close as *mut ::core::ffi::c_char)
                    as ::core::ffi::c_long as usize];
                n = 1 as u_int;
                failed = 0 as ::core::ffi::c_int;
                loop {
                    if px == 0 as u_int {
                        if py == 0 as u_int {
                            failed = 1 as ::core::ffi::c_int;
                            break;
                        } else {
                            loop {
                                py = py.wrapping_sub(1);
                                xx = window_copy_find_length(wme, py);
                                if !(xx == 0 as u_int && py > 0 as u_int) {
                                    break;
                                }
                            }
                            if xx == 0 as u_int && py == 0 as u_int {
                                failed = 1 as ::core::ffi::c_int;
                                break;
                            } else {
                                px = xx.wrapping_sub(1 as u_int);
                            }
                        }
                    } else {
                        px = px.wrapping_sub(1);
                    }
                    grid_get_cell((*s).grid, px, py, &raw mut gc);
                    if gc.data.size as ::core::ffi::c_int == 1 as ::core::ffi::c_int
                        && !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_PADDING != 0
                    {
                        if *(&raw mut gc.data.data as *mut u_char) as ::core::ffi::c_int
                            == found as ::core::ffi::c_int
                        {
                            n = n.wrapping_add(1);
                        } else if *(&raw mut gc.data.data as *mut u_char) as ::core::ffi::c_int
                            == start as ::core::ffi::c_int
                        {
                            n = n.wrapping_sub(1);
                        }
                    }
                    if !(n != 0 as u_int) {
                        break;
                    }
                }
                if failed == 0 {
                    window_copy_scroll_to(wme, px, py, 0 as ::core::ffi::c_int);
                }
                break;
            }
        }
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_next_matching_bracket(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut np: u_int = (*wme).prefix;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut s: *mut screen = (*data).backing;
    let mut open: [::core::ffi::c_char; 4] =
        ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"{[(\0");
    let mut close: [::core::ffi::c_char; 4] =
        ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"}])\0");
    let mut tried: ::core::ffi::c_char = 0;
    let mut found: ::core::ffi::c_char = 0;
    let mut end: ::core::ffi::c_char = 0;
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut xx: u_int = 0;
    let mut yy: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut n: u_int = 0;
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
    let mut failed: ::core::ffi::c_int = 0;
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    's_22: while np != 0 as u_int {
        px = (*data).cx;
        py = (*(*s).grid)
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        xx = window_copy_find_length(wme, py);
        yy = (*(*s).grid)
            .hsize
            .wrapping_add((*(*s).grid).sy)
            .wrapping_sub(1 as u_int);
        if xx == 0 as u_int {
            break;
        }
        tried = 0 as ::core::ffi::c_char;
        loop {
            grid_get_cell((*s).grid, px, py, &raw mut gc);
            if gc.data.size as ::core::ffi::c_int != 1 as ::core::ffi::c_int
                || gc.flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0
            {
                cp = ::core::ptr::null_mut::<::core::ffi::c_char>();
            } else {
                found = *(&raw mut gc.data.data as *mut u_char) as ::core::ffi::c_char;
                cp = strchr(
                    &raw mut close as *mut ::core::ffi::c_char,
                    found as ::core::ffi::c_int,
                );
                if !cp.is_null() && (*data).modekeys == MODEKEY_VI {
                    sx = (*data).cx;
                    sy = (*(*s).grid)
                        .hsize
                        .wrapping_add((*data).cy)
                        .wrapping_sub((*data).oy);
                    window_copy_scroll_to(wme, px, py, 0 as ::core::ffi::c_int);
                    window_copy_cmd_previous_matching_bracket(cs);
                    px = (*data).cx;
                    py = (*(*s).grid)
                        .hsize
                        .wrapping_add((*data).cy)
                        .wrapping_sub((*data).oy);
                    grid_get_cell((*s).grid, px, py, &raw mut gc);
                    if gc.data.size as ::core::ffi::c_int == 1 as ::core::ffi::c_int
                        && !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_PADDING != 0
                        && !strchr(
                            &raw mut close as *mut ::core::ffi::c_char,
                            *(&raw mut gc.data.data as *mut u_char) as ::core::ffi::c_int,
                        )
                        .is_null()
                    {
                        window_copy_scroll_to(wme, sx, sy, 0 as ::core::ffi::c_int);
                    }
                    break 's_22;
                } else {
                    cp = strchr(
                        &raw mut open as *mut ::core::ffi::c_char,
                        found as ::core::ffi::c_int,
                    );
                }
            }
            if cp.is_null() {
                if (*data).modekeys == MODEKEY_EMACS {
                    if tried == 0 && px <= xx {
                        px = px.wrapping_add(1);
                        tried = 1 as ::core::ffi::c_char;
                    } else {
                        window_copy_cursor_next_word_end(
                            wme,
                            &raw mut open as *mut ::core::ffi::c_char,
                            0 as ::core::ffi::c_int,
                        );
                        break;
                    }
                } else if px > xx {
                    if py == yy {
                        break;
                    }
                    gl = grid_get_line((*s).grid, py);
                    if !((*gl).flags as ::core::ffi::c_int) & GRID_LINE_WRAPPED != 0 {
                        break;
                    }
                    if (*gl).cellsize as u_int > (*(*s).grid).sx {
                        break;
                    }
                    px = 0 as u_int;
                    py = py.wrapping_add(1);
                    xx = window_copy_find_length(wme, py);
                } else {
                    px = px.wrapping_add(1);
                }
            } else {
                end = close[cp.offset_from(&raw mut open as *mut ::core::ffi::c_char)
                    as ::core::ffi::c_long as usize];
                n = 1 as u_int;
                failed = 0 as ::core::ffi::c_int;
                loop {
                    if px > xx {
                        if py == yy {
                            failed = 1 as ::core::ffi::c_int;
                            break;
                        } else {
                            px = 0 as u_int;
                            py = py.wrapping_add(1);
                            xx = window_copy_find_length(wme, py);
                        }
                    } else {
                        px = px.wrapping_add(1);
                    }
                    grid_get_cell((*s).grid, px, py, &raw mut gc);
                    if gc.data.size as ::core::ffi::c_int == 1 as ::core::ffi::c_int
                        && !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_PADDING != 0
                    {
                        if *(&raw mut gc.data.data as *mut u_char) as ::core::ffi::c_int
                            == found as ::core::ffi::c_int
                        {
                            n = n.wrapping_add(1);
                        } else if *(&raw mut gc.data.data as *mut u_char) as ::core::ffi::c_int
                            == end as ::core::ffi::c_int
                        {
                            n = n.wrapping_sub(1);
                        }
                    }
                    if !(n != 0 as u_int) {
                        break;
                    }
                }
                if failed == 0 {
                    window_copy_scroll_to(wme, px, py, 0 as ::core::ffi::c_int);
                }
                break;
            }
        }
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_next_paragraph(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut np: u_int = (*wme).prefix;
    while np != 0 as u_int {
        window_copy_next_paragraph(wme);
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_next_space(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut np: u_int = (*wme).prefix;
    while np != 0 as u_int {
        window_copy_cursor_next_word(wme, b"\0" as *const u8 as *const ::core::ffi::c_char);
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_next_space_end(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut np: u_int = (*wme).prefix;
    while np != 0 as u_int {
        window_copy_cursor_next_word_end(
            wme,
            b"\0" as *const u8 as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
        );
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_next_word(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut np: u_int = (*wme).prefix;
    let mut separators: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    separators = options_get_string(
        (*(*cs).s).options,
        b"word-separators\0" as *const u8 as *const ::core::ffi::c_char,
    );
    while np != 0 as u_int {
        window_copy_cursor_next_word(wme, separators);
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_next_word_end(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut np: u_int = (*wme).prefix;
    let mut separators: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    separators = options_get_string(
        (*(*cs).s).options,
        b"word-separators\0" as *const u8 as *const ::core::ffi::c_char,
    );
    while np != 0 as u_int {
        window_copy_cursor_next_word_end(wme, separators, 0 as ::core::ffi::c_int);
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_other_end(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut np: u_int = (*wme).prefix;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    (*data).selflag = SEL_CHAR;
    if np.wrapping_rem(2 as u_int) != 0 as u_int {
        window_copy_other_end(wme);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_selection_mode(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut so: *mut options = (*(*cs).s).options;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut gr: grid_reader = grid_reader {
        gd: ::core::ptr::null_mut::<grid>(),
        cx: 0,
        cy: 0,
    };
    let mut s: *const ::core::ffi::c_char = args_string((*cs).wargs, 0 as u_int);
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut ex: u_int = 0;
    let mut ey: u_int = 0;
    let mut fx: u_int = 0;
    let mut fy: u_int = 0;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    if s.is_null()
        || strcasecmp(s, b"char\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        || strcasecmp(s, b"c\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
    {
        (*data).selflag = SEL_CHAR;
    } else if strcasecmp(s, b"word\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
        || strcasecmp(s, b"w\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
    {
        (*data).separators = options_get_string(
            so,
            b"word-separators\0" as *const u8 as *const ::core::ffi::c_char,
        );
        (*data).selflag = SEL_WORD;
    } else if strcasecmp(s, b"line\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
        || strcasecmp(s, b"l\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
    {
        (*data).selflag = SEL_LINE;
        if (*data).screen.sel.is_null() {
            return WINDOW_COPY_CMD_MOVE;
        }
        if (*data).cursordrag as ::core::ffi::c_uint
            == CURSORDRAG_SEL as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            fx = (*data).endselx;
            fy = (*data).endsely;
        } else {
            fx = (*data).selx;
            fy = (*data).sely;
        }
        sx = (*data).selx;
        sy = (*data).sely;
        ex = (*data).endselx;
        ey = (*data).endsely;
        if ey < sy || ey == sy && ex < sx {
            x = sx;
            sx = ex;
            ex = x;
            y = sy;
            sy = ey;
            ey = y;
        }
        grid_reader_start(&raw mut gr, (*(*data).backing).grid, sx, sy);
        grid_reader_cursor_start_of_line(&raw mut gr, 1 as ::core::ffi::c_int);
        grid_reader_get_cursor(&raw mut gr, &raw mut sx, &raw mut sy);
        grid_reader_start(&raw mut gr, (*(*data).backing).grid, ex, ey);
        grid_reader_cursor_end_of_line(
            &raw mut gr,
            1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        grid_reader_get_cursor(&raw mut gr, &raw mut ex, &raw mut ey);
        (*data).rectflag = 0 as ::core::ffi::c_int;
        (*data).selx = sx;
        (*data).selrx = (*data).selx;
        (*data).sely = sy;
        (*data).selry = (*data).sely;
        (*data).endselx = ex;
        (*data).endselrx = (*data).endselx;
        (*data).endsely = ey;
        (*data).endselry = (*data).endsely;
        x = (*data).cx;
        y = (*(*(*data).backing).grid)
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        (*data).dx = fx;
        (*data).dy = fy;
        if (*data).cursordrag as ::core::ffi::c_uint
            != CURSORDRAG_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
            && (y < fy || y == fy && x < fx)
        {
            (*data).lineflag = LINE_SEL_RIGHT_LEFT;
            (*data).cursordrag = CURSORDRAG_SEL;
            window_copy_scroll_to(wme, sx, sy, 1 as ::core::ffi::c_int);
        } else {
            (*data).lineflag = LINE_SEL_LEFT_RIGHT;
            if (*data).cursordrag as ::core::ffi::c_uint
                != CURSORDRAG_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*data).cursordrag = CURSORDRAG_ENDSEL;
                x = window_copy_cursor_limit(wme, ey, 0 as ::core::ffi::c_int);
                window_copy_scroll_to(wme, x, ey, 1 as ::core::ffi::c_int);
            }
        }
        if (*data).cursordrag as ::core::ffi::c_uint
            == CURSORDRAG_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            window_copy_set_selection(wme, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        }
        return WINDOW_COPY_CMD_REDRAW;
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_page_down(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut np: u_int = (*wme).prefix;
    while np != 0 as u_int {
        if window_copy_pagedown1(wme, 0 as ::core::ffi::c_int, (*data).scroll_exit) != 0 {
            return WINDOW_COPY_CMD_CANCEL;
        }
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_page_down_and_cancel(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut np: u_int = (*wme).prefix;
    while np != 0 as u_int {
        if window_copy_pagedown1(wme, 0 as ::core::ffi::c_int, 1 as ::core::ffi::c_int) != 0 {
            return WINDOW_COPY_CMD_CANCEL;
        }
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_page_up(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut np: u_int = (*wme).prefix;
    while np != 0 as u_int {
        window_copy_pageup1(wme, 0 as ::core::ffi::c_int);
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_previous_paragraph(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut np: u_int = (*wme).prefix;
    while np != 0 as u_int {
        window_copy_previous_paragraph(wme);
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_previous_space(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut np: u_int = (*wme).prefix;
    while np != 0 as u_int {
        window_copy_cursor_previous_word(
            wme,
            b"\0" as *const u8 as *const ::core::ffi::c_char,
            1 as ::core::ffi::c_int,
        );
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_previous_word(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut np: u_int = (*wme).prefix;
    let mut separators: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    separators = options_get_string(
        (*(*cs).s).options,
        b"word-separators\0" as *const u8 as *const ::core::ffi::c_char,
    );
    while np != 0 as u_int {
        window_copy_cursor_previous_word(wme, separators, 1 as ::core::ffi::c_int);
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_rectangle_on(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    (*data).lineflag = LINE_SEL_NONE;
    window_copy_rectangle_set(wme, 1 as ::core::ffi::c_int);
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_rectangle_off(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    (*data).lineflag = LINE_SEL_NONE;
    window_copy_rectangle_set(wme, 0 as ::core::ffi::c_int);
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_rectangle_toggle(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    (*data).lineflag = LINE_SEL_NONE;
    window_copy_rectangle_set(wme, ((*data).rectflag == 0) as ::core::ffi::c_int);
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_scroll_exit_on(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut data: *mut window_copy_mode_data = (*(*cs).wme).data as *mut window_copy_mode_data;
    (*data).scroll_exit = 1 as ::core::ffi::c_int;
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_scroll_exit_off(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut data: *mut window_copy_mode_data = (*(*cs).wme).data as *mut window_copy_mode_data;
    (*data).scroll_exit = 0 as ::core::ffi::c_int;
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_scroll_exit_toggle(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut data: *mut window_copy_mode_data = (*(*cs).wme).data as *mut window_copy_mode_data;
    (*data).scroll_exit = ((*data).scroll_exit == 0) as ::core::ffi::c_int;
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_scroll_down(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut np: u_int = (*wme).prefix;
    let mut dragging: ::core::ffi::c_int = 0;
    if (*data).oy == 0 as u_int {
        if (*data).scroll_exit != 0 && (*data).screen.sel.is_null() {
            return WINDOW_COPY_CMD_CANCEL;
        }
        return WINDOW_COPY_CMD_NOTHING;
    }
    dragging = (!(*cs).c.is_null() && (*(*cs).c).tty.mouse_drag_flag != 0 as ::core::ffi::c_int)
        as ::core::ffi::c_int;
    if !(*data).screen.sel.is_null() && dragging == 0 {
        (*data).cursordrag = CURSORDRAG_NONE;
        (*data).lineflag = LINE_SEL_NONE;
        window_copy_scroll_up(wme, np);
        return WINDOW_COPY_CMD_NOTHING;
    }
    while np != 0 as u_int {
        window_copy_cursor_down(wme, 1 as ::core::ffi::c_int);
        np = np.wrapping_sub(1);
    }
    if (*data).scroll_exit != 0 && (*data).oy == 0 as u_int && (*data).screen.sel.is_null() {
        return WINDOW_COPY_CMD_CANCEL;
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_scroll_down_and_cancel(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut np: u_int = (*wme).prefix;
    let mut dragging: ::core::ffi::c_int = 0;
    dragging = (!(*cs).c.is_null() && (*(*cs).c).tty.mouse_drag_flag != 0 as ::core::ffi::c_int)
        as ::core::ffi::c_int;
    if !(*data).screen.sel.is_null() && dragging == 0 {
        (*data).cursordrag = CURSORDRAG_NONE;
        (*data).lineflag = LINE_SEL_NONE;
        window_copy_scroll_up(wme, np);
        return WINDOW_COPY_CMD_NOTHING;
    }
    while np != 0 as u_int {
        window_copy_cursor_down(wme, 1 as ::core::ffi::c_int);
        np = np.wrapping_sub(1);
    }
    if (*data).oy == 0 as u_int {
        return WINDOW_COPY_CMD_CANCEL;
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_scroll_up(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut np: u_int = (*wme).prefix;
    let mut dragging: ::core::ffi::c_int = 0;
    if (*data).oy == (*(*(*data).backing).grid).hsize {
        return WINDOW_COPY_CMD_NOTHING;
    }
    dragging = (!(*cs).c.is_null() && (*(*cs).c).tty.mouse_drag_flag != 0 as ::core::ffi::c_int)
        as ::core::ffi::c_int;
    if !(*data).screen.sel.is_null() && dragging == 0 {
        (*data).cursordrag = CURSORDRAG_NONE;
        (*data).lineflag = LINE_SEL_NONE;
        window_copy_scroll_down(wme, np);
        return WINDOW_COPY_CMD_NOTHING;
    }
    while np != 0 as u_int {
        window_copy_cursor_up(wme, 1 as ::core::ffi::c_int);
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_search_again(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut np: u_int = (*wme).prefix;
    if (*data).searchtype == WINDOW_COPY_SEARCHUP as ::core::ffi::c_int {
        while np != 0 as u_int {
            window_copy_search_up(wme, (*data).searchregex);
            np = np.wrapping_sub(1);
        }
    } else if (*data).searchtype == WINDOW_COPY_SEARCHDOWN as ::core::ffi::c_int {
        while np != 0 as u_int {
            window_copy_search_down(wme, (*data).searchregex);
            np = np.wrapping_sub(1);
        }
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_search_reverse(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut np: u_int = (*wme).prefix;
    if (*data).searchtype == WINDOW_COPY_SEARCHUP as ::core::ffi::c_int {
        while np != 0 as u_int {
            window_copy_search_down(wme, (*data).searchregex);
            np = np.wrapping_sub(1);
        }
    } else if (*data).searchtype == WINDOW_COPY_SEARCHDOWN as ::core::ffi::c_int {
        while np != 0 as u_int {
            window_copy_search_up(wme, (*data).searchregex);
            np = np.wrapping_sub(1);
        }
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_select_line(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut np: u_int = (*wme).prefix;
    (*data).lineflag = LINE_SEL_LEFT_RIGHT;
    (*data).rectflag = 0 as ::core::ffi::c_int;
    (*data).selflag = SEL_LINE;
    (*data).dx = (*data).cx;
    (*data).dy = (*(*(*data).backing).grid)
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    window_copy_cursor_start_of_line(wme);
    (*data).selrx = (*data).cx;
    (*data).selry = (*(*(*data).backing).grid)
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    (*data).endselry = (*data).selry;
    window_copy_start_selection(wme);
    window_copy_cursor_end_of_line(wme);
    (*data).endselry = (*(*(*data).backing).grid)
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    (*data).endselrx = window_copy_find_length(wme, (*data).endselry);
    while np > 1 as u_int {
        window_copy_cursor_down(wme, 0 as ::core::ffi::c_int);
        window_copy_cursor_end_of_line(wme);
        np = np.wrapping_sub(1);
    }
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe extern "C" fn window_copy_cmd_select_word(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut so: *mut options = (*(*cs).s).options;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut nextx: u_int = 0;
    let mut nexty: u_int = 0;
    (*data).lineflag = LINE_SEL_LEFT_RIGHT;
    (*data).rectflag = 0 as ::core::ffi::c_int;
    (*data).selflag = SEL_WORD;
    (*data).dx = (*data).cx;
    (*data).dy = (*(*(*data).backing).grid)
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    (*data).separators = options_get_string(
        so,
        b"word-separators\0" as *const u8 as *const ::core::ffi::c_char,
    );
    window_copy_cursor_previous_word(wme, (*data).separators, 0 as ::core::ffi::c_int);
    px = (*data).cx;
    py = (*(*(*data).backing).grid)
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    (*data).selrx = px;
    (*data).selry = py;
    window_copy_start_selection(wme);
    nextx = px.wrapping_add(1 as u_int);
    nexty = py;
    if (*grid_get_line((*(*data).backing).grid, nexty)).flags as ::core::ffi::c_int
        & GRID_LINE_WRAPPED
        != 0
        && nextx > (*(*(*data).backing).grid).sx.wrapping_sub(1 as u_int)
    {
        nextx = 0 as u_int;
        nexty = nexty.wrapping_add(1);
    }
    if px >= window_copy_find_length(wme, py)
        || window_copy_in_set(wme, nextx, nexty, WHITESPACE.as_ptr()) == 0
    {
        window_copy_cursor_next_word_end(wme, (*data).separators, 1 as ::core::ffi::c_int);
    } else {
        window_copy_update_cursor(wme, px, (*data).cy);
        if window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 1 as ::core::ffi::c_int) != 0
        {
            window_copy_redraw_lines(wme, (*data).cy, 1 as u_int);
        }
    }
    (*data).endselrx = (*data).cx;
    (*data).endselry = (*(*(*data).backing).grid)
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    if (*data).dy > (*data).endselry {
        (*data).dy = (*data).endselry;
        (*data).dx = (*data).endselrx;
    } else if (*data).dx > (*data).endselrx {
        (*data).dx = (*data).endselrx;
    }
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe extern "C" fn window_copy_cmd_set_mark(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut data: *mut window_copy_mode_data = (*(*cs).wme).data as *mut window_copy_mode_data;
    (*data).mx = (*data).cx;
    (*data).my = (*(*(*data).backing).grid)
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    (*data).showmark = 1 as ::core::ffi::c_int;
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe extern "C" fn window_copy_cmd_start_of_line(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    window_copy_cursor_start_of_line(wme);
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_top_line(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    (*data).cx = 0 as u_int;
    (*data).cy = 0 as u_int;
    window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe extern "C" fn window_copy_cmd_copy_pipe_no_clear(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut c: *mut client = (*cs).c;
    let mut s: *mut session = (*cs).s;
    let mut wl: *mut winlink = (*cs).wl;
    let mut wp: *mut window_pane = (*wme).wp;
    let mut command: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut prefix: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut arg0: *const ::core::ffi::c_char = args_string((*cs).wargs, 0 as u_int);
    let mut arg1: *const ::core::ffi::c_char = args_string((*cs).wargs, 1 as u_int);
    let mut set_paste: ::core::ffi::c_int =
        (args_has((*cs).wargs, 'P' as i32 as u_char) == 0) as ::core::ffi::c_int;
    let mut set_clip: ::core::ffi::c_int =
        (args_has((*cs).wargs, 'C' as i32 as u_char) == 0) as ::core::ffi::c_int;
    if !arg1.is_null() {
        prefix = format_single(::core::ptr::null_mut::<cmdq_item>(), arg1, c, s, wl, wp);
    }
    if !s.is_null() && !arg0.is_null() && *arg0 as ::core::ffi::c_int != '\0' as i32 {
        command = format_single(::core::ptr::null_mut::<cmdq_item>(), arg0, c, s, wl, wp);
    }
    window_copy_copy_pipe(wme, s, prefix, command, set_paste, set_clip);
    free(command as *mut ::core::ffi::c_void);
    free(prefix as *mut ::core::ffi::c_void);
    return WINDOW_COPY_CMD_NOTHING;
}
unsafe extern "C" fn window_copy_cmd_copy_pipe(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    window_copy_cmd_copy_pipe_no_clear(cs);
    window_copy_clear_selection(wme);
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe extern "C" fn window_copy_cmd_copy_pipe_and_cancel(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    window_copy_cmd_copy_pipe_no_clear(cs);
    window_copy_clear_selection(wme);
    return WINDOW_COPY_CMD_CANCEL;
}
unsafe extern "C" fn window_copy_cmd_pipe_no_clear(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut c: *mut client = (*cs).c;
    let mut s: *mut session = (*cs).s;
    let mut wl: *mut winlink = (*cs).wl;
    let mut wp: *mut window_pane = (*wme).wp;
    let mut command: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut arg0: *const ::core::ffi::c_char = args_string((*cs).wargs, 0 as u_int);
    if !s.is_null() && !arg0.is_null() && *arg0 as ::core::ffi::c_int != '\0' as i32 {
        command = format_single(::core::ptr::null_mut::<cmdq_item>(), arg0, c, s, wl, wp);
    }
    window_copy_pipe(wme, s, command);
    free(command as *mut ::core::ffi::c_void);
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_pipe(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    window_copy_cmd_pipe_no_clear(cs);
    window_copy_clear_selection(wme);
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe extern "C" fn window_copy_cmd_pipe_and_cancel(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    window_copy_cmd_pipe_no_clear(cs);
    window_copy_clear_selection(wme);
    return WINDOW_COPY_CMD_CANCEL;
}
unsafe extern "C" fn window_copy_cmd_goto_line(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut arg0: *const ::core::ffi::c_char = args_string((*cs).wargs, 0 as u_int);
    if *arg0 as ::core::ffi::c_int != '\0' as i32 {
        window_copy_goto_line(wme, arg0);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_jump_backward(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut np: u_int = (*wme).prefix;
    let mut arg0: *const ::core::ffi::c_char = args_string((*cs).wargs, 0 as u_int);
    if *arg0 as ::core::ffi::c_int != '\0' as i32 {
        (*data).jumptype = WINDOW_COPY_JUMPBACKWARD as ::core::ffi::c_int;
        free((*data).jumpchar as *mut ::core::ffi::c_void);
        (*data).jumpchar = utf8_fromcstr(arg0);
        while np != 0 as u_int {
            window_copy_cursor_jump_back(wme);
            np = np.wrapping_sub(1);
        }
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_jump_forward(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut np: u_int = (*wme).prefix;
    let mut arg0: *const ::core::ffi::c_char = args_string((*cs).wargs, 0 as u_int);
    if *arg0 as ::core::ffi::c_int != '\0' as i32 {
        (*data).jumptype = WINDOW_COPY_JUMPFORWARD as ::core::ffi::c_int;
        free((*data).jumpchar as *mut ::core::ffi::c_void);
        (*data).jumpchar = utf8_fromcstr(arg0);
        while np != 0 as u_int {
            window_copy_cursor_jump(wme);
            np = np.wrapping_sub(1);
        }
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_jump_to_backward(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut np: u_int = (*wme).prefix;
    let mut arg0: *const ::core::ffi::c_char = args_string((*cs).wargs, 0 as u_int);
    if *arg0 as ::core::ffi::c_int != '\0' as i32 {
        (*data).jumptype = WINDOW_COPY_JUMPTOBACKWARD as ::core::ffi::c_int;
        free((*data).jumpchar as *mut ::core::ffi::c_void);
        (*data).jumpchar = utf8_fromcstr(arg0);
        while np != 0 as u_int {
            window_copy_cursor_jump_to_back(wme);
            np = np.wrapping_sub(1);
        }
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_jump_to_forward(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut np: u_int = (*wme).prefix;
    let mut arg0: *const ::core::ffi::c_char = args_string((*cs).wargs, 0 as u_int);
    if *arg0 as ::core::ffi::c_int != '\0' as i32 {
        (*data).jumptype = WINDOW_COPY_JUMPTOFORWARD as ::core::ffi::c_int;
        free((*data).jumpchar as *mut ::core::ffi::c_void);
        (*data).jumpchar = utf8_fromcstr(arg0);
        while np != 0 as u_int {
            window_copy_cursor_jump_to(wme);
            np = np.wrapping_sub(1);
        }
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_jump_to_mark(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    window_copy_jump_to_mark(wme);
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_next_prompt(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    window_copy_cursor_prompt(
        wme,
        1 as ::core::ffi::c_int,
        args_has((*cs).wargs, 'o' as i32 as u_char),
    );
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_previous_prompt(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    window_copy_cursor_prompt(
        wme,
        0 as ::core::ffi::c_int,
        args_has((*cs).wargs, 'o' as i32 as u_char),
    );
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_search_backward(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut np: u_int = (*wme).prefix;
    if window_copy_expand_search_string(cs) == 0 {
        return WINDOW_COPY_CMD_MOVE;
    }
    if !(*data).searchstr.is_null() {
        (*data).searchtype = WINDOW_COPY_SEARCHUP as ::core::ffi::c_int;
        (*data).searchregex = 1 as ::core::ffi::c_int;
        (*data).timeout = 0 as ::core::ffi::c_int;
        while np != 0 as u_int {
            window_copy_search_up(wme, 1 as ::core::ffi::c_int);
            np = np.wrapping_sub(1);
        }
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_search_backward_text(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut np: u_int = (*wme).prefix;
    if window_copy_expand_search_string(cs) == 0 {
        return WINDOW_COPY_CMD_MOVE;
    }
    if !(*data).searchstr.is_null() {
        (*data).searchtype = WINDOW_COPY_SEARCHUP as ::core::ffi::c_int;
        (*data).searchregex = 0 as ::core::ffi::c_int;
        (*data).timeout = 0 as ::core::ffi::c_int;
        while np != 0 as u_int {
            window_copy_search_up(wme, 0 as ::core::ffi::c_int);
            np = np.wrapping_sub(1);
        }
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_search_forward(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut np: u_int = (*wme).prefix;
    if window_copy_expand_search_string(cs) == 0 {
        return WINDOW_COPY_CMD_MOVE;
    }
    if !(*data).searchstr.is_null() {
        (*data).searchtype = WINDOW_COPY_SEARCHDOWN as ::core::ffi::c_int;
        (*data).searchregex = 1 as ::core::ffi::c_int;
        (*data).timeout = 0 as ::core::ffi::c_int;
        while np != 0 as u_int {
            window_copy_search_down(wme, 1 as ::core::ffi::c_int);
            np = np.wrapping_sub(1);
        }
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_search_forward_text(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut np: u_int = (*wme).prefix;
    if window_copy_expand_search_string(cs) == 0 {
        return WINDOW_COPY_CMD_MOVE;
    }
    if !(*data).searchstr.is_null() {
        (*data).searchtype = WINDOW_COPY_SEARCHDOWN as ::core::ffi::c_int;
        (*data).searchregex = 0 as ::core::ffi::c_int;
        (*data).timeout = 0 as ::core::ffi::c_int;
        while np != 0 as u_int {
            window_copy_search_down(wme, 0 as ::core::ffi::c_int);
            np = np.wrapping_sub(1);
        }
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_search_backward_incremental(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut arg0: *const ::core::ffi::c_char = args_string((*cs).wargs, 0 as u_int);
    let mut ss: *const ::core::ffi::c_char = (*data).searchstr;
    let mut prefix: ::core::ffi::c_char = 0;
    let mut action: window_copy_cmd_action = WINDOW_COPY_CMD_MOVE;
    (*data).timeout = 0 as ::core::ffi::c_int;
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"window_copy_cmd_search_backward_incremental\0" as *const u8 as *const ::core::ffi::c_char,
        arg0,
    );
    let fresh3 = arg0;
    arg0 = arg0.offset(1);
    prefix = *fresh3;
    if (*data).searchx == -(1 as ::core::ffi::c_int)
        || (*data).searchy == -(1 as ::core::ffi::c_int)
    {
        (*data).searchx = (*data).cx as ::core::ffi::c_int;
        (*data).searchy = (*data).cy as ::core::ffi::c_int;
        (*data).searcho = (*data).oy as ::core::ffi::c_int;
    } else if !ss.is_null() && strcmp(arg0, ss) != 0 as ::core::ffi::c_int {
        (*data).cx = (*data).searchx as u_int;
        (*data).cy = (*data).searchy as u_int;
        (*data).oy = (*data).searcho as u_int;
        (*data).cx = window_copy_cursor_limit(
            wme,
            (*(*(*data).backing).grid)
                .hsize
                .wrapping_add((*data).cy)
                .wrapping_sub((*data).oy),
            0 as ::core::ffi::c_int,
        );
        action = WINDOW_COPY_CMD_REDRAW;
    }
    if *arg0 as ::core::ffi::c_int == '\0' as i32 {
        window_copy_clear_marks(wme);
        return WINDOW_COPY_CMD_REDRAW;
    }
    match prefix as ::core::ffi::c_int {
        61 | 45 => {
            (*data).searchtype = WINDOW_COPY_SEARCHUP as ::core::ffi::c_int;
            (*data).searchregex = 0 as ::core::ffi::c_int;
            free((*data).searchstr as *mut ::core::ffi::c_void);
            (*data).searchstr = xstrdup(arg0);
            if window_copy_search_up(wme, 0 as ::core::ffi::c_int) == 0 {
                window_copy_clear_marks(wme);
                return WINDOW_COPY_CMD_REDRAW;
            }
        }
        43 => {
            (*data).searchtype = WINDOW_COPY_SEARCHDOWN as ::core::ffi::c_int;
            (*data).searchregex = 0 as ::core::ffi::c_int;
            free((*data).searchstr as *mut ::core::ffi::c_void);
            (*data).searchstr = xstrdup(arg0);
            if window_copy_search_down(wme, 0 as ::core::ffi::c_int) == 0 {
                window_copy_clear_marks(wme);
                return WINDOW_COPY_CMD_REDRAW;
            }
        }
        _ => {}
    }
    return action;
}
unsafe extern "C" fn window_copy_cmd_search_forward_incremental(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut arg0: *const ::core::ffi::c_char = args_string((*cs).wargs, 0 as u_int);
    let mut ss: *const ::core::ffi::c_char = (*data).searchstr;
    let mut prefix: ::core::ffi::c_char = 0;
    let mut action: window_copy_cmd_action = WINDOW_COPY_CMD_MOVE;
    (*data).timeout = 0 as ::core::ffi::c_int;
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"window_copy_cmd_search_forward_incremental\0" as *const u8 as *const ::core::ffi::c_char,
        arg0,
    );
    let fresh2 = arg0;
    arg0 = arg0.offset(1);
    prefix = *fresh2;
    if (*data).searchx == -(1 as ::core::ffi::c_int)
        || (*data).searchy == -(1 as ::core::ffi::c_int)
    {
        (*data).searchx = (*data).cx as ::core::ffi::c_int;
        (*data).searchy = (*data).cy as ::core::ffi::c_int;
        (*data).searcho = (*data).oy as ::core::ffi::c_int;
    } else if !ss.is_null() && strcmp(arg0, ss) != 0 as ::core::ffi::c_int {
        (*data).cx = (*data).searchx as u_int;
        (*data).cy = (*data).searchy as u_int;
        (*data).oy = (*data).searcho as u_int;
        (*data).cx = window_copy_cursor_limit(
            wme,
            (*(*(*data).backing).grid)
                .hsize
                .wrapping_add((*data).cy)
                .wrapping_sub((*data).oy),
            0 as ::core::ffi::c_int,
        );
        action = WINDOW_COPY_CMD_REDRAW;
    }
    if *arg0 as ::core::ffi::c_int == '\0' as i32 {
        window_copy_clear_marks(wme);
        return WINDOW_COPY_CMD_REDRAW;
    }
    match prefix as ::core::ffi::c_int {
        61 | 43 => {
            (*data).searchtype = WINDOW_COPY_SEARCHDOWN as ::core::ffi::c_int;
            (*data).searchregex = 0 as ::core::ffi::c_int;
            free((*data).searchstr as *mut ::core::ffi::c_void);
            (*data).searchstr = xstrdup(arg0);
            if window_copy_search_down(wme, 0 as ::core::ffi::c_int) == 0 {
                window_copy_clear_marks(wme);
                return WINDOW_COPY_CMD_REDRAW;
            }
        }
        45 => {
            (*data).searchtype = WINDOW_COPY_SEARCHUP as ::core::ffi::c_int;
            (*data).searchregex = 0 as ::core::ffi::c_int;
            free((*data).searchstr as *mut ::core::ffi::c_void);
            (*data).searchstr = xstrdup(arg0);
            if window_copy_search_up(wme, 0 as ::core::ffi::c_int) == 0 {
                window_copy_clear_marks(wme);
                return WINDOW_COPY_CMD_REDRAW;
            }
        }
        _ => {}
    }
    return action;
}
unsafe extern "C" fn window_copy_do_refresh(
    mut wme: *mut window_mode_entry,
    mut follow: ::core::ffi::c_int,
) {
    let mut wp: *mut window_pane = (*wme).swp;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut oy_from_top: u_int = 0;
    if (*data).oy > (*(*(*data).backing).grid).hsize {
        (*data).oy = (*(*(*data).backing).grid).hsize;
    }
    oy_from_top = (*(*(*data).backing).grid).hsize.wrapping_sub((*data).oy);
    if window_copy_sync_backing(wme) == 0 {
        screen_free((*data).backing);
        free((*data).backing as *mut ::core::ffi::c_void);
        (*data).backing = window_copy_clone_screen(
            &raw mut (*wp).base,
            &raw mut (*data).screen,
            ::core::ptr::null_mut::<u_int>(),
            ::core::ptr::null_mut::<u_int>(),
            ((*wme).swp != (*wme).wp) as ::core::ffi::c_int,
        );
    }
    if follow != 0 {
        (*data).cy = (*(*data).screen.grid).sy.wrapping_sub(1 as u_int);
        (*data).cx = window_copy_cursor_limit(
            wme,
            (*(*(*data).backing).grid).hsize.wrapping_add((*data).cy),
            0 as ::core::ffi::c_int,
        );
        (*data).oy = 0 as u_int;
    } else if oy_from_top <= (*(*(*data).backing).grid).hsize {
        (*data).oy = (*(*(*data).backing).grid).hsize.wrapping_sub(oy_from_top);
    } else {
        (*data).cy = 0 as u_int;
        (*data).oy = (*(*(*data).backing).grid).hsize;
    }
    window_copy_sync_snapshot(data, (*wp).base.grid);
    window_copy_size_changed(wme);
}
unsafe extern "C" fn window_copy_refresh_arm(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut tv: timeval = timeval {
        tv_sec: (WINDOW_COPY_REFRESH_INTERVAL / 1000000 as ::core::ffi::c_int) as __time_t,
        tv_usec: (WINDOW_COPY_REFRESH_INTERVAL % 1000000 as ::core::ffi::c_int) as __suseconds_t,
    };
    if (*data).refresh_active != 0 {
        event_add(&raw mut (*data).refresh_timer, &raw mut tv);
    }
}
unsafe extern "C" fn window_copy_refresh_allowed(
    mut wme: *mut window_mode_entry,
) -> ::core::ffi::c_int {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    if (*data).viewmode != 0 || (*wme).swp != (*wme).wp {
        return 0 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn window_copy_refresh_timer(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut wme: *mut window_mode_entry = arg as *mut window_mode_entry;
    let mut wp: *mut window_pane = (*wme).wp;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut follow: ::core::ffi::c_int = 0;
    if (*wp).modes.tqh_first != wme || (*data).refresh_active == 0 {
        return;
    }
    if (*wp).flags & PANE_UNSEENCHANGES != 0
        && (*data).screen.sel.is_null()
        && (*data).cursordrag as ::core::ffi::c_uint
            == CURSORDRAG_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        follow = ((*data).oy == 0 as u_int
            && (*data).cy == (*(*data).screen.grid).sy.wrapping_sub(1 as u_int))
            as ::core::ffi::c_int;
        window_copy_do_refresh(wme, follow);
        window_copy_redraw_screen(wme);
        (*wp).flags |= PANE_REDRAW;
        (*wp).flags &= !PANE_UNSEENCHANGES;
    }
    window_copy_refresh_arm(wme);
}
unsafe extern "C" fn window_copy_refresh_start(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    if window_copy_refresh_allowed(wme) == 0 || (*data).refresh_active != 0 {
        return;
    }
    (*data).refresh_active = 1 as ::core::ffi::c_int;
    window_copy_refresh_arm(wme);
}
unsafe extern "C" fn window_copy_refresh_stop(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    (*data).refresh_active = 0 as ::core::ffi::c_int;
    event_del(&raw mut (*data).refresh_timer);
}
unsafe extern "C" fn window_copy_cmd_refresh_now(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut wp: *mut window_pane = (*wme).wp;
    let mut follow: ::core::ffi::c_int = 0;
    if window_copy_refresh_allowed(wme) == 0 {
        return WINDOW_COPY_CMD_NOTHING;
    }
    follow = ((*data).oy == 0 as u_int
        && (*data).cy == (*(*data).screen.grid).sy.wrapping_sub(1 as u_int))
        as ::core::ffi::c_int;
    window_copy_do_refresh(wme, follow);
    (*wp).flags &= !PANE_UNSEENCHANGES;
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe extern "C" fn window_copy_cmd_refresh_on(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    window_copy_refresh_start((*cs).wme);
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_refresh_off(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    window_copy_refresh_stop((*cs).wme);
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_refresh_toggle(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut data: *mut window_copy_mode_data = (*(*cs).wme).data as *mut window_copy_mode_data;
    if (*data).refresh_active != 0 {
        window_copy_refresh_stop((*cs).wme);
    } else {
        window_copy_refresh_start((*cs).wme);
    }
    return WINDOW_COPY_CMD_MOVE;
}
unsafe extern "C" fn window_copy_cmd_recentre_top_bottom(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    let mut wme: *mut window_mode_entry = (*cs).wme;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut cy: u_int = (*data).cy;
    let mut oy: u_int = (*data).oy;
    let mut sy: u_int = (*(*data).screen.grid).sy.wrapping_sub(1 as u_int);
    let mut sm: u_int = sy.wrapping_div(2 as u_int);
    let mut backing_row: u_int = 0;
    let mut target: C2RustUnnamed_48 = MIDDLE;
    backing_row = (*(*(*data).backing).grid)
        .hsize
        .wrapping_add(cy)
        .wrapping_sub((*data).oy);
    if (*data).recentre_line != backing_row {
        (*data).recentre_state = RECENTRE_MIDDLE;
        (*data).recentre_line = backing_row;
    }
    match (*data).recentre_state as ::core::ffi::c_uint {
        1 => {
            (*data).recentre_state = RECENTRE_TOP;
            target = MIDDLE;
        }
        0 => {
            (*data).recentre_state = RECENTRE_BOTTOM;
            target = TOP;
        }
        2 | _ => {
            (*data).recentre_state = RECENTRE_MIDDLE;
            target = BOTTOM;
        }
    }
    oy = (*data).oy;
    match target as ::core::ffi::c_uint {
        0 => {
            if cy < sm {
                window_copy_scroll_down(wme, sm.wrapping_sub(cy));
            } else if cy > sm {
                window_copy_scroll_up(wme, cy.wrapping_sub(sm));
            }
            if (*data).oy != oy {
                (*data).cy = cy.wrapping_add((*data).oy.wrapping_sub(oy));
            }
        }
        1 => {
            window_copy_scroll_up(wme, cy);
            (*data).cy = cy.wrapping_sub(oy.wrapping_sub((*data).oy));
        }
        2 => {
            window_copy_scroll_down(wme, sy.wrapping_sub(cy));
            (*data).cy = cy.wrapping_add((*data).oy.wrapping_sub(oy));
        }
        _ => {}
    }
    window_copy_update_selection_view(wme, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    return WINDOW_COPY_CMD_REDRAW;
}
unsafe extern "C" fn window_copy_cmd_line_numbers_on(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    window_copy_set_line_numbers1((*cs).wme, 1 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
    return WINDOW_COPY_CMD_NOTHING;
}
unsafe extern "C" fn window_copy_cmd_line_numbers_off(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    window_copy_set_line_numbers1((*cs).wme, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    return WINDOW_COPY_CMD_NOTHING;
}
unsafe extern "C" fn window_copy_cmd_line_numbers_toggle(
    mut cs: *mut window_copy_cmd_state,
) -> window_copy_cmd_action {
    window_copy_set_line_numbers1(
        (*cs).wme,
        (window_copy_line_numbers_active((*cs).wme) == 0) as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    return WINDOW_COPY_CMD_NOTHING;
}
static mut window_copy_cmd_table: [C2RustUnnamed_46; 99] = unsafe {
    [
        C2RustUnnamed_46 {
            command: b"append-selection\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_append_selection
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"append-selection-and-cancel\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_append_selection_and_cancel
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"back-to-indentation\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_back_to_indentation
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"begin-selection\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_begin_selection
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"bottom-line\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(
                window_copy_cmd_bottom_line
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"cancel\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_cancel
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"clear-selection\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_clear_selection
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"copy-end-of-line\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"CP\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_copy_end_of_line
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"copy-end-of-line-and-cancel\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"CP\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_copy_end_of_line_and_cancel
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"copy-pipe-end-of-line\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"CP\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 2 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_copy_pipe_end_of_line
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"copy-pipe-end-of-line-and-cancel\0" as *const u8
                as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"CP\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 2 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_copy_pipe_end_of_line_and_cancel
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"copy-line\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"CP\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_copy_line
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"copy-line-and-cancel\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"CP\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_copy_line_and_cancel
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"copy-pipe-line\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"CP\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 2 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_copy_pipe_line
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"copy-pipe-line-and-cancel\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"CP\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 2 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_copy_pipe_line_and_cancel
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"copy-pipe-no-clear\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"CP\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 2 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_NEVER,
            f: Some(
                window_copy_cmd_copy_pipe_no_clear
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"copy-pipe\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"CP\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 2 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_copy_pipe
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"copy-pipe-and-cancel\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"CP\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 2 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_copy_pipe_and_cancel
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"copy-selection-no-clear\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"CP\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_NEVER,
            f: Some(
                window_copy_cmd_copy_selection_no_clear
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"copy-selection\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"CP\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_copy_selection
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"copy-selection-and-cancel\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"CP\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_copy_selection_and_cancel
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"cursor-down\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(
                window_copy_cmd_cursor_down
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"cursor-down-and-cancel\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_cursor_down_and_cancel
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"cursor-left\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(
                window_copy_cmd_cursor_left
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"cursor-right\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(
                window_copy_cmd_cursor_right
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"cursor-up\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(
                window_copy_cmd_cursor_up
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"cursor-centre-vertical\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(
                window_copy_cmd_centre_vertical
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"cursor-centre-horizontal\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(
                window_copy_cmd_centre_horizontal
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"end-of-line\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(
                window_copy_cmd_end_of_line
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"goto-line\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 1 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(
                window_copy_cmd_goto_line
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"halfpage-down\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(
                window_copy_cmd_halfpage_down
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"halfpage-down-and-cancel\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_halfpage_down_and_cancel
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"halfpage-up\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(
                window_copy_cmd_halfpage_up
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"history-bottom\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(
                window_copy_cmd_history_bottom
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"history-top\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(
                window_copy_cmd_history_top
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"jump-again\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(
                window_copy_cmd_jump_again
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"jump-backward\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 1 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(
                window_copy_cmd_jump_backward
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"jump-forward\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 1 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(
                window_copy_cmd_jump_forward
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"jump-reverse\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(
                window_copy_cmd_jump_reverse
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"jump-to-backward\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 1 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(
                window_copy_cmd_jump_to_backward
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"jump-to-forward\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 1 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(
                window_copy_cmd_jump_to_forward
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"jump-to-mark\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_jump_to_mark
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"line-numbers-on\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_NEVER,
            f: Some(
                window_copy_cmd_line_numbers_on
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"line-numbers-off\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_NEVER,
            f: Some(
                window_copy_cmd_line_numbers_off
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"line-numbers-toggle\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_NEVER,
            f: Some(
                window_copy_cmd_line_numbers_toggle
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"next-prompt\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"o\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_next_prompt
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"previous-prompt\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"o\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_previous_prompt
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"middle-line\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(
                window_copy_cmd_middle_line
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"next-matching-bracket\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_next_matching_bracket
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"next-paragraph\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(
                window_copy_cmd_next_paragraph
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"next-space\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(
                window_copy_cmd_next_space
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"next-space-end\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(
                window_copy_cmd_next_space_end
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"next-word\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(
                window_copy_cmd_next_word
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"next-word-end\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(
                window_copy_cmd_next_word_end
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"other-end\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(
                window_copy_cmd_other_end
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"page-down\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(
                window_copy_cmd_page_down
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"page-down-and-cancel\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_page_down_and_cancel
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"page-up\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(
                window_copy_cmd_page_up
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"pipe-no-clear\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_NEVER,
            f: Some(
                window_copy_cmd_pipe_no_clear
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"pipe\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_pipe
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"pipe-and-cancel\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_pipe_and_cancel
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"previous-matching-bracket\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_previous_matching_bracket
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"previous-paragraph\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(
                window_copy_cmd_previous_paragraph
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"previous-space\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(
                window_copy_cmd_previous_space
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"previous-word\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(
                window_copy_cmd_previous_word
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"recentre-top-bottom\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_recentre_top_bottom
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"rectangle-on\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_rectangle_on
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"rectangle-off\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_rectangle_off
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"rectangle-toggle\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_rectangle_toggle
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"refresh-on\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_NEVER,
            f: Some(
                window_copy_cmd_refresh_on
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"refresh-off\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_NEVER,
            f: Some(
                window_copy_cmd_refresh_off
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"refresh-now\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_NEVER,
            f: Some(
                window_copy_cmd_refresh_now
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"refresh-toggle\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_NEVER,
            f: Some(
                window_copy_cmd_refresh_toggle
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"scroll-bottom\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_scroll_bottom
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"scroll-down\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(
                window_copy_cmd_scroll_down
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"scroll-down-and-cancel\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_scroll_down_and_cancel
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"scroll-exit-on\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_scroll_exit_on
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"scroll-exit-off\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_scroll_exit_off
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"scroll-exit-toggle\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_scroll_exit_toggle
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"scroll-middle\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_scroll_middle
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"scroll-to-mouse\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"e\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(
                window_copy_cmd_scroll_to_mouse
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"scroll-top\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_scroll_top
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"scroll-up\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(
                window_copy_cmd_scroll_up
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"search-again\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_search_again
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"search-backward\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_search_backward
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"search-backward-text\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_search_backward_text
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"search-backward-incremental\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 1 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_search_backward_incremental
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"search-forward\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_search_forward
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"search-forward-text\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_search_forward_text
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"search-forward-incremental\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 1 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_search_forward_incremental
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"search-reverse\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_search_reverse
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"select-line\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_select_line
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"select-word\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_select_word
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"selection-mode\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_selection_mode
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"set-mark\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_set_mark
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"start-of-line\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(
                window_copy_cmd_start_of_line
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"stop-selection\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(
                window_copy_cmd_stop_selection
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"toggle-position\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_NEVER,
            f: Some(
                window_copy_cmd_toggle_position
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
        C2RustUnnamed_46 {
            command: b"top-line\0" as *const u8 as *const ::core::ffi::c_char,
            minargs: 0,
            maxargs: 0,
            args: args_parse {
                template: b"\0" as *const u8 as *const ::core::ffi::c_char,
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(
                window_copy_cmd_top_line
                    as unsafe extern "C" fn(*mut window_copy_cmd_state) -> window_copy_cmd_action,
            ),
        },
    ]
};
pub const WINDOW_COPY_CMD_FLAG_READONLY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
unsafe extern "C" fn window_copy_command(
    mut wme: *mut window_mode_entry,
    mut c: *mut client,
    mut s: *mut session,
    mut wl: *mut winlink,
    mut args: *mut args,
    mut m: *mut mouse_event,
) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut wp: *mut window_pane = (*wme).wp;
    let mut cs: window_copy_cmd_state = window_copy_cmd_state {
        wme: ::core::ptr::null_mut::<window_mode_entry>(),
        args: ::core::ptr::null_mut::<args>(),
        wargs: ::core::ptr::null_mut::<args>(),
        m: ::core::ptr::null_mut::<mouse_event>(),
        c: ::core::ptr::null_mut::<client>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
    };
    let mut action: window_copy_cmd_action = WINDOW_COPY_CMD_NOTHING;
    let mut clear: window_copy_cmd_clear = WINDOW_COPY_CMD_CLEAR_NEVER;
    let mut command: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut i: u_int = 0;
    let mut count: u_int = args_count(args);
    let mut keys: ::core::ffi::c_int = 0;
    let mut flags: ::core::ffi::c_int = 0;
    let mut error: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if count == 0 as u_int {
        return;
    }
    command = args_string(args, 0 as u_int);
    if !m.is_null()
        && (*m).valid != 0
        && !((*m).b & MOUSE_MASK_BUTTONS as u_int == MOUSE_WHEEL_UP as u_int
            || (*m).b & MOUSE_MASK_BUTTONS as u_int == MOUSE_WHEEL_DOWN as u_int)
    {
        window_copy_move_mouse(m);
    }
    cs.wme = wme;
    cs.args = args;
    cs.wargs = ::core::ptr::null_mut::<args>();
    cs.m = m;
    cs.c = c;
    cs.s = s;
    cs.wl = wl;
    action = WINDOW_COPY_CMD_MOVE;
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[C2RustUnnamed_46; 99]>() as usize)
            .wrapping_div(::core::mem::size_of::<C2RustUnnamed_46>() as usize)
    {
        if strcmp(window_copy_cmd_table[i as usize].command, command) == 0 as ::core::ffi::c_int {
            flags = window_copy_cmd_table[i as usize].flags;
            if !c.is_null()
                && (*c).flags & CLIENT_READONLY as uint64_t != 0
                && !flags & WINDOW_COPY_CMD_FLAG_READONLY != 0
            {
                status_message_set(
                    c,
                    -(1 as ::core::ffi::c_int),
                    1 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    b"client is read-only\0" as *const u8 as *const ::core::ffi::c_char,
                );
                return;
            }
            cs.wargs = args_parse(
                &raw const (*(&raw const window_copy_cmd_table as *const C2RustUnnamed_46)
                    .offset(i as isize))
                .args,
                args_values(args),
                count,
                &raw mut error,
            );
            if !error.is_null() {
                free(error as *mut ::core::ffi::c_void);
                error = ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            if cs.wargs.is_null() {
                break;
            }
            clear = window_copy_cmd_table[i as usize].clear;
            action = window_copy_cmd_table[i as usize]
                .f
                .expect("non-null function pointer")(&raw mut cs);
            args_free(cs.wargs);
            cs.wargs = ::core::ptr::null_mut::<args>();
            break;
        } else {
            i = i.wrapping_add(1);
        }
    }
    if strncmp(
        command,
        b"search-\0" as *const u8 as *const ::core::ffi::c_char,
        7 as size_t,
    ) != 0 as ::core::ffi::c_int
        && !(*data).searchmark.is_null()
    {
        keys = options_get_number(
            (*(*wp).window).options,
            b"mode-keys\0" as *const u8 as *const ::core::ffi::c_char,
        ) as ::core::ffi::c_int;
        if clear as ::core::ffi::c_uint
            == WINDOW_COPY_CMD_CLEAR_EMACS_ONLY as ::core::ffi::c_int as ::core::ffi::c_uint
            && keys == MODEKEY_VI
        {
            clear = WINDOW_COPY_CMD_CLEAR_NEVER;
        }
        if clear as ::core::ffi::c_uint
            != WINDOW_COPY_CMD_CLEAR_NEVER as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            window_copy_clear_marks(wme);
            (*data).searchy = -(1 as ::core::ffi::c_int);
            (*data).searchx = (*data).searchy;
        }
        if action as ::core::ffi::c_uint
            == WINDOW_COPY_CMD_MOVE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            action = WINDOW_COPY_CMD_REDRAW;
        }
    }
    (*wme).prefix = 1 as u_int;
    if action as ::core::ffi::c_uint
        == WINDOW_COPY_CMD_CANCEL as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        window_pane_reset_mode(wp);
    } else if action as ::core::ffi::c_uint
        == WINDOW_COPY_CMD_REDRAW as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        window_copy_redraw_screen(wme);
    } else if action as ::core::ffi::c_uint
        == WINDOW_COPY_CMD_MOVE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        window_copy_redraw_lines(wme, 0 as u_int, 1 as u_int);
    }
}
unsafe extern "C" fn window_copy_scroll_to(
    mut wme: *mut window_mode_entry,
    mut px: u_int,
    mut py: u_int,
    mut no_redraw: ::core::ffi::c_int,
) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut gd: *mut grid = (*(*data).backing).grid;
    let mut offset: u_int = 0;
    let mut gap: u_int = 0;
    let mut old_oy: u_int = (*data).oy;
    (*data).cx = px;
    if py >= (*gd).hsize.wrapping_sub((*data).oy)
        && py < (*gd).hsize.wrapping_sub((*data).oy).wrapping_add((*gd).sy)
    {
        (*data).cy = py.wrapping_sub((*gd).hsize.wrapping_sub((*data).oy));
    } else {
        gap = (*gd).sy.wrapping_div(4 as u_int);
        if py < (*gd).sy {
            offset = 0 as u_int;
            (*data).cy = py;
        } else if py > (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(gap) {
            offset = (*gd).hsize;
            (*data).cy = py.wrapping_sub((*gd).hsize);
        } else {
            offset = py.wrapping_add(gap).wrapping_sub((*gd).sy);
            (*data).cy = py.wrapping_sub(offset);
        }
        (*data).oy = (*gd).hsize.wrapping_sub(offset);
    }
    if no_redraw == 0 && !(*data).searchmark.is_null() && (*data).timeout == 0 {
        window_copy_search_marks(
            wme,
            ::core::ptr::null_mut::<screen>(),
            (*data).searchregex,
            1 as ::core::ffi::c_int,
        );
    }
    window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    if (*data).oy != old_oy {
        window_pane_scrollbar_show((*wme).wp, 1 as ::core::ffi::c_int);
    }
    if no_redraw == 0 {
        window_copy_redraw_screen(wme);
    }
}
unsafe extern "C" fn window_copy_search_compare(
    mut gd: *mut grid,
    mut px: u_int,
    mut py: u_int,
    mut sgd: *mut grid,
    mut spx: u_int,
    mut cis: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
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
    let mut sgc: grid_cell = grid_cell {
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
    let mut ud: *const utf8_data = ::core::ptr::null::<utf8_data>();
    let mut sud: *const utf8_data = ::core::ptr::null::<utf8_data>();
    grid_get_cell(gd, px, py, &raw mut gc);
    ud = &raw mut gc.data;
    grid_get_cell(sgd, spx, 0 as u_int, &raw mut sgc);
    sud = &raw mut sgc.data;
    if *(&raw const (*sud).data as *const u_char) as ::core::ffi::c_int == '\t' as i32
        && (*sud).size as ::core::ffi::c_int == 1 as ::core::ffi::c_int
        && gc.flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0
    {
        return 1 as ::core::ffi::c_int;
    }
    if (*ud).size as ::core::ffi::c_int != (*sud).size as ::core::ffi::c_int
        || (*ud).width as ::core::ffi::c_int != (*sud).width as ::core::ffi::c_int
    {
        return 0 as ::core::ffi::c_int;
    }
    if cis != 0 && (*ud).size as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        return (({
            let mut __res: ::core::ffi::c_int = 0;
            if ::core::mem::size_of::<u_char>() as usize > 1 as usize {
                if 0 != 0 {
                    let mut __c: ::core::ffi::c_int =
                        (*ud).data[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int;
                    __res =
                        (if __c < -(128 as ::core::ffi::c_int) || __c > 255 as ::core::ffi::c_int {
                            __c as __int32_t
                        } else {
                            *(*__ctype_tolower_loc()).offset(__c as isize)
                        }) as ::core::ffi::c_int;
                } else {
                    __res =
                        tolower((*ud).data[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int);
                }
            } else {
                __res = *(*__ctype_tolower_loc()).offset(
                    (*ud).data[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int as isize,
                ) as ::core::ffi::c_int;
            }
            __res
        }) == (*sud).data[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int)
            as ::core::ffi::c_int;
    }
    return (memcmp(
        &raw const (*ud).data as *const u_char as *const ::core::ffi::c_void,
        &raw const (*sud).data as *const u_char as *const ::core::ffi::c_void,
        (*ud).size as size_t,
    ) == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
unsafe extern "C" fn window_copy_search_lr(
    mut gd: *mut grid,
    mut sgd: *mut grid,
    mut ppx: *mut u_int,
    mut py: u_int,
    mut first: u_int,
    mut last: u_int,
    mut cis: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut ax: u_int = 0;
    let mut bx: u_int = 0;
    let mut px: u_int = 0;
    let mut pywrap: u_int = 0;
    let mut endline: u_int = 0;
    let mut padding: u_int = 0;
    let mut matched: ::core::ffi::c_int = 0;
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
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
    endline = (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(1 as u_int);
    ax = first;
    while ax < last {
        padding = 0 as u_int;
        bx = 0 as u_int;
        while bx < (*sgd).sx {
            px = ax.wrapping_add(bx).wrapping_add(padding);
            pywrap = py;
            while px >= (*gd).sx && pywrap < endline {
                gl = grid_get_line(gd, pywrap);
                if !((*gl).flags as ::core::ffi::c_int) & GRID_LINE_WRAPPED != 0 {
                    break;
                }
                px = px.wrapping_sub((*gd).sx);
                pywrap = pywrap.wrapping_add(1);
            }
            if px.wrapping_sub(padding) >= (*gd).sx {
                break;
            }
            grid_get_cell(gd, px, pywrap, &raw mut gc);
            if gc.flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
                padding = padding.wrapping_add(
                    (gc.data.width as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as u_int,
                );
            }
            matched = window_copy_search_compare(gd, px, pywrap, sgd, bx, cis);
            if matched == 0 {
                break;
            }
            bx = bx.wrapping_add(1);
        }
        if bx == (*sgd).sx {
            *ppx = ax;
            return 1 as ::core::ffi::c_int;
        }
        ax = ax.wrapping_add(1);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn window_copy_search_rl(
    mut gd: *mut grid,
    mut sgd: *mut grid,
    mut ppx: *mut u_int,
    mut py: u_int,
    mut first: u_int,
    mut last: u_int,
    mut cis: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut ax: u_int = 0;
    let mut bx: u_int = 0;
    let mut px: u_int = 0;
    let mut pywrap: u_int = 0;
    let mut endline: u_int = 0;
    let mut padding: u_int = 0;
    let mut matched: ::core::ffi::c_int = 0;
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
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
    endline = (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(1 as u_int);
    ax = last;
    while ax > first {
        padding = 0 as u_int;
        bx = 0 as u_int;
        while bx < (*sgd).sx {
            px = ax
                .wrapping_sub(1 as u_int)
                .wrapping_add(bx)
                .wrapping_add(padding);
            pywrap = py;
            while px >= (*gd).sx && pywrap < endline {
                gl = grid_get_line(gd, pywrap);
                if !((*gl).flags as ::core::ffi::c_int) & GRID_LINE_WRAPPED != 0 {
                    break;
                }
                px = px.wrapping_sub((*gd).sx);
                pywrap = pywrap.wrapping_add(1);
            }
            if px.wrapping_sub(padding) >= (*gd).sx {
                break;
            }
            grid_get_cell(gd, px, pywrap, &raw mut gc);
            if gc.flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
                padding = padding.wrapping_add(
                    (gc.data.width as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as u_int,
                );
            }
            matched = window_copy_search_compare(gd, px, pywrap, sgd, bx, cis);
            if matched == 0 {
                break;
            }
            bx = bx.wrapping_add(1);
        }
        if bx == (*sgd).sx {
            *ppx = ax.wrapping_sub(1 as u_int);
            return 1 as ::core::ffi::c_int;
        }
        ax = ax.wrapping_sub(1);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn window_copy_search_lr_regex(
    mut gd: *mut grid,
    mut ppx: *mut u_int,
    mut psx: *mut u_int,
    mut py: u_int,
    mut first: u_int,
    mut last: u_int,
    mut reg: *mut regex_t,
) -> ::core::ffi::c_int {
    let mut eflags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut endline: u_int = 0;
    let mut foundx: u_int = 0;
    let mut foundy: u_int = 0;
    let mut len: u_int = 0;
    let mut pywrap: u_int = 0;
    let mut size: u_int = 1 as u_int;
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut regmatch: regmatch_t = regmatch_t { rm_so: 0, rm_eo: 0 };
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    if first >= last {
        return 0 as ::core::ffi::c_int;
    }
    if first != 0 as u_int {
        eflags |= REG_NOTBOL;
    }
    buf = xmalloc(size as size_t) as *mut ::core::ffi::c_char;
    *buf.offset(0 as ::core::ffi::c_int as isize) = '\0' as i32 as ::core::ffi::c_char;
    buf = window_copy_stringify(gd, py, first, (*gd).sx, buf, &raw mut size);
    len = (*gd).sx.wrapping_sub(first);
    endline = (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(1 as u_int);
    pywrap = py;
    while !buf.is_null() && pywrap < endline && len < WINDOW_COPY_SEARCH_MAX_LINE as u_int {
        gl = grid_get_line(gd, pywrap);
        if !((*gl).flags as ::core::ffi::c_int) & GRID_LINE_WRAPPED != 0 {
            break;
        }
        pywrap = pywrap.wrapping_add(1);
        buf = window_copy_stringify(gd, pywrap, 0 as u_int, (*gd).sx, buf, &raw mut size);
        len = len.wrapping_add((*gd).sx);
    }
    if regexec(reg, buf, 1 as size_t, &raw mut regmatch, eflags) == 0 as ::core::ffi::c_int
        && regmatch.rm_so != regmatch.rm_eo
    {
        foundx = first;
        foundy = py;
        window_copy_cstrtocellpos(
            gd,
            len,
            &raw mut foundx,
            &raw mut foundy,
            buf.offset(regmatch.rm_so as isize),
        );
        if foundy == py && foundx < last {
            *ppx = foundx;
            len = len.wrapping_sub(foundx.wrapping_sub(first));
            window_copy_cstrtocellpos(
                gd,
                len,
                &raw mut foundx,
                &raw mut foundy,
                buf.offset(regmatch.rm_eo as isize),
            );
            *psx = foundx;
            while foundy > py {
                *psx = (*psx).wrapping_add((*gd).sx);
                foundy = foundy.wrapping_sub(1);
            }
            *psx = (*psx).wrapping_sub(*ppx);
            free(buf as *mut ::core::ffi::c_void);
            return 1 as ::core::ffi::c_int;
        }
    }
    free(buf as *mut ::core::ffi::c_void);
    *ppx = 0 as u_int;
    *psx = 0 as u_int;
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn window_copy_search_rl_regex(
    mut gd: *mut grid,
    mut ppx: *mut u_int,
    mut psx: *mut u_int,
    mut py: u_int,
    mut first: u_int,
    mut last: u_int,
    mut reg: *mut regex_t,
) -> ::core::ffi::c_int {
    let mut eflags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut endline: u_int = 0;
    let mut len: u_int = 0;
    let mut pywrap: u_int = 0;
    let mut size: u_int = 1 as u_int;
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    if first != 0 as u_int {
        eflags |= REG_NOTBOL;
    }
    buf = xmalloc(size as size_t) as *mut ::core::ffi::c_char;
    *buf.offset(0 as ::core::ffi::c_int as isize) = '\0' as i32 as ::core::ffi::c_char;
    buf = window_copy_stringify(gd, py, first, (*gd).sx, buf, &raw mut size);
    len = (*gd).sx.wrapping_sub(first);
    endline = (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(1 as u_int);
    pywrap = py;
    while !buf.is_null() && pywrap < endline && len < WINDOW_COPY_SEARCH_MAX_LINE as u_int {
        gl = grid_get_line(gd, pywrap);
        if !((*gl).flags as ::core::ffi::c_int) & GRID_LINE_WRAPPED != 0 {
            break;
        }
        pywrap = pywrap.wrapping_add(1);
        buf = window_copy_stringify(gd, pywrap, 0 as u_int, (*gd).sx, buf, &raw mut size);
        len = len.wrapping_add((*gd).sx);
    }
    if window_copy_last_regex(gd, py, first, last, len, ppx, psx, buf, reg, eflags) != 0 {
        free(buf as *mut ::core::ffi::c_void);
        return 1 as ::core::ffi::c_int;
    }
    free(buf as *mut ::core::ffi::c_void);
    *ppx = 0 as u_int;
    *psx = 0 as u_int;
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn window_copy_cellstring(
    mut gl: *const grid_line,
    mut px: u_int,
    mut size: *mut size_t,
    mut allocated: *mut ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    static mut ud: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    let mut gce: *mut grid_cell_entry = ::core::ptr::null_mut::<grid_cell_entry>();
    let mut copy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if px >= (*gl).cellsize as u_int {
        *size = 1 as size_t;
        *allocated = 0 as ::core::ffi::c_int;
        return b" \0" as *const u8 as *const ::core::ffi::c_char;
    }
    gce = (*gl).celldata.offset(px as isize) as *mut grid_cell_entry;
    if (*gce).flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0 {
        *size = 0 as size_t;
        *allocated = 0 as ::core::ffi::c_int;
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    if !((*gce).flags as ::core::ffi::c_int) & GRID_FLAG_EXTENDED != 0 {
        *size = 1 as size_t;
        *allocated = 0 as ::core::ffi::c_int;
        return &raw mut (*gce).c2rust_unnamed.data.data as *const ::core::ffi::c_char;
    }
    if (*gce).flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
        *size = 1 as size_t;
        *allocated = 0 as ::core::ffi::c_int;
        return b"\t\0" as *const u8 as *const ::core::ffi::c_char;
    }
    utf8_to_data(
        (*(*gl).extddata.offset((*gce).c2rust_unnamed.offset as isize)).data,
        &raw mut ud,
    );
    if ud.size as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        *size = 0 as size_t;
        *allocated = 0 as ::core::ffi::c_int;
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    *size = ud.size as size_t;
    *allocated = 1 as ::core::ffi::c_int;
    copy = xmalloc(ud.size as size_t) as *mut ::core::ffi::c_char;
    memcpy(
        copy as *mut ::core::ffi::c_void,
        &raw mut ud.data as *mut u_char as *const ::core::ffi::c_void,
        ud.size as size_t,
    );
    return copy;
}
unsafe extern "C" fn window_copy_last_regex(
    mut gd: *mut grid,
    mut py: u_int,
    mut first: u_int,
    mut last: u_int,
    mut len: u_int,
    mut ppx: *mut u_int,
    mut psx: *mut u_int,
    mut buf: *const ::core::ffi::c_char,
    mut preg: *const regex_t,
    mut eflags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut foundx: u_int = 0;
    let mut foundy: u_int = 0;
    let mut oldx: u_int = 0;
    let mut px: u_int = 0 as u_int;
    let mut savepx: u_int = 0;
    let mut savesx: u_int = 0 as u_int;
    let mut regmatch: regmatch_t = regmatch_t { rm_so: 0, rm_eo: 0 };
    foundx = first;
    foundy = py;
    oldx = first;
    while regexec(
        preg,
        buf.offset(px as isize),
        1 as size_t,
        &raw mut regmatch,
        eflags,
    ) == 0 as ::core::ffi::c_int
    {
        if regmatch.rm_so == regmatch.rm_eo {
            break;
        }
        window_copy_cstrtocellpos(
            gd,
            len,
            &raw mut foundx,
            &raw mut foundy,
            buf.offset(px as isize).offset(regmatch.rm_so as isize),
        );
        if foundy > py || foundx >= last {
            break;
        }
        len = len.wrapping_sub(foundx.wrapping_sub(oldx));
        savepx = foundx;
        window_copy_cstrtocellpos(
            gd,
            len,
            &raw mut foundx,
            &raw mut foundy,
            buf.offset(px as isize).offset(regmatch.rm_eo as isize),
        );
        if foundy > py || foundx >= last {
            *ppx = savepx;
            *psx = foundx;
            while foundy > py {
                *psx = (*psx).wrapping_add((*gd).sx);
                foundy = foundy.wrapping_sub(1);
            }
            *psx = (*psx).wrapping_sub(*ppx);
            return 1 as ::core::ffi::c_int;
        } else {
            savesx = foundx.wrapping_sub(savepx);
            len = len.wrapping_sub(savesx);
            oldx = foundx;
        }
        px = px.wrapping_add(regmatch.rm_eo as u_int);
    }
    if savesx > 0 as u_int {
        *ppx = savepx;
        *psx = savesx;
        return 1 as ::core::ffi::c_int;
    } else {
        *ppx = 0 as u_int;
        *psx = 0 as u_int;
        return 0 as ::core::ffi::c_int;
    };
}
unsafe extern "C" fn window_copy_stringify(
    mut gd: *mut grid,
    mut py: u_int,
    mut first: u_int,
    mut last: u_int,
    mut buf: *mut ::core::ffi::c_char,
    mut size: *mut u_int,
) -> *mut ::core::ffi::c_char {
    let mut ax: u_int = 0;
    let mut bx: u_int = 0;
    let mut newsize: u_int = *size;
    let mut gl: *const grid_line = ::core::ptr::null::<grid_line>();
    let mut d: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut bufsize: size_t = 1024 as size_t;
    let mut dlen: size_t = 0;
    let mut allocated: ::core::ffi::c_int = 0;
    while bufsize < newsize as size_t {
        bufsize = bufsize.wrapping_mul(2 as size_t);
    }
    buf = xrealloc(buf as *mut ::core::ffi::c_void, bufsize) as *mut ::core::ffi::c_char;
    gl = grid_peek_line(gd, py);
    if gl.is_null() {
        *buf.offset((*size).wrapping_sub(1 as u_int) as isize) = '\0' as i32 as ::core::ffi::c_char;
        return buf;
    }
    bx = (*size).wrapping_sub(1 as u_int);
    ax = first;
    while ax < last {
        d = window_copy_cellstring(gl, ax, &raw mut dlen, &raw mut allocated);
        newsize = (newsize as size_t).wrapping_add(dlen) as u_int as u_int;
        while bufsize < newsize as size_t {
            bufsize = bufsize.wrapping_mul(2 as size_t);
            buf = xrealloc(buf as *mut ::core::ffi::c_void, bufsize) as *mut ::core::ffi::c_char;
        }
        if dlen == 1 as size_t {
            let fresh1 = bx;
            bx = bx.wrapping_add(1);
            *buf.offset(fresh1 as isize) = *d;
        } else if dlen != 0 as size_t {
            memcpy(
                buf.offset(bx as isize) as *mut ::core::ffi::c_void,
                d as *const ::core::ffi::c_void,
                dlen,
            );
            bx = (bx as size_t).wrapping_add(dlen) as u_int as u_int;
        }
        if allocated != 0 {
            free(d as *mut ::core::ffi::c_void);
        }
        ax = ax.wrapping_add(1);
    }
    *buf.offset(newsize.wrapping_sub(1 as u_int) as isize) = '\0' as i32 as ::core::ffi::c_char;
    *size = newsize;
    return buf;
}
unsafe extern "C" fn window_copy_cstrtocellpos(
    mut gd: *mut grid,
    mut ncells: u_int,
    mut ppx: *mut u_int,
    mut ppy: *mut u_int,
    mut str: *const ::core::ffi::c_char,
) {
    let mut cell: u_int = 0;
    let mut ccell: u_int = 0;
    let mut px: u_int = 0;
    let mut pywrap: u_int = 0;
    let mut pos: u_int = 0;
    let mut len: u_int = 0;
    let mut match_0: ::core::ffi::c_int = 0;
    let mut gl: *const grid_line = ::core::ptr::null::<grid_line>();
    let mut d: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut dlen: size_t = 0;
    let mut cells: *mut C2RustUnnamed_47 = ::core::ptr::null_mut::<C2RustUnnamed_47>();
    cells = xreallocarray(
        NULL,
        ncells as size_t,
        ::core::mem::size_of::<C2RustUnnamed_47>() as size_t,
    ) as *mut C2RustUnnamed_47;
    cell = 0 as u_int;
    px = *ppx;
    pywrap = *ppy;
    gl = grid_peek_line(gd, pywrap);
    if gl.is_null() {
        free(cells as *mut ::core::ffi::c_void);
        return;
    }
    while cell < ncells {
        let ref mut fresh0 = (*cells.offset(cell as isize)).d;
        *fresh0 = window_copy_cellstring(
            gl,
            px,
            &raw mut (*cells.offset(cell as isize)).dlen,
            &raw mut (*cells.offset(cell as isize)).allocated,
        );
        cell = cell.wrapping_add(1);
        px = px.wrapping_add(1);
        if !(px == (*gd).sx) {
            continue;
        }
        px = 0 as u_int;
        pywrap = pywrap.wrapping_add(1);
        gl = grid_peek_line(gd, pywrap);
        if gl.is_null() {
            break;
        }
    }
    ncells = cell;
    cell = 0 as u_int;
    len = strlen(str) as u_int;
    while cell < ncells {
        ccell = cell;
        pos = 0 as u_int;
        match_0 = 1 as ::core::ffi::c_int;
        while ccell < ncells {
            if *str.offset(pos as isize) as ::core::ffi::c_int == '\0' as i32 {
                match_0 = 0 as ::core::ffi::c_int;
                break;
            } else {
                d = (*cells.offset(ccell as isize)).d;
                dlen = (*cells.offset(ccell as isize)).dlen;
                if dlen == 1 as size_t {
                    if *str.offset(pos as isize) as ::core::ffi::c_int != *d as ::core::ffi::c_int {
                        match_0 = 0 as ::core::ffi::c_int;
                        break;
                    } else {
                        pos = pos.wrapping_add(1);
                    }
                } else {
                    if dlen > len.wrapping_sub(pos) as size_t {
                        dlen = len.wrapping_sub(pos) as size_t;
                    }
                    if memcmp(
                        str.offset(pos as isize) as *const ::core::ffi::c_void,
                        d as *const ::core::ffi::c_void,
                        dlen,
                    ) != 0 as ::core::ffi::c_int
                    {
                        match_0 = 0 as ::core::ffi::c_int;
                        break;
                    } else {
                        pos = (pos as size_t).wrapping_add(dlen) as u_int as u_int;
                    }
                }
                ccell = ccell.wrapping_add(1);
            }
        }
        if match_0 != 0 {
            break;
        }
        cell = cell.wrapping_add(1);
    }
    px = (*ppx).wrapping_add(cell);
    pywrap = *ppy;
    while px >= (*gd).sx {
        px = px.wrapping_sub((*gd).sx);
        pywrap = pywrap.wrapping_add(1);
    }
    *ppx = px;
    *ppy = pywrap;
    cell = 0 as u_int;
    while cell < ncells {
        if (*cells.offset(cell as isize)).allocated != 0 {
            free((*cells.offset(cell as isize)).d as *mut ::core::ffi::c_void);
        }
        cell = cell.wrapping_add(1);
    }
    free(cells as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn window_copy_move_left(
    mut s: *mut screen,
    mut fx: *mut u_int,
    mut fy: *mut u_int,
    mut wrapflag: ::core::ffi::c_int,
) {
    if *fx == 0 as u_int {
        if *fy == 0 as u_int {
            if wrapflag != 0 {
                *fx = (*(*s).grid).sx.wrapping_sub(1 as u_int);
                *fy = (*(*s).grid)
                    .hsize
                    .wrapping_add((*(*s).grid).sy)
                    .wrapping_sub(1 as u_int);
            }
            return;
        }
        *fx = (*(*s).grid).sx.wrapping_sub(1 as u_int);
        *fy = (*fy).wrapping_sub(1 as u_int);
    } else {
        *fx = (*fx).wrapping_sub(1 as u_int);
    };
}
unsafe extern "C" fn window_copy_move_right(
    mut s: *mut screen,
    mut fx: *mut u_int,
    mut fy: *mut u_int,
    mut wrapflag: ::core::ffi::c_int,
) {
    if *fx == (*(*s).grid).sx.wrapping_sub(1 as u_int) {
        if *fy
            == (*(*s).grid)
                .hsize
                .wrapping_add((*(*s).grid).sy)
                .wrapping_sub(1 as u_int)
        {
            if wrapflag != 0 {
                *fx = 0 as u_int;
                *fy = 0 as u_int;
            }
            return;
        }
        *fx = 0 as u_int;
        *fy = (*fy).wrapping_add(1 as u_int);
    } else {
        *fx = (*fx).wrapping_add(1 as u_int);
    };
}
unsafe extern "C" fn window_copy_is_lowercase(
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
unsafe extern "C" fn window_copy_search_back_overlap(
    mut gd: *mut grid,
    mut preg: *mut regex_t,
    mut ppx: *mut u_int,
    mut psx: *mut u_int,
    mut ppy: *mut u_int,
    mut endline: u_int,
) {
    let mut endx: u_int = 0;
    let mut endy: u_int = 0;
    let mut oldendx: u_int = 0;
    let mut oldendy: u_int = 0;
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut sx: u_int = 0;
    let mut found: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    oldendx = (*ppx).wrapping_add(*psx);
    oldendy = (*ppy).wrapping_sub(1 as u_int);
    while oldendx > (*gd).sx.wrapping_sub(1 as u_int) {
        oldendx = oldendx.wrapping_sub((*gd).sx);
        oldendy = oldendy.wrapping_add(1);
    }
    endx = oldendx;
    endy = oldendy;
    px = *ppx;
    py = *ppy;
    while found != 0
        && px == 0 as u_int
        && py.wrapping_sub(1 as u_int) > endline
        && (*grid_get_line(gd, py.wrapping_sub(2 as u_int))).flags as ::core::ffi::c_int
            & GRID_LINE_WRAPPED
            != 0
        && endx == oldendx
        && endy == oldendy
    {
        py = py.wrapping_sub(1);
        found = window_copy_search_rl_regex(
            gd,
            &raw mut px,
            &raw mut sx,
            py.wrapping_sub(1 as u_int),
            0 as u_int,
            (*gd).sx,
            preg,
        );
        if found != 0 {
            endx = px.wrapping_add(sx);
            endy = py.wrapping_sub(1 as u_int);
            while endx > (*gd).sx.wrapping_sub(1 as u_int) {
                endx = endx.wrapping_sub((*gd).sx);
                endy = endy.wrapping_add(1);
            }
            if endx == oldendx && endy == oldendy {
                *ppx = px;
                *ppy = py;
            }
        }
    }
}
unsafe extern "C" fn window_copy_search_jump(
    mut wme: *mut window_mode_entry,
    mut gd: *mut grid,
    mut sgd: *mut grid,
    mut fx: u_int,
    mut fy: u_int,
    mut endline: u_int,
    mut cis: ::core::ffi::c_int,
    mut wrap: ::core::ffi::c_int,
    mut direction: ::core::ffi::c_int,
    mut regex: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut i: u_int = 0;
    let mut px: u_int = 0;
    let mut sx: u_int = 0;
    let mut ssize: u_int = 1 as u_int;
    let mut found: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut cflags: ::core::ffi::c_int = REG_EXTENDED;
    let mut sbuf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut reg: regex_t = re_pattern_buffer {
        buffer: ::core::ptr::null_mut::<re_dfa_t>(),
        allocated: 0,
        used: 0,
        syntax: 0,
        fastmap: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        translate: ::core::ptr::null_mut::<::core::ffi::c_uchar>(),
        re_nsub: 0,
        can_be_null_regs_allocated_fastmap_accurate_no_sub_not_bol_not_eol_newline_anchor: [0; 1],
        c2rust_padding: [0; 7],
    };
    if regex != 0 {
        sbuf = xmalloc(ssize as size_t) as *mut ::core::ffi::c_char;
        *sbuf.offset(0 as ::core::ffi::c_int as isize) = '\0' as i32 as ::core::ffi::c_char;
        sbuf = window_copy_stringify(sgd, 0 as u_int, 0 as u_int, (*sgd).sx, sbuf, &raw mut ssize);
        if cis != 0 {
            cflags |= REG_ICASE;
        }
        if regcomp(&raw mut reg, sbuf, cflags) != 0 as ::core::ffi::c_int {
            free(sbuf as *mut ::core::ffi::c_void);
            return 0 as ::core::ffi::c_int;
        }
        free(sbuf as *mut ::core::ffi::c_void);
    }
    if direction != 0 {
        i = fy;
        while i <= endline {
            if regex != 0 {
                found = window_copy_search_lr_regex(
                    gd,
                    &raw mut px,
                    &raw mut sx,
                    i,
                    fx,
                    (*gd).sx,
                    &raw mut reg,
                );
            } else {
                found = window_copy_search_lr(gd, sgd, &raw mut px, i, fx, (*gd).sx, cis);
            }
            if found != 0 {
                break;
            }
            fx = 0 as u_int;
            i = i.wrapping_add(1);
        }
    } else {
        i = fy.wrapping_add(1 as u_int);
        while endline < i {
            if regex != 0 {
                found = window_copy_search_rl_regex(
                    gd,
                    &raw mut px,
                    &raw mut sx,
                    i.wrapping_sub(1 as u_int),
                    0 as u_int,
                    fx.wrapping_add(1 as u_int),
                    &raw mut reg,
                );
                if found != 0 {
                    window_copy_search_back_overlap(
                        gd,
                        &raw mut reg,
                        &raw mut px,
                        &raw mut sx,
                        &raw mut i,
                        endline,
                    );
                }
            } else {
                found = window_copy_search_rl(
                    gd,
                    sgd,
                    &raw mut px,
                    i.wrapping_sub(1 as u_int),
                    0 as u_int,
                    fx.wrapping_add(1 as u_int),
                    cis,
                );
            }
            if found != 0 {
                i = i.wrapping_sub(1);
                break;
            } else {
                fx = (*gd).sx.wrapping_sub(1 as u_int);
                i = i.wrapping_sub(1);
            }
        }
    }
    if regex != 0 {
        regfree(&raw mut reg);
    }
    if found != 0 {
        window_copy_scroll_to(wme, px, i, 1 as ::core::ffi::c_int);
        return 1 as ::core::ffi::c_int;
    }
    if wrap != 0 {
        return window_copy_search_jump(
            wme,
            gd,
            sgd,
            if direction != 0 {
                0 as u_int
            } else {
                (*gd).sx.wrapping_sub(1 as u_int)
            },
            if direction != 0 {
                0 as u_int
            } else {
                (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(1 as u_int)
            },
            fy,
            cis,
            0 as ::core::ffi::c_int,
            direction,
            regex,
        );
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn window_copy_move_after_search_mark(
    mut data: *mut window_copy_mode_data,
    mut fx: *mut u_int,
    mut fy: *mut u_int,
    mut wrapflag: ::core::ffi::c_int,
) {
    let mut s: *mut screen = (*data).backing;
    let mut at: u_int = 0;
    let mut start: u_int = 0;
    if window_copy_search_mark_at(data, *fx, *fy, &raw mut start) == 0 as ::core::ffi::c_int
        && *(*data).searchmark.offset(start as isize) as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        while window_copy_search_mark_at(data, *fx, *fy, &raw mut at) == 0 as ::core::ffi::c_int {
            if *(*data).searchmark.offset(at as isize) as ::core::ffi::c_int
                != *(*data).searchmark.offset(start as isize) as ::core::ffi::c_int
            {
                break;
            }
            if wrapflag == 0
                && *fx == (*(*s).grid).sx.wrapping_sub(1 as u_int)
                && *fy
                    == (*(*s).grid)
                        .hsize
                        .wrapping_add((*(*s).grid).sy)
                        .wrapping_sub(1 as u_int)
            {
                break;
            }
            window_copy_move_right(s, fx, fy, wrapflag);
        }
    }
}
unsafe extern "C" fn window_copy_search(
    mut wme: *mut window_mode_entry,
    mut direction: ::core::ffi::c_int,
    mut regex: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut wp: *mut window_pane = (*wme).wp;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut s: *mut screen = (*data).backing;
    let mut ss: screen = screen {
        title: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        path: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        titles: ::core::ptr::null_mut::<screen_titles>(),
        ntitles: 0,
        grid: ::core::ptr::null_mut::<grid>(),
        cx: 0,
        cy: 0,
        cstyle: SCREEN_CURSOR_DEFAULT,
        default_cstyle: SCREEN_CURSOR_DEFAULT,
        ccolour: 0,
        default_ccolour: 0,
        rupper: 0,
        rlower: 0,
        mode: 0,
        default_mode: 0,
        saved_cx: 0,
        saved_cy: 0,
        saved_grid: ::core::ptr::null_mut::<grid>(),
        saved_cell: grid_cell {
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
        saved_flags: 0,
        tabs: ::core::ptr::null_mut::<bitstr_t>(),
        sel: ::core::ptr::null_mut::<screen_sel>(),
        write_list: ::core::ptr::null_mut::<screen_write_cline>(),
        hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        progress_bar: progress_bar {
            state: PROGRESS_BAR_HIDDEN,
            progress: 0,
        },
    };
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
    let mut gd: *mut grid = (*s).grid;
    let mut str: *const ::core::ffi::c_char = (*data).searchstr;
    let mut at: u_int = 0;
    let mut endline: u_int = 0;
    let mut fx: u_int = 0;
    let mut fy: u_int = 0;
    let mut start: u_int = 0;
    let mut ssx: u_int = 0;
    let mut cis: ::core::ffi::c_int = 0;
    let mut found: ::core::ffi::c_int = 0;
    let mut keys: ::core::ffi::c_int = 0;
    let mut visible_only: ::core::ffi::c_int = 0;
    let mut wrapflag: ::core::ffi::c_int = 0;
    if regex != 0
        && *str.offset(strcspn(
            str,
            b"^$*+()?[].\\\0" as *const u8 as *const ::core::ffi::c_char,
        ) as isize) as ::core::ffi::c_int
            == '\0' as i32
    {
        regex = 0 as ::core::ffi::c_int;
    }
    (*data).searchdirection = direction;
    if (*data).timeout != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if (*data).searchall != 0 || (*wp).searchstr.is_null() || (*wp).searchregex != regex {
        visible_only = 0 as ::core::ffi::c_int;
        (*data).searchall = 0 as ::core::ffi::c_int;
    } else {
        visible_only =
            (strcmp((*wp).searchstr, str) == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
    }
    if visible_only == 0 as ::core::ffi::c_int && !(*data).searchmark.is_null() {
        window_copy_clear_marks(wme);
    }
    free((*wp).searchstr as *mut ::core::ffi::c_void);
    (*wp).searchstr = xstrdup(str);
    (*wp).searchregex = regex;
    fx = (*data).cx;
    fy = (*(*(*data).backing).grid)
        .hsize
        .wrapping_sub((*data).oy)
        .wrapping_add((*data).cy);
    ssx = screen_write_strlen(b"%s\0" as *const u8 as *const ::core::ffi::c_char, str) as u_int;
    if ssx == 0 as u_int {
        return 0 as ::core::ffi::c_int;
    }
    screen_init(&raw mut ss, ssx, 1 as u_int, 0 as u_int);
    screen_write_start(&raw mut ctx, &raw mut ss);
    screen_write_nputs(
        &raw mut ctx,
        -(1 as ::core::ffi::c_int) as ssize_t,
        &raw const grid_default_cell,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        str,
    );
    screen_write_stop(&raw mut ctx);
    wrapflag = options_get_number(
        (*(*wp).window).options,
        b"wrap-search\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    cis = window_copy_is_lowercase(str);
    keys = options_get_number(
        (*(*wp).window).options,
        b"mode-keys\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    if direction != 0 {
        if keys == MODEKEY_VI {
            if !(*data).searchmark.is_null() {
                window_copy_move_after_search_mark(data, &raw mut fx, &raw mut fy, wrapflag);
            } else {
                window_copy_move_right(s, &raw mut fx, &raw mut fy, wrapflag);
            }
        }
        endline = (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(1 as u_int);
    } else {
        window_copy_move_left(s, &raw mut fx, &raw mut fy, wrapflag);
        endline = 0 as u_int;
    }
    found = window_copy_search_jump(
        wme, gd, ss.grid, fx, fy, endline, cis, wrapflag, direction, regex,
    );
    if found != 0 {
        window_copy_search_marks(wme, &raw mut ss, regex, visible_only);
        fx = (*data).cx;
        fy = (*(*(*data).backing).grid)
            .hsize
            .wrapping_sub((*data).oy)
            .wrapping_add((*data).cy);
        if direction != 0
            && window_copy_search_mark_at(data, fx, fy, &raw mut at) == 0 as ::core::ffi::c_int
            && at > 0 as u_int
            && !(*data).searchmark.is_null()
            && *(*data).searchmark.offset(at as isize) as ::core::ffi::c_int
                == *(*data)
                    .searchmark
                    .offset(at.wrapping_sub(1 as u_int) as isize)
                    as ::core::ffi::c_int
        {
            window_copy_move_after_search_mark(data, &raw mut fx, &raw mut fy, wrapflag);
            window_copy_search_jump(
                wme, gd, ss.grid, fx, fy, endline, cis, wrapflag, direction, regex,
            );
            fx = (*data).cx;
            fy = (*(*(*data).backing).grid)
                .hsize
                .wrapping_sub((*data).oy)
                .wrapping_add((*data).cy);
        }
        if direction != 0 {
            if keys == MODEKEY_EMACS {
                window_copy_move_after_search_mark(data, &raw mut fx, &raw mut fy, wrapflag);
                (*data).cx = fx;
                (*data).cy = fy
                    .wrapping_sub((*(*(*data).backing).grid).hsize)
                    .wrapping_add((*data).oy);
            }
        } else if window_copy_search_mark_at(data, fx, fy, &raw mut start)
            == 0 as ::core::ffi::c_int
        {
            while window_copy_search_mark_at(data, fx, fy, &raw mut at) == 0 as ::core::ffi::c_int
                && !(*data).searchmark.is_null()
                && *(*data).searchmark.offset(at as isize) as ::core::ffi::c_int
                    == *(*data).searchmark.offset(start as isize) as ::core::ffi::c_int
            {
                (*data).cx = fx;
                (*data).cy = fy
                    .wrapping_sub((*(*(*data).backing).grid).hsize)
                    .wrapping_add((*data).oy);
                if at == 0 as u_int {
                    break;
                }
                window_copy_move_left(s, &raw mut fx, &raw mut fy, 0 as ::core::ffi::c_int);
            }
        }
    }
    window_copy_redraw_screen(wme);
    screen_free(&raw mut ss);
    return found;
}
unsafe extern "C" fn window_copy_visible_lines(
    mut data: *mut window_copy_mode_data,
    mut start: *mut u_int,
    mut end: *mut u_int,
) {
    let mut gd: *mut grid = (*(*data).backing).grid;
    let mut gl: *const grid_line = ::core::ptr::null::<grid_line>();
    *start = (*gd).hsize.wrapping_sub((*data).oy);
    while *start > 0 as u_int {
        gl = grid_peek_line(gd, (*start).wrapping_sub(1 as u_int));
        if gl.is_null() || !((*gl).flags as ::core::ffi::c_int) & GRID_LINE_WRAPPED != 0 {
            break;
        }
        *start = (*start).wrapping_sub(1);
    }
    *end = (*gd).hsize.wrapping_sub((*data).oy).wrapping_add((*gd).sy);
}
unsafe extern "C" fn window_copy_search_mark_at(
    mut data: *mut window_copy_mode_data,
    mut px: u_int,
    mut py: u_int,
    mut at: *mut u_int,
) -> ::core::ffi::c_int {
    let mut s: *mut screen = (*data).backing;
    let mut gd: *mut grid = (*s).grid;
    if py < (*gd).hsize.wrapping_sub((*data).oy) {
        return -(1 as ::core::ffi::c_int);
    }
    if py
        > (*gd)
            .hsize
            .wrapping_sub((*data).oy)
            .wrapping_add((*gd).sy)
            .wrapping_sub(1 as u_int)
    {
        return -(1 as ::core::ffi::c_int);
    }
    *at = py
        .wrapping_sub((*gd).hsize.wrapping_sub((*data).oy))
        .wrapping_mul((*gd).sx)
        .wrapping_add(px);
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn window_copy_clip_width(
    mut width: u_int,
    mut b: u_int,
    mut sx: u_int,
    mut sy: u_int,
) -> u_int {
    return if b.wrapping_add(width) > sx.wrapping_mul(sy) {
        sx.wrapping_mul(sy).wrapping_sub(b)
    } else {
        width
    };
}
unsafe extern "C" fn window_copy_search_mark_match(
    mut data: *mut window_copy_mode_data,
    mut px: u_int,
    mut py: u_int,
    mut width: u_int,
    mut regex: ::core::ffi::c_int,
) -> u_int {
    let mut gd: *mut grid = (*(*data).backing).grid;
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
    let mut i: u_int = 0;
    let mut b: u_int = 0;
    let mut w: u_int = width;
    let mut sx: u_int = (*gd).sx;
    let mut sy: u_int = (*gd).sy;
    if window_copy_search_mark_at(data, px, py, &raw mut b) == 0 as ::core::ffi::c_int {
        width = window_copy_clip_width(width, b, sx, sy);
        w = width;
        i = b;
        while i < b.wrapping_add(w) {
            if regex == 0 {
                grid_get_cell(gd, px.wrapping_add(i.wrapping_sub(b)), py, &raw mut gc);
                if gc.flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
                    w = w.wrapping_add(
                        (gc.data.width as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as u_int,
                    );
                }
                w = window_copy_clip_width(w, b, sx, sy);
            }
            if !(*(*data).searchmark.offset(i as isize) as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int)
            {
                *(*data).searchmark.offset(i as isize) = (*data).searchgen;
            }
            i = i.wrapping_add(1);
        }
        if (*data).searchgen as ::core::ffi::c_int == UCHAR_MAX {
            (*data).searchgen = 1 as u_char;
        } else {
            (*data).searchgen = (*data).searchgen.wrapping_add(1);
        }
    }
    return w;
}
unsafe extern "C" fn window_copy_search_marks(
    mut wme: *mut window_mode_entry,
    mut ssp: *mut screen,
    mut regex: ::core::ffi::c_int,
    mut visible_only: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut s: *mut screen = (*data).backing;
    let mut ss: screen = screen {
        title: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        path: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        titles: ::core::ptr::null_mut::<screen_titles>(),
        ntitles: 0,
        grid: ::core::ptr::null_mut::<grid>(),
        cx: 0,
        cy: 0,
        cstyle: SCREEN_CURSOR_DEFAULT,
        default_cstyle: SCREEN_CURSOR_DEFAULT,
        ccolour: 0,
        default_ccolour: 0,
        rupper: 0,
        rlower: 0,
        mode: 0,
        default_mode: 0,
        saved_cx: 0,
        saved_cy: 0,
        saved_grid: ::core::ptr::null_mut::<grid>(),
        saved_cell: grid_cell {
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
        saved_flags: 0,
        tabs: ::core::ptr::null_mut::<bitstr_t>(),
        sel: ::core::ptr::null_mut::<screen_sel>(),
        write_list: ::core::ptr::null_mut::<screen_write_cline>(),
        hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        progress_bar: progress_bar {
            state: PROGRESS_BAR_HIDDEN,
            progress: 0,
        },
    };
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
    let mut gd: *mut grid = (*s).grid;
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
    let mut found: ::core::ffi::c_int = 0;
    let mut cis: ::core::ffi::c_int = 0;
    let mut stopped: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut cflags: ::core::ffi::c_int = REG_EXTENDED;
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut nfound: u_int = 0 as u_int;
    let mut width: u_int = 0;
    let mut ssize: u_int = 1 as u_int;
    let mut start: u_int = 0;
    let mut end: u_int = 0;
    let mut sx: u_int = (*gd).sx;
    let mut sy: u_int = (*gd).sy;
    let mut sbuf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut reg: regex_t = re_pattern_buffer {
        buffer: ::core::ptr::null_mut::<re_dfa_t>(),
        allocated: 0,
        used: 0,
        syntax: 0,
        fastmap: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        translate: ::core::ptr::null_mut::<::core::ffi::c_uchar>(),
        re_nsub: 0,
        can_be_null_regs_allocated_fastmap_accurate_no_sub_not_bol_not_eol_newline_anchor: [0; 1],
        c2rust_padding: [0; 7],
    };
    let mut stop: uint64_t = 0 as uint64_t;
    let mut tstart: uint64_t = 0;
    let mut t: uint64_t = 0;
    if ssp.is_null() {
        width = screen_write_strlen(
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            (*data).searchstr,
        ) as u_int;
        screen_init(&raw mut ss, width, 1 as u_int, 0 as u_int);
        screen_write_start(&raw mut ctx, &raw mut ss);
        screen_write_nputs(
            &raw mut ctx,
            -(1 as ::core::ffi::c_int) as ssize_t,
            &raw const grid_default_cell,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            (*data).searchstr,
        );
        screen_write_stop(&raw mut ctx);
        ssp = &raw mut ss;
    } else {
        width = (*(*ssp).grid).sx;
    }
    cis = window_copy_is_lowercase((*data).searchstr);
    if regex != 0 {
        sbuf = xmalloc(ssize as size_t) as *mut ::core::ffi::c_char;
        *sbuf.offset(0 as ::core::ffi::c_int as isize) = '\0' as i32 as ::core::ffi::c_char;
        sbuf = window_copy_stringify(
            (*ssp).grid,
            0 as u_int,
            0 as u_int,
            (*(*ssp).grid).sx,
            sbuf,
            &raw mut ssize,
        );
        if cis != 0 {
            cflags |= REG_ICASE;
        }
        if regcomp(&raw mut reg, sbuf, cflags) != 0 as ::core::ffi::c_int {
            free(sbuf as *mut ::core::ffi::c_void);
            free((*data).searchmark as *mut ::core::ffi::c_void);
            (*data).searchmark = ::core::ptr::null_mut::<u_char>();
            return 0 as ::core::ffi::c_int;
        }
        free(sbuf as *mut ::core::ffi::c_void);
    }
    tstart = get_timer();
    if visible_only != 0 {
        window_copy_visible_lines(data, &raw mut start, &raw mut end);
    } else {
        start = 0 as u_int;
        end = (*gd).hsize.wrapping_add(sy);
        stop = get_timer().wrapping_add(WINDOW_COPY_SEARCH_ALL_TIMEOUT as uint64_t);
    }
    loop {
        free((*data).searchmark as *mut ::core::ffi::c_void);
        (*data).searchmark = xcalloc(sx as size_t, sy as size_t) as *mut u_char;
        (*data).searchgen = 1 as u_char;
        py = start;
        while py < end {
            px = 0 as u_int;
            loop {
                if regex != 0 {
                    found = window_copy_search_lr_regex(
                        gd,
                        &raw mut px,
                        &raw mut width,
                        py,
                        px,
                        sx,
                        &raw mut reg,
                    );
                    grid_get_cell(
                        gd,
                        px.wrapping_add(width).wrapping_sub(1 as u_int),
                        py,
                        &raw mut gc,
                    );
                    if gc.data.width as ::core::ffi::c_int > 2 as ::core::ffi::c_int {
                        width = width.wrapping_add(
                            (gc.data.width as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                                as u_int,
                        );
                    }
                    if found == 0 {
                        break;
                    }
                } else {
                    found = window_copy_search_lr(gd, (*ssp).grid, &raw mut px, py, px, sx, cis);
                    if found == 0 {
                        break;
                    }
                }
                nfound = nfound.wrapping_add(1);
                px = px.wrapping_add(window_copy_search_mark_match(data, px, py, width, regex));
            }
            t = get_timer();
            if t.wrapping_sub(tstart) > WINDOW_COPY_SEARCH_TIMEOUT as uint64_t {
                (*data).timeout = 1 as ::core::ffi::c_int;
                break;
            } else if stop != 0 as uint64_t && t > stop {
                stopped = 1 as ::core::ffi::c_int;
                break;
            } else {
                py = py.wrapping_add(1);
            }
        }
        if (*data).timeout != 0 {
            window_copy_clear_marks(wme);
            break;
        } else if stopped != 0 && stop != 0 as uint64_t {
            window_copy_visible_lines(data, &raw mut start, &raw mut end);
            stop = 0 as uint64_t;
        } else {
            if visible_only == 0 {
                if stopped != 0 {
                    if nfound > 1000 as u_int {
                        (*data).searchcount = 1000 as ::core::ffi::c_int;
                    } else if nfound > 100 as u_int {
                        (*data).searchcount = 100 as ::core::ffi::c_int;
                    } else if nfound > 10 as u_int {
                        (*data).searchcount = 10 as ::core::ffi::c_int;
                    } else {
                        (*data).searchcount = -(1 as ::core::ffi::c_int);
                    }
                    (*data).searchmore = 1 as ::core::ffi::c_int;
                } else {
                    (*data).searchcount = nfound as ::core::ffi::c_int;
                    (*data).searchmore = 0 as ::core::ffi::c_int;
                }
            }
            break;
        }
    }
    if ssp == &raw mut ss {
        screen_free(&raw mut ss);
    }
    if regex != 0 {
        regfree(&raw mut reg);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn window_copy_clear_marks(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    (*data).searchcount = -(1 as ::core::ffi::c_int);
    (*data).searchmore = 0 as ::core::ffi::c_int;
    free((*data).searchmark as *mut ::core::ffi::c_void);
    (*data).searchmark = ::core::ptr::null_mut::<u_char>();
}
unsafe extern "C" fn window_copy_search_up(
    mut wme: *mut window_mode_entry,
    mut regex: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    return window_copy_search(wme, 0 as ::core::ffi::c_int, regex);
}
unsafe extern "C" fn window_copy_search_down(
    mut wme: *mut window_mode_entry,
    mut regex: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    return window_copy_search(wme, 1 as ::core::ffi::c_int, regex);
}
unsafe extern "C" fn window_copy_goto_line(
    mut wme: *mut window_mode_entry,
    mut linestr: *const ::core::ffi::c_char,
) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut hsize: u_int = (*(*(*data).backing).grid).hsize;
    let mut line: u_int = 0;
    let mut lineno: ::core::ffi::c_int = 0;
    lineno = strtonum(
        linestr,
        -(1 as ::core::ffi::c_int) as ::core::ffi::c_longlong,
        INT_MAX as ::core::ffi::c_longlong,
        &raw mut errstr,
    ) as ::core::ffi::c_int;
    if !errstr.is_null() {
        return;
    }
    if window_copy_line_number_is_absolute(wme) != 0 {
        if lineno <= 0 as ::core::ffi::c_int {
            line = 1 as u_int;
        } else if lineno as u_int > hsize.wrapping_add(1 as u_int) {
            line = hsize.wrapping_add(1 as u_int);
        } else {
            line = lineno as u_int;
        }
        (*data).oy = hsize.wrapping_sub(line.wrapping_sub(1 as u_int));
    } else {
        if lineno < 0 as ::core::ffi::c_int || lineno as u_int > hsize {
            lineno = hsize as ::core::ffi::c_int;
        }
        (*data).oy = lineno as u_int;
    }
    window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    window_copy_redraw_screen(wme);
}
unsafe extern "C" fn window_copy_match_start_end(
    mut data: *mut window_copy_mode_data,
    mut at: u_int,
    mut start: *mut u_int,
    mut end: *mut u_int,
) {
    let mut gd: *mut grid = (*(*data).backing).grid;
    let mut last: u_int = (*gd).sy.wrapping_mul((*gd).sx).wrapping_sub(1 as u_int);
    let mut mark: u_char = *(*data).searchmark.offset(at as isize);
    *end = at;
    *start = *end;
    while *start != 0 as u_int
        && *(*data).searchmark.offset(*start as isize) as ::core::ffi::c_int
            == mark as ::core::ffi::c_int
    {
        *start = (*start).wrapping_sub(1);
    }
    if *(*data).searchmark.offset(*start as isize) as ::core::ffi::c_int
        != mark as ::core::ffi::c_int
    {
        *start = (*start).wrapping_add(1);
    }
    while *end != last
        && *(*data).searchmark.offset(*end as isize) as ::core::ffi::c_int
            == mark as ::core::ffi::c_int
    {
        *end = (*end).wrapping_add(1);
    }
    if *(*data).searchmark.offset(*end as isize) as ::core::ffi::c_int != mark as ::core::ffi::c_int
    {
        *end = (*end).wrapping_sub(1);
    }
}
unsafe extern "C" fn window_copy_match_at_cursor(
    mut data: *mut window_copy_mode_data,
) -> *mut ::core::ffi::c_char {
    let mut gd: *mut grid = (*(*data).backing).grid;
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
    let mut at: u_int = 0;
    let mut start: u_int = 0;
    let mut end: u_int = 0;
    let mut cy: u_int = 0;
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut sx: u_int = (*(*(*data).backing).grid).sx;
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: size_t = 0 as size_t;
    if (*data).searchmark.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    cy = (*(*(*data).backing).grid)
        .hsize
        .wrapping_sub((*data).oy)
        .wrapping_add((*data).cy);
    if window_copy_search_mark_at(data, (*data).cx, cy, &raw mut at) != 0 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if *(*data).searchmark.offset(at as isize) as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        if at == 0 as u_int || {
            at = at.wrapping_sub(1);
            *(*data).searchmark.offset(at as isize) as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        } {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
    }
    window_copy_match_start_end(data, at, &raw mut start, &raw mut end);
    at = start;
    while at <= end {
        py = at.wrapping_div(sx);
        px = at.wrapping_sub(py.wrapping_mul(sx));
        grid_get_cell(
            gd,
            px,
            (*gd).hsize.wrapping_add(py).wrapping_sub((*data).oy),
            &raw mut gc,
        );
        if gc.flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
            buf = xrealloc(
                buf as *mut ::core::ffi::c_void,
                len.wrapping_add(2 as size_t),
            ) as *mut ::core::ffi::c_char;
            *buf.offset(len as isize) = '\t' as i32 as ::core::ffi::c_char;
            len = len.wrapping_add(1);
        } else if !(gc.flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0) {
            buf = xrealloc(
                buf as *mut ::core::ffi::c_void,
                len.wrapping_add(gc.data.size as size_t)
                    .wrapping_add(1 as size_t),
            ) as *mut ::core::ffi::c_char;
            memcpy(
                buf.offset(len as isize) as *mut ::core::ffi::c_void,
                &raw mut gc.data.data as *mut u_char as *const ::core::ffi::c_void,
                gc.data.size as size_t,
            );
            len = len.wrapping_add(gc.data.size as size_t);
        }
        at = at.wrapping_add(1);
    }
    if len != 0 as size_t {
        *buf.offset(len as isize) = '\0' as i32 as ::core::ffi::c_char;
    }
    return buf;
}
unsafe extern "C" fn window_copy_update_style(
    mut wme: *mut window_mode_entry,
    mut fx: u_int,
    mut fy: u_int,
    mut gc: *mut grid_cell,
    mut mgc: *const grid_cell,
    mut cgc: *const grid_cell,
    mut mkgc: *const grid_cell,
    mut clgc: *const grid_cell,
) {
    let mut wp: *mut window_pane = (*wme).wp;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut mark: u_int = 0;
    let mut start: u_int = 0;
    let mut end: u_int = 0;
    let mut cy: u_int = 0;
    let mut cursor: u_int = 0;
    let mut current: u_int = 0;
    let mut inv: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut found: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut keys: ::core::ffi::c_int = 0;
    cy = (*(*(*data).backing).grid)
        .hsize
        .wrapping_sub((*data).oy)
        .wrapping_add((*data).cy);
    if fy == cy {
        if (*clgc).fg != 8 as ::core::ffi::c_int {
            (*gc).fg = (*clgc).fg;
        }
        if (*clgc).bg != 8 as ::core::ffi::c_int {
            (*gc).bg = (*clgc).bg;
        }
        (*gc).attr =
            ((*gc).attr as ::core::ffi::c_int | (*clgc).attr as ::core::ffi::c_int) as u_short;
    }
    if (*data).showmark != 0 && fy == (*data).my {
        (*gc).attr = (*mkgc).attr;
        if fx == (*data).mx {
            inv = 1 as ::core::ffi::c_int;
        }
        if inv != 0 {
            (*gc).fg = (*mkgc).bg;
            (*gc).bg = (*mkgc).fg;
        } else {
            (*gc).fg = (*mkgc).fg;
            (*gc).bg = (*mkgc).bg;
        }
    }
    if (*data).searchmark.is_null() {
        return;
    }
    if window_copy_search_mark_at(data, fx, fy, &raw mut current) != 0 as ::core::ffi::c_int {
        return;
    }
    mark = *(*data).searchmark.offset(current as isize) as u_int;
    if mark == 0 as u_int {
        return;
    }
    if window_copy_search_mark_at(data, (*data).cx, cy, &raw mut cursor) == 0 as ::core::ffi::c_int
    {
        keys = options_get_number(
            (*(*wp).window).options,
            b"mode-keys\0" as *const u8 as *const ::core::ffi::c_char,
        ) as ::core::ffi::c_int;
        if cursor != 0 as u_int && keys == MODEKEY_EMACS && (*data).searchdirection != 0 {
            if *(*data)
                .searchmark
                .offset(cursor.wrapping_sub(1 as u_int) as isize) as u_int
                == mark
            {
                cursor = cursor.wrapping_sub(1);
                found = 1 as ::core::ffi::c_int;
            }
        } else if *(*data).searchmark.offset(cursor as isize) as u_int == mark {
            found = 1 as ::core::ffi::c_int;
        }
        if found != 0 {
            window_copy_match_start_end(data, cursor, &raw mut start, &raw mut end);
            if current >= start && current <= end {
                (*gc).attr = (*cgc).attr;
                if inv != 0 {
                    (*gc).fg = (*cgc).bg;
                    (*gc).bg = (*cgc).fg;
                } else {
                    (*gc).fg = (*cgc).fg;
                    (*gc).bg = (*cgc).bg;
                }
                return;
            }
        }
    }
    (*gc).attr = (*mgc).attr;
    if inv != 0 {
        (*gc).fg = (*mgc).bg;
        (*gc).bg = (*mgc).fg;
    } else {
        (*gc).fg = (*mgc).fg;
        (*gc).bg = (*mgc).bg;
    };
}
unsafe extern "C" fn window_copy_write_one(
    mut wme: *mut window_mode_entry,
    mut ctx: *mut screen_write_ctx,
    mut px: u_int,
    mut py: u_int,
    mut fy: u_int,
    mut nx: u_int,
    mut mgc: *const grid_cell,
    mut cgc: *const grid_cell,
    mut mkgc: *const grid_cell,
    mut clgc: *const grid_cell,
) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut gd: *mut grid = (*(*data).backing).grid;
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
    let mut fx: u_int = 0;
    let mut i: u_int = 0;
    let mut width: u_int = 0;
    screen_write_cursormove(
        ctx,
        px as ::core::ffi::c_int,
        py as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    fx = 0 as u_int;
    while fx < nx {
        grid_get_cell(gd, fx, fy, &raw mut gc);
        if fx.wrapping_add(gc.data.width as u_int) <= nx {
            window_copy_update_style(wme, fx, fy, &raw mut gc, mgc, cgc, mkgc, clgc);
            if gc.flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0 {
                if (*(*ctx).s).cy == py && (*(*ctx).s).cx <= px.wrapping_add(fx) {
                    gc.flags = (gc.flags as ::core::ffi::c_int & !GRID_FLAG_PADDING) as u_char;
                    screen_write_cursormove(
                        ctx,
                        px.wrapping_add(fx) as ::core::ffi::c_int,
                        py as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    );
                    screen_write_putc(ctx, &raw mut gc, ' ' as i32 as u_char);
                }
            } else if gc.flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
                width = gc.data.width as u_int;
                gc.flags = (gc.flags as ::core::ffi::c_int & !GRID_FLAG_TAB) as u_char;
                i = 0 as u_int;
                while i < width {
                    screen_write_putc(ctx, &raw mut gc, ' ' as i32 as u_char);
                    i = i.wrapping_add(1);
                }
            } else {
                screen_write_cell(ctx, &raw mut gc);
            }
        } else {
            screen_write_putc(ctx, &raw const grid_default_cell, ' ' as i32 as u_char);
        }
        fx = fx.wrapping_add(1);
    }
}
unsafe extern "C" fn window_copy_line_number_mode(
    mut wme: *mut window_mode_entry,
) -> ::core::ffi::c_int {
    let mut wp: *mut window_pane = (*wme).wp;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut oo: *mut options = (*(*wp).window).options;
    let mut mode: ::core::ffi::c_int = 0;
    if (*data).line_numbers == 0 {
        return WINDOW_COPY_LINE_NUMBERS_OFF as ::core::ffi::c_int;
    }
    mode = options_get_number(
        oo,
        b"copy-mode-line-numbers\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    if (*data).line_numbers == 2 as ::core::ffi::c_int
        && mode == WINDOW_COPY_LINE_NUMBERS_OFF as ::core::ffi::c_int
    {
        return WINDOW_COPY_LINE_NUMBERS_DEFAULT as ::core::ffi::c_int;
    }
    return mode;
}
unsafe extern "C" fn window_copy_line_number_is_absolute(
    mut wme: *mut window_mode_entry,
) -> ::core::ffi::c_int {
    match window_copy_line_number_mode(wme) {
        2 | 3 | 4 => return 1 as ::core::ffi::c_int,
        0 | 1 => return 0 as ::core::ffi::c_int,
        _ => {}
    }
    fatalx(b"bad line number mode\0" as *const u8 as *const ::core::ffi::c_char);
}
unsafe extern "C" fn window_copy_line_numbers_active(
    mut wme: *mut window_mode_entry,
) -> ::core::ffi::c_int {
    return (window_copy_line_number_mode(wme) != WINDOW_COPY_LINE_NUMBERS_OFF as ::core::ffi::c_int)
        as ::core::ffi::c_int;
}
unsafe extern "C" fn window_copy_cursor_line_active(
    mut wme: *mut window_mode_entry,
) -> ::core::ffi::c_int {
    let mut oo: *mut options = (*(*(*wme).wp).window).options;
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    s = options_get_string(
        oo,
        b"copy-mode-current-line-style\0" as *const u8 as *const ::core::ffi::c_char,
    );
    return (strcmp(s, b"default\0" as *const u8 as *const ::core::ffi::c_char)
        != 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
unsafe extern "C" fn window_copy_line_number_width(mut wme: *mut window_mode_entry) -> u_int {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut lines: u_int = 0;
    let mut digits: u_int = 0;
    if window_copy_line_numbers_active(wme) == 0 {
        return 0 as u_int;
    }
    lines = (*(*(*data).backing).grid)
        .hsize
        .wrapping_add((*(*(*data).backing).grid).sy)
        .wrapping_add(1 as u_int);
    digits = 1 as u_int;
    while lines >= 10 as u_int {
        lines = lines.wrapping_div(10 as u_int);
        digits = digits.wrapping_add(1);
    }
    if digits < 3 as u_int {
        digits = 3 as u_int;
    }
    return digits.wrapping_add(1 as u_int);
}
unsafe extern "C" fn window_copy_cursor_offset(
    mut wme: *mut window_mode_entry,
    mut cx: u_int,
    mut sx: u_int,
) -> u_int {
    let mut width: u_int = window_copy_line_number_width(wme);
    let mut content: u_int = 0;
    if width == 0 as u_int {
        return cx;
    }
    if width >= sx {
        content = 1 as u_int;
    } else {
        content = sx.wrapping_sub(width);
    }
    if cx >= content {
        return sx.wrapping_sub(1 as u_int);
    }
    return width.wrapping_add(cx);
}
unsafe extern "C" fn window_copy_cursor_unoffset(
    mut wme: *mut window_mode_entry,
    mut vx: u_int,
    mut sx: u_int,
) -> u_int {
    let mut width: u_int = window_copy_line_number_width(wme);
    let mut content: u_int = 0;
    if width == 0 as u_int {
        return vx;
    }
    if width >= sx {
        content = 1 as u_int;
    } else {
        content = sx.wrapping_sub(width);
    }
    if vx < width {
        return 0 as u_int;
    }
    vx = vx.wrapping_sub(width);
    if vx >= content {
        return content.wrapping_sub(1 as u_int);
    }
    return vx;
}
#[no_mangle]
pub unsafe extern "C" fn window_copy_set_line_numbers(
    mut wp: *mut window_pane,
    mut enabled: ::core::ffi::c_int,
) {
    let mut wme: *mut window_mode_entry = (*wp).modes.tqh_first;
    if wme.is_null() || (*wme).mode != &raw const window_copy_mode {
        return;
    }
    window_copy_set_line_numbers1(wme, enabled, 0 as ::core::ffi::c_int);
}
unsafe extern "C" fn window_copy_set_line_numbers1(
    mut wme: *mut window_mode_entry,
    mut enabled: ::core::ffi::c_int,
    mut force: ::core::ffi::c_int,
) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut oo: *mut options = (*(*(*wme).wp).window).options;
    let mut active: ::core::ffi::c_int = 0;
    let mut line_numbers: ::core::ffi::c_int = 0;
    if data.is_null() {
        return;
    }
    active = window_copy_line_numbers_active(wme);
    if enabled == 0 {
        line_numbers = 0 as ::core::ffi::c_int;
    } else if force != 0
        && options_get_number(
            oo,
            b"copy-mode-line-numbers\0" as *const u8 as *const ::core::ffi::c_char,
        ) == WINDOW_COPY_LINE_NUMBERS_OFF as ::core::ffi::c_int as ::core::ffi::c_longlong
    {
        line_numbers = 2 as ::core::ffi::c_int;
    } else {
        line_numbers = 1 as ::core::ffi::c_int;
    }
    if (*data).line_numbers == line_numbers && active == enabled {
        return;
    }
    (*data).line_numbers = line_numbers;
    window_copy_redraw_screen(wme);
}
#[no_mangle]
pub unsafe extern "C" fn window_copy_get_current_offset(
    mut wp: *mut window_pane,
    mut offset: *mut u_int,
    mut size: *mut u_int,
) -> ::core::ffi::c_int {
    let mut wme: *mut window_mode_entry = (*wp).modes.tqh_first;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut hsize: u_int = 0;
    if data.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    hsize = (*(*(*data).backing).grid).hsize;
    *offset = hsize.wrapping_sub((*data).oy);
    *size = hsize;
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn window_copy_write_line(
    mut wme: *mut window_mode_entry,
    mut ctx: *mut screen_write_ctx,
    mut py: u_int,
) {
    let mut wp: *mut window_pane = (*wme).wp;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut s: *mut screen = &raw mut (*data).screen;
    let mut oo: *mut options = (*(*wp).window).options;
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
    let mut mgc: grid_cell = grid_cell {
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
    let mut cgc: grid_cell = grid_cell {
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
    let mut mkgc: grid_cell = grid_cell {
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
    let mut clgc: grid_cell = grid_cell {
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
    let mut ln_gc: grid_cell = grid_cell {
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
    let mut cur_ln_gc: grid_cell = grid_cell {
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
    let mut sx: u_int = (*(*s).grid).sx;
    let mut hsize: u_int = (*(*(*data).backing).grid).hsize;
    let mut width: u_int = 0;
    let mut absolute: u_int = 0;
    let mut line_number: u_int = 0;
    let mut content_sx: u_int = 0;
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut current: ::core::ffi::c_int = 0;
    let mut mode: ::core::ffi::c_int = 0;
    width = window_copy_line_number_width(wme);
    if width >= sx {
        content_sx = 1 as u_int;
    } else if width != 0 as u_int {
        content_sx = sx.wrapping_sub(width);
    } else {
        content_sx = sx;
    }
    screen_write_cursormove(
        ctx,
        0 as ::core::ffi::c_int,
        py as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    ft = format_create_defaults(
        ::core::ptr::null_mut::<cmdq_item>(),
        ::core::ptr::null_mut::<client>(),
        ::core::ptr::null_mut::<session>(),
        ::core::ptr::null_mut::<winlink>(),
        wp,
    );
    style_apply(
        &raw mut gc,
        oo,
        b"copy-mode-position-style\0" as *const u8 as *const ::core::ffi::c_char,
        ft,
    );
    gc.flags = (gc.flags as ::core::ffi::c_int | GRID_FLAG_NOPALETTE) as u_char;
    style_apply(
        &raw mut mgc,
        oo,
        b"copy-mode-match-style\0" as *const u8 as *const ::core::ffi::c_char,
        ft,
    );
    mgc.flags = (mgc.flags as ::core::ffi::c_int | GRID_FLAG_NOPALETTE) as u_char;
    style_apply(
        &raw mut cgc,
        oo,
        b"copy-mode-current-match-style\0" as *const u8 as *const ::core::ffi::c_char,
        ft,
    );
    cgc.flags = (cgc.flags as ::core::ffi::c_int | GRID_FLAG_NOPALETTE) as u_char;
    style_apply(
        &raw mut mkgc,
        oo,
        b"copy-mode-mark-style\0" as *const u8 as *const ::core::ffi::c_char,
        ft,
    );
    mkgc.flags = (mkgc.flags as ::core::ffi::c_int | GRID_FLAG_NOPALETTE) as u_char;
    style_apply(
        &raw mut clgc,
        oo,
        b"copy-mode-current-line-style\0" as *const u8 as *const ::core::ffi::c_char,
        ft,
    );
    clgc.flags = (clgc.flags as ::core::ffi::c_int | GRID_FLAG_NOPALETTE) as u_char;
    if width != 0 as u_int {
        style_apply(
            &raw mut ln_gc,
            oo,
            b"copy-mode-line-number-style\0" as *const u8 as *const ::core::ffi::c_char,
            ft,
        );
        ln_gc.flags = (ln_gc.flags as ::core::ffi::c_int | GRID_FLAG_NOPALETTE) as u_char;
        style_apply(
            &raw mut cur_ln_gc,
            oo,
            b"copy-mode-current-line-number-style\0" as *const u8 as *const ::core::ffi::c_char,
            ft,
        );
        cur_ln_gc.flags = (cur_ln_gc.flags as ::core::ffi::c_int | GRID_FLAG_NOPALETTE) as u_char;
        current = (py == (*data).cy) as ::core::ffi::c_int;
        absolute = hsize
            .wrapping_sub((*data).oy)
            .wrapping_add(py)
            .wrapping_add(1 as u_int);
        mode = window_copy_line_number_mode(wme);
        if mode == WINDOW_COPY_LINE_NUMBERS_DEFAULT as ::core::ffi::c_int {
            if py < (*data).oy {
                line_number = (*data).oy.wrapping_sub(py);
            } else {
                line_number = py.wrapping_sub((*data).oy);
            }
        } else if mode == WINDOW_COPY_LINE_NUMBERS_ABSOLUTE as ::core::ffi::c_int {
            line_number = absolute;
        } else if mode == WINDOW_COPY_LINE_NUMBERS_HYBRID as ::core::ffi::c_int && current != 0 {
            line_number = absolute;
        } else if py > (*data).cy {
            line_number = py.wrapping_sub((*data).cy);
        } else {
            line_number = (*data).cy.wrapping_sub(py);
        }
        screen_write_cursormove(
            ctx,
            0 as ::core::ffi::c_int,
            py as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_nputs(
            ctx,
            width as ssize_t,
            if current != 0 {
                &raw mut cur_ln_gc
            } else {
                &raw mut ln_gc
            },
            b"%*u \0" as *const u8 as *const ::core::ffi::c_char,
            width as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
            line_number,
        );
    }
    window_copy_write_one(
        wme,
        ctx,
        width,
        py,
        hsize.wrapping_sub((*data).oy).wrapping_add(py),
        content_sx,
        &raw mut mgc,
        &raw mut cgc,
        &raw mut mkgc,
        &raw mut clgc,
    );
    if py == 0 as u_int && (*s).rupper < (*s).rlower && (*data).hide_position == 0 {
        value = options_get_string(
            oo,
            b"copy-mode-position-format\0" as *const u8 as *const ::core::ffi::c_char,
        );
        if *value as ::core::ffi::c_int != '\0' as i32 {
            expanded = format_expand(ft, value);
            if *expanded as ::core::ffi::c_int != '\0' as i32 {
                screen_write_cursormove(
                    ctx,
                    width as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
                format_draw(
                    ctx,
                    &raw mut gc,
                    content_sx,
                    expanded,
                    ::core::ptr::null_mut::<style_ranges>(),
                    0 as ::core::ffi::c_int,
                );
            }
            free(expanded as *mut ::core::ffi::c_void);
        }
    }
    if py == (*data).cy && (*data).cx >= content_sx {
        screen_write_cursormove(
            ctx,
            window_copy_cursor_offset(wme, (*data).cx, (*(*s).grid).sx) as ::core::ffi::c_int,
            py as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_putc(ctx, &raw const grid_default_cell, '$' as i32 as u_char);
    }
    format_free(ft);
}
unsafe extern "C" fn window_copy_write_lines(
    mut wme: *mut window_mode_entry,
    mut ctx: *mut screen_write_ctx,
    mut py: u_int,
    mut ny: u_int,
) {
    let mut yy: u_int = 0;
    yy = py;
    while yy < py.wrapping_add(ny) {
        window_copy_write_line(wme, ctx, yy);
        yy = yy.wrapping_add(1);
    }
}
unsafe extern "C" fn window_copy_redraw_selection(
    mut wme: *mut window_mode_entry,
    mut old_y: u_int,
) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut gd: *mut grid = (*(*data).backing).grid;
    let mut new_y: u_int = 0;
    let mut start: u_int = 0;
    let mut end: u_int = 0;
    new_y = (*data).cy;
    if old_y <= new_y {
        start = old_y;
        end = new_y;
    } else {
        start = new_y;
        end = old_y;
    }
    if (*data).selflag as ::core::ffi::c_uint
        == SEL_WORD as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if end < (*gd).sy.wrapping_add((*data).oy).wrapping_sub(1 as u_int) {
            end = end.wrapping_add(1);
        }
    }
    window_copy_redraw_lines(wme, start, end.wrapping_sub(start).wrapping_add(1 as u_int));
}
unsafe extern "C" fn window_copy_redraw_lines(
    mut wme: *mut window_mode_entry,
    mut py: u_int,
    mut ny: u_int,
) {
    let mut wp: *mut window_pane = (*wme).wp;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut s: *mut screen = &raw mut (*data).screen;
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
    let mut i: u_int = 0;
    if window_copy_line_number_width(wme) != 0 as u_int {
        screen_write_start(&raw mut ctx, &raw mut (*data).screen);
        i = py;
        while i < py.wrapping_add(ny) {
            window_copy_write_line(wme, &raw mut ctx, i);
            i = i.wrapping_add(1);
        }
        screen_write_cursormove(
            &raw mut ctx,
            window_copy_cursor_offset(wme, (*data).cx, (*(*s).grid).sx) as ::core::ffi::c_int,
            (*data).cy as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_stop(&raw mut ctx);
        (*wp).flags |= PANE_REDRAW | PANE_REDRAWSCROLLBAR;
        return;
    }
    if window_pane_scrollbar_overlay_visible(wp) != 0 {
        screen_write_start(&raw mut ctx, &raw mut (*data).screen);
    } else {
        screen_write_start_pane(&raw mut ctx, wp, ::core::ptr::null_mut::<screen>());
    }
    i = py;
    while i < py.wrapping_add(ny) {
        window_copy_write_line(wme, &raw mut ctx, i);
        i = i.wrapping_add(1);
    }
    screen_write_cursormove(
        &raw mut ctx,
        window_copy_cursor_offset(wme, (*data).cx, (*(*s).grid).sx) as ::core::ffi::c_int,
        (*data).cy as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    screen_write_stop(&raw mut ctx);
    window_pane_scrollbar_redraw(wp);
}
unsafe extern "C" fn window_copy_redraw_screen(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    window_copy_redraw_lines(wme, 0 as u_int, (*(*data).screen.grid).sy);
}
unsafe extern "C" fn window_copy_style_changed(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    if !(*data).screen.sel.is_null() {
        window_copy_set_selection(wme, 0 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
    }
    window_copy_redraw_screen(wme);
}
unsafe extern "C" fn window_copy_synchronize_cursor_end(
    mut wme: *mut window_mode_entry,
    mut begin: ::core::ffi::c_int,
    mut no_reset: ::core::ffi::c_int,
) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut xx: u_int = 0;
    let mut yy: u_int = 0;
    xx = (*data).cx;
    yy = (*(*(*data).backing).grid)
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    match (*data).selflag as ::core::ffi::c_uint {
        1 => {
            if !(no_reset != 0) {
                begin = 0 as ::core::ffi::c_int;
                if (*data).dy > yy || (*data).dy == yy && (*data).dx > xx {
                    window_copy_cursor_previous_word_pos(
                        wme,
                        (*data).separators,
                        &raw mut xx,
                        &raw mut yy,
                    );
                    begin = 1 as ::core::ffi::c_int;
                    (*data).endselx = (*data).endselrx;
                    (*data).endsely = (*data).endselry;
                } else {
                    if xx >= window_copy_find_length(wme, yy)
                        || window_copy_in_set(
                            wme,
                            xx.wrapping_add(1 as u_int),
                            yy,
                            WHITESPACE.as_ptr(),
                        ) == 0
                    {
                        window_copy_cursor_next_word_end_pos(
                            wme,
                            (*data).separators,
                            &raw mut xx,
                            &raw mut yy,
                        );
                    }
                    (*data).selx = (*data).selrx;
                    (*data).sely = (*data).selry;
                }
            }
        }
        2 => {
            if !(no_reset != 0) {
                begin = 0 as ::core::ffi::c_int;
                if (*data).dy > yy {
                    xx = 0 as u_int;
                    begin = 1 as ::core::ffi::c_int;
                    (*data).endselx = (*data).endselrx;
                    (*data).endsely = (*data).endselry;
                } else {
                    if yy < (*data).endselry {
                        yy = (*data).endselry;
                    }
                    xx = window_copy_find_length(wme, yy);
                    (*data).selx = (*data).selrx;
                    (*data).sely = (*data).selry;
                }
            }
        }
        0 | _ => {}
    }
    if begin != 0 {
        (*data).selx = xx;
        (*data).sely = yy;
    } else {
        (*data).endselx = xx;
        (*data).endsely = yy;
    };
}
unsafe extern "C" fn window_copy_synchronize_cursor(
    mut wme: *mut window_mode_entry,
    mut no_reset: ::core::ffi::c_int,
) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    match (*data).cursordrag as ::core::ffi::c_uint {
        1 => {
            window_copy_synchronize_cursor_end(wme, 0 as ::core::ffi::c_int, no_reset);
        }
        2 => {
            window_copy_synchronize_cursor_end(wme, 1 as ::core::ffi::c_int, no_reset);
        }
        0 | _ => {}
    };
}
unsafe extern "C" fn window_copy_update_cursor(
    mut wme: *mut window_mode_entry,
    mut cx: u_int,
    mut cy: u_int,
) {
    let mut wp: *mut window_pane = (*wme).wp;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut s: *mut screen = &raw mut (*data).screen;
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
    let mut old_cx: u_int = 0;
    let mut old_cy: u_int = 0;
    let mut py: u_int = 0;
    let mut width: u_int = 0;
    let mut content_sx: u_int = 0;
    let mut maxx: u_int = 0;
    let mut allow_onemore: ::core::ffi::c_int = 0;
    if (*data).rectflag == 0 && cy < (*(*s).grid).sy {
        allow_onemore =
            (!(*data).screen.sel.is_null() && (*data).rectflag != 0) as ::core::ffi::c_int;
        py = (*(*(*data).backing).grid)
            .hsize
            .wrapping_add(cy)
            .wrapping_sub((*data).oy);
        maxx = window_copy_cursor_limit(wme, py, allow_onemore);
        if cx > maxx {
            cx = maxx;
        }
    }
    old_cx = (*data).cx;
    old_cy = (*data).cy;
    (*data).cx = cx;
    (*data).cy = cy;
    if window_copy_line_numbers_active(wme) != 0 {
        width = window_copy_line_number_width(wme);
        if !(*s).sel.is_null()
            || (*data).lineflag as ::core::ffi::c_uint
                != LINE_SEL_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
            || old_cy != (*data).cy
        {
            window_copy_redraw_screen(wme);
            return;
        }
        if width >= (*(*s).grid).sx {
            content_sx = 1 as u_int;
        } else {
            content_sx = (*(*s).grid).sx.wrapping_sub(width);
        }
        if old_cx >= content_sx || (*data).cx >= content_sx {
            window_copy_redraw_screen(wme);
            return;
        }
        screen_write_start_pane(&raw mut ctx, wp, ::core::ptr::null_mut::<screen>());
        screen_write_cursormove(
            &raw mut ctx,
            window_copy_cursor_offset(wme, (*data).cx, (*(*s).grid).sx) as ::core::ffi::c_int,
            (*data).cy as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_stop(&raw mut ctx);
        return;
    }
    if old_cy != (*data).cy && window_copy_cursor_line_active(wme) != 0 {
        window_copy_redraw_lines(wme, old_cy, 1 as u_int);
        window_copy_redraw_lines(wme, (*data).cy, 1 as u_int);
        return;
    }
    if old_cx == (*(*s).grid).sx {
        window_copy_redraw_lines(wme, old_cy, 1 as u_int);
    }
    if (*data).cx == (*(*s).grid).sx {
        window_copy_redraw_lines(wme, (*data).cy, 1 as u_int);
    } else {
        screen_write_start_pane(&raw mut ctx, wp, ::core::ptr::null_mut::<screen>());
        screen_write_cursormove(
            &raw mut ctx,
            window_copy_cursor_offset(wme, (*data).cx, (*(*s).grid).sx) as ::core::ffi::c_int,
            (*data).cy as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_stop(&raw mut ctx);
    };
}
unsafe extern "C" fn window_copy_start_selection(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    (*data).selx = (*data).cx;
    (*data).sely = (*(*(*data).backing).grid)
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    (*data).endselx = (*data).selx;
    (*data).endsely = (*data).sely;
    (*data).cursordrag = CURSORDRAG_ENDSEL;
    window_copy_set_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
}
unsafe extern "C" fn window_copy_mouse_in_selection(
    mut wme: *mut window_mode_entry,
    mut x: u_int,
    mut y: u_int,
    mut on_start: *mut ::core::ffi::c_int,
    mut on_end: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut sx: u_int = (*(*data).screen.grid).sx;
    let mut sy: u_int = (*(*data).screen.grid).sy;
    let mut hsize: u_int = 0;
    let mut screeny: u_int = 0;
    let mut mx: u_int = 0;
    let mut my: u_int = 0;
    let mut selx: u_int = 0;
    let mut sely: u_int = 0;
    let mut endselx: u_int = 0;
    let mut endsely: u_int = 0;
    let mut cursorx: u_int = 0;
    let mut cursory: u_int = 0;
    let mut mpos: ::core::ffi::c_longlong = 0;
    let mut spos: ::core::ffi::c_longlong = 0;
    let mut epos: ::core::ffi::c_longlong = 0;
    let mut dstart: ::core::ffi::c_longlong = 0;
    let mut dend: ::core::ffi::c_longlong = 0;
    if !on_start.is_null() {
        *on_start = 0 as ::core::ffi::c_int;
    }
    if !on_end.is_null() {
        *on_end = 0 as ::core::ffi::c_int;
    }
    if (*data).screen.sel.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    hsize = (*(*(*data).backing).grid).hsize;
    screeny = hsize.wrapping_sub((*data).oy);
    selx = window_copy_cursor_offset(wme, (*data).selx, sx);
    sely = (*data).sely.wrapping_sub(screeny);
    if (*data).sely >= screeny && sely < sy && x == selx && y == sely {
        if !on_start.is_null() {
            *on_start = 1 as ::core::ffi::c_int;
        }
        return 1 as ::core::ffi::c_int;
    }
    endselx = window_copy_cursor_offset(wme, (*data).endselx, sx);
    endsely = (*data).endsely.wrapping_sub(screeny);
    if (*data).endsely >= screeny && endsely < sy && x == endselx && y == endsely {
        if !on_end.is_null() {
            *on_end = 1 as ::core::ffi::c_int;
        }
        return 1 as ::core::ffi::c_int;
    }
    cursorx = window_copy_cursor_offset(wme, (*data).cx, sx);
    cursory = (*data).cy;
    if x != cursorx || y != cursory {
        if screen_check_selection(&raw mut (*data).screen, x, y) == 0 {
            return 0 as ::core::ffi::c_int;
        }
    }
    if !on_start.is_null() || !on_end.is_null() {
        mx = window_copy_cursor_unoffset(wme, x, sx);
        my = screeny.wrapping_add(y);
        mpos = my as ::core::ffi::c_longlong
            * sx.wrapping_add(1 as u_int) as ::core::ffi::c_longlong
            + mx as ::core::ffi::c_longlong;
        spos = (*data).sely as ::core::ffi::c_longlong
            * sx.wrapping_add(1 as u_int) as ::core::ffi::c_longlong
            + (*data).selx as ::core::ffi::c_longlong;
        epos = (*data).endsely as ::core::ffi::c_longlong
            * sx.wrapping_add(1 as u_int) as ::core::ffi::c_longlong
            + (*data).endselx as ::core::ffi::c_longlong;
        dstart = llabs(mpos - spos);
        dend = llabs(mpos - epos);
        if dstart <= dend {
            if !on_start.is_null() {
                *on_start = 1 as ::core::ffi::c_int;
            }
        } else if !on_end.is_null() {
            *on_end = 1 as ::core::ffi::c_int;
        }
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn window_copy_adjust_selection(
    mut wme: *mut window_mode_entry,
    mut selx: *mut u_int,
    mut sely: *mut u_int,
) -> ::core::ffi::c_int {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut s: *mut screen = &raw mut (*data).screen;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut ty: u_int = 0;
    let mut relpos: ::core::ffi::c_int = 0;
    sx = *selx;
    sy = *sely;
    ty = (*(*(*data).backing).grid).hsize.wrapping_sub((*data).oy);
    if sy < ty {
        relpos = WINDOW_COPY_REL_POS_ABOVE as ::core::ffi::c_int;
        if (*data).rectflag == 0 {
            sx = 0 as u_int;
        }
        sy = 0 as u_int;
    } else if sy > ty.wrapping_add((*(*s).grid).sy).wrapping_sub(1 as u_int) {
        relpos = WINDOW_COPY_REL_POS_BELOW as ::core::ffi::c_int;
        if (*data).rectflag == 0 {
            sx = (*(*s).grid).sx.wrapping_sub(1 as u_int);
        }
        sy = (*(*s).grid).sy.wrapping_sub(1 as u_int);
    } else {
        relpos = WINDOW_COPY_REL_POS_ON_SCREEN as ::core::ffi::c_int;
        sy = sy.wrapping_sub(ty);
    }
    *selx = sx;
    *sely = sy;
    return relpos;
}
unsafe extern "C" fn window_copy_update_selection(
    mut wme: *mut window_mode_entry,
    mut may_redraw: ::core::ffi::c_int,
    mut no_reset: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut s: *mut screen = &raw mut (*data).screen;
    if (*s).sel.is_null()
        && (*data).lineflag as ::core::ffi::c_uint
            == LINE_SEL_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return 0 as ::core::ffi::c_int;
    }
    return window_copy_set_selection(wme, may_redraw, no_reset);
}
unsafe extern "C" fn window_copy_update_selection_view(
    mut wme: *mut window_mode_entry,
    mut may_redraw: ::core::ffi::c_int,
    mut no_reset: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut selx: u_int = 0;
    let mut sely: u_int = 0;
    let mut endselx: u_int = 0;
    let mut endsely: u_int = 0;
    let mut changed: ::core::ffi::c_int = 0;
    if (*data).cursordrag as ::core::ffi::c_uint
        != CURSORDRAG_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return window_copy_update_selection(wme, may_redraw, no_reset);
    }
    selx = (*data).selx;
    sely = (*data).sely;
    endselx = (*data).endselx;
    endsely = (*data).endsely;
    changed = window_copy_update_selection(wme, may_redraw, 1 as ::core::ffi::c_int);
    (*data).selx = selx;
    (*data).sely = sely;
    (*data).endselx = endselx;
    (*data).endsely = endsely;
    return changed;
}
unsafe extern "C" fn window_copy_set_selection(
    mut wme: *mut window_mode_entry,
    mut may_redraw: ::core::ffi::c_int,
    mut no_reset: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut wp: *mut window_pane = (*wme).wp;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut s: *mut screen = &raw mut (*data).screen;
    let mut oo: *mut options = (*(*wp).window).options;
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
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut cy: u_int = 0;
    let mut endsx: u_int = 0;
    let mut endsy: u_int = 0;
    let mut clipx: u_int = 0;
    let mut startrelpos: ::core::ffi::c_int = 0;
    let mut endrelpos: ::core::ffi::c_int = 0;
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    window_copy_synchronize_cursor(wme, no_reset);
    sx = (*data).selx;
    sy = (*data).sely;
    startrelpos = window_copy_adjust_selection(wme, &raw mut sx, &raw mut sy);
    endsx = (*data).endselx;
    endsy = (*data).endsely;
    endrelpos = window_copy_adjust_selection(wme, &raw mut endsx, &raw mut endsy);
    if startrelpos == endrelpos
        && startrelpos != WINDOW_COPY_REL_POS_ON_SCREEN as ::core::ffi::c_int
    {
        screen_hide_selection(s);
        return 0 as ::core::ffi::c_int;
    }
    ft = format_create_defaults(
        ::core::ptr::null_mut::<cmdq_item>(),
        ::core::ptr::null_mut::<client>(),
        ::core::ptr::null_mut::<session>(),
        ::core::ptr::null_mut::<winlink>(),
        wp,
    );
    style_apply(
        &raw mut gc,
        oo,
        b"copy-mode-selection-style\0" as *const u8 as *const ::core::ffi::c_char,
        ft,
    );
    gc.flags = (gc.flags as ::core::ffi::c_int | GRID_FLAG_NOPALETTE) as u_char;
    format_free(ft);
    clipx = window_copy_line_number_width(wme);
    if clipx >= (*(*s).grid).sx {
        clipx = (*(*s).grid).sx.wrapping_sub(1 as u_int);
    }
    if window_copy_line_numbers_active(wme) != 0 {
        sx = window_copy_cursor_offset(wme, sx, (*(*s).grid).sx);
        endsx = window_copy_cursor_offset(wme, endsx, (*(*s).grid).sx);
    }
    screen_set_selection(
        s,
        sx,
        sy,
        endsx,
        endsy,
        (*data).rectflag as u_int,
        clipx,
        (*data).modekeys,
        &raw mut gc,
    );
    if (*data).rectflag != 0 && may_redraw != 0 {
        cy = (*data).cy;
        if (*data).cursordrag as ::core::ffi::c_uint
            == CURSORDRAG_ENDSEL as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if sy < cy {
                window_copy_redraw_lines(wme, sy, cy.wrapping_sub(sy).wrapping_add(1 as u_int));
            } else {
                window_copy_redraw_lines(wme, cy, sy.wrapping_sub(cy).wrapping_add(1 as u_int));
            }
        } else if endsy < cy {
            window_copy_redraw_lines(wme, endsy, cy.wrapping_sub(endsy).wrapping_add(1 as u_int));
        } else {
            window_copy_redraw_lines(wme, cy, endsy.wrapping_sub(cy).wrapping_add(1 as u_int));
        }
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn window_copy_get_selection(
    mut wme: *mut window_mode_entry,
    mut len: *mut size_t,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*wme).wp;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut s: *mut screen = &raw mut (*data).screen;
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut off: size_t = 0;
    let mut i: u_int = 0;
    let mut xx: u_int = 0;
    let mut yy: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut ex: u_int = 0;
    let mut ey: u_int = 0;
    let mut ey_last: u_int = 0;
    let mut firstsx: u_int = 0;
    let mut lastex: u_int = 0;
    let mut restex: u_int = 0;
    let mut restsx: u_int = 0;
    let mut selx: u_int = 0;
    let mut keys: ::core::ffi::c_int = 0;
    if (*data).screen.sel.is_null()
        && (*data).lineflag as ::core::ffi::c_uint
            == LINE_SEL_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        buf = window_copy_match_at_cursor(data);
        if !buf.is_null() {
            *len = strlen(buf);
        } else {
            *len = 0 as size_t;
        }
        return buf as *mut ::core::ffi::c_void;
    }
    buf = xmalloc(1 as size_t) as *mut ::core::ffi::c_char;
    off = 0 as size_t;
    *buf = '\0' as i32 as ::core::ffi::c_char;
    xx = (*data).endselx;
    yy = (*data).endsely;
    if yy < (*data).sely || yy == (*data).sely && xx < (*data).selx {
        sx = xx;
        sy = yy;
        ex = (*data).selx;
        ey = (*data).sely;
    } else {
        sx = (*data).selx;
        sy = (*data).sely;
        ex = xx;
        ey = yy;
    }
    ey_last = window_copy_find_length(wme, ey);
    if ex > ey_last {
        ex = ey_last;
    }
    xx = (*(*s).grid).sx;
    keys = options_get_number(
        (*(*wp).window).options,
        b"mode-keys\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    if (*data).rectflag != 0 {
        if (*data).cursordrag as ::core::ffi::c_uint
            == CURSORDRAG_ENDSEL as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            selx = (*data).selx;
        } else {
            selx = (*data).endselx;
        }
        if selx < (*data).cx {
            if keys == MODEKEY_EMACS {
                lastex = (*data).cx;
                restex = (*data).cx;
            } else {
                lastex = (*data).cx.wrapping_add(1 as u_int);
                restex = (*data).cx.wrapping_add(1 as u_int);
            }
            firstsx = selx;
            restsx = selx;
        } else {
            lastex = selx.wrapping_add(1 as u_int);
            restex = selx.wrapping_add(1 as u_int);
            firstsx = (*data).cx;
            restsx = (*data).cx;
        }
    } else {
        if keys == MODEKEY_EMACS {
            lastex = ex;
        } else {
            lastex = ex.wrapping_add(1 as u_int);
        }
        restex = xx;
        firstsx = sx;
        restsx = 0 as u_int;
    }
    i = sy;
    while i <= ey {
        window_copy_copy_line(
            wme,
            &raw mut buf,
            &raw mut off,
            i,
            if i == sy { firstsx } else { restsx },
            if i == ey { lastex } else { restex },
        );
        i = i.wrapping_add(1);
    }
    if off == 0 as size_t {
        free(buf as *mut ::core::ffi::c_void);
        *len = 0 as size_t;
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if keys == MODEKEY_EMACS || lastex <= ey_last {
        if !((*grid_get_line((*(*data).backing).grid, ey)).flags as ::core::ffi::c_int)
            & GRID_LINE_WRAPPED
            != 0
            || lastex != ey_last
        {
            off = off.wrapping_sub(1 as size_t);
        }
    }
    *len = off;
    return buf as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn window_copy_copy_buffer(
    mut wme: *mut window_mode_entry,
    mut prefix: *const ::core::ffi::c_char,
    mut buf: *mut ::core::ffi::c_void,
    mut len: size_t,
    mut set_paste: ::core::ffi::c_int,
    mut set_clip: ::core::ffi::c_int,
) {
    let mut wp: *mut window_pane = (*wme).wp;
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
    let mut redraw: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if set_clip != 0
        && options_get_number(
            global_options,
            b"set-clipboard\0" as *const u8 as *const ::core::ffi::c_char,
        ) != 0 as ::core::ffi::c_longlong
    {
        if window_copy_line_numbers_active(wme) != 0 && (*wp).flags & PANE_REDRAW != 0 {
            redraw = PANE_REDRAW;
            (*wp).flags &= !PANE_REDRAW;
        }
        screen_write_start_pane(&raw mut ctx, wp, ::core::ptr::null_mut::<screen>());
        screen_write_setselection(
            &raw mut ctx,
            b"\0" as *const u8 as *const ::core::ffi::c_char,
            buf as *mut u_char,
            len as u_int,
        );
        screen_write_stop(&raw mut ctx);
        (*wp).flags |= redraw;
        events_fire_pane(
            b"pane-set-clipboard\0" as *const u8 as *const ::core::ffi::c_char,
            wp,
        );
    }
    if set_paste != 0 {
        paste_add(prefix, buf as *mut ::core::ffi::c_char, len);
    } else {
        free(buf);
    };
}
unsafe extern "C" fn window_copy_pipe_run(
    mut wme: *mut window_mode_entry,
    mut s: *mut session,
    mut cmd: *const ::core::ffi::c_char,
    mut len: *mut size_t,
) -> *mut ::core::ffi::c_void {
    let mut buf: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut job: *mut job = ::core::ptr::null_mut::<job>();
    buf = window_copy_get_selection(wme, len);
    if cmd.is_null() || *cmd as ::core::ffi::c_int == '\0' as i32 {
        cmd = options_get_string(
            global_options,
            b"copy-command\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if !cmd.is_null() && *cmd as ::core::ffi::c_int != '\0' as i32 {
        job = job_run(
            cmd,
            0 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<environ>(),
            s,
            ::core::ptr::null::<::core::ffi::c_char>(),
            None,
            None,
            None,
            NULL,
            JOB_NOWAIT,
            -(1 as ::core::ffi::c_int),
            -(1 as ::core::ffi::c_int),
        );
        if !job.is_null() {
            bufferevent_write(job_get_event(job), buf, *len);
        }
    }
    return buf;
}
unsafe extern "C" fn window_copy_pipe(
    mut wme: *mut window_mode_entry,
    mut s: *mut session,
    mut cmd: *const ::core::ffi::c_char,
) {
    let mut buf: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut len: size_t = 0;
    buf = window_copy_pipe_run(wme, s, cmd, &raw mut len);
    free(buf);
}
unsafe extern "C" fn window_copy_copy_pipe(
    mut wme: *mut window_mode_entry,
    mut s: *mut session,
    mut prefix: *const ::core::ffi::c_char,
    mut cmd: *const ::core::ffi::c_char,
    mut set_paste: ::core::ffi::c_int,
    mut set_clip: ::core::ffi::c_int,
) {
    let mut buf: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut len: size_t = 0;
    buf = window_copy_pipe_run(wme, s, cmd, &raw mut len);
    if !buf.is_null() {
        window_copy_copy_buffer(wme, prefix, buf, len, set_paste, set_clip);
    }
}
unsafe extern "C" fn window_copy_copy_selection(
    mut wme: *mut window_mode_entry,
    mut prefix: *const ::core::ffi::c_char,
    mut set_paste: ::core::ffi::c_int,
    mut set_clip: ::core::ffi::c_int,
) {
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: size_t = 0;
    buf = window_copy_get_selection(wme, &raw mut len) as *mut ::core::ffi::c_char;
    if !buf.is_null() {
        window_copy_copy_buffer(
            wme,
            prefix,
            buf as *mut ::core::ffi::c_void,
            len,
            set_paste,
            set_clip,
        );
    }
}
unsafe extern "C" fn window_copy_append_selection(mut wme: *mut window_mode_entry) {
    let mut wp: *mut window_pane = (*wme).wp;
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut bufname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut bufdata: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut len: size_t = 0;
    let mut bufsize: size_t = 0;
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
    buf = window_copy_get_selection(wme, &raw mut len) as *mut ::core::ffi::c_char;
    if buf.is_null() {
        return;
    }
    if options_get_number(
        global_options,
        b"set-clipboard\0" as *const u8 as *const ::core::ffi::c_char,
    ) != 0 as ::core::ffi::c_longlong
    {
        screen_write_start_pane(&raw mut ctx, wp, ::core::ptr::null_mut::<screen>());
        screen_write_setselection(
            &raw mut ctx,
            b"\0" as *const u8 as *const ::core::ffi::c_char,
            buf as *mut u_char,
            len as u_int,
        );
        screen_write_stop(&raw mut ctx);
        events_fire_pane(
            b"pane-set-clipboard\0" as *const u8 as *const ::core::ffi::c_char,
            wp,
        );
    }
    pb = paste_get_top(&raw mut bufname);
    if !pb.is_null() {
        bufdata = paste_buffer_data(pb, &raw mut bufsize);
        buf = xrealloc(buf as *mut ::core::ffi::c_void, len.wrapping_add(bufsize))
            as *mut ::core::ffi::c_char;
        memmove(
            buf.offset(bufsize as isize) as *mut ::core::ffi::c_void,
            buf as *const ::core::ffi::c_void,
            len,
        );
        memcpy(
            buf as *mut ::core::ffi::c_void,
            bufdata as *const ::core::ffi::c_void,
            bufsize,
        );
        len = len.wrapping_add(bufsize);
    }
    if paste_set(
        buf,
        len,
        bufname,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
    ) != 0 as ::core::ffi::c_int
    {
        free(buf as *mut ::core::ffi::c_void);
    }
    free(bufname as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn window_copy_copy_line(
    mut wme: *mut window_mode_entry,
    mut buf: *mut *mut ::core::ffi::c_char,
    mut off: *mut size_t,
    mut sy: u_int,
    mut sx: u_int,
    mut ex: u_int,
) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut gd: *mut grid = (*(*data).backing).grid;
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
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut ud: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    let mut i: u_int = 0;
    let mut xx: u_int = 0;
    let mut wrapped: u_int = 0 as u_int;
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if sx > ex {
        return;
    }
    gl = grid_get_line(gd, sy);
    if (*gl).flags as ::core::ffi::c_int & GRID_LINE_WRAPPED != 0
        && (*gl).cellsize as u_int <= (*gd).sx
    {
        wrapped = 1 as u_int;
    }
    if wrapped != 0 {
        xx = (*gl).cellsize as u_int;
    } else {
        xx = window_copy_find_length(wme, sy);
    }
    if ex > xx {
        ex = xx;
    }
    if sx > xx {
        sx = xx;
    }
    if sx < ex {
        i = sx;
        while i < ex {
            grid_get_cell(gd, i, sy, &raw mut gc);
            if !(gc.flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0) {
                if gc.flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
                    utf8_set(&raw mut ud, '\t' as i32 as u_char);
                } else {
                    utf8_copy(&raw mut ud, &raw mut gc.data);
                }
                if ud.size as ::core::ffi::c_int == 1 as ::core::ffi::c_int
                    && gc.attr as ::core::ffi::c_int & GRID_ATTR_CHARSET != 0
                {
                    s = tty_acs_get(
                        ::core::ptr::null_mut::<tty>(),
                        ud.data[0 as ::core::ffi::c_int as usize],
                    );
                    if !s.is_null() && strlen(s) <= ::core::mem::size_of::<[u_char; 32]>() as usize
                    {
                        ud.size = strlen(s) as u_char;
                        memcpy(
                            &raw mut ud.data as *mut u_char as *mut ::core::ffi::c_void,
                            s as *const ::core::ffi::c_void,
                            ud.size as size_t,
                        );
                    }
                }
                *buf = xrealloc(
                    *buf as *mut ::core::ffi::c_void,
                    (*off).wrapping_add(ud.size as size_t),
                ) as *mut ::core::ffi::c_char;
                memcpy(
                    (*buf).offset(*off as isize) as *mut ::core::ffi::c_void,
                    &raw mut ud.data as *mut u_char as *const ::core::ffi::c_void,
                    ud.size as size_t,
                );
                *off = (*off).wrapping_add(ud.size as size_t);
            }
            i = i.wrapping_add(1);
        }
    }
    if wrapped == 0 || ex != xx {
        *buf = xrealloc(
            *buf as *mut ::core::ffi::c_void,
            (*off).wrapping_add(1 as size_t),
        ) as *mut ::core::ffi::c_char;
        let fresh4 = *off;
        *off = (*off).wrapping_add(1);
        *(*buf).offset(fresh4 as isize) = '\n' as i32 as ::core::ffi::c_char;
    }
}
unsafe extern "C" fn window_copy_clear_selection(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    screen_clear_selection(&raw mut (*data).screen);
    (*data).cursordrag = CURSORDRAG_NONE;
    (*data).lineflag = LINE_SEL_NONE;
    (*data).selflag = SEL_CHAR;
    py = (*(*(*data).backing).grid)
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    px = window_copy_cursor_limit(wme, py, (*data).rectflag);
    if (*data).cx > px {
        window_copy_update_cursor(wme, px, (*data).cy);
    }
}
unsafe extern "C" fn window_copy_in_set(
    mut wme: *mut window_mode_entry,
    mut px: u_int,
    mut py: u_int,
    mut set: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    return grid_in_set((*(*data).backing).grid, px, py, set);
}
unsafe extern "C" fn window_copy_find_length(
    mut wme: *mut window_mode_entry,
    mut py: u_int,
) -> u_int {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    return grid_line_length((*(*data).backing).grid, py);
}
unsafe extern "C" fn window_copy_cursor_limit(
    mut wme: *mut window_mode_entry,
    mut py: u_int,
    mut allow_onemore: ::core::ffi::c_int,
) -> u_int {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut oo: *mut options = (*(*(*wme).wp).window).options;
    if allow_onemore != 0
        || options_get_number(
            oo,
            b"mode-keys\0" as *const u8 as *const ::core::ffi::c_char,
        ) != MODEKEY_VI as ::core::ffi::c_longlong
    {
        return window_copy_find_length(wme, py);
    }
    return grid_line_limit((*(*data).backing).grid, py);
}
unsafe extern "C" fn window_copy_cursor_start_of_line(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut back_s: *mut screen = (*data).backing;
    let mut gr: grid_reader = grid_reader {
        gd: ::core::ptr::null_mut::<grid>(),
        cx: 0,
        cy: 0,
    };
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut oldy: u_int = 0;
    let mut hsize: u_int = 0;
    px = (*data).cx;
    hsize = (*(*back_s).grid).hsize;
    py = hsize.wrapping_add((*data).cy).wrapping_sub((*data).oy);
    oldy = (*data).cy;
    grid_reader_start(&raw mut gr, (*back_s).grid, px, py);
    grid_reader_cursor_start_of_line(&raw mut gr, 1 as ::core::ffi::c_int);
    grid_reader_get_cursor(&raw mut gr, &raw mut px, &raw mut py);
    window_copy_acquire_cursor_up(wme, hsize, (*data).oy, oldy, px, py);
}
unsafe extern "C" fn window_copy_cursor_back_to_indentation(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut back_s: *mut screen = (*data).backing;
    let mut gr: grid_reader = grid_reader {
        gd: ::core::ptr::null_mut::<grid>(),
        cx: 0,
        cy: 0,
    };
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut oldy: u_int = 0;
    let mut hsize: u_int = 0;
    px = (*data).cx;
    hsize = (*(*back_s).grid).hsize;
    py = hsize.wrapping_add((*data).cy).wrapping_sub((*data).oy);
    oldy = (*data).cy;
    grid_reader_start(&raw mut gr, (*back_s).grid, px, py);
    grid_reader_cursor_back_to_indentation(&raw mut gr);
    grid_reader_get_cursor(&raw mut gr, &raw mut px, &raw mut py);
    window_copy_acquire_cursor_up(wme, hsize, (*data).oy, oldy, px, py);
}
unsafe extern "C" fn window_copy_cursor_end_of_line(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut back_s: *mut screen = (*data).backing;
    let mut gr: grid_reader = grid_reader {
        gd: ::core::ptr::null_mut::<grid>(),
        cx: 0,
        cy: 0,
    };
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut oldy: u_int = 0;
    let mut hsize: u_int = 0;
    px = (*data).cx;
    hsize = (*(*back_s).grid).hsize;
    py = hsize.wrapping_add((*data).cy).wrapping_sub((*data).oy);
    oldy = (*data).cy;
    grid_reader_start(&raw mut gr, (*back_s).grid, px, py);
    if !(*data).screen.sel.is_null() && (*data).rectflag != 0 {
        grid_reader_cursor_end_of_line(
            &raw mut gr,
            1 as ::core::ffi::c_int,
            1 as ::core::ffi::c_int,
        );
    } else {
        grid_reader_cursor_end_of_line(
            &raw mut gr,
            1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
    }
    grid_reader_get_cursor(&raw mut gr, &raw mut px, &raw mut py);
    if (*data).screen.sel.is_null() || (*data).rectflag == 0 {
        px = window_copy_cursor_limit(wme, py, 0 as ::core::ffi::c_int);
    }
    window_copy_acquire_cursor_down(
        wme,
        hsize,
        (*(*back_s).grid).sy,
        (*data).oy,
        oldy,
        px,
        py,
        0 as ::core::ffi::c_int,
    );
}
unsafe extern "C" fn window_copy_other_end(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut s: *mut screen = &raw mut (*data).screen;
    let mut selx: u_int = 0;
    let mut sely: u_int = 0;
    let mut cy: u_int = 0;
    let mut yy: u_int = 0;
    let mut hsize: u_int = 0;
    if (*s).sel.is_null()
        && (*data).lineflag as ::core::ffi::c_uint
            == LINE_SEL_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return;
    }
    if (*data).lineflag as ::core::ffi::c_uint
        == LINE_SEL_LEFT_RIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*data).lineflag = LINE_SEL_RIGHT_LEFT;
    } else if (*data).lineflag as ::core::ffi::c_uint
        == LINE_SEL_RIGHT_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*data).lineflag = LINE_SEL_LEFT_RIGHT;
    }
    match (*data).cursordrag as ::core::ffi::c_uint {
        0 | 2 => {
            (*data).cursordrag = CURSORDRAG_ENDSEL;
        }
        1 => {
            (*data).cursordrag = CURSORDRAG_SEL;
        }
        _ => {}
    }
    selx = (*data).endselx;
    sely = (*data).endsely;
    if (*data).cursordrag as ::core::ffi::c_uint
        == CURSORDRAG_SEL as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        selx = (*data).selx;
        sely = (*data).sely;
    }
    cy = (*data).cy;
    yy = (*(*(*data).backing).grid)
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    (*data).cx = selx;
    hsize = (*(*(*data).backing).grid).hsize;
    if sely < hsize.wrapping_sub((*data).oy) {
        (*data).oy = hsize.wrapping_sub(sely);
        (*data).cy = 0 as u_int;
    } else if sely > hsize.wrapping_sub((*data).oy).wrapping_add((*(*s).grid).sy) {
        (*data).oy = hsize
            .wrapping_sub(sely)
            .wrapping_add((*(*s).grid).sy)
            .wrapping_sub(1 as u_int);
        (*data).cy = (*(*s).grid).sy.wrapping_sub(1 as u_int);
    } else {
        (*data).cy = cy.wrapping_add(sely).wrapping_sub(yy);
    }
    yy = (*(*(*data).backing).grid)
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    hsize = window_copy_cursor_limit(wme, yy, (*data).rectflag);
    if (*data).cx > hsize {
        (*data).cx = hsize;
    }
    window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
    window_copy_redraw_screen(wme);
}
unsafe extern "C" fn window_copy_cursor_left(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut back_s: *mut screen = (*data).backing;
    let mut gr: grid_reader = grid_reader {
        gd: ::core::ptr::null_mut::<grid>(),
        cx: 0,
        cy: 0,
    };
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut oldy: u_int = 0;
    let mut hsize: u_int = 0;
    px = (*data).cx;
    hsize = (*(*back_s).grid).hsize;
    py = hsize.wrapping_add((*data).cy).wrapping_sub((*data).oy);
    oldy = (*data).cy;
    grid_reader_start(&raw mut gr, (*back_s).grid, px, py);
    grid_reader_cursor_left(&raw mut gr, 1 as ::core::ffi::c_int);
    grid_reader_get_cursor(&raw mut gr, &raw mut px, &raw mut py);
    window_copy_acquire_cursor_up(wme, hsize, (*data).oy, oldy, px, py);
}
unsafe extern "C" fn window_copy_cursor_right(
    mut wme: *mut window_mode_entry,
    mut all: ::core::ffi::c_int,
) {
    let mut wp: *mut window_pane = (*wme).wp;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut oo: *mut options = (*(*wp).window).options;
    let mut back_s: *mut screen = (*data).backing;
    let mut gr: grid_reader = grid_reader {
        gd: ::core::ptr::null_mut::<grid>(),
        cx: 0,
        cy: 0,
    };
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut oldy: u_int = 0;
    let mut hsize: u_int = 0;
    let mut onemore: ::core::ffi::c_int = 0;
    px = (*data).cx;
    hsize = (*(*back_s).grid).hsize;
    py = hsize.wrapping_add((*data).cy).wrapping_sub((*data).oy);
    oldy = (*data).cy;
    onemore = (options_get_number(
        oo,
        b"mode-keys\0" as *const u8 as *const ::core::ffi::c_char,
    ) != MODEKEY_VI as ::core::ffi::c_longlong) as ::core::ffi::c_int;
    grid_reader_start(&raw mut gr, (*back_s).grid, px, py);
    grid_reader_cursor_right(&raw mut gr, 1 as ::core::ffi::c_int, all, onemore);
    grid_reader_get_cursor(&raw mut gr, &raw mut px, &raw mut py);
    window_copy_acquire_cursor_down(
        wme,
        hsize,
        (*(*back_s).grid).sy,
        (*data).oy,
        oldy,
        px,
        py,
        0 as ::core::ffi::c_int,
    );
}
unsafe extern "C" fn window_copy_cursor_up(
    mut wme: *mut window_mode_entry,
    mut scroll_only: ::core::ffi::c_int,
) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut oo: *mut options = (*(*(*wme).wp).window).options;
    let mut s: *mut screen = &raw mut (*data).screen;
    let mut ox: u_int = 0;
    let mut oy: u_int = 0;
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut norectsel: ::core::ffi::c_int = 0;
    norectsel = ((*data).screen.sel.is_null() || (*data).rectflag == 0) as ::core::ffi::c_int;
    oy = (*(*(*data).backing).grid)
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    ox = window_copy_find_length(wme, oy);
    if norectsel != 0 && (*data).cx != ox {
        (*data).lastcx = (*data).cx;
        (*data).lastsx = ox;
    }
    if (*data).lineflag as ::core::ffi::c_uint
        == LINE_SEL_LEFT_RIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
        && oy == (*data).sely
    {
        window_copy_other_end(wme);
    }
    if scroll_only != 0
        && options_get_number(
            oo,
            b"mode-keys\0" as *const u8 as *const ::core::ffi::c_char,
        ) == MODEKEY_VI as ::core::ffi::c_longlong
    {
        if (*data).cy < (*(*s).grid).sy.wrapping_sub(1 as u_int) {
            window_copy_update_cursor(wme, (*data).cx, (*data).cy.wrapping_add(1 as u_int));
        }
    }
    if scroll_only != 0 || (*data).cy == 0 as u_int {
        if norectsel != 0 {
            (*data).cx = (*data).lastcx;
        }
        window_copy_scroll_down(wme, 1 as u_int);
        if scroll_only != 0 {
            if (*data).cy == (*(*s).grid).sy.wrapping_sub(1 as u_int) {
                window_copy_redraw_lines(wme, (*data).cy, 1 as u_int);
            } else {
                window_copy_redraw_lines(wme, (*data).cy, 2 as u_int);
            }
        }
    } else {
        if norectsel != 0 {
            window_copy_update_cursor(wme, (*data).lastcx, (*data).cy.wrapping_sub(1 as u_int));
        } else {
            window_copy_update_cursor(wme, (*data).cx, (*data).cy.wrapping_sub(1 as u_int));
        }
        if window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int) != 0
        {
            if (*data).cy == (*(*s).grid).sy.wrapping_sub(1 as u_int) {
                window_copy_redraw_lines(wme, (*data).cy, 1 as u_int);
            } else {
                window_copy_redraw_lines(wme, (*data).cy, 2 as u_int);
            }
        }
    }
    if norectsel != 0 {
        py = (*(*(*data).backing).grid)
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        px = window_copy_find_length(wme, py);
        if (*data).cx >= (*data).lastsx && (*data).cx != px || (*data).cx > px {
            window_copy_update_cursor(wme, px, (*data).cy);
            if window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int)
                != 0
            {
                window_copy_redraw_lines(wme, (*data).cy, 1 as u_int);
            }
        }
    }
    if (*data).lineflag as ::core::ffi::c_uint
        == LINE_SEL_LEFT_RIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        py = (*(*(*data).backing).grid)
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        if (*data).rectflag != 0 {
            px = (*(*(*data).backing).grid).sx;
        } else {
            px = window_copy_find_length(wme, py);
        }
        window_copy_update_cursor(wme, px, (*data).cy);
        if window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int) != 0
        {
            window_copy_redraw_lines(wme, (*data).cy, 1 as u_int);
        }
    } else if (*data).lineflag as ::core::ffi::c_uint
        == LINE_SEL_RIGHT_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        window_copy_update_cursor(wme, 0 as u_int, (*data).cy);
        if window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int) != 0
        {
            window_copy_redraw_lines(wme, (*data).cy, 1 as u_int);
        }
    }
}
unsafe extern "C" fn window_copy_cursor_down(
    mut wme: *mut window_mode_entry,
    mut scroll_only: ::core::ffi::c_int,
) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut oo: *mut options = (*(*(*wme).wp).window).options;
    let mut s: *mut screen = &raw mut (*data).screen;
    let mut ox: u_int = 0;
    let mut oy: u_int = 0;
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut norectsel: ::core::ffi::c_int = 0;
    norectsel = ((*data).screen.sel.is_null() || (*data).rectflag == 0) as ::core::ffi::c_int;
    oy = (*(*(*data).backing).grid)
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    ox = window_copy_find_length(wme, oy);
    if norectsel != 0 && (*data).cx != ox {
        (*data).lastcx = (*data).cx;
        (*data).lastsx = ox;
    }
    if (*data).lineflag as ::core::ffi::c_uint
        == LINE_SEL_RIGHT_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
        && oy == (*data).endsely
    {
        window_copy_other_end(wme);
    }
    if scroll_only != 0
        && options_get_number(
            oo,
            b"mode-keys\0" as *const u8 as *const ::core::ffi::c_char,
        ) == MODEKEY_VI as ::core::ffi::c_longlong
    {
        if (*data).cy > 0 as u_int {
            window_copy_update_cursor(wme, (*data).cx, (*data).cy.wrapping_sub(1 as u_int));
        }
    }
    if scroll_only != 0 || (*data).cy == (*(*s).grid).sy.wrapping_sub(1 as u_int) {
        if norectsel != 0 {
            (*data).cx = (*data).lastcx;
        }
        window_copy_scroll_up(wme, 1 as u_int);
        if scroll_only != 0 && (*data).cy > 0 as u_int {
            window_copy_redraw_lines(wme, (*data).cy.wrapping_sub(1 as u_int), 2 as u_int);
        }
    } else {
        if norectsel != 0 {
            window_copy_update_cursor(wme, (*data).lastcx, (*data).cy.wrapping_add(1 as u_int));
        } else {
            window_copy_update_cursor(wme, (*data).cx, (*data).cy.wrapping_add(1 as u_int));
        }
        if window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int) != 0
        {
            window_copy_redraw_lines(wme, (*data).cy.wrapping_sub(1 as u_int), 2 as u_int);
        }
    }
    if norectsel != 0 {
        py = (*(*(*data).backing).grid)
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        px = window_copy_find_length(wme, py);
        if (*data).cx >= (*data).lastsx && (*data).cx != px || (*data).cx > px {
            window_copy_update_cursor(wme, px, (*data).cy);
            if window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int)
                != 0
            {
                window_copy_redraw_lines(wme, (*data).cy, 1 as u_int);
            }
        }
    }
    if (*data).lineflag as ::core::ffi::c_uint
        == LINE_SEL_LEFT_RIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        py = (*(*(*data).backing).grid)
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        if (*data).rectflag != 0 {
            px = (*(*(*data).backing).grid).sx;
        } else {
            px = window_copy_find_length(wme, py);
        }
        window_copy_update_cursor(wme, px, (*data).cy);
        if window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int) != 0
        {
            window_copy_redraw_lines(wme, (*data).cy, 1 as u_int);
        }
    } else if (*data).lineflag as ::core::ffi::c_uint
        == LINE_SEL_RIGHT_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        window_copy_update_cursor(wme, 0 as u_int, (*data).cy);
        if window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int) != 0
        {
            window_copy_redraw_lines(wme, (*data).cy, 1 as u_int);
        }
    }
}
unsafe extern "C" fn window_copy_cursor_jump(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut back_s: *mut screen = (*data).backing;
    let mut gr: grid_reader = grid_reader {
        gd: ::core::ptr::null_mut::<grid>(),
        cx: 0,
        cy: 0,
    };
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut oldy: u_int = 0;
    let mut hsize: u_int = 0;
    px = (*data).cx.wrapping_add(1 as u_int);
    hsize = (*(*back_s).grid).hsize;
    py = hsize.wrapping_add((*data).cy).wrapping_sub((*data).oy);
    oldy = (*data).cy;
    grid_reader_start(&raw mut gr, (*back_s).grid, px, py);
    if grid_reader_cursor_jump(&raw mut gr, (*data).jumpchar) != 0 {
        grid_reader_get_cursor(&raw mut gr, &raw mut px, &raw mut py);
        window_copy_acquire_cursor_down(
            wme,
            hsize,
            (*(*back_s).grid).sy,
            (*data).oy,
            oldy,
            px,
            py,
            0 as ::core::ffi::c_int,
        );
    }
}
unsafe extern "C" fn window_copy_cursor_jump_back(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut back_s: *mut screen = (*data).backing;
    let mut gr: grid_reader = grid_reader {
        gd: ::core::ptr::null_mut::<grid>(),
        cx: 0,
        cy: 0,
    };
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut oldy: u_int = 0;
    let mut hsize: u_int = 0;
    px = (*data).cx;
    hsize = (*(*back_s).grid).hsize;
    py = hsize.wrapping_add((*data).cy).wrapping_sub((*data).oy);
    oldy = (*data).cy;
    grid_reader_start(&raw mut gr, (*back_s).grid, px, py);
    grid_reader_cursor_left(&raw mut gr, 0 as ::core::ffi::c_int);
    if grid_reader_cursor_jump_back(&raw mut gr, (*data).jumpchar) != 0 {
        grid_reader_get_cursor(&raw mut gr, &raw mut px, &raw mut py);
        window_copy_acquire_cursor_up(wme, hsize, (*data).oy, oldy, px, py);
    }
}
unsafe extern "C" fn window_copy_cursor_jump_to(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut back_s: *mut screen = (*data).backing;
    let mut gr: grid_reader = grid_reader {
        gd: ::core::ptr::null_mut::<grid>(),
        cx: 0,
        cy: 0,
    };
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut oldy: u_int = 0;
    let mut hsize: u_int = 0;
    px = (*data).cx.wrapping_add(2 as u_int);
    hsize = (*(*back_s).grid).hsize;
    py = hsize.wrapping_add((*data).cy).wrapping_sub((*data).oy);
    oldy = (*data).cy;
    grid_reader_start(&raw mut gr, (*back_s).grid, px, py);
    if grid_reader_cursor_jump(&raw mut gr, (*data).jumpchar) != 0 {
        grid_reader_cursor_left(&raw mut gr, 1 as ::core::ffi::c_int);
        grid_reader_get_cursor(&raw mut gr, &raw mut px, &raw mut py);
        window_copy_acquire_cursor_down(
            wme,
            hsize,
            (*(*back_s).grid).sy,
            (*data).oy,
            oldy,
            px,
            py,
            0 as ::core::ffi::c_int,
        );
    }
}
unsafe extern "C" fn window_copy_cursor_jump_to_back(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut oo: *mut options = (*(*(*wme).wp).window).options;
    let mut back_s: *mut screen = (*data).backing;
    let mut gr: grid_reader = grid_reader {
        gd: ::core::ptr::null_mut::<grid>(),
        cx: 0,
        cy: 0,
    };
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut oldy: u_int = 0;
    let mut hsize: u_int = 0;
    let mut onemore: ::core::ffi::c_int = 0;
    px = (*data).cx;
    hsize = (*(*back_s).grid).hsize;
    py = hsize.wrapping_add((*data).cy).wrapping_sub((*data).oy);
    oldy = (*data).cy;
    onemore = (options_get_number(
        oo,
        b"mode-keys\0" as *const u8 as *const ::core::ffi::c_char,
    ) != MODEKEY_VI as ::core::ffi::c_longlong) as ::core::ffi::c_int;
    grid_reader_start(&raw mut gr, (*back_s).grid, px, py);
    grid_reader_cursor_left(&raw mut gr, 0 as ::core::ffi::c_int);
    grid_reader_cursor_left(&raw mut gr, 0 as ::core::ffi::c_int);
    if grid_reader_cursor_jump_back(&raw mut gr, (*data).jumpchar) != 0 {
        grid_reader_cursor_right(
            &raw mut gr,
            1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            onemore,
        );
        grid_reader_get_cursor(&raw mut gr, &raw mut px, &raw mut py);
        window_copy_acquire_cursor_up(wme, hsize, (*data).oy, oldy, px, py);
    }
}
unsafe extern "C" fn window_copy_cursor_next_word(
    mut wme: *mut window_mode_entry,
    mut separators: *const ::core::ffi::c_char,
) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut back_s: *mut screen = (*data).backing;
    let mut gr: grid_reader = grid_reader {
        gd: ::core::ptr::null_mut::<grid>(),
        cx: 0,
        cy: 0,
    };
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut oldy: u_int = 0;
    let mut hsize: u_int = 0;
    px = (*data).cx;
    hsize = (*(*back_s).grid).hsize;
    py = hsize.wrapping_add((*data).cy).wrapping_sub((*data).oy);
    oldy = (*data).cy;
    grid_reader_start(&raw mut gr, (*back_s).grid, px, py);
    grid_reader_cursor_next_word(&raw mut gr, separators);
    grid_reader_get_cursor(&raw mut gr, &raw mut px, &raw mut py);
    window_copy_acquire_cursor_down(
        wme,
        hsize,
        (*(*back_s).grid).sy,
        (*data).oy,
        oldy,
        px,
        py,
        0 as ::core::ffi::c_int,
    );
}
unsafe extern "C" fn window_copy_cursor_next_word_end_pos(
    mut wme: *mut window_mode_entry,
    mut separators: *const ::core::ffi::c_char,
    mut ppx: *mut u_int,
    mut ppy: *mut u_int,
) {
    let mut wp: *mut window_pane = (*wme).wp;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut oo: *mut options = (*(*wp).window).options;
    let mut back_s: *mut screen = (*data).backing;
    let mut gr: grid_reader = grid_reader {
        gd: ::core::ptr::null_mut::<grid>(),
        cx: 0,
        cy: 0,
    };
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut hsize: u_int = 0;
    px = (*data).cx;
    hsize = (*(*back_s).grid).hsize;
    py = hsize.wrapping_add((*data).cy).wrapping_sub((*data).oy);
    grid_reader_start(&raw mut gr, (*back_s).grid, px, py);
    if options_get_number(
        oo,
        b"mode-keys\0" as *const u8 as *const ::core::ffi::c_char,
    ) == MODEKEY_VI as ::core::ffi::c_longlong
    {
        if grid_reader_in_set(&raw mut gr, WHITESPACE.as_ptr()) == 0 {
            grid_reader_cursor_right(
                &raw mut gr,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
        }
        grid_reader_cursor_next_word_end(&raw mut gr, separators);
        grid_reader_cursor_left(&raw mut gr, 1 as ::core::ffi::c_int);
    } else {
        grid_reader_cursor_next_word_end(&raw mut gr, separators);
    }
    grid_reader_get_cursor(&raw mut gr, &raw mut px, &raw mut py);
    *ppx = px;
    *ppy = py;
}
unsafe extern "C" fn window_copy_cursor_next_word_end(
    mut wme: *mut window_mode_entry,
    mut separators: *const ::core::ffi::c_char,
    mut no_reset: ::core::ffi::c_int,
) {
    let mut wp: *mut window_pane = (*wme).wp;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut oo: *mut options = (*(*wp).window).options;
    let mut back_s: *mut screen = (*data).backing;
    let mut gr: grid_reader = grid_reader {
        gd: ::core::ptr::null_mut::<grid>(),
        cx: 0,
        cy: 0,
    };
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut oldy: u_int = 0;
    let mut hsize: u_int = 0;
    px = (*data).cx;
    hsize = (*(*back_s).grid).hsize;
    py = hsize.wrapping_add((*data).cy).wrapping_sub((*data).oy);
    oldy = (*data).cy;
    grid_reader_start(&raw mut gr, (*back_s).grid, px, py);
    if options_get_number(
        oo,
        b"mode-keys\0" as *const u8 as *const ::core::ffi::c_char,
    ) == MODEKEY_VI as ::core::ffi::c_longlong
    {
        if grid_reader_in_set(&raw mut gr, WHITESPACE.as_ptr()) == 0 {
            grid_reader_cursor_right(
                &raw mut gr,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
        }
        grid_reader_cursor_next_word_end(&raw mut gr, separators);
        grid_reader_cursor_left(&raw mut gr, 1 as ::core::ffi::c_int);
    } else {
        grid_reader_cursor_next_word_end(&raw mut gr, separators);
    }
    grid_reader_get_cursor(&raw mut gr, &raw mut px, &raw mut py);
    window_copy_acquire_cursor_down(
        wme,
        hsize,
        (*(*back_s).grid).sy,
        (*data).oy,
        oldy,
        px,
        py,
        no_reset,
    );
}
unsafe extern "C" fn window_copy_cursor_previous_word_pos(
    mut wme: *mut window_mode_entry,
    mut separators: *const ::core::ffi::c_char,
    mut ppx: *mut u_int,
    mut ppy: *mut u_int,
) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut back_s: *mut screen = (*data).backing;
    let mut gr: grid_reader = grid_reader {
        gd: ::core::ptr::null_mut::<grid>(),
        cx: 0,
        cy: 0,
    };
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut hsize: u_int = 0;
    px = (*data).cx;
    hsize = (*(*back_s).grid).hsize;
    py = hsize.wrapping_add((*data).cy).wrapping_sub((*data).oy);
    grid_reader_start(&raw mut gr, (*back_s).grid, px, py);
    grid_reader_cursor_previous_word(
        &raw mut gr,
        separators,
        0 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    grid_reader_get_cursor(&raw mut gr, &raw mut px, &raw mut py);
    *ppx = px;
    *ppy = py;
}
unsafe extern "C" fn window_copy_cursor_previous_word(
    mut wme: *mut window_mode_entry,
    mut separators: *const ::core::ffi::c_char,
    mut already: ::core::ffi::c_int,
) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut w: *mut window = (*(*wme).wp).window as *mut window;
    let mut back_s: *mut screen = (*data).backing;
    let mut gr: grid_reader = grid_reader {
        gd: ::core::ptr::null_mut::<grid>(),
        cx: 0,
        cy: 0,
    };
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut oldy: u_int = 0;
    let mut hsize: u_int = 0;
    let mut stop_at_eol: ::core::ffi::c_int = 0;
    if options_get_number(
        (*w).options,
        b"mode-keys\0" as *const u8 as *const ::core::ffi::c_char,
    ) == MODEKEY_EMACS as ::core::ffi::c_longlong
    {
        stop_at_eol = 1 as ::core::ffi::c_int;
    } else {
        stop_at_eol = 0 as ::core::ffi::c_int;
    }
    px = (*data).cx;
    hsize = (*(*back_s).grid).hsize;
    py = hsize.wrapping_add((*data).cy).wrapping_sub((*data).oy);
    oldy = (*data).cy;
    grid_reader_start(&raw mut gr, (*back_s).grid, px, py);
    grid_reader_cursor_previous_word(&raw mut gr, separators, already, stop_at_eol);
    grid_reader_get_cursor(&raw mut gr, &raw mut px, &raw mut py);
    window_copy_acquire_cursor_up(wme, hsize, (*data).oy, oldy, px, py);
}
unsafe extern "C" fn window_copy_cursor_prompt(
    mut wme: *mut window_mode_entry,
    mut direction: ::core::ffi::c_int,
    mut start_output: ::core::ffi::c_int,
) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut s: *mut screen = (*data).backing;
    let mut gd: *mut grid = (*s).grid;
    let mut end_line: u_int = 0;
    let mut line: u_int = (*gd)
        .hsize
        .wrapping_sub((*data).oy)
        .wrapping_add((*data).cy);
    let mut add: ::core::ffi::c_int = 0;
    let mut line_flag: ::core::ffi::c_int = 0;
    if start_output != 0 {
        line_flag = GRID_LINE_START_OUTPUT;
    } else {
        line_flag = GRID_LINE_START_PROMPT;
    }
    if direction == 0 as ::core::ffi::c_int {
        add = -(1 as ::core::ffi::c_int);
        end_line = 0 as u_int;
    } else {
        add = 1 as ::core::ffi::c_int;
        end_line = (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(1 as u_int);
    }
    if line == end_line {
        return;
    }
    loop {
        if line == end_line {
            return;
        }
        line = line.wrapping_add(add as u_int);
        if (*grid_get_line(gd, line)).flags as ::core::ffi::c_int & line_flag != 0 {
            break;
        }
    }
    (*data).cx = 0 as u_int;
    if line > (*gd).hsize {
        (*data).cy = line.wrapping_sub((*gd).hsize);
        (*data).oy = 0 as u_int;
    } else {
        (*data).cy = 0 as u_int;
        (*data).oy = (*gd).hsize.wrapping_sub(line);
    }
    window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    window_copy_redraw_screen(wme);
}
unsafe extern "C" fn window_copy_scroll_up(mut wme: *mut window_mode_entry, mut ny: u_int) {
    let mut wp: *mut window_pane = (*wme).wp;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut s: *mut screen = &raw mut (*data).screen;
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
    if (*data).oy < ny {
        ny = (*data).oy;
    }
    if ny == 0 as u_int {
        return;
    }
    (*data).oy = (*data).oy.wrapping_sub(ny);
    window_pane_scrollbar_show(wp, 1 as ::core::ffi::c_int);
    if !(*data).searchmark.is_null() && (*data).timeout == 0 {
        window_copy_search_marks(
            wme,
            ::core::ptr::null_mut::<screen>(),
            (*data).searchregex,
            1 as ::core::ffi::c_int,
        );
    }
    window_copy_update_selection_view(wme, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    if window_copy_cursor_line_active(wme) != 0 {
        window_copy_redraw_screen(wme);
        return;
    }
    if window_copy_line_numbers_active(wme) != 0 {
        if window_copy_line_number_mode(wme)
            != WINDOW_COPY_LINE_NUMBERS_ABSOLUTE as ::core::ffi::c_int
        {
            window_copy_redraw_screen(wme);
            return;
        }
        screen_write_start(&raw mut ctx, &raw mut (*data).screen);
        screen_write_cursormove(
            &raw mut ctx,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_deleteline(&raw mut ctx, ny, 8 as u_int);
        window_copy_write_lines(wme, &raw mut ctx, (*(*s).grid).sy.wrapping_sub(ny), ny);
        window_copy_write_line(wme, &raw mut ctx, 0 as u_int);
        if (*(*s).grid).sy > 1 as u_int {
            window_copy_write_line(wme, &raw mut ctx, 1 as u_int);
        }
        if (*(*s).grid).sy > 3 as u_int {
            window_copy_write_line(wme, &raw mut ctx, (*(*s).grid).sy.wrapping_sub(2 as u_int));
        }
        if !(*s).sel.is_null() && (*(*s).grid).sy > ny {
            window_copy_write_line(
                wme,
                &raw mut ctx,
                (*(*s).grid).sy.wrapping_sub(ny).wrapping_sub(1 as u_int),
            );
        }
        screen_write_cursormove(
            &raw mut ctx,
            window_copy_cursor_offset(wme, (*data).cx, (*(*s).grid).sx) as ::core::ffi::c_int,
            (*data).cy as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_stop(&raw mut ctx);
        (*wp).flags |= PANE_REDRAW | PANE_REDRAWSCROLLBAR;
        return;
    }
    if window_pane_scrollbar_overlay_visible(wp) != 0 {
        screen_write_start(&raw mut ctx, &raw mut (*data).screen);
    } else {
        screen_write_start_pane(&raw mut ctx, wp, ::core::ptr::null_mut::<screen>());
    }
    screen_write_cursormove(
        &raw mut ctx,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    screen_write_deleteline(&raw mut ctx, ny, 8 as u_int);
    window_copy_write_lines(wme, &raw mut ctx, (*(*s).grid).sy.wrapping_sub(ny), ny);
    window_copy_write_line(wme, &raw mut ctx, 0 as u_int);
    if (*(*s).grid).sy > 1 as u_int {
        window_copy_write_line(wme, &raw mut ctx, 1 as u_int);
    }
    if (*(*s).grid).sy > 3 as u_int {
        window_copy_write_line(wme, &raw mut ctx, (*(*s).grid).sy.wrapping_sub(2 as u_int));
    }
    if !(*s).sel.is_null() && (*(*s).grid).sy > ny {
        window_copy_write_line(
            wme,
            &raw mut ctx,
            (*(*s).grid).sy.wrapping_sub(ny).wrapping_sub(1 as u_int),
        );
    }
    screen_write_cursormove(
        &raw mut ctx,
        window_copy_cursor_offset(wme, (*data).cx, (*(*s).grid).sx) as ::core::ffi::c_int,
        (*data).cy as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    screen_write_stop(&raw mut ctx);
    window_pane_scrollbar_redraw(wp);
}
unsafe extern "C" fn window_copy_scroll_down(mut wme: *mut window_mode_entry, mut ny: u_int) {
    let mut wp: *mut window_pane = (*wme).wp;
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut s: *mut screen = &raw mut (*data).screen;
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
    if ny > (*(*(*data).backing).grid).hsize {
        return;
    }
    if (*data).oy > (*(*(*data).backing).grid).hsize.wrapping_sub(ny) {
        ny = (*(*(*data).backing).grid).hsize.wrapping_sub((*data).oy);
    }
    if ny == 0 as u_int {
        return;
    }
    (*data).oy = (*data).oy.wrapping_add(ny);
    window_pane_scrollbar_show(wp, 1 as ::core::ffi::c_int);
    if !(*data).searchmark.is_null() && (*data).timeout == 0 {
        window_copy_search_marks(
            wme,
            ::core::ptr::null_mut::<screen>(),
            (*data).searchregex,
            1 as ::core::ffi::c_int,
        );
    }
    window_copy_update_selection_view(wme, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    if window_copy_cursor_line_active(wme) != 0 {
        window_copy_redraw_screen(wme);
        return;
    }
    if window_copy_line_numbers_active(wme) != 0 {
        if window_copy_line_number_mode(wme)
            != WINDOW_COPY_LINE_NUMBERS_ABSOLUTE as ::core::ffi::c_int
        {
            window_copy_redraw_screen(wme);
            return;
        }
        screen_write_start(&raw mut ctx, &raw mut (*data).screen);
        screen_write_cursormove(
            &raw mut ctx,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_insertline(&raw mut ctx, ny, 8 as u_int);
        window_copy_write_lines(wme, &raw mut ctx, 0 as u_int, ny);
        if !(*s).sel.is_null() && (*(*s).grid).sy > ny {
            window_copy_write_line(wme, &raw mut ctx, ny);
        } else if ny == 1 as u_int {
            window_copy_write_line(wme, &raw mut ctx, 1 as u_int);
        }
        screen_write_cursormove(
            &raw mut ctx,
            window_copy_cursor_offset(wme, (*data).cx, (*(*s).grid).sx) as ::core::ffi::c_int,
            (*data).cy as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_stop(&raw mut ctx);
        (*wp).flags |= PANE_REDRAW | PANE_REDRAWSCROLLBAR;
        return;
    }
    if window_pane_scrollbar_overlay_visible(wp) != 0 {
        screen_write_start(&raw mut ctx, &raw mut (*data).screen);
    } else {
        screen_write_start_pane(&raw mut ctx, wp, ::core::ptr::null_mut::<screen>());
    }
    screen_write_cursormove(
        &raw mut ctx,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    screen_write_insertline(&raw mut ctx, ny, 8 as u_int);
    window_copy_write_lines(wme, &raw mut ctx, 0 as u_int, ny);
    if !(*s).sel.is_null() && (*(*s).grid).sy > ny {
        window_copy_write_line(wme, &raw mut ctx, ny);
    } else if ny == 1 as u_int {
        window_copy_write_line(wme, &raw mut ctx, 1 as u_int);
    }
    screen_write_cursormove(
        &raw mut ctx,
        window_copy_cursor_offset(wme, (*data).cx, (*(*s).grid).sx) as ::core::ffi::c_int,
        (*data).cy as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    screen_write_stop(&raw mut ctx);
    window_pane_scrollbar_redraw(wp);
}
unsafe extern "C" fn window_copy_rectangle_set(
    mut wme: *mut window_mode_entry,
    mut rectflag: ::core::ffi::c_int,
) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    (*data).rectflag = rectflag;
    py = (*(*(*data).backing).grid)
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    px = window_copy_cursor_limit(wme, py, (*data).rectflag);
    if (*data).cx > px {
        window_copy_update_cursor(wme, px, (*data).cy);
    }
    window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    window_copy_redraw_screen(wme);
}
unsafe extern "C" fn window_copy_move_mouse(mut m: *mut mouse_event) {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut data: *mut window_copy_mode_data = ::core::ptr::null_mut::<window_copy_mode_data>();
    wp = cmd_mouse_pane(
        m,
        ::core::ptr::null_mut::<*mut session>(),
        ::core::ptr::null_mut::<*mut winlink>(),
    );
    if wp.is_null() {
        return;
    }
    wme = (*wp).modes.tqh_first;
    if wme.is_null() {
        return;
    }
    if (*wme).mode != &raw const window_copy_mode && (*wme).mode != &raw const window_view_mode {
        return;
    }
    if cmd_mouse_at(wp, m, &raw mut x, &raw mut y, 0 as ::core::ffi::c_int)
        != 0 as ::core::ffi::c_int
    {
        return;
    }
    data = (*wme).data as *mut window_copy_mode_data;
    x = window_copy_cursor_unoffset(wme, x, (*(*data).screen.grid).sx);
    window_copy_update_cursor(wme, x, y);
}
#[no_mangle]
pub unsafe extern "C" fn window_copy_start_drag(mut c: *mut client, mut m: *mut mouse_event) {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    let mut data: *mut window_copy_mode_data = ::core::ptr::null_mut::<window_copy_mode_data>();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut yg: u_int = 0;
    let mut inside_selection: ::core::ffi::c_int = 0;
    let mut on_start: ::core::ffi::c_int = 0;
    let mut on_end: ::core::ffi::c_int = 0;
    if c.is_null() {
        return;
    }
    wp = cmd_mouse_pane(
        m,
        ::core::ptr::null_mut::<*mut session>(),
        ::core::ptr::null_mut::<*mut winlink>(),
    );
    if wp.is_null() {
        return;
    }
    wme = (*wp).modes.tqh_first;
    if wme.is_null() {
        return;
    }
    if (*wme).mode != &raw const window_copy_mode && (*wme).mode != &raw const window_view_mode {
        return;
    }
    if cmd_mouse_at(wp, m, &raw mut x, &raw mut y, 1 as ::core::ffi::c_int)
        != 0 as ::core::ffi::c_int
    {
        return;
    }
    (*c).tty.mouse_drag_update =
        Some(window_copy_drag_update as unsafe extern "C" fn(*mut client, *mut mouse_event) -> ())
            as Option<unsafe extern "C" fn(*mut client, *mut mouse_event) -> ()>;
    (*c).tty.mouse_drag_release =
        Some(window_copy_drag_release as unsafe extern "C" fn(*mut client, *mut mouse_event) -> ())
            as Option<unsafe extern "C" fn(*mut client, *mut mouse_event) -> ()>;
    data = (*wme).data as *mut window_copy_mode_data;
    on_end = 0 as ::core::ffi::c_int;
    on_start = on_end;
    inside_selection =
        window_copy_mouse_in_selection(wme, x, y, &raw mut on_start, &raw mut on_end);
    x = window_copy_cursor_unoffset(wme, x, (*(*data).screen.grid).sx);
    yg = (*(*(*data).backing).grid)
        .hsize
        .wrapping_add(y)
        .wrapping_sub((*data).oy);
    if on_start != 0
        || on_end != 0
        || inside_selection == 0
        || x < (*data).selrx
        || x > (*data).endselrx
        || yg != (*data).selry
    {
        (*data).lineflag = LINE_SEL_NONE;
        (*data).selflag = SEL_CHAR;
    }
    match (*data).selflag as ::core::ffi::c_uint {
        1 => {
            if !(*data).separators.is_null() {
                window_copy_update_cursor(wme, x, y);
                window_copy_cursor_previous_word_pos(
                    wme,
                    (*data).separators,
                    &raw mut x,
                    &raw mut y,
                );
                y = y.wrapping_sub((*(*(*data).backing).grid).hsize.wrapping_sub((*data).oy));
            }
            window_copy_update_cursor(wme, x, y);
        }
        2 => {
            window_copy_update_cursor(wme, 0 as u_int, y);
        }
        0 => {
            window_copy_update_cursor(wme, x, y);
            if inside_selection == 0 {
                window_copy_start_selection(wme);
            } else {
                if on_start != 0 {
                    (*data).cursordrag = CURSORDRAG_SEL;
                } else if on_end != 0 {
                    (*data).cursordrag = CURSORDRAG_ENDSEL;
                }
                window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
            }
        }
        _ => {}
    }
    window_copy_redraw_screen(wme);
    window_copy_drag_update(c, m);
}
unsafe extern "C" fn window_copy_drag_update(mut c: *mut client, mut m: *mut mouse_event) {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    let mut data: *mut window_copy_mode_data = ::core::ptr::null_mut::<window_copy_mode_data>();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut old_cx: u_int = 0;
    let mut old_cy: u_int = 0;
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: WINDOW_COPY_DRAG_REPEAT_TIME as __suseconds_t,
    };
    if c.is_null() {
        return;
    }
    wp = cmd_mouse_pane(
        m,
        ::core::ptr::null_mut::<*mut session>(),
        ::core::ptr::null_mut::<*mut winlink>(),
    );
    if wp.is_null() {
        return;
    }
    wme = (*wp).modes.tqh_first;
    if wme.is_null() {
        return;
    }
    if (*wme).mode != &raw const window_copy_mode && (*wme).mode != &raw const window_view_mode {
        return;
    }
    data = (*wme).data as *mut window_copy_mode_data;
    event_del(&raw mut (*data).dragtimer);
    if cmd_mouse_at(wp, m, &raw mut x, &raw mut y, 0 as ::core::ffi::c_int)
        != 0 as ::core::ffi::c_int
    {
        return;
    }
    x = window_copy_cursor_unoffset(wme, x, (*(*data).screen.grid).sx);
    old_cx = (*data).cx;
    old_cy = (*data).cy;
    window_copy_update_cursor(wme, x, y);
    if window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int) != 0 {
        window_copy_redraw_selection(wme, old_cy);
    }
    if old_cy != (*data).cy || old_cx == (*data).cx {
        if y == 0 as u_int {
            event_add(&raw mut (*data).dragtimer, &raw mut tv);
            window_copy_cursor_up(wme, 1 as ::core::ffi::c_int);
        } else if y == (*(*data).screen.grid).sy.wrapping_sub(1 as u_int) {
            event_add(&raw mut (*data).dragtimer, &raw mut tv);
            window_copy_cursor_down(wme, 1 as ::core::ffi::c_int);
        }
    }
}
unsafe extern "C" fn window_copy_drag_release(mut c: *mut client, mut m: *mut mouse_event) {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    let mut data: *mut window_copy_mode_data = ::core::ptr::null_mut::<window_copy_mode_data>();
    if c.is_null() {
        return;
    }
    wp = cmd_mouse_pane(
        m,
        ::core::ptr::null_mut::<*mut session>(),
        ::core::ptr::null_mut::<*mut winlink>(),
    );
    if wp.is_null() {
        return;
    }
    wme = (*wp).modes.tqh_first;
    if wme.is_null() {
        return;
    }
    if (*wme).mode != &raw const window_copy_mode && (*wme).mode != &raw const window_view_mode {
        return;
    }
    data = (*wme).data as *mut window_copy_mode_data;
    if window_copy_line_numbers_active(wme) != 0 {
        window_copy_drag_update(c, m);
    }
    (*data).cursordrag = CURSORDRAG_NONE;
    event_del(&raw mut (*data).dragtimer);
}
unsafe extern "C" fn window_copy_jump_to_mark(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_copy_mode_data = (*wme).data as *mut window_copy_mode_data;
    let mut tmx: u_int = 0;
    let mut tmy: u_int = 0;
    tmx = (*data).cx;
    tmy = (*(*(*data).backing).grid)
        .hsize
        .wrapping_add((*data).cy)
        .wrapping_sub((*data).oy);
    (*data).cx = (*data).mx;
    if (*data).my < (*(*(*data).backing).grid).hsize {
        (*data).cy = 0 as u_int;
        (*data).oy = (*(*(*data).backing).grid).hsize.wrapping_sub((*data).my);
    } else {
        (*data).cy = (*data).my.wrapping_sub((*(*(*data).backing).grid).hsize);
        (*data).oy = 0 as u_int;
    }
    (*data).mx = tmx;
    (*data).my = tmy;
    (*data).showmark = 1 as ::core::ffi::c_int;
    window_copy_update_selection(wme, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    window_copy_redraw_screen(wme);
}
unsafe extern "C" fn window_copy_acquire_cursor_up(
    mut wme: *mut window_mode_entry,
    mut hsize: u_int,
    mut oy: u_int,
    mut oldy: u_int,
    mut px: u_int,
    mut py: u_int,
) {
    let mut cy: u_int = 0;
    let mut yy: u_int = 0;
    let mut ny: u_int = 0;
    let mut nd: u_int = 0;
    yy = hsize.wrapping_sub(oy);
    if py < yy {
        ny = yy.wrapping_sub(py);
        cy = 0 as u_int;
        nd = 1 as u_int;
    } else {
        ny = 0 as u_int;
        cy = py.wrapping_sub(yy);
        nd = oldy.wrapping_sub(cy).wrapping_add(1 as u_int);
    }
    while ny > 0 as u_int {
        window_copy_cursor_up(wme, 1 as ::core::ffi::c_int);
        ny = ny.wrapping_sub(1);
    }
    window_copy_update_cursor(wme, px, cy);
    if window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int) != 0 {
        window_copy_redraw_lines(wme, cy, nd);
    }
}
unsafe extern "C" fn window_copy_acquire_cursor_down(
    mut wme: *mut window_mode_entry,
    mut hsize: u_int,
    mut sy: u_int,
    mut oy: u_int,
    mut oldy: u_int,
    mut px: u_int,
    mut py: u_int,
    mut no_reset: ::core::ffi::c_int,
) {
    let mut cy: u_int = 0;
    let mut yy: u_int = 0;
    let mut ny: u_int = 0;
    let mut nd: u_int = 0;
    cy = py.wrapping_sub(hsize).wrapping_add(oy);
    yy = sy.wrapping_sub(1 as u_int);
    if cy > yy {
        ny = cy.wrapping_sub(yy);
        oldy = yy;
        nd = 1 as u_int;
    } else {
        ny = 0 as u_int;
        nd = cy.wrapping_sub(oldy).wrapping_add(1 as u_int);
    }
    while ny > 0 as u_int {
        window_copy_cursor_down(wme, 1 as ::core::ffi::c_int);
        ny = ny.wrapping_sub(1);
    }
    if cy > yy {
        window_copy_update_cursor(wme, px, yy);
    } else {
        window_copy_update_cursor(wme, px, cy);
    }
    if window_copy_update_selection(wme, 1 as ::core::ffi::c_int, no_reset) != 0 {
        window_copy_redraw_lines(wme, oldy, nd);
    }
}
