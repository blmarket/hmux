use crate::window_dimensions::WindowDimensionsState;
use crate::window_trait::Window as _;

use crate::ffi::sscanf;
use crate::fmt_args;

use crate::log::log_debug;
use crate::notify::notify_window;

use crate::server::{client_walk, with_clients};

use crate::session::SESSIONS;
use crate::status::status_line_size;
use crate::tmux::global_w_options;

pub use crate::types::*;
use crate::window::WINDOWS;
use crate::window::window_active_pane;
use crate::window::window_get_latest;
use ::core::ffi::c_int;

pub use crate::consts::{
    CLIENT_CONTROL, CLIENT_DEAD, CLIENT_EXIT, CLIENT_IGNORESIZE, CLIENT_STATUSOFF,
    CLIENT_SUSPENDED, CLIENT_UNATTACHEDFLAGS, PANE_MINIMUM, RB_NEGINF, UINT_MAX, WINDOW_MAXIMUM,
    WINDOW_MINIMUM, WINDOW_RESIZE, WINDOW_SIZE_LARGEST, WINDOW_SIZE_LATEST, WINDOW_SIZE_MANUAL,
    WINDOW_ZOOMED,
};

pub const CLIENT_SIZECHANGED: c_int = 0x400000;

pub const CLIENT_WINDOWSIZECHANGED: uint64_t = 0x400000000;

pub const CLIENT_NOSIZEFLAGS: c_int = CLIENT_DEAD | CLIENT_SUSPENDED | CLIENT_EXIT;

/// Whether `c`'s terminal size has no say in how big a window is: it has no
/// session, it is on its way out, it was told to ignore its own size while some
/// other client was not, or it is a control client that has not reported a size
/// yet.
fn ignore_client_size(c: &client) -> bool {
    if c.attached_session().is_none() {
        return true;
    }
    if c.flags & CLIENT_NOSIZEFLAGS as uint64_t != 0 {
        return true;
    }
    if c.flags & CLIENT_IGNORESIZE as uint64_t != 0
        && with_clients(|clients| {
            clients.iter().any(|owner| {
                { owner.attached_session() }.is_some()
                    && unsafe { owner.flags() } & CLIENT_NOSIZEFLAGS as uint64_t == 0
                    && unsafe { owner.flags() } & CLIENT_IGNORESIZE as uint64_t == 0
            })
        })
    {
        return true;
    }
    c.flags & CLIENT_CONTROL as uint64_t != 0
        && c.flags & CLIENT_SIZECHANGED as uint64_t == 0
        && c.flags & CLIENT_WINDOWSIZECHANGED == 0
}

/// How many clients have a say in `w`'s size, counted no further than two,
/// which is all the latest-client policy needs to know.
unsafe fn clients_with_window(w: &WindowRef) -> u_int {
    unsafe {
        with_clients(|clients| {
            clients
                .iter()
                .filter(|c| {
                    !ignore_client_size(c.as_client())
                        && c.attached_session().is_some_and(|session| session.has(w))
                })
                .take(2)
                .count() as u_int
        })
    }
}

/// The size the clients settle on for a window.
///
/// `found` says whether any client had a say. When none did, `sx` and `sy` are
/// left at the starting values of the policy — zero for the largest size,
/// `UINT_MAX` for the smallest and the latest — and the caller writes them out
/// all the same before looking for a size of its own, which is what the C's
/// out-parameters did.
#[derive(Default)]
struct client_size {
    found: bool,
    sx: u_int,
    sy: u_int,
    xpixel: u_int,
    ypixel: u_int,
}

unsafe fn clients_calculate_size(
    type_0: c_int,
    c: Option<&client>,
    w: Option<&WindowRef>,
    skip_client: impl Fn(&client) -> bool,
) -> client_size {
    unsafe {
        let mut size = client_size::default();
        if type_0 == WINDOW_SIZE_LARGEST {
            size.sx = 0;
            size.sy = 0;
        } else if let Some(w) = w
            && type_0 == WINDOW_SIZE_MANUAL
        {
            size.sx = w.dimensions().manual_size.width;
            size.sy = w.dimensions().manual_size.height;
            log_debug(
                c"%s: manual size %ux%u",
                fmt_args![c"clients_calculate_size".as_ptr(), size.sx, size.sy],
            );
        } else {
            size.sx = UINT_MAX;
            size.sy = UINT_MAX;
        }

        let mut n = 0;
        if type_0 == WINDOW_SIZE_LATEST
            && let Some(w) = w
        {
            n = clients_with_window(w);
        }

        if type_0 != WINDOW_SIZE_MANUAL {
            with_clients(|clients| {
                for owner in clients {
                    let loop_0 = owner.as_client();
                    if c.is_none_or(|c| !core::ptr::eq(loop_0, c)) && ignore_client_size(loop_0) {
                        log_debug(
                            c"%s: ignoring %s (1)",
                            fmt_args![c"clients_calculate_size".as_ptr(), loop_0.name.as_deref()],
                        );
                    } else if c.is_none_or(|c| !core::ptr::eq(loop_0, c)) && skip_client(loop_0) {
                        log_debug(
                            c"%s: skipping %s (1)",
                            fmt_args![c"clients_calculate_size".as_ptr(), loop_0.name.as_deref()],
                        );
                    } else if type_0 == WINDOW_SIZE_LATEST
                        && n > 1
                        && window_get_latest(
                            &w.expect("the latest policy has a window").as_window(),
                        )
                        .is_none_or(|latest| !owner.ptr_eq(&latest))
                    {
                        log_debug(
                            c"%s: %s is not latest",
                            fmt_args![c"clients_calculate_size".as_ptr(), loop_0.name.as_deref()],
                        );
                    } else {
                        let (cx, cy) = match w.and_then(|w| loop_0.windows.get(&w.window_id())) {
                            Some(cw) if cw.sx != 0 && cw.sy != 0 => (cw.sx, cw.sy),
                            _ => (
                                loop_0.tty.sx,
                                loop_0.tty.sy.wrapping_sub(status_line_size(loop_0)),
                            ),
                        };
                        if type_0 == WINDOW_SIZE_LARGEST {
                            size.sx = size.sx.max(cx);
                            size.sy = size.sy.max(cy);
                        } else {
                            size.sx = size.sx.min(cx);
                            size.sy = size.sy.min(cy);
                        }
                        if loop_0.tty.xpixel > size.xpixel && loop_0.tty.ypixel > size.ypixel {
                            size.xpixel = loop_0.tty.xpixel;
                            size.ypixel = loop_0.tty.ypixel;
                        }
                        log_debug(
                            c"%s: after %s (%ux%u), size is %ux%u",
                            fmt_args![
                                c"clients_calculate_size".as_ptr(),
                                loop_0.name.as_deref(),
                                cx,
                                cy,
                                size.sx,
                                size.sy
                            ],
                        );
                    }
                }
            });
            log_calculated(&size);
        }

        if let Some(w) = w {
            with_clients(|clients| {
                for owner in clients {
                    let loop_0 = owner.as_client();
                    if c.is_none_or(|c| !core::ptr::eq(loop_0, c)) && ignore_client_size(loop_0) {
                        continue;
                    }
                    if c.is_none_or(|c| !core::ptr::eq(loop_0, c)) && skip_client(loop_0) {
                        continue;
                    }
                    if loop_0.flags & CLIENT_WINDOWSIZECHANGED == 0 {
                        continue;
                    }
                    let Some((cw_sx, cw_sy)) =
                        loop_0.windows.get(&w.window_id()).map(|cw| (cw.sx, cw.sy))
                    else {
                        continue;
                    };
                    log_debug(
                        c"%s: %s size for @%u is %ux%u",
                        fmt_args![
                            c"clients_calculate_size".as_ptr(),
                            loop_0.name.as_deref(),
                            w.window_id(),
                            cw_sx,
                            cw_sy
                        ],
                    );
                    if cw_sx != 0 && size.sx > cw_sx {
                        size.sx = cw_sx;
                    }
                    if cw_sy != 0 && size.sy > cw_sy {
                        size.sy = cw_sy;
                    }
                }
            });
        }
        log_calculated(&size);

        if type_0 == WINDOW_SIZE_MANUAL {
            log_debug(
                c"%s: type is manual",
                fmt_args![c"clients_calculate_size".as_ptr()],
            );
            size.found = w.is_some();
            return size;
        }
        if type_0 == WINDOW_SIZE_LARGEST {
            log_debug(
                c"%s: type is largest",
                fmt_args![c"clients_calculate_size".as_ptr()],
            );
            size.found = size.sx != 0 && size.sy != 0;
            return size;
        }
        if type_0 == WINDOW_SIZE_LATEST {
            log_debug(
                c"%s: type is latest",
                fmt_args![c"clients_calculate_size".as_ptr()],
            );
        } else {
            log_debug(
                c"%s: type is smallest",
                fmt_args![c"clients_calculate_size".as_ptr()],
            );
        }
        size.found = size.sx != UINT_MAX && size.sy != UINT_MAX;
        size
    }
}

/// Says in the log whether a size has been worked out yet.
fn log_calculated(size: &client_size) {
    if size.sx != UINT_MAX && size.sy != UINT_MAX {
        log_debug(
            c"%s: calculated size %ux%u",
            fmt_args![c"clients_calculate_size".as_ptr(), size.sx, size.sy],
        );
    } else {
        log_debug(
            c"%s: no calculated size",
            fmt_args![c"clients_calculate_size".as_ptr()],
        );
    }
}

/// For a new window: a client showing something else has no say. With no
/// window yet, a client of another session has none either.
fn default_window_size_skip_client(
    loop_0: &client,
    s: Option<&session>,
    w: Option<&WindowRef>,
) -> bool {
    {
        let attached = loop_0.attached_session();
        if let Some(w) = w {
            return attached.as_ref().is_none_or(|session| !session.has(w));
        }
        match s {
            Some(s) => attached
                .as_ref()
                .is_none_or(|session| !session.points_to(s)),
            None => attached.is_some(),
        }
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
    c: Option<&client>,
    s: &session,
    w: Option<&WindowRef>,
    mut type_0: c_int,
) -> (u_int, u_int, u_int, u_int) {
    unsafe {
        let (mut sx, mut sy, xpixel, ypixel);
        if type_0 == -1 {
            type_0 = (global_w_options
                .as_ref()
                .expect("global options are initialized"))
            .number(c"window-size") as c_int;
        }
        if type_0 == WINDOW_SIZE_LATEST
            && let Some(c) = c
            && !ignore_client_size(c)
        {
            sx = c.tty.sx;
            sy = c.tty.sy.wrapping_sub(status_line_size(c));
            xpixel = c.tty.xpixel;
            ypixel = c.tty.ypixel;
            log_debug(
                c"%s: using %ux%u from %s",
                fmt_args![c"default_window_size".as_ptr(), sx, sy, c.name.as_deref()],
            );
        } else {
            let c = c.filter(|c| !c.flags & CLIENT_CONTROL as uint64_t != 0);
            let size = clients_calculate_size(type_0, c, w, |client| {
                default_window_size_skip_client(client, Some(s), w)
            });
            sx = size.sx;
            sy = size.sy;
            xpixel = size.xpixel;
            ypixel = size.ypixel;
            if !size.found {
                let value = ((s).options_ref().clone()).string_ref(c"default-size");
                if sscanf(value.as_ptr(), c"%ux%u".as_ptr(), &raw mut sx, &raw mut sy) != 2 {
                    sx = 80;
                    sy = 24;
                }
                log_debug(
                    c"%s: using %ux%u from default-size",
                    fmt_args![c"default_window_size".as_ptr(), sx, sy],
                );
            }
        }
        sx = sx.clamp(WINDOW_MINIMUM as u_int, WINDOW_MAXIMUM as u_int);
        sy = sy.clamp(WINDOW_MINIMUM as u_int, WINDOW_MAXIMUM as u_int);
        log_debug(
            c"%s: resulting size is %ux%u",
            fmt_args![c"default_window_size".as_ptr(), sx, sy],
        );
        (sx, sy, xpixel, ypixel)
    }
}

/// For an existing window: a client whose session shows nothing has no say, and
/// under `aggressive-resize` only a client whose current window is this one
/// does.
fn recalculate_size_skip_client(loop_0: &client, current: c_int, w: &WindowRef) -> bool {
    {
        let Some(session) = loop_0.attached_session() else {
            return true;
        };
        let Some(link) = session.curw() else {
            return true;
        };
        if current != 0 {
            return link.window().is_none_or(|window| !window.ptr_eq(w));
        }
        !session.has(w)
    }
}

/// Recalculates every window's size, holding each one until the next redraw.
pub fn recalculate_sizes() {
    recalculate_sizes_now(0);
}

/// Recalculates every window's size, first counting how many clients each
/// session has attached and deciding which clients have room for a status line.
pub fn recalculate_sizes_now(now: c_int) {
    WINDOWS.with(|windows| unsafe {
        for s_ref in SESSIONS.read().values() {
            s_ref.clear_attached();
            s_ref.update_status_cache();
        }
        for mut c in client_walk() {
            let Some(session) = c.attached_session() else {
                continue;
            };
            if c.flags() & CLIENT_UNATTACHEDFLAGS as uint64_t == 0 {
                session.add_attached();
            }
            if !ignore_client_size(c.as_client()) {
                if c.as_tty().sy <= session.as_session().statuslines
                    || c.flags() & CLIENT_CONTROL as uint64_t != 0
                {
                    *c.flags_mut() |= CLIENT_STATUSOFF as uint64_t;
                } else {
                    *c.flags_mut() &= !(CLIENT_STATUSOFF as uint64_t);
                }
            }
        }
        for w_ref in windows.iter().filter_map(|(_, window)| window.upgrade()) {
            w_ref.recalculate_size(now);
        }
    });
}
#[cfg(test)]
#[path = "tests/test_resize.rs"]
mod tests;

impl WindowRef {
    /// Gives `w` a new size, resizing its layout tree and its panes with it.
    ///
    /// The size asked for is held between the window minimum and maximum, and then
    /// raised again to whatever the layout tree settled on, which may be more than
    /// was asked for when the panes in it cannot fit. A zoomed window is unzoomed
    /// for the resize and zoomed again after, so the layout underneath is the one
    /// that moves.
    pub unsafe fn resize_with_layout(
        &self,
        mut sx: u_int,
        mut sy: u_int,
        xpixel: c_int,
        ypixel: c_int,
    ) {
        let owner = self;

        unsafe {
            sx = sx.clamp(WINDOW_MINIMUM as u_int, WINDOW_MAXIMUM as u_int);
            sy = sy.clamp(WINDOW_MINIMUM as u_int, WINDOW_MAXIMUM as u_int);

            let zoomed = owner.as_window().flags & WINDOW_ZOOMED != 0;
            if zoomed {
                owner.unzoom(1);
            }

            owner.resize_layout(sx, sy);
            let mut payload = owner.as_window_mut();
            let w = &mut *payload;
            let root = w
                .layout_root
                .as_deref()
                .expect("a resized window has a layout");
            sx = sx.max(root.sx);
            sy = sy.max(root.sy);
            drop(payload);
            owner.resize(sx, sy, xpixel, ypixel);
            let payload = owner.as_window();
            let w = &*payload;
            let root = w
                .layout_root
                .as_deref()
                .expect("window resizing keeps its layout");
            log_debug(
                c"%s: @%u resized to %ux%u; layout %ux%u",
                fmt_args![c"resize_window", w.window_id(), sx, sy, root.sx, root.sy],
            );

            let active_id = w.active_pane_id();
            drop(payload);
            if zoomed && let Some(active_id) = active_id {
                owner.zoom(
                    &crate::window::window_pane_find_by_id(active_id)
                        .expect("the selected pane exists"),
                );
            }
            owner.update_client_offsets();
            owner.redraw();
            notify_window(c"window-layout-changed", Some(owner));
            notify_window(c"window-resized", Some(owner));
            owner.as_window_mut().flags &= !WINDOW_RESIZE;
        }
    }
    /// Works out what size the window should be and gives it that size, or notes
    /// it for the next redraw. `now` resizes at once rather than waiting.
    pub(crate) unsafe fn recalculate_size(&self, now: c_int) {
        let w_ref = self;

        unsafe {
            let w = w_ref.as_window();
            if window_active_pane(&w).is_none() {
                return;
            }
            log_debug(
                c"%s: @%u is %ux%u",
                fmt_args![
                    c"recalculate_size".as_ptr(),
                    w.window_id(),
                    w.dimensions().size.width,
                    w.dimensions().size.height
                ],
            );

            let type_0 = w.options_ref().number(c"window-size") as c_int;
            let current = w.options_ref().number(c"aggressive-resize") as c_int;
            let size = clients_calculate_size(type_0, None, Some(w_ref), |client| {
                recalculate_size_skip_client(client, current, w_ref)
            });

            let mut changed = size.found;
            if w.flags & WINDOW_RESIZE != 0 {
                if now == 0
                    && changed
                    && w.dimensions().pending_size.width == size.sx
                    && w.dimensions().pending_size.height == size.sy
                {
                    changed = false;
                }
            } else if now == 0
                && changed
                && w.dimensions().size.width == size.sx
                && w.dimensions().size.height == size.sy
            {
                changed = false;
            }

            if !changed {
                log_debug(
                    c"%s: @%u no size change",
                    fmt_args![c"recalculate_size".as_ptr(), w.window_id()],
                );
                drop(w);
                w_ref.update_client_offsets();
                return;
            }
            log_debug(
                c"%s: @%u new size %ux%u",
                fmt_args![
                    c"recalculate_size".as_ptr(),
                    w.window_id(),
                    size.sx,
                    size.sy
                ],
            );

            drop(w);
            let owner = w_ref.clone();
            if now != 0 || type_0 == WINDOW_SIZE_MANUAL {
                owner.resize_with_layout(
                    size.sx,
                    size.sy,
                    size.xpixel as c_int,
                    size.ypixel as c_int,
                );
            } else {
                let mut w = owner.as_window_mut();
                w.set_pending_size(crate::pane_resize::PaneSize {
                    width: size.sx,
                    height: size.sy,
                });
                w.set_pending_pixels(crate::window_dimensions::WindowPixelSize {
                    width: size.xpixel,
                    height: size.ypixel,
                });
                w.flags |= WINDOW_RESIZE;
                drop(w);
                owner.update_client_offsets();
            }
        }
    }
}
