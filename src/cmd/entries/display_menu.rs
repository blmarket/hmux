use crate::src::options::options_owner_ptr;
use crate::src::arguments::{
    args_count, args_flag_values, args_get, args_has, args_percentage_result, args_string,
    args_strtonum_result, args_to_vector,
};
use crate::src::cmd::queue::{cmdq_error, cmdq_get_event, cmdq_get_target, cmdq_get_target_client};
use crate::src::cmd::{cmd_append_argv, cmd_get_args_mut};
use crate::src::environ::{environ_create, environ_put};
use crate::src::ffi::libc::{strcmp, strtol};
use crate::src::format::bytes::write_cstr;
use crate::src::format::{
    format_add, format_create_from_target, format_expand_cstring, format_free,
    format_single_from_target_cstring,
};
use crate::src::key_string::key_string_parse_cstr;
use crate::src::log::{log_cstr, log_debug};
use crate::src::menu::{menu_add_item, menu_create, menu_display};
use crate::src::options::options_table_entry;
use crate::src::options::{
    options_find_choice, options_get, options_get_number, options_get_string,
};
use crate::src::popup::{popup_display, popup_modify, popup_present};
use crate::src::server_client::{server_client_clear_overlay, server_client_get_cwd};
use crate::src::shared::abi::*;
use crate::src::shared::arguments::*;
use crate::src::shared::arguments::{args, args_parse, args_value};
use crate::src::shared::client::client;
use crate::src::shared::client::CLIENT_CONTROL;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item};
use crate::src::shared::command::{CMD_AFTERHOOK, CMD_CLIENT_CFLAG};
use crate::src::shared::environment::environ;
use crate::src::shared::format::format_tree;
use crate::src::shared::key::key_event;
use crate::src::shared::key::*;
use crate::src::shared::layout::*;
use crate::src::shared::limits::UINT_MAX;
use crate::src::shared::menu::{menu_item, MENU_NOMOUSE, MENU_STAYOPEN};
use crate::src::shared::options::{options, options_entry};
use crate::src::shared::pane::window_pane;
use crate::src::shared::popup::{POPUP_CLOSEANYKEY, POPUP_CLOSEEXIT, POPUP_CLOSEEXITZERO};
use crate::src::shared::posix_io::_PATH_BSHELL;
use crate::src::shared::session::session;
use crate::src::shared::style::*;
use crate::src::shared::tty::{tty, tty_window_view};
use crate::src::shared::window::{window, winlink};
use crate::src::status::{status_at_line, status_line_size};
use crate::src::tmux::checkshell;
use crate::src::tty::tty_window_offset;
pub static cmd_display_menu_entry: cmd_entry = {
    cmd_entry {
        name: c"display-menu",
        alias: Some(c"menu"),
        args: args_parse {
            template: c"b:c:C:H:s:S:MOt:T:x:y:",
            lower: 1 as ::core::ffi::c_int,
            upper: -(1 as ::core::ffi::c_int),
            cb: Some(
                cmd_display_menu_args_parse
            ),
        },
        usage: c"[-MO] [-b border-lines] [-c target-client] [-C starting-choice] [-H selected-style] [-s style] [-S border-style] [-t target-pane] [-T title] [-x position] [-y position] name [key] [command] ...",
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
                as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
pub static cmd_display_popup_entry: cmd_entry = {
    cmd_entry {
        name: c"display-popup",
        alias: Some(c"popup"),
        args: args_parse {
            template: c"Bb:Cc:d:e:Eh:kNs:S:t:T:w:x:y:",
            lower: 0 as ::core::ffi::c_int,
            upper: -(1 as ::core::ffi::c_int),
            cb: None,
        },
        usage: c"[-BCEkN] [-b border-lines] [-c target-client] [-d start-directory] [-e environment] [-h height] [-s style] [-S border-style] [-t target-pane] [-T title] [-w width] [-x position] [-y position] [shell-command [argument ...]]",
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
                as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
fn cmd_display_menu_args_parse(
    args: &mut args,
    idx: u_int,
) -> Result<args_parse_type, ArgsParseError> {
    let mut i: u_int = 0 as u_int;
    let mut type_0: args_parse_type = ARGS_PARSE_STRING;
    loop {
        type_0 = ARGS_PARSE_STRING;
        if i == idx {
            break;
        }
        let fresh0 = i;
        i = i.wrapping_add(1);
        if unsafe { *args_string(&mut *(args as *mut args), fresh0).map_or(std::ptr::null(), |value| value.as_ptr()) } as ::core::ffi::c_int == '\0' as i32 {
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
    Ok(type_0)
}
unsafe fn cmd_display_menu_get_popup_pos(
    mut tc: *mut client,
    mut item: *mut cmdq_item,
    mut args: *mut args,
    mut px: *mut u_int,
    mut py: *mut u_int,
    mut w: u_int,
    mut h: u_int,
) -> ::core::ffi::c_int {
    let mut tty: *mut tty = &raw mut (*tc).tty;
    let mut target: *mut cmd_find_state = crate::src::cmd::queue::cmdq_get_target_mut(&mut *item);
    let mut event_snapshot = cmdq_get_event(item);
    let event: *mut key_event = &mut event_snapshot;
    let mut s: *mut session = (*tc).session_ptr();
    let mut wl: *mut winlink = (*target).wl_ptr();
    let mut wp: *mut window_pane = (*target).wp_ptr();
    let mut ranges: *mut style_ranges = ::core::ptr::null_mut::<style_ranges>();
    let mut sr: *mut style_range = ::core::ptr::null_mut::<style_range>();
    let mut xp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut yp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut top: ::core::ffi::c_int = 0;
    let mut line: u_int = 0;
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
            |out| write!(out, "{}", ((*event).m.x) as u32),
        );
        format_add(
            ft,
            b"popup_mouse_y\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write!(out, "{}", ((*event).m.y) as u32),
        );
    }
    format_add(
        ft,
        b"popup_last_x\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write!(out, "{}", ((*(*target).w_ptr()).menu_last_px) as u32),
    );
    format_add(
        ft,
        b"popup_last_y\0" as *const u8 as *const ::core::ffi::c_char,
        |out| {
            write!(
                out,
                "{}",
                ((*(*target).w_ptr()).menu_last_py.wrapping_add(h)) as u32
            )
        },
    );
    top = status_at_line(&*tc);
    if top != -(1 as ::core::ffi::c_int) {
        lines = status_line_size(&*tc);
        if top == 0 as ::core::ffi::c_int {
            top = lines as ::core::ffi::c_int;
        } else {
            top = 0 as ::core::ffi::c_int;
        }
        position = options_get_number(
            options_owner_ptr(&mut (*s).options).map_or(std::ptr::null_mut(), |options| options),
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
                |out| write!(out, "{}", ((*sr).start) as u32),
            );
            if position == 0 as u_int {
                format_add(
                    ft,
                    b"popup_window_status_line_y\0" as *const u8 as *const ::core::ffi::c_char,
                    |out| {
                        write!(
                            out,
                            "{}",
                            (line.wrapping_add(1 as u_int).wrapping_add(h)) as u32
                        )
                    },
                );
            } else {
                format_add(
                    ft,
                    b"popup_window_status_line_y\0" as *const u8 as *const ::core::ffi::c_char,
                    |out| {
                        write!(
                            out,
                            "{}",
                            ((*tty).sy.wrapping_sub(lines).wrapping_add(line)) as u32
                        )
                    },
                );
            }
        }
        if position == 0 as u_int {
            format_add(
                ft,
                b"popup_status_line_y\0" as *const u8 as *const ::core::ffi::c_char,
                |out| write!(out, "{}", (lines.wrapping_add(h)) as u32),
            );
        } else {
            format_add(
                ft,
                b"popup_status_line_y\0" as *const u8 as *const ::core::ffi::c_char,
                |out| write!(out, "{}", ((*tty).sy.wrapping_sub(lines)) as u32),
            );
        }
    } else {
        top = 0 as ::core::ffi::c_int;
    }
    format_add(
        ft,
        b"popup_width\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write!(out, "{}", (w) as u32),
    );
    format_add(
        ft,
        b"popup_height\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write!(out, "{}", (h) as u32),
    );
    n = (*tty).sx.wrapping_sub(1 as u_int) as ::core::ffi::c_long / 2 as ::core::ffi::c_long
        - w.wrapping_div(2 as u_int) as ::core::ffi::c_long;
    if n < 0 as ::core::ffi::c_long {
        format_add(
            ft,
            b"popup_centre_x\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write!(out, "{}", (0 as ::core::ffi::c_int) as u32),
        );
    } else {
        format_add(
            ft,
            b"popup_centre_x\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write!(out, "{}", (n) as ::core::ffi::c_long),
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
            |out| write!(out, "{}", ((*tty).sy.wrapping_sub(h)) as u32),
        );
    } else {
        format_add(
            ft,
            b"popup_centre_y\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write!(out, "{}", (n) as ::core::ffi::c_long),
        );
    }
    if (*event).m.valid != 0 {
        n = (*event).m.x as ::core::ffi::c_long - w.wrapping_div(2 as u_int) as ::core::ffi::c_long;
        if n < 0 as ::core::ffi::c_long {
            format_add(
                ft,
                b"popup_mouse_centre_x\0" as *const u8 as *const ::core::ffi::c_char,
                |out| write!(out, "{}", (0 as ::core::ffi::c_int) as u32),
            );
        } else {
            format_add(
                ft,
                b"popup_mouse_centre_x\0" as *const u8 as *const ::core::ffi::c_char,
                |out| write!(out, "{}", (n) as ::core::ffi::c_long),
            );
        }
        n = (*event).m.y.wrapping_sub(h.wrapping_div(2 as u_int)) as ::core::ffi::c_long;
        if n + h as ::core::ffi::c_long >= (*tty).sy as ::core::ffi::c_long {
            format_add(
                ft,
                b"popup_mouse_centre_y\0" as *const u8 as *const ::core::ffi::c_char,
                |out| write!(out, "{}", ((*tty).sy.wrapping_sub(h)) as u32),
            );
        } else {
            format_add(
                ft,
                b"popup_mouse_centre_y\0" as *const u8 as *const ::core::ffi::c_char,
                |out| write!(out, "{}", (n) as ::core::ffi::c_long),
            );
        }
        n = (*event).m.y as ::core::ffi::c_long + h as ::core::ffi::c_long;
        if n >= (*tty).sy as ::core::ffi::c_long {
            format_add(
                ft,
                b"popup_mouse_top\0" as *const u8 as *const ::core::ffi::c_char,
                |out| write!(out, "{}", ((*tty).sy.wrapping_sub(1 as u_int)) as u32),
            );
        } else {
            format_add(
                ft,
                b"popup_mouse_top\0" as *const u8 as *const ::core::ffi::c_char,
                |out| write!(out, "{}", (n) as ::core::ffi::c_long),
            );
        }
        n = (*event).m.y.wrapping_sub(h) as ::core::ffi::c_long;
        if n < 0 as ::core::ffi::c_long {
            format_add(
                ft,
                b"popup_mouse_bottom\0" as *const u8 as *const ::core::ffi::c_char,
                |out| write!(out, "{}", (0 as ::core::ffi::c_int) as u32),
            );
        } else {
            format_add(
                ft,
                b"popup_mouse_bottom\0" as *const u8 as *const ::core::ffi::c_char,
                |out| write!(out, "{}", (n) as ::core::ffi::c_long),
            );
        }
    }
    let tty_window_view { ox, oy, .. } = tty_window_offset(&(*tc).tty);
    n = ((top + (*wp).yoff) as u_int)
        .wrapping_sub(oy)
        .wrapping_add(h) as ::core::ffi::c_long;
    if n >= (*tty).sy as ::core::ffi::c_long {
        format_add(
            ft,
            b"popup_pane_top\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write!(out, "{}", ((*tty).sy.wrapping_sub(h)) as u32),
        );
    } else {
        format_add(
            ft,
            b"popup_pane_top\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write!(out, "{}", (n) as ::core::ffi::c_long),
        );
    }
    format_add(
        ft,
        b"popup_pane_bottom\0" as *const u8 as *const ::core::ffi::c_char,
        |out| {
            write!(
                out,
                "{}",
                (((top + (*wp).yoff) as u_int)
                    .wrapping_add((*wp).sy)
                    .wrapping_sub(oy)) as u32
            )
        },
    );
    format_add(
        ft,
        b"popup_pane_left\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write!(out, "{}", (((*wp).xoff as u_int).wrapping_sub(ox)) as u32),
    );
    n = (*wp).xoff as ::core::ffi::c_long + (*wp).sx as ::core::ffi::c_long
        - ox as ::core::ffi::c_long
        - w as ::core::ffi::c_long;
    if n < 0 as ::core::ffi::c_long {
        format_add(
            ft,
            b"popup_pane_right\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write!(out, "{}", (0 as ::core::ffi::c_int) as u32),
        );
    } else {
        format_add(
            ft,
            b"popup_pane_right\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write!(out, "{}", (n) as ::core::ffi::c_long),
        );
    }
    xp = args_get(&*(args), 'x' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr());
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
    log_debug(format_args!(
        "{}: -x: {} = {} = {} (-w {})",
        "cmd_display_menu_get_popup_pos",
        log_cstr((xp) as *const _),
        log_cstr((p.as_ptr()) as *const _),
        (*px) as u32,
        (w) as u32
    ));
    yp = args_get(&*(args), 'y' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr());
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
    log_debug(format_args!(
        "{}: -y: {} = {} = {} (-h {})",
        "cmd_display_menu_get_popup_pos",
        log_cstr((yp) as *const _),
        log_cstr((p.as_ptr()) as *const _),
        (*py) as u32,
        (h) as u32
    ));
    format_free(Box::from_raw(ft));
    return 1 as ::core::ffi::c_int;
}
unsafe fn cmd_display_menu_get_menu_pos(
    mut tc: *mut client,
    mut item: *mut cmdq_item,
    mut args: *mut args,
    mut px: *mut u_int,
    mut py: *mut u_int,
    mut w: u_int,
    mut h: u_int,
) -> ::core::ffi::c_int {
    let mut target: *mut cmd_find_state = crate::src::cmd::queue::cmdq_get_target_mut(&mut *item);
    let mut event_snapshot = cmdq_get_event(item);
    let event: *mut key_event = &mut event_snapshot;
    let mut s: *mut session = (*tc).session_ptr();
    let mut wl: *mut winlink = (*target).wl_ptr();
    let mut window: *mut window = (*target).w_ptr();
    let mut wp: *mut window_pane = (*target).wp_ptr();
    let mut ranges: *mut style_ranges = ::core::ptr::null_mut::<style_ranges>();
    let mut sr: *mut style_range = ::core::ptr::null_mut::<style_range>();
    let mut xp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut yp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut line: u_int = 0;
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
    let tty_window_view { ox, oy, sy, .. } = tty_window_offset(&(*tc).tty);
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
            |out| write!(out, "{}", (mouse_x) as ::core::ffi::c_long),
        );
        format_add(
            ft,
            b"popup_mouse_y\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write!(out, "{}", (mouse_y) as ::core::ffi::c_long),
        );
    }
    format_add(
        ft,
        b"popup_last_x\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write!(out, "{}", ((*window).menu_last_px) as u32),
    );
    format_add(
        ft,
        b"popup_last_y\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write!(out, "{}", ((*window).menu_last_py.wrapping_add(h)) as u32),
    );
    lines = status_line_size(&*tc);
    position = options_get_number(
        options_owner_ptr(&mut (*s).options).map_or(std::ptr::null_mut(), |options| options),
        b"status-position\0" as *const u8 as *const ::core::ffi::c_char,
    ) as u_int;
    if status_at_line(&*tc) != -(1 as ::core::ffi::c_int) && lines != 0 as u_int {
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
                |out| write!(out, "{}", ((*sr).start.wrapping_add(ox)) as u32),
            );
            if position == 0 as u_int {
                format_add(
                    ft,
                    b"popup_window_status_line_y\0" as *const u8 as *const ::core::ffi::c_char,
                    |out| write!(out, "{}", (h) as u32),
                );
            } else {
                format_add(
                    ft,
                    b"popup_window_status_line_y\0" as *const u8 as *const ::core::ffi::c_char,
                    |out| write!(out, "{}", ((*window).sy) as u32),
                );
            }
        }
        if position == 0 as u_int {
            format_add(
                ft,
                b"popup_status_line_y\0" as *const u8 as *const ::core::ffi::c_char,
                |out| write!(out, "{}", (h) as u32),
            );
        } else {
            format_add(
                ft,
                b"popup_status_line_y\0" as *const u8 as *const ::core::ffi::c_char,
                |out| write!(out, "{}", ((*window).sy) as u32),
            );
        }
    }
    format_add(
        ft,
        b"popup_width\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write!(out, "{}", (w) as u32),
    );
    format_add(
        ft,
        b"popup_height\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write!(out, "{}", (h) as u32),
    );
    n = ((*window).sx as ::core::ffi::c_long - 1 as ::core::ffi::c_long) / 2 as ::core::ffi::c_long
        - w.wrapping_div(2 as u_int) as ::core::ffi::c_long;
    if n < 0 as ::core::ffi::c_long {
        format_add(
            ft,
            b"popup_centre_x\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write!(out, "{}", (0 as ::core::ffi::c_int) as u32),
        );
    } else {
        format_add(
            ft,
            b"popup_centre_x\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write!(out, "{}", (n) as ::core::ffi::c_long),
        );
    }
    n = ((*window).sy as ::core::ffi::c_long - 1 as ::core::ffi::c_long) / 2 as ::core::ffi::c_long
        + h.wrapping_div(2 as u_int) as ::core::ffi::c_long;
    if n >= (*window).sy as ::core::ffi::c_long {
        format_add(
            ft,
            b"popup_centre_y\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write!(out, "{}", (max_y) as ::core::ffi::c_long),
        );
    } else {
        format_add(
            ft,
            b"popup_centre_y\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write!(out, "{}", (n) as ::core::ffi::c_long),
        );
    }
    if (*event).m.valid != 0 {
        n = mouse_x - w.wrapping_div(2 as u_int) as ::core::ffi::c_long;
        if n < 0 as ::core::ffi::c_long {
            format_add(
                ft,
                b"popup_mouse_centre_x\0" as *const u8 as *const ::core::ffi::c_char,
                |out| write!(out, "{}", (0 as ::core::ffi::c_int) as u32),
            );
        } else {
            format_add(
                ft,
                b"popup_mouse_centre_x\0" as *const u8 as *const ::core::ffi::c_char,
                |out| write!(out, "{}", (n) as ::core::ffi::c_long),
            );
        }
        n = mouse_y - h.wrapping_div(2 as u_int) as ::core::ffi::c_long;
        if n + h as ::core::ffi::c_long >= (*window).sy as ::core::ffi::c_long {
            format_add(
                ft,
                b"popup_mouse_centre_y\0" as *const u8 as *const ::core::ffi::c_char,
                |out| write!(out, "{}", (max_y) as ::core::ffi::c_long),
            );
        } else {
            format_add(
                ft,
                b"popup_mouse_centre_y\0" as *const u8 as *const ::core::ffi::c_char,
                |out| write!(out, "{}", (n) as ::core::ffi::c_long),
            );
        }
        n = mouse_y + h as ::core::ffi::c_long;
        if n >= (*window).sy as ::core::ffi::c_long {
            format_add(
                ft,
                b"popup_mouse_top\0" as *const u8 as *const ::core::ffi::c_char,
                |out| write!(out, "{}", ((*window).sy.wrapping_sub(1 as u_int)) as u32),
            );
        } else {
            format_add(
                ft,
                b"popup_mouse_top\0" as *const u8 as *const ::core::ffi::c_char,
                |out| write!(out, "{}", (n) as ::core::ffi::c_long),
            );
        }
        n = mouse_y - h as ::core::ffi::c_long;
        if n < 0 as ::core::ffi::c_long {
            format_add(
                ft,
                b"popup_mouse_bottom\0" as *const u8 as *const ::core::ffi::c_char,
                |out| write!(out, "{}", (0 as ::core::ffi::c_int) as u32),
            );
        } else {
            format_add(
                ft,
                b"popup_mouse_bottom\0" as *const u8 as *const ::core::ffi::c_char,
                |out| write!(out, "{}", (n) as ::core::ffi::c_long),
            );
        }
    }
    n = ((*wp).yoff as u_int).wrapping_add(h) as ::core::ffi::c_long;
    if n >= (*window).sy as ::core::ffi::c_long {
        format_add(
            ft,
            b"popup_pane_top\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write!(out, "{}", (max_y) as ::core::ffi::c_long),
        );
    } else {
        format_add(
            ft,
            b"popup_pane_top\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write!(out, "{}", (n) as ::core::ffi::c_long),
        );
    }
    format_add(
        ft,
        b"popup_pane_bottom\0" as *const u8 as *const ::core::ffi::c_char,
        |out| {
            write!(
                out,
                "{}",
                (((*wp).yoff as u_int).wrapping_add((*wp).sy)) as u32
            )
        },
    );
    format_add(
        ft,
        b"popup_pane_left\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write!(out, "{}", ((*wp).xoff) as u32),
    );
    n = (*wp).xoff as ::core::ffi::c_long + (*wp).sx as ::core::ffi::c_long
        - w as ::core::ffi::c_long;
    if n < 0 as ::core::ffi::c_long {
        format_add(
            ft,
            b"popup_pane_right\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write!(out, "{}", (0 as ::core::ffi::c_int) as u32),
        );
    } else {
        format_add(
            ft,
            b"popup_pane_right\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write!(out, "{}", (n) as ::core::ffi::c_long),
        );
    }
    xp = args_get(&*(args), 'x' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr());
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
    log_debug(format_args!(
        "{}: -x: {} = {} = {} (-w {})",
        "cmd_display_menu_get_menu_pos",
        log_cstr((xp) as *const _),
        log_cstr((p.as_ptr()) as *const _),
        (*px) as u32,
        (w) as u32
    ));
    yp = args_get(&*(args), 'y' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr());
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
    log_debug(format_args!(
        "{}: -y: {} = {} = {} (-h {})",
        "cmd_display_menu_get_menu_pos",
        log_cstr((yp) as *const _),
        log_cstr((p.as_ptr()) as *const _),
        (*py) as u32,
        (h) as u32
    ));
    format_free(Box::from_raw(ft));
    return 1 as ::core::ffi::c_int;
}
unsafe fn cmd_display_menu_exec(self_0: *mut cmd, item: *mut cmdq_item) -> cmd_retval {
    let args = cmd_get_args_mut(&mut *self_0).map_or(std::ptr::null_mut(), |args| args);
    let target = crate::src::cmd::queue::cmdq_get_target_mut(&mut *item);
    let mut event_snapshot = cmdq_get_event(item);
    let event: *mut key_event = &mut event_snapshot;
    let tc_owner = cmdq_get_target_client(item);
    let tc = tc_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    let style = args_get(&*(args), b's').map_or(std::ptr::null(), |value| value.as_ptr());
    let border_style = args_get(&*(args), b'S').map_or(std::ptr::null(), |value| value.as_ptr());
    let selected_style = args_get(&*(args), b'H').map_or(std::ptr::null(), |value| value.as_ptr());
    let o = options_owner_ptr(&mut (*(*(*(*target).s_ptr()).curw_ptr()).window_ptr()).options).map_or(std::ptr::null_mut(), |options| options);
    let mut starting_choice = 0;
    if args_has(args, b'C') != 0 {
        if std::ffi::CStr::from_ptr(args_get(&*(args), b'C').map_or(std::ptr::null(), |value| value.as_ptr())) == c"-" {
            starting_choice = -1;
        } else {
            match args_strtonum_result(args, b'C', 0, UINT_MAX as i64) {
                Ok(value) => starting_choice = value as i32,
                Err(error) => {
                    cmdq_error(item, |out| {
                        out.write_all(b"starting choice ")?;
                        write_cstr(out, error.message().as_ptr())
                    });
                    return CMD_RETURN_ERROR;
                }
            }
        }
    }
    let title = if args_has(args, b'T') != 0 {
        Some(format_single_from_target_cstring(
            item,
            args_get(&*(args), b'T').map_or(std::ptr::null(), |value| value.as_ptr()),
        ))
    } else {
        None
    };
    let mut menu = menu_create(title.as_deref().unwrap_or(c""));
    drop(title);
    let count = args_count(args);
    let mut i = 0;
    while i != count {
        let name = std::ffi::CStr::from_ptr(args_string(&mut *(args), i).map_or(std::ptr::null(), |value| value.as_ptr()));
        i += 1;
        if name.is_empty() {
            menu_add_item(&mut menu, None, item, tc_owner.as_ref(), target);
            continue;
        }
        if count - i < 2 {
            cmdq_error(item, |out| out.write_all(b"not enough arguments"));
            return CMD_RETURN_ERROR;
        }
        let key = std::ffi::CStr::from_ptr(args_string(&mut *(args), i).map_or(std::ptr::null(), |value| value.as_ptr()));
        let command = std::ffi::CStr::from_ptr(args_string(&mut *(args), i + 1).map_or(std::ptr::null(), |value| value.as_ptr()));
        i += 2;
        let definition = menu_item {
            name,
            key: key_string_parse_cstr(key).unwrap_or(KEYC_UNKNOWN),
            command: Some(command),
        };
        menu_add_item(&mut menu, Some(&definition), item, tc_owner.as_ref(), target);
    }
    let mut px = 0;
    let mut py = 0;
    if menu.count == 0
        || cmd_display_menu_get_menu_pos(
            tc,
            item,
            args,
            &mut px,
            &mut py,
            menu.width.wrapping_add(4),
            menu.count.wrapping_add(2),
        ) == 0
    {
        return CMD_RETURN_NORMAL;
    }
    let mut lines = BOX_LINES_DEFAULT;
    let value = args_get(&*(args), b'b').map_or(std::ptr::null(), |value| value.as_ptr());
    if !value.is_null() {
        let oe = options_get(o, c"menu-border-lines".as_ptr());
        let mut cause = None;
        lines = options_find_choice(options_table_entry(&*(oe)).map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry), value, &mut cause) as box_lines;
        if lines == -1 {
            cmdq_error(item, |out| {
                out.write_all(b"menu-border-lines ")?;
                write_cstr(out, cause.as_ref().unwrap().as_ptr())
            });
            return CMD_RETURN_ERROR;
        }
    }
    let mut flags = 0;
    if args_has(args, b'O') != 0 {
        flags |= MENU_STAYOPEN;
    }
    if (*event).m.valid == 0 && args_has(args, b'M') == 0 {
        flags |= MENU_NOMOUSE;
    }
    menu_display(
        menu,
        flags,
        starting_choice,
        Some(&*event),
        px,
        py,
        tc_owner.as_ref(),
        lines,
        (!style.is_null()).then(|| std::ffi::CStr::from_ptr(style)),
        (!selected_style.is_null()).then(|| std::ffi::CStr::from_ptr(selected_style)),
        (!border_style.is_null()).then(|| std::ffi::CStr::from_ptr(border_style)),
        target,
        None,
    );
    CMD_RETURN_NORMAL
}
unsafe fn cmd_display_popup_exec(mut self_0: *mut cmd, mut item: *mut cmdq_item) -> cmd_retval {
    let mut current_block: u64;
    let mut args: *mut args = cmd_get_args_mut(&mut *self_0).map_or(std::ptr::null_mut(), |args| args);
    let mut target: *mut cmd_find_state = crate::src::cmd::queue::cmdq_get_target_mut(&mut *item);
    let mut s: *mut session = (*target).s_ptr();
    let tc_owner = cmdq_get_target_client(item);
    let mut tc: *mut client = tc_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    let mut tty: *mut tty = &raw mut (*tc).tty;
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut shell: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut shellcmd: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut style: *const ::core::ffi::c_char = args_get(&*(args), 's' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr());
    let mut border_style: *const ::core::ffi::c_char = args_get(&*(args), 'S' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr());
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
    let mut env: Option<Box<environ>> = None;
    let mut o: *mut options = options_owner_ptr(&mut (*(*(*s).curw_ptr()).window_ptr()).options).map_or(std::ptr::null_mut(), |options| options);
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
                    cmdq_error(item, |out| {
                        out.write_all(b"height ")?;
                        write_cstr(out, error.message().as_ptr())
                    });
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
                            cmdq_error(item, |out| {
                                out.write_all(b"width ")?;
                                write_cstr(out, error.message().as_ptr())
                            });
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
                            value = args_get(&*(args), 'd' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr());
                            if !value.is_null() {
                                formatted_cwd =
                                    Some(format_single_from_target_cstring(item, value));
                                cwd = formatted_cwd
                                    .as_ref()
                                    .expect("formatted cwd was set")
                                    .as_ptr();
                            } else {
                                default_cwd = server_client_get_cwd(tc.as_ref(), s.as_ref());
                                cwd = default_cwd
                                    .as_ref()
                                    .expect("default cwd was copied")
                                    .as_ptr();
                            }
                            if count == 0 as u_int {
                                shellcmd = options_get_string(
                                    options_owner_ptr(&mut (*s).options).map_or(std::ptr::null_mut(), |options| options),
                                    b"default-command\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                            } else if count == 1 as u_int {
                                shellcmd = args_string(&mut *(args), 0 as u_int).map_or(std::ptr::null(), |value| value.as_ptr());
                            }
                            if count <= 1 as u_int
                                && (shellcmd.is_null()
                                    || *shellcmd as ::core::ffi::c_int == '\0' as i32)
                            {
                                shellcmd = ::core::ptr::null::<::core::ffi::c_char>();
                                shell = options_get_string(
                                    options_owner_ptr(&mut (*s).options).map_or(std::ptr::null_mut(), |options| options),
                                    b"default-shell\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                if checkshell(shell) == 0 {
                                    shell = _PATH_BSHELL.as_ptr();
                                }
                                cmd_append_argv(&mut argv_owner, std::ffi::CStr::from_ptr(shell));
                            } else {
                                argv_owner = args_to_vector(&*args);
                            }
                            if args_has(args, 'e' as i32 as u_char) >= 1 as ::core::ffi::c_int {
                                env = Some(environ_create());
                                for av in args_flag_values(&*args, 'e' as i32 as u_char) {
                                    environ_put(
                                        env.as_deref_mut().expect("environment"),
                                        av.string_ptr(),
                                        0 as ::core::ffi::c_int,
                                    );
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
            value = args_get(&*(args), 'b' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr());
            if args_has(args, 'B' as i32 as u_char) != 0 {
                lines = BOX_LINES_NONE;
                current_block = 8151474771948790331;
            } else if !value.is_null() {
                oe = options_get(
                    o,
                    b"popup-border-lines\0" as *const u8 as *const ::core::ffi::c_char,
                );
                lines = options_find_choice(options_table_entry(&*(oe)).map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry), value, &raw mut cause)
                    as box_lines;
                if let Some(cause) = cause.as_ref() {
                    cmdq_error(item, |out| {
                        out.write_all(b"popup-border-lines ")?;
                        write_cstr(out, cause.as_ptr())
                    });
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
                            args_get(&*(args), 'T' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr()),
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
                        env.as_deref(),
                        shellcmd,
                        &argv_owner,
                        cwd,
                        title,
                        tc,
                        s,
                        style,
                        border_style,
                    ) != 0 as ::core::ffi::c_int)
                    {
                        drop(env.take());
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
            drop(env.take());
            return CMD_RETURN_ERROR;
        }
        _ => {
            drop(env.take());
            return CMD_RETURN_NORMAL;
        }
    };
}
