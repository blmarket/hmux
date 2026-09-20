pub use crate::src::shared::prompt::{prompt_create_data, prompt_draw_data};
pub use crate::src::shared::client::{clients};
pub use crate::src::shared::arguments::{args};
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
pub use crate::src::shared::command::{
    cmd_find_state, cmd_list, cmdq_cb, cmdq_item, cmdq_list, cmds,
};
pub use crate::src::shared::control::{control_state};
pub use crate::src::shared::format::{format_job_tree, format_tree};
pub use crate::src::shared::input::{input_ctx, input_request, input_requests};
pub use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
pub use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
pub use crate::src::shared::menu::{menu_data};
pub use crate::src::shared::options::{
    options, options_array, options_array_item, options_entry, options_value,
};
pub use crate::src::shared::pane::{
    window_pane, window_pane_entry, window_pane_modes, window_pane_prompt, window_pane_sentry,
    window_pane_tree_entry, window_pane_zentry, window_panes,
};
pub use crate::src::shared::process::{tmuxpeer};
pub use crate::src::shared::prompt::{prompt};
pub use crate::src::shared::redraw::{redraw_scene};
pub use crate::src::shared::screen_write::{screen_write_ctx, screen_write_init_ctx_cb};
pub use crate::src::shared::session::{session, session_entry, session_gentry};
pub use crate::src::shared::spawn::{spawn_editor_state};
pub use crate::src::shared::status::{status_line, status_prompt_input_cb};
pub use crate::src::shared::tty::{
    tty, tty_code, tty_ctx, tty_ctx_c2rust_unnamed, tty_ctx_c2rust_unnamed_data,
    tty_ctx_c2rust_unnamed_sel, tty_ctx_redraw_cb, tty_ctx_set_client_cb, tty_key,
    tty_style_ctx, tty_term, tty_term_entry,
};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
pub use crate::src::shared::environment::{environ};
pub use crate::src::shared::format::{FORMAT_FORCE, FORMAT_NONE, FORMAT_STATUS};
pub use crate::src::shared::prompt::{
    PROMPT_ACCEPT, PROMPT_CLOSE, PROMPT_CONTINUE, PROMPT_INCREMENTAL, PROMPT_NOFREEZE,
    PROMPT_SINGLE, prompt_free_cb, prompt_input_cb, prompt_result,
};
pub use crate::src::shared::variadic::{__builtin_va_list, __va_list_tag, va_list};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_citem, screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::tty::{TTY_FREEZE, TTY_NOCURSOR};
pub use crate::src::shared::client::{
    CLIENT_ALLREDRAWFLAGS, CLIENT_CONTROL, CLIENT_REDRAWBORDERS, CLIENT_REDRAWMENU,
    CLIENT_REDRAWOVERLAY, CLIENT_REDRAWSTATUS, CLIENT_REDRAWSTATUSALWAYS, CLIENT_REDRAWWINDOW,
    CLIENT_STATUSFORCE, CLIENT_STATUSOFF,
};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{
    MOUSE_BUTTON_1, MOUSE_MASK_BUTTONS, MOUSE_MASK_DRAG, mouse_event,
};
use crate::src::shared::client::*;
use crate::src::shared::prompt::*;
use crate::src::shared::terminal::*;
use crate::src::shared::event::*;
use crate::src::shared::display::*;
use crate::src::shared::layout::*;
use crate::src::shared::message::*;
use crate::src::shared::abi::*;
use crate::src::shared::colour::*;
use crate::src::shared::command::*;
use crate::src::shared::grid::*;
use crate::src::shared::key::*;
use crate::src::shared::style::*;
extern "C" {

    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn event_add(ev: *mut event, timeout: *const timeval) -> ::core::ffi::c_int;
    fn event_del(_: *mut event) -> ::core::ffi::c_int;
    fn event_initialized(ev: *const event) -> ::core::ffi::c_int;
    fn event_set(
        _: *mut event,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_short,
        _: Option<
            unsafe extern "C" fn(
                ::core::ffi::c_int,
                ::core::ffi::c_short,
                *mut ::core::ffi::c_void,
            ) -> (),
        >,
        _: *mut ::core::ffi::c_void,
    );
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xmalloc(_: size_t) -> *mut ::core::ffi::c_void;
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn xvasprintf(
        _: *mut *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::VaList,
    ) -> ::core::ffi::c_int;
    static mut global_s_options: *mut options;
    fn format_create(
        _: *mut client,
        _: *mut cmdq_item,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    ) -> *mut format_tree;
    fn format_free(_: *mut format_tree);
    fn format_add(
        _: *mut format_tree,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    );
    fn format_expand_time(
        _: *mut format_tree,
        _: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn format_create_defaults(
        _: *mut cmdq_item,
        _: *mut client,
        _: *mut session,
        _: *mut winlink,
        _: *mut window_pane,
    ) -> *mut format_tree;
    fn format_defaults(
        _: *mut format_tree,
        _: *mut client,
        _: *mut session,
        _: *mut winlink,
        _: *mut window_pane,
    );
    fn format_draw(
        _: *mut screen_write_ctx,
        _: *const grid_cell,
        _: u_int,
        _: *const ::core::ffi::c_char,
        _: *mut style_ranges,
        _: ::core::ffi::c_int,
    );
    fn options_get(_: *mut options, _: *const ::core::ffi::c_char) -> *mut options_entry;
    fn options_array_getv(
        _: *mut options_entry,
        _: *const ::core::ffi::c_char,
        ...
    ) -> *mut options_value;
    fn options_get_string(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> *const ::core::ffi::c_char;
    fn options_get_number(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn options_string_to_style(
        _: *mut options,
        _: *const ::core::ffi::c_char,
        _: *mut format_tree,
    ) -> *mut style;
    fn cmdq_get_callback1(
        _: *const ::core::ffi::c_char,
        _: cmdq_cb,
        _: *mut ::core::ffi::c_void,
    ) -> *mut cmdq_item;
    fn cmdq_append(_: *mut client, _: *mut cmdq_item) -> *mut cmdq_item;
    static mut clients: clients;
    fn server_add_message(_: *const ::core::ffi::c_char, ...);
    fn server_client_clear_overlay(_: *mut client);
    fn prompt_set_options(_: *mut prompt_create_data, _: *mut session);
    fn prompt_create(_: *const prompt_create_data) -> *mut prompt;
    fn prompt_free(_: *mut prompt);
    fn prompt_incremental_start(_: *mut prompt);
    fn prompt_draw(_: *mut prompt, _: *mut prompt_draw_data);
    fn prompt_key(_: *mut prompt, _: key_code, _: *mut ::core::ffi::c_int) -> prompt_key_result;
    fn prompt_mouse(
        _: *mut prompt,
        _: u_int,
        _: u_int,
        _: u_int,
        _: *mut ::core::ffi::c_int,
    ) -> prompt_key_result;
    fn prompt_update(_: *mut prompt, _: *const ::core::ffi::c_char, _: *const ::core::ffi::c_char);
    fn prompt_closed(_: *mut prompt) -> ::core::ffi::c_int;
    fn grid_cells_equal(_: *const grid_cell, _: *const grid_cell) -> ::core::ffi::c_int;
    fn grid_compare(_: *mut grid, _: *mut grid) -> ::core::ffi::c_int;
    fn screen_write_start(_: *mut screen_write_ctx, _: *mut screen);
    fn screen_write_stop(_: *mut screen_write_ctx);
    fn screen_write_putc(_: *mut screen_write_ctx, _: *const grid_cell, _: u_char);
    fn screen_write_fast_copy(
        _: *mut screen_write_ctx,
        _: *mut screen,
        _: u_int,
        _: u_int,
        _: u_int,
        _: u_int,
    );
    fn screen_write_cursormove(
        _: *mut screen_write_ctx,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    );
    fn screen_init(_: *mut screen, _: u_int, _: u_int, _: u_int);
    fn screen_free(_: *mut screen);
    fn screen_resize(_: *mut screen, _: u_int, _: u_int, _: ::core::ffi::c_int);
    fn log_debug(_: *const ::core::ffi::c_char, ...);
    fn fatalx(_: *const ::core::ffi::c_char, ...) -> !;
    fn style_apply(
        _: *mut grid_cell,
        _: *mut options,
        _: *const ::core::ffi::c_char,
        _: *mut format_tree,
    );
    fn style_ranges_init(_: *mut style_ranges);
    fn style_ranges_free(_: *mut style_ranges);
    fn style_ranges_get_range(_: *mut style_ranges, _: u_int) -> *mut style_range;
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_12 {
    pub offset: u_int,
    pub data: C2RustUnnamed_13,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_13 {
    pub attr: u_char,
    pub fg: u_char,
    pub bg: u_char,
    pub data: u_char,
}

pub type C2RustUnnamed_38 = ::core::ffi::c_ulong;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct status_prompt_data {
    pub c: *mut client,
    pub inputcb: status_prompt_input_cb,
    pub freecb: prompt_free_cb,
    pub data: *mut ::core::ffi::c_void,
}

unsafe extern "C" fn status_timer_callback(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut c: *mut client = arg as *mut client;
    let mut s: *mut session = (*c).session;
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    event_del(&raw mut (*c).status.timer);
    if s.is_null() {
        return;
    }
    if (*c).message_string.is_null() && (*c).prompt.is_null() {
        (*c).flags |= CLIENT_REDRAWSTATUS as uint64_t;
    }
    tv.tv_usec = 0 as __suseconds_t;
    tv.tv_sec = tv.tv_usec as __time_t;
    tv.tv_sec = options_get_number(
        (*s).options,
        b"status-interval\0" as *const u8 as *const ::core::ffi::c_char,
    ) as __time_t;
    if tv.tv_sec != 0 as __time_t {
        event_add(&raw mut (*c).status.timer, &raw mut tv);
    }
    log_debug(
        b"client %p, status interval %d\0" as *const u8 as *const ::core::ffi::c_char,
        c,
        tv.tv_sec as ::core::ffi::c_int,
    );
}
#[no_mangle]
pub unsafe extern "C" fn status_timer_start(mut c: *mut client) {
    let mut s: *mut session = (*c).session;
    if event_initialized(&raw mut (*c).status.timer) != 0 {
        event_del(&raw mut (*c).status.timer);
    } else {
        event_set(
            &raw mut (*c).status.timer,
            -(1 as ::core::ffi::c_int),
            0 as ::core::ffi::c_short,
            Some(
                status_timer_callback
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        ::core::ffi::c_short,
                        *mut ::core::ffi::c_void,
                    ) -> (),
            ),
            c as *mut ::core::ffi::c_void,
        );
    }
    if !s.is_null()
        && options_get_number(
            (*s).options,
            b"status\0" as *const u8 as *const ::core::ffi::c_char,
        ) != 0
    {
        status_timer_callback(
            -(1 as ::core::ffi::c_int),
            0 as ::core::ffi::c_short,
            c as *mut ::core::ffi::c_void,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn status_timer_start_all() {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    c = clients.tqh_first;
    while !c.is_null() {
        status_timer_start(c);
        c = (*c).entry.tqe_next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn status_update_cache(mut s: *mut session) {
    (*s).statuslines = options_get_number(
        (*s).options,
        b"status\0" as *const u8 as *const ::core::ffi::c_char,
    ) as u_int;
    if (*s).statuslines == 0 as u_int {
        (*s).statusat = -(1 as ::core::ffi::c_int);
    } else if options_get_number(
        (*s).options,
        b"status-position\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_longlong
    {
        (*s).statusat = 0 as ::core::ffi::c_int;
    } else {
        (*s).statusat = 1 as ::core::ffi::c_int;
    };
}
#[no_mangle]
pub unsafe extern "C" fn status_at_line(mut c: *mut client) -> ::core::ffi::c_int {
    let mut s: *mut session = (*c).session;
    if (*c).flags & (CLIENT_STATUSOFF | CLIENT_CONTROL) as uint64_t != 0 {
        return -(1 as ::core::ffi::c_int);
    }
    if (*s).statusat != 1 as ::core::ffi::c_int {
        return (*s).statusat;
    }
    return (*c).tty.sy.wrapping_sub(status_line_size(c)) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn status_line_size(mut c: *mut client) -> u_int {
    let mut s: *mut session = (*c).session;
    if (*c).flags & (CLIENT_STATUSOFF | CLIENT_CONTROL) as uint64_t != 0 {
        return 0 as u_int;
    }
    if s.is_null() {
        return options_get_number(
            global_s_options,
            b"status\0" as *const u8 as *const ::core::ffi::c_char,
        ) as u_int;
    }
    return (*s).statuslines;
}
#[no_mangle]
pub unsafe extern "C" fn status_prompt_line_at(mut c: *mut client) -> u_int {
    let mut s: *mut session = (*c).session;
    let mut line: u_int = 0;
    let mut lines: u_int = 0;
    lines = status_line_size(c);
    if lines == 0 as u_int {
        return 0 as u_int;
    }
    line = options_get_number(
        (*s).options,
        b"message-line\0" as *const u8 as *const ::core::ffi::c_char,
    ) as u_int;
    if line >= lines {
        return lines.wrapping_sub(1 as u_int);
    }
    return line;
}
#[no_mangle]
pub unsafe extern "C" fn status_get_range(
    mut c: *mut client,
    mut x: u_int,
    mut y: u_int,
) -> *mut style_range {
    let mut sl: *mut status_line = &raw mut (*c).status;
    if y as usize
        >= (::core::mem::size_of::<[style_line_entry; 5]>() as usize)
            .wrapping_div(::core::mem::size_of::<style_line_entry>() as usize)
    {
        return ::core::ptr::null_mut::<style_range>();
    }
    return style_ranges_get_range(
        &raw mut (*(&raw mut (*sl).entries as *mut style_line_entry).offset(y as isize)).ranges,
        x,
    );
}
unsafe extern "C" fn status_push_screen(mut c: *mut client) {
    let mut sl: *mut status_line = &raw mut (*c).status;
    if (*sl).active == &raw mut (*sl).screen {
        (*sl).active = xmalloc(::core::mem::size_of::<screen>() as size_t) as *mut screen;
        screen_init((*sl).active, (*c).tty.sx, status_line_size(c), 0 as u_int);
    }
    (*sl).references += 1;
}
unsafe extern "C" fn status_pop_screen(mut c: *mut client) {
    let mut sl: *mut status_line = &raw mut (*c).status;
    (*sl).references -= 1;
    if (*sl).references == 0 as ::core::ffi::c_int {
        screen_free((*sl).active);
        free((*sl).active as *mut ::core::ffi::c_void);
        (*sl).active = &raw mut (*sl).screen;
    }
}
#[no_mangle]
pub unsafe extern "C" fn status_init(mut c: *mut client) {
    let mut sl: *mut status_line = &raw mut (*c).status;
    let mut i: u_int = 0;
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[style_line_entry; 5]>() as usize)
            .wrapping_div(::core::mem::size_of::<style_line_entry>() as usize)
    {
        style_ranges_init(
            &raw mut (*(&raw mut (*sl).entries as *mut style_line_entry).offset(i as isize)).ranges,
        );
        i = i.wrapping_add(1);
    }
    screen_init(&raw mut (*sl).screen, (*c).tty.sx, 1 as u_int, 0 as u_int);
    (*sl).active = &raw mut (*sl).screen;
}
#[no_mangle]
pub unsafe extern "C" fn status_free(mut c: *mut client) {
    let mut sl: *mut status_line = &raw mut (*c).status;
    let mut i: u_int = 0;
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[style_line_entry; 5]>() as usize)
            .wrapping_div(::core::mem::size_of::<style_line_entry>() as usize)
    {
        style_ranges_free(
            &raw mut (*(&raw mut (*sl).entries as *mut style_line_entry).offset(i as isize)).ranges,
        );
        free((*sl).entries[i as usize].expanded as *mut ::core::ffi::c_void);
        i = i.wrapping_add(1);
    }
    if event_initialized(&raw mut (*sl).timer) != 0 {
        event_del(&raw mut (*sl).timer);
    }
    if (*sl).active != &raw mut (*sl).screen {
        screen_free((*sl).active);
        free((*sl).active as *mut ::core::ffi::c_void);
    }
    screen_free(&raw mut (*sl).screen);
}
#[no_mangle]
pub unsafe extern "C" fn status_redraw(mut c: *mut client) -> ::core::ffi::c_int {
    let mut sl: *mut status_line = &raw mut (*c).status;
    let mut sle: *mut style_line_entry = ::core::ptr::null_mut::<style_line_entry>();
    let mut s: *mut session = (*c).session;
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: ::core::ptr::null_mut::<window_pane>(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        item: ::core::ptr::null_mut::<screen_write_citem>(),
        scrolled: 0,
        bg: 0,
    };
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
    let mut lines: u_int = 0;
    let mut i: u_int = 0;
    let mut n: u_int = 0;
    let mut width: u_int = (*c).tty.sx;
    let mut flags: ::core::ffi::c_int = 0;
    let mut force: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut changed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut fg: ::core::ffi::c_int = 0;
    let mut bg: ::core::ffi::c_int = 0;
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut ov: *mut options_value = ::core::ptr::null_mut::<options_value>();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    log_debug(
        b"%s enter\0" as *const u8 as *const ::core::ffi::c_char,
        b"status_redraw\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if (*sl).active != &raw mut (*sl).screen {
        fatalx(b"not the active screen\0" as *const u8 as *const ::core::ffi::c_char);
    }
    lines = status_line_size(c);
    if (*c).tty.sy == 0 as u_int || lines == 0 as u_int {
        return 1 as ::core::ffi::c_int;
    }
    flags = FORMAT_STATUS;
    if (*c).flags & CLIENT_STATUSFORCE as uint64_t != 0 {
        flags |= FORMAT_FORCE;
    }
    ft = format_create(c, ::core::ptr::null_mut::<cmdq_item>(), FORMAT_NONE, flags);
    format_defaults(
        ft,
        c,
        ::core::ptr::null_mut::<session>(),
        ::core::ptr::null_mut::<winlink>(),
        ::core::ptr::null_mut::<window_pane>(),
    );
    style_apply(
        &raw mut gc,
        (*s).options,
        b"status-style\0" as *const u8 as *const ::core::ffi::c_char,
        ft,
    );
    fg = options_get_number(
        (*s).options,
        b"status-fg\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    if !(fg == 8 as ::core::ffi::c_int || fg == 9 as ::core::ffi::c_int) {
        gc.fg = fg;
    }
    bg = options_get_number(
        (*s).options,
        b"status-bg\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    if !(bg == 8 as ::core::ffi::c_int || bg == 9 as ::core::ffi::c_int) {
        gc.bg = bg;
    }
    if grid_cells_equal(&raw mut gc, &raw mut (*sl).style) == 0 {
        force = 1 as ::core::ffi::c_int;
        memcpy(
            &raw mut (*sl).style as *mut ::core::ffi::c_void,
            &raw mut gc as *const ::core::ffi::c_void,
            ::core::mem::size_of::<grid_cell>() as size_t,
        );
    }
    if (*(*sl).screen.grid).sx != width || (*(*sl).screen.grid).sy != lines {
        screen_resize(&raw mut (*sl).screen, width, lines, 0 as ::core::ffi::c_int);
        force = 1 as ::core::ffi::c_int;
        changed = force;
    }
    screen_write_start(&raw mut ctx, &raw mut (*sl).screen);
    o = options_get(
        (*s).options,
        b"status-format\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if o.is_null() {
        n = 0 as u_int;
        while n < width.wrapping_mul(lines) {
            screen_write_putc(&raw mut ctx, &raw mut gc, ' ' as i32 as u_char);
            n = n.wrapping_add(1);
        }
    } else {
        i = 0 as u_int;
        while i < lines {
            screen_write_cursormove(
                &raw mut ctx,
                0 as ::core::ffi::c_int,
                i as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            ov = options_array_getv(o, b"%u\0" as *const u8 as *const ::core::ffi::c_char, i);
            if ov.is_null() {
                n = 0 as u_int;
                while n < width {
                    screen_write_putc(&raw mut ctx, &raw mut gc, ' ' as i32 as u_char);
                    n = n.wrapping_add(1);
                }
            } else {
                sle = (&raw mut (*sl).entries as *mut style_line_entry).offset(i as isize)
                    as *mut style_line_entry;
                expanded = format_expand_time(ft, (*ov).string);
                if force == 0
                    && !(*sle).expanded.is_null()
                    && strcmp(expanded, (*sle).expanded) == 0 as ::core::ffi::c_int
                {
                    free(expanded as *mut ::core::ffi::c_void);
                } else {
                    changed = 1 as ::core::ffi::c_int;
                    n = 0 as u_int;
                    while n < width {
                        screen_write_putc(&raw mut ctx, &raw mut gc, ' ' as i32 as u_char);
                        n = n.wrapping_add(1);
                    }
                    screen_write_cursormove(
                        &raw mut ctx,
                        0 as ::core::ffi::c_int,
                        i as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    );
                    style_ranges_free(&raw mut (*sle).ranges);
                    format_draw(
                        &raw mut ctx,
                        &raw mut gc,
                        width,
                        expanded,
                        &raw mut (*sle).ranges,
                        0 as ::core::ffi::c_int,
                    );
                    free((*sle).expanded as *mut ::core::ffi::c_void);
                    (*sle).expanded = expanded;
                }
            }
            i = i.wrapping_add(1);
        }
    }
    screen_write_stop(&raw mut ctx);
    format_free(ft);
    log_debug(
        b"%s exit: force=%d, changed=%d\0" as *const u8 as *const ::core::ffi::c_char,
        b"status_redraw\0" as *const u8 as *const ::core::ffi::c_char,
        force,
        changed,
    );
    return (force != 0 || changed != 0) as ::core::ffi::c_int;
}
unsafe extern "C" fn status_message_escape(
    mut s: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut cp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut out: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut n: size_t = 0 as size_t;
    cp = s;
    while *cp as ::core::ffi::c_int != '\0' as i32 {
        if *cp as ::core::ffi::c_int == '#' as i32 {
            n = n.wrapping_add(1);
        }
        cp = cp.offset(1);
    }
    out = xmalloc(strlen(s).wrapping_add(n).wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
    p = out;
    cp = s;
    while *cp as ::core::ffi::c_int != '\0' as i32 {
        if *cp as ::core::ffi::c_int == '#' as i32 {
            let fresh0 = p;
            p = p.offset(1);
            *fresh0 = '#' as i32 as ::core::ffi::c_char;
        }
        let fresh1 = p;
        p = p.offset(1);
        *fresh1 = *cp;
        cp = cp.offset(1);
    }
    *p = '\0' as i32 as ::core::ffi::c_char;
    return out;
}
#[no_mangle]
pub unsafe extern "C" fn status_message_set(
    mut c: *mut client,
    mut delay: ::core::ffi::c_int,
    mut ignore_styles: ::core::ffi::c_int,
    mut ignore_keys: ::core::ffi::c_int,
    mut no_freeze: ::core::ffi::c_int,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    let mut ap: ::core::ffi::VaList;
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    ap = args.clone();
    xvasprintf(&raw mut s, fmt, ap);
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"status_message_set\0" as *const u8 as *const ::core::ffi::c_char,
        s,
    );
    if c.is_null() {
        server_add_message(
            b"message: %s\0" as *const u8 as *const ::core::ffi::c_char,
            s,
        );
        free(s as *mut ::core::ffi::c_void);
        return;
    }
    status_message_clear(c);
    status_push_screen(c);
    (*c).message_string = s;
    server_add_message(
        b"%s message: %s\0" as *const u8 as *const ::core::ffi::c_char,
        (*c).name,
        s,
    );
    if delay == -(1 as ::core::ffi::c_int) {
        delay = options_get_number(
            (*(*c).session).options,
            b"display-time\0" as *const u8 as *const ::core::ffi::c_char,
        ) as ::core::ffi::c_int;
    }
    if delay > 0 as ::core::ffi::c_int {
        tv.tv_sec = (delay / 1000 as ::core::ffi::c_int) as __time_t;
        tv.tv_usec = ((delay % 1000 as ::core::ffi::c_int) as ::core::ffi::c_long
            * 1000 as ::core::ffi::c_long) as __suseconds_t;
        if event_initialized(&raw mut (*c).message_timer) != 0 {
            event_del(&raw mut (*c).message_timer);
        }
        event_set(
            &raw mut (*c).message_timer,
            -(1 as ::core::ffi::c_int),
            0 as ::core::ffi::c_short,
            Some(
                status_message_callback
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        ::core::ffi::c_short,
                        *mut ::core::ffi::c_void,
                    ) -> (),
            ),
            c as *mut ::core::ffi::c_void,
        );
        event_add(&raw mut (*c).message_timer, &raw mut tv);
    }
    if delay != 0 as ::core::ffi::c_int {
        (*c).message_ignore_keys = ignore_keys;
    }
    (*c).message_ignore_styles = ignore_styles;
    if no_freeze == 0 {
        (*c).tty.flags |= TTY_FREEZE;
    }
    (*c).tty.flags |= TTY_NOCURSOR;
    (*c).flags |= CLIENT_REDRAWSTATUS as uint64_t;
}
#[no_mangle]
pub unsafe extern "C" fn status_message_clear(mut c: *mut client) {
    if (*c).message_string.is_null() {
        return;
    }
    free((*c).message_string as *mut ::core::ffi::c_void);
    (*c).message_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*c).prompt.is_null() {
        (*c).tty.flags &= !(TTY_NOCURSOR | TTY_FREEZE);
    }
    (*c).flags |= CLIENT_ALLREDRAWFLAGS as uint64_t;
    status_pop_screen(c);
}
unsafe extern "C" fn status_message_area(
    mut c: *mut client,
    mut area_x: *mut u_int,
    mut area_w: *mut u_int,
) {
    let mut s: *mut session = (*c).session;
    let mut sy: *mut style = ::core::ptr::null_mut::<style>();
    let mut w: u_int = 0;
    sy = options_string_to_style(
        (*s).options,
        b"message-style\0" as *const u8 as *const ::core::ffi::c_char,
        ::core::ptr::null_mut::<format_tree>(),
    );
    if !sy.is_null() && (*sy).width >= 0 as ::core::ffi::c_int {
        if (*sy).width_percentage != 0 {
            w = (*c)
                .tty
                .sx
                .wrapping_mul((*sy).width as u_int)
                .wrapping_div(100 as u_int);
        } else {
            w = (*sy).width as u_int;
        }
    } else {
        w = (*c).tty.sx;
    }
    if w == 0 as u_int || w > (*c).tty.sx {
        w = (*c).tty.sx;
    }
    if !sy.is_null() {
        match (*sy).align as ::core::ffi::c_uint {
            2 | 4 => {
                *area_x = (*c).tty.sx.wrapping_sub(w).wrapping_div(2 as u_int);
            }
            3 => {
                *area_x = (*c).tty.sx.wrapping_sub(w);
            }
            _ => {
                *area_x = 0 as u_int;
            }
        }
    } else {
        *area_x = 0 as u_int;
    }
    *area_w = w;
}
unsafe extern "C" fn status_message_callback(
    mut fd: ::core::ffi::c_int,
    mut event: ::core::ffi::c_short,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut c: *mut client = data as *mut client;
    status_message_clear(c);
}
#[no_mangle]
pub unsafe extern "C" fn status_message_redraw(mut c: *mut client) -> ::core::ffi::c_int {
    let mut sl: *mut status_line = &raw mut (*c).status;
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: ::core::ptr::null_mut::<window_pane>(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        item: ::core::ptr::null_mut::<screen_write_citem>(),
        scrolled: 0,
        bg: 0,
    };
    let mut s: *mut session = (*c).session;
    let mut old_screen: screen = screen {
        title: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        path: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        titles: ::core::ptr::null_mut::<screen_titles>(),
        ntitles: 0,
        grid: ::core::ptr::null_mut::<grid>(),
        cx: 0,
        cy: 0,
        cstyle: SCREEN_CURSOR_DEFAULT,
        default_cstyle: SCREEN_CURSOR_DEFAULT,
        ccolour: 0,
        default_ccolour: 0,
        rupper: 0,
        rlower: 0,
        mode: 0,
        default_mode: 0,
        saved_cx: 0,
        saved_cy: 0,
        saved_grid: ::core::ptr::null_mut::<grid>(),
        saved_cell: grid_cell {
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
        saved_flags: 0,
        tabs: ::core::ptr::null_mut::<bitstr_t>(),
        sel: ::core::ptr::null_mut::<screen_sel>(),
        write_list: ::core::ptr::null_mut::<screen_write_cline>(),
        hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        progress_bar: progress_bar {
            state: PROGRESS_BAR_HIDDEN,
            progress: 0,
        },
    };
    let mut lines: u_int = 0;
    let mut messageline: u_int = 0;
    let mut ax: u_int = 0;
    let mut aw: u_int = 0;
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
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut msgfmt: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut msg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*c).tty.sx == 0 as u_int || (*c).tty.sy == 0 as u_int {
        return 0 as ::core::ffi::c_int;
    }
    memcpy(
        &raw mut old_screen as *mut ::core::ffi::c_void,
        (*sl).active as *const ::core::ffi::c_void,
        ::core::mem::size_of::<screen>() as size_t,
    );
    lines = status_line_size(c);
    if lines <= 1 as u_int {
        lines = 1 as u_int;
    }
    screen_init((*sl).active, (*c).tty.sx, lines, 0 as u_int);
    messageline = status_prompt_line_at(c);
    if messageline > lines.wrapping_sub(1 as u_int) {
        messageline = lines.wrapping_sub(1 as u_int);
    }
    status_message_area(c, &raw mut ax, &raw mut aw);
    ft = format_create_defaults(
        ::core::ptr::null_mut::<cmdq_item>(),
        c,
        ::core::ptr::null_mut::<session>(),
        ::core::ptr::null_mut::<winlink>(),
        ::core::ptr::null_mut::<window_pane>(),
    );
    style_apply(
        &raw mut gc,
        (*s).options,
        b"message-style\0" as *const u8 as *const ::core::ffi::c_char,
        ft,
    );
    if (*c).message_ignore_styles != 0 {
        msg = status_message_escape((*c).message_string);
        format_add(
            ft,
            b"message\0" as *const u8 as *const ::core::ffi::c_char,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            msg,
        );
        free(msg as *mut ::core::ffi::c_void);
    } else {
        format_add(
            ft,
            b"message\0" as *const u8 as *const ::core::ffi::c_char,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).message_string,
        );
    }
    format_add(
        ft,
        b"command_prompt\0" as *const u8 as *const ::core::ffi::c_char,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        0 as ::core::ffi::c_int,
    );
    msgfmt = options_get_string(
        (*s).options,
        b"message-format\0" as *const u8 as *const ::core::ffi::c_char,
    );
    expanded = format_expand_time(ft, msgfmt);
    format_free(ft);
    screen_write_start(&raw mut ctx, (*sl).active);
    screen_write_fast_copy(
        &raw mut ctx,
        &raw mut (*sl).screen,
        0 as u_int,
        0 as u_int,
        (*c).tty.sx,
        lines,
    );
    screen_write_cursormove(
        &raw mut ctx,
        ax as ::core::ffi::c_int,
        messageline as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    format_draw(
        &raw mut ctx,
        &raw mut gc,
        aw,
        expanded,
        ::core::ptr::null_mut::<style_ranges>(),
        0 as ::core::ffi::c_int,
    );
    screen_write_stop(&raw mut ctx);
    free(expanded as *mut ::core::ffi::c_void);
    if grid_compare((*(*sl).active).grid, old_screen.grid) == 0 as ::core::ffi::c_int {
        screen_free(&raw mut old_screen);
        return 0 as ::core::ffi::c_int;
    }
    screen_free(&raw mut old_screen);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn status_prompt_input_callback(
    mut data: *mut ::core::ffi::c_void,
    mut s: *const ::core::ffi::c_char,
    mut key: prompt_key_result,
) -> prompt_result {
    let mut spd: *mut status_prompt_data = data as *mut status_prompt_data;
    let mut c: *mut client = (*spd).c;
    let mut inputcb: status_prompt_input_cb = (*spd).inputcb;
    let mut arg: *mut ::core::ffi::c_void = (*spd).data;
    if inputcb.is_some() {
        return inputcb.expect("non-null function pointer")(c, arg, s, key);
    }
    return PROMPT_CLOSE;
}
unsafe extern "C" fn status_prompt_free_callback(mut data: *mut ::core::ffi::c_void) {
    let mut spd: *mut status_prompt_data = data as *mut status_prompt_data;
    let mut freecb: prompt_free_cb = (*spd).freecb;
    let mut arg: *mut ::core::ffi::c_void = (*spd).data;
    if freecb.is_some() {
        freecb.expect("non-null function pointer")(arg);
    }
    free(spd as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn status_prompt_accept(
    mut item: *mut cmdq_item,
    mut data: *mut ::core::ffi::c_void,
) -> cmd_retval {
    let mut c: *mut client = data as *mut client;
    if !(*c).prompt.is_null() {
        status_prompt_key(
            c,
            'y' as i32 as key_code,
            ::core::ptr::null_mut::<mouse_event>(),
        );
    }
    return CMD_RETURN_NORMAL;
}
#[no_mangle]
pub unsafe extern "C" fn status_prompt_set(
    mut c: *mut client,
    mut fs: *mut cmd_find_state,
    mut msg: *const ::core::ffi::c_char,
    mut input: *const ::core::ffi::c_char,
    mut inputcb: status_prompt_input_cb,
    mut freecb: prompt_free_cb,
    mut data: *mut ::core::ffi::c_void,
    mut flags: ::core::ffi::c_int,
    mut prompt_type: prompt_type,
) {
    let mut pd: prompt_create_data = prompt_create_data {
        fs: ::core::ptr::null_mut::<cmd_find_state>(),
        prompt: ::core::ptr::null::<::core::ffi::c_char>(),
        input: ::core::ptr::null::<::core::ffi::c_char>(),
        type_0: PROMPT_TYPE_COMMAND,
        flags: 0,
        style: grid_cell {
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
        command_style: grid_cell {
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
        style_str: ::core::ptr::null::<::core::ffi::c_char>(),
        command_style_str: ::core::ptr::null::<::core::ffi::c_char>(),
        cstyle: SCREEN_CURSOR_DEFAULT,
        command_cstyle: SCREEN_CURSOR_DEFAULT,
        ccolour: 0,
        command_ccolour: 0,
        cmode: 0,
        command_cmode: 0,
        message_format: ::core::ptr::null::<::core::ffi::c_char>(),
        keys: 0,
        word_separators: ::core::ptr::null::<::core::ffi::c_char>(),
        inputcb: None,
        freecb: None,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut spd: *mut status_prompt_data = ::core::ptr::null_mut::<status_prompt_data>();
    server_client_clear_overlay(c);
    status_message_clear(c);
    status_prompt_clear(c);
    status_push_screen(c);
    spd = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<status_prompt_data>() as size_t,
    ) as *mut status_prompt_data;
    (*spd).c = c;
    (*spd).inputcb = inputcb;
    (*spd).freecb = freecb;
    (*spd).data = data;
    memset(
        &raw mut pd as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<prompt_create_data>() as size_t,
    );
    prompt_set_options(&raw mut pd, (*c).session);
    pd.fs = fs;
    pd.prompt = msg;
    pd.input = input;
    pd.type_0 = prompt_type;
    pd.flags = flags;
    pd.inputcb = Some(
        status_prompt_input_callback
            as unsafe extern "C" fn(
                *mut ::core::ffi::c_void,
                *const ::core::ffi::c_char,
                prompt_key_result,
            ) -> prompt_result,
    ) as prompt_input_cb;
    pd.freecb =
        Some(status_prompt_free_callback as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ())
            as prompt_free_cb;
    pd.data = spd as *mut ::core::ffi::c_void;
    (*c).prompt = prompt_create(&raw mut pd);
    if !flags & PROMPT_INCREMENTAL != 0 && !flags & PROMPT_NOFREEZE != 0 {
        (*c).tty.flags |= TTY_FREEZE;
    }
    (*c).flags |= CLIENT_REDRAWSTATUS as uint64_t;
    prompt_incremental_start((*c).prompt);
    if flags & PROMPT_SINGLE != 0 && flags & PROMPT_ACCEPT != 0 {
        cmdq_append(
            c,
            cmdq_get_callback1(
                b"status_prompt_accept\0" as *const u8 as *const ::core::ffi::c_char,
                Some(
                    status_prompt_accept
                        as unsafe extern "C" fn(
                            *mut cmdq_item,
                            *mut ::core::ffi::c_void,
                        ) -> cmd_retval,
                ),
                c as *mut ::core::ffi::c_void,
            ),
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn status_prompt_clear(mut c: *mut client) {
    if (*c).prompt.is_null() {
        return;
    }
    prompt_free((*c).prompt);
    (*c).prompt = ::core::ptr::null_mut::<prompt>();
    (*c).tty.flags &= !(TTY_NOCURSOR | TTY_FREEZE);
    (*c).flags |= CLIENT_ALLREDRAWFLAGS as uint64_t;
    status_pop_screen(c);
}
#[no_mangle]
pub unsafe extern "C" fn status_prompt_update(
    mut c: *mut client,
    mut msg: *const ::core::ffi::c_char,
    mut input: *const ::core::ffi::c_char,
) {
    if (*c).prompt.is_null() {
        return;
    }
    prompt_update((*c).prompt, msg, input);
    (*c).flags |= CLIENT_REDRAWSTATUS as uint64_t;
}
unsafe extern "C" fn status_prompt_screen_line(mut c: *mut client) -> u_int {
    let mut tty: *mut tty = &raw mut (*c).tty;
    let mut n: u_int = 0;
    if options_get_number(
        (*(*c).session).options,
        b"status-position\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_longlong
    {
        return status_prompt_line_at(c);
    }
    n = status_line_size(c).wrapping_sub(status_prompt_line_at(c));
    if n <= (*tty).sy {
        return (*tty).sy.wrapping_sub(n);
    }
    return (*tty).sy.wrapping_sub(1 as u_int);
}
#[no_mangle]
pub unsafe extern "C" fn status_prompt_redraw(mut c: *mut client) -> ::core::ffi::c_int {
    let mut sl: *mut status_line = &raw mut (*c).status;
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: ::core::ptr::null_mut::<window_pane>(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        item: ::core::ptr::null_mut::<screen_write_citem>(),
        scrolled: 0,
        bg: 0,
    };
    let mut old_screen: screen = screen {
        title: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        path: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        titles: ::core::ptr::null_mut::<screen_titles>(),
        ntitles: 0,
        grid: ::core::ptr::null_mut::<grid>(),
        cx: 0,
        cy: 0,
        cstyle: SCREEN_CURSOR_DEFAULT,
        default_cstyle: SCREEN_CURSOR_DEFAULT,
        ccolour: 0,
        default_ccolour: 0,
        rupper: 0,
        rlower: 0,
        mode: 0,
        default_mode: 0,
        saved_cx: 0,
        saved_cy: 0,
        saved_grid: ::core::ptr::null_mut::<grid>(),
        saved_cell: grid_cell {
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
        saved_flags: 0,
        tabs: ::core::ptr::null_mut::<bitstr_t>(),
        sel: ::core::ptr::null_mut::<screen_sel>(),
        write_list: ::core::ptr::null_mut::<screen_write_cline>(),
        hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        progress_bar: progress_bar {
            state: PROGRESS_BAR_HIDDEN,
            progress: 0,
        },
    };
    let mut pdd: prompt_draw_data = prompt_draw_data {
        ctx: ::core::ptr::null_mut::<screen_write_ctx>(),
        cursor_x: ::core::ptr::null_mut::<u_int>(),
        area_x: 0,
        area_width: 0,
        prompt_line: 0,
    };
    let mut lines: u_int = 0;
    let mut ax: u_int = 0;
    let mut aw: u_int = 0;
    let mut promptline: u_int = 0;
    if (*c).tty.sx == 0 as u_int || (*c).tty.sy == 0 as u_int {
        return 0 as ::core::ffi::c_int;
    }
    memcpy(
        &raw mut old_screen as *mut ::core::ffi::c_void,
        (*sl).active as *const ::core::ffi::c_void,
        ::core::mem::size_of::<screen>() as size_t,
    );
    lines = status_line_size(c);
    if lines <= 1 as u_int {
        lines = 1 as u_int;
    }
    screen_init((*sl).active, (*c).tty.sx, lines, 0 as u_int);
    promptline = status_prompt_line_at(c);
    if promptline > lines.wrapping_sub(1 as u_int) {
        promptline = lines.wrapping_sub(1 as u_int);
    }
    status_message_area(c, &raw mut ax, &raw mut aw);
    screen_write_start(&raw mut ctx, (*sl).active);
    screen_write_fast_copy(
        &raw mut ctx,
        &raw mut (*sl).screen,
        0 as u_int,
        0 as u_int,
        (*c).tty.sx,
        lines,
    );
    pdd.ctx = &raw mut ctx;
    pdd.area_x = ax;
    pdd.area_width = aw;
    pdd.prompt_line = promptline;
    pdd.cursor_x = &raw mut (*sl).prompt_cx;
    prompt_draw((*c).prompt, &raw mut pdd);
    screen_write_stop(&raw mut ctx);
    if grid_compare((*(*sl).active).grid, old_screen.grid) == 0 as ::core::ffi::c_int {
        screen_free(&raw mut old_screen);
        return 0 as ::core::ffi::c_int;
    }
    screen_free(&raw mut old_screen);
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn status_prompt_cursor(
    mut c: *mut client,
    mut cx: *mut u_int,
    mut cy: *mut u_int,
) {
    *cy = status_prompt_screen_line(c);
    *cx = (*c).status.prompt_cx;
}
#[no_mangle]
pub unsafe extern "C" fn status_prompt_key(
    mut c: *mut client,
    mut key: key_code,
    mut m: *mut mouse_event,
) -> prompt_key_result {
    let mut result: prompt_key_result = PROMPT_KEY_NOT_HANDLED;
    let mut ax: u_int = 0;
    let mut aw: u_int = 0;
    let mut redraw: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
        == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
        || key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
            >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                << 32 as ::core::ffi::c_int
            && key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int
    {
        if m.is_null()
            || (*m).b & MOUSE_MASK_BUTTONS as u_int != MOUSE_BUTTON_1 as u_int
            || (*m).b & MOUSE_MASK_DRAG as u_int != 0
            || (*m).b & MOUSE_MASK_BUTTONS as u_int == 3 as u_int
            || (*m).y != status_prompt_screen_line(c)
        {
            return PROMPT_KEY_NOT_HANDLED;
        }
        status_message_area(c, &raw mut ax, &raw mut aw);
        result = prompt_mouse((*c).prompt, (*m).x, ax, aw, &raw mut redraw);
    } else {
        result = prompt_key((*c).prompt, key, &raw mut redraw);
    }
    if redraw != 0 && !(*c).prompt.is_null() {
        (*c).flags |= CLIENT_REDRAWSTATUS as uint64_t;
    }
    if !(*c).prompt.is_null() && prompt_closed((*c).prompt) != 0 {
        status_prompt_clear(c);
    }
    return result;
}
