use crate::src::cmd::find::{cmd_find_clear_state, cmd_find_copy_state, cmd_find_from_window};
use crate::src::cmd::parse::cmd_parse_and_append;
use crate::src::cmd::queue::{
    cmdq_append, cmdq_free_state, cmdq_get_error, cmdq_get_event, cmdq_new_state,
};
use crate::src::ffi::libc::{memcpy, strlen};
use crate::src::format::{
    format_create_defaults, format_free, format_single_cstring, format_single_from_state_cstring,
};
use crate::src::format_draw::{format_trim_right_bytes, format_width};
use crate::src::grid::grid_default_cell;
use crate::src::key_string::key_string_format;
use crate::src::options::options_get_number;
use crate::src::screen::{screen_free, screen_init};
use crate::src::screen_redraw::redraw_invalidate_scene;
use crate::src::screen_write::{
    screen_write_box, screen_write_clearscreen, screen_write_menu, screen_write_start,
    screen_write_stop,
};
use crate::src::server_fn::{server_redraw_window, server_redraw_window_menu};
use crate::src::shared::abi::*;
use crate::src::shared::client::client;
use crate::src::shared::command::cmd_parse_input;
use crate::src::shared::command::{cmd_find_state, cmdq_item, cmdq_state};
use crate::src::shared::format::format_tree;
use crate::src::shared::grid::*;
use crate::src::shared::key::key_event;
use crate::src::shared::key::*;
use crate::src::shared::layout::*;
use crate::src::shared::menu::{
    menu, menu_item, MenuRow, MenuSelection, MENU_NOMOUSE, MENU_STAYOPEN, MENU_TAB,
};
use crate::src::shared::menu::{menu_choice_cb, menu_data};
use crate::src::shared::mouse::{
    mouse_event, MOUSE_BUTTON_1, MOUSE_MASK_BUTTONS, MOUSE_MASK_DRAG, MOUSE_WHEEL_DOWN,
    MOUSE_WHEEL_UP,
};
use crate::src::shared::options::options;
use crate::src::shared::pane::window_pane;
use crate::src::shared::screen::{screen, MODE_CURSOR, MODE_MOUSE_ALL, MODE_MOUSE_BUTTON};
use crate::src::shared::screen_write::screen_write_ctx;
use crate::src::shared::session::session;
use crate::src::shared::style::*;
use crate::src::shared::window::{window, winlink};
use crate::src::style::{style_apply, style_parse, style_set};
use crate::src::window::window_update_focus;
use std::ffi::{CStr, CString};

impl menu {
    fn refresh_items(&mut self) {
        self.count = self.items.len().try_into().expect("too many menu items");
    }

    fn push_empty(&mut self) -> usize {
        let index = self.items.len();
        self.items.push(MenuRow::default());
        self.refresh_items();
        index
    }
}
pub unsafe fn menu_add_items(mut menu: *mut menu, mut items: *const menu_item, mut c: *mut client) {
    let mut qitem: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    let mut fs: *mut cmd_find_state = ::core::ptr::null_mut::<cmd_find_state>();
    let mut loop_0: *const menu_item = ::core::ptr::null::<menu_item>();
    loop_0 = items;
    while !(*loop_0).name.is_null() {
        menu_add_item(menu, loop_0, qitem, c, fs);
        loop_0 = loop_0.offset(1);
    }
}
pub unsafe fn menu_add_item(
    mut menu: *mut menu,
    mut item: *const menu_item,
    mut qitem: *mut cmdq_item,
    mut c: *mut client,
    mut fs: *mut cmd_find_state,
) {
    let index: usize;
    let mut key: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut key_owned: Option<std::ffi::CString> = None;
    let mut cmd: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut suffix: *const ::core::ffi::c_char = b"\0" as *const u8 as *const ::core::ffi::c_char;
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
            .as_mut_ptr()
            .offset((*menu).count.wrapping_sub(1 as u_int) as isize))
        .name_ptr()
        .is_null()
    {
        return;
    }
    index = (*(menu as *mut menu)).push_empty();
    if line != 0 {
        return;
    }
    let s = if !fs.is_null() {
        format_single_from_state_cstring(qitem, (*item).name, c, fs)
    } else {
        format_single_cstring(
            qitem,
            (*item).name,
            c,
            ::core::ptr::null_mut::<session>(),
            ::core::ptr::null_mut::<winlink>(),
            ::core::ptr::null_mut::<window_pane>(),
        )
    };
    if s.is_empty() {
        let owner = &mut *(menu as *mut menu);
        // The menu is still local during formatting, so this placeholder is
        // the last row if expansion suppresses it.
        owner.items.pop();
        owner.refresh_items();
        return;
    }
    max_width = (*c).tty.sx.wrapping_sub(4 as u_int);
    slen = strlen(s.as_ptr());
    if *s.as_ptr() as ::core::ffi::c_int != '-' as i32
        && (*item).key != KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code
        && (*item).key != KEYC_NONE as ::core::ffi::c_ulong as key_code
    {
        key_owned = Some(key_string_format((*item).key, false));
        key = key_owned.as_ref().unwrap().as_ptr();
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
    let mut name = format_trim_right_bytes(s.as_c_str(), max_width);
    name.extend_from_slice(CStr::from_ptr(suffix).to_bytes());
    if !key.is_null() {
        name.extend_from_slice(b"#[default] #[align=right](");
        name.extend_from_slice(CStr::from_ptr(key).to_bytes());
        name.push(b')');
    }
    {
        let owner = &mut *(menu as *mut menu);
        owner.items[index].name = Some(CString::new(name).expect("menu name contains no NUL"));
    }
    cmd = (*item).command;
    let command = if !cmd.is_null() {
        if !fs.is_null() {
            Some(format_single_from_state_cstring(qitem, cmd, c, fs))
        } else {
            Some(format_single_cstring(
                qitem,
                cmd,
                c,
                ::core::ptr::null_mut::<session>(),
                ::core::ptr::null_mut::<winlink>(),
                ::core::ptr::null_mut::<window_pane>(),
            ))
        }
    } else {
        None
    };
    {
        let owner = &mut *(menu as *mut menu);
        if let Some(command) = command {
            owner.items[index].command = Some(command);
        }
        owner.items[index].key = (*item).key;
    }
    let row_name = (*(*menu).items.as_mut_ptr().add(index)).name_ptr();
    width = format_width(row_name);
    if *row_name as ::core::ffi::c_int == '-' as i32 {
        width = width.wrapping_sub(1);
    }
    if width > (*menu).width {
        (*menu).width = width;
    }
}
pub unsafe fn menu_create(mut title: *const ::core::ffi::c_char) -> *mut menu {
    let title = CStr::from_ptr(title).to_owned();
    let width = format_width(title.as_ptr());
    let owner = Box::new(menu {
        title: title,
        items: Vec::new(),
        count: 0,
        width: width,
    });
    Box::into_raw(owner) as *mut menu
}
pub unsafe fn menu_free(mut menu: *mut menu) {
    if menu.is_null() {
        return;
    }
    drop(Box::from_raw(menu as *mut menu));
}
unsafe fn menu_reapply_styles(mut md: *mut menu_data) {
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
    if !(*md).style.is_none() {
        style_set(&raw mut sytmp, &raw const grid_default_cell);
        if style_parse(
            &raw mut sytmp,
            &raw mut (*md).style_gc,
            ((*md).style)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        ) == 0 as ::core::ffi::c_int
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
    if !(*md).selected_style.is_none() {
        style_set(&raw mut sytmp, &raw const grid_default_cell);
        if style_parse(
            &raw mut sytmp,
            &raw mut (*md).selected_style_gc,
            ((*md).selected_style)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
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
    if !(*md).border_style.is_none() {
        style_set(&raw mut sytmp, &raw const grid_default_cell);
        if style_parse(
            &raw mut sytmp,
            &raw mut (*md).border_style_gc,
            ((*md).border_style)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        ) == 0 as ::core::ffi::c_int
        {
            (*md).border_style_gc.fg = sytmp.gc.fg;
            (*md).border_style_gc.bg = sytmp.gc.bg;
        }
    }
    format_free(ft);
}
pub unsafe fn menu_update(mut md: *mut menu_data) {
    let mut s: *mut screen = &raw mut (*md).s;
    let mut menu: *mut menu = (*md).menu;
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: ::core::ptr::null_mut::<window_pane>(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        item: None,
        scrolled: 0,
        bg: 0,
    };
    menu_reapply_styles(md);
    screen_write_start(&mut ctx, s);
    screen_write_clearscreen(&raw mut ctx, 8 as u_int);
    if (*md).border_lines as ::core::ffi::c_int != BOX_LINES_NONE as ::core::ffi::c_int {
        screen_write_box(
            &mut ctx,
            (*menu).width.wrapping_add(4 as u_int),
            (*menu).count.wrapping_add(2 as u_int),
            (*md).border_lines,
            Some(&(*md).border_style_gc),
            Some((*menu).title.as_c_str()),
        );
    }
    screen_write_menu(
        &mut ctx,
        &*menu,
        (*md).choice,
        (*md).border_lines,
        &(*md).style_gc,
        Some(&(*md).border_style_gc),
        &(*md).selected_style_gc,
    );
    screen_write_stop(&mut ctx);
}
unsafe fn menu_free_data(mut md: *mut menu_data) {
    if !md.is_null() {
        if let Some(callback) = (*md).cb.take() {
            callback(MenuSelection::Cancelled);
        }
        screen_free(&mut (*md).s);
        menu_free((*md).menu);
        drop(Box::from_raw(md));
    }
}
pub unsafe fn menu_close(mut w: *mut window) {
    if !(*w).menu.is_null() {
        menu_free_data((*w).menu);
        (*w).menu = ::core::ptr::null_mut::<menu_data>();
        redraw_invalidate_scene(w);
        window_update_focus(w);
        server_redraw_window(w);
    }
}
pub unsafe fn menu_destroy(mut w: *mut window) {
    menu_free_data((*w).menu);
    (*w).menu = ::core::ptr::null_mut::<menu_data>();
}
pub unsafe fn menu_get_cursor(mut md: *mut menu_data, mut cx: *mut u_int, mut cy: *mut u_int) {
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
pub unsafe fn menu_screen(mut md: *mut menu_data) -> *mut screen {
    return &raw mut (*md).s;
}
pub unsafe fn menu_width(mut md: *mut menu_data) -> u_int {
    return (*(*md).menu).width.wrapping_add(4 as u_int);
}
pub unsafe fn menu_height(mut md: *mut menu_data) -> u_int {
    return (*(*md).menu).count.wrapping_add(2 as u_int);
}
pub unsafe fn menu_x(mut md: *mut menu_data) -> u_int {
    return (*md).px;
}
pub unsafe fn menu_y(mut md: *mut menu_data) -> u_int {
    return (*md).py;
}
pub unsafe fn menu_key(
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
    let mut item: *const MenuRow = ::core::ptr::null::<MenuRow>();
    let mut saved_event = key_event {
        client: ::core::ptr::null_mut(),
        key: 0,
        m: Default::default(),
        bytes: None,
    };
    let mut state: *mut cmdq_state = ::core::ptr::null_mut::<cmdq_state>();
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
            name = (*(*menu).items.as_mut_ptr().offset(i as isize)).name_ptr();
            if !(name.is_null() || *name as ::core::ffi::c_int == '-' as i32) {
                key = ((*event).key as ::core::ffi::c_ulonglong & !KEYC_MASK_FLAGS) as key_code;
                if key
                    == (*(*menu).items.as_mut_ptr().offset(i as isize)).key
                        as ::core::ffi::c_ulonglong
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
                            name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                .name_ptr();
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != n - 1 as ::core::ffi::c_int
                            {
                                (*md).choice += 1;
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        5459197107747055838 => {
                            if (*md).choice > n - 6 as ::core::ffi::c_int {
                                (*md).choice = n - 1 as ::core::ffi::c_int;
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice += 1;
                                    name =
                                        (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                            .name_ptr();
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
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
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
                                    name =
                                        (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                            .name_ptr();
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
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
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
                            name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                .name_ptr();
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
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
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
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
                            name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                .name_ptr();
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != n - 1 as ::core::ffi::c_int
                            {
                                (*md).choice += 1;
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        5459197107747055838 => {
                            if (*md).choice > n - 6 as ::core::ffi::c_int {
                                (*md).choice = n - 1 as ::core::ffi::c_int;
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice += 1;
                                    name =
                                        (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                            .name_ptr();
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
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
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
                                    name =
                                        (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                            .name_ptr();
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
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
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
                            name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                .name_ptr();
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
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
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
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
                            name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                .name_ptr();
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != n - 1 as ::core::ffi::c_int
                            {
                                (*md).choice += 1;
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        5459197107747055838 => {
                            if (*md).choice > n - 6 as ::core::ffi::c_int {
                                (*md).choice = n - 1 as ::core::ffi::c_int;
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice += 1;
                                    name =
                                        (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                            .name_ptr();
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
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
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
                                    name =
                                        (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                            .name_ptr();
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
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
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
                            name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                .name_ptr();
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
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
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
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
                            name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                .name_ptr();
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != n - 1 as ::core::ffi::c_int
                            {
                                (*md).choice += 1;
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        5459197107747055838 => {
                            if (*md).choice > n - 6 as ::core::ffi::c_int {
                                (*md).choice = n - 1 as ::core::ffi::c_int;
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice += 1;
                                    name =
                                        (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                            .name_ptr();
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
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
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
                                    name =
                                        (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                            .name_ptr();
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
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
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
                            name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                .name_ptr();
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
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
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
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
                            name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                .name_ptr();
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != n - 1 as ::core::ffi::c_int
                            {
                                (*md).choice += 1;
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        5459197107747055838 => {
                            if (*md).choice > n - 6 as ::core::ffi::c_int {
                                (*md).choice = n - 1 as ::core::ffi::c_int;
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice += 1;
                                    name =
                                        (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                            .name_ptr();
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
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
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
                                    name =
                                        (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                            .name_ptr();
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
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
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
                            name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                .name_ptr();
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
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
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
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
                            name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                .name_ptr();
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != n - 1 as ::core::ffi::c_int
                            {
                                (*md).choice += 1;
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        5459197107747055838 => {
                            if (*md).choice > n - 6 as ::core::ffi::c_int {
                                (*md).choice = n - 1 as ::core::ffi::c_int;
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice += 1;
                                    name =
                                        (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                            .name_ptr();
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
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
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
                                    name =
                                        (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                            .name_ptr();
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
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
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
                            name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                .name_ptr();
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
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
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
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
                            name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                .name_ptr();
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != n - 1 as ::core::ffi::c_int
                            {
                                (*md).choice += 1;
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        5459197107747055838 => {
                            if (*md).choice > n - 6 as ::core::ffi::c_int {
                                (*md).choice = n - 1 as ::core::ffi::c_int;
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice += 1;
                                    name =
                                        (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                            .name_ptr();
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
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
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
                                    name =
                                        (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                            .name_ptr();
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
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
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
                            name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                .name_ptr();
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
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
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
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
                            name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                .name_ptr();
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != n - 1 as ::core::ffi::c_int
                            {
                                (*md).choice += 1;
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        5459197107747055838 => {
                            if (*md).choice > n - 6 as ::core::ffi::c_int {
                                (*md).choice = n - 1 as ::core::ffi::c_int;
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice += 1;
                                    name =
                                        (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                            .name_ptr();
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
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
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
                                    name =
                                        (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                            .name_ptr();
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
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
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
                            name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                .name_ptr();
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
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
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
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
                            name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                .name_ptr();
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != n - 1 as ::core::ffi::c_int
                            {
                                (*md).choice += 1;
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        5459197107747055838 => {
                            if (*md).choice > n - 6 as ::core::ffi::c_int {
                                (*md).choice = n - 1 as ::core::ffi::c_int;
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice += 1;
                                    name =
                                        (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                            .name_ptr();
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
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
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
                                    name =
                                        (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                            .name_ptr();
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
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
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
                            name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                .name_ptr();
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
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
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
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
                            name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                .name_ptr();
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != n - 1 as ::core::ffi::c_int
                            {
                                (*md).choice += 1;
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        5459197107747055838 => {
                            if (*md).choice > n - 6 as ::core::ffi::c_int {
                                (*md).choice = n - 1 as ::core::ffi::c_int;
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice += 1;
                                    name =
                                        (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                            .name_ptr();
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
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
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
                                    name =
                                        (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                            .name_ptr();
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
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
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
                            name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                .name_ptr();
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
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
                                name = (*(*menu).items.as_mut_ptr().offset((*md).choice as isize))
                                    .name_ptr();
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
    item = (*menu).items.as_mut_ptr().offset((*md).choice as isize) as *mut MenuRow;
    if (*item).name_ptr().is_null() || *(*item).name_ptr() as ::core::ffi::c_int == '-' as i32 {
        if (*md).flags & MENU_STAYOPEN != 0 {
            return 0 as ::core::ffi::c_int;
        }
        return 1 as ::core::ffi::c_int;
    }
    if let Some(callback) = (*md).cb.take() {
        callback(MenuSelection::Selected {
            index: (*md).choice as u_int,
            key: (*item).key,
        });
        return 1 as ::core::ffi::c_int;
    }
    if (*md).key != KEYC_NONE as ::core::ffi::c_ulong as key_code {
        saved_event.key = (*md).key;
        saved_event.m = (*md).m;
        event = &raw mut saved_event;
    } else {
        event = ::core::ptr::null_mut::<key_event>();
    }
    state = cmdq_new_state(&raw mut (*md).fs, event, 0 as ::core::ffi::c_int);
    if let Err(error) = cmd_parse_and_append(
        (*item)
            .command
            .as_deref()
            .expect("menu command row has a command"),
        c,
        state,
    ) {
        cmdq_append(
            c,
            cmdq_get_error(
                error
                    .as_ref()
                    .map_or(::core::ptr::null(), |cause| cause.as_ptr()),
            ),
        );
    }
    cmdq_free_state(state);
    return 1 as ::core::ffi::c_int;
}
pub unsafe fn menu_resize(mut md: *mut menu_data, mut w: *mut window) {
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
pub unsafe fn menu_display(
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
    cb: menu_choice_cb,
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
    let mut owner = Box::new(menu_data {
        style: (!style.is_null()).then(|| CStr::from_ptr(style).to_owned()),
        selected_style: (!selected_style.is_null())
            .then(|| CStr::from_ptr(selected_style).to_owned()),
        border_style: (!border_style.is_null()).then(|| CStr::from_ptr(border_style).to_owned()),
        ..menu_data::empty()
    });

    md = &raw mut *owner;
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
    if !fs.is_null() {
        cmd_find_copy_state(&raw mut (*md).fs, fs);
    } else if cmd_find_from_window(&raw mut (*md).fs, w, 0 as ::core::ffi::c_int)
        != 0 as ::core::ffi::c_int
    {
        cmd_find_clear_state(&raw mut (*md).fs, 0 as ::core::ffi::c_int);
    }
    screen_init(&mut (*md).s, sx, sy, 0 as u_int);
    if !(*md).flags & MENU_NOMOUSE != 0 {
        (*md).s.mode |= MODE_MOUSE_ALL | MODE_MOUSE_BUTTON;
    }
    (*md).s.mode &= !MODE_CURSOR;
    (*md).px = px;
    (*md).py = py;
    (*md).menu = menu;
    (*md).choice = -(1 as ::core::ffi::c_int);
    (*md).cb = cb;
    if (*md).flags & MENU_NOMOUSE != 0 {
        if starting_choice >= (*menu).count as ::core::ffi::c_int {
            starting_choice = (*menu).count.wrapping_sub(1 as u_int) as ::core::ffi::c_int;
            choice = starting_choice + 1 as ::core::ffi::c_int;
            loop {
                name = (*(*menu)
                    .items
                    .as_mut_ptr()
                    .offset((choice - 1 as ::core::ffi::c_int) as isize))
                .name_ptr();
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
                name = (*(*menu).items.as_mut_ptr().offset(choice as isize)).name_ptr();
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
    let md = Box::into_raw(owner).cast::<menu_data>();
    (*(*md).w).menu = md;
    redraw_invalidate_scene((*md).w);
    window_update_focus((*md).w);
    server_redraw_window((*md).w);
    return 0 as ::core::ffi::c_int;
}
