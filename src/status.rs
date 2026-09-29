use crate::src::options::options_owner_ptr;
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

unsafe fn status_timer_callback(c_owner: &std::rc::Rc<std::cell::UnsafeCell<client>>) {
    let mut c = c_owner.get();
    let mut s: *mut session = (*c).session_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    event_del(&raw mut (*c).status.timer);
    if s.is_null() {
        return;
    }
    if (*c).message_string.is_none() && (*c).prompt.is_none() {
        (*c).flags |= CLIENT_REDRAWSTATUS as uint64_t;
    }
    tv.tv_usec = 0 as __suseconds_t;
    tv.tv_sec = tv.tv_usec as __time_t;
    tv.tv_sec = options_get_number(
        options_owner_ptr(&mut (*s).options).map_or(std::ptr::null_mut(), |options| options),
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
pub unsafe fn status_timer_start(c_owner: &std::rc::Rc<std::cell::UnsafeCell<client>>) {
    let mut c = c_owner.get();
    let mut s: *mut session = (*c).session_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    if event_initialized(&(*c).status.timer) != 0 {
        event_del(&raw mut (*c).status.timer);
    } else {
        event_set(
            &raw mut (*c).status.timer,
            -(1 as ::core::ffi::c_int),
            0 as ::core::ffi::c_short,
            {
                let observer = std::rc::Rc::downgrade(c_owner);
                move |_, _| unsafe {
                    if let Some(owner) = observer.upgrade() {
                        status_timer_callback(&owner);
                    }
                }
            },
        );
    }
    if !s.is_null()
        && options_get_number(
            options_owner_ptr(&mut (*s).options).map_or(std::ptr::null_mut(), |options| options),
            b"status\0" as *const u8 as *const ::core::ffi::c_char,
        ) != 0
    {
        status_timer_callback(c_owner);
    }
}
pub unsafe fn status_timer_start_all() {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut registry_c_owner = clients.first();
    c = registry_c_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    while !c.is_null() {
        status_timer_start(&(*(c)).observer.upgrade().expect("live client"));
        registry_c_owner = clients.next(registry_c_owner.as_ref().expect("current registry client"));
        c = registry_c_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    }
}
pub unsafe fn status_update_cache(s_value: &mut session) {
    let s: *mut session = s_value as *mut _;
    (*s).statuslines = options_get_number(
        options_owner_ptr(&mut (*s).options).map_or(std::ptr::null_mut(), |options| options),
        b"status\0" as *const u8 as *const ::core::ffi::c_char,
    ) as u_int;
    if (*s).statuslines == 0 as u_int {
        (*s).statusat = -(1 as ::core::ffi::c_int);
    } else if options_get_number(
        options_owner_ptr(&mut (*s).options).map_or(std::ptr::null_mut(), |options| options),
        b"status-position\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_longlong
    {
        (*s).statusat = 0 as ::core::ffi::c_int;
    } else {
        (*s).statusat = 1 as ::core::ffi::c_int;
    };
}
pub unsafe fn status_at_line(c: &client) -> ::core::ffi::c_int {
    let mut s: *mut session = c.session_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    if c.flags & (CLIENT_STATUSOFF | CLIENT_CONTROL) as uint64_t != 0 {
        return -(1 as ::core::ffi::c_int);
    }
    if (*s).statusat != 1 as ::core::ffi::c_int {
        return (*s).statusat;
    }
    return c.tty.sy.wrapping_sub(status_line_size(c)) as ::core::ffi::c_int;
}
pub unsafe fn status_line_size(c: &client) -> u_int {
    let mut s: *mut session = c.session_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    if c.flags & (CLIENT_STATUSOFF | CLIENT_CONTROL) as uint64_t != 0 {
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
pub unsafe fn status_prompt_line_at(c: &client) -> u_int {
    let mut s: *mut session = c.session_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut line: u_int = 0;
    let mut lines: u_int = 0;
    lines = status_line_size(c);
    if lines == 0 as u_int {
        return 0 as u_int;
    }
    line = options_get_number(
        options_owner_ptr(&mut (*s).options).map_or(std::ptr::null_mut(), |options| options),
        b"message-line\0" as *const u8 as *const ::core::ffi::c_char,
    ) as u_int;
    if line >= lines {
        return lines.wrapping_sub(1 as u_int);
    }
    return line;
}
pub fn status_get_range(c: &client, x: u_int, y: u_int) -> Option<style_range> {
    c.status.entries.get(y as usize)?.ranges.as_slice().iter()
        .find(|range| x >= range.start && x < range.end)
        .map(|range| **range)
}
unsafe fn status_push_screen(c_value: &mut client) {
    let c: *mut client = c_value as *mut _;
    let mut sl: *mut status_line = &raw mut (*c).status;
    if (*sl).active.is_none() {
        (*sl).active = Some(Box::new(screen::empty()));
        let lines = status_line_size(&*c);
        screen_init(
            (*sl).active_screen(),
            (*c).tty.sx,
            lines,
            0 as u_int,
        );
    }
    (*sl).screen_users += 1;
}
unsafe fn status_pop_screen(c_value: &mut client) {
    let c: *mut client = c_value as *mut _;
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
pub unsafe fn status_init(c_value: &mut client) {
    let c: *mut client = c_value as *mut _;
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
    screen_init(&mut (*sl).screen, (*c).tty.sx, 1 as u_int, 0 as u_int);
    (*sl).active = None;
}
pub unsafe fn status_free(c_value: &mut client) {
    let c: *mut client = c_value as *mut _;
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
    screen_free(&mut (*sl).screen);
}
pub unsafe fn status_redraw(c_owner: &std::rc::Rc<std::cell::UnsafeCell<client>>) -> ::core::ffi::c_int {
    let mut c = c_owner.get();
    let mut sl: *mut status_line = &raw mut (*c).status;
    let mut sle: *mut style_line_entry = ::core::ptr::null_mut::<style_line_entry>();
    let mut s: *mut session = (*c).session_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: std::rc::Weak::new(),
        target: Default::default(),
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
    lines = status_line_size(&*c);
    if (*c).tty.sy == 0 as u_int || lines == 0 as u_int {
        return 1 as ::core::ffi::c_int;
    }
    flags = FORMAT_STATUS;
    if (*c).flags & CLIENT_STATUSFORCE as uint64_t != 0 {
        flags |= FORMAT_FORCE;
    }
    let mut ft_owner = format_create(Some(c_owner), None, FORMAT_NONE, flags);
    ft = &raw mut *ft_owner;
    format_defaults(
        ft,
        Some(c_owner),
        None,
        (refbox::Weak::new()).clone(),
        None,
    );
    style_apply(
        &raw mut gc,
        options_owner_ptr(&mut (*s).options).map_or(std::ptr::null_mut(), |options| options),
        b"status-style\0" as *const u8 as *const ::core::ffi::c_char,
        ft,
    );
    fg = options_get_number(
        options_owner_ptr(&mut (*s).options).map_or(std::ptr::null_mut(), |options| options),
        b"status-fg\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    if !(fg == 8 as ::core::ffi::c_int || fg == 9 as ::core::ffi::c_int) {
        gc.fg = fg;
    }
    bg = options_get_number(
        options_owner_ptr(&mut (*s).options).map_or(std::ptr::null_mut(), |options| options),
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
        screen_resize(&mut (*sl).screen, width, lines, 0 as ::core::ffi::c_int);
        force = 1 as ::core::ffi::c_int;
        changed = force;
    }
    screen_write_start(&mut ctx, &raw mut (*sl).screen);
    o = options_get(
        options_owner_ptr(&mut (*s).options).map_or(std::ptr::null_mut(), |options| options),
        b"status-format\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if o.is_null() {
        n = 0 as u_int;
        while n < width.wrapping_mul(lines) {
            screen_write_putc(&mut ctx, &gc, ' ' as i32 as u_char);
            n = n.wrapping_add(1);
        }
    } else {
        i = 0 as u_int;
        while i < lines {
            screen_write_cursormove(
                &mut ctx,
                0 as ::core::ffi::c_int,
                i as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            ov = crate::src::options::options_array_get_index_mut(&mut *(o), i).map_or(std::ptr::null_mut(), |value| value);
            if ov.is_null() {
                n = 0 as u_int;
                while n < width {
                    screen_write_putc(&mut ctx, &gc, ' ' as i32 as u_char);
                    n = n.wrapping_add(1);
                }
            } else {
                sle = (&raw mut (*sl).entries as *mut style_line_entry).offset(i as isize)
                    as *mut style_line_entry;
                let expanded = format_expand_time_cstring(ft, (*ov).string_ptr().map_or(std::ptr::null_mut(), |value| value.as_ptr().cast_mut()));
                if force != 0
                    || (*c).status.entries[i as usize].expanded.as_ref() != Some(&expanded)
                {
                    changed = 1 as ::core::ffi::c_int;
                    n = 0 as u_int;
                    while n < width {
                        screen_write_putc(&mut ctx, &gc, ' ' as i32 as u_char);
                        n = n.wrapping_add(1);
                    }
                    screen_write_cursormove(
                        &mut ctx,
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
    screen_write_stop(&mut ctx);
    format_free(ft_owner);
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
    c_owner: Option<&std::rc::Rc<std::cell::UnsafeCell<client>>>,
    mut delay: ::core::ffi::c_int,
    mut ignore_styles: ::core::ffi::c_int,
    mut ignore_keys: ::core::ffi::c_int,
    mut no_freeze: ::core::ffi::c_int,
    write: impl FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>,
) {
    let mut c = c_owner.map_or(std::ptr::null_mut(), |owner| owner.get());
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
    status_message_clear(&mut *c);
    status_push_screen(&mut *(c));
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
            options_owner_ptr(&mut (*(*c).session_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get())).options).map_or(std::ptr::null_mut(), |options| options),
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
            {
                let observer = std::rc::Rc::downgrade(c_owner.expect("message client checked above"));
                move |_, _| unsafe {
                    if let Some(owner) = observer.upgrade() {
                        status_message_callback(&owner);
                    }
                }
            },
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
pub unsafe fn status_message_clear(c: &mut client) {
    if (*c).message_string.is_none() {
        return;
    }
    server_client_set_message(&mut *c, None);
    if (*c).prompt.is_none() {
        (*c).tty.flags &= !(TTY_NOCURSOR | TTY_FREEZE);
    }
    (*c).flags |= CLIENT_ALLREDRAWFLAGS as uint64_t;
    status_pop_screen(&mut *(c));
}
unsafe fn status_message_area(c: &client) -> (u_int, u_int) {
    let sy = options_string_to_style(
        options_owner_ptr(&mut (*c.session_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get())).options).map_or(std::ptr::null_mut(), |options| options),
        c"message-style".as_ptr(),
        std::ptr::null_mut(),
    )
    .as_ref();
    let mut width = match sy.filter(|style| style.width >= 0) {
        Some(style) if style.width_percentage != 0 => {
            c.tty.sx.wrapping_mul(style.width as u_int) / 100
        }
        Some(style) => style.width as u_int,
        None => c.tty.sx,
    };
    if width == 0 || width > c.tty.sx {
        width = c.tty.sx;
    }
    let x = match sy.map(|style| style.align) {
        Some(STYLE_ALIGN_CENTRE | STYLE_ALIGN_ABSOLUTE_CENTRE) => c.tty.sx.wrapping_sub(width) / 2,
        Some(STYLE_ALIGN_RIGHT) => c.tty.sx.wrapping_sub(width),
        _ => 0,
    };
    (x, width)
}
unsafe fn status_message_callback(c_owner: &std::rc::Rc<std::cell::UnsafeCell<client>>) {
    let _c = c_owner.get();
    status_message_clear(&mut *(c_owner).get());
}
pub unsafe fn status_message_redraw(c_owner: &std::rc::Rc<std::cell::UnsafeCell<client>>) -> ::core::ffi::c_int {
    let mut c = c_owner.get();
    let mut sl: *mut status_line = &raw mut (*c).status;
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: std::rc::Weak::new(),
        target: Default::default(),
        flags: 0,
        init_ctx_cb: None,
        item: None,
        scrolled: 0,
        bg: 0,
    };
    let mut s: *mut session = (*c).session_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut old_screen: screen = screen::empty();
    let mut lines: u_int = 0;
    let mut messageline: u_int = 0;
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
    lines = status_line_size(&*c);
    if lines <= 1 as u_int {
        lines = 1 as u_int;
    }
    screen_init((*sl).active_screen(), (*c).tty.sx, lines, 0 as u_int);
    messageline = status_prompt_line_at(&*c);
    if messageline > lines.wrapping_sub(1 as u_int) {
        messageline = lines.wrapping_sub(1 as u_int);
    }
    let (ax, aw) = status_message_area(&*c);
    let mut ft_owner = format_create_defaults(
        None,
        Some(c_owner),
        None,
        (refbox::Weak::new()).clone(),
        None,
    );
    ft = &raw mut *ft_owner;
    style_apply(
        &raw mut gc,
        options_owner_ptr(&mut (*s).options).map_or(std::ptr::null_mut(), |options| options),
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
        options_owner_ptr(&mut (*s).options).map_or(std::ptr::null_mut(), |options| options),
        b"message-format\0" as *const u8 as *const ::core::ffi::c_char,
    );
    let expanded = format_expand_time_cstring(ft, msgfmt);
    format_free(ft_owner);
    screen_write_start(&mut ctx, (*sl).active_screen());
    screen_write_fast_copy(
        &mut ctx,
        &(*sl).screen,
        0 as u_int,
        0 as u_int,
        (*c).tty.sx,
        lines,
    );
    screen_write_cursormove(
        &mut ctx,
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
    screen_write_stop(&mut ctx);
    if grid_compare((*sl).active_screen().grid(), old_screen.grid()) == 0 as ::core::ffi::c_int {
        screen_free(&mut old_screen);
        return 0 as ::core::ffi::c_int;
    }
    screen_free(&mut old_screen);
    return 1 as ::core::ffi::c_int;
}
unsafe fn status_prompt_accept(c_owner: &std::rc::Rc<std::cell::UnsafeCell<client>>) -> cmd_retval {
    let mut c = c_owner.get();
    if (*c).prompt.is_some() {
        status_prompt_key(
            c_owner,
            'y' as i32 as key_code,
            ::core::ptr::null_mut::<mouse_event>(),
        );
    }
    return CMD_RETURN_NORMAL;
}
pub unsafe fn status_prompt_set(
    c_owner: &std::rc::Rc<std::cell::UnsafeCell<client>>,
    mut fs: *mut cmd_find_state,
    mut msg: *const ::core::ffi::c_char,
    mut input: *const ::core::ffi::c_char,
    mut inputcb: status_prompt_input_cb,
    mut freecb: prompt_free_cb,
    mut flags: ::core::ffi::c_int,
    mut prompt_type: prompt_type,
) {
    let mut c = c_owner.get();
    let mut pd = prompt_create_data::default();
    server_client_clear_overlay(c_owner);
    status_message_clear(&mut *(c_owner).get());
    status_prompt_clear(c_owner);
    status_push_screen(&mut *(c));
    prompt_set_options(&mut pd, ((*c).session_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get())).as_mut());
    pd.fs = fs.as_ref();
    pd.prompt = CStr::from_ptr(msg);
    pd.input = if input.is_null() {
        None
    } else {
        Some(CStr::from_ptr(input))
    };
    pd.type_0 = prompt_type;
    pd.flags = flags;
    if let Some(mut inputcb) = inputcb.take() {
        let client = (*c).observer.clone();
        pd.inputcb = Some(Box::new(move |s, key| {
            let owner = client.upgrade();
            inputcb(owner.as_ref(), s, key)
        }));
    }
    pd.freecb = freecb.take();
    let prompt = prompt_create(pd);
    let prompt_observer = prompt.downgrade();
    (*c).prompt = Some(prompt);
    let prompt = prompt_observer;
    if !flags & PROMPT_INCREMENTAL != 0 && !flags & PROMPT_NOFREEZE != 0 {
        (*c).tty.flags |= TTY_FREEZE;
    }
    (*c).flags |= CLIENT_REDRAWSTATUS as uint64_t;
    prompt_incremental_start(&prompt);
    if flags & PROMPT_SINGLE != 0 && flags & PROMPT_ACCEPT != 0 {
        cmdq_append(
            c.as_ref().map(|client| client.observer.upgrade().expect("queue client is live")).as_ref(),
            cmdq_get_callback_owned(
                c"status_prompt_accept",
                Some(Box::new({
                    let observer = std::rc::Rc::downgrade(c_owner);
                    move |_| unsafe {
                        observer.upgrade().map_or(CMD_RETURN_NORMAL, |owner| status_prompt_accept(&owner))
                    }
                })),
            ),
        );
    }
}
pub unsafe fn status_prompt_clear(c_owner: &std::rc::Rc<std::cell::UnsafeCell<client>>) {
    let mut c = c_owner.get();
    let Some(prompt) = (*c).prompt.take() else {
        return;
    };
    (*c).tty.flags &= !(TTY_NOCURSOR | TTY_FREEZE);
    (*c).flags |= CLIENT_ALLREDRAWFLAGS as uint64_t;
    status_pop_screen(&mut *(c));
    // The old screen and owner are detached before cleanup can install a new prompt.
    prompt_free(&prompt.downgrade());
}
pub unsafe fn status_prompt_update(
    c_owner: &std::rc::Rc<std::cell::UnsafeCell<client>>,
    mut msg: *const ::core::ffi::c_char,
    mut input: *const ::core::ffi::c_char,
) {
    let mut c = c_owner.get();
    if (*c).prompt.is_none() {
        return;
    }
    prompt_update(
        &mut (*c).prompt.as_ref().expect("active prompt").try_borrow_mut().expect("unborrowed prompt"),
        CStr::from_ptr(msg),
        (!input.is_null()).then(|| CStr::from_ptr(input)),
    );
    (*c).flags |= CLIENT_REDRAWSTATUS as uint64_t;
}
unsafe fn status_prompt_screen_line(c: &client) -> u_int {
    let tty = &c.tty;
    let mut n: u_int = 0;
    if options_get_number(
        options_owner_ptr(&mut (*c.session_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get())).options).map_or(std::ptr::null_mut(), |options| options),
        b"status-position\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_longlong
    {
        return status_prompt_line_at(c);
    }
    n = status_line_size(c).wrapping_sub(status_prompt_line_at(c));
    if n <= tty.sy {
        return tty.sy.wrapping_sub(n);
    }
    return tty.sy.wrapping_sub(1 as u_int);
}
pub unsafe fn status_prompt_redraw(c_owner: &std::rc::Rc<std::cell::UnsafeCell<client>>) -> ::core::ffi::c_int {
    let mut c = c_owner.get();
    let mut sl: *mut status_line = &raw mut (*c).status;
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: std::rc::Weak::new(),
        target: Default::default(),
        flags: 0,
        init_ctx_cb: None,
        item: None,
        scrolled: 0,
        bg: 0,
    };
    let mut old_screen: screen = screen::empty();
    let mut lines: u_int = 0;
    let mut promptline: u_int = 0;
    if (*c).tty.sx == 0 as u_int || (*c).tty.sy == 0 as u_int {
        return 0 as ::core::ffi::c_int;
    }
    old_screen = std::mem::replace((*sl).active_screen(), screen::empty());
    lines = status_line_size(&*c);
    if lines <= 1 as u_int {
        lines = 1 as u_int;
    }
    screen_init((*sl).active_screen(), (*c).tty.sx, lines, 0 as u_int);
    promptline = status_prompt_line_at(&*c);
    if promptline > lines.wrapping_sub(1 as u_int) {
        promptline = lines.wrapping_sub(1 as u_int);
    }
    let (ax, aw) = status_message_area(&*c);
    screen_write_start(&mut ctx, (*sl).active_screen());
    screen_write_fast_copy(
        &mut ctx,
        &(*sl).screen,
        0 as u_int,
        0 as u_int,
        (*c).tty.sx,
        lines,
    );
    let pdd = prompt_draw_data {
        area_x: ax,
        area_width: aw,
        prompt_line: promptline,
    };
    (*sl).prompt_cx = prompt_draw(
        &(*c).prompt.as_ref().expect("active prompt").try_borrow_mut().expect("unborrowed prompt"),
        &mut ctx,
        pdd,
    );
    screen_write_stop(&mut ctx);
    if grid_compare((*sl).active_screen().grid(), old_screen.grid()) == 0 as ::core::ffi::c_int {
        screen_free(&mut old_screen);
        return 0 as ::core::ffi::c_int;
    }
    screen_free(&mut old_screen);
    return 1 as ::core::ffi::c_int;
}
pub unsafe fn status_prompt_cursor(c: &client) -> (u_int, u_int) {
    let y = status_prompt_screen_line(c);
    (c.status.prompt_cx, y)
}
pub unsafe fn status_prompt_key(
    c_owner: &std::rc::Rc<std::cell::UnsafeCell<client>>,
    mut key: key_code,
    mut m: *mut mouse_event,
) -> prompt_key_result {
    let mut c = c_owner.get();
    let Some(prompt) = (*c).prompt.as_ref().map(|prompt| prompt.downgrade()) else {
        return PROMPT_KEY_NOT_HANDLED;
    };
    let mut result: prompt_key_result = PROMPT_KEY_NOT_HANDLED;
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
            || (*m).y != status_prompt_screen_line(&*c)
        {
            return PROMPT_KEY_NOT_HANDLED;
        }
        let (ax, aw) = status_message_area(&*c);
        result = prompt_mouse(
            &mut (*c).prompt.as_ref().expect("active prompt").try_borrow_mut().expect("unborrowed prompt"),
            (*m).x,
            ax,
            aw,
            Some(&mut redraw),
        );
    } else {
        result = prompt_key(&prompt, key, &mut redraw);
    }
    if redraw != 0 && (*c).prompt.is_some() {
        (*c).flags |= CLIENT_REDRAWSTATUS as uint64_t;
    }
    if (*c).prompt.is_some()
        && prompt_closed(&(*c).prompt.as_ref().expect("active prompt").try_borrow_mut().expect("unborrowed prompt")) != 0
    {
        status_prompt_clear(c_owner);
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
    fn message_area_and_cursor_preserve_width_alignment_and_status_position() {
        use crate::src::options::{options_set_number, options_set_string};

        unsafe {
            let mut oo_owner = options_create(std::ptr::null_mut());
            let oo = &raw mut *oo_owner;
            for name in [
                c"message-style",
                c"message-line",
                c"status-position",
                c"status",
            ] {
                let definition = (&*std::ptr::addr_of!(options_table))
                    .iter()
                    .find(|entry| entry.name == Some(name))
                    .unwrap();
                options_default(oo, definition);
            }
            let session_owner = session::new();
            let session = &mut *session_owner.get();
            session.options = Some(oo_owner);
            let mut c = client::empty();
            c.set_session(Some(session));
            c.tty.sx = 80;
            for (style, expected) in [
                ("default", (0, 80)),
                ("width=20,align=left", (0, 20)),
                ("width=21,align=centre", (29, 21)),
                ("width=21,align=absolute-centre", (29, 21)),
                ("width=21,align=right", (59, 21)),
                ("width=25%,align=centre", (30, 20)),
                ("width=0,align=right", (0, 80)),
                ("width=90,align=right", (0, 80)),
                ("invalid-style", (0, 80)),
            ] {
                options_set_string(oo, c"message-style".as_ptr(), 0, |out| {
                    out.write_all(style.as_bytes())
                });
                assert_eq!(status_message_area(&c), expected, "{style}");
            }
            c.tty.sx = 0;
            assert_eq!(status_message_area(&c), (0, 0));
            c.status.prompt_cx = 37;
            for (position, lines, message_line, height, flags, expected) in [
                (0, 3, 0, 24, 0, (0, 0)),
                (0, 3, 1, 24, 0, (0, 1)),
                (0, 3, 9, 24, 0, (0, 2)),
                (1, 3, 0, 24, 0, (21, 21)),
                (1, 3, 1, 24, 0, (21, 22)),
                (1, 3, 9, 24, 0, (21, 23)),
                (1, 3, 0, 1, 0, (-2, 0)),
                (1, 0, 0, 24, 0, (24, 24)),
                (0, 3, 1, 24, CLIENT_STATUSOFF, (-1, 0)),
                (1, 3, 1, 24, CLIENT_CONTROL, (-1, 24)),
            ] {
                session.statusat = position;
                session.statuslines = lines;
                options_set_number(oo, c"status-position".as_ptr(), position as i64);
                options_set_number(oo, c"message-line".as_ptr(), message_line);
                c.tty.sy = height;
                c.flags = flags as u64;
                assert_eq!(status_at_line(&c), expected.0);
                assert_eq!(status_prompt_cursor(&c), (37, expected.1));
            }
            let saved = global_s_options;
            global_s_options = oo;
            c.set_session(None);
            c.flags = 0;
            options_set_number(oo, c"status".as_ptr(), 4);
            assert_eq!(status_line_size(&c), 4);
            global_s_options = saved;
            // The session owns and releases its option table.
        }
    }

    #[test]
    fn temporary_screen_is_shared_until_last_pop_and_base_survives() {
        unsafe {
            let previous = (global_options, global_s_options);
            let mut global_options_owner = options_create(std::ptr::null_mut());
            global_options = &raw mut *global_options_owner;
            let mut global_s_options_owner = options_create(std::ptr::null_mut());
            global_s_options = &raw mut *global_s_options_owner;
            for (options, name) in [
                (global_options, c"extended-keys"),
                (global_s_options, c"status"),
            ] {
                let definition = options_table
                    .iter()
                    .find(|entry| entry.name == Some(name))
                    .unwrap();
                options_default(options, definition);
            }

            let mut c = Box::new(client::empty());
            c.tty.sx = 80;
            c.tty.sy = 24;
            status_init(&mut *(&mut *c));
            let base = &raw mut c.status.screen;
            let base_grid = c.status.screen.grid() as *const grid as usize;
            assert!(c.status.active.is_none());
            assert_eq!(c.status.active_screen() as *mut screen, base);
            let mut cell = grid_default_cell;
            cell.data.data[0] = b'B';
            grid_set_cell(c.status.screen.grid_mut(), 0, 0, &cell);

            // A prompt and a message share one temporary screen.
            status_push_screen(&mut *(&mut *c));
            let temporary = c.status.active_screen() as *mut screen;
            let temporary_grid = c.status.active_screen().grid() as *const grid as usize;
            assert_ne!(temporary, base);
            assert_ne!(temporary_grid, base_grid);
            cell.data.data[0] = b'T';
            grid_set_cell(c.status.active_screen().grid_mut(), 0, 0, &cell);
            status_push_screen(&mut *(&mut *c));
            assert_eq!(c.status.screen_users, 2);
            assert_eq!(c.status.active_screen() as *mut screen, temporary);
            status_pop_screen(&mut *(&mut *c));
            assert_eq!(c.status.screen_users, 1);
            assert_eq!(c.status.active_screen().grid() as *const grid as usize, temporary_grid);
            grid_get_cell(c.status.active_screen().grid(), 0, 0, &mut cell);
            assert_eq!(cell.data.data[0], b'T');

            status_pop_screen(&mut *(&mut *c));
            assert_eq!(c.status.screen_users, 0);
            assert!(c.status.active.is_none());
            assert_eq!(c.status.active_screen() as *mut screen, base);
            assert_eq!(c.status.screen.grid() as *const grid as usize, base_grid);
            grid_get_cell(c.status.screen.grid(), 0, 0, &mut cell);
            assert_eq!(cell.data.data[0], b'B');
            status_free(&mut *(&mut *c));
            assert!(c.status.screen.grid.is_none());

            // Teardown also releases an active temporary screen without a pop.
            status_init(&mut *(&mut *c));
            status_push_screen(&mut *(&mut *c));
            assert!(c.status.active.is_some());
            status_free(&mut *(&mut *c));
            assert!(c.status.active.is_none());
            assert!(c.status.screen.grid.is_none());

            // A cleanup callback may clear the old prompt recursively and
            // install a replacement. The outer clear must leave its screen.
            let c_owner = client::new();
            let c = c_owner.get();
            (*c).tty.sx = 80;
            (*c).tty.sy = 24;
            status_init(&mut *(&mut *c));
            status_push_screen(&mut *(&mut *c));
            let old = refbox::RefBox::new(prompt::default());
            let replacement = refbox::RefBox::new(prompt::default());
            let replacement_observer = replacement.downgrade();
            let next = replacement;
            let client = &raw mut *c;
            old.try_borrow_mut().unwrap().freecb = Some(Box::new(move || {
                status_prompt_clear(&(*(client)).observer.upgrade().expect("live client"));
                status_push_screen(&mut *(client));
                (*client).prompt = Some(next);
                (*client).tty.flags |= TTY_FREEZE;
            }));
            (*c).prompt = Some(old);
            status_prompt_clear(&(*(&mut *c)).observer.upgrade().expect("live client"));
            assert!(replacement_observer.is((*c).prompt.as_ref().unwrap()));
            assert_eq!((*c).status.screen_users, 1);
            assert!((*c).status.active.is_some());
            assert_ne!((*c).tty.flags & TTY_FREEZE, 0);
            status_prompt_clear(&(*(&mut *c)).observer.upgrade().expect("live client"));
            assert!((*c).prompt.is_none());
            assert_eq!((*c).status.screen_users, 0);
            assert!((*c).status.active.is_none());
            status_free(&mut *(&mut *c));

            options_free(global_options_owner);
            options_free(global_s_options_owner);
            (global_options, global_s_options) = previous;
        }
    }
}
