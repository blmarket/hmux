use crate::src::cmd::queue::{cmdq_continue, cmdq_get_client};
use crate::src::ffi::libc::memcpy;
use crate::src::format::{format_create_defaults, format_free};
use crate::src::grid::grid_default_cell;
use crate::src::input::{input_free, input_init, input_parse_screen};
use crate::src::input_keys::{input_key, input_key_get_mouse};
use crate::src::job::{job_free, job_get_event, job_resize, job_run};
use crate::src::options::options_get_number;
use crate::src::reactor::{
    bufferevent_write, evbuffer_drain, evbuffer_get_length, evbuffer_pullup,
};
use crate::src::screen::screen_share_hyperlinks;
use crate::src::screen::{screen_free, screen_init, screen_resize, screen_set_default_cursor};
use crate::src::screen_write::{
    screen_write_box, screen_write_clearscreen, screen_write_cursormove, screen_write_fast_copy,
    screen_write_start, screen_write_stop,
};
use crate::src::server_client::{
    server_client_clear_overlay, server_client_overlay_range, server_client_set_overlay,
    server_client_unref,
};
use crate::src::server_fn::server_redraw_client;
use crate::src::shared::abi::ssize_t;
use crate::src::shared::abi::*;
use crate::src::shared::client::CLIENT_REDRAWOVERLAY;
use crate::src::shared::client::{
    client, overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
use crate::src::shared::colour::*;
use crate::src::shared::command::cmdq_item;
use crate::src::shared::display::visible_ranges;
use crate::src::shared::environment::environ;
use crate::src::shared::event::*;
use crate::src::shared::format::format_tree;
use crate::src::shared::grid::*;
use crate::src::shared::input::input_ctx;
use crate::src::shared::job::{job, job_update_callback, JobCompletion, JobExitStatus};
use crate::src::shared::job::{JOB_DEFAULTSHELL, JOB_KEEPWRITE, JOB_NOWAIT, JOB_PTY};
use crate::src::shared::key::key_event;
use crate::src::shared::key::*;
use crate::src::shared::layout::*;
use crate::src::shared::mouse::{
    mouse_event, MOUSE_BUTTON_1, MOUSE_BUTTON_3, MOUSE_MASK_BUTTONS, MOUSE_MASK_DRAG,
    MOUSE_MASK_META, MOUSE_MASK_MODIFIERS,
};
use crate::src::shared::options::options;
use crate::src::shared::pane::window_pane;
use crate::src::shared::popup::{POPUP_CLOSEANYKEY, POPUP_CLOSEEXIT, POPUP_CLOSEEXITZERO};
use crate::src::shared::screen::screen;
use crate::src::shared::screen_write::screen_write_ctx;
use crate::src::shared::session::session;
use crate::src::shared::signal::SIGHUP;
use crate::src::shared::style::*;
use crate::src::shared::tty::TTY_CTX_WINDOW_BIGGER;
pub use crate::src::shared::tty::{
    tty, tty_ctx, tty_ctx_redraw_cb, tty_ctx_set_client_cb, tty_style_ctx, tty_term,
};
use crate::src::style::colour::{
    colour_palette_free, colour_palette_from_option, colour_palette_init,
};
use crate::src::style::{style_apply, style_parse, style_set};
use crate::src::tmux::global_w_options;
use crate::src::tty::tty_resize;
use crate::src::tty_draw::tty_draw_line;
use std::cell::UnsafeCell;
use std::ffi::{CStr, CString};
use std::rc::Weak;

pub struct popup_data {
    published: bool,
    pub c: *mut client,
    pub item: *mut cmdq_item,
    pub flags: ::core::ffi::c_int,
    pub title: Option<std::ffi::CString>,
    pub style: Option<std::ffi::CString>,
    pub border_style: Option<std::ffi::CString>,
    pub border_cell: grid_cell,
    pub border_lines: box_lines,
    pub s: screen,
    pub defaults: grid_cell,
    pub palette: refbox::RefBox<colour_palette>,
    pub job: *mut job,
    pub ictx: *mut input_ctx,
    pub status: ::core::ffi::c_int,
    pub px: u_int,
    pub py: u_int,
    pub sx: u_int,
    pub sy: u_int,
    pub ppx: u_int,
    pub ppy: u_int,
    pub psx: u_int,
    pub psy: u_int,
    pub dragging: C2RustUnnamed_39,
    pub dx: u_int,
    pub dy: u_int,
    pub lx: u_int,
    pub ly: u_int,
    pub lb: u_int,
}

impl popup_data {
    pub fn empty() -> Self {
        Self {
            published: false,
            c: Default::default(),
            item: Default::default(),
            flags: Default::default(),
            title: Default::default(),
            style: Default::default(),
            border_style: Default::default(),
            border_cell: Default::default(),
            border_lines: Default::default(),
            s: screen::empty(),
            defaults: Default::default(),
            palette: refbox::RefBox::new(colour_palette::default()),
            job: Default::default(),
            ictx: Default::default(),
            status: Default::default(),
            px: Default::default(),
            py: Default::default(),
            sx: Default::default(),
            sy: Default::default(),
            ppx: Default::default(),
            ppy: Default::default(),
            psx: Default::default(),
            psy: Default::default(),
            dragging: Default::default(),
            dx: Default::default(),
            dy: Default::default(),
            lx: Default::default(),
            ly: Default::default(),
            lb: Default::default(),
        }
    }
}

// The overlay solely owns popup state. Dispatch borrows through weak handles;
// removing the owner invalidates observers while an active borrow defers Drop.
struct PopupOwner {
    state: refbox::RefBox<PopupState>,
}

struct PopupState {
    data: UnsafeCell<Box<popup_data>>,
}

#[derive(Clone)]
struct PopupHandle(refbox::Weak<PopupState>);

struct PopupGuard<'a> {
    state: refbox::Borrow<'a, PopupState>,
    handle: &'a PopupHandle,
}

impl PopupOwner {
    fn new(data: Box<popup_data>) -> Self {
        Self {
            state: refbox::RefBox::new(PopupState {
                data: UnsafeCell::new(data),
            }),
        }
    }

    fn handle(&self) -> PopupHandle {
        PopupHandle(self.state.downgrade())
    }
}

impl PopupHandle {
    fn upgrade(&self) -> Option<PopupGuard<'_>> {
        match self.0.try_borrow_mut() {
            Ok(state) => Some(PopupGuard {
                state,
                handle: self,
            }),
            Err(refbox::BorrowError::Dropped) => None,
            Err(refbox::BorrowError::Borrowed) => panic!("popup already borrowed during dispatch"),
        }
    }
}

impl PopupGuard<'_> {
    // The parser holds interior screen/input pointers. Project fields without
    // creating a whole-popup mutable reference over those existing aliases.
    unsafe fn as_ptr(&self) -> *mut popup_data {
        Box::as_mut_ptr(&mut *self.state.data.get())
    }

    fn handle(&self) -> PopupHandle {
        self.handle.clone()
    }

    fn is_current(&self, c: &client) -> bool {
        self.handle.0.is_alive()
            && c.overlay_data
                .as_ref()
                .and_then(|data| data.downcast_ref::<PopupOwner>())
                .is_some_and(|owner| self.handle.0.is(&owner.state))
    }
}

fn popup_check_callback(handle: PopupHandle) -> overlay_check_cb {
    Some(Box::new(move |_, px, py, nx| {
        handle
            .upgrade()
            .map_or_else(visible_ranges::default, |popup| unsafe {
                popup_check(&popup, px, py, nx)
            })
    }))
}

unsafe fn popup_restore_check(c: *mut client, popup: &PopupGuard) {
    if popup.is_current(&*c) {
        (*c).overlay_check = popup_check_callback(popup.handle());
    }
}

fn popup_optional_string(value: Option<&CStr>) -> Option<CString> {
    value.map(CStr::to_owned)
}
pub type C2RustUnnamed_39 = ::core::ffi::c_uint;
pub const SIZE: C2RustUnnamed_39 = 2;
pub const MOVE: C2RustUnnamed_39 = 1;
pub const OFF: C2RustUnnamed_39 = 0;
pub const NONE: C2RustUnnamed_40 = 0;
pub type C2RustUnnamed_40 = ::core::ffi::c_uint;
pub const BOTTOM: C2RustUnnamed_40 = 4;
pub const TOP: C2RustUnnamed_40 = 3;
pub const RIGHT: C2RustUnnamed_40 = 2;
pub const LEFT: C2RustUnnamed_40 = 1;

impl Drop for popup_data {
    fn drop(&mut self) {
        unsafe {
            // Startup failure does not resume the command: popup_display's
            // caller handles that error. Published overlays resume once.
            if self.published && !self.item.is_null() {
                let c = cmdq_get_client(self.item);
                if !c.is_null() && (*c).session.is_null() {
                    (*c).retval = self.status;
                }
                cmdq_continue(self.item);
            }
            if !self.c.is_null() {
                server_client_unref(self.c);
            }
            if !self.job.is_null() {
                job_free(self.job);
            }
            if !self.ictx.is_null() {
                input_free(self.ictx);
            }
            screen_free(&mut self.s);
            colour_palette_free(Some(
                &mut self
                    .palette
                    .try_borrow_mut()
                    .expect("unborrowed popup palette"),
            ));
        }
    }
}
unsafe fn popup_reapply_styles(popup: &PopupGuard) {
    let pd = popup.as_ptr();
    let mut c: *mut client = (*pd).c;
    let mut s: *mut session = (*c).session;
    let mut o: *mut options = ::core::ptr::null_mut::<options>();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut sytmp: style = style {
        gc: grid_cell {
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
        },
        ignore: 0,
        dim: 0,
        fill: 0,
        align: STYLE_ALIGN_DEFAULT,
        list: STYLE_LIST_OFF,
        range_type: STYLE_RANGE_NONE,
        range_argument: 0,
        range_string: [0; 16],
        width: 0,
        width_percentage: 0,
        pad: 0,
        default_type: STYLE_DEFAULT_BASE,
        link: 0,
    };
    if s.is_null() {
        return;
    }
    o = (*(*(*s).curw).window).options;
    ft = format_create_defaults(
        ::core::ptr::null_mut::<cmdq_item>(),
        c,
        s,
        (*s).curw,
        ::core::ptr::null_mut::<window_pane>(),
    );
    memcpy(
        &raw mut (*pd).defaults as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    style_apply(
        &raw mut (*pd).defaults,
        o,
        b"popup-style\0" as *const u8 as *const ::core::ffi::c_char,
        ft,
    );
    if !(*pd).style.is_none() {
        style_set(&raw mut sytmp, &raw const grid_default_cell);
        if style_parse(
            &raw mut sytmp,
            &raw mut (*pd).defaults,
            ((*pd).style)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        ) == 0 as ::core::ffi::c_int
        {
            (*pd).defaults.fg = sytmp.gc.fg;
            (*pd).defaults.bg = sytmp.gc.bg;
        }
    }
    (*pd).defaults.attr = 0 as u_short;
    memcpy(
        &raw mut (*pd).border_cell as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    style_apply(
        &raw mut (*pd).border_cell,
        o,
        b"popup-border-style\0" as *const u8 as *const ::core::ffi::c_char,
        ft,
    );
    if !(*pd).border_style.is_none() {
        style_set(&raw mut sytmp, &raw const grid_default_cell);
        if style_parse(
            &raw mut sytmp,
            &raw mut (*pd).border_cell,
            ((*pd).border_style)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        ) == 0 as ::core::ffi::c_int
        {
            (*pd).border_cell.fg = sytmp.gc.fg;
            (*pd).border_cell.bg = sytmp.gc.bg;
        }
    }
    (*pd).border_cell.attr = 0 as u_short;
    format_free(ft);
}
/// Geometry and defaults are fixed for one synchronous input batch. The palette
/// has a separate sole owner so each terminal command can snapshot the colours
/// after the parser's latest OSC update without reborrowing the popup itself.
#[derive(Clone)]
struct PopupRenderSnapshot {
    popup: PopupHandle,
    client: Weak<UnsafeCell<client>>,
    palette: refbox::Weak<colour_palette>,
    defaults: grid_cell,
    xoff: u_int,
    yoff: u_int,
}

impl PopupRenderSnapshot {
    unsafe fn new(popup: &PopupGuard<'_>) -> Self {
        let pd = popup.as_ptr();
        let border = u_int::from((*pd).border_lines != BOX_LINES_NONE);
        Self {
            popup: popup.handle(),
            client: if (*pd).c.is_null() {
                Weak::new()
            } else {
                crate::src::shared::rc::downgrade((*pd).c)
            },
            palette: (*pd).palette.downgrade(),
            defaults: (*pd).defaults,
            xoff: (*pd).px.wrapping_add(border),
            yoff: (*pd).py.wrapping_add(border),
        }
    }

    fn init_ctx(&self, ttyctx: &mut tty_ctx) {
        if !self.popup.0.is_alive() {
            ttyctx.set_client_cb = Some(Box::new(|_, _| 0));
            return;
        }
        let palette = match self.palette.try_borrow_mut() {
            Ok(palette) => palette,
            Err(refbox::BorrowError::Dropped) => {
                ttyctx.set_client_cb = Some(Box::new(|_, _| 0));
                return;
            }
            Err(refbox::BorrowError::Borrowed) => panic!("popup palette already borrowed"),
        };
        ttyctx.owned_palette = Some(Box::new(palette.clone()));
        ttyctx.style_ctx.palette = ttyctx.owned_palette.as_deref_mut().unwrap();
        ttyctx.style_ctx.defaults = self.defaults;
        ttyctx.flags &= !TTY_CTX_WINDOW_BIGGER;
        let redraw = self.clone();
        ttyctx.redraw_cb = Some(Box::new(move |_| {
            if redraw.popup.0.is_alive() {
                if let Some(client) = redraw.client.upgrade() {
                    unsafe {
                        (*client.get()).flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                    }
                }
            }
        }));
        let set_client = self.clone();
        ttyctx.set_client_cb = Some(Box::new(move |ttyctx, c| {
            if !set_client.popup.0.is_alive() {
                return 0;
            }
            let Some(client) = set_client.client.upgrade() else {
                return 0;
            };
            if client.get() != std::ptr::from_mut(c)
                || c.flags & CLIENT_REDRAWOVERLAY as uint64_t != 0
            {
                return 0;
            }
            ttyctx.wox = 0;
            ttyctx.woy = 0;
            ttyctx.wsx = c.tty.sx;
            ttyctx.wsy = c.tty.sy;
            ttyctx.rxoff = set_client.xoff as ::core::ffi::c_int;
            ttyctx.xoff = ttyctx.rxoff;
            ttyctx.ryoff = set_client.yoff as ::core::ffi::c_int;
            ttyctx.yoff = ttyctx.ryoff;
            1
        }));
    }
}

unsafe fn popup_mode(popup: &PopupGuard) -> Option<(std::ptr::NonNull<screen>, u_int, u_int)> {
    let pd = popup.as_ptr();
    if (*pd).border_lines as ::core::ffi::c_int == BOX_LINES_NONE as ::core::ffi::c_int {
        Some((
            std::ptr::NonNull::from(&mut (*pd).s),
            (*pd).px.wrapping_add((*pd).s.cx),
            (*pd).py.wrapping_add((*pd).s.cy),
        ))
    } else {
        Some((
            std::ptr::NonNull::from(&mut (*pd).s),
            (*pd).px.wrapping_add(1).wrapping_add((*pd).s.cx),
            (*pd).py.wrapping_add(1).wrapping_add((*pd).s.cy),
        ))
    }
}
unsafe fn popup_check(popup: &PopupGuard, px: u_int, py: u_int, nx: u_int) -> visible_ranges {
    let pd = popup.as_ptr();
    let mut ranges = visible_ranges::default();
    server_client_overlay_range(
        (*pd).px,
        (*pd).py,
        (*pd).sx,
        (*pd).sy,
        px,
        py,
        nx,
        &mut ranges,
    );
    ranges
}
unsafe fn popup_draw(c: *mut client, popup: &PopupGuard) {
    let pd = popup.as_ptr();
    let mut tty: *mut tty = &raw mut (*c).tty;
    let mut s: screen = screen::empty();
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: ::core::ptr::null_mut::<window_pane>(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        item: None,
        scrolled: 0,
        bg: 0,
    };
    let mut i: u_int = 0;
    let mut px: u_int = (*pd).px;
    let mut py: u_int = (*pd).py;
    let mut defaults: grid_cell = grid_cell {
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
    let mut style_ctx: tty_style_ctx = tty_style_ctx {
        defaults: grid_cell::default(),
        palette: ::core::ptr::null_mut::<colour_palette>(),
        dim: 0,
        hyperlinks: None,
    };
    popup_reapply_styles(popup);
    screen_init(&mut s, (*pd).sx, (*pd).sy, 0 as u_int);
    if (*pd).s.hyperlinks.is_some() {
        screen_share_hyperlinks(&mut s, &(*pd).s);
    }
    screen_write_start(&mut ctx, &raw mut s);
    screen_write_clearscreen(&mut ctx, 8 as u_int);
    if (*pd).border_lines as ::core::ffi::c_int == BOX_LINES_NONE as ::core::ffi::c_int {
        screen_write_cursormove(
            &mut ctx,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_fast_copy(
            &mut ctx,
            &(*pd).s,
            0 as u_int,
            0 as u_int,
            (*pd).sx,
            (*pd).sy,
        );
    } else if (*pd).sx > 2 as u_int && (*pd).sy > 2 as u_int {
        screen_write_box(
            &mut ctx,
            (*pd).sx,
            (*pd).sy,
            (*pd).border_lines,
            Some(&(*pd).border_cell),
            (*pd).title.as_deref(),
        );
        screen_write_cursormove(
            &mut ctx,
            1 as ::core::ffi::c_int,
            1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_fast_copy(
            &mut ctx,
            &(*pd).s,
            0 as u_int,
            0 as u_int,
            (*pd).sx.wrapping_sub(2 as u_int),
            (*pd).sy.wrapping_sub(2 as u_int),
        );
    }
    screen_write_stop(&mut ctx);
    memcpy(
        &raw mut defaults as *mut ::core::ffi::c_void,
        &raw mut (*pd).defaults as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    let mut palette = (*pd)
        .palette
        .try_borrow_mut()
        .expect("unborrowed popup palette");
    if defaults.fg == 8 as ::core::ffi::c_int {
        defaults.fg = palette.fg;
    }
    if defaults.bg == 8 as ::core::ffi::c_int {
        defaults.bg = palette.bg;
    }
    style_ctx.defaults = defaults;
    style_ctx.palette = &mut *palette;
    style_ctx.dim = 0 as u_int;
    style_ctx.hyperlinks = s.hyperlinks.clone();
    (*c).overlay_check = None;
    i = 0 as u_int;
    while i < (*pd).sy {
        tty_draw_line(
            tty,
            &s,
            0 as u_int,
            i,
            (*pd).sx,
            px,
            py.wrapping_add(i),
            Some(&style_ctx),
        );
        i = i.wrapping_add(1);
    }
    screen_free(&mut s);
    popup_restore_check(c, popup);
}
unsafe fn popup_resize(c: *mut client, popup: &PopupGuard) {
    let pd = popup.as_ptr();
    let mut tty: *mut tty = &raw mut (*c).tty;
    if (*pd).psy > (*tty).sy {
        (*pd).sy = (*tty).sy;
    } else {
        (*pd).sy = (*pd).psy;
    }
    if (*pd).psx > (*tty).sx {
        (*pd).sx = (*tty).sx;
    } else {
        (*pd).sx = (*pd).psx;
    }
    if (*pd).ppy.wrapping_add((*pd).sy) > (*tty).sy {
        (*pd).py = (*tty).sy.wrapping_sub((*pd).sy);
    } else {
        (*pd).py = (*pd).ppy;
    }
    if (*pd).ppx.wrapping_add((*pd).sx) > (*tty).sx {
        (*pd).px = (*tty).sx.wrapping_sub((*pd).sx);
    } else {
        (*pd).px = (*pd).ppx;
    }
    if (*pd).border_lines as ::core::ffi::c_int == BOX_LINES_NONE as ::core::ffi::c_int {
        screen_resize(&mut (*pd).s, (*pd).sx, (*pd).sy, 0 as ::core::ffi::c_int);
        if !(*pd).job.is_null() {
            job_resize((*pd).job, (*pd).sx, (*pd).sy);
        }
    } else if (*pd).sx > 2 as u_int && (*pd).sy > 2 as u_int {
        screen_resize(
            &mut (*pd).s,
            (*pd).sx.wrapping_sub(2 as u_int),
            (*pd).sy.wrapping_sub(2 as u_int),
            0 as ::core::ffi::c_int,
        );
        if !(*pd).job.is_null() {
            job_resize(
                (*pd).job,
                (*pd).sx.wrapping_sub(2 as u_int),
                (*pd).sy.wrapping_sub(2 as u_int),
            );
        }
    }
}
unsafe fn popup_handle_drag(mut c: *mut client, popup: &PopupGuard, mut m: *mut mouse_event) {
    let pd = popup.as_ptr();
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    if (*m).b & MOUSE_MASK_DRAG as u_int == 0 {
        (*pd).dragging = OFF;
    } else if (*pd).dragging as ::core::ffi::c_uint
        == MOVE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*m).x < (*pd).dx {
            px = 0 as u_int;
        } else if (*m).x.wrapping_sub((*pd).dx).wrapping_add((*pd).sx) > (*c).tty.sx {
            px = (*c).tty.sx.wrapping_sub((*pd).sx);
        } else {
            px = (*m).x.wrapping_sub((*pd).dx);
        }
        if (*m).y < (*pd).dy {
            py = 0 as u_int;
        } else if (*m).y.wrapping_sub((*pd).dy).wrapping_add((*pd).sy) > (*c).tty.sy {
            py = (*c).tty.sy.wrapping_sub((*pd).sy);
        } else {
            py = (*m).y.wrapping_sub((*pd).dy);
        }
        (*pd).px = px;
        (*pd).py = py;
        (*pd).dx = (*m).x.wrapping_sub((*pd).px);
        (*pd).dy = (*m).y.wrapping_sub((*pd).py);
        (*pd).ppx = px;
        (*pd).ppy = py;
        server_redraw_client(c);
    } else if (*pd).dragging as ::core::ffi::c_uint
        == SIZE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*pd).border_lines as ::core::ffi::c_int == BOX_LINES_NONE as ::core::ffi::c_int {
            if (*m).x < (*pd).px.wrapping_add(1 as u_int) {
                return;
            }
            if (*m).y < (*pd).py.wrapping_add(1 as u_int) {
                return;
            }
        } else {
            if (*m).x < (*pd).px.wrapping_add(3 as u_int) {
                return;
            }
            if (*m).y < (*pd).py.wrapping_add(3 as u_int) {
                return;
            }
        }
        (*pd).sx = (*m).x.wrapping_sub((*pd).px);
        (*pd).sy = (*m).y.wrapping_sub((*pd).py);
        (*pd).psx = (*pd).sx;
        (*pd).psy = (*pd).sy;
        if (*pd).border_lines as ::core::ffi::c_int == BOX_LINES_NONE as ::core::ffi::c_int {
            screen_resize(&mut (*pd).s, (*pd).sx, (*pd).sy, 0 as ::core::ffi::c_int);
            if !(*pd).job.is_null() {
                job_resize((*pd).job, (*pd).sx, (*pd).sy);
            }
        } else {
            screen_resize(
                &mut (*pd).s,
                (*pd).sx.wrapping_sub(2 as u_int),
                (*pd).sy.wrapping_sub(2 as u_int),
                0 as ::core::ffi::c_int,
            );
            if !(*pd).job.is_null() {
                job_resize(
                    (*pd).job,
                    (*pd).sx.wrapping_sub(2 as u_int),
                    (*pd).sy.wrapping_sub(2 as u_int),
                );
            }
        }
        server_redraw_client(c);
    }
}
unsafe fn popup_key(c: *mut client, popup: &PopupGuard, event: *mut key_event) -> i32 {
    let pd = popup.as_ptr();
    let mut current_block: u64;
    let mut m: *mut mouse_event = &raw mut (*event).m;
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut border: C2RustUnnamed_40 = NONE;
    if (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
        == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
        || (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
            >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                << 32 as ::core::ffi::c_int
            && (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int
    {
        if (*pd).dragging as ::core::ffi::c_uint != OFF as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            popup_handle_drag(c, popup, m);
            current_block = 9022331712714349549;
        } else {
            if (*m).x < (*pd).px
                || (*m).x > (*pd).px.wrapping_add((*pd).sx).wrapping_sub(1 as u_int)
                || (*m).y < (*pd).py
                || (*m).y > (*pd).py.wrapping_add((*pd).sy).wrapping_sub(1 as u_int)
            {
                return 0 as ::core::ffi::c_int;
            }
            if (*pd).border_lines as ::core::ffi::c_int != BOX_LINES_NONE as ::core::ffi::c_int {
                if (*m).x == (*pd).px {
                    border = LEFT;
                } else if (*m).x == (*pd).px.wrapping_add((*pd).sx).wrapping_sub(1 as u_int) {
                    border = RIGHT;
                } else if (*m).y == (*pd).py {
                    border = TOP;
                } else if (*m).y == (*pd).py.wrapping_add((*pd).sy).wrapping_sub(1 as u_int) {
                    border = BOTTOM;
                }
            }
            if (*m).b & MOUSE_MASK_MODIFIERS as u_int == 0 as u_int
                && (*m).b & MOUSE_MASK_BUTTONS as u_int == MOUSE_BUTTON_3 as u_int
                && (border as ::core::ffi::c_uint
                    == LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
                    || border as ::core::ffi::c_uint
                        == TOP as ::core::ffi::c_int as ::core::ffi::c_uint)
            {
                current_block = 9022331712714349549;
            } else if (*m).b & MOUSE_MASK_MODIFIERS as u_int == MOUSE_MASK_META as u_int
                || border as ::core::ffi::c_uint
                    != NONE as ::core::ffi::c_int as ::core::ffi::c_uint
                    && (*m).lb & MOUSE_MASK_DRAG as u_int == 0
            {
                if (*m).b & MOUSE_MASK_DRAG as u_int == 0 {
                    current_block = 9022331712714349549;
                } else {
                    if (*m).lb & MOUSE_MASK_BUTTONS as u_int == MOUSE_BUTTON_1 as u_int {
                        (*pd).dragging = MOVE;
                    } else if (*m).lb & MOUSE_MASK_BUTTONS as u_int == MOUSE_BUTTON_3 as u_int {
                        (*pd).dragging = SIZE;
                    }
                    (*pd).dx = (*m).lx.wrapping_sub((*pd).px);
                    (*pd).dy = (*m).ly.wrapping_sub((*pd).py);
                    current_block = 9022331712714349549;
                }
            } else {
                current_block = 13472856163611868459;
            }
        }
        match current_block {
            13472856163611868459 => {}
            _ => {
                (*pd).lx = (*m).x;
                (*pd).ly = (*m).y;
                (*pd).lb = (*m).b;
                return 0 as ::core::ffi::c_int;
            }
        }
    }
    if ((*pd).flags & (POPUP_CLOSEEXIT | POPUP_CLOSEEXITZERO) == 0 as ::core::ffi::c_int
        || (*pd).job.is_null())
        && ((*event).key == '\u{1b}' as i32 as key_code
            || (*event).key == 'c' as i32 as ::core::ffi::c_ulonglong | KEYC_CTRL)
    {
        return 1 as ::core::ffi::c_int;
    }
    if (*pd).job.is_null()
        && (*pd).flags & POPUP_CLOSEANYKEY != 0
        && !((*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
            == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
            || (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int
                && (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                    <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                        << 32 as ::core::ffi::c_int)
        && !((*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
            == (KEYC_TYPE_FUNCTION as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                << 32 as ::core::ffi::c_int
            && ((*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                == KEYC_PASTE_START as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
                || (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                    == KEYC_PASTE_END as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong))
    {
        return 1 as ::core::ffi::c_int;
    }
    if !(*pd).job.is_null() {
        if (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
            == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
            || (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int
                && (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                    <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                        << 32 as ::core::ffi::c_int
        {
            if (*pd).border_lines as ::core::ffi::c_int == BOX_LINES_NONE as ::core::ffi::c_int {
                px = (*m).x.wrapping_sub((*pd).px);
                py = (*m).y.wrapping_sub((*pd).py);
            } else {
                px = (*m).x.wrapping_sub((*pd).px).wrapping_sub(1 as u_int);
                py = (*m).y.wrapping_sub((*pd).py).wrapping_sub(1 as u_int);
            }
            let mut buf = [0; 40];
            let Some(len) = input_key_get_mouse(&raw mut (*pd).s, m, px, py, &mut buf) else {
                return 0 as ::core::ffi::c_int;
            };
            bufferevent_write(job_get_event((*pd).job), buf.as_ptr().cast(), len);
            return 0 as ::core::ffi::c_int;
        }
        input_key(&raw mut (*pd).s, job_get_event((*pd).job), (*event).key);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe fn popup_job_update_cb(job: &mut job, popup: &PopupGuard) {
    let pd = popup.as_ptr();
    let evb: &mut evbuffer = &mut *(*job_get_event(job as *mut job)).input;
    let mut c: *mut client = (*pd).c;
    let mut s: *mut screen = &raw mut (*pd).s;
    let mut data: *mut ::core::ffi::c_void = evbuffer_pullup(evb, -1)
        .map_or(std::ptr::null_mut(), |bytes| bytes.as_mut_ptr())
        as *mut ::core::ffi::c_void;
    let mut size: size_t = evbuffer_get_length(&*(evb));
    if size == 0 as size_t {
        return;
    }
    (*c).overlay_check = None;
    let render = PopupRenderSnapshot::new(popup);
    input_parse_screen(
        (*pd).ictx,
        s,
        Some(Box::new(move |ttyctx| render.init_ctx(ttyctx))),
        data as *const u_char,
        size,
    );
    popup_restore_check(c, popup);
    evbuffer_drain(evb, size);
}
unsafe fn popup_job_complete_cb(completion: JobCompletion, popup: &PopupGuard) {
    let pd = popup.as_ptr();
    (*pd).status = match completion.status {
        JobExitStatus::Exited(code) => code,
        JobExitStatus::Signaled(signal) => signal,
        JobExitStatus::Other(_) => 0,
    };
    (*pd).job = ::core::ptr::null_mut::<job>();
    if (*pd).flags & POPUP_CLOSEEXIT != 0
        || (*pd).flags & POPUP_CLOSEEXITZERO != 0 && (*pd).status == 0 as ::core::ffi::c_int
    {
        if popup.is_current(&*(*pd).c) {
            server_client_clear_overlay((*pd).c);
        }
    }
}
pub unsafe fn popup_present(mut c: *mut client) -> ::core::ffi::c_int {
    return (*c)
        .overlay_data
        .as_ref()
        .is_some_and(|data| data.is::<PopupOwner>()) as ::core::ffi::c_int;
}
pub unsafe fn popup_modify(
    mut c: *mut client,
    mut title: *const ::core::ffi::c_char,
    mut style: *const ::core::ffi::c_char,
    mut border_style: *const ::core::ffi::c_char,
    mut lines: box_lines,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let Some(handle) = (*c)
        .overlay_data
        .as_ref()
        .and_then(|data| data.downcast_ref::<PopupOwner>())
        .map(PopupOwner::handle)
    else {
        return -1;
    };
    let Some(popup) = handle.upgrade() else {
        return -1;
    };
    let pd = popup.as_ptr();
    let mut sytmp: style = style {
        gc: grid_cell {
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
        },
        ignore: 0,
        dim: 0,
        fill: 0,
        align: STYLE_ALIGN_DEFAULT,
        list: STYLE_LIST_OFF,
        range_type: STYLE_RANGE_NONE,
        range_argument: 0,
        range_string: [0; 16],
        width: 0,
        width_percentage: 0,
        pad: 0,
        default_type: STYLE_DEFAULT_BASE,
        link: 0,
    };
    if !title.is_null() {
        let updated = popup_optional_string(Some(CStr::from_ptr(title)));
        (*pd).title = updated;
    }
    if !border_style.is_null() {
        let updated = popup_optional_string(Some(CStr::from_ptr(border_style)));
        (*pd).border_style = updated;

        style_set(&raw mut sytmp, &raw mut (*pd).border_cell);
        if style_parse(
            &raw mut sytmp,
            &raw mut (*pd).border_cell,
            ((*pd).border_style)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        ) == 0 as ::core::ffi::c_int
        {
            (*pd).border_cell.fg = sytmp.gc.fg;
            (*pd).border_cell.bg = sytmp.gc.bg;
        }
    }
    if !style.is_null() {
        let updated = popup_optional_string(Some(CStr::from_ptr(style)));
        (*pd).style = updated;

        style_set(&raw mut sytmp, &raw mut (*pd).defaults);
        if style_parse(
            &raw mut sytmp,
            &raw mut (*pd).defaults,
            ((*pd).style)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        ) == 0 as ::core::ffi::c_int
        {
            (*pd).defaults.fg = sytmp.gc.fg;
            (*pd).defaults.bg = sytmp.gc.bg;
        }
    }
    if lines as ::core::ffi::c_int != BOX_LINES_DEFAULT as ::core::ffi::c_int {
        if lines as ::core::ffi::c_int == BOX_LINES_NONE as ::core::ffi::c_int
            && (*pd).border_lines as ::core::ffi::c_int != lines as ::core::ffi::c_int
        {
            screen_resize(&mut (*pd).s, (*pd).sx, (*pd).sy, 1 as ::core::ffi::c_int);
            job_resize((*pd).job, (*pd).sx, (*pd).sy);
        } else if (*pd).border_lines as ::core::ffi::c_int == BOX_LINES_NONE as ::core::ffi::c_int
            && (*pd).border_lines as ::core::ffi::c_int != lines as ::core::ffi::c_int
        {
            screen_resize(
                &mut (*pd).s,
                (*pd).sx.wrapping_sub(2 as u_int),
                (*pd).sy.wrapping_sub(2 as u_int),
                1 as ::core::ffi::c_int,
            );
            job_resize(
                (*pd).job,
                (*pd).sx.wrapping_sub(2 as u_int),
                (*pd).sy.wrapping_sub(2 as u_int),
            );
        }
        (*pd).border_lines = lines;
        tty_resize(&raw mut (*c).tty);
    }
    if flags != -(1 as ::core::ffi::c_int) {
        (*pd).flags = flags;
    }
    server_redraw_client(c);
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn popup_display(
    mut flags: ::core::ffi::c_int,
    mut lines: box_lines,
    mut item: *mut cmdq_item,
    mut px: u_int,
    mut py: u_int,
    mut sx: u_int,
    mut sy: u_int,
    env: Option<&environ>,
    mut shellcmd: *const ::core::ffi::c_char,
    argv: &Vec<CString>,
    mut cwd: *const ::core::ffi::c_char,
    mut title: *const ::core::ffi::c_char,
    mut c: *mut client,
    mut s: *mut session,
    mut style: *const ::core::ffi::c_char,
    mut border_style: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut jx: u_int = 0;
    let mut jy: u_int = 0;
    let mut o: *mut options = ::core::ptr::null_mut::<options>();
    let mut sytmp: style = style {
        gc: grid_cell {
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
        },
        ignore: 0,
        dim: 0,
        fill: 0,
        align: STYLE_ALIGN_DEFAULT,
        list: STYLE_LIST_OFF,
        range_type: STYLE_RANGE_NONE,
        range_argument: 0,
        range_string: [0; 16],
        width: 0,
        width_percentage: 0,
        pad: 0,
        default_type: STYLE_DEFAULT_BASE,
        link: 0,
    };
    if !s.is_null() {
        o = (*(*(*s).curw).window).options;
    } else {
        o = (*(*(*(*c).session).curw).window).options;
    }
    if lines as ::core::ffi::c_int == BOX_LINES_DEFAULT as ::core::ffi::c_int {
        lines = options_get_number(
            o,
            b"popup-border-lines\0" as *const u8 as *const ::core::ffi::c_char,
        ) as box_lines;
    }
    if lines as ::core::ffi::c_int == BOX_LINES_NONE as ::core::ffi::c_int {
        if sx < 1 as u_int || sy < 1 as u_int {
            return -(1 as ::core::ffi::c_int);
        }
        jx = sx;
        jy = sy;
    } else {
        if sx < 3 as u_int || sy < 3 as u_int {
            return -(1 as ::core::ffi::c_int);
        }
        jx = sx.wrapping_sub(2 as u_int);
        jy = sy.wrapping_sub(2 as u_int);
    }
    if (*c).tty.sx < sx || (*c).tty.sy < sy {
        return -(1 as ::core::ffi::c_int);
    }
    let mut data = Box::new(popup_data::empty());
    data.title = popup_optional_string((!title.is_null()).then(|| CStr::from_ptr(title)));
    data.style = popup_optional_string((!style.is_null()).then(|| CStr::from_ptr(style)));
    data.border_style =
        popup_optional_string((!border_style.is_null()).then(|| CStr::from_ptr(border_style)));
    let owner = PopupOwner::new(data);
    let handle = owner.handle();
    let popup = handle.upgrade().expect("new popup");
    let pd = popup.as_ptr();
    (*pd).item = item;
    (*pd).flags = flags;
    (*pd).c = c;
    crate::src::shared::rc::retain((*pd).c);
    (*pd).status = 128 as ::core::ffi::c_int + SIGHUP;
    (*pd).border_lines = lines;
    memcpy(
        &raw mut (*pd).border_cell as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    style_apply(
        &raw mut (*pd).border_cell,
        o,
        b"popup-border-style\0" as *const u8 as *const ::core::ffi::c_char,
        ::core::ptr::null_mut::<format_tree>(),
    );
    if !border_style.is_null() {
        style_set(&raw mut sytmp, &raw const grid_default_cell);
        if style_parse(
            &raw mut sytmp,
            &raw mut (*pd).border_cell,
            ((*pd).border_style)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        ) == 0 as ::core::ffi::c_int
        {
            (*pd).border_cell.fg = sytmp.gc.fg;
            (*pd).border_cell.bg = sytmp.gc.bg;
        }
    }
    (*pd).border_cell.attr = 0 as u_short;
    screen_init(&mut (*pd).s, jx, jy, 0 as u_int);
    screen_set_default_cursor(&mut (*pd).s, global_w_options);
    {
        let mut palette = (*pd).palette.try_borrow_mut().expect("new popup palette");
        colour_palette_init(&mut palette);
        colour_palette_from_option(Some(&mut palette), global_w_options);
    }
    memcpy(
        &raw mut (*pd).defaults as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    style_apply(
        &raw mut (*pd).defaults,
        o,
        b"popup-style\0" as *const u8 as *const ::core::ffi::c_char,
        ::core::ptr::null_mut::<format_tree>(),
    );
    if !style.is_null() {
        style_set(&raw mut sytmp, &raw const grid_default_cell);
        if style_parse(
            &raw mut sytmp,
            &raw mut (*pd).defaults,
            ((*pd).style)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        ) == 0 as ::core::ffi::c_int
        {
            (*pd).defaults.fg = sytmp.gc.fg;
            (*pd).defaults.bg = sytmp.gc.bg;
        }
    }
    (*pd).defaults.attr = 0 as u_short;
    (*pd).px = px;
    (*pd).py = py;
    (*pd).sx = sx;
    (*pd).sy = sy;
    (*pd).ppx = px;
    (*pd).ppy = py;
    (*pd).psx = sx;
    (*pd).psy = sy;
    let update = owner.handle();
    let complete = owner.handle();
    (*pd).job = job_run(
        (!shellcmd.is_null()).then(|| CStr::from_ptr(shellcmd)),
        argv,
        env,
        s,
        (!cwd.is_null()).then(|| CStr::from_ptr(cwd)),
        job_update_callback(move |job| unsafe {
            if let Some(popup) = update.upgrade() {
                popup_job_update_cb(job, &popup);
            }
        }),
        Some(Box::new(move |completion| unsafe {
            if let Some(popup) = complete.upgrade() {
                popup_job_complete_cb(completion, &popup);
            }
        })),
        None,
        JOB_NOWAIT | JOB_PTY | JOB_KEEPWRITE | JOB_DEFAULTSHELL,
        jx as ::core::ffi::c_int,
        jy as ::core::ffi::c_int,
    );
    if (*pd).job.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    (*pd).ictx = input_init(
        ::core::ptr::null_mut::<window_pane>(),
        job_get_event((*pd).job),
        (*pd).palette.as_ptr().cast_mut(),
        c,
    );
    (*pd).published = true;
    let check_cb = popup_check_callback(owner.handle());
    let mode = owner.handle();
    let mode_cb: overlay_mode_cb = Some(Box::new(move |_| {
        mode.upgrade()
            .and_then(|popup| unsafe { popup_mode(&popup) })
    }));
    let draw = owner.handle();
    let draw_cb: overlay_draw_cb = Some(Box::new(move |c| unsafe {
        if let Some(popup) = draw.upgrade() {
            popup_draw(c as *mut client, &popup);
        }
    }));
    let key = owner.handle();
    let key_cb: overlay_key_cb = Some(Box::new(move |c, event| unsafe {
        key.upgrade().map_or(0, |popup| popup_key(c, &popup, event))
    }));
    let free_cb: overlay_free_cb = None;
    let resize = owner.handle();
    let resize_cb: overlay_resize_cb = Some(Box::new(move |c| unsafe {
        if let Some(popup) = resize.upgrade() {
            popup_resize(c as *mut client, &popup);
        }
    }));
    drop(popup);
    server_client_set_overlay(
        c,
        check_cb,
        mode_cb,
        draw_cb,
        key_cb,
        free_cb,
        resize_cb,
        Box::new(owner),
    );
    return 0 as ::core::ffi::c_int;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::src::server_client::{
        server_client_overlay_check, server_client_overlay_draw, server_client_overlay_key,
        server_client_overlay_mode, server_client_overlay_resize,
    };
    use crate::src::shared::command::CMDQ_WAITING;

    fn owner() -> PopupOwner {
        let mut data = Box::new(popup_data::empty());
        data.title = Some(c"active popup".to_owned());
        PopupOwner::new(data)
    }

    #[test]
    fn owner_close_invalidates_observers_and_defers_cleanup_until_dispatch_finishes() {
        unsafe {
            let mut item = Box::new(cmdq_item::empty());
            item.flags = CMDQ_WAITING;
            let mut data = Box::new(popup_data::empty());
            data.item = &mut *item;
            data.published = true;
            let original = Box::as_mut_ptr(&mut data);
            let owner = PopupOwner::new(data);
            let handle = owner.handle();
            let guard = handle.upgrade().unwrap();
            assert_eq!(guard.as_ptr(), original);
            drop(owner);
            assert!(handle.upgrade().is_none());
            assert_eq!(item.flags & CMDQ_WAITING, CMDQ_WAITING);
            (*guard.as_ptr()).status = 7;
            assert_eq!((*guard.as_ptr()).status, 7);
            drop(guard);
            assert_eq!(item.flags & CMDQ_WAITING, 0);
            assert!(!handle.0.is_alive());
        }
    }

    #[test]
    fn unpublished_startup_drop_does_not_resume_the_callers_command() {
        let mut item = Box::new(cmdq_item::empty());
        item.flags = CMDQ_WAITING;
        let mut data = Box::new(popup_data::empty());
        data.item = &mut *item;
        let owner = PopupOwner::new(data);
        let handle = owner.handle();
        drop(owner);
        assert_eq!(item.flags & CMDQ_WAITING, CMDQ_WAITING);
        assert!(!handle.0.is_alive());
    }

    #[test]
    fn terminal_contexts_own_palette_snapshots_without_retaining_the_popup() {
        unsafe {
            let owner = owner();
            let handle = owner.handle();
            let guard = handle.upgrade().unwrap();
            let source = &(*guard.as_ptr()).palette;
            source.try_borrow_mut().unwrap().fg = 3;
            crate::src::style::colour::colour_palette_set(
                Some(&mut source.try_borrow_mut().unwrap()),
                1,
                7,
            );
            let render = PopupRenderSnapshot::new(&guard);
            let mut first = tty_ctx::default();
            render.init_ctx(&mut first);
            assert_ne!(first.style_ctx.palette.cast_const(), source.as_ptr());
            assert_eq!((*first.style_ctx.palette).fg, 3);
            source.try_borrow_mut().unwrap().fg = 5;
            let mut second = tty_ctx::default();
            render.init_ctx(&mut second);
            assert_eq!((*second.style_ctx.palette).fg, 5);
            assert_eq!((*first.style_ctx.palette).fg, 3);
            drop(owner);
            drop(guard);
            assert!(!handle.0.is_alive());
            assert!(!render.palette.is_alive());
            assert_eq!(
                crate::src::style::colour::colour_palette_get(Some(&*first.style_ctx.palette), 1),
                7
            );
            let mut client = client::empty();
            let mut set_client = first.set_client_cb.take().unwrap();
            assert_eq!(set_client(&mut first, &mut client), 0);
            (first.redraw_cb.as_ref().unwrap())(&first);
            let mut expired = tty_ctx::default();
            render.init_ctx(&mut expired);
            assert!(expired.style_ctx.palette.is_null());
            assert!(expired.owned_palette.is_none());
        }
    }

    unsafe fn close_during_dispatch<'a>(handle: &'a PopupHandle, c: &mut client) -> PopupGuard<'a> {
        let popup = handle.upgrade().unwrap();
        server_client_clear_overlay(c);
        assert!(handle.upgrade().is_none());
        assert_eq!((*popup.as_ptr()).title.as_deref(), Some(c"active popup"));
        popup
    }

    #[test]
    fn every_overlay_dispatch_holds_the_owner_across_self_close() {
        unsafe {
            for kind in 0..5 {
                let mut client = Box::new(client::empty());
                let owner = owner();
                let handle = owner.handle();
                let observer = handle.clone();
                client.overlay_data = Some(Box::new(owner));
                client.overlay_draw = Some(Box::new(|_| {}));

                match kind {
                    0 => {
                        client.overlay_draw = Some(Box::new(move |c| {
                            let _active = close_during_dispatch(&handle, c);
                        }));
                        server_client_overlay_draw(&mut *client);
                        assert!(client.overlay_draw.is_none());
                    }
                    1 => {
                        client.overlay_key = Some(Box::new(move |c, _| {
                            let _active = close_during_dispatch(&handle, c);
                            1
                        }));
                        let mut event = key_event::new(0, mouse_event::default(), None);
                        assert_eq!(server_client_overlay_key(&mut *client, &mut event), Some(0));
                        assert!(client.overlay_key.is_none());
                    }
                    2 => {
                        client.overlay_resize = Some(Box::new(move |c| {
                            let _active = close_during_dispatch(&handle, c);
                        }));
                        server_client_overlay_resize(&mut *client);
                        assert!(client.overlay_resize.is_none());
                    }
                    3 => {
                        client.overlay_mode = Some(Box::new(move |c| {
                            let active = close_during_dispatch(&handle, c);
                            popup_mode(&active)
                        }));
                        assert!(server_client_overlay_mode(&mut *client).is_none());
                        assert!(client.overlay_mode.is_none());
                    }
                    _ => {
                        client.overlay_check = Some(Box::new(move |c, x, y, n| {
                            let active = close_during_dispatch(&handle, c);
                            popup_check(&active, x, y, n)
                        }));
                        assert!(server_client_overlay_check(&mut *client, 0, 0, 10).is_none());
                        assert!(client.overlay_check.is_none());
                    }
                }
                assert!(client.overlay_data.is_none());
                assert!(!observer.0.is_alive());
            }
        }
    }

    #[test]
    fn completion_closes_once_preserves_status_and_releases_the_retained_client() {
        use crate::src::shared::rc;
        unsafe {
            let c = rc::new(client::empty());
            let client_observer = rc::downgrade(c);
            let mut item = Box::new(cmdq_item::empty());
            item.client = c;
            item.flags = CMDQ_WAITING;
            let mut data = Box::new(popup_data::empty());
            rc::retain(c);
            data.c = c;
            data.item = &mut *item;
            data.flags = POPUP_CLOSEEXIT;
            data.published = true;
            let owner = PopupOwner::new(data);
            let handle = owner.handle();
            (*c).overlay_data = Some(Box::new(owner));
            (*c).overlay_draw = Some(Box::new(|_| {}));

            popup_job_complete_cb(
                JobCompletion {
                    status: JobExitStatus::Exited(42),
                    output: Vec::new(),
                },
                &handle.upgrade().unwrap(),
            );
            assert_eq!((*c).retval, 42);
            assert_eq!(item.flags & CMDQ_WAITING, 0);
            assert!(!handle.0.is_alive());
            assert!((*c).overlay_data.is_none());
            rc::release(c);
            assert!(client_observer.upgrade().is_some());
            crate::src::reactor::shutdown_runtime();
            assert!(client_observer.upgrade().is_none());
        }
    }
}
