use crate::src::cmd::find::{cmd_find_clear_state, cmd_find_copy_state, cmd_find_from_window};
use crate::src::cmd::parse::cmd_parse_and_append;
use crate::src::cmd::queue::{
    cmdq_append, cmdq_free_state, cmdq_get_error, cmdq_get_event, cmdq_new_state,
};
use crate::src::ffi::libc::memcpy;
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
pub unsafe fn menu_add_items(menu: &mut menu, items: &[menu_item<'_>], c: *mut client) {
    for item in items {
        menu_add_item(
            menu,
            Some(item),
            std::ptr::null_mut(),
            c,
            std::ptr::null_mut(),
        );
    }
}
pub unsafe fn menu_add_item(
    menu: &mut menu,
    item: Option<&menu_item<'_>>,
    qitem: *mut cmdq_item,
    c: *mut client,
    fs: *mut cmd_find_state,
) {
    let Some(item) = item.filter(|item| !item.name.is_empty()) else {
        if menu.items.last().is_some_and(|row| row.name.is_some()) {
            menu.push_empty();
        }
        return;
    };
    let index = menu.push_empty();
    let expanded = if !fs.is_null() {
        format_single_from_state_cstring(qitem, item.name.as_ptr(), c, fs)
    } else {
        format_single_cstring(
            qitem,
            item.name.as_ptr(),
            c,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    if expanded.is_empty() {
        // Construction has not published this menu, so the placeholder is last.
        menu.items.pop();
        menu.refresh_items();
        return;
    }
    let mut max_width = (*c).tty.sx.wrapping_sub(4);
    let text = expanded.as_bytes();
    let mut key = if text[0] != b'-' && item.key != KEYC_UNKNOWN && item.key != KEYC_NONE {
        Some(key_string_format(item.key, false))
    } else {
        None
    };
    if let Some(label) = &key {
        let keylen = label.as_bytes().len().wrapping_add(3);
        if keylen <= (max_width / 4) as usize {
            max_width = (max_width as usize).wrapping_sub(keylen) as u_int;
        } else if keylen >= max_width as usize
            || text.len() >= (max_width as usize).wrapping_sub(keylen)
        {
            key = None;
        }
    }
    let truncated = text.len() > max_width as usize;
    if truncated {
        max_width = max_width.wrapping_sub(1);
    }
    let mut name = format_trim_right_bytes(&expanded, max_width);
    if truncated {
        name.push(b'>');
    }
    if let Some(key) = &key {
        name.extend_from_slice(b"#[default] #[align=right](");
        name.extend_from_slice(key.as_bytes());
        name.push(b')');
    }
    menu.items[index].name = Some(CString::new(name).expect("menu name contains no NUL"));
    menu.items[index].command = item.command.map(|command| {
        if !fs.is_null() {
            format_single_from_state_cstring(qitem, command.as_ptr(), c, fs)
        } else {
            format_single_cstring(
                qitem,
                command.as_ptr(),
                c,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        }
    });
    menu.items[index].key = item.key;
    let row_name = menu.items[index].name.as_ref().unwrap();
    let mut width = format_width(row_name.as_ptr());
    if row_name.as_bytes().starts_with(b"-") {
        width = width.wrapping_sub(1);
    }
    menu.width = menu.width.max(width);
}
pub unsafe fn menu_create(title: &CStr) -> Box<menu> {
    let title = title.to_owned();
    let width = format_width(title.as_ptr());
    let owner = Box::new(menu {
        title: title,
        items: Vec::new(),
        count: 0,
        width: width,
    });
    owner
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
    let menu = &(*md).menu;
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
    screen_write_clearscreen(&mut ctx, 8 as u_int);
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
    return (*md).menu.width.wrapping_add(4 as u_int);
}
pub unsafe fn menu_height(mut md: *mut menu_data) -> u_int {
    return (*md).menu.count.wrapping_add(2 as u_int);
}
pub unsafe fn menu_x(mut md: *mut menu_data) -> u_int {
    return (*md).px;
}
pub unsafe fn menu_y(mut md: *mut menu_data) -> u_int {
    return (*md).py;
}
#[derive(Debug, PartialEq, Eq)]
enum MenuKeyAction {
    Unchanged,
    Redraw,
    Close,
    Chosen { index: usize, key: key_code },
}

fn menu_initial_choice(menu: &menu, starting_choice: i32) -> i32 {
    if starting_choice < 0 || menu.items.is_empty() {
        return -1;
    }
    let count = menu.items.len();
    if starting_choice as usize >= count {
        return menu
            .items
            .iter()
            .rposition(MenuRow::is_selectable)
            .map_or(-1, |index| index as i32);
    }
    let start = starting_choice as usize;
    (start..count)
        .chain(0..start)
        .find(|&index| menu.items[index].is_selectable())
        .map_or(-1, |index| index as i32)
}

fn menu_chosen(md: &menu_data) -> MenuKeyAction {
    let Some(row) = md.menu.items.get(md.choice as usize) else {
        return MenuKeyAction::Close;
    };
    if !row.is_selectable() {
        return if md.flags & MENU_STAYOPEN != 0 {
            MenuKeyAction::Unchanged
        } else {
            MenuKeyAction::Close
        };
    }
    MenuKeyAction::Chosen {
        index: md.choice as usize,
        key: row.key,
    }
}

fn menu_handle_key(md: &mut menu_data, event: &key_event) -> MenuKeyAction {
    let rows = &md.menu.items;
    if rows.is_empty() {
        return MenuKeyAction::Close;
    }
    let n = rows.len() as i32;
    let old = md.choice;
    let key_type = event.key & KEYC_MASK_TYPE;
    if event.key & KEYC_MASK_KEY == KEYC_MOUSE
        || (key_type >= (KEYC_TYPE_MOUSEMOVE as key_code) << 32
            && key_type <= (KEYC_TYPE_TRIPLECLICK as key_code) << 32)
    {
        let m = &event.m;
        let buttons = m.b & MOUSE_MASK_BUTTONS as u_int;
        let drag = m.b & MOUSE_MASK_DRAG as u_int != 0;
        let release = buttons == 3;
        let wheel = buttons == MOUSE_WHEEL_UP as u_int || buttons == MOUSE_WHEEL_DOWN as u_int;
        let movement = drag && release;
        if md.flags & MENU_NOMOUSE != 0 {
            return if buttons != MOUSE_BUTTON_1 as u_int {
                MenuKeyAction::Close
            } else {
                MenuKeyAction::Unchanged
            };
        }
        if m.x < md.px
            || m.x > md.px.wrapping_add(4).wrapping_add(md.menu.width)
            || m.y < md.py.wrapping_add(1)
            || m.y
                > md.py
                    .wrapping_add(1)
                    .wrapping_add(n as u_int)
                    .wrapping_sub(1)
        {
            if (md.flags & MENU_STAYOPEN == 0 && !movement && release)
                || (md.flags & MENU_STAYOPEN != 0 && !release && !wheel && !drag)
            {
                return MenuKeyAction::Close;
            }
            md.choice = -1;
            return if old != -1 {
                MenuKeyAction::Redraw
            } else {
                MenuKeyAction::Unchanged
            };
        }
        if (md.flags & MENU_STAYOPEN == 0 && !movement && release)
            || (md.flags & MENU_STAYOPEN != 0 && !wheel && !drag)
        {
            // A release selects the previously highlighted row, not its own coordinates.
            return menu_chosen(md);
        }
        md.choice = m.y.wrapping_sub(md.py.wrapping_add(1)) as i32;
        return if md.choice != old {
            MenuKeyAction::Redraw
        } else {
            MenuKeyAction::Unchanged
        };
    }
    let key = event.key & !KEYC_MASK_FLAGS;
    if let Some(index) = rows
        .iter()
        .position(|row| row.is_selectable() && key == row.key & !KEYC_MASK_FLAGS)
    {
        md.choice = index as i32;
        return menu_chosen(md);
    }
    const CTRL_B: key_code = b'b' as key_code | KEYC_CTRL;
    const CTRL_C: key_code = b'c' as key_code | KEYC_CTRL;
    const CTRL_G: key_code = b'g' as key_code | KEYC_CTRL;
    const CTRL_BRACKET: key_code = b'[' as key_code | KEYC_CTRL;
    match key {
        KEYC_BTAB | KEYC_UP | 107 => {
            let stop = if old == -1 { 0 } else { old };
            loop {
                md.choice = if md.choice <= 0 { n - 1 } else { md.choice - 1 };
                if rows[md.choice as usize].is_selectable() || md.choice == stop {
                    break;
                }
            }
        }
        KEYC_BSPACE => {
            return if md.flags & MENU_TAB != 0 {
                MenuKeyAction::Close
            } else {
                MenuKeyAction::Unchanged
            };
        }
        9 | KEYC_DOWN | 106 => {
            if key == 9 {
                if md.flags & MENU_TAB == 0 {
                    return MenuKeyAction::Unchanged;
                }
                if md.choice == n - 1 {
                    return MenuKeyAction::Close;
                }
            }
            let stop = if old == -1 { 0 } else { old };
            loop {
                md.choice = if md.choice == -1 || md.choice == n - 1 {
                    0
                } else {
                    md.choice + 1
                };
                if rows[md.choice as usize].is_selectable() || md.choice == stop {
                    break;
                }
            }
        }
        KEYC_PPAGE | CTRL_B => {
            if md.choice < 6 {
                md.choice = 0;
            } else {
                let mut remaining = 5;
                while remaining > 0 {
                    md.choice -= 1;
                    if md.choice == 0 {
                        break;
                    }
                    if rows[md.choice as usize].is_selectable() {
                        remaining -= 1;
                    }
                }
            }
        }
        KEYC_NPAGE => {
            if md.choice > n - 6 {
                md.choice = n - 1;
            } else {
                let mut remaining = 5;
                while remaining > 0 {
                    md.choice += 1;
                    if md.choice == n - 1 {
                        break;
                    }
                    if rows[md.choice as usize].is_selectable() {
                        remaining -= 1;
                    }
                }
            }
            while md.choice != 0 && !rows[md.choice as usize].is_selectable() {
                md.choice -= 1;
            }
        }
        103 | KEYC_HOME => {
            md.choice = 0;
            while md.choice != n - 1 && !rows[md.choice as usize].is_selectable() {
                md.choice += 1;
            }
        }
        71 | KEYC_END => {
            md.choice = n - 1;
            while md.choice != 0 && !rows[md.choice as usize].is_selectable() {
                md.choice -= 1;
            }
        }
        13 => return menu_chosen(md),
        27 | 113 | CTRL_BRACKET | CTRL_C | CTRL_G => return MenuKeyAction::Close,
        _ => return MenuKeyAction::Unchanged,
    }
    MenuKeyAction::Redraw
}

pub unsafe fn menu_key(
    c: *mut client,
    md: *mut menu_data,
    event: &key_event,
) -> ::core::ffi::c_int {
    let (index, key) = match menu_handle_key(&mut *md, event) {
        MenuKeyAction::Unchanged => return 0,
        MenuKeyAction::Redraw => {
            server_redraw_window_menu((*md).w);
            return 0;
        }
        MenuKeyAction::Close => return 1,
        MenuKeyAction::Chosen { index, key } => (index, key),
    };
    // No row or menu-state borrow spans a user callback.
    if let Some(callback) = (*md).cb.take() {
        callback(MenuSelection::Selected {
            index: index as u_int,
            key,
        });
        return 1;
    }
    let mut saved_event = key_event {
        client: std::ptr::null_mut(),
        key: (*md).key,
        m: (*md).m,
        bytes: None,
    };
    let event = if (*md).key != KEYC_NONE {
        &mut saved_event
    } else {
        std::ptr::null_mut()
    };
    let state = cmdq_new_state(&raw mut (*md).fs, event, 0);
    if let Err(error) = cmd_parse_and_append(
        (&(*md).menu.items)[index]
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
                    .map_or(std::ptr::null(), |cause| cause.as_ptr()),
            ),
        );
    }
    cmdq_free_state(state);
    1
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
    sx = (*md).menu.width.wrapping_add(4 as u_int);
    sy = (*md).menu.count.wrapping_add(2 as u_int);
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
    menu: Box<menu>,
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
) {
    let mut md: *mut menu_data = ::core::ptr::null_mut::<menu_data>();
    let mut event: *mut key_event = ::core::ptr::null_mut::<key_event>();
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
        ..menu_data::new(menu)
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
    (*md).choice = if flags & MENU_NOMOUSE != 0 {
        menu_initial_choice(&(*md).menu, starting_choice)
    } else {
        -1
    };
    (*md).cb = cb;
    menu_close((*md).w);
    let md = Box::into_raw(owner).cast::<menu_data>();
    (*(*md).w).menu = md;
    redraw_invalidate_scene((*md).w);
    window_update_focus((*md).w);
    server_redraw_window((*md).w);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    fn state(names: &[Option<&CStr>]) -> menu_data {
        menu_data::new(Box::new(menu {
            title: c"Navigation".to_owned(),
            items: names
                .iter()
                .enumerate()
                .map(|(index, name)| MenuRow {
                    name: name.map(CStr::to_owned),
                    key: b'a' as key_code + index as key_code,
                    command: None,
                })
                .collect(),
            count: names.len() as u_int,
            width: 20,
        }))
    }

    fn event(key: key_code) -> key_event {
        key_event {
            client: std::ptr::null_mut(),
            key,
            m: mouse_event::default(),
            bytes: None,
        }
    }

    #[test]
    fn navigation_skips_unselectable_rows_and_shortcuts_take_priority() {
        let mut md = state(&[
            None,
            Some(c"-disabled"),
            Some(c"first"),
            None,
            Some(c"last"),
        ]);
        md.choice = -1;
        for (key, choice) in [
            // With no highlight, Down stops at row zero even if it is a separator.
            (KEYC_DOWN, 0),
            (KEYC_DOWN, 2),
            (KEYC_DOWN, 4),
            (KEYC_DOWN, 2),
            (KEYC_UP, 4),
            (KEYC_HOME, 2),
            (KEYC_END, 4),
            // Page up lands on row zero even when it is a separator.
            (KEYC_PPAGE, 0),
            (KEYC_NPAGE, 4),
        ] {
            assert_eq!(menu_handle_key(&mut md, &event(key)), MenuKeyAction::Redraw);
            assert_eq!(md.choice, choice);
        }
        // Flags are ignored for matching, but the callback receives the row's key.
        md.menu.items[2].key = KEYC_UP | KEYC_VI;
        md.menu.items[4].key = KEYC_UP;
        assert_eq!(
            menu_handle_key(&mut md, &event(KEYC_UP | KEYC_SENT)),
            MenuKeyAction::Chosen {
                index: 2,
                key: KEYC_UP | KEYC_VI
            }
        );
        md.menu.items[2].key = b'q' as key_code;
        assert_eq!(
            menu_handle_key(&mut md, &event(b'q' as key_code)),
            MenuKeyAction::Chosen {
                index: 2,
                key: b'q' as key_code
            }
        );
    }

    #[test]
    fn hover_only_highlights_and_release_uses_the_previous_row() {
        for flags in [0, MENU_STAYOPEN] {
            let mut md = state(&[Some(c"first"), None, Some(c"-disabled"), Some(c"last")]);
            md.flags = flags;
            md.choice = -1;
            md.px = 10;
            md.py = 5;
            let mut mouse = event(KEYC_MOUSE);
            mouse.m.x = 12;
            mouse.m.y = 6;
            mouse.m.b = MOUSE_MASK_DRAG as u_int | 3;
            assert_eq!(menu_handle_key(&mut md, &mouse), MenuKeyAction::Redraw);
            assert_eq!(md.choice, 0);
            assert_eq!(menu_handle_key(&mut md, &mouse), MenuKeyAction::Unchanged);
            mouse.m.y = 9;
            mouse.m.b = 3;
            assert_eq!(
                menu_handle_key(&mut md, &mouse),
                MenuKeyAction::Chosen {
                    index: 0,
                    key: b'a' as key_code
                }
            );
            // Hovering outside clears the highlight and keeps both menu styles open.
            mouse.m.x = 9;
            mouse.m.b = MOUSE_MASK_DRAG as u_int | 3;
            assert_eq!(menu_handle_key(&mut md, &mouse), MenuKeyAction::Redraw);
            assert_eq!(md.choice, -1);
            assert_eq!(menu_handle_key(&mut md, &mouse), MenuKeyAction::Unchanged);
            // A separator or disabled row never invokes its command.
            mouse.m.x = 12;
            for y in [7, 8] {
                mouse.m.y = y;
                mouse.m.b = MOUSE_MASK_DRAG as u_int | 3;
                assert_eq!(menu_handle_key(&mut md, &mouse), MenuKeyAction::Redraw);
                mouse.m.b = 3;
                assert_eq!(
                    menu_handle_key(&mut md, &mouse),
                    if flags == MENU_STAYOPEN {
                        MenuKeyAction::Unchanged
                    } else {
                        MenuKeyAction::Close
                    }
                );
            }
            md.flags = MENU_NOMOUSE;
            mouse.m.b = 0;
            assert_eq!(menu_handle_key(&mut md, &mouse), MenuKeyAction::Unchanged);
            mouse.m.b = 3;
            assert_eq!(menu_handle_key(&mut md, &mouse), MenuKeyAction::Close);
        }
    }

    #[test]
    fn starting_choice_wraps_or_searches_backwards_without_selecting_disabled_rows() {
        let md = state(&[
            None,
            Some(c"first"),
            Some(c"-disabled"),
            None,
            Some(c"last"),
            None,
        ]);
        for (start, choice) in [
            (-1, -1),
            (0, 1),
            (1, 1),
            (2, 4),
            (5, 1),
            (6, 4),
            (i32::MAX, 4),
        ] {
            assert_eq!(menu_initial_choice(&md.menu, start), choice);
        }
        for names in [&[][..], &[None, Some(c"-disabled")][..]] {
            let mut md = state(names);
            for start in [-1, 0, 1, 10] {
                assert_eq!(menu_initial_choice(&md.menu, start), -1);
            }
            md.choice = -1;
            assert_eq!(menu_handle_key(&mut md, &event(13)), MenuKeyAction::Close);
        }
    }

    #[test]
    fn selected_callback_can_destroy_its_menu_data() {
        let owner = Rc::new(RefCell::new(None));
        let weak_owner = Rc::downgrade(&owner);
        let called = Rc::new(Cell::new(false));
        let callback_called = Rc::clone(&called);
        let mut md = Box::new(state(&[Some(c"close myself")]));
        md.cb = Some(Box::new(move |selection| {
            assert_eq!(
                selection,
                MenuSelection::Selected {
                    index: 0,
                    key: b'a' as key_code
                }
            );
            let owner = weak_owner.upgrade().unwrap();
            let data: Box<menu_data> = owner.borrow_mut().take().unwrap();
            assert!(data.cb.is_none());
            drop(data);
            callback_called.set(true);
        }));
        let data = &raw mut *md;
        *owner.borrow_mut() = Some(md);
        let event = event(b'a' as key_code);
        assert_eq!(unsafe { menu_key(std::ptr::null_mut(), data, &event) }, 1);
        assert!(called.get());
        assert!(owner.borrow().is_none());
    }
}
