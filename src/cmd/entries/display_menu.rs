use crate::src::arguments::{
    args_count, args_first_value, args_get, args_has, args_next_value, args_percentage_result,
    args_string, args_strtonum_result, args_to_vector,
};
use crate::src::cmd::{cmd_append_argv, cmd_get_args};
use crate::src::cmd_queue::{cmdq_error, cmdq_get_event, cmdq_get_target, cmdq_get_target_client};
use crate::src::environ::{environ_create, environ_free, environ_put};
use crate::src::ffi::libc::{free, strcmp, strtol};
use crate::src::format::{
    format_add, format_create_from_target, format_expand_cstring, format_free,
    format_single_from_target_cstring,
};
use crate::src::key_string::key_string_parse_cstr;
use crate::src::log::log_debug;
use crate::src::menu::{menu_add_item, menu_create, menu_display, menu_free};
pub use crate::src::options::options_table_entry;
use crate::src::options::{
    options_find_choice, options_get, options_get_number, options_get_string,
};
use crate::src::popup::{popup_display, popup_modify, popup_present};
use crate::src::server_client::{server_client_clear_overlay, server_client_get_cwd};
use crate::src::shared::abi::*;
use crate::src::shared::arguments::*;
pub use crate::src::shared::arguments::{
    args, args_parse, args_parse_cb, args_value, args_value_c2rust_unnamed, args_value_entry,
};
pub use crate::src::shared::client::CLIENT_CONTROL;
use crate::src::shared::client::*;
pub use crate::src::shared::client::{
    client, client_file, client_file_cb, client_file_entry, client_files, overlay_check_cb,
    overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb, overlay_resize_cb,
};
use crate::src::shared::colour::*;
use crate::src::shared::command::*;
pub use crate::src::shared::command::{
    cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds,
};
pub use crate::src::shared::command::{CMD_AFTERHOOK, CMD_CLIENT_CFLAG};
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
pub use crate::src::shared::limits::{__INT_MAX__, UINT_MAX};
pub use crate::src::shared::menu::{menu, menu_item, MENU_NOMOUSE, MENU_STAYOPEN};
pub use crate::src::shared::menu::{menu_choice_cb, menu_data};
use crate::src::shared::message::*;
pub use crate::src::shared::mouse::mouse_event;
use crate::src::shared::options::*;
pub use crate::src::shared::options::{options, options_entry};
pub use crate::src::shared::pane::{
    window_pane, window_pane_entry, window_pane_modes, window_pane_prompt, window_pane_sentry,
    window_pane_tree_entry, window_pane_zentry, window_panes,
};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::popup::popup_close_cb;
pub use crate::src::shared::popup::{POPUP_CLOSEANYKEY, POPUP_CLOSEEXIT, POPUP_CLOSEEXITZERO};
pub use crate::src::shared::posix_io::_PATH_BSHELL;
pub use crate::src::shared::process::tmuxpeer;
pub use crate::src::shared::prompt::prompt;
pub use crate::src::shared::redraw::redraw_scene;
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::screen_write_cline;
pub use crate::src::shared::session::{session, session_entry, session_gentry};
pub use crate::src::shared::spawn::spawn_editor_state;
pub use crate::src::shared::status::status_line;
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
use crate::src::status::{status_at_line, status_line_size};
use crate::src::tmux::checkshell;
use crate::src::tty::tty_window_offset;

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

#[no_mangle]
pub static mut cmd_display_menu_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"display-menu\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"menu\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"b:c:C:H:s:S:MOt:T:x:y:\0" as *const u8
                as *const ::core::ffi::c_char,
            lower: 1 as ::core::ffi::c_int,
            upper: -(1 as ::core::ffi::c_int),
            cb: Some(
                cmd_display_menu_args_parse
                    as unsafe extern "C" fn(
                        *mut args,
                        u_int,
                        *mut *mut ::core::ffi::c_char,
                    ) -> args_parse_type,
            ),
        },
        usage: b"[-MO] [-b border-lines] [-c target-client] [-C starting-choice] [-H selected-style] [-s style] [-S border-style] [-t target-pane] [-T title] [-x position] [-y position] name [key] [command] ...\0"
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
        flags: CMD_AFTERHOOK | CMD_CLIENT_CFLAG,
        exec: Some(
            cmd_display_menu_exec
                as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
#[no_mangle]
pub static mut cmd_display_popup_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"display-popup\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"popup\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"Bb:Cc:d:e:Eh:kNs:S:t:T:w:x:y:\0" as *const u8
                as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: -(1 as ::core::ffi::c_int),
            cb: None,
        },
        usage: b"[-BCEkN] [-b border-lines] [-c target-client] [-d start-directory] [-e environment] [-h height] [-s style] [-S border-style] [-t target-pane] [-T title] [-w width] [-x position] [-y position] [shell-command [argument ...]]\0"
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
        flags: CMD_AFTERHOOK | CMD_CLIENT_CFLAG,
        exec: Some(
            cmd_display_popup_exec
                as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_display_menu_args_parse(
    mut args: *mut args,
    mut idx: u_int,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> args_parse_type {
    let mut i: u_int = 0 as u_int;
    let mut type_0: args_parse_type = ARGS_PARSE_STRING;
    loop {
        type_0 = ARGS_PARSE_STRING;
        if i == idx {
            break;
        }
        let fresh0 = i;
        i = i.wrapping_add(1);
        if *args_string(args, fresh0) as ::core::ffi::c_int == '\0' as i32 {
            continue;
        }
        type_0 = ARGS_PARSE_STRING;
        let fresh1 = i;
        i = i.wrapping_add(1);
        if fresh1 == idx {
            break;
        }
        type_0 = ARGS_PARSE_COMMANDS_OR_STRING;
        let fresh2 = i;
        i = i.wrapping_add(1);
        if fresh2 == idx {
            break;
        }
    }
    return type_0;
}
unsafe extern "C" fn cmd_display_menu_get_popup_pos(
    mut tc: *mut client,
    mut item: *mut cmdq_item,
    mut args: *mut args,
    mut px: *mut u_int,
    mut py: *mut u_int,
    mut w: u_int,
    mut h: u_int,
) -> ::core::ffi::c_int {
    let mut tty: *mut tty = &raw mut (*tc).tty;
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut event: *mut key_event = cmdq_get_event(item);
    let mut s: *mut session = (*tc).session;
    let mut wl: *mut winlink = (*target).wl;
    let mut wp: *mut window_pane = (*target).wp;
    let mut ranges: *mut style_ranges = ::core::ptr::null_mut::<style_ranges>();
    let mut sr: *mut style_range = ::core::ptr::null_mut::<style_range>();
    let mut xp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut yp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut top: ::core::ffi::c_int = 0;
    let mut line: u_int = 0;
    let mut ox: u_int = 0;
    let mut oy: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut lines: u_int = 0;
    let mut position: u_int = 0;
    let mut n: ::core::ffi::c_long = 0;
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    if w > (*tty).sx || h > (*tty).sy {
        return 0 as ::core::ffi::c_int;
    }
    ft = format_create_from_target(item);
    if (*event).m.valid != 0 {
        format_add(
            ft,
            b"popup_mouse_x\0" as *const u8 as *const ::core::ffi::c_char,
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*event).m.x,
        );
        format_add(
            ft,
            b"popup_mouse_y\0" as *const u8 as *const ::core::ffi::c_char,
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*event).m.y,
        );
    }
    format_add(
        ft,
        b"popup_last_x\0" as *const u8 as *const ::core::ffi::c_char,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        (*(*target).w).menu_last_px,
    );
    format_add(
        ft,
        b"popup_last_y\0" as *const u8 as *const ::core::ffi::c_char,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        (*(*target).w).menu_last_py.wrapping_add(h),
    );
    top = status_at_line(tc);
    if top != -(1 as ::core::ffi::c_int) {
        lines = status_line_size(tc);
        if top == 0 as ::core::ffi::c_int {
            top = lines as ::core::ffi::c_int;
        } else {
            top = 0 as ::core::ffi::c_int;
        }
        position = options_get_number(
            (*s).options,
            b"status-position\0" as *const u8 as *const ::core::ffi::c_char,
        ) as u_int;
        line = 0 as u_int;
        while line < lines {
            ranges = &raw mut (*(&raw mut (*tc).status.entries as *mut style_line_entry)
                .offset(line as isize))
            .ranges;
            sr = ::core::ptr::null_mut::<style_range>();
            for range in (*ranges).as_slice() {
                let candidate = range.as_ref();
                if !((*candidate).type_0 as ::core::ffi::c_uint
                    != STYLE_RANGE_WINDOW as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    if (*candidate).argument == (*wl).idx as u_int {
                        sr = candidate as *const style_range as *mut style_range;
                        break;
                    }
                }
            }
            if !sr.is_null() {
                break;
            }
            line = line.wrapping_add(1);
        }
        if !sr.is_null() {
            format_add(
                ft,
                b"popup_window_status_line_x\0" as *const u8 as *const ::core::ffi::c_char,
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*sr).start,
            );
            if position == 0 as u_int {
                format_add(
                    ft,
                    b"popup_window_status_line_y\0" as *const u8 as *const ::core::ffi::c_char,
                    b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                    line.wrapping_add(1 as u_int).wrapping_add(h),
                );
            } else {
                format_add(
                    ft,
                    b"popup_window_status_line_y\0" as *const u8 as *const ::core::ffi::c_char,
                    b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                    (*tty).sy.wrapping_sub(lines).wrapping_add(line),
                );
            }
        }
        if position == 0 as u_int {
            format_add(
                ft,
                b"popup_status_line_y\0" as *const u8 as *const ::core::ffi::c_char,
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                lines.wrapping_add(h),
            );
        } else {
            format_add(
                ft,
                b"popup_status_line_y\0" as *const u8 as *const ::core::ffi::c_char,
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*tty).sy.wrapping_sub(lines),
            );
        }
    } else {
        top = 0 as ::core::ffi::c_int;
    }
    format_add(
        ft,
        b"popup_width\0" as *const u8 as *const ::core::ffi::c_char,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
    format_add(
        ft,
        b"popup_height\0" as *const u8 as *const ::core::ffi::c_char,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        h,
    );
    n = (*tty).sx.wrapping_sub(1 as u_int) as ::core::ffi::c_long / 2 as ::core::ffi::c_long
        - w.wrapping_div(2 as u_int) as ::core::ffi::c_long;
    if n < 0 as ::core::ffi::c_long {
        format_add(
            ft,
            b"popup_centre_x\0" as *const u8 as *const ::core::ffi::c_char,
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
        );
    } else {
        format_add(
            ft,
            b"popup_centre_x\0" as *const u8 as *const ::core::ffi::c_char,
            b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
            n,
        );
    }
    n = (*tty)
        .sy
        .wrapping_sub(1 as u_int)
        .wrapping_div(2 as u_int)
        .wrapping_add(h.wrapping_div(2 as u_int)) as ::core::ffi::c_long;
    if n >= (*tty).sy as ::core::ffi::c_long {
        format_add(
            ft,
            b"popup_centre_y\0" as *const u8 as *const ::core::ffi::c_char,
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*tty).sy.wrapping_sub(h),
        );
    } else {
        format_add(
            ft,
            b"popup_centre_y\0" as *const u8 as *const ::core::ffi::c_char,
            b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
            n,
        );
    }
    if (*event).m.valid != 0 {
        n = (*event).m.x as ::core::ffi::c_long - w.wrapping_div(2 as u_int) as ::core::ffi::c_long;
        if n < 0 as ::core::ffi::c_long {
            format_add(
                ft,
                b"popup_mouse_centre_x\0" as *const u8 as *const ::core::ffi::c_char,
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
            );
        } else {
            format_add(
                ft,
                b"popup_mouse_centre_x\0" as *const u8 as *const ::core::ffi::c_char,
                b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
                n,
            );
        }
        n = (*event).m.y.wrapping_sub(h.wrapping_div(2 as u_int)) as ::core::ffi::c_long;
        if n + h as ::core::ffi::c_long >= (*tty).sy as ::core::ffi::c_long {
            format_add(
                ft,
                b"popup_mouse_centre_y\0" as *const u8 as *const ::core::ffi::c_char,
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*tty).sy.wrapping_sub(h),
            );
        } else {
            format_add(
                ft,
                b"popup_mouse_centre_y\0" as *const u8 as *const ::core::ffi::c_char,
                b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
                n,
            );
        }
        n = (*event).m.y as ::core::ffi::c_long + h as ::core::ffi::c_long;
        if n >= (*tty).sy as ::core::ffi::c_long {
            format_add(
                ft,
                b"popup_mouse_top\0" as *const u8 as *const ::core::ffi::c_char,
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*tty).sy.wrapping_sub(1 as u_int),
            );
        } else {
            format_add(
                ft,
                b"popup_mouse_top\0" as *const u8 as *const ::core::ffi::c_char,
                b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
                n,
            );
        }
        n = (*event).m.y.wrapping_sub(h) as ::core::ffi::c_long;
        if n < 0 as ::core::ffi::c_long {
            format_add(
                ft,
                b"popup_mouse_bottom\0" as *const u8 as *const ::core::ffi::c_char,
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
            );
        } else {
            format_add(
                ft,
                b"popup_mouse_bottom\0" as *const u8 as *const ::core::ffi::c_char,
                b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
                n,
            );
        }
    }
    tty_window_offset(
        &raw mut (*tc).tty,
        &raw mut ox,
        &raw mut oy,
        &raw mut sx,
        &raw mut sy,
    );
    n = ((top + (*wp).yoff) as u_int)
        .wrapping_sub(oy)
        .wrapping_add(h) as ::core::ffi::c_long;
    if n >= (*tty).sy as ::core::ffi::c_long {
        format_add(
            ft,
            b"popup_pane_top\0" as *const u8 as *const ::core::ffi::c_char,
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*tty).sy.wrapping_sub(h),
        );
    } else {
        format_add(
            ft,
            b"popup_pane_top\0" as *const u8 as *const ::core::ffi::c_char,
            b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
            n,
        );
    }
    format_add(
        ft,
        b"popup_pane_bottom\0" as *const u8 as *const ::core::ffi::c_char,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        ((top + (*wp).yoff) as u_int)
            .wrapping_add((*wp).sy)
            .wrapping_sub(oy),
    );
    format_add(
        ft,
        b"popup_pane_left\0" as *const u8 as *const ::core::ffi::c_char,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        ((*wp).xoff as u_int).wrapping_sub(ox),
    );
    n = (*wp).xoff as ::core::ffi::c_long + (*wp).sx as ::core::ffi::c_long
        - ox as ::core::ffi::c_long
        - w as ::core::ffi::c_long;
    if n < 0 as ::core::ffi::c_long {
        format_add(
            ft,
            b"popup_pane_right\0" as *const u8 as *const ::core::ffi::c_char,
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
        );
    } else {
        format_add(
            ft,
            b"popup_pane_right\0" as *const u8 as *const ::core::ffi::c_char,
            b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
            n,
        );
    }
    xp = args_get(args, 'x' as i32 as u_char);
    if xp.is_null()
        || strcmp(xp, b"C\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
    {
        xp = b"#{popup_centre_x}\0" as *const u8 as *const ::core::ffi::c_char;
    } else if strcmp(xp, b"R\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        xp = b"#{popup_pane_right}\0" as *const u8 as *const ::core::ffi::c_char;
    } else if strcmp(xp, b"P\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        xp = b"#{popup_pane_left}\0" as *const u8 as *const ::core::ffi::c_char;
    } else if strcmp(xp, b"M\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        xp = b"#{popup_mouse_centre_x}\0" as *const u8 as *const ::core::ffi::c_char;
    } else if strcmp(xp, b"L\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        xp = b"#{popup_last_x}\0" as *const u8 as *const ::core::ffi::c_char;
    } else if strcmp(xp, b"W\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        xp = b"#{popup_window_status_line_x}\0" as *const u8 as *const ::core::ffi::c_char;
    }
    let p = format_expand_cstring(ft, xp);
    n = strtol(
        p.as_ptr(),
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        10 as ::core::ffi::c_int,
    );
    if n + w as ::core::ffi::c_long >= (*tty).sx as ::core::ffi::c_long {
        n = (*tty).sx.wrapping_sub(w) as ::core::ffi::c_long;
    } else if n < 0 as ::core::ffi::c_long {
        n = 0 as ::core::ffi::c_long;
    }
    *px = n as u_int;
    log_debug(
        b"%s: -x: %s = %s = %u (-w %u)\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_display_menu_get_popup_pos\0" as *const u8 as *const ::core::ffi::c_char,
        xp,
        p.as_ptr(),
        *px,
        w,
    );
    yp = args_get(args, 'y' as i32 as u_char);
    if yp.is_null()
        || strcmp(yp, b"C\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
    {
        yp = b"#{popup_centre_y}\0" as *const u8 as *const ::core::ffi::c_char;
    } else if strcmp(yp, b"P\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        yp = b"#{popup_pane_bottom}\0" as *const u8 as *const ::core::ffi::c_char;
    } else if strcmp(yp, b"M\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        yp = b"#{popup_mouse_top}\0" as *const u8 as *const ::core::ffi::c_char;
    } else if strcmp(yp, b"L\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        yp = b"#{popup_last_y}\0" as *const u8 as *const ::core::ffi::c_char;
    } else if strcmp(yp, b"S\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        yp = b"#{popup_status_line_y}\0" as *const u8 as *const ::core::ffi::c_char;
    } else if strcmp(yp, b"W\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        yp = b"#{popup_window_status_line_y}\0" as *const u8 as *const ::core::ffi::c_char;
    }
    let p = format_expand_cstring(ft, yp);
    n = strtol(
        p.as_ptr(),
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        10 as ::core::ffi::c_int,
    );
    if n < h as ::core::ffi::c_long {
        n = 0 as ::core::ffi::c_long;
    } else {
        n -= h as ::core::ffi::c_long;
    }
    if n + h as ::core::ffi::c_long >= (*tty).sy as ::core::ffi::c_long {
        n = (*tty).sy.wrapping_sub(h) as ::core::ffi::c_long;
    } else if n < 0 as ::core::ffi::c_long {
        n = 0 as ::core::ffi::c_long;
    }
    *py = n as u_int;
    log_debug(
        b"%s: -y: %s = %s = %u (-h %u)\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_display_menu_get_popup_pos\0" as *const u8 as *const ::core::ffi::c_char,
        yp,
        p.as_ptr(),
        *py,
        h,
    );
    format_free(ft);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn cmd_display_menu_get_menu_pos(
    mut tc: *mut client,
    mut item: *mut cmdq_item,
    mut args: *mut args,
    mut px: *mut u_int,
    mut py: *mut u_int,
    mut w: u_int,
    mut h: u_int,
) -> ::core::ffi::c_int {
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut event: *mut key_event = cmdq_get_event(item);
    let mut s: *mut session = (*tc).session;
    let mut wl: *mut winlink = (*target).wl;
    let mut window: *mut window = (*target).w;
    let mut wp: *mut window_pane = (*target).wp;
    let mut ranges: *mut style_ranges = ::core::ptr::null_mut::<style_ranges>();
    let mut sr: *mut style_range = ::core::ptr::null_mut::<style_range>();
    let mut xp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut yp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut line: u_int = 0;
    let mut ox: u_int = 0;
    let mut oy: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut lines: u_int = 0;
    let mut position: u_int = 0;
    let mut n: ::core::ffi::c_long = 0;
    let mut max_x: ::core::ffi::c_long = 0;
    let mut max_y: ::core::ffi::c_long = 0;
    let mut mouse_x: ::core::ffi::c_long = 0 as ::core::ffi::c_long;
    let mut mouse_y: ::core::ffi::c_long = 0 as ::core::ffi::c_long;
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    max_x = (if (*window).sx > w {
        (*window).sx.wrapping_sub(w)
    } else {
        0 as u_int
    }) as ::core::ffi::c_long;
    max_y = (if (*window).sy > h {
        (*window).sy.wrapping_sub(h)
    } else {
        0 as u_int
    }) as ::core::ffi::c_long;
    tty_window_offset(
        &raw mut (*tc).tty,
        &raw mut ox,
        &raw mut oy,
        &raw mut sx,
        &raw mut sy,
    );
    ft = format_create_from_target(item);
    if (*event).m.valid != 0 {
        mouse_x = (*event).m.x.wrapping_add(ox) as ::core::ffi::c_long;
        if (*event).m.statusat == 0 as ::core::ffi::c_int {
            if (*event).m.y >= (*event).m.statuslines {
                mouse_y = (*event)
                    .m
                    .y
                    .wrapping_sub((*event).m.statuslines)
                    .wrapping_add(oy) as ::core::ffi::c_long;
            } else {
                mouse_y = oy as ::core::ffi::c_long;
            }
        } else if (*event).m.statusat > 0 as ::core::ffi::c_int
            && (*event).m.y >= (*event).m.statusat as u_int
        {
            mouse_y = oy.wrapping_add(sy).wrapping_sub(1 as u_int) as ::core::ffi::c_long;
        } else {
            mouse_y = (*event).m.y.wrapping_add(oy) as ::core::ffi::c_long;
        }
        format_add(
            ft,
            b"popup_mouse_x\0" as *const u8 as *const ::core::ffi::c_char,
            b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
            mouse_x,
        );
        format_add(
            ft,
            b"popup_mouse_y\0" as *const u8 as *const ::core::ffi::c_char,
            b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
            mouse_y,
        );
    }
    format_add(
        ft,
        b"popup_last_x\0" as *const u8 as *const ::core::ffi::c_char,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        (*window).menu_last_px,
    );
    format_add(
        ft,
        b"popup_last_y\0" as *const u8 as *const ::core::ffi::c_char,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        (*window).menu_last_py.wrapping_add(h),
    );
    lines = status_line_size(tc);
    position = options_get_number(
        (*s).options,
        b"status-position\0" as *const u8 as *const ::core::ffi::c_char,
    ) as u_int;
    if status_at_line(tc) != -(1 as ::core::ffi::c_int) && lines != 0 as u_int {
        line = 0 as u_int;
        while line < lines {
            ranges = &raw mut (*(&raw mut (*tc).status.entries as *mut style_line_entry)
                .offset(line as isize))
            .ranges;
            sr = ::core::ptr::null_mut::<style_range>();
            for range in (*ranges).as_slice() {
                let candidate = range.as_ref();
                if !((*candidate).type_0 as ::core::ffi::c_uint
                    != STYLE_RANGE_WINDOW as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    if (*candidate).argument == (*wl).idx as u_int {
                        sr = candidate as *const style_range as *mut style_range;
                        break;
                    }
                }
            }
            if !sr.is_null() {
                break;
            }
            line = line.wrapping_add(1);
        }
        if !sr.is_null() {
            format_add(
                ft,
                b"popup_window_status_line_x\0" as *const u8 as *const ::core::ffi::c_char,
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*sr).start.wrapping_add(ox),
            );
            if position == 0 as u_int {
                format_add(
                    ft,
                    b"popup_window_status_line_y\0" as *const u8 as *const ::core::ffi::c_char,
                    b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                    h,
                );
            } else {
                format_add(
                    ft,
                    b"popup_window_status_line_y\0" as *const u8 as *const ::core::ffi::c_char,
                    b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                    (*window).sy,
                );
            }
        }
        if position == 0 as u_int {
            format_add(
                ft,
                b"popup_status_line_y\0" as *const u8 as *const ::core::ffi::c_char,
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                h,
            );
        } else {
            format_add(
                ft,
                b"popup_status_line_y\0" as *const u8 as *const ::core::ffi::c_char,
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*window).sy,
            );
        }
    }
    format_add(
        ft,
        b"popup_width\0" as *const u8 as *const ::core::ffi::c_char,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
    format_add(
        ft,
        b"popup_height\0" as *const u8 as *const ::core::ffi::c_char,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        h,
    );
    n = ((*window).sx as ::core::ffi::c_long - 1 as ::core::ffi::c_long) / 2 as ::core::ffi::c_long
        - w.wrapping_div(2 as u_int) as ::core::ffi::c_long;
    if n < 0 as ::core::ffi::c_long {
        format_add(
            ft,
            b"popup_centre_x\0" as *const u8 as *const ::core::ffi::c_char,
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
        );
    } else {
        format_add(
            ft,
            b"popup_centre_x\0" as *const u8 as *const ::core::ffi::c_char,
            b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
            n,
        );
    }
    n = ((*window).sy as ::core::ffi::c_long - 1 as ::core::ffi::c_long) / 2 as ::core::ffi::c_long
        + h.wrapping_div(2 as u_int) as ::core::ffi::c_long;
    if n >= (*window).sy as ::core::ffi::c_long {
        format_add(
            ft,
            b"popup_centre_y\0" as *const u8 as *const ::core::ffi::c_char,
            b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
            max_y,
        );
    } else {
        format_add(
            ft,
            b"popup_centre_y\0" as *const u8 as *const ::core::ffi::c_char,
            b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
            n,
        );
    }
    if (*event).m.valid != 0 {
        n = mouse_x - w.wrapping_div(2 as u_int) as ::core::ffi::c_long;
        if n < 0 as ::core::ffi::c_long {
            format_add(
                ft,
                b"popup_mouse_centre_x\0" as *const u8 as *const ::core::ffi::c_char,
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
            );
        } else {
            format_add(
                ft,
                b"popup_mouse_centre_x\0" as *const u8 as *const ::core::ffi::c_char,
                b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
                n,
            );
        }
        n = mouse_y - h.wrapping_div(2 as u_int) as ::core::ffi::c_long;
        if n + h as ::core::ffi::c_long >= (*window).sy as ::core::ffi::c_long {
            format_add(
                ft,
                b"popup_mouse_centre_y\0" as *const u8 as *const ::core::ffi::c_char,
                b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
                max_y,
            );
        } else {
            format_add(
                ft,
                b"popup_mouse_centre_y\0" as *const u8 as *const ::core::ffi::c_char,
                b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
                n,
            );
        }
        n = mouse_y + h as ::core::ffi::c_long;
        if n >= (*window).sy as ::core::ffi::c_long {
            format_add(
                ft,
                b"popup_mouse_top\0" as *const u8 as *const ::core::ffi::c_char,
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*window).sy.wrapping_sub(1 as u_int),
            );
        } else {
            format_add(
                ft,
                b"popup_mouse_top\0" as *const u8 as *const ::core::ffi::c_char,
                b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
                n,
            );
        }
        n = mouse_y - h as ::core::ffi::c_long;
        if n < 0 as ::core::ffi::c_long {
            format_add(
                ft,
                b"popup_mouse_bottom\0" as *const u8 as *const ::core::ffi::c_char,
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
            );
        } else {
            format_add(
                ft,
                b"popup_mouse_bottom\0" as *const u8 as *const ::core::ffi::c_char,
                b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
                n,
            );
        }
    }
    n = ((*wp).yoff as u_int).wrapping_add(h) as ::core::ffi::c_long;
    if n >= (*window).sy as ::core::ffi::c_long {
        format_add(
            ft,
            b"popup_pane_top\0" as *const u8 as *const ::core::ffi::c_char,
            b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
            max_y,
        );
    } else {
        format_add(
            ft,
            b"popup_pane_top\0" as *const u8 as *const ::core::ffi::c_char,
            b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
            n,
        );
    }
    format_add(
        ft,
        b"popup_pane_bottom\0" as *const u8 as *const ::core::ffi::c_char,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        ((*wp).yoff as u_int).wrapping_add((*wp).sy),
    );
    format_add(
        ft,
        b"popup_pane_left\0" as *const u8 as *const ::core::ffi::c_char,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).xoff,
    );
    n = (*wp).xoff as ::core::ffi::c_long + (*wp).sx as ::core::ffi::c_long
        - w as ::core::ffi::c_long;
    if n < 0 as ::core::ffi::c_long {
        format_add(
            ft,
            b"popup_pane_right\0" as *const u8 as *const ::core::ffi::c_char,
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
        );
    } else {
        format_add(
            ft,
            b"popup_pane_right\0" as *const u8 as *const ::core::ffi::c_char,
            b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
            n,
        );
    }
    xp = args_get(args, 'x' as i32 as u_char);
    if xp.is_null()
        || strcmp(xp, b"C\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
    {
        xp = b"#{popup_centre_x}\0" as *const u8 as *const ::core::ffi::c_char;
    } else if strcmp(xp, b"R\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        xp = b"#{popup_pane_right}\0" as *const u8 as *const ::core::ffi::c_char;
    } else if strcmp(xp, b"P\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        xp = b"#{popup_pane_left}\0" as *const u8 as *const ::core::ffi::c_char;
    } else if strcmp(xp, b"M\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        xp = b"#{popup_mouse_centre_x}\0" as *const u8 as *const ::core::ffi::c_char;
    } else if strcmp(xp, b"L\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        xp = b"#{popup_last_x}\0" as *const u8 as *const ::core::ffi::c_char;
    } else if strcmp(xp, b"W\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        xp = b"#{popup_window_status_line_x}\0" as *const u8 as *const ::core::ffi::c_char;
    }
    let p = format_expand_cstring(ft, xp);
    n = strtol(
        p.as_ptr(),
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        10 as ::core::ffi::c_int,
    );
    if n < 0 as ::core::ffi::c_long {
        n = 0 as ::core::ffi::c_long;
    }
    if n > max_x {
        n = max_x;
    }
    *px = n as u_int;
    log_debug(
        b"%s: -x: %s = %s = %u (-w %u)\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_display_menu_get_menu_pos\0" as *const u8 as *const ::core::ffi::c_char,
        xp,
        p.as_ptr(),
        *px,
        w,
    );
    yp = args_get(args, 'y' as i32 as u_char);
    if yp.is_null()
        || strcmp(yp, b"C\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
    {
        yp = b"#{popup_centre_y}\0" as *const u8 as *const ::core::ffi::c_char;
    } else if strcmp(yp, b"P\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        yp = b"#{popup_pane_bottom}\0" as *const u8 as *const ::core::ffi::c_char;
    } else if strcmp(yp, b"M\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        yp = b"#{popup_mouse_top}\0" as *const u8 as *const ::core::ffi::c_char;
    } else if strcmp(yp, b"L\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        yp = b"#{popup_last_y}\0" as *const u8 as *const ::core::ffi::c_char;
    } else if strcmp(yp, b"S\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        yp = b"#{popup_status_line_y}\0" as *const u8 as *const ::core::ffi::c_char;
    } else if strcmp(yp, b"W\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        yp = b"#{popup_window_status_line_y}\0" as *const u8 as *const ::core::ffi::c_char;
    }
    let p = format_expand_cstring(ft, yp);
    n = strtol(
        p.as_ptr(),
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        10 as ::core::ffi::c_int,
    );
    if n < h as ::core::ffi::c_long {
        n = 0 as ::core::ffi::c_long;
    } else {
        n -= h as ::core::ffi::c_long;
    }
    if n < 0 as ::core::ffi::c_long {
        n = 0 as ::core::ffi::c_long;
    }
    if n > max_y {
        n = max_y;
    }
    *py = n as u_int;
    log_debug(
        b"%s: -y: %s = %s = %u (-h %u)\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_display_menu_get_menu_pos\0" as *const u8 as *const ::core::ffi::c_char,
        yp,
        p.as_ptr(),
        *py,
        h,
    );
    format_free(ft);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn cmd_display_menu_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut current_block: u64;
    let mut args: *mut args = cmd_get_args(self_0);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut event: *mut key_event = cmdq_get_event(item);
    let mut tc: *mut client = cmdq_get_target_client(item);
    let mut menu: *mut menu = ::core::ptr::null_mut::<menu>();
    let mut menu_item: menu_item = menu_item {
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        key: 0,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    };
    let mut key: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut style: *const ::core::ffi::c_char = args_get(args, 's' as i32 as u_char);
    let mut border_style: *const ::core::ffi::c_char = args_get(args, 'S' as i32 as u_char);
    let mut selected_style: *const ::core::ffi::c_char = args_get(args, 'H' as i32 as u_char);
    let mut lines: box_lines = BOX_LINES_DEFAULT;
    let mut cause: Option<std::ffi::CString> = None;
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut starting_choice: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut i: u_int = 0;
    let mut count: u_int = args_count(args);
    let mut o: *mut options = (*(*(*(*target).s).curw).window).options;
    let mut oe: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    if args_has(args, 'C' as i32 as u_char) != 0 {
        if strcmp(
            args_get(args, 'C' as i32 as u_char),
            b"-\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            starting_choice = -(1 as ::core::ffi::c_int);
            current_block = 1841672684692190573;
        } else {
            match args_strtonum_result(
                args,
                'C' as i32 as u_char,
                0 as ::core::ffi::c_longlong,
                UINT_MAX as ::core::ffi::c_longlong,
            ) {
                Ok(value) => {
                    starting_choice = value as ::core::ffi::c_int;
                    current_block = 1841672684692190573;
                }
                Err(error) => {
                    cmdq_error(
                        item,
                        b"starting choice %s\0" as *const u8 as *const ::core::ffi::c_char,
                        error.message().as_ptr(),
                    );
                    current_block = 17658167438033882251;
                }
            }
        }
    } else {
        current_block = 1841672684692190573;
    }
    match current_block {
        1841672684692190573 => {
            let formatted_title = if args_has(args, 'T' as i32 as u_char) != 0 {
                Some(format_single_from_target_cstring(
                    item,
                    args_get(args, 'T' as i32 as u_char),
                ))
            } else {
                None
            };
            let title = formatted_title
                .as_ref()
                .map_or(c"".as_ptr(), |title| title.as_ptr());
            menu = menu_create(title);
            i = 0 as u_int;
            loop {
                if !(i != count) {
                    current_block = 2232869372362427478;
                    break;
                }
                let fresh3 = i;
                i = i.wrapping_add(1);
                name = args_string(args, fresh3);
                if *name as ::core::ffi::c_int == '\0' as i32 {
                    menu_add_item(menu, ::core::ptr::null::<menu_item>(), item, tc, target);
                } else if count.wrapping_sub(i) < 2 as u_int {
                    cmdq_error(
                        item,
                        b"not enough arguments\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                    current_block = 17658167438033882251;
                    break;
                } else {
                    let fresh4 = i;
                    i = i.wrapping_add(1);
                    key = args_string(args, fresh4);
                    menu_item.name = name;
                    menu_item.key = key_string_parse_cstr(std::ffi::CStr::from_ptr(key))
                        .unwrap_or(KEYC_UNKNOWN);
                    let fresh5 = i;
                    i = i.wrapping_add(1);
                    menu_item.command = args_string(args, fresh5);
                    menu_add_item(menu, &raw mut menu_item, item, tc, target);
                }
            }
            match current_block {
                17658167438033882251 => {}
                _ => {
                    if menu.is_null() {
                        cmdq_error(
                            item,
                            b"invalid menu arguments\0" as *const u8 as *const ::core::ffi::c_char,
                        );
                    } else {
                        if (*menu).count == 0 as u_int {
                            current_block = 14896172439631163786;
                        } else if cmd_display_menu_get_menu_pos(
                            tc,
                            item,
                            args,
                            &raw mut px,
                            &raw mut py,
                            (*menu).width.wrapping_add(4 as u_int),
                            (*menu).count.wrapping_add(2 as u_int),
                        ) == 0
                        {
                            current_block = 14896172439631163786;
                        } else {
                            value = args_get(args, 'b' as i32 as u_char);
                            if !value.is_null() {
                                oe = options_get(
                                    o,
                                    b"menu-border-lines\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                );
                                lines = options_find_choice(
                                    options_table_entry(oe),
                                    value,
                                    &raw mut cause,
                                ) as box_lines;
                                if lines as ::core::ffi::c_int == -(1 as ::core::ffi::c_int) {
                                    cmdq_error(
                                        item,
                                        b"menu-border-lines %s\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                        cause.as_ref().unwrap().as_ptr(),
                                    );
                                    current_block = 17658167438033882251;
                                } else {
                                    current_block = 7245201122033322888;
                                }
                            } else {
                                current_block = 7245201122033322888;
                            }
                            match current_block {
                                17658167438033882251 => {}
                                _ => {
                                    if args_has(args, 'O' as i32 as u_char) != 0 {
                                        flags |= MENU_STAYOPEN;
                                    }
                                    if (*event).m.valid == 0
                                        && args_has(args, 'M' as i32 as u_char) == 0
                                    {
                                        flags |= MENU_NOMOUSE;
                                    }
                                    if menu_display(
                                        menu,
                                        flags,
                                        starting_choice,
                                        item,
                                        px,
                                        py,
                                        tc,
                                        lines,
                                        style,
                                        selected_style,
                                        border_style,
                                        target,
                                        None,
                                        NULL,
                                    ) != 0 as ::core::ffi::c_int
                                    {
                                        current_block = 14896172439631163786;
                                    } else {
                                        return CMD_RETURN_NORMAL;
                                    }
                                }
                            }
                        }
                        match current_block {
                            17658167438033882251 => {}
                            _ => {
                                menu_free(menu);
                                return CMD_RETURN_NORMAL;
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
    menu_free(menu);
    return CMD_RETURN_ERROR;
}
unsafe extern "C" fn cmd_display_popup_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut current_block: u64;
    let mut args: *mut args = cmd_get_args(self_0);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut s: *mut session = (*target).s;
    let mut tc: *mut client = cmdq_get_target_client(item);
    let mut tty: *mut tty = &raw mut (*tc).tty;
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut shell: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut shellcmd: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut style: *const ::core::ffi::c_char = args_get(args, 's' as i32 as u_char);
    let mut border_style: *const ::core::ffi::c_char = args_get(args, 'S' as i32 as u_char);
    let mut cwd: *const ::core::ffi::c_char = ::core::ptr::null();
    let mut formatted_cwd: Option<std::ffi::CString> = None;
    let mut default_cwd: Option<std::ffi::CString> = None;
    let mut cause: Option<std::ffi::CString> = None;
    let mut argv_owner = Vec::new();
    let mut title: *const ::core::ffi::c_char = ::core::ptr::null();
    let mut formatted_title: Option<std::ffi::CString> = None;
    let mut modify: ::core::ffi::c_int = popup_present(tc);
    let mut flags: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    let mut lines: box_lines = BOX_LINES_DEFAULT;
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut w: u_int = 0;
    let mut h: u_int = 0;
    let mut count: u_int = args_count(args);
    let mut av: *mut args_value = ::core::ptr::null_mut::<args_value>();
    let mut env: *mut environ = ::core::ptr::null_mut::<environ>();
    let mut o: *mut options = (*(*(*s).curw).window).options;
    let mut oe: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    if args_has(args, 'C' as i32 as u_char) != 0 {
        server_client_clear_overlay(tc);
        return CMD_RETURN_NORMAL;
    }
    if (*tc).flags & CLIENT_CONTROL as uint64_t != 0 {
        return CMD_RETURN_NORMAL;
    }
    if modify == 0 && (*tc).overlay_draw.is_some() {
        return CMD_RETURN_NORMAL;
    }
    if modify == 0 {
        h = (*tty).sy.wrapping_div(2 as u_int);
        if args_has(args, 'h' as i32 as u_char) != 0 {
            match args_percentage_result(
                args,
                'h' as i32 as u_char,
                1 as ::core::ffi::c_longlong,
                (*tty).sy as ::core::ffi::c_longlong,
                (*tty).sy as ::core::ffi::c_longlong,
            ) {
                Ok(value) => {
                    h = value as u_int;
                    current_block = 17833034027772472439;
                }
                Err(error) => {
                    cmdq_error(
                        item,
                        b"height %s\0" as *const u8 as *const ::core::ffi::c_char,
                        error.message().as_ptr(),
                    );
                    current_block = 1988999557336856620;
                }
            }
        } else {
            current_block = 17833034027772472439;
        }
        match current_block {
            1988999557336856620 => {}
            _ => {
                w = (*tty).sx.wrapping_div(2 as u_int);
                if args_has(args, 'w' as i32 as u_char) != 0 {
                    match args_percentage_result(
                        args,
                        'w' as i32 as u_char,
                        1 as ::core::ffi::c_longlong,
                        (*tty).sx as ::core::ffi::c_longlong,
                        (*tty).sx as ::core::ffi::c_longlong,
                    ) {
                        Ok(value) => {
                            w = value as u_int;
                            current_block = 11042950489265723346;
                        }
                        Err(error) => {
                            cmdq_error(
                                item,
                                b"width %s\0" as *const u8 as *const ::core::ffi::c_char,
                                error.message().as_ptr(),
                            );
                            current_block = 1988999557336856620;
                        }
                    }
                } else {
                    current_block = 11042950489265723346;
                }
                match current_block {
                    1988999557336856620 => {}
                    _ => {
                        if w > (*tty).sx {
                            w = (*tty).sx;
                        }
                        if h > (*tty).sy {
                            h = (*tty).sy;
                        }
                        if cmd_display_menu_get_popup_pos(
                            tc,
                            item,
                            args,
                            &raw mut px,
                            &raw mut py,
                            w,
                            h,
                        ) == 0
                        {
                            current_block = 6589043366517631393;
                        } else {
                            value = args_get(args, 'd' as i32 as u_char);
                            if !value.is_null() {
                                formatted_cwd =
                                    Some(format_single_from_target_cstring(item, value));
                                cwd = formatted_cwd
                                    .as_ref()
                                    .expect("formatted cwd was set")
                                    .as_ptr();
                            } else {
                                default_cwd = Some(
                                    std::ffi::CStr::from_ptr(server_client_get_cwd(tc, s))
                                        .to_owned(),
                                );
                                cwd = default_cwd
                                    .as_ref()
                                    .expect("default cwd was copied")
                                    .as_ptr();
                            }
                            if count == 0 as u_int {
                                shellcmd = options_get_string(
                                    (*s).options,
                                    b"default-command\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                            } else if count == 1 as u_int {
                                shellcmd = args_string(args, 0 as u_int);
                            }
                            if count <= 1 as u_int
                                && (shellcmd.is_null()
                                    || *shellcmd as ::core::ffi::c_int == '\0' as i32)
                            {
                                shellcmd = ::core::ptr::null::<::core::ffi::c_char>();
                                shell = options_get_string(
                                    (*s).options,
                                    b"default-shell\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                if checkshell(shell) == 0 {
                                    shell = _PATH_BSHELL.as_ptr();
                                }
                                cmd_append_argv(&mut argv_owner, shell);
                            } else {
                                argv_owner = args_to_vector(args);
                            }
                            if args_has(args, 'e' as i32 as u_char) >= 1 as ::core::ffi::c_int {
                                env = environ_create();
                                av = args_first_value(args, 'e' as i32 as u_char);
                                while !av.is_null() {
                                    environ_put(
                                        env,
                                        (*av).c2rust_unnamed.string,
                                        0 as ::core::ffi::c_int,
                                    );
                                    av = args_next_value(av);
                                }
                            }
                            current_block = 1345366029464561491;
                        }
                    }
                }
            }
        }
    } else {
        current_block = 1345366029464561491;
    }
    match current_block {
        1345366029464561491 => {
            value = args_get(args, 'b' as i32 as u_char);
            if args_has(args, 'B' as i32 as u_char) != 0 {
                lines = BOX_LINES_NONE;
                current_block = 8151474771948790331;
            } else if !value.is_null() {
                oe = options_get(
                    o,
                    b"popup-border-lines\0" as *const u8 as *const ::core::ffi::c_char,
                );
                lines = options_find_choice(options_table_entry(oe), value, &raw mut cause)
                    as box_lines;
                if let Some(cause) = cause.as_ref() {
                    cmdq_error(
                        item,
                        b"popup-border-lines %s\0" as *const u8 as *const ::core::ffi::c_char,
                        cause.as_ptr(),
                    );
                    current_block = 1988999557336856620;
                } else {
                    current_block = 8151474771948790331;
                }
            } else {
                current_block = 8151474771948790331;
            }
            match current_block {
                1988999557336856620 => {}
                _ => {
                    if args_has(args, 'T' as i32 as u_char) != 0 {
                        formatted_title = Some(format_single_from_target_cstring(
                            item,
                            args_get(args, 'T' as i32 as u_char),
                        ));
                        title = formatted_title
                            .as_ref()
                            .expect("formatted title was set")
                            .as_ptr();
                    } else {
                        title = c"".as_ptr();
                    }
                    if args_has(args, 'N' as i32 as u_char) != 0 || modify == 0 {
                        flags = 0 as ::core::ffi::c_int;
                    }
                    if args_has(args, 'E' as i32 as u_char) > 1 as ::core::ffi::c_int {
                        if flags == -(1 as ::core::ffi::c_int) {
                            flags = 0 as ::core::ffi::c_int;
                        }
                        flags |= POPUP_CLOSEEXITZERO;
                    } else if args_has(args, 'E' as i32 as u_char) != 0 {
                        if flags == -(1 as ::core::ffi::c_int) {
                            flags = 0 as ::core::ffi::c_int;
                        }
                        flags |= POPUP_CLOSEEXIT;
                    }
                    if args_has(args, 'k' as i32 as u_char) != 0 {
                        if flags == -(1 as ::core::ffi::c_int) {
                            flags = 0 as ::core::ffi::c_int;
                        }
                        flags |= POPUP_CLOSEANYKEY;
                    }
                    if modify != 0 {
                        popup_modify(tc, title, style, border_style, lines, flags);
                    } else if !(popup_display(
                        flags,
                        lines,
                        item,
                        px,
                        py,
                        w,
                        h,
                        env,
                        shellcmd,
                        &argv_owner,
                        cwd,
                        title,
                        tc,
                        s,
                        style,
                        border_style,
                        None,
                        NULL,
                    ) != 0 as ::core::ffi::c_int)
                    {
                        environ_free(env);
                        return CMD_RETURN_WAIT;
                    }
                    current_block = 6589043366517631393;
                }
            }
        }
        _ => {}
    }
    match current_block {
        1988999557336856620 => {
            environ_free(env);
            return CMD_RETURN_ERROR;
        }
        _ => {
            environ_free(env);
            return CMD_RETURN_NORMAL;
        }
    };
}
