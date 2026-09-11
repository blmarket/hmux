use crate::screen::Screen as _;
use super::menu::{menu_add_items, menu_check_cb, menu_create, menu_mode_cb, menu_prepare};
use crate::WindowPane;
use crate::cmd::{CmdqItemRef, CmdqItemWeak};
use crate::environ::RustEnvironment;
use crate::ffi::mkstemp;
use crate::fmt_args;
use crate::format::format_create_defaults;
use crate::grid::grid_default_cell;
use crate::input::InputOwner;
use crate::input::{MouseInputEncoder, RustMouseInputEncoder, input_key};
use crate::job::{job_event_by_id, job_free, job_resize, job_run, job_transfer};

use crate::log::fatalx;

use crate::pane_geometry::PaneGeometryState;
use crate::paste::{PasteBufferStore, with_paste_buffers};
use crate::screen::{Screen, ScreenModeState};
use crate::screen::{
    ScreenWriteCtx, screen_write_ctx_on_screen, screen_write_init_ctx,
};
use crate::server::server_redraw_client;
use crate::server::{client_ref_of, server_client_clear_overlay, server_client_set_overlay};

pub use crate::consts::{
    _PATH_BSHELL, BOX_LINES_DEFAULT, BOX_LINES_NONE, CLIENT_REDRAWOVERLAY, JOB_DEFAULTSHELL,
    JOB_KEEPWRITE, JOB_NOWAIT, JOB_PTY, KEYC_CTRL, KEYC_MASK_KEY, KEYC_MASK_TYPE, KEYC_MOUSE,
    KEYC_NONE, KEYC_PASTE_END, KEYC_PASTE_START, KEYC_TYPE_FUNCTION, KEYC_TYPE_MOUSEMOVE,
    KEYC_TYPE_TRIPLECLICK, LAYOUT_LEFTRIGHT, LAYOUT_TOPBOTTOM, MOUSE_BUTTON_1, MOUSE_BUTTON_3,
    MOUSE_MASK_BUTTONS, MOUSE_MASK_CTRL, MOUSE_MASK_DRAG, MOUSE_MASK_META, MOUSE_MASK_SHIFT,
    PANE_CHANGED, POPUP_CLOSEANYKEY, POPUP_CLOSEEXIT, POPUP_CLOSEEXITZERO, POPUP_NOJOB, SIGHUP,
    TTY_CTX_WINDOW_BIGGER,
};
use crate::style::{ColourEngine, RustColourEngine};
use crate::style::{RustStyleCodec, StyleCodec, style_apply, style_default};
use crate::tmux::checkshell;
use crate::tmux::{global_options, global_w_options};
use crate::tty::tty_draw_line;
use crate::tty::tty_resize;
pub use crate::types::*;
use crate::window::{window_add_pane};
use crate::xmalloc::xasprintf;
use ::core::ffi::CStr;
use ::std::ffi::OsStr;
use ::std::fs::{self, File};
use ::std::io::Write;
use ::std::os::fd::FromRawFd;
use ::std::os::unix::ffi::OsStrExt;
use ::std::path::{Path, PathBuf};
use ::std::rc::{Rc, Weak};
use std::cell::RefCell;

#[derive(Default)]
#[repr(C)]
pub struct popup_data {
    /// The popup's observation of itself, which is what its job and its menu
    /// hold it by.
    pub(crate) owner: Option<PopupDataWeak>,
    pub(crate) c: Option<ClientRef>,
    pub(crate) item: Option<CmdqItemWeak>,
    pub flags: core::ffi::c_int,
    pub title: Option<std::ffi::CString>,
    pub style: Option<std::ffi::CString>,
    pub border_style: Option<std::ffi::CString>,
    pub border_cell: grid_cell,
    pub border_lines: box_lines,
    pub(crate) s: ScreenRef,
    pub defaults: grid_cell,
    pub palette: colour_palette,
    pub r: VisibleRangesRef,
    pub or: [visible_ranges; 2],
    /// The id of the job the popup is running, or nothing while it runs
    /// none. A job is named by its id and nothing else, so the popup never
    /// names one that has finished.
    pub(crate) job: Option<u_int>,
    pub ictx: Option<InputCtxRef>,
    pub status: core::ffi::c_int,
    pub(crate) cb: Option<Box<dyn FnOnce(core::ffi::c_int)>>,
    pub md: Option<MenuDataRef>,
    pub close: core::ffi::c_int,
    pub px: u_int,
    pub py: u_int,
    pub sx: u_int,
    pub sy: u_int,
    pub ppx: u_int,
    pub ppy: u_int,
    pub psx: u_int,
    pub psy: u_int,
    pub dragging: popup_data_dragging,
    pub dx: u_int,
    pub dy: u_int,
    pub lx: u_int,
    pub ly: u_int,
    pub lb: u_int,
}

impl popup_data {
    /// The stream of the popup's registered job, if it is still running.
    pub(crate) fn job_event(&self) -> Option<Stream> {
        self.job.and_then(job_event_by_id)
    }

    /// The client the popup is drawn on, if the popup still holds it.
    pub(crate) fn client(&self) -> Option<ClientRef> {
        self.c.clone()
    }
}
pub type popup_data_dragging = core::ffi::c_uint;
pub const SIZE: popup_data_dragging = 2;
pub const MOVE: popup_data_dragging = 1;
pub const OFF: popup_data_dragging = 0;
pub const NONE: popup_border = 0;
pub type popup_border = core::ffi::c_uint;
pub const BOTTOM: popup_border = 4;
pub const TOP: popup_border = 3;
pub const RIGHT: popup_border = 2;
pub const LEFT: popup_border = 1;

pub const _PATH_TMP: &CStr = c"/tmp/";

pub const MOUSE_MASK_MODIFIERS: core::ffi::c_int =
    MOUSE_MASK_SHIFT | MOUSE_MASK_META | MOUSE_MASK_CTRL;

pub const POPUP_INTERNAL: core::ffi::c_int = 0x4 as core::ffi::c_int;

static popup_menu_items: [menu_item<'static>; 8] = [
    menu_item {
        name: Some(c"Close"),
        key: 'q' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: Some(c"#{?buffer_name,Paste #[underscore]#{buffer_name},}"),
        key: 'p' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: Some(c""),
        key: KEYC_NONE as core::ffi::c_ulong as key_code,
        command: None,
    },
    menu_item {
        name: Some(c"Fill Space"),
        key: 'F' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: Some(c"Centre"),
        key: 'C' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: Some(c""),
        key: KEYC_NONE as core::ffi::c_ulong as key_code,
        command: None,
    },
    menu_item {
        name: Some(c"To Horizontal Pane"),
        key: 'h' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: Some(c"To Vertical Pane"),
        key: 'v' as i32 as key_code,
        command: None,
    },
];
static popup_internal_menu_items: [menu_item<'static>; 4] = [
    menu_item {
        name: Some(c"Close"),
        key: 'q' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: Some(c""),
        key: KEYC_NONE as core::ffi::c_ulong as key_code,
        command: None,
    },
    menu_item {
        name: Some(c"Fill Space"),
        key: 'F' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: Some(c"Centre"),
        key: 'C' as i32 as key_code,
        command: None,
    },
];

/// A strong owner of popup state with checked shared and exclusive borrows.
#[derive(Clone)]
pub struct PopupDataRef(Rc<RefCell<popup_data>>);

/// A non-owning observation of a popup. Its job and its menu hold it this
/// way: either can outlive the overlay the popup itself belongs to, and then
/// finds nothing rather than a freed popup.
#[derive(Clone, Debug)]
pub struct PopupDataWeak(Weak<RefCell<popup_data>>);

impl PartialEq for PopupDataRef {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for PopupDataRef {}

impl std::fmt::Debug for PopupDataRef {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_tuple("PopupDataRef")
            .field(&Rc::as_ptr(&self.0))
            .finish()
    }
}

impl PopupDataRef {
    pub(crate) fn new(value: popup_data) -> Self {
        let reference = Self(Rc::new(RefCell::new(value)));
        reference.borrow_mut().owner = Some(reference.downgrade());
        reference
    }

    pub(crate) fn borrow(&self) -> std::cell::Ref<'_, popup_data> {
        self.0.borrow()
    }

    pub(crate) fn borrow_mut(&self) -> std::cell::RefMut<'_, popup_data> {
        self.0.borrow_mut()
    }

    /// Makes a non-owning observation of this popup.
    pub(crate) fn downgrade(&self) -> PopupDataWeak {
        PopupDataWeak(Rc::downgrade(&self.0))
    }
}

impl PartialEq for PopupDataWeak {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for PopupDataWeak {}

impl PopupDataWeak {
    /// Upgrades the observation while an overlay or dispatch still owns the popup.
    pub(crate) fn upgrade(&self) -> Option<PopupDataRef> {
        self.0.upgrade().map(PopupDataRef)
    }
}

/// The popup's own observation of itself, which is how it is handed to the
/// things that outlive an overlay.
fn popup_owner(pd: &popup_data) -> PopupDataWeak {
    pd.owner.clone().expect("a popup holds itself")
}

unsafe fn popup_reapply_styles(pd: &mut popup_data) {
    unsafe {
        let c = pd.client().expect("an active popup has a client");
        let Some(session) = c.attached_session() else {
            return;
        };
        let s = session.as_session();
        let wl = s.curw().expect("an attached client has a current window");
        let o = wl
            .window_handle()
            .expect("a window link owns its window")
            .options();
        let mut sytmp = style_default;
        let mut ft = format_create_defaults(
            None,
            Some(c.as_client()),
            Some(s),
            Some(wl),
            None::<&dyn crate::WindowPane>,
        );
        pd.defaults = grid_default_cell;
        style_apply(&mut pd.defaults, &o, c"popup-style", Some(&mut ft));
        if let Some(style) = pd.style.as_deref() {
            RustStyleCodec.set(&mut sytmp, &grid_default_cell);
            if RustStyleCodec
                .parse(&mut sytmp, &pd.defaults, style.to_bytes())
                .is_ok()
            {
                pd.defaults.fg = sytmp.gc.fg;
                pd.defaults.bg = sytmp.gc.bg;
            }
        }
        pd.defaults.attr = 0 as u_short;
        pd.border_cell = grid_default_cell;
        style_apply(
            &mut pd.border_cell,
            &o,
            c"popup-border-style",
            Some(&mut ft),
        );
        if let Some(style) = pd.border_style.as_deref() {
            RustStyleCodec.set(&mut sytmp, &grid_default_cell);
            if RustStyleCodec
                .parse(&mut sytmp, &pd.border_cell, style.to_bytes())
                .is_ok()
            {
                pd.border_cell.fg = sytmp.gc.fg;
                pd.border_cell.bg = sytmp.gc.bg;
            }
        }
        pd.border_cell.attr = 0 as u_short;
    }
}
unsafe fn popup_redraw_cb(ttyctx: &tty_ctx) {
    unsafe {
        let TtyCtxArg::Popup(owner) = &ttyctx.arg else {
            return;
        };
        let Some(reference) = owner.upgrade() else {
            return;
        };
        let Some(mut c) = reference.borrow().client() else {
            return;
        };
        *c.flags_mut() |= CLIENT_REDRAWOVERLAY as uint64_t;
    }
}
fn popup_set_client_cb(ttyctx: &mut tty_ctx, c: &mut client) -> core::ffi::c_int {
    {
        let TtyCtxArg::Popup(owner) = &ttyctx.arg else {
            return 0 as core::ffi::c_int;
        };
        let Some(reference) = owner.upgrade() else {
            return 0;
        };
        let pd = reference.borrow();
        let Some(held) = pd.client() else {
            return 0;
        };
        if !core::ptr::eq(held.as_ptr(), c) {
            return 0 as core::ffi::c_int;
        }
        if c.flags & CLIENT_REDRAWOVERLAY as uint64_t != 0 {
            return 0 as core::ffi::c_int;
        }
        ttyctx.wox = 0 as u_int;
        ttyctx.woy = 0 as u_int;
        ttyctx.wsx = c.tty.sx;
        ttyctx.wsy = c.tty.sy;
        if pd.border_lines as core::ffi::c_int == BOX_LINES_NONE as core::ffi::c_int {
            ttyctx.rxoff = pd.px as core::ffi::c_int;
            ttyctx.xoff = ttyctx.rxoff;
            ttyctx.ryoff = pd.py as core::ffi::c_int;
            ttyctx.yoff = ttyctx.ryoff;
        } else {
            ttyctx.rxoff = pd.px.wrapping_add(1 as u_int) as core::ffi::c_int;
            ttyctx.xoff = ttyctx.rxoff;
            ttyctx.ryoff = pd.py.wrapping_add(1 as u_int) as core::ffi::c_int;
            ttyctx.yoff = ttyctx.ryoff;
        }
        1 as core::ffi::c_int
    }
}
fn popup_init_ctx_cb(pd: &popup_data, ttyctx: &mut tty_ctx) {
    ttyctx.defaults = pd.defaults;
    ttyctx.flags &= !TTY_CTX_WINDOW_BIGGER;
    ttyctx.palette = Some(pd.palette.clone());
    ttyctx.redraw_cb = Some(popup_redraw_cb);
    ttyctx.set_client_cb = Some(popup_set_client_cb);
    ttyctx.arg = TtyCtxArg::Popup(popup_owner(pd));
}

fn popup_init_ctx(owner: PopupDataWeak) -> screen_write_init_ctx {
    Some(Rc::new(move |ttyctx| {
        let Some(reference) = owner.upgrade() else {
            return;
        };
        popup_init_ctx_cb(&reference.borrow(), ttyctx);
    }))
}
/// The popup's screen mode, and where its cursor stands on the terminal.
pub(crate) fn popup_mode_cb(pd: &popup_data) -> (ScreenModeState, u_int, u_int) {
    if let Some(menu) = pd.md.as_ref() {
        return menu_mode_cb(&menu.borrow());
    }
    let (scx, scy) = pd.s.borrow().cursor();
    let (cx, cy) = if pd.border_lines as core::ffi::c_int == BOX_LINES_NONE as core::ffi::c_int {
        (pd.px.wrapping_add(scx), pd.py.wrapping_add(scy))
    } else {
        (
            pd.px.wrapping_add(1 as u_int).wrapping_add(scx),
            pd.py.wrapping_add(1 as u_int).wrapping_add(scy),
        )
    };
    (pd.s.borrow().mode_state(), cx, cy)
}

/// Computes the visible ranges using the popup's reusable storage.
///
/// With a menu attached, the C code clips the previously stored popup ranges,
/// uses the menu only for the number of ranges, and retains the old output
/// count. Those distinctions are preserved here.
pub(crate) fn popup_check_cb<'a>(
    pd: &'a mut popup_data,
    px: u_int,
    py: u_int,
    nx: u_int,
) -> std::cell::RefMut<'a, visible_ranges> {
    {
        let mut r = pd.r.borrow_mut();
        if let Some(owner) = pd.md.as_ref() {
            let mut menu = owner.borrow_mut();
            let menu_ranges = menu_check_cb(&mut menu, px, py, nx);
            if menu_ranges.used > 2 {
                fatalx(c"too many menu ranges", fmt_args![]);
            }
            let count = menu_ranges.used as usize;
            for (range, output) in r.ranges[..count].iter().zip(&mut pd.or[..count]) {
                output.set_overlay_range(pd.px, pd.py, pd.sx, pd.sy, range.px, py, range.nx);
            }
            r.ensure_capacity(3);
            let mut k = 0;
            for output in &pd.or[..count] {
                for range in &output.ranges[..output.used as usize] {
                    if range.nx == 0 {
                        continue;
                    }
                    if k >= 3 {
                        fatalx(c"too many popup & menu ranges", fmt_args![]);
                    }
                    r.ranges[k] = *range;
                    k += 1;
                }
            }
            return r;
        }
        r.set_overlay_range(pd.px, pd.py, pd.sx, pd.sy, px, py, nx);
        r
    }
}

unsafe fn popup_make_pane(pd: &mut popup_data, type_0: layout_type) {
    unsafe {
        let c = pd.client().expect("an active popup has a client");
        let session = c
            .attached_session()
            .expect("an attached popup has a session owner");
        let owner = session
            .current_window()
            .expect("an attached popup has a window")
            .clone();
        let pane_id = owner
            .active_pane_id()
            .expect("the popup's window has an active pane");
        owner.unzoom(1);
        let Some(slot) = owner.split_pane_layout(
            &crate::window::window_pane_find_by_id(pane_id).expect("the layout pane exists"),
            type_0,
            -1,
            0,
        ) else {
            return;
        };
        let hlimit = session.options().number(c"history-limit") as u_int;
        let mut pane = window_add_pane(&mut owner.as_window_mut(), None, hlimit, 0);
        let new_id = pane.id();
        owner.assign_pane_layout(&slot, &pane, 0);
        let new = pane.get_mut().expect("the new pane is present");
        if let Some(id) = pd.job
            && new.take_job(id)
        {
            pd.job = None;
        }
        pd.s.borrow_mut().set_title(
            new.base()
                .title()
                .expect("a new pane screen always carries a title"),
            0,
        );
        new.adopt_popup_screen(pd.s.take_for_pane());
        let configured = session.options().string_ref(c"default-shell");
        let shell = if checkshell(Some(&configured)) == 0 {
            _PATH_BSHELL
        } else {
            &configured
        };
        let mut command = new.pane_command();
        command.shell = Some(shell.to_owned());
        new.set_pane_command(&command);
        new.activate_transferred_process();
        owner.set_active_pane(
            &crate::window::window_pane_find_by_id(new_id).expect("the selected pane exists"),
            1,
        );
        pane.get_mut().expect("the converted pane is present").name_changed();
        pd.close = 1;
    }
}
unsafe fn popup_menu_done(_choice: u_int, key: key_code, data: PopupDataWeak) {
    unsafe {
        let Some(data) = data.upgrade() else {
            return;
        };
        let (client, menu) = {
            let mut pd = data.borrow_mut();
            (pd.client(), pd.md.take())
        };
        let Some(mut c) = client else {
            return;
        };
        if let Some(menu) = menu {
            c.clear_overlay_view();
            menu.close(c.as_client_mut());
        }
        let mut guard = data.borrow_mut();
        let pd = &mut *guard;
        server_redraw_client(c.as_client_mut());
        match key {
            112 => {
                if let Some(bytes) =
                    with_paste_buffers(|buffers| buffers.top().map(|buffer| buffer.data.to_vec()))
                    && let Some(event) = pd.job_event()
                {
                    event.write(&bytes);
                }
            }
            70 => {
                pd.sx = c.as_tty().sx;
                pd.sy = c.as_tty().sy;
                pd.px = 0 as u_int;
                pd.py = 0 as u_int;
                server_redraw_client(c.as_client_mut());
            }
            67 => {
                pd.px = c
                    .as_tty()
                    .sx
                    .wrapping_div(2 as u_int)
                    .wrapping_sub(pd.sx.wrapping_div(2 as u_int));
                pd.py = c
                    .as_tty()
                    .sy
                    .wrapping_div(2 as u_int)
                    .wrapping_sub(pd.sy.wrapping_div(2 as u_int));
                server_redraw_client(c.as_client_mut());
            }
            104 => {
                popup_make_pane(&mut *pd, LAYOUT_LEFTRIGHT);
            }
            118 => {
                popup_make_pane(&mut *pd, LAYOUT_TOPBOTTOM);
            }
            113 => {
                pd.close = 1 as core::ffi::c_int;
            }
            _ => {}
        };
    }
}
unsafe fn popup_handle_drag(c: &mut client, pd: &mut popup_data, m: &mouse_event) {
    unsafe {
        let px: u_int;
        let py: u_int;
        if m.b & MOUSE_MASK_DRAG as u_int == 0 {
            pd.dragging = OFF;
        } else if pd.dragging as core::ffi::c_uint == MOVE as core::ffi::c_int as core::ffi::c_uint
        {
            if m.x < pd.dx {
                px = 0 as u_int;
            } else if m.x.wrapping_sub(pd.dx).wrapping_add(pd.sx) > c.tty.sx {
                px = c.tty.sx.wrapping_sub(pd.sx);
            } else {
                px = m.x.wrapping_sub(pd.dx);
            }
            if m.y < pd.dy {
                py = 0 as u_int;
            } else if m.y.wrapping_sub(pd.dy).wrapping_add(pd.sy) > c.tty.sy {
                py = c.tty.sy.wrapping_sub(pd.sy);
            } else {
                py = m.y.wrapping_sub(pd.dy);
            }
            pd.px = px;
            pd.py = py;
            pd.dx = m.x.wrapping_sub(pd.px);
            pd.dy = m.y.wrapping_sub(pd.py);
            pd.ppx = px;
            pd.ppy = py;
            server_redraw_client(&mut *c);
        } else if pd.dragging as core::ffi::c_uint == SIZE as core::ffi::c_int as core::ffi::c_uint
        {
            if pd.border_lines as core::ffi::c_int == BOX_LINES_NONE as core::ffi::c_int {
                if m.x < pd.px.wrapping_add(1 as u_int) {
                    return;
                }
                if m.y < pd.py.wrapping_add(1 as u_int) {
                    return;
                }
            } else {
                if m.x < pd.px.wrapping_add(3 as u_int) {
                    return;
                }
                if m.y < pd.py.wrapping_add(3 as u_int) {
                    return;
                }
            }
            pd.sx = m.x.wrapping_sub(pd.px);
            pd.sy = m.y.wrapping_sub(pd.py);
            pd.psx = pd.sx;
            pd.psy = pd.sy;
            if pd.border_lines as core::ffi::c_int == BOX_LINES_NONE as core::ffi::c_int {
                (&mut pd.s.borrow_mut()).resize(pd.sx, pd.sy, 0 as core::ffi::c_int);
                if let Some(id) = pd.job {
                    job_resize(id, pd.sx, pd.sy);
                }
            } else {
                (&mut pd.s.borrow_mut()).resize(pd.sx.wrapping_sub(2 as u_int),
                    pd.sy.wrapping_sub(2 as u_int),
                    0 as core::ffi::c_int);
                if let Some(id) = pd.job {
                    job_resize(
                        id,
                        pd.sx.wrapping_sub(2 as u_int),
                        pd.sy.wrapping_sub(2 as u_int),
                    );
                }
            }
            server_redraw_client(&mut *c);
        }
    }
}

unsafe fn popup_job_update_cb(job: JobEvent, popup: &PopupDataWeak) {
    unsafe {
        let Some(owner) = popup.upgrade() else {
            return;
        };
        let (mut c, screen, ictx, init, view) = {
            let pd = owner.borrow();
            let Some(c) = pd.client() else {
                return;
            };
            (
                c,
                pd.s.clone(),
                pd.ictx.clone(),
                popup_init_ctx(popup_owner(&pd)),
                if pd.md.is_some() {
                    OverlayView::Menu
                } else {
                    OverlayView::Nothing
                },
            )
        };
        let Some(data) = job
            .event
            .with_input(|buffer| buffer.copy_to_bytes(buffer.len()))
        else {
            return;
        };
        let size = data.len();
        if size == 0 {
            return;
        }
        c.set_overlay_view(view);
        ictx.as_ref()
            .expect("an input owner has a parser")
            .parse_shared_screen(&screen, init, ByteBuffer::from(data));
        c.set_overlay_view(OverlayView::Popup);
    }
}
unsafe fn popup_job_complete_cb(job: JobEvent, popup: &PopupDataWeak) {
    unsafe {
        let Some(owner) = popup.upgrade() else {
            return;
        };
        let mut guard = owner.borrow_mut();
        let pd = &mut *guard;

        let status: core::ffi::c_int = job.status;
        if status & 0x7f as core::ffi::c_int == 0 as core::ffi::c_int {
            pd.status = (status & 0xff00 as core::ffi::c_int) >> 8 as core::ffi::c_int;
        } else if ((status & 0x7f as core::ffi::c_int) + 1 as core::ffi::c_int)
            as core::ffi::c_schar as core::ffi::c_int
            >> 1 as core::ffi::c_int
            > 0 as core::ffi::c_int
        {
            pd.status = status & 0x7f as core::ffi::c_int;
        } else {
            pd.status = 0 as core::ffi::c_int;
        }
        pd.job = None;
        if pd.flags & POPUP_CLOSEEXIT != 0
            || pd.flags & POPUP_CLOSEEXITZERO != 0 && pd.status == 0 as core::ffi::c_int
        {
            let client = pd.client();
            drop(guard);
            if let Some(mut c) = client {
                server_client_clear_overlay(c.as_client_mut());
            }
        }
    }
}
pub fn popup_present(c: &client) -> core::ffi::c_int {
    (c.overlay() == Overlay::Popup) as core::ffi::c_int
}
pub unsafe fn popup_modify(
    c: &mut client,
    title: Option<&CStr>,
    style: Option<&CStr>,
    border_style: Option<&CStr>,
    lines: box_lines,
    flags: core::ffi::c_int,
) -> core::ffi::c_int {
    unsafe {
        let owner = c.overlay_data().popup();
        let mut guard = owner.borrow_mut();
        let pd = &mut *guard;
        let mut sytmp = style_default;
        if let Some(title) = title {
            pd.title = Some(title.to_owned());
        }
        if let Some(border_style) = border_style {
            pd.border_style = Some(border_style.to_owned());
            RustStyleCodec.set(&mut sytmp, &pd.border_cell);
            if RustStyleCodec
                .parse(&mut sytmp, &pd.border_cell, border_style.to_bytes())
                .is_ok()
            {
                pd.border_cell.fg = sytmp.gc.fg;
                pd.border_cell.bg = sytmp.gc.bg;
            }
        }
        if let Some(style) = style {
            pd.style = Some(style.to_owned());
            RustStyleCodec.set(&mut sytmp, &pd.defaults);
            if RustStyleCodec
                .parse(&mut sytmp, &pd.defaults, style.to_bytes())
                .is_ok()
            {
                pd.defaults.fg = sytmp.gc.fg;
                pd.defaults.bg = sytmp.gc.bg;
            }
        }
        if lines as core::ffi::c_int != BOX_LINES_DEFAULT as core::ffi::c_int {
            if lines as core::ffi::c_int == BOX_LINES_NONE as core::ffi::c_int
                && pd.border_lines as core::ffi::c_int != lines as core::ffi::c_int
            {
                (&mut pd.s.borrow_mut()).resize(pd.sx, pd.sy, 1 as core::ffi::c_int);
                if let Some(id) = pd.job {
                    job_resize(id, pd.sx, pd.sy);
                }
            } else if pd.border_lines as core::ffi::c_int == BOX_LINES_NONE as core::ffi::c_int
                && pd.border_lines as core::ffi::c_int != lines as core::ffi::c_int
            {
                (&mut pd.s.borrow_mut()).resize(pd.sx.wrapping_sub(2 as u_int),
                    pd.sy.wrapping_sub(2 as u_int),
                    1 as core::ffi::c_int);
                if let Some(id) = pd.job {
                    job_resize(
                        id,
                        pd.sx.wrapping_sub(2 as u_int),
                        pd.sy.wrapping_sub(2 as u_int),
                    );
                }
            }
            pd.border_lines = lines;
            tty_resize(&mut c.tty);
        }
        if flags != -(1 as core::ffi::c_int) {
            pd.flags = flags;
        }
        server_redraw_client(&mut *c);
        0 as core::ffi::c_int
    }
}
#[allow(clippy::too_many_arguments)]
pub unsafe fn popup_display(
    flags: core::ffi::c_int,
    mut lines: box_lines,
    item: Option<&CmdqItemRef>,
    px: u_int,
    py: u_int,
    sx: u_int,
    sy: u_int,
    env: Option<&RustEnvironment>,
    shellcmd: Option<&CStr>,
    argv: &[std::ffi::CString],
    cwd: Option<&CStr>,
    title: Option<&CStr>,
    c: &mut client,
    s: Option<&session>,
    style: Option<&CStr>,
    border_style: Option<&CStr>,
    close_cb: Option<Box<dyn FnOnce(core::ffi::c_int)>>,
) -> core::ffi::c_int {
    unsafe {
        let jx: u_int;
        let jy: u_int;
        let attached = c.attached_session();
        let Some(window) = s
            .or_else(|| attached.as_ref().map(|session| session.as_session()))
            .and_then(session::curw)
            .and_then(|link| link.window_handle().cloned())
        else {
            return -1;
        };
        let o = window.options();
        let mut sytmp = style_default;
        if lines as core::ffi::c_int == BOX_LINES_DEFAULT as core::ffi::c_int {
            lines = (o).number(c"popup-border-lines") as box_lines;
        }
        if lines as core::ffi::c_int == BOX_LINES_NONE as core::ffi::c_int {
            if sx < 1 as u_int || sy < 1 as u_int {
                return -(1 as core::ffi::c_int);
            }
            jx = sx;
            jy = sy;
        } else {
            if sx < 3 as u_int || sy < 3 as u_int {
                return -(1 as core::ffi::c_int);
            }
            jx = sx.wrapping_sub(2 as u_int);
            jy = sy.wrapping_sub(2 as u_int);
        }
        if c.tty.sx < sx || c.tty.sy < sy {
            return -(1 as core::ffi::c_int);
        }
        let pd_box: PopupDataRef = PopupDataRef::new(popup_data {
            border_cell: grid_default_cell,
            border_lines: BOX_LINES_NONE,
            defaults: grid_default_cell,
            palette: colour_palette {
                fg: 8,
                bg: 8,
                ..colour_palette::default()
            },
            ..popup_data::default()
        });
        let mut guard = pd_box.borrow_mut();
        let pd = &mut *guard;
        pd.item = item.map(CmdqItemRef::downgrade);
        pd.flags = flags;
        pd.title = title.map(CStr::to_owned);
        pd.style = style.map(CStr::to_owned);
        pd.border_style = border_style.map(CStr::to_owned);
        pd.c = client_ref_of(c);
        pd.cb = close_cb;
        pd.status = 128 as core::ffi::c_int + SIGHUP;
        pd.border_lines = lines;
        pd.border_cell = grid_default_cell;
        style_apply(&mut pd.border_cell, &o, c"popup-border-style", None);
        if let Some(border_style) = border_style {
            RustStyleCodec.set(&mut sytmp, &grid_default_cell);
            if RustStyleCodec
                .parse(&mut sytmp, &pd.border_cell, border_style.to_bytes())
                .is_ok()
            {
                pd.border_cell.fg = sytmp.gc.fg;
                pd.border_cell.bg = sytmp.gc.bg;
            }
        }
        pd.border_cell.attr = 0 as u_short;
        pd.s = ScreenRef::new(RustScreen::new_with_server_options(jx, jy, 0 as u_int));
        pd.s.borrow_mut().set_default_cursor(
            global_w_options
                .get()
                .as_ref()
                .expect("global options are initialized"),
        );
        RustColourEngine.init_palette(&mut pd.palette);
        (global_w_options
            .get()
            .as_ref()
            .expect("global options are initialized"))
        .load_pane_colours(Some(&mut pd.palette));
        pd.defaults = grid_default_cell;
        style_apply(&mut pd.defaults, &o, c"popup-style", None);
        if let Some(style) = style {
            RustStyleCodec.set(&mut sytmp, &grid_default_cell);
            if RustStyleCodec
                .parse(&mut sytmp, &pd.defaults, style.to_bytes())
                .is_ok()
            {
                pd.defaults.fg = sytmp.gc.fg;
                pd.defaults.bg = sytmp.gc.bg;
            }
        }
        pd.defaults.attr = 0 as u_short;
        pd.px = px;
        pd.py = py;
        pd.sx = sx;
        pd.sy = sy;
        pd.ppx = px;
        pd.ppy = py;
        pd.psx = sx;
        pd.psy = sy;
        if flags & POPUP_NOJOB != 0 {
            pd.ictx = Some(InputCtxRef::create(
                InputOwner::Popup(popup_owner(&*pd), None),
                Stream::NONE,
            ));
        } else {
            let popup = popup_owner(&*pd);
            let update_popup = popup.clone();
            let job = job_run(
                shellcmd,
                argv,
                env,
                s,
                cwd,
                Some(std::rc::Rc::new(move |job| {
                    popup_job_update_cb(job, &update_popup)
                })),
                Some(Box::new(move |job| popup_job_complete_cb(job, &popup))),
                JOB_NOWAIT | JOB_PTY | JOB_KEEPWRITE | JOB_DEFAULTSHELL,
                jx as core::ffi::c_int,
                jy as core::ffi::c_int,
            );
            pd.job = job;
            let Some(event) = job.and_then(job_event_by_id) else {
                drop(guard);
                pd_box.free_resources();
                return -(1 as core::ffi::c_int);
            };
            pd.ictx = Some(InputCtxRef::create(
                InputOwner::Popup(popup_owner(&*pd), client_ref_of(c).map(|c| c.downgrade())),
                event,
            ));
        }
        drop(guard);
        server_client_set_overlay(
            &mut *c,
            0 as u_int,
            Overlay::Popup,
            OverlayState::Popup(pd_box),
        );
        0 as core::ffi::c_int
    }
}
pub unsafe fn popup_write(c: &mut client, data: &[u8]) {
    unsafe {
        if popup_present(c) == 0 {
            return;
        }
        let owner = c.overlay_data().popup();
        let (screen, ictx, init) = {
            let pd = owner.borrow();
            (
                pd.s.clone(),
                pd.ictx.clone(),
                popup_init_ctx(popup_owner(&pd)),
            )
        };
        c.set_overlay_view(OverlayView::Nothing);
        ictx.as_ref()
            .expect("an input owner has a parser")
            .parse_shared_screen(
                &screen,
                init,
                ByteBuffer::from(bytes::Bytes::copy_from_slice(data)),
            );
        c.set_overlay_view(OverlayView::Popup);
    }
}
fn popup_editor_close(status: core::ffi::c_int, path: &Path, cb: popup_finish_edit_cb) {
    let buf = if status == 0 as core::ffi::c_int {
        fs::read(path).unwrap_or_default()
    } else {
        Vec::new()
    };
    cb(buf);
    let _ = fs::remove_file(path);
}
/// Opens `buf` in an editor and hands what comes back to `cb`.
pub unsafe fn popup_editor(
    c: &mut client,
    buf: &[u8],
    cb: popup_finish_edit_cb,
) -> core::ffi::c_int {
    unsafe {
        let mut path = *b"/tmp/tmux.XXXXXXXX\0";

        let editor = global_options
            .get()
            .expect("global options are initialized")
            .string_ref(c"editor");
        if editor.is_empty() {
            return -(1 as core::ffi::c_int);
        }
        let fd = mkstemp(path.as_mut_ptr().cast());
        if fd == -(1 as core::ffi::c_int) {
            return -(1 as core::ffi::c_int);
        }
        let path = CStr::from_bytes_with_nul(&path).expect("mkstemp leaves a NUL-terminated path");
        let editor_path = PathBuf::from(OsStr::from_bytes(path.to_bytes()));
        let mut file = File::from_raw_fd(fd);
        if buf.is_empty() || file.write_all(buf).is_err() {
            return -(1 as core::ffi::c_int);
        }
        drop(file);
        let close_cb: Box<dyn FnOnce(core::ffi::c_int)> = Box::new(move |status| {
            popup_editor_close(status, &editor_path, cb);
        });
        let sx: u_int = c.tty.sx.wrapping_mul(9 as u_int).wrapping_div(10 as u_int);
        let sy: u_int = c.tty.sy.wrapping_mul(9 as u_int).wrapping_div(10 as u_int);
        let px: u_int = c
            .tty
            .sx
            .wrapping_div(2 as u_int)
            .wrapping_sub(sx.wrapping_div(2 as u_int));
        let py: u_int = c
            .tty
            .sy
            .wrapping_div(2 as u_int)
            .wrapping_sub(sy.wrapping_div(2 as u_int));
        let cmd = xasprintf(c"%s %s", fmt_args![editor.as_ref(), path]);
        if popup_display(
            POPUP_INTERNAL | POPUP_CLOSEEXIT,
            BOX_LINES_DEFAULT,
            None,
            px,
            py,
            sx,
            sy,
            None,
            Some(cmd.as_c_str()),
            &[],
            Some(_PATH_TMP),
            None,
            c,
            None,
            None,
            None,
            Some(close_cb),
        ) != 0 as core::ffi::c_int
        {
            let _ = fs::remove_file(OsStr::from_bytes(path.to_bytes()));
            return -(1 as core::ffi::c_int);
        }
        0 as core::ffi::c_int
    }
}
use crate::screen::RustScreen;

#[cfg(test)]
#[path = "../tests/test_popup_focused.rs"]
mod focused_tests;

impl PopupDataRef {
    unsafe fn free_resources(self) {
        let reference = self;

        unsafe {
            let (client, job, ictx) = {
                let mut pd = reference.borrow_mut();
                (pd.c.take(), pd.job.take(), pd.ictx.take())
            };
            drop(client);
            if let Some(id) = job {
                job_free(id);
            }
            if let Some(ictx) = ictx {
                ictx.close();
            }
            let mut pd = reference.borrow_mut();
            pd.s = ScreenRef::default();
            RustColourEngine.free_palette(Some(&mut pd.palette));
        }
    }
    pub(crate) unsafe fn close(self, c: &mut client) {
        let reference = self;

        unsafe {
            let (item, menu) = {
                let mut pd = reference.borrow_mut();
                (
                    pd.item.as_ref().and_then(CmdqItemWeak::upgrade),
                    pd.md.take(),
                )
            };
            if let Some(menu) = menu {
                menu.close(c);
            }
            let (callback, status) = {
                let mut pd = reference.borrow_mut();
                (pd.cb.take(), pd.status)
            };
            if let Some(callback) = callback {
                callback(status);
            }
            if let Some(item) = &item {
                if let Some(mut asked) = item.client()
                    && asked.attached_session().is_none()
                {
                    asked.as_client_mut().retval = reference.borrow().status;
                }
                item.resume();
            }
            reference.free_resources();
        }
    }
    pub(crate) unsafe fn draw(&self, c: &mut client, rctx: &mut screen_redraw_ctx) {
        let owner = self;

        unsafe {
            let (s, defaults, palette, px, py, sx, sy, menu) = {
                let mut guard = owner.borrow_mut();
                let pd = &mut *guard;
                let px: u_int = pd.px;
                let py: u_int = pd.py;
                popup_reapply_styles(pd);
                let palette: &mut colour_palette = &mut pd.palette;
                let mut s = RustScreen::new_sharing_hyperlinks(pd.sx, pd.sy, 0, &pd.s.borrow());
                let mut writer = screen_write_ctx_on_screen(&mut s);
                writer.clearscreen(8 as u_int);
                if pd.border_lines as core::ffi::c_int == BOX_LINES_NONE as core::ffi::c_int {
                    writer.cursormove(
                        0 as core::ffi::c_int,
                        0 as core::ffi::c_int,
                        0 as core::ffi::c_int,
                    );
                    writer.fast_copy(&pd.s.borrow(), 0 as u_int, 0 as u_int, pd.sx, pd.sy);
                } else if pd.sx > 2 as u_int && pd.sy > 2 as u_int {
                    writer.box_(
                        pd.sx,
                        pd.sy,
                        pd.border_lines,
                        Some(&pd.border_cell),
                        pd.title.as_deref(),
                    );
                    writer.cursormove(
                        1 as core::ffi::c_int,
                        1 as core::ffi::c_int,
                        0 as core::ffi::c_int,
                    );
                    writer.fast_copy(
                        &pd.s.borrow(),
                        0 as u_int,
                        0 as u_int,
                        pd.sx.wrapping_sub(2 as u_int),
                        pd.sy.wrapping_sub(2 as u_int),
                    );
                }
                writer.stop();
                drop(writer);
                let mut defaults: grid_cell = pd.defaults;
                if defaults.fg == 8 as core::ffi::c_int {
                    defaults.fg = palette.fg;
                }
                if defaults.bg == 8 as core::ffi::c_int {
                    defaults.bg = palette.bg;
                }
                (
                    s,
                    defaults,
                    palette.clone(),
                    px,
                    py,
                    pd.sx,
                    pd.sy,
                    pd.md.is_some(),
                )
            };
            if menu {
                (*c).set_overlay_view(OverlayView::Menu);
            } else {
                (*c).set_overlay_view(OverlayView::Nothing);
            }
            for i in 0..sy {
                tty_draw_line(
                    &mut c.tty,
                    &s,
                    0 as u_int,
                    i,
                    sx,
                    px,
                    py.wrapping_add(i),
                    &defaults,
                    Some(&palette),
                );
            }
            let menu = owner.borrow().md.clone();
            if let Some(menu) = menu {
                (*c).set_overlay_view(OverlayView::Nothing);
                menu.draw(c, rctx);
            }
            (*c).set_overlay_view(OverlayView::Popup);
        }
    }
    pub(crate) unsafe fn resize(&self, c: &mut client) {
        let owner = self;

        unsafe {
            c.clear_overlay_view();
            let menu = owner.borrow_mut().md.take();
            if let Some(menu) = menu {
                menu.close(c);
            }
            let mut guard = owner.borrow_mut();
            let pd = &mut *guard;
            if pd.psy > c.tty.sy {
                pd.sy = c.tty.sy;
            } else {
                pd.sy = pd.psy;
            }
            if pd.psx > c.tty.sx {
                pd.sx = c.tty.sx;
            } else {
                pd.sx = pd.psx;
            }
            if pd.ppy.wrapping_add(pd.sy) > c.tty.sy {
                pd.py = c.tty.sy.wrapping_sub(pd.sy);
            } else {
                pd.py = pd.ppy;
            }
            if pd.ppx.wrapping_add(pd.sx) > c.tty.sx {
                pd.px = c.tty.sx.wrapping_sub(pd.sx);
            } else {
                pd.px = pd.ppx;
            }
            if pd.border_lines as core::ffi::c_int == BOX_LINES_NONE as core::ffi::c_int {
                (&mut pd.s.borrow_mut()).resize(pd.sx, pd.sy, 0 as core::ffi::c_int);
                if let Some(id) = pd.job {
                    job_resize(id, pd.sx, pd.sy);
                }
            } else if pd.sx > 2 as u_int && pd.sy > 2 as u_int {
                (&mut pd.s.borrow_mut()).resize(pd.sx.wrapping_sub(2 as u_int),
                    pd.sy.wrapping_sub(2 as u_int),
                    0 as core::ffi::c_int);
                if let Some(id) = pd.job {
                    job_resize(
                        id,
                        pd.sx.wrapping_sub(2 as u_int),
                        pd.sy.wrapping_sub(2 as u_int),
                    );
                }
            }
        }
    }
    pub(crate) unsafe fn key(&self, c: &mut client, event: &mut key_event) -> core::ffi::c_int {
        let owner = self;

        unsafe {
            let mut current_block: u64;
            let m: &mut mouse_event = &mut event.m;
            let px: u_int;
            let py: u_int;
            let x: u_int;
            let mut border: popup_border = NONE;
            let menu = owner.borrow_mut().md.take();
            if let Some(menu) = menu {
                if menu.key(c, event) == 1 {
                    c.clear_overlay_view();
                    menu.close(c);
                    if owner.borrow().close != 0 {
                        server_client_clear_overlay(c);
                    } else {
                        server_redraw_client(c);
                    }
                } else {
                    owner.borrow_mut().md = Some(menu);
                }
                return 0;
            }
            let mut guard = owner.borrow_mut();
            let pd = &mut *guard;
            if event.key as core::ffi::c_ulonglong & KEYC_MASK_KEY
                == KEYC_MOUSE as core::ffi::c_ulong as core::ffi::c_ulonglong
                || event.key as core::ffi::c_ulonglong & KEYC_MASK_TYPE
                    >= (KEYC_TYPE_MOUSEMOVE as core::ffi::c_int as core::ffi::c_ulonglong)
                        << 32 as core::ffi::c_int
                    && event.key as core::ffi::c_ulonglong & KEYC_MASK_TYPE
                        <= (KEYC_TYPE_TRIPLECLICK as core::ffi::c_int as core::ffi::c_ulonglong)
                            << 32 as core::ffi::c_int
            {
                if pd.dragging as core::ffi::c_uint != OFF as core::ffi::c_int as core::ffi::c_uint
                {
                    popup_handle_drag(c, &mut *pd, m);
                    current_block = 12200637015916837909;
                } else {
                    if m.x < pd.px
                        || m.x > pd.px.wrapping_add(pd.sx).wrapping_sub(1 as u_int)
                        || m.y < pd.py
                        || m.y > pd.py.wrapping_add(pd.sy).wrapping_sub(1 as u_int)
                    {
                        if m.b & MOUSE_MASK_BUTTONS as u_int == MOUSE_BUTTON_3 as u_int {
                            current_block = 1749512497212737283;
                        } else {
                            return 0 as core::ffi::c_int;
                        }
                    } else {
                        if pd.border_lines as core::ffi::c_int != BOX_LINES_NONE as core::ffi::c_int
                        {
                            if m.x == pd.px {
                                border = LEFT;
                            } else if m.x == pd.px.wrapping_add(pd.sx).wrapping_sub(1 as u_int) {
                                border = RIGHT;
                            } else if m.y == pd.py {
                                border = TOP;
                            } else if m.y == pd.py.wrapping_add(pd.sy).wrapping_sub(1 as u_int) {
                                border = BOTTOM;
                            }
                        }
                        if m.b & MOUSE_MASK_MODIFIERS as u_int == 0 as u_int
                            && m.b & MOUSE_MASK_BUTTONS as u_int == MOUSE_BUTTON_3 as u_int
                            && (border as core::ffi::c_uint
                                == LEFT as core::ffi::c_int as core::ffi::c_uint
                                || border as core::ffi::c_uint
                                    == TOP as core::ffi::c_int as core::ffi::c_uint)
                        {
                            current_block = 1749512497212737283;
                        } else if m.b & MOUSE_MASK_MODIFIERS as u_int == MOUSE_MASK_META as u_int
                            || border as core::ffi::c_uint
                                != NONE as core::ffi::c_int as core::ffi::c_uint
                                && m.lb & MOUSE_MASK_DRAG as u_int == 0
                        {
                            if m.b & MOUSE_MASK_DRAG as u_int == 0 {
                                current_block = 12200637015916837909;
                            } else {
                                if m.lb & MOUSE_MASK_BUTTONS as u_int == MOUSE_BUTTON_1 as u_int {
                                    pd.dragging = MOVE;
                                } else if m.lb & MOUSE_MASK_BUTTONS as u_int
                                    == MOUSE_BUTTON_3 as u_int
                                {
                                    pd.dragging = SIZE;
                                }
                                pd.dx = m.lx.wrapping_sub(pd.px);
                                pd.dy = m.ly.wrapping_sub(pd.py);
                                current_block = 12200637015916837909;
                            }
                        } else {
                            current_block = 3275366147856559585;
                        }
                    }
                    match current_block {
                        12200637015916837909 => {}
                        3275366147856559585 => {}
                        _ => {
                            let mut menu = menu_create(c"");
                            if pd.flags & POPUP_INTERNAL != 0 {
                                menu_add_items(
                                    &mut menu,
                                    &popup_internal_menu_items,
                                    None,
                                    c,
                                    None,
                                );
                            } else {
                                menu_add_items(&mut menu, &popup_menu_items, None, c, None);
                            }
                            if m.x >= menu.width.wrapping_add(4 as u_int).wrapping_div(2 as u_int) {
                                x = m.x.wrapping_sub(
                                    menu.width.wrapping_add(4 as u_int).wrapping_div(2 as u_int),
                                );
                            } else {
                                x = 0 as u_int;
                            }
                            let owner = popup_owner(&*pd);
                            let md = menu_prepare(
                                menu,
                                0 as core::ffi::c_int,
                                0 as core::ffi::c_int,
                                None,
                                x,
                                m.y,
                                c,
                                BOX_LINES_DEFAULT,
                                None,
                                None,
                                None,
                                None,
                                Some(Box::new(move |choice, key| {
                                    popup_menu_done(choice, key, owner)
                                })),
                            );
                            if let Some(md) = md {
                                pd.md = Some(md);
                            }
                            c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                            current_block = 12200637015916837909;
                        }
                    }
                }
                match current_block {
                    3275366147856559585 => {}
                    _ => {
                        pd.lx = m.x;
                        pd.ly = m.y;
                        pd.lb = m.b;
                        return 0 as core::ffi::c_int;
                    }
                }
            }
            if (pd.flags & (POPUP_CLOSEEXIT | POPUP_CLOSEEXITZERO) == 0 as core::ffi::c_int
                || pd.job_event().is_none())
                && (event.key == '\u{1b}' as i32 as key_code
                    || event.key == 'c' as i32 as core::ffi::c_ulonglong | KEYC_CTRL)
            {
                return 1 as core::ffi::c_int;
            }
            if pd.job_event().is_none()
                && pd.flags & POPUP_CLOSEANYKEY != 0
                && !(event.key as core::ffi::c_ulonglong & KEYC_MASK_KEY
                    == KEYC_MOUSE as core::ffi::c_ulong as core::ffi::c_ulonglong
                    || event.key as core::ffi::c_ulonglong & KEYC_MASK_TYPE
                        >= (KEYC_TYPE_MOUSEMOVE as core::ffi::c_int as core::ffi::c_ulonglong)
                            << 32 as core::ffi::c_int
                        && event.key as core::ffi::c_ulonglong & KEYC_MASK_TYPE
                            <= (KEYC_TYPE_TRIPLECLICK as core::ffi::c_int
                                as core::ffi::c_ulonglong)
                                << 32 as core::ffi::c_int)
                && !(event.key as core::ffi::c_ulonglong & KEYC_MASK_TYPE
                    == (KEYC_TYPE_FUNCTION as core::ffi::c_int as core::ffi::c_ulonglong)
                        << 32 as core::ffi::c_int
                    && (event.key as core::ffi::c_ulonglong & KEYC_MASK_KEY
                        == KEYC_PASTE_START as core::ffi::c_ulong as core::ffi::c_ulonglong
                        || event.key as core::ffi::c_ulonglong & KEYC_MASK_KEY
                            == KEYC_PASTE_END as core::ffi::c_ulong as core::ffi::c_ulonglong))
            {
                return 1 as core::ffi::c_int;
            }
            if let Some(stream) = pd.job_event() {
                if event.key as core::ffi::c_ulonglong & KEYC_MASK_KEY
                    == KEYC_MOUSE as core::ffi::c_ulong as core::ffi::c_ulonglong
                    || event.key as core::ffi::c_ulonglong & KEYC_MASK_TYPE
                        >= (KEYC_TYPE_MOUSEMOVE as core::ffi::c_int as core::ffi::c_ulonglong)
                            << 32 as core::ffi::c_int
                        && event.key as core::ffi::c_ulonglong & KEYC_MASK_TYPE
                            <= (KEYC_TYPE_TRIPLECLICK as core::ffi::c_int as core::ffi::c_ulonglong)
                                << 32 as core::ffi::c_int
                {
                    if pd.border_lines as core::ffi::c_int == BOX_LINES_NONE as core::ffi::c_int {
                        px = m.x.wrapping_sub(pd.px);
                        py = m.y.wrapping_sub(pd.py);
                    } else {
                        px = m.x.wrapping_sub(pd.px).wrapping_sub(1 as u_int);
                        py = m.y.wrapping_sub(pd.py).wrapping_sub(1 as u_int);
                    }
                    let Some(report) =
                        RustMouseInputEncoder.encode(pd.s.borrow().mode(), &*m, px, py)
                    else {
                        return 0 as core::ffi::c_int;
                    };
                    stream.write(&report);
                    return 0 as core::ffi::c_int;
                }
                input_key(&pd.s.borrow(), stream, event.key);
            }
            0 as core::ffi::c_int
        }
    }
}

/// Uses the existing overlay implementation with handle-based client/session context.
/// Target resolution, sizing, ownership, failure cleanup and callback timing are unchanged.
///
/// # Safety
/// Run on the server thread without conflicting client, TTY, session or queue
/// payload access. Existing replacement/completion callbacks may run inline;
/// exclude conflicting callback state access. No payload reference escapes.
pub(crate) unsafe fn popup_present_for_client(c: &ClientRef) -> core::ffi::c_int {
    unsafe { popup_present(c.as_client()) }
}

/// Uses the existing overlay implementation with handle-based client/session context.
/// Target resolution, sizing, ownership, failure cleanup and callback timing are unchanged.
///
/// # Safety
/// Run on the server thread without conflicting client, TTY, session or queue
/// payload access. Existing replacement/completion callbacks may run inline;
/// exclude conflicting callback state access. No payload reference escapes.
pub(crate) unsafe fn popup_modify_for_client(
    c: &mut ClientRef,
    title: Option<&CStr>,
    style: Option<&CStr>,
    border_style: Option<&CStr>,
    lines: box_lines,
    flags: core::ffi::c_int,
) -> core::ffi::c_int {
    unsafe { popup_modify(c.as_client_mut(), title, style, border_style, lines, flags) }
}

/// Uses the existing overlay implementation with handle-based client/session context.
/// Target resolution, sizing, ownership, failure cleanup and callback timing are unchanged.
///
/// # Safety
/// Run on the server thread without conflicting client, TTY, session or queue
/// payload access. Existing replacement/completion callbacks may run inline;
/// exclude conflicting callback state access. No payload reference escapes.
pub(crate) unsafe fn popup_display_for_client(
    flags: core::ffi::c_int,
    lines: box_lines,
    item: Option<&CmdqItemRef>,
    px: u_int,
    py: u_int,
    sx: u_int,
    sy: u_int,
    env: Option<&RustEnvironment>,
    shellcmd: Option<&CStr>,
    argv: &[std::ffi::CString],
    cwd: Option<&CStr>,
    title: Option<&CStr>,
    c: &mut ClientRef,
    s: Option<&SessionRef>,
    style: Option<&CStr>,
    border_style: Option<&CStr>,
    close_cb: Option<Box<dyn FnOnce(core::ffi::c_int)>>,
) -> core::ffi::c_int {
    unsafe {
        popup_display(
            flags,
            lines,
            item,
            px,
            py,
            sx,
            sy,
            env,
            shellcmd,
            argv,
            cwd,
            title,
            c.as_client_mut(),
            s.map(|s| s.as_session()),
            style,
            border_style,
            close_cb,
        )
    }
}

#[cfg(test)]
pub use crate::consts::{
    BOX_LINES_SINGLE, CLIENT_EXIT_DETACH, CLIENT_EXIT_RETURN, CLIENT_EXIT_SHUTDOWN, MSG_COMMAND,
    MSG_DETACH, MSG_DETACHKILL, MSG_EXIT, MSG_EXITED, MSG_EXITING, MSG_FLAGS, MSG_IDENTIFY_CWD,
    MSG_IDENTIFY_DONE, MSG_IDENTIFY_FLAGS, MSG_IDENTIFY_TERM, MSG_LOCK, MSG_READ, MSG_READ_DONE,
    MSG_READ_OPEN, MSG_READY, MSG_RESIZE, MSG_SHELL, MSG_SHUTDOWN, MSG_VERSION, MSG_WRITE,
    MSG_WRITE_CLOSE, MSG_WRITE_OPEN, MSG_WRITE_READY, PANE_LINES_DOUBLE, PANE_LINES_HEAVY,
    PANE_LINES_NUMBER, PANE_LINES_SIMPLE, PANE_LINES_SINGLE, PANE_LINES_SPACES, PROGRESS_BAR_ERROR,
    PROGRESS_BAR_HIDDEN, PROGRESS_BAR_INDETERMINATE, PROGRESS_BAR_NORMAL, PROGRESS_BAR_PAUSED,
    PROMPT_COMMAND, PROMPT_ENTRY, PROMPT_TYPE_COMMAND, PROMPT_TYPE_SEARCH, SCREEN_CURSOR_BAR,
    SCREEN_CURSOR_BLOCK, SCREEN_CURSOR_DEFAULT, SCREEN_CURSOR_UNDERLINE,
    STYLE_ALIGN_ABSOLUTE_CENTRE, STYLE_ALIGN_CENTRE, STYLE_ALIGN_DEFAULT, STYLE_ALIGN_LEFT,
    STYLE_ALIGN_RIGHT, STYLE_DEFAULT_BASE, STYLE_DEFAULT_POP, STYLE_DEFAULT_PUSH,
    STYLE_DEFAULT_SET, STYLE_RANGE_CONTROL, STYLE_RANGE_LEFT, STYLE_RANGE_NONE, STYLE_RANGE_PANE,
    STYLE_RANGE_RIGHT,
};
