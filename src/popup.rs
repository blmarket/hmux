use crate::src::cmd::queue::{cmdq_continue, cmdq_get_client};
use crate::src::ffi::libc::memcpy;
use crate::src::format::{format_create_defaults, format_free};
use crate::src::grid::grid_default_cell;
use crate::src::input::{input_free, input_init, input_parse_screen};
use crate::src::input_keys::{input_key, input_key_get_mouse};
use crate::src::job::{job_free, job_get_event, job_resize, job_run};
use crate::src::options::options_get_number;
use crate::src::reactor::{
    bufferevent_get_input, bufferevent_write, evbuffer_drain, evbuffer_get_length, evbuffer_pullup,
};
use crate::src::screen::screen_share_hyperlinks;
use crate::src::screen::{screen_free, screen_init, screen_resize, screen_set_default_cursor};
use crate::src::screen_write::{
    screen_write_box, screen_write_clearscreen, screen_write_cursormove, screen_write_fast_copy,
    screen_write_start, screen_write_stop,
};
use crate::src::server_client::server_client_unref_owned;
use crate::src::server_client::{server_client_overlay_range, Client};
use crate::src::session::Session;
use crate::src::shared::abi::ssize_t;
use crate::src::shared::abi::*;
use crate::src::shared::client::{
    client, overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
use crate::src::shared::client::{ClientRef, ClientWeak};
use crate::src::shared::client::{CLIENT_ALLREDRAWFLAGS, CLIENT_REDRAWOVERLAY};
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
use crate::src::shared::screen::{screen, ScreenMode};
use crate::src::shared::screen_write::screen_write_ctx;
use crate::src::shared::session::session;
use crate::src::shared::session::SessionRef;
use crate::src::shared::signal::SIGHUP;
use crate::src::shared::style::*;
use crate::src::shared::tty::TTY_CTX_WINDOW_BIGGER;
pub use crate::src::shared::tty::{
    tty, tty_ctx, tty_ctx_redraw_cb, tty_ctx_set_client_cb, tty_style_ctx, tty_term,
};
use crate::src::style::colour::{
    colour_palette_free, colour_palette_from_option, colour_palette_init,
};
use crate::src::style::{style_apply_with_options, style_parse, style_set};
use crate::src::tmux::global_w_options;
use crate::src::window::Window;
use hmux_buffer::SegmentedBuf;
use std::cell::UnsafeCell;
use std::ffi::{CStr, CString};
use std::rc::Rc;
use std::rc::Weak;

pub struct popup_data {
    published: bool,
    pub c: Option<ClientRef>,
    pub item: Weak<UnsafeCell<cmdq_item>>,
    pub flags: ::core::ffi::c_int,
    pub title: Option<std::ffi::CString>,
    pub style: Option<std::ffi::CString>,
    pub border_style: Option<std::ffi::CString>,
    pub border_cell: grid_cell,
    pub border_lines: box_lines,
    pub s: screen,
    pub defaults: grid_cell,
    pub palette: refbox::RefBox<colour_palette>,
    pub job: refbox::Weak<job>,
    pub ictx: Option<Box<input_ctx>>,
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
struct PopupState {
    data: UnsafeCell<Box<popup_data>>,
}

#[derive(Clone)]
struct PopupHandle(refbox::Weak<PopupState>);

struct PopupGuard<'a> {
    state: refbox::Borrow<'a, PopupState>,
    handle: &'a PopupHandle,
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

    unsafe fn is_current(&self, client: &ClientRef) -> bool {
        self.handle.0.is_alive()
            && client.with_overlay_data(|data| {
                data.and_then(|data| data.downcast_ref::<refbox::RefBox<PopupState>>())
                    .is_some_and(|owner| self.handle.0.is(owner))
            })
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
            if self.published {
                if let Some(item) = self.item.upgrade() {
                    let c_owner = cmdq_get_client((item.get()).as_ref());
                    if let Some(client) = c_owner.as_ref() {
                        if client.attached_session().upgrade().is_none() {
                            client.set_return_value(self.status);
                        }
                    }
                    cmdq_continue(
                        &(*(item.get()))
                            .observer
                            .upgrade()
                            .expect("live command queue item"),
                    );
                }
            }
            if let Some(client) = self.c.take() {
                server_client_unref_owned(client);
            }
            if !self.job.is_empty() {
                job_free(&self.job);
            }
            if let Some(ictx) = self.ictx.take() {
                input_free(ictx);
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
    let client = (*pd).c.as_ref().expect("popup client");
    let Some(session) = client.attached_session().upgrade() else {
        return;
    };
    let link = session.current_winlink();
    let window = link
        .get_unchecked()
        .window_handle()
        .cloned()
        .expect("popup window");
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
    let mut ft_owner = format_create_defaults(None, Some(client), Some(&session), link, None);
    ft = &raw mut *ft_owner;
    memcpy(
        &raw mut (*pd).defaults as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    style_apply_with_options(
        &mut (*pd).defaults,
        c"popup-style",
        Some(&mut *ft),
        |read| window.with_options_mut(read),
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
    style_apply_with_options(
        &mut (*pd).border_cell,
        c"popup-border-style",
        Some(&mut *ft),
        |read| window.with_options_mut(read),
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
    window.release(c"popup styles");
    format_free(ft_owner);
}
/// Geometry and defaults are fixed for one synchronous input batch. The palette
/// has a separate sole owner so each terminal command can snapshot the colours
/// after the parser's latest OSC update without reborrowing the popup itself.
#[derive(Clone)]
struct PopupRenderSnapshot {
    popup: PopupHandle,
    client: ClientWeak,
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
            client: (*pd).c.as_ref().map_or_else(Weak::new, Rc::downgrade),
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
        ttyctx.style_ctx.palette =
            crate::src::shared::tty::PaletteSource::Snapshot(Box::new(palette.clone()));
        ttyctx.style_ctx.defaults = self.defaults;
        ttyctx.flags &= !TTY_CTX_WINDOW_BIGGER;
        let redraw = self.clone();
        ttyctx.redraw_cb = Some(Box::new(move |_| {
            if redraw.popup.0.is_alive() {
                if let Some(client) = redraw.client.upgrade() {
                    unsafe {
                        client.request_redraw(CLIENT_REDRAWOVERLAY as u64);
                    }
                }
            }
        }));
        let set_client = self.clone();
        ttyctx.set_client_cb = Some(Box::new(move |ttyctx, c| unsafe {
            if !set_client.popup.0.is_alive() {
                return 0;
            }
            let Some(client) = set_client.client.upgrade() else {
                return 0;
            };
            if !Rc::ptr_eq(&client, c) {
                return 0;
            }
            i32::from(c.prepare_overlay_render(ttyctx, set_client.xoff, set_client.yoff))
        }));
    }
}

unsafe fn popup_mode(popup: &PopupGuard) -> Option<(ScreenMode, u_int, u_int)> {
    let pd = popup.as_ptr();
    if (*pd).border_lines as ::core::ffi::c_int == BOX_LINES_NONE as ::core::ffi::c_int {
        Some((
            ScreenMode::from(&(*pd).s),
            (*pd).px.wrapping_add((*pd).s.cx),
            (*pd).py.wrapping_add((*pd).s.cy),
        ))
    } else {
        Some((
            ScreenMode::from(&(*pd).s),
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
unsafe fn popup_draw(c_owner: &ClientRef, popup: &PopupGuard) {
    let pd = popup.as_ptr();
    let mut s: screen = screen::empty();
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: std::rc::Weak::new(),
        target: Default::default(),
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
        palette: crate::src::shared::tty::PaletteSource::None,
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
    style_ctx.palette = crate::src::shared::tty::PaletteSource::Popup((*pd).palette.downgrade());
    drop(palette);
    style_ctx.dim = 0 as u_int;
    style_ctx.hyperlinks = s.hyperlinks.clone();
    c_owner.with_overlay_check_disabled(popup_check_callback(popup.handle()), || {
        c_owner.draw_overlay_screen(&s, px, py, (*pd).sx, (*pd).sy, &style_ctx);
        screen_free(&mut s);
    });
}
unsafe fn popup_resize(c_owner: &ClientRef, popup: &PopupGuard) {
    let (terminal_sx, terminal_sy) = c_owner.terminal_size();
    let pd = popup.as_ptr();
    if (*pd).psy > terminal_sy {
        (*pd).sy = terminal_sy;
    } else {
        (*pd).sy = (*pd).psy;
    }
    if (*pd).psx > terminal_sx {
        (*pd).sx = terminal_sx;
    } else {
        (*pd).sx = (*pd).psx;
    }
    if (*pd).ppy.wrapping_add((*pd).sy) > terminal_sy {
        (*pd).py = terminal_sy.wrapping_sub((*pd).sy);
    } else {
        (*pd).py = (*pd).ppy;
    }
    if (*pd).ppx.wrapping_add((*pd).sx) > terminal_sx {
        (*pd).px = terminal_sx.wrapping_sub((*pd).sx);
    } else {
        (*pd).px = (*pd).ppx;
    }
    if (*pd).border_lines as ::core::ffi::c_int == BOX_LINES_NONE as ::core::ffi::c_int {
        screen_resize(&mut (*pd).s, (*pd).sx, (*pd).sy, 0 as ::core::ffi::c_int);
        if !(*pd).job.is_empty() {
            job_resize(&(*pd).job, (*pd).sx, (*pd).sy);
        }
    } else if (*pd).sx > 2 as u_int && (*pd).sy > 2 as u_int {
        screen_resize(
            &mut (*pd).s,
            (*pd).sx.wrapping_sub(2 as u_int),
            (*pd).sy.wrapping_sub(2 as u_int),
            0 as ::core::ffi::c_int,
        );
        if !(*pd).job.is_empty() {
            job_resize(
                &(*pd).job,
                (*pd).sx.wrapping_sub(2 as u_int),
                (*pd).sy.wrapping_sub(2 as u_int),
            );
        }
    }
}
unsafe fn popup_handle_drag(c_owner: &ClientRef, popup: &PopupGuard, mut m: *mut mouse_event) {
    let (terminal_sx, terminal_sy) = c_owner.terminal_size();
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
        } else if (*m).x.wrapping_sub((*pd).dx).wrapping_add((*pd).sx) > terminal_sx {
            px = terminal_sx.wrapping_sub((*pd).sx);
        } else {
            px = (*m).x.wrapping_sub((*pd).dx);
        }
        if (*m).y < (*pd).dy {
            py = 0 as u_int;
        } else if (*m).y.wrapping_sub((*pd).dy).wrapping_add((*pd).sy) > terminal_sy {
            py = terminal_sy.wrapping_sub((*pd).sy);
        } else {
            py = (*m).y.wrapping_sub((*pd).dy);
        }
        (*pd).px = px;
        (*pd).py = py;
        (*pd).dx = (*m).x.wrapping_sub((*pd).px);
        (*pd).dy = (*m).y.wrapping_sub((*pd).py);
        (*pd).ppx = px;
        (*pd).ppy = py;
        c_owner.request_redraw(CLIENT_ALLREDRAWFLAGS as u64);
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
            if !(*pd).job.is_empty() {
                job_resize(&(*pd).job, (*pd).sx, (*pd).sy);
            }
        } else {
            screen_resize(
                &mut (*pd).s,
                (*pd).sx.wrapping_sub(2 as u_int),
                (*pd).sy.wrapping_sub(2 as u_int),
                0 as ::core::ffi::c_int,
            );
            if !(*pd).job.is_empty() {
                job_resize(
                    &(*pd).job,
                    (*pd).sx.wrapping_sub(2 as u_int),
                    (*pd).sy.wrapping_sub(2 as u_int),
                );
            }
        }
        c_owner.request_redraw(CLIENT_ALLREDRAWFLAGS as u64);
    }
}
unsafe fn popup_key(c_owner: &ClientRef, popup: &PopupGuard, event: *mut key_event) -> i32 {
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
            popup_handle_drag(c_owner, popup, m);
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
        || (*pd).job.is_empty())
        && ((*event).key == '\u{1b}' as i32 as key_code
            || (*event).key == 'c' as i32 as ::core::ffi::c_ulonglong | KEYC_CTRL)
    {
        return 1 as ::core::ffi::c_int;
    }
    if (*pd).job.is_empty()
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
    if !(*pd).job.is_empty() {
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
            bufferevent_write(job_get_event(&(*pd).job), buf.as_ptr().cast(), len);
            return 0 as ::core::ffi::c_int;
        }
        input_key(&raw mut (*pd).s, job_get_event(&(*pd).job), (*event).key);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe fn popup_job_update_cb(job: &refbox::Weak<job>, popup: &PopupGuard) {
    let pd = popup.as_ptr();
    let evb: &mut SegmentedBuf = bufferevent_get_input(&mut *job_get_event(job));
    let client = (*pd).c.as_ref().expect("popup client");
    let mut s: *mut screen = &raw mut (*pd).s;
    let mut data: *mut ::core::ffi::c_void = evbuffer_pullup(evb, -1)
        .map_or(std::ptr::null_mut(), |bytes| bytes.as_mut_ptr())
        as *mut ::core::ffi::c_void;
    let mut size: size_t = evbuffer_get_length(&*(evb));
    if size == 0 as size_t {
        return;
    }
    let render = PopupRenderSnapshot::new(popup);
    client.with_overlay_check_disabled(popup_check_callback(popup.handle()), || {
        input_parse_screen(
            (*pd)
                .ictx
                .as_deref_mut()
                .map_or(std::ptr::null_mut(), |ictx| ictx),
            s,
            Some(Box::new(move |ttyctx| render.init_ctx(ttyctx))),
            data as *const u_char,
            size,
        )
    });
    evbuffer_drain(evb, size);
}
unsafe fn popup_job_complete_cb(completion: JobCompletion, popup: &PopupGuard) {
    let pd = popup.as_ptr();
    (*pd).status = match completion.status {
        JobExitStatus::Exited(code) => code,
        JobExitStatus::Signaled(signal) => signal,
        JobExitStatus::Other(_) => 0,
    };
    (*pd).job = refbox::Weak::new();
    if (*pd).flags & POPUP_CLOSEEXIT != 0
        || (*pd).flags & POPUP_CLOSEEXITZERO != 0 && (*pd).status == 0 as ::core::ffi::c_int
    {
        let client = (*pd).c.as_ref().expect("popup client");
        if popup.is_current(client) {
            client.clear_overlay();
        }
    }
}
pub unsafe fn popup_present(client: &ClientRef) -> i32 {
    client
        .with_overlay_data(|data| data.is_some_and(|data| data.is::<refbox::RefBox<PopupState>>()))
        as i32
}
pub unsafe fn popup_modify(
    c_owner: &ClientRef,
    mut title: *const ::core::ffi::c_char,
    mut style: *const ::core::ffi::c_char,
    mut border_style: *const ::core::ffi::c_char,
    mut lines: box_lines,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let Some(handle) = c_owner.with_overlay_data(|data| {
        data.and_then(|data| data.downcast_ref::<refbox::RefBox<PopupState>>())
            .map(|owner| PopupHandle(owner.downgrade()))
    }) else {
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
            job_resize(&(*pd).job, (*pd).sx, (*pd).sy);
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
                &(*pd).job,
                (*pd).sx.wrapping_sub(2 as u_int),
                (*pd).sy.wrapping_sub(2 as u_int),
            );
        }
        (*pd).border_lines = lines;
        c_owner.refresh_terminal_size();
    }
    if flags != -(1 as ::core::ffi::c_int) {
        (*pd).flags = flags;
    }
    c_owner.request_redraw(CLIENT_ALLREDRAWFLAGS as u64);
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn popup_display(
    mut flags: ::core::ffi::c_int,
    mut lines: box_lines,
    item_handle: Option<&std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>>,
    mut px: u_int,
    mut py: u_int,
    mut sx: u_int,
    mut sy: u_int,
    env: Option<&environ>,
    mut shellcmd: *const ::core::ffi::c_char,
    argv: &Vec<CString>,
    mut cwd: *const ::core::ffi::c_char,
    mut title: *const ::core::ffi::c_char,
    c_owner: &ClientRef,
    s_owner: Option<&SessionRef>,
    mut style: *const ::core::ffi::c_char,
    mut border_style: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let item = item_handle.map_or(std::ptr::null_mut(), |item| item.get());
    let mut jx: u_int = 0;
    let mut jy: u_int = 0;
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
    let session = s_owner
        .cloned()
        .or_else(|| c_owner.attached_session().upgrade())
        .expect("popup session");
    let link = session.current_winlink();
    let window = link
        .get_unchecked()
        .window_handle()
        .cloned()
        .expect("popup window");
    if lines == BOX_LINES_DEFAULT {
        lines = window
            .with_options_mut(|options| options_get_number(options, c"popup-border-lines".as_ptr()))
            as box_lines;
    }
    if lines as ::core::ffi::c_int == BOX_LINES_NONE as ::core::ffi::c_int {
        if sx < 1 as u_int || sy < 1 as u_int {
            window.release(c"popup invalid size");
            return -(1 as ::core::ffi::c_int);
        }
        jx = sx;
        jy = sy;
    } else {
        if sx < 3 as u_int || sy < 3 as u_int {
            window.release(c"popup invalid size");
            return -(1 as ::core::ffi::c_int);
        }
        jx = sx.wrapping_sub(2 as u_int);
        jy = sy.wrapping_sub(2 as u_int);
    }
    if c_owner.terminal_size().0 < sx || c_owner.terminal_size().1 < sy {
        window.release(c"popup invalid size");
        return -(1 as ::core::ffi::c_int);
    }
    let mut data = Box::new(popup_data::empty());
    data.title = popup_optional_string((!title.is_null()).then(|| CStr::from_ptr(title)));
    data.style = popup_optional_string((!style.is_null()).then(|| CStr::from_ptr(style)));
    data.border_style =
        popup_optional_string((!border_style.is_null()).then(|| CStr::from_ptr(border_style)));
    let owner = refbox::RefBox::new(PopupState {
        data: UnsafeCell::new(data),
    });
    let handle = PopupHandle(owner.downgrade());
    let popup = handle.upgrade().expect("new popup");
    let pd = popup.as_ptr();
    (*pd).item = if item.is_null() {
        Weak::new()
    } else {
        (*item).observer.clone()
    };
    (*pd).flags = flags;
    (*pd).c = Some(c_owner.clone());
    (*pd).status = 128 as ::core::ffi::c_int + SIGHUP;
    (*pd).border_lines = lines;
    memcpy(
        &raw mut (*pd).border_cell as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    style_apply_with_options(
        &mut (*pd).border_cell,
        c"popup-border-style",
        None,
        |read| window.with_options_mut(read),
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
    style_apply_with_options(&mut (*pd).defaults, c"popup-style", None, |read| {
        window.with_options_mut(read)
    });
    window.release(c"popup styles");
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
    let update = PopupHandle(owner.downgrade());
    let complete = PopupHandle(owner.downgrade());
    (*pd).job = job_run(
        (!shellcmd.is_null()).then(|| CStr::from_ptr(shellcmd)),
        argv,
        env,
        s_owner,
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
    if (*pd).job.is_empty() {
        return -(1 as ::core::ffi::c_int);
    }
    (*pd).ictx = Some(input_init(
        None,
        job_get_event(&(*pd).job),
        crate::src::shared::input::InputPalette::Popup((*pd).palette.downgrade()),
        (*pd).c.as_ref(),
    ));
    (*pd).published = true;
    let check_cb = popup_check_callback(PopupHandle(owner.downgrade()));
    let mode = PopupHandle(owner.downgrade());
    let mode_cb: overlay_mode_cb = Some(Box::new(move |_| {
        mode.upgrade()
            .and_then(|popup| unsafe { popup_mode(&popup) })
    }));
    let draw = PopupHandle(owner.downgrade());
    let draw_cb: overlay_draw_cb = Some(Box::new(move |c| unsafe {
        if let Some(popup) = draw.upgrade() {
            popup_draw(c, &popup);
        }
    }));
    let key = PopupHandle(owner.downgrade());
    let key_cb: overlay_key_cb = Some(Box::new(move |c, event| unsafe {
        key.upgrade().map_or(0, |popup| popup_key(c, &popup, event))
    }));
    let free_cb: overlay_free_cb = None;
    let resize = PopupHandle(owner.downgrade());
    let resize_cb: overlay_resize_cb = Some(Box::new(move |c| unsafe {
        if let Some(popup) = resize.upgrade() {
            popup_resize(c, &popup);
        }
    }));
    drop(popup);
    c_owner.set_overlay(
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

    fn owner() -> refbox::RefBox<PopupState> {
        let mut data = Box::new(popup_data::empty());
        data.title = Some(c"active popup".to_owned());
        refbox::RefBox::new(PopupState {
            data: UnsafeCell::new(data),
        })
    }

    #[test]
    fn terminal_contexts_own_palette_snapshots_without_retaining_the_popup() {
        unsafe {
            let owner = owner();
            let handle = PopupHandle(owner.downgrade());
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
            let crate::src::shared::tty::PaletteSource::Snapshot(snapshot) =
                &first.style_ctx.palette
            else {
                panic!("popup context owns its palette snapshot");
            };
            assert_ne!(&**snapshot as *const colour_palette, source.as_ptr());
            assert_eq!(
                first
                    .style_ctx
                    .palette
                    .with_palette(|palette| palette.unwrap().fg),
                3
            );
            source.try_borrow_mut().unwrap().fg = 5;
            let mut second = tty_ctx::default();
            render.init_ctx(&mut second);
            assert_eq!(
                second
                    .style_ctx
                    .palette
                    .with_palette(|palette| palette.unwrap().fg),
                5
            );
            assert_eq!(
                first
                    .style_ctx
                    .palette
                    .with_palette(|palette| palette.unwrap().fg),
                3
            );
            drop(owner);
            drop(guard);
            assert!(!handle.0.is_alive());
            assert!(!render.palette.is_alive());
            assert_eq!(
                first.style_ctx.palette.with_palette(|palette| {
                    crate::src::style::colour::colour_palette_get(palette, 1)
                }),
                7
            );
            let client = client::new();
            let mut set_client = first.set_client_cb.take().unwrap();
            assert_eq!(set_client(&mut first, &client), 0);
            (first.redraw_cb.as_ref().unwrap())(&first);
            let mut expired = tty_ctx::default();
            render.init_ctx(&mut expired);
            assert!(expired
                .style_ctx
                .palette
                .with_palette(|palette| palette.is_none()));
        }
    }

    unsafe fn close_during_dispatch<'a>(handle: &'a PopupHandle, c: &ClientRef) -> PopupGuard<'a> {
        let popup = handle.upgrade().unwrap();
        c.clear_overlay();
        assert!(handle.upgrade().is_none());
        assert_eq!((*popup.as_ptr()).title.as_deref(), Some(c"active popup"));
        popup
    }

    #[test]
    fn overlay_mode_snapshot_survives_explicit_overlay_close() {
        unsafe {
            for bordered in [false, true] {
                let client = client::new();
                let owner = owner();
                let handle = PopupHandle(owner.downgrade());
                let observer = handle.clone();
                {
                    let popup = handle.upgrade().unwrap();
                    let data = &mut *popup.as_ptr();
                    data.px = 10;
                    data.py = 20;
                    data.border_lines = if bordered {
                        BOX_LINES_SINGLE
                    } else {
                        BOX_LINES_NONE
                    };
                    data.s.cx = 3;
                    data.s.cy = 4;
                    data.s.mode = crate::src::shared::screen::MODE_CURSOR;
                    data.s.ccolour = 7;
                    data.s.default_ccolour = 2;
                    data.s.cstyle = crate::src::shared::display::SCREEN_CURSOR_BAR;
                    data.s.default_cstyle = crate::src::shared::display::SCREEN_CURSOR_UNDERLINE;
                    data.s.default_mode = crate::src::shared::screen::MODE_CURSOR_BLINKING;
                }
                client.set_overlay(
                    None,
                    Some(Box::new(move |_| popup_mode(&handle.upgrade().unwrap()))),
                    Some(Box::new(|_| {})),
                    None,
                    None,
                    None,
                    Box::new(owner),
                );
                let (mode, x, y) = server_client_overlay_mode(&client).unwrap();
                client.clear_overlay();
                assert!(!observer.0.is_alive());
                assert_eq!((x, y), if bordered { (14, 25) } else { (13, 24) });
                assert_eq!((mode.cx, mode.cy), (3, 4));
                assert_eq!(mode.mode, crate::src::shared::screen::MODE_CURSOR);
                assert_eq!((mode.ccolour, mode.default_ccolour), (7, 2));
                assert_eq!(mode.cstyle, crate::src::shared::display::SCREEN_CURSOR_BAR);
                assert_eq!(
                    mode.default_cstyle,
                    crate::src::shared::display::SCREEN_CURSOR_UNDERLINE
                );
                assert_eq!(
                    mode.default_mode,
                    crate::src::shared::screen::MODE_CURSOR_BLINKING
                );
            }
        }
    }

    #[test]
    fn every_overlay_dispatch_holds_the_owner_across_self_close() {
        unsafe {
            for kind in 0..5 {
                let client = client::new();
                let owner = owner();
                let handle = PopupHandle(owner.downgrade());
                let observer = handle.clone();
                let mut check: overlay_check_cb = None;
                let mut mode: overlay_mode_cb = None;
                let mut draw: overlay_draw_cb = Some(Box::new(|_| {}));
                let mut key: overlay_key_cb = None;
                let mut resize: overlay_resize_cb = None;
                match kind {
                    0 => {
                        draw = Some(Box::new(move |c| {
                            let _active = close_during_dispatch(&handle, c);
                        }))
                    }
                    1 => {
                        key = Some(Box::new(move |c, _| {
                            let _active = close_during_dispatch(&handle, c);
                            1
                        }))
                    }
                    2 => {
                        resize = Some(Box::new(move |c| {
                            let _active = close_during_dispatch(&handle, c);
                        }))
                    }
                    3 => {
                        mode = Some(Box::new(move |c| {
                            let active = close_during_dispatch(&handle, c);
                            popup_mode(&active)
                        }))
                    }
                    _ => {
                        check = Some(Box::new(move |c, x, y, n| {
                            let active = close_during_dispatch(&handle, c);
                            popup_check(&active, x, y, n)
                        }))
                    }
                }
                client.set_overlay(check, mode, draw, key, None, resize, Box::new(owner));
                match kind {
                    0 => server_client_overlay_draw(&client),
                    1 => {
                        let mut event = key_event::new(0, mouse_event::default(), None);
                        assert_eq!(server_client_overlay_key(&client, &mut event), Some(0));
                        assert_eq!(server_client_overlay_key(&client, &mut event), None);
                    }
                    2 => server_client_overlay_resize(&client),
                    3 => assert!(server_client_overlay_mode(&client).is_none()),
                    _ => assert!(server_client_overlay_check(&client, 0, 0, 10).is_none()),
                }
                assert!(!client.has_overlay());
                assert!(!client.clips_terminal_output());
                assert!(client.with_overlay_data(|data| data.is_none()));
                assert!(!observer.0.is_alive());
            }
        }
    }
}
