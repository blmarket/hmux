use crate::ffi::sscanf;
use crate::fmt_args;
use crate::layout::{layout_resize, layout_root_ptr};
use crate::log::log_debug;
use crate::notify::notify_window;
use crate::options::{options_get_number, options_get_string, options_ptr};
use crate::server::client_walk;
use crate::server::server_client_get_client_window;
use crate::server::server_redraw_window;
use crate::session::{
    session_add_attached, session_clear_attached, session_get_curw, session_options,
};
use crate::session::{session_has, sessions_after, sessions_first};
use crate::status::{status_line_size, status_update_cache};
use crate::tmux::global_w_options;
use crate::tty::tty_update_window_offset;
pub use crate::types::*;
use crate::window::window_get_active;
use crate::window::window_get_latest;
use crate::window::{window_find_by_id_ref, window_resize, window_unzoom, window_zoom, windows};
use ::core::ffi::c_int;
use ::core::ptr::null_mut;

pub const UINT_MAX: u_int = u_int::MAX;
pub const RB_NEGINF: c_int = -1;
pub const PANE_MINIMUM: c_int = 1;
pub const WINDOW_MINIMUM: c_int = PANE_MINIMUM;
pub const WINDOW_MAXIMUM: c_int = 10000;
pub const WINDOW_ZOOMED: c_int = 0x8;
pub const WINDOW_RESIZE: c_int = 0x20;
pub const WINDOW_SIZE_LARGEST: c_int = 0;
pub const WINDOW_SIZE_MANUAL: c_int = 2;
pub const WINDOW_SIZE_LATEST: c_int = 3;
pub const CLIENT_EXIT: c_int = 0x4;
pub const CLIENT_SUSPENDED: c_int = 0x40;
pub const CLIENT_DEAD: c_int = 0x200;
pub const CLIENT_CONTROL: c_int = 0x2000;
pub const CLIENT_IGNORESIZE: c_int = 0x20000;
pub const CLIENT_SIZECHANGED: c_int = 0x400000;
pub const CLIENT_STATUSOFF: c_int = 0x800000;
pub const CLIENT_WINDOWSIZECHANGED: uint64_t = 0x400000000;
pub const CLIENT_UNATTACHEDFLAGS: c_int = CLIENT_DEAD | CLIENT_SUSPENDED | CLIENT_EXIT;
pub const CLIENT_NOSIZEFLAGS: c_int = CLIENT_DEAD | CLIENT_SUSPENDED | CLIENT_EXIT;

/// The sessions in the server's tree, in name order.
fn each_session() -> impl Iterator<Item = *mut session> {
    let mut current = null_mut::<session>();
    let mut started = false;
    ::core::iter::from_fn(move || unsafe {
        current = if started {
            sessions_after(current)
        } else {
            started = true;
            sessions_first()
        };
        (!current.is_null()).then_some(current)
    })
}

/// The windows in the server's tree, in id order.
fn each_window() -> impl Iterator<Item = WindowRef> {
    let ids: Vec<u_int> = windows.map().keys().copied().collect();
    ids.into_iter().filter_map(window_find_by_id_ref)
}

/// Gives `w` a new size, resizing its layout tree and its panes with it.
///
/// The size asked for is held between the window minimum and maximum, and then
/// raised again to whatever the layout tree settled on, which may be more than
/// was asked for when the panes in it cannot fit. A zoomed window is unzoomed
/// for the resize and zoomed again after, so the layout underneath is the one
/// that moves.
pub unsafe fn resize_window(
    w: *mut window,
    mut sx: u_int,
    mut sy: u_int,
    xpixel: c_int,
    ypixel: c_int,
) {
    unsafe {
        sx = sx.clamp(WINDOW_MINIMUM as u_int, WINDOW_MAXIMUM as u_int);
        sy = sy.clamp(WINDOW_MINIMUM as u_int, WINDOW_MAXIMUM as u_int);

        let zoomed = (*w).flags & WINDOW_ZOOMED != 0;
        if zoomed {
            window_unzoom(w, 1);
        }

        layout_resize(w, sx, sy);
        sx = sx.max((*layout_root_ptr(&(*w).layout_root)).sx);
        sy = sy.max((*layout_root_ptr(&(*w).layout_root)).sy);
        window_resize(w, sx, sy, xpixel, ypixel);
        log_debug(
            c"%s: @%u resized to %ux%u; layout %ux%u".as_ptr(),
            fmt_args![
                c"resize_window".as_ptr(),
                (*w).id,
                sx,
                sy,
                (*layout_root_ptr(&(*w).layout_root)).sx,
                (*layout_root_ptr(&(*w).layout_root)).sy
            ],
        );

        if zoomed {
            window_zoom(window_get_active(w));
        }

        tty_update_window_offset(w);
        server_redraw_window(w);
        notify_window(c"window-layout-changed".as_ptr(), w);
        notify_window(c"window-resized".as_ptr(), w);
        (*w).flags &= !WINDOW_RESIZE;
    }
}

/// Whether `c`'s terminal size has no say in how big a window is: it has no
/// session, it is on its way out, it was told to ignore its own size while some
/// other client was not, or it is a control client that has not reported a size
/// yet.
unsafe fn ignore_client_size(c: *mut client) -> bool {
    unsafe {
        if (*c).session.is_null() {
            return true;
        }
        if (*c).flags & CLIENT_NOSIZEFLAGS as uint64_t != 0 {
            return true;
        }
        if (*c).flags & CLIENT_IGNORESIZE as uint64_t != 0
            && client_walk().any(|loop_0| {
                !(*loop_0).session.is_null()
                    && (*loop_0).flags & CLIENT_NOSIZEFLAGS as uint64_t == 0
                    && (*loop_0).flags & CLIENT_IGNORESIZE as uint64_t == 0
            })
        {
            return true;
        }
        (*c).flags & CLIENT_CONTROL as uint64_t != 0
            && (*c).flags & CLIENT_SIZECHANGED as uint64_t == 0
            && (*c).flags & CLIENT_WINDOWSIZECHANGED == 0
    }
}

/// How many clients have a say in `w`'s size, counted no further than two,
/// which is all the latest-client policy needs to know.
unsafe fn clients_with_window(w: *mut window) -> u_int {
    unsafe {
        client_walk()
            .filter(|c| !ignore_client_size(*c) && session_has((**c).session, w) != 0)
            .take(2)
            .count() as u_int
    }
}

/// Whether a client is not one of the ones a size is being worked out for.
type skip_client = unsafe fn(&client, c_int, c_int, *mut session, *mut window) -> bool;

/// The size the clients settle on for a window.
///
/// `found` says whether any client had a say. When none did, `sx` and `sy` are
/// left at the starting values of the policy — zero for the largest size,
/// `UINT_MAX` for the smallest and the latest — and the caller writes them out
/// all the same before looking for a size of its own, which is what the C's
/// out-parameters did.
struct client_size {
    found: bool,
    sx: u_int,
    sy: u_int,
    xpixel: u_int,
    ypixel: u_int,
}

unsafe fn clients_calculate_size(
    type_0: c_int,
    current: c_int,
    c: *mut client,
    s: *mut session,
    w: *mut window,
    skip_client: skip_client,
) -> client_size {
    unsafe {
        let mut size = client_size {
            found: false,
            sx: 0,
            sy: 0,
            xpixel: 0,
            ypixel: 0,
        };
        if type_0 == WINDOW_SIZE_LARGEST {
            size.sx = 0;
            size.sy = 0;
        } else if !w.is_null() && type_0 == WINDOW_SIZE_MANUAL {
            size.sx = (*w).manual_sx;
            size.sy = (*w).manual_sy;
            log_debug(
                c"%s: manual size %ux%u".as_ptr(),
                fmt_args![c"clients_calculate_size".as_ptr(), size.sx, size.sy],
            );
        } else {
            size.sx = UINT_MAX;
            size.sy = UINT_MAX;
        }

        let mut n = 0;
        if type_0 == WINDOW_SIZE_LATEST && !w.is_null() {
            n = clients_with_window(w);
        }

        if type_0 != WINDOW_SIZE_MANUAL {
            for loop_0 in client_walk() {
                if loop_0 != c && ignore_client_size(loop_0) {
                    log_debug(
                        c"%s: ignoring %s (1)".as_ptr(),
                        fmt_args![
                            c"clients_calculate_size".as_ptr(),
                            cstr_ptr(&(*loop_0).name)
                        ],
                    );
                } else if loop_0 != c && skip_client(&*loop_0, type_0, current, s, w) {
                    log_debug(
                        c"%s: skipping %s (1)".as_ptr(),
                        fmt_args![
                            c"clients_calculate_size".as_ptr(),
                            cstr_ptr(&(*loop_0).name)
                        ],
                    );
                } else if type_0 == WINDOW_SIZE_LATEST && n > 1 && loop_0 != window_get_latest(w) {
                    log_debug(
                        c"%s: %s is not latest".as_ptr(),
                        fmt_args![
                            c"clients_calculate_size".as_ptr(),
                            cstr_ptr(&(*loop_0).name)
                        ],
                    );
                } else {
                    let cw = if w.is_null() {
                        null_mut::<client_window>()
                    } else {
                        server_client_get_client_window(loop_0, (*w).id)
                    };
                    let (cx, cy) = if !cw.is_null() && (*cw).sx != 0 && (*cw).sy != 0 {
                        ((*cw).sx, (*cw).sy)
                    } else {
                        (
                            (*loop_0).tty.sx,
                            (*loop_0).tty.sy.wrapping_sub(status_line_size(loop_0)),
                        )
                    };
                    if type_0 == WINDOW_SIZE_LARGEST {
                        size.sx = size.sx.max(cx);
                        size.sy = size.sy.max(cy);
                    } else {
                        size.sx = size.sx.min(cx);
                        size.sy = size.sy.min(cy);
                    }
                    if (*loop_0).tty.xpixel > size.xpixel && (*loop_0).tty.ypixel > size.ypixel {
                        size.xpixel = (*loop_0).tty.xpixel;
                        size.ypixel = (*loop_0).tty.ypixel;
                    }
                    log_debug(
                        c"%s: after %s (%ux%u), size is %ux%u".as_ptr(),
                        fmt_args![
                            c"clients_calculate_size".as_ptr(),
                            cstr_ptr(&(*loop_0).name),
                            cx,
                            cy,
                            size.sx,
                            size.sy
                        ],
                    );
                }
            }
            log_calculated(&size);
        }

        if !w.is_null() {
            for loop_0 in client_walk() {
                if loop_0 != c && ignore_client_size(loop_0) {
                    continue;
                }
                if loop_0 != c && skip_client(&*loop_0, type_0, current, s, w) {
                    continue;
                }
                if (*loop_0).flags & CLIENT_WINDOWSIZECHANGED == 0 {
                    continue;
                }
                let cw = server_client_get_client_window(loop_0, (*w).id);
                if cw.is_null() {
                    continue;
                }
                log_debug(
                    c"%s: %s size for @%u is %ux%u".as_ptr(),
                    fmt_args![
                        c"clients_calculate_size".as_ptr(),
                        cstr_ptr(&(*loop_0).name),
                        (*w).id,
                        (*cw).sx,
                        (*cw).sy
                    ],
                );
                if (*cw).sx != 0 && size.sx > (*cw).sx {
                    size.sx = (*cw).sx;
                }
                if (*cw).sy != 0 && size.sy > (*cw).sy {
                    size.sy = (*cw).sy;
                }
            }
        }
        log_calculated(&size);

        if type_0 == WINDOW_SIZE_MANUAL {
            log_debug(
                c"%s: type is manual".as_ptr(),
                fmt_args![c"clients_calculate_size".as_ptr()],
            );
            size.found = !w.is_null();
            return size;
        }
        if type_0 == WINDOW_SIZE_LARGEST {
            log_debug(
                c"%s: type is largest".as_ptr(),
                fmt_args![c"clients_calculate_size".as_ptr()],
            );
            size.found = size.sx != 0 && size.sy != 0;
            return size;
        }
        if type_0 == WINDOW_SIZE_LATEST {
            log_debug(
                c"%s: type is latest".as_ptr(),
                fmt_args![c"clients_calculate_size".as_ptr()],
            );
        } else {
            log_debug(
                c"%s: type is smallest".as_ptr(),
                fmt_args![c"clients_calculate_size".as_ptr()],
            );
        }
        size.found = size.sx != UINT_MAX && size.sy != UINT_MAX;
        size
    }
}

/// Says in the log whether a size has been worked out yet.
fn log_calculated(size: &client_size) {
    unsafe {
        if size.sx != UINT_MAX && size.sy != UINT_MAX {
            log_debug(
                c"%s: calculated size %ux%u".as_ptr(),
                fmt_args![c"clients_calculate_size".as_ptr(), size.sx, size.sy],
            );
        } else {
            log_debug(
                c"%s: no calculated size".as_ptr(),
                fmt_args![c"clients_calculate_size".as_ptr()],
            );
        }
    }
}

/// For a new window: a client showing something else has no say. With no
/// window yet, a client of another session has none either.
unsafe fn default_window_size_skip_client(
    loop_0: &client,
    _type_0: c_int,
    _current: c_int,
    s: *mut session,
    w: *mut window,
) -> bool {
    unsafe {
        if !w.is_null() {
            return session_has(loop_0.session, w) == 0;
        }
        loop_0.session != s
    }
}

/// The size a window should be created at.
///
/// `type_0` is a `WINDOW_SIZE_*` policy, or -1 to read the global `window-size`
/// option. The latest-client policy takes the size straight off `c` when it has
/// a say; otherwise the clients settle it between them, and a window no client
/// can size falls back on the session's `default-size` option and then on
/// 80 by 24.
pub unsafe fn default_window_size(
    mut c: *mut client,
    s: *mut session,
    w: *mut window,
    mut type_0: c_int,
) -> (u_int, u_int, u_int, u_int) {
    unsafe {
        let (mut sx, mut sy, mut xpixel, mut ypixel) = (0, 0, 0, 0);
        if type_0 == -1 {
            type_0 = options_get_number(global_w_options, c"window-size".as_ptr()) as c_int;
        }
        if type_0 == WINDOW_SIZE_LATEST && !c.is_null() && !ignore_client_size(c) {
            sx = (*c).tty.sx;
            sy = (*c).tty.sy.wrapping_sub(status_line_size(c));
            xpixel = (*c).tty.xpixel;
            ypixel = (*c).tty.ypixel;
            log_debug(
                c"%s: using %ux%u from %s".as_ptr(),
                fmt_args![
                    c"default_window_size".as_ptr(),
                    sx,
                    sy,
                    cstr_ptr(&(*c).name)
                ],
            );
        } else {
            if !c.is_null() && (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
                c = null_mut::<client>();
            }
            let size = clients_calculate_size(type_0, 0, c, s, w, default_window_size_skip_client);
            sx = size.sx;
            sy = size.sy;
            xpixel = size.xpixel;
            ypixel = size.ypixel;
            if !size.found {
                let value = options_get_string(session_options(s), c"default-size".as_ptr());
                if sscanf(value, c"%ux%u".as_ptr(), &raw mut sx, &raw mut sy) != 2 {
                    sx = 80;
                    sy = 24;
                }
                log_debug(
                    c"%s: using %ux%u from default-size".as_ptr(),
                    fmt_args![c"default_window_size".as_ptr(), sx, sy],
                );
            }
        }
        sx = sx.clamp(WINDOW_MINIMUM as u_int, WINDOW_MAXIMUM as u_int);
        sy = sy.clamp(WINDOW_MINIMUM as u_int, WINDOW_MAXIMUM as u_int);
        log_debug(
            c"%s: resulting size is %ux%u".as_ptr(),
            fmt_args![c"default_window_size".as_ptr(), sx, sy],
        );
        (sx, sy, xpixel, ypixel)
    }
}

/// For an existing window: a client whose session shows nothing has no say, and
/// under `aggressive-resize` only a client whose current window is this one
/// does.
unsafe fn recalculate_size_skip_client(
    loop_0: &client,
    _type_0: c_int,
    current: c_int,
    _s: *mut session,
    w: *mut window,
) -> bool {
    unsafe {
        if session_get_curw(loop_0.session).is_null() {
            return true;
        }
        if current != 0 {
            return (*session_get_curw(loop_0.session)).window() != w;
        }
        session_has(loop_0.session, w) == 0
    }
}

/// Works out what size `w` should be and gives it that size, or notes it for
/// the next redraw. `now` resizes at once rather than waiting.
pub unsafe fn recalculate_size(w: *mut window, now: c_int) {
    unsafe {
        if window_get_active(w).is_null() {
            return;
        }
        log_debug(
            c"%s: @%u is %ux%u".as_ptr(),
            fmt_args![c"recalculate_size".as_ptr(), (*w).id, (*w).sx, (*w).sy],
        );

        let type_0 =
            options_get_number(options_ptr(&(*w).options), c"window-size".as_ptr()) as c_int;
        let current =
            options_get_number(options_ptr(&(*w).options), c"aggressive-resize".as_ptr()) as c_int;
        let size = clients_calculate_size(
            type_0,
            current,
            null_mut::<client>(),
            null_mut::<session>(),
            w,
            recalculate_size_skip_client,
        );

        let mut changed = size.found;
        if (*w).flags & WINDOW_RESIZE != 0 {
            if now == 0 && changed && (*w).new_sx == size.sx && (*w).new_sy == size.sy {
                changed = false;
            }
        } else if now == 0 && changed && (*w).sx == size.sx && (*w).sy == size.sy {
            changed = false;
        }

        if !changed {
            log_debug(
                c"%s: @%u no size change".as_ptr(),
                fmt_args![c"recalculate_size".as_ptr(), (*w).id],
            );
            tty_update_window_offset(w);
            return;
        }
        log_debug(
            c"%s: @%u new size %ux%u".as_ptr(),
            fmt_args![c"recalculate_size".as_ptr(), (*w).id, size.sx, size.sy],
        );

        if now != 0 || type_0 == WINDOW_SIZE_MANUAL {
            resize_window(
                w,
                size.sx,
                size.sy,
                size.xpixel as c_int,
                size.ypixel as c_int,
            );
        } else {
            (*w).new_sx = size.sx;
            (*w).new_sy = size.sy;
            (*w).new_xpixel = size.xpixel;
            (*w).new_ypixel = size.ypixel;
            (*w).flags |= WINDOW_RESIZE;
            tty_update_window_offset(w);
        }
    }
}

/// Recalculates every window's size, holding each one until the next redraw.
pub fn recalculate_sizes() {
    recalculate_sizes_now(0);
}

/// Recalculates every window's size, first counting how many clients each
/// session has attached and deciding which clients have room for a status line.
pub fn recalculate_sizes_now(now: c_int) {
    unsafe {
        for s in each_session() {
            session_clear_attached(s);
            status_update_cache(s);
        }
        for c in client_walk() {
            let s = (*c).session;
            if !s.is_null() && (*c).flags & CLIENT_UNATTACHEDFLAGS as uint64_t == 0 {
                session_add_attached(s);
            }
            if !ignore_client_size(c) {
                if (*c).tty.sy <= (*s).statuslines || (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
                    (*c).flags |= CLIENT_STATUSOFF as uint64_t;
                } else {
                    (*c).flags &= !(CLIENT_STATUSOFF as uint64_t);
                }
            }
        }
        for w_ref in each_window() {
            recalculate_size(w_ref.as_ptr(), now);
        }
    }
}
#[cfg(test)]
#[path = "tests/test_resize.rs"]
mod tests;
