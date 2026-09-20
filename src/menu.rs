pub use crate::src::shared::command::{cmd_parse_input};
pub use crate::src::shared::arguments::{args};
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
pub use crate::src::shared::command::{
    cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmdq_state, cmds,
};
pub use crate::src::shared::control::{control_state};
pub use crate::src::shared::format::{format_job_tree, format_tree};
pub use crate::src::shared::input::{input_ctx, input_request, input_requests};
pub use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
pub use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
pub use crate::src::shared::menu::{menu_choice_cb, menu_data};
pub use crate::src::shared::options::{options};
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
pub use crate::src::shared::status::{status_line};
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
pub use crate::src::shared::limits::{__INT_MAX__, UINT_MAX};
pub use crate::src::shared::screen::{
    MODE_CURSOR, MODE_MOUSE_ALL, MODE_MOUSE_BUTTON, screen, screen_sel, screen_titles,
};
pub use crate::src::shared::menu::{MENU_NOMOUSE, MENU_STAYOPEN, MENU_TAB, menu, menu_item};
pub use crate::src::shared::screen_write::{screen_write_citem, screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{
    MOUSE_BUTTON_1, MOUSE_MASK_BUTTONS, MOUSE_MASK_DRAG, MOUSE_WHEEL_DOWN, MOUSE_WHEEL_UP,
    mouse_event,
};
use crate::src::shared::client::*;
use crate::src::shared::layout::*;
use crate::src::shared::command::*;
use crate::src::shared::terminal::*;
use crate::src::shared::event::*;
use crate::src::shared::display::*;
use crate::src::shared::layout::*;
use crate::src::shared::message::*;
use crate::src::shared::abi::*;
use crate::src::shared::colour::*;
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
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn xreallocarray(_: *mut ::core::ffi::c_void, _: size_t, _: size_t)
        -> *mut ::core::ffi::c_void;
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn xasprintf(
        _: *mut *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn format_free(_: *mut format_tree);
    fn format_single(
        _: *mut cmdq_item,
        _: *const ::core::ffi::c_char,
        _: *mut client,
        _: *mut session,
        _: *mut winlink,
        _: *mut window_pane,
    ) -> *mut ::core::ffi::c_char;
    fn format_single_from_state(
        _: *mut cmdq_item,
        _: *const ::core::ffi::c_char,
        _: *mut client,
        _: *mut cmd_find_state,
    ) -> *mut ::core::ffi::c_char;
    fn format_create_defaults(
        _: *mut cmdq_item,
        _: *mut client,
        _: *mut session,
        _: *mut winlink,
        _: *mut window_pane,
    ) -> *mut format_tree;
    fn format_width(_: *const ::core::ffi::c_char) -> u_int;
    fn format_trim_right(_: *const ::core::ffi::c_char, _: u_int) -> *mut ::core::ffi::c_char;
    fn options_get_number(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn cmd_find_clear_state(_: *mut cmd_find_state, _: ::core::ffi::c_int);
    fn cmd_find_copy_state(_: *mut cmd_find_state, _: *mut cmd_find_state);
    fn cmd_find_from_window(
        _: *mut cmd_find_state,
        _: *mut window,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn cmd_parse_and_append(
        _: *const ::core::ffi::c_char,
        _: *mut cmd_parse_input,
        _: *mut client,
        _: *mut cmdq_state,
        _: *mut *mut ::core::ffi::c_char,
    ) -> cmd_parse_status;
    fn cmdq_new_state(
        _: *mut cmd_find_state,
        _: *mut key_event,
        _: ::core::ffi::c_int,
    ) -> *mut cmdq_state;
    fn cmdq_free_state(_: *mut cmdq_state);
    fn cmdq_get_event(_: *mut cmdq_item) -> *mut key_event;
    fn cmdq_get_error(_: *const ::core::ffi::c_char) -> *mut cmdq_item;
    fn cmdq_append(_: *mut client, _: *mut cmdq_item) -> *mut cmdq_item;
    fn key_string_lookup_key(_: key_code, _: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn server_redraw_window(_: *mut window);
    fn server_redraw_window_menu(_: *mut window);
    static grid_default_cell: grid_cell;
    fn screen_write_start(_: *mut screen_write_ctx, _: *mut screen);
    fn screen_write_stop(_: *mut screen_write_ctx);
    fn screen_write_menu(
        _: *mut screen_write_ctx,
        _: *mut menu,
        _: ::core::ffi::c_int,
        _: box_lines,
        _: *const grid_cell,
        _: *const grid_cell,
        _: *const grid_cell,
    );
    fn screen_write_box(
        _: *mut screen_write_ctx,
        _: u_int,
        _: u_int,
        _: box_lines,
        _: *const grid_cell,
        _: *const ::core::ffi::c_char,
    );
    fn screen_write_clearscreen(_: *mut screen_write_ctx, _: u_int);
    fn redraw_invalidate_scene(_: *mut window);
    fn screen_init(_: *mut screen, _: u_int, _: u_int, _: u_int);
    fn screen_free(_: *mut screen);
    fn window_update_focus(_: *mut window);
    fn style_parse(
        _: *mut style,
        _: *const grid_cell,
        _: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn style_apply(
        _: *mut grid_cell,
        _: *mut options,
        _: *const ::core::ffi::c_char,
        _: *mut format_tree,
    );
    fn style_set(_: *mut style, _: *const grid_cell);
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

#[no_mangle]
pub unsafe extern "C" fn menu_add_items(
    mut menu: *mut menu,
    mut items: *const menu_item,
    mut qitem: *mut cmdq_item,
    mut c: *mut client,
    mut fs: *mut cmd_find_state,
) {
    let mut loop_0: *const menu_item = ::core::ptr::null::<menu_item>();
    loop_0 = items;
    while !(*loop_0).name.is_null() {
        menu_add_item(menu, loop_0, qitem, c, fs);
        loop_0 = loop_0.offset(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn menu_add_item(
    mut menu: *mut menu,
    mut item: *const menu_item,
    mut qitem: *mut cmdq_item,
    mut c: *mut client,
    mut fs: *mut cmd_find_state,
) {
    let mut new_item: *mut menu_item = ::core::ptr::null_mut::<menu_item>();
    let mut key: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut cmd: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut suffix: *const ::core::ffi::c_char = b"\0" as *const u8 as *const ::core::ffi::c_char;
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut trimmed: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut width: u_int = 0;
    let mut max_width: u_int = 0;
    let mut line: ::core::ffi::c_int = 0;
    let mut keylen: size_t = 0;
    let mut slen: size_t = 0;
    line = (item.is_null()
        || (*item).name.is_null()
        || *(*item).name as ::core::ffi::c_int == '\0' as i32) as ::core::ffi::c_int;
    if line != 0 && (*menu).count == 0 as u_int {
        return;
    }
    if line != 0
        && (*(*menu)
            .items
            .offset((*menu).count.wrapping_sub(1 as u_int) as isize))
        .name
        .is_null()
    {
        return;
    }
    (*menu).items = xreallocarray(
        (*menu).items as *mut ::core::ffi::c_void,
        (*menu).count.wrapping_add(1 as u_int) as size_t,
        ::core::mem::size_of::<menu_item>() as size_t,
    ) as *mut menu_item;
    let fresh0 = (*menu).count;
    (*menu).count = (*menu).count.wrapping_add(1);
    new_item = (*menu).items.offset(fresh0 as isize) as *mut menu_item;
    memset(
        new_item as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<menu_item>() as size_t,
    );
    if line != 0 {
        return;
    }
    if !fs.is_null() {
        s = format_single_from_state(qitem, (*item).name, c, fs);
    } else {
        s = format_single(
            qitem,
            (*item).name,
            c,
            ::core::ptr::null_mut::<session>(),
            ::core::ptr::null_mut::<winlink>(),
            ::core::ptr::null_mut::<window_pane>(),
        );
    }
    if *s as ::core::ffi::c_int == '\0' as i32 {
        free(s as *mut ::core::ffi::c_void);
        (*menu).count = (*menu).count.wrapping_sub(1);
        return;
    }
    max_width = (*c).tty.sx.wrapping_sub(4 as u_int);
    slen = strlen(s);
    if *s as ::core::ffi::c_int != '-' as i32
        && (*item).key != KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code
        && (*item).key != KEYC_NONE as ::core::ffi::c_ulong as key_code
    {
        key = key_string_lookup_key((*item).key, 0 as ::core::ffi::c_int);
        keylen = strlen(key).wrapping_add(3 as size_t);
        if keylen <= max_width.wrapping_div(4 as u_int) as size_t {
            max_width = (max_width as size_t).wrapping_sub(keylen) as u_int as u_int;
        } else if keylen >= max_width as size_t
            || slen >= (max_width as size_t).wrapping_sub(keylen)
        {
            key = ::core::ptr::null::<::core::ffi::c_char>();
        }
    }
    if slen > max_width as size_t {
        max_width = max_width.wrapping_sub(1);
        suffix = b">\0" as *const u8 as *const ::core::ffi::c_char;
    }
    trimmed = format_trim_right(s, max_width);
    if !key.is_null() {
        xasprintf(
            &raw mut name,
            b"%s%s#[default] #[align=right](%s)\0" as *const u8 as *const ::core::ffi::c_char,
            trimmed,
            suffix,
            key,
        );
    } else {
        xasprintf(
            &raw mut name,
            b"%s%s\0" as *const u8 as *const ::core::ffi::c_char,
            trimmed,
            suffix,
        );
    }
    free(trimmed as *mut ::core::ffi::c_void);
    (*new_item).name = name;
    free(s as *mut ::core::ffi::c_void);
    cmd = (*item).command;
    if !cmd.is_null() {
        if !fs.is_null() {
            s = format_single_from_state(qitem, cmd, c, fs);
        } else {
            s = format_single(
                qitem,
                cmd,
                c,
                ::core::ptr::null_mut::<session>(),
                ::core::ptr::null_mut::<winlink>(),
                ::core::ptr::null_mut::<window_pane>(),
            );
        }
    } else {
        s = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    (*new_item).command = s;
    (*new_item).key = (*item).key;
    width = format_width((*new_item).name);
    if *(*new_item).name as ::core::ffi::c_int == '-' as i32 {
        width = width.wrapping_sub(1);
    }
    if width > (*menu).width {
        (*menu).width = width;
    }
}
#[no_mangle]
pub unsafe extern "C" fn menu_create(mut title: *const ::core::ffi::c_char) -> *mut menu {
    let mut menu: *mut menu = ::core::ptr::null_mut::<menu>();
    menu = xcalloc(1 as size_t, ::core::mem::size_of::<menu>() as size_t) as *mut menu;
    (*menu).title = xstrdup(title);
    (*menu).width = format_width(title);
    return menu;
}
#[no_mangle]
pub unsafe extern "C" fn menu_free(mut menu: *mut menu) {
    let mut i: u_int = 0;
    if menu.is_null() {
        return;
    }
    i = 0 as u_int;
    while i < (*menu).count {
        free((*(*menu).items.offset(i as isize)).name as *mut ::core::ffi::c_void);
        free((*(*menu).items.offset(i as isize)).command as *mut ::core::ffi::c_void);
        i = i.wrapping_add(1);
    }
    free((*menu).items as *mut ::core::ffi::c_void);
    free((*menu).title as *mut ::core::ffi::c_void);
    free(menu as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn menu_reapply_styles(mut md: *mut menu_data) {
    let mut o: *mut options = (*(*md).w).options;
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
    ft = format_create_defaults(
        ::core::ptr::null_mut::<cmdq_item>(),
        ::core::ptr::null_mut::<client>(),
        (*md).fs.s,
        (*md).fs.wl,
        (*md).fs.wp,
    );
    memcpy(
        &raw mut (*md).style_gc as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    style_apply(
        &raw mut (*md).style_gc,
        o,
        b"menu-style\0" as *const u8 as *const ::core::ffi::c_char,
        ft,
    );
    if !(*md).style.is_null() {
        style_set(&raw mut sytmp, &raw const grid_default_cell);
        if style_parse(&raw mut sytmp, &raw mut (*md).style_gc, (*md).style)
            == 0 as ::core::ffi::c_int
        {
            (*md).style_gc.fg = sytmp.gc.fg;
            (*md).style_gc.bg = sytmp.gc.bg;
        }
    }
    memcpy(
        &raw mut (*md).selected_style_gc as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    style_apply(
        &raw mut (*md).selected_style_gc,
        o,
        b"menu-selected-style\0" as *const u8 as *const ::core::ffi::c_char,
        ft,
    );
    if !(*md).selected_style.is_null() {
        style_set(&raw mut sytmp, &raw const grid_default_cell);
        if style_parse(
            &raw mut sytmp,
            &raw mut (*md).selected_style_gc,
            (*md).selected_style,
        ) == 0 as ::core::ffi::c_int
        {
            (*md).selected_style_gc.fg = sytmp.gc.fg;
            (*md).selected_style_gc.bg = sytmp.gc.bg;
        }
    }
    memcpy(
        &raw mut (*md).border_style_gc as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    style_apply(
        &raw mut (*md).border_style_gc,
        o,
        b"menu-border-style\0" as *const u8 as *const ::core::ffi::c_char,
        ft,
    );
    if !(*md).border_style.is_null() {
        style_set(&raw mut sytmp, &raw const grid_default_cell);
        if style_parse(
            &raw mut sytmp,
            &raw mut (*md).border_style_gc,
            (*md).border_style,
        ) == 0 as ::core::ffi::c_int
        {
            (*md).border_style_gc.fg = sytmp.gc.fg;
            (*md).border_style_gc.bg = sytmp.gc.bg;
        }
    }
    format_free(ft);
}
#[no_mangle]
pub unsafe extern "C" fn menu_update(mut md: *mut menu_data) {
    let mut s: *mut screen = &raw mut (*md).s;
    let mut menu: *mut menu = (*md).menu;
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
    menu_reapply_styles(md);
    screen_write_start(&raw mut ctx, s);
    screen_write_clearscreen(&raw mut ctx, 8 as u_int);
    if (*md).border_lines as ::core::ffi::c_int != BOX_LINES_NONE as ::core::ffi::c_int {
        screen_write_box(
            &raw mut ctx,
            (*menu).width.wrapping_add(4 as u_int),
            (*menu).count.wrapping_add(2 as u_int),
            (*md).border_lines,
            &raw mut (*md).border_style_gc,
            (*menu).title,
        );
    }
    screen_write_menu(
        &raw mut ctx,
        menu,
        (*md).choice,
        (*md).border_lines,
        &raw mut (*md).style_gc,
        &raw mut (*md).border_style_gc,
        &raw mut (*md).selected_style_gc,
    );
    screen_write_stop(&raw mut ctx);
}
unsafe extern "C" fn menu_free_data(mut md: *mut menu_data) {
    if !md.is_null() {
        if (*md).cb.is_some() {
            (*md).cb.expect("non-null function pointer")(
                (*md).menu,
                UINT_MAX,
                KEYC_NONE as ::core::ffi::c_ulong as key_code,
                (*md).data,
            );
        }
        screen_free(&raw mut (*md).s);
        menu_free((*md).menu);
        free((*md).style as *mut ::core::ffi::c_void);
        free((*md).selected_style as *mut ::core::ffi::c_void);
        free((*md).border_style as *mut ::core::ffi::c_void);
        free(md as *mut ::core::ffi::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn menu_close(mut w: *mut window) {
    if !(*w).menu.is_null() {
        menu_free_data((*w).menu);
        (*w).menu = ::core::ptr::null_mut::<menu_data>();
        redraw_invalidate_scene(w);
        window_update_focus(w);
        server_redraw_window(w);
    }
}
#[no_mangle]
pub unsafe extern "C" fn menu_destroy(mut w: *mut window) {
    menu_free_data((*w).menu);
    (*w).menu = ::core::ptr::null_mut::<menu_data>();
}
#[no_mangle]
pub unsafe extern "C" fn menu_get_cursor(
    mut md: *mut menu_data,
    mut cx: *mut u_int,
    mut cy: *mut u_int,
) {
    *cx = (*md).px.wrapping_add(2 as u_int);
    if (*md).choice == -(1 as ::core::ffi::c_int) {
        *cy = (*md).py;
    } else {
        *cy = (*md)
            .py
            .wrapping_add(1 as u_int)
            .wrapping_add((*md).choice as u_int);
    };
}
#[no_mangle]
pub unsafe extern "C" fn menu_screen(mut md: *mut menu_data) -> *mut screen {
    return &raw mut (*md).s;
}
#[no_mangle]
pub unsafe extern "C" fn menu_width(mut md: *mut menu_data) -> u_int {
    return (*(*md).menu).width.wrapping_add(4 as u_int);
}
#[no_mangle]
pub unsafe extern "C" fn menu_height(mut md: *mut menu_data) -> u_int {
    return (*(*md).menu).count.wrapping_add(2 as u_int);
}
#[no_mangle]
pub unsafe extern "C" fn menu_x(mut md: *mut menu_data) -> u_int {
    return (*md).px;
}
#[no_mangle]
pub unsafe extern "C" fn menu_y(mut md: *mut menu_data) -> u_int {
    return (*md).py;
}
#[no_mangle]
pub unsafe extern "C" fn menu_key(
    mut c: *mut client,
    mut md: *mut menu_data,
    mut event: *mut key_event,
) -> ::core::ffi::c_int {
    let mut current_block: u64;
    let mut menu: *mut menu = (*md).menu;
    let mut m: *mut mouse_event = &raw mut (*event).m;
    let mut i: u_int = 0;
    let mut n: ::core::ffi::c_int = (*menu).count as ::core::ffi::c_int;
    let mut old: ::core::ffi::c_int = (*md).choice;
    let mut move_0: ::core::ffi::c_int = 0;
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut item: *const menu_item = ::core::ptr::null::<menu_item>();
    let mut saved_event: key_event = key_event {
        client: ::core::ptr::null_mut::<client>(),
        key: 0,
        m: mouse_event {
            valid: 0,
            ignore: 0,
            key: 0,
            statusat: 0,
            statuslines: 0,
            x: 0,
            y: 0,
            b: 0,
            lx: 0,
            ly: 0,
            lb: 0,
            ox: 0,
            oy: 0,
            s: 0,
            w: 0,
            wp: 0,
            sgr_type: 0,
            sgr_b: 0,
        },
        buf: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        len: 0,
    };
    let mut state: *mut cmdq_state = ::core::ptr::null_mut::<cmdq_state>();
    let mut status: cmd_parse_status = CMD_PARSE_ERROR;
    let mut error: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut key: key_code = 0;
    if (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
        == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
        || (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
            >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                << 32 as ::core::ffi::c_int
            && (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int
    {
        move_0 = ((*m).b & MOUSE_MASK_DRAG as u_int != 0
            && (*m).b & MOUSE_MASK_BUTTONS as u_int == 3 as u_int)
            as ::core::ffi::c_int;
        if (*md).flags & MENU_NOMOUSE != 0 {
            if (*m).b & MOUSE_MASK_BUTTONS as u_int != MOUSE_BUTTON_1 as u_int {
                return 1 as ::core::ffi::c_int;
            }
            return 0 as ::core::ffi::c_int;
        }
        if (*m).x < (*md).px
            || (*m).x
                > (*md)
                    .px
                    .wrapping_add(4 as u_int)
                    .wrapping_add((*menu).width)
            || (*m).y < (*md).py.wrapping_add(1 as u_int)
            || (*m).y
                > (*md)
                    .py
                    .wrapping_add(1 as u_int)
                    .wrapping_add(n as u_int)
                    .wrapping_sub(1 as u_int)
        {
            if !(*md).flags & MENU_STAYOPEN != 0 {
                if move_0 == 0 && (*m).b & MOUSE_MASK_BUTTONS as u_int == 3 as u_int {
                    return 1 as ::core::ffi::c_int;
                }
            } else if !((*m).b & MOUSE_MASK_BUTTONS as u_int == 3 as u_int)
                && !((*m).b & MOUSE_MASK_BUTTONS as u_int == MOUSE_WHEEL_UP as u_int
                    || (*m).b & MOUSE_MASK_BUTTONS as u_int == MOUSE_WHEEL_DOWN as u_int)
                && (*m).b & MOUSE_MASK_DRAG as u_int == 0
            {
                return 1 as ::core::ffi::c_int;
            }
            if (*md).choice != -(1 as ::core::ffi::c_int) {
                (*md).choice = -(1 as ::core::ffi::c_int);
                server_redraw_window_menu((*md).w);
            }
            return 0 as ::core::ffi::c_int;
        }
        if !(*md).flags & MENU_STAYOPEN != 0 {
            if move_0 == 0 && (*m).b & MOUSE_MASK_BUTTONS as u_int == 3 as u_int {
                current_block = 4062906366992634423;
            } else {
                current_block = 11194104282611034094;
            }
        } else if !((*m).b & MOUSE_MASK_BUTTONS as u_int == MOUSE_WHEEL_UP as u_int
            || (*m).b & MOUSE_MASK_BUTTONS as u_int == MOUSE_WHEEL_DOWN as u_int)
            && (*m).b & MOUSE_MASK_DRAG as u_int == 0
        {
            current_block = 4062906366992634423;
        } else {
            current_block = 11194104282611034094;
        }
        match current_block {
            4062906366992634423 => {}
            _ => {
                (*md).choice =
                    (*m).y.wrapping_sub((*md).py.wrapping_add(1 as u_int)) as ::core::ffi::c_int;
                if (*md).choice != old {
                    server_redraw_window_menu((*md).w);
                }
                return 0 as ::core::ffi::c_int;
            }
        }
    } else {
        i = 0 as u_int;
        loop {
            if !(i < n as u_int) {
                current_block = 14434620278749266018;
                break;
            }
            name = (*(*menu).items.offset(i as isize)).name;
            if !(name.is_null() || *name as ::core::ffi::c_int == '-' as i32) {
                key = ((*event).key as ::core::ffi::c_ulonglong & !KEYC_MASK_FLAGS) as key_code;
                if key
                    == (*(*menu).items.offset(i as isize)).key as ::core::ffi::c_ulonglong
                        & !KEYC_MASK_FLAGS
                {
                    (*md).choice = i as ::core::ffi::c_int;
                    current_block = 4062906366992634423;
                    break;
                }
            }
            i = i.wrapping_add(1);
        }
        match current_block {
            4062906366992634423 => {}
            _ => match (*event).key as ::core::ffi::c_ulonglong & !KEYC_MASK_FLAGS {
                8589934618 | 8589934619 | 107 => {
                    current_block = 18228927260028731949;
                    match current_block {
                        10426959295196933295 => {
                            (*md).choice = 0 as ::core::ffi::c_int;
                            name = (*(*menu).items.offset((*md).choice as isize)).name;
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != n - 1 as ::core::ffi::c_int
                            {
                                (*md).choice += 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        5459197107747055838 => {
                            if (*md).choice > n - 6 as ::core::ffi::c_int {
                                (*md).choice = n - 1 as ::core::ffi::c_int;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice += 1;
                                    name = (*(*menu).items.offset((*md).choice as isize)).name;
                                    if (*md).choice != n - 1 as ::core::ffi::c_int
                                        && (!name.is_null()
                                            && *name as ::core::ffi::c_int != '-' as i32)
                                    {
                                        i = i.wrapping_sub(1);
                                    } else if (*md).choice == n - 1 as ::core::ffi::c_int {
                                        break;
                                    }
                                }
                            }
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        11150558847591123549 => {
                            if (*md).choice < 6 as ::core::ffi::c_int {
                                (*md).choice = 0 as ::core::ffi::c_int;
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice -= 1;
                                    name = (*(*menu).items.offset((*md).choice as isize)).name;
                                    if (*md).choice != 0 as ::core::ffi::c_int
                                        && (!name.is_null()
                                            && *name as ::core::ffi::c_int != '-' as i32)
                                    {
                                        i = i.wrapping_sub(1);
                                    } else if (*md).choice == 0 as ::core::ffi::c_int {
                                        break;
                                    }
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        6621853080098874574 => {
                            if !(*md).flags & MENU_TAB != 0 {
                                current_block = 12369290732426379360;
                            } else {
                                if (*md).choice == n - 1 as ::core::ffi::c_int {
                                    return 1 as ::core::ffi::c_int;
                                }
                                current_block = 17659224811226724223;
                            }
                        }
                        5908772614365292188 => {
                            if !(*md).flags & MENU_TAB != 0 {
                                current_block = 12369290732426379360;
                            } else {
                                return 1 as ::core::ffi::c_int;
                            }
                        }
                        18228927260028731949 => {
                            if old == -(1 as ::core::ffi::c_int) {
                                old = 0 as ::core::ffi::c_int;
                            }
                            loop {
                                if (*md).choice == -(1 as ::core::ffi::c_int)
                                    || (*md).choice == 0 as ::core::ffi::c_int
                                {
                                    (*md).choice = n - 1 as ::core::ffi::c_int;
                                } else {
                                    (*md).choice -= 1;
                                }
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                                if !((name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                    && (*md).choice != old)
                                {
                                    break;
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            return 0 as ::core::ffi::c_int;
                        }
                        4678245943260944876 => {
                            (*md).choice = n - 1 as ::core::ffi::c_int;
                            name = (*(*menu).items.offset((*md).choice as isize)).name;
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        16418760376262495662 => return 1 as ::core::ffi::c_int,
                        _ => {}
                    }
                    match current_block {
                        12369290732426379360 => return 0 as ::core::ffi::c_int,
                        _ => {
                            if old == -(1 as ::core::ffi::c_int) {
                                old = 0 as ::core::ffi::c_int;
                            }
                            loop {
                                if (*md).choice == -(1 as ::core::ffi::c_int)
                                    || (*md).choice == n - 1 as ::core::ffi::c_int
                                {
                                    (*md).choice = 0 as ::core::ffi::c_int;
                                } else {
                                    (*md).choice += 1;
                                }
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                                if !((name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                    && (*md).choice != old)
                                {
                                    break;
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            return 0 as ::core::ffi::c_int;
                        }
                    }
                }
                8589934599 => {
                    current_block = 5908772614365292188;
                    match current_block {
                        10426959295196933295 => {
                            (*md).choice = 0 as ::core::ffi::c_int;
                            name = (*(*menu).items.offset((*md).choice as isize)).name;
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != n - 1 as ::core::ffi::c_int
                            {
                                (*md).choice += 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        5459197107747055838 => {
                            if (*md).choice > n - 6 as ::core::ffi::c_int {
                                (*md).choice = n - 1 as ::core::ffi::c_int;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice += 1;
                                    name = (*(*menu).items.offset((*md).choice as isize)).name;
                                    if (*md).choice != n - 1 as ::core::ffi::c_int
                                        && (!name.is_null()
                                            && *name as ::core::ffi::c_int != '-' as i32)
                                    {
                                        i = i.wrapping_sub(1);
                                    } else if (*md).choice == n - 1 as ::core::ffi::c_int {
                                        break;
                                    }
                                }
                            }
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        11150558847591123549 => {
                            if (*md).choice < 6 as ::core::ffi::c_int {
                                (*md).choice = 0 as ::core::ffi::c_int;
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice -= 1;
                                    name = (*(*menu).items.offset((*md).choice as isize)).name;
                                    if (*md).choice != 0 as ::core::ffi::c_int
                                        && (!name.is_null()
                                            && *name as ::core::ffi::c_int != '-' as i32)
                                    {
                                        i = i.wrapping_sub(1);
                                    } else if (*md).choice == 0 as ::core::ffi::c_int {
                                        break;
                                    }
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        6621853080098874574 => {
                            if !(*md).flags & MENU_TAB != 0 {
                                current_block = 12369290732426379360;
                            } else {
                                if (*md).choice == n - 1 as ::core::ffi::c_int {
                                    return 1 as ::core::ffi::c_int;
                                }
                                current_block = 17659224811226724223;
                            }
                        }
                        5908772614365292188 => {
                            if !(*md).flags & MENU_TAB != 0 {
                                current_block = 12369290732426379360;
                            } else {
                                return 1 as ::core::ffi::c_int;
                            }
                        }
                        18228927260028731949 => {
                            if old == -(1 as ::core::ffi::c_int) {
                                old = 0 as ::core::ffi::c_int;
                            }
                            loop {
                                if (*md).choice == -(1 as ::core::ffi::c_int)
                                    || (*md).choice == 0 as ::core::ffi::c_int
                                {
                                    (*md).choice = n - 1 as ::core::ffi::c_int;
                                } else {
                                    (*md).choice -= 1;
                                }
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                                if !((name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                    && (*md).choice != old)
                                {
                                    break;
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            return 0 as ::core::ffi::c_int;
                        }
                        4678245943260944876 => {
                            (*md).choice = n - 1 as ::core::ffi::c_int;
                            name = (*(*menu).items.offset((*md).choice as isize)).name;
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        16418760376262495662 => return 1 as ::core::ffi::c_int,
                        _ => {}
                    }
                    match current_block {
                        12369290732426379360 => return 0 as ::core::ffi::c_int,
                        _ => {
                            if old == -(1 as ::core::ffi::c_int) {
                                old = 0 as ::core::ffi::c_int;
                            }
                            loop {
                                if (*md).choice == -(1 as ::core::ffi::c_int)
                                    || (*md).choice == n - 1 as ::core::ffi::c_int
                                {
                                    (*md).choice = 0 as ::core::ffi::c_int;
                                } else {
                                    (*md).choice += 1;
                                }
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                                if !((name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                    && (*md).choice != old)
                                {
                                    break;
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            return 0 as ::core::ffi::c_int;
                        }
                    }
                }
                9 => {
                    current_block = 6621853080098874574;
                    match current_block {
                        10426959295196933295 => {
                            (*md).choice = 0 as ::core::ffi::c_int;
                            name = (*(*menu).items.offset((*md).choice as isize)).name;
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != n - 1 as ::core::ffi::c_int
                            {
                                (*md).choice += 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        5459197107747055838 => {
                            if (*md).choice > n - 6 as ::core::ffi::c_int {
                                (*md).choice = n - 1 as ::core::ffi::c_int;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice += 1;
                                    name = (*(*menu).items.offset((*md).choice as isize)).name;
                                    if (*md).choice != n - 1 as ::core::ffi::c_int
                                        && (!name.is_null()
                                            && *name as ::core::ffi::c_int != '-' as i32)
                                    {
                                        i = i.wrapping_sub(1);
                                    } else if (*md).choice == n - 1 as ::core::ffi::c_int {
                                        break;
                                    }
                                }
                            }
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        11150558847591123549 => {
                            if (*md).choice < 6 as ::core::ffi::c_int {
                                (*md).choice = 0 as ::core::ffi::c_int;
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice -= 1;
                                    name = (*(*menu).items.offset((*md).choice as isize)).name;
                                    if (*md).choice != 0 as ::core::ffi::c_int
                                        && (!name.is_null()
                                            && *name as ::core::ffi::c_int != '-' as i32)
                                    {
                                        i = i.wrapping_sub(1);
                                    } else if (*md).choice == 0 as ::core::ffi::c_int {
                                        break;
                                    }
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        6621853080098874574 => {
                            if !(*md).flags & MENU_TAB != 0 {
                                current_block = 12369290732426379360;
                            } else {
                                if (*md).choice == n - 1 as ::core::ffi::c_int {
                                    return 1 as ::core::ffi::c_int;
                                }
                                current_block = 17659224811226724223;
                            }
                        }
                        5908772614365292188 => {
                            if !(*md).flags & MENU_TAB != 0 {
                                current_block = 12369290732426379360;
                            } else {
                                return 1 as ::core::ffi::c_int;
                            }
                        }
                        18228927260028731949 => {
                            if old == -(1 as ::core::ffi::c_int) {
                                old = 0 as ::core::ffi::c_int;
                            }
                            loop {
                                if (*md).choice == -(1 as ::core::ffi::c_int)
                                    || (*md).choice == 0 as ::core::ffi::c_int
                                {
                                    (*md).choice = n - 1 as ::core::ffi::c_int;
                                } else {
                                    (*md).choice -= 1;
                                }
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                                if !((name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                    && (*md).choice != old)
                                {
                                    break;
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            return 0 as ::core::ffi::c_int;
                        }
                        4678245943260944876 => {
                            (*md).choice = n - 1 as ::core::ffi::c_int;
                            name = (*(*menu).items.offset((*md).choice as isize)).name;
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        16418760376262495662 => return 1 as ::core::ffi::c_int,
                        _ => {}
                    }
                    match current_block {
                        12369290732426379360 => return 0 as ::core::ffi::c_int,
                        _ => {
                            if old == -(1 as ::core::ffi::c_int) {
                                old = 0 as ::core::ffi::c_int;
                            }
                            loop {
                                if (*md).choice == -(1 as ::core::ffi::c_int)
                                    || (*md).choice == n - 1 as ::core::ffi::c_int
                                {
                                    (*md).choice = 0 as ::core::ffi::c_int;
                                } else {
                                    (*md).choice += 1;
                                }
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                                if !((name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                    && (*md).choice != old)
                                {
                                    break;
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            return 0 as ::core::ffi::c_int;
                        }
                    }
                }
                8589934620 | 106 => {
                    current_block = 17659224811226724223;
                    match current_block {
                        10426959295196933295 => {
                            (*md).choice = 0 as ::core::ffi::c_int;
                            name = (*(*menu).items.offset((*md).choice as isize)).name;
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != n - 1 as ::core::ffi::c_int
                            {
                                (*md).choice += 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        5459197107747055838 => {
                            if (*md).choice > n - 6 as ::core::ffi::c_int {
                                (*md).choice = n - 1 as ::core::ffi::c_int;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice += 1;
                                    name = (*(*menu).items.offset((*md).choice as isize)).name;
                                    if (*md).choice != n - 1 as ::core::ffi::c_int
                                        && (!name.is_null()
                                            && *name as ::core::ffi::c_int != '-' as i32)
                                    {
                                        i = i.wrapping_sub(1);
                                    } else if (*md).choice == n - 1 as ::core::ffi::c_int {
                                        break;
                                    }
                                }
                            }
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        11150558847591123549 => {
                            if (*md).choice < 6 as ::core::ffi::c_int {
                                (*md).choice = 0 as ::core::ffi::c_int;
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice -= 1;
                                    name = (*(*menu).items.offset((*md).choice as isize)).name;
                                    if (*md).choice != 0 as ::core::ffi::c_int
                                        && (!name.is_null()
                                            && *name as ::core::ffi::c_int != '-' as i32)
                                    {
                                        i = i.wrapping_sub(1);
                                    } else if (*md).choice == 0 as ::core::ffi::c_int {
                                        break;
                                    }
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        6621853080098874574 => {
                            if !(*md).flags & MENU_TAB != 0 {
                                current_block = 12369290732426379360;
                            } else {
                                if (*md).choice == n - 1 as ::core::ffi::c_int {
                                    return 1 as ::core::ffi::c_int;
                                }
                                current_block = 17659224811226724223;
                            }
                        }
                        5908772614365292188 => {
                            if !(*md).flags & MENU_TAB != 0 {
                                current_block = 12369290732426379360;
                            } else {
                                return 1 as ::core::ffi::c_int;
                            }
                        }
                        18228927260028731949 => {
                            if old == -(1 as ::core::ffi::c_int) {
                                old = 0 as ::core::ffi::c_int;
                            }
                            loop {
                                if (*md).choice == -(1 as ::core::ffi::c_int)
                                    || (*md).choice == 0 as ::core::ffi::c_int
                                {
                                    (*md).choice = n - 1 as ::core::ffi::c_int;
                                } else {
                                    (*md).choice -= 1;
                                }
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                                if !((name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                    && (*md).choice != old)
                                {
                                    break;
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            return 0 as ::core::ffi::c_int;
                        }
                        4678245943260944876 => {
                            (*md).choice = n - 1 as ::core::ffi::c_int;
                            name = (*(*menu).items.offset((*md).choice as isize)).name;
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        16418760376262495662 => return 1 as ::core::ffi::c_int,
                        _ => {}
                    }
                    match current_block {
                        12369290732426379360 => return 0 as ::core::ffi::c_int,
                        _ => {
                            if old == -(1 as ::core::ffi::c_int) {
                                old = 0 as ::core::ffi::c_int;
                            }
                            loop {
                                if (*md).choice == -(1 as ::core::ffi::c_int)
                                    || (*md).choice == n - 1 as ::core::ffi::c_int
                                {
                                    (*md).choice = 0 as ::core::ffi::c_int;
                                } else {
                                    (*md).choice += 1;
                                }
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                                if !((name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                    && (*md).choice != old)
                                {
                                    break;
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            return 0 as ::core::ffi::c_int;
                        }
                    }
                }
                8589934617 | 35184372088930 => {
                    current_block = 11150558847591123549;
                    match current_block {
                        10426959295196933295 => {
                            (*md).choice = 0 as ::core::ffi::c_int;
                            name = (*(*menu).items.offset((*md).choice as isize)).name;
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != n - 1 as ::core::ffi::c_int
                            {
                                (*md).choice += 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        5459197107747055838 => {
                            if (*md).choice > n - 6 as ::core::ffi::c_int {
                                (*md).choice = n - 1 as ::core::ffi::c_int;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice += 1;
                                    name = (*(*menu).items.offset((*md).choice as isize)).name;
                                    if (*md).choice != n - 1 as ::core::ffi::c_int
                                        && (!name.is_null()
                                            && *name as ::core::ffi::c_int != '-' as i32)
                                    {
                                        i = i.wrapping_sub(1);
                                    } else if (*md).choice == n - 1 as ::core::ffi::c_int {
                                        break;
                                    }
                                }
                            }
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        11150558847591123549 => {
                            if (*md).choice < 6 as ::core::ffi::c_int {
                                (*md).choice = 0 as ::core::ffi::c_int;
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice -= 1;
                                    name = (*(*menu).items.offset((*md).choice as isize)).name;
                                    if (*md).choice != 0 as ::core::ffi::c_int
                                        && (!name.is_null()
                                            && *name as ::core::ffi::c_int != '-' as i32)
                                    {
                                        i = i.wrapping_sub(1);
                                    } else if (*md).choice == 0 as ::core::ffi::c_int {
                                        break;
                                    }
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        6621853080098874574 => {
                            if !(*md).flags & MENU_TAB != 0 {
                                current_block = 12369290732426379360;
                            } else {
                                if (*md).choice == n - 1 as ::core::ffi::c_int {
                                    return 1 as ::core::ffi::c_int;
                                }
                                current_block = 17659224811226724223;
                            }
                        }
                        5908772614365292188 => {
                            if !(*md).flags & MENU_TAB != 0 {
                                current_block = 12369290732426379360;
                            } else {
                                return 1 as ::core::ffi::c_int;
                            }
                        }
                        18228927260028731949 => {
                            if old == -(1 as ::core::ffi::c_int) {
                                old = 0 as ::core::ffi::c_int;
                            }
                            loop {
                                if (*md).choice == -(1 as ::core::ffi::c_int)
                                    || (*md).choice == 0 as ::core::ffi::c_int
                                {
                                    (*md).choice = n - 1 as ::core::ffi::c_int;
                                } else {
                                    (*md).choice -= 1;
                                }
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                                if !((name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                    && (*md).choice != old)
                                {
                                    break;
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            return 0 as ::core::ffi::c_int;
                        }
                        4678245943260944876 => {
                            (*md).choice = n - 1 as ::core::ffi::c_int;
                            name = (*(*menu).items.offset((*md).choice as isize)).name;
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        16418760376262495662 => return 1 as ::core::ffi::c_int,
                        _ => {}
                    }
                    match current_block {
                        12369290732426379360 => return 0 as ::core::ffi::c_int,
                        _ => {
                            if old == -(1 as ::core::ffi::c_int) {
                                old = 0 as ::core::ffi::c_int;
                            }
                            loop {
                                if (*md).choice == -(1 as ::core::ffi::c_int)
                                    || (*md).choice == n - 1 as ::core::ffi::c_int
                                {
                                    (*md).choice = 0 as ::core::ffi::c_int;
                                } else {
                                    (*md).choice += 1;
                                }
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                                if !((name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                    && (*md).choice != old)
                                {
                                    break;
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            return 0 as ::core::ffi::c_int;
                        }
                    }
                }
                8589934616 => {
                    current_block = 5459197107747055838;
                    match current_block {
                        10426959295196933295 => {
                            (*md).choice = 0 as ::core::ffi::c_int;
                            name = (*(*menu).items.offset((*md).choice as isize)).name;
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != n - 1 as ::core::ffi::c_int
                            {
                                (*md).choice += 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        5459197107747055838 => {
                            if (*md).choice > n - 6 as ::core::ffi::c_int {
                                (*md).choice = n - 1 as ::core::ffi::c_int;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice += 1;
                                    name = (*(*menu).items.offset((*md).choice as isize)).name;
                                    if (*md).choice != n - 1 as ::core::ffi::c_int
                                        && (!name.is_null()
                                            && *name as ::core::ffi::c_int != '-' as i32)
                                    {
                                        i = i.wrapping_sub(1);
                                    } else if (*md).choice == n - 1 as ::core::ffi::c_int {
                                        break;
                                    }
                                }
                            }
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        11150558847591123549 => {
                            if (*md).choice < 6 as ::core::ffi::c_int {
                                (*md).choice = 0 as ::core::ffi::c_int;
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice -= 1;
                                    name = (*(*menu).items.offset((*md).choice as isize)).name;
                                    if (*md).choice != 0 as ::core::ffi::c_int
                                        && (!name.is_null()
                                            && *name as ::core::ffi::c_int != '-' as i32)
                                    {
                                        i = i.wrapping_sub(1);
                                    } else if (*md).choice == 0 as ::core::ffi::c_int {
                                        break;
                                    }
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        6621853080098874574 => {
                            if !(*md).flags & MENU_TAB != 0 {
                                current_block = 12369290732426379360;
                            } else {
                                if (*md).choice == n - 1 as ::core::ffi::c_int {
                                    return 1 as ::core::ffi::c_int;
                                }
                                current_block = 17659224811226724223;
                            }
                        }
                        5908772614365292188 => {
                            if !(*md).flags & MENU_TAB != 0 {
                                current_block = 12369290732426379360;
                            } else {
                                return 1 as ::core::ffi::c_int;
                            }
                        }
                        18228927260028731949 => {
                            if old == -(1 as ::core::ffi::c_int) {
                                old = 0 as ::core::ffi::c_int;
                            }
                            loop {
                                if (*md).choice == -(1 as ::core::ffi::c_int)
                                    || (*md).choice == 0 as ::core::ffi::c_int
                                {
                                    (*md).choice = n - 1 as ::core::ffi::c_int;
                                } else {
                                    (*md).choice -= 1;
                                }
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                                if !((name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                    && (*md).choice != old)
                                {
                                    break;
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            return 0 as ::core::ffi::c_int;
                        }
                        4678245943260944876 => {
                            (*md).choice = n - 1 as ::core::ffi::c_int;
                            name = (*(*menu).items.offset((*md).choice as isize)).name;
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        16418760376262495662 => return 1 as ::core::ffi::c_int,
                        _ => {}
                    }
                    match current_block {
                        12369290732426379360 => return 0 as ::core::ffi::c_int,
                        _ => {
                            if old == -(1 as ::core::ffi::c_int) {
                                old = 0 as ::core::ffi::c_int;
                            }
                            loop {
                                if (*md).choice == -(1 as ::core::ffi::c_int)
                                    || (*md).choice == n - 1 as ::core::ffi::c_int
                                {
                                    (*md).choice = 0 as ::core::ffi::c_int;
                                } else {
                                    (*md).choice += 1;
                                }
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                                if !((name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                    && (*md).choice != old)
                                {
                                    break;
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            return 0 as ::core::ffi::c_int;
                        }
                    }
                }
                103 | 8589934614 => {
                    current_block = 10426959295196933295;
                    match current_block {
                        10426959295196933295 => {
                            (*md).choice = 0 as ::core::ffi::c_int;
                            name = (*(*menu).items.offset((*md).choice as isize)).name;
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != n - 1 as ::core::ffi::c_int
                            {
                                (*md).choice += 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        5459197107747055838 => {
                            if (*md).choice > n - 6 as ::core::ffi::c_int {
                                (*md).choice = n - 1 as ::core::ffi::c_int;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice += 1;
                                    name = (*(*menu).items.offset((*md).choice as isize)).name;
                                    if (*md).choice != n - 1 as ::core::ffi::c_int
                                        && (!name.is_null()
                                            && *name as ::core::ffi::c_int != '-' as i32)
                                    {
                                        i = i.wrapping_sub(1);
                                    } else if (*md).choice == n - 1 as ::core::ffi::c_int {
                                        break;
                                    }
                                }
                            }
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        11150558847591123549 => {
                            if (*md).choice < 6 as ::core::ffi::c_int {
                                (*md).choice = 0 as ::core::ffi::c_int;
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice -= 1;
                                    name = (*(*menu).items.offset((*md).choice as isize)).name;
                                    if (*md).choice != 0 as ::core::ffi::c_int
                                        && (!name.is_null()
                                            && *name as ::core::ffi::c_int != '-' as i32)
                                    {
                                        i = i.wrapping_sub(1);
                                    } else if (*md).choice == 0 as ::core::ffi::c_int {
                                        break;
                                    }
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        6621853080098874574 => {
                            if !(*md).flags & MENU_TAB != 0 {
                                current_block = 12369290732426379360;
                            } else {
                                if (*md).choice == n - 1 as ::core::ffi::c_int {
                                    return 1 as ::core::ffi::c_int;
                                }
                                current_block = 17659224811226724223;
                            }
                        }
                        5908772614365292188 => {
                            if !(*md).flags & MENU_TAB != 0 {
                                current_block = 12369290732426379360;
                            } else {
                                return 1 as ::core::ffi::c_int;
                            }
                        }
                        18228927260028731949 => {
                            if old == -(1 as ::core::ffi::c_int) {
                                old = 0 as ::core::ffi::c_int;
                            }
                            loop {
                                if (*md).choice == -(1 as ::core::ffi::c_int)
                                    || (*md).choice == 0 as ::core::ffi::c_int
                                {
                                    (*md).choice = n - 1 as ::core::ffi::c_int;
                                } else {
                                    (*md).choice -= 1;
                                }
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                                if !((name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                    && (*md).choice != old)
                                {
                                    break;
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            return 0 as ::core::ffi::c_int;
                        }
                        4678245943260944876 => {
                            (*md).choice = n - 1 as ::core::ffi::c_int;
                            name = (*(*menu).items.offset((*md).choice as isize)).name;
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        16418760376262495662 => return 1 as ::core::ffi::c_int,
                        _ => {}
                    }
                    match current_block {
                        12369290732426379360 => return 0 as ::core::ffi::c_int,
                        _ => {
                            if old == -(1 as ::core::ffi::c_int) {
                                old = 0 as ::core::ffi::c_int;
                            }
                            loop {
                                if (*md).choice == -(1 as ::core::ffi::c_int)
                                    || (*md).choice == n - 1 as ::core::ffi::c_int
                                {
                                    (*md).choice = 0 as ::core::ffi::c_int;
                                } else {
                                    (*md).choice += 1;
                                }
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                                if !((name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                    && (*md).choice != old)
                                {
                                    break;
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            return 0 as ::core::ffi::c_int;
                        }
                    }
                }
                71 | 8589934615 => {
                    current_block = 4678245943260944876;
                    match current_block {
                        10426959295196933295 => {
                            (*md).choice = 0 as ::core::ffi::c_int;
                            name = (*(*menu).items.offset((*md).choice as isize)).name;
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != n - 1 as ::core::ffi::c_int
                            {
                                (*md).choice += 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        5459197107747055838 => {
                            if (*md).choice > n - 6 as ::core::ffi::c_int {
                                (*md).choice = n - 1 as ::core::ffi::c_int;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice += 1;
                                    name = (*(*menu).items.offset((*md).choice as isize)).name;
                                    if (*md).choice != n - 1 as ::core::ffi::c_int
                                        && (!name.is_null()
                                            && *name as ::core::ffi::c_int != '-' as i32)
                                    {
                                        i = i.wrapping_sub(1);
                                    } else if (*md).choice == n - 1 as ::core::ffi::c_int {
                                        break;
                                    }
                                }
                            }
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        11150558847591123549 => {
                            if (*md).choice < 6 as ::core::ffi::c_int {
                                (*md).choice = 0 as ::core::ffi::c_int;
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice -= 1;
                                    name = (*(*menu).items.offset((*md).choice as isize)).name;
                                    if (*md).choice != 0 as ::core::ffi::c_int
                                        && (!name.is_null()
                                            && *name as ::core::ffi::c_int != '-' as i32)
                                    {
                                        i = i.wrapping_sub(1);
                                    } else if (*md).choice == 0 as ::core::ffi::c_int {
                                        break;
                                    }
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        6621853080098874574 => {
                            if !(*md).flags & MENU_TAB != 0 {
                                current_block = 12369290732426379360;
                            } else {
                                if (*md).choice == n - 1 as ::core::ffi::c_int {
                                    return 1 as ::core::ffi::c_int;
                                }
                                current_block = 17659224811226724223;
                            }
                        }
                        5908772614365292188 => {
                            if !(*md).flags & MENU_TAB != 0 {
                                current_block = 12369290732426379360;
                            } else {
                                return 1 as ::core::ffi::c_int;
                            }
                        }
                        18228927260028731949 => {
                            if old == -(1 as ::core::ffi::c_int) {
                                old = 0 as ::core::ffi::c_int;
                            }
                            loop {
                                if (*md).choice == -(1 as ::core::ffi::c_int)
                                    || (*md).choice == 0 as ::core::ffi::c_int
                                {
                                    (*md).choice = n - 1 as ::core::ffi::c_int;
                                } else {
                                    (*md).choice -= 1;
                                }
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                                if !((name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                    && (*md).choice != old)
                                {
                                    break;
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            return 0 as ::core::ffi::c_int;
                        }
                        4678245943260944876 => {
                            (*md).choice = n - 1 as ::core::ffi::c_int;
                            name = (*(*menu).items.offset((*md).choice as isize)).name;
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        16418760376262495662 => return 1 as ::core::ffi::c_int,
                        _ => {}
                    }
                    match current_block {
                        12369290732426379360 => return 0 as ::core::ffi::c_int,
                        _ => {
                            if old == -(1 as ::core::ffi::c_int) {
                                old = 0 as ::core::ffi::c_int;
                            }
                            loop {
                                if (*md).choice == -(1 as ::core::ffi::c_int)
                                    || (*md).choice == n - 1 as ::core::ffi::c_int
                                {
                                    (*md).choice = 0 as ::core::ffi::c_int;
                                } else {
                                    (*md).choice += 1;
                                }
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                                if !((name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                    && (*md).choice != old)
                                {
                                    break;
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            return 0 as ::core::ffi::c_int;
                        }
                    }
                }
                13 => {}
                27 | 35184372088923 | 35184372088931 | 35184372088935 | 113 => {
                    current_block = 16418760376262495662;
                    match current_block {
                        10426959295196933295 => {
                            (*md).choice = 0 as ::core::ffi::c_int;
                            name = (*(*menu).items.offset((*md).choice as isize)).name;
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != n - 1 as ::core::ffi::c_int
                            {
                                (*md).choice += 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        5459197107747055838 => {
                            if (*md).choice > n - 6 as ::core::ffi::c_int {
                                (*md).choice = n - 1 as ::core::ffi::c_int;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice += 1;
                                    name = (*(*menu).items.offset((*md).choice as isize)).name;
                                    if (*md).choice != n - 1 as ::core::ffi::c_int
                                        && (!name.is_null()
                                            && *name as ::core::ffi::c_int != '-' as i32)
                                    {
                                        i = i.wrapping_sub(1);
                                    } else if (*md).choice == n - 1 as ::core::ffi::c_int {
                                        break;
                                    }
                                }
                            }
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        11150558847591123549 => {
                            if (*md).choice < 6 as ::core::ffi::c_int {
                                (*md).choice = 0 as ::core::ffi::c_int;
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice -= 1;
                                    name = (*(*menu).items.offset((*md).choice as isize)).name;
                                    if (*md).choice != 0 as ::core::ffi::c_int
                                        && (!name.is_null()
                                            && *name as ::core::ffi::c_int != '-' as i32)
                                    {
                                        i = i.wrapping_sub(1);
                                    } else if (*md).choice == 0 as ::core::ffi::c_int {
                                        break;
                                    }
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        6621853080098874574 => {
                            if !(*md).flags & MENU_TAB != 0 {
                                current_block = 12369290732426379360;
                            } else {
                                if (*md).choice == n - 1 as ::core::ffi::c_int {
                                    return 1 as ::core::ffi::c_int;
                                }
                                current_block = 17659224811226724223;
                            }
                        }
                        5908772614365292188 => {
                            if !(*md).flags & MENU_TAB != 0 {
                                current_block = 12369290732426379360;
                            } else {
                                return 1 as ::core::ffi::c_int;
                            }
                        }
                        18228927260028731949 => {
                            if old == -(1 as ::core::ffi::c_int) {
                                old = 0 as ::core::ffi::c_int;
                            }
                            loop {
                                if (*md).choice == -(1 as ::core::ffi::c_int)
                                    || (*md).choice == 0 as ::core::ffi::c_int
                                {
                                    (*md).choice = n - 1 as ::core::ffi::c_int;
                                } else {
                                    (*md).choice -= 1;
                                }
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                                if !((name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                    && (*md).choice != old)
                                {
                                    break;
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            return 0 as ::core::ffi::c_int;
                        }
                        4678245943260944876 => {
                            (*md).choice = n - 1 as ::core::ffi::c_int;
                            name = (*(*menu).items.offset((*md).choice as isize)).name;
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        16418760376262495662 => return 1 as ::core::ffi::c_int,
                        _ => {}
                    }
                    match current_block {
                        12369290732426379360 => return 0 as ::core::ffi::c_int,
                        _ => {
                            if old == -(1 as ::core::ffi::c_int) {
                                old = 0 as ::core::ffi::c_int;
                            }
                            loop {
                                if (*md).choice == -(1 as ::core::ffi::c_int)
                                    || (*md).choice == n - 1 as ::core::ffi::c_int
                                {
                                    (*md).choice = 0 as ::core::ffi::c_int;
                                } else {
                                    (*md).choice += 1;
                                }
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                                if !((name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                    && (*md).choice != old)
                                {
                                    break;
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            return 0 as ::core::ffi::c_int;
                        }
                    }
                }
                35184372088934 | _ => {
                    current_block = 12369290732426379360;
                    match current_block {
                        10426959295196933295 => {
                            (*md).choice = 0 as ::core::ffi::c_int;
                            name = (*(*menu).items.offset((*md).choice as isize)).name;
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != n - 1 as ::core::ffi::c_int
                            {
                                (*md).choice += 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        5459197107747055838 => {
                            if (*md).choice > n - 6 as ::core::ffi::c_int {
                                (*md).choice = n - 1 as ::core::ffi::c_int;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice += 1;
                                    name = (*(*menu).items.offset((*md).choice as isize)).name;
                                    if (*md).choice != n - 1 as ::core::ffi::c_int
                                        && (!name.is_null()
                                            && *name as ::core::ffi::c_int != '-' as i32)
                                    {
                                        i = i.wrapping_sub(1);
                                    } else if (*md).choice == n - 1 as ::core::ffi::c_int {
                                        break;
                                    }
                                }
                            }
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        11150558847591123549 => {
                            if (*md).choice < 6 as ::core::ffi::c_int {
                                (*md).choice = 0 as ::core::ffi::c_int;
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice -= 1;
                                    name = (*(*menu).items.offset((*md).choice as isize)).name;
                                    if (*md).choice != 0 as ::core::ffi::c_int
                                        && (!name.is_null()
                                            && *name as ::core::ffi::c_int != '-' as i32)
                                    {
                                        i = i.wrapping_sub(1);
                                    } else if (*md).choice == 0 as ::core::ffi::c_int {
                                        break;
                                    }
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        6621853080098874574 => {
                            if !(*md).flags & MENU_TAB != 0 {
                                current_block = 12369290732426379360;
                            } else {
                                if (*md).choice == n - 1 as ::core::ffi::c_int {
                                    return 1 as ::core::ffi::c_int;
                                }
                                current_block = 17659224811226724223;
                            }
                        }
                        5908772614365292188 => {
                            if !(*md).flags & MENU_TAB != 0 {
                                current_block = 12369290732426379360;
                            } else {
                                return 1 as ::core::ffi::c_int;
                            }
                        }
                        18228927260028731949 => {
                            if old == -(1 as ::core::ffi::c_int) {
                                old = 0 as ::core::ffi::c_int;
                            }
                            loop {
                                if (*md).choice == -(1 as ::core::ffi::c_int)
                                    || (*md).choice == 0 as ::core::ffi::c_int
                                {
                                    (*md).choice = n - 1 as ::core::ffi::c_int;
                                } else {
                                    (*md).choice -= 1;
                                }
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                                if !((name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                    && (*md).choice != old)
                                {
                                    break;
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            return 0 as ::core::ffi::c_int;
                        }
                        4678245943260944876 => {
                            (*md).choice = n - 1 as ::core::ffi::c_int;
                            name = (*(*menu).items.offset((*md).choice as isize)).name;
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        16418760376262495662 => return 1 as ::core::ffi::c_int,
                        _ => {}
                    }
                    match current_block {
                        12369290732426379360 => return 0 as ::core::ffi::c_int,
                        _ => {
                            if old == -(1 as ::core::ffi::c_int) {
                                old = 0 as ::core::ffi::c_int;
                            }
                            loop {
                                if (*md).choice == -(1 as ::core::ffi::c_int)
                                    || (*md).choice == n - 1 as ::core::ffi::c_int
                                {
                                    (*md).choice = 0 as ::core::ffi::c_int;
                                } else {
                                    (*md).choice += 1;
                                }
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                                if !((name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                    && (*md).choice != old)
                                {
                                    break;
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            return 0 as ::core::ffi::c_int;
                        }
                    }
                }
            },
        }
    }
    if (*md).choice == -(1 as ::core::ffi::c_int) {
        return 1 as ::core::ffi::c_int;
    }
    item = (*menu).items.offset((*md).choice as isize) as *mut menu_item;
    if (*item).name.is_null() || *(*item).name as ::core::ffi::c_int == '-' as i32 {
        if (*md).flags & MENU_STAYOPEN != 0 {
            return 0 as ::core::ffi::c_int;
        }
        return 1 as ::core::ffi::c_int;
    }
    if (*md).cb.is_some() {
        (*md).cb.expect("non-null function pointer")(
            (*md).menu,
            (*md).choice as u_int,
            (*item).key,
            (*md).data,
        );
        (*md).cb = None;
        return 1 as ::core::ffi::c_int;
    }
    if (*md).key != KEYC_NONE as ::core::ffi::c_ulong as key_code {
        memset(
            &raw mut saved_event as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<key_event>() as size_t,
        );
        saved_event.key = (*md).key;
        memcpy(
            &raw mut saved_event.m as *mut ::core::ffi::c_void,
            &raw mut (*md).m as *const ::core::ffi::c_void,
            ::core::mem::size_of::<mouse_event>() as size_t,
        );
        event = &raw mut saved_event;
    } else {
        event = ::core::ptr::null_mut::<key_event>();
    }
    state = cmdq_new_state(&raw mut (*md).fs, event, 0 as ::core::ffi::c_int);
    status = cmd_parse_and_append(
        (*item).command,
        ::core::ptr::null_mut::<cmd_parse_input>(),
        c,
        state,
        &raw mut error,
    );
    if status as ::core::ffi::c_uint == CMD_PARSE_ERROR as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        cmdq_append(c, cmdq_get_error(error));
        free(error as *mut ::core::ffi::c_void);
    }
    cmdq_free_state(state);
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn menu_resize(mut md: *mut menu_data, mut w: *mut window) {
    let mut nx: u_int = 0;
    let mut ny: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    if md.is_null() {
        return;
    }
    nx = (*md).px;
    ny = (*md).py;
    sx = (*(*md).menu).width.wrapping_add(4 as u_int);
    sy = (*(*md).menu).count.wrapping_add(2 as u_int);
    if nx.wrapping_add(sx) > (*w).sx {
        if (*w).sx <= sx {
            nx = 0 as u_int;
        } else {
            nx = (*w).sx.wrapping_sub(sx);
        }
    }
    if ny.wrapping_add(sy) > (*w).sy {
        if (*w).sy <= sy {
            ny = 0 as u_int;
        } else {
            ny = (*w).sy.wrapping_sub(sy);
        }
    }
    (*md).px = nx;
    (*md).py = ny;
}
#[no_mangle]
pub unsafe extern "C" fn menu_display(
    mut menu: *mut menu,
    mut flags: ::core::ffi::c_int,
    mut starting_choice: ::core::ffi::c_int,
    mut item: *mut cmdq_item,
    mut px: u_int,
    mut py: u_int,
    mut c: *mut client,
    mut lines: box_lines,
    mut style: *const ::core::ffi::c_char,
    mut selected_style: *const ::core::ffi::c_char,
    mut border_style: *const ::core::ffi::c_char,
    mut fs: *mut cmd_find_state,
    mut cb: menu_choice_cb,
    mut data: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut md: *mut menu_data = ::core::ptr::null_mut::<menu_data>();
    let mut event: *mut key_event = ::core::ptr::null_mut::<key_event>();
    let mut choice: ::core::ffi::c_int = 0;
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut o: *mut options = ::core::ptr::null_mut::<options>();
    if fs.is_null() {
        w = (*(*(*c).session).curw).window;
    } else {
        w = (*fs).w;
    }
    o = (*w).options;
    sx = (*menu).width.wrapping_add(4 as u_int);
    sy = (*menu).count.wrapping_add(2 as u_int);
    if sx >= (*w).sx {
        px = 0 as u_int;
    } else if px.wrapping_add(sx) > (*w).sx {
        px = (*w).sx.wrapping_sub(sx);
    }
    if sy >= (*w).sy {
        py = 0 as u_int;
    } else if py.wrapping_add(sy) > (*w).sy {
        py = (*w).sy.wrapping_sub(sy);
    }
    (*w).menu_last_px = px;
    (*w).menu_last_py = py;
    if lines as ::core::ffi::c_int == BOX_LINES_DEFAULT as ::core::ffi::c_int {
        lines = options_get_number(
            o,
            b"menu-border-lines\0" as *const u8 as *const ::core::ffi::c_char,
        ) as box_lines;
    }
    md = xcalloc(1 as size_t, ::core::mem::size_of::<menu_data>() as size_t) as *mut menu_data;
    (*md).w = w;
    (*md).flags = flags;
    (*md).border_lines = lines;
    (*md).key = KEYC_NONE as ::core::ffi::c_ulong as key_code;
    if !item.is_null() {
        event = cmdq_get_event(item);
        (*md).key = (*event).key;
        memcpy(
            &raw mut (*md).m as *mut ::core::ffi::c_void,
            &raw mut (*event).m as *const ::core::ffi::c_void,
            ::core::mem::size_of::<mouse_event>() as size_t,
        );
    }
    if !style.is_null() {
        (*md).style = xstrdup(style);
    }
    if !selected_style.is_null() {
        (*md).selected_style = xstrdup(selected_style);
    }
    if !border_style.is_null() {
        (*md).border_style = xstrdup(border_style);
    }
    if !fs.is_null() {
        cmd_find_copy_state(&raw mut (*md).fs, fs);
    } else if cmd_find_from_window(&raw mut (*md).fs, w, 0 as ::core::ffi::c_int)
        != 0 as ::core::ffi::c_int
    {
        cmd_find_clear_state(&raw mut (*md).fs, 0 as ::core::ffi::c_int);
    }
    screen_init(&raw mut (*md).s, sx, sy, 0 as u_int);
    if !(*md).flags & MENU_NOMOUSE != 0 {
        (*md).s.mode |= MODE_MOUSE_ALL | MODE_MOUSE_BUTTON;
    }
    (*md).s.mode &= !MODE_CURSOR;
    (*md).px = px;
    (*md).py = py;
    (*md).menu = menu;
    (*md).choice = -(1 as ::core::ffi::c_int);
    (*md).cb = cb;
    (*md).data = data;
    if (*md).flags & MENU_NOMOUSE != 0 {
        if starting_choice >= (*menu).count as ::core::ffi::c_int {
            starting_choice = (*menu).count.wrapping_sub(1 as u_int) as ::core::ffi::c_int;
            choice = starting_choice + 1 as ::core::ffi::c_int;
            loop {
                name = (*(*menu)
                    .items
                    .offset((choice - 1 as ::core::ffi::c_int) as isize))
                .name;
                if !name.is_null() && *name as ::core::ffi::c_int != '-' as i32 {
                    (*md).choice = choice - 1 as ::core::ffi::c_int;
                    break;
                } else {
                    choice -= 1;
                    if choice == 0 as ::core::ffi::c_int {
                        choice = (*menu).count as ::core::ffi::c_int;
                    }
                    if choice == starting_choice + 1 as ::core::ffi::c_int {
                        break;
                    }
                }
            }
        } else if starting_choice >= 0 as ::core::ffi::c_int {
            choice = starting_choice;
            loop {
                name = (*(*menu).items.offset(choice as isize)).name;
                if !name.is_null() && *name as ::core::ffi::c_int != '-' as i32 {
                    (*md).choice = choice;
                    break;
                } else {
                    choice += 1;
                    if choice == (*menu).count as ::core::ffi::c_int {
                        choice = 0 as ::core::ffi::c_int;
                    }
                    if choice == starting_choice {
                        break;
                    }
                }
            }
        }
    }
    menu_close((*md).w);
    (*(*md).w).menu = md;
    redraw_invalidate_scene((*md).w);
    window_update_focus((*md).w);
    server_redraw_window((*md).w);
    return 0 as ::core::ffi::c_int;
}
