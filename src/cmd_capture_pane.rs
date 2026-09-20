pub use crate::src::shared::arguments::{args, args_parse, args_parse_cb};
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
pub use crate::src::shared::command::{
    cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds,
};
pub use crate::src::shared::control::{control_state};
pub use crate::src::shared::format::{format_job_tree, format_tree};
pub use crate::src::shared::input::{input_ctx, input_request, input_requests};
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
pub use crate::src::shared::process::{tmuxpeer};
pub use crate::src::shared::prompt::{prompt};
pub use crate::src::shared::redraw::{redraw_scene};
pub use crate::src::shared::session::{session, session_entry, session_gentry};
pub use crate::src::shared::spawn::{spawn_editor_state};
pub use crate::src::shared::status::{status_line};
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
pub use crate::src::shared::environment::{environ};
pub use crate::src::shared::vis::{VIS_CSTYLE, VIS_NL, VIS_OCTAL, VIS_TAB};
pub use crate::src::shared::limits::{__INT_MAX__, __SHRT_MAX__, INT_MIN, SHRT_MAX};
pub use crate::src::shared::abi::{ssize_t};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::colour::{COLOUR_FLAG_256};
pub use crate::src::shared::command::{CMD_AFTERHOOK};
pub use crate::src::shared::client::{CLIENT_CONTROL};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{mouse_event};
use crate::src::shared::client::*;
use crate::src::shared::arguments::*;
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

    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn evbuffer_get_length(buf: *const evbuffer) -> size_t;
    fn evbuffer_pullup(buf: *mut evbuffer, size: ssize_t) -> *mut ::core::ffi::c_uchar;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xrealloc(_: *mut ::core::ffi::c_void, _: size_t) -> *mut ::core::ffi::c_void;
    fn xreallocarray(_: *mut ::core::ffi::c_void, _: size_t, _: size_t)
        -> *mut ::core::ffi::c_void;
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn xasprintf(
        _: *mut *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn paste_set(
        _: *mut ::core::ffi::c_char,
        _: size_t,
        _: *const ::core::ffi::c_char,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn args_has(_: *mut args, _: u_char) -> ::core::ffi::c_int;
    fn args_get(_: *mut args, _: u_char) -> *const ::core::ffi::c_char;
    fn args_strtonum_and_expand(
        _: *mut args,
        _: u_char,
        _: ::core::ffi::c_longlong,
        _: ::core::ffi::c_longlong,
        _: *mut cmdq_item,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn cmd_get_entry(_: *mut cmd) -> *const cmd_entry;
    fn cmd_get_args(_: *mut cmd) -> *mut args;
    fn cmdq_get_client(_: *mut cmdq_item) -> *mut client;
    fn cmdq_get_target(_: *mut cmdq_item) -> *mut cmd_find_state;
    fn cmdq_error(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    fn file_can_print(_: *mut client) -> ::core::ffi::c_int;
    fn file_print(_: *mut client, _: *const ::core::ffi::c_char, ...);
    fn file_print_buffer(_: *mut client, _: *mut ::core::ffi::c_void, _: size_t);
    fn server_redraw_window(_: *mut window);
    fn input_pending(_: *mut input_ctx) -> *mut evbuffer;
    fn colour_tostring(_: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn grid_line_flags_string(_: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn grid_cell_flags_string(_: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn grid_cell_attr_string(_: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn grid_line_time(_: *const grid_line) -> time_t;
    fn grid_clear_history(_: *mut grid);
    fn grid_peek_line(_: *mut grid, _: u_int) -> *const grid_line;
    fn grid_get_cell(_: *mut grid, _: u_int, _: u_int, _: *mut grid_cell);
    fn grid_get_line(_: *mut grid, _: u_int) -> *mut grid_line;
    fn grid_string_cells(
        _: *mut grid,
        _: u_int,
        _: u_int,
        _: u_int,
        _: *mut *mut grid_cell,
        _: ::core::ffi::c_int,
        _: *mut screen,
    ) -> *mut ::core::ffi::c_char;
    fn screen_reset_hyperlinks(_: *mut screen);
    fn window_pane_reset_mode_all(_: *mut window_pane);
    fn control_write(_: *mut client, _: *const ::core::ffi::c_char, ...);
    fn utf8_stravis(
        _: *mut *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
    ) -> size_t;
    fn hyperlinks_get(
        _: *mut hyperlinks,
        _: u_int,
        _: *mut *const ::core::ffi::c_char,
        _: *mut *const ::core::ffi::c_char,
        _: *mut *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
}

pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;
pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;

#[no_mangle]
pub static mut cmd_capture_pane_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"capture-pane\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"capturep\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"ab:CeE:FHIJLMNpPqRS:Tt:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage:
            b"[-aCeFHIJLMNpPqRT] [-b buffer-name] [-E end-line] [-S start-line] [-t target-pane]\0"
                as *const u8 as *const ::core::ffi::c_char,
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: CMD_AFTERHOOK,
        exec: Some(
            cmd_capture_pane_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
#[no_mangle]
pub static mut cmd_clear_history_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"clear-history\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"clearhist\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"Ht:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-H] [-t target-pane]\0" as *const u8 as *const ::core::ffi::c_char,
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: CMD_AFTERHOOK,
        exec: Some(
            cmd_capture_pane_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_capture_pane_append(
    mut buf: *mut ::core::ffi::c_char,
    mut len: *mut size_t,
    mut line: *const ::core::ffi::c_char,
    mut linelen: size_t,
) -> *mut ::core::ffi::c_char {
    buf = xrealloc(
        buf as *mut ::core::ffi::c_void,
        (*len).wrapping_add(linelen).wrapping_add(1 as size_t),
    ) as *mut ::core::ffi::c_char;
    memcpy(
        buf.offset(*len as isize) as *mut ::core::ffi::c_void,
        line as *const ::core::ffi::c_void,
        linelen,
    );
    *len = (*len).wrapping_add(linelen);
    return buf;
}
unsafe extern "C" fn cmd_capture_pane_cell(
    mut s: *mut screen,
    mut xx: u_int,
    mut yy: u_int,
) -> *mut ::core::ffi::c_char {
    let mut gd: *mut grid = (*s).grid;
    let mut hl: *mut hyperlinks = (*s).hyperlinks;
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
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut data: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut link: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut linkid: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut f: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut b: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut u: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut c: [::core::ffi::c_char; 33] = [0; 33];
    let mut uri: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut iid: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut flags: u_int = 0;
    grid_get_cell(gd, xx, yy, &raw mut gc);
    memcpy(
        &raw mut c as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
        &raw mut gc.data.data as *mut u_char as *const ::core::ffi::c_void,
        gc.data.size as size_t,
    );
    c[gc.data.size as usize] = '\0' as i32 as ::core::ffi::c_char;
    utf8_stravis(
        &raw mut data,
        &raw mut c as *mut ::core::ffi::c_char,
        VIS_OCTAL | VIS_CSTYLE | VIS_TAB | VIS_NL,
    );
    if gc.link != 0 as u_int
        && hyperlinks_get(
            hl,
            gc.link,
            &raw mut uri,
            &raw mut iid,
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
        ) != 0
    {
        xasprintf(
            &raw mut link,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            uri,
        );
        if !iid.is_null() && *iid as ::core::ffi::c_int != '\0' as i32 {
            xasprintf(
                &raw mut linkid,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                iid,
            );
        } else {
            xasprintf(
                &raw mut linkid,
                b"NONE\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
    } else {
        xasprintf(
            &raw mut link,
            b"NONE\0" as *const u8 as *const ::core::ffi::c_char,
        );
        xasprintf(
            &raw mut linkid,
            b"NONE\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    flags = gc.flags as u_int;
    if gc.fg & COLOUR_FLAG_256 != 0 {
        flags |= GRID_FLAG_FG256 as u_int;
    }
    if gc.bg & COLOUR_FLAG_256 != 0 {
        flags |= GRID_FLAG_BG256 as u_int;
    }
    xasprintf(
        &raw mut f,
        b"%s[%x]\0" as *const u8 as *const ::core::ffi::c_char,
        colour_tostring(gc.fg),
        gc.fg,
    );
    xasprintf(
        &raw mut b,
        b"%s[%x]\0" as *const u8 as *const ::core::ffi::c_char,
        colour_tostring(gc.bg),
        gc.bg,
    );
    xasprintf(
        &raw mut u,
        b"%s[%x]\0" as *const u8 as *const ::core::ffi::c_char,
        colour_tostring(gc.us),
        gc.us,
    );
    xasprintf(
        &raw mut line,
        b"\t\tC %u,%u data=(%u,%u,%s) flags=%s[%x] attr=%s[%x] fg=%s bg=%s us=%s link=%s linkid=%s\n\0"
            as *const u8 as *const ::core::ffi::c_char,
        yy,
        xx,
        gc.data.width as ::core::ffi::c_int,
        gc.data.size as ::core::ffi::c_int,
        data,
        grid_cell_flags_string(flags as ::core::ffi::c_int),
        flags,
        grid_cell_attr_string(gc.attr as ::core::ffi::c_int),
        gc.attr as ::core::ffi::c_int,
        f,
        b,
        u,
        link,
        linkid,
    );
    free(f as *mut ::core::ffi::c_void);
    free(b as *mut ::core::ffi::c_void);
    free(u as *mut ::core::ffi::c_void);
    free(link as *mut ::core::ffi::c_void);
    free(linkid as *mut ::core::ffi::c_void);
    free(data as *mut ::core::ffi::c_void);
    return line;
}
unsafe extern "C" fn cmd_capture_pane_grid(
    mut wp: *mut window_pane,
    mut len: *mut size_t,
) -> *mut ::core::ffi::c_char {
    let mut s: *mut screen = &raw mut (*wp).base;
    let mut gd: *mut grid = (*s).grid;
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut od: *mut osc133_data = ::core::ptr::null_mut::<osc133_data>();
    let mut buf: *mut ::core::ffi::c_char =
        xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut p: [::core::ffi::c_char; 11] = [0; 11];
    let mut yy: u_int = 0;
    let mut xx: u_int = 0;
    let mut total: u_int = (*gd).hsize.wrapping_add((*gd).sy);
    xasprintf(
        &raw mut line,
        b"G %ux%u (%u/%u)\n\0" as *const u8 as *const ::core::ffi::c_char,
        (*gd).sx,
        (*gd).sy,
        (*gd).hsize,
        (*gd).hlimit,
    );
    buf = cmd_capture_pane_append(buf, len, line, strlen(line));
    free(line as *mut ::core::ffi::c_void);
    yy = 0 as u_int;
    while yy < total {
        gl = grid_get_line(gd, yy);
        if yy < (*gd).hsize {
            snprintf(
                &raw mut p as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 11]>() as size_t,
                b"-\0" as *const u8 as *const ::core::ffi::c_char,
            );
        } else {
            snprintf(
                &raw mut p as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 11]>() as size_t,
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                yy.wrapping_sub((*gd).hsize),
            );
        }
        od = &raw mut (*gl).osc133_data;
        if (*gl).flags as ::core::ffi::c_int & GRID_LINE_OSC133_FLAGS != 0 {
            xasprintf(
                &raw mut line,
                b"\tL %u (%s) flags=%s[%x] %u/%u osc133=%u,%u,%u,%u,%u\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                yy,
                &raw mut p as *mut ::core::ffi::c_char,
                grid_line_flags_string((*gl).flags as ::core::ffi::c_int),
                (*gl).flags as ::core::ffi::c_int,
                (*gl).cellused as ::core::ffi::c_int,
                (*gl).cellsize as ::core::ffi::c_int,
                (*od).prompt_col as ::core::ffi::c_int,
                (*od).cmd_col as ::core::ffi::c_int,
                (*od).out_start_col as ::core::ffi::c_int,
                (*od).out_end_col as ::core::ffi::c_int,
                (*od).exit_status as ::core::ffi::c_int,
            );
        } else {
            xasprintf(
                &raw mut line,
                b"\tL %u (%s) flags=%s[%x] %u/%u\n\0" as *const u8 as *const ::core::ffi::c_char,
                yy,
                &raw mut p as *mut ::core::ffi::c_char,
                grid_line_flags_string((*gl).flags as ::core::ffi::c_int),
                (*gl).flags as ::core::ffi::c_int,
                (*gl).cellused as ::core::ffi::c_int,
                (*gl).cellsize as ::core::ffi::c_int,
            );
        }
        buf = cmd_capture_pane_append(buf, len, line, strlen(line));
        free(line as *mut ::core::ffi::c_void);
        xx = 0 as u_int;
        while xx < (*gd).sx {
            line = cmd_capture_pane_cell(s, xx, yy);
            buf = cmd_capture_pane_append(buf, len, line, strlen(line));
            free(line as *mut ::core::ffi::c_void);
            xx = xx.wrapping_add(1);
        }
        yy = yy.wrapping_add(1);
    }
    return buf;
}
unsafe extern "C" fn cmd_capture_pane_pending(
    mut args: *mut args,
    mut wp: *mut window_pane,
    mut len: *mut size_t,
) -> *mut ::core::ffi::c_char {
    let mut pending: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut tmp: [::core::ffi::c_char; 5] = [0; 5];
    let mut linelen: size_t = 0;
    let mut i: u_int = 0;
    pending = input_pending((*wp).ictx);
    if pending.is_null() {
        return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    }
    line =
        evbuffer_pullup(pending, -(1 as ::core::ffi::c_int) as ssize_t) as *mut ::core::ffi::c_char;
    linelen = evbuffer_get_length(pending);
    buf = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    if args_has(args, 'C' as i32 as u_char) != 0 {
        i = 0 as u_int;
        while (i as size_t) < linelen {
            if *line.offset(i as isize) as ::core::ffi::c_int >= ' ' as i32
                && *line.offset(i as isize) as ::core::ffi::c_int != '\\' as i32
            {
                tmp[0 as ::core::ffi::c_int as usize] = *line.offset(i as isize);
                tmp[1 as ::core::ffi::c_int as usize] = '\0' as i32 as ::core::ffi::c_char;
            } else {
                snprintf(
                    &raw mut tmp as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 5]>() as size_t,
                    b"\\%03hho\0" as *const u8 as *const ::core::ffi::c_char,
                    *line.offset(i as isize) as ::core::ffi::c_int,
                );
            }
            buf = cmd_capture_pane_append(
                buf,
                len,
                &raw mut tmp as *mut ::core::ffi::c_char,
                strlen(&raw mut tmp as *mut ::core::ffi::c_char),
            );
            i = i.wrapping_add(1);
        }
    } else {
        buf = cmd_capture_pane_append(buf, len, line, linelen);
    }
    return buf;
}
unsafe extern "C" fn cmd_capture_pane_hyperlinks(
    mut gd: *mut grid,
    mut s: *mut screen,
    mut py: u_int,
    mut links: *mut u_int,
    mut nlinks: *mut u_int,
    mut len: *mut size_t,
) -> *mut ::core::ffi::c_char {
    let mut gl: *const grid_line = grid_peek_line(gd, py);
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
    let mut uri: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut line: *mut ::core::ffi::c_char =
        xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    let mut i: u_int = 0;
    let mut j: u_int = 0;
    *len = 0 as size_t;
    if (*s).hyperlinks.is_null() || !((*gl).flags as ::core::ffi::c_int) & GRID_LINE_HYPERLINK != 0
    {
        return line;
    }
    i = 0 as u_int;
    while i < (*gl).cellused as u_int {
        grid_get_cell(gd, i, py, &raw mut gc);
        if !(gc.link == 0 as u_int) {
            j = 0 as u_int;
            while j < *nlinks {
                if *links.offset(j as isize) == gc.link {
                    break;
                }
                j = j.wrapping_add(1);
            }
            if !(j != *nlinks) {
                if !(hyperlinks_get(
                    (*s).hyperlinks,
                    gc.link,
                    &raw mut uri,
                    ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
                    ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
                ) == 0)
                {
                    if *nlinks == (*gd).sx {
                        break;
                    }
                    let fresh9 = *nlinks;
                    *nlinks = (*nlinks).wrapping_add(1);
                    *links.offset(fresh9 as isize) = gc.link;
                    if *len != 0 as size_t {
                        line = cmd_capture_pane_append(
                            line,
                            len,
                            b" \0" as *const u8 as *const ::core::ffi::c_char,
                            1 as size_t,
                        );
                    }
                    line = cmd_capture_pane_append(line, len, uri, strlen(uri));
                }
            }
        }
        i = i.wrapping_add(1);
    }
    return line;
}
unsafe extern "C" fn cmd_capture_pane_history(
    mut args: *mut args,
    mut item: *mut cmdq_item,
    mut wp: *mut window_pane,
    mut len: *mut size_t,
) -> *mut ::core::ffi::c_char {
    let mut gd: *mut grid = ::core::ptr::null_mut::<grid>();
    let mut gl: *const grid_line = ::core::ptr::null::<grid_line>();
    let mut s: *mut screen = ::core::ptr::null_mut::<screen>();
    let mut gc: *mut grid_cell = ::core::ptr::null_mut::<grid_cell>();
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    let mut n: ::core::ffi::c_int = 0;
    let mut join_lines: ::core::ffi::c_int = 0;
    let mut number_lines: ::core::ffi::c_int = 0;
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut show_flags: ::core::ffi::c_int = 0;
    let mut show_time: ::core::ffi::c_int = 0;
    let mut hyperlinks: ::core::ffi::c_int = 0;
    let mut links: *mut u_int = ::core::ptr::null_mut::<u_int>();
    let mut nlinks: u_int = 0 as u_int;
    let mut i: u_int = 0;
    let mut sx: u_int = 0;
    let mut top: u_int = 0;
    let mut bottom: u_int = 0;
    let mut tmp: u_int = 0;
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut b: [::core::ffi::c_char; 64] = [0; 64];
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut Sflag: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut Eflag: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut linelen: size_t = 0;
    sx = (*(*wp).base.grid).sx;
    if args_has(args, 'a' as i32 as u_char) != 0 {
        gd = (*wp).base.saved_grid;
        if gd.is_null() {
            if args_has(args, 'q' as i32 as u_char) == 0 {
                cmdq_error(
                    item,
                    b"no alternate screen\0" as *const u8 as *const ::core::ffi::c_char,
                );
                return ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
        }
        s = &raw mut (*wp).base;
    } else if args_has(args, 'M' as i32 as u_char) != 0 {
        wme = (*wp).modes.tqh_first;
        if !wme.is_null() && (*(*wme).mode).get_screen.is_some() {
            s = (*(*wme).mode)
                .get_screen
                .expect("non-null function pointer")(wme);
            gd = (*s).grid;
        } else {
            s = &raw mut (*wp).base;
            gd = (*wp).base.grid;
        }
    } else {
        s = &raw mut (*wp).base;
        gd = (*wp).base.grid;
    }
    Sflag = args_get(args, 'S' as i32 as u_char);
    if !Sflag.is_null()
        && strcmp(Sflag, b"-\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
    {
        top = 0 as u_int;
    } else {
        n = args_strtonum_and_expand(
            args,
            'S' as i32 as u_char,
            INT_MIN as ::core::ffi::c_longlong,
            SHRT_MAX as ::core::ffi::c_longlong,
            item,
            &raw mut cause,
        ) as ::core::ffi::c_int;
        if !cause.is_null() {
            top = (*gd).hsize;
            free(cause as *mut ::core::ffi::c_void);
        } else if n < 0 as ::core::ffi::c_int && -n as u_int > (*gd).hsize {
            top = 0 as u_int;
        } else {
            top = (*gd).hsize.wrapping_add(n as u_int);
        }
        if top > (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(1 as u_int) {
            top = (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(1 as u_int);
        }
    }
    Eflag = args_get(args, 'E' as i32 as u_char);
    if !Eflag.is_null()
        && strcmp(Eflag, b"-\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
    {
        bottom = (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(1 as u_int);
    } else {
        n = args_strtonum_and_expand(
            args,
            'E' as i32 as u_char,
            INT_MIN as ::core::ffi::c_longlong,
            SHRT_MAX as ::core::ffi::c_longlong,
            item,
            &raw mut cause,
        ) as ::core::ffi::c_int;
        if !cause.is_null() {
            bottom = (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(1 as u_int);
            free(cause as *mut ::core::ffi::c_void);
        } else if n < 0 as ::core::ffi::c_int && -n as u_int > (*gd).hsize {
            bottom = 0 as u_int;
        } else {
            bottom = (*gd).hsize.wrapping_add(n as u_int);
        }
        if bottom > (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(1 as u_int) {
            bottom = (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(1 as u_int);
        }
    }
    if bottom < top {
        tmp = bottom;
        bottom = top;
        top = tmp;
    }
    join_lines = args_has(args, 'J' as i32 as u_char);
    if args_has(args, 'e' as i32 as u_char) != 0 {
        flags |= GRID_STRING_WITH_SEQUENCES;
    }
    if args_has(args, 'C' as i32 as u_char) != 0 {
        flags |= GRID_STRING_ESCAPE_SEQUENCES;
    }
    if join_lines == 0 && args_has(args, 'T' as i32 as u_char) == 0 {
        flags |= GRID_STRING_EMPTY_CELLS;
    }
    if join_lines == 0 && args_has(args, 'N' as i32 as u_char) == 0 {
        flags |= GRID_STRING_TRIM_SPACES;
    }
    number_lines = args_has(args, 'L' as i32 as u_char);
    show_flags = args_has(args, 'F' as i32 as u_char);
    show_time = args_has(args, 'I' as i32 as u_char);
    hyperlinks = args_has(args, 'H' as i32 as u_char);
    if hyperlinks != 0 {
        links = xreallocarray(
            NULL,
            (*gd).sx as size_t,
            ::core::mem::size_of::<u_int>() as size_t,
        ) as *mut u_int;
    }
    i = top;
    while i <= bottom {
        if hyperlinks != 0 {
            line = cmd_capture_pane_hyperlinks(gd, s, i, links, &raw mut nlinks, &raw mut linelen);
        } else {
            line = grid_string_cells(gd, 0 as u_int, i, sx, &raw mut gc, flags, s);
            linelen = strlen(line);
        }
        if hyperlinks != 0 && linelen == 0 as size_t {
            free(line as *mut ::core::ffi::c_void);
        } else {
            gl = grid_peek_line(gd, i);
            if number_lines != 0 {
                if i >= (*gd).hsize {
                    n = i.wrapping_sub((*gd).hsize) as ::core::ffi::c_int;
                } else {
                    n = i as ::core::ffi::c_int - (*gd).hsize as ::core::ffi::c_int;
                }
                n = snprintf(
                    &raw mut b as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as size_t,
                    b"%d \0" as *const u8 as *const ::core::ffi::c_char,
                    n,
                );
                if n >= 0 as ::core::ffi::c_int {
                    buf = cmd_capture_pane_append(
                        buf,
                        len,
                        &raw mut b as *mut ::core::ffi::c_char,
                        n as size_t,
                    );
                }
            }
            if show_time != 0 {
                n = snprintf(
                    &raw mut b as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as size_t,
                    b"%llu \0" as *const u8 as *const ::core::ffi::c_char,
                    grid_line_time(gl) as ::core::ffi::c_ulonglong,
                );
                if n >= 0 as ::core::ffi::c_int {
                    buf = cmd_capture_pane_append(
                        buf,
                        len,
                        &raw mut b as *mut ::core::ffi::c_char,
                        n as size_t,
                    );
                }
            }
            if show_flags != 0 {
                cp = &raw mut b as *mut ::core::ffi::c_char;
                *cp = '\0' as i32 as ::core::ffi::c_char;
                if (*gl).flags as ::core::ffi::c_int & GRID_LINE_DEAD != 0 {
                    let fresh0 = cp;
                    cp = cp.offset(1);
                    *fresh0 = 'D' as i32 as ::core::ffi::c_char;
                }
                if (*gl).flags as ::core::ffi::c_int & GRID_LINE_HYPERLINK != 0 {
                    let fresh1 = cp;
                    cp = cp.offset(1);
                    *fresh1 = 'H' as i32 as ::core::ffi::c_char;
                }
                if (*gl).flags as ::core::ffi::c_int & GRID_LINE_START_OUTPUT != 0 {
                    let fresh2 = cp;
                    cp = cp.offset(1);
                    *fresh2 = 'O' as i32 as ::core::ffi::c_char;
                }
                if (*gl).flags as ::core::ffi::c_int & GRID_LINE_START_PROMPT != 0 {
                    let fresh3 = cp;
                    cp = cp.offset(1);
                    *fresh3 = 'P' as i32 as ::core::ffi::c_char;
                }
                if (*gl).flags as ::core::ffi::c_int & GRID_LINE_WRAPPED != 0 {
                    let fresh4 = cp;
                    cp = cp.offset(1);
                    *fresh4 = 'W' as i32 as ::core::ffi::c_char;
                }
                if (*gl).flags as ::core::ffi::c_int & GRID_LINE_EXTENDED != 0 {
                    let fresh5 = cp;
                    cp = cp.offset(1);
                    *fresh5 = 'X' as i32 as ::core::ffi::c_char;
                }
                if &raw mut b as *mut ::core::ffi::c_char == cp {
                    let fresh6 = cp;
                    cp = cp.offset(1);
                    *fresh6 = '-' as i32 as ::core::ffi::c_char;
                }
                let fresh7 = cp;
                cp = cp.offset(1);
                *fresh7 = ' ' as i32 as ::core::ffi::c_char;
                *cp = '\0' as i32 as ::core::ffi::c_char;
                buf = cmd_capture_pane_append(
                    buf,
                    len,
                    &raw mut b as *mut ::core::ffi::c_char,
                    strlen(&raw mut b as *mut ::core::ffi::c_char),
                );
            }
            buf = cmd_capture_pane_append(buf, len, line, linelen);
            if join_lines == 0 || (*gl).flags as ::core::ffi::c_int & GRID_LINE_WRAPPED == 0 {
                let fresh8 = *len;
                *len = (*len).wrapping_add(1);
                *buf.offset(fresh8 as isize) = '\n' as i32 as ::core::ffi::c_char;
            }
            free(line as *mut ::core::ffi::c_void);
        }
        i = i.wrapping_add(1);
    }
    free(links as *mut ::core::ffi::c_void);
    if buf.is_null() {
        buf = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    }
    return buf;
}
unsafe extern "C" fn cmd_capture_pane_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut c: *mut client = cmdq_get_client(item);
    let mut wp: *mut window_pane = (*cmdq_get_target(item)).wp;
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut bufname: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut len: size_t = 0;
    if cmd_get_entry(self_0) == &raw const cmd_clear_history_entry {
        window_pane_reset_mode_all(wp);
        grid_clear_history((*wp).base.grid);
        if args_has(args, 'H' as i32 as u_char) != 0 {
            screen_reset_hyperlinks((*wp).screen);
        }
        server_redraw_window((*wp).window as *mut window);
        return CMD_RETURN_NORMAL;
    }
    len = 0 as size_t;
    if args_has(args, 'R' as i32 as u_char) != 0 {
        buf = cmd_capture_pane_grid(wp, &raw mut len);
    } else if args_has(args, 'P' as i32 as u_char) != 0 && args_has(args, 'H' as i32 as u_char) == 0
    {
        buf = cmd_capture_pane_pending(args, wp, &raw mut len);
    } else {
        buf = cmd_capture_pane_history(args, item, wp, &raw mut len);
    }
    if buf.is_null() {
        return CMD_RETURN_ERROR;
    }
    if args_has(args, 'p' as i32 as u_char) != 0 {
        if len > 0 as size_t
            && *buf.offset(len.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                == '\n' as i32
        {
            len = len.wrapping_sub(1);
        }
        if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            control_write(
                c,
                b"%.*s\0" as *const u8 as *const ::core::ffi::c_char,
                len as ::core::ffi::c_int,
                buf,
            );
        } else {
            if file_can_print(c) == 0 {
                cmdq_error(
                    item,
                    b"can't write to client\0" as *const u8 as *const ::core::ffi::c_char,
                );
                free(buf as *mut ::core::ffi::c_void);
                return CMD_RETURN_ERROR;
            }
            file_print_buffer(c, buf as *mut ::core::ffi::c_void, len);
            file_print(c, b"\n\0" as *const u8 as *const ::core::ffi::c_char);
        }
        free(buf as *mut ::core::ffi::c_void);
    } else {
        bufname = ::core::ptr::null::<::core::ffi::c_char>();
        if args_has(args, 'b' as i32 as u_char) != 0 {
            bufname = args_get(args, 'b' as i32 as u_char);
        }
        if paste_set(buf, len, bufname, &raw mut cause) != 0 as ::core::ffi::c_int {
            cmdq_error(
                item,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                cause,
            );
            free(cause as *mut ::core::ffi::c_void);
            free(buf as *mut ::core::ffi::c_void);
            return CMD_RETURN_ERROR;
        }
    }
    return CMD_RETURN_NORMAL;
}
