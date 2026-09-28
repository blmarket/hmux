use crate::src::tty_term::tty_term_owner_ptr;
use crate::src::options::options_owner_ptr;
use crate::src::ffi::libc::__useconds_t;
use crate::src::ffi::libc::{
    __errno_location, abs, fcntl, getpid, ioctl, isatty, memcpy, memset, open, strcmp, strerror,
    strlen, strncmp, tcflush, tcgetattr, tcsetattr, time, usleep, write,
};
use crate::src::ffi::resolv::__b64_ntop;
use crate::src::format::bytes::write_cstr;
use crate::src::format::bytes::{format_cstring, xformat};
use crate::src::format::{format_create, format_defaults, format_free};
use crate::src::grid::{grid_cells_equal, grid_default_cell};
use crate::src::hyperlinks::hyperlinks_get;
use crate::src::log::{fatal, fatalx, log_cstr, log_cstr_n, log_debug, log_get_level};
use crate::src::options::{options_get_number, options_get_string};
use crate::src::reactor::{
    evbuffer_add, evbuffer_drain, evbuffer_get_length, evbuffer_new, evbuffer_read,
    evbuffer_write, event_add, event_del, event_initialized, event_pending, event_set,
};
use crate::src::screen::screen_mode_display;
use crate::src::server::clients;
use crate::src::server_client::{
    server_client_ensure_ranges, server_client_lost, server_client_overlay_check,
    server_client_ranges_is_empty,
};
use crate::src::server_fn::server_redraw_client;
use crate::src::shared::abi::ssize_t;
use crate::src::shared::abi::*;
use crate::src::shared::client::client;
use crate::src::shared::mouse::mouse_event;
use crate::src::shared::client::{
    CLIENT_ALLREDRAWFLAGS, CLIENT_REDRAWSTATUS, CLIENT_REDRAWWINDOW, CLIENT_SUSPENDED,
    CLIENT_TERMINAL, CLIENT_UTF8,
};
use crate::src::shared::colour::*;
use crate::src::shared::colour::{
    COLOUR_FLAG_256, COLOUR_FLAG_RGB, COLOUR_FLAG_THEME, COLOUR_THEME_COUNT,
};
use crate::src::shared::command::cmdq_item;
use crate::src::shared::display::*;
use crate::src::shared::display::{visible_range, visible_ranges};
use crate::src::shared::errno::EAGAIN;
use crate::src::shared::event::{EV_PERSIST, EV_READ, EV_WRITE};
use crate::src::shared::format::format_tree;
use crate::src::shared::format::{FORMAT_NOJOBS, FORMAT_PANE};
use crate::src::shared::grid::*;
use crate::src::shared::limits::UINT_MAX;
use crate::src::shared::options::options;
use crate::src::shared::pane::window_pane;
use crate::src::shared::pane::PANE_STYLECHANGED;
use crate::src::shared::posix_io::{O_CREAT, O_TRUNC, O_WRONLY};
use crate::src::shared::posix_terminal::{winsize, ICRNL, ONLCR, OPOST, TCSANOW, VMIN, VTIME};
use crate::src::shared::screen::{
    screen, ALL_MODES, ALL_MOUSE_MODES, CURSOR_MODES, MODE_CURSOR, MODE_CURSOR_BLINKING,
    MODE_CURSOR_BLINKING_SET, MODE_CURSOR_VERY_VISIBLE, MODE_MOUSE_ALL, MODE_MOUSE_BUTTON,
    MODE_MOUSE_STANDARD,
};
use crate::src::shared::session::session;
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
use crate::src::shared::tty::*;
pub use crate::src::shared::tty::{tty, tty_ctx, tty_ctx_set_client_cb, tty_style_ctx, tty_term};
use crate::src::shared::tty::{
    TERM_256COLOURS, TERM_DECFRA, TERM_DECSLRM, TERM_NOAM, TERM_RGBCOLOURS, TERM_VT100LIKE,
    TTY_ALL_REQUEST_FLAGS, TTY_BLOCK, TTY_BLOCK_INTERVAL, TTY_CTX_CELL_INVALIDATE,
    TTY_CTX_INVISIBLE_PANES, TTY_CTX_OVERLAY_SYNC, TTY_CTX_PANE_OBSCURED, TTY_CTX_SYNC,
    TTY_CTX_WINDOW_BIGGER, TTY_CTX_WRAPPED, TTY_FREEZE, TTY_HAVEDA, TTY_HAVEDA2, TTY_HAVESYNC,
    TTY_HAVEXDA, TTY_NOBLOCK, TTY_NOCURSOR, TTY_OPENED, TTY_OSC52QUERY, TTY_QUERY_TIMEOUT,
    TTY_REQUEST_LIMIT, TTY_STARTED, TTY_SYNCING, TTY_TIMER, TTY_WAITBG, TTY_WAITFG,
    TTY_WINSIZEQUERY,
};
use crate::src::shared::utf8::UTF8_SIZE;
use crate::src::shared::window::{window, winlink};
use crate::src::status::status_line_size;
use crate::src::style::colour::{
    colour_256to16, colour_dim, colour_find_rgb, colour_force_rgb, colour_palette_get,
    colour_split_rgb,
};
use crate::src::style::style_add;
use crate::src::text::utf8::utf8_set;
use crate::src::tmux::{global_options, setblocking};
use crate::src::tty_acs::{tty_acs_get, tty_acs_needed, tty_acs_reverse_get};
use crate::src::tty_draw::tty_draw_line;
use crate::src::tty_features::tty_apply_features;
use crate::src::tty_keys::{tty_keys_build, tty_keys_free, tty_keys_next};
use crate::src::tty_term::{
    tty_term_apply_overrides, tty_term_create, tty_term_flag, tty_term_free, tty_term_has,
    tty_term_number, tty_term_string, tty_term_string_i, tty_term_string_ii, tty_term_string_iii,
    tty_term_string_s, tty_term_string_ss,
};

use std::ffi::CStr;

pub const TIOCGWINSZ: ::core::ffi::c_int = 0x5413 as ::core::ffi::c_int;

pub const F_SETFD: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const FD_CLOEXEC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;

pub const IGNBRK: ::core::ffi::c_int = 0o1 as ::core::ffi::c_int;
pub const ISTRIP: ::core::ffi::c_int = 0o40 as ::core::ffi::c_int;
pub const INLCR: ::core::ffi::c_int = 0o100 as ::core::ffi::c_int;
pub const IGNCR: ::core::ffi::c_int = 0o200 as ::core::ffi::c_int;

pub const IXON: ::core::ffi::c_int = 0o2000 as ::core::ffi::c_int;
pub const IXOFF: ::core::ffi::c_int = 0o10000 as ::core::ffi::c_int;
pub const IMAXBEL: ::core::ffi::c_int = 0o20000 as ::core::ffi::c_int;

pub const OCRNL: ::core::ffi::c_int = 0o10 as ::core::ffi::c_int;
pub const ONLRET: ::core::ffi::c_int = 0o40 as ::core::ffi::c_int;
pub const ISIG: ::core::ffi::c_int = 0o1 as ::core::ffi::c_int;
pub const ICANON: ::core::ffi::c_int = 0o2 as ::core::ffi::c_int;
pub const ECHO: ::core::ffi::c_int = 0o10 as ::core::ffi::c_int;
pub const ECHOE: ::core::ffi::c_int = 0o20 as ::core::ffi::c_int;
pub const ECHONL: ::core::ffi::c_int = 0o100 as ::core::ffi::c_int;
pub const ECHOCTL: ::core::ffi::c_int = 0o1000 as ::core::ffi::c_int;
pub const ECHOPRT: ::core::ffi::c_int = 0o2000 as ::core::ffi::c_int;
pub const ECHOKE: ::core::ffi::c_int = 0o4000 as ::core::ffi::c_int;
pub const IEXTEN: ::core::ffi::c_int = 0o100000 as ::core::ffi::c_int;

pub const TCOFLUSH: ::core::ffi::c_int = 1 as ::core::ffi::c_int;

static mut tty_log_fd: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
static mut tty_default_style_ctx: tty_style_ctx = {
    tty_style_ctx {
        defaults: grid_default_cell,
        palette: crate::src::shared::tty::PaletteSource::None,
        dim: 0 as u_int,
        hyperlinks: None,
    }
};
pub unsafe fn tty_create_log() {
    let mut name: [::core::ffi::c_char; 64] = [0; 64];
    xformat(
        &mut name,
        format_args!("tmux-out-{}.log", getpid() as ::core::ffi::c_long),
    );
    tty_log_fd = open(
        &raw mut name as *mut ::core::ffi::c_char,
        O_WRONLY | O_CREAT | O_TRUNC,
        0o644 as ::core::ffi::c_int,
    );
    if tty_log_fd != -(1 as ::core::ffi::c_int)
        && fcntl(tty_log_fd, F_SETFD, FD_CLOEXEC) == -(1 as ::core::ffi::c_int)
    {
        fatal(|out| out.write_all(b"fcntl failed"));
    }
}
pub unsafe fn tty_init(owner: &std::rc::Rc<std::cell::UnsafeCell<client>>) -> ::core::ffi::c_int {
    let c = owner.get();
    let tty = &raw mut (*c).tty;
    if isatty((*c).fd) == 0 {
        return -(1 as ::core::ffi::c_int);
    }
    tty_keys_free(tty);
    // Reset Rust-owned fields normally; byte-zeroing would invalidate Weak.
    *tty = crate::src::shared::tty::tty::empty();
    (*tty).client = std::rc::Rc::downgrade(owner);
    (*tty).cstyle = SCREEN_CURSOR_DEFAULT;
    (*tty).ccolour = -(1 as ::core::ffi::c_int);
    (*tty).bg = -(1 as ::core::ffi::c_int);
    (*tty).fg = (*tty).bg;
    (*tty).mouse_last_pane = -(1 as ::core::ffi::c_int);
    if tcgetattr((*c).fd, &raw mut (*tty).tio) != 0 as ::core::ffi::c_int {
        return -(1 as ::core::ffi::c_int);
    }
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn tty_resize(mut tty: *mut tty) {
    let terminal_client_owner = (*tty).client.upgrade().expect("terminal belongs to a live client");
    let terminal_client = terminal_client_owner.get();
    let mut c: *mut client = terminal_client;
    let mut ws: winsize = winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut xpixel: u_int = 0;
    let mut ypixel: u_int = 0;
    if ioctl((*c).fd, TIOCGWINSZ as ::core::ffi::c_ulong, &raw mut ws) != -(1 as ::core::ffi::c_int)
    {
        sx = ws.ws_col as u_int;
        if sx == 0 as u_int {
            sx = 80 as u_int;
            xpixel = 0 as u_int;
        } else {
            xpixel = (ws.ws_xpixel as u_int).wrapping_div(sx);
        }
        sy = ws.ws_row as u_int;
        if sy == 0 as u_int {
            sy = 24 as u_int;
            ypixel = 0 as u_int;
        } else {
            ypixel = (ws.ws_ypixel as u_int).wrapping_div(sy);
        }
        if (xpixel == 0 as u_int || ypixel == 0 as u_int)
            && (*tty).out.is_some()
            && (*tty).flags & TTY_WINSIZEQUERY == 0
            && (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags & TERM_VT100LIKE != 0
        {
            tty_puts(tty, c"\x1B[18t\x1B[14t");
            (*tty).flags |= TTY_WINSIZEQUERY;
        }
    } else {
        sx = 80 as u_int;
        sy = 24 as u_int;
        xpixel = 0 as u_int;
        ypixel = 0 as u_int;
    }
    log_debug(format_args!(
        "{}: {} now {}x{} ({}x{})",
        "tty_resize",
        log_cstr(
            (((*c).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                as *const _
        ),
        (sx) as u32,
        (sy) as u32,
        (xpixel) as u32,
        (ypixel) as u32
    ));
    tty_set_size(tty, sx, sy, xpixel, ypixel);
    tty_invalidate(tty);
}
pub unsafe fn tty_set_size(
    mut tty: *mut tty,
    mut sx: u_int,
    mut sy: u_int,
    mut xpixel: u_int,
    mut ypixel: u_int,
) {
    (*tty).sx = sx;
    (*tty).sy = sy;
    (*tty).xpixel = xpixel;
    (*tty).ypixel = ypixel;
}
unsafe fn tty_read_callback(owner: &std::rc::Rc<std::cell::UnsafeCell<client>>) {
    let c = owner.get();
    let tty = &raw mut (*c).tty;
    let mut name: *const ::core::ffi::c_char = ((*c).name)
        .as_ref()
        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut());
    let mut size: size_t = evbuffer_get_length((*tty).in_0.as_deref().expect("open TTY buffer"));
    let mut nread: ::core::ffi::c_int = 0;
    nread = evbuffer_read(
        (*tty).in_0.as_deref_mut().expect("open TTY buffer"),
        (*c).fd,
        -(1 as ::core::ffi::c_int),
    );
    if nread == 0 as ::core::ffi::c_int || nread == -(1 as ::core::ffi::c_int) {
        if nread == 0 as ::core::ffi::c_int {
            log_debug(format_args!(
                "{}: read closed",
                log_cstr((name) as *const _)
            ));
        } else {
            log_debug(format_args!(
                "{}: read error: {}",
                log_cstr((name) as *const _),
                log_cstr((strerror(*__errno_location())) as *const _)
            ));
        }
        event_del(&raw mut (*tty).event_in);
        server_client_lost(owner);
        return;
    }
    log_debug(format_args!(
        "{}: read {} bytes (already {})",
        log_cstr((name) as *const _),
        (nread) as i32,
        (size) as usize
    ));
    while tty_keys_next(tty) != 0 {}
}
unsafe fn tty_timer_callback(owner: &std::rc::Rc<std::cell::UnsafeCell<client>>) {
    let c = owner.get();
    let tty = &raw mut (*c).tty;
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: TTY_BLOCK_INTERVAL as __suseconds_t,
    };
    log_debug(format_args!(
        "{}: {} discarded",
        log_cstr(
            (((*c).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                as *const _
        ),
        ((*tty).discarded) as usize
    ));
    (*c).flags |= CLIENT_ALLREDRAWFLAGS as uint64_t;
    (*c).discarded = (*c).discarded.wrapping_add((*tty).discarded);
    if (*tty).discarded
        < (1 as u_int).wrapping_add((*tty).sx.wrapping_mul((*tty).sy).wrapping_div(8 as u_int))
            as size_t
    {
        (*tty).flags &= !TTY_BLOCK;
        tty_invalidate(tty);
        return;
    }
    (*tty).discarded = 0 as size_t;
    event_add(&raw mut (*tty).timer, &raw mut tv);
}
unsafe fn tty_block_maybe(mut tty: *mut tty) -> ::core::ffi::c_int {
    let terminal_client_owner = (*tty).client.upgrade().expect("terminal belongs to a live client");
    let terminal_client = terminal_client_owner.get();
    let mut c: *mut client = terminal_client;
    let mut size: size_t = evbuffer_get_length((*tty).out.as_deref().expect("open TTY buffer"));
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: TTY_BLOCK_INTERVAL as __suseconds_t,
    };
    if size == 0 as size_t {
        (*tty).flags &= !TTY_NOBLOCK;
    } else if (*tty).flags & TTY_NOBLOCK != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if size
        < (1 as u_int).wrapping_add((*tty).sx.wrapping_mul((*tty).sy).wrapping_mul(8 as u_int))
            as size_t
    {
        return 0 as ::core::ffi::c_int;
    }
    if (*tty).flags & TTY_BLOCK != 0 {
        return 1 as ::core::ffi::c_int;
    }
    (*tty).flags |= TTY_BLOCK;
    log_debug(format_args!(
        "{}: can't keep up, {} discarded",
        log_cstr(
            (((*c).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                as *const _
        ),
        (size) as usize
    ));
    evbuffer_drain((*tty).out.as_deref_mut().expect("open TTY buffer"), size);
    (*c).discarded = (*c).discarded.wrapping_add(size);
    (*tty).discarded = 0 as size_t;
    event_add(&raw mut (*tty).timer, &raw mut tv);
    return 1 as ::core::ffi::c_int;
}
unsafe fn tty_write_callback(owner: &std::rc::Rc<std::cell::UnsafeCell<client>>) {
    let c = owner.get();
    let tty = &raw mut (*c).tty;
    let mut size: size_t = evbuffer_get_length((*tty).out.as_deref().expect("open TTY buffer"));
    let mut nwrite: ::core::ffi::c_int = 0;
    nwrite = evbuffer_write((*tty).out.as_deref_mut().expect("open TTY buffer"), (*c).fd);
    if nwrite == -(1 as ::core::ffi::c_int) {
        return;
    }
    log_debug(format_args!(
        "{}: wrote {} bytes (of {})",
        log_cstr(
            (((*c).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                as *const _
        ),
        (nwrite) as i32,
        (size) as usize
    ));
    if (*c).redraw > 0 as size_t {
        if nwrite as size_t >= (*c).redraw {
            (*c).redraw = 0 as size_t;
        } else {
            (*c).redraw = (*c).redraw.wrapping_sub(nwrite as size_t);
        }
        log_debug(format_args!(
            "{}: waiting for redraw, {} bytes left",
            log_cstr(
                (((*c).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                    as *const _
            ),
            ((*c).redraw) as usize
        ));
    } else if tty_block_maybe(tty) != 0 {
        return;
    }
    if evbuffer_get_length((*tty).out.as_deref().expect("open TTY buffer")) != 0 as size_t {
        event_add(&raw mut (*tty).event_out, ::core::ptr::null::<timeval>());
    }
}
pub(crate) fn tty_client_callback(
    owner: &std::rc::Rc<std::cell::UnsafeCell<client>>,
    callback: unsafe fn(&std::rc::Rc<std::cell::UnsafeCell<client>>),
) -> impl FnMut(::core::ffi::c_int, ::core::ffi::c_short) {
    let observer = std::rc::Rc::downgrade(owner);
    move |_, _| {
        if let Some(owner) = observer.upgrade() {
            unsafe { callback(&owner) };
        }
    }
}

pub(crate) fn tty_mouse_client_callback(
    owner: &std::rc::Rc<std::cell::UnsafeCell<client>>,
    callback: unsafe fn(&std::rc::Rc<std::cell::UnsafeCell<client>>, *mut mouse_event),
) -> impl FnMut(&mut mouse_event) {
    let observer = std::rc::Rc::downgrade(owner);
    move |mouse| {
        if let Some(owner) = observer.upgrade() {
            unsafe { callback(&owner, mouse) };
        }
    }
}

pub unsafe fn tty_open(owner: &std::rc::Rc<std::cell::UnsafeCell<client>>) -> Result<(), std::ffi::CString> {
    let c = owner.get();
    let tty = &raw mut (*c).tty;
    // The synchronous terminfo constructor borrows these string pointers.
    let mut caps: Vec<_> = (*c)
        .term_caps
        .iter()
        .map(|cap| cap.as_ptr().cast_mut())
        .collect();
    (*tty).term = match tty_term_create(
        tty,
        ((*c).term_name)
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        caps.as_mut_ptr(),
        caps.len() as u_int,
    ) {
        Ok(term) => Some(term),
        Err(error) => {
            tty_close(tty);
            return Err(error);
        }
    };
    (*tty).flags |= TTY_OPENED;
    (*tty).flags &= !(TTY_NOCURSOR | TTY_FREEZE | TTY_BLOCK | TTY_TIMER);
    event_set(
        &raw mut (*tty).event_in,
        (*c).fd,
        (EV_PERSIST | EV_READ) as ::core::ffi::c_short,
        tty_client_callback(owner, tty_read_callback),
    );
    (*tty).in_0 = Some(evbuffer_new());
    event_set(
        &raw mut (*tty).event_out,
        (*c).fd,
        EV_WRITE as ::core::ffi::c_short,
        tty_client_callback(owner, tty_write_callback),
    );
    (*tty).out = Some(evbuffer_new());
    event_set(
        &raw mut (*tty).clipboard_timer,
        -(1 as ::core::ffi::c_int),
        0 as ::core::ffi::c_short,
        tty_client_callback(owner, tty_clipboard_query_callback),
    );
    event_set(
        &raw mut (*tty).start_timer,
        -(1 as ::core::ffi::c_int),
        0 as ::core::ffi::c_short,
        tty_client_callback(owner, tty_start_timer_callback),
    );
    event_set(
        &raw mut (*tty).timer,
        -(1 as ::core::ffi::c_int),
        0 as ::core::ffi::c_short,
        tty_client_callback(owner, tty_timer_callback),
    );
    tty_start_tty(tty);
    tty_keys_build(tty);
    Ok(())
}
unsafe fn tty_start_timer_callback(owner: &std::rc::Rc<std::cell::UnsafeCell<client>>) {
    let c = owner.get();
    let tty = &raw mut (*c).tty;
    log_debug(format_args!(
        "{}: start timer fired",
        log_cstr(
            (((*c).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                as *const _
        )
    ));
    if (*tty).flags & (TTY_HAVEDA | TTY_HAVEDA2 | TTY_HAVEXDA) == 0 as ::core::ffi::c_int {
        tty_update_features(tty);
    }
    (*tty).flags |= TTY_ALL_REQUEST_FLAGS;
    (*tty).flags &= !(TTY_WAITBG | TTY_WAITFG);
}
unsafe fn tty_start_start_timer(mut tty: *mut tty) {
    let terminal_client_owner = (*tty).client.upgrade().expect("terminal belongs to a live client");
    let terminal_client = terminal_client_owner.get();
    let mut c: *mut client = terminal_client;
    let mut tv: timeval = timeval {
        tv_sec: TTY_QUERY_TIMEOUT as __time_t,
        tv_usec: 0,
    };
    log_debug(format_args!(
        "{}: start timer started",
        log_cstr(
            (((*c).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                as *const _
        )
    ));
    event_del(&raw mut (*tty).start_timer);
    event_add(&raw mut (*tty).start_timer, &raw mut tv);
}
pub unsafe fn tty_start_tty(mut tty: *mut tty) {
    let terminal_client_owner = (*tty).client.upgrade().expect("terminal belongs to a live client");
    let terminal_client = terminal_client_owner.get();
    let mut c: *mut client = terminal_client;
    let mut tio: termios = termios {
        c_iflag: 0,
        c_oflag: 0,
        c_cflag: 0,
        c_lflag: 0,
        c_line: 0,
        c_cc: [0; 32],
        c2rust_unnamed: termios_input_speed { __ispeed: 0 },
        c2rust_unnamed_0: termios_output_speed { __ospeed: 0 },
    };
    let mut i: u_int = 0;
    setblocking((*c).fd, 0 as ::core::ffi::c_int);
    event_add(&raw mut (*tty).event_in, ::core::ptr::null::<timeval>());
    memcpy(
        &raw mut tio as *mut ::core::ffi::c_void,
        &raw mut (*tty).tio as *const ::core::ffi::c_void,
        ::core::mem::size_of::<termios>() as size_t,
    );
    tio.c_iflag &= !(IXON | IXOFF | ICRNL | INLCR | IGNCR | IMAXBEL | ISTRIP) as tcflag_t;
    tio.c_iflag |= IGNBRK as tcflag_t;
    tio.c_oflag &= !(OPOST | ONLCR | OCRNL | ONLRET) as tcflag_t;
    tio.c_lflag &=
        !(IEXTEN | ICANON | ECHO | ECHOE | ECHONL | ECHOCTL | ECHOPRT | ECHOKE | ISIG) as tcflag_t;
    tio.c_cc[VMIN as usize] = 1 as cc_t;
    tio.c_cc[VTIME as usize] = 0 as cc_t;
    if tcsetattr((*c).fd, TCSANOW, &raw mut tio) == 0 as ::core::ffi::c_int {
        tcflush((*c).fd, TCOFLUSH);
    }
    if options_get_number(
        global_options,
        b"clear-on-attach\0" as *const u8 as *const ::core::ffi::c_char,
    ) != 0
    {
        tty_putcode(tty, TTYC_SMCUP);
        tty_putcode(tty, TTYC_CLEAR);
    } else {
        tty_putcode_ii(
            tty,
            TTYC_CSR,
            0 as ::core::ffi::c_int,
            (*tty).sy.wrapping_sub(1 as u_int) as ::core::ffi::c_int,
        );
        tty_putcode_ii(
            tty,
            TTYC_CUP,
            0 as ::core::ffi::c_int,
            (*tty).sy.wrapping_sub(1 as u_int) as ::core::ffi::c_int,
        );
        if tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_INDN) != 0 {
            tty_putcode_i(
                tty,
                TTYC_INDN,
                (*tty).sy.wrapping_add(1 as u_int) as ::core::ffi::c_int,
            );
        } else if tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_IND) != 0 {
            i = 0 as u_int;
            while i < (*tty).sy.wrapping_add(1 as u_int) {
                tty_putcode(tty, TTYC_IND);
                i = i.wrapping_add(1);
            }
        } else {
            tty_putcode(tty, TTYC_CLEAR);
        }
    }
    tty_putcode(tty, TTYC_SMKX);
    if tty_acs_needed(tty.as_ref()) != 0 {
        log_debug(format_args!(
            "{}: using capabilities for ACS",
            log_cstr(
                (((*c).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                    as *const _
            )
        ));
        tty_putcode(tty, TTYC_ENACS);
    } else {
        log_debug(format_args!(
            "{}: using UTF-8 for ACS",
            log_cstr(
                (((*c).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                    as *const _
            )
        ));
    }
    tty_putcode(tty, TTYC_CNORM);
    if tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_KMOUS) != 0 {
        tty_puts(tty, c"\x1B[?1000l\x1B[?1002l\x1B[?1003l");
        tty_puts(tty, c"\x1B[?1006l\x1B[?1005l");
    }
    if tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_ENBP) != 0 {
        tty_putcode(tty, TTYC_ENBP);
    }
    if (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags & TERM_VT100LIKE != 0 {
        tty_puts(tty, c"\x1B[?2031h\x1B[?996n");
    }
    tty_start_start_timer(tty);
    (*tty).flags |= TTY_STARTED;
    tty_invalidate(tty);
    if (*tty).ccolour != -(1 as ::core::ffi::c_int) {
        tty_force_cursor_colour(tty, -(1 as ::core::ffi::c_int));
    }
    (*tty).mouse_drag_flag = 0 as ::core::ffi::c_int;
    (*tty).mouse_drag_update = None;
    (*tty).mouse_drag_release = None;
}
pub unsafe fn tty_send_requests(mut tty: *mut tty) {
    if !(*tty).flags & TTY_STARTED != 0 {
        return;
    }
    if (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags & TERM_VT100LIKE != 0 {
        if !(*tty).flags & TTY_HAVEDA != 0 {
            tty_puts(tty, c"\x1B[c");
        }
        if !(*tty).flags & TTY_HAVEDA2 != 0 {
            tty_puts(tty, c"\x1B[>c");
        }
        if !(*tty).flags & TTY_HAVEXDA != 0 {
            tty_puts(tty, c"\x1B[>q");
        }
        if !(*tty).flags & TTY_HAVESYNC != 0 {
            tty_puts(tty, c"\x1B[?2026$p");
        }
        tty_puts(tty, c"\x1B]10;?\x1B\\\x1B]11;?\x1B\\");
        (*tty).flags |= TTY_WAITBG | TTY_WAITFG;
    } else {
        (*tty).flags |= TTY_ALL_REQUEST_FLAGS;
    }
    (*tty).last_requests = time(::core::ptr::null_mut::<time_t>());
}
pub unsafe fn tty_repeat_requests(mut tty: *mut tty, mut force: ::core::ffi::c_int) {
    let terminal_client_owner = (*tty).client.upgrade().expect("terminal belongs to a live client");
    let terminal_client = terminal_client_owner.get();
    let mut c: *mut client = terminal_client;
    let mut t: time_t = time(::core::ptr::null_mut::<time_t>());
    let mut n: u_int = (t - (*tty).last_requests) as u_int;
    if !(*tty).flags & TTY_STARTED != 0 {
        return;
    }
    if force == 0 && n <= TTY_REQUEST_LIMIT as u_int {
        log_debug(format_args!(
            "{}: not repeating requests ({} seconds)",
            log_cstr(
                (((*c).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                    as *const _
            ),
            (n) as u32
        ));
        return;
    }
    log_debug(format_args!(
        "{}: {}repeating requests ({} seconds)",
        log_cstr(
            (((*c).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                as *const _
        ),
        log_cstr(
            (if force != 0 {
                b"(force) \0" as *const u8 as *const ::core::ffi::c_char
            } else {
                b"\0" as *const u8 as *const ::core::ffi::c_char
            }) as *const _
        ),
        (n) as u32
    ));
    (*tty).last_requests = t;
    if (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags & TERM_VT100LIKE != 0 {
        tty_puts(tty, c"\x1B]10;?\x1B\\\x1B]11;?\x1B\\");
        (*tty).flags |= TTY_WAITBG | TTY_WAITFG;
    }
    tty_start_start_timer(tty);
}
pub unsafe fn tty_stop_tty(mut tty: *mut tty) {
    if (*tty).flags & TTY_STARTED == 0 {
        return;
    }

    let terminal_client_owner = (*tty).client.upgrade().expect("terminal belongs to a live client");
    let terminal_client = terminal_client_owner.get();
    let mut c: *mut client = terminal_client;
    let mut ws: winsize = winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    (*tty).flags &= !TTY_STARTED;
    event_del(&raw mut (*tty).start_timer);
    event_del(&raw mut (*tty).clipboard_timer);
    event_del(&raw mut (*tty).timer);
    (*tty).flags &= !TTY_BLOCK;
    event_del(&raw mut (*tty).event_in);
    event_del(&raw mut (*tty).event_out);
    if ioctl((*c).fd, TIOCGWINSZ as ::core::ffi::c_ulong, &raw mut ws) == -(1 as ::core::ffi::c_int)
    {
        return;
    }
    if tcsetattr((*c).fd, TCSANOW, &raw mut (*tty).tio) == -(1 as ::core::ffi::c_int) {
        return;
    }
    tty_raw(
        tty,
        tty_term_string_ii(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_CSR, 0 as ::core::ffi::c_int, ws.ws_row as ::core::ffi::c_int - 1 as ::core::ffi::c_int).as_ptr(),
    );
    if tty_acs_needed(tty.as_ref()) != 0 {
        tty_raw(tty, tty_term_string(&*(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)), TTYC_RMACS).as_ptr());
    }
    tty_raw(tty, tty_term_string(&*(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)), TTYC_SGR0).as_ptr());
    tty_raw(tty, tty_term_string(&*(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)), TTYC_RMKX).as_ptr());
    if options_get_number(
        global_options,
        b"clear-on-attach\0" as *const u8 as *const ::core::ffi::c_char,
    ) != 0
    {
        tty_raw(tty, tty_term_string(&*(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)), TTYC_CLEAR).as_ptr());
    }
    if (*tty).cstyle as ::core::ffi::c_uint
        != SCREEN_CURSOR_DEFAULT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_SE) != 0 {
            tty_raw(tty, tty_term_string(&*(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)), TTYC_SE).as_ptr());
        } else if tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_SS) != 0 {
            tty_raw(
                tty,
                tty_term_string_i(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_SS, 0 as ::core::ffi::c_int).as_ptr(),
            );
        }
    }
    if (*tty).ccolour != -(1 as ::core::ffi::c_int) {
        tty_raw(tty, tty_term_string(&*(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)), TTYC_CR).as_ptr());
    }
    tty_raw(tty, tty_term_string(&*(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)), TTYC_CNORM).as_ptr());
    if tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_KMOUS) != 0 {
        tty_raw(
            tty,
            b"\x1B[?1000l\x1B[?1002l\x1B[?1003l\0" as *const u8 as *const ::core::ffi::c_char,
        );
        tty_raw(
            tty,
            b"\x1B[?1006l\x1B[?1005l\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_DSBP) != 0 {
        tty_raw(tty, tty_term_string(&*(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)), TTYC_DSBP).as_ptr());
    }
    if (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags & TERM_VT100LIKE != 0 {
        tty_raw(
            tty,
            b"\x1B[?7727l\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    tty_raw(tty, tty_term_string(&*(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)), TTYC_DSFCS).as_ptr());
    tty_raw(tty, tty_term_string(&*(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)), TTYC_DSEKS).as_ptr());
    if (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags & TERM_DECSLRM != 0 {
        tty_raw(tty, tty_term_string(&*(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)), TTYC_DSMG).as_ptr());
    }
    if options_get_number(
        global_options,
        b"clear-on-attach\0" as *const u8 as *const ::core::ffi::c_char,
    ) != 0
    {
        tty_raw(tty, tty_term_string(&*(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)), TTYC_RMCUP).as_ptr());
    } else {
        tty_raw(tty, tty_term_string(&*(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)), TTYC_CLEAR).as_ptr());
    }
    if (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags & TERM_VT100LIKE != 0 {
        tty_raw(
            tty,
            b"\x1B[?2031l\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    setblocking((*c).fd, 1 as ::core::ffi::c_int);
}
pub unsafe fn tty_close(mut tty: *mut tty) {
    if event_initialized(&(*tty).key_timer) != 0 {
        event_del(&raw mut (*tty).key_timer);
    }
    tty_stop_tty(tty);
    if (*tty).flags & TTY_OPENED != 0 {
        (*tty).in_0 = None;
        event_del(&raw mut (*tty).event_in);
        (*tty).out = None;
        event_del(&raw mut (*tty).event_out);
        if let Some(term) = (*tty).term.take() {
            tty_term_free(term);
        }
        tty_keys_free(tty);
        (*tty).flags &= !TTY_OPENED;
    }
}
pub unsafe fn tty_free(mut tty: *mut tty) {
    tty_close(tty);
    (*tty).r.clear();
}
pub unsafe fn tty_update_features(mut tty: *mut tty) {
    let terminal_client_owner = (*tty).client.upgrade().expect("terminal belongs to a live client");
    let terminal_client = terminal_client_owner.get();
    let mut c: *mut client = terminal_client;
    if tty_apply_features((*tty).term.as_deref_mut().expect("open terminal")) != 0 {
        tty_term_apply_overrides((*tty).term.as_deref_mut().expect("open terminal"));
    }
    if (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags & TERM_DECSLRM != 0 {
        tty_putcode(tty, TTYC_ENMG);
    }
    if options_get_number(
        global_options,
        b"extended-keys\0" as *const u8 as *const ::core::ffi::c_char,
    ) != 0
    {
        tty_puts(
            tty,
            std::ffi::CStr::from_ptr(tty_term_string(&*(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)), TTYC_ENEKS).as_ptr()),
        );
    }
    if options_get_number(
        global_options,
        b"focus-events\0" as *const u8 as *const ::core::ffi::c_char,
    ) != 0
    {
        tty_puts(
            tty,
            std::ffi::CStr::from_ptr(tty_term_string(&*(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)), TTYC_ENFCS).as_ptr()),
        );
    }
    if (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags & TERM_VT100LIKE != 0 {
        tty_puts(tty, c"\x1B[?7727h");
    }
    server_redraw_client(&mut *(c));
    tty_invalidate(tty);
}
pub unsafe fn tty_raw(mut tty: *mut tty, mut s: *const ::core::ffi::c_char) {
    let terminal_client_owner = (*tty).client.upgrade().expect("terminal belongs to a live client");
    let terminal_client = terminal_client_owner.get();
    let mut c: *mut client = terminal_client;
    let mut n: ssize_t = 0;
    let mut slen: ssize_t = 0;
    let mut i: u_int = 0;
    slen = strlen(s) as ssize_t;
    i = 0 as u_int;
    while i < 5 as u_int {
        n = write((*c).fd, s as *const ::core::ffi::c_void, slen as size_t);
        if n >= 0 as ssize_t {
            s = s.offset(n as isize);
            slen -= n;
            if slen == 0 as ssize_t {
                break;
            }
        } else if n == -(1 as ::core::ffi::c_int) as ssize_t && *__errno_location() != EAGAIN {
            break;
        }
        usleep(100 as __useconds_t);
        i = i.wrapping_add(1);
    }
}
pub unsafe fn tty_putcode(mut tty: *mut tty, mut code: tty_code_code) {
    tty_puts(
        tty,
        std::ffi::CStr::from_ptr(tty_term_string(&*(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)), code).as_ptr()),
    );
}
pub unsafe fn tty_putcode_i(mut tty: *mut tty, mut code: tty_code_code, mut a: ::core::ffi::c_int) {
    if a < 0 as ::core::ffi::c_int {
        return;
    }
    tty_puts(
        tty,
        std::ffi::CStr::from_ptr(tty_term_string_i(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), code, a).as_ptr()),
    );
}
pub unsafe fn tty_putcode_ii(
    mut tty: *mut tty,
    mut code: tty_code_code,
    mut a: ::core::ffi::c_int,
    mut b: ::core::ffi::c_int,
) {
    if a < 0 as ::core::ffi::c_int || b < 0 as ::core::ffi::c_int {
        return;
    }
    tty_puts(
        tty,
        std::ffi::CStr::from_ptr(tty_term_string_ii(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), code, a, b).as_ptr()),
    );
}
pub unsafe fn tty_putcode_iii(
    mut tty: *mut tty,
    mut code: tty_code_code,
    mut a: ::core::ffi::c_int,
    mut b: ::core::ffi::c_int,
    mut c: ::core::ffi::c_int,
) {
    if a < 0 as ::core::ffi::c_int || b < 0 as ::core::ffi::c_int || c < 0 as ::core::ffi::c_int {
        return;
    }
    tty_puts(
        tty,
        std::ffi::CStr::from_ptr(tty_term_string_iii(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), code, a, b, c).as_ptr()),
    );
}
pub unsafe fn tty_putcode_s(mut tty: *mut tty, mut a: *const ::core::ffi::c_char) {
    let mut code: tty_code_code = TTYC_CS;
    if !a.is_null() {
        tty_puts(
            tty,
            std::ffi::CStr::from_ptr(tty_term_string_s(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), code, a).as_ptr()),
        );
    }
}
pub unsafe fn tty_putcode_ss(
    mut tty: *mut tty,
    mut code: tty_code_code,
    mut a: *const ::core::ffi::c_char,
    mut b: *const ::core::ffi::c_char,
) {
    if !a.is_null() && !b.is_null() {
        tty_puts(
            tty,
            std::ffi::CStr::from_ptr(tty_term_string_ss(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), code, a, b).as_ptr()),
        );
    }
}
unsafe fn tty_add(mut tty: *mut tty, buf: &[u8]) {
    let terminal_client_owner = (*tty).client.upgrade().expect("terminal belongs to a live client");
    let terminal_client = terminal_client_owner.get();
    let len = buf.len();
    let mut c: *mut client = terminal_client;
    if (*tty).flags & TTY_BLOCK != 0 {
        (*tty).discarded = (*tty).discarded.wrapping_add(len);
        return;
    }
    evbuffer_add(
        (*tty).out.as_deref_mut().expect("open TTY buffer"),
        buf.as_ptr().cast(),
        len,
    );
    log_debug(format_args!(
        "{}: {}",
        log_cstr(
            (((*c).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                as *const _
        ),
        log_cstr_n(buf.as_ptr().cast(), len as ::core::ffi::c_int)
    ));
    (*c).written = (*c).written.wrapping_add(len);
    if tty_log_fd != -(1 as ::core::ffi::c_int) {
        write(tty_log_fd, buf.as_ptr().cast(), len);
    }
    if (*tty).flags & TTY_STARTED != 0
        && event_pending(
            &raw mut (*tty).event_out,
            EV_WRITE as ::core::ffi::c_short,
            ::core::ptr::null_mut::<timeval>(),
        ) == 0
    {
        event_add(&raw mut (*tty).event_out, ::core::ptr::null::<timeval>());
    }
}
pub unsafe fn tty_puts(tty: *mut tty, text: &CStr) {
    if !text.is_empty() {
        tty_add(tty, text.to_bytes());
    }
}
pub unsafe fn tty_putc(mut tty: *mut tty, mut ch: u_char) {
    if (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags & TERM_NOAM != 0
        && ch as ::core::ffi::c_int >= 0x20 as ::core::ffi::c_int
        && ch as ::core::ffi::c_int != 0x7f as ::core::ffi::c_int
        && (*tty).cy == (*tty).sy.wrapping_sub(1 as u_int)
        && (*tty).cx.wrapping_add(1 as u_int) >= (*tty).sx
    {
        return;
    }
    if (*tty).cell.attr as ::core::ffi::c_int & GRID_ATTR_CHARSET != 0 {
        if let Some(acs) = tty_acs_get(Some(&*tty), ch) {
            tty_add(tty, acs.to_bytes());
        } else {
            tty_add(tty, &[ch]);
        }
    } else {
        tty_add(tty, &[ch]);
    }
    if ch as ::core::ffi::c_int >= 0x20 as ::core::ffi::c_int
        && ch as ::core::ffi::c_int != 0x7f as ::core::ffi::c_int
    {
        if (*tty).cx >= (*tty).sx {
            (*tty).cx = 1 as u_int;
            if (*tty).cy != (*tty).rlower {
                (*tty).cy = (*tty).cy.wrapping_add(1);
            }
            if (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags & TERM_NOAM != 0 {
                tty_putcode_ii(
                    tty,
                    TTYC_CUP,
                    (*tty).cy as ::core::ffi::c_int,
                    (*tty).cx as ::core::ffi::c_int,
                );
            }
        } else {
            (*tty).cx = (*tty).cx.wrapping_add(1);
        }
    }
}
pub unsafe fn tty_putn(mut tty: *mut tty, buf: &[u8], mut width: u_int) {
    let mut len = buf.len();
    if (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags & TERM_NOAM != 0
        && (*tty).cy == (*tty).sy.wrapping_sub(1 as u_int)
        && ((*tty).cx as size_t).wrapping_add(len) >= (*tty).sx as size_t
    {
        len = (*tty).sx.saturating_sub((*tty).cx).saturating_sub(1) as usize;
    }
    tty_add(tty, &buf[..len]);
    if (*tty).cx.wrapping_add(width) > (*tty).sx {
        (*tty).cx = (*tty).cx.wrapping_add(width).wrapping_sub((*tty).sx);
        if (*tty).cx <= (*tty).sx {
            (*tty).cy = (*tty).cy.wrapping_add(1);
        } else {
            (*tty).cy = UINT_MAX as u_int;
            (*tty).cx = (*tty).cy;
        }
    } else {
        (*tty).cx = (*tty).cx.wrapping_add(width);
    };
}
unsafe fn tty_set_italics(mut tty: *mut tty) {
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_SITM) != 0 {
        s = options_get_string(
            global_options,
            b"default-terminal\0" as *const u8 as *const ::core::ffi::c_char,
        );
        if strcmp(s, b"screen\0" as *const u8 as *const ::core::ffi::c_char)
            != 0 as ::core::ffi::c_int
            && strncmp(
                s,
                b"screen-\0" as *const u8 as *const ::core::ffi::c_char,
                7 as size_t,
            ) != 0 as ::core::ffi::c_int
        {
            tty_putcode(tty, TTYC_SITM);
            return;
        }
    }
    tty_putcode(tty, TTYC_SMSO);
}
pub unsafe fn tty_set_title(mut tty: *mut tty, title: &CStr) {
    if tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_TSL) == 0 || tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_FSL) == 0 {
        return;
    }
    tty_putcode(tty, TTYC_TSL);
    tty_puts(tty, title);
    tty_putcode(tty, TTYC_FSL);
}
pub unsafe fn tty_set_path(mut tty: *mut tty, title: &CStr) {
    if tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_SWD) == 0 || tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_FSL) == 0 {
        return;
    }
    tty_putcode(tty, TTYC_SWD);
    tty_puts(tty, title);
    tty_putcode(tty, TTYC_FSL);
}
unsafe fn tty_force_cursor_colour(mut tty: *mut tty, mut c: ::core::ffi::c_int) {
    let mut r: u_char = 0;
    let mut g: u_char = 0;
    let mut b: u_char = 0;
    if c != -(1 as ::core::ffi::c_int) {
        c = tty_map_theme_colour(tty, c);
        c = colour_force_rgb(c);
    }
    if c == (*tty).ccolour {
        return;
    }
    if c == -(1 as ::core::ffi::c_int) {
        tty_putcode(tty, TTYC_CR);
    } else {
        (r, g, b) = colour_split_rgb(c);
        let colour = format_cstring(format_args!("rgb:{r:02x}/{g:02x}/{b:02x}"))
            .expect("RGB colour contains no NUL");
        tty_putcode_s(tty, colour.as_ptr());
    }
    (*tty).ccolour = c;
}
unsafe fn tty_update_cursor(
    mut tty: *mut tty,
    mut mode: ::core::ffi::c_int,
    s: Option<&screen>,
) -> ::core::ffi::c_int {
    let mut cstyle: screen_cursor_style = SCREEN_CURSOR_DEFAULT;
    let mut ccolour: ::core::ffi::c_int = 0;
    let mut changed: ::core::ffi::c_int = 0;
    let mut cmode: ::core::ffi::c_int = mode;
    if let Some(s) = s {
        ccolour = s.ccolour;
        if s.ccolour == -(1 as ::core::ffi::c_int) {
            ccolour = s.default_ccolour;
        }
        tty_force_cursor_colour(tty, ccolour);
    }
    if !cmode & MODE_CURSOR != 0 {
        if (*tty).mode & MODE_CURSOR != 0 {
            tty_putcode(tty, TTYC_CIVIS);
        }
        return cmode;
    }
    if let Some(s) = s {
        cstyle = s.cstyle;
        if cstyle as ::core::ffi::c_uint
            == SCREEN_CURSOR_DEFAULT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if !cmode & MODE_CURSOR_BLINKING_SET != 0 {
                if s.default_mode & MODE_CURSOR_BLINKING != 0 {
                    cmode |= MODE_CURSOR_BLINKING;
                } else {
                    cmode &= !MODE_CURSOR_BLINKING;
                }
            }
            cstyle = s.default_cstyle;
        }
    } else {
        cstyle = (*tty).cstyle;
    }
    changed = cmode ^ (*tty).mode;
    if changed & CURSOR_MODES == 0 as ::core::ffi::c_int
        && cstyle as ::core::ffi::c_uint == (*tty).cstyle as ::core::ffi::c_uint
    {
        return cmode;
    }
    tty_putcode(tty, TTYC_CNORM);
    match cstyle as ::core::ffi::c_uint {
        0 => {
            if (*tty).cstyle as ::core::ffi::c_uint
                != SCREEN_CURSOR_DEFAULT as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_SE) != 0 {
                    tty_putcode(tty, TTYC_SE);
                } else {
                    tty_putcode_i(tty, TTYC_SS, 0 as ::core::ffi::c_int);
                }
            }
            if cmode & (MODE_CURSOR_BLINKING | MODE_CURSOR_VERY_VISIBLE) != 0 {
                tty_putcode(tty, TTYC_CVVIS);
            }
        }
        1 => {
            if tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_SS) != 0 {
                if cmode & MODE_CURSOR_BLINKING != 0 {
                    tty_putcode_i(tty, TTYC_SS, 1 as ::core::ffi::c_int);
                } else {
                    tty_putcode_i(tty, TTYC_SS, 2 as ::core::ffi::c_int);
                }
            } else if cmode & MODE_CURSOR_BLINKING != 0 {
                tty_putcode(tty, TTYC_CVVIS);
            }
        }
        2 => {
            if tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_SS) != 0 {
                if cmode & MODE_CURSOR_BLINKING != 0 {
                    tty_putcode_i(tty, TTYC_SS, 3 as ::core::ffi::c_int);
                } else {
                    tty_putcode_i(tty, TTYC_SS, 4 as ::core::ffi::c_int);
                }
            } else if cmode & MODE_CURSOR_BLINKING != 0 {
                tty_putcode(tty, TTYC_CVVIS);
            }
        }
        3 => {
            if tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_SS) != 0 {
                if cmode & MODE_CURSOR_BLINKING != 0 {
                    tty_putcode_i(tty, TTYC_SS, 5 as ::core::ffi::c_int);
                } else {
                    tty_putcode_i(tty, TTYC_SS, 6 as ::core::ffi::c_int);
                }
            } else if cmode & MODE_CURSOR_BLINKING != 0 {
                tty_putcode(tty, TTYC_CVVIS);
            }
        }
        _ => {}
    }
    (*tty).cstyle = cstyle;
    return cmode;
}
pub unsafe fn tty_update_mode(mut tty: *mut tty, mut mode: ::core::ffi::c_int, s: Option<&screen>) {
    let terminal_client_owner = (*tty).client.upgrade().expect("terminal belongs to a live client");
    let terminal_client = terminal_client_owner.get();
    let mut term: *const tty_term = tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term);
    let mut c: *mut client = terminal_client;
    let mut changed: ::core::ffi::c_int = 0;
    if (*tty).flags & TTY_NOCURSOR != 0 {
        mode &= !MODE_CURSOR;
    }
    if tty_update_cursor(tty, mode, s) & MODE_CURSOR_BLINKING != 0 {
        mode |= MODE_CURSOR_BLINKING;
    } else {
        mode &= !MODE_CURSOR_BLINKING;
    }
    changed = mode ^ (*tty).mode;
    if log_get_level() != 0 as ::core::ffi::c_int && changed != 0 as ::core::ffi::c_int {
        log_debug(format_args!(
            "{}: current mode {}",
            log_cstr(
                (((*c).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                    as *const _
            ),
            screen_mode_display((*tty).mode)
        ));
        log_debug(format_args!(
            "{}: setting mode {}",
            log_cstr(
                (((*c).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                    as *const _
            ),
            screen_mode_display(mode)
        ));
    }
    if changed & ALL_MOUSE_MODES != 0 && tty_term_has(term, TTYC_KMOUS) != 0 {
        tty_puts(tty, c"\x1B[?1006l\x1B[?1000l\x1B[?1002l\x1B[?1003l");
        if mode & ALL_MOUSE_MODES != 0 {
            tty_puts(tty, c"\x1B[?1006h");
        }
        if mode & MODE_MOUSE_ALL != 0 {
            tty_puts(tty, c"\x1B[?1000h\x1B[?1002h\x1B[?1003h");
        } else if mode & MODE_MOUSE_BUTTON != 0 {
            tty_puts(tty, c"\x1B[?1000h\x1B[?1002h");
        } else if mode & MODE_MOUSE_STANDARD != 0 {
            tty_puts(tty, c"\x1B[?1000h");
        }
    }
    (*tty).mode = mode;
}
unsafe fn tty_emulate_repeat(
    mut tty: *mut tty,
    mut code: tty_code_code,
    mut code1: tty_code_code,
    mut n: u_int,
) {
    if tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), code) != 0 {
        tty_putcode_i(tty, code, n as ::core::ffi::c_int);
    } else {
        loop {
            let fresh0 = n;
            n = n.wrapping_sub(1);
            if !(fresh0 > 0 as u_int) {
                break;
            }
            tty_putcode(tty, code1);
        }
    };
}
pub unsafe fn tty_repeat_space(tty: *mut tty, mut n: u_int) {
    const SPACES: [u8; 500] = [b' '; 500];
    while n as usize > SPACES.len() {
        tty_putn(tty, &SPACES, SPACES.len() as u_int);
        n -= SPACES.len() as u_int;
    }
    if n != 0 {
        tty_putn(tty, &SPACES[..n as usize], n);
    }
}
pub unsafe fn tty_window_bigger(mut tty: *mut tty) -> ::core::ffi::c_int {
    let terminal_client_owner = (*tty).client.upgrade().expect("terminal belongs to a live client");
    let terminal_client = terminal_client_owner.get();
    let mut c: *mut client = terminal_client;
    let mut w: *mut window = (*(*(*c).session_ptr()).curw_ptr()).window_ptr();
    return ((*tty).sx < (*w).sx || (*tty).sy.wrapping_sub(status_line_size(&*c)) < (*w).sy)
        as ::core::ffi::c_int;
}
pub fn tty_window_offset(tty: &tty) -> tty_window_view {
    tty_window_view {
        bigger: tty.oflag != 0,
        ox: tty.oox,
        oy: tty.ooy,
        sx: tty.osx,
        sy: tty.osy,
    }
}
unsafe fn tty_window_offset1(c: &mut client) -> tty_window_view {
    let w = (*(*c.session_ptr()).curw_ptr()).window_ptr();
    let wp = (*w).active_ptr();
    let lines = status_line_size(c);
    if c.tty.sx >= (*w).sx && c.tty.sy.wrapping_sub(lines) >= (*w).sy {
        c.pan_window = std::rc::Weak::new();
        return tty_window_view {
            bigger: false,
            ox: 0,
            oy: 0,
            sx: (*w).sx,
            sy: (*w).sy,
        };
    }
    let mut view = tty_window_view {
        bigger: true,
        ox: 0,
        oy: 0,
        sx: c.tty.sx,
        sy: c.tty.sy.wrapping_sub(lines),
    };
    if c.pan_window_is(&*w) {
        if view.sx >= (*w).sx {
            c.pan_ox = 0;
        } else if c.pan_ox.wrapping_add(view.sx) > (*w).sx {
            c.pan_ox = (*w).sx.wrapping_sub(view.sx);
        }
        view.ox = c.pan_ox;
        if view.sy >= (*w).sy {
            c.pan_oy = 0;
        } else if c.pan_oy.wrapping_add(view.sy) > (*w).sy {
            c.pan_oy = (*w).sy.wrapping_sub(view.sy);
        }
        view.oy = c.pan_oy;
        return view;
    }
    if (*(*wp).screen_ptr()).mode & MODE_CURSOR != 0 {
        let cx = ((*wp).xoff as u_int).wrapping_add((*(*wp).screen_ptr()).cx);
        let cy = ((*wp).yoff as u_int).wrapping_add((*(*wp).screen_ptr()).cy);
        view.ox = if cx < view.sx {
            0
        } else if cx > (*w).sx.wrapping_sub(view.sx) {
            (*w).sx.wrapping_sub(view.sx)
        } else {
            cx.wrapping_sub(view.sx / 2)
        };
        view.oy = if cy < view.sy {
            0
        } else if cy > (*w).sy.wrapping_sub(view.sy) {
            (*w).sy.wrapping_sub(view.sy)
        } else {
            cy.wrapping_sub(view.sy).wrapping_add(1)
        };
    }
    c.pan_window = std::rc::Weak::new();
    view
}
pub unsafe fn tty_update_window_offset(mut w: *mut window) {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut registry_c_owner = clients.first();
    c = registry_c_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    while !c.is_null() {
        if !(*c).session_ptr().is_null()
            && !(*(*c).session_ptr()).curw_ptr().is_null()
            && (*(*(*c).session_ptr()).curw_ptr()).window_ptr() == w
        {
            tty_update_client_offset(c);
        }
        registry_c_owner = clients.next(registry_c_owner.as_ref().expect("current registry client"));
        c = registry_c_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    }
}
pub unsafe fn tty_update_client_offset(mut c: *mut client) {
    if !(*c).flags & CLIENT_TERMINAL as uint64_t != 0 {
        return;
    }
    let tty_window_view {
        bigger,
        ox,
        oy,
        sx,
        sy,
    } = tty_window_offset1(&mut *c);
    (*c).tty.oflag = bigger as ::core::ffi::c_int;
    if ox == (*c).tty.oox && oy == (*c).tty.ooy && sx == (*c).tty.osx && sy == (*c).tty.osy {
        return;
    }
    log_debug(format_args!(
        "{}: {} offset has changed ({},{} {}x{} -> {},{} {}x{})",
        "tty_update_client_offset",
        log_cstr(
            (((*c).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                as *const _
        ),
        ((*c).tty.oox) as u32,
        ((*c).tty.ooy) as u32,
        ((*c).tty.osx) as u32,
        ((*c).tty.osy) as u32,
        (ox) as u32,
        (oy) as u32,
        (sx) as u32,
        (sy) as u32
    ));
    (*c).tty.oox = ox;
    (*c).tty.ooy = oy;
    (*c).tty.osx = sx;
    (*c).tty.osy = sy;
    (*c).flags |= (CLIENT_REDRAWWINDOW | CLIENT_REDRAWSTATUS) as uint64_t;
}
fn tty_large_region(ctx: &tty_ctx) -> ::core::ffi::c_int {
    return (ctx.orlower.wrapping_sub(ctx.orupper) >= ctx.sy.wrapping_div(2 as u_int))
        as ::core::ffi::c_int;
}
pub unsafe fn tty_fake_bce(tty: &tty, gc: &grid_cell, mut bg: u_int) -> ::core::ffi::c_int {
    if tty_term_flag(tty_term_owner_ptr(&tty.term).map_or(std::ptr::null(), |term| term), TTYC_BCE) != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if !(bg == 8 as u_int || bg == 9 as u_int)
        || !(gc.bg == 8 as ::core::ffi::c_int || gc.bg == 9 as ::core::ffi::c_int)
    {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe fn tty_redraw_region(mut tty: *mut tty, ctx: &tty_ctx, s: &screen) {
    let terminal_client_owner = (*tty).client.upgrade().expect("terminal belongs to a live client");
    let terminal_client = terminal_client_owner.get();
    let mut c: *mut client = terminal_client;
    let mut i: u_int = 0;
    if tty_large_region(ctx) != 0 || ctx.flags & TTY_CTX_PANE_OBSCURED != 0 {
        log_debug(format_args!(
            "{}: {} large region redraw",
            "tty_redraw_region",
            log_cstr(
                (((*c).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                    as *const _
            )
        ));
        ctx.redraw_cb.as_ref().expect("non-null redraw callback")(ctx);
        return;
    }
    log_debug(format_args!(
        "{}: {} small region redraw ({}-{})",
        "tty_redraw_region",
        log_cstr(
            (((*c).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                as *const _
        ),
        (ctx.orupper) as u32,
        (ctx.orlower) as u32
    ));
    i = ctx.orupper;
    while i <= ctx.orlower {
        tty_draw_pane(tty, ctx, s, i);
        i = i.wrapping_add(1);
    }
}
fn tty_is_visible(
    ctx: &tty_ctx,
    mut px: u_int,
    mut py: u_int,
    mut nx: u_int,
    mut ny: u_int,
) -> ::core::ffi::c_int {
    let mut xoff: u_int = (ctx.rxoff as u_int).wrapping_add(px);
    let mut yoff: u_int = (ctx.ryoff as u_int).wrapping_add(py);
    if !ctx.flags & TTY_CTX_WINDOW_BIGGER != 0 {
        return 1 as ::core::ffi::c_int;
    }
    if xoff.wrapping_add(nx) <= ctx.wox
        || xoff >= ctx.wox.wrapping_add(ctx.wsx)
        || yoff.wrapping_add(ny) <= ctx.woy
        || yoff >= ctx.woy.wrapping_add(ctx.wsy)
    {
        return 0 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
#[derive(Debug, PartialEq, Eq)]
struct tty_clamped_line {
    skip: u_int,
    x: u_int,
    y: u_int,
    width: u_int,
}

fn tty_clamp_line(ctx: &tty_ctx, px: u_int, py: u_int, nx: u_int) -> Option<tty_clamped_line> {
    let xoff = (ctx.rxoff as u_int).wrapping_add(px) as i32;
    if tty_is_visible(ctx, px, py, nx, 1) == 0 {
        return None;
    }
    let y = (ctx.yoff as u_int).wrapping_add(py).wrapping_sub(ctx.woy);
    let (skip, x, width) = if xoff >= ctx.wox as i32
        && (xoff as u_int).wrapping_add(nx) <= ctx.wox.wrapping_add(ctx.wsx)
    {
        (
            0,
            (ctx.xoff as u_int).wrapping_add(px).wrapping_sub(ctx.wox),
            nx,
        )
    } else if xoff < ctx.wox as i32
        && (xoff as u_int).wrapping_add(nx) > ctx.wox.wrapping_add(ctx.wsx)
    {
        (ctx.wox, 0, ctx.wsx)
    } else if xoff < ctx.wox as i32 {
        let skip = ctx.wox.wrapping_sub((ctx.xoff as u_int).wrapping_add(px));
        (skip, 0, nx.wrapping_sub(skip))
    } else {
        let x = (ctx.xoff as u_int).wrapping_add(px).wrapping_sub(ctx.wox);
        (0, x, ctx.wsx.wrapping_sub(x))
    };
    if width > nx {
        unsafe {
            fatalx(|out| write!(out, "tty_clamp_line: x too big, {} > {}", width, nx));
        }
    }
    Some(tty_clamped_line { skip, x, y, width })
}
unsafe fn tty_clear_line(
    mut tty: *mut tty,
    defaults: &grid_cell,
    mut py: u_int,
    mut px: u_int,
    mut nx: u_int,
    mut bg: u_int,
) {
    let terminal_client_owner = (*tty).client.upgrade().expect("terminal belongs to a live client");
    let terminal_client = terminal_client_owner.get();
    let mut c: *mut client = terminal_client;
    let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
    let mut rr: *mut visible_range = ::core::ptr::null_mut::<visible_range>();
    let mut i: u_int = 0;
    log_debug(format_args!(
        "{}: {}, {} at {},{}",
        "tty_clear_line",
        log_cstr(
            (((*c).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                as *const _
        ),
        (nx) as u32,
        (px) as u32,
        (py) as u32
    ));
    if nx == 0 as u_int {
        return;
    }
    if (*c).overlay_check.is_none() && tty_fake_bce(&*tty, defaults, bg) == 0 {
        if px.wrapping_add(nx) >= (*tty).sx && tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_EL) != 0 {
            tty_cursor(tty, px, py);
            tty_putcode(tty, TTYC_EL);
            return;
        }
        if px == 0 as u_int && tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_EL1) != 0 {
            tty_cursor(tty, px.wrapping_add(nx).wrapping_sub(1 as u_int), py);
            tty_putcode(tty, TTYC_EL1);
            return;
        }
        if tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_ECH) != 0 {
            tty_cursor(tty, px, py);
            tty_putcode_i(tty, TTYC_ECH, nx as ::core::ffi::c_int);
            return;
        }
    }
    r = tty_check_overlay_range(tty, px, py, nx);
    i = 0 as u_int;
    while i < (*r).used {
        rr = &raw mut (&mut (*r).storage)[i as usize];
        if (*rr).nx != 0 as u_int {
            tty_cursor(tty, (*rr).px, py);
            tty_repeat_space(tty, (*rr).nx);
        }
        i = i.wrapping_add(1);
    }
}
unsafe fn tty_clear_pane_line(
    mut tty: *mut tty,
    ctx: &tty_ctx,
    mut py: u_int,
    mut px: u_int,
    mut nx: u_int,
    mut bg: u_int,
) {
    let terminal_client_owner = (*tty).client.upgrade().expect("terminal belongs to a live client");
    let terminal_client = terminal_client_owner.get();
    let mut c: *mut client = terminal_client;
    let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
    let mut ri: *mut visible_range = ::core::ptr::null_mut::<visible_range>();
    let mut i: u_int = 0;
    log_debug(format_args!(
        "{}: {}, {} at {},{}",
        "tty_clear_pane_line",
        log_cstr(
            (((*c).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                as *const _
        ),
        (nx) as u32,
        (px) as u32,
        (py) as u32
    ));
    if let Some(tty_clamped_line {
        skip: _,
        x,
        y: ry,
        width: rx,
    }) = tty_clamp_line(ctx, px, py, nx)
    {
        r = tty_check_overlay_range(tty, x, ry, rx);
        i = 0 as u_int;
        while i < (*r).used {
            ri = &raw mut (&mut (*r).storage)[i as usize];
            if !((*ri).nx == 0 as u_int) {
                tty_clear_line(tty, &ctx.style_ctx.defaults, ry, (*ri).px, (*ri).nx, bg);
            }
            i = i.wrapping_add(1);
        }
    }
}
#[derive(Debug, PartialEq, Eq)]
struct tty_clamped_area {
    x: u_int,
    y: u_int,
    width: u_int,
    height: u_int,
}

fn tty_clamp_area(
    ctx: &tty_ctx,
    px: u_int,
    py: u_int,
    nx: u_int,
    ny: u_int,
) -> Option<tty_clamped_area> {
    let xoff = (ctx.rxoff as u_int).wrapping_add(px);
    let yoff = (ctx.ryoff as u_int).wrapping_add(py);
    if tty_is_visible(ctx, px, py, nx, ny) == 0 {
        return None;
    }
    let (x, width) = if xoff >= ctx.wox && xoff.wrapping_add(nx) <= ctx.wox.wrapping_add(ctx.wsx) {
        (
            (ctx.xoff as u_int).wrapping_add(px).wrapping_sub(ctx.wox),
            nx,
        )
    } else if xoff < ctx.wox && xoff.wrapping_add(nx) > ctx.wox.wrapping_add(ctx.wsx) {
        (0, ctx.wsx)
    } else if xoff < ctx.wox {
        let skip = ctx.wox.wrapping_sub((ctx.xoff as u_int).wrapping_add(px));
        (0, nx.wrapping_sub(skip))
    } else {
        let x = (ctx.xoff as u_int).wrapping_add(px).wrapping_sub(ctx.wox);
        (x, ctx.wsx.wrapping_sub(x))
    };
    if width > nx {
        unsafe {
            fatalx(|out| write!(out, "tty_clamp_area: x too big, {} > {}", width, nx));
        }
    }
    let (y, height) = if yoff >= ctx.woy && yoff.wrapping_add(ny) <= ctx.woy.wrapping_add(ctx.wsy) {
        (
            (ctx.yoff as u_int).wrapping_add(py).wrapping_sub(ctx.woy),
            ny,
        )
    } else if yoff < ctx.woy && yoff.wrapping_add(ny) > ctx.woy.wrapping_add(ctx.wsy) {
        (0, ctx.wsy)
    } else if yoff < ctx.woy {
        let skip = ctx.woy.wrapping_sub((ctx.yoff as u_int).wrapping_add(py));
        (0, ny.wrapping_sub(skip))
    } else {
        let y = (ctx.yoff as u_int).wrapping_add(py).wrapping_sub(ctx.woy);
        (y, ctx.wsy.wrapping_sub(y))
    };
    if height > ny {
        unsafe {
            fatalx(|out| write!(out, "tty_clamp_area: y too big, {} > {}", height, ny));
        }
    }
    Some(tty_clamped_area {
        x,
        y,
        width,
        height,
    })
}
unsafe fn tty_clear_area(
    mut tty: *mut tty,
    ctx: &tty_ctx,
    mut py: u_int,
    mut ny: u_int,
    mut px: u_int,
    mut nx: u_int,
    mut bg: u_int,
) {
    let terminal_client_owner = (*tty).client.upgrade().expect("terminal belongs to a live client");
    let terminal_client = terminal_client_owner.get();
    let mut c: *mut client = terminal_client;
    let defaults = &ctx.style_ctx.defaults;
    let mut yy: u_int = 0;
    let mut tmp: [::core::ffi::c_char; 64] = [0; 64];
    log_debug(format_args!(
        "{}: {}, {},{} at {},{}",
        "tty_clear_area",
        log_cstr(
            (((*c).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                as *const _
        ),
        (nx) as u32,
        (ny) as u32,
        (px) as u32,
        (py) as u32
    ));
    if nx == 0 as u_int || ny == 0 as u_int {
        return;
    }
    if (*c).overlay_check.is_none() && tty_fake_bce(&*tty, defaults, bg) == 0 {
        if px == 0 as u_int
            && px.wrapping_add(nx) >= (*tty).sx
            && py.wrapping_add(ny) >= (*tty).sy
            && tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_ED) != 0
        {
            tty_cursor(tty, 0 as u_int, py);
            tty_putcode(tty, TTYC_ED);
            return;
        }
        if (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags & TERM_DECFRA != 0 && !(bg == 8 as u_int || bg == 9 as u_int) {
            xformat(
                &mut tmp,
                format_args!(
                    "\x1B[32;{};{};{};{}$x",
                    (py.wrapping_add(1 as u_int)) as u32,
                    (px.wrapping_add(1 as u_int)) as u32,
                    (py.wrapping_add(ny)) as u32,
                    (px.wrapping_add(nx)) as u32
                ),
            );
            tty_puts(tty, std::ffi::CStr::from_ptr(tmp.as_ptr()));
            return;
        }
        if px == 0 as u_int
            && px.wrapping_add(nx) >= (*tty).sx
            && ny > 2 as u_int
            && tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_CSR) != 0
            && tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_INDN) != 0
        {
            tty_region(tty, py, py.wrapping_add(ny).wrapping_sub(1 as u_int));
            tty_margin_off(tty);
            tty_putcode_i(tty, TTYC_INDN, ny as ::core::ffi::c_int);
            return;
        }
        if nx > 2 as u_int
            && ny > 2 as u_int
            && tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_CSR) != 0
            && (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags & TERM_DECSLRM != 0
            && tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_INDN) != 0
        {
            tty_region(tty, py, py.wrapping_add(ny).wrapping_sub(1 as u_int));
            tty_margin(tty, px, px.wrapping_add(nx).wrapping_sub(1 as u_int));
            tty_putcode_i(tty, TTYC_INDN, ny as ::core::ffi::c_int);
            return;
        }
    }
    yy = py;
    while yy < py.wrapping_add(ny) {
        tty_clear_line(tty, defaults, yy, px, nx, bg);
        yy = yy.wrapping_add(1);
    }
}
unsafe fn tty_clear_pane_area(
    mut tty: *mut tty,
    ctx: &tty_ctx,
    mut py: u_int,
    mut ny: u_int,
    mut px: u_int,
    mut nx: u_int,
    mut bg: u_int,
) {
    if let Some(area) = tty_clamp_area(ctx, px, py, nx, ny) {
        tty_clear_area(tty, ctx, area.y, area.height, area.x, area.width, bg);
    }
}
unsafe fn tty_draw_pane(mut tty: *mut tty, ctx: &tty_ctx, s: &screen, mut py: u_int) {
    let terminal_client_owner = (*tty).client.upgrade().expect("terminal belongs to a live client");
    let terminal_client = terminal_client_owner.get();
    let mut nx: u_int = ctx.sx;
    let mut j: u_int = 0;
    let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
    let mut rr: *mut visible_range = ::core::ptr::null_mut::<visible_range>();
    log_debug(format_args!(
        "{}: {} {}",
        "tty_draw_pane",
        log_cstr(
            (((*terminal_client).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                as *const _
        ),
        (py) as u32
    ));
    if !ctx.flags & TTY_CTX_WINDOW_BIGGER != 0 {
        r = tty_check_overlay_range(
            tty,
            ctx.xoff as u_int,
            (ctx.yoff as u_int).wrapping_add(py),
            nx,
        );
        j = 0 as u_int;
        while j < (*r).used {
            rr = &raw mut (&mut (*r).storage)[j as usize];
            if !((*rr).nx == 0 as u_int) {
                tty_draw_line(
                    tty,
                    s,
                    (*rr).px.wrapping_sub(ctx.xoff as u_int),
                    py,
                    (*rr).nx,
                    (*rr).px,
                    (ctx.yoff as u_int).wrapping_add(py),
                    Some(&ctx.style_ctx),
                );
            }
            j = j.wrapping_add(1);
        }
        return;
    }
    if let Some(tty_clamped_line {
        skip: i,
        x,
        y: ry,
        width: rx,
    }) = tty_clamp_line(ctx, 0 as u_int, py, nx)
    {
        r = tty_check_overlay_range(tty, x, ry, rx);
        j = 0 as u_int;
        while j < (*r).used {
            rr = &raw mut (&mut (*r).storage)[j as usize];
            if !((*rr).nx == 0 as u_int) {
                tty_draw_line(
                    tty,
                    s,
                    i.wrapping_add((*rr).px).wrapping_sub(x),
                    py,
                    (*rr).nx,
                    (*rr).px,
                    ry,
                    Some(&ctx.style_ctx),
                );
            }
            j = j.wrapping_add(1);
        }
    }
}
pub unsafe fn tty_cmd_redrawline(mut tty: *mut tty, ctx: &tty_ctx, s: &screen) {
    let mut j: u_int = 0;
    let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
    let mut rr: *mut visible_range = ::core::ptr::null_mut::<visible_range>();
    if let Some(tty_clamped_line {
        skip: i,
        x,
        y: ry,
        width: rx,
    }) = tty_clamp_line(ctx, ctx.ocx, ctx.ocy, ctx.data.count())
    {
        r = tty_check_overlay_range(tty, x, ry, rx);
        j = 0 as u_int;
        while j < (*r).used {
            rr = &raw mut (&mut (*r).storage)[j as usize];
            if !((*rr).nx == 0 as u_int) {
                tty_draw_line(
                    tty,
                    s,
                    ctx.ocx
                        .wrapping_add(i)
                        .wrapping_add((*rr).px)
                        .wrapping_sub(x),
                    ctx.ocy,
                    (*rr).nx,
                    (*rr).px,
                    ry,
                    Some(&ctx.style_ctx),
                );
            }
            j = j.wrapping_add(1);
        }
    }
}
pub unsafe fn tty_check_codeset(tty: &tty, gc: &grid_cell) -> grid_cell {
    if gc.data.size as ::core::ffi::c_int == 1 as ::core::ffi::c_int
        && (gc.data.data[0] as ::core::ffi::c_int) < 0x7f as ::core::ffi::c_int
    {
        return *gc;
    }
    if gc.flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
        return *gc;
    }

    let terminal_client_owner = tty.client.upgrade().expect("terminal belongs to a live client");
    let terminal_client = terminal_client_owner.get();
    if (*terminal_client).flags & CLIENT_UTF8 as uint64_t != 0 {
        return *gc;
    }
    let mut new = *gc;
    if let Some(ch) = CStr::from_bytes_until_nul(&gc.data.data)
        .ok()
        .and_then(|text| tty_acs_reverse_get(text, gc.data.size as usize))
    {
        utf8_set(&mut new.data, ch);
        new.attr = (new.attr as ::core::ffi::c_int | GRID_ATTR_CHARSET) as u_short;
        return new;
    }
    new.data.size = gc.data.width;
    if new.data.size as ::core::ffi::c_int > UTF8_SIZE {
        new.data.size = UTF8_SIZE as u_char;
    }
    new.data.data[..new.data.size as usize].fill(b'_');
    return new;
}
unsafe fn tty_check_overlay(mut tty: *mut tty, mut px: u_int, mut py: u_int) -> ::core::ffi::c_int {
    let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
    r = tty_check_overlay_range(tty, px, py, 1 as u_int);
    return (server_client_ranges_is_empty(r) == 0) as ::core::ffi::c_int;
}
pub unsafe fn tty_check_overlay_range(
    mut tty: *mut tty,
    mut px: u_int,
    mut py: u_int,
    mut nx: u_int,
) -> *mut visible_ranges {
    let terminal_client_owner = (*tty).client.upgrade().expect("terminal belongs to a live client");
    let terminal_client = terminal_client_owner.get();
    let mut c: *mut client = terminal_client;
    if let Some(ranges) = server_client_overlay_check(c, px, py, nx) {
        (*tty).r = ranges;
    } else {
        server_client_ensure_ranges(&raw mut (*tty).r, 1 as u_int);
        (&mut (*tty).r.storage)[0].px = px;
        (&mut (*tty).r.storage)[0].nx = nx;
        (*tty).r.used = 1 as u_int;
    }
    &raw mut (*tty).r
}
pub unsafe fn tty_sync_start(mut tty: *mut tty) {
    let terminal_client_owner = (*tty).client.upgrade().expect("terminal belongs to a live client");
    let terminal_client = terminal_client_owner.get();
    if (*tty).flags & TTY_BLOCK != 0 {
        return;
    }
    if (*tty).flags & TTY_SYNCING != 0 {
        return;
    }
    (*tty).flags |= TTY_SYNCING;
    if tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_SYNC) != 0 {
        log_debug(format_args!(
            "{} sync start",
            log_cstr(
                (((*terminal_client).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                    as *const _
            )
        ));
        tty_putcode_i(tty, TTYC_SYNC, 1 as ::core::ffi::c_int);
    }
}
pub unsafe fn tty_sync_end(mut tty: *mut tty) {
    let terminal_client_owner = (*tty).client.upgrade().expect("terminal belongs to a live client");
    let terminal_client = terminal_client_owner.get();
    if (*tty).flags & TTY_BLOCK != 0 {
        return;
    }
    if !(*tty).flags & TTY_SYNCING != 0 {
        return;
    }
    (*tty).flags &= !TTY_SYNCING;
    if tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_SYNC) != 0 {
        log_debug(format_args!(
            "{} sync end",
            log_cstr(
                (((*terminal_client).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                    as *const _
            )
        ));
        tty_putcode_i(tty, TTYC_SYNC, 2 as ::core::ffi::c_int);
    }
}
unsafe fn tty_client_ready(ctx: &tty_ctx, mut c: *mut client) -> ::core::ffi::c_int {
    if (*c).session_ptr().is_null() || tty_term_owner_ptr(&(*c).tty.term).map_or(std::ptr::null(), |term| term).is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if (*c).flags & CLIENT_SUSPENDED as uint64_t != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if ctx.flags & TTY_CTX_INVISIBLE_PANES != 0 {
        return 1 as ::core::ffi::c_int;
    }
    if (*c).flags & CLIENT_REDRAWWINDOW as uint64_t != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if (*c).tty.flags & TTY_FREEZE != 0 {
        return 0 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
pub unsafe fn tty_write(mut cmdfn: impl FnMut(*mut tty, &tty_ctx), ctx: &mut tty_ctx) {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut state: ::core::ffi::c_int = 0;
    let Some(mut set_client_cb) = ctx.set_client_cb.take() else {
        return;
    };
    let mut registry_c_owner = clients.first();
    c = registry_c_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    while !c.is_null() {
        if tty_client_ready(ctx, c) != 0 {
            state = set_client_cb(ctx, &mut *c);
            if state == -(1 as ::core::ffi::c_int) {
                break;
            }
            if !(state == 0 as ::core::ffi::c_int) {
                cmdfn(&raw mut (*c).tty, ctx);
            }
        }
        registry_c_owner = clients.next(registry_c_owner.as_ref().expect("current registry client"));
        c = registry_c_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    }
    if ctx.set_client_cb.is_none() {
        ctx.set_client_cb = Some(set_client_cb);
    }
}
pub unsafe fn tty_cmd_insertcharacter(mut tty: *mut tty, ctx: &tty_ctx, s: &screen) {
    let terminal_client_owner = (*tty).client.upgrade().expect("terminal belongs to a live client");
    let terminal_client = terminal_client_owner.get();
    let mut c: *mut client = terminal_client;
    if ctx.flags & TTY_CTX_WINDOW_BIGGER != 0
        || !(ctx.xoff == 0 as ::core::ffi::c_int && ctx.sx >= (*tty).sx)
        || tty_fake_bce(&*tty, &ctx.style_ctx.defaults, ctx.bg) != 0
        || tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_ICH) == 0 && tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_ICH1) == 0
        || (*c).overlay_check.is_some()
    {
        tty_draw_pane(tty, ctx, s, ctx.ocy);
        return;
    }
    tty_default_attributes(tty, ctx.bg, Some(&ctx.style_ctx));
    tty_cursor_pane(tty, ctx, ctx.ocx, ctx.ocy);
    tty_emulate_repeat(tty, TTYC_ICH, TTYC_ICH1, ctx.data.count());
}
pub unsafe fn tty_cmd_deletecharacter(mut tty: *mut tty, ctx: &tty_ctx, s: &screen) {
    let terminal_client_owner = (*tty).client.upgrade().expect("terminal belongs to a live client");
    let terminal_client = terminal_client_owner.get();
    let mut c: *mut client = terminal_client;
    if ctx.flags & TTY_CTX_WINDOW_BIGGER != 0
        || !(ctx.xoff == 0 as ::core::ffi::c_int && ctx.sx >= (*tty).sx)
        || tty_fake_bce(&*tty, &ctx.style_ctx.defaults, ctx.bg) != 0
        || tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_DCH) == 0 && tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_DCH1) == 0
        || (*c).overlay_check.is_some()
    {
        tty_draw_pane(tty, ctx, s, ctx.ocy);
        return;
    }
    tty_default_attributes(tty, ctx.bg, Some(&ctx.style_ctx));
    tty_cursor_pane(tty, ctx, ctx.ocx, ctx.ocy);
    tty_emulate_repeat(tty, TTYC_DCH, TTYC_DCH1, ctx.data.count());
}
pub unsafe fn tty_cmd_clearcharacter(mut tty: *mut tty, ctx: &tty_ctx) {
    tty_default_attributes(tty, ctx.bg, Some(&ctx.style_ctx));
    tty_clear_pane_line(tty, ctx, ctx.ocy, ctx.ocx, ctx.data.count(), ctx.bg);
}
pub unsafe fn tty_cmd_insertline(mut tty: *mut tty, ctx: &tty_ctx, s: &screen) {
    let terminal_client_owner = (*tty).client.upgrade().expect("terminal belongs to a live client");
    let terminal_client = terminal_client_owner.get();
    let mut c: *mut client = terminal_client;
    if ctx.flags & TTY_CTX_WINDOW_BIGGER != 0
        || !(ctx.xoff == 0 as ::core::ffi::c_int && ctx.sx >= (*tty).sx)
        || tty_fake_bce(&*tty, &ctx.style_ctx.defaults, ctx.bg) != 0
        || tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_CSR) == 0
        || tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_IL1) == 0
        || ctx.sx == 1 as u_int
        || ctx.sy == 1 as u_int
        || (*c).overlay_check.is_some()
    {
        tty_redraw_region(tty, ctx, s);
        return;
    }
    tty_default_attributes(tty, ctx.bg, Some(&ctx.style_ctx));
    tty_region_pane(tty, ctx, ctx.orupper, ctx.orlower);
    tty_margin_off(tty);
    tty_cursor_pane(tty, ctx, ctx.ocx, ctx.ocy);
    tty_emulate_repeat(tty, TTYC_IL, TTYC_IL1, ctx.data.count());
    (*tty).cy = UINT_MAX as u_int;
    (*tty).cx = (*tty).cy;
}
pub unsafe fn tty_cmd_deleteline(mut tty: *mut tty, ctx: &tty_ctx, s: &screen) {
    let terminal_client_owner = (*tty).client.upgrade().expect("terminal belongs to a live client");
    let terminal_client = terminal_client_owner.get();
    let mut c: *mut client = terminal_client;
    if ctx.flags & TTY_CTX_WINDOW_BIGGER != 0
        || !(ctx.xoff == 0 as ::core::ffi::c_int && ctx.sx >= (*tty).sx)
        || tty_fake_bce(&*tty, &ctx.style_ctx.defaults, ctx.bg) != 0
        || tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_CSR) == 0
        || tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_DL1) == 0
        || ctx.sx == 1 as u_int
        || ctx.sy == 1 as u_int
        || (*c).overlay_check.is_some()
    {
        tty_redraw_region(tty, ctx, s);
        return;
    }
    tty_default_attributes(tty, ctx.bg, Some(&ctx.style_ctx));
    tty_region_pane(tty, ctx, ctx.orupper, ctx.orlower);
    tty_margin_off(tty);
    tty_cursor_pane(tty, ctx, ctx.ocx, ctx.ocy);
    tty_emulate_repeat(tty, TTYC_DL, TTYC_DL1, ctx.data.count());
    (*tty).cy = UINT_MAX as u_int;
    (*tty).cx = (*tty).cy;
}
pub unsafe fn tty_cmd_reverseindex(mut tty: *mut tty, ctx: &tty_ctx, s: &screen) {
    let terminal_client_owner = (*tty).client.upgrade().expect("terminal belongs to a live client");
    let terminal_client = terminal_client_owner.get();
    let mut c: *mut client = terminal_client;
    if ctx.ocy != ctx.orupper {
        return;
    }
    if ctx.flags & TTY_CTX_WINDOW_BIGGER != 0
        || !(ctx.xoff == 0 as ::core::ffi::c_int && ctx.sx >= (*tty).sx)
            && (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags & TERM_DECSLRM == 0
        || tty_fake_bce(&*tty, &ctx.style_ctx.defaults, 8 as u_int) != 0
        || tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_CSR) == 0
        || tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_RI) == 0 && tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_RIN) == 0
        || ctx.sx == 1 as u_int
        || ctx.sy == 1 as u_int
        || (*c).overlay_check.is_some()
    {
        tty_redraw_region(tty, ctx, s);
        return;
    }
    tty_default_attributes(tty, ctx.bg, Some(&ctx.style_ctx));
    tty_region_pane(tty, ctx, ctx.orupper, ctx.orlower);
    tty_margin_pane(tty, ctx);
    tty_cursor_pane(tty, ctx, ctx.ocx, ctx.orupper);
    if tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_RI) != 0 {
        tty_putcode(tty, TTYC_RI);
    } else {
        tty_putcode_i(tty, TTYC_RIN, 1 as ::core::ffi::c_int);
    };
}
pub unsafe fn tty_cmd_scrollup(mut tty: *mut tty, ctx: &tty_ctx, s: &screen) {
    let terminal_client_owner = (*tty).client.upgrade().expect("terminal belongs to a live client");
    let terminal_client = terminal_client_owner.get();
    let mut c: *mut client = terminal_client;
    let mut i: u_int = 0;
    if ctx.flags & TTY_CTX_WINDOW_BIGGER != 0
        || !(ctx.xoff == 0 as ::core::ffi::c_int && ctx.sx >= (*tty).sx)
            && (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags & TERM_DECSLRM == 0
        || tty_fake_bce(&*tty, &ctx.style_ctx.defaults, 8 as u_int) != 0
        || tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_CSR) == 0
        || ctx.sx == 1 as u_int
        || ctx.sy == 1 as u_int
        || (*c).overlay_check.is_some()
    {
        tty_redraw_region(tty, ctx, s);
        return;
    }
    tty_default_attributes(tty, ctx.bg, Some(&ctx.style_ctx));
    tty_region_pane(tty, ctx, ctx.orupper, ctx.orlower);
    tty_margin_pane(tty, ctx);
    if ctx.data.count() == 1 as u_int || tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_INDN) == 0 {
        if (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags & TERM_DECSLRM == 0 {
            tty_cursor(tty, 0 as u_int, (*tty).rlower);
        } else {
            tty_cursor(tty, (*tty).rright, (*tty).rlower);
        }
        i = 0 as u_int;
        while i < ctx.data.count() {
            tty_putc(tty, '\n' as i32 as u_char);
            i = i.wrapping_add(1);
        }
    } else {
        if (*tty).cy == UINT_MAX {
            tty_cursor(tty, 0 as u_int, 0 as u_int);
        } else {
            tty_cursor(tty, 0 as u_int, (*tty).cy);
        }
        tty_putcode_i(tty, TTYC_INDN, ctx.data.count() as ::core::ffi::c_int);
    };
}
pub unsafe fn tty_cmd_scrolldown(mut tty: *mut tty, ctx: &tty_ctx, s: &screen) {
    let terminal_client_owner = (*tty).client.upgrade().expect("terminal belongs to a live client");
    let terminal_client = terminal_client_owner.get();
    let mut i: u_int = 0;
    let mut c: *mut client = terminal_client;
    if ctx.flags & TTY_CTX_WINDOW_BIGGER != 0
        || !(ctx.xoff == 0 as ::core::ffi::c_int && ctx.sx >= (*tty).sx)
            && (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags & TERM_DECSLRM == 0
        || tty_fake_bce(&*tty, &ctx.style_ctx.defaults, 8 as u_int) != 0
        || tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_CSR) == 0
        || tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_RI) == 0 && tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_RIN) == 0
        || ctx.sx == 1 as u_int
        || ctx.sy == 1 as u_int
        || (*c).overlay_check.is_some()
    {
        tty_redraw_region(tty, ctx, s);
        return;
    }
    tty_default_attributes(tty, ctx.bg, Some(&ctx.style_ctx));
    tty_region_pane(tty, ctx, ctx.orupper, ctx.orlower);
    tty_margin_pane(tty, ctx);
    tty_cursor_pane(tty, ctx, ctx.ocx, ctx.orupper);
    if tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_RIN) != 0 {
        tty_putcode_i(tty, TTYC_RIN, ctx.data.count() as ::core::ffi::c_int);
    } else {
        i = 0 as u_int;
        while i < ctx.data.count() {
            tty_putcode(tty, TTYC_RI);
            i = i.wrapping_add(1);
        }
    };
}
pub unsafe fn tty_cmd_clearendofscreen(mut tty: *mut tty, ctx: &tty_ctx) {
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut nx: u_int = 0;
    let mut ny: u_int = 0;
    tty_default_attributes(tty, ctx.bg, Some(&ctx.style_ctx));
    tty_region_pane(tty, ctx, 0 as u_int, ctx.sy.wrapping_sub(1 as u_int));
    tty_margin_off(tty);
    px = 0 as u_int;
    nx = ctx.sx;
    py = ctx.ocy.wrapping_add(1 as u_int);
    ny = ctx.sy.wrapping_sub(ctx.ocy).wrapping_sub(1 as u_int);
    tty_clear_pane_area(tty, ctx, py, ny, px, nx, ctx.bg);
    px = ctx.ocx;
    nx = ctx.sx.wrapping_sub(ctx.ocx);
    py = ctx.ocy;
    tty_clear_pane_line(tty, ctx, py, px, nx, ctx.bg);
}
pub unsafe fn tty_cmd_clearstartofscreen(mut tty: *mut tty, ctx: &tty_ctx) {
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut nx: u_int = 0;
    let mut ny: u_int = 0;
    tty_default_attributes(tty, ctx.bg, Some(&ctx.style_ctx));
    tty_region_pane(tty, ctx, 0 as u_int, ctx.sy.wrapping_sub(1 as u_int));
    tty_margin_off(tty);
    px = 0 as u_int;
    nx = ctx.sx;
    py = 0 as u_int;
    ny = ctx.ocy;
    tty_clear_pane_area(tty, ctx, py, ny, px, nx, ctx.bg);
    px = 0 as u_int;
    nx = ctx.ocx.wrapping_add(1 as u_int);
    py = ctx.ocy;
    tty_clear_pane_line(tty, ctx, py, px, nx, ctx.bg);
}
pub unsafe fn tty_cmd_clearscreen(mut tty: *mut tty, ctx: &tty_ctx) {
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut nx: u_int = 0;
    let mut ny: u_int = 0;
    tty_default_attributes(tty, ctx.bg, Some(&ctx.style_ctx));
    tty_region_pane(tty, ctx, 0 as u_int, ctx.sy.wrapping_sub(1 as u_int));
    tty_margin_off(tty);
    px = 0 as u_int;
    nx = ctx.sx;
    py = 0 as u_int;
    ny = ctx.sy;
    tty_clear_pane_area(tty, ctx, py, ny, px, nx, ctx.bg);
}
pub unsafe fn tty_cmd_alignmenttest(mut tty: *mut tty, ctx: &tty_ctx) {
    let terminal_client_owner = (*tty).client.upgrade().expect("terminal belongs to a live client");
    let terminal_client = terminal_client_owner.get();
    let mut c: *mut client = terminal_client;
    let mut i: u_int = 0;
    let mut j: u_int = 0;
    if ctx.flags & TTY_CTX_WINDOW_BIGGER != 0 || (*c).overlay_check.is_some() {
        ctx.redraw_cb.as_ref().expect("non-null redraw callback")(ctx);
        return;
    }
    tty_attributes(tty, &grid_default_cell, Some(&ctx.style_ctx));
    tty_region_pane(tty, ctx, 0 as u_int, ctx.sy.wrapping_sub(1 as u_int));
    tty_margin_off(tty);
    j = 0 as u_int;
    while j < ctx.sy {
        tty_cursor_pane(tty, ctx, 0 as u_int, j);
        i = 0 as u_int;
        while i < ctx.sx {
            tty_putc(tty, 'E' as i32 as u_char);
            i = i.wrapping_add(1);
        }
        j = j.wrapping_add(1);
    }
}
pub unsafe fn tty_cmd_cell(mut tty: *mut tty, ctx: &tty_ctx, s: &screen, gc: &grid_cell) {
    let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut i: u_int = 0;
    let mut vis: u_int = 0 as u_int;
    px = (ctx.xoff as u_int)
        .wrapping_add(ctx.ocx)
        .wrapping_sub(ctx.wox);
    py = (ctx.yoff as u_int)
        .wrapping_add(ctx.ocy)
        .wrapping_sub(ctx.woy);
    if tty_is_visible(ctx, ctx.ocx, ctx.ocy, 1 as u_int, 1 as u_int) == 0 {
        return;
    }
    if gc.data.width as ::core::ffi::c_int == 1 as ::core::ffi::c_int
        && tty_check_overlay(tty, px, py) == 0
    {
        return;
    }
    if gc.data.width as ::core::ffi::c_int > 1 as ::core::ffi::c_int {
        r = tty_check_overlay_range(tty, px, py, gc.data.width as u_int);
        i = 0 as u_int;
        while i < (*r).used {
            vis = vis.wrapping_add((&(*r).storage)[i as usize].nx);
            i = i.wrapping_add(1);
        }
        if vis < gc.data.width as u_int {
            tty_draw_line(
                tty,
                s,
                s.cx,
                s.cy,
                gc.data.width as u_int,
                px,
                py,
                Some(&ctx.style_ctx),
            );
            return;
        }
    }
    if (ctx.xoff as u_int)
        .wrapping_add(ctx.ocx)
        .wrapping_sub(ctx.wox)
        > (*tty).sx.wrapping_sub(1 as u_int)
        && ctx.ocy == ctx.orlower
        && (ctx.xoff == 0 as ::core::ffi::c_int && ctx.sx >= (*tty).sx)
    {
        tty_region_pane(tty, ctx, ctx.orupper, ctx.orlower);
    }
    tty_margin_off(tty);
    if ctx.flags & TTY_CTX_CELL_INVALIDATE != 0 {
        tty_invalidate(tty);
    }
    tty_cursor_pane_unless_wrap(tty, ctx, ctx.ocx, ctx.ocy);
    tty_cell(tty, gc, Some(&ctx.style_ctx));
    if ctx.flags & TTY_CTX_CELL_INVALIDATE != 0 {
        tty_invalidate(tty);
    }
}
pub unsafe fn tty_cmd_cells(mut tty: *mut tty, ctx: &tty_ctx, s: &screen, gc: &grid_cell) {
    let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
    let mut ri: *mut visible_range = ::core::ptr::null_mut::<visible_range>();
    let mut i: u_int = 0;
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut cx: u_int = 0;
    let data = ctx.data.bytes();
    let n = data.len();
    if tty_is_visible(ctx, ctx.ocx, ctx.ocy, n as u_int, 1 as u_int) == 0 {
        return;
    }
    if ctx.flags & TTY_CTX_WINDOW_BIGGER != 0
        && ((ctx.xoff as u_int).wrapping_add(ctx.ocx) < ctx.wox
            || ((ctx.xoff as u_int).wrapping_add(ctx.ocx) as size_t).wrapping_add(n)
                > ctx.wox.wrapping_add(ctx.wsx) as size_t)
    {
        if !ctx.flags & TTY_CTX_WRAPPED != 0
            || !(ctx.xoff == 0 as ::core::ffi::c_int && ctx.sx >= (*tty).sx)
            || (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags & TERM_NOAM != 0
            || (ctx.xoff as u_int).wrapping_add(ctx.ocx) != 0 as u_int
            || (ctx.yoff as u_int).wrapping_add(ctx.ocy) != (*tty).cy.wrapping_add(1 as u_int)
            || (*tty).cx < (*tty).sx
            || (*tty).cy == (*tty).rlower
        {
            tty_draw_pane(tty, ctx, s, ctx.ocy);
        } else {
            ctx.redraw_cb.as_ref().expect("non-null redraw callback")(ctx);
        }
        return;
    }
    tty_margin_off(tty);
    tty_cursor_pane_unless_wrap(tty, ctx, ctx.ocx, ctx.ocy);
    tty_attributes(tty, gc, Some(&ctx.style_ctx));
    px = (ctx.xoff as u_int)
        .wrapping_add(ctx.ocx)
        .wrapping_sub(ctx.wox);
    py = (ctx.yoff as u_int)
        .wrapping_add(ctx.ocy)
        .wrapping_sub(ctx.woy);
    r = tty_check_overlay_range(tty, px, py, n as u_int);
    i = 0 as u_int;
    while i < (*r).used {
        ri = &raw mut (&mut (*r).storage)[i as usize];
        if (*ri).nx != 0 as u_int {
            cx = (*ri)
                .px
                .wrapping_sub(ctx.xoff as u_int)
                .wrapping_add(ctx.wox);
            tty_cursor_pane_unless_wrap(tty, ctx, cx, ctx.ocy);
            tty_putn(
                tty,
                &data[((*ri).px - px) as usize..((*ri).px - px + (*ri).nx) as usize],
                (*ri).nx,
            );
        }
        i = i.wrapping_add(1);
    }
}
pub unsafe fn tty_cmd_setselection(mut tty: *mut tty, ctx: &tty_ctx) {
    let (clip, data) = ctx.data.selection();
    tty_set_selection(tty, clip, data);
}
pub unsafe fn tty_set_selection(mut tty: *mut tty, clip: &CStr, data: &[u8]) {
    let mut size: size_t = 0;
    if !(*tty).flags & TTY_STARTED != 0 {
        return;
    }
    if tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_MS) == 0 {
        return;
    }
    size = (4 as size_t)
        .wrapping_mul(
            data.len()
                .wrapping_add(2 as size_t)
                .wrapping_div(3 as size_t),
        )
        .wrapping_add(1 as size_t);
    let mut encoded = vec![0; size];
    __b64_ntop(data.as_ptr(), data.len(), encoded.as_mut_ptr().cast(), size);
    (*tty).flags |= TTY_NOBLOCK;
    tty_putcode_ss(tty, TTYC_MS, clip.as_ptr(), encoded.as_ptr().cast());
}
pub unsafe fn tty_cmd_rawstring(mut tty: *mut tty, ctx: &tty_ctx) {
    (*tty).flags |= TTY_NOBLOCK;
    tty_add(tty, ctx.data.bytes());
    tty_invalidate(tty);
}
pub unsafe fn tty_cmd_syncstart(mut tty: *mut tty, ctx: &tty_ctx) {
    let terminal_client_owner = (*tty).client.upgrade().expect("terminal belongs to a live client");
    let terminal_client = terminal_client_owner.get();
    let mut c: *mut client = terminal_client;
    if ctx.flags & TTY_CTX_OVERLAY_SYNC != 0 && ctx.flags & TTY_CTX_SYNC != 0 {
        tty_sync_start(tty);
    } else if !ctx.flags & TTY_CTX_OVERLAY_SYNC != 0 {
        if ctx.flags & TTY_CTX_SYNC != 0 || (*c).overlay_draw.is_some() {
            tty_sync_start(tty);
        }
    }
}
pub unsafe fn tty_cell(tty: *mut tty, gc: &grid_cell, style_ctx: Option<&tty_style_ctx>) {
    if (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags & TERM_NOAM != 0
        && (*tty).cy == (*tty).sy.wrapping_sub(1)
        && (*tty).cx == (*tty).sx.wrapping_sub(1)
    {
        return;
    }
    if gc.flags as i32 & GRID_FLAG_PADDING != 0 {
        return;
    }
    if tty_check_overlay(tty, (*tty).cx, (*tty).cy) == 0 {
        return;
    }
    let converted = tty_check_codeset(&*tty, gc);
    tty_attributes(tty, &converted, style_ctx);
    if converted.data.size == 1 {
        let byte = converted.data.data[0];
        if byte < 0x20 || byte == 0x7f {
            return;
        }
        tty_putc(tty, byte);
        return;
    }
    tty_putn(
        tty,
        &converted.data.data[..converted.data.size as usize],
        converted.data.width as u_int,
    );
}
pub unsafe fn tty_reset(mut tty: *mut tty) {
    let mut gc: *mut grid_cell = &raw mut (*tty).cell;
    if !grid_cells_equal(&*gc, &grid_default_cell) {
        if (*gc).link != 0 as u_int {
            tty_putcode_ss(
                tty,
                TTYC_HLS,
                b"\0" as *const u8 as *const ::core::ffi::c_char,
                b"\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        if (*gc).attr as ::core::ffi::c_int & GRID_ATTR_CHARSET != 0 && tty_acs_needed(tty.as_ref()) != 0 {
            tty_putcode(tty, TTYC_RMACS);
        }
        tty_putcode(tty, TTYC_SGR0);
        memcpy(
            gc as *mut ::core::ffi::c_void,
            &raw const grid_default_cell as *const ::core::ffi::c_void,
            ::core::mem::size_of::<grid_cell>() as size_t,
        );
    }
    memcpy(
        &raw mut (*tty).last_cell as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
}
pub unsafe fn tty_invalidate(mut tty: *mut tty) {
    memcpy(
        &raw mut (*tty).cell as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    memcpy(
        &raw mut (*tty).last_cell as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    (*tty).cy = UINT_MAX as u_int;
    (*tty).cx = (*tty).cy;
    (*tty).rleft = UINT_MAX as u_int;
    (*tty).rupper = (*tty).rleft;
    (*tty).rright = UINT_MAX as u_int;
    (*tty).rlower = (*tty).rright;
    if (*tty).flags & TTY_STARTED != 0 {
        if (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags & TERM_DECSLRM != 0 {
            tty_putcode(tty, TTYC_ENMG);
        }
        tty_putcode(tty, TTYC_SGR0);
        (*tty).mode = ALL_MODES;
        tty_update_mode(tty, MODE_CURSOR, None);
        tty_cursor(tty, 0 as u_int, 0 as u_int);
        tty_region_off(tty);
        tty_margin_off(tty);
    } else {
        (*tty).mode = MODE_CURSOR;
    };
}
pub unsafe fn tty_region_off(mut tty: *mut tty) {
    tty_region(tty, 0 as u_int, (*tty).sy.wrapping_sub(1 as u_int));
}
unsafe fn tty_region_pane(mut tty: *mut tty, ctx: &tty_ctx, mut rupper: u_int, mut rlower: u_int) {
    tty_region(
        tty,
        (ctx.yoff as u_int)
            .wrapping_add(rupper)
            .wrapping_sub(ctx.woy),
        (ctx.yoff as u_int)
            .wrapping_add(rlower)
            .wrapping_sub(ctx.woy),
    );
}
unsafe fn tty_region(mut tty: *mut tty, mut rupper: u_int, mut rlower: u_int) {
    if (*tty).rlower == rlower && (*tty).rupper == rupper {
        return;
    }
    if tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_CSR) == 0 {
        return;
    }
    (*tty).rupper = rupper;
    (*tty).rlower = rlower;
    if (*tty).cx >= (*tty).sx {
        if (*tty).cy == UINT_MAX {
            tty_cursor(tty, 0 as u_int, 0 as u_int);
        } else {
            tty_cursor(tty, 0 as u_int, (*tty).cy);
        }
    }
    tty_putcode_ii(
        tty,
        TTYC_CSR,
        (*tty).rupper as ::core::ffi::c_int,
        (*tty).rlower as ::core::ffi::c_int,
    );
    (*tty).cy = UINT_MAX as u_int;
    (*tty).cx = (*tty).cy;
}
pub unsafe fn tty_margin_off(mut tty: *mut tty) {
    tty_margin(tty, 0 as u_int, (*tty).sx.wrapping_sub(1 as u_int));
}
unsafe fn tty_margin_pane(mut tty: *mut tty, ctx: &tty_ctx) {
    let mut l: ::core::ffi::c_int = 0;
    let mut r: ::core::ffi::c_int = 0;
    l = (ctx.xoff as u_int).wrapping_sub(ctx.wox) as ::core::ffi::c_int;
    r = (ctx.xoff as u_int)
        .wrapping_add(ctx.sx)
        .wrapping_sub(1 as u_int)
        .wrapping_sub(ctx.wox) as ::core::ffi::c_int;
    if l < 0 as ::core::ffi::c_int {
        l = 0 as ::core::ffi::c_int;
    }
    if l > ctx.wsx as ::core::ffi::c_int {
        l = ctx.wsx as ::core::ffi::c_int;
    }
    if r < 0 as ::core::ffi::c_int {
        r = 0 as ::core::ffi::c_int;
    }
    if r > ctx.wsx as ::core::ffi::c_int {
        r = ctx.wsx as ::core::ffi::c_int;
    }
    tty_margin(tty, l as u_int, r as u_int);
}
unsafe fn tty_margin(mut tty: *mut tty, mut rleft: u_int, mut rright: u_int) {
    if (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags & TERM_DECSLRM == 0 {
        return;
    }
    if (*tty).rleft == rleft && (*tty).rright == rright {
        return;
    }
    tty_putcode_ii(
        tty,
        TTYC_CSR,
        (*tty).rupper as ::core::ffi::c_int,
        (*tty).rlower as ::core::ffi::c_int,
    );
    (*tty).rleft = rleft;
    (*tty).rright = rright;
    if rleft == 0 as u_int && rright == (*tty).sx.wrapping_sub(1 as u_int) {
        tty_putcode(tty, TTYC_CLMG);
    } else {
        tty_putcode_ii(
            tty,
            TTYC_CMG,
            rleft as ::core::ffi::c_int,
            rright as ::core::ffi::c_int,
        );
    }
    (*tty).cy = UINT_MAX as u_int;
    (*tty).cx = (*tty).cy;
}
unsafe fn tty_cursor_pane_unless_wrap(
    mut tty: *mut tty,
    ctx: &tty_ctx,
    mut cx: u_int,
    mut cy: u_int,
) {
    if !ctx.flags & TTY_CTX_WRAPPED != 0
        || !(ctx.xoff == 0 as ::core::ffi::c_int && ctx.sx >= (*tty).sx)
        || (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags & TERM_NOAM != 0
        || (ctx.xoff as u_int).wrapping_add(cx) != 0 as u_int
        || (ctx.yoff as u_int).wrapping_add(cy) != (*tty).cy.wrapping_add(1 as u_int)
        || (*tty).cx < (*tty).sx
        || (*tty).cy == (*tty).rlower
    {
        tty_cursor_pane(tty, ctx, cx, cy);
    } else {
        log_debug(format_args!(
            "{}: will wrap at {},{}",
            "tty_cursor_pane_unless_wrap",
            ((*tty).cx) as u32,
            ((*tty).cy) as u32
        ));
    };
}
unsafe fn tty_cursor_pane(mut tty: *mut tty, ctx: &tty_ctx, mut cx: u_int, mut cy: u_int) {
    tty_cursor(
        tty,
        (ctx.xoff as u_int).wrapping_add(cx).wrapping_sub(ctx.wox),
        (ctx.yoff as u_int).wrapping_add(cy).wrapping_sub(ctx.woy),
    );
}
pub unsafe fn tty_cursor(mut tty: *mut tty, mut cx: u_int, mut cy: u_int) {
    let mut current_block: u64;
    let mut term: *const tty_term = tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term);
    let mut thisx: u_int = 0;
    let mut thisy: u_int = 0;
    let mut change: ::core::ffi::c_int = 0;
    if (*tty).flags & TTY_BLOCK != 0 {
        return;
    }
    thisx = (*tty).cx;
    thisy = (*tty).cy;
    if cx == thisx && cy == thisy && cx == (*tty).sx {
        return;
    }
    if cx > (*tty).sx.wrapping_sub(1 as u_int) {
        log_debug(format_args!(
            "{}: x too big {} > {}",
            "tty_cursor",
            (cx) as u32,
            ((*tty).sx.wrapping_sub(1 as u_int)) as u32
        ));
        cx = (*tty).sx.wrapping_sub(1 as u_int);
    }
    if cx == thisx && cy == thisy {
        return;
    }
    if thisx > (*tty).sx.wrapping_sub(1 as u_int) {
        current_block = 11555347550778173209;
    } else if cx == 0 as u_int && cy == 0 as u_int && tty_term_has(term, TTYC_HOME) != 0 {
        tty_putcode(tty, TTYC_HOME);
        current_block = 5411263895410993842;
    } else if cx == 0 as u_int
        && cy == thisy.wrapping_add(1 as u_int)
        && thisy != (*tty).rlower
        && ((*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags & TERM_DECSLRM == 0 || (*tty).rleft == 0 as u_int)
    {
        tty_putc(tty, '\r' as i32 as u_char);
        tty_putc(tty, '\n' as i32 as u_char);
        current_block = 5411263895410993842;
    } else if cy == thisy {
        if cx == 0 as u_int
            && ((*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags & TERM_DECSLRM == 0 || (*tty).rleft == 0 as u_int)
        {
            tty_putc(tty, '\r' as i32 as u_char);
            current_block = 5411263895410993842;
        } else if cx == thisx.wrapping_sub(1 as u_int) && tty_term_has(term, TTYC_CUB1) != 0 {
            tty_putcode(tty, TTYC_CUB1);
            current_block = 5411263895410993842;
        } else if cx == thisx.wrapping_add(1 as u_int) && tty_term_has(term, TTYC_CUF1) != 0 {
            tty_putcode(tty, TTYC_CUF1);
            current_block = 5411263895410993842;
        } else {
            change = thisx.wrapping_sub(cx) as ::core::ffi::c_int;
            if abs(change) as u_int > cx && tty_term_has(term, TTYC_HPA) != 0 {
                tty_putcode_i(tty, TTYC_HPA, cx as ::core::ffi::c_int);
                current_block = 5411263895410993842;
            } else if change > 0 as ::core::ffi::c_int
                && tty_term_has(term, TTYC_CUB) != 0
                && (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags & TERM_DECSLRM == 0
            {
                if change == 2 as ::core::ffi::c_int && tty_term_has(term, TTYC_CUB1) != 0 {
                    tty_putcode(tty, TTYC_CUB1);
                    tty_putcode(tty, TTYC_CUB1);
                } else {
                    tty_putcode_i(tty, TTYC_CUB, change);
                }
                current_block = 5411263895410993842;
            } else if change < 0 as ::core::ffi::c_int
                && tty_term_has(term, TTYC_CUF) != 0
                && (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags & TERM_DECSLRM == 0
            {
                tty_putcode_i(tty, TTYC_CUF, -change);
                current_block = 5411263895410993842;
            } else {
                current_block = 11555347550778173209;
            }
        }
    } else if cx == thisx {
        if thisy != (*tty).rupper
            && cy == thisy.wrapping_sub(1 as u_int)
            && tty_term_has(term, TTYC_CUU1) != 0
        {
            tty_putcode(tty, TTYC_CUU1);
            current_block = 5411263895410993842;
        } else if thisy != (*tty).rlower
            && cy == thisy.wrapping_add(1 as u_int)
            && tty_term_has(term, TTYC_CUD1) != 0
        {
            tty_putcode(tty, TTYC_CUD1);
            current_block = 5411263895410993842;
        } else {
            change = thisy.wrapping_sub(cy) as ::core::ffi::c_int;
            if abs(change) as u_int > cy
                || change < 0 as ::core::ffi::c_int
                    && cy.wrapping_sub(change as u_int) > (*tty).rlower
                || change > 0 as ::core::ffi::c_int
                    && cy.wrapping_sub(change as u_int) < (*tty).rupper
            {
                if tty_term_has(term, TTYC_VPA) != 0 {
                    tty_putcode_i(tty, TTYC_VPA, cy as ::core::ffi::c_int);
                    current_block = 5411263895410993842;
                } else {
                    current_block = 11555347550778173209;
                }
            } else if change > 0 as ::core::ffi::c_int && tty_term_has(term, TTYC_CUU) != 0 {
                tty_putcode_i(tty, TTYC_CUU, change);
                current_block = 5411263895410993842;
            } else if change < 0 as ::core::ffi::c_int && tty_term_has(term, TTYC_CUD) != 0 {
                tty_putcode_i(tty, TTYC_CUD, -change);
                current_block = 5411263895410993842;
            } else {
                current_block = 11555347550778173209;
            }
        }
    } else {
        current_block = 11555347550778173209;
    }
    match current_block {
        11555347550778173209 => {
            tty_putcode_ii(
                tty,
                TTYC_CUP,
                cy as ::core::ffi::c_int,
                cx as ::core::ffi::c_int,
            );
        }
        _ => {}
    }
    (*tty).cx = cx;
    (*tty).cy = cy;
}
unsafe fn tty_hyperlink(
    tty: *mut tty,
    gc: &grid_cell,
    hl: Option<&crate::src::hyperlinks::HyperlinksRef>,
) {
    if gc.link == (*tty).cell.link {
        return;
    }
    (*tty).cell.link = gc.link;
    let Some(hl) = hl else {
        return;
    };
    let link = if gc.link == 0 {
        None
    } else {
        hyperlinks_get(hl, gc.link)
    };
    if let Some(link) = link {
        tty_putcode_ss(tty, TTYC_HLS, link.external_id.as_ptr(), link.uri.as_ptr());
    } else {
        tty_putcode_ss(tty, TTYC_HLS, c"".as_ptr(), c"".as_ptr());
    }
}
unsafe fn tty_dim_default_colour(
    mut tty: *mut tty,
    mut c: ::core::ffi::c_int,
    mut foreground: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let terminal_client_owner = (*tty).client.upgrade().expect("terminal belongs to a live client");
    let terminal_client = terminal_client_owner.get();
    let mut theme: client_theme = THEME_UNKNOWN;
    if !(c == 8 as ::core::ffi::c_int || c == 9 as ::core::ffi::c_int) {
        return c;
    }
    if foreground != 0 && (*tty).fg != -(1 as ::core::ffi::c_int) {
        return (*tty).fg;
    }
    if foreground == 0 && (*tty).bg != -(1 as ::core::ffi::c_int) {
        return (*tty).bg;
    }
    theme = (*terminal_client).theme;
    if theme as ::core::ffi::c_uint == THEME_DARK as ::core::ffi::c_int as ::core::ffi::c_uint {
        return if foreground != 0 {
            7 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        };
    }
    if theme as ::core::ffi::c_uint == THEME_LIGHT as ::core::ffi::c_int as ::core::ffi::c_uint {
        return if foreground != 0 {
            0 as ::core::ffi::c_int
        } else {
            7 as ::core::ffi::c_int
        };
    }
    return c;
}
pub unsafe fn tty_attributes(mut tty: *mut tty, gc: &grid_cell, style_ctx: Option<&tty_style_ctx>) {
    let mut gc2 = *gc;
    let mut changed: ::core::ffi::c_int = 0;
    let style_ctx = style_ctx.unwrap_or(&*std::ptr::addr_of!(tty_default_style_ctx));
    let palette_guard = style_ctx.palette.resolve();
    let palette = palette_guard.as_ref();
    if !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_NOPALETTE != 0 {
        if gc2.fg == 8 as ::core::ffi::c_int {
            gc2.fg = style_ctx.defaults.fg;
        }
        if gc2.bg == 8 as ::core::ffi::c_int {
            gc2.bg = style_ctx.defaults.bg;
        }
        if palette.is_some() {
            changed = colour_palette_get(palette, gc2.fg);
            if changed != -(1 as ::core::ffi::c_int) {
                gc2.fg = changed;
            }
            changed = colour_palette_get(palette, gc2.bg);
            if changed != -(1 as ::core::ffi::c_int) {
                gc2.bg = changed;
            }
        }
    }
    gc2.fg = tty_map_theme_colour(tty, gc2.fg);
    gc2.bg = tty_map_theme_colour(tty, gc2.bg);
    gc2.us = tty_map_theme_colour(tty, gc2.us);
    if style_ctx.dim != 0 as u_int {
        gc2.fg = tty_dim_default_colour(tty, gc2.fg, 1 as ::core::ffi::c_int);
        gc2.bg = tty_dim_default_colour(tty, gc2.bg, 0 as ::core::ffi::c_int);
        changed = colour_dim(gc2.fg, style_ctx.dim);
        if changed != -(1 as ::core::ffi::c_int) {
            gc2.fg = changed;
        }
        changed = colour_dim(gc2.bg, style_ctx.dim);
        if changed != -(1 as ::core::ffi::c_int) {
            gc2.bg = changed;
        }
    }
    if gc2.attr as ::core::ffi::c_int == (*tty).last_cell.attr as ::core::ffi::c_int
        && gc2.fg == (*tty).last_cell.fg
        && gc2.bg == (*tty).last_cell.bg
        && gc2.us == (*tty).last_cell.us
        && gc2.link == (*tty).last_cell.link
    {
        return;
    }
    if tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_SETAB) == 0 {
        if gc2.attr as ::core::ffi::c_int & GRID_ATTR_REVERSE != 0 {
            if gc2.fg != 7 as ::core::ffi::c_int
                && !(gc2.fg == 8 as ::core::ffi::c_int || gc2.fg == 9 as ::core::ffi::c_int)
            {
                gc2.attr = (gc2.attr as ::core::ffi::c_int & !GRID_ATTR_REVERSE) as u_short;
            }
        } else if gc2.bg != 0 as ::core::ffi::c_int
            && !(gc2.bg == 8 as ::core::ffi::c_int || gc2.bg == 9 as ::core::ffi::c_int)
        {
            gc2.attr = (gc2.attr as ::core::ffi::c_int | GRID_ATTR_REVERSE) as u_short;
        }
    }
    tty_check_fg(tty, palette, &mut gc2);
    tty_check_bg(tty, palette, &mut gc2);
    tty_check_us(tty, palette, &mut gc2);
    if (*tty).cell.attr as ::core::ffi::c_int & !(gc2.attr as ::core::ffi::c_int) != 0
        || (*tty).cell.us != gc2.us && gc2.us == 0 as ::core::ffi::c_int
    {
        tty_reset(tty);
    }
    tty_colours(tty, &gc2);
    changed = gc2.attr as ::core::ffi::c_int & !((*tty).cell.attr as ::core::ffi::c_int);
    (*tty).cell.attr = gc2.attr;
    if changed & GRID_ATTR_BRIGHT != 0 {
        tty_putcode(tty, TTYC_BOLD);
    }
    if changed & GRID_ATTR_DIM != 0 {
        tty_putcode(tty, TTYC_DIM);
    }
    if changed & GRID_ATTR_ITALICS != 0 {
        tty_set_italics(tty);
    }
    if changed & GRID_ATTR_ALL_UNDERSCORE != 0 {
        if changed & GRID_ATTR_UNDERSCORE != 0 {
            tty_putcode(tty, TTYC_SMUL);
        } else if changed & GRID_ATTR_UNDERSCORE_2 != 0 {
            tty_putcode_i(tty, TTYC_SMULX, 2 as ::core::ffi::c_int);
        } else if changed & GRID_ATTR_UNDERSCORE_3 != 0 {
            tty_putcode_i(tty, TTYC_SMULX, 3 as ::core::ffi::c_int);
        } else if changed & GRID_ATTR_UNDERSCORE_4 != 0 {
            tty_putcode_i(tty, TTYC_SMULX, 4 as ::core::ffi::c_int);
        } else if changed & GRID_ATTR_UNDERSCORE_5 != 0 {
            tty_putcode_i(tty, TTYC_SMULX, 5 as ::core::ffi::c_int);
        }
    }
    if changed & GRID_ATTR_BLINK != 0 {
        tty_putcode(tty, TTYC_BLINK);
    }
    if changed & GRID_ATTR_REVERSE != 0 {
        if tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_REV) != 0 {
            tty_putcode(tty, TTYC_REV);
        } else if tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_SMSO) != 0 {
            tty_putcode(tty, TTYC_SMSO);
        }
    }
    if changed & GRID_ATTR_HIDDEN != 0 {
        tty_putcode(tty, TTYC_INVIS);
    }
    if changed & GRID_ATTR_STRIKETHROUGH != 0 {
        tty_putcode(tty, TTYC_SMXX);
    }
    if changed & GRID_ATTR_OVERLINE != 0 {
        tty_putcode(tty, TTYC_SMOL);
    }
    if changed & GRID_ATTR_CHARSET != 0 && tty_acs_needed(tty.as_ref()) != 0 {
        tty_putcode(tty, TTYC_SMACS);
    }
    tty_hyperlink(tty, gc, style_ctx.hyperlinks.as_ref());
    (*tty).last_cell = gc2;
}
unsafe fn tty_colours(mut tty: *mut tty, gc: &grid_cell) {
    if gc.fg == (*tty).cell.fg && gc.bg == (*tty).cell.bg && gc.us == (*tty).cell.us {
        return;
    }
    if gc.fg == 8 as ::core::ffi::c_int
        || gc.fg == 9 as ::core::ffi::c_int
        || (gc.bg == 8 as ::core::ffi::c_int || gc.bg == 9 as ::core::ffi::c_int)
    {
        if tty_term_flag(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_AX) == 0 {
            tty_reset(tty);
        } else {
            if (gc.fg == 8 as ::core::ffi::c_int || gc.fg == 9 as ::core::ffi::c_int)
                && !((*tty).cell.fg == 8 as ::core::ffi::c_int
                    || (*tty).cell.fg == 9 as ::core::ffi::c_int)
            {
                tty_puts(tty, c"\x1B[39m");
                (*tty).cell.fg = gc.fg;
            }
            if (gc.bg == 8 as ::core::ffi::c_int || gc.bg == 9 as ::core::ffi::c_int)
                && !((*tty).cell.bg == 8 as ::core::ffi::c_int
                    || (*tty).cell.bg == 9 as ::core::ffi::c_int)
            {
                tty_puts(tty, c"\x1B[49m");
                (*tty).cell.bg = gc.bg;
            }
        }
    }
    if !(gc.fg == 8 as ::core::ffi::c_int || gc.fg == 9 as ::core::ffi::c_int)
        && gc.fg != (*tty).cell.fg
    {
        tty_colours_fg(tty, gc);
    }
    if !(gc.bg == 8 as ::core::ffi::c_int || gc.bg == 9 as ::core::ffi::c_int)
        && gc.bg != (*tty).cell.bg
    {
        tty_colours_bg(tty, gc);
    }
    if gc.us != (*tty).cell.us {
        tty_colours_us(tty, gc);
    }
}
unsafe fn tty_map_theme_colour(
    mut tty: *mut tty,
    mut colour: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let terminal_client_owner = (*tty).client.upgrade().expect("terminal belongs to a live client");
    let terminal_client = terminal_client_owner.get();
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut n: u_int = 0;
    let mut m: ::core::ffi::c_int = 0;
    if !colour & COLOUR_FLAG_THEME != 0 {
        return colour;
    }
    n = (colour & 0xff as ::core::ffi::c_int) as u_int;
    if n >= COLOUR_THEME_COUNT as u_int {
        return 8 as ::core::ffi::c_int;
    }
    if tty.is_null() || {
        c = terminal_client;
        c.is_null()
    } {
        return 8 as ::core::ffi::c_int;
    }
    m = (*c).theme_colours[n as usize];
    if m == -(1 as ::core::ffi::c_int) || m & COLOUR_FLAG_THEME != 0 {
        return 8 as ::core::ffi::c_int;
    }
    return m;
}
unsafe fn tty_check_fg(mut tty: *mut tty, palette: Option<&colour_palette>, gc: &mut grid_cell) {
    let mut r: u_char = 0;
    let mut g: u_char = 0;
    let mut b: u_char = 0;
    let mut colours: u_int = 0;
    let mut c: ::core::ffi::c_int = 0;
    if !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_NOPALETTE != 0 {
        c = gc.fg;
        if c < 8 as ::core::ffi::c_int
            && gc.attr as ::core::ffi::c_int & GRID_ATTR_BRIGHT != 0
            && tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_NOBR) == 0
        {
            c += 90 as ::core::ffi::c_int;
        }
        c = colour_palette_get(palette, c);
        if c != -(1 as ::core::ffi::c_int) {
            gc.fg = c;
        }
    }
    gc.fg = tty_map_theme_colour(tty, gc.fg);
    if gc.fg & COLOUR_FLAG_RGB != 0 {
        if (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags & TERM_RGBCOLOURS != 0 {
            return;
        }
        (r, g, b) = colour_split_rgb(gc.fg);
        gc.fg = colour_find_rgb(r, g, b);
    }
    if (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags & TERM_256COLOURS != 0 {
        colours = 256 as u_int;
    } else {
        colours = tty_term_number(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_COLORS) as u_int;
    }
    if gc.fg & COLOUR_FLAG_256 != 0 {
        if colours >= 256 as u_int {
            return;
        }
        gc.fg = colour_256to16(gc.fg);
        if !gc.fg & 8 as ::core::ffi::c_int != 0 {
            return;
        }
        gc.fg &= 7 as ::core::ffi::c_int;
        if colours >= 16 as u_int {
            gc.fg += 90 as ::core::ffi::c_int;
        } else if gc.fg == 0 as ::core::ffi::c_int && gc.bg == 0 as ::core::ffi::c_int {
            gc.fg = 7 as ::core::ffi::c_int;
        } else if gc.fg == 7 as ::core::ffi::c_int && gc.bg == 7 as ::core::ffi::c_int {
            gc.fg = 0 as ::core::ffi::c_int;
        }
        return;
    }
    if gc.fg >= 90 as ::core::ffi::c_int
        && gc.fg <= 97 as ::core::ffi::c_int
        && colours < 16 as u_int
    {
        gc.fg -= 90 as ::core::ffi::c_int;
        gc.attr = (gc.attr as ::core::ffi::c_int | GRID_ATTR_BRIGHT) as u_short;
    }
}
unsafe fn tty_check_bg(mut tty: *mut tty, palette: Option<&colour_palette>, gc: &mut grid_cell) {
    let mut r: u_char = 0;
    let mut g: u_char = 0;
    let mut b: u_char = 0;
    let mut colours: u_int = 0;
    let mut c: ::core::ffi::c_int = 0;
    if !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_NOPALETTE != 0 {
        c = colour_palette_get(palette, gc.bg);
        if c != -(1 as ::core::ffi::c_int) {
            gc.bg = c;
        }
    }
    gc.bg = tty_map_theme_colour(tty, gc.bg);
    if gc.bg & COLOUR_FLAG_RGB != 0 {
        if (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags & TERM_RGBCOLOURS != 0 {
            return;
        }
        (r, g, b) = colour_split_rgb(gc.bg);
        gc.bg = colour_find_rgb(r, g, b);
    }
    if (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags & TERM_256COLOURS != 0 {
        colours = 256 as u_int;
    } else {
        colours = tty_term_number(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_COLORS) as u_int;
    }
    if gc.bg & COLOUR_FLAG_256 != 0 {
        if colours >= 256 as u_int {
            return;
        }
        gc.bg = colour_256to16(gc.bg);
        if !gc.bg & 8 as ::core::ffi::c_int != 0 {
            return;
        }
        gc.bg &= 7 as ::core::ffi::c_int;
        if colours >= 16 as u_int {
            gc.bg += 90 as ::core::ffi::c_int;
        }
        return;
    }
    if gc.bg >= 90 as ::core::ffi::c_int
        && gc.bg <= 97 as ::core::ffi::c_int
        && colours < 16 as u_int
    {
        gc.bg -= 90 as ::core::ffi::c_int;
    }
}
unsafe fn tty_check_us(mut tty: *mut tty, palette: Option<&colour_palette>, gc: &mut grid_cell) {
    let mut c: ::core::ffi::c_int = 0;
    if !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_NOPALETTE != 0 {
        c = colour_palette_get(palette, gc.us);
        if c != -(1 as ::core::ffi::c_int) {
            gc.us = c;
        }
    }
    gc.us = tty_map_theme_colour(tty, gc.us);
    if tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_SETULC1) == 0 {
        c = colour_force_rgb(gc.us);
        if c == -(1 as ::core::ffi::c_int) {
            gc.us = 8 as ::core::ffi::c_int;
        } else {
            gc.us = c;
        }
    }
}
unsafe fn tty_colours_fg(mut tty: *mut tty, gc: &grid_cell) {
    let mut s: [::core::ffi::c_char; 32] = [0; 32];
    if (*tty).cell.fg >= 90 as ::core::ffi::c_int
        && (*tty).cell.bg <= 97 as ::core::ffi::c_int
        && (gc.fg < 90 as ::core::ffi::c_int || gc.fg > 97 as ::core::ffi::c_int)
    {
        tty_reset(tty);
    }
    if gc.fg & COLOUR_FLAG_RGB != 0 || gc.fg & COLOUR_FLAG_256 != 0 {
        if !(tty_try_colour(tty, gc.fg, true) == 0 as ::core::ffi::c_int) {
            return;
        }
    } else if gc.fg >= 90 as ::core::ffi::c_int && gc.fg <= 97 as ::core::ffi::c_int {
        if (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags & TERM_256COLOURS != 0 {
            xformat(&mut s, format_args!("\x1B[{}m", (gc.fg) as i32));
            tty_puts(tty, std::ffi::CStr::from_ptr(s.as_ptr()));
        } else {
            tty_putcode_i(
                tty,
                TTYC_SETAF,
                gc.fg - 90 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
            );
        }
    } else {
        tty_putcode_i(tty, TTYC_SETAF, gc.fg);
    }
    (*tty).cell.fg = gc.fg;
}
unsafe fn tty_colours_bg(mut tty: *mut tty, gc: &grid_cell) {
    let mut s: [::core::ffi::c_char; 32] = [0; 32];
    if gc.bg & COLOUR_FLAG_RGB != 0 || gc.bg & COLOUR_FLAG_256 != 0 {
        if !(tty_try_colour(tty, gc.bg, false) == 0 as ::core::ffi::c_int) {
            return;
        }
    } else if gc.bg >= 90 as ::core::ffi::c_int && gc.bg <= 97 as ::core::ffi::c_int {
        if (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags & TERM_256COLOURS != 0 {
            xformat(
                &mut s,
                format_args!("\x1B[{}m", (gc.bg + 10 as ::core::ffi::c_int) as i32),
            );
            tty_puts(tty, std::ffi::CStr::from_ptr(s.as_ptr()));
        } else {
            tty_putcode_i(
                tty,
                TTYC_SETAB,
                gc.bg - 90 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
            );
        }
    } else {
        tty_putcode_i(tty, TTYC_SETAB, gc.bg);
    }
    (*tty).cell.bg = gc.bg;
}
unsafe fn tty_colours_us(mut tty: *mut tty, gc: &grid_cell) {
    let mut c: u_int = 0;
    let mut r: u_char = 0;
    let mut g: u_char = 0;
    let mut b: u_char = 0;
    if gc.us == 8 as ::core::ffi::c_int || gc.us == 9 as ::core::ffi::c_int {
        tty_putcode(tty, TTYC_OL);
    } else {
        if !gc.us & COLOUR_FLAG_RGB != 0 {
            c = gc.us as u_int;
            if !c & COLOUR_FLAG_256 as u_int != 0 && (c >= 90 as u_int && c <= 97 as u_int) {
                c = c.wrapping_sub(82 as u_int);
            }
            tty_putcode_i(
                tty,
                TTYC_SETULC1,
                (c & !COLOUR_FLAG_256 as u_int) as ::core::ffi::c_int,
            );
            return;
        }
        (r, g, b) = colour_split_rgb(gc.us);
        c = (65536 as ::core::ffi::c_int * r as ::core::ffi::c_int
            + 256 as ::core::ffi::c_int * g as ::core::ffi::c_int
            + b as ::core::ffi::c_int) as u_int;
        if tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_SETULC) != 0 {
            tty_putcode_i(tty, TTYC_SETULC, c as ::core::ffi::c_int);
        } else if tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_SETAL) != 0
            && tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_RGB) != 0
        {
            tty_putcode_i(tty, TTYC_SETAL, c as ::core::ffi::c_int);
        }
    }
    (*tty).cell.us = gc.us;
}
unsafe fn tty_try_colour(
    mut tty: *mut tty,
    mut colour: ::core::ffi::c_int,
    foreground: bool,
) -> ::core::ffi::c_int {
    let mut r: u_char = 0;
    let mut g: u_char = 0;
    let mut b: u_char = 0;
    if colour & COLOUR_FLAG_256 != 0 {
        if foreground && tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_SETAF) != 0 {
            tty_putcode_i(tty, TTYC_SETAF, colour & 0xff as ::core::ffi::c_int);
        } else if tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_SETAB) != 0 {
            tty_putcode_i(tty, TTYC_SETAB, colour & 0xff as ::core::ffi::c_int);
        }
        return 0 as ::core::ffi::c_int;
    }
    if colour & COLOUR_FLAG_RGB != 0 {
        (r, g, b) = colour_split_rgb(colour & 0xffffff as ::core::ffi::c_int);
        if foreground && tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_SETRGBF) != 0 {
            tty_putcode_iii(
                tty,
                TTYC_SETRGBF,
                r as ::core::ffi::c_int,
                g as ::core::ffi::c_int,
                b as ::core::ffi::c_int,
            );
        } else if tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_SETRGBB) != 0 {
            tty_putcode_iii(
                tty,
                TTYC_SETRGBB,
                r as ::core::ffi::c_int,
                g as ::core::ffi::c_int,
                b as ::core::ffi::c_int,
            );
        }
        return 0 as ::core::ffi::c_int;
    }
    return -(1 as ::core::ffi::c_int);
}
fn tty_window_default_style(palette: &colour_palette) -> grid_cell {
    grid_cell {
        fg: palette.fg,
        bg: palette.bg,
        ..grid_default_cell
    }
}
unsafe fn tty_style_changed(mut wp: *mut window_pane) {
    let mut oo: *mut options = options_owner_ptr(&mut (*wp).options).map_or(std::ptr::null_mut(), |options| options);
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut sy: *mut style = ::core::ptr::null_mut::<style>();
    log_debug(format_args!("%{}: style changed", ((*wp).id) as u32));
    (*wp).flags &= !PANE_STYLECHANGED;
    ft = format_create(
        ::core::ptr::null_mut::<client>(),
        ::core::ptr::null_mut::<cmdq_item>(),
        (FORMAT_PANE | (*wp).id) as ::core::ffi::c_int,
        FORMAT_NOJOBS,
    );
    format_defaults(
        ft,
        ::core::ptr::null_mut::<client>(),
        ::core::ptr::null_mut::<session>(),
        ::core::ptr::null_mut::<winlink>(),
        wp,
    );
    (*wp).cached_active_gc = tty_window_default_style(&(*wp).palette);
    sy = style_add(
        &raw mut (*wp).cached_active_gc,
        oo,
        b"window-active-style\0" as *const u8 as *const ::core::ffi::c_char,
        ft,
    );
    (*wp).cached_active_dim = (*sy).dim as u_int;
    (*wp).cached_gc = tty_window_default_style(&(*wp).palette);
    sy = style_add(
        &raw mut (*wp).cached_gc,
        oo,
        b"window-style\0" as *const u8 as *const ::core::ffi::c_char,
        ft,
    );
    (*wp).cached_dim = (*sy).dim as u_int;
    format_free(Box::from_raw(ft));
}
pub unsafe fn tty_default_colours(wp: *mut window_pane) -> (grid_cell, u_int) {
    if (*wp).flags & PANE_STYLECHANGED != 0 {
        tty_style_changed(wp);
    }
    let active = wp == (*(*wp).window_ptr()).active_ptr();
    let mut gc = grid_default_cell;
    gc.fg = if active && (*wp).cached_active_gc.fg != 8 {
        (*wp).cached_active_gc.fg
    } else {
        (*wp).cached_gc.fg
    };
    gc.bg = if active && (*wp).cached_active_gc.bg != 8 {
        (*wp).cached_active_gc.bg
    } else {
        (*wp).cached_gc.bg
    };
    let dim = if active {
        (*wp).cached_active_dim
    } else {
        (*wp).cached_dim
    };
    (gc, dim)
}
pub unsafe fn tty_default_attributes(tty: *mut tty, bg: u_int, style_ctx: Option<&tty_style_ctx>) {
    let mut gc = grid_default_cell;
    gc.bg = bg as i32;
    tty_attributes(tty, &gc, style_ctx);
}
unsafe fn tty_clipboard_query_callback(owner: &std::rc::Rc<std::cell::UnsafeCell<client>>) {
    let c = owner.get();
    let tty = &raw mut (*c).tty;
    (*tty).flags &= !TTY_OSC52QUERY;
}
pub unsafe fn tty_clipboard_query(mut tty: *mut tty) {
    let mut tv: timeval = timeval {
        tv_sec: TTY_QUERY_TIMEOUT as __time_t,
        tv_usec: 0,
    };
    if (*tty).flags & TTY_STARTED != 0 && !(*tty).flags & TTY_OSC52QUERY != 0 {
        tty_putcode_ss(
            tty,
            TTYC_MS,
            b"\0" as *const u8 as *const ::core::ffi::c_char,
            b"?\0" as *const u8 as *const ::core::ffi::c_char,
        );
        (*tty).flags |= TTY_OSC52QUERY;
        event_add(&raw mut (*tty).clipboard_timer, &raw mut tv);
    }
}
pub unsafe fn tty_set_progress_bar(mut tty: *mut tty, mut pb: *mut progress_bar) {
    if tty_term_has(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term), TTYC_SPB) != 0 {
        tty_putcode_ii(
            tty,
            TTYC_SPB,
            (*pb).state as ::core::ffi::c_int,
            (*pb).progress,
        );
    }
}

#[cfg(test)]
mod clipping_tests {
    use super::*;

    #[test]
    fn clipping_matches_tmux_at_viewport_edges_and_for_empty_regions() {
        let ctx = tty_ctx {
            flags: TTY_CTX_WINDOW_BIGGER,
            xoff: 8,
            yoff: 18,
            rxoff: 8,
            ryoff: 18,
            wox: 10,
            woy: 20,
            wsx: 30,
            wsy: 10,
            ..Default::default()
        };
        // Expected geometry from the pinned tmux tty_clamp_line/area functions.
        // The source skip when both sides are clipped deliberately follows tmux.
        for (input, line, area) in [
            ((3, 2, 5, 2), Some((0, 1, 0, 5)), Some((1, 0, 5, 2))),
            ((0, 2, 5, 3), Some((2, 0, 0, 3)), Some((0, 0, 3, 3))),
            ((30, 2, 6, 5), Some((0, 28, 0, 2)), Some((28, 0, 2, 5))),
            ((0, 2, 40, 20), Some((10, 0, 0, 30)), Some((0, 0, 30, 10))),
            ((0, 2, 2, 1), None, None),
            ((32, 2, 2, 1), None, None),
            ((3, 0, 5, 2), None, None),
            ((3, 12, 5, 1), None, None),
            ((3, 3, 0, 0), Some((0, 1, 1, 0)), Some((1, 1, 0, 0))),
            ((0, 0, 5, 5), None, Some((0, 0, 3, 3))),
            ((30, 10, 6, 5), Some((0, 28, 8, 2)), Some((28, 8, 2, 2))),
            ((0, 0, 40, 20), None, Some((0, 0, 30, 10))),
        ] {
            let (px, py, nx, ny) = input;
            assert_eq!(
                tty_clamp_line(&ctx, px, py, nx).map(|r| (r.skip, r.x, r.y, r.width)),
                line,
                "line {input:?}"
            );
            assert_eq!(
                tty_clamp_area(&ctx, px, py, nx, ny).map(|r| (r.x, r.y, r.width, r.height)),
                area,
                "area {input:?}"
            );
        }
    }
}

#[cfg(test)]
mod initialization_owner_tests {
    use super::*;
    use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
    use std::rc::Rc;

    #[test]
    fn mouse_callbacks_retain_live_clients_and_skip_expired_clients() {
        unsafe fn update(owner: &Rc<std::cell::UnsafeCell<client>>, mouse: *mut mouse_event) {
            assert_eq!(Rc::strong_count(owner), 2);
            (*mouse).valid += 1;
        }
        unsafe {
            let owner = client::new();
            let observer = Rc::downgrade(&owner);
            let mut callback = tty_mouse_client_callback(&owner, update);
            assert_eq!(Rc::strong_count(&owner), 1);
            let mut mouse = mouse_event::default();
            callback(&mut mouse);
            assert_eq!(mouse.valid, 1);
            drop(owner);
            assert!(observer.upgrade().is_none());
            callback(&mut mouse);
            assert_eq!(mouse.valid, 1);
        }
    }

    #[test]
    fn terminal_event_callbacks_skip_expired_clients() {
        unsafe {
            let owner = client::new();
            let observer = Rc::downgrade(&owner);
            let callbacks: [unsafe fn(&Rc<std::cell::UnsafeCell<client>>); 5] = [
                tty_read_callback, tty_write_callback, tty_timer_callback,
                tty_start_timer_callback, tty_clipboard_query_callback,
            ];
            let mut callbacks: Vec<_> = callbacks.into_iter()
                .map(|callback| tty_client_callback(&owner, callback)).collect();
            assert_eq!(Rc::strong_count(&owner), 1);
            (*owner.get()).tty.flags |= TTY_OSC52QUERY;
            callbacks[4](-1, 0);
            assert_eq!((*owner.get()).tty.flags & TTY_OSC52QUERY, 0);
            drop(owner);
            assert!(observer.upgrade().is_none());
            for callback in &mut callbacks {
                callback(-1, 0);
            }
        }
    }

    #[test]
    fn initialization_resets_terminal_and_replaces_weak_client_without_leaking() {
        unsafe {
            let mut master = -1;
            let mut slave = -1;
            assert_eq!(libc::openpty(
                &mut master, &mut slave, std::ptr::null_mut(),
                std::ptr::null(), std::ptr::null(),
            ), 0);
            let _master = OwnedFd::from_raw_fd(master);
            let slave = OwnedFd::from_raw_fd(slave);
            let owner = client::new();
            (*owner.get()).fd = slave.as_raw_fd();
            assert_eq!(tty_init(&owner), 0);
            let weak_count = Rc::weak_count(&owner);
            (*owner.get()).tty.sx = 99;
            (*owner.get()).tty.r.ensure(4);
            assert_eq!(tty_init(&owner), 0);
            assert_eq!(Rc::weak_count(&owner), weak_count);
            assert_eq!((*owner.get()).tty.sx, 0);
            assert!(Rc::ptr_eq(&(*owner.get()).tty.client.upgrade().unwrap(), &owner));
            (*owner.get()).fd = -1;
            // A failed initialization must leave the existing terminal intact.
            (*owner.get()).tty.sx = 42;
            assert_eq!(tty_init(&owner), -1);
            assert_eq!((*owner.get()).tty.sx, 42);
            let terminal = std::mem::take(&mut (*owner.get()).tty);
            drop(owner);
            assert!(terminal.client.upgrade().is_none());
        }
    }
}
