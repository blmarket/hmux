//! Component-only terminal output. No Client queries, model callbacks or format
//! expansion may run while this scope is borrowed. The caller resolves those
//! before entering and releases the scope before dispatching further work.
use super::*;
use std::ops::{Deref, DerefMut};

/// A bounded output operation over disjoint pieces of one borrowed Client.
/// A future whole-Client RefMut remains owned by with_terminal_output while this
/// view exists; none of these references outlive that invocation.
pub struct TerminalOutput<'a> {
    terminal: &'a mut tty,
    written: &'a mut usize,
    total_discarded: &'a mut usize,
    redraw_remaining: &'a mut usize,
    fd: i32,
    name: Option<&'a CStr>,
    utf8: bool,
    theme: client_theme,
    theme_colours: &'a [i32; COLOUR_THEME_COUNT as usize],
    clips_output: bool,
}

impl<'a> TerminalOutput<'a> {
    pub(crate) fn new(
        terminal: &'a mut tty,
        written: &'a mut usize,
        discarded: &'a mut usize,
        redraw: &'a mut usize,
        fd: i32,
        name: Option<&'a CStr>,
        utf8: bool,
        theme: client_theme,
        theme_colours: &'a [i32; COLOUR_THEME_COUNT as usize],
        clips_output: bool,
    ) -> Self {
        Self {
            terminal,
            written,
            total_discarded: discarded,
            redraw_remaining: redraw,
            fd,
            name,
            utf8,
            theme,
            theme_colours,
            clips_output,
        }
    }
    pub(crate) fn fd(&self) -> i32 {
        self.fd
    }
    pub(crate) fn name(&self) -> Option<&CStr> {
        self.name
    }
    pub(crate) fn record_discard(&mut self, bytes: usize) {
        *self.total_discarded = self.total_discarded.wrapping_add(bytes);
    }
    /// Some(0) still identifies a completed redraw write. Backpressure starts
    /// only on a subsequent write, matching the original branch ordering.
    pub(crate) fn acknowledge_redraw(&mut self, bytes: usize) -> Option<usize> {
        if *self.redraw_remaining == 0 {
            return None;
        }
        *self.redraw_remaining = self.redraw_remaining.saturating_sub(bytes);
        Some(*self.redraw_remaining)
    }
    pub(crate) fn utf8(&self) -> bool {
        self.utf8
    }
    pub(crate) fn clips_output(&self) -> bool {
        self.clips_output
    }
}

impl Deref for TerminalOutput<'_> {
    type Target = tty;
    fn deref(&self) -> &tty {
        self.terminal
    }
}
impl DerefMut for TerminalOutput<'_> {
    fn deref_mut(&mut self) -> &mut tty {
        self.terminal
    }
}

pub unsafe fn tty_putcode(tty: &mut TerminalOutput<'_>, mut code: tty_code_code) {
    tty_puts(
        tty,
        tty_term_string(
            &*(tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)),
            code,
        ),
    );
}

pub unsafe fn tty_putcode_i(
    tty: &mut TerminalOutput<'_>,
    mut code: tty_code_code,
    mut a: ::core::ffi::c_int,
) {
    if a < 0 as ::core::ffi::c_int {
        return;
    }
    tty_puts(
        tty,
        &tty_term_string_i(
            tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term),
            code,
            a,
        ),
    );
}

pub unsafe fn tty_putcode_ii(
    tty: &mut TerminalOutput<'_>,
    mut code: tty_code_code,
    mut a: ::core::ffi::c_int,
    mut b: ::core::ffi::c_int,
) {
    if a < 0 as ::core::ffi::c_int || b < 0 as ::core::ffi::c_int {
        return;
    }
    tty_puts(
        tty,
        &tty_term_string_ii(
            tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term),
            code,
            a,
            b,
        ),
    );
}

pub unsafe fn tty_putcode_iii(
    tty: &mut TerminalOutput<'_>,
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
        &tty_term_string_iii(
            tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term),
            code,
            a,
            b,
            c,
        ),
    );
}

pub unsafe fn tty_putcode_s(tty: &mut TerminalOutput<'_>, mut a: *const ::core::ffi::c_char) {
    let mut code: tty_code_code = TTYC_CS;
    if !a.is_null() {
        tty_puts(
            tty,
            &tty_term_string_s(
                tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term),
                code,
                a,
            ),
        );
    }
}

pub unsafe fn tty_putcode_ss(
    tty: &mut TerminalOutput<'_>,
    mut code: tty_code_code,
    mut a: *const ::core::ffi::c_char,
    mut b: *const ::core::ffi::c_char,
) {
    if !a.is_null() && !b.is_null() {
        tty_puts(
            tty,
            &tty_term_string_ss(
                tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term),
                code,
                a,
                b,
            ),
        );
    }
}

pub(super) unsafe fn tty_add(tty: &mut TerminalOutput<'_>, buf: &[u8]) {
    tty_enqueue_bytes(tty.terminal, tty.name, tty.written, buf);
}

pub(crate) unsafe fn tty_enqueue_bytes(
    tty: &mut tty,
    name: Option<&CStr>,
    written: &mut usize,
    buf: &[u8],
) {
    let len = buf.len();
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
        log_cstr(name.map_or(std::ptr::null(), CStr::as_ptr)),
        log_cstr_n(buf.as_ptr().cast(), len as ::core::ffi::c_int)
    ));
    *written = written.wrapping_add(len);
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

pub unsafe fn tty_puts(tty: &mut TerminalOutput<'_>, text: &CStr) {
    if !text.is_empty() {
        tty_add(tty, text.to_bytes());
    }
}

pub unsafe fn tty_putc(tty: &mut TerminalOutput<'_>, mut ch: u_char) {
    if (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags & TERM_NOAM
        != 0
        && ch as ::core::ffi::c_int >= 0x20 as ::core::ffi::c_int
        && ch as ::core::ffi::c_int != 0x7f as ::core::ffi::c_int
        && (*tty).cy == (*tty).sy.wrapping_sub(1 as u_int)
        && (*tty).cx.wrapping_add(1 as u_int) >= (*tty).sx
    {
        return;
    }
    if (*tty).cell.attr as ::core::ffi::c_int & GRID_ATTR_CHARSET != 0 {
        let utf8 = tty.utf8;
        let acs = tty_acs_get(Some(&*tty), utf8, ch).map(ToOwned::to_owned);
        if let Some(acs) = acs {
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
            if (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags
                & TERM_NOAM
                != 0
            {
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

pub unsafe fn tty_putn(tty: &mut TerminalOutput<'_>, buf: &[u8], mut width: u_int) {
    let mut len = buf.len();
    if (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags & TERM_NOAM
        != 0
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

pub(super) unsafe fn tty_set_italics(tty: &mut TerminalOutput<'_>) {
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if tty_term_has(
        tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term),
        TTYC_SITM,
    ) != 0
    {
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

pub unsafe fn tty_set_title(tty: &mut TerminalOutput<'_>, title: &CStr) {
    if tty_term_has(
        tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term),
        TTYC_TSL,
    ) == 0
        || tty_term_has(
            tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term),
            TTYC_FSL,
        ) == 0
    {
        return;
    }
    tty_putcode(tty, TTYC_TSL);
    tty_puts(tty, title);
    tty_putcode(tty, TTYC_FSL);
}

pub unsafe fn tty_set_path(tty: &mut TerminalOutput<'_>, title: &CStr) {
    if tty_term_has(
        tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term),
        TTYC_SWD,
    ) == 0
        || tty_term_has(
            tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term),
            TTYC_FSL,
        ) == 0
    {
        return;
    }
    tty_putcode(tty, TTYC_SWD);
    tty_puts(tty, title);
    tty_putcode(tty, TTYC_FSL);
}

pub(super) unsafe fn tty_force_cursor_colour(
    tty: &mut TerminalOutput<'_>,
    mut c: ::core::ffi::c_int,
) {
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

pub(super) unsafe fn tty_update_cursor(
    tty: &mut TerminalOutput<'_>,
    mut mode: ::core::ffi::c_int,
    s: Option<ScreenMode>,
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
                if tty_term_has(
                    tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term),
                    TTYC_SE,
                ) != 0
                {
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
            if tty_term_has(
                tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term),
                TTYC_SS,
            ) != 0
            {
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
            if tty_term_has(
                tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term),
                TTYC_SS,
            ) != 0
            {
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
            if tty_term_has(
                tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term),
                TTYC_SS,
            ) != 0
            {
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

pub unsafe fn tty_update_mode(
    tty: &mut TerminalOutput<'_>,
    mut mode: ::core::ffi::c_int,
    s: Option<ScreenMode>,
) {
    let mut term: *const tty_term =
        tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term);
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
                ((tty.name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                    as *const _
            ),
            screen_mode_display((*tty).mode)
        ));
        log_debug(format_args!(
            "{}: setting mode {}",
            log_cstr(
                ((tty.name)
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

pub(super) unsafe fn tty_emulate_repeat(
    tty: &mut TerminalOutput<'_>,
    mut code: tty_code_code,
    mut code1: tty_code_code,
    mut n: u_int,
) {
    if tty_term_has(
        tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term),
        code,
    ) != 0
    {
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

pub unsafe fn tty_repeat_space(tty: &mut TerminalOutput<'_>, mut n: u_int) {
    const SPACES: [u8; 500] = [b' '; 500];
    while n as usize > SPACES.len() {
        tty_putn(tty, &SPACES, SPACES.len() as u_int);
        n -= SPACES.len() as u_int;
    }
    if n != 0 {
        tty_putn(tty, &SPACES[..n as usize], n);
    }
}

pub unsafe fn tty_sync_start(tty: &mut TerminalOutput<'_>) {
    if (*tty).flags & TTY_BLOCK != 0 {
        return;
    }
    if (*tty).flags & TTY_SYNCING != 0 {
        return;
    }
    (*tty).flags |= TTY_SYNCING;
    if tty_term_has(
        tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term),
        TTYC_SYNC,
    ) != 0
    {
        log_debug(format_args!(
            "{} sync start",
            log_cstr(
                ((tty.name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                    as *const _
            )
        ));
        tty_putcode_i(tty, TTYC_SYNC, 1 as ::core::ffi::c_int);
    }
}

pub unsafe fn tty_sync_end(tty: &mut TerminalOutput<'_>) {
    if (*tty).flags & TTY_BLOCK != 0 {
        return;
    }
    if !(*tty).flags & TTY_SYNCING != 0 {
        return;
    }
    (*tty).flags &= !TTY_SYNCING;
    if tty_term_has(
        tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term),
        TTYC_SYNC,
    ) != 0
    {
        log_debug(format_args!(
            "{} sync end",
            log_cstr(
                ((tty.name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                    as *const _
            )
        ));
        tty_putcode_i(tty, TTYC_SYNC, 2 as ::core::ffi::c_int);
    }
}

pub unsafe fn tty_reset(tty: &mut TerminalOutput<'_>) {
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
        if (*gc).attr as ::core::ffi::c_int & GRID_ATTR_CHARSET != 0 && {
            let utf8 = tty.utf8;
            tty_acs_needed(Some(&**tty), utf8)
        } != 0
        {
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

pub unsafe fn tty_invalidate(tty: &mut TerminalOutput<'_>) {
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
        if (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags
            & TERM_DECSLRM
            != 0
        {
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

pub unsafe fn tty_region_off(tty: &mut TerminalOutput<'_>) {
    tty_region(tty, 0 as u_int, (*tty).sy.wrapping_sub(1 as u_int));
}

pub(super) unsafe fn tty_region_pane(
    tty: &mut TerminalOutput<'_>,
    ctx: &tty_ctx,
    mut rupper: u_int,
    mut rlower: u_int,
) {
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

pub(super) unsafe fn tty_region(
    tty: &mut TerminalOutput<'_>,
    mut rupper: u_int,
    mut rlower: u_int,
) {
    if (*tty).rlower == rlower && (*tty).rupper == rupper {
        return;
    }
    if tty_term_has(
        tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term),
        TTYC_CSR,
    ) == 0
    {
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

pub unsafe fn tty_margin_off(tty: &mut TerminalOutput<'_>) {
    tty_margin(tty, 0 as u_int, (*tty).sx.wrapping_sub(1 as u_int));
}

pub(super) unsafe fn tty_margin_pane(tty: &mut TerminalOutput<'_>, ctx: &tty_ctx) {
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

pub(super) unsafe fn tty_margin(tty: &mut TerminalOutput<'_>, mut rleft: u_int, mut rright: u_int) {
    if (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags
        & TERM_DECSLRM
        == 0
    {
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

pub(super) unsafe fn tty_cursor_pane_unless_wrap(
    tty: &mut TerminalOutput<'_>,
    ctx: &tty_ctx,
    mut cx: u_int,
    mut cy: u_int,
) {
    if !ctx.flags & TTY_CTX_WRAPPED != 0
        || !(ctx.xoff == 0 as ::core::ffi::c_int && ctx.sx >= (*tty).sx)
        || (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags
            & TERM_NOAM
            != 0
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

pub(super) unsafe fn tty_cursor_pane(
    tty: &mut TerminalOutput<'_>,
    ctx: &tty_ctx,
    mut cx: u_int,
    mut cy: u_int,
) {
    tty_cursor(
        tty,
        (ctx.xoff as u_int).wrapping_add(cx).wrapping_sub(ctx.wox),
        (ctx.yoff as u_int).wrapping_add(cy).wrapping_sub(ctx.woy),
    );
}

pub unsafe fn tty_cursor(tty: &mut TerminalOutput<'_>, mut cx: u_int, mut cy: u_int) {
    let mut current_block: u64;
    let mut term: *const tty_term =
        tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term);
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
        && ((*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags
            & TERM_DECSLRM
            == 0
            || (*tty).rleft == 0 as u_int)
    {
        tty_putc(tty, '\r' as i32 as u_char);
        tty_putc(tty, '\n' as i32 as u_char);
        current_block = 5411263895410993842;
    } else if cy == thisy {
        if cx == 0 as u_int
            && ((*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags
                & TERM_DECSLRM
                == 0
                || (*tty).rleft == 0 as u_int)
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
                && (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags
                    & TERM_DECSLRM
                    == 0
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
                && (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags
                    & TERM_DECSLRM
                    == 0
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

pub(super) unsafe fn tty_hyperlink(
    tty: &mut TerminalOutput<'_>,
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

pub(super) unsafe fn tty_dim_default_colour(
    tty: &mut TerminalOutput<'_>,
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
    theme = tty.theme;
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
    tty: &mut TerminalOutput<'_>,
    gc: &grid_cell,
    style_ctx: Option<&tty_style_ctx>,
) {
    let mut gc2 = *gc;
    let mut changed: ::core::ffi::c_int = 0;
    let style_ctx = style_ctx.unwrap_or(&tty_default_style_ctx);
    let palette = &style_ctx.palette;
    if !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_NOPALETTE != 0 {
        if gc2.fg == 8 as ::core::ffi::c_int {
            gc2.fg = style_ctx.defaults.fg;
        }
        if gc2.bg == 8 as ::core::ffi::c_int {
            gc2.bg = style_ctx.defaults.bg;
        }
        palette.with_palette(|palette| {
            let changed = colour_palette_get(palette, gc2.fg);
            if changed != -(1 as ::core::ffi::c_int) {
                gc2.fg = changed;
            }
            let changed = colour_palette_get(palette, gc2.bg);
            if changed != -(1 as ::core::ffi::c_int) {
                gc2.bg = changed;
            }
        });
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
    if tty_term_has(
        tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term),
        TTYC_SETAB,
    ) == 0
    {
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
        if tty_term_has(
            tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term),
            TTYC_REV,
        ) != 0
        {
            tty_putcode(tty, TTYC_REV);
        } else if tty_term_has(
            tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term),
            TTYC_SMSO,
        ) != 0
        {
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
    if changed & GRID_ATTR_CHARSET != 0 && {
        let utf8 = tty.utf8;
        tty_acs_needed(Some(&**tty), utf8)
    } != 0
    {
        tty_putcode(tty, TTYC_SMACS);
    }
    tty_hyperlink(tty, gc, style_ctx.hyperlinks.as_ref());
    (*tty).last_cell = gc2;
}

pub(super) unsafe fn tty_colours(tty: &mut TerminalOutput<'_>, gc: &grid_cell) {
    if gc.fg == (*tty).cell.fg && gc.bg == (*tty).cell.bg && gc.us == (*tty).cell.us {
        return;
    }
    if gc.fg == 8 as ::core::ffi::c_int
        || gc.fg == 9 as ::core::ffi::c_int
        || (gc.bg == 8 as ::core::ffi::c_int || gc.bg == 9 as ::core::ffi::c_int)
    {
        if tty_term_flag(
            tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term),
            TTYC_AX,
        ) == 0
        {
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

pub(super) unsafe fn tty_map_theme_colour(tty: &mut TerminalOutput<'_>, colour: i32) -> i32 {
    if colour & COLOUR_FLAG_THEME == 0 {
        return colour;
    }
    let index = (colour & 0xff) as usize;
    let mapped = tty.theme_colours.get(index).copied().unwrap_or(-1);
    if mapped == -1 || mapped & COLOUR_FLAG_THEME != 0 {
        8
    } else {
        mapped
    }
}

pub(super) unsafe fn tty_check_fg(
    tty: &mut TerminalOutput<'_>,
    palette: &crate::src::shared::tty::PaletteSource,
    gc: &mut grid_cell,
) {
    let mut r: u_char = 0;
    let mut g: u_char = 0;
    let mut b: u_char = 0;
    let mut colours: u_int = 0;
    let mut c: ::core::ffi::c_int = 0;
    if !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_NOPALETTE != 0 {
        c = gc.fg;
        if c < 8 as ::core::ffi::c_int
            && gc.attr as ::core::ffi::c_int & GRID_ATTR_BRIGHT != 0
            && tty_term_has(
                tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term),
                TTYC_NOBR,
            ) == 0
        {
            c += 90 as ::core::ffi::c_int;
        }
        c = palette.with_palette(|palette| colour_palette_get(palette, c));
        if c != -(1 as ::core::ffi::c_int) {
            gc.fg = c;
        }
    }
    gc.fg = tty_map_theme_colour(tty, gc.fg);
    if gc.fg & COLOUR_FLAG_RGB != 0 {
        if (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags
            & TERM_RGBCOLOURS
            != 0
        {
            return;
        }
        (r, g, b) = colour_split_rgb(gc.fg);
        gc.fg = colour_find_rgb(r, g, b);
    }
    if (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags
        & TERM_256COLOURS
        != 0
    {
        colours = 256 as u_int;
    } else {
        colours = tty_term_number(
            tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term),
            TTYC_COLORS,
        ) as u_int;
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

pub(super) unsafe fn tty_check_bg(
    tty: &mut TerminalOutput<'_>,
    palette: &crate::src::shared::tty::PaletteSource,
    gc: &mut grid_cell,
) {
    let mut r: u_char = 0;
    let mut g: u_char = 0;
    let mut b: u_char = 0;
    let mut colours: u_int = 0;
    let mut c: ::core::ffi::c_int = 0;
    if !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_NOPALETTE != 0 {
        c = palette.with_palette(|palette| colour_palette_get(palette, gc.bg));
        if c != -(1 as ::core::ffi::c_int) {
            gc.bg = c;
        }
    }
    gc.bg = tty_map_theme_colour(tty, gc.bg);
    if gc.bg & COLOUR_FLAG_RGB != 0 {
        if (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags
            & TERM_RGBCOLOURS
            != 0
        {
            return;
        }
        (r, g, b) = colour_split_rgb(gc.bg);
        gc.bg = colour_find_rgb(r, g, b);
    }
    if (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags
        & TERM_256COLOURS
        != 0
    {
        colours = 256 as u_int;
    } else {
        colours = tty_term_number(
            tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term),
            TTYC_COLORS,
        ) as u_int;
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

pub(super) unsafe fn tty_check_us(
    tty: &mut TerminalOutput<'_>,
    palette: &crate::src::shared::tty::PaletteSource,
    gc: &mut grid_cell,
) {
    let mut c: ::core::ffi::c_int = 0;
    if !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_NOPALETTE != 0 {
        c = palette.with_palette(|palette| colour_palette_get(palette, gc.us));
        if c != -(1 as ::core::ffi::c_int) {
            gc.us = c;
        }
    }
    gc.us = tty_map_theme_colour(tty, gc.us);
    if tty_term_has(
        tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term),
        TTYC_SETULC1,
    ) == 0
    {
        c = colour_force_rgb(gc.us);
        if c == -(1 as ::core::ffi::c_int) {
            gc.us = 8 as ::core::ffi::c_int;
        } else {
            gc.us = c;
        }
    }
}

pub(super) unsafe fn tty_colours_fg(tty: &mut TerminalOutput<'_>, gc: &grid_cell) {
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
        if (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags
            & TERM_256COLOURS
            != 0
        {
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

pub(super) unsafe fn tty_colours_bg(tty: &mut TerminalOutput<'_>, gc: &grid_cell) {
    let mut s: [::core::ffi::c_char; 32] = [0; 32];
    if gc.bg & COLOUR_FLAG_RGB != 0 || gc.bg & COLOUR_FLAG_256 != 0 {
        if !(tty_try_colour(tty, gc.bg, false) == 0 as ::core::ffi::c_int) {
            return;
        }
    } else if gc.bg >= 90 as ::core::ffi::c_int && gc.bg <= 97 as ::core::ffi::c_int {
        if (*tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term)).flags
            & TERM_256COLOURS
            != 0
        {
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

pub(super) unsafe fn tty_colours_us(tty: &mut TerminalOutput<'_>, gc: &grid_cell) {
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
        if tty_term_has(
            tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term),
            TTYC_SETULC,
        ) != 0
        {
            tty_putcode_i(tty, TTYC_SETULC, c as ::core::ffi::c_int);
        } else if tty_term_has(
            tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term),
            TTYC_SETAL,
        ) != 0
            && tty_term_has(
                tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term),
                TTYC_RGB,
            ) != 0
        {
            tty_putcode_i(tty, TTYC_SETAL, c as ::core::ffi::c_int);
        }
    }
    (*tty).cell.us = gc.us;
}

pub(super) unsafe fn tty_try_colour(
    tty: &mut TerminalOutput<'_>,
    mut colour: ::core::ffi::c_int,
    foreground: bool,
) -> ::core::ffi::c_int {
    let mut r: u_char = 0;
    let mut g: u_char = 0;
    let mut b: u_char = 0;
    if colour & COLOUR_FLAG_256 != 0 {
        if foreground
            && tty_term_has(
                tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term),
                TTYC_SETAF,
            ) != 0
        {
            tty_putcode_i(tty, TTYC_SETAF, colour & 0xff as ::core::ffi::c_int);
        } else if tty_term_has(
            tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term),
            TTYC_SETAB,
        ) != 0
        {
            tty_putcode_i(tty, TTYC_SETAB, colour & 0xff as ::core::ffi::c_int);
        }
        return 0 as ::core::ffi::c_int;
    }
    if colour & COLOUR_FLAG_RGB != 0 {
        (r, g, b) = colour_split_rgb(colour & 0xffffff as ::core::ffi::c_int);
        if foreground
            && tty_term_has(
                tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term),
                TTYC_SETRGBF,
            ) != 0
        {
            tty_putcode_iii(
                tty,
                TTYC_SETRGBF,
                r as ::core::ffi::c_int,
                g as ::core::ffi::c_int,
                b as ::core::ffi::c_int,
            );
        } else if tty_term_has(
            tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term),
            TTYC_SETRGBB,
        ) != 0
        {
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

pub unsafe fn tty_default_attributes(
    tty: &mut TerminalOutput<'_>,
    bg: u_int,
    style_ctx: Option<&tty_style_ctx>,
) {
    let mut gc = grid_default_cell;
    gc.bg = bg as i32;
    tty_attributes(tty, &gc, style_ctx);
}

pub unsafe fn tty_set_progress_bar(tty: &mut TerminalOutput<'_>, mut pb: *mut progress_bar) {
    if tty_term_has(
        tty_term_owner_ptr(&(*tty).term).map_or(std::ptr::null(), |term| term),
        TTYC_SPB,
    ) != 0
    {
        tty_putcode_ii(
            tty,
            TTYC_SPB,
            (*pb).state as ::core::ffi::c_int,
            (*pb).progress,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn component_output_works_inside_one_refcell_without_a_client_backreference() {
        use std::cell::RefCell;
        let state = RefCell::new((tty::empty(), 0_usize, 0_usize, 3_usize));
        let colours = [-1; COLOUR_THEME_COUNT as usize];
        unsafe {
            let mut state = state.borrow_mut();
            let (terminal, written, discarded, redraw) = &mut *state;
            terminal.sx = 80;
            terminal.sy = 24;
            terminal.cell = grid_default_cell;
            terminal.last_cell = grid_default_cell;
            terminal.out = Some(evbuffer_new());
            let mut term = tty_term::empty();
            term.codes = vec![tty_code::None; crate::src::tty_term::tty_term_ncodes() as usize]
                .into_boxed_slice();
            term.codes[TTYC_BEL as usize] = tty_code::String(c"bell".to_owned());
            terminal.term = Some(Box::new(term));
            let mut output = TerminalOutput::new(
                terminal,
                written,
                discarded,
                redraw,
                -1,
                Some(c"component"),
                true,
                THEME_UNKNOWN,
                &colours,
                false,
            );
            tty_putn(&mut output, b"a\0b", 3);
            tty_putcode(&mut output, TTYC_BEL);
            output.flags |= TTY_BLOCK;
            tty_putn(&mut output, b"lost", 4);
            assert_eq!(output.discarded, 4);
            output.record_discard(4);
            assert_eq!(output.acknowledge_redraw(3), Some(0));
            assert_eq!(output.acknowledge_redraw(1), None);
            assert!(output.client.upgrade().is_none());
        }
        let mut state = state.borrow_mut();
        assert_eq!((state.1, state.2, state.3), (7, 4, 0));
        assert_eq!(
            crate::src::reactor::evbuffer_pullup(state.0.out.as_deref_mut().unwrap(), -1).unwrap(),
            b"a\0bbell"
        );
    }
}
