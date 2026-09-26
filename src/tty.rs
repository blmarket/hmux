use crate::src::ffi::libc::__useconds_t;
use crate::src::ffi::libc::{
    __errno_location, abs, fcntl, getpid, ioctl, isatty, memcpy, memset, open, strcmp, strerror,
    strlen, strncmp, tcflush, tcgetattr, tcsetattr, time, usleep, write,
};
use crate::src::ffi::resolv::__b64_ntop;
use crate::src::format::bytes::format_cstring;
use crate::src::format::{format_create, format_defaults, format_free};
use crate::src::grid::{grid_cells_equal, grid_default_cell};
use crate::src::hyperlinks::hyperlinks_get;
use crate::src::log::{fatal, fatalx, log_cstr, log_cstr_n, log_debug, log_get_level};
use crate::src::options::{options_get_number, options_get_string};
use crate::src::reactor::{
    evbuffer_add, evbuffer_drain, evbuffer_free, evbuffer_get_length, evbuffer_new, evbuffer_read,
    evbuffer_write, event_add, event_del, event_initialized, event_pending, event_set,
};
use crate::src::screen::screen_mode_to_string;
use crate::src::server::clients;
use crate::src::server_client::{
    server_client_ensure_ranges, server_client_lost, server_client_ranges_is_empty,
};
use crate::src::server_fn::server_redraw_client;
use crate::src::shared::abi::ssize_t;
use crate::src::shared::abi::*;
use crate::src::shared::client::client;
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
use crate::src::shared::hyperlinks::hyperlinks;
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
use crate::src::xmalloc::xsnprintf;

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
        defaults: &raw const grid_default_cell,
        palette: ::core::ptr::null::<colour_palette>() as *mut colour_palette,
        dim: 0 as u_int,
        hyperlinks: ::core::ptr::null::<hyperlinks>() as *mut hyperlinks,
    }
};
pub unsafe fn tty_create_log() {
    let mut name: [::core::ffi::c_char; 64] = [0; 64];
    xsnprintf(
        &raw mut name as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as size_t,
        b"tmux-out-%ld.log\0" as *const u8 as *const ::core::ffi::c_char,
        getpid() as ::core::ffi::c_long,
    );
    tty_log_fd = open(
        &raw mut name as *mut ::core::ffi::c_char,
        O_WRONLY | O_CREAT | O_TRUNC,
        0o644 as ::core::ffi::c_int,
    );
    if tty_log_fd != -(1 as ::core::ffi::c_int)
        && fcntl(tty_log_fd, F_SETFD, FD_CLOEXEC) == -(1 as ::core::ffi::c_int)
    {
        fatal(b"fcntl failed\0" as *const u8 as *const ::core::ffi::c_char);
    }
}
pub unsafe fn tty_init(mut tty: *mut tty, mut c: *mut client) -> ::core::ffi::c_int {
    if isatty((*c).fd) == 0 {
        return -(1 as ::core::ffi::c_int);
    }
    // Drop the owned key tree before the translated field reset below.
    tty_keys_free(tty);
    // `r` owns a Vec now, so preserve its valid empty value while resetting
    // the translated C fields around it.
    (*tty).r.clear();
    let range_offset = ::core::mem::offset_of!(tty, r);
    let range_end = range_offset + ::core::mem::size_of::<visible_ranges>();
    let tty_size = ::core::mem::size_of::<tty>();
    memset(
        tty as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        range_offset as size_t,
    );
    memset(
        (tty as *mut u8).add(range_end) as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        tty_size.wrapping_sub(range_end) as size_t,
    );
    (*tty).client = c;
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
    let mut c: *mut client = (*tty).client;
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
            && !(*tty).out.is_null()
            && (*tty).flags & TTY_WINSIZEQUERY == 0
            && (*(*tty).term).flags & TERM_VT100LIKE != 0
        {
            tty_puts(
                tty,
                b"\x1B[18t\x1B[14t\0" as *const u8 as *const ::core::ffi::c_char,
            );
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
unsafe fn tty_read_callback(mut data: *mut ::core::ffi::c_void) {
    let mut tty: *mut tty = data as *mut tty;
    let mut c: *mut client = (*tty).client;
    let mut name: *const ::core::ffi::c_char = ((*c).name)
        .as_ref()
        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut());
    let mut size: size_t = evbuffer_get_length(&*((*tty).in_0));
    let mut nread: ::core::ffi::c_int = 0;
    nread = evbuffer_read((*tty).in_0, (*c).fd, -(1 as ::core::ffi::c_int));
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
        server_client_lost((*tty).client);
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
unsafe fn tty_timer_callback(mut data: *mut ::core::ffi::c_void) {
    let mut tty: *mut tty = data as *mut tty;
    let mut c: *mut client = (*tty).client;
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
    let mut c: *mut client = (*tty).client;
    let mut size: size_t = evbuffer_get_length(&*((*tty).out));
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
    evbuffer_drain((*tty).out, size);
    (*c).discarded = (*c).discarded.wrapping_add(size);
    (*tty).discarded = 0 as size_t;
    event_add(&raw mut (*tty).timer, &raw mut tv);
    return 1 as ::core::ffi::c_int;
}
unsafe fn tty_write_callback(mut data: *mut ::core::ffi::c_void) {
    let mut tty: *mut tty = data as *mut tty;
    let mut c: *mut client = (*tty).client;
    let mut size: size_t = evbuffer_get_length(&*((*tty).out));
    let mut nwrite: ::core::ffi::c_int = 0;
    nwrite = evbuffer_write((*tty).out, (*c).fd);
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
    if evbuffer_get_length(&*((*tty).out)) != 0 as size_t {
        event_add(&raw mut (*tty).event_out, ::core::ptr::null::<timeval>());
    }
}
pub unsafe fn tty_open(mut tty: *mut tty) -> Result<(), std::ffi::CString> {
    let mut c: *mut client = (*tty).client;
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
        Ok(term) => term,
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
        move |_, _| unsafe { tty_read_callback(tty as *mut ::core::ffi::c_void) },
    );
    (*tty).in_0 = evbuffer_new();
    if (*tty).in_0.is_null() {
        fatal(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    event_set(
        &raw mut (*tty).event_out,
        (*c).fd,
        EV_WRITE as ::core::ffi::c_short,
        move |_, _| unsafe { tty_write_callback(tty as *mut ::core::ffi::c_void) },
    );
    (*tty).out = evbuffer_new();
    if (*tty).out.is_null() {
        fatal(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    event_set(
        &raw mut (*tty).clipboard_timer,
        -(1 as ::core::ffi::c_int),
        0 as ::core::ffi::c_short,
        move |_, _| unsafe { tty_clipboard_query_callback(tty as *mut ::core::ffi::c_void) },
    );
    event_set(
        &raw mut (*tty).start_timer,
        -(1 as ::core::ffi::c_int),
        0 as ::core::ffi::c_short,
        move |_, _| unsafe { tty_start_timer_callback(tty as *mut ::core::ffi::c_void) },
    );
    event_set(
        &raw mut (*tty).timer,
        -(1 as ::core::ffi::c_int),
        0 as ::core::ffi::c_short,
        move |_, _| unsafe { tty_timer_callback(tty as *mut ::core::ffi::c_void) },
    );
    tty_start_tty(tty);
    tty_keys_build(tty);
    Ok(())
}
unsafe fn tty_start_timer_callback(mut data: *mut ::core::ffi::c_void) {
    let mut tty: *mut tty = data as *mut tty;
    let mut c: *mut client = (*tty).client;
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
    let mut c: *mut client = (*tty).client;
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
    let mut c: *mut client = (*tty).client;
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
        if tty_term_has((*tty).term, TTYC_INDN) != 0 {
            tty_putcode_i(
                tty,
                TTYC_INDN,
                (*tty).sy.wrapping_add(1 as u_int) as ::core::ffi::c_int,
            );
        } else if tty_term_has((*tty).term, TTYC_IND) != 0 {
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
    if tty_acs_needed(tty) != 0 {
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
    if tty_term_has((*tty).term, TTYC_KMOUS) != 0 {
        tty_puts(
            tty,
            b"\x1B[?1000l\x1B[?1002l\x1B[?1003l\0" as *const u8 as *const ::core::ffi::c_char,
        );
        tty_puts(
            tty,
            b"\x1B[?1006l\x1B[?1005l\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if tty_term_has((*tty).term, TTYC_ENBP) != 0 {
        tty_putcode(tty, TTYC_ENBP);
    }
    if (*(*tty).term).flags & TERM_VT100LIKE != 0 {
        tty_puts(
            tty,
            b"\x1B[?2031h\x1B[?996n\0" as *const u8 as *const ::core::ffi::c_char,
        );
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
    if (*(*tty).term).flags & TERM_VT100LIKE != 0 {
        if !(*tty).flags & TTY_HAVEDA != 0 {
            tty_puts(tty, b"\x1B[c\0" as *const u8 as *const ::core::ffi::c_char);
        }
        if !(*tty).flags & TTY_HAVEDA2 != 0 {
            tty_puts(tty, b"\x1B[>c\0" as *const u8 as *const ::core::ffi::c_char);
        }
        if !(*tty).flags & TTY_HAVEXDA != 0 {
            tty_puts(tty, b"\x1B[>q\0" as *const u8 as *const ::core::ffi::c_char);
        }
        if !(*tty).flags & TTY_HAVESYNC != 0 {
            tty_puts(
                tty,
                b"\x1B[?2026$p\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        tty_puts(
            tty,
            b"\x1B]10;?\x1B\\\x1B]11;?\x1B\\\0" as *const u8 as *const ::core::ffi::c_char,
        );
        (*tty).flags |= TTY_WAITBG | TTY_WAITFG;
    } else {
        (*tty).flags |= TTY_ALL_REQUEST_FLAGS;
    }
    (*tty).last_requests = time(::core::ptr::null_mut::<time_t>());
}
pub unsafe fn tty_repeat_requests(mut tty: *mut tty, mut force: ::core::ffi::c_int) {
    let mut c: *mut client = (*tty).client;
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
    if (*(*tty).term).flags & TERM_VT100LIKE != 0 {
        tty_puts(
            tty,
            b"\x1B]10;?\x1B\\\x1B]11;?\x1B\\\0" as *const u8 as *const ::core::ffi::c_char,
        );
        (*tty).flags |= TTY_WAITBG | TTY_WAITFG;
    }
    tty_start_start_timer(tty);
}
pub unsafe fn tty_stop_tty(mut tty: *mut tty) {
    let mut c: *mut client = (*tty).client;
    let mut ws: winsize = winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    if (*tty).flags & TTY_STARTED == 0 {
        return;
    }
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
        tty_term_string_ii(
            (*tty).term,
            TTYC_CSR,
            0 as ::core::ffi::c_int,
            ws.ws_row as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
        ),
    );
    if tty_acs_needed(tty) != 0 {
        tty_raw(tty, tty_term_string((*tty).term, TTYC_RMACS));
    }
    tty_raw(tty, tty_term_string((*tty).term, TTYC_SGR0));
    tty_raw(tty, tty_term_string((*tty).term, TTYC_RMKX));
    if options_get_number(
        global_options,
        b"clear-on-attach\0" as *const u8 as *const ::core::ffi::c_char,
    ) != 0
    {
        tty_raw(tty, tty_term_string((*tty).term, TTYC_CLEAR));
    }
    if (*tty).cstyle as ::core::ffi::c_uint
        != SCREEN_CURSOR_DEFAULT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if tty_term_has((*tty).term, TTYC_SE) != 0 {
            tty_raw(tty, tty_term_string((*tty).term, TTYC_SE));
        } else if tty_term_has((*tty).term, TTYC_SS) != 0 {
            tty_raw(
                tty,
                tty_term_string_i((*tty).term, TTYC_SS, 0 as ::core::ffi::c_int),
            );
        }
    }
    if (*tty).ccolour != -(1 as ::core::ffi::c_int) {
        tty_raw(tty, tty_term_string((*tty).term, TTYC_CR));
    }
    tty_raw(tty, tty_term_string((*tty).term, TTYC_CNORM));
    if tty_term_has((*tty).term, TTYC_KMOUS) != 0 {
        tty_raw(
            tty,
            b"\x1B[?1000l\x1B[?1002l\x1B[?1003l\0" as *const u8 as *const ::core::ffi::c_char,
        );
        tty_raw(
            tty,
            b"\x1B[?1006l\x1B[?1005l\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if tty_term_has((*tty).term, TTYC_DSBP) != 0 {
        tty_raw(tty, tty_term_string((*tty).term, TTYC_DSBP));
    }
    if (*(*tty).term).flags & TERM_VT100LIKE != 0 {
        tty_raw(
            tty,
            b"\x1B[?7727l\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    tty_raw(tty, tty_term_string((*tty).term, TTYC_DSFCS));
    tty_raw(tty, tty_term_string((*tty).term, TTYC_DSEKS));
    if (*(*tty).term).flags & TERM_DECSLRM != 0 {
        tty_raw(tty, tty_term_string((*tty).term, TTYC_DSMG));
    }
    if options_get_number(
        global_options,
        b"clear-on-attach\0" as *const u8 as *const ::core::ffi::c_char,
    ) != 0
    {
        tty_raw(tty, tty_term_string((*tty).term, TTYC_RMCUP));
    } else {
        tty_raw(tty, tty_term_string((*tty).term, TTYC_CLEAR));
    }
    if (*(*tty).term).flags & TERM_VT100LIKE != 0 {
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
        evbuffer_free((*tty).in_0);
        event_del(&raw mut (*tty).event_in);
        evbuffer_free((*tty).out);
        event_del(&raw mut (*tty).event_out);
        tty_term_free((*tty).term);
        (*tty).term = ::core::ptr::null_mut();
        tty_keys_free(tty);
        (*tty).flags &= !TTY_OPENED;
    }
}
pub unsafe fn tty_free(mut tty: *mut tty) {
    tty_close(tty);
    (*tty).r.clear();
}
pub unsafe fn tty_update_features(mut tty: *mut tty) {
    let mut c: *mut client = (*tty).client;
    if tty_apply_features((*tty).term) != 0 {
        tty_term_apply_overrides((*tty).term);
    }
    if (*(*tty).term).flags & TERM_DECSLRM != 0 {
        tty_putcode(tty, TTYC_ENMG);
    }
    if options_get_number(
        global_options,
        b"extended-keys\0" as *const u8 as *const ::core::ffi::c_char,
    ) != 0
    {
        tty_puts(tty, tty_term_string((*tty).term, TTYC_ENEKS));
    }
    if options_get_number(
        global_options,
        b"focus-events\0" as *const u8 as *const ::core::ffi::c_char,
    ) != 0
    {
        tty_puts(tty, tty_term_string((*tty).term, TTYC_ENFCS));
    }
    if (*(*tty).term).flags & TERM_VT100LIKE != 0 {
        tty_puts(
            tty,
            b"\x1B[?7727h\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    server_redraw_client(c);
    tty_invalidate(tty);
}
pub unsafe fn tty_raw(mut tty: *mut tty, mut s: *const ::core::ffi::c_char) {
    let mut c: *mut client = (*tty).client;
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
    tty_puts(tty, tty_term_string((*tty).term, code));
}
pub unsafe fn tty_putcode_i(mut tty: *mut tty, mut code: tty_code_code, mut a: ::core::ffi::c_int) {
    if a < 0 as ::core::ffi::c_int {
        return;
    }
    tty_puts(tty, tty_term_string_i((*tty).term, code, a));
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
    tty_puts(tty, tty_term_string_ii((*tty).term, code, a, b));
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
    tty_puts(tty, tty_term_string_iii((*tty).term, code, a, b, c));
}
pub unsafe fn tty_putcode_s(mut tty: *mut tty, mut a: *const ::core::ffi::c_char) {
    let mut code: tty_code_code = TTYC_CS;
    if !a.is_null() {
        tty_puts(tty, tty_term_string_s((*tty).term, code, a));
    }
}
pub unsafe fn tty_putcode_ss(
    mut tty: *mut tty,
    mut code: tty_code_code,
    mut a: *const ::core::ffi::c_char,
    mut b: *const ::core::ffi::c_char,
) {
    if !a.is_null() && !b.is_null() {
        tty_puts(tty, tty_term_string_ss((*tty).term, code, a, b));
    }
}
unsafe fn tty_add(mut tty: *mut tty, mut buf: *const ::core::ffi::c_char, mut len: size_t) {
    let mut c: *mut client = (*tty).client;
    if (*tty).flags & TTY_BLOCK != 0 {
        (*tty).discarded = (*tty).discarded.wrapping_add(len);
        return;
    }
    evbuffer_add((*tty).out, buf as *const ::core::ffi::c_void, len);
    log_debug(format_args!(
        "{}: {}",
        log_cstr(
            (((*c).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                as *const _
        ),
        log_cstr_n((buf) as *const _, len as ::core::ffi::c_int)
    ));
    (*c).written = (*c).written.wrapping_add(len);
    if tty_log_fd != -(1 as ::core::ffi::c_int) {
        write(tty_log_fd, buf as *const ::core::ffi::c_void, len);
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
pub unsafe fn tty_puts(mut tty: *mut tty, mut s: *const ::core::ffi::c_char) {
    if *s as ::core::ffi::c_int != '\0' as i32 {
        tty_add(tty, s, strlen(s));
    }
}
pub unsafe fn tty_putc(mut tty: *mut tty, mut ch: u_char) {
    let mut acs: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if (*(*tty).term).flags & TERM_NOAM != 0
        && ch as ::core::ffi::c_int >= 0x20 as ::core::ffi::c_int
        && ch as ::core::ffi::c_int != 0x7f as ::core::ffi::c_int
        && (*tty).cy == (*tty).sy.wrapping_sub(1 as u_int)
        && (*tty).cx.wrapping_add(1 as u_int) >= (*tty).sx
    {
        return;
    }
    if (*tty).cell.attr as ::core::ffi::c_int & GRID_ATTR_CHARSET != 0 {
        acs = tty_acs_get(tty, ch);
        if !acs.is_null() {
            tty_add(tty, acs, strlen(acs));
        } else {
            tty_add(tty, &raw mut ch as *const ::core::ffi::c_char, 1 as size_t);
        }
    } else {
        tty_add(tty, &raw mut ch as *const ::core::ffi::c_char, 1 as size_t);
    }
    if ch as ::core::ffi::c_int >= 0x20 as ::core::ffi::c_int
        && ch as ::core::ffi::c_int != 0x7f as ::core::ffi::c_int
    {
        if (*tty).cx >= (*tty).sx {
            (*tty).cx = 1 as u_int;
            if (*tty).cy != (*tty).rlower {
                (*tty).cy = (*tty).cy.wrapping_add(1);
            }
            if (*(*tty).term).flags & TERM_NOAM != 0 {
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
pub unsafe fn tty_putn(
    mut tty: *mut tty,
    mut buf: *const ::core::ffi::c_void,
    mut len: size_t,
    mut width: u_int,
) {
    if (*(*tty).term).flags & TERM_NOAM != 0
        && (*tty).cy == (*tty).sy.wrapping_sub(1 as u_int)
        && ((*tty).cx as size_t).wrapping_add(len) >= (*tty).sx as size_t
    {
        len = (*tty).sx.wrapping_sub((*tty).cx).wrapping_sub(1 as u_int) as size_t;
    }
    tty_add(tty, buf as *const ::core::ffi::c_char, len);
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
    if tty_term_has((*tty).term, TTYC_SITM) != 0 {
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
pub unsafe fn tty_set_title(mut tty: *mut tty, mut title: *const ::core::ffi::c_char) {
    if tty_term_has((*tty).term, TTYC_TSL) == 0 || tty_term_has((*tty).term, TTYC_FSL) == 0 {
        return;
    }
    tty_putcode(tty, TTYC_TSL);
    tty_puts(tty, title);
    tty_putcode(tty, TTYC_FSL);
}
pub unsafe fn tty_set_path(mut tty: *mut tty, mut title: *const ::core::ffi::c_char) {
    if tty_term_has((*tty).term, TTYC_SWD) == 0 || tty_term_has((*tty).term, TTYC_FSL) == 0 {
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
        colour_split_rgb(c, &raw mut r, &raw mut g, &raw mut b);
        let colour = format_cstring(format_args!("rgb:{r:02x}/{g:02x}/{b:02x}"))
            .expect("RGB colour contains no NUL");
        tty_putcode_s(tty, colour.as_ptr());
    }
    (*tty).ccolour = c;
}
unsafe fn tty_update_cursor(
    mut tty: *mut tty,
    mut mode: ::core::ffi::c_int,
    mut s: *mut screen,
) -> ::core::ffi::c_int {
    let mut cstyle: screen_cursor_style = SCREEN_CURSOR_DEFAULT;
    let mut ccolour: ::core::ffi::c_int = 0;
    let mut changed: ::core::ffi::c_int = 0;
    let mut cmode: ::core::ffi::c_int = mode;
    if !s.is_null() {
        ccolour = (*s).ccolour;
        if (*s).ccolour == -(1 as ::core::ffi::c_int) {
            ccolour = (*s).default_ccolour;
        }
        tty_force_cursor_colour(tty, ccolour);
    }
    if !cmode & MODE_CURSOR != 0 {
        if (*tty).mode & MODE_CURSOR != 0 {
            tty_putcode(tty, TTYC_CIVIS);
        }
        return cmode;
    }
    if s.is_null() {
        cstyle = (*tty).cstyle;
    } else {
        cstyle = (*s).cstyle;
        if cstyle as ::core::ffi::c_uint
            == SCREEN_CURSOR_DEFAULT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if !cmode & MODE_CURSOR_BLINKING_SET != 0 {
                if (*s).default_mode & MODE_CURSOR_BLINKING != 0 {
                    cmode |= MODE_CURSOR_BLINKING;
                } else {
                    cmode &= !MODE_CURSOR_BLINKING;
                }
            }
            cstyle = (*s).default_cstyle;
        }
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
                if tty_term_has((*tty).term, TTYC_SE) != 0 {
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
            if tty_term_has((*tty).term, TTYC_SS) != 0 {
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
            if tty_term_has((*tty).term, TTYC_SS) != 0 {
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
            if tty_term_has((*tty).term, TTYC_SS) != 0 {
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
pub unsafe fn tty_update_mode(mut tty: *mut tty, mut mode: ::core::ffi::c_int, mut s: *mut screen) {
    let mut term: *mut tty_term = (*tty).term;
    let mut c: *mut client = (*tty).client;
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
            log_cstr((screen_mode_to_string((*tty).mode)) as *const _)
        ));
        log_debug(format_args!(
            "{}: setting mode {}",
            log_cstr(
                (((*c).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                    as *const _
            ),
            log_cstr((screen_mode_to_string(mode)) as *const _)
        ));
    }
    if changed & ALL_MOUSE_MODES != 0 && tty_term_has(term, TTYC_KMOUS) != 0 {
        tty_puts(
            tty,
            b"\x1B[?1006l\x1B[?1000l\x1B[?1002l\x1B[?1003l\0" as *const u8
                as *const ::core::ffi::c_char,
        );
        if mode & ALL_MOUSE_MODES != 0 {
            tty_puts(
                tty,
                b"\x1B[?1006h\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        if mode & MODE_MOUSE_ALL != 0 {
            tty_puts(
                tty,
                b"\x1B[?1000h\x1B[?1002h\x1B[?1003h\0" as *const u8 as *const ::core::ffi::c_char,
            );
        } else if mode & MODE_MOUSE_BUTTON != 0 {
            tty_puts(
                tty,
                b"\x1B[?1000h\x1B[?1002h\0" as *const u8 as *const ::core::ffi::c_char,
            );
        } else if mode & MODE_MOUSE_STANDARD != 0 {
            tty_puts(
                tty,
                b"\x1B[?1000h\0" as *const u8 as *const ::core::ffi::c_char,
            );
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
    if tty_term_has((*tty).term, code) != 0 {
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
pub unsafe fn tty_repeat_space(mut tty: *mut tty, mut n: u_int) {
    static mut s: [::core::ffi::c_char; 500] = [0; 500];
    if *(&raw mut s as *mut ::core::ffi::c_char) as ::core::ffi::c_int != ' ' as i32 {
        memset(
            &raw mut s as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            ' ' as i32,
            ::core::mem::size_of::<[::core::ffi::c_char; 500]>() as size_t,
        );
    }
    while n as usize > ::core::mem::size_of::<[::core::ffi::c_char; 500]>() as usize {
        tty_putn(
            tty,
            &raw mut s as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
            ::core::mem::size_of::<[::core::ffi::c_char; 500]>() as size_t,
            ::core::mem::size_of::<[::core::ffi::c_char; 500]>() as u_int,
        );
        n = (n as ::core::ffi::c_ulong).wrapping_sub(::core::mem::size_of::<
            [::core::ffi::c_char; 500],
        >() as usize as ::core::ffi::c_ulong) as u_int as u_int;
    }
    if n != 0 as u_int {
        tty_putn(
            tty,
            &raw mut s as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
            n as size_t,
            n,
        );
    }
}
pub unsafe fn tty_window_bigger(mut tty: *mut tty) -> ::core::ffi::c_int {
    let mut c: *mut client = (*tty).client;
    let mut w: *mut window = (*(*(*c).session).curw).window;
    return ((*tty).sx < (*w).sx || (*tty).sy.wrapping_sub(status_line_size(c)) < (*w).sy)
        as ::core::ffi::c_int;
}
pub unsafe fn tty_window_offset(
    mut tty: *mut tty,
    mut ox: *mut u_int,
    mut oy: *mut u_int,
    mut sx: *mut u_int,
    mut sy: *mut u_int,
) -> ::core::ffi::c_int {
    *ox = (*tty).oox;
    *oy = (*tty).ooy;
    *sx = (*tty).osx;
    *sy = (*tty).osy;
    return (*tty).oflag;
}
unsafe fn tty_window_offset1(
    mut tty: *mut tty,
    mut ox: *mut u_int,
    mut oy: *mut u_int,
    mut sx: *mut u_int,
    mut sy: *mut u_int,
) -> ::core::ffi::c_int {
    let mut c: *mut client = (*tty).client;
    let mut w: *mut window = (*(*(*c).session).curw).window;
    let mut wp: *mut window_pane = (*w).active;
    let mut cx: u_int = 0;
    let mut cy: u_int = 0;
    let mut lines: u_int = 0;
    lines = status_line_size(c);
    if (*tty).sx >= (*w).sx && (*tty).sy.wrapping_sub(lines) >= (*w).sy {
        *ox = 0 as u_int;
        *oy = 0 as u_int;
        *sx = (*w).sx;
        *sy = (*w).sy;
        (*c).pan_window = NULL;
        return 0 as ::core::ffi::c_int;
    }
    *sx = (*tty).sx;
    *sy = (*tty).sy.wrapping_sub(lines);
    if (*c).pan_window == w as *mut ::core::ffi::c_void {
        if *sx >= (*w).sx {
            (*c).pan_ox = 0 as u_int;
        } else if (*c).pan_ox.wrapping_add(*sx) > (*w).sx {
            (*c).pan_ox = (*w).sx.wrapping_sub(*sx);
        }
        *ox = (*c).pan_ox;
        if *sy >= (*w).sy {
            (*c).pan_oy = 0 as u_int;
        } else if (*c).pan_oy.wrapping_add(*sy) > (*w).sy {
            (*c).pan_oy = (*w).sy.wrapping_sub(*sy);
        }
        *oy = (*c).pan_oy;
        return 1 as ::core::ffi::c_int;
    }
    if !(*(*wp).screen).mode & MODE_CURSOR != 0 {
        *ox = 0 as u_int;
        *oy = 0 as u_int;
    } else {
        cx = ((*wp).xoff as u_int).wrapping_add((*(*wp).screen).cx);
        cy = ((*wp).yoff as u_int).wrapping_add((*(*wp).screen).cy);
        if cx < *sx {
            *ox = 0 as u_int;
        } else if cx > (*w).sx.wrapping_sub(*sx) {
            *ox = (*w).sx.wrapping_sub(*sx);
        } else {
            *ox = cx.wrapping_sub((*sx).wrapping_div(2 as u_int));
        }
        if cy < *sy {
            *oy = 0 as u_int;
        } else if cy > (*w).sy.wrapping_sub(*sy) {
            *oy = (*w).sy.wrapping_sub(*sy);
        } else {
            *oy = cy.wrapping_sub(*sy).wrapping_add(1 as u_int);
        }
    }
    (*c).pan_window = NULL;
    return 1 as ::core::ffi::c_int;
}
pub unsafe fn tty_update_window_offset(mut w: *mut window) {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    c = clients.first();
    while !c.is_null() {
        if !(*c).session.is_null()
            && !(*(*c).session).curw.is_null()
            && (*(*(*c).session).curw).window == w
        {
            tty_update_client_offset(c);
        }
        c = clients.next(c);
    }
}
pub unsafe fn tty_update_client_offset(mut c: *mut client) {
    let mut ox: u_int = 0;
    let mut oy: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    if !(*c).flags & CLIENT_TERMINAL as uint64_t != 0 {
        return;
    }
    (*c).tty.oflag = tty_window_offset1(
        &raw mut (*c).tty,
        &raw mut ox,
        &raw mut oy,
        &raw mut sx,
        &raw mut sy,
    );
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
unsafe fn tty_large_region(mut ctx: *const tty_ctx) -> ::core::ffi::c_int {
    return ((*ctx).orlower.wrapping_sub((*ctx).orupper) >= (*ctx).sy.wrapping_div(2 as u_int))
        as ::core::ffi::c_int;
}
pub unsafe fn tty_fake_bce(
    mut tty: *const tty,
    mut gc: *const grid_cell,
    mut bg: u_int,
) -> ::core::ffi::c_int {
    if tty_term_flag((*tty).term, TTYC_BCE) != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if !(bg == 8 as u_int || bg == 9 as u_int)
        || !((*gc).bg == 8 as ::core::ffi::c_int || (*gc).bg == 9 as ::core::ffi::c_int)
    {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe fn tty_redraw_region(mut tty: *mut tty, mut ctx: *const tty_ctx) {
    let mut c: *mut client = (*tty).client;
    let mut i: u_int = 0;
    if tty_large_region(ctx) != 0 || (*ctx).flags & TTY_CTX_PANE_OBSCURED != 0 {
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
        (*ctx).redraw_cb.as_ref().expect("non-null redraw callback")(&*ctx);
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
        ((*ctx).orupper) as u32,
        ((*ctx).orlower) as u32
    ));
    i = (*ctx).orupper;
    while i <= (*ctx).orlower {
        tty_draw_pane(tty, ctx, i);
        i = i.wrapping_add(1);
    }
}
unsafe fn tty_is_visible(
    mut ctx: *const tty_ctx,
    mut px: u_int,
    mut py: u_int,
    mut nx: u_int,
    mut ny: u_int,
) -> ::core::ffi::c_int {
    let mut xoff: u_int = ((*ctx).rxoff as u_int).wrapping_add(px);
    let mut yoff: u_int = ((*ctx).ryoff as u_int).wrapping_add(py);
    if !(*ctx).flags & TTY_CTX_WINDOW_BIGGER != 0 {
        return 1 as ::core::ffi::c_int;
    }
    if xoff.wrapping_add(nx) <= (*ctx).wox
        || xoff >= (*ctx).wox.wrapping_add((*ctx).wsx)
        || yoff.wrapping_add(ny) <= (*ctx).woy
        || yoff >= (*ctx).woy.wrapping_add((*ctx).wsy)
    {
        return 0 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe fn tty_clamp_line(
    mut ctx: *const tty_ctx,
    mut px: u_int,
    mut py: u_int,
    mut nx: u_int,
    mut i: *mut u_int,
    mut x: *mut u_int,
    mut rx: *mut u_int,
    mut ry: *mut u_int,
) -> ::core::ffi::c_int {
    let mut xoff: ::core::ffi::c_int =
        ((*ctx).rxoff as u_int).wrapping_add(px) as ::core::ffi::c_int;
    if tty_is_visible(ctx, px, py, nx, 1 as u_int) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    *ry = ((*ctx).yoff as u_int)
        .wrapping_add(py)
        .wrapping_sub((*ctx).woy);
    if xoff >= (*ctx).wox as ::core::ffi::c_int
        && (xoff as u_int).wrapping_add(nx) <= (*ctx).wox.wrapping_add((*ctx).wsx)
    {
        *i = 0 as u_int;
        *x = ((*ctx).xoff as u_int)
            .wrapping_add(px)
            .wrapping_sub((*ctx).wox);
        *rx = nx;
    } else if xoff < (*ctx).wox as ::core::ffi::c_int
        && (xoff as u_int).wrapping_add(nx) > (*ctx).wox.wrapping_add((*ctx).wsx)
    {
        *i = (*ctx).wox;
        *x = 0 as u_int;
        *rx = (*ctx).wsx;
    } else if xoff < (*ctx).wox as ::core::ffi::c_int {
        *i = (*ctx)
            .wox
            .wrapping_sub(((*ctx).xoff as u_int).wrapping_add(px));
        *x = 0 as u_int;
        *rx = nx.wrapping_sub(*i);
    } else {
        *i = 0 as u_int;
        *x = ((*ctx).xoff as u_int)
            .wrapping_add(px)
            .wrapping_sub((*ctx).wox);
        *rx = (*ctx).wsx.wrapping_sub(*x);
    }
    if *rx > nx {
        fatalx(
            b"%s: x too big, %u > %u\0" as *const u8 as *const ::core::ffi::c_char,
            b"tty_clamp_line\0" as *const u8 as *const ::core::ffi::c_char,
            *rx,
            nx,
        );
    }
    return 1 as ::core::ffi::c_int;
}
unsafe fn tty_clear_line(
    mut tty: *mut tty,
    mut defaults: *const grid_cell,
    mut py: u_int,
    mut px: u_int,
    mut nx: u_int,
    mut bg: u_int,
) {
    let mut c: *mut client = (*tty).client;
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
    if (*c).overlay_check.is_none() && tty_fake_bce(tty, defaults, bg) == 0 {
        if px.wrapping_add(nx) >= (*tty).sx && tty_term_has((*tty).term, TTYC_EL) != 0 {
            tty_cursor(tty, px, py);
            tty_putcode(tty, TTYC_EL);
            return;
        }
        if px == 0 as u_int && tty_term_has((*tty).term, TTYC_EL1) != 0 {
            tty_cursor(tty, px.wrapping_add(nx).wrapping_sub(1 as u_int), py);
            tty_putcode(tty, TTYC_EL1);
            return;
        }
        if tty_term_has((*tty).term, TTYC_ECH) != 0 {
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
    mut ctx: *const tty_ctx,
    mut py: u_int,
    mut px: u_int,
    mut nx: u_int,
    mut bg: u_int,
) {
    let mut c: *mut client = (*tty).client;
    let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
    let mut ri: *mut visible_range = ::core::ptr::null_mut::<visible_range>();
    let mut i: u_int = 0;
    let mut l: u_int = 0;
    let mut x: u_int = 0;
    let mut rx: u_int = 0;
    let mut ry: u_int = 0;
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
    if tty_clamp_line(
        ctx,
        px,
        py,
        nx,
        &raw mut l,
        &raw mut x,
        &raw mut rx,
        &raw mut ry,
    ) != 0
    {
        r = tty_check_overlay_range(tty, x, ry, rx);
        i = 0 as u_int;
        while i < (*r).used {
            ri = &raw mut (&mut (*r).storage)[i as usize];
            if !((*ri).nx == 0 as u_int) {
                tty_clear_line(tty, &raw const (*ctx).defaults, ry, (*ri).px, (*ri).nx, bg);
            }
            i = i.wrapping_add(1);
        }
    }
}
unsafe fn tty_clamp_area(
    mut ctx: *const tty_ctx,
    mut px: u_int,
    mut py: u_int,
    mut nx: u_int,
    mut ny: u_int,
    mut i: *mut u_int,
    mut j: *mut u_int,
    mut x: *mut u_int,
    mut y: *mut u_int,
    mut rx: *mut u_int,
    mut ry: *mut u_int,
) -> ::core::ffi::c_int {
    let mut xoff: u_int = ((*ctx).rxoff as u_int).wrapping_add(px);
    let mut yoff: u_int = ((*ctx).ryoff as u_int).wrapping_add(py);
    if tty_is_visible(ctx, px, py, nx, ny) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    if xoff >= (*ctx).wox && xoff.wrapping_add(nx) <= (*ctx).wox.wrapping_add((*ctx).wsx) {
        *i = 0 as u_int;
        *x = ((*ctx).xoff as u_int)
            .wrapping_add(px)
            .wrapping_sub((*ctx).wox);
        *rx = nx;
    } else if xoff < (*ctx).wox && xoff.wrapping_add(nx) > (*ctx).wox.wrapping_add((*ctx).wsx) {
        *i = (*ctx).wox;
        *x = 0 as u_int;
        *rx = (*ctx).wsx;
    } else if xoff < (*ctx).wox {
        *i = (*ctx)
            .wox
            .wrapping_sub(((*ctx).xoff as u_int).wrapping_add(px));
        *x = 0 as u_int;
        *rx = nx.wrapping_sub(*i);
    } else {
        *i = 0 as u_int;
        *x = ((*ctx).xoff as u_int)
            .wrapping_add(px)
            .wrapping_sub((*ctx).wox);
        *rx = (*ctx).wsx.wrapping_sub(*x);
    }
    if *rx > nx {
        fatalx(
            b"%s: x too big, %u > %u\0" as *const u8 as *const ::core::ffi::c_char,
            b"tty_clamp_area\0" as *const u8 as *const ::core::ffi::c_char,
            *rx,
            nx,
        );
    }
    if yoff >= (*ctx).woy && yoff.wrapping_add(ny) <= (*ctx).woy.wrapping_add((*ctx).wsy) {
        *j = 0 as u_int;
        *y = ((*ctx).yoff as u_int)
            .wrapping_add(py)
            .wrapping_sub((*ctx).woy);
        *ry = ny;
    } else if yoff < (*ctx).woy && yoff.wrapping_add(ny) > (*ctx).woy.wrapping_add((*ctx).wsy) {
        *j = (*ctx).woy;
        *y = 0 as u_int;
        *ry = (*ctx).wsy;
    } else if yoff < (*ctx).woy {
        *j = (*ctx)
            .woy
            .wrapping_sub(((*ctx).yoff as u_int).wrapping_add(py));
        *y = 0 as u_int;
        *ry = ny.wrapping_sub(*j);
    } else {
        *j = 0 as u_int;
        *y = ((*ctx).yoff as u_int)
            .wrapping_add(py)
            .wrapping_sub((*ctx).woy);
        *ry = (*ctx).wsy.wrapping_sub(*y);
    }
    if *ry > ny {
        fatalx(
            b"%s: y too big, %u > %u\0" as *const u8 as *const ::core::ffi::c_char,
            b"tty_clamp_area\0" as *const u8 as *const ::core::ffi::c_char,
            *ry,
            ny,
        );
    }
    return 1 as ::core::ffi::c_int;
}
unsafe fn tty_clear_area(
    mut tty: *mut tty,
    mut ctx: *const tty_ctx,
    mut py: u_int,
    mut ny: u_int,
    mut px: u_int,
    mut nx: u_int,
    mut bg: u_int,
) {
    let mut c: *mut client = (*tty).client;
    let mut defaults: *const grid_cell = &raw const (*ctx).defaults;
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
    if (*c).overlay_check.is_none() && tty_fake_bce(tty, defaults, bg) == 0 {
        if px == 0 as u_int
            && px.wrapping_add(nx) >= (*tty).sx
            && py.wrapping_add(ny) >= (*tty).sy
            && tty_term_has((*tty).term, TTYC_ED) != 0
        {
            tty_cursor(tty, 0 as u_int, py);
            tty_putcode(tty, TTYC_ED);
            return;
        }
        if (*(*tty).term).flags & TERM_DECFRA != 0 && !(bg == 8 as u_int || bg == 9 as u_int) {
            xsnprintf(
                &raw mut tmp as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as size_t,
                b"\x1B[32;%u;%u;%u;%u$x\0" as *const u8 as *const ::core::ffi::c_char,
                py.wrapping_add(1 as u_int),
                px.wrapping_add(1 as u_int),
                py.wrapping_add(ny),
                px.wrapping_add(nx),
            );
            tty_puts(tty, &raw mut tmp as *mut ::core::ffi::c_char);
            return;
        }
        if px == 0 as u_int
            && px.wrapping_add(nx) >= (*tty).sx
            && ny > 2 as u_int
            && tty_term_has((*tty).term, TTYC_CSR) != 0
            && tty_term_has((*tty).term, TTYC_INDN) != 0
        {
            tty_region(tty, py, py.wrapping_add(ny).wrapping_sub(1 as u_int));
            tty_margin_off(tty);
            tty_putcode_i(tty, TTYC_INDN, ny as ::core::ffi::c_int);
            return;
        }
        if nx > 2 as u_int
            && ny > 2 as u_int
            && tty_term_has((*tty).term, TTYC_CSR) != 0
            && (*(*tty).term).flags & TERM_DECSLRM != 0
            && tty_term_has((*tty).term, TTYC_INDN) != 0
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
    mut ctx: *const tty_ctx,
    mut py: u_int,
    mut ny: u_int,
    mut px: u_int,
    mut nx: u_int,
    mut bg: u_int,
) {
    let mut i: u_int = 0;
    let mut j: u_int = 0;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut rx: u_int = 0;
    let mut ry: u_int = 0;
    if tty_clamp_area(
        ctx,
        px,
        py,
        nx,
        ny,
        &raw mut i,
        &raw mut j,
        &raw mut x,
        &raw mut y,
        &raw mut rx,
        &raw mut ry,
    ) != 0
    {
        tty_clear_area(tty, ctx, y, ry, x, rx, bg);
    }
}
unsafe fn tty_draw_pane(mut tty: *mut tty, mut ctx: *const tty_ctx, mut py: u_int) {
    let mut s: *mut screen = (*ctx).s;
    let mut nx: u_int = (*ctx).sx;
    let mut i: u_int = 0;
    let mut x: u_int = 0;
    let mut rx: u_int = 0;
    let mut ry: u_int = 0;
    let mut j: u_int = 0;
    let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
    let mut rr: *mut visible_range = ::core::ptr::null_mut::<visible_range>();
    log_debug(format_args!(
        "{}: {} {}",
        "tty_draw_pane",
        log_cstr(
            (((*(*tty).client).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                as *const _
        ),
        (py) as u32
    ));
    if !(*ctx).flags & TTY_CTX_WINDOW_BIGGER != 0 {
        r = tty_check_overlay_range(
            tty,
            (*ctx).xoff as u_int,
            ((*ctx).yoff as u_int).wrapping_add(py),
            nx,
        );
        j = 0 as u_int;
        while j < (*r).used {
            rr = &raw mut (&mut (*r).storage)[j as usize];
            if !((*rr).nx == 0 as u_int) {
                tty_draw_line(
                    tty,
                    s,
                    (*rr).px.wrapping_sub((*ctx).xoff as u_int),
                    py,
                    (*rr).nx,
                    (*rr).px,
                    ((*ctx).yoff as u_int).wrapping_add(py),
                    &raw const (*ctx).style_ctx,
                );
            }
            j = j.wrapping_add(1);
        }
        return;
    }
    if tty_clamp_line(
        ctx,
        0 as u_int,
        py,
        nx,
        &raw mut i,
        &raw mut x,
        &raw mut rx,
        &raw mut ry,
    ) != 0
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
                    &raw const (*ctx).style_ctx,
                );
            }
            j = j.wrapping_add(1);
        }
    }
}
pub unsafe fn tty_cmd_redrawline(mut tty: *mut tty, mut ctx: *const tty_ctx) {
    let mut i: u_int = 0;
    let mut x: u_int = 0;
    let mut rx: u_int = 0;
    let mut ry: u_int = 0;
    let mut j: u_int = 0;
    let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
    let mut rr: *mut visible_range = ::core::ptr::null_mut::<visible_range>();
    if tty_clamp_line(
        ctx,
        (*ctx).ocx,
        (*ctx).ocy,
        (*ctx).c2rust_unnamed.n,
        &raw mut i,
        &raw mut x,
        &raw mut rx,
        &raw mut ry,
    ) != 0
    {
        r = tty_check_overlay_range(tty, x, ry, rx);
        j = 0 as u_int;
        while j < (*r).used {
            rr = &raw mut (&mut (*r).storage)[j as usize];
            if !((*rr).nx == 0 as u_int) {
                tty_draw_line(
                    tty,
                    (*ctx).s,
                    (*ctx)
                        .ocx
                        .wrapping_add(i)
                        .wrapping_add((*rr).px)
                        .wrapping_sub(x),
                    (*ctx).ocy,
                    (*rr).nx,
                    (*rr).px,
                    ry,
                    &raw const (*ctx).style_ctx,
                );
            }
            j = j.wrapping_add(1);
        }
    }
}
pub unsafe fn tty_check_codeset(mut tty: *mut tty, mut gc: *const grid_cell) -> *const grid_cell {
    static mut new: grid_cell = grid_cell {
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
    let mut c: ::core::ffi::c_int = 0;
    if (*gc).data.size as ::core::ffi::c_int == 1 as ::core::ffi::c_int
        && (*(&raw const (*gc).data.data as *const u_char) as ::core::ffi::c_int)
            < 0x7f as ::core::ffi::c_int
    {
        return gc;
    }
    if (*gc).flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
        return gc;
    }
    if (*(*tty).client).flags & CLIENT_UTF8 as uint64_t != 0 {
        return gc;
    }
    memcpy(
        &raw mut new as *mut ::core::ffi::c_void,
        gc as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    c = tty_acs_reverse_get(
        &raw const (*gc).data.data as *const u_char as *const ::core::ffi::c_char,
        (*gc).data.size as size_t,
    );
    if c != -(1 as ::core::ffi::c_int) {
        utf8_set(&raw mut new.data, c as u_char);
        new.attr = (new.attr as ::core::ffi::c_int | GRID_ATTR_CHARSET) as u_short;
        return &raw mut new;
    }
    new.data.size = (*gc).data.width;
    if new.data.size as ::core::ffi::c_int > UTF8_SIZE {
        new.data.size = UTF8_SIZE as u_char;
    }
    memset(
        &raw mut new.data.data as *mut u_char as *mut ::core::ffi::c_void,
        '_' as i32,
        new.data.size as size_t,
    );
    return &raw mut new;
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
    let mut c: *mut client = (*tty).client;
    if (*c).overlay_check.is_none() {
        server_client_ensure_ranges(&raw mut (*tty).r, 1 as u_int);
        (&mut (*tty).r.storage)[0].px = px;
        (&mut (*tty).r.storage)[0].nx = nx;
        (*tty).r.used = 1 as u_int;
        return &raw mut (*tty).r;
    }
    let mut overlay_check = (*c)
        .overlay_check
        .take()
        .expect("non-null overlay check callback");
    let ranges = overlay_check(&mut *c, px, py, nx);
    if (*c).overlay_check.is_none() {
        (*c).overlay_check = Some(overlay_check);
    }
    (*tty).r = ranges;
    return &raw mut (*tty).r;
}
pub unsafe fn tty_sync_start(mut tty: *mut tty) {
    if (*tty).flags & TTY_BLOCK != 0 {
        return;
    }
    if (*tty).flags & TTY_SYNCING != 0 {
        return;
    }
    (*tty).flags |= TTY_SYNCING;
    if tty_term_has((*tty).term, TTYC_SYNC) != 0 {
        log_debug(format_args!(
            "{} sync start",
            log_cstr(
                (((*(*tty).client).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                    as *const _
            )
        ));
        tty_putcode_i(tty, TTYC_SYNC, 1 as ::core::ffi::c_int);
    }
}
pub unsafe fn tty_sync_end(mut tty: *mut tty) {
    if (*tty).flags & TTY_BLOCK != 0 {
        return;
    }
    if !(*tty).flags & TTY_SYNCING != 0 {
        return;
    }
    (*tty).flags &= !TTY_SYNCING;
    if tty_term_has((*tty).term, TTYC_SYNC) != 0 {
        log_debug(format_args!(
            "{} sync end",
            log_cstr(
                (((*(*tty).client).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                    as *const _
            )
        ));
        tty_putcode_i(tty, TTYC_SYNC, 2 as ::core::ffi::c_int);
    }
}
unsafe fn tty_client_ready(mut ctx: *const tty_ctx, mut c: *mut client) -> ::core::ffi::c_int {
    if (*c).session.is_null() || (*c).tty.term.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if (*c).flags & CLIENT_SUSPENDED as uint64_t != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if (*ctx).flags & TTY_CTX_INVISIBLE_PANES != 0 {
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
pub unsafe fn tty_write(
    mut cmdfn: Option<unsafe fn(*mut tty, *const tty_ctx)>,
    mut ctx: *mut tty_ctx,
) {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut state: ::core::ffi::c_int = 0;
    let Some(mut set_client_cb) = (*ctx).set_client_cb.take() else {
        return;
    };
    c = clients.first();
    while !c.is_null() {
        if tty_client_ready(ctx, c) != 0 {
            state = set_client_cb(&mut *ctx, &mut *c);
            if state == -(1 as ::core::ffi::c_int) {
                break;
            }
            if !(state == 0 as ::core::ffi::c_int) {
                cmdfn.expect("non-null function pointer")(&raw mut (*c).tty, ctx);
            }
        }
        c = clients.next(c);
    }
    if (*ctx).set_client_cb.is_none() {
        (*ctx).set_client_cb = Some(set_client_cb);
    }
}
pub unsafe fn tty_cmd_insertcharacter(mut tty: *mut tty, mut ctx: *const tty_ctx) {
    let mut c: *mut client = (*tty).client;
    if (*ctx).flags & TTY_CTX_WINDOW_BIGGER != 0
        || !((*ctx).xoff == 0 as ::core::ffi::c_int && (*ctx).sx >= (*tty).sx)
        || tty_fake_bce(tty, &raw const (*ctx).defaults, (*ctx).bg) != 0
        || tty_term_has((*tty).term, TTYC_ICH) == 0 && tty_term_has((*tty).term, TTYC_ICH1) == 0
        || (*c).overlay_check.is_some()
    {
        tty_draw_pane(tty, ctx, (*ctx).ocy);
        return;
    }
    tty_default_attributes(tty, (*ctx).bg, &raw const (*ctx).style_ctx);
    tty_cursor_pane(tty, ctx, (*ctx).ocx, (*ctx).ocy);
    tty_emulate_repeat(tty, TTYC_ICH, TTYC_ICH1, (*ctx).c2rust_unnamed.n);
}
pub unsafe fn tty_cmd_deletecharacter(mut tty: *mut tty, mut ctx: *const tty_ctx) {
    let mut c: *mut client = (*tty).client;
    if (*ctx).flags & TTY_CTX_WINDOW_BIGGER != 0
        || !((*ctx).xoff == 0 as ::core::ffi::c_int && (*ctx).sx >= (*tty).sx)
        || tty_fake_bce(tty, &raw const (*ctx).defaults, (*ctx).bg) != 0
        || tty_term_has((*tty).term, TTYC_DCH) == 0 && tty_term_has((*tty).term, TTYC_DCH1) == 0
        || (*c).overlay_check.is_some()
    {
        tty_draw_pane(tty, ctx, (*ctx).ocy);
        return;
    }
    tty_default_attributes(tty, (*ctx).bg, &raw const (*ctx).style_ctx);
    tty_cursor_pane(tty, ctx, (*ctx).ocx, (*ctx).ocy);
    tty_emulate_repeat(tty, TTYC_DCH, TTYC_DCH1, (*ctx).c2rust_unnamed.n);
}
pub unsafe fn tty_cmd_clearcharacter(mut tty: *mut tty, mut ctx: *const tty_ctx) {
    tty_default_attributes(tty, (*ctx).bg, &raw const (*ctx).style_ctx);
    tty_clear_pane_line(
        tty,
        ctx,
        (*ctx).ocy,
        (*ctx).ocx,
        (*ctx).c2rust_unnamed.n,
        (*ctx).bg,
    );
}
pub unsafe fn tty_cmd_insertline(mut tty: *mut tty, mut ctx: *const tty_ctx) {
    let mut c: *mut client = (*tty).client;
    if (*ctx).flags & TTY_CTX_WINDOW_BIGGER != 0
        || !((*ctx).xoff == 0 as ::core::ffi::c_int && (*ctx).sx >= (*tty).sx)
        || tty_fake_bce(tty, &raw const (*ctx).defaults, (*ctx).bg) != 0
        || tty_term_has((*tty).term, TTYC_CSR) == 0
        || tty_term_has((*tty).term, TTYC_IL1) == 0
        || (*ctx).sx == 1 as u_int
        || (*ctx).sy == 1 as u_int
        || (*c).overlay_check.is_some()
    {
        tty_redraw_region(tty, ctx);
        return;
    }
    tty_default_attributes(tty, (*ctx).bg, &raw const (*ctx).style_ctx);
    tty_region_pane(tty, ctx, (*ctx).orupper, (*ctx).orlower);
    tty_margin_off(tty);
    tty_cursor_pane(tty, ctx, (*ctx).ocx, (*ctx).ocy);
    tty_emulate_repeat(tty, TTYC_IL, TTYC_IL1, (*ctx).c2rust_unnamed.n);
    (*tty).cy = UINT_MAX as u_int;
    (*tty).cx = (*tty).cy;
}
pub unsafe fn tty_cmd_deleteline(mut tty: *mut tty, mut ctx: *const tty_ctx) {
    let mut c: *mut client = (*tty).client;
    if (*ctx).flags & TTY_CTX_WINDOW_BIGGER != 0
        || !((*ctx).xoff == 0 as ::core::ffi::c_int && (*ctx).sx >= (*tty).sx)
        || tty_fake_bce(tty, &raw const (*ctx).defaults, (*ctx).bg) != 0
        || tty_term_has((*tty).term, TTYC_CSR) == 0
        || tty_term_has((*tty).term, TTYC_DL1) == 0
        || (*ctx).sx == 1 as u_int
        || (*ctx).sy == 1 as u_int
        || (*c).overlay_check.is_some()
    {
        tty_redraw_region(tty, ctx);
        return;
    }
    tty_default_attributes(tty, (*ctx).bg, &raw const (*ctx).style_ctx);
    tty_region_pane(tty, ctx, (*ctx).orupper, (*ctx).orlower);
    tty_margin_off(tty);
    tty_cursor_pane(tty, ctx, (*ctx).ocx, (*ctx).ocy);
    tty_emulate_repeat(tty, TTYC_DL, TTYC_DL1, (*ctx).c2rust_unnamed.n);
    (*tty).cy = UINT_MAX as u_int;
    (*tty).cx = (*tty).cy;
}
pub unsafe fn tty_cmd_reverseindex(mut tty: *mut tty, mut ctx: *const tty_ctx) {
    let mut c: *mut client = (*tty).client;
    if (*ctx).ocy != (*ctx).orupper {
        return;
    }
    if (*ctx).flags & TTY_CTX_WINDOW_BIGGER != 0
        || !((*ctx).xoff == 0 as ::core::ffi::c_int && (*ctx).sx >= (*tty).sx)
            && (*(*tty).term).flags & TERM_DECSLRM == 0
        || tty_fake_bce(tty, &raw const (*ctx).defaults, 8 as u_int) != 0
        || tty_term_has((*tty).term, TTYC_CSR) == 0
        || tty_term_has((*tty).term, TTYC_RI) == 0 && tty_term_has((*tty).term, TTYC_RIN) == 0
        || (*ctx).sx == 1 as u_int
        || (*ctx).sy == 1 as u_int
        || (*c).overlay_check.is_some()
    {
        tty_redraw_region(tty, ctx);
        return;
    }
    tty_default_attributes(tty, (*ctx).bg, &raw const (*ctx).style_ctx);
    tty_region_pane(tty, ctx, (*ctx).orupper, (*ctx).orlower);
    tty_margin_pane(tty, ctx);
    tty_cursor_pane(tty, ctx, (*ctx).ocx, (*ctx).orupper);
    if tty_term_has((*tty).term, TTYC_RI) != 0 {
        tty_putcode(tty, TTYC_RI);
    } else {
        tty_putcode_i(tty, TTYC_RIN, 1 as ::core::ffi::c_int);
    };
}
pub unsafe fn tty_cmd_scrollup(mut tty: *mut tty, mut ctx: *const tty_ctx) {
    let mut c: *mut client = (*tty).client;
    let mut i: u_int = 0;
    if (*ctx).flags & TTY_CTX_WINDOW_BIGGER != 0
        || !((*ctx).xoff == 0 as ::core::ffi::c_int && (*ctx).sx >= (*tty).sx)
            && (*(*tty).term).flags & TERM_DECSLRM == 0
        || tty_fake_bce(tty, &raw const (*ctx).defaults, 8 as u_int) != 0
        || tty_term_has((*tty).term, TTYC_CSR) == 0
        || (*ctx).sx == 1 as u_int
        || (*ctx).sy == 1 as u_int
        || (*c).overlay_check.is_some()
    {
        tty_redraw_region(tty, ctx);
        return;
    }
    tty_default_attributes(tty, (*ctx).bg, &raw const (*ctx).style_ctx);
    tty_region_pane(tty, ctx, (*ctx).orupper, (*ctx).orlower);
    tty_margin_pane(tty, ctx);
    if (*ctx).c2rust_unnamed.n == 1 as u_int || tty_term_has((*tty).term, TTYC_INDN) == 0 {
        if (*(*tty).term).flags & TERM_DECSLRM == 0 {
            tty_cursor(tty, 0 as u_int, (*tty).rlower);
        } else {
            tty_cursor(tty, (*tty).rright, (*tty).rlower);
        }
        i = 0 as u_int;
        while i < (*ctx).c2rust_unnamed.n {
            tty_putc(tty, '\n' as i32 as u_char);
            i = i.wrapping_add(1);
        }
    } else {
        if (*tty).cy == UINT_MAX {
            tty_cursor(tty, 0 as u_int, 0 as u_int);
        } else {
            tty_cursor(tty, 0 as u_int, (*tty).cy);
        }
        tty_putcode_i(
            tty,
            TTYC_INDN,
            (*ctx).c2rust_unnamed.n as ::core::ffi::c_int,
        );
    };
}
pub unsafe fn tty_cmd_scrolldown(mut tty: *mut tty, mut ctx: *const tty_ctx) {
    let mut i: u_int = 0;
    let mut c: *mut client = (*tty).client;
    if (*ctx).flags & TTY_CTX_WINDOW_BIGGER != 0
        || !((*ctx).xoff == 0 as ::core::ffi::c_int && (*ctx).sx >= (*tty).sx)
            && (*(*tty).term).flags & TERM_DECSLRM == 0
        || tty_fake_bce(tty, &raw const (*ctx).defaults, 8 as u_int) != 0
        || tty_term_has((*tty).term, TTYC_CSR) == 0
        || tty_term_has((*tty).term, TTYC_RI) == 0 && tty_term_has((*tty).term, TTYC_RIN) == 0
        || (*ctx).sx == 1 as u_int
        || (*ctx).sy == 1 as u_int
        || (*c).overlay_check.is_some()
    {
        tty_redraw_region(tty, ctx);
        return;
    }
    tty_default_attributes(tty, (*ctx).bg, &raw const (*ctx).style_ctx);
    tty_region_pane(tty, ctx, (*ctx).orupper, (*ctx).orlower);
    tty_margin_pane(tty, ctx);
    tty_cursor_pane(tty, ctx, (*ctx).ocx, (*ctx).orupper);
    if tty_term_has((*tty).term, TTYC_RIN) != 0 {
        tty_putcode_i(tty, TTYC_RIN, (*ctx).c2rust_unnamed.n as ::core::ffi::c_int);
    } else {
        i = 0 as u_int;
        while i < (*ctx).c2rust_unnamed.n {
            tty_putcode(tty, TTYC_RI);
            i = i.wrapping_add(1);
        }
    };
}
pub unsafe fn tty_cmd_clearendofscreen(mut tty: *mut tty, mut ctx: *const tty_ctx) {
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut nx: u_int = 0;
    let mut ny: u_int = 0;
    tty_default_attributes(tty, (*ctx).bg, &raw const (*ctx).style_ctx);
    tty_region_pane(tty, ctx, 0 as u_int, (*ctx).sy.wrapping_sub(1 as u_int));
    tty_margin_off(tty);
    px = 0 as u_int;
    nx = (*ctx).sx;
    py = (*ctx).ocy.wrapping_add(1 as u_int);
    ny = (*ctx).sy.wrapping_sub((*ctx).ocy).wrapping_sub(1 as u_int);
    tty_clear_pane_area(tty, ctx, py, ny, px, nx, (*ctx).bg);
    px = (*ctx).ocx;
    nx = (*ctx).sx.wrapping_sub((*ctx).ocx);
    py = (*ctx).ocy;
    tty_clear_pane_line(tty, ctx, py, px, nx, (*ctx).bg);
}
pub unsafe fn tty_cmd_clearstartofscreen(mut tty: *mut tty, mut ctx: *const tty_ctx) {
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut nx: u_int = 0;
    let mut ny: u_int = 0;
    tty_default_attributes(tty, (*ctx).bg, &raw const (*ctx).style_ctx);
    tty_region_pane(tty, ctx, 0 as u_int, (*ctx).sy.wrapping_sub(1 as u_int));
    tty_margin_off(tty);
    px = 0 as u_int;
    nx = (*ctx).sx;
    py = 0 as u_int;
    ny = (*ctx).ocy;
    tty_clear_pane_area(tty, ctx, py, ny, px, nx, (*ctx).bg);
    px = 0 as u_int;
    nx = (*ctx).ocx.wrapping_add(1 as u_int);
    py = (*ctx).ocy;
    tty_clear_pane_line(tty, ctx, py, px, nx, (*ctx).bg);
}
pub unsafe fn tty_cmd_clearscreen(mut tty: *mut tty, mut ctx: *const tty_ctx) {
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut nx: u_int = 0;
    let mut ny: u_int = 0;
    tty_default_attributes(tty, (*ctx).bg, &raw const (*ctx).style_ctx);
    tty_region_pane(tty, ctx, 0 as u_int, (*ctx).sy.wrapping_sub(1 as u_int));
    tty_margin_off(tty);
    px = 0 as u_int;
    nx = (*ctx).sx;
    py = 0 as u_int;
    ny = (*ctx).sy;
    tty_clear_pane_area(tty, ctx, py, ny, px, nx, (*ctx).bg);
}
pub unsafe fn tty_cmd_alignmenttest(mut tty: *mut tty, mut ctx: *const tty_ctx) {
    let mut c: *mut client = (*tty).client;
    let mut i: u_int = 0;
    let mut j: u_int = 0;
    if (*ctx).flags & TTY_CTX_WINDOW_BIGGER != 0 || (*c).overlay_check.is_some() {
        (*ctx).redraw_cb.as_ref().expect("non-null redraw callback")(&*ctx);
        return;
    }
    tty_attributes(
        tty,
        &raw const grid_default_cell,
        &raw const (*ctx).style_ctx,
    );
    tty_region_pane(tty, ctx, 0 as u_int, (*ctx).sy.wrapping_sub(1 as u_int));
    tty_margin_off(tty);
    j = 0 as u_int;
    while j < (*ctx).sy {
        tty_cursor_pane(tty, ctx, 0 as u_int, j);
        i = 0 as u_int;
        while i < (*ctx).sx {
            tty_putc(tty, 'E' as i32 as u_char);
            i = i.wrapping_add(1);
        }
        j = j.wrapping_add(1);
    }
}
pub unsafe fn tty_cmd_cell(mut tty: *mut tty, mut ctx: *const tty_ctx) {
    let mut gcp: *const grid_cell = (*ctx).cell;
    let mut s: *mut screen = (*ctx).s;
    let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut i: u_int = 0;
    let mut vis: u_int = 0 as u_int;
    px = ((*ctx).xoff as u_int)
        .wrapping_add((*ctx).ocx)
        .wrapping_sub((*ctx).wox);
    py = ((*ctx).yoff as u_int)
        .wrapping_add((*ctx).ocy)
        .wrapping_sub((*ctx).woy);
    if tty_is_visible(ctx, (*ctx).ocx, (*ctx).ocy, 1 as u_int, 1 as u_int) == 0 {
        return;
    }
    if (*gcp).data.width as ::core::ffi::c_int == 1 as ::core::ffi::c_int
        && tty_check_overlay(tty, px, py) == 0
    {
        return;
    }
    if (*gcp).data.width as ::core::ffi::c_int > 1 as ::core::ffi::c_int {
        r = tty_check_overlay_range(tty, px, py, (*gcp).data.width as u_int);
        i = 0 as u_int;
        while i < (*r).used {
            vis = vis.wrapping_add((&(*r).storage)[i as usize].nx);
            i = i.wrapping_add(1);
        }
        if vis < (*gcp).data.width as u_int {
            tty_draw_line(
                tty,
                s,
                (*s).cx,
                (*s).cy,
                (*gcp).data.width as u_int,
                px,
                py,
                &raw const (*ctx).style_ctx,
            );
            return;
        }
    }
    if ((*ctx).xoff as u_int)
        .wrapping_add((*ctx).ocx)
        .wrapping_sub((*ctx).wox)
        > (*tty).sx.wrapping_sub(1 as u_int)
        && (*ctx).ocy == (*ctx).orlower
        && ((*ctx).xoff == 0 as ::core::ffi::c_int && (*ctx).sx >= (*tty).sx)
    {
        tty_region_pane(tty, ctx, (*ctx).orupper, (*ctx).orlower);
    }
    tty_margin_off(tty);
    if (*ctx).flags & TTY_CTX_CELL_INVALIDATE != 0 {
        tty_invalidate(tty);
    }
    tty_cursor_pane_unless_wrap(tty, ctx, (*ctx).ocx, (*ctx).ocy);
    tty_cell(tty, (*ctx).cell, &raw const (*ctx).style_ctx);
    if (*ctx).flags & TTY_CTX_CELL_INVALIDATE != 0 {
        tty_invalidate(tty);
    }
}
pub unsafe fn tty_cmd_cells(mut tty: *mut tty, mut ctx: *const tty_ctx) {
    let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
    let mut ri: *mut visible_range = ::core::ptr::null_mut::<visible_range>();
    let mut i: u_int = 0;
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut cx: u_int = 0;
    let mut cp: *const ::core::ffi::c_char = (*ctx).c2rust_unnamed.data.data;
    let mut n: size_t = (*ctx).c2rust_unnamed.data.size;
    if tty_is_visible(ctx, (*ctx).ocx, (*ctx).ocy, n as u_int, 1 as u_int) == 0 {
        return;
    }
    if (*ctx).flags & TTY_CTX_WINDOW_BIGGER != 0
        && (((*ctx).xoff as u_int).wrapping_add((*ctx).ocx) < (*ctx).wox
            || (((*ctx).xoff as u_int).wrapping_add((*ctx).ocx) as size_t).wrapping_add(n)
                > (*ctx).wox.wrapping_add((*ctx).wsx) as size_t)
    {
        if !(*ctx).flags & TTY_CTX_WRAPPED != 0
            || !((*ctx).xoff == 0 as ::core::ffi::c_int && (*ctx).sx >= (*tty).sx)
            || (*(*tty).term).flags & TERM_NOAM != 0
            || ((*ctx).xoff as u_int).wrapping_add((*ctx).ocx) != 0 as u_int
            || ((*ctx).yoff as u_int).wrapping_add((*ctx).ocy) != (*tty).cy.wrapping_add(1 as u_int)
            || (*tty).cx < (*tty).sx
            || (*tty).cy == (*tty).rlower
        {
            tty_draw_pane(tty, ctx, (*ctx).ocy);
        } else {
            (*ctx).redraw_cb.as_ref().expect("non-null redraw callback")(&*ctx);
        }
        return;
    }
    tty_margin_off(tty);
    tty_cursor_pane_unless_wrap(tty, ctx, (*ctx).ocx, (*ctx).ocy);
    tty_attributes(tty, (*ctx).cell, &raw const (*ctx).style_ctx);
    px = ((*ctx).xoff as u_int)
        .wrapping_add((*ctx).ocx)
        .wrapping_sub((*ctx).wox);
    py = ((*ctx).yoff as u_int)
        .wrapping_add((*ctx).ocy)
        .wrapping_sub((*ctx).woy);
    r = tty_check_overlay_range(tty, px, py, n as u_int);
    i = 0 as u_int;
    while i < (*r).used {
        ri = &raw mut (&mut (*r).storage)[i as usize];
        if (*ri).nx != 0 as u_int {
            cx = (*ri)
                .px
                .wrapping_sub((*ctx).xoff as u_int)
                .wrapping_add((*ctx).wox);
            tty_cursor_pane_unless_wrap(tty, ctx, cx, (*ctx).ocy);
            tty_putn(
                tty,
                cp.offset((*ri).px as isize).offset(-(px as isize)) as *const ::core::ffi::c_void,
                (*ri).nx as size_t,
                (*ri).nx,
            );
        }
        i = i.wrapping_add(1);
    }
}
pub unsafe fn tty_cmd_setselection(mut tty: *mut tty, mut ctx: *const tty_ctx) {
    tty_set_selection(
        tty,
        (*ctx).c2rust_unnamed.sel.clip,
        (*ctx).c2rust_unnamed.sel.data,
        (*ctx).c2rust_unnamed.sel.size,
    );
}
pub unsafe fn tty_set_selection(
    mut tty: *mut tty,
    mut clip: *const ::core::ffi::c_char,
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
) {
    let mut size: size_t = 0;
    if !(*tty).flags & TTY_STARTED != 0 {
        return;
    }
    if tty_term_has((*tty).term, TTYC_MS) == 0 {
        return;
    }
    size = (4 as size_t)
        .wrapping_mul(len.wrapping_add(2 as size_t).wrapping_div(3 as size_t))
        .wrapping_add(1 as size_t);
    let mut encoded = vec![0; size];
    __b64_ntop(
        buf as *const ::core::ffi::c_uchar,
        len,
        encoded.as_mut_ptr().cast(),
        size,
    );
    (*tty).flags |= TTY_NOBLOCK;
    tty_putcode_ss(tty, TTYC_MS, clip, encoded.as_ptr().cast());
}
pub unsafe fn tty_cmd_rawstring(mut tty: *mut tty, mut ctx: *const tty_ctx) {
    (*tty).flags |= TTY_NOBLOCK;
    tty_add(
        tty,
        (*ctx).c2rust_unnamed.data.data,
        (*ctx).c2rust_unnamed.data.size,
    );
    tty_invalidate(tty);
}
pub unsafe fn tty_cmd_syncstart(mut tty: *mut tty, mut ctx: *const tty_ctx) {
    let mut c: *mut client = (*tty).client;
    if (*ctx).flags & TTY_CTX_OVERLAY_SYNC != 0 && (*ctx).flags & TTY_CTX_SYNC != 0 {
        tty_sync_start(tty);
    } else if !(*ctx).flags & TTY_CTX_OVERLAY_SYNC != 0 {
        if (*ctx).flags & TTY_CTX_SYNC != 0 || (*c).overlay_draw.is_some() {
            tty_sync_start(tty);
        }
    }
}
pub unsafe fn tty_cell(
    mut tty: *mut tty,
    mut gc: *const grid_cell,
    mut style_ctx: *const tty_style_ctx,
) {
    let mut gcp: *const grid_cell = ::core::ptr::null::<grid_cell>();
    if (*(*tty).term).flags & TERM_NOAM != 0
        && (*tty).cy == (*tty).sy.wrapping_sub(1 as u_int)
        && (*tty).cx == (*tty).sx.wrapping_sub(1 as u_int)
    {
        return;
    }
    if (*gc).flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0 {
        return;
    }
    if tty_check_overlay(tty, (*tty).cx, (*tty).cy) == 0 {
        return;
    }
    gcp = tty_check_codeset(tty, gc);
    tty_attributes(tty, gcp, style_ctx);
    if (*gcp).data.size as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        if (*(&raw const (*gcp).data.data as *const u_char) as ::core::ffi::c_int)
            < 0x20 as ::core::ffi::c_int
            || *(&raw const (*gcp).data.data as *const u_char) as ::core::ffi::c_int
                == 0x7f as ::core::ffi::c_int
        {
            return;
        }
        tty_putc(tty, *(&raw const (*gcp).data.data as *const u_char));
        return;
    }
    tty_putn(
        tty,
        &raw const (*gcp).data.data as *const u_char as *const ::core::ffi::c_void,
        (*gcp).data.size as size_t,
        (*gcp).data.width as u_int,
    );
}
pub unsafe fn tty_reset(mut tty: *mut tty) {
    let mut gc: *mut grid_cell = &raw mut (*tty).cell;
    if grid_cells_equal(gc, &raw const grid_default_cell) == 0 {
        if (*gc).link != 0 as u_int {
            tty_putcode_ss(
                tty,
                TTYC_HLS,
                b"\0" as *const u8 as *const ::core::ffi::c_char,
                b"\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        if (*gc).attr as ::core::ffi::c_int & GRID_ATTR_CHARSET != 0 && tty_acs_needed(tty) != 0 {
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
        if (*(*tty).term).flags & TERM_DECSLRM != 0 {
            tty_putcode(tty, TTYC_ENMG);
        }
        tty_putcode(tty, TTYC_SGR0);
        (*tty).mode = ALL_MODES;
        tty_update_mode(tty, MODE_CURSOR, ::core::ptr::null_mut::<screen>());
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
unsafe fn tty_region_pane(
    mut tty: *mut tty,
    mut ctx: *const tty_ctx,
    mut rupper: u_int,
    mut rlower: u_int,
) {
    tty_region(
        tty,
        ((*ctx).yoff as u_int)
            .wrapping_add(rupper)
            .wrapping_sub((*ctx).woy),
        ((*ctx).yoff as u_int)
            .wrapping_add(rlower)
            .wrapping_sub((*ctx).woy),
    );
}
unsafe fn tty_region(mut tty: *mut tty, mut rupper: u_int, mut rlower: u_int) {
    if (*tty).rlower == rlower && (*tty).rupper == rupper {
        return;
    }
    if tty_term_has((*tty).term, TTYC_CSR) == 0 {
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
unsafe fn tty_margin_pane(mut tty: *mut tty, mut ctx: *const tty_ctx) {
    let mut l: ::core::ffi::c_int = 0;
    let mut r: ::core::ffi::c_int = 0;
    l = ((*ctx).xoff as u_int).wrapping_sub((*ctx).wox) as ::core::ffi::c_int;
    r = ((*ctx).xoff as u_int)
        .wrapping_add((*ctx).sx)
        .wrapping_sub(1 as u_int)
        .wrapping_sub((*ctx).wox) as ::core::ffi::c_int;
    if l < 0 as ::core::ffi::c_int {
        l = 0 as ::core::ffi::c_int;
    }
    if l > (*ctx).wsx as ::core::ffi::c_int {
        l = (*ctx).wsx as ::core::ffi::c_int;
    }
    if r < 0 as ::core::ffi::c_int {
        r = 0 as ::core::ffi::c_int;
    }
    if r > (*ctx).wsx as ::core::ffi::c_int {
        r = (*ctx).wsx as ::core::ffi::c_int;
    }
    tty_margin(tty, l as u_int, r as u_int);
}
unsafe fn tty_margin(mut tty: *mut tty, mut rleft: u_int, mut rright: u_int) {
    if (*(*tty).term).flags & TERM_DECSLRM == 0 {
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
    mut ctx: *const tty_ctx,
    mut cx: u_int,
    mut cy: u_int,
) {
    if !(*ctx).flags & TTY_CTX_WRAPPED != 0
        || !((*ctx).xoff == 0 as ::core::ffi::c_int && (*ctx).sx >= (*tty).sx)
        || (*(*tty).term).flags & TERM_NOAM != 0
        || ((*ctx).xoff as u_int).wrapping_add(cx) != 0 as u_int
        || ((*ctx).yoff as u_int).wrapping_add(cy) != (*tty).cy.wrapping_add(1 as u_int)
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
unsafe fn tty_cursor_pane(
    mut tty: *mut tty,
    mut ctx: *const tty_ctx,
    mut cx: u_int,
    mut cy: u_int,
) {
    tty_cursor(
        tty,
        ((*ctx).xoff as u_int)
            .wrapping_add(cx)
            .wrapping_sub((*ctx).wox),
        ((*ctx).yoff as u_int)
            .wrapping_add(cy)
            .wrapping_sub((*ctx).woy),
    );
}
pub unsafe fn tty_cursor(mut tty: *mut tty, mut cx: u_int, mut cy: u_int) {
    let mut current_block: u64;
    let mut term: *mut tty_term = (*tty).term;
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
        && ((*(*tty).term).flags & TERM_DECSLRM == 0 || (*tty).rleft == 0 as u_int)
    {
        tty_putc(tty, '\r' as i32 as u_char);
        tty_putc(tty, '\n' as i32 as u_char);
        current_block = 5411263895410993842;
    } else if cy == thisy {
        if cx == 0 as u_int
            && ((*(*tty).term).flags & TERM_DECSLRM == 0 || (*tty).rleft == 0 as u_int)
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
                && (*(*tty).term).flags & TERM_DECSLRM == 0
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
                && (*(*tty).term).flags & TERM_DECSLRM == 0
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
unsafe fn tty_hyperlink(mut tty: *mut tty, mut gc: *const grid_cell, mut hl: *mut hyperlinks) {
    let mut uri: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut id: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if (*gc).link == (*tty).cell.link {
        return;
    }
    (*tty).cell.link = (*gc).link;
    if hl.is_null() {
        return;
    }
    if (*gc).link == 0 as u_int
        || hyperlinks_get(
            hl,
            (*gc).link,
            &raw mut uri,
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            &raw mut id,
        ) == 0
    {
        tty_putcode_ss(
            tty,
            TTYC_HLS,
            b"\0" as *const u8 as *const ::core::ffi::c_char,
            b"\0" as *const u8 as *const ::core::ffi::c_char,
        );
    } else {
        tty_putcode_ss(tty, TTYC_HLS, id, uri);
    };
}
unsafe fn tty_dim_default_colour(
    mut tty: *mut tty,
    mut c: ::core::ffi::c_int,
    mut foreground: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
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
    theme = (*(*tty).client).theme;
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
pub unsafe fn tty_attributes(
    mut tty: *mut tty,
    mut gc: *const grid_cell,
    mut style_ctx: *const tty_style_ctx,
) {
    let mut tc: *mut grid_cell = &raw mut (*tty).cell;
    let mut gc2: grid_cell = grid_cell {
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
    let mut palette: *mut colour_palette = ::core::ptr::null_mut::<colour_palette>();
    let mut changed: ::core::ffi::c_int = 0;
    if style_ctx.is_null() {
        style_ctx = &raw mut tty_default_style_ctx;
    }
    palette = (*style_ctx).palette;
    memcpy(
        &raw mut gc2 as *mut ::core::ffi::c_void,
        gc as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    if !((*gc).flags as ::core::ffi::c_int) & GRID_FLAG_NOPALETTE != 0 {
        if gc2.fg == 8 as ::core::ffi::c_int {
            gc2.fg = (*(*style_ctx).defaults).fg;
        }
        if gc2.bg == 8 as ::core::ffi::c_int {
            gc2.bg = (*(*style_ctx).defaults).bg;
        }
        if !palette.is_null() {
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
    if (*style_ctx).dim != 0 as u_int {
        gc2.fg = tty_dim_default_colour(tty, gc2.fg, 1 as ::core::ffi::c_int);
        gc2.bg = tty_dim_default_colour(tty, gc2.bg, 0 as ::core::ffi::c_int);
        changed = colour_dim(gc2.fg, (*style_ctx).dim);
        if changed != -(1 as ::core::ffi::c_int) {
            gc2.fg = changed;
        }
        changed = colour_dim(gc2.bg, (*style_ctx).dim);
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
    if tty_term_has((*tty).term, TTYC_SETAB) == 0 {
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
    tty_check_fg(tty, palette, &raw mut gc2);
    tty_check_bg(tty, palette, &raw mut gc2);
    tty_check_us(tty, palette, &raw mut gc2);
    if (*tc).attr as ::core::ffi::c_int & !(gc2.attr as ::core::ffi::c_int) != 0
        || (*tc).us != gc2.us && gc2.us == 0 as ::core::ffi::c_int
    {
        tty_reset(tty);
    }
    tty_colours(tty, &raw mut gc2);
    changed = gc2.attr as ::core::ffi::c_int & !((*tc).attr as ::core::ffi::c_int);
    (*tc).attr = gc2.attr;
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
        if tty_term_has((*tty).term, TTYC_REV) != 0 {
            tty_putcode(tty, TTYC_REV);
        } else if tty_term_has((*tty).term, TTYC_SMSO) != 0 {
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
    if changed & GRID_ATTR_CHARSET != 0 && tty_acs_needed(tty) != 0 {
        tty_putcode(tty, TTYC_SMACS);
    }
    tty_hyperlink(tty, gc, (*style_ctx).hyperlinks);
    memcpy(
        &raw mut (*tty).last_cell as *mut ::core::ffi::c_void,
        &raw mut gc2 as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
}
unsafe fn tty_colours(mut tty: *mut tty, mut gc: *const grid_cell) {
    let mut tc: *mut grid_cell = &raw mut (*tty).cell;
    if (*gc).fg == (*tc).fg && (*gc).bg == (*tc).bg && (*gc).us == (*tc).us {
        return;
    }
    if (*gc).fg == 8 as ::core::ffi::c_int
        || (*gc).fg == 9 as ::core::ffi::c_int
        || ((*gc).bg == 8 as ::core::ffi::c_int || (*gc).bg == 9 as ::core::ffi::c_int)
    {
        if tty_term_flag((*tty).term, TTYC_AX) == 0 {
            tty_reset(tty);
        } else {
            if ((*gc).fg == 8 as ::core::ffi::c_int || (*gc).fg == 9 as ::core::ffi::c_int)
                && !((*tc).fg == 8 as ::core::ffi::c_int || (*tc).fg == 9 as ::core::ffi::c_int)
            {
                tty_puts(
                    tty,
                    b"\x1B[39m\0" as *const u8 as *const ::core::ffi::c_char,
                );
                (*tc).fg = (*gc).fg;
            }
            if ((*gc).bg == 8 as ::core::ffi::c_int || (*gc).bg == 9 as ::core::ffi::c_int)
                && !((*tc).bg == 8 as ::core::ffi::c_int || (*tc).bg == 9 as ::core::ffi::c_int)
            {
                tty_puts(
                    tty,
                    b"\x1B[49m\0" as *const u8 as *const ::core::ffi::c_char,
                );
                (*tc).bg = (*gc).bg;
            }
        }
    }
    if !((*gc).fg == 8 as ::core::ffi::c_int || (*gc).fg == 9 as ::core::ffi::c_int)
        && (*gc).fg != (*tc).fg
    {
        tty_colours_fg(tty, gc);
    }
    if !((*gc).bg == 8 as ::core::ffi::c_int || (*gc).bg == 9 as ::core::ffi::c_int)
        && (*gc).bg != (*tc).bg
    {
        tty_colours_bg(tty, gc);
    }
    if (*gc).us != (*tc).us {
        tty_colours_us(tty, gc);
    }
}
unsafe fn tty_map_theme_colour(
    mut tty: *mut tty,
    mut colour: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
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
        c = (*tty).client;
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
unsafe fn tty_check_fg(
    mut tty: *mut tty,
    mut palette: *mut colour_palette,
    mut gc: *mut grid_cell,
) {
    let mut r: u_char = 0;
    let mut g: u_char = 0;
    let mut b: u_char = 0;
    let mut colours: u_int = 0;
    let mut c: ::core::ffi::c_int = 0;
    if !((*gc).flags as ::core::ffi::c_int) & GRID_FLAG_NOPALETTE != 0 {
        c = (*gc).fg;
        if c < 8 as ::core::ffi::c_int
            && (*gc).attr as ::core::ffi::c_int & GRID_ATTR_BRIGHT != 0
            && tty_term_has((*tty).term, TTYC_NOBR) == 0
        {
            c += 90 as ::core::ffi::c_int;
        }
        c = colour_palette_get(palette, c);
        if c != -(1 as ::core::ffi::c_int) {
            (*gc).fg = c;
        }
    }
    (*gc).fg = tty_map_theme_colour(tty, (*gc).fg);
    if (*gc).fg & COLOUR_FLAG_RGB != 0 {
        if (*(*tty).term).flags & TERM_RGBCOLOURS != 0 {
            return;
        }
        colour_split_rgb((*gc).fg, &raw mut r, &raw mut g, &raw mut b);
        (*gc).fg = colour_find_rgb(r, g, b);
    }
    if (*(*tty).term).flags & TERM_256COLOURS != 0 {
        colours = 256 as u_int;
    } else {
        colours = tty_term_number((*tty).term, TTYC_COLORS) as u_int;
    }
    if (*gc).fg & COLOUR_FLAG_256 != 0 {
        if colours >= 256 as u_int {
            return;
        }
        (*gc).fg = colour_256to16((*gc).fg);
        if !(*gc).fg & 8 as ::core::ffi::c_int != 0 {
            return;
        }
        (*gc).fg &= 7 as ::core::ffi::c_int;
        if colours >= 16 as u_int {
            (*gc).fg += 90 as ::core::ffi::c_int;
        } else if (*gc).fg == 0 as ::core::ffi::c_int && (*gc).bg == 0 as ::core::ffi::c_int {
            (*gc).fg = 7 as ::core::ffi::c_int;
        } else if (*gc).fg == 7 as ::core::ffi::c_int && (*gc).bg == 7 as ::core::ffi::c_int {
            (*gc).fg = 0 as ::core::ffi::c_int;
        }
        return;
    }
    if (*gc).fg >= 90 as ::core::ffi::c_int
        && (*gc).fg <= 97 as ::core::ffi::c_int
        && colours < 16 as u_int
    {
        (*gc).fg -= 90 as ::core::ffi::c_int;
        (*gc).attr = ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_BRIGHT) as u_short;
    }
}
unsafe fn tty_check_bg(
    mut tty: *mut tty,
    mut palette: *mut colour_palette,
    mut gc: *mut grid_cell,
) {
    let mut r: u_char = 0;
    let mut g: u_char = 0;
    let mut b: u_char = 0;
    let mut colours: u_int = 0;
    let mut c: ::core::ffi::c_int = 0;
    if !((*gc).flags as ::core::ffi::c_int) & GRID_FLAG_NOPALETTE != 0 {
        c = colour_palette_get(palette, (*gc).bg);
        if c != -(1 as ::core::ffi::c_int) {
            (*gc).bg = c;
        }
    }
    (*gc).bg = tty_map_theme_colour(tty, (*gc).bg);
    if (*gc).bg & COLOUR_FLAG_RGB != 0 {
        if (*(*tty).term).flags & TERM_RGBCOLOURS != 0 {
            return;
        }
        colour_split_rgb((*gc).bg, &raw mut r, &raw mut g, &raw mut b);
        (*gc).bg = colour_find_rgb(r, g, b);
    }
    if (*(*tty).term).flags & TERM_256COLOURS != 0 {
        colours = 256 as u_int;
    } else {
        colours = tty_term_number((*tty).term, TTYC_COLORS) as u_int;
    }
    if (*gc).bg & COLOUR_FLAG_256 != 0 {
        if colours >= 256 as u_int {
            return;
        }
        (*gc).bg = colour_256to16((*gc).bg);
        if !(*gc).bg & 8 as ::core::ffi::c_int != 0 {
            return;
        }
        (*gc).bg &= 7 as ::core::ffi::c_int;
        if colours >= 16 as u_int {
            (*gc).bg += 90 as ::core::ffi::c_int;
        }
        return;
    }
    if (*gc).bg >= 90 as ::core::ffi::c_int
        && (*gc).bg <= 97 as ::core::ffi::c_int
        && colours < 16 as u_int
    {
        (*gc).bg -= 90 as ::core::ffi::c_int;
    }
}
unsafe fn tty_check_us(
    mut tty: *mut tty,
    mut palette: *mut colour_palette,
    mut gc: *mut grid_cell,
) {
    let mut c: ::core::ffi::c_int = 0;
    if !((*gc).flags as ::core::ffi::c_int) & GRID_FLAG_NOPALETTE != 0 {
        c = colour_palette_get(palette, (*gc).us);
        if c != -(1 as ::core::ffi::c_int) {
            (*gc).us = c;
        }
    }
    (*gc).us = tty_map_theme_colour(tty, (*gc).us);
    if tty_term_has((*tty).term, TTYC_SETULC1) == 0 {
        c = colour_force_rgb((*gc).us);
        if c == -(1 as ::core::ffi::c_int) {
            (*gc).us = 8 as ::core::ffi::c_int;
        } else {
            (*gc).us = c;
        }
    }
}
unsafe fn tty_colours_fg(mut tty: *mut tty, mut gc: *const grid_cell) {
    let mut tc: *mut grid_cell = &raw mut (*tty).cell;
    let mut s: [::core::ffi::c_char; 32] = [0; 32];
    if (*tty).cell.fg >= 90 as ::core::ffi::c_int
        && (*tty).cell.bg <= 97 as ::core::ffi::c_int
        && ((*gc).fg < 90 as ::core::ffi::c_int || (*gc).fg > 97 as ::core::ffi::c_int)
    {
        tty_reset(tty);
    }
    if (*gc).fg & COLOUR_FLAG_RGB != 0 || (*gc).fg & COLOUR_FLAG_256 != 0 {
        if !(tty_try_colour(
            tty,
            (*gc).fg,
            b"38\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int)
        {
            return;
        }
    } else if (*gc).fg >= 90 as ::core::ffi::c_int && (*gc).fg <= 97 as ::core::ffi::c_int {
        if (*(*tty).term).flags & TERM_256COLOURS != 0 {
            xsnprintf(
                &raw mut s as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
                b"\x1B[%dm\0" as *const u8 as *const ::core::ffi::c_char,
                (*gc).fg,
            );
            tty_puts(tty, &raw mut s as *mut ::core::ffi::c_char);
        } else {
            tty_putcode_i(
                tty,
                TTYC_SETAF,
                (*gc).fg - 90 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
            );
        }
    } else {
        tty_putcode_i(tty, TTYC_SETAF, (*gc).fg);
    }
    (*tc).fg = (*gc).fg;
}
unsafe fn tty_colours_bg(mut tty: *mut tty, mut gc: *const grid_cell) {
    let mut tc: *mut grid_cell = &raw mut (*tty).cell;
    let mut s: [::core::ffi::c_char; 32] = [0; 32];
    if (*gc).bg & COLOUR_FLAG_RGB != 0 || (*gc).bg & COLOUR_FLAG_256 != 0 {
        if !(tty_try_colour(
            tty,
            (*gc).bg,
            b"48\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int)
        {
            return;
        }
    } else if (*gc).bg >= 90 as ::core::ffi::c_int && (*gc).bg <= 97 as ::core::ffi::c_int {
        if (*(*tty).term).flags & TERM_256COLOURS != 0 {
            xsnprintf(
                &raw mut s as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
                b"\x1B[%dm\0" as *const u8 as *const ::core::ffi::c_char,
                (*gc).bg + 10 as ::core::ffi::c_int,
            );
            tty_puts(tty, &raw mut s as *mut ::core::ffi::c_char);
        } else {
            tty_putcode_i(
                tty,
                TTYC_SETAB,
                (*gc).bg - 90 as ::core::ffi::c_int + 8 as ::core::ffi::c_int,
            );
        }
    } else {
        tty_putcode_i(tty, TTYC_SETAB, (*gc).bg);
    }
    (*tc).bg = (*gc).bg;
}
unsafe fn tty_colours_us(mut tty: *mut tty, mut gc: *const grid_cell) {
    let mut tc: *mut grid_cell = &raw mut (*tty).cell;
    let mut c: u_int = 0;
    let mut r: u_char = 0;
    let mut g: u_char = 0;
    let mut b: u_char = 0;
    if (*gc).us == 8 as ::core::ffi::c_int || (*gc).us == 9 as ::core::ffi::c_int {
        tty_putcode(tty, TTYC_OL);
    } else {
        if !(*gc).us & COLOUR_FLAG_RGB != 0 {
            c = (*gc).us as u_int;
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
        colour_split_rgb((*gc).us, &raw mut r, &raw mut g, &raw mut b);
        c = (65536 as ::core::ffi::c_int * r as ::core::ffi::c_int
            + 256 as ::core::ffi::c_int * g as ::core::ffi::c_int
            + b as ::core::ffi::c_int) as u_int;
        if tty_term_has((*tty).term, TTYC_SETULC) != 0 {
            tty_putcode_i(tty, TTYC_SETULC, c as ::core::ffi::c_int);
        } else if tty_term_has((*tty).term, TTYC_SETAL) != 0
            && tty_term_has((*tty).term, TTYC_RGB) != 0
        {
            tty_putcode_i(tty, TTYC_SETAL, c as ::core::ffi::c_int);
        }
    }
    (*tc).us = (*gc).us;
}
unsafe fn tty_try_colour(
    mut tty: *mut tty,
    mut colour: ::core::ffi::c_int,
    mut type_0: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut r: u_char = 0;
    let mut g: u_char = 0;
    let mut b: u_char = 0;
    if colour & COLOUR_FLAG_256 != 0 {
        if *type_0 as ::core::ffi::c_int == '3' as i32 && tty_term_has((*tty).term, TTYC_SETAF) != 0
        {
            tty_putcode_i(tty, TTYC_SETAF, colour & 0xff as ::core::ffi::c_int);
        } else if tty_term_has((*tty).term, TTYC_SETAB) != 0 {
            tty_putcode_i(tty, TTYC_SETAB, colour & 0xff as ::core::ffi::c_int);
        }
        return 0 as ::core::ffi::c_int;
    }
    if colour & COLOUR_FLAG_RGB != 0 {
        colour_split_rgb(
            colour & 0xffffff as ::core::ffi::c_int,
            &raw mut r,
            &raw mut g,
            &raw mut b,
        );
        if *type_0 as ::core::ffi::c_int == '3' as i32
            && tty_term_has((*tty).term, TTYC_SETRGBF) != 0
        {
            tty_putcode_iii(
                tty,
                TTYC_SETRGBF,
                r as ::core::ffi::c_int,
                g as ::core::ffi::c_int,
                b as ::core::ffi::c_int,
            );
        } else if tty_term_has((*tty).term, TTYC_SETRGBB) != 0 {
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
unsafe fn tty_window_default_style(mut gc: *mut grid_cell, mut wp: *mut window_pane) {
    memcpy(
        gc as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    (*gc).fg = (*wp).palette.fg;
    (*gc).bg = (*wp).palette.bg;
}
unsafe fn tty_style_changed(mut wp: *mut window_pane) {
    let mut oo: *mut options = (*wp).options;
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
    tty_window_default_style(&raw mut (*wp).cached_active_gc, wp);
    sy = style_add(
        &raw mut (*wp).cached_active_gc,
        oo,
        b"window-active-style\0" as *const u8 as *const ::core::ffi::c_char,
        ft,
    );
    (*wp).cached_active_dim = (*sy).dim as u_int;
    tty_window_default_style(&raw mut (*wp).cached_gc, wp);
    sy = style_add(
        &raw mut (*wp).cached_gc,
        oo,
        b"window-style\0" as *const u8 as *const ::core::ffi::c_char,
        ft,
    );
    (*wp).cached_dim = (*sy).dim as u_int;
    format_free(ft);
}
pub unsafe fn tty_default_colours(
    mut gc: *mut grid_cell,
    mut wp: *mut window_pane,
    mut dim: *mut u_int,
) {
    if (*wp).flags & PANE_STYLECHANGED != 0 {
        tty_style_changed(wp);
    }
    memcpy(
        gc as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    if wp == (*(*wp).window).active && (*wp).cached_active_gc.fg != 8 as ::core::ffi::c_int {
        (*gc).fg = (*wp).cached_active_gc.fg;
    } else {
        (*gc).fg = (*wp).cached_gc.fg;
    }
    if wp == (*(*wp).window).active && (*wp).cached_active_gc.bg != 8 as ::core::ffi::c_int {
        (*gc).bg = (*wp).cached_active_gc.bg;
    } else {
        (*gc).bg = (*wp).cached_gc.bg;
    }
    if !dim.is_null() {
        if wp == (*(*wp).window).active {
            *dim = (*wp).cached_active_dim;
        } else {
            *dim = (*wp).cached_dim;
        }
    }
}
pub unsafe fn tty_default_attributes(
    mut tty: *mut tty,
    mut bg: u_int,
    mut style_ctx: *const tty_style_ctx,
) {
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
    memcpy(
        &raw mut gc as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    gc.bg = bg as ::core::ffi::c_int;
    tty_attributes(tty, &raw mut gc, style_ctx);
}
unsafe fn tty_clipboard_query_callback(mut data: *mut ::core::ffi::c_void) {
    let mut tty: *mut tty = data as *mut tty;
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
    if tty_term_has((*tty).term, TTYC_SPB) != 0 {
        tty_putcode_ii(
            tty,
            TTYC_SPB,
            (*pb).state as ::core::ffi::c_int,
            (*pb).progress,
        );
    }
}
