use crate::WindowPane;
use crate::compat::error_message;

use super::draw::tty_draw_line;
use super::keys::{tty_keys_build, tty_keys_free};
use crate::ffi::{
    __b64_ntop, __errno_location, abs, fcntl, getpid, ioctl, isatty, open, tcflush, tcgetattr,
    tcsetattr, time, usleep, write,
};
use crate::fmt_args;
use crate::format::{format_create, format_defaults};
use crate::grid::grid_cells_equal;
use crate::grid::grid_default_cell;
use crate::grid::{Hyperlinks, RustHyperlinks};
use crate::log::{fatal, fatalx, log_debug, log_get_level};

use crate::pane_geometry::PaneGeometryState;
use crate::pane_style_cache::PaneStyleCells;
use crate::reactor::{Interest, IoWatch, Timer, WatchMode};
use crate::screen::screen_mode_to_string;
use crate::screen::{Screen, ScreenModeState};
use crate::server::client_get_pan_window;
use crate::server::client_ref_of;
use crate::server::client_walk;

pub use crate::consts::{
    ALL_MODES, ALL_MOUSE_MODES, CLIENT_ALLREDRAWFLAGS, CLIENT_REDRAWSTATUS, CLIENT_REDRAWWINDOW,
    CLIENT_SUSPENDED, CLIENT_TERMINAL, CLIENT_UTF8, COLOUR_FLAG_256, COLOUR_FLAG_RGB, EAGAIN,
    FORMAT_NOJOBS, FORMAT_PANE, GRID_ATTR_ALL_UNDERSCORE, GRID_ATTR_BLINK, GRID_ATTR_BRIGHT,
    GRID_ATTR_CHARSET, GRID_ATTR_DIM, GRID_ATTR_HIDDEN, GRID_ATTR_ITALICS, GRID_ATTR_OVERLINE,
    GRID_ATTR_REVERSE, GRID_ATTR_STRIKETHROUGH, GRID_ATTR_UNDERSCORE, GRID_ATTR_UNDERSCORE_2,
    GRID_ATTR_UNDERSCORE_3, GRID_ATTR_UNDERSCORE_4, GRID_ATTR_UNDERSCORE_5, GRID_FLAG_NOPALETTE,
    GRID_FLAG_PADDING, GRID_FLAG_TAB, ICRNL, MODE_CURSOR, MODE_CURSOR_BLINKING,
    MODE_CURSOR_BLINKING_SET, MODE_CURSOR_VERY_VISIBLE, MODE_MOUSE_ALL, MODE_MOUSE_BUTTON,
    MODE_MOUSE_STANDARD, O_CREAT, O_TRUNC, O_WRONLY, ONLCR, OPOST, PANE_STYLECHANGED,
    SCREEN_CURSOR_BAR, SCREEN_CURSOR_BLOCK, SCREEN_CURSOR_DEFAULT, SCREEN_CURSOR_UNDERLINE,
    TCSANOW, TERM_256COLOURS, TERM_DECFRA, TERM_DECSLRM, TERM_NOAM, TERM_RGBCOLOURS,
    TERM_VT100LIKE, TTY_ALL_REQUEST_FLAGS, TTY_BLOCK, TTY_CTX_CELL_INVALIDATE,
    TTY_CTX_INVISIBLE_PANES, TTY_CTX_OVERLAY_SYNC, TTY_CTX_PANE_OBSCURED, TTY_CTX_SYNC,
    TTY_CTX_WINDOW_BIGGER, TTY_CTX_WRAPPED, TTY_FREEZE, TTY_HAVEDA, TTY_HAVEDA2, TTY_HAVEXDA,
    TTY_NOCURSOR, TTY_OPENED, TTY_OSC52QUERY, TTY_STARTED, TTY_TIMER, TTY_WAITBG, TTY_WAITFG,
    TTY_WINSIZEQUERY, TTYC_AX, TTYC_BCE, TTYC_CLEAR, TTYC_CLMG, TTYC_CMG, TTYC_COLORS, TTYC_CUP,
    TTYC_ECH, TTYC_EL, TTYC_EL1, TTYC_ENBP, TTYC_MS, TTYC_RGB, TTYC_SETRGBB, TTYC_SETRGBF,
    TTYC_SMCUP, UINT_MAX, UTF8_SIZE, VMIN, VTIME,
};
use crate::status::status_line_size;
use crate::style::style_add;
use crate::style::{ColourEngine, RustColourEngine};
use crate::terminfo::{
    AlternateCharacterSet, RustAlternateCharacterSet, tty_acs_get, tty_acs_needed,
};
use crate::terminfo::{
    TerminalCapabilities, tty_term_apply_overrides, tty_term_create, tty_term_free, tty_term_of,
    tty_term_opt_mut,
};
use crate::text::utf8_set;
use crate::tmux::global_options;
use crate::tmux::setblocking;
pub use crate::types::*;
use crate::xmalloc::xasprintf;
use ::core::ffi::CStr;

pub const TTYC_VPA: tty_code_code = 231;
pub const TTYC_TSL: tty_code_code = 229;
pub const TTYC_SYNC: tty_code_code = 227;
pub const TTYC_SWD: tty_code_code = 226;
pub const TTYC_SS: tty_code_code = 225;
pub const TTYC_SPB: tty_code_code = 223;
pub const TTYC_SMXX: tty_code_code = 222;
pub const TTYC_SMULX: tty_code_code = 221;
pub const TTYC_SMUL: tty_code_code = 220;
pub const TTYC_SMSO: tty_code_code = 219;
pub const TTYC_SMOL: tty_code_code = 218;
pub const TTYC_SMKX: tty_code_code = 217;

pub const TTYC_SMACS: tty_code_code = 215;
pub const TTYC_SITM: tty_code_code = 214;
pub const TTYC_SGR0: tty_code_code = 213;
pub const TTYC_SETULC1: tty_code_code = 212;
pub const TTYC_SETULC: tty_code_code = 211;

pub const TTYC_SETAL: tty_code_code = 208;
pub const TTYC_SETAF: tty_code_code = 207;
pub const TTYC_SETAB: tty_code_code = 206;
pub const TTYC_SE: tty_code_code = 205;
pub const TTYC_RMKX: tty_code_code = 204;
pub const TTYC_RMCUP: tty_code_code = 203;
pub const TTYC_RMACS: tty_code_code = 202;
pub const TTYC_RIN: tty_code_code = 201;
pub const TTYC_RI: tty_code_code = 200;

pub const TTYC_REV: tty_code_code = 198;
pub const TTYC_OL: tty_code_code = 195;
pub const TTYC_NOBR: tty_code_code = 194;

pub const TTYC_KMOUS: tty_code_code = 165;
pub const TTYC_INVIS: tty_code_code = 55;
pub const TTYC_INDN: tty_code_code = 54;
pub const TTYC_IL1: tty_code_code = 53;
pub const TTYC_IL: tty_code_code = 52;
pub const TTYC_ICH1: tty_code_code = 51;
pub const TTYC_ICH: tty_code_code = 50;
pub const TTYC_HPA: tty_code_code = 49;
pub const TTYC_HOME: tty_code_code = 48;
pub const TTYC_HLS: tty_code_code = 47;
pub const TTYC_FSL: tty_code_code = 46;
pub const TTYC_ENMG: tty_code_code = 45;
pub const TTYC_ENFCS: tty_code_code = 44;
pub const TTYC_ENEKS: tty_code_code = 43;

pub const TTYC_ENACS: tty_code_code = 41;

pub const TTYC_ED: tty_code_code = 38;

pub const TTYC_DSMG: tty_code_code = 35;
pub const TTYC_DSFCS: tty_code_code = 34;
pub const TTYC_DSEKS: tty_code_code = 33;
pub const TTYC_DSBP: tty_code_code = 32;
pub const TTYC_DL1: tty_code_code = 31;
pub const TTYC_DL: tty_code_code = 30;
pub const TTYC_DIM: tty_code_code = 29;
pub const TTYC_DCH1: tty_code_code = 28;
pub const TTYC_DCH: tty_code_code = 27;
pub const TTYC_CVVIS: tty_code_code = 26;
pub const TTYC_CUU1: tty_code_code = 25;
pub const TTYC_CUU: tty_code_code = 24;

pub const TTYC_CUF1: tty_code_code = 22;
pub const TTYC_CUF: tty_code_code = 21;
pub const TTYC_CUD1: tty_code_code = 20;
pub const TTYC_CUD: tty_code_code = 19;
pub const TTYC_CUB1: tty_code_code = 18;
pub const TTYC_CUB: tty_code_code = 17;
pub const TTYC_CSR: tty_code_code = 16;
pub const TTYC_CS: tty_code_code = 15;
pub const TTYC_CR: tty_code_code = 14;

pub const TTYC_CNORM: tty_code_code = 12;

pub const TTYC_CIVIS: tty_code_code = 8;
pub const TTYC_BOLD: tty_code_code = 7;
pub const TTYC_BLINK: tty_code_code = 6;

pub const TIOCGWINSZ: core::ffi::c_int = 0x5413 as core::ffi::c_int;

pub const F_SETFD: core::ffi::c_int = 2 as core::ffi::c_int;
pub const FD_CLOEXEC: core::ffi::c_int = 1 as core::ffi::c_int;

pub const IGNBRK: core::ffi::c_int = 0o1 as core::ffi::c_int;
pub const ISTRIP: core::ffi::c_int = 0o40 as core::ffi::c_int;
pub const INLCR: core::ffi::c_int = 0o100 as core::ffi::c_int;
pub const IGNCR: core::ffi::c_int = 0o200 as core::ffi::c_int;

pub const IXON: core::ffi::c_int = 0o2000 as core::ffi::c_int;
pub const IXOFF: core::ffi::c_int = 0o10000 as core::ffi::c_int;
pub const IMAXBEL: core::ffi::c_int = 0o20000 as core::ffi::c_int;

pub const OCRNL: core::ffi::c_int = 0o10 as core::ffi::c_int;
pub const ONLRET: core::ffi::c_int = 0o40 as core::ffi::c_int;
pub const ISIG: core::ffi::c_int = 0o1 as core::ffi::c_int;
pub const ICANON: core::ffi::c_int = 0o2 as core::ffi::c_int;
pub const ECHO: core::ffi::c_int = 0o10 as core::ffi::c_int;
pub const ECHOE: core::ffi::c_int = 0o20 as core::ffi::c_int;
pub const ECHONL: core::ffi::c_int = 0o100 as core::ffi::c_int;
pub const ECHOCTL: core::ffi::c_int = 0o1000 as core::ffi::c_int;
pub const ECHOPRT: core::ffi::c_int = 0o2000 as core::ffi::c_int;
pub const ECHOKE: core::ffi::c_int = 0o4000 as core::ffi::c_int;
pub const IEXTEN: core::ffi::c_int = 0o100000 as core::ffi::c_int;

pub const TCOFLUSH: core::ffi::c_int = 1 as core::ffi::c_int;

pub const CURSOR_MODES: core::ffi::c_int =
    MODE_CURSOR | MODE_CURSOR_BLINKING | MODE_CURSOR_VERY_VISIBLE;

pub const TTY_NOBLOCK: core::ffi::c_int = 0x8 as core::ffi::c_int;

pub const TTY_SYNCING: core::ffi::c_int = 0x400 as core::ffi::c_int;

const tty_log_fd: crate::server_state::Value<core::ffi::c_int> =
    crate::server_state::Value::new(|state| &state.tty_log_fd);
pub const TTY_BLOCK_INTERVAL: core::ffi::c_int = 100000 as core::ffi::c_int;
pub const TTY_QUERY_TIMEOUT: core::ffi::c_int = 5 as core::ffi::c_int;
pub const TTY_REQUEST_LIMIT: core::ffi::c_int = 30 as core::ffi::c_int;
pub fn tty_create_log() {
    unsafe {
        let name = xasprintf(
            c"tmux-out-%ld.log",
            fmt_args![getpid() as core::ffi::c_long],
        );
        tty_log_fd.set(open(
            name.as_ptr(),
            O_WRONLY | O_CREAT | O_TRUNC,
            0o644 as core::ffi::c_int,
        ));
        if tty_log_fd.get() != -(1 as core::ffi::c_int)
            && fcntl(tty_log_fd.get(), F_SETFD, FD_CLOEXEC) == -(1 as core::ffi::c_int)
        {
            fatal(c"fcntl failed", fmt_args![]);
        }
    }
}
/// The client whose terminal this is, held for as long as the caller keeps it.
pub fn tty_client(tty: &tty) -> Option<ClientRef> {
    tty.client.as_ref().and_then(ClientWeak::upgrade)
}

pub unsafe fn tty_init(c: &mut client) -> core::ffi::c_int {
    unsafe {
        if isatty(c.fd) == 0 {
            return -(1 as core::ffi::c_int);
        }
        let owner = client_ref_of(c).map(|c| c.downgrade());
        let fd = c.fd;
        let tty = &mut c.tty;
        *tty = tty::default();
        tty.client = owner;
        tty.cstyle = SCREEN_CURSOR_DEFAULT;
        tty.ccolour = -(1 as core::ffi::c_int);
        tty.bg = -(1 as core::ffi::c_int);
        tty.fg = tty.bg;
        tty.mouse_last_pane = -(1 as core::ffi::c_int);
        if tcgetattr(fd, &raw mut tty.tio) != 0 as core::ffi::c_int {
            return -(1 as core::ffi::c_int);
        }
        0 as core::ffi::c_int
    }
}
pub unsafe fn tty_resize(tty: &mut tty) {
    unsafe {
        let c = tty_client(tty);
        let mut ws = winsize::default();
        let mut sx: u_int;
        let mut sy: u_int;
        let xpixel: u_int;
        let ypixel: u_int;
        if ioctl(
            c.as_ref().expect("the tty has a client").fd(),
            TIOCGWINSZ as core::ffi::c_ulong,
            &raw mut ws,
        ) != -(1 as core::ffi::c_int)
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
                && tty.out.is_some()
                && tty.flags & TTY_WINSIZEQUERY == 0
                && tty_term_of(tty).flags() & TERM_VT100LIKE != 0
            {
                tty_puts(tty, c"\x1B[18t\x1B[14t");
                tty.flags |= TTY_WINSIZEQUERY;
            }
        } else {
            sx = 80 as u_int;
            sy = 24 as u_int;
            xpixel = 0 as u_int;
            ypixel = 0 as u_int;
        }
        log_debug(
            c"%s: %s now %ux%u (%ux%u)",
            fmt_args![
                c"tty_resize".as_ptr(),
                c.as_ref().expect("the tty has a client").name(),
                sx,
                sy,
                xpixel,
                ypixel
            ],
        );
        tty_set_size(tty, sx, sy, xpixel, ypixel);
        tty_invalidate(tty);
    }
}
pub fn tty_set_size(tty: &mut tty, sx: u_int, sy: u_int, xpixel: u_int, ypixel: u_int) {
    {
        tty.sx = sx;
        tty.sy = sy;
        tty.xpixel = xpixel;
        tty.ypixel = ypixel;
    }
}

unsafe fn tty_timer_callback(tty: &mut tty) {
    unsafe {
        let mut c = tty_client(tty);
        let tv = timeval::from_usecs(TTY_BLOCK_INTERVAL as __suseconds_t);
        log_debug(
            c"%s: %zu discarded",
            fmt_args![
                c.as_ref().expect("the tty has a client").name(),
                tty.discarded
            ],
        );
        *c.as_mut().expect("the tty has a client").flags_mut() |= CLIENT_ALLREDRAWFLAGS as uint64_t;
        let discarded = c.as_mut().expect("the tty has a client").discarded_mut();
        *discarded = discarded.wrapping_add(tty.discarded);
        if tty.discarded
            < (1 as u_int).wrapping_add(tty.sx.wrapping_mul(tty.sy).wrapping_div(8 as u_int))
                as size_t
        {
            tty.flags &= !TTY_BLOCK;
            tty_invalidate(tty);
            return;
        }
        tty.discarded = 0 as size_t;
        tty.timer.arm(tv);
    }
}
unsafe fn tty_block_maybe(tty: &mut tty) -> core::ffi::c_int {
    unsafe {
        let mut c = tty_client(tty);
        let size = tty.out.as_ref().unwrap().len();
        let tv = timeval::from_usecs(TTY_BLOCK_INTERVAL as __suseconds_t);
        if size == 0 as size_t {
            tty.flags &= !TTY_NOBLOCK;
        } else if tty.flags & TTY_NOBLOCK != 0 {
            return 0 as core::ffi::c_int;
        }
        if size
            < (1 as u_int).wrapping_add(tty.sx.wrapping_mul(tty.sy).wrapping_mul(8 as u_int))
                as size_t
        {
            return 0 as core::ffi::c_int;
        }
        if tty.flags & TTY_BLOCK != 0 {
            return 1 as core::ffi::c_int;
        }
        tty.flags |= TTY_BLOCK;
        log_debug(
            c"%s: can't keep up, %zu discarded",
            fmt_args![c.as_ref().expect("the tty has a client").name(), size],
        );
        tty.out.as_mut().unwrap().drain(size);
        let discarded = c.as_mut().expect("the tty has a client").discarded_mut();
        *discarded = discarded.wrapping_add(size);
        tty.discarded = 0 as size_t;
        tty.timer.arm(tv);
        1 as core::ffi::c_int
    }
}
unsafe fn tty_write_callback(tty: &mut tty) {
    unsafe {
        let mut c = tty_client(tty);
        let size = tty.out.as_ref().unwrap().len();
        let nwrite = match tty
            .out
            .as_mut()
            .unwrap()
            .write_to_fd(c.as_ref().expect("the tty has a client").fd())
        {
            Ok(nwrite) => nwrite as core::ffi::c_int,
            Err(_) => -(1 as core::ffi::c_int),
        };
        if nwrite < 0 {
            return;
        }
        log_debug(
            c"%s: wrote %d bytes (of %zu)",
            fmt_args![
                c.as_ref().expect("the tty has a client").name(),
                nwrite,
                size
            ],
        );
        let redraw = c.as_mut().expect("the tty has a client").redraw_mut();
        if *redraw > 0 as size_t {
            if nwrite as size_t >= *redraw {
                *redraw = 0 as size_t;
            } else {
                *redraw = redraw.wrapping_sub(nwrite as size_t);
            }
            let remaining = *redraw;
            log_debug(
                c"%s: waiting for redraw, %zu bytes left",
                fmt_args![c.as_ref().expect("the tty has a client").name(), remaining],
            );
        } else if tty_block_maybe(tty) != 0 {
            return;
        }
        if tty.out.as_ref().unwrap().len() != 0 as size_t {
            tty.event_out.enable();
        }
    }
}
pub unsafe fn tty_open(tty: &mut tty, cause: &mut Option<std::ffi::CString>) -> core::ffi::c_int {
    unsafe {
        let mut c = tty_client(tty);
        let result = {
            let (name, capabilities, features) = c
                .as_mut()
                .expect("the tty has a client")
                .terminal_definition();
            tty_term_create(tty, name.unwrap_or(c""), capabilities, features)
        };
        match result {
            Ok(term) => tty.term = Some(term),
            Err(err) => {
                *cause = Some(err);
                tty_close(tty);
                return -(1 as core::ffi::c_int);
            }
        }
        tty.flags |= TTY_OPENED;
        tty.flags &= !(TTY_NOCURSOR | TTY_FREEZE | TTY_BLOCK | TTY_TIMER);
        let owner = tty.client.clone();
        tty.event_in.set_callback(
            c.as_ref().expect("the tty has a client").fd(),
            Interest::Read,
            WatchMode::Persistent,
            move |_, _| {
                if let Some(mut c) = owner.as_ref().and_then(ClientWeak::upgrade) {
                    c.on_tty_read();
                }
            },
        );
        tty.r#in = Some(Box::new(ByteBuffer::new()));
        let owner = tty.client.clone();
        tty.event_out.set_callback(
            c.as_ref().expect("the tty has a client").fd(),
            Interest::Write,
            WatchMode::Once,
            move |_, _| {
                if let Some(mut c) = owner.as_ref().and_then(ClientWeak::upgrade) {
                    tty_write_callback(c.as_tty_mut());
                }
            },
        );
        tty.out = Some(Box::new(ByteBuffer::new()));
        let owner = tty.client.clone();
        tty.clipboard_timer.set_callback(move || {
            if let Some(mut c) = owner.as_ref().and_then(ClientWeak::upgrade) {
                c.as_tty_mut().flags &= !TTY_OSC52QUERY;
            }
        });
        let owner = tty.client.clone();
        tty.start_timer.set_callback(move || {
            if let Some(mut c) = owner.as_ref().and_then(ClientWeak::upgrade) {
                tty_start_timer_callback(c.as_tty_mut());
            }
        });
        let owner = tty.client.clone();
        tty.timer.set_callback(move || {
            if let Some(mut c) = owner.as_ref().and_then(ClientWeak::upgrade) {
                tty_timer_callback(c.as_tty_mut());
            }
        });
        tty_start_tty(tty);
        tty_keys_build(tty);
        0 as core::ffi::c_int
    }
}
unsafe fn tty_start_timer_callback(tty: &mut tty) {
    unsafe {
        let c = tty_client(tty);
        log_debug(
            c"%s: start timer fired",
            fmt_args![c.as_ref().expect("the tty has a client").name()],
        );
        if tty.flags & (TTY_HAVEDA | TTY_HAVEDA2 | TTY_HAVEXDA) == 0 as core::ffi::c_int {
            tty_update_features(tty);
        }
        tty.flags |= TTY_ALL_REQUEST_FLAGS;
        tty.flags &= !(TTY_WAITBG | TTY_WAITFG);
    }
}
unsafe fn tty_start_start_timer(tty: &mut tty) {
    unsafe {
        let c = tty_client(tty);
        let tv = timeval::from_secs(TTY_QUERY_TIMEOUT as __time_t);
        log_debug(
            c"%s: start timer started",
            fmt_args![c.as_ref().expect("the tty has a client").name()],
        );
        tty.start_timer.disarm();
        tty.start_timer.arm(tv);
    }
}
pub unsafe fn tty_start_tty(tty: &mut tty) {
    unsafe {
        let c = tty_client(tty);
        let mut tio: termios;
        setblocking(
            c.as_ref().expect("the tty has a client").fd(),
            0 as core::ffi::c_int,
        );
        tty.event_in.enable();
        tio = tty.tio;
        tio.c_iflag &= !(IXON | IXOFF | ICRNL | INLCR | IGNCR | IMAXBEL | ISTRIP) as tcflag_t;
        tio.c_iflag |= IGNBRK as tcflag_t;
        tio.c_oflag &= !(OPOST | ONLCR | OCRNL | ONLRET) as tcflag_t;
        tio.c_lflag &=
            !(IEXTEN | ICANON | ECHO | ECHOE | ECHONL | ECHOCTL | ECHOPRT | ECHOKE | ISIG)
                as tcflag_t;
        tio.c_cc[VMIN as usize] = 1 as cc_t;
        tio.c_cc[VTIME as usize] = 0 as cc_t;
        if tcsetattr(
            c.as_ref().expect("the tty has a client").fd(),
            TCSANOW,
            &raw mut tio,
        ) == 0 as core::ffi::c_int
        {
            tcflush(c.as_ref().expect("the tty has a client").fd(), TCOFLUSH);
        }
        tty_putcode(tty, TTYC_SMCUP);
        tty_putcode(tty, TTYC_SMKX);
        tty_putcode(tty, TTYC_CLEAR);
        if tty_acs_needed(Some(tty)) != 0 {
            log_debug(
                c"%s: using capabilities for ACS",
                fmt_args![c.as_ref().expect("the tty has a client").name()],
            );
            tty_putcode(tty, TTYC_ENACS);
        } else {
            log_debug(
                c"%s: using UTF-8 for ACS",
                fmt_args![c.as_ref().expect("the tty has a client").name()],
            );
        }
        tty_putcode(tty, TTYC_CNORM);
        if tty_term_of(tty).has(TTYC_KMOUS) {
            tty_puts(tty, c"\x1B[?1000l\x1B[?1002l\x1B[?1003l");
            tty_puts(tty, c"\x1B[?1006l\x1B[?1005l");
        }
        if tty_term_of(tty).has(TTYC_ENBP) {
            tty_putcode(tty, TTYC_ENBP);
        }
        if tty_term_of(tty).flags() & TERM_VT100LIKE != 0 {
            tty_puts(tty, c"\x1B[?2031h\x1B[?996n");
        }
        tty_start_start_timer(tty);
        tty.flags |= TTY_STARTED;
        tty_invalidate(tty);
        if tty.ccolour != -(1 as core::ffi::c_int) {
            tty_force_cursor_colour(tty, -(1 as core::ffi::c_int));
        }
        tty.mouse_drag_flag = 0 as core::ffi::c_int;
        tty.mouse_drag_update = None;
        tty.mouse_drag_release = None;
    }
}
pub unsafe fn tty_send_requests(tty: &mut tty) {
    unsafe {
        if !tty.flags & TTY_STARTED != 0 {
            return;
        }
        if tty_term_of(tty).flags() & TERM_VT100LIKE != 0 {
            if !tty.flags & TTY_HAVEDA != 0 {
                tty_puts(tty, c"\x1B[c");
            }
            if !tty.flags & TTY_HAVEDA2 != 0 {
                tty_puts(tty, c"\x1B[>c");
            }
            if !tty.flags & TTY_HAVEXDA != 0 {
                tty_puts(tty, c"\x1B[>q");
            }
            tty_puts(tty, c"\x1B]10;?\x1B\\\x1B]11;?\x1B\\");
            tty.flags |= TTY_WAITBG | TTY_WAITFG;
        } else {
            tty.flags |= TTY_ALL_REQUEST_FLAGS;
        }
        tty.last_requests = time(core::ptr::null_mut::<time_t>());
    }
}
pub unsafe fn tty_repeat_requests(tty: &mut tty, force: core::ffi::c_int) {
    unsafe {
        let c = tty_client(tty);
        let t: time_t = time(core::ptr::null_mut::<time_t>());
        let n: u_int = (t - tty.last_requests) as u_int;
        if !tty.flags & TTY_STARTED != 0 {
            return;
        }
        if force == 0 && n <= TTY_REQUEST_LIMIT as u_int {
            log_debug(
                c"%s: not repeating requests (%u seconds)",
                fmt_args![c.as_ref().expect("the tty has a client").name(), n],
            );
            return;
        }
        log_debug(
            c"%s: %srepeating requests (%u seconds)",
            fmt_args![
                c.as_ref().expect("the tty has a client").name(),
                if force != 0 {
                    c"(force) ".as_ptr()
                } else {
                    c"".as_ptr()
                },
                n
            ],
        );
        tty.last_requests = t;
        if tty_term_of(tty).flags() & TERM_VT100LIKE != 0 {
            tty_puts(tty, c"\x1B]10;?\x1B\\\x1B]11;?\x1B\\");
            tty.flags |= TTY_WAITBG | TTY_WAITFG;
        }
        tty_start_start_timer(tty);
    }
}
pub unsafe fn tty_stop_tty(tty: &mut tty) {
    unsafe {
        let c = tty_client(tty);
        let mut ws = winsize::default();
        if tty.flags & TTY_STARTED == 0 {
            return;
        }
        tty.flags &= !TTY_STARTED;
        tty.start_timer.disarm();
        tty.clipboard_timer.disarm();
        tty.timer.disarm();
        tty.flags &= !TTY_BLOCK;
        tty.event_in.disable();
        tty.event_out.disable();
        if ioctl(
            c.as_ref().expect("the tty has a client").fd(),
            TIOCGWINSZ as core::ffi::c_ulong,
            &raw mut ws,
        ) == -(1 as core::ffi::c_int)
        {
            return;
        }
        if tcsetattr(
            c.as_ref().expect("the tty has a client").fd(),
            TCSANOW,
            &raw mut tty.tio,
        ) == -(1 as core::ffi::c_int)
        {
            return;
        }
        let csr = tty_term_of(tty).expand_ii(TTYC_CSR, 0, ws.ws_row as core::ffi::c_int - 1);
        tty_raw(tty, &csr);
        if tty_acs_needed(Some(tty)) != 0 {
            tty_raw(tty, &tty_term_string_for(tty, TTYC_RMACS));
        }
        tty_raw(tty, &tty_term_string_for(tty, TTYC_SGR0));
        tty_raw(tty, &tty_term_string_for(tty, TTYC_RMKX));
        tty_raw(tty, &tty_term_string_for(tty, TTYC_CLEAR));
        if tty.cstyle as core::ffi::c_uint
            != SCREEN_CURSOR_DEFAULT as core::ffi::c_int as core::ffi::c_uint
        {
            if tty_term_of(tty).has(TTYC_SE) {
                tty_raw(tty, &tty_term_string_for(tty, TTYC_SE));
            } else if tty_term_of(tty).has(TTYC_SS) {
                let cursor = tty_term_of(tty).expand_i(TTYC_SS, 0);
                tty_raw(tty, &cursor);
            }
        }
        if tty.ccolour != -(1 as core::ffi::c_int) {
            tty_raw(tty, &tty_term_string_for(tty, TTYC_CR));
        }
        tty_raw(tty, &tty_term_string_for(tty, TTYC_CNORM));
        if tty_term_of(tty).has(TTYC_KMOUS) {
            tty_raw(tty, c"\x1B[?1000l\x1B[?1002l\x1B[?1003l");
            tty_raw(tty, c"\x1B[?1006l\x1B[?1005l");
        }
        if tty_term_of(tty).has(TTYC_DSBP) {
            tty_raw(tty, &tty_term_string_for(tty, TTYC_DSBP));
        }
        if tty_term_of(tty).flags() & TERM_VT100LIKE != 0 {
            tty_raw(tty, c"\x1B[?7727l");
        }
        tty_raw(tty, &tty_term_string_for(tty, TTYC_DSFCS));
        tty_raw(tty, &tty_term_string_for(tty, TTYC_DSEKS));
        if tty_term_of(tty).flags() & TERM_DECSLRM != 0 {
            tty_raw(tty, &tty_term_string_for(tty, TTYC_DSMG));
        }
        tty_raw(tty, &tty_term_string_for(tty, TTYC_RMCUP));
        if tty_term_of(tty).flags() & TERM_VT100LIKE != 0 {
            tty_raw(tty, c"\x1B[?2031l");
        }
        setblocking(
            c.as_ref().expect("the tty has a client").fd(),
            1 as core::ffi::c_int,
        );
    }
}
pub unsafe fn tty_close(tty: &mut tty) {
    unsafe {
        tty.key_timer.disarm();
        tty_stop_tty(tty);
        if tty.flags & TTY_OPENED != 0 {
            tty.r#in = None;
            tty.event_in.disable();
            tty.out = None;
            tty.event_out.disable();
            if let Some(term) = tty.term.take() {
                tty_term_free(term);
            }
            tty_keys_free(tty);
            tty.flags &= !TTY_OPENED;
        }
    }
}
pub unsafe fn tty_free(tty: &mut tty) {
    unsafe {
        tty_close(tty);
        *tty.r.borrow_mut() = visible_ranges::default();
    }
}
pub unsafe fn tty_update_features(tty: &mut tty) {
    unsafe {
        let mut c = tty_client(tty);
        if tty_term_opt_mut(&mut tty.term)
            .expect("a tty being driven has a terminal")
            .apply_features(
                c.as_ref()
                    .expect("the tty has a client")
                    .terminal_features(),
            )
        {
            tty_term_apply_overrides(
                &mut tty_term_opt_mut(&mut tty.term).expect("a tty being driven has a terminal"),
            );
        }
        if tty_term_of(tty).flags() & TERM_DECSLRM != 0 {
            tty_putcode(tty, TTYC_ENMG);
        }
        if (global_options
            .get()
            .as_ref()
            .expect("global options are initialized"))
        .number(c"extended-keys")
            != 0
        {
            tty_puts(tty, &tty_term_string_for(tty, TTYC_ENEKS));
        }
        if (global_options
            .get()
            .as_ref()
            .expect("global options are initialized"))
        .number(c"focus-events")
            != 0
        {
            tty_puts(tty, &tty_term_string_for(tty, TTYC_ENFCS));
        }
        if tty_term_of(tty).flags() & TERM_VT100LIKE != 0 {
            tty_puts(tty, c"\x1B[?7727h");
        }
        *c.as_mut().expect("the tty has a client").flags_mut() |= CLIENT_ALLREDRAWFLAGS as uint64_t;
        tty_invalidate(tty);
    }
}
pub unsafe fn tty_raw(tty: &mut tty, s: &core::ffi::CStr) {
    unsafe {
        let c = tty_client(tty);
        let mut left = s.to_bytes();
        for _ in 0..5 {
            let n = write(
                c.as_ref().expect("the tty has a client").fd(),
                left.as_ptr() as *const core::ffi::c_void,
                left.len() as size_t,
            );
            if n >= 0 as ssize_t {
                left = &left[n as usize..];
                if left.is_empty() {
                    break;
                }
            } else if n == -(1 as core::ffi::c_int) as ssize_t && *__errno_location() != EAGAIN {
                break;
            }
            usleep(100 as __useconds_t);
        }
    }
}

fn tty_term_string_for(tty: &tty, code: tty_code_code) -> std::ffi::CString {
    tty_term_of(tty).string(code).to_owned()
}

pub fn tty_putcode(tty: &mut tty, code: tty_code_code) {
    {
        tty_puts(tty, &tty_term_string_for(tty, code));
    }
}
pub fn tty_putcode_i(tty: &mut tty, code: tty_code_code, a: core::ffi::c_int) {
    if a < 0 as core::ffi::c_int {
        return;
    }
    let expanded = tty_term_of(tty).expand_i(code, a);
    tty_puts(tty, &expanded);
}
pub fn tty_putcode_ii(
    tty: &mut tty,
    code: tty_code_code,
    a: core::ffi::c_int,
    b: core::ffi::c_int,
) {
    if a < 0 as core::ffi::c_int || b < 0 as core::ffi::c_int {
        return;
    }
    let expanded = tty_term_of(tty).expand_ii(code, a, b);
    tty_puts(tty, &expanded);
}
pub fn tty_putcode_iii(
    tty: &mut tty,
    code: tty_code_code,
    a: core::ffi::c_int,
    b: core::ffi::c_int,
    c: core::ffi::c_int,
) {
    if a < 0 as core::ffi::c_int || b < 0 as core::ffi::c_int || c < 0 as core::ffi::c_int {
        return;
    }
    let expanded = tty_term_of(tty).expand_iii(code, a, b, c);
    tty_puts(tty, &expanded);
}
pub fn tty_putcode_s(tty: &mut tty, code: tty_code_code, a: &core::ffi::CStr) {
    let expanded = tty_term_of(tty).expand_s(code, a);
    tty_puts(tty, &expanded);
}
pub fn tty_putcode_ss(
    tty: &mut tty,
    code: tty_code_code,
    a: &core::ffi::CStr,
    b: &core::ffi::CStr,
) {
    let expanded = tty_term_of(tty).expand_ss(code, a, b);
    tty_puts(tty, &expanded);
}
fn tty_add(tty: &mut tty, buf: &[u8]) {
    unsafe {
        let mut held = tty_client(tty);
        let len = buf.len();
        if tty.flags & TTY_BLOCK != 0 {
            tty.discarded = tty.discarded.wrapping_add(len);
            return;
        }
        tty.out.as_mut().unwrap().append(buf);
        log_debug(
            c"%s: %.*s",
            fmt_args![
                held.as_ref().and_then(|client| client.name()),
                len as core::ffi::c_int,
                buf.as_ptr().cast::<core::ffi::c_char>()
            ],
        );
        if let Some(client) = held.as_mut() {
            client.record_written(len);
        }
        if tty_log_fd.get() != -(1 as core::ffi::c_int) {
            write(
                tty_log_fd.get(),
                buf.as_ptr().cast::<core::ffi::c_void>(),
                len,
            );
        }
        if tty.flags & TTY_STARTED != 0 {
            tty.event_out.enable();
        }
    }
}
pub fn tty_puts(tty: &mut tty, s: &core::ffi::CStr) {
    if !s.to_bytes().is_empty() {
        tty_add(tty, s.to_bytes());
    }
}
pub unsafe fn tty_putc(tty: &mut tty, ch: u_char) {
    unsafe {
        if tty_term_of(tty).flags() & TERM_NOAM != 0
            && ch as core::ffi::c_int >= 0x20 as core::ffi::c_int
            && ch as core::ffi::c_int != 0x7f as core::ffi::c_int
            && tty.cy == tty.sy.wrapping_sub(1 as u_int)
            && tty.cx.wrapping_add(1 as u_int) >= tty.sx
        {
            return;
        }
        if tty.cell.attr as core::ffi::c_int & GRID_ATTR_CHARSET != 0
            && let Some(acs) = tty_acs_get(Some(tty), ch)
        {
            tty_add(tty, acs.to_bytes());
        } else {
            tty_add(tty, core::slice::from_ref(&ch));
        }
        if ch as core::ffi::c_int >= 0x20 as core::ffi::c_int
            && ch as core::ffi::c_int != 0x7f as core::ffi::c_int
        {
            if tty.cx >= tty.sx {
                tty.cx = 1 as u_int;
                if tty.cy != tty.rlower {
                    tty.cy = tty.cy.wrapping_add(1);
                }
                if tty_term_of(tty).flags() & TERM_NOAM != 0 {
                    tty_putcode_ii(
                        tty,
                        TTYC_CUP,
                        tty.cy as core::ffi::c_int,
                        tty.cx as core::ffi::c_int,
                    );
                }
            } else {
                tty.cx = tty.cx.wrapping_add(1);
            }
        }
    }
}
/// Writes `buf` and moves the cursor on by `width` columns.
///
/// A terminal that does not wrap keeps the last column of the last line
/// clear, so the run is cut short of it. The cut can only shorten the run:
/// it is reached with the cursor inside the line, where the columns left of
/// the last one are fewer than the bytes that would have run past them.
pub fn tty_putn(tty: &mut tty, buf: &[u8], width: u_int) {
    let mut len = buf.len();
    if tty_term_of(tty).flags() & TERM_NOAM != 0
        && tty.cy == tty.sy.wrapping_sub(1 as u_int)
        && (tty.cx as size_t).wrapping_add(len) >= tty.sx as size_t
    {
        len = len.min(tty.sx.wrapping_sub(tty.cx).wrapping_sub(1 as u_int) as size_t);
    }
    tty_add(tty, &buf[..len]);
    if tty.cx.wrapping_add(width) > tty.sx {
        tty.cx = tty.cx.wrapping_add(width).wrapping_sub(tty.sx);
        if tty.cx <= tty.sx {
            tty.cy = tty.cy.wrapping_add(1);
        } else {
            tty.cy = UINT_MAX as u_int;
            tty.cx = tty.cy;
        }
    } else {
        tty.cx = tty.cx.wrapping_add(width);
    };
}
unsafe fn tty_set_italics(tty: &mut tty) {
    {
        if tty_term_of(tty).has(TTYC_SITM) {
            let s = global_options
                .get()
                .expect("global options are initialized")
                .string_ref(c"default-terminal");
            if s.as_ref() != c"screen" && !s.to_bytes().starts_with(b"screen-") {
                tty_putcode(tty, TTYC_SITM);
                return;
            }
        }
        tty_putcode(tty, TTYC_SMSO);
    }
}
pub fn tty_set_title(tty: &mut tty, title: &core::ffi::CStr) {
    {
        if !tty_term_of(tty).has(TTYC_TSL) || !tty_term_of(tty).has(TTYC_FSL) {
            return;
        }
        tty_putcode(tty, TTYC_TSL);
        tty_puts(tty, title);
        tty_putcode(tty, TTYC_FSL);
    }
}
pub fn tty_set_path(tty: &mut tty, title: &core::ffi::CStr) {
    {
        if !tty_term_of(tty).has(TTYC_SWD) || !tty_term_of(tty).has(TTYC_FSL) {
            return;
        }
        tty_putcode(tty, TTYC_SWD);
        tty_puts(tty, title);
        tty_putcode(tty, TTYC_FSL);
    }
}
fn tty_force_cursor_colour(tty: &mut tty, mut c: core::ffi::c_int) {
    {
        if c != -(1 as core::ffi::c_int) {
            c = RustColourEngine.force_rgb(c);
        }
        if c == tty.ccolour {
            return;
        }
        if c == -(1 as core::ffi::c_int) {
            tty_putcode(tty, TTYC_CR);
        } else {
            let (r, g, b) = RustColourEngine.split_rgb(c);
            let s = xasprintf(
                c"rgb:%02hhx/%02hhx/%02hhx",
                fmt_args![
                    r as core::ffi::c_int,
                    g as core::ffi::c_int,
                    b as core::ffi::c_int
                ],
            );
            tty_putcode_s(tty, TTYC_CS, &s);
        }
        tty.ccolour = c;
    }
}
fn tty_update_cursor(
    tty: &mut tty,
    mode: core::ffi::c_int,
    s: Option<ScreenModeState>,
) -> core::ffi::c_int {
    {
        let mut cstyle: screen_cursor_style;
        let ccolour: core::ffi::c_int;

        let mut cmode: core::ffi::c_int = mode;
        if let Some(s) = s {
            ccolour = s.cursor_colour;
            tty_force_cursor_colour(tty, ccolour);
        }
        if !cmode & MODE_CURSOR != 0 {
            if tty.mode & MODE_CURSOR != 0 {
                tty_putcode(tty, TTYC_CIVIS);
            }
            return cmode;
        }
        if let Some(s) = s {
            cstyle = s.cursor_style;
            if cstyle as core::ffi::c_uint
                == SCREEN_CURSOR_DEFAULT as core::ffi::c_int as core::ffi::c_uint
            {
                if !cmode & MODE_CURSOR_BLINKING_SET != 0 {
                    if s.default_cursor_mode & MODE_CURSOR_BLINKING != 0 {
                        cmode |= MODE_CURSOR_BLINKING;
                    } else {
                        cmode &= !MODE_CURSOR_BLINKING;
                    }
                }
                cstyle = s.default_cursor_style;
            }
        } else {
            cstyle = tty.cstyle;
        }
        let changed: core::ffi::c_int = cmode ^ tty.mode;
        if changed & CURSOR_MODES == 0 as core::ffi::c_int
            && cstyle as core::ffi::c_uint == tty.cstyle as core::ffi::c_uint
        {
            return cmode;
        }
        tty_putcode(tty, TTYC_CNORM);
        match cstyle {
            SCREEN_CURSOR_DEFAULT => {
                if tty.cstyle as core::ffi::c_uint
                    != SCREEN_CURSOR_DEFAULT as core::ffi::c_int as core::ffi::c_uint
                {
                    if tty_term_of(tty).has(TTYC_SE) {
                        tty_putcode(tty, TTYC_SE);
                    } else {
                        tty_putcode_i(tty, TTYC_SS, 0 as core::ffi::c_int);
                    }
                }
                if cmode & (MODE_CURSOR_BLINKING | MODE_CURSOR_VERY_VISIBLE) != 0 {
                    tty_putcode(tty, TTYC_CVVIS);
                }
            }
            SCREEN_CURSOR_BLOCK => {
                if tty_term_of(tty).has(TTYC_SS) {
                    if cmode & MODE_CURSOR_BLINKING != 0 {
                        tty_putcode_i(tty, TTYC_SS, 1 as core::ffi::c_int);
                    } else {
                        tty_putcode_i(tty, TTYC_SS, 2 as core::ffi::c_int);
                    }
                } else if cmode & MODE_CURSOR_BLINKING != 0 {
                    tty_putcode(tty, TTYC_CVVIS);
                }
            }
            SCREEN_CURSOR_UNDERLINE => {
                if tty_term_of(tty).has(TTYC_SS) {
                    if cmode & MODE_CURSOR_BLINKING != 0 {
                        tty_putcode_i(tty, TTYC_SS, 3 as core::ffi::c_int);
                    } else {
                        tty_putcode_i(tty, TTYC_SS, 4 as core::ffi::c_int);
                    }
                } else if cmode & MODE_CURSOR_BLINKING != 0 {
                    tty_putcode(tty, TTYC_CVVIS);
                }
            }
            SCREEN_CURSOR_BAR => {
                if tty_term_of(tty).has(TTYC_SS) {
                    if cmode & MODE_CURSOR_BLINKING != 0 {
                        tty_putcode_i(tty, TTYC_SS, 5 as core::ffi::c_int);
                    } else {
                        tty_putcode_i(tty, TTYC_SS, 6 as core::ffi::c_int);
                    }
                } else if cmode & MODE_CURSOR_BLINKING != 0 {
                    tty_putcode(tty, TTYC_CVVIS);
                }
            }
            _ => {}
        }
        tty.cstyle = cstyle;
        cmode
    }
}
pub unsafe fn tty_update_mode(
    tty: &mut tty,
    mut mode: core::ffi::c_int,
    s: Option<ScreenModeState>,
) {
    unsafe {
        let c = tty_client(tty);

        if tty.flags & TTY_NOCURSOR != 0 {
            mode &= !MODE_CURSOR;
        }
        if tty_update_cursor(tty, mode, s) & MODE_CURSOR_BLINKING != 0 {
            mode |= MODE_CURSOR_BLINKING;
        } else {
            mode &= !MODE_CURSOR_BLINKING;
        }
        let changed: core::ffi::c_int = mode ^ tty.mode;
        if log_get_level() != 0 as core::ffi::c_int && changed != 0 as core::ffi::c_int {
            log_debug(
                c"%s: current mode %s",
                fmt_args![
                    c.as_ref().expect("the tty has a client").name(),
                    screen_mode_to_string(tty.mode).as_c_str()
                ],
            );
            log_debug(
                c"%s: setting mode %s",
                fmt_args![
                    c.as_ref().expect("the tty has a client").name(),
                    screen_mode_to_string(mode).as_c_str()
                ],
            );
        }
        if changed & ALL_MOUSE_MODES != 0 && tty_term_of(tty).has(TTYC_KMOUS) {
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
        tty.mode = mode;
    }
}
fn tty_emulate_repeat(tty: &mut tty, code: tty_code_code, code1: tty_code_code, mut n: u_int) {
    {
        if tty_term_of(tty).has(code) {
            tty_putcode_i(tty, code, n as core::ffi::c_int);
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
}
pub unsafe fn tty_repeat_space(tty: &mut tty, mut n: u_int) {
    const SPACES: crate::server_state::Value<[u_char; 500]> =
        crate::server_state::Value::new(|state| &state.tty_string_buffer);
    SPACES.with_mut(|spaces| {
        if spaces[0] != b' ' {
            spaces.fill(b' ');
        }
        while n as usize > spaces.len() {
            tty_putn(tty, spaces, spaces.len() as u_int);
            n -= spaces.len() as u_int;
        }
        if n != 0 {
            tty_putn(tty, &spaces[..n as usize], n);
        }
    });
}

pub unsafe fn tty_window_bigger(tty: &tty) -> core::ffi::c_int {
    unsafe {
        let held = tty_client(tty).expect("the tty has a client");
        let Some(session) = held.attached_session() else {
            return 0;
        };
        let Some(window) = session.curw().and_then(|link| link.window()) else {
            return 0;
        };
        let size = window.dimensions().size;
        (tty.sx < size.width
            || tty.sy.wrapping_sub(status_line_size(held.as_client())) < size.height)
            as core::ffi::c_int
    }
}
/// Whether the window is bigger than the terminal, and the part of it the
/// terminal shows: offset and size.
pub fn tty_window_offset(tty: &tty) -> (core::ffi::c_int, u_int, u_int, u_int, u_int) {
    (tty.oflag, tty.oox, tty.ooy, tty.osx, tty.osy)
}
unsafe fn tty_window_offset1(
    c: &mut client,
    width: u_int,
    height: u_int,
    ox: &mut u_int,
    oy: &mut u_int,
    sx: &mut u_int,
    sy: &mut u_int,
) -> core::ffi::c_int {
    unsafe {
        let lines = status_line_size(c);
        *ox = 0;
        *oy = 0;
        *sx = width;
        *sy = height.wrapping_sub(lines);
        let Some(session) = c.attached_session() else {
            c.pan_window = None;
            return 0;
        };
        let Some(window) = session.curw().and_then(|link| link.window()) else {
            c.pan_window = None;
            return 0;
        };
        let size = window.dimensions().size;
        let cx: u_int;
        let cy: u_int;
        if width >= size.width && height.wrapping_sub(lines) >= size.height {
            *ox = 0 as u_int;
            *oy = 0 as u_int;
            *sx = size.width;
            *sy = size.height;
            c.pan_window = None;
            return 0 as core::ffi::c_int;
        }
        *sx = width;
        *sy = height.wrapping_sub(lines);
        if client_get_pan_window(&*c).is_some_and(|pw| pw.ptr_eq(&window)) {
            if *sx >= size.width {
                c.pan_ox = 0 as u_int;
            } else if c.pan_ox.wrapping_add(*sx) > size.width {
                c.pan_ox = size.width.wrapping_sub(*sx);
            }
            *ox = c.pan_ox;
            if *sy >= size.height {
                c.pan_oy = 0 as u_int;
            } else if c.pan_oy.wrapping_add(*sy) > size.height {
                c.pan_oy = size.height.wrapping_sub(*sy);
            }
            *oy = c.pan_oy;
            return 1 as core::ffi::c_int;
        }
        let pane = crate::server::server_client_get_pane_in_window(c, &window);
        let Some(wp) = pane.as_ref().and_then(|pane| pane.get()) else {
            c.pan_window = None;
            return 1;
        };
        let pane_screen = wp.screen_ref();
        if !pane_screen.mode() & MODE_CURSOR != 0 {
            *ox = 0 as u_int;
            *oy = 0 as u_int;
        } else {
            let (screen_cx, screen_cy) = pane_screen.cursor();
            cx = (wp.geometry().xoff as u_int).wrapping_add(screen_cx);
            cy = (wp.geometry().yoff as u_int).wrapping_add(screen_cy);
            if cx < *sx {
                *ox = 0 as u_int;
            } else if cx > size.width.wrapping_sub(*sx) {
                *ox = size.width.wrapping_sub(*sx);
            } else {
                *ox = cx.wrapping_sub((*sx).wrapping_div(2 as u_int));
            }
            if cy < *sy {
                *oy = 0 as u_int;
            } else if cy > size.height.wrapping_sub(*sy) {
                *oy = size.height.wrapping_sub(*sy);
            } else {
                *oy = cy.wrapping_sub(*sy).wrapping_add(1 as u_int);
            }
        }
        c.pan_window = None;
        1 as core::ffi::c_int
    }
}

pub unsafe fn tty_update_client_offset(c: &mut client) {
    unsafe {
        let mut ox: u_int = 0;
        let mut oy: u_int = 0;
        let mut sx: u_int = 0;
        let mut sy: u_int = 0;
        if !c.flags & CLIENT_TERMINAL as uint64_t != 0 {
            return;
        }
        let (width, height) = (c.tty.sx, c.tty.sy);
        c.tty.oflag = tty_window_offset1(c, width, height, &mut ox, &mut oy, &mut sx, &mut sy);
        if ox == c.tty.oox && oy == c.tty.ooy && sx == c.tty.osx && sy == c.tty.osy {
            return;
        }
        log_debug(
            c"%s: %s offset has changed (%u,%u %ux%u -> %u,%u %ux%u)",
            fmt_args![
                c"tty_update_client_offset".as_ptr(),
                c.name.as_deref(),
                c.tty.oox,
                c.tty.ooy,
                c.tty.osx,
                c.tty.osy,
                ox,
                oy,
                sx,
                sy
            ],
        );
        c.tty.oox = ox;
        c.tty.ooy = oy;
        c.tty.osx = sx;
        c.tty.osy = sy;
        c.flags |= (CLIENT_REDRAWWINDOW | CLIENT_REDRAWSTATUS) as uint64_t;
    }
}
fn tty_large_region(ctx: &tty_ctx) -> core::ffi::c_int {
    (ctx.orlower.wrapping_sub(ctx.orupper) >= ctx.sy.wrapping_div(2 as u_int)) as core::ffi::c_int
}
pub fn tty_fake_bce(tty: &tty, gc: &grid_cell, bg: u_int) -> core::ffi::c_int {
    {
        if tty_term_of(tty).flag(TTYC_BCE) != 0 {
            return 0 as core::ffi::c_int;
        }
        if !(bg == 8 as u_int || bg == 9 as u_int)
            || !(gc.bg == 8 as core::ffi::c_int || gc.bg == 9 as core::ffi::c_int)
        {
            return 1 as core::ffi::c_int;
        }
        0 as core::ffi::c_int
    }
}
unsafe fn tty_redraw_region(tty: &mut tty, ctx: &tty_ctx, s: &RustScreen) {
    unsafe {
        let c = tty_client(tty);
        let mut i: u_int;
        if tty_large_region(ctx) != 0 || ctx.flags & TTY_CTX_PANE_OBSCURED != 0 {
            log_debug(
                c"%s: %s large region redraw",
                fmt_args![
                    c"tty_redraw_region".as_ptr(),
                    c.as_ref().expect("the tty has a client").name()
                ],
            );
            ctx.redraw_cb.expect("non-null function pointer")(ctx);
            return;
        }
        log_debug(
            c"%s: %s small region redraw (%u-%u)",
            fmt_args![
                c"tty_redraw_region".as_ptr(),
                c.as_ref().expect("the tty has a client").name(),
                ctx.orupper,
                ctx.orlower
            ],
        );
        i = ctx.orupper;
        while i <= ctx.orlower {
            tty_draw_pane(tty, ctx, i, s);
            i = i.wrapping_add(1);
        }
    }
}
fn tty_is_visible(ctx: &tty_ctx, px: u_int, py: u_int, nx: u_int, ny: u_int) -> core::ffi::c_int {
    let xoff: u_int = (ctx.rxoff as u_int).wrapping_add(px);
    let yoff: u_int = (ctx.ryoff as u_int).wrapping_add(py);
    if !ctx.flags & TTY_CTX_WINDOW_BIGGER != 0 {
        return 1 as core::ffi::c_int;
    }
    if xoff.wrapping_add(nx) <= ctx.wox
        || xoff >= ctx.wox.wrapping_add(ctx.wsx)
        || yoff.wrapping_add(ny) <= ctx.woy
        || yoff >= ctx.woy.wrapping_add(ctx.wsy)
    {
        return 0 as core::ffi::c_int;
    }
    1 as core::ffi::c_int
}
fn tty_clamp_line(
    ctx: &tty_ctx,
    px: u_int,
    py: u_int,
    nx: u_int,
) -> Option<(u_int, u_int, u_int, u_int)> {
    {
        let i: u_int;
        let x: u_int;
        let rx: u_int;

        let xoff: core::ffi::c_int = (ctx.rxoff as u_int).wrapping_add(px) as core::ffi::c_int;
        if tty_is_visible(ctx, px, py, nx, 1 as u_int) == 0 {
            return None;
        }
        let ry: u_int = (ctx.yoff as u_int).wrapping_add(py).wrapping_sub(ctx.woy);
        if xoff >= ctx.wox as core::ffi::c_int
            && (xoff as u_int).wrapping_add(nx) <= ctx.wox.wrapping_add(ctx.wsx)
        {
            i = 0 as u_int;
            x = (ctx.xoff as u_int).wrapping_add(px).wrapping_sub(ctx.wox);
            rx = nx;
        } else if xoff < ctx.wox as core::ffi::c_int
            && (xoff as u_int).wrapping_add(nx) > ctx.wox.wrapping_add(ctx.wsx)
        {
            i = ctx.wox;
            x = 0 as u_int;
            rx = ctx.wsx;
        } else if xoff < ctx.wox as core::ffi::c_int {
            i = ctx.wox.wrapping_sub((ctx.xoff as u_int).wrapping_add(px));
            x = 0 as u_int;
            rx = nx.wrapping_sub(i);
        } else {
            i = 0 as u_int;
            x = (ctx.xoff as u_int).wrapping_add(px).wrapping_sub(ctx.wox);
            rx = ctx.wsx.wrapping_sub(x);
        }
        if rx > nx {
            fatalx(
                c"%s: x too big, %u > %u",
                fmt_args![c"tty_clamp_line".as_ptr(), rx, nx],
            );
        }
        Some((i, x, rx, ry))
    }
}
unsafe fn tty_clear_line(
    tty: &mut tty,
    defaults: &grid_cell,
    py: u_int,
    px: u_int,
    nx: u_int,
    bg: u_int,
) {
    unsafe {
        let c = tty_client(tty);
        let mut i: u_int;
        log_debug(
            c"%s: %s, %u at %u,%u",
            fmt_args![
                c"tty_clear_line".as_ptr(),
                c.as_ref().expect("the tty has a client").name(),
                nx,
                px,
                py
            ],
        );
        if nx == 0 as u_int {
            return;
        }
        if !c
            .as_ref()
            .expect("the tty has a client")
            .has_overlay_check()
            && tty_fake_bce(tty, defaults, bg) == 0
        {
            if px.wrapping_add(nx) >= tty.sx && tty_term_of(tty).has(TTYC_EL) {
                tty_cursor(tty, px, py);
                tty_putcode(tty, TTYC_EL);
                return;
            }
            if px == 0 as u_int && tty_term_of(tty).has(TTYC_EL1) {
                tty_cursor(tty, px.wrapping_add(nx).wrapping_sub(1 as u_int), py);
                tty_putcode(tty, TTYC_EL1);
                return;
            }
            if tty_term_of(tty).has(TTYC_ECH) {
                tty_cursor(tty, px, py);
                tty_putcode_i(tty, TTYC_ECH, nx as core::ffi::c_int);
                return;
            }
        }
        let r = tty_check_overlay_range(tty, px, py, nx);
        i = 0 as u_int;
        while let Some(rr) = r.range_at(i) {
            if rr.nx != 0 as u_int {
                tty_cursor(tty, rr.px, py);
                tty_repeat_space(tty, rr.nx);
            }
            i = i.wrapping_add(1);
        }
    }
}
unsafe fn tty_clear_pane_line(
    tty: &mut tty,
    ctx: &tty_ctx,
    py: u_int,
    px: u_int,
    nx: u_int,
    bg: u_int,
) {
    unsafe {
        let c = tty_client(tty);
        let mut i: u_int;
        let x: u_int;
        let rx: u_int;
        let ry: u_int;
        log_debug(
            c"%s: %s, %u at %u,%u",
            fmt_args![
                c"tty_clear_pane_line".as_ptr(),
                c.as_ref().expect("the tty has a client").name(),
                nx,
                px,
                py
            ],
        );
        if let Some((_clamped_l, clamped_x, clamped_rx, clamped_ry)) =
            tty_clamp_line(ctx, px, py, nx)
        {
            (x, rx, ry) = (clamped_x, clamped_rx, clamped_ry);
            let r = tty_check_overlay_range(tty, x, ry, rx);
            i = 0 as u_int;
            while let Some(ri) = r.range_at(i) {
                if !(ri.nx == 0 as u_int) {
                    tty_clear_line(tty, &ctx.defaults, ry, ri.px, ri.nx, bg);
                }
                i = i.wrapping_add(1);
            }
        }
    }
}
fn tty_clamp_area(
    ctx: &tty_ctx,
    px: u_int,
    py: u_int,
    nx: u_int,
    ny: u_int,
) -> Option<(u_int, u_int, u_int, u_int, u_int, u_int)> {
    {
        let i: u_int;
        let j: u_int;
        let x: u_int;
        let y: u_int;
        let rx: u_int;
        let ry: u_int;
        let xoff: u_int = (ctx.rxoff as u_int).wrapping_add(px);
        let yoff: u_int = (ctx.ryoff as u_int).wrapping_add(py);
        if tty_is_visible(ctx, px, py, nx, ny) == 0 {
            return None;
        }
        if xoff >= ctx.wox && xoff.wrapping_add(nx) <= ctx.wox.wrapping_add(ctx.wsx) {
            i = 0 as u_int;
            x = (ctx.xoff as u_int).wrapping_add(px).wrapping_sub(ctx.wox);
            rx = nx;
        } else if xoff < ctx.wox && xoff.wrapping_add(nx) > ctx.wox.wrapping_add(ctx.wsx) {
            i = ctx.wox;
            x = 0 as u_int;
            rx = ctx.wsx;
        } else if xoff < ctx.wox {
            i = ctx.wox.wrapping_sub((ctx.xoff as u_int).wrapping_add(px));
            x = 0 as u_int;
            rx = nx.wrapping_sub(i);
        } else {
            i = 0 as u_int;
            x = (ctx.xoff as u_int).wrapping_add(px).wrapping_sub(ctx.wox);
            rx = ctx.wsx.wrapping_sub(x);
        }
        if rx > nx {
            fatalx(
                c"%s: x too big, %u > %u",
                fmt_args![c"tty_clamp_area".as_ptr(), rx, nx],
            );
        }
        if yoff >= ctx.woy && yoff.wrapping_add(ny) <= ctx.woy.wrapping_add(ctx.wsy) {
            j = 0 as u_int;
            y = (ctx.yoff as u_int).wrapping_add(py).wrapping_sub(ctx.woy);
            ry = ny;
        } else if yoff < ctx.woy && yoff.wrapping_add(ny) > ctx.woy.wrapping_add(ctx.wsy) {
            j = ctx.woy;
            y = 0 as u_int;
            ry = ctx.wsy;
        } else if yoff < ctx.woy {
            j = ctx.woy.wrapping_sub((ctx.yoff as u_int).wrapping_add(py));
            y = 0 as u_int;
            ry = ny.wrapping_sub(j);
        } else {
            j = 0 as u_int;
            y = (ctx.yoff as u_int).wrapping_add(py).wrapping_sub(ctx.woy);
            ry = ctx.wsy.wrapping_sub(y);
        }
        if ry > ny {
            fatalx(
                c"%s: y too big, %u > %u",
                fmt_args![c"tty_clamp_area".as_ptr(), ry, ny],
            );
        }
        Some((i, j, x, y, rx, ry))
    }
}
unsafe fn tty_clear_area(
    tty: &mut tty,
    ctx: &tty_ctx,
    py: u_int,
    ny: u_int,
    px: u_int,
    nx: u_int,
    bg: u_int,
) {
    unsafe {
        let c = tty_client(tty);
        let defaults = &ctx.defaults;
        let mut yy: u_int;
        log_debug(
            c"%s: %s, %u,%u at %u,%u",
            fmt_args![
                c"tty_clear_area".as_ptr(),
                c.as_ref().expect("the tty has a client").name(),
                nx,
                ny,
                px,
                py
            ],
        );
        if nx == 0 as u_int || ny == 0 as u_int {
            return;
        }
        if !c
            .as_ref()
            .expect("the tty has a client")
            .has_overlay_check()
            && tty_fake_bce(tty, defaults, bg) == 0
        {
            if px == 0 as u_int
                && px.wrapping_add(nx) >= tty.sx
                && py.wrapping_add(ny) >= tty.sy
                && tty_term_of(tty).has(TTYC_ED)
            {
                tty_cursor(tty, 0 as u_int, py);
                tty_putcode(tty, TTYC_ED);
                return;
            }
            if tty_term_of(tty).flags() & TERM_DECFRA != 0
                && !(bg == 8 as u_int || bg == 9 as u_int)
            {
                let tmp = xasprintf(
                    c"\x1B[32;%u;%u;%u;%u$x",
                    fmt_args![
                        py.wrapping_add(1 as u_int),
                        px.wrapping_add(1 as u_int),
                        py.wrapping_add(ny),
                        px.wrapping_add(nx)
                    ],
                );
                tty_puts(tty, &tmp);
                return;
            }
            if px == 0 as u_int
                && px.wrapping_add(nx) >= tty.sx
                && ny > 2 as u_int
                && tty_term_of(tty).has(TTYC_CSR)
                && tty_term_of(tty).has(TTYC_INDN)
            {
                tty_region(tty, py, py.wrapping_add(ny).wrapping_sub(1 as u_int));
                tty_margin_off(tty);
                tty_putcode_i(tty, TTYC_INDN, ny as core::ffi::c_int);
                return;
            }
            if nx > 2 as u_int
                && ny > 2 as u_int
                && tty_term_of(tty).has(TTYC_CSR)
                && tty_term_of(tty).flags() & TERM_DECSLRM != 0
                && tty_term_of(tty).has(TTYC_INDN)
            {
                tty_region(tty, py, py.wrapping_add(ny).wrapping_sub(1 as u_int));
                tty_margin(tty, px, px.wrapping_add(nx).wrapping_sub(1 as u_int));
                tty_putcode_i(tty, TTYC_INDN, ny as core::ffi::c_int);
                return;
            }
        }
        yy = py;
        while yy < py.wrapping_add(ny) {
            tty_clear_line(tty, defaults, yy, px, nx, bg);
            yy = yy.wrapping_add(1);
        }
    }
}
unsafe fn tty_clear_pane_area(
    tty: &mut tty,
    ctx: &tty_ctx,
    py: u_int,
    ny: u_int,
    px: u_int,
    nx: u_int,
    bg: u_int,
) {
    unsafe {
        let x: u_int;
        let y: u_int;
        let rx: u_int;
        let ry: u_int;
        if let Some((_clamped_i, _clamped_j, clamped_x, clamped_y, clamped_rx, clamped_ry)) =
            tty_clamp_area(ctx, px, py, nx, ny)
        {
            (x, y, rx, ry) = (clamped_x, clamped_y, clamped_rx, clamped_ry);
            tty_clear_area(tty, ctx, y, ry, x, rx, bg);
        }
    }
}
unsafe fn tty_draw_pane(tty: &mut tty, ctx: &tty_ctx, py: u_int, s: &RustScreen) {
    unsafe {
        let nx: u_int = ctx.sx;
        let i: u_int;
        let x: u_int;
        let rx: u_int;
        let ry: u_int;
        let mut j: u_int;
        log_debug(
            c"%s: %s %u",
            fmt_args![
                c"tty_draw_pane".as_ptr(),
                tty_client(tty).expect("the tty has a client").name(),
                py
            ],
        );
        if !ctx.flags & TTY_CTX_WINDOW_BIGGER != 0 {
            let r = tty_check_overlay_range(
                tty,
                ctx.xoff as u_int,
                (ctx.yoff as u_int).wrapping_add(py),
                nx,
            );
            j = 0 as u_int;
            while let Some(rr) = r.range_at(j) {
                if !(rr.nx == 0 as u_int) {
                    tty_draw_line(
                        tty,
                        s,
                        rr.px.wrapping_sub(ctx.xoff as u_int),
                        py,
                        rr.nx,
                        rr.px,
                        (ctx.yoff as u_int).wrapping_add(py),
                        &ctx.defaults,
                        ctx.palette.as_ref(),
                    );
                }
                j = j.wrapping_add(1);
            }
            return;
        }
        if let Some((clamped_i, clamped_x, clamped_rx, clamped_ry)) =
            tty_clamp_line(ctx, 0 as u_int, py, nx)
        {
            (i, x, rx, ry) = (clamped_i, clamped_x, clamped_rx, clamped_ry);
            let r = tty_check_overlay_range(tty, x, ry, rx);
            j = 0 as u_int;
            while let Some(rr) = r.range_at(j) {
                if !(rr.nx == 0 as u_int) {
                    tty_draw_line(
                        tty,
                        s,
                        i.wrapping_add(rr.px).wrapping_sub(x),
                        py,
                        rr.nx,
                        rr.px,
                        ry,
                        &ctx.defaults,
                        ctx.palette.as_ref(),
                    );
                }
                j = j.wrapping_add(1);
            }
        }
    }
}
pub unsafe fn tty_cmd_redrawline(tty: &mut tty, ctx: &tty_ctx, s: &RustScreen) {
    unsafe {
        let i: u_int;
        let x: u_int;
        let rx: u_int;
        let ry: u_int;
        let mut j: u_int;
        if let Some((clamped_i, clamped_x, clamped_rx, clamped_ry)) =
            tty_clamp_line(ctx, ctx.ocx, ctx.ocy, tty_ctx_num(ctx))
        {
            (i, x, rx, ry) = (clamped_i, clamped_x, clamped_rx, clamped_ry);
            let r = tty_check_overlay_range(tty, x, ry, rx);
            j = 0 as u_int;
            while let Some(rr) = r.range_at(j) {
                if !(rr.nx == 0 as u_int) {
                    tty_draw_line(
                        tty,
                        s,
                        ctx.ocx.wrapping_add(i).wrapping_add(rr.px).wrapping_sub(x),
                        ctx.ocy,
                        rr.nx,
                        rr.px,
                        ry,
                        &ctx.defaults,
                        ctx.palette.as_ref(),
                    );
                }
                j = j.wrapping_add(1);
            }
        }
    }
}
pub unsafe fn tty_check_codeset(tty: &mut tty, gc: &grid_cell) -> grid_cell {
    unsafe {
        let mut new: grid_cell;
        if gc.data.size as core::ffi::c_int == 1 as core::ffi::c_int
            && (gc.data.data[0] as core::ffi::c_int) < 0x7f as core::ffi::c_int
        {
            return *gc;
        }
        if gc.flags as core::ffi::c_int & GRID_FLAG_TAB != 0 {
            return *gc;
        }
        if tty_client(tty).expect("the tty has a client").flags() & CLIENT_UTF8 as uint64_t != 0 {
            return *gc;
        }
        new = *gc;
        if let Some(c) =
            RustAlternateCharacterSet.key_for_unicode(&gc.data.data[..gc.data.size as usize])
        {
            utf8_set(&mut new.data, c);
            new.attr = (new.attr as core::ffi::c_int | GRID_ATTR_CHARSET) as u_short;
            return new;
        }
        new.data.size = gc.data.width;
        if new.data.size as core::ffi::c_int > UTF8_SIZE {
            new.data.size = UTF8_SIZE as u_char;
        }
        new.data.data[..new.data.size as usize].fill(b'_');
        new
    }
}
unsafe fn tty_check_overlay(tty: &mut tty, px: u_int, py: u_int) -> core::ffi::c_int {
    unsafe {
        let r = tty_check_overlay_range(tty, px, py, 1 as u_int);
        (!r.is_empty()) as core::ffi::c_int
    }
}
pub unsafe fn tty_check_overlay_range(
    tty: &mut tty,
    px: u_int,
    py: u_int,
    nx: u_int,
) -> VisibleRangesRef {
    unsafe {
        let mut held = tty_client(tty).expect("the tty has a client");
        if let Some(ranges) = held.overlay_ranges(px, py, nx) {
            ranges
        } else {
            let mut ranges = tty.r.borrow_mut();
            ranges.ensure_capacity(1);
            ranges.ranges[0] = visible_range { px, nx };
            ranges.used = 1;
            tty.r.clone()
        }
    }
}

pub unsafe fn tty_sync_start(tty: &mut tty) {
    unsafe {
        if tty.flags & TTY_BLOCK != 0 {
            return;
        }
        if tty.flags & TTY_SYNCING != 0 {
            return;
        }
        tty.flags |= TTY_SYNCING;
        if tty_term_of(tty).has(TTYC_SYNC) {
            log_debug(
                c"%s sync start",
                fmt_args![tty_client(tty).expect("the tty has a client").name()],
            );
            tty_putcode_i(tty, TTYC_SYNC, 1 as core::ffi::c_int);
        }
    }
}
pub unsafe fn tty_sync_end(tty: &mut tty) {
    unsafe {
        if tty.flags & TTY_BLOCK != 0 {
            return;
        }
        if !tty.flags & TTY_SYNCING != 0 {
            return;
        }
        tty.flags &= !TTY_SYNCING;
        if tty_term_of(tty).has(TTYC_SYNC) {
            log_debug(
                c"%s sync end",
                fmt_args![tty_client(tty).expect("the tty has a client").name()],
            );
            tty_putcode_i(tty, TTYC_SYNC, 2 as core::ffi::c_int);
        }
    }
}
fn tty_client_ready(ctx: &tty_ctx, c: &client) -> core::ffi::c_int {
    {
        if c.attached_session().is_none() || c.tty.term.is_none() {
            return 0 as core::ffi::c_int;
        }
        if c.flags & CLIENT_SUSPENDED as uint64_t != 0 {
            return 0 as core::ffi::c_int;
        }
        if ctx.flags & TTY_CTX_INVISIBLE_PANES != 0 {
            return 1 as core::ffi::c_int;
        }
        if c.flags & CLIENT_REDRAWWINDOW as uint64_t != 0 {
            return 0 as core::ffi::c_int;
        }
        if c.tty.flags & TTY_FREEZE != 0 {
            return 0 as core::ffi::c_int;
        }
        1 as core::ffi::c_int
    }
}
pub unsafe fn tty_write(mut command: impl FnMut(&mut tty, &tty_ctx), ctx: &mut tty_ctx) {
    unsafe {
        let mut state: core::ffi::c_int;
        if ctx.set_client_cb.is_none() {
            return;
        }
        for mut c in client_walk() {
            if tty_client_ready(ctx, c.as_client()) != 0 {
                state =
                    ctx.set_client_cb.expect("non-null function pointer")(ctx, c.as_client_mut());
                if state == -(1 as core::ffi::c_int) {
                    break;
                }
                if !(state == 0 as core::ffi::c_int) {
                    command(c.as_tty_mut(), ctx);
                }
            }
        }
    }
}
/// The count a terminal command carries. Only asked of the commands whose
/// context was built with one.
fn tty_ctx_num(ctx: &tty_ctx) -> u_int {
    {
        match ctx.value {
            TtyCtxValue::Num(n) => n,
            _ => fatalx(c"terminal command has no count", fmt_args![]),
        }
    }
}

/// The bytes a terminal command carries.
fn tty_ctx_bytes<'a>(ctx: &tty_ctx<'a>) -> tty_ctx_data<'a> {
    {
        match ctx.value {
            TtyCtxValue::Data(data) => data,
            _ => fatalx(c"terminal command has no data", fmt_args![]),
        }
    }
}

/// The selection a terminal command carries.
fn tty_ctx_sel_of<'a>(ctx: &tty_ctx<'a>) -> tty_ctx_sel<'a> {
    {
        match ctx.value {
            TtyCtxValue::Sel(sel) => sel,
            _ => fatalx(c"terminal command has no selection", fmt_args![]),
        }
    }
}

pub unsafe fn tty_cmd_insertcharacter(tty: &mut tty, ctx: &tty_ctx, s: &RustScreen) {
    unsafe {
        let c = tty_client(tty);
        if ctx.flags & TTY_CTX_WINDOW_BIGGER != 0
            || !(ctx.xoff == 0 as core::ffi::c_int && ctx.sx >= tty.sx)
            || tty_fake_bce(tty, &ctx.defaults, ctx.bg) != 0
            || !tty_term_of(tty).has(TTYC_ICH) && !tty_term_of(tty).has(TTYC_ICH1)
            || c.as_ref()
                .expect("the tty has a client")
                .has_overlay_check()
        {
            tty_draw_pane(tty, ctx, ctx.ocy, s);
            return;
        }
        tty_default_attributes(
            tty,
            &ctx.defaults,
            ctx.palette.as_ref(),
            ctx.bg,
            Some(&s.hyperlinks()),
        );
        tty_cursor_pane(tty, ctx, ctx.ocx, ctx.ocy);
        tty_emulate_repeat(tty, TTYC_ICH, TTYC_ICH1, tty_ctx_num(ctx));
    }
}
pub unsafe fn tty_cmd_deletecharacter(tty: &mut tty, ctx: &tty_ctx, s: &RustScreen) {
    unsafe {
        let c = tty_client(tty);
        if ctx.flags & TTY_CTX_WINDOW_BIGGER != 0
            || !(ctx.xoff == 0 as core::ffi::c_int && ctx.sx >= tty.sx)
            || tty_fake_bce(tty, &ctx.defaults, ctx.bg) != 0
            || !tty_term_of(tty).has(TTYC_DCH) && !tty_term_of(tty).has(TTYC_DCH1)
            || c.as_ref()
                .expect("the tty has a client")
                .has_overlay_check()
        {
            tty_draw_pane(tty, ctx, ctx.ocy, s);
            return;
        }
        tty_default_attributes(
            tty,
            &ctx.defaults,
            ctx.palette.as_ref(),
            ctx.bg,
            Some(&s.hyperlinks()),
        );
        tty_cursor_pane(tty, ctx, ctx.ocx, ctx.ocy);
        tty_emulate_repeat(tty, TTYC_DCH, TTYC_DCH1, tty_ctx_num(ctx));
    }
}
pub unsafe fn tty_cmd_clearcharacter(tty: &mut tty, ctx: &tty_ctx, s: &RustScreen) {
    unsafe {
        tty_default_attributes(
            tty,
            &ctx.defaults,
            ctx.palette.as_ref(),
            ctx.bg,
            Some(&s.hyperlinks()),
        );
        tty_clear_pane_line(tty, ctx, ctx.ocy, ctx.ocx, tty_ctx_num(ctx), ctx.bg);
    }
}
pub unsafe fn tty_cmd_insertline(tty: &mut tty, ctx: &tty_ctx, s: &RustScreen) {
    unsafe {
        let c = tty_client(tty);
        if ctx.flags & TTY_CTX_WINDOW_BIGGER != 0
            || !(ctx.xoff == 0 as core::ffi::c_int && ctx.sx >= tty.sx)
            || tty_fake_bce(tty, &ctx.defaults, ctx.bg) != 0
            || !tty_term_of(tty).has(TTYC_CSR)
            || !tty_term_of(tty).has(TTYC_IL1)
            || ctx.sx == 1 as u_int
            || ctx.sy == 1 as u_int
            || c.as_ref()
                .expect("the tty has a client")
                .has_overlay_check()
        {
            tty_redraw_region(tty, ctx, s);
            return;
        }
        tty_default_attributes(
            tty,
            &ctx.defaults,
            ctx.palette.as_ref(),
            ctx.bg,
            Some(&s.hyperlinks()),
        );
        tty_region_pane(tty, ctx, ctx.orupper, ctx.orlower);
        tty_margin_off(tty);
        tty_cursor_pane(tty, ctx, ctx.ocx, ctx.ocy);
        tty_emulate_repeat(tty, TTYC_IL, TTYC_IL1, tty_ctx_num(ctx));
        tty.cy = UINT_MAX as u_int;
        tty.cx = tty.cy;
    }
}
pub unsafe fn tty_cmd_deleteline(tty: &mut tty, ctx: &tty_ctx, s: &RustScreen) {
    unsafe {
        let c = tty_client(tty);
        if ctx.flags & TTY_CTX_WINDOW_BIGGER != 0
            || !(ctx.xoff == 0 as core::ffi::c_int && ctx.sx >= tty.sx)
            || tty_fake_bce(tty, &ctx.defaults, ctx.bg) != 0
            || !tty_term_of(tty).has(TTYC_CSR)
            || !tty_term_of(tty).has(TTYC_DL1)
            || ctx.sx == 1 as u_int
            || ctx.sy == 1 as u_int
            || c.as_ref()
                .expect("the tty has a client")
                .has_overlay_check()
        {
            tty_redraw_region(tty, ctx, s);
            return;
        }
        tty_default_attributes(
            tty,
            &ctx.defaults,
            ctx.palette.as_ref(),
            ctx.bg,
            Some(&s.hyperlinks()),
        );
        tty_region_pane(tty, ctx, ctx.orupper, ctx.orlower);
        tty_margin_off(tty);
        tty_cursor_pane(tty, ctx, ctx.ocx, ctx.ocy);
        tty_emulate_repeat(tty, TTYC_DL, TTYC_DL1, tty_ctx_num(ctx));
        tty.cy = UINT_MAX as u_int;
        tty.cx = tty.cy;
    }
}
pub unsafe fn tty_cmd_reverseindex(tty: &mut tty, ctx: &tty_ctx, s: &RustScreen) {
    unsafe {
        let c = tty_client(tty);
        if ctx.ocy != ctx.orupper {
            return;
        }
        if ctx.flags & TTY_CTX_WINDOW_BIGGER != 0
            || !(ctx.xoff == 0 as core::ffi::c_int && ctx.sx >= tty.sx)
                && tty_term_of(tty).flags() & TERM_DECSLRM == 0
            || tty_fake_bce(tty, &ctx.defaults, 8 as u_int) != 0
            || !tty_term_of(tty).has(TTYC_CSR)
            || !tty_term_of(tty).has(TTYC_RI) && !tty_term_of(tty).has(TTYC_RIN)
            || ctx.sx == 1 as u_int
            || ctx.sy == 1 as u_int
            || c.as_ref()
                .expect("the tty has a client")
                .has_overlay_check()
        {
            tty_redraw_region(tty, ctx, s);
            return;
        }
        tty_default_attributes(
            tty,
            &ctx.defaults,
            ctx.palette.as_ref(),
            ctx.bg,
            Some(&s.hyperlinks()),
        );
        tty_region_pane(tty, ctx, ctx.orupper, ctx.orlower);
        tty_margin_pane(tty, ctx);
        tty_cursor_pane(tty, ctx, ctx.ocx, ctx.orupper);
        if tty_term_of(tty).has(TTYC_RI) {
            tty_putcode(tty, TTYC_RI);
        } else {
            tty_putcode_i(tty, TTYC_RIN, 1 as core::ffi::c_int);
        };
    }
}
pub unsafe fn tty_cmd_scrollup(tty: &mut tty, ctx: &tty_ctx, s: &RustScreen) {
    unsafe {
        let c = tty_client(tty);
        let mut i: u_int;
        if ctx.flags & TTY_CTX_WINDOW_BIGGER != 0
            || !(ctx.xoff == 0 as core::ffi::c_int && ctx.sx >= tty.sx)
                && tty_term_of(tty).flags() & TERM_DECSLRM == 0
            || tty_fake_bce(tty, &ctx.defaults, 8 as u_int) != 0
            || !tty_term_of(tty).has(TTYC_CSR)
            || ctx.sx == 1 as u_int
            || ctx.sy == 1 as u_int
            || c.as_ref()
                .expect("the tty has a client")
                .has_overlay_check()
        {
            tty_redraw_region(tty, ctx, s);
            return;
        }
        tty_default_attributes(
            tty,
            &ctx.defaults,
            ctx.palette.as_ref(),
            ctx.bg,
            Some(&s.hyperlinks()),
        );
        tty_region_pane(tty, ctx, ctx.orupper, ctx.orlower);
        tty_margin_pane(tty, ctx);
        if tty_ctx_num(ctx) == 1 as u_int || !tty_term_of(tty).has(TTYC_INDN) {
            if tty_term_of(tty).flags() & TERM_DECSLRM == 0 {
                tty_cursor(tty, 0 as u_int, tty.rlower);
            } else {
                tty_cursor(tty, tty.rright, tty.rlower);
            }
            i = 0 as u_int;
            while i < tty_ctx_num(ctx) {
                tty_putc(tty, '\n' as i32 as u_char);
                i = i.wrapping_add(1);
            }
        } else {
            if tty.cy == UINT_MAX {
                tty_cursor(tty, 0 as u_int, 0 as u_int);
            } else {
                tty_cursor(tty, 0 as u_int, tty.cy);
            }
            tty_putcode_i(tty, TTYC_INDN, tty_ctx_num(ctx) as core::ffi::c_int);
        };
    }
}
pub unsafe fn tty_cmd_scrolldown(tty: &mut tty, ctx: &tty_ctx, s: &RustScreen) {
    unsafe {
        let mut i: u_int;
        let c = tty_client(tty);
        if ctx.flags & TTY_CTX_WINDOW_BIGGER != 0
            || !(ctx.xoff == 0 as core::ffi::c_int && ctx.sx >= tty.sx)
                && tty_term_of(tty).flags() & TERM_DECSLRM == 0
            || tty_fake_bce(tty, &ctx.defaults, 8 as u_int) != 0
            || !tty_term_of(tty).has(TTYC_CSR)
            || !tty_term_of(tty).has(TTYC_RI) && !tty_term_of(tty).has(TTYC_RIN)
            || ctx.sx == 1 as u_int
            || ctx.sy == 1 as u_int
            || c.as_ref()
                .expect("the tty has a client")
                .has_overlay_check()
        {
            tty_redraw_region(tty, ctx, s);
            return;
        }
        tty_default_attributes(
            tty,
            &ctx.defaults,
            ctx.palette.as_ref(),
            ctx.bg,
            Some(&s.hyperlinks()),
        );
        tty_region_pane(tty, ctx, ctx.orupper, ctx.orlower);
        tty_margin_pane(tty, ctx);
        tty_cursor_pane(tty, ctx, ctx.ocx, ctx.orupper);
        if tty_term_of(tty).has(TTYC_RIN) {
            tty_putcode_i(tty, TTYC_RIN, tty_ctx_num(ctx) as core::ffi::c_int);
        } else {
            i = 0 as u_int;
            while i < tty_ctx_num(ctx) {
                tty_putcode(tty, TTYC_RI);
                i = i.wrapping_add(1);
            }
        };
    }
}
pub unsafe fn tty_cmd_clearendofscreen(tty: &mut tty, ctx: &tty_ctx, s: &RustScreen) {
    unsafe {
        let mut px: u_int;
        let mut py: u_int;
        let mut nx: u_int;

        tty_default_attributes(
            tty,
            &ctx.defaults,
            ctx.palette.as_ref(),
            ctx.bg,
            Some(&s.hyperlinks()),
        );
        tty_region_pane(tty, ctx, 0 as u_int, ctx.sy.wrapping_sub(1 as u_int));
        tty_margin_off(tty);
        px = 0 as u_int;
        nx = ctx.sx;
        py = ctx.ocy.wrapping_add(1 as u_int);
        let ny: u_int = ctx.sy.wrapping_sub(ctx.ocy).wrapping_sub(1 as u_int);
        tty_clear_pane_area(tty, ctx, py, ny, px, nx, ctx.bg);
        px = ctx.ocx;
        nx = ctx.sx.wrapping_sub(ctx.ocx);
        py = ctx.ocy;
        tty_clear_pane_line(tty, ctx, py, px, nx, ctx.bg);
    }
}
pub unsafe fn tty_cmd_clearstartofscreen(tty: &mut tty, ctx: &tty_ctx, s: &RustScreen) {
    unsafe {
        let mut px: u_int;
        let mut py: u_int;
        let mut nx: u_int;

        tty_default_attributes(
            tty,
            &ctx.defaults,
            ctx.palette.as_ref(),
            ctx.bg,
            Some(&s.hyperlinks()),
        );
        tty_region_pane(tty, ctx, 0 as u_int, ctx.sy.wrapping_sub(1 as u_int));
        tty_margin_off(tty);
        px = 0 as u_int;
        nx = ctx.sx;
        py = 0 as u_int;
        let ny: u_int = ctx.ocy;
        tty_clear_pane_area(tty, ctx, py, ny, px, nx, ctx.bg);
        px = 0 as u_int;
        nx = ctx.ocx.wrapping_add(1 as u_int);
        py = ctx.ocy;
        tty_clear_pane_line(tty, ctx, py, px, nx, ctx.bg);
    }
}
pub unsafe fn tty_cmd_clearscreen(tty: &mut tty, ctx: &tty_ctx, s: &RustScreen) {
    unsafe {
        tty_default_attributes(
            tty,
            &ctx.defaults,
            ctx.palette.as_ref(),
            ctx.bg,
            Some(&s.hyperlinks()),
        );
        tty_region_pane(tty, ctx, 0 as u_int, ctx.sy.wrapping_sub(1 as u_int));
        tty_margin_off(tty);
        let px: u_int = 0 as u_int;
        let nx: u_int = ctx.sx;
        let py: u_int = 0 as u_int;
        let ny: u_int = ctx.sy;
        tty_clear_pane_area(tty, ctx, py, ny, px, nx, ctx.bg);
    }
}
pub unsafe fn tty_cmd_alignmenttest(tty: &mut tty, ctx: &tty_ctx, s: &RustScreen) {
    unsafe {
        let c = tty_client(tty);
        let mut i: u_int;
        let mut j: u_int;
        if ctx.flags & TTY_CTX_WINDOW_BIGGER != 0
            || c.as_ref()
                .expect("the tty has a client")
                .has_overlay_check()
        {
            ctx.redraw_cb.expect("non-null function pointer")(ctx);
            return;
        }
        tty_attributes(
            tty,
            &grid_default_cell,
            &ctx.defaults,
            ctx.palette.as_ref(),
            Some(&s.hyperlinks()),
        );
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
}
pub unsafe fn tty_cmd_cell(tty: &mut tty, ctx: &tty_ctx, s: &RustScreen) {
    unsafe {
        let mut i: u_int;
        let mut vis: u_int = 0 as u_int;
        let px: u_int = (ctx.xoff as u_int)
            .wrapping_add(ctx.ocx)
            .wrapping_sub(ctx.wox);
        let py: u_int = (ctx.yoff as u_int)
            .wrapping_add(ctx.ocy)
            .wrapping_sub(ctx.woy);
        if tty_is_visible(ctx, ctx.ocx, ctx.ocy, 1 as u_int, 1 as u_int) == 0 {
            return;
        }
        let gc = ctx.cell.as_ref().expect("a cell command carries a cell");
        if gc.data.width as core::ffi::c_int == 1 as core::ffi::c_int
            && tty_check_overlay(tty, px, py) == 0
        {
            return;
        }
        if gc.data.width as core::ffi::c_int > 1 as core::ffi::c_int {
            let r = tty_check_overlay_range(tty, px, py, gc.data.width as u_int);
            i = 0 as u_int;
            while let Some(range) = r.range_at(i) {
                vis = vis.wrapping_add(range.nx);
                i = i.wrapping_add(1);
            }
            if vis < gc.data.width as u_int {
                let (screen_cx, screen_cy) = s.cursor();
                tty_draw_line(
                    tty,
                    s,
                    screen_cx,
                    screen_cy,
                    gc.data.width as u_int,
                    px,
                    py,
                    &ctx.defaults,
                    ctx.palette.as_ref(),
                );
                return;
            }
        }
        if (ctx.xoff as u_int)
            .wrapping_add(ctx.ocx)
            .wrapping_sub(ctx.wox)
            > tty.sx.wrapping_sub(1 as u_int)
            && ctx.ocy == ctx.orlower
            && (ctx.xoff == 0 as core::ffi::c_int && ctx.sx >= tty.sx)
        {
            tty_region_pane(tty, ctx, ctx.orupper, ctx.orlower);
        }
        tty_margin_off(tty);
        if ctx.flags & TTY_CTX_CELL_INVALIDATE != 0 {
            tty_invalidate(tty);
        }
        tty_cursor_pane_unless_wrap(tty, ctx, ctx.ocx, ctx.ocy);
        tty_cell(
            tty,
            gc,
            &ctx.defaults,
            ctx.palette.as_ref(),
            Some(&s.hyperlinks()),
        );
        if ctx.flags & TTY_CTX_CELL_INVALIDATE != 0 {
            tty_invalidate(tty);
        }
    }
}
pub unsafe fn tty_cmd_cells(tty: &mut tty, ctx: &tty_ctx, s: &RustScreen) {
    unsafe {
        let mut i: u_int;

        let mut cx: u_int;
        let data = tty_ctx_bytes(ctx);
        let cells = data.data;
        let n: size_t = cells.len();
        if tty_is_visible(ctx, ctx.ocx, ctx.ocy, n as u_int, 1 as u_int) == 0 {
            return;
        }
        if ctx.flags & TTY_CTX_WINDOW_BIGGER != 0
            && ((ctx.xoff as u_int).wrapping_add(ctx.ocx) < ctx.wox
                || ((ctx.xoff as u_int).wrapping_add(ctx.ocx) as size_t).wrapping_add(n)
                    > ctx.wox.wrapping_add(ctx.wsx) as size_t)
        {
            if !ctx.flags & TTY_CTX_WRAPPED != 0
                || !(ctx.xoff == 0 as core::ffi::c_int && ctx.sx >= tty.sx)
                || tty_term_of(tty).flags() & TERM_NOAM != 0
                || (ctx.xoff as u_int).wrapping_add(ctx.ocx) != 0 as u_int
                || (ctx.yoff as u_int).wrapping_add(ctx.ocy) != tty.cy.wrapping_add(1 as u_int)
                || tty.cx < tty.sx
                || tty.cy == tty.rlower
            {
                tty_draw_pane(tty, ctx, ctx.ocy, s);
            } else {
                ctx.redraw_cb.expect("non-null function pointer")(ctx);
            }
            return;
        }
        tty_margin_off(tty);
        tty_cursor_pane_unless_wrap(tty, ctx, ctx.ocx, ctx.ocy);
        tty_attributes(
            tty,
            ctx.cell.as_ref().expect("a text command carries a cell"),
            &ctx.defaults,
            ctx.palette.as_ref(),
            Some(&s.hyperlinks()),
        );
        let px: u_int = (ctx.xoff as u_int)
            .wrapping_add(ctx.ocx)
            .wrapping_sub(ctx.wox);
        let py: u_int = (ctx.yoff as u_int)
            .wrapping_add(ctx.ocy)
            .wrapping_sub(ctx.woy);
        let r = tty_check_overlay_range(tty, px, py, n as u_int);
        i = 0 as u_int;
        while let Some(ri) = r.range_at(i) {
            if ri.nx != 0 as u_int {
                cx = ri.px.wrapping_sub(ctx.xoff as u_int).wrapping_add(ctx.wox);
                tty_cursor_pane_unless_wrap(tty, ctx, cx, ctx.ocy);
                let at = ri.px.wrapping_sub(px) as usize;
                tty_putn(tty, &cells[at..at + ri.nx as usize], ri.nx);
            }
            i = i.wrapping_add(1);
        }
    }
}
pub unsafe fn tty_cmd_setselection(tty: &mut tty, ctx: &tty_ctx) {
    unsafe {
        let sel = tty_ctx_sel_of(ctx);
        tty_set_selection(tty, sel.clip, sel.data);
    }
}
pub unsafe fn tty_set_selection(tty: &mut tty, clip: &CStr, buf: &[u8]) {
    unsafe {
        let len = buf.len() as size_t;
        if !tty.flags & TTY_STARTED != 0 {
            return;
        }
        if !tty_term_of(tty).has(TTYC_MS) {
            return;
        }
        let size: size_t = (4 as size_t)
            .wrapping_mul(len.wrapping_add(2 as size_t).wrapping_div(3 as size_t))
            .wrapping_add(1 as size_t);
        let mut encoded: Vec<u8> = vec![0_u8; size as usize];
        __b64_ntop(
            buf.as_ptr(),
            len,
            encoded.as_mut_ptr() as *mut core::ffi::c_char,
            size,
        );
        tty.flags |= TTY_NOBLOCK;
        tty_putcode_ss(
            tty,
            TTYC_MS,
            clip,
            core::ffi::CStr::from_ptr(encoded.as_ptr() as *const core::ffi::c_char),
        );
    }
}
pub unsafe fn tty_cmd_rawstring(tty: &mut tty, ctx: &tty_ctx) {
    unsafe {
        tty.flags |= TTY_NOBLOCK;
        let data = tty_ctx_bytes(ctx);
        tty_add(tty, data.data);
        tty_invalidate(tty);
    }
}
pub unsafe fn tty_cmd_syncstart(tty: &mut tty, ctx: &tty_ctx) {
    unsafe {
        let c = tty_client(tty);
        if (ctx.flags & TTY_CTX_OVERLAY_SYNC != 0 && ctx.flags & TTY_CTX_SYNC != 0)
            || (!ctx.flags & TTY_CTX_OVERLAY_SYNC != 0
                && (ctx.flags & TTY_CTX_SYNC != 0
                    || c.as_ref().expect("the tty has a client").has_overlay()))
        {
            tty_sync_start(tty);
        }
    }
}
pub(crate) unsafe fn tty_cell(
    tty: &mut tty,
    gc: &grid_cell,
    defaults: &grid_cell,
    palette: Option<&colour_palette>,
    hl: Option<&RustHyperlinks>,
) {
    unsafe {
        if tty_term_of(tty).flags() & TERM_NOAM != 0
            && tty.cy == tty.sy.wrapping_sub(1 as u_int)
            && tty.cx == tty.sx.wrapping_sub(1 as u_int)
        {
            return;
        }
        if gc.flags as core::ffi::c_int & GRID_FLAG_PADDING != 0 {
            return;
        }
        if tty_check_overlay(tty, tty.cx, tty.cy) == 0 {
            return;
        }
        let checked = tty_check_codeset(tty, gc);
        tty_attributes(tty, &checked, defaults, palette, hl);
        if checked.data.size as core::ffi::c_int == 1 as core::ffi::c_int {
            if (checked.data.data[0] as core::ffi::c_int) < 0x20 as core::ffi::c_int
                || checked.data.data[0] as core::ffi::c_int == 0x7f as core::ffi::c_int
            {
                return;
            }
            tty_putc(tty, checked.data.data[0]);
            return;
        }
        tty_putn(
            tty,
            &checked.data.data[..checked.data.size as usize],
            checked.data.width as u_int,
        );
    }
}
pub unsafe fn tty_reset(tty: &mut tty) {
    unsafe {
        if grid_cells_equal(&tty.cell, &grid_default_cell) == 0 {
            if tty.cell.link != 0 as u_int {
                tty_putcode_ss(tty, TTYC_HLS, c"", c"");
            }
            if tty.cell.attr as core::ffi::c_int & GRID_ATTR_CHARSET != 0
                && tty_acs_needed(Some(tty)) != 0
            {
                tty_putcode(tty, TTYC_RMACS);
            }
            tty_putcode(tty, TTYC_SGR0);
            tty.cell = grid_default_cell;
        }
        tty.last_cell = grid_default_cell;
    }
}
pub unsafe fn tty_invalidate(tty: &mut tty) {
    unsafe {
        tty.cell = grid_default_cell;
        tty.last_cell = grid_default_cell;
        tty.cy = UINT_MAX as u_int;
        tty.cx = tty.cy;
        tty.rleft = UINT_MAX as u_int;
        tty.rupper = tty.rleft;
        tty.rright = UINT_MAX as u_int;
        tty.rlower = tty.rright;
        if tty.flags & TTY_STARTED != 0 {
            if tty_term_of(tty).flags() & TERM_DECSLRM != 0 {
                tty_putcode(tty, TTYC_ENMG);
            }
            tty_putcode(tty, TTYC_SGR0);
            tty.mode = ALL_MODES;
            tty_update_mode(tty, MODE_CURSOR, None);
            tty_cursor(tty, 0 as u_int, 0 as u_int);
            tty_region_off(tty);
            tty_margin_off(tty);
        } else {
            tty.mode = MODE_CURSOR;
        };
    }
}
pub unsafe fn tty_region_off(tty: &mut tty) {
    unsafe {
        tty_region(tty, 0 as u_int, tty.sy.wrapping_sub(1 as u_int));
    }
}
unsafe fn tty_region_pane(tty: &mut tty, ctx: &tty_ctx, rupper: u_int, rlower: u_int) {
    unsafe {
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
}
unsafe fn tty_region(tty: &mut tty, rupper: u_int, rlower: u_int) {
    unsafe {
        if tty.rlower == rlower && tty.rupper == rupper {
            return;
        }
        if !tty_term_of(tty).has(TTYC_CSR) {
            return;
        }
        tty.rupper = rupper;
        tty.rlower = rlower;
        if tty.cx >= tty.sx {
            if tty.cy == UINT_MAX {
                tty_cursor(tty, 0 as u_int, 0 as u_int);
            } else {
                tty_cursor(tty, 0 as u_int, tty.cy);
            }
        }
        tty_putcode_ii(
            tty,
            TTYC_CSR,
            tty.rupper as core::ffi::c_int,
            tty.rlower as core::ffi::c_int,
        );
        tty.cy = UINT_MAX as u_int;
        tty.cx = tty.cy;
    }
}
pub fn tty_margin_off(tty: &mut tty) {
    {
        tty_margin(tty, 0 as u_int, tty.sx.wrapping_sub(1 as u_int));
    }
}
fn tty_margin_pane(tty: &mut tty, ctx: &tty_ctx) {
    {
        let mut l: core::ffi::c_int;
        let mut r: core::ffi::c_int;
        l = (ctx.xoff as u_int).wrapping_sub(ctx.wox) as core::ffi::c_int;
        r = (ctx.xoff as u_int)
            .wrapping_add(ctx.sx)
            .wrapping_sub(1 as u_int)
            .wrapping_sub(ctx.wox) as core::ffi::c_int;
        if l < 0 as core::ffi::c_int {
            l = 0 as core::ffi::c_int;
        }
        if l > ctx.wsx as core::ffi::c_int {
            l = ctx.wsx as core::ffi::c_int;
        }
        if r < 0 as core::ffi::c_int {
            r = 0 as core::ffi::c_int;
        }
        if r > ctx.wsx as core::ffi::c_int {
            r = ctx.wsx as core::ffi::c_int;
        }
        tty_margin(tty, l as u_int, r as u_int);
    }
}
fn tty_margin(tty: &mut tty, rleft: u_int, rright: u_int) {
    {
        if tty_term_of(tty).flags() & TERM_DECSLRM == 0 {
            return;
        }
        if tty.rleft == rleft && tty.rright == rright {
            return;
        }
        tty_putcode_ii(
            tty,
            TTYC_CSR,
            tty.rupper as core::ffi::c_int,
            tty.rlower as core::ffi::c_int,
        );
        tty.rleft = rleft;
        tty.rright = rright;
        if rleft == 0 as u_int && rright == tty.sx.wrapping_sub(1 as u_int) {
            tty_putcode(tty, TTYC_CLMG);
        } else {
            tty_putcode_ii(
                tty,
                TTYC_CMG,
                rleft as core::ffi::c_int,
                rright as core::ffi::c_int,
            );
        }
        tty.cy = UINT_MAX as u_int;
        tty.cx = tty.cy;
    }
}
unsafe fn tty_cursor_pane_unless_wrap(tty: &mut tty, ctx: &tty_ctx, cx: u_int, cy: u_int) {
    unsafe {
        if !ctx.flags & TTY_CTX_WRAPPED != 0
            || !(ctx.xoff == 0 as core::ffi::c_int && ctx.sx >= tty.sx)
            || tty_term_of(tty).flags() & TERM_NOAM != 0
            || (ctx.xoff as u_int).wrapping_add(cx) != 0 as u_int
            || (ctx.yoff as u_int).wrapping_add(cy) != tty.cy.wrapping_add(1 as u_int)
            || tty.cx < tty.sx
            || tty.cy == tty.rlower
        {
            tty_cursor_pane(tty, ctx, cx, cy);
        } else {
            log_debug(
                c"%s: will wrap at %u,%u",
                fmt_args![c"tty_cursor_pane_unless_wrap".as_ptr(), tty.cx, tty.cy],
            );
        };
    }
}
unsafe fn tty_cursor_pane(tty: &mut tty, ctx: &tty_ctx, cx: u_int, cy: u_int) {
    unsafe {
        tty_cursor(
            tty,
            (ctx.xoff as u_int).wrapping_add(cx).wrapping_sub(ctx.wox),
            (ctx.yoff as u_int).wrapping_add(cy).wrapping_sub(ctx.woy),
        );
    }
}
pub unsafe fn tty_cursor(tty: &mut tty, mut cx: u_int, cy: u_int) {
    unsafe {
        let current_block: u64;
        let terminal = tty
            .term
            .as_ref()
            .expect("a tty being driven has a terminal")
            .clone();
        let term = terminal.borrow();

        let change: core::ffi::c_int;
        if tty.flags & TTY_BLOCK != 0 {
            return;
        }
        let thisx: u_int = tty.cx;
        let thisy: u_int = tty.cy;
        if cx == thisx && cy == thisy && cx == tty.sx {
            return;
        }
        if cx > tty.sx.wrapping_sub(1 as u_int) {
            log_debug(
                c"%s: x too big %u > %u",
                fmt_args![c"tty_cursor".as_ptr(), cx, tty.sx.wrapping_sub(1 as u_int)],
            );
            cx = tty.sx.wrapping_sub(1 as u_int);
        }
        if cx == thisx && cy == thisy {
            return;
        }
        if thisx > tty.sx.wrapping_sub(1 as u_int) {
            current_block = 415867615225958941;
        } else if cx == 0 as u_int && cy == 0 as u_int && term.has(TTYC_HOME) {
            tty_putcode(tty, TTYC_HOME);
            current_block = 6541453531065591362;
        } else if cx == 0 as u_int
            && cy == thisy.wrapping_add(1 as u_int)
            && thisy != tty.rlower
            && (tty_term_of(tty).flags() & TERM_DECSLRM == 0 || tty.rleft == 0 as u_int)
        {
            tty_putc(tty, '\r' as i32 as u_char);
            tty_putc(tty, '\n' as i32 as u_char);
            current_block = 6541453531065591362;
        } else if cy == thisy {
            if cx == 0 as u_int
                && (tty_term_of(tty).flags() & TERM_DECSLRM == 0 || tty.rleft == 0 as u_int)
            {
                tty_putc(tty, '\r' as i32 as u_char);
                current_block = 6541453531065591362;
            } else if cx == thisx.wrapping_sub(1 as u_int) && term.has(TTYC_CUB1) {
                tty_putcode(tty, TTYC_CUB1);
                current_block = 6541453531065591362;
            } else if cx == thisx.wrapping_add(1 as u_int) && term.has(TTYC_CUF1) {
                tty_putcode(tty, TTYC_CUF1);
                current_block = 6541453531065591362;
            } else {
                change = thisx.wrapping_sub(cx) as core::ffi::c_int;
                if abs(change) as u_int > cx && term.has(TTYC_HPA) {
                    tty_putcode_i(tty, TTYC_HPA, cx as core::ffi::c_int);
                    current_block = 6541453531065591362;
                } else if change > 0 as core::ffi::c_int
                    && term.has(TTYC_CUB)
                    && tty_term_of(tty).flags() & TERM_DECSLRM == 0
                {
                    if change == 2 as core::ffi::c_int && term.has(TTYC_CUB1) {
                        tty_putcode(tty, TTYC_CUB1);
                        tty_putcode(tty, TTYC_CUB1);
                    } else {
                        tty_putcode_i(tty, TTYC_CUB, change);
                    }
                    current_block = 6541453531065591362;
                } else if change < 0 as core::ffi::c_int
                    && term.has(TTYC_CUF)
                    && tty_term_of(tty).flags() & TERM_DECSLRM == 0
                {
                    tty_putcode_i(tty, TTYC_CUF, -change);
                    current_block = 6541453531065591362;
                } else {
                    current_block = 415867615225958941;
                }
            }
        } else if cx == thisx {
            if thisy != tty.rupper && cy == thisy.wrapping_sub(1 as u_int) && term.has(TTYC_CUU1) {
                tty_putcode(tty, TTYC_CUU1);
                current_block = 6541453531065591362;
            } else if thisy != tty.rlower
                && cy == thisy.wrapping_add(1 as u_int)
                && term.has(TTYC_CUD1)
            {
                tty_putcode(tty, TTYC_CUD1);
                current_block = 6541453531065591362;
            } else {
                change = thisy.wrapping_sub(cy) as core::ffi::c_int;
                if abs(change) as u_int > cy
                    || change < 0 as core::ffi::c_int
                        && cy.wrapping_sub(change as u_int) > tty.rlower
                    || change > 0 as core::ffi::c_int
                        && cy.wrapping_sub(change as u_int) < tty.rupper
                {
                    if term.has(TTYC_VPA) {
                        tty_putcode_i(tty, TTYC_VPA, cy as core::ffi::c_int);
                        current_block = 6541453531065591362;
                    } else {
                        current_block = 415867615225958941;
                    }
                } else if change > 0 as core::ffi::c_int && term.has(TTYC_CUU) {
                    tty_putcode_i(tty, TTYC_CUU, change);
                    current_block = 6541453531065591362;
                } else if change < 0 as core::ffi::c_int && term.has(TTYC_CUD) {
                    tty_putcode_i(tty, TTYC_CUD, -change);
                    current_block = 6541453531065591362;
                } else {
                    current_block = 415867615225958941;
                }
            }
        } else {
            current_block = 415867615225958941;
        }
        if current_block == 415867615225958941 {
            tty_putcode_ii(
                tty,
                TTYC_CUP,
                cy as core::ffi::c_int,
                cx as core::ffi::c_int,
            );
        }
        tty.cx = cx;
        tty.cy = cy;
    }
}
fn tty_hyperlink(tty: &mut tty, gc: &grid_cell, hl: Option<&RustHyperlinks>) {
    if gc.link == tty.cell.link {
        return;
    }
    tty.cell.link = gc.link;
    let Some(hl) = hl else {
        return;
    };
    let found = if gc.link == 0 as u_int {
        None
    } else {
        hl.get(gc.link)
    };
    match found {
        Some((uri, _, external_id)) => tty_putcode_ss(tty, TTYC_HLS, &external_id, &uri),
        None => tty_putcode_ss(tty, TTYC_HLS, c"", c""),
    };
}
pub(crate) unsafe fn tty_attributes(
    tty: &mut tty,
    gc: &grid_cell,
    defaults: &grid_cell,
    palette: Option<&colour_palette>,
    hl: Option<&RustHyperlinks>,
) {
    unsafe {
        let mut gc2;

        gc2 = *gc;
        if !(gc.flags as core::ffi::c_int) & GRID_FLAG_NOPALETTE != 0 {
            if gc2.fg == 8 as core::ffi::c_int {
                gc2.fg = defaults.fg;
            }
            if gc2.bg == 8 as core::ffi::c_int {
                gc2.bg = defaults.bg;
            }
        }
        if gc2.attr as core::ffi::c_int == tty.last_cell.attr as core::ffi::c_int
            && gc2.fg == tty.last_cell.fg
            && gc2.bg == tty.last_cell.bg
            && gc2.us == tty.last_cell.us
            && gc2.link == tty.last_cell.link
        {
            return;
        }
        if !tty_term_of(tty).has(TTYC_SETAB) {
            if gc2.attr as core::ffi::c_int & GRID_ATTR_REVERSE != 0 {
                if gc2.fg != 7 as core::ffi::c_int
                    && !(gc2.fg == 8 as core::ffi::c_int || gc2.fg == 9 as core::ffi::c_int)
                {
                    gc2.attr = (gc2.attr as core::ffi::c_int & !GRID_ATTR_REVERSE) as u_short;
                }
            } else if gc2.bg != 0 as core::ffi::c_int
                && !(gc2.bg == 8 as core::ffi::c_int || gc2.bg == 9 as core::ffi::c_int)
            {
                gc2.attr = (gc2.attr as core::ffi::c_int | GRID_ATTR_REVERSE) as u_short;
            }
        }
        tty_check_fg(tty, palette, &mut gc2);
        tty_check_bg(tty, palette, &mut gc2);
        tty_check_us(tty, palette, &mut gc2);
        if tty.cell.attr as core::ffi::c_int & !(gc2.attr as core::ffi::c_int) != 0
            || tty.cell.us != gc2.us && gc2.us == 0 as core::ffi::c_int
        {
            tty_reset(tty);
        }
        tty_colours(tty, &gc2);
        let changed: core::ffi::c_int =
            gc2.attr as core::ffi::c_int & !(tty.cell.attr as core::ffi::c_int);
        tty.cell.attr = gc2.attr;
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
                tty_putcode_i(tty, TTYC_SMULX, 2 as core::ffi::c_int);
            } else if changed & GRID_ATTR_UNDERSCORE_3 != 0 {
                tty_putcode_i(tty, TTYC_SMULX, 3 as core::ffi::c_int);
            } else if changed & GRID_ATTR_UNDERSCORE_4 != 0 {
                tty_putcode_i(tty, TTYC_SMULX, 4 as core::ffi::c_int);
            } else if changed & GRID_ATTR_UNDERSCORE_5 != 0 {
                tty_putcode_i(tty, TTYC_SMULX, 5 as core::ffi::c_int);
            }
        }
        if changed & GRID_ATTR_BLINK != 0 {
            tty_putcode(tty, TTYC_BLINK);
        }
        if changed & GRID_ATTR_REVERSE != 0 {
            if tty_term_of(tty).has(TTYC_REV) {
                tty_putcode(tty, TTYC_REV);
            } else if tty_term_of(tty).has(TTYC_SMSO) {
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
        if changed & GRID_ATTR_CHARSET != 0 && tty_acs_needed(Some(tty)) != 0 {
            tty_putcode(tty, TTYC_SMACS);
        }
        if let Some(hl) = hl {
            tty_hyperlink(tty, gc, Some(hl));
        }
        tty.last_cell = gc2;
    }
}
unsafe fn tty_colours(tty: &mut tty, gc: &grid_cell) {
    unsafe {
        if gc.fg == tty.cell.fg && gc.bg == tty.cell.bg && gc.us == tty.cell.us {
            return;
        }
        if gc.fg == 8 as core::ffi::c_int
            || gc.fg == 9 as core::ffi::c_int
            || (gc.bg == 8 as core::ffi::c_int || gc.bg == 9 as core::ffi::c_int)
        {
            if tty_term_of(tty).flag(TTYC_AX) == 0 {
                tty_reset(tty);
            } else {
                if (gc.fg == 8 as core::ffi::c_int || gc.fg == 9 as core::ffi::c_int)
                    && !(tty.cell.fg == 8 as core::ffi::c_int
                        || tty.cell.fg == 9 as core::ffi::c_int)
                {
                    tty_puts(tty, c"\x1B[39m");
                    tty.cell.fg = gc.fg;
                }
                if (gc.bg == 8 as core::ffi::c_int || gc.bg == 9 as core::ffi::c_int)
                    && !(tty.cell.bg == 8 as core::ffi::c_int
                        || tty.cell.bg == 9 as core::ffi::c_int)
                {
                    tty_puts(tty, c"\x1B[49m");
                    tty.cell.bg = gc.bg;
                }
            }
        }
        if !(gc.fg == 8 as core::ffi::c_int || gc.fg == 9 as core::ffi::c_int)
            && gc.fg != tty.cell.fg
        {
            tty_colours_fg(tty, gc);
        }
        if !(gc.bg == 8 as core::ffi::c_int || gc.bg == 9 as core::ffi::c_int)
            && gc.bg != tty.cell.bg
        {
            tty_colours_bg(tty, gc);
        }
        if gc.us != tty.cell.us {
            tty_colours_us(tty, gc);
        }
    }
}
fn tty_check_fg(tty: &mut tty, palette: Option<&colour_palette>, gc: &mut grid_cell) {
    let mut c: core::ffi::c_int;
    if !(gc.flags as core::ffi::c_int) & GRID_FLAG_NOPALETTE != 0 {
        c = gc.fg;
        if c < 8 as core::ffi::c_int
            && gc.attr as core::ffi::c_int & GRID_ATTR_BRIGHT != 0
            && !tty_term_of(tty).has(TTYC_NOBR)
        {
            c += 90 as core::ffi::c_int;
        }
        c = RustColourEngine.get_palette(palette, c);
        if c != -(1 as core::ffi::c_int) {
            gc.fg = c;
        }
    }
    if gc.fg & COLOUR_FLAG_RGB != 0 {
        if tty_term_of(tty).flags() & TERM_RGBCOLOURS != 0 {
            return;
        }
        let (r, g, b) = RustColourEngine.split_rgb(gc.fg);
        gc.fg = RustColourEngine.find_rgb(r, g, b);
    }
    let colours: u_int = if tty_term_of(tty).flags() & TERM_256COLOURS != 0 {
        256 as u_int
    } else {
        tty_term_of(tty).number(TTYC_COLORS) as u_int
    };
    if gc.fg & COLOUR_FLAG_256 != 0 {
        if colours >= 256 as u_int {
            return;
        }
        gc.fg = RustColourEngine.indexed_to_basic(gc.fg);
        if !gc.fg & 8 as core::ffi::c_int != 0 {
            return;
        }
        gc.fg &= 7 as core::ffi::c_int;
        if colours >= 16 as u_int {
            gc.fg += 90 as core::ffi::c_int;
        } else if gc.fg == 0 as core::ffi::c_int && gc.bg == 0 as core::ffi::c_int {
            gc.fg = 7 as core::ffi::c_int;
        } else if gc.fg == 7 as core::ffi::c_int && gc.bg == 7 as core::ffi::c_int {
            gc.fg = 0 as core::ffi::c_int;
        }
        return;
    }
    if gc.fg >= 90 as core::ffi::c_int && gc.fg <= 97 as core::ffi::c_int && colours < 16 as u_int {
        gc.fg -= 90 as core::ffi::c_int;
        gc.attr = (gc.attr as core::ffi::c_int | GRID_ATTR_BRIGHT) as u_short;
    }
}
fn tty_check_bg(tty: &mut tty, palette: Option<&colour_palette>, gc: &mut grid_cell) {
    let c: core::ffi::c_int;
    if !(gc.flags as core::ffi::c_int) & GRID_FLAG_NOPALETTE != 0 {
        c = RustColourEngine.get_palette(palette, gc.bg);
        if c != -(1 as core::ffi::c_int) {
            gc.bg = c;
        }
    }
    if gc.bg & COLOUR_FLAG_RGB != 0 {
        if tty_term_of(tty).flags() & TERM_RGBCOLOURS != 0 {
            return;
        }
        let (r, g, b) = RustColourEngine.split_rgb(gc.bg);
        gc.bg = RustColourEngine.find_rgb(r, g, b);
    }
    let colours: u_int = if tty_term_of(tty).flags() & TERM_256COLOURS != 0 {
        256 as u_int
    } else {
        tty_term_of(tty).number(TTYC_COLORS) as u_int
    };
    if gc.bg & COLOUR_FLAG_256 != 0 {
        if colours >= 256 as u_int {
            return;
        }
        gc.bg = RustColourEngine.indexed_to_basic(gc.bg);
        if !gc.bg & 8 as core::ffi::c_int != 0 {
            return;
        }
        gc.bg &= 7 as core::ffi::c_int;
        if colours >= 16 as u_int {
            gc.bg += 90 as core::ffi::c_int;
        }
        return;
    }
    if gc.bg >= 90 as core::ffi::c_int && gc.bg <= 97 as core::ffi::c_int && colours < 16 as u_int {
        gc.bg -= 90 as core::ffi::c_int;
    }
}
fn tty_check_us(tty: &mut tty, palette: Option<&colour_palette>, gc: &mut grid_cell) {
    let mut c: core::ffi::c_int;
    if !(gc.flags as core::ffi::c_int) & GRID_FLAG_NOPALETTE != 0 {
        c = RustColourEngine.get_palette(palette, gc.us);
        if c != -(1 as core::ffi::c_int) {
            gc.us = c;
        }
    }
    if !tty_term_of(tty).has(TTYC_SETULC1) {
        c = RustColourEngine.force_rgb(gc.us);
        if c == -(1 as core::ffi::c_int) {
            gc.us = 8 as core::ffi::c_int;
        } else {
            gc.us = c;
        }
    }
}
unsafe fn tty_colours_fg(tty: &mut tty, gc: &grid_cell) {
    unsafe {
        if tty.cell.fg >= 90 as core::ffi::c_int
            && tty.cell.bg <= 97 as core::ffi::c_int
            && (gc.fg < 90 as core::ffi::c_int || gc.fg > 97 as core::ffi::c_int)
        {
            tty_reset(tty);
        }
        if gc.fg & COLOUR_FLAG_RGB != 0 || gc.fg & COLOUR_FLAG_256 != 0 {
            if !(tty_try_colour(tty, gc.fg, c"38") == 0 as core::ffi::c_int) {
                return;
            }
        } else if gc.fg >= 90 as core::ffi::c_int && gc.fg <= 97 as core::ffi::c_int {
            if tty_term_of(tty).flags() & TERM_256COLOURS != 0 {
                let s = xasprintf(c"\x1B[%dm", fmt_args![gc.fg]);
                tty_puts(tty, &s);
            } else {
                tty_putcode_i(
                    tty,
                    TTYC_SETAF,
                    gc.fg - 90 as core::ffi::c_int + 8 as core::ffi::c_int,
                );
            }
        } else {
            tty_putcode_i(tty, TTYC_SETAF, gc.fg);
        }
        tty.cell.fg = gc.fg;
    }
}
fn tty_colours_bg(tty: &mut tty, gc: &grid_cell) {
    {
        if gc.bg & COLOUR_FLAG_RGB != 0 || gc.bg & COLOUR_FLAG_256 != 0 {
            if !(tty_try_colour(tty, gc.bg, c"48") == 0 as core::ffi::c_int) {
                return;
            }
        } else if gc.bg >= 90 as core::ffi::c_int && gc.bg <= 97 as core::ffi::c_int {
            if tty_term_of(tty).flags() & TERM_256COLOURS != 0 {
                let s = xasprintf(c"\x1B[%dm", fmt_args![gc.bg + 10 as core::ffi::c_int]);
                tty_puts(tty, &s);
            } else {
                tty_putcode_i(
                    tty,
                    TTYC_SETAB,
                    gc.bg - 90 as core::ffi::c_int + 8 as core::ffi::c_int,
                );
            }
        } else {
            tty_putcode_i(tty, TTYC_SETAB, gc.bg);
        }
        tty.cell.bg = gc.bg;
    }
}
fn tty_colours_us(tty: &mut tty, gc: &grid_cell) {
    {
        let mut c: u_int;
        if gc.us == 8 as core::ffi::c_int || gc.us == 9 as core::ffi::c_int {
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
                    (c & !COLOUR_FLAG_256 as u_int) as core::ffi::c_int,
                );
                return;
            }
            let (r, g, b) = RustColourEngine.split_rgb(gc.us);
            c = (65536 as core::ffi::c_int * r as core::ffi::c_int
                + 256 as core::ffi::c_int * g as core::ffi::c_int
                + b as core::ffi::c_int) as u_int;
            if tty_term_of(tty).has(TTYC_SETULC) {
                tty_putcode_i(tty, TTYC_SETULC, c as core::ffi::c_int);
            } else if tty_term_of(tty).has(TTYC_SETAL) && tty_term_of(tty).has(TTYC_RGB) {
                tty_putcode_i(tty, TTYC_SETAL, c as core::ffi::c_int);
            }
        }
        tty.cell.us = gc.us;
    }
}
fn tty_try_colour(
    tty: &mut tty,
    colour: core::ffi::c_int,
    type_0: &core::ffi::CStr,
) -> core::ffi::c_int {
    {
        let is_fg = type_0.to_bytes().first() == Some(&b'3');
        if colour & COLOUR_FLAG_256 != 0 {
            if is_fg && tty_term_of(tty).has(TTYC_SETAF) {
                tty_putcode_i(tty, TTYC_SETAF, colour & 0xff as core::ffi::c_int);
            } else if tty_term_of(tty).has(TTYC_SETAB) {
                tty_putcode_i(tty, TTYC_SETAB, colour & 0xff as core::ffi::c_int);
            }
            return 0 as core::ffi::c_int;
        }
        if colour & COLOUR_FLAG_RGB != 0 {
            let (r, g, b) = RustColourEngine.split_rgb(colour & 0xffffff as core::ffi::c_int);
            if is_fg && tty_term_of(tty).has(TTYC_SETRGBF) {
                tty_putcode_iii(
                    tty,
                    TTYC_SETRGBF,
                    r as core::ffi::c_int,
                    g as core::ffi::c_int,
                    b as core::ffi::c_int,
                );
            } else if tty_term_of(tty).has(TTYC_SETRGBB) {
                tty_putcode_iii(
                    tty,
                    TTYC_SETRGBB,
                    r as core::ffi::c_int,
                    g as core::ffi::c_int,
                    b as core::ffi::c_int,
                );
            }
            return 0 as core::ffi::c_int;
        }
        -(1 as core::ffi::c_int)
    }
}
fn tty_window_default_style(gc: &mut grid_cell, wp: &impl crate::WindowPane) {
    *gc = grid_default_cell;
    gc.fg = wp.palette().fg;
    gc.bg = wp.palette().bg;
}
unsafe fn tty_style_changed(wp: &mut impl crate::WindowPane) {
    unsafe {
        let oo = wp.options_ref().clone();
        log_debug(c"%%%u: style changed", fmt_args![wp.pane_id()]);
        *wp.flags_mut() &= !PANE_STYLECHANGED;
        let mut ft = format_create(
            None,
            None,
            (FORMAT_PANE | wp.pane_id()) as core::ffi::c_int,
            FORMAT_NOJOBS,
        );
        format_defaults(&mut ft, None, None, None, Some(wp));
        let mut active = grid_default_cell;
        tty_window_default_style(&mut active, wp);
        style_add(&mut active, &oo, c"window-active-style", Some(&mut ft));
        let mut normal = grid_default_cell;
        tty_window_default_style(&mut normal, wp);
        style_add(&mut normal, &oo, c"window-style", Some(&mut ft));
        wp.set_styles(PaneStyleCells { cached_gc: normal, cached_active_gc: active });
    }
}
pub unsafe fn tty_default_colours(gc: &mut grid_cell, wp: &mut impl crate::WindowPane) {
    unsafe {
        if *wp.flags() & PANE_STYLECHANGED != 0 {
            tty_style_changed(wp);
        }
        *gc = grid_default_cell;
        let styles = wp.styles();
        let active = wp.window_context().is_some_and(|window| {
            window
                .active_pane()
                .is_some_and(|pane| wp.observation().as_ref() == Some(&pane))
        });
        if active && styles.cached_active_gc.fg != 8 as core::ffi::c_int {
            gc.fg = styles.cached_active_gc.fg;
        } else {
            gc.fg = styles.cached_gc.fg;
        }
        if active && styles.cached_active_gc.bg != 8 as core::ffi::c_int {
            gc.bg = styles.cached_active_gc.bg;
        } else {
            gc.bg = styles.cached_gc.bg;
        };
    }
}
pub unsafe fn tty_default_attributes(
    tty: &mut tty,
    defaults: &grid_cell,
    palette: Option<&colour_palette>,
    bg: u_int,
    hl: Option<&RustHyperlinks>,
) {
    unsafe {
        let mut gc;
        gc = grid_default_cell;
        gc.bg = bg as core::ffi::c_int;
        tty_attributes(tty, &gc, defaults, palette, hl);
    }
}
pub fn tty_clipboard_query(tty: &mut tty) {
    let tv = timeval::from_secs(TTY_QUERY_TIMEOUT as __time_t);
    if tty.flags & TTY_STARTED != 0 && !tty.flags & TTY_OSC52QUERY != 0 {
        tty_putcode_ss(tty, TTYC_MS, c"", c"?");
        tty.flags |= TTY_OSC52QUERY;
        tty.clipboard_timer.arm(tv);
    }
}
pub fn tty_set_progress_bar(tty: &mut tty, pb: &progress_bar) {
    {
        if tty_term_of(tty).has(TTYC_SPB) {
            tty_putcode_ii(tty, TTYC_SPB, pb.state as core::ffi::c_int, pb.progress);
        }
    }
}

use crate::screen::RustScreen;

#[cfg(test)]
#[path = "../tests/test_tty_driver_focused.rs"]
mod focused_tests;

impl WindowRef {
    pub unsafe fn update_client_offsets(&self) {
        let w = self;

        unsafe {
            for mut client in client_walk() {
                let Some(session) = client.attached_session() else {
                    continue;
                };
                let current = session.current_window();
                if current.as_ref().is_some_and(|current| current.ptr_eq(w)) {
                    tty_update_client_offset(client.as_client_mut());
                }
            }
        }
    }
}

impl ClientRef {
    /// Requests the terminal clipboard when the TTY is started and no query is
    /// outstanding. The existing terminal capability controls query output.
    /// The query timer and later terminal replies retain their existing buffer
    /// import and rearming behavior; no command callback runs synchronously.
    ///
    /// # Safety
    /// Run on the server thread without conflicting access to the client's TTY.
    pub(crate) unsafe fn query_clipboard(&mut self) {
        unsafe { tty_clipboard_query(self.as_tty_mut()) }
    }

    unsafe fn on_tty_read(&mut self) {
        let c = self;

        unsafe {
            let identity = c.clone();
            let name = identity.name();
            let fd = c.fd();
            let size = c.as_tty().r#in.as_ref().unwrap().len();
            let nread = match c
                .as_tty_mut()
                .r#in
                .as_mut()
                .unwrap()
                .read_from_fd(fd, 64 * 1024)
            {
                Ok(nread) => nread as core::ffi::c_int,
                Err(_) => -(1 as core::ffi::c_int),
            };
            if nread == 0 as core::ffi::c_int || nread == -(1 as core::ffi::c_int) {
                if nread == 0 as core::ffi::c_int {
                    log_debug(c"%s: read closed", fmt_args![name]);
                } else {
                    log_debug(
                        c"%s: read error: %s",
                        fmt_args![name, error_message(*__errno_location()).as_c_str()],
                    );
                }
                c.as_tty_mut().event_in.disable();
                (c.clone()).on_lost();
                return;
            }
            log_debug(
                c"%s: read %d bytes (already %zu)",
                fmt_args![name, nread, size],
            );
            while c.next_tty_key() != 0 {}
        }
    }
}

#[cfg(test)]
pub use crate::consts::{PROGRESS_BAR_NORMAL, TTYC_ACSC, TTYC_AM, TTYC_BEL};
