use crate::cmd::CmdqItemRef;
use crate::cmd::CmdqStateRef;
use crate::cmd::cmd_find_copy_state;
use crate::cmd::cmd_parse_and_append;
use crate::cmd::cmdq_item;
use crate::cmd::{CmdqItemWeak, cmdq_append};
use crate::fmt_args;
use crate::format::{format_create_defaults, format_create_from_state_for_client, format_expand};
use crate::grid::grid_default_cell;

use crate::screen::{Screen, ScreenModeState};
use crate::screen::ScreenWriteCtx;

use crate::server::{client_ref_of, server_client_set_overlay};

use crate::style::{RustStyleCodec, StyleCodec, style_apply, style_default};
use crate::text::{KeyStringCodec, RustKeyStringCodec};
use crate::tty::tty_draw_line;
pub use crate::types::*;
use crate::xmalloc::xasprintf;
use crate::{FormatText, RustFormatText};
use ::core::ffi::CStr;
use ::std::ffi::CString;
#[repr(C)]
pub struct menu_data {
    pub(crate) item: Option<CmdqItemWeak>,
    pub flags: core::ffi::c_int,
    pub style: Option<std::ffi::CString>,
    pub border_style: Option<std::ffi::CString>,
    pub selected_style: Option<std::ffi::CString>,
    pub style_gc: grid_cell,
    pub border_style_gc: grid_cell,
    pub selected_style_gc: grid_cell,
    pub border_lines: box_lines,
    pub fs: cmd_find_state,
    pub(crate) s: ScreenRef,
    pub r: VisibleRangesRef,
    pub px: u_int,
    pub py: u_int,
    pub menu: Box<menu>,
    pub choice: core::ffi::c_int,
    pub cb: menu_choice_cb,
}

#[derive(Clone)]
pub struct MenuDataRef(std::rc::Rc<std::cell::RefCell<menu_data>>);

impl MenuDataRef {
    pub(crate) fn new(value: menu_data) -> Self {
        Self(std::rc::Rc::new(std::cell::RefCell::new(value)))
    }

    pub(crate) fn borrow(&self) -> std::cell::Ref<'_, menu_data> {
        self.0.borrow()
    }

    pub(crate) fn borrow_mut(&self) -> std::cell::RefMut<'_, menu_data> {
        self.0.borrow_mut()
    }
}

impl PartialEq for MenuDataRef {
    fn eq(&self, other: &Self) -> bool {
        std::rc::Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for MenuDataRef {}

impl std::fmt::Debug for MenuDataRef {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_tuple("MenuDataRef")
            .field(&std::rc::Rc::as_ptr(&self.0))
            .finish()
    }
}

pub use crate::consts::{
    BOX_LINES_DEFAULT, BOX_LINES_NONE, CLIENT_REDRAWOVERLAY, KEYC_MASK_FLAGS, KEYC_MASK_KEY,
    KEYC_MASK_TYPE, KEYC_MOUSE, KEYC_NONE, KEYC_TYPE_MOUSEMOVE, KEYC_TYPE_TRIPLECLICK,
    KEYC_UNKNOWN, MENU_NOMOUSE, MENU_STAYOPEN, MENU_TAB, MODE_CURSOR, MODE_MOUSE_ALL,
    MODE_MOUSE_BUTTON, MOUSE_BUTTON_1, MOUSE_MASK_BUTTONS, MOUSE_MASK_DRAG, MOUSE_WHEEL_DOWN,
    MOUSE_WHEEL_UP, UINT_MAX,
};

pub unsafe fn menu_add_items(
    menu: &mut menu,
    items: &[menu_item],
    qitem: Option<&cmdq_item>,
    c: &mut client,
    fs: Option<&cmd_find_state>,
) {
    unsafe {
        for item in items {
            menu_add_item(menu, Some(item), qitem, c, fs);
        }
    }
}
pub unsafe fn menu_add_item(
    menu: &mut menu,
    item: Option<&menu_item<'_>>,
    qitem: Option<&cmdq_item>,
    c: &mut client,
    fs: Option<&cmd_find_state>,
) {
    let owner = client_ref_of(c);
    unsafe { menu_add_item_with_width(menu, item, qitem, owner.as_ref(), fs, || c.tty.sx) };
}

/// Adds one formatted item using a retained drawn client and optional targets.
///
/// The queue item's current client supplies format jobs independently of `c`.
/// Missing targets follow existing defaults inheritance; retained clients need
/// not be registered. Leading/repeated separators and empty expanded names are
/// skipped. Name and command use separate fresh contexts, and terminal width is
/// read after a nonempty name expands. Trimming, key suffixes, disabled markers,
/// stored command text and the menu's maximum rendered width are updated together.
/// No menu choice or stored command is executed here.
///
/// # Safety
/// Run on the initialized server thread, excluding conflicting entity/mode access
/// and destruction of panes resolved during defaults or expansion. Mode formats
/// read those entities and write only their format tree. Hold no entity payload
/// borrow across this call; the width snapshot's checked borrow ends internally.
pub(crate) unsafe fn menu_add_item_for_client(
    menu: &mut menu,
    item: Option<&menu_item<'_>>,
    qitem: Option<&cmdq_item>,
    c: &ClientRef,
    fs: Option<&cmd_find_state>,
) {
    unsafe { menu_add_item_with_width(menu, item, qitem, Some(c), fs, || c.terminal_size().width) };
}

unsafe fn menu_add_item_with_width(
    menu: &mut menu,
    item: Option<&menu_item<'_>>,
    qitem: Option<&cmdq_item>,
    c: Option<&ClientRef>,
    fs: Option<&cmd_find_state>,
    terminal_width: impl FnOnce() -> u_int,
) {
    unsafe {
        let mut key: Option<CString> = None;
        let mut suffix = c"";
        let mut max_width: u_int;

        let keylen: size_t;

        let line: core::ffi::c_int = !item
            .is_some_and(|item| item.name.is_some_and(|name| !name.is_empty()))
            as core::ffi::c_int;
        if line != 0 && menu.items.is_empty() {
            return;
        }
        if line != 0 && (&menu.items)[menu.items.len() - 1].name.is_none() {
            return;
        }
        if line != 0 {
            menu.items.push(menu_entry::default());
            return;
        }
        let item = item.expect("a menu line has an item");
        let item_name = item.name.expect("a menu line has a name");
        let empty = cmd_find_state::default();
        let fs = fs.unwrap_or(&empty);
        let expanded = {
            let mut ft = format_create_from_state_for_client(qitem, c, fs);
            format_expand(&mut ft, item_name)
        };
        let s = expanded.as_c_str();
        if s.is_empty() {
            return;
        }
        max_width = terminal_width().wrapping_sub(4 as u_int);
        let slen: size_t = s.to_bytes().len();
        if !s.to_bytes().starts_with(b"-")
            && item.key != KEYC_UNKNOWN as core::ffi::c_ulong as key_code
            && item.key != KEYC_NONE as core::ffi::c_ulong as key_code
        {
            key = Some(RustKeyStringCodec.format_key(item.key, false));
            keylen = key
                .as_ref()
                .unwrap()
                .as_bytes()
                .len()
                .wrapping_add(3 as size_t);
            if keylen <= max_width.wrapping_div(4 as u_int) as size_t {
                max_width = (max_width as size_t).wrapping_sub(keylen) as u_int as u_int;
            } else if keylen >= max_width as size_t
                || slen >= (max_width as size_t).wrapping_sub(keylen)
            {
                key = None;
            }
        }
        if slen > max_width as size_t {
            max_width = max_width.wrapping_sub(1);
            suffix = c">";
        }
        let trimmed = RustFormatText.trim_right(s.to_bytes(), max_width);
        let name = if let Some(key) = key {
            xasprintf(
                c"%s%s#[default] #[align=right](%s)",
                fmt_args![trimmed.as_c_str(), suffix, key.as_c_str()],
            )
        } else {
            xasprintf(c"%s%s", fmt_args![trimmed.as_c_str(), suffix])
        };
        let command = item.command.map(|cmd| {
            let mut ft = format_create_from_state_for_client(qitem, c, fs);
            format_expand(&mut ft, cmd)
        });
        let mut width = RustFormatText.width(name.as_bytes());
        if name.as_bytes().starts_with(b"-") {
            width = width.wrapping_sub(1);
        }
        menu.width = menu.width.max(width);
        menu.items.push(menu_entry {
            name: Some(name),
            key: item.key,
            command,
        });
    }
}
pub fn menu_create(title: &CStr) -> Box<menu> {
    {
        Box::new(menu {
            title: Some(title.to_owned()),
            items: Vec::new(),
            width: RustFormatText.width(title.to_bytes()),
        })
    }
}
/// The menu's screen mode, and where its cursor stands on the terminal.
pub fn menu_mode_cb(data: &menu_data) -> (ScreenModeState, u_int, u_int) {
    let cx = data.px.wrapping_add(2);
    let cy = if data.choice == -1 {
        data.py
    } else {
        data.py.wrapping_add(1).wrapping_add(data.choice as u_int)
    };
    (data.s.borrow().mode_state(), cx, cy)
}

pub fn menu_check_cb<'a>(
    data: &'a mut menu_data,
    px: u_int,
    py: u_int,
    nx: u_int,
) -> std::cell::RefMut<'a, visible_ranges> {
    let mut ranges = data.r.borrow_mut();
    ranges.set_overlay_range(
        data.px,
        data.py,
        data.menu.width.wrapping_add(4),
        (data.menu.items.len() as u_int).wrapping_add(2),
        px,
        py,
        nx,
    );
    ranges
}
unsafe fn menu_reapply_styles(md: &mut menu_data, c: &client) {
    unsafe {
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
            Some(c),
            Some(s),
            Some(wl),
            None::<&dyn crate::WindowPane>,
        );
        md.style_gc = grid_default_cell;
        style_apply(&mut md.style_gc, &o, c"menu-style", Some(&mut ft));
        if let Some(style) = md.style.as_deref() {
            RustStyleCodec.set(&mut sytmp, &grid_default_cell);
            if RustStyleCodec
                .parse(&mut sytmp, &md.style_gc, style.to_bytes())
                .is_ok()
            {
                md.style_gc.fg = sytmp.gc.fg;
                md.style_gc.bg = sytmp.gc.bg;
            }
        }
        md.selected_style_gc = grid_default_cell;
        style_apply(
            &mut md.selected_style_gc,
            &o,
            c"menu-selected-style",
            Some(&mut ft),
        );
        if let Some(style) = md.selected_style.as_deref() {
            RustStyleCodec.set(&mut sytmp, &grid_default_cell);
            if RustStyleCodec
                .parse(&mut sytmp, &md.selected_style_gc, style.to_bytes())
                .is_ok()
            {
                md.selected_style_gc.fg = sytmp.gc.fg;
                md.selected_style_gc.bg = sytmp.gc.bg;
            }
        }
        md.border_style_gc = grid_default_cell;
        style_apply(
            &mut md.border_style_gc,
            &o,
            c"menu-border-style",
            Some(&mut ft),
        );
        if let Some(style) = md.border_style.as_deref() {
            RustStyleCodec.set(&mut sytmp, &grid_default_cell);
            if RustStyleCodec
                .parse(&mut sytmp, &md.border_style_gc, style.to_bytes())
                .is_ok()
            {
                md.border_style_gc.fg = sytmp.gc.fg;
                md.border_style_gc.bg = sytmp.gc.bg;
            }
        }
    }
}

pub(crate) fn menu_resize_cb(c: &client, data: &mut menu_data) {
    let mut nx: u_int;
    let mut ny: u_int;

    nx = data.px;
    ny = data.py;
    let w: u_int = data.menu.width.wrapping_add(4 as u_int);
    let h: u_int = (data.menu.items.len() as u_int).wrapping_add(2 as u_int);
    if nx.wrapping_add(w) > c.tty.sx {
        if c.tty.sx <= w {
            nx = 0 as u_int;
        } else {
            nx = c.tty.sx.wrapping_sub(w);
        }
    }
    if ny.wrapping_add(h) > c.tty.sy {
        if c.tty.sy <= h {
            ny = 0 as u_int;
        } else {
            ny = c.tty.sy.wrapping_sub(h);
        }
    }
    data.px = nx;
    data.py = ny;
}

#[allow(clippy::too_many_arguments)]
pub fn menu_prepare(
    menu: Box<menu>,
    flags: core::ffi::c_int,
    mut starting_choice: core::ffi::c_int,
    item: Option<&cmdq_item>,
    mut px: u_int,
    mut py: u_int,
    c: &client,
    mut lines: box_lines,
    style: Option<&CStr>,
    selected_style: Option<&CStr>,
    border_style: Option<&CStr>,
    fs: Option<&cmd_find_state>,
    cb: menu_choice_cb,
) -> Option<MenuDataRef> {
    {
        let mut choice: core::ffi::c_int;
        let session = c.attached_session().expect("a menu client has a session");
        let link = session.curw().expect("a menu client has a current window");
        let options = link
            .window()
            .expect("a window link owns its window")
            .options();
        if c.tty.sx < menu.width.wrapping_add(4 as u_int)
            || c.tty.sy < (menu.items.len() as u_int).wrapping_add(2 as u_int)
        {
            return None;
        }
        if px.wrapping_add(menu.width).wrapping_add(4 as u_int) > c.tty.sx {
            px = c.tty.sx.wrapping_sub(menu.width).wrapping_sub(4 as u_int);
        }
        if py
            .wrapping_add(menu.items.len() as u_int)
            .wrapping_add(2 as u_int)
            > c.tty.sy
        {
            py = c
                .tty
                .sy
                .wrapping_sub(menu.items.len() as u_int)
                .wrapping_sub(2 as u_int);
        }
        if lines as core::ffi::c_int == BOX_LINES_DEFAULT as core::ffi::c_int {
            lines = options.number(c"menu-border-lines") as box_lines;
        }
        let mut state = menu_data {
            item: item
                .and_then(crate::cmd::cmdq_item_ref_of)
                .map(|item| item.downgrade()),
            flags,
            style: None,
            border_style: None,
            selected_style: None,
            style_gc: grid_default_cell,
            border_style_gc: grid_default_cell,
            selected_style_gc: grid_default_cell,
            border_lines: lines,
            fs: cmd_find_state::default(),
            s: ScreenRef::default(),
            r: VisibleRangesRef::default(),
            px: 0,
            py: 0,
            menu,
            choice: -(1 as core::ffi::c_int),
            cb: None,
        };
        let md = &mut state;
        md.style = style.map(CStr::to_owned);
        md.selected_style = selected_style.map(CStr::to_owned);
        md.border_style = border_style.map(CStr::to_owned);
        if let Some(fs) = fs {
            cmd_find_copy_state(&mut md.fs, fs);
        }
        md.s = ScreenRef::new(RustScreen::new_with_server_options(
            md.menu.width.wrapping_add(4 as u_int),
            (md.menu.items.len() as u_int).wrapping_add(2 as u_int),
            0 as u_int,
        ));
        let mut mode = md.s.borrow().mode();
        if !md.flags & MENU_NOMOUSE != 0 {
            mode |= MODE_MOUSE_ALL | MODE_MOUSE_BUTTON;
        }
        mode &= !MODE_CURSOR;
        md.s.borrow_mut().set_mode(mode);
        md.px = px;
        md.py = py;
        md.choice = -(1 as core::ffi::c_int);
        if md.flags & MENU_NOMOUSE != 0 {
            if starting_choice >= md.menu.items.len() as core::ffi::c_int {
                starting_choice =
                    (md.menu.items.len() as u_int).wrapping_sub(1 as u_int) as core::ffi::c_int;
                choice = starting_choice + 1 as core::ffi::c_int;
                loop {
                    if md.menu.items[(choice - 1 as core::ffi::c_int) as usize].is_selectable() {
                        md.choice = choice - 1 as core::ffi::c_int;
                        break;
                    } else {
                        choice -= 1;
                        if choice == 0 as core::ffi::c_int {
                            choice = md.menu.items.len() as core::ffi::c_int;
                        }
                        if choice == starting_choice + 1 as core::ffi::c_int {
                            break;
                        }
                    }
                }
            } else if starting_choice >= 0 as core::ffi::c_int {
                choice = starting_choice;
                loop {
                    if md.menu.items[choice as usize].is_selectable() {
                        md.choice = choice;
                        break;
                    } else {
                        choice += 1;
                        if choice == md.menu.items.len() as core::ffi::c_int {
                            choice = 0 as core::ffi::c_int;
                        }
                        if choice == starting_choice {
                            break;
                        }
                    }
                }
            }
        }
        md.cb = cb;
        Some(MenuDataRef::new(state))
    }
}
#[allow(clippy::too_many_arguments)]
pub unsafe fn menu_display(
    menu: Box<menu>,
    flags: core::ffi::c_int,
    starting_choice: core::ffi::c_int,
    item: Option<&cmdq_item>,
    px: u_int,
    py: u_int,
    c: &mut client,
    lines: box_lines,
    style: Option<&CStr>,
    selected_style: Option<&CStr>,
    border_style: Option<&CStr>,
    fs: Option<&cmd_find_state>,
    cb: menu_choice_cb,
) -> core::ffi::c_int {
    unsafe {
        let Some(md) = menu_prepare(
            menu,
            flags,
            starting_choice,
            item,
            px,
            py,
            c,
            lines,
            style,
            selected_style,
            border_style,
            fs,
            cb,
        ) else {
            return -(1 as core::ffi::c_int);
        };
        server_client_set_overlay(&mut *c, 0 as u_int, Overlay::Menu, OverlayState::Menu(md));
        0 as core::ffi::c_int
    }
}

use crate::screen::RustScreen;

#[cfg(test)]
#[path = "menu_focused_tests.rs"]
mod focused_tests;

impl MenuDataRef {
    pub unsafe fn draw(&self, c: &mut client, _rctx: &mut screen_redraw_ctx) {
        let owner = self;

        unsafe {
            let (screen, width, px, py) = {
                let mut guard = owner.borrow_mut();
                let data = &mut *guard;
                menu_reapply_styles(data, c);
                let screen = data.s.clone();
                let menu = &data.menu;
                let width = menu.width.wrapping_add(4);
                let px: u_int = data.px;
                let py: u_int = data.py;
                let mut writer = crate::screen::RustScreenWriteCtx::on_shared_screen(&screen);
                writer.clearscreen(8 as u_int);
                if data.border_lines as core::ffi::c_int != BOX_LINES_NONE as core::ffi::c_int {
                    writer.box_(
                        width,
                        (menu.items.len() as u_int).wrapping_add(2 as u_int),
                        data.border_lines,
                        Some(&data.border_style_gc),
                        menu.title.as_deref(),
                    );
                }
                writer.menu(
                    menu,
                    data.choice,
                    data.border_lines,
                    &data.style_gc,
                    &data.border_style_gc,
                    &data.selected_style_gc,
                );
                writer.finish();
                (screen, width, px, py)
            };
            let screen = screen.borrow();
            for i in 0..screen.size().1 {
                tty_draw_line(
                    &mut c.tty,
                    &screen,
                    0 as u_int,
                    i,
                    width,
                    px,
                    py.wrapping_add(i),
                    &grid_default_cell,
                    None,
                );
            }
        }
    }
    pub(crate) fn close(self, _c: &mut client) {
        let owner = self;

        let (item, callback) = {
            let mut md = owner.borrow_mut();
            (md.item.take().and_then(|item| item.upgrade()), md.cb.take())
        };
        if let Some(item) = item {
            item.resume();
        }
        if let Some(callback) = callback {
            callback(UINT_MAX, KEYC_NONE as key_code);
        }
    }
    pub unsafe fn key(&self, c: &mut client, event: &mut key_event) -> core::ffi::c_int {
        let owner = self;

        unsafe {
            let mut current_block: u64;
            let mut guard = owner.borrow_mut();
            let md = &mut *guard;
            let menu = &md.menu;
            let m: &mut mouse_event = &mut event.m;
            let mut i: u_int;
            let count: core::ffi::c_int = menu.items.len() as core::ffi::c_int;
            let mut old: core::ffi::c_int = md.choice;
            let mut selectable = false;
            let mut error = None;
            if event.key as core::ffi::c_ulonglong & KEYC_MASK_KEY
                == KEYC_MOUSE as core::ffi::c_ulong as core::ffi::c_ulonglong
                || event.key as core::ffi::c_ulonglong & KEYC_MASK_TYPE
                    >= (KEYC_TYPE_MOUSEMOVE as core::ffi::c_int as core::ffi::c_ulonglong)
                        << 32 as core::ffi::c_int
                    && event.key as core::ffi::c_ulonglong & KEYC_MASK_TYPE
                        <= (KEYC_TYPE_TRIPLECLICK as core::ffi::c_int as core::ffi::c_ulonglong)
                            << 32 as core::ffi::c_int
            {
                if md.flags & MENU_NOMOUSE != 0 {
                    if m.b & MOUSE_MASK_BUTTONS as u_int != MOUSE_BUTTON_1 as u_int {
                        return 1 as core::ffi::c_int;
                    }
                    return 0 as core::ffi::c_int;
                }
                if m.x < md.px
                    || m.x > md.px.wrapping_add(4 as u_int).wrapping_add(menu.width)
                    || m.y < md.py.wrapping_add(1 as u_int)
                    || m.y
                        > md.py
                            .wrapping_add(1 as u_int)
                            .wrapping_add(count as u_int)
                            .wrapping_sub(1 as u_int)
                {
                    if !md.flags & MENU_STAYOPEN != 0 {
                        if m.b & MOUSE_MASK_BUTTONS as u_int == 3 as u_int {
                            return 1 as core::ffi::c_int;
                        }
                    } else if !(m.b & MOUSE_MASK_BUTTONS as u_int == 3 as u_int)
                        && !(m.b & MOUSE_MASK_BUTTONS as u_int == MOUSE_WHEEL_UP as u_int
                            || m.b & MOUSE_MASK_BUTTONS as u_int == MOUSE_WHEEL_DOWN as u_int)
                        && m.b & MOUSE_MASK_DRAG as u_int == 0
                    {
                        return 1 as core::ffi::c_int;
                    }
                    if md.choice != -(1 as core::ffi::c_int) {
                        md.choice = -(1 as core::ffi::c_int);
                        c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                    }
                    return 0 as core::ffi::c_int;
                }
                if !md.flags & MENU_STAYOPEN != 0 {
                    if m.b & MOUSE_MASK_BUTTONS as u_int == 3 as u_int {
                        current_block = 3989530248692967800;
                    } else {
                        current_block = 14576567515993809846;
                    }
                } else if !(m.b & MOUSE_MASK_BUTTONS as u_int == MOUSE_WHEEL_UP as u_int
                    || m.b & MOUSE_MASK_BUTTONS as u_int == MOUSE_WHEEL_DOWN as u_int)
                    && m.b & MOUSE_MASK_DRAG as u_int == 0
                {
                    current_block = 3989530248692967800;
                } else {
                    current_block = 14576567515993809846;
                }
                match current_block {
                    3989530248692967800 => {}
                    _ => {
                        md.choice =
                            m.y.wrapping_sub(md.py.wrapping_add(1 as u_int)) as core::ffi::c_int;
                        if md.choice != old {
                            c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                        }
                        return 0 as core::ffi::c_int;
                    }
                }
            } else {
                i = 0 as u_int;
                loop {
                    if !(i < count as u_int) {
                        current_block = 2891135413264362348;
                        break;
                    }
                    selectable = menu.items[i as usize].is_selectable();
                    if !(!selectable)
                        && event.key as core::ffi::c_ulonglong & !KEYC_MASK_FLAGS
                            == (&menu.items)[i as usize].key
                    {
                        md.choice = i as core::ffi::c_int;
                        current_block = 3989530248692967800;
                        break;
                    }
                    i = i.wrapping_add(1);
                }
                match current_block {
                    3989530248692967800 => {}
                    _ => match event.key as core::ffi::c_ulonglong & !KEYC_MASK_FLAGS {
                        8589934618 | 8589934619 | 107 => {
                            current_block = 4458768223536474289;
                            match current_block {
                                9508525478627385121 => {
                                    md.choice = count - 1 as core::ffi::c_int;
                                    selectable = menu.items[md.choice as usize].is_selectable();
                                    while !selectable {
                                        md.choice -= 1;
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    current_block = 18425699056680496821;
                                }
                                10434433275429566483 => {
                                    md.choice = 0 as core::ffi::c_int;
                                    selectable = menu.items[md.choice as usize].is_selectable();
                                    while !selectable {
                                        md.choice += 1;
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    current_block = 18425699056680496821;
                                }
                                6983216613468852725 => {
                                    if md.choice > count - 6 as core::ffi::c_int {
                                        md.choice = count - 1 as core::ffi::c_int;
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                    } else {
                                        i = 5 as u_int;
                                        while i > 0 as u_int {
                                            md.choice += 1;
                                            selectable =
                                                menu.items[md.choice as usize].is_selectable();
                                            if md.choice != count - 1 as core::ffi::c_int
                                                && (selectable)
                                            {
                                                i = i.wrapping_sub(1);
                                            } else if md.choice == count - 1 as core::ffi::c_int {
                                                break;
                                            }
                                        }
                                    }
                                    while !selectable {
                                        md.choice -= 1;
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    current_block = 18425699056680496821;
                                }
                                4604734892166874824 => {
                                    if md.choice < 6 as core::ffi::c_int {
                                        md.choice = 0 as core::ffi::c_int;
                                    } else {
                                        i = 5 as u_int;
                                        while i > 0 as u_int {
                                            md.choice -= 1;
                                            selectable =
                                                menu.items[md.choice as usize].is_selectable();
                                            if md.choice != 0 as core::ffi::c_int && (selectable) {
                                                i = i.wrapping_sub(1);
                                            } else if md.choice == 0 as core::ffi::c_int {
                                                break;
                                            }
                                        }
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    current_block = 18425699056680496821;
                                }
                                5386196988030698307 => {
                                    if !md.flags & MENU_TAB != 0 {
                                        current_block = 18425699056680496821;
                                    } else {
                                        if md.choice == count - 1 as core::ffi::c_int {
                                            return 1 as core::ffi::c_int;
                                        }
                                        current_block = 16350834992246794677;
                                    }
                                }
                                5589979562539812337 => {
                                    if !md.flags & MENU_TAB != 0 {
                                        current_block = 18425699056680496821;
                                    } else {
                                        return 1 as core::ffi::c_int;
                                    }
                                }
                                4458768223536474289 => {
                                    if old == -(1 as core::ffi::c_int) {
                                        old = 0 as core::ffi::c_int;
                                    }
                                    loop {
                                        if md.choice == -(1 as core::ffi::c_int)
                                            || md.choice == 0 as core::ffi::c_int
                                        {
                                            md.choice = count - 1 as core::ffi::c_int;
                                        } else {
                                            md.choice -= 1;
                                        }
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                        if !((!selectable) && md.choice != old) {
                                            break;
                                        }
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    return 0 as core::ffi::c_int;
                                }
                                15099294739746082289 => return 1 as core::ffi::c_int,
                                _ => {}
                            }
                            return match current_block {
                                18425699056680496821 => 0 as core::ffi::c_int,
                                _ => {
                                    if old == -(1 as core::ffi::c_int) {
                                        old = 0 as core::ffi::c_int;
                                    }
                                    loop {
                                        if md.choice == -(1 as core::ffi::c_int)
                                            || md.choice == count - 1 as core::ffi::c_int
                                        {
                                            md.choice = 0 as core::ffi::c_int;
                                        } else {
                                            md.choice += 1;
                                        }
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                        if !((!selectable) && md.choice != old) {
                                            break;
                                        }
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    0 as core::ffi::c_int
                                }
                            };
                        }
                        8589934599 => {
                            current_block = 5589979562539812337;
                            match current_block {
                                9508525478627385121 => {
                                    md.choice = count - 1 as core::ffi::c_int;
                                    selectable = menu.items[md.choice as usize].is_selectable();
                                    while !selectable {
                                        md.choice -= 1;
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    current_block = 18425699056680496821;
                                }
                                10434433275429566483 => {
                                    md.choice = 0 as core::ffi::c_int;
                                    selectable = menu.items[md.choice as usize].is_selectable();
                                    while !selectable {
                                        md.choice += 1;
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    current_block = 18425699056680496821;
                                }
                                6983216613468852725 => {
                                    if md.choice > count - 6 as core::ffi::c_int {
                                        md.choice = count - 1 as core::ffi::c_int;
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                    } else {
                                        i = 5 as u_int;
                                        while i > 0 as u_int {
                                            md.choice += 1;
                                            selectable =
                                                menu.items[md.choice as usize].is_selectable();
                                            if md.choice != count - 1 as core::ffi::c_int
                                                && (selectable)
                                            {
                                                i = i.wrapping_sub(1);
                                            } else if md.choice == count - 1 as core::ffi::c_int {
                                                break;
                                            }
                                        }
                                    }
                                    while !selectable {
                                        md.choice -= 1;
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    current_block = 18425699056680496821;
                                }
                                4604734892166874824 => {
                                    if md.choice < 6 as core::ffi::c_int {
                                        md.choice = 0 as core::ffi::c_int;
                                    } else {
                                        i = 5 as u_int;
                                        while i > 0 as u_int {
                                            md.choice -= 1;
                                            selectable =
                                                menu.items[md.choice as usize].is_selectable();
                                            if md.choice != 0 as core::ffi::c_int && (selectable) {
                                                i = i.wrapping_sub(1);
                                            } else if md.choice == 0 as core::ffi::c_int {
                                                break;
                                            }
                                        }
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    current_block = 18425699056680496821;
                                }
                                5386196988030698307 => {
                                    if !md.flags & MENU_TAB != 0 {
                                        current_block = 18425699056680496821;
                                    } else {
                                        if md.choice == count - 1 as core::ffi::c_int {
                                            return 1 as core::ffi::c_int;
                                        }
                                        current_block = 16350834992246794677;
                                    }
                                }
                                5589979562539812337 => {
                                    if !md.flags & MENU_TAB != 0 {
                                        current_block = 18425699056680496821;
                                    } else {
                                        return 1 as core::ffi::c_int;
                                    }
                                }
                                4458768223536474289 => {
                                    if old == -(1 as core::ffi::c_int) {
                                        old = 0 as core::ffi::c_int;
                                    }
                                    loop {
                                        if md.choice == -(1 as core::ffi::c_int)
                                            || md.choice == 0 as core::ffi::c_int
                                        {
                                            md.choice = count - 1 as core::ffi::c_int;
                                        } else {
                                            md.choice -= 1;
                                        }
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                        if !((!selectable) && md.choice != old) {
                                            break;
                                        }
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    return 0 as core::ffi::c_int;
                                }
                                15099294739746082289 => return 1 as core::ffi::c_int,
                                _ => {}
                            }
                            return match current_block {
                                18425699056680496821 => 0 as core::ffi::c_int,
                                _ => {
                                    if old == -(1 as core::ffi::c_int) {
                                        old = 0 as core::ffi::c_int;
                                    }
                                    loop {
                                        if md.choice == -(1 as core::ffi::c_int)
                                            || md.choice == count - 1 as core::ffi::c_int
                                        {
                                            md.choice = 0 as core::ffi::c_int;
                                        } else {
                                            md.choice += 1;
                                        }
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                        if !((!selectable) && md.choice != old) {
                                            break;
                                        }
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    0 as core::ffi::c_int
                                }
                            };
                        }
                        9 => {
                            current_block = 5386196988030698307;
                            match current_block {
                                9508525478627385121 => {
                                    md.choice = count - 1 as core::ffi::c_int;
                                    selectable = menu.items[md.choice as usize].is_selectable();
                                    while !selectable {
                                        md.choice -= 1;
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    current_block = 18425699056680496821;
                                }
                                10434433275429566483 => {
                                    md.choice = 0 as core::ffi::c_int;
                                    selectable = menu.items[md.choice as usize].is_selectable();
                                    while !selectable {
                                        md.choice += 1;
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    current_block = 18425699056680496821;
                                }
                                6983216613468852725 => {
                                    if md.choice > count - 6 as core::ffi::c_int {
                                        md.choice = count - 1 as core::ffi::c_int;
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                    } else {
                                        i = 5 as u_int;
                                        while i > 0 as u_int {
                                            md.choice += 1;
                                            selectable =
                                                menu.items[md.choice as usize].is_selectable();
                                            if md.choice != count - 1 as core::ffi::c_int
                                                && (selectable)
                                            {
                                                i = i.wrapping_sub(1);
                                            } else if md.choice == count - 1 as core::ffi::c_int {
                                                break;
                                            }
                                        }
                                    }
                                    while !selectable {
                                        md.choice -= 1;
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    current_block = 18425699056680496821;
                                }
                                4604734892166874824 => {
                                    if md.choice < 6 as core::ffi::c_int {
                                        md.choice = 0 as core::ffi::c_int;
                                    } else {
                                        i = 5 as u_int;
                                        while i > 0 as u_int {
                                            md.choice -= 1;
                                            selectable =
                                                menu.items[md.choice as usize].is_selectable();
                                            if md.choice != 0 as core::ffi::c_int && (selectable) {
                                                i = i.wrapping_sub(1);
                                            } else if md.choice == 0 as core::ffi::c_int {
                                                break;
                                            }
                                        }
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    current_block = 18425699056680496821;
                                }
                                5386196988030698307 => {
                                    if !md.flags & MENU_TAB != 0 {
                                        current_block = 18425699056680496821;
                                    } else {
                                        if md.choice == count - 1 as core::ffi::c_int {
                                            return 1 as core::ffi::c_int;
                                        }
                                        current_block = 16350834992246794677;
                                    }
                                }
                                5589979562539812337 => {
                                    if !md.flags & MENU_TAB != 0 {
                                        current_block = 18425699056680496821;
                                    } else {
                                        return 1 as core::ffi::c_int;
                                    }
                                }
                                4458768223536474289 => {
                                    if old == -(1 as core::ffi::c_int) {
                                        old = 0 as core::ffi::c_int;
                                    }
                                    loop {
                                        if md.choice == -(1 as core::ffi::c_int)
                                            || md.choice == 0 as core::ffi::c_int
                                        {
                                            md.choice = count - 1 as core::ffi::c_int;
                                        } else {
                                            md.choice -= 1;
                                        }
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                        if !((!selectable) && md.choice != old) {
                                            break;
                                        }
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    return 0 as core::ffi::c_int;
                                }
                                15099294739746082289 => return 1 as core::ffi::c_int,
                                _ => {}
                            }
                            return match current_block {
                                18425699056680496821 => 0 as core::ffi::c_int,
                                _ => {
                                    if old == -(1 as core::ffi::c_int) {
                                        old = 0 as core::ffi::c_int;
                                    }
                                    loop {
                                        if md.choice == -(1 as core::ffi::c_int)
                                            || md.choice == count - 1 as core::ffi::c_int
                                        {
                                            md.choice = 0 as core::ffi::c_int;
                                        } else {
                                            md.choice += 1;
                                        }
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                        if !((!selectable) && md.choice != old) {
                                            break;
                                        }
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    0 as core::ffi::c_int
                                }
                            };
                        }
                        8589934620 | 106 => {
                            current_block = 16350834992246794677;
                            match current_block {
                                9508525478627385121 => {
                                    md.choice = count - 1 as core::ffi::c_int;
                                    selectable = menu.items[md.choice as usize].is_selectable();
                                    while !selectable {
                                        md.choice -= 1;
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    current_block = 18425699056680496821;
                                }
                                10434433275429566483 => {
                                    md.choice = 0 as core::ffi::c_int;
                                    selectable = menu.items[md.choice as usize].is_selectable();
                                    while !selectable {
                                        md.choice += 1;
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    current_block = 18425699056680496821;
                                }
                                6983216613468852725 => {
                                    if md.choice > count - 6 as core::ffi::c_int {
                                        md.choice = count - 1 as core::ffi::c_int;
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                    } else {
                                        i = 5 as u_int;
                                        while i > 0 as u_int {
                                            md.choice += 1;
                                            selectable =
                                                menu.items[md.choice as usize].is_selectable();
                                            if md.choice != count - 1 as core::ffi::c_int
                                                && (selectable)
                                            {
                                                i = i.wrapping_sub(1);
                                            } else if md.choice == count - 1 as core::ffi::c_int {
                                                break;
                                            }
                                        }
                                    }
                                    while !selectable {
                                        md.choice -= 1;
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    current_block = 18425699056680496821;
                                }
                                4604734892166874824 => {
                                    if md.choice < 6 as core::ffi::c_int {
                                        md.choice = 0 as core::ffi::c_int;
                                    } else {
                                        i = 5 as u_int;
                                        while i > 0 as u_int {
                                            md.choice -= 1;
                                            selectable =
                                                menu.items[md.choice as usize].is_selectable();
                                            if md.choice != 0 as core::ffi::c_int && (selectable) {
                                                i = i.wrapping_sub(1);
                                            } else if md.choice == 0 as core::ffi::c_int {
                                                break;
                                            }
                                        }
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    current_block = 18425699056680496821;
                                }
                                5386196988030698307 => {
                                    if !md.flags & MENU_TAB != 0 {
                                        current_block = 18425699056680496821;
                                    } else {
                                        if md.choice == count - 1 as core::ffi::c_int {
                                            return 1 as core::ffi::c_int;
                                        }
                                        current_block = 16350834992246794677;
                                    }
                                }
                                5589979562539812337 => {
                                    if !md.flags & MENU_TAB != 0 {
                                        current_block = 18425699056680496821;
                                    } else {
                                        return 1 as core::ffi::c_int;
                                    }
                                }
                                4458768223536474289 => {
                                    if old == -(1 as core::ffi::c_int) {
                                        old = 0 as core::ffi::c_int;
                                    }
                                    loop {
                                        if md.choice == -(1 as core::ffi::c_int)
                                            || md.choice == 0 as core::ffi::c_int
                                        {
                                            md.choice = count - 1 as core::ffi::c_int;
                                        } else {
                                            md.choice -= 1;
                                        }
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                        if !((!selectable) && md.choice != old) {
                                            break;
                                        }
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    return 0 as core::ffi::c_int;
                                }
                                15099294739746082289 => return 1 as core::ffi::c_int,
                                _ => {}
                            }
                            return match current_block {
                                18425699056680496821 => 0 as core::ffi::c_int,
                                _ => {
                                    if old == -(1 as core::ffi::c_int) {
                                        old = 0 as core::ffi::c_int;
                                    }
                                    loop {
                                        if md.choice == -(1 as core::ffi::c_int)
                                            || md.choice == count - 1 as core::ffi::c_int
                                        {
                                            md.choice = 0 as core::ffi::c_int;
                                        } else {
                                            md.choice += 1;
                                        }
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                        if !((!selectable) && md.choice != old) {
                                            break;
                                        }
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    0 as core::ffi::c_int
                                }
                            };
                        }
                        8589934617 | 35184372088930 => {
                            current_block = 4604734892166874824;
                            match current_block {
                                9508525478627385121 => {
                                    md.choice = count - 1 as core::ffi::c_int;
                                    selectable = menu.items[md.choice as usize].is_selectable();
                                    while !selectable {
                                        md.choice -= 1;
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    current_block = 18425699056680496821;
                                }
                                10434433275429566483 => {
                                    md.choice = 0 as core::ffi::c_int;
                                    selectable = menu.items[md.choice as usize].is_selectable();
                                    while !selectable {
                                        md.choice += 1;
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    current_block = 18425699056680496821;
                                }
                                6983216613468852725 => {
                                    if md.choice > count - 6 as core::ffi::c_int {
                                        md.choice = count - 1 as core::ffi::c_int;
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                    } else {
                                        i = 5 as u_int;
                                        while i > 0 as u_int {
                                            md.choice += 1;
                                            selectable =
                                                menu.items[md.choice as usize].is_selectable();
                                            if md.choice != count - 1 as core::ffi::c_int
                                                && (selectable)
                                            {
                                                i = i.wrapping_sub(1);
                                            } else if md.choice == count - 1 as core::ffi::c_int {
                                                break;
                                            }
                                        }
                                    }
                                    while !selectable {
                                        md.choice -= 1;
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    current_block = 18425699056680496821;
                                }
                                4604734892166874824 => {
                                    if md.choice < 6 as core::ffi::c_int {
                                        md.choice = 0 as core::ffi::c_int;
                                    } else {
                                        i = 5 as u_int;
                                        while i > 0 as u_int {
                                            md.choice -= 1;
                                            selectable =
                                                menu.items[md.choice as usize].is_selectable();
                                            if md.choice != 0 as core::ffi::c_int && (selectable) {
                                                i = i.wrapping_sub(1);
                                            } else if md.choice == 0 as core::ffi::c_int {
                                                break;
                                            }
                                        }
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    current_block = 18425699056680496821;
                                }
                                5386196988030698307 => {
                                    if !md.flags & MENU_TAB != 0 {
                                        current_block = 18425699056680496821;
                                    } else {
                                        if md.choice == count - 1 as core::ffi::c_int {
                                            return 1 as core::ffi::c_int;
                                        }
                                        current_block = 16350834992246794677;
                                    }
                                }
                                5589979562539812337 => {
                                    if !md.flags & MENU_TAB != 0 {
                                        current_block = 18425699056680496821;
                                    } else {
                                        return 1 as core::ffi::c_int;
                                    }
                                }
                                4458768223536474289 => {
                                    if old == -(1 as core::ffi::c_int) {
                                        old = 0 as core::ffi::c_int;
                                    }
                                    loop {
                                        if md.choice == -(1 as core::ffi::c_int)
                                            || md.choice == 0 as core::ffi::c_int
                                        {
                                            md.choice = count - 1 as core::ffi::c_int;
                                        } else {
                                            md.choice -= 1;
                                        }
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                        if !((!selectable) && md.choice != old) {
                                            break;
                                        }
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    return 0 as core::ffi::c_int;
                                }
                                15099294739746082289 => return 1 as core::ffi::c_int,
                                _ => {}
                            }
                            return match current_block {
                                18425699056680496821 => 0 as core::ffi::c_int,
                                _ => {
                                    if old == -(1 as core::ffi::c_int) {
                                        old = 0 as core::ffi::c_int;
                                    }
                                    loop {
                                        if md.choice == -(1 as core::ffi::c_int)
                                            || md.choice == count - 1 as core::ffi::c_int
                                        {
                                            md.choice = 0 as core::ffi::c_int;
                                        } else {
                                            md.choice += 1;
                                        }
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                        if !((!selectable) && md.choice != old) {
                                            break;
                                        }
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    0 as core::ffi::c_int
                                }
                            };
                        }
                        8589934616 => {
                            current_block = 6983216613468852725;
                            match current_block {
                                9508525478627385121 => {
                                    md.choice = count - 1 as core::ffi::c_int;
                                    selectable = menu.items[md.choice as usize].is_selectable();
                                    while !selectable {
                                        md.choice -= 1;
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    current_block = 18425699056680496821;
                                }
                                10434433275429566483 => {
                                    md.choice = 0 as core::ffi::c_int;
                                    selectable = menu.items[md.choice as usize].is_selectable();
                                    while !selectable {
                                        md.choice += 1;
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    current_block = 18425699056680496821;
                                }
                                6983216613468852725 => {
                                    if md.choice > count - 6 as core::ffi::c_int {
                                        md.choice = count - 1 as core::ffi::c_int;
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                    } else {
                                        i = 5 as u_int;
                                        while i > 0 as u_int {
                                            md.choice += 1;
                                            selectable =
                                                menu.items[md.choice as usize].is_selectable();
                                            if md.choice != count - 1 as core::ffi::c_int
                                                && (selectable)
                                            {
                                                i = i.wrapping_sub(1);
                                            } else if md.choice == count - 1 as core::ffi::c_int {
                                                break;
                                            }
                                        }
                                    }
                                    while !selectable {
                                        md.choice -= 1;
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    current_block = 18425699056680496821;
                                }
                                4604734892166874824 => {
                                    if md.choice < 6 as core::ffi::c_int {
                                        md.choice = 0 as core::ffi::c_int;
                                    } else {
                                        i = 5 as u_int;
                                        while i > 0 as u_int {
                                            md.choice -= 1;
                                            selectable =
                                                menu.items[md.choice as usize].is_selectable();
                                            if md.choice != 0 as core::ffi::c_int && (selectable) {
                                                i = i.wrapping_sub(1);
                                            } else if md.choice == 0 as core::ffi::c_int {
                                                break;
                                            }
                                        }
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    current_block = 18425699056680496821;
                                }
                                5386196988030698307 => {
                                    if !md.flags & MENU_TAB != 0 {
                                        current_block = 18425699056680496821;
                                    } else {
                                        if md.choice == count - 1 as core::ffi::c_int {
                                            return 1 as core::ffi::c_int;
                                        }
                                        current_block = 16350834992246794677;
                                    }
                                }
                                5589979562539812337 => {
                                    if !md.flags & MENU_TAB != 0 {
                                        current_block = 18425699056680496821;
                                    } else {
                                        return 1 as core::ffi::c_int;
                                    }
                                }
                                4458768223536474289 => {
                                    if old == -(1 as core::ffi::c_int) {
                                        old = 0 as core::ffi::c_int;
                                    }
                                    loop {
                                        if md.choice == -(1 as core::ffi::c_int)
                                            || md.choice == 0 as core::ffi::c_int
                                        {
                                            md.choice = count - 1 as core::ffi::c_int;
                                        } else {
                                            md.choice -= 1;
                                        }
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                        if !((!selectable) && md.choice != old) {
                                            break;
                                        }
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    return 0 as core::ffi::c_int;
                                }
                                15099294739746082289 => return 1 as core::ffi::c_int,
                                _ => {}
                            }
                            return match current_block {
                                18425699056680496821 => 0 as core::ffi::c_int,
                                _ => {
                                    if old == -(1 as core::ffi::c_int) {
                                        old = 0 as core::ffi::c_int;
                                    }
                                    loop {
                                        if md.choice == -(1 as core::ffi::c_int)
                                            || md.choice == count - 1 as core::ffi::c_int
                                        {
                                            md.choice = 0 as core::ffi::c_int;
                                        } else {
                                            md.choice += 1;
                                        }
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                        if !((!selectable) && md.choice != old) {
                                            break;
                                        }
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    0 as core::ffi::c_int
                                }
                            };
                        }
                        103 | 8589934614 => {
                            current_block = 10434433275429566483;
                            match current_block {
                                9508525478627385121 => {
                                    md.choice = count - 1 as core::ffi::c_int;
                                    selectable = menu.items[md.choice as usize].is_selectable();
                                    while !selectable {
                                        md.choice -= 1;
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    current_block = 18425699056680496821;
                                }
                                10434433275429566483 => {
                                    md.choice = 0 as core::ffi::c_int;
                                    selectable = menu.items[md.choice as usize].is_selectable();
                                    while !selectable {
                                        md.choice += 1;
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    current_block = 18425699056680496821;
                                }
                                6983216613468852725 => {
                                    if md.choice > count - 6 as core::ffi::c_int {
                                        md.choice = count - 1 as core::ffi::c_int;
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                    } else {
                                        i = 5 as u_int;
                                        while i > 0 as u_int {
                                            md.choice += 1;
                                            selectable =
                                                menu.items[md.choice as usize].is_selectable();
                                            if md.choice != count - 1 as core::ffi::c_int
                                                && (selectable)
                                            {
                                                i = i.wrapping_sub(1);
                                            } else if md.choice == count - 1 as core::ffi::c_int {
                                                break;
                                            }
                                        }
                                    }
                                    while !selectable {
                                        md.choice -= 1;
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    current_block = 18425699056680496821;
                                }
                                4604734892166874824 => {
                                    if md.choice < 6 as core::ffi::c_int {
                                        md.choice = 0 as core::ffi::c_int;
                                    } else {
                                        i = 5 as u_int;
                                        while i > 0 as u_int {
                                            md.choice -= 1;
                                            selectable =
                                                menu.items[md.choice as usize].is_selectable();
                                            if md.choice != 0 as core::ffi::c_int && (selectable) {
                                                i = i.wrapping_sub(1);
                                            } else if md.choice == 0 as core::ffi::c_int {
                                                break;
                                            }
                                        }
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    current_block = 18425699056680496821;
                                }
                                5386196988030698307 => {
                                    if !md.flags & MENU_TAB != 0 {
                                        current_block = 18425699056680496821;
                                    } else {
                                        if md.choice == count - 1 as core::ffi::c_int {
                                            return 1 as core::ffi::c_int;
                                        }
                                        current_block = 16350834992246794677;
                                    }
                                }
                                5589979562539812337 => {
                                    if !md.flags & MENU_TAB != 0 {
                                        current_block = 18425699056680496821;
                                    } else {
                                        return 1 as core::ffi::c_int;
                                    }
                                }
                                4458768223536474289 => {
                                    if old == -(1 as core::ffi::c_int) {
                                        old = 0 as core::ffi::c_int;
                                    }
                                    loop {
                                        if md.choice == -(1 as core::ffi::c_int)
                                            || md.choice == 0 as core::ffi::c_int
                                        {
                                            md.choice = count - 1 as core::ffi::c_int;
                                        } else {
                                            md.choice -= 1;
                                        }
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                        if !((!selectable) && md.choice != old) {
                                            break;
                                        }
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    return 0 as core::ffi::c_int;
                                }
                                15099294739746082289 => return 1 as core::ffi::c_int,
                                _ => {}
                            }
                            return match current_block {
                                18425699056680496821 => 0 as core::ffi::c_int,
                                _ => {
                                    if old == -(1 as core::ffi::c_int) {
                                        old = 0 as core::ffi::c_int;
                                    }
                                    loop {
                                        if md.choice == -(1 as core::ffi::c_int)
                                            || md.choice == count - 1 as core::ffi::c_int
                                        {
                                            md.choice = 0 as core::ffi::c_int;
                                        } else {
                                            md.choice += 1;
                                        }
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                        if !((!selectable) && md.choice != old) {
                                            break;
                                        }
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    0 as core::ffi::c_int
                                }
                            };
                        }
                        71 | 8589934615 => {
                            current_block = 9508525478627385121;
                            match current_block {
                                9508525478627385121 => {
                                    md.choice = count - 1 as core::ffi::c_int;
                                    selectable = menu.items[md.choice as usize].is_selectable();
                                    while !selectable {
                                        md.choice -= 1;
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    current_block = 18425699056680496821;
                                }
                                10434433275429566483 => {
                                    md.choice = 0 as core::ffi::c_int;
                                    selectable = menu.items[md.choice as usize].is_selectable();
                                    while !selectable {
                                        md.choice += 1;
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    current_block = 18425699056680496821;
                                }
                                6983216613468852725 => {
                                    if md.choice > count - 6 as core::ffi::c_int {
                                        md.choice = count - 1 as core::ffi::c_int;
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                    } else {
                                        i = 5 as u_int;
                                        while i > 0 as u_int {
                                            md.choice += 1;
                                            selectable =
                                                menu.items[md.choice as usize].is_selectable();
                                            if md.choice != count - 1 as core::ffi::c_int
                                                && (selectable)
                                            {
                                                i = i.wrapping_sub(1);
                                            } else if md.choice == count - 1 as core::ffi::c_int {
                                                break;
                                            }
                                        }
                                    }
                                    while !selectable {
                                        md.choice -= 1;
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    current_block = 18425699056680496821;
                                }
                                4604734892166874824 => {
                                    if md.choice < 6 as core::ffi::c_int {
                                        md.choice = 0 as core::ffi::c_int;
                                    } else {
                                        i = 5 as u_int;
                                        while i > 0 as u_int {
                                            md.choice -= 1;
                                            selectable =
                                                menu.items[md.choice as usize].is_selectable();
                                            if md.choice != 0 as core::ffi::c_int && (selectable) {
                                                i = i.wrapping_sub(1);
                                            } else if md.choice == 0 as core::ffi::c_int {
                                                break;
                                            }
                                        }
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    current_block = 18425699056680496821;
                                }
                                5386196988030698307 => {
                                    if !md.flags & MENU_TAB != 0 {
                                        current_block = 18425699056680496821;
                                    } else {
                                        if md.choice == count - 1 as core::ffi::c_int {
                                            return 1 as core::ffi::c_int;
                                        }
                                        current_block = 16350834992246794677;
                                    }
                                }
                                5589979562539812337 => {
                                    if !md.flags & MENU_TAB != 0 {
                                        current_block = 18425699056680496821;
                                    } else {
                                        return 1 as core::ffi::c_int;
                                    }
                                }
                                4458768223536474289 => {
                                    if old == -(1 as core::ffi::c_int) {
                                        old = 0 as core::ffi::c_int;
                                    }
                                    loop {
                                        if md.choice == -(1 as core::ffi::c_int)
                                            || md.choice == 0 as core::ffi::c_int
                                        {
                                            md.choice = count - 1 as core::ffi::c_int;
                                        } else {
                                            md.choice -= 1;
                                        }
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                        if !((!selectable) && md.choice != old) {
                                            break;
                                        }
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    return 0 as core::ffi::c_int;
                                }
                                15099294739746082289 => return 1 as core::ffi::c_int,
                                _ => {}
                            }
                            return match current_block {
                                18425699056680496821 => 0 as core::ffi::c_int,
                                _ => {
                                    if old == -(1 as core::ffi::c_int) {
                                        old = 0 as core::ffi::c_int;
                                    }
                                    loop {
                                        if md.choice == -(1 as core::ffi::c_int)
                                            || md.choice == count - 1 as core::ffi::c_int
                                        {
                                            md.choice = 0 as core::ffi::c_int;
                                        } else {
                                            md.choice += 1;
                                        }
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                        if !((!selectable) && md.choice != old) {
                                            break;
                                        }
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    0 as core::ffi::c_int
                                }
                            };
                        }
                        13 => {}
                        27 | 35184372088923 | 35184372088931 | 35184372088935 | 113 => {
                            current_block = 15099294739746082289;
                            match current_block {
                                9508525478627385121 => {
                                    md.choice = count - 1 as core::ffi::c_int;
                                    selectable = menu.items[md.choice as usize].is_selectable();
                                    while !selectable {
                                        md.choice -= 1;
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    current_block = 18425699056680496821;
                                }
                                10434433275429566483 => {
                                    md.choice = 0 as core::ffi::c_int;
                                    selectable = menu.items[md.choice as usize].is_selectable();
                                    while !selectable {
                                        md.choice += 1;
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    current_block = 18425699056680496821;
                                }
                                6983216613468852725 => {
                                    if md.choice > count - 6 as core::ffi::c_int {
                                        md.choice = count - 1 as core::ffi::c_int;
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                    } else {
                                        i = 5 as u_int;
                                        while i > 0 as u_int {
                                            md.choice += 1;
                                            selectable =
                                                menu.items[md.choice as usize].is_selectable();
                                            if md.choice != count - 1 as core::ffi::c_int
                                                && (selectable)
                                            {
                                                i = i.wrapping_sub(1);
                                            } else if md.choice == count - 1 as core::ffi::c_int {
                                                break;
                                            }
                                        }
                                    }
                                    while !selectable {
                                        md.choice -= 1;
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    current_block = 18425699056680496821;
                                }
                                4604734892166874824 => {
                                    if md.choice < 6 as core::ffi::c_int {
                                        md.choice = 0 as core::ffi::c_int;
                                    } else {
                                        i = 5 as u_int;
                                        while i > 0 as u_int {
                                            md.choice -= 1;
                                            selectable =
                                                menu.items[md.choice as usize].is_selectable();
                                            if md.choice != 0 as core::ffi::c_int && (selectable) {
                                                i = i.wrapping_sub(1);
                                            } else if md.choice == 0 as core::ffi::c_int {
                                                break;
                                            }
                                        }
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    current_block = 18425699056680496821;
                                }
                                5386196988030698307 => {
                                    if !md.flags & MENU_TAB != 0 {
                                        current_block = 18425699056680496821;
                                    } else {
                                        if md.choice == count - 1 as core::ffi::c_int {
                                            return 1 as core::ffi::c_int;
                                        }
                                        current_block = 16350834992246794677;
                                    }
                                }
                                5589979562539812337 => {
                                    if !md.flags & MENU_TAB != 0 {
                                        current_block = 18425699056680496821;
                                    } else {
                                        return 1 as core::ffi::c_int;
                                    }
                                }
                                4458768223536474289 => {
                                    if old == -(1 as core::ffi::c_int) {
                                        old = 0 as core::ffi::c_int;
                                    }
                                    loop {
                                        if md.choice == -(1 as core::ffi::c_int)
                                            || md.choice == 0 as core::ffi::c_int
                                        {
                                            md.choice = count - 1 as core::ffi::c_int;
                                        } else {
                                            md.choice -= 1;
                                        }
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                        if !((!selectable) && md.choice != old) {
                                            break;
                                        }
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    return 0 as core::ffi::c_int;
                                }
                                15099294739746082289 => return 1 as core::ffi::c_int,
                                _ => {}
                            }
                            return match current_block {
                                18425699056680496821 => 0 as core::ffi::c_int,
                                _ => {
                                    if old == -(1 as core::ffi::c_int) {
                                        old = 0 as core::ffi::c_int;
                                    }
                                    loop {
                                        if md.choice == -(1 as core::ffi::c_int)
                                            || md.choice == count - 1 as core::ffi::c_int
                                        {
                                            md.choice = 0 as core::ffi::c_int;
                                        } else {
                                            md.choice += 1;
                                        }
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                        if !((!selectable) && md.choice != old) {
                                            break;
                                        }
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    0 as core::ffi::c_int
                                }
                            };
                        }
                        _ => {
                            current_block = 18425699056680496821;
                            match current_block {
                                9508525478627385121 => {
                                    md.choice = count - 1 as core::ffi::c_int;
                                    selectable = menu.items[md.choice as usize].is_selectable();
                                    while !selectable {
                                        md.choice -= 1;
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    current_block = 18425699056680496821;
                                }
                                10434433275429566483 => {
                                    md.choice = 0 as core::ffi::c_int;
                                    selectable = menu.items[md.choice as usize].is_selectable();
                                    while !selectable {
                                        md.choice += 1;
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    current_block = 18425699056680496821;
                                }
                                6983216613468852725 => {
                                    if md.choice > count - 6 as core::ffi::c_int {
                                        md.choice = count - 1 as core::ffi::c_int;
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                    } else {
                                        i = 5 as u_int;
                                        while i > 0 as u_int {
                                            md.choice += 1;
                                            selectable =
                                                menu.items[md.choice as usize].is_selectable();
                                            if md.choice != count - 1 as core::ffi::c_int
                                                && (selectable)
                                            {
                                                i = i.wrapping_sub(1);
                                            } else if md.choice == count - 1 as core::ffi::c_int {
                                                break;
                                            }
                                        }
                                    }
                                    while !selectable {
                                        md.choice -= 1;
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    current_block = 18425699056680496821;
                                }
                                4604734892166874824 => {
                                    if md.choice < 6 as core::ffi::c_int {
                                        md.choice = 0 as core::ffi::c_int;
                                    } else {
                                        i = 5 as u_int;
                                        while i > 0 as u_int {
                                            md.choice -= 1;
                                            selectable =
                                                menu.items[md.choice as usize].is_selectable();
                                            if md.choice != 0 as core::ffi::c_int && (selectable) {
                                                i = i.wrapping_sub(1);
                                            } else if md.choice == 0 as core::ffi::c_int {
                                                break;
                                            }
                                        }
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    current_block = 18425699056680496821;
                                }
                                5386196988030698307 => {
                                    if !md.flags & MENU_TAB != 0 {
                                        current_block = 18425699056680496821;
                                    } else {
                                        if md.choice == count - 1 as core::ffi::c_int {
                                            return 1 as core::ffi::c_int;
                                        }
                                        current_block = 16350834992246794677;
                                    }
                                }
                                5589979562539812337 => {
                                    if !md.flags & MENU_TAB != 0 {
                                        current_block = 18425699056680496821;
                                    } else {
                                        return 1 as core::ffi::c_int;
                                    }
                                }
                                4458768223536474289 => {
                                    if old == -(1 as core::ffi::c_int) {
                                        old = 0 as core::ffi::c_int;
                                    }
                                    loop {
                                        if md.choice == -(1 as core::ffi::c_int)
                                            || md.choice == 0 as core::ffi::c_int
                                        {
                                            md.choice = count - 1 as core::ffi::c_int;
                                        } else {
                                            md.choice -= 1;
                                        }
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                        if !((!selectable) && md.choice != old) {
                                            break;
                                        }
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    return 0 as core::ffi::c_int;
                                }
                                15099294739746082289 => return 1 as core::ffi::c_int,
                                _ => {}
                            }
                            return match current_block {
                                18425699056680496821 => 0 as core::ffi::c_int,
                                _ => {
                                    if old == -(1 as core::ffi::c_int) {
                                        old = 0 as core::ffi::c_int;
                                    }
                                    loop {
                                        if md.choice == -(1 as core::ffi::c_int)
                                            || md.choice == count - 1 as core::ffi::c_int
                                        {
                                            md.choice = 0 as core::ffi::c_int;
                                        } else {
                                            md.choice += 1;
                                        }
                                        selectable = menu.items[md.choice as usize].is_selectable();
                                        if !((!selectable) && md.choice != old) {
                                            break;
                                        }
                                    }
                                    c.flags |= CLIENT_REDRAWOVERLAY as uint64_t;
                                    0 as core::ffi::c_int
                                }
                            };
                        }
                    },
                }
            }
            if md.choice == -(1 as core::ffi::c_int) {
                return 1 as core::ffi::c_int;
            }
            let item = &menu.items[md.choice as usize];
            if !item.is_selectable() {
                if md.flags & MENU_STAYOPEN != 0 {
                    return 0 as core::ffi::c_int;
                }
                return 1 as core::ffi::c_int;
            }
            if let Some(cb) = md.cb.take() {
                let choice = md.choice as u_int;
                let key = item.key;
                drop(guard);
                cb(choice, key);
                return 1 as core::ffi::c_int;
            }
            let queue_item = md.item.as_ref().and_then(CmdqItemWeak::upgrade);
            let queue_event = queue_item.as_ref().map(|item| item.event_snapshot());
            let state =
                CmdqStateRef::create(Some(&md.fs), queue_event.as_ref(), 0 as core::ffi::c_int);
            let command = item.command().unwrap_or(c"").to_owned();
            drop(guard);
            cmd_parse_and_append(
                &command,
                None,
                crate::server::client_ref_of(c).as_ref(),
                &state,
                &mut error,
            );
            if let Some(error) = error.as_ref() {
                cmdq_append(
                    crate::server::client_ref_of(c).as_ref(),
                    CmdqItemRef::error_items(error),
                );
            }
            1 as core::ffi::c_int
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
pub(crate) unsafe fn menu_display_for_client(
    menu: Box<menu>,
    flags: core::ffi::c_int,
    starting_choice: core::ffi::c_int,
    item: Option<&cmdq_item>,
    px: u_int,
    py: u_int,
    c: &mut ClientRef,
    lines: box_lines,
    style: Option<&CStr>,
    selected_style: Option<&CStr>,
    border_style: Option<&CStr>,
    fs: Option<&cmd_find_state>,
    cb: menu_choice_cb,
) -> core::ffi::c_int {
    unsafe {
        menu_display(
            menu,
            flags,
            starting_choice,
            item,
            px,
            py,
            c.as_client_mut(),
            lines,
            style,
            selected_style,
            border_style,
            fs,
            cb,
        )
    }
}

#[cfg(test)]
pub use crate::consts::{
    BOX_LINES_DOUBLE, BOX_LINES_HEAVY, BOX_LINES_PADDED, BOX_LINES_ROUNDED, BOX_LINES_SIMPLE,
    BOX_LINES_SINGLE, KEYC_BTAB, KEYC_CTRL, KEYC_DOWN, KEYC_END, KEYC_HOME, KEYC_NPAGE, KEYC_PPAGE,
    KEYC_UP,
};
