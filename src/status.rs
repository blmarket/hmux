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
use crate::src::options::options_owner_ptr;
use crate::src::options::{
    options_array_get_index, options_get_number, options_get_string, options_string_to_style,
    OptionsScope,
};
use crate::src::prompt::{
    prompt_closed, prompt_create, prompt_draw, prompt_free, prompt_incremental_start, prompt_key,
    prompt_mouse, prompt_set_options, prompt_update,
};
use crate::src::screen::{screen_free, screen_init, screen_resize};
use crate::src::screen_write::{
    screen_write_cursormove, screen_write_fast_copy, screen_write_putc, screen_write_start,
    screen_write_stop,
};
use crate::src::server::clients;
use crate::src::server::server_add_message;
use crate::src::server_client::server_client_clear_overlay;
use crate::src::server_client::Client as _;
use crate::src::session::Session as _;
use crate::src::shared::client::ClientRef;
use crate::src::shared::prompt::{prompt_create_data, prompt_draw_data};
use crate::src::shared::session::SessionRef;
use crate::src::style::{
    style_apply, style_ranges_clear, style_ranges_free, style_ranges_get_range, style_ranges_init,
};
use crate::src::tmux::global_s_options;
use std::ffi::{CStr, CString};
use std::time::Duration;

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

unsafe fn status_timer_callback(client: &ClientRef) {
    let session = client.attached_session().upgrade();
    {
        let mut status = client.borrow_status_mut();
        status.timer.cancel();
    }
    let Some(session) = session else { return };
    client.redraw_status_if_unobscured();
    let timeout = Duration::from_secs(
        (session
            .with_options_mut(|options| options_get_number(options, c"status-interval".as_ptr())))
            as u64,
    );
    if timeout.as_secs() != 0 {
        let mut status = client.borrow_status_mut();
        status.timer.arm(timeout).expect("arm timer");
    }
    log_debug(format_args!(
        "client {}, status interval {}",
        log_pointer(std::rc::Rc::as_ptr(client).cast()),
        timeout.as_secs() as i32
    ));
}
pub unsafe fn status_timer_start(client: &ClientRef) {
    let session = client.attached_session().upgrade();
    {
        let mut status = client.borrow_status_mut();
        if status.timer.is_initialized() {
            status.timer.cancel();
        } else {
            let observer = std::rc::Rc::downgrade(client);
            status.timer.set(move || unsafe {
                if let Some(owner) = observer.upgrade() {
                    status_timer_callback(&owner);
                }
            });
        }
    }
    if session.is_some_and(|session| {
        session.with_options_mut(|options| options_get_number(options, c"status".as_ptr())) != 0
    }) {
        status_timer_callback(client);
    }
}
pub unsafe fn status_timer_start_all() {
    let mut c: Option<ClientRef> = None;
    let mut registry_c_owner = clients.first();
    c = registry_c_owner.clone();
    while !c.is_none() {
        status_timer_start(&c.clone().expect("live client"));
        registry_c_owner =
            clients.next(registry_c_owner.as_ref().expect("current registry client"));
        c = registry_c_owner.clone();
    }
}
pub unsafe fn status_at_line(c: &ClientRef) -> ::core::ffi::c_int {
    use crate::src::session::Session;
    if c.flags() & (CLIENT_STATUSOFF | CLIENT_CONTROL) as uint64_t != 0 {
        return -1;
    }
    let session = c
        .attached_session()
        .upgrade()
        .expect("status client session");
    let (position, _) = session.status_layout();
    if position != 1 {
        return position;
    }
    c.terminal_size().1.wrapping_sub(status_line_size(c)) as ::core::ffi::c_int
}
pub unsafe fn status_line_size(c: &ClientRef) -> u_int {
    use crate::src::session::Session;
    if c.flags() & (CLIENT_STATUSOFF | CLIENT_CONTROL) as uint64_t != 0 {
        return 0;
    }
    match c.attached_session().upgrade() {
        Some(session) => session.status_layout().1,
        None => options_get_number(global_s_options, c"status".as_ptr()) as u_int,
    }
}
pub unsafe fn status_prompt_line_at(c: &ClientRef) -> u_int {
    let mut s: Option<SessionRef> = c.attached_session().upgrade();
    let mut line: u_int = 0;
    let mut lines: u_int = 0;
    lines = status_line_size(c);
    if lines == 0 as u_int {
        return 0 as u_int;
    }
    line = s
        .as_ref()
        .expect("live session")
        .with_options_mut(|options| {
            options_get_number(
                options,
                b"message-line\0" as *const u8 as *const ::core::ffi::c_char,
            )
        }) as u_int;
    if line >= lines {
        return lines.wrapping_sub(1 as u_int);
    }
    return line;
}
pub unsafe fn status_get_range(c: &ClientRef, x: u_int, y: u_int) -> Option<style_range> {
    c.borrow_status()
        .entries
        .get(y as usize)?
        .ranges
        .as_slice()
        .iter()
        .find(|range| x >= range.start && x < range.end)
        .map(|range| **range)
}
pub(crate) unsafe fn status_push_screen(client: &ClientRef) {
    let needs_screen = { client.borrow_status().active.is_none() };
    if needs_screen {
        let lines = status_line_size(client);
        let (width, _) = client.terminal_size();
        let mut status = client.borrow_status_mut();
        status.active = Some(Box::new(screen::empty()));
        screen_init(status.active_screen(), width, lines, 0);
    }
    client.borrow_status_mut().screen_users += 1;
}

pub(crate) unsafe fn status_pop_screen(client: &ClientRef) {
    let active = {
        let mut status = client.borrow_status_mut();
        status.screen_users -= 1;
        (status.screen_users == 0).then(|| {
            status
                .active
                .take()
                .expect("status screen push/pop balanced")
        })
    };
    if let Some(mut active) = active {
        screen_free(&mut active);
    }
}
pub unsafe fn status_init(status: &mut status_line, width: u_int) {
    let sl = status as *mut status_line;
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
    screen_init(&mut (*sl).screen, width, 1, 0);
    (*sl).active = None;
}
pub unsafe fn status_free(status: &mut status_line) {
    let sl = status as *mut status_line;
    let mut i: u_int = 0;
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[style_line_entry; 5]>() as usize)
            .wrapping_div(::core::mem::size_of::<style_line_entry>() as usize)
    {
        style_ranges_free(
            &raw mut (*(&raw mut (*sl).entries as *mut style_line_entry).offset(i as isize)).ranges,
        );
        (*sl).entries[i as usize].expanded = None;
        i = i.wrapping_add(1);
    }
    if (*sl).timer.is_initialized() {
        (*sl).timer.cancel();
    }
    if let Some(mut active) = (*sl).active.take() {
        screen_free(&mut *active);
    }
    screen_free(&mut (*sl).screen);
}
pub unsafe fn status_redraw(c_owner: &ClientRef) -> ::core::ffi::c_int {
    let mut c: Option<ClientRef> = Some(c_owner.clone());

    let mut s: Option<SessionRef> = c
        .as_ref()
        .expect("live client")
        .attached_session()
        .upgrade();
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
    let (width, terminal_height) = c_owner.terminal_size();
    let mut flags: ::core::ffi::c_int = 0;
    let mut force: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut changed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut fg: ::core::ffi::c_int = 0;
    let mut bg: ::core::ffi::c_int = 0;
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut ov: *mut options_value = ::core::ptr::null_mut::<options_value>();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    log_debug(format_args!("{} enter", "status_redraw"));
    if c_owner.borrow_status().active.is_some() {
        fatalx(|out| out.write_all(b"not the active screen"));
    }
    lines = status_line_size(c.as_ref().expect("live client"));
    if terminal_height == 0 as u_int || lines == 0 as u_int {
        return 1 as ::core::ffi::c_int;
    }
    flags = FORMAT_STATUS;
    if c.as_ref().expect("live client").flags() & CLIENT_STATUSFORCE as uint64_t != 0 {
        flags |= FORMAT_FORCE;
    }
    let mut ft_owner = format_create(Some(c_owner), None, FORMAT_NONE, flags);
    ft = &raw mut *ft_owner;
    format_defaults(ft, Some(c_owner), None, (refbox::Weak::new()).clone(), None);
    crate::src::style::style_apply_with_options(&mut gc, c"status-style", ft.as_mut(), |visit| {
        s.as_ref().expect("live session").with_options_mut(visit)
    });
    fg = s
        .as_ref()
        .expect("live session")
        .with_options_mut(|options| {
            options_get_number(
                options,
                b"status-fg\0" as *const u8 as *const ::core::ffi::c_char,
            )
        }) as ::core::ffi::c_int;
    if !(fg == 8 as ::core::ffi::c_int || fg == 9 as ::core::ffi::c_int) {
        gc.fg = fg;
    }
    bg = s
        .as_ref()
        .expect("live session")
        .with_options_mut(|options| {
            options_get_number(
                options,
                b"status-bg\0" as *const u8 as *const ::core::ffi::c_char,
            )
        }) as ::core::ffi::c_int;
    if !(bg == 8 as ::core::ffi::c_int || bg == 9 as ::core::ffi::c_int) {
        gc.bg = bg;
    }
    {
        let mut status = c_owner.borrow_status_mut();
        let sl: *mut status_line = &mut *status;
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
        // Keep pending collection state, but never the borrowed screen pointer,
        // across option access or format callbacks. Resume under a fresh guard.
        ctx.target = Default::default();
    }
    let format_owner =
        OptionsScope::Session(std::rc::Rc::downgrade(s.as_ref().expect("live session")))
            .resolve(c"status-format", false);
    if format_owner.is_none() {
        let mut status = c_owner.borrow_status_mut();
        ctx.borrow_screen(&mut status.screen);
        n = 0 as u_int;
        while n < width.wrapping_mul(lines) {
            screen_write_putc(&mut ctx, &gc, ' ' as i32 as u_char);
            n = n.wrapping_add(1);
        }
        ctx.target = Default::default();
    } else {
        i = 0 as u_int;
        while i < lines {
            {
                let mut status = c_owner.borrow_status_mut();
                ctx.borrow_screen(&mut status.screen);
                screen_write_cursormove(
                    &mut ctx,
                    0 as ::core::ffi::c_int,
                    i as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
                ctx.target = Default::default();
            }
            let line_format = format_owner
                .as_ref()
                .expect("resolved format owner")
                .with_entry(c"status-format", |entry| {
                    crate::src::options::options_array_get_index_mut(entry, i)
                        .map(|value| value.string_ptr().map(|text| text.to_owned()))
                })
                .expect("status format stays live during expansion");
            if line_format.is_none() {
                let mut status = c_owner.borrow_status_mut();
                ctx.borrow_screen(&mut status.screen);
                n = 0 as u_int;
                while n < width {
                    screen_write_putc(&mut ctx, &gc, ' ' as i32 as u_char);
                    n = n.wrapping_add(1);
                }
                ctx.target = Default::default();
            } else {
                let expanded = format_expand_time_cstring(
                    ft,
                    line_format
                        .as_ref()
                        .and_then(|value| value.as_ref())
                        .map_or(std::ptr::null(), |value| value.as_ptr()),
                );
                let mut status = c_owner.borrow_status_mut();
                ctx.borrow_screen(&mut status.screen);
                let sle = &mut status.entries[i as usize] as *mut style_line_entry;
                if force != 0 || (*sle).expanded.as_ref() != Some(&expanded) {
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
                    (*sle).expanded = Some(expanded);
                }
                ctx.target = Default::default();
            }
            i = i.wrapping_add(1);
        }
    }
    {
        let mut status = c_owner.borrow_status_mut();
        ctx.borrow_screen(&mut status.screen);
        screen_write_stop(&mut ctx);
    }
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
    c_owner: Option<&ClientRef>,
    mut delay: ::core::ffi::c_int,
    mut ignore_styles: ::core::ffi::c_int,
    mut ignore_keys: ::core::ffi::c_int,
    mut no_freeze: ::core::ffi::c_int,
    write: impl FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>,
) {
    let message = format_message_with(write);
    log_debug(format_args!(
        "status_message_set: {}",
        log_cstr(message.as_ptr())
    ));
    if let Some(client) = c_owner {
        client.show_status_message(message, delay, ignore_styles, ignore_keys, no_freeze);
    } else {
        server_add_message(|out| {
            out.write_all(b"message: ")?;
            write_cstr(out, message.as_ptr())
        });
    }
}
pub unsafe fn status_message_clear(client: &ClientRef) {
    client.clear_status_message();
}

unsafe fn status_message_area(c: &ClientRef) -> (u_int, u_int) {
    let (terminal_width, _) = c.terminal_size();
    let sy = crate::src::style::style_resolve_with_options(c"message-style", None, |visit| {
        c.attached_session()
            .upgrade()
            .expect("live session")
            .with_options_mut(visit)
    });
    let mut width = match sy.filter(|style| style.width >= 0) {
        Some(style) if style.width_percentage != 0 => {
            terminal_width.wrapping_mul(style.width as u_int) / 100
        }
        Some(style) => style.width as u_int,
        None => terminal_width,
    };
    if width == 0 || width > terminal_width {
        width = terminal_width;
    }
    let x = match sy.map(|style| style.align) {
        Some(STYLE_ALIGN_CENTRE | STYLE_ALIGN_ABSOLUTE_CENTRE) => {
            terminal_width.wrapping_sub(width) / 2
        }
        Some(STYLE_ALIGN_RIGHT) => terminal_width.wrapping_sub(width),
        _ => 0,
    };
    (x, width)
}
pub unsafe fn status_message_redraw(c_owner: &ClientRef) -> ::core::ffi::c_int {
    let mut msgfmt_session_value: Option<std::ffi::CString> = None;

    let mut c: Option<ClientRef> = Some(c_owner.clone());
    let (terminal_width, terminal_height) = c_owner.terminal_size();
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: std::rc::Weak::new(),
        target: Default::default(),
        flags: 0,
        init_ctx_cb: None,
        item: None,
        scrolled: 0,
        bg: 0,
    };
    let mut s: Option<SessionRef> = c
        .as_ref()
        .expect("live client")
        .attached_session()
        .upgrade();
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
    if terminal_width == 0 as u_int || terminal_height == 0 as u_int {
        return 0 as ::core::ffi::c_int;
    }
    old_screen = std::mem::replace(c_owner.borrow_status_mut().active_screen(), screen::empty());
    lines = status_line_size(c.as_ref().expect("live client"));
    if lines <= 1 as u_int {
        lines = 1 as u_int;
    }
    screen_init(
        c_owner.borrow_status_mut().active_screen(),
        terminal_width,
        lines,
        0,
    );
    messageline = status_prompt_line_at(c.as_ref().expect("live client"));
    if messageline > lines.wrapping_sub(1 as u_int) {
        messageline = lines.wrapping_sub(1 as u_int);
    }
    let (ax, aw) = status_message_area(c.as_ref().expect("live client"));
    let mut ft_owner = format_create_defaults(
        None,
        Some(c_owner),
        None,
        (refbox::Weak::new()).clone(),
        None,
    );
    ft = &raw mut *ft_owner;
    crate::src::style::style_apply_with_options(&mut gc, c"message-style", ft.as_mut(), |visit| {
        s.as_ref().expect("live session").with_options_mut(visit)
    });
    let (message, ignore_styles) = c_owner.status_message_text();
    if ignore_styles {
        let msg = status_message_escape(message.as_deref().unwrap_or(c""));
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
                    (message)
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
    msgfmt_session_value = Some(
        s.as_ref()
            .expect("live session")
            .with_options_mut(|options| {
                options_get_string(
                    options,
                    b"message-format\0" as *const u8 as *const ::core::ffi::c_char,
                )
            }),
    );
    msgfmt = msgfmt_session_value
        .as_ref()
        .expect("option snapshot")
        .as_ptr();
    let expanded = format_expand_time_cstring(ft, msgfmt);
    format_free(ft_owner);
    // Formatting above may reenter Client. Only component-only screen drawing
    // follows; with no pane or init callback it cannot dispatch terminal output.
    let mut status = c_owner.borrow_status_mut();
    let sl: *mut status_line = &mut *status;
    screen_write_start(&mut ctx, (*sl).active_screen());
    screen_write_fast_copy(
        &mut ctx,
        &(*sl).screen,
        0 as u_int,
        0 as u_int,
        terminal_width,
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
    let changed = grid_compare((*sl).active_screen().grid(), old_screen.grid()) != 0;
    drop(status);
    screen_free(&mut old_screen);
    i32::from(changed)
}
unsafe fn status_prompt_accept(c_owner: &ClientRef) -> cmd_retval {
    let mut c: Option<ClientRef> = Some(c_owner.clone());
    if c_owner.prompt_observer().is_alive() {
        status_prompt_key(
            c_owner,
            'y' as i32 as key_code,
            ::core::ptr::null_mut::<mouse_event>(),
        );
    }
    return CMD_RETURN_NORMAL;
}
pub unsafe fn status_prompt_set(
    c_owner: &ClientRef,
    mut fs: *mut cmd_find_state,
    mut msg: *const ::core::ffi::c_char,
    mut input: *const ::core::ffi::c_char,
    mut inputcb: status_prompt_input_cb,
    mut freecb: prompt_free_cb,
    mut flags: ::core::ffi::c_int,
    mut prompt_type: prompt_type,
) {
    let mut c: Option<ClientRef> = Some(c_owner.clone());
    let mut pd = prompt_create_data::default();
    server_client_clear_overlay(c_owner);
    status_message_clear(c_owner);
    status_prompt_clear(c_owner);
    status_push_screen(c.as_ref().expect("live client"));
    prompt_set_options(
        &mut pd,
        c.as_ref()
            .expect("live client")
            .attached_session()
            .upgrade()
            .as_ref(),
    );
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
        let client = std::rc::Rc::downgrade(c_owner);
        pd.inputcb = Some(Box::new(move |s, key| {
            let owner = client.upgrade();
            inputcb(owner.as_ref(), s, key)
        }));
    }
    pd.freecb = freecb.take();
    let prompt = prompt_create(pd);
    let prompt_observer = prompt.downgrade();
    c_owner.install_prompt(prompt, flags);
    let prompt = prompt_observer;
    prompt_incremental_start(&prompt);
    if flags & PROMPT_SINGLE != 0 && flags & PROMPT_ACCEPT != 0 {
        cmdq_append(
            c.as_ref(),
            cmdq_get_callback_owned(
                c"status_prompt_accept",
                Some(Box::new({
                    let observer = std::rc::Rc::downgrade(c_owner);
                    move |_| unsafe {
                        observer
                            .upgrade()
                            .map_or(CMD_RETURN_NORMAL, |owner| status_prompt_accept(&owner))
                    }
                })),
            ),
        );
    }
}
pub unsafe fn status_prompt_clear(c_owner: &ClientRef) {
    c_owner.clear_prompt();
}

pub unsafe fn status_prompt_update(
    c_owner: &ClientRef,
    mut msg: *const ::core::ffi::c_char,
    mut input: *const ::core::ffi::c_char,
) {
    let mut c: Option<ClientRef> = Some(c_owner.clone());
    let prompt = c_owner.prompt_observer();
    if !prompt.is_alive() {
        return;
    }
    prompt_update(
        &mut prompt.try_borrow_mut().expect("unborrowed prompt"),
        CStr::from_ptr(msg),
        (!input.is_null()).then(|| CStr::from_ptr(input)),
    );
    c.as_ref()
        .expect("live client")
        .update_flags(CLIENT_REDRAWSTATUS as uint64_t, 0);
}
unsafe fn status_prompt_screen_line(c: &ClientRef) -> u_int {
    let (_, height) = c.terminal_size();
    let mut n: u_int = 0;
    if c.attached_session()
        .upgrade()
        .expect("live session")
        .with_options_mut(|options| options_get_number(options, c"status-position".as_ptr()))
        == 0 as ::core::ffi::c_longlong
    {
        return status_prompt_line_at(c);
    }
    n = status_line_size(c).wrapping_sub(status_prompt_line_at(c));
    if n <= height {
        return height.wrapping_sub(n);
    }
    return height.wrapping_sub(1 as u_int);
}
pub unsafe fn status_prompt_redraw(c_owner: &ClientRef) -> ::core::ffi::c_int {
    let mut c: Option<ClientRef> = Some(c_owner.clone());
    let (terminal_width, terminal_height) = c_owner.terminal_size();
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
    if terminal_width == 0 as u_int || terminal_height == 0 as u_int {
        return 0 as ::core::ffi::c_int;
    }
    old_screen = std::mem::replace(c_owner.borrow_status_mut().active_screen(), screen::empty());
    lines = status_line_size(c.as_ref().expect("live client"));
    if lines <= 1 as u_int {
        lines = 1 as u_int;
    }
    screen_init(
        c_owner.borrow_status_mut().active_screen(),
        terminal_width,
        lines,
        0,
    );
    promptline = status_prompt_line_at(c.as_ref().expect("live client"));
    if promptline > lines.wrapping_sub(1 as u_int) {
        promptline = lines.wrapping_sub(1 as u_int);
    }
    let (ax, aw) = status_message_area(c.as_ref().expect("live client"));
    {
        let mut status = c_owner.borrow_status_mut();
        let sl: *mut status_line = &mut *status;
        screen_write_start(&mut ctx, (*sl).active_screen());
        screen_write_fast_copy(
            &mut ctx,
            &(*sl).screen,
            0 as u_int,
            0 as u_int,
            terminal_width,
            lines,
        );
        ctx.target = Default::default();
    }
    let pdd = prompt_draw_data {
        area_x: ax,
        area_width: aw,
        prompt_line: promptline,
    };
    let prompt = c_owner.prompt_observer();
    let prompt = prompt.try_borrow_mut().expect("unborrowed active prompt");
    {
        let mut status = c_owner.borrow_status_mut();
        crate::src::prompt::prompt_draw_cursor_style(&prompt, status.active_screen());
    }
    // Cursor defaults and the background copy are visible to format callbacks,
    // but no Client screen pointer or guard survives into expansion.
    let prepared = crate::src::prompt::prompt_prepare_draw(&prompt, ax, aw);
    let mut status = c_owner.borrow_status_mut();
    let sl: *mut status_line = &mut *status;
    ctx.borrow_screen((*sl).active_screen());
    (*sl).prompt_cx = crate::src::prompt::prompt_draw_prepared(&prompt, &mut ctx, pdd, prepared);
    drop(prompt);
    screen_write_stop(&mut ctx);
    let changed = grid_compare((*sl).active_screen().grid(), old_screen.grid()) != 0;
    drop(status);
    screen_free(&mut old_screen);
    i32::from(changed)
}
pub unsafe fn status_prompt_cursor(c: &ClientRef) -> (u_int, u_int) {
    let y = status_prompt_screen_line(c);
    (c.borrow_status().prompt_cx, y)
}
pub unsafe fn status_prompt_key(
    c_owner: &ClientRef,
    mut key: key_code,
    mut m: *mut mouse_event,
) -> prompt_key_result {
    let mut c: Option<ClientRef> = Some(c_owner.clone());
    let prompt = c_owner.prompt_observer();
    if !prompt.is_alive() {
        return PROMPT_KEY_NOT_HANDLED;
    }
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
            || (*m).y != status_prompt_screen_line(c.as_ref().expect("live client"))
        {
            return PROMPT_KEY_NOT_HANDLED;
        }
        let (ax, aw) = status_message_area(c.as_ref().expect("live client"));
        result = prompt_mouse(
            &mut c_owner
                .prompt_observer()
                .try_borrow_mut()
                .expect("unborrowed prompt"),
            (*m).x,
            ax,
            aw,
            Some(&mut redraw),
        );
    } else {
        result = prompt_key(&prompt, key, &mut redraw);
    }
    if redraw != 0 && c_owner.prompt_observer().is_alive() {
        c.as_ref()
            .expect("live client")
            .update_flags(CLIENT_REDRAWSTATUS as uint64_t, 0);
    }
    // Input callbacks can close or replace the prompt. Inspect the currently
    // installed prompt after dispatch, matching the original close decision.
    let current = c_owner.prompt_observer();
    let closed = current.is_alive()
        && prompt_closed(&current.try_borrow_mut().expect("unborrowed prompt")) != 0;
    if closed {
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
            let mut oo_owner = options_create(None);
            for name in [
                c"message-style",
                c"message-line",
                c"status-position",
                c"status",
            ] {
                let definition = (&options_table)
                    .iter()
                    .find(|entry| entry.name == Some(name))
                    .unwrap();
                options_default(&mut *oo_owner, definition);
            }
            let session_owner = session::with_options_for_test(oo_owner);
            let previous_sessions = std::mem::replace(
                &mut crate::src::session::sessions,
                crate::src::shared::session::sessions { storage: None },
            );
            crate::src::session::sessions_insert(
                &mut crate::src::session::sessions,
                session_owner.clone(),
            );
            let mut c = client::with_session_for_test(Some(&session_owner));
            c.borrow_terminal_mut().sx = 80;
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
                session_owner.with_options_mut(|options| {
                    options_set_string(options, c"message-style".as_ptr(), 0, |out| {
                        out.write_all(style.as_bytes())
                    });
                });
                assert_eq!(status_message_area(&c), expected, "{style}");
            }
            c.borrow_terminal_mut().sx = 0;
            assert_eq!(status_message_area(&c), (0, 0));
            c.borrow_status_mut().prompt_cx = 37;
            for (position, lines, message_line, height, flags, expected) in [
                (0, 3, 0, 24, 0, (0, 0)),
                (0, 3, 1, 24, 0, (0, 1)),
                (0, 3, 9, 24, 0, (0, 2)),
                (1, 3, 0, 24, 0, (21, 21)),
                (1, 3, 1, 24, 0, (21, 22)),
                (1, 3, 9, 24, 0, (21, 23)),
                (1, 3, 0, 1, 0, (-2, 0)),
                (1, 0, 0, 24, 0, (-1, 24)),
                (0, 3, 1, 24, CLIENT_STATUSOFF, (-1, 0)),
                (1, 3, 1, 24, CLIENT_CONTROL, (-1, 24)),
            ] {
                session_owner.with_options_mut(|options| {
                    options_set_number(options, c"status".as_ptr(), lines);
                    options_set_number(options, c"status-position".as_ptr(), position as i64);
                    options_set_number(options, c"message-line".as_ptr(), message_line);
                });
                crate::src::session::recalculate_size_state();
                c.borrow_terminal_mut().sy = height;
                c.update_flags(flags as u64, !(flags as u64));
                assert_eq!(status_at_line(&c), expected.0);
                assert_eq!(status_prompt_cursor(&c), (37, expected.1));
            }
            let saved = global_s_options;
            let mut global_options_owner = options_create(None);
            let definition = options_table
                .iter()
                .find(|entry| entry.name == Some(c"status"))
                .unwrap();
            options_default(&mut *global_options_owner, definition);
            global_s_options = &mut *global_options_owner;
            c = client::new();
            c.update_flags(0, u64::MAX);
            options_set_number(global_s_options, c"status".as_ptr(), 4);
            assert_eq!(status_line_size(&c), 4);
            global_s_options = saved;
            options_free(global_options_owner);
            // The session owns and releases its option table.
            crate::src::session::sessions_remove(
                &mut crate::src::session::sessions,
                &session_owner,
            );
            crate::src::session::sessions = previous_sessions;
        }
    }

    #[test]
    fn temporary_screen_is_shared_until_last_pop_and_base_survives() {
        unsafe {
            let previous = (global_options, global_s_options);
            let mut global_options_owner = options_create(None);
            global_options = &raw mut *global_options_owner;
            let mut global_s_options_owner = options_create(None);
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

            let c = client::new();
            c.borrow_terminal_mut().sx = 80;
            c.borrow_terminal_mut().sy = 24;
            status_init(&mut *c.borrow_status_mut(), 80);
            let base = (&c.borrow_status().screen as *const screen) as usize;
            let base_grid = c.borrow_status_mut().screen.grid() as *const grid as usize;
            assert!(c.borrow_status_mut().active.is_none());
            assert_eq!(
                c.borrow_status_mut().active_screen() as *mut screen as usize,
                base
            );
            let mut cell = grid_default_cell;
            cell.data.data[0] = b'B';
            grid_set_cell(c.borrow_status_mut().screen.grid_mut(), 0, 0, &cell);

            // A prompt and a message share one temporary screen.
            status_push_screen(&c);
            let temporary = c.borrow_status_mut().active_screen() as *mut screen as usize;
            let temporary_grid =
                c.borrow_status_mut().active_screen().grid() as *const grid as usize;
            assert_ne!(temporary, base);
            assert_ne!(temporary_grid, base_grid);
            cell.data.data[0] = b'T';
            grid_set_cell(
                c.borrow_status_mut().active_screen().grid_mut(),
                0,
                0,
                &cell,
            );
            status_push_screen(&c);
            assert_eq!(c.borrow_status_mut().screen_users, 2);
            assert_eq!(
                c.borrow_status_mut().active_screen() as *mut screen as usize,
                temporary
            );
            status_pop_screen(&c);
            assert_eq!(c.borrow_status_mut().screen_users, 1);
            assert_eq!(
                c.borrow_status_mut().active_screen().grid() as *const grid as usize,
                temporary_grid
            );
            grid_get_cell(
                c.borrow_status_mut().active_screen().grid(),
                0,
                0,
                &mut cell,
            );
            assert_eq!(cell.data.data[0], b'T');

            status_pop_screen(&c);
            assert_eq!(c.borrow_status_mut().screen_users, 0);
            assert!(c.borrow_status_mut().active.is_none());
            assert_eq!(
                c.borrow_status_mut().active_screen() as *mut screen as usize,
                base
            );
            assert_eq!(
                c.borrow_status_mut().screen.grid() as *const grid as usize,
                base_grid
            );
            grid_get_cell(c.borrow_status_mut().screen.grid(), 0, 0, &mut cell);
            assert_eq!(cell.data.data[0], b'B');
            status_free(&mut *c.borrow_status_mut());
            assert!(c.borrow_status_mut().screen.grid.is_none());

            // Teardown also releases an active temporary screen without a pop.
            status_init(&mut *c.borrow_status_mut(), 80);
            status_push_screen(&c);
            assert!(c.borrow_status_mut().active.is_some());
            status_free(&mut *c.borrow_status_mut());
            assert!(c.borrow_status_mut().active.is_none());
            assert!(c.borrow_status_mut().screen.grid.is_none());

            // A cleanup callback may clear the old prompt recursively and
            // install a replacement. The outer clear must leave its screen.
            let client = client::new();
            client.borrow_terminal_mut().sx = 80;
            client.borrow_terminal_mut().sy = 24;
            status_init(&mut *client.borrow_status_mut(), 80);
            status_push_screen(&client);
            let old = refbox::RefBox::new(prompt::default());
            let replacement = refbox::RefBox::new(prompt::default());
            let replacement_observer = replacement.downgrade();
            let next = replacement;
            let observer = std::rc::Rc::downgrade(&client);
            old.try_borrow_mut().unwrap().freecb = Some(Box::new(move || {
                let client = observer.upgrade().expect("live prompt client");
                status_prompt_clear(&client);
                status_push_screen(&client);
                client.install_prompt(next, 0);
            }));
            client.install_prompt(old, 0);
            status_prompt_clear(&client);
            assert_eq!(
                replacement_observer.as_ptr(),
                client.prompt_observer().as_ptr()
            );
            assert_eq!(client.borrow_status().screen_users, 1);
            assert!(client.borrow_status().active.is_some());
            assert_ne!(client.borrow_terminal().flags & TTY_FREEZE, 0);
            status_prompt_clear(&client);
            assert!(!client.prompt_observer().is_alive());
            assert_eq!(client.borrow_status().screen_users, 0);
            assert!(client.borrow_status().active.is_none());
            status_free(&mut *client.borrow_status_mut());

            options_free(global_options_owner);
            options_free(global_s_options_owner);
            (global_options, global_s_options) = previous;
        }
    }
}
