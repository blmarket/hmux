use crate::src::cmd::queue::{cmdq_append, cmdq_get_callback_owned};
use crate::src::ffi::libc::{memcpy, memset};
use crate::src::format::bytes::format_message_with;
use crate::src::format::bytes::write_cstr;
use crate::src::format::{
    format_add, format_create, format_create_defaults, format_defaults, format_expand_time_cstring,
    format_free,
};
use crate::src::format_draw::format_draw;
use crate::src::grid::{grid_cells_equal, grid_compare};
use crate::src::log::{fatalx, log_cstr, log_debug, log_pointer};
use crate::src::options::{
    options_array_get_index, options_get, options_get_number, options_get_string,
    options_string_to_style,
};
use crate::src::prompt::{
    prompt_closed, prompt_create, prompt_draw, prompt_free, prompt_incremental_start, prompt_key,
    prompt_mouse, prompt_set_options, prompt_update,
};
use crate::src::reactor::{event_add, event_del, event_initialized, event_set};
use crate::src::screen::{screen_free, screen_init, screen_resize};
use crate::src::screen_write::{
    screen_write_cursormove, screen_write_fast_copy, screen_write_putc, screen_write_start,
    screen_write_stop,
};
use crate::src::server::clients;
use crate::src::server::server_add_message;
use crate::src::server_client::{
    server_client_clear_overlay, server_client_set_message, server_client_set_status_expanded,
};
use crate::src::shared::prompt::{prompt_create_data, prompt_draw_data};
use crate::src::style::{
    style_apply, style_ranges_clear, style_ranges_free, style_ranges_get_range, style_ranges_init,
};
use crate::src::tmux::global_s_options;
use std::ffi::{CStr, CString};

use crate::src::shared::abi::*;
use crate::src::shared::client::client;
use crate::src::shared::client::{
    CLIENT_ALLREDRAWFLAGS, CLIENT_CONTROL, CLIENT_REDRAWSTATUS, CLIENT_STATUSFORCE,
    CLIENT_STATUSOFF,
};
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd_find_state, cmdq_item};
use crate::src::shared::display::*;
use crate::src::shared::format::format_tree;
use crate::src::shared::format::{FORMAT_FORCE, FORMAT_NONE, FORMAT_STATUS};
use crate::src::shared::grid::*;
use crate::src::shared::key::*;
use crate::src::shared::mouse::{mouse_event, MOUSE_BUTTON_1, MOUSE_MASK_BUTTONS, MOUSE_MASK_DRAG};
use crate::src::shared::options::{options_entry, options_value};
use crate::src::shared::pane::window_pane;
use crate::src::shared::prompt::prompt;
use crate::src::shared::prompt::*;
use crate::src::shared::prompt::{
    prompt_free_cb, prompt_result, PROMPT_ACCEPT, PROMPT_CLOSE, PROMPT_INCREMENTAL,
    PROMPT_NOFREEZE, PROMPT_SINGLE,
};
use crate::src::shared::screen::screen;
use crate::src::shared::screen_write::screen_write_ctx;
use crate::src::shared::session::session;
use crate::src::shared::status::{status_line, status_prompt_input_cb};
use crate::src::shared::style::*;
use crate::src::shared::tty::tty;
use crate::src::shared::tty::{TTY_FREEZE, TTY_NOCURSOR};
use crate::src::shared::window::winlink;

unsafe fn status_timer_callback(mut arg: *mut ::core::ffi::c_void) {
    let mut c: *mut client = arg as *mut client;
    let mut s: *mut session = (*c).session;
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    event_del(&raw mut (*c).status.timer);
    if s.is_null() {
        return;
    }
    if (*c).message_string.is_none() && (*c).prompt.is_null() {
        (*c).flags |= CLIENT_REDRAWSTATUS as uint64_t;
    }
    tv.tv_usec = 0 as __suseconds_t;
    tv.tv_sec = tv.tv_usec as __time_t;
    tv.tv_sec = options_get_number(
        (*s).options,
        b"status-interval\0" as *const u8 as *const ::core::ffi::c_char,
    ) as __time_t;
    if tv.tv_sec != 0 as __time_t {
        event_add(&raw mut (*c).status.timer, &raw mut tv);
    }
    log_debug(format_args!(
        "client {}, status interval {}",
        log_pointer((c) as *const ::core::ffi::c_void),
        tv.tv_sec as ::core::ffi::c_int
    ));
}
pub unsafe fn status_timer_start(mut c: *mut client) {
    let mut s: *mut session = (*c).session;
    if event_initialized(&(*c).status.timer) != 0 {
        event_del(&raw mut (*c).status.timer);
    } else {
        event_set(
            &raw mut (*c).status.timer,
            -(1 as ::core::ffi::c_int),
            0 as ::core::ffi::c_short,
            move |_, _| unsafe { status_timer_callback(c as *mut ::core::ffi::c_void) },
        );
    }
    if !s.is_null()
        && options_get_number(
            (*s).options,
            b"status\0" as *const u8 as *const ::core::ffi::c_char,
        ) != 0
    {
        status_timer_callback(c as *mut ::core::ffi::c_void);
    }
}
pub unsafe fn status_timer_start_all() {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    c = clients.first();
    while !c.is_null() {
        status_timer_start(c);
        c = clients.next(c);
    }
}
pub unsafe fn status_update_cache(mut s: *mut session) {
    (*s).statuslines = options_get_number(
        (*s).options,
        b"status\0" as *const u8 as *const ::core::ffi::c_char,
    ) as u_int;
    if (*s).statuslines == 0 as u_int {
        (*s).statusat = -(1 as ::core::ffi::c_int);
    } else if options_get_number(
        (*s).options,
        b"status-position\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_longlong
    {
        (*s).statusat = 0 as ::core::ffi::c_int;
    } else {
        (*s).statusat = 1 as ::core::ffi::c_int;
    };
}
pub unsafe fn status_at_line(mut c: *mut client) -> ::core::ffi::c_int {
    let mut s: *mut session = (*c).session;
    if (*c).flags & (CLIENT_STATUSOFF | CLIENT_CONTROL) as uint64_t != 0 {
        return -(1 as ::core::ffi::c_int);
    }
    if (*s).statusat != 1 as ::core::ffi::c_int {
        return (*s).statusat;
    }
    return (*c).tty.sy.wrapping_sub(status_line_size(c)) as ::core::ffi::c_int;
}
pub unsafe fn status_line_size(mut c: *mut client) -> u_int {
    let mut s: *mut session = (*c).session;
    if (*c).flags & (CLIENT_STATUSOFF | CLIENT_CONTROL) as uint64_t != 0 {
        return 0 as u_int;
    }
    if s.is_null() {
        return options_get_number(
            global_s_options,
            b"status\0" as *const u8 as *const ::core::ffi::c_char,
        ) as u_int;
    }
    return (*s).statuslines;
}
pub unsafe fn status_prompt_line_at(mut c: *mut client) -> u_int {
    let mut s: *mut session = (*c).session;
    let mut line: u_int = 0;
    let mut lines: u_int = 0;
    lines = status_line_size(c);
    if lines == 0 as u_int {
        return 0 as u_int;
    }
    line = options_get_number(
        (*s).options,
        b"message-line\0" as *const u8 as *const ::core::ffi::c_char,
    ) as u_int;
    if line >= lines {
        return lines.wrapping_sub(1 as u_int);
    }
    return line;
}
pub unsafe fn status_get_range(mut c: *mut client, mut x: u_int, mut y: u_int) -> *mut style_range {
    let mut sl: *mut status_line = &raw mut (*c).status;
    if y as usize
        >= (::core::mem::size_of::<[style_line_entry; 5]>() as usize)
            .wrapping_div(::core::mem::size_of::<style_line_entry>() as usize)
    {
        return ::core::ptr::null_mut::<style_range>();
    }
    return style_ranges_get_range(
        &raw mut (*(&raw mut (*sl).entries as *mut style_line_entry).offset(y as isize)).ranges,
        x,
    );
}
unsafe fn status_push_screen(mut c: *mut client) {
    let mut sl: *mut status_line = &raw mut (*c).status;
    if (*sl).active.is_none() {
        (*sl).active = Some(Box::new(screen::empty()));
        screen_init(
            (*sl).active_screen(),
            (*c).tty.sx,
            status_line_size(c),
            0 as u_int,
        );
    }
    (*sl).screen_users += 1;
}
unsafe fn status_pop_screen(mut c: *mut client) {
    let mut sl: *mut status_line = &raw mut (*c).status;
    (*sl).screen_users -= 1;
    if (*sl).screen_users == 0 as ::core::ffi::c_int {
        let mut active = (*sl)
            .active
            .take()
            .expect("status screen push/pop balanced");
        screen_free(&mut *active);
    }
}
pub unsafe fn status_init(mut c: *mut client) {
    let mut sl: *mut status_line = &raw mut (*c).status;
    let mut i: u_int = 0;
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[style_line_entry; 5]>() as usize)
            .wrapping_div(::core::mem::size_of::<style_line_entry>() as usize)
    {
        style_ranges_init(
            &raw mut (*(&raw mut (*sl).entries as *mut style_line_entry).offset(i as isize)).ranges,
        );
        i = i.wrapping_add(1);
    }
    screen_init(&raw mut (*sl).screen, (*c).tty.sx, 1 as u_int, 0 as u_int);
    (*sl).active = None;
}
pub unsafe fn status_free(mut c: *mut client) {
    let mut sl: *mut status_line = &raw mut (*c).status;
    let mut i: u_int = 0;
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[style_line_entry; 5]>() as usize)
            .wrapping_div(::core::mem::size_of::<style_line_entry>() as usize)
    {
        style_ranges_free(
            &raw mut (*(&raw mut (*sl).entries as *mut style_line_entry).offset(i as isize)).ranges,
        );
        server_client_set_status_expanded(&mut *c, i as usize, None);
        i = i.wrapping_add(1);
    }
    if event_initialized(&(*sl).timer) != 0 {
        event_del(&raw mut (*sl).timer);
    }
    if let Some(mut active) = (*sl).active.take() {
        screen_free(&mut *active);
    }
    screen_free(&raw mut (*sl).screen);
}
pub unsafe fn status_redraw(mut c: *mut client) -> ::core::ffi::c_int {
    let mut sl: *mut status_line = &raw mut (*c).status;
    let mut sle: *mut style_line_entry = ::core::ptr::null_mut::<style_line_entry>();
    let mut s: *mut session = (*c).session;
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: ::core::ptr::null_mut::<window_pane>(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        item: None,
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
    let mut lines: u_int = 0;
    let mut i: u_int = 0;
    let mut n: u_int = 0;
    let mut width: u_int = (*c).tty.sx;
    let mut flags: ::core::ffi::c_int = 0;
    let mut force: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut changed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut fg: ::core::ffi::c_int = 0;
    let mut bg: ::core::ffi::c_int = 0;
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut ov: *mut options_value = ::core::ptr::null_mut::<options_value>();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    log_debug(format_args!("{} enter", "status_redraw"));
    if (*sl).active.is_some() {
        fatalx(|out| out.write_all(b"not the active screen"));
    }
    lines = status_line_size(c);
    if (*c).tty.sy == 0 as u_int || lines == 0 as u_int {
        return 1 as ::core::ffi::c_int;
    }
    flags = FORMAT_STATUS;
    if (*c).flags & CLIENT_STATUSFORCE as uint64_t != 0 {
        flags |= FORMAT_FORCE;
    }
    ft = format_create(c, ::core::ptr::null_mut::<cmdq_item>(), FORMAT_NONE, flags);
    format_defaults(
        ft,
        c,
        ::core::ptr::null_mut::<session>(),
        ::core::ptr::null_mut::<winlink>(),
        ::core::ptr::null_mut::<window_pane>(),
    );
    style_apply(
        &raw mut gc,
        (*s).options,
        b"status-style\0" as *const u8 as *const ::core::ffi::c_char,
        ft,
    );
    fg = options_get_number(
        (*s).options,
        b"status-fg\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    if !(fg == 8 as ::core::ffi::c_int || fg == 9 as ::core::ffi::c_int) {
        gc.fg = fg;
    }
    bg = options_get_number(
        (*s).options,
        b"status-bg\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    if !(bg == 8 as ::core::ffi::c_int || bg == 9 as ::core::ffi::c_int) {
        gc.bg = bg;
    }
    if !grid_cells_equal(&gc, &(*sl).style) {
        force = 1 as ::core::ffi::c_int;
        memcpy(
            &raw mut (*sl).style as *mut ::core::ffi::c_void,
            &raw mut gc as *const ::core::ffi::c_void,
            ::core::mem::size_of::<grid_cell>() as size_t,
        );
    }
    if (*sl).screen.grid().sx != width || (*sl).screen.grid().sy != lines {
        screen_resize(&raw mut (*sl).screen, width, lines, 0 as ::core::ffi::c_int);
        force = 1 as ::core::ffi::c_int;
        changed = force;
    }
    screen_write_start(&raw mut ctx, &raw mut (*sl).screen);
    o = options_get(
        (*s).options,
        b"status-format\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if o.is_null() {
        n = 0 as u_int;
        while n < width.wrapping_mul(lines) {
            screen_write_putc(&raw mut ctx, &raw mut gc, ' ' as i32 as u_char);
            n = n.wrapping_add(1);
        }
    } else {
        i = 0 as u_int;
        while i < lines {
            screen_write_cursormove(
                &raw mut ctx,
                0 as ::core::ffi::c_int,
                i as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            ov = options_array_get_index(o, i);
            if ov.is_null() {
                n = 0 as u_int;
                while n < width {
                    screen_write_putc(&raw mut ctx, &raw mut gc, ' ' as i32 as u_char);
                    n = n.wrapping_add(1);
                }
            } else {
                sle = (&raw mut (*sl).entries as *mut style_line_entry).offset(i as isize)
                    as *mut style_line_entry;
                let expanded = format_expand_time_cstring(ft, (*ov).string_ptr());
                if force != 0
                    || (*c).status.entries[i as usize].expanded.as_ref() != Some(&expanded)
                {
                    changed = 1 as ::core::ffi::c_int;
                    n = 0 as u_int;
                    while n < width {
                        screen_write_putc(&raw mut ctx, &raw mut gc, ' ' as i32 as u_char);
                        n = n.wrapping_add(1);
                    }
                    screen_write_cursormove(
                        &raw mut ctx,
                        0 as ::core::ffi::c_int,
                        i as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    );
                    style_ranges_clear(&raw mut (*sle).ranges);
                    format_draw(
                        &raw mut ctx,
                        &raw mut gc,
                        width,
                        expanded.as_ptr(),
                        &raw mut (*sle).ranges,
                        0 as ::core::ffi::c_int,
                    );
                    server_client_set_status_expanded(&mut *c, i as usize, Some(expanded));
                }
            }
            i = i.wrapping_add(1);
        }
    }
    screen_write_stop(&raw mut ctx);
    format_free(ft);
    log_debug(format_args!(
        "{} exit: force={}, changed={}",
        "status_redraw",
        (force) as i32,
        (changed) as i32
    ));
    return (force != 0 || changed != 0) as ::core::ffi::c_int;
}
fn status_message_escape(s: &CStr) -> CString {
    let source = s.to_bytes();
    let extra = source.iter().filter(|&&byte| byte == b'#').count();
    let mut escaped = Vec::with_capacity(source.len() + extra);
    for &byte in source {
        if byte == b'#' {
            escaped.push(b'#');
        }
        escaped.push(byte);
    }
    CString::new(escaped).expect("C string has no interior NUL")
}

#[cfg(test)]
mod status_message_escape_tests {
    use super::{status_message_escape, CString};

    #[test]
    fn doubles_hash_without_changing_other_bytes() {
        let input = CString::new(b"#\xff##tail".as_slice()).unwrap();
        let escaped = status_message_escape(input.as_c_str());
        assert_eq!(escaped.as_bytes(), b"##\xff####tail");
        assert!(status_message_escape(c"").as_bytes().is_empty());
    }
}
pub unsafe fn status_message_set(
    mut c: *mut client,
    mut delay: ::core::ffi::c_int,
    mut ignore_styles: ::core::ffi::c_int,
    mut ignore_keys: ::core::ffi::c_int,
    mut no_freeze: ::core::ffi::c_int,
    write: impl FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>,
) {
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    let s = format_message_with(write);
    log_debug(format_args!(
        "{}: {}",
        "status_message_set",
        log_cstr((s.as_ptr()) as *const _)
    ));
    if c.is_null() {
        server_add_message(|out| {
            out.write_all(b"message: ")?;
            write_cstr(out, s.as_ptr())
        });
        return;
    }
    status_message_clear(c);
    status_push_screen(c);
    server_client_set_message(&mut *c, Some(s));
    server_add_message(|out| {
        write_cstr(
            out,
            ((*c).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        )?;
        out.write_all(b" message: ")?;
        write_cstr(
            out,
            ((*c).message_string)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        )
    });
    if delay == -(1 as ::core::ffi::c_int) {
        delay = options_get_number(
            (*(*c).session).options,
            b"display-time\0" as *const u8 as *const ::core::ffi::c_char,
        ) as ::core::ffi::c_int;
    }
    if delay > 0 as ::core::ffi::c_int {
        tv.tv_sec = (delay / 1000 as ::core::ffi::c_int) as __time_t;
        tv.tv_usec = ((delay % 1000 as ::core::ffi::c_int) as ::core::ffi::c_long
            * 1000 as ::core::ffi::c_long) as __suseconds_t;
        if event_initialized(&(*c).message_timer) != 0 {
            event_del(&raw mut (*c).message_timer);
        }
        event_set(
            &raw mut (*c).message_timer,
            -(1 as ::core::ffi::c_int),
            0 as ::core::ffi::c_short,
            move |_, _| unsafe { status_message_callback(c as *mut ::core::ffi::c_void) },
        );
        event_add(&raw mut (*c).message_timer, &raw mut tv);
    }
    if delay != 0 as ::core::ffi::c_int {
        (*c).message_ignore_keys = ignore_keys;
    }
    (*c).message_ignore_styles = ignore_styles;
    if no_freeze == 0 {
        (*c).tty.flags |= TTY_FREEZE;
    }
    (*c).tty.flags |= TTY_NOCURSOR;
    (*c).flags |= CLIENT_REDRAWSTATUS as uint64_t;
}
pub unsafe fn status_message_clear(mut c: *mut client) {
    if (*c).message_string.is_none() {
        return;
    }
    server_client_set_message(&mut *c, None);
    if (*c).prompt.is_null() {
        (*c).tty.flags &= !(TTY_NOCURSOR | TTY_FREEZE);
    }
    (*c).flags |= CLIENT_ALLREDRAWFLAGS as uint64_t;
    status_pop_screen(c);
}
unsafe fn status_message_area(mut c: *mut client, mut area_x: *mut u_int, mut area_w: *mut u_int) {
    let mut s: *mut session = (*c).session;
    let mut sy: *mut style = ::core::ptr::null_mut::<style>();
    let mut w: u_int = 0;
    sy = options_string_to_style(
        (*s).options,
        b"message-style\0" as *const u8 as *const ::core::ffi::c_char,
        ::core::ptr::null_mut::<format_tree>(),
    );
    if !sy.is_null() && (*sy).width >= 0 as ::core::ffi::c_int {
        if (*sy).width_percentage != 0 {
            w = (*c)
                .tty
                .sx
                .wrapping_mul((*sy).width as u_int)
                .wrapping_div(100 as u_int);
        } else {
            w = (*sy).width as u_int;
        }
    } else {
        w = (*c).tty.sx;
    }
    if w == 0 as u_int || w > (*c).tty.sx {
        w = (*c).tty.sx;
    }
    if !sy.is_null() {
        match (*sy).align as ::core::ffi::c_uint {
            2 | 4 => {
                *area_x = (*c).tty.sx.wrapping_sub(w).wrapping_div(2 as u_int);
            }
            3 => {
                *area_x = (*c).tty.sx.wrapping_sub(w);
            }
            _ => {
                *area_x = 0 as u_int;
            }
        }
    } else {
        *area_x = 0 as u_int;
    }
    *area_w = w;
}
unsafe fn status_message_callback(mut data: *mut ::core::ffi::c_void) {
    let mut c: *mut client = data as *mut client;
    status_message_clear(c);
}
pub unsafe fn status_message_redraw(mut c: *mut client) -> ::core::ffi::c_int {
    let mut sl: *mut status_line = &raw mut (*c).status;
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: ::core::ptr::null_mut::<window_pane>(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        item: None,
        scrolled: 0,
        bg: 0,
    };
    let mut s: *mut session = (*c).session;
    let mut old_screen: screen = screen::empty();
    let mut lines: u_int = 0;
    let mut messageline: u_int = 0;
    let mut ax: u_int = 0;
    let mut aw: u_int = 0;
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
    let mut msgfmt: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if (*c).tty.sx == 0 as u_int || (*c).tty.sy == 0 as u_int {
        return 0 as ::core::ffi::c_int;
    }
    old_screen = std::mem::replace((*sl).active_screen(), screen::empty());
    lines = status_line_size(c);
    if lines <= 1 as u_int {
        lines = 1 as u_int;
    }
    screen_init((*sl).active_screen(), (*c).tty.sx, lines, 0 as u_int);
    messageline = status_prompt_line_at(c);
    if messageline > lines.wrapping_sub(1 as u_int) {
        messageline = lines.wrapping_sub(1 as u_int);
    }
    status_message_area(c, &raw mut ax, &raw mut aw);
    ft = format_create_defaults(
        ::core::ptr::null_mut::<cmdq_item>(),
        c,
        ::core::ptr::null_mut::<session>(),
        ::core::ptr::null_mut::<winlink>(),
        ::core::ptr::null_mut::<window_pane>(),
    );
    style_apply(
        &raw mut gc,
        (*s).options,
        b"message-style\0" as *const u8 as *const ::core::ffi::c_char,
        ft,
    );
    if (*c).message_ignore_styles != 0 {
        let msg = status_message_escape((*c).message_string.as_deref().unwrap_or(c""));
        format_add(
            ft,
            b"message\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write_cstr(out, msg.as_ptr()),
        );
    } else {
        format_add(
            ft,
            b"message\0" as *const u8 as *const ::core::ffi::c_char,
            |out| {
                write_cstr(
                    out,
                    ((*c).message_string)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                )
            },
        );
    }
    format_add(
        ft,
        b"command_prompt\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write!(out, "{}", (0 as ::core::ffi::c_int) as i32),
    );
    msgfmt = options_get_string(
        (*s).options,
        b"message-format\0" as *const u8 as *const ::core::ffi::c_char,
    );
    let expanded = format_expand_time_cstring(ft, msgfmt);
    format_free(ft);
    screen_write_start(&raw mut ctx, (*sl).active_screen());
    screen_write_fast_copy(
        &raw mut ctx,
        &raw mut (*sl).screen,
        0 as u_int,
        0 as u_int,
        (*c).tty.sx,
        lines,
    );
    screen_write_cursormove(
        &raw mut ctx,
        ax as ::core::ffi::c_int,
        messageline as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    format_draw(
        &raw mut ctx,
        &raw mut gc,
        aw,
        expanded.as_ptr(),
        ::core::ptr::null_mut::<style_ranges>(),
        0 as ::core::ffi::c_int,
    );
    screen_write_stop(&raw mut ctx);
    if grid_compare((*sl).active_screen().grid(), old_screen.grid()) == 0 as ::core::ffi::c_int {
        screen_free(&raw mut old_screen);
        return 0 as ::core::ffi::c_int;
    }
    screen_free(&raw mut old_screen);
    return 1 as ::core::ffi::c_int;
}
unsafe fn status_prompt_accept(mut c: *mut client) -> cmd_retval {
    if !(*c).prompt.is_null() {
        status_prompt_key(
            c,
            'y' as i32 as key_code,
            ::core::ptr::null_mut::<mouse_event>(),
        );
    }
    return CMD_RETURN_NORMAL;
}
pub unsafe fn status_prompt_set(
    mut c: *mut client,
    mut fs: *mut cmd_find_state,
    mut msg: *const ::core::ffi::c_char,
    mut input: *const ::core::ffi::c_char,
    mut inputcb: status_prompt_input_cb,
    mut freecb: prompt_free_cb,
    mut flags: ::core::ffi::c_int,
    mut prompt_type: prompt_type,
) {
    let mut pd = prompt_create_data::default();
    server_client_clear_overlay(c);
    status_message_clear(c);
    status_prompt_clear(c);
    status_push_screen(c);
    prompt_set_options(&raw mut pd, (*c).session);
    pd.fs = fs;
    pd.prompt = msg;
    pd.input = input;
    pd.type_0 = prompt_type;
    pd.flags = flags;
    if let Some(mut inputcb) = inputcb.take() {
        pd.inputcb = Some(Box::new(move |s, key| {
            inputcb(std::ptr::NonNull::new(c), s, key)
        }));
    }
    pd.freecb = freecb.take();
    (*c).prompt = prompt_create(&raw mut pd);
    if !flags & PROMPT_INCREMENTAL != 0 && !flags & PROMPT_NOFREEZE != 0 {
        (*c).tty.flags |= TTY_FREEZE;
    }
    (*c).flags |= CLIENT_REDRAWSTATUS as uint64_t;
    prompt_incremental_start((*c).prompt);
    if flags & PROMPT_SINGLE != 0 && flags & PROMPT_ACCEPT != 0 {
        cmdq_append(
            c,
            cmdq_get_callback_owned(
                b"status_prompt_accept\0" as *const u8 as *const ::core::ffi::c_char,
                Some(Box::new(move |_| unsafe { status_prompt_accept(c) })),
            ),
        );
    }
}
pub unsafe fn status_prompt_clear(mut c: *mut client) {
    if (*c).prompt.is_null() {
        return;
    }
    prompt_free((*c).prompt);
    (*c).prompt = ::core::ptr::null_mut::<prompt>();
    (*c).tty.flags &= !(TTY_NOCURSOR | TTY_FREEZE);
    (*c).flags |= CLIENT_ALLREDRAWFLAGS as uint64_t;
    status_pop_screen(c);
}
pub unsafe fn status_prompt_update(
    mut c: *mut client,
    mut msg: *const ::core::ffi::c_char,
    mut input: *const ::core::ffi::c_char,
) {
    if (*c).prompt.is_null() {
        return;
    }
    prompt_update((*c).prompt, msg, input);
    (*c).flags |= CLIENT_REDRAWSTATUS as uint64_t;
}
unsafe fn status_prompt_screen_line(mut c: *mut client) -> u_int {
    let mut tty: *mut tty = &raw mut (*c).tty;
    let mut n: u_int = 0;
    if options_get_number(
        (*(*c).session).options,
        b"status-position\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_longlong
    {
        return status_prompt_line_at(c);
    }
    n = status_line_size(c).wrapping_sub(status_prompt_line_at(c));
    if n <= (*tty).sy {
        return (*tty).sy.wrapping_sub(n);
    }
    return (*tty).sy.wrapping_sub(1 as u_int);
}
pub unsafe fn status_prompt_redraw(mut c: *mut client) -> ::core::ffi::c_int {
    let mut sl: *mut status_line = &raw mut (*c).status;
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: ::core::ptr::null_mut::<window_pane>(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        item: None,
        scrolled: 0,
        bg: 0,
    };
    let mut old_screen: screen = screen::empty();
    let mut pdd: prompt_draw_data = prompt_draw_data {
        ctx: ::core::ptr::null_mut::<screen_write_ctx>(),
        cursor_x: ::core::ptr::null_mut::<u_int>(),
        area_x: 0,
        area_width: 0,
        prompt_line: 0,
    };
    let mut lines: u_int = 0;
    let mut ax: u_int = 0;
    let mut aw: u_int = 0;
    let mut promptline: u_int = 0;
    if (*c).tty.sx == 0 as u_int || (*c).tty.sy == 0 as u_int {
        return 0 as ::core::ffi::c_int;
    }
    old_screen = std::mem::replace((*sl).active_screen(), screen::empty());
    lines = status_line_size(c);
    if lines <= 1 as u_int {
        lines = 1 as u_int;
    }
    screen_init((*sl).active_screen(), (*c).tty.sx, lines, 0 as u_int);
    promptline = status_prompt_line_at(c);
    if promptline > lines.wrapping_sub(1 as u_int) {
        promptline = lines.wrapping_sub(1 as u_int);
    }
    status_message_area(c, &raw mut ax, &raw mut aw);
    screen_write_start(&raw mut ctx, (*sl).active_screen());
    screen_write_fast_copy(
        &raw mut ctx,
        &raw mut (*sl).screen,
        0 as u_int,
        0 as u_int,
        (*c).tty.sx,
        lines,
    );
    pdd.ctx = &raw mut ctx;
    pdd.area_x = ax;
    pdd.area_width = aw;
    pdd.prompt_line = promptline;
    pdd.cursor_x = &raw mut (*sl).prompt_cx;
    prompt_draw((*c).prompt, &raw mut pdd);
    screen_write_stop(&raw mut ctx);
    if grid_compare((*sl).active_screen().grid(), old_screen.grid()) == 0 as ::core::ffi::c_int {
        screen_free(&raw mut old_screen);
        return 0 as ::core::ffi::c_int;
    }
    screen_free(&raw mut old_screen);
    return 1 as ::core::ffi::c_int;
}
pub unsafe fn status_prompt_cursor(mut c: *mut client, mut cx: *mut u_int, mut cy: *mut u_int) {
    *cy = status_prompt_screen_line(c);
    *cx = (*c).status.prompt_cx;
}
pub unsafe fn status_prompt_key(
    mut c: *mut client,
    mut key: key_code,
    mut m: *mut mouse_event,
) -> prompt_key_result {
    let mut result: prompt_key_result = PROMPT_KEY_NOT_HANDLED;
    let mut ax: u_int = 0;
    let mut aw: u_int = 0;
    let mut redraw: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
        == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
        || key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
            >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                << 32 as ::core::ffi::c_int
            && key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int
    {
        if m.is_null()
            || (*m).b & MOUSE_MASK_BUTTONS as u_int != MOUSE_BUTTON_1 as u_int
            || (*m).b & MOUSE_MASK_DRAG as u_int != 0
            || (*m).b & MOUSE_MASK_BUTTONS as u_int == 3 as u_int
            || (*m).y != status_prompt_screen_line(c)
        {
            return PROMPT_KEY_NOT_HANDLED;
        }
        status_message_area(c, &raw mut ax, &raw mut aw);
        result = prompt_mouse((*c).prompt, (*m).x, ax, aw, &raw mut redraw);
    } else {
        result = prompt_key((*c).prompt, key, &raw mut redraw);
    }
    if redraw != 0 && !(*c).prompt.is_null() {
        (*c).flags |= CLIENT_REDRAWSTATUS as uint64_t;
    }
    if !(*c).prompt.is_null() && prompt_closed((*c).prompt) != 0 {
        status_prompt_clear(c);
    }
    return result;
}

#[cfg(test)]
mod status_screen_tests {
    use super::*;
    use crate::src::grid::{grid_default_cell, grid_get_cell, grid_set_cell};
    use crate::src::options::{options_create, options_default, options_free};
    use crate::src::options_table::options_table;
    use crate::src::tmux::{global_options, global_s_options};

    #[test]
    fn temporary_screen_is_shared_until_last_pop_and_base_survives() {
        unsafe {
            let previous = (global_options, global_s_options);
            global_options = options_create(std::ptr::null_mut());
            global_s_options = options_create(std::ptr::null_mut());
            for (options, name) in [
                (global_options, c"extended-keys"),
                (global_s_options, c"status"),
            ] {
                let definition = options_table
                    .iter()
                    .find(|entry| !entry.name.is_null() && CStr::from_ptr(entry.name) == name)
                    .unwrap();
                options_default(options, definition);
            }

            let mut c = Box::new(client::empty());
            c.tty.sx = 80;
            c.tty.sy = 24;
            status_init(&mut *c);
            let base = &raw mut c.status.screen;
            let base_grid = c.status.screen.grid() as *const grid as usize;
            assert!(c.status.active.is_none());
            assert_eq!(c.status.active_screen() as *mut screen, base);
            let mut cell = grid_default_cell;
            cell.data.data[0] = b'B';
            grid_set_cell(c.status.screen.grid_mut(), 0, 0, &cell);

            // A prompt and a message share one temporary screen.
            status_push_screen(&mut *c);
            let temporary = c.status.active_screen() as *mut screen;
            let temporary_grid = c.status.active_screen().grid() as *const grid as usize;
            assert_ne!(temporary, base);
            assert_ne!(temporary_grid, base_grid);
            cell.data.data[0] = b'T';
            grid_set_cell(c.status.active_screen().grid_mut(), 0, 0, &cell);
            status_push_screen(&mut *c);
            assert_eq!(c.status.screen_users, 2);
            assert_eq!(c.status.active_screen() as *mut screen, temporary);
            status_pop_screen(&mut *c);
            assert_eq!(c.status.screen_users, 1);
            assert_eq!(c.status.active_screen().grid() as *const grid as usize, temporary_grid);
            grid_get_cell(c.status.active_screen().grid(), 0, 0, &mut cell);
            assert_eq!(cell.data.data[0], b'T');

            status_pop_screen(&mut *c);
            assert_eq!(c.status.screen_users, 0);
            assert!(c.status.active.is_none());
            assert_eq!(c.status.active_screen() as *mut screen, base);
            assert_eq!(c.status.screen.grid() as *const grid as usize, base_grid);
            grid_get_cell(c.status.screen.grid(), 0, 0, &mut cell);
            assert_eq!(cell.data.data[0], b'B');
            status_free(&mut *c);
            assert!(c.status.screen.grid.is_none());

            // Teardown also releases an active temporary screen without a pop.
            status_init(&mut *c);
            status_push_screen(&mut *c);
            assert!(c.status.active.is_some());
            status_free(&mut *c);
            assert!(c.status.active.is_none());
            assert!(c.status.screen.grid.is_none());

            options_free(global_options);
            options_free(global_s_options);
            (global_options, global_s_options) = previous;
        }
    }
}
