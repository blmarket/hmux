use crate::src::cmd::find::{cmd_find_clear_state, cmd_find_copy_state, cmd_find_from_window};
use crate::src::cmd::parse::cmd_parse_and_append;
use crate::src::cmd::queue::{cmdq_append, cmdq_free_state, cmdq_get_error, cmdq_new_state};
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
use crate::src::shared::command::{cmd_find_state, cmdq_item};
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
use crate::src::shared::screen::{screen, MODE_CURSOR, MODE_MOUSE_ALL, MODE_MOUSE_BUTTON};
use crate::src::shared::screen_write::screen_write_ctx;
use crate::src::shared::style::*;
use crate::src::shared::window::window;
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
unsafe fn menu_reapply_styles(md: &mut menu_data) {
    let options = (*md.w).options;
    let ft = format_create_defaults(
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        md.fs.s,
        md.fs.wl,
        md.fs.wp,
    );
    let mut parsed = style::default();
    for (cell, option, override_style) in [
        (&mut md.style_gc, c"menu-style", md.style.as_deref()),
        (
            &mut md.selected_style_gc,
            c"menu-selected-style",
            md.selected_style.as_deref(),
        ),
        (
            &mut md.border_style_gc,
            c"menu-border-style",
            md.border_style.as_deref(),
        ),
    ] {
        *cell = grid_default_cell;
        style_apply(cell, options, option.as_ptr(), ft);
        if let Some(override_style) = override_style {
            style_set(&mut parsed, &raw const grid_default_cell);
            if style_parse(&mut parsed, cell, override_style.as_ptr()) == 0 {
                cell.fg = parsed.gc.fg;
                cell.bg = parsed.gc.bg;
            }
        }
    }
    format_free(ft);
}

pub unsafe fn menu_update(md: &mut menu_data) {
    menu_reapply_styles(md);
    let menu = &md.menu;
    let mut ctx = screen_write_ctx::default();
    screen_write_start(&mut ctx, &mut md.s);
    screen_write_clearscreen(&mut ctx, 8);
    if md.border_lines != BOX_LINES_NONE {
        screen_write_box(
            &mut ctx,
            menu.width.wrapping_add(4),
            menu.count.wrapping_add(2),
            md.border_lines,
            Some(&md.border_style_gc),
            Some(&menu.title),
        );
    }
    screen_write_menu(
        &mut ctx,
        menu,
        md.choice,
        md.border_lines,
        &md.style_gc,
        Some(&md.border_style_gc),
        &md.selected_style_gc,
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
pub fn menu_get_cursor(md: &menu_data) -> (u_int, u_int) {
    let cy = if md.choice == -1 {
        md.py
    } else {
        md.py.wrapping_add(1).wrapping_add(md.choice as u_int)
    };
    (md.px.wrapping_add(2), cy)
}
pub fn menu_screen(md: &menu_data) -> &screen {
    &md.s
}
pub fn menu_width(md: &menu_data) -> u_int {
    md.menu.width.wrapping_add(4)
}
pub fn menu_height(md: &menu_data) -> u_int {
    md.menu.count.wrapping_add(2)
}
pub fn menu_x(md: &menu_data) -> u_int {
    md.px
}
pub fn menu_y(md: &menu_data) -> u_int {
    md.py
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
pub fn menu_resize(md: &mut menu_data, window_width: u_int, window_height: u_int) {
    let mut nx = md.px;
    let mut ny = md.py;
    let sx = menu_width(md);
    let sy = menu_height(md);
    if nx.wrapping_add(sx) > window_width {
        if window_width <= sx {
            nx = 0;
        } else {
            nx = window_width.wrapping_sub(sx);
        }
    }
    if ny.wrapping_add(sy) > window_height {
        if window_height <= sy {
            ny = 0;
        } else {
            ny = window_height.wrapping_sub(sy);
        }
    }
    md.px = nx;
    md.py = ny;
}
pub unsafe fn menu_display(
    menu: Box<menu>,
    flags: ::core::ffi::c_int,
    starting_choice: ::core::ffi::c_int,
    event: Option<&key_event>,
    mut px: u_int,
    mut py: u_int,
    c: *mut client,
    mut lines: box_lines,
    style: Option<&CStr>,
    selected_style: Option<&CStr>,
    border_style: Option<&CStr>,
    fs: *mut cmd_find_state,
    cb: menu_choice_cb,
) {
    let w = if fs.is_null() {
        (*(*(*c).session).curw).window
    } else {
        (*fs).w
    };
    let sx = menu.width.wrapping_add(4);
    let sy = menu.count.wrapping_add(2);
    if sx >= (*w).sx {
        px = 0;
    } else if px.wrapping_add(sx) > (*w).sx {
        px = (*w).sx.wrapping_sub(sx);
    }
    if sy >= (*w).sy {
        py = 0;
    } else if py.wrapping_add(sy) > (*w).sy {
        py = (*w).sy.wrapping_sub(sy);
    }
    (*w).menu_last_px = px;
    (*w).menu_last_py = py;
    if lines == BOX_LINES_DEFAULT {
        lines = options_get_number((*w).options, c"menu-border-lines".as_ptr()) as box_lines;
    }
    let mut owner = Box::new(menu_data {
        w,
        flags,
        border_lines: lines,
        style: style.map(CStr::to_owned),
        selected_style: selected_style.map(CStr::to_owned),
        border_style: border_style.map(CStr::to_owned),
        key: event.map_or(KEYC_NONE, |event| event.key),
        m: event.map_or_else(mouse_event::default, |event| event.m),
        px,
        py,
        choice: if flags & MENU_NOMOUSE != 0 {
            menu_initial_choice(&menu, starting_choice)
        } else {
            -1
        },
        cb,
        ..menu_data::new(menu)
    });
    if !fs.is_null() {
        cmd_find_copy_state(&mut owner.fs, fs);
    } else if cmd_find_from_window(&mut owner.fs, w, 0) != 0 {
        cmd_find_clear_state(&mut owner.fs, 0);
    }
    screen_init(&mut owner.s, sx, sy, 0);
    if flags & MENU_NOMOUSE == 0 {
        owner.s.mode |= MODE_MOUSE_ALL | MODE_MOUSE_BUTTON;
    }
    owner.s.mode &= !MODE_CURSOR;
    menu_close(w);
    (*w).menu = Box::into_raw(owner);
    redraw_invalidate_scene(w);
    window_update_focus(w);
    server_redraw_window(w);
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
