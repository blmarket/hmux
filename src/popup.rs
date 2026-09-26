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
use crate::src::shared::hyperlinks::hyperlinks;
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
use std::ffi::{CStr, CString};

pub struct popup_data {
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
    pub palette: colour_palette,
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
            palette: Default::default(),
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

unsafe fn popup_free_resources(pd: *mut popup_data) {
    server_client_unref((*pd).c);
    if !(*pd).job.is_null() {
        job_free((*pd).job);
    }
    if !(*pd).ictx.is_null() {
        input_free((*pd).ictx);
    }
    screen_free(&raw mut (*pd).s);
    colour_palette_free(&raw mut (*pd).palette);
}
unsafe fn popup_free(mut pd: *mut popup_data) {
    popup_free_resources(pd);
    drop(Box::from_raw(pd));
}
unsafe fn popup_reapply_styles(mut pd: *mut popup_data) {
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
unsafe fn popup_set_client(pd: *mut popup_data, ttyctx: &mut tty_ctx, c: *mut client) -> i32 {
    if c != (*pd).c {
        return 0;
    }
    if (*(*pd).c).flags & CLIENT_REDRAWOVERLAY as uint64_t != 0 {
        return 0;
    }
    (*ttyctx).wox = 0 as u_int;
    (*ttyctx).woy = 0 as u_int;
    (*ttyctx).wsx = (*c).tty.sx;
    (*ttyctx).wsy = (*c).tty.sy;
    if (*pd).border_lines as ::core::ffi::c_int == BOX_LINES_NONE as ::core::ffi::c_int {
        (*ttyctx).rxoff = (*pd).px as ::core::ffi::c_int;
        (*ttyctx).xoff = (*ttyctx).rxoff;
        (*ttyctx).ryoff = (*pd).py as ::core::ffi::c_int;
        (*ttyctx).yoff = (*ttyctx).ryoff;
    } else {
        (*ttyctx).rxoff = (*pd).px.wrapping_add(1 as u_int) as ::core::ffi::c_int;
        (*ttyctx).xoff = (*ttyctx).rxoff;
        (*ttyctx).ryoff = (*pd).py.wrapping_add(1 as u_int) as ::core::ffi::c_int;
        (*ttyctx).yoff = (*ttyctx).ryoff;
    }
    1
}
unsafe fn popup_init_ctx(pd: *mut popup_data, ttyctx: *mut tty_ctx) {
    memcpy(
        &raw mut (*ttyctx).defaults as *mut ::core::ffi::c_void,
        &raw mut (*pd).defaults as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    (*ttyctx).flags &= !TTY_CTX_WINDOW_BIGGER;
    (*ttyctx).style_ctx.defaults = &raw mut (*ttyctx).defaults;
    (*ttyctx).style_ctx.palette = &raw mut (*pd).palette;
    (*ttyctx).redraw_cb = Some(Box::new(move |_| unsafe {
        (*(*pd).c).flags |= CLIENT_REDRAWOVERLAY as uint64_t;
    }));
    (*ttyctx).set_client_cb = Some(Box::new(move |ttyctx, c| unsafe {
        popup_set_client(pd, ttyctx, c as *mut client)
    }));
}
unsafe fn popup_mode(pd: *mut popup_data) -> Option<(std::ptr::NonNull<screen>, u_int, u_int)> {
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
unsafe fn popup_check(pd: *mut popup_data, px: u_int, py: u_int, nx: u_int) -> visible_ranges {
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
unsafe fn popup_draw(c: *mut client, pd: *mut popup_data) {
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
        defaults: ::core::ptr::null::<grid_cell>(),
        palette: ::core::ptr::null_mut::<colour_palette>(),
        dim: 0,
        hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
    };
    popup_reapply_styles(pd);
    screen_init(&raw mut s, (*pd).sx, (*pd).sy, 0 as u_int);
    if !(*pd).s.hyperlinks.is_null() {
        screen_share_hyperlinks(&raw mut s, &raw const (*pd).s);
    }
    screen_write_start(&raw mut ctx, &raw mut s);
    screen_write_clearscreen(&raw mut ctx, 8 as u_int);
    if (*pd).border_lines as ::core::ffi::c_int == BOX_LINES_NONE as ::core::ffi::c_int {
        screen_write_cursormove(
            &raw mut ctx,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_fast_copy(
            &raw mut ctx,
            &raw mut (*pd).s,
            0 as u_int,
            0 as u_int,
            (*pd).sx,
            (*pd).sy,
        );
    } else if (*pd).sx > 2 as u_int && (*pd).sy > 2 as u_int {
        screen_write_box(
            &raw mut ctx,
            (*pd).sx,
            (*pd).sy,
            (*pd).border_lines,
            &raw mut (*pd).border_cell,
            ((*pd).title)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        );
        screen_write_cursormove(
            &raw mut ctx,
            1 as ::core::ffi::c_int,
            1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_fast_copy(
            &raw mut ctx,
            &raw mut (*pd).s,
            0 as u_int,
            0 as u_int,
            (*pd).sx.wrapping_sub(2 as u_int),
            (*pd).sy.wrapping_sub(2 as u_int),
        );
    }
    screen_write_stop(&raw mut ctx);
    memcpy(
        &raw mut defaults as *mut ::core::ffi::c_void,
        &raw mut (*pd).defaults as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    if defaults.fg == 8 as ::core::ffi::c_int {
        defaults.fg = (*pd).palette.fg;
    }
    if defaults.bg == 8 as ::core::ffi::c_int {
        defaults.bg = (*pd).palette.bg;
    }
    style_ctx.defaults = &raw mut defaults;
    style_ctx.palette = &raw mut (*pd).palette;
    style_ctx.dim = 0 as u_int;
    style_ctx.hyperlinks = s.hyperlinks;
    (*c).overlay_check = None;
    i = 0 as u_int;
    while i < (*pd).sy {
        tty_draw_line(
            tty,
            &raw mut s,
            0 as u_int,
            i,
            (*pd).sx,
            px,
            py.wrapping_add(i),
            &raw mut style_ctx,
        );
        i = i.wrapping_add(1);
    }
    screen_free(&raw mut s);
    let pd = std::ptr::NonNull::new(pd).expect("live popup");
    (*c).overlay_check = Some(Box::new(move |_, px, py, nx| unsafe {
        popup_check(pd.as_ptr(), px, py, nx)
    }));
}
unsafe fn popup_free_callback(pd: *mut popup_data, _c: &mut client) {
    let mut item: *mut cmdq_item = (*pd).item;
    if !item.is_null() {
        if !cmdq_get_client(item).is_null() && (*cmdq_get_client(item)).session.is_null() {
            (*cmdq_get_client(item)).retval = (*pd).status;
        }
        cmdq_continue(item);
    }
    popup_free_resources(pd);
}
unsafe fn popup_resize(c: *mut client, pd: *mut popup_data) {
    let mut tty: *mut tty = &raw mut (*c).tty;
    if pd.is_null() {
        return;
    }
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
        screen_resize(
            &raw mut (*pd).s,
            (*pd).sx,
            (*pd).sy,
            0 as ::core::ffi::c_int,
        );
        if !(*pd).job.is_null() {
            job_resize((*pd).job, (*pd).sx, (*pd).sy);
        }
    } else if (*pd).sx > 2 as u_int && (*pd).sy > 2 as u_int {
        screen_resize(
            &raw mut (*pd).s,
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
unsafe fn popup_handle_drag(mut c: *mut client, mut pd: *mut popup_data, mut m: *mut mouse_event) {
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
            screen_resize(
                &raw mut (*pd).s,
                (*pd).sx,
                (*pd).sy,
                0 as ::core::ffi::c_int,
            );
            if !(*pd).job.is_null() {
                job_resize((*pd).job, (*pd).sx, (*pd).sy);
            }
        } else {
            screen_resize(
                &raw mut (*pd).s,
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
unsafe fn popup_key(c: *mut client, pd: *mut popup_data, event: *mut key_event) -> i32 {
    let mut current_block: u64;
    let mut m: *mut mouse_event = &raw mut (*event).m;
    let mut buf: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut len: size_t = 0;
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
            popup_handle_drag(c, pd, m);
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
            if input_key_get_mouse(&raw mut (*pd).s, m, px, py, &raw mut buf, &raw mut len) == 0 {
                return 0 as ::core::ffi::c_int;
            }
            bufferevent_write(
                job_get_event((*pd).job),
                buf as *const ::core::ffi::c_void,
                len,
            );
            return 0 as ::core::ffi::c_int;
        }
        input_key(&raw mut (*pd).s, job_get_event((*pd).job), (*event).key);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe fn popup_job_update_cb(job: &mut job, mut pd: *mut popup_data) {
    let mut evb: *mut evbuffer = (*job_get_event(job as *mut job)).input;
    let mut c: *mut client = (*pd).c;
    let mut s: *mut screen = &raw mut (*pd).s;
    let mut data: *mut ::core::ffi::c_void =
        evbuffer_pullup(evb, -(1 as ::core::ffi::c_int) as ssize_t) as *mut ::core::ffi::c_void;
    let mut size: size_t = evbuffer_get_length(&*(evb));
    if size == 0 as size_t {
        return;
    }
    (*c).overlay_check = None;
    input_parse_screen(
        (*pd).ictx,
        s,
        Some(Box::new(move |ttyctx| unsafe {
            popup_init_ctx(pd, ttyctx)
        })),
        data as *const u_char,
        size,
    );
    let pd = std::ptr::NonNull::new(pd).expect("live popup");
    (*c).overlay_check = Some(Box::new(move |_, px, py, nx| unsafe {
        popup_check(pd.as_ptr(), px, py, nx)
    }));
    evbuffer_drain(evb, size);
}
unsafe fn popup_job_complete_cb(completion: JobCompletion, mut pd: *mut popup_data) {
    (*pd).status = match completion.status {
        JobExitStatus::Exited(code) => code,
        JobExitStatus::Signaled(signal) => signal,
        JobExitStatus::Other(_) => 0,
    };
    (*pd).job = ::core::ptr::null_mut::<job>();
    if (*pd).flags & POPUP_CLOSEEXIT != 0
        || (*pd).flags & POPUP_CLOSEEXITZERO != 0 && (*pd).status == 0 as ::core::ffi::c_int
    {
        server_client_clear_overlay((*pd).c);
    }
}
pub unsafe fn popup_present(mut c: *mut client) -> ::core::ffi::c_int {
    return (*c)
        .overlay_data
        .as_ref()
        .is_some_and(|data| data.is::<popup_data>()) as ::core::ffi::c_int;
}
pub unsafe fn popup_modify(
    mut c: *mut client,
    mut title: *const ::core::ffi::c_char,
    mut style: *const ::core::ffi::c_char,
    mut border_style: *const ::core::ffi::c_char,
    mut lines: box_lines,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut pd: *mut popup_data = (*c)
        .overlay_data
        .as_deref_mut()
        .and_then(|data| data.downcast_mut::<popup_data>())
        .map_or(::core::ptr::null_mut(), |data| data as *mut popup_data);
    if pd.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
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
        let owner = &mut *pd;
        owner.title = updated;
    }
    if !border_style.is_null() {
        let updated = popup_optional_string(Some(CStr::from_ptr(border_style)));
        let owner = &mut *pd;
        owner.border_style = updated;

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
        let owner = &mut *pd;
        owner.style = updated;

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
            screen_resize(
                &raw mut (*pd).s,
                (*pd).sx,
                (*pd).sy,
                1 as ::core::ffi::c_int,
            );
            job_resize((*pd).job, (*pd).sx, (*pd).sy);
        } else if (*pd).border_lines as ::core::ffi::c_int == BOX_LINES_NONE as ::core::ffi::c_int
            && (*pd).border_lines as ::core::ffi::c_int != lines as ::core::ffi::c_int
        {
            screen_resize(
                &raw mut (*pd).s,
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
    mut env: *mut environ,
    mut shellcmd: *const ::core::ffi::c_char,
    argv: &Vec<CString>,
    mut cwd: *const ::core::ffi::c_char,
    mut title: *const ::core::ffi::c_char,
    mut c: *mut client,
    mut s: *mut session,
    mut style: *const ::core::ffi::c_char,
    mut border_style: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut pd: *mut popup_data = ::core::ptr::null_mut::<popup_data>();
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
    let owner = Box::new(popup_data {
        title: popup_optional_string(if title.is_null() {
            None
        } else {
            Some(CStr::from_ptr(title))
        }),
        style: popup_optional_string(if style.is_null() {
            None
        } else {
            Some(CStr::from_ptr(style))
        }),
        border_style: popup_optional_string(if border_style.is_null() {
            None
        } else {
            Some(CStr::from_ptr(border_style))
        }),
        ..popup_data::empty()
    });
    pd = Box::into_raw(owner).cast::<popup_data>();
    (*pd).item = item;
    (*pd).flags = flags;
    (*pd).c = c;
    (*(*pd).c).references += 1;
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
    screen_init(&raw mut (*pd).s, jx, jy, 0 as u_int);
    screen_set_default_cursor(&raw mut (*pd).s, global_w_options);
    colour_palette_init(&raw mut (*pd).palette);
    colour_palette_from_option(&raw mut (*pd).palette, global_w_options);
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
    (*pd).job = job_run(
        (!shellcmd.is_null()).then(|| CStr::from_ptr(shellcmd)),
        argv,
        env,
        s,
        (!cwd.is_null()).then(|| CStr::from_ptr(cwd)),
        job_update_callback(move |job| unsafe { popup_job_update_cb(job, pd) }),
        Some(Box::new(move |completion| unsafe {
            popup_job_complete_cb(completion, pd)
        })),
        None,
        JOB_NOWAIT | JOB_PTY | JOB_KEEPWRITE | JOB_DEFAULTSHELL,
        jx as ::core::ffi::c_int,
        jy as ::core::ffi::c_int,
    );
    if (*pd).job.is_null() {
        popup_free(pd);
        return -(1 as ::core::ffi::c_int);
    }
    (*pd).ictx = input_init(
        ::core::ptr::null_mut::<window_pane>(),
        job_get_event((*pd).job),
        &raw mut (*pd).palette,
        c,
    );
    let pd_handle = std::ptr::NonNull::new(pd).expect("live popup");
    let overlay_state = Box::from_raw(pd);
    let check_cb: overlay_check_cb = Some(Box::new(move |_, px, py, nx| unsafe {
        popup_check(pd_handle.as_ptr(), px, py, nx)
    }));
    let mode_cb: overlay_mode_cb =
        Some(Box::new(move |_| unsafe { popup_mode(pd_handle.as_ptr()) }));
    let draw_cb: overlay_draw_cb = Some(Box::new(move |c| unsafe {
        popup_draw(c as *mut client, pd_handle.as_ptr())
    }));
    let key_cb: overlay_key_cb = Some(Box::new(move |c, event| unsafe {
        popup_key(
            c as *mut client,
            pd_handle.as_ptr(),
            event as *mut key_event,
        )
    }));
    let free_cb: overlay_free_cb = Some(Box::new(move |c| unsafe {
        popup_free_callback(pd_handle.as_ptr(), c)
    }));
    let resize_cb: overlay_resize_cb = Some(Box::new(move |c| unsafe {
        popup_resize(c as *mut client, pd_handle.as_ptr())
    }));
    server_client_set_overlay(
        c,
        check_cb,
        mode_cb,
        draw_cb,
        key_cb,
        free_cb,
        resize_cb,
        overlay_state,
    );
    return 0 as ::core::ffi::c_int;
}
