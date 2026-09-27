use crate::src::arguments::{args_get, args_has};
use crate::src::cmd::parse::{cmd_parse_and_append, cmd_parse_error_uppercase_first};
use crate::src::cmd::queue::{
    cmdq_append, cmdq_get_callback_owned, cmdq_get_client, cmdq_new_state,
};
use crate::src::cmd::{cmd_mouse_at, cmd_template_replace_cstring};
use crate::src::ffi::libc::{
    __ctype_tolower_loc, __ctype_toupper_loc, memcpy, memset, strcasestr, strlen, strstr,
};
use crate::src::format::bytes::write_cstr;
use crate::src::format::{format_add, format_create_defaults, format_expand_cstring, format_free};
use crate::src::format_draw::{format_draw, format_width};
use crate::src::grid::grid_default_cell;
use crate::src::key_string::key_string_format;
use crate::src::log::{log_bytes, log_debug};
use crate::src::menu::{menu_add_items, menu_create, menu_display};
use crate::src::options::options_get_number;
use crate::src::prompt::{
    prompt_closed, prompt_create, prompt_draw, prompt_free, prompt_key, prompt_mouse,
    prompt_set_options,
};
use crate::src::screen::{screen_free, screen_init, screen_resize};
use crate::src::screen_write::{
    screen_write_box, screen_write_clearcharacter, screen_write_clearendofline,
    screen_write_clearscreen, screen_write_cursormove, screen_write_puts, screen_write_start,
    screen_write_stop,
};
use crate::src::server_fn::{server_redraw_window, server_unzoom_window};
use crate::src::shared::abi::__int32_t;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::args;
use crate::src::shared::client::{client, CLIENT_DEAD};
use crate::src::shared::colour::{COLOUR_FLAG_THEME, COLOUR_THEME_CYAN};
use crate::src::shared::command::cmd_parse_input;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd_find_state, cmdq_item};
use crate::src::shared::display::*;
use crate::src::shared::format::format_tree;
use crate::src::shared::grid::*;
use crate::src::shared::key::key_event;
use crate::src::shared::key::*;
use crate::src::shared::layout::*;
use crate::src::shared::menu::{menu_choice_cb, menu_item, MenuSelection};
use crate::src::shared::mode_tree::{
    mode_tree_build_cb, mode_tree_data, mode_tree_draw_cb, mode_tree_height_cb, mode_tree_help_cb,
    mode_tree_help_info, mode_tree_item, mode_tree_key_cb, mode_tree_line, mode_tree_list,
    mode_tree_menu_cb, mode_tree_prompt, mode_tree_prompt_input_cb, mode_tree_search_cb,
    mode_tree_search_dir, mode_tree_sort_cb, mode_tree_swap_cb, ModeTreeItemData, ModeTreeItemRef,
    ModeTreePromptRef,
};
use crate::src::shared::mouse::{mouse_event, MOUSE_BUTTON_1, MOUSE_MASK_BUTTONS, MOUSE_MASK_DRAG};
use crate::src::shared::options::options;
use crate::src::shared::pane::window_pane;
use crate::src::shared::pane::PANE_REDRAW;
use crate::src::shared::prompt::prompt;
use crate::src::shared::prompt::*;
use crate::src::shared::prompt::{prompt_create_data, prompt_draw_data};
use crate::src::shared::prompt::{
    prompt_free_cb, prompt_result, PROMPT_ACCEPT, PROMPT_CLOSE, PROMPT_CONTINUE, PROMPT_ISMODE,
    PROMPT_NOFORMAT, PROMPT_SINGLE,
};
use crate::src::shared::screen::{screen, MODE_CURSOR};
use crate::src::shared::screen_write::screen_write_ctx;
use crate::src::shared::session::session;
use crate::src::shared::sort::sort_criteria;
use crate::src::shared::style::*;
use crate::src::shared::window::WINDOW_ZOOMED;
use crate::src::shared::window::{window, winlink};
use crate::src::sort::{sort_next_order, sort_order_from_string, sort_order_to_string};
use crate::src::status::status_message_set;
use crate::src::style::style_apply;
use crate::src::tmux::global_s_options;
use crate::src::window::window_zoom;
use std::cell::{RefCell, UnsafeCell};
use std::ffi::{CStr, CString};
use std::rc::{Rc, Weak};

pub const MODE_TREE_SEARCH_BACKWARD: mode_tree_search_dir = 1;
pub const MODE_TREE_SEARCH_FORWARD: mode_tree_search_dir = 0;

impl mode_tree_item {
    fn set_keystr(&mut self, keystr: Option<CString>) {
        self.keystr = keystr;

        self.keylen = self.keystr.as_ref().map_or(0, |s| s.as_bytes().len());
    }
}
pub type mode_tree_preview = ::core::ffi::c_uint;
pub const MODE_TREE_PREVIEW_BIG: mode_tree_preview = 2;
pub const MODE_TREE_PREVIEW_NORMAL: mode_tree_preview = 1;
pub const MODE_TREE_PREVIEW_OFF: mode_tree_preview = 0;
#[inline]
unsafe fn tolower(mut __c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    return if __c >= -(128 as ::core::ffi::c_int) && __c < 256 as ::core::ffi::c_int {
        *(*__ctype_tolower_loc()).offset(__c as isize) as ::core::ffi::c_int
    } else {
        __c
    };
}
pub const UINT64_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
static mode_tree_menu_items: [menu_item<'static>; 4] = [
    menu_item {
        name: c"Scroll Left",
        key: '<' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: c"Scroll Right",
        key: '>' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: c"",
        key: KEYC_NONE as ::core::ffi::c_ulong as key_code,
        command: None,
    },
    menu_item {
        name: c"Cancel",
        key: 'q' as i32 as key_code,
        command: None,
    },
];
static mode_tree_help_start: &[&'static CStr] = &[
    c"#[fg=themelightgrey]      Up, k #[#{E:tree-mode-border-style},acs]x#[default] Move cursor up",
    c"#[fg=themelightgrey]    Down, j #[#{E:tree-mode-border-style},acs]x#[default] Move cursor down",
    c"#[fg=themelightgrey]          g #[#{E:tree-mode-border-style},acs]x#[default] Go to top",
    c"#[fg=themelightgrey]          G #[#{E:tree-mode-border-style},acs]x#[default] Go to bottom",
    c"#[fg=themelightgrey] PPage, C-b #[#{E:tree-mode-border-style},acs]x#[default] Page up",
    c"#[fg=themelightgrey] NPage, C-f #[#{E:tree-mode-border-style},acs]x#[default] Page down",
    c"#[fg=themelightgrey]    Left, h #[#{E:tree-mode-border-style},acs]x#[default] Collapse %1",
    c"#[fg=themelightgrey]   Right, l #[#{E:tree-mode-border-style},acs]x#[default] Expand %1",
    c"#[fg=themelightgrey]        M-- #[#{E:tree-mode-border-style},acs]x#[default] Collapse all %1s",
    c"#[fg=themelightgrey]        M-+ #[#{E:tree-mode-border-style},acs]x#[default] Expand all %1s",
    c"#[fg=themelightgrey]          t #[#{E:tree-mode-border-style},acs]x#[default] Toggle %1 tag",
    c"#[fg=themelightgrey]          T #[#{E:tree-mode-border-style},acs]x#[default] Untag all %1s",
    c"#[fg=themelightgrey]        C-t #[#{E:tree-mode-border-style},acs]x#[default] Tag all %1s",
    c"#[fg=themelightgrey]        C-s #[#{E:tree-mode-border-style},acs]x#[default] Search forward",
    c"#[fg=themelightgrey]          n #[#{E:tree-mode-border-style},acs]x#[default] Repeat search forward",
    c"#[fg=themelightgrey]          N #[#{E:tree-mode-border-style},acs]x#[default] Repeat search backward",
    c"#[fg=themelightgrey]          f #[#{E:tree-mode-border-style},acs]x#[default] Filter %1s",
    c"#[fg=themelightgrey]          O #[#{E:tree-mode-border-style},acs]x#[default] Change sort order",
    c"#[fg=themelightgrey]          r #[#{E:tree-mode-border-style},acs]x#[default] Reverse sort order",
    c"#[fg=themelightgrey]          v #[#{E:tree-mode-border-style},acs]x#[default] Toggle preview",
];
static mode_tree_help_end: &[&'static CStr] =
    &[c"#[fg=themelightgrey]  q, Escape #[#{E:tree-mode-border-style},acs]x#[default] Exit mode"];
pub const MODE_TREE_HELP_DEFAULT_WIDTH: ::core::ffi::c_int = 39 as ::core::ffi::c_int;
unsafe fn mode_tree_is_lowercase(mut ptr: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    while *ptr as ::core::ffi::c_int != '\0' as i32 {
        if *ptr as ::core::ffi::c_int
            != ({
                let mut __res: ::core::ffi::c_int = 0;
                if ::core::mem::size_of::<u_char>() as usize > 1 as usize {
                    if 0 != 0 {
                        let mut __c: ::core::ffi::c_int = *ptr as u_char as ::core::ffi::c_int;
                        __res = (if __c < -(128 as ::core::ffi::c_int)
                            || __c > 255 as ::core::ffi::c_int
                        {
                            __c as __int32_t
                        } else {
                            *(*__ctype_tolower_loc()).offset(__c as isize)
                        }) as ::core::ffi::c_int;
                    } else {
                        __res = tolower(*ptr as u_char as ::core::ffi::c_int);
                    }
                } else {
                    __res = *(*__ctype_tolower_loc())
                        .offset(*ptr as u_char as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int;
                }
                __res
            })
        {
            return 0 as ::core::ffi::c_int;
        }
        ptr = ptr.offset(1);
    }
    return 1 as ::core::ffi::c_int;
}
fn mode_tree_find_item(list: &mode_tree_list, tag: uint64_t) -> Option<ModeTreeItemRef> {
    for item in &list.items {
        let row = item.borrow();
        if row.tag == tag {
            return Some(Rc::clone(item));
        }
        if let Some(child) = mode_tree_find_item(&row.children, tag) {
            return Some(child);
        }
    }
    None
}
fn mode_tree_free_items(list: &mut mode_tree_list) {
    list.items.clear();
}

fn mode_tree_sibling(
    mtd: &mode_tree_data,
    item: &ModeTreeItemRef,
    forward: bool,
) -> Option<ModeTreeItemRef> {
    let parent = item.borrow().parent.upgrade();
    let sibling = |list: &mode_tree_list| {
        if forward {
            list.next(item)
        } else {
            list.previous(item)
        }
    };
    if let Some(parent) = parent {
        sibling(&parent.borrow().children)
    } else {
        sibling(&mtd.children)
    }
}

unsafe fn mode_tree_check_selected(mut mtd: *mut mode_tree_data) {
    if (*mtd).current > (*mtd).height.wrapping_sub(1 as u_int) {
        (*mtd).offset = (*mtd)
            .current
            .wrapping_sub((*mtd).height)
            .wrapping_add(1 as u_int);
    }
}
unsafe fn mode_tree_alloc_data() -> *mut mode_tree_data {
    crate::src::shared::rc::new(mode_tree_data::default())
}

#[inline]
unsafe fn mode_tree_line_count(mtd: &mode_tree_data) -> u_int {
    mtd.lines.len() as u_int
}

unsafe fn mode_tree_clear_lines(mut mtd: *mut mode_tree_data) {
    (*mtd).lines = Vec::new();
}
unsafe fn mode_tree_build_lines(mtd: *mut mode_tree_data, items: &[ModeTreeItemRef], depth: u_int) {
    (*mtd).depth = depth;
    (*mtd).maxdepth = (*mtd).maxdepth.max(depth);
    let mut flat = 1;
    for (index, item) in items.iter().enumerate() {
        let line = (*mtd).lines.len() as u_int;
        (*mtd).lines.push(mode_tree_line {
            item: Rc::clone(item),
            depth,
            last: (index + 1 == items.len()) as i32,
            flat: 0,
        });
        let (children, expanded) = {
            let mut row = item.borrow_mut();
            row.line = line;
            if !row.children.items.is_empty() {
                flat = 0;
            }
            (row.children.items.clone(), row.expanded)
        };
        if expanded != 0 {
            mode_tree_build_lines(mtd, &children, depth.wrapping_add(1));
        }
        let key = if let Some(callback) = (*mtd).keycb.as_mut() {
            let data = item.borrow().itemdata.clone();
            let key = callback(&data, line);
            if key == KEYC_UNKNOWN {
                KEYC_NONE
            } else {
                key
            }
        } else if line < 10 {
            (b'0' as u_int).wrapping_add(line) as key_code
        } else if line < 36 {
            KEYC_META | (b'a' as u_int).wrapping_add(line).wrapping_sub(10) as key_code
        } else {
            KEYC_NONE
        };
        let keystr = (key != KEYC_NONE).then(|| key_string_format(key, false));
        let mut row = item.borrow_mut();
        row.key = key;
        row.set_keystr(keystr);
    }
    for item in items {
        let index = item.borrow().line as usize;
        (&mut (*mtd).lines)[index].flat = flat;
    }
}
fn mode_tree_clear_tagged(list: &mode_tree_list) {
    for item in &list.items {
        let mut row = item.borrow_mut();
        row.tagged = 0;
        mode_tree_clear_tagged(&row.children);
    }
}
pub unsafe fn mode_tree_up(mut mtd: *mut mode_tree_data, mut wrap: ::core::ffi::c_int) {
    if mode_tree_line_count(&*mtd) == 0 as u_int {
        return;
    }
    if (*mtd).current == 0 as u_int {
        if wrap != 0 {
            (*mtd).current = mode_tree_line_count(&*mtd).wrapping_sub(1 as u_int);
            if mode_tree_line_count(&*mtd) >= (*mtd).height {
                (*mtd).offset = mode_tree_line_count(&*mtd).wrapping_sub((*mtd).height);
            }
        }
    } else {
        (*mtd).current = (*mtd).current.wrapping_sub(1);
        if (*mtd).current < (*mtd).offset {
            (*mtd).offset = (*mtd).offset.wrapping_sub(1);
        }
    };
}
pub unsafe fn mode_tree_down(
    mut mtd: *mut mode_tree_data,
    mut wrap: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if mode_tree_line_count(&*mtd) == 0 as u_int {
        return 0 as ::core::ffi::c_int;
    }
    if (*mtd).current == mode_tree_line_count(&*mtd).wrapping_sub(1 as u_int) {
        if wrap != 0 {
            (*mtd).current = 0 as u_int;
            (*mtd).offset = 0 as u_int;
        } else {
            return 0 as ::core::ffi::c_int;
        }
    } else {
        (*mtd).current = (*mtd).current.wrapping_add(1);
        if (*mtd).current
            > (*mtd)
                .offset
                .wrapping_add((*mtd).height)
                .wrapping_sub(1 as u_int)
        {
            (*mtd).offset = (*mtd).offset.wrapping_add(1);
        }
    }
    return 1 as ::core::ffi::c_int;
}
unsafe fn mode_tree_swap(mtd: *mut mode_tree_data, direction: i32) {
    if (*mtd).swapcb.is_none() {
        return;
    }
    let depth = (&(*mtd).lines)[(*mtd).current as usize].depth;
    let mut swap_with = (*mtd).current;
    loop {
        if direction < 0 && swap_with < -direction as u_int {
            return;
        }
        let next = swap_with.wrapping_add(direction as u_int);
        if direction > 0 && next >= mode_tree_line_count(&*mtd) {
            return;
        }
        swap_with = next;
        let other_depth = (&(*mtd).lines)[swap_with as usize].depth;
        if other_depth > depth {
            continue;
        }
        if other_depth != depth {
            return;
        }
        break;
    }
    let current = (&(*mtd).lines)[(*mtd).current as usize]
        .item
        .borrow()
        .itemdata
        .clone();
    let other = (&(*mtd).lines)[swap_with as usize]
        .item
        .borrow()
        .itemdata
        .clone();
    if (*mtd).swapcb.as_mut().unwrap()(&current, &other, &mut (*mtd).sort_crit) {
        (*mtd).current = swap_with;
        mode_tree_build(mtd);
    }
}
pub fn mode_tree_get_current(mtd: &mode_tree_data) -> ModeTreeItemData {
    if mtd.lines.is_empty() {
        return ModeTreeItemData::None;
    }
    mtd.lines[mtd.current as usize]
        .item
        .borrow()
        .itemdata
        .clone()
}
pub fn mode_tree_get_current_name(mtd: &mode_tree_data) -> Rc<CStr> {
    Rc::clone(&mtd.lines[mtd.current as usize].item.borrow().name)
}
pub unsafe fn mode_tree_expand_current(mtd: *mut mode_tree_data) {
    let item = Rc::clone(&(&(*mtd).lines)[(*mtd).current as usize].item);
    if item.borrow().expanded == 0 {
        item.borrow_mut().expanded = 1;
        mode_tree_build(mtd);
    }
}
fn mode_tree_get_tag(mtd: &mode_tree_data, tag: uint64_t) -> Option<u_int> {
    mtd.lines
        .iter()
        .position(|line| line.item.borrow().tag == tag)
        .map(|index| index as u_int)
}
pub unsafe fn mode_tree_expand(mtd: *mut mode_tree_data, tag: uint64_t) {
    let Some(index) = mode_tree_get_tag(&*mtd, tag) else {
        return;
    };
    let item = Rc::clone(&(&(*mtd).lines)[index as usize].item);
    if item.borrow().expanded == 0 {
        item.borrow_mut().expanded = 1;
        mode_tree_build(mtd);
    }
}
pub unsafe fn mode_tree_set_current(
    mut mtd: *mut mode_tree_data,
    mut tag: uint64_t,
) -> ::core::ffi::c_int {
    if let Some(found) = mode_tree_get_tag(&*mtd, tag) {
        (*mtd).current = found;
        if (*mtd).current > (*mtd).height.wrapping_sub(1 as u_int) {
            (*mtd).offset = (*mtd)
                .current
                .wrapping_sub((*mtd).height)
                .wrapping_add(1 as u_int);
        } else {
            (*mtd).offset = 0 as u_int;
        }
        return 1 as ::core::ffi::c_int;
    }
    if (*mtd).current >= mode_tree_line_count(&*mtd) {
        if mode_tree_line_count(&*mtd) == 0 as u_int {
            return 0 as ::core::ffi::c_int;
        }
        (*mtd).current = mode_tree_line_count(&*mtd).wrapping_sub(1 as u_int);
        if (*mtd).current > (*mtd).height.wrapping_sub(1 as u_int) {
            (*mtd).offset = (*mtd)
                .current
                .wrapping_sub((*mtd).height)
                .wrapping_add(1 as u_int);
        } else {
            (*mtd).offset = 0 as u_int;
        }
    }
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn mode_tree_count_tagged(mtd: *mut mode_tree_data) -> u_int {
    (*mtd)
        .lines
        .iter()
        .filter(|line| line.item.borrow().tagged != 0)
        .count() as u_int
}
pub unsafe fn mode_tree_each_tagged(
    mtd: *mut mode_tree_data,
    mut cb: impl FnMut(&ModeTreeItemRef, *mut client, key_code),
    c: *mut client,
    key: key_code,
    current: i32,
) {
    let mut fired = false;
    let mut index = 0;
    while index < (*mtd).lines.len() {
        let item = Rc::clone(&(&(*mtd).lines)[index].item);
        if item.borrow().tagged != 0 {
            fired = true;
            cb(&item, c, key);
        }
        index += 1;
    }
    if !fired && current != 0 {
        let item = Rc::clone(&(&(*mtd).lines)[(*mtd).current as usize].item);
        cb(&item, c, key);
    }
}
pub unsafe fn mode_tree_start(
    wp: *mut window_pane,
    args: *mut args,
    buildcb: mode_tree_build_cb,
    drawcb: mode_tree_draw_cb,
    searchcb: mode_tree_search_cb,
    menucb: mode_tree_menu_cb,
    heightcb: mode_tree_height_cb,
    keycb: mode_tree_key_cb,
    swapcb: mode_tree_swap_cb,
    sortcb: mode_tree_sort_cb,
    helpcb: mode_tree_help_cb,
    menu: &'static [menu_item<'static>],
    s: *mut *mut screen,
) -> *mut mode_tree_data {
    let mut mtd: *mut mode_tree_data = ::core::ptr::null_mut::<mode_tree_data>();
    mtd = mode_tree_alloc_data();
    (*mtd).wp = wp;
    (*mtd).menu = menu;
    if drawcb.is_none() {
        (*mtd).preview = MODE_TREE_PREVIEW_OFF as ::core::ffi::c_int;
    } else if args_has(args, 'N' as i32 as u_char) > 1 as ::core::ffi::c_int {
        (*mtd).preview = MODE_TREE_PREVIEW_BIG as ::core::ffi::c_int;
    } else if args_has(args, 'N' as i32 as u_char) != 0 {
        (*mtd).preview = MODE_TREE_PREVIEW_OFF as ::core::ffi::c_int;
    } else {
        (*mtd).preview = MODE_TREE_PREVIEW_NORMAL as ::core::ffi::c_int;
    }
    (*mtd).sort_crit.order = sort_order_from_string(args_get(args, 'O' as i32 as u_char));
    (*mtd).sort_crit.reversed = args_has(args, 'r' as i32 as u_char);
    if args_has(args, 'f' as i32 as u_char) != 0 {
        (*mtd).filter = Some(CStr::from_ptr(args_get(args, 'f' as i32 as u_char)).to_owned());
    } else {
        (*mtd).filter = None;
    }
    (*mtd).buildcb = buildcb;
    (*mtd).drawcb = drawcb;
    (*mtd).searchcb = searchcb;
    (*mtd).menucb = menucb;
    (*mtd).heightcb = heightcb;
    (*mtd).keycb = keycb;
    (*mtd).swapcb = swapcb;
    (*mtd).sortcb = sortcb;
    (*mtd).helpcb = helpcb;
    *s = &raw mut (*mtd).screen;
    screen_init(&mut **s, (*wp).base.grid().sx, (*wp).base.grid().sy, 0 as u_int);
    (**s).mode &= !MODE_CURSOR;
    return mtd;
}
pub unsafe fn mode_tree_zoom(mut mtd: *mut mode_tree_data, mut args: *mut args) {
    let mut wp: *mut window_pane = (*mtd).wp;
    if args_has(args, 'Z' as i32 as u_char) != 0 {
        (*mtd).zoomed = (*(*wp).window).flags & WINDOW_ZOOMED;
        if (*mtd).zoomed == 0 && window_zoom(wp) == 0 as ::core::ffi::c_int {
            server_redraw_window((*wp).window as *mut window);
        }
    } else {
        (*mtd).zoomed = -(1 as ::core::ffi::c_int);
    };
}
unsafe fn mode_tree_set_height(mut mtd: *mut mode_tree_data) {
    let mut s: *mut screen = &raw mut (*mtd).screen;
    let mut height: u_int = 0;
    if (*mtd).heightcb.is_some() {
        height = (*mtd).heightcb.as_mut().expect("non-null height callback")((*s).grid().sy);
        if height < (*s).grid().sy {
            (*mtd).height = (*s).grid().sy.wrapping_sub(height);
        }
    } else if (*mtd).preview == MODE_TREE_PREVIEW_NORMAL as ::core::ffi::c_int {
        (*mtd).height = (*s).grid()
            .sy
            .wrapping_div(3 as u_int)
            .wrapping_mul(2 as u_int);
        if (*mtd).height > mode_tree_line_count(&*mtd) {
            (*mtd).height = (*s).grid().sy.wrapping_div(2 as u_int);
        }
        if (*mtd).height < 10 as u_int {
            (*mtd).height = (*s).grid().sy;
        }
    } else if (*mtd).preview == MODE_TREE_PREVIEW_BIG as ::core::ffi::c_int {
        (*mtd).height = (*s).grid().sy.wrapping_div(4 as u_int);
        if (*mtd).height > mode_tree_line_count(&*mtd) {
            (*mtd).height = mode_tree_line_count(&*mtd);
        }
        if (*mtd).height < 2 as u_int {
            (*mtd).height = 2 as u_int;
        }
    } else {
        (*mtd).height = (*s).grid().sy;
    }
    if (*s).grid().sy.wrapping_sub((*mtd).height) < 2 as u_int {
        (*mtd).height = (*s).grid().sy;
    }
}
pub unsafe fn mode_tree_build(mut mtd: *mut mode_tree_data) {
    let mut s: *mut screen = &raw mut (*mtd).screen;
    let mut tag: Option<uint64_t>;
    if !(*mtd).lines.is_empty() {
        tag = Some((&(*mtd).lines)[(*mtd).current as usize].item.borrow().tag);
    } else {
        tag = None;
    }
    (*mtd).saved.items.append(&mut (*mtd).children.items);
    if (*mtd).sortcb.is_some() {
        (*mtd).sortcb.expect("non-null sort callback")(&mut (*mtd).sort_crit);
    }
    tag = (*mtd).buildcb.as_mut().expect("non-null build callback")(
        &mut (*mtd).sort_crit,
        tag,
        (*mtd).filter.as_deref(),
    );
    (*mtd).no_matches = (*mtd).children.items.is_empty() as i32;
    if (*mtd).no_matches != 0 {
        tag = (*mtd).buildcb.as_mut().expect("non-null build callback")(
            &mut (*mtd).sort_crit,
            tag,
            None,
        );
    }
    mode_tree_free_items(&mut (*mtd).saved);
    mode_tree_clear_lines(mtd);
    (*mtd).maxdepth = 0 as u_int;
    mode_tree_build_lines(mtd, &(*mtd).children.items.clone(), 0);
    if !(*mtd).lines.is_empty() && tag.is_none() {
        tag = Some((&(*mtd).lines)[(*mtd).current as usize].item.borrow().tag);
    }
    mode_tree_set_current(mtd, tag.unwrap_or(UINT64_MAX as uint64_t));
    (*mtd).width = (*s).grid().sx;
    if (*mtd).preview != MODE_TREE_PREVIEW_OFF as ::core::ffi::c_int {
        mode_tree_set_height(mtd);
    } else {
        (*mtd).height = (*s).grid().sy;
    }
    mode_tree_check_selected(mtd);
}

unsafe fn mode_tree_remove_ref(mut mtd: *mut mode_tree_data) {
    crate::src::shared::rc::release(mtd);
}
pub unsafe fn mode_tree_free(mut mtd: *mut mode_tree_data) {
    let mut wp: *mut window_pane = (*mtd).wp;
    if (*mtd).zoomed == 0 as ::core::ffi::c_int {
        server_unzoom_window((*wp).window as *mut window);
    }
    mode_tree_clear_prompt(mtd);
    mode_tree_free_items(&mut (*mtd).children);
    mode_tree_clear_lines(mtd);
    screen_free(&mut (*mtd).screen);
    (*mtd).search = None;
    (*mtd).filter = None;
    (*mtd).dead = 1 as ::core::ffi::c_int;
    mode_tree_remove_ref(mtd);
}
pub unsafe fn mode_tree_resize(mut mtd: *mut mode_tree_data, mut sx: u_int, mut sy: u_int) {
    let mut s: *mut screen = &raw mut (*mtd).screen;
    screen_resize(&mut *s, sx, sy, 0 as ::core::ffi::c_int);
    mode_tree_build(mtd);
    mode_tree_draw(mtd);
    (*(*mtd).wp).flags |= PANE_REDRAW;
}
pub unsafe fn mode_tree_add(
    mtd: *mut mode_tree_data,
    parent: Option<&ModeTreeItemRef>,
    itemdata: ModeTreeItemData,
    tag: uint64_t,
    name: &CStr,
    text: Option<&CStr>,
    expanded: i32,
) -> ModeTreeItemRef {
    log_debug(format_args!(
        "mode_tree_add: {tag}, {} {}",
        log_bytes(name.to_bytes()),
        log_bytes(text.unwrap_or(c"").to_bytes())
    ));
    let mut row = mode_tree_item {
        parent: parent.map_or_else(Weak::new, Rc::downgrade),
        itemdata,
        tag,
        name: Rc::from(name),
        text: text.map(CStr::to_owned),
        ..mode_tree_item::empty()
    };
    if let Some(saved) = mode_tree_find_item(&(*mtd).saved, tag) {
        let saved = saved.borrow();
        if parent.is_none_or(|parent| parent.borrow().expanded != 0) {
            row.tagged = saved.tagged;
        }
        row.expanded = saved.expanded;
    } else {
        row.expanded = if expanded == -1 { 1 } else { expanded };
    }
    let owner = Rc::new(RefCell::new(row));
    if let Some(parent) = parent {
        parent.borrow_mut().children.items.push(Rc::clone(&owner));
    } else {
        (*mtd).children.items.push(Rc::clone(&owner));
    }
    owner
}
pub fn mode_tree_view_name(mtd: &mut mode_tree_data, name: Option<&'static CStr>) {
    mtd.view_name = name;
}
pub fn mode_tree_draw_as_parent(item: &ModeTreeItemRef) {
    item.borrow_mut().draw_as_parent = 1;
}
pub fn mode_tree_no_tag(item: &ModeTreeItemRef) {
    item.borrow_mut().no_tag = 1;
}
pub fn mode_tree_align(item: &ModeTreeItemRef) {
    item.borrow_mut().align = 1;
}
pub unsafe fn mode_tree_remove(mtd: *mut mode_tree_data, item: &ModeTreeItemRef) {
    let parent = item.borrow().parent.upgrade();
    if let Some(parent) = parent {
        let mut parent = parent.borrow_mut();
        let position = parent.children.position(item);
        parent.children.items.remove(position);
    } else {
        let position = (*mtd).children.position(item);
        (*mtd).children.items.remove(position);
    }
}
fn mode_tree_append_printf_string(bytes: &mut Vec<u8>, value: Option<&CStr>) {
    if let Some(value) = value {
        bytes.extend_from_slice(value.to_bytes());
    } else {
        bytes.extend_from_slice(b"(null)");
    }
}
pub unsafe fn mode_tree_draw(mut mtd: *mut mode_tree_data) {
    let mut wp: *mut window_pane = (*mtd).wp;
    let mut s: *mut screen = &raw mut (*mtd).screen;
    let mut oo: *mut options = (*(*wp).window).options;
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: ::core::ptr::null_mut::<window_pane>(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        item: None,
        scrolled: 0,
        bg: 0,
    };
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut gc0: grid_cell = grid_cell {
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
    let mut box_gc: grid_cell = grid_cell {
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
    let mut w: u_int = 0;
    let mut h: u_int = 0;
    let mut i: u_int = 0;
    let mut sy: u_int = 0;
    let mut box_x: u_int = 0;
    let mut box_y: u_int = 0;
    let mut width: u_int = 0;
    let mut text_width: u_int = 0;
    let mut prefix_width: u_int = 0;
    let mut left: u_int = 0;
    let mut tag: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut separator: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut n: size_t = 0;
    let mut keylen: ::core::ffi::c_int = 0;
    let vla = (*mtd).maxdepth.wrapping_add(1 as u_int) as usize;
    let mut alignlen: Vec<::core::ffi::c_int> = ::std::vec::from_elem(0, vla);
    let mut dfg: ::core::ffi::c_int = 0;
    let mut dfg0: ::core::ffi::c_int = 0;
    if mode_tree_line_count(&*mtd) == 0 as u_int {
        return;
    }
    w = (*mtd).width;
    h = (*mtd).height;
    if w == 0 as u_int || h == 0 as u_int {
        return;
    }
    memcpy(
        &raw mut gc0 as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    memcpy(
        &raw mut gc as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    style_apply(
        &raw mut gc,
        oo,
        b"tree-mode-selection-style\0" as *const u8 as *const ::core::ffi::c_char,
        ::core::ptr::null_mut::<format_tree>(),
    );
    memcpy(
        &raw mut box_gc as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    style_apply(
        &raw mut box_gc,
        oo,
        b"tree-mode-border-style\0" as *const u8 as *const ::core::ffi::c_char,
        ::core::ptr::null_mut::<format_tree>(),
    );
    dfg = gc.fg;
    dfg0 = gc0.fg;
    screen_write_start(&mut ctx, s);
    screen_write_clearscreen(&mut ctx, 8 as u_int);
    ft = format_create_defaults(
        ::core::ptr::null_mut::<cmdq_item>(),
        ::core::ptr::null_mut::<client>(),
        ::core::ptr::null_mut::<session>(),
        ::core::ptr::null_mut::<winlink>(),
        wp,
    );
    keylen = 0 as ::core::ffi::c_int;
    i = 0 as u_int;
    while i < mode_tree_line_count(&*mtd) {
        let mti = (&(*mtd).lines)[i as usize].item.borrow();
        if !(mti.key == KEYC_NONE as ::core::ffi::c_ulong as key_code) {
            if mti.keylen as ::core::ffi::c_int + 3 as ::core::ffi::c_int > keylen {
                keylen = mti.keylen.wrapping_add(3 as size_t) as ::core::ffi::c_int;
            }
        }
        i = i.wrapping_add(1);
    }
    i = 0 as u_int;
    while i < (*mtd).maxdepth.wrapping_add(1 as u_int) {
        *alignlen.as_mut_ptr().offset(i as isize) = 0 as ::core::ffi::c_int;
        i = i.wrapping_add(1);
    }
    i = 0 as u_int;
    while i < mode_tree_line_count(&*mtd) {
        let line = (&(*mtd).lines)[i as usize].clone();
        let mti = line.item.borrow();
        if mti.align != 0
            && strlen((mti.name).as_ptr().cast_mut()) as ::core::ffi::c_int
                > *alignlen.as_mut_ptr().offset(line.depth as isize)
        {
            *alignlen.as_mut_ptr().offset(line.depth as isize) =
                strlen((mti.name).as_ptr().cast_mut()) as ::core::ffi::c_int;
        }
        i = i.wrapping_add(1);
    }
    i = 0 as u_int;
    while i < mode_tree_line_count(&*mtd) {
        if !(i < (*mtd).offset) {
            if i > (*mtd).offset.wrapping_add(h).wrapping_sub(1 as u_int) {
                break;
            }
            let line = (&(*mtd).lines)[i as usize].clone();
            let mti = line.item.borrow();
            screen_write_cursormove(
                &mut ctx,
                0 as ::core::ffi::c_int,
                i.wrapping_sub((*mtd).offset) as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            if mti.key != KEYC_NONE as ::core::ffi::c_ulong as key_code {
                format_add(
                    ft,
                    b"mode_tree_key\0" as *const u8 as *const ::core::ffi::c_char,
                    |out| {
                        write_cstr(
                            out,
                            (mti.keystr)
                                .as_ref()
                                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                        )
                    },
                );
            } else {
                format_add(
                    ft,
                    b"mode_tree_key\0" as *const u8 as *const ::core::ffi::c_char,
                    |out| write_cstr(out, b"\0" as *const u8 as *const ::core::ffi::c_char),
                );
            }
            format_add(
                ft,
                b"mode_tree_key_width\0" as *const u8 as *const ::core::ffi::c_char,
                |out| write!(out, "{}", (keylen) as i32),
            );
            format_add(
                ft,
                b"mode_tree_selected\0" as *const u8 as *const ::core::ffi::c_char,
                |out| {
                    write!(
                        out,
                        "{}",
                        ((i == (*mtd).current) as ::core::ffi::c_int) as i32
                    )
                },
            );
            if line.depth == 0 as u_int {
                format_add(
                    ft,
                    b"mode_tree_repeat\0" as *const u8 as *const ::core::ffi::c_char,
                    |out| write!(out, "{}", (0 as ::core::ffi::c_int) as u32),
                );
                format_add(
                    ft,
                    b"mode_tree_branch\0" as *const u8 as *const ::core::ffi::c_char,
                    |out| out.write_all(b"0"),
                );
                format_add(
                    ft,
                    b"mode_tree_parent_last\0" as *const u8 as *const ::core::ffi::c_char,
                    |out| out.write_all(b"0"),
                );
            } else {
                format_add(
                    ft,
                    b"mode_tree_repeat\0" as *const u8 as *const ::core::ffi::c_char,
                    |out| write!(out, "{}", (line.depth.wrapping_sub(1 as u_int)) as u32),
                );
                format_add(
                    ft,
                    b"mode_tree_branch\0" as *const u8 as *const ::core::ffi::c_char,
                    |out| out.write_all(b"1"),
                );
                if mti
                    .parent
                    .upgrade()
                    .is_some_and(|parent| (&(*mtd).lines)[parent.borrow().line as usize].last != 0)
                {
                    format_add(
                        ft,
                        b"mode_tree_parent_last\0" as *const u8 as *const ::core::ffi::c_char,
                        |out| out.write_all(b"1"),
                    );
                } else {
                    format_add(
                        ft,
                        b"mode_tree_parent_last\0" as *const u8 as *const ::core::ffi::c_char,
                        |out| out.write_all(b"0"),
                    );
                }
            }
            if mti.children.items.is_empty() {
                format_add(
                    ft,
                    b"mode_tree_has_children\0" as *const u8 as *const ::core::ffi::c_char,
                    |out| out.write_all(b"0"),
                );
            } else {
                format_add(
                    ft,
                    b"mode_tree_has_children\0" as *const u8 as *const ::core::ffi::c_char,
                    |out| out.write_all(b"1"),
                );
            }
            format_add(
                ft,
                b"mode_tree_last\0" as *const u8 as *const ::core::ffi::c_char,
                |out| write!(out, "{}", (line.last) as i32),
            );
            format_add(
                ft,
                b"mode_tree_expanded\0" as *const u8 as *const ::core::ffi::c_char,
                |out| write!(out, "{}", (mti.expanded) as i32),
            );
            format_add(
                ft,
                b"mode_tree_flat\0" as *const u8 as *const ::core::ffi::c_char,
                |out| write!(out, "{}", (line.flat) as i32),
            );
            let prefix = format_expand_cstring(
                ft,
                b"#[fg=themelightgrey]#[bg=default]#[noacs]#{p/#{mode_tree_key_width}:#{?#{!=:#{mode_tree_key},},(#{mode_tree_key}),}}#{R:#{?mode_tree_parent_last,    ,#[acs]x#[fg=themelightgrey]#[bg=default]#[noacs]   },#{mode_tree_repeat}}#{?mode_tree_branch,#[acs]#{?mode_tree_last,mq,tq}+#[fg=themelightgrey]#[bg=default]#[noacs] ,}#{?mode_tree_has_children,#{?mode_tree_expanded,#[fg=themered]-#[fg=themelightgrey]#[bg=default]#[noacs] ,#[fg=themegreen]+#[fg=themelightgrey]#[bg=default]#[noacs] },#{?mode_tree_flat,,  }}\0"
                    as *const u8 as *const ::core::ffi::c_char,
            );
            prefix_width = format_width(prefix.as_ptr());
            if prefix_width > w {
                prefix_width = w;
            }
            if mti.tagged != 0 {
                tag = b"*\0" as *const u8 as *const ::core::ffi::c_char;
            } else {
                tag = b"\0" as *const u8 as *const ::core::ffi::c_char;
            }
            if !mti.text.is_none() {
                separator = b"#[fg=themelightgrey]: #[default]\0" as *const u8
                    as *const ::core::ffi::c_char;
            } else {
                separator = b"\0" as *const u8 as *const ::core::ffi::c_char;
            }
            let field_width = mti.align * *alignlen.as_mut_ptr().offset(line.depth as isize);
            let mut name = Vec::new();
            mode_tree_append_printf_string(&mut name, Some(mti.name.as_ref()));
            let padding = (field_width.unsigned_abs() as usize).saturating_sub(name.len());
            let mut row = Vec::with_capacity(
                name.len()
                    + padding
                    + CStr::from_ptr(tag).to_bytes().len()
                    + CStr::from_ptr(separator).to_bytes().len(),
            );
            if field_width >= 0 {
                row.extend(std::iter::repeat_n(b' ', padding));
            }
            row.extend_from_slice(&name);
            if field_width < 0 {
                row.extend(std::iter::repeat_n(b' ', padding));
            }
            row.extend_from_slice(CStr::from_ptr(tag).to_bytes());
            row.extend_from_slice(CStr::from_ptr(separator).to_bytes());
            let text = CString::new(row).expect("mode-tree row contains no NUL");
            text_width = format_width(text.as_ptr());
            left = if prefix_width < w {
                w.wrapping_sub(prefix_width)
            } else {
                0 as u_int
            };
            if text_width > left {
                text_width = left;
            }
            width = prefix_width.wrapping_add(text_width);
            if mti.tagged != 0 {
                gc.fg = COLOUR_THEME_CYAN as ::core::ffi::c_int | COLOUR_FLAG_THEME;
                gc0.fg = COLOUR_THEME_CYAN as ::core::ffi::c_int | COLOUR_FLAG_THEME;
            }
            if i != (*mtd).current {
                screen_write_clearendofline(&mut ctx, 8 as u_int);
                format_draw(
                    &raw mut ctx,
                    &raw const grid_default_cell,
                    prefix_width,
                    prefix.as_ptr(),
                    ::core::ptr::null_mut::<style_ranges>(),
                    0 as ::core::ffi::c_int,
                );
                if left != 0 as u_int {
                    screen_write_cursormove(
                        &mut ctx,
                        prefix_width as ::core::ffi::c_int,
                        i.wrapping_sub((*mtd).offset) as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    );
                    format_draw(
                        &raw mut ctx,
                        &raw mut gc0,
                        left,
                        text.as_ptr(),
                        ::core::ptr::null_mut::<style_ranges>(),
                        0 as ::core::ffi::c_int,
                    );
                    if !mti.text.is_none() && width < w {
                        screen_write_cursormove(
                            &mut ctx,
                            width as ::core::ffi::c_int,
                            i.wrapping_sub((*mtd).offset) as ::core::ffi::c_int,
                            0 as ::core::ffi::c_int,
                        );
                        format_draw(
                            &raw mut ctx,
                            &raw mut gc0,
                            w.wrapping_sub(width),
                            (mti.text)
                                .as_ref()
                                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                            ::core::ptr::null_mut::<style_ranges>(),
                            0 as ::core::ffi::c_int,
                        );
                    }
                }
            } else {
                screen_write_clearendofline(&mut ctx, gc.bg as u_int);
                format_draw(
                    &raw mut ctx,
                    &raw mut gc,
                    prefix_width,
                    prefix.as_ptr(),
                    ::core::ptr::null_mut::<style_ranges>(),
                    1 as ::core::ffi::c_int,
                );
                if left != 0 as u_int {
                    screen_write_cursormove(
                        &mut ctx,
                        prefix_width as ::core::ffi::c_int,
                        i.wrapping_sub((*mtd).offset) as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    );
                    format_draw(
                        &raw mut ctx,
                        &raw mut gc,
                        left,
                        text.as_ptr(),
                        ::core::ptr::null_mut::<style_ranges>(),
                        1 as ::core::ffi::c_int,
                    );
                    if !mti.text.is_none() && width < w {
                        screen_write_cursormove(
                            &mut ctx,
                            width as ::core::ffi::c_int,
                            i.wrapping_sub((*mtd).offset) as ::core::ffi::c_int,
                            0 as ::core::ffi::c_int,
                        );
                        format_draw(
                            &raw mut ctx,
                            &raw mut gc,
                            w.wrapping_sub(width),
                            (mti.text)
                                .as_ref()
                                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                            ::core::ptr::null_mut::<style_ranges>(),
                            1 as ::core::ffi::c_int,
                        );
                    }
                }
            }
            drop(text);
            if mti.tagged != 0 {
                gc.fg = dfg;
                gc0.fg = dfg0;
            }
        }
        i = i.wrapping_add(1);
    }
    format_free(ft);
    if !((*mtd).preview == MODE_TREE_PREVIEW_OFF as ::core::ffi::c_int) {
        sy = (*s).grid().sy;
        if !(sy <= 4 as u_int
            || h < 2 as u_int
            || sy.wrapping_sub(h) <= 4 as u_int
            || w <= 4 as u_int)
        {
            let line = (&(*mtd).lines)[(*mtd).current as usize].clone();
            let mut owner = Rc::clone(&line.item);
            if owner.borrow().draw_as_parent != 0 {
                let parent = owner
                    .borrow()
                    .parent
                    .upgrade()
                    .expect("preview row has a parent");
                owner = parent;
            }
            let mti = owner.borrow();
            screen_write_cursormove(
                &mut ctx,
                0 as ::core::ffi::c_int,
                h as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            screen_write_box(
                &mut ctx,
                w,
                sy.wrapping_sub(h),
                BOX_LINES_DEFAULT,
                Some(&box_gc),
                None,
            );
            let mut label_bytes = b" ".to_vec();
            mode_tree_append_printf_string(&mut label_bytes, Some(mti.name.as_ref()));
            if !(&(*mtd).sort_crit.order_seq).is_empty() {
                label_bytes.extend_from_slice(b" (sort: ");
                let order = sort_order_to_string((*mtd).sort_crit.order);
                let order = if order.is_null() {
                    None
                } else {
                    Some(CStr::from_ptr(order))
                };
                mode_tree_append_printf_string(&mut label_bytes, order);
                if (*mtd).sort_crit.reversed != 0 {
                    label_bytes.extend_from_slice(b", reversed");
                }
                label_bytes.push(b')');
                if let Some(view_name) = (*mtd).view_name {
                    label_bytes.extend_from_slice(b" (view: ");
                    mode_tree_append_printf_string(&mut label_bytes, Some(view_name));
                    label_bytes.push(b')');
                }
            }
            let label = CString::new(label_bytes).expect("preview label contains no NUL");
            let label_len = label.as_bytes().len() as size_t;
            if w.wrapping_sub(2 as u_int) as size_t >= label_len {
                screen_write_cursormove(
                    &mut ctx,
                    1 as ::core::ffi::c_int,
                    h as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
                screen_write_puts(&mut ctx, &box_gc, |out| write_cstr(out, label.as_ptr()));
                if (*mtd).no_matches != 0 {
                    n = (::core::mem::size_of::<[::core::ffi::c_char; 11]>() as usize)
                        .wrapping_sub(1 as usize) as size_t;
                } else {
                    n = (::core::mem::size_of::<[::core::ffi::c_char; 7]>() as usize)
                        .wrapping_sub(1 as usize) as size_t;
                }
                if (*mtd).filter.is_some()
                    && w.wrapping_sub(2 as u_int) as size_t
                        >= label_len
                            .wrapping_add(10 as size_t)
                            .wrapping_add(n)
                            .wrapping_add(2 as size_t)
                {
                    screen_write_puts(&mut ctx, &box_gc, |out| out.write_all(b" (filter: "));
                    if (*mtd).no_matches != 0 {
                        screen_write_puts(&mut ctx, &box_gc, |out| out.write_all(b"no matches"));
                    } else {
                        screen_write_puts(&mut ctx, &box_gc, |out| out.write_all(b"active"));
                    }
                    screen_write_puts(&mut ctx, &box_gc, |out| out.write_all(b") "));
                } else {
                    screen_write_puts(&mut ctx, &box_gc, |out| out.write_all(b" "));
                }
            }
            drop(label);
            box_x = w.wrapping_sub(4 as u_int);
            box_y = sy.wrapping_sub(h).wrapping_sub(2 as u_int);
            if box_x != 0 as u_int && box_y != 0 as u_int {
                screen_write_cursormove(
                    &mut ctx,
                    2 as ::core::ffi::c_int,
                    h.wrapping_add(1 as u_int) as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
                let itemdata = mti.itemdata.clone();
                drop(mti);
                (*mtd).drawcb.as_mut().expect("non-null draw callback")(
                    &itemdata, &mut ctx, box_x, box_y,
                );
            }
        }
    }
    if (*mtd).help != 0 {
        mode_tree_draw_help(mtd, &raw mut ctx);
    }
    if (*mtd).prompt.is_some() {
        mode_tree_draw_prompt(mtd, &mut ctx);
    } else {
        (*s).mode &= !MODE_CURSOR;
        screen_write_cursormove(
            &mut ctx,
            0 as ::core::ffi::c_int,
            (*mtd).current.wrapping_sub((*mtd).offset) as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
    }
    screen_write_stop(&mut ctx);
}
unsafe fn mode_tree_draw_prompt(mut mtd: *mut mode_tree_data, ctx: &mut screen_write_ctx) {
    let mut s: *mut screen = &raw mut (*mtd).screen;
    let mut sx: u_int = (*s).grid().sx;
    let mut sy: u_int = (*s).grid().sy;
    let mut py: u_int = 0;
    if sx == 0 as u_int || sy == 0 as u_int {
        return;
    }
    if (*mtd).prompt_top != 0 {
        py = 0 as u_int;
    } else {
        py = sy.wrapping_sub(1 as u_int);
    }
    let pdd = prompt_draw_data {
        area_x: 0 as u_int,
        area_width: sx,
        prompt_line: py,
    };
    (*s).mode |= MODE_CURSOR;
    (*mtd).prompt_cx = prompt_draw(
        &(*mtd).prompt.as_ref().expect("active prompt").borrow(),
        ctx,
        pdd,
    );
    screen_write_cursormove(
        ctx,
        (*mtd).prompt_cx as ::core::ffi::c_int,
        py as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
}
pub unsafe fn mode_tree_clear_prompt(mut mtd: *mut mode_tree_data) {
    if let Some(prompt) = (*mtd).prompt.take() {
        prompt_free(&prompt);
        (*mtd).screen.mode &= !MODE_CURSOR;
    }
}
fn mode_tree_prompt_accept(tree: Rc<UnsafeCell<mode_tree_data>>) -> cmdq_cb {
    Some(Box::new(move |item| unsafe {
        let mtd = crate::src::shared::rc::as_ptr(&tree);
        let c = cmdq_get_client(item.as_ptr());
        let mut key = b'y' as key_code;
        if (*mtd).prompt.is_some() && !c.is_null() {
            mode_tree_key(
                mtd,
                c,
                &mut key,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
        }
        CMD_RETURN_NORMAL
    }))
}
fn mode_tree_prompt_input_callback(
    data: &ModeTreePromptRef,
    input: Option<&CStr>,
    key: prompt_key_result,
) -> prompt_result {
    let (client, callback) = {
        let mut state = data.borrow_mut();
        (state.c.upgrade(), state.inputcb.take())
    };
    let Some(mut callback) = callback else {
        return PROMPT_CLOSE;
    };
    let result = callback(
        client
            .as_ref()
            .and_then(|client| std::ptr::NonNull::new(crate::src::shared::rc::as_ptr(client))),
        input,
        key,
    );
    data.borrow_mut().inputcb = Some(callback);
    result
}
unsafe fn mode_tree_prompt_free_callback(data: &ModeTreePromptRef) {
    let (mtd, callback, inputcb) = {
        let mut state = data.borrow_mut();
        (state.mtd.take(), state.freecb.take(), state.inputcb.take())
    };
    if let Some(mtd) = &mtd {
        let mtd = crate::src::shared::rc::as_ptr(mtd);
        if (*mtd).prompt_data.ptr_eq(&Rc::downgrade(data)) {
            (*mtd).prompt_data = Weak::new();
        }
    }
    if let Some(callback) = callback {
        callback();
    }
    // Match tmux's reference release after the user cleanup callback. Cached
    // callback records must not extend the mode tree's lifetime beyond it.
    drop(mtd);
    drop(inputcb);
}
pub unsafe fn mode_tree_set_prompt(
    mut mtd: *mut mode_tree_data,
    mut c: *mut client,
    prompt: &CStr,
    input: Option<&CStr>,
    mut type_0: prompt_type,
    mut flags: ::core::ffi::c_int,
    mut inputcb: mode_tree_prompt_input_cb,
    mut freecb: prompt_free_cb,
) {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    let mut pd = prompt_create_data::default();
    if !c.is_null() && !(*c).session.is_null() {
        s = (*c).session;
        oo = (*s).options;
    } else {
        s = ::core::ptr::null_mut::<session>();
        oo = global_s_options;
    }
    mode_tree_clear_prompt(mtd);
    crate::src::shared::rc::retain(mtd);
    let mtp = Rc::new(RefCell::new(mode_tree_prompt {
        mtd: Some(crate::src::shared::rc::take(mtd)),
        c: if c.is_null() {
            Weak::new()
        } else {
            crate::src::shared::rc::downgrade(c)
        },
        inputcb,
        freecb,
    }));
    (*mtd).prompt_top = (options_get_number(
        oo,
        b"status-position\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_longlong) as ::core::ffi::c_int;
    prompt_set_options(&mut pd, s.as_ref());
    pd.prompt = prompt;
    pd.input = input;
    pd.type_0 = type_0;
    pd.flags = flags | PROMPT_ISMODE;
    let input_data = mtp.clone();
    pd.inputcb = Some(Box::new(move |s, key| {
        mode_tree_prompt_input_callback(&input_data, s, key)
    }));
    let free_data = mtp.clone();
    pd.freecb = Some(Box::new(move || unsafe {
        mode_tree_prompt_free_callback(&free_data)
    }));
    let prompt = prompt_create(pd);
    (*mtd).prompt = Some(prompt.clone());
    (*mtd).prompt_data = Rc::downgrade(&mtp);
    mode_tree_draw(mtd);
    (*(*mtd).wp).flags |= PANE_REDRAW;
    if flags & PROMPT_SINGLE != 0 && flags & PROMPT_ACCEPT != 0 && !c.is_null() {
        crate::src::shared::rc::retain(mtd);
        let tree = crate::src::shared::rc::take(mtd);
        let item = cmdq_get_callback_owned(
            c"mode_tree_prompt_accept".as_ptr(),
            mode_tree_prompt_accept(tree),
        );
        cmdq_append(c, item);
    }
}
unsafe fn mode_tree_search_backward(mtd: *mut mode_tree_data) -> Option<ModeTreeItemRef> {
    mode_tree_search(mtd, false)
}
unsafe fn mode_tree_search_forward(mtd: *mut mode_tree_data) -> Option<ModeTreeItemRef> {
    mode_tree_search(mtd, true)
}

fn mode_tree_search_next(mtd: &mode_tree_data, item: &ModeTreeItemRef) -> Option<ModeTreeItemRef> {
    if let Some(child) = item.borrow().children.first() {
        return Some(child);
    }
    let mut current = Rc::clone(item);
    loop {
        if let Some(next) = mode_tree_sibling(mtd, &current, true) {
            return Some(next);
        }
        let parent = current.borrow().parent.upgrade();
        let Some(parent) = parent else {
            return mtd.children.first();
        };
        current = parent;
    }
}

fn mode_tree_search_previous(
    mtd: &mode_tree_data,
    item: &ModeTreeItemRef,
) -> Option<ModeTreeItemRef> {
    let mut previous = if let Some(previous) = mode_tree_sibling(mtd, item, false) {
        previous
    } else if let Some(parent) = item.borrow().parent.upgrade() {
        return Some(parent);
    } else {
        mtd.children.last()?
    };
    loop {
        let last = previous.borrow().children.last();
        let Some(last) = last else {
            return Some(previous);
        };
        previous = last;
    }
}

unsafe fn mode_tree_search(mtd: *mut mode_tree_data, forward: bool) -> Option<ModeTreeItemRef> {
    let search = (*mtd).search.as_deref()?;
    let icase = (*mtd).search_icase != 0;
    let last = Rc::clone(&(&(*mtd).lines)[(*mtd).current as usize].item);
    let mut item = Rc::clone(&last);
    loop {
        item = if forward {
            mode_tree_search_next(&*mtd, &item)?
        } else {
            mode_tree_search_previous(&*mtd, &item)?
        };
        if Rc::ptr_eq(&item, &last) {
            return None;
        }
        let (name, itemdata) = {
            let row = item.borrow();
            (Rc::clone(&row.name), row.itemdata.clone())
        };
        let matched = if let Some(callback) = (*mtd).searchcb.as_mut() {
            callback(&itemdata, search, icase)
        } else if icase {
            !strcasestr(name.as_ptr(), search.as_ptr()).is_null()
        } else {
            !strstr(name.as_ptr(), search.as_ptr()).is_null()
        };
        if matched {
            return Some(item);
        }
    }
}
unsafe fn mode_tree_search_set(mtd: *mut mode_tree_data) {
    let found = if (*mtd).search_dir == MODE_TREE_SEARCH_FORWARD {
        mode_tree_search_forward(mtd)
    } else {
        mode_tree_search_backward(mtd)
    };
    let Some(item) = found else {
        return;
    };
    let tag = item.borrow().tag;
    let mut parent = item.borrow().parent.upgrade();
    while let Some(owner) = parent {
        let mut row = owner.borrow_mut();
        row.expanded = 1;
        parent = row.parent.upgrade();
    }
    mode_tree_build(mtd);
    mode_tree_set_current(mtd, tag);
    mode_tree_draw(mtd);
    (*(*mtd).wp).flags |= PANE_REDRAW;
}
unsafe fn mode_tree_search_callback(
    mut mtd: *mut mode_tree_data,
    s: Option<&CStr>,
    mut key: prompt_key_result,
) -> prompt_result {
    if (*mtd).dead != 0 {
        return PROMPT_CLOSE;
    }
    let replacement = s
        .filter(|text| !text.to_bytes().is_empty())
        .map(CStr::to_owned);
    (*mtd).search = replacement;
    if let Some(search) = (*mtd).search.as_ref() {
        (*mtd).search_icase = mode_tree_is_lowercase(search.as_ptr());
        mode_tree_search_set(mtd);
    }
    if key as ::core::ffi::c_uint == PROMPT_KEY_HANDLED as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return PROMPT_CONTINUE;
    }
    return PROMPT_CLOSE;
}
unsafe fn mode_tree_filter_callback(
    mut mtd: *mut mode_tree_data,
    s: Option<&CStr>,
    mut key: prompt_key_result,
) -> prompt_result {
    if (*mtd).dead != 0 {
        return PROMPT_CLOSE;
    }
    let replacement = s
        .filter(|text| !text.to_bytes().is_empty())
        .map(CStr::to_owned);
    (*mtd).filter = replacement;
    mode_tree_build(mtd);
    mode_tree_draw(mtd);
    (*(*mtd).wp).flags |= PANE_REDRAW;
    if key as ::core::ffi::c_uint == PROMPT_KEY_HANDLED as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return PROMPT_CONTINUE;
    }
    return PROMPT_CLOSE;
}
unsafe fn mode_tree_clear_filter(mut mtd: *mut mode_tree_data) {
    (*mtd).filter = None;
    mode_tree_build(mtd);
    mode_tree_draw(mtd);
    (*(*mtd).wp).flags |= PANE_REDRAW;
}
fn mode_tree_menu_callback(
    tree: Rc<UnsafeCell<mode_tree_data>>,
    client: Weak<UnsafeCell<client>>,
    line: u_int,
) -> menu_choice_cb {
    Some(Box::new(move |selection| unsafe {
        let MenuSelection::Selected { key, .. } = selection else {
            return;
        };
        let mtd = crate::src::shared::rc::as_ptr(&tree);
        if (*mtd).dead != 0 || key == KEYC_NONE || line >= mode_tree_line_count(&*mtd) {
            return;
        }
        let Some(client) = client.upgrade() else {
            return;
        };
        if (*crate::src::shared::rc::as_ptr(&client)).flags & CLIENT_DEAD as uint64_t != 0 {
            return;
        }
        (*mtd).current = line;
        let callback = (*mtd).menucb.take();
        if let Some(mut callback) = callback {
            callback(&client, key);
            // The handler may close the mode or install another callback.
            if (*mtd).dead == 0 && (*mtd).menucb.is_none() {
                (*mtd).menucb = Some(callback);
            }
        }
    }))
}

unsafe fn mode_tree_display_menu(
    mut mtd: *mut mode_tree_data,
    mut c: *mut client,
    mut x: u_int,
    mut y: u_int,
    mut outside: ::core::ffi::c_int,
) {
    let items: &[menu_item<'_>];
    let mut line: u_int = 0;
    if (*mtd).offset.wrapping_add(y) > mode_tree_line_count(&*mtd).wrapping_sub(1 as u_int) {
        line = (*mtd).current;
    } else {
        line = (*mtd).offset.wrapping_add(y);
    }

    let title = if outside == 0 {
        items = (*mtd).menu;
        let mut bytes = b"#[align=centre]".to_vec();
        bytes.extend_from_slice((&(*mtd).lines)[line as usize].item.borrow().name.to_bytes());
        CString::new(bytes).expect("mode tree item names contain no NUL")
    } else {
        items = &mode_tree_menu_items;
        c"".to_owned()
    };
    let mut menu = menu_create(&title);
    menu_add_items(&mut menu, items, c);
    drop(title);
    let tree = crate::src::shared::rc::downgrade(mtd)
        .upgrade()
        .expect("live mode tree");
    let client = crate::src::shared::rc::downgrade(c);
    if x >= (*menu)
        .width
        .wrapping_add(4 as u_int)
        .wrapping_div(2 as u_int)
    {
        x = x.wrapping_sub(
            (*menu)
                .width
                .wrapping_add(4 as u_int)
                .wrapping_div(2 as u_int),
        );
    } else {
        x = 0 as u_int;
    }
    x = x.wrapping_add((*(*mtd).wp).xoff as u_int);
    y = y.wrapping_add((*(*mtd).wp).yoff as u_int);
    menu_display(
        menu,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        None,
        x,
        y,
        c,
        BOX_LINES_DEFAULT,
        None,
        None,
        None,
        ::core::ptr::null_mut::<cmd_find_state>(),
        mode_tree_menu_callback(tree, client, line),
    );
}
unsafe fn mode_tree_draw_help_line(
    mut ctx: *mut screen_write_ctx,
    mut gc: *const grid_cell,
    mut ft: *mut format_tree,
    line: &CStr,
    item: &CStr,
    mut x: u_int,
    mut y: u_int,
    mut w: u_int,
) {
    let replaced = cmd_template_replace_cstring(line, item, 1 as ::core::ffi::c_int);
    let expanded = format_expand_cstring(ft, replaced.as_ptr());
    drop(replaced);
    screen_write_cursormove(
        &mut *ctx,
        x as ::core::ffi::c_int,
        y as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    screen_write_clearcharacter(&mut *ctx, w, (*gc).bg as u_int);
    screen_write_cursormove(
        &mut *ctx,
        x as ::core::ffi::c_int,
        y as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    format_draw(
        ctx,
        gc,
        w,
        expanded.as_ptr(),
        ::core::ptr::null_mut::<style_ranges>(),
        0 as ::core::ffi::c_int,
    );
}
unsafe fn mode_tree_draw_help(mut mtd: *mut mode_tree_data, mut ctx: *mut screen_write_ctx) {
    let mut s: *mut screen = &raw mut (*mtd).screen;
    let mut oo: *mut options = (*(*(*mtd).wp).window).options;
    let mut box_gc: grid_cell = grid_cell {
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
    let help = (*mtd).helpcb.map(|callback| callback());
    let lines: &[&'static CStr] = help.map_or(&[], |info| info.lines);
    let item = help.map_or(c"item", |info| info.item);
    let mut sx: u_int = (*s).grid().sx;
    let mut sy: u_int = (*s).grid().sy;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut w = help
        .map_or(MODE_TREE_HELP_DEFAULT_WIDTH as u_int, |info| info.width)
        .max(MODE_TREE_HELP_DEFAULT_WIDTH as u_int);
    let h = (mode_tree_help_start.len() + lines.len() + mode_tree_help_end.len()) as u_int;
    let mut box_w: u_int = 0;
    let mut box_h: u_int = 0;
    box_w = w.wrapping_add(2 as u_int);
    box_h = h.wrapping_add(2 as u_int);
    if sx < box_w || sy < box_h {
        return;
    }
    x = sx.wrapping_sub(box_w).wrapping_div(2 as u_int);
    y = sy.wrapping_sub(box_h).wrapping_div(2 as u_int);
    memcpy(
        &raw mut box_gc as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    style_apply(
        &raw mut box_gc,
        oo,
        b"tree-mode-border-style\0" as *const u8 as *const ::core::ffi::c_char,
        ::core::ptr::null_mut::<format_tree>(),
    );
    memcpy(
        &raw mut gc as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    ft = format_create_defaults(
        ::core::ptr::null_mut::<cmdq_item>(),
        ::core::ptr::null_mut::<client>(),
        ::core::ptr::null_mut::<session>(),
        ::core::ptr::null_mut::<winlink>(),
        (*mtd).wp,
    );
    screen_write_cursormove(
        &mut *ctx,
        x as ::core::ffi::c_int,
        y as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    screen_write_box(
        &mut *ctx,
        box_w,
        box_h,
        BOX_LINES_DEFAULT,
        Some(&box_gc),
        None,
    );
    y = y.wrapping_add(1);
    x = x.wrapping_add(1);
    for line in mode_tree_help_start.iter().copied() {
        mode_tree_draw_help_line(ctx, &gc, ft, line, item, x, y, w);
        y = y.wrapping_add(1);
    }
    for line in lines.iter().copied() {
        mode_tree_draw_help_line(ctx, &gc, ft, line, item, x, y, w);
        y = y.wrapping_add(1);
    }
    for line in mode_tree_help_end.iter().copied() {
        mode_tree_draw_help_line(ctx, &gc, ft, line, item, x, y, w);
        y = y.wrapping_add(1);
    }
    format_free(ft);
}
unsafe fn mode_tree_display_help(mut mtd: *mut mode_tree_data) {
    (*mtd).help = 1 as ::core::ffi::c_int;
    mode_tree_draw(mtd);
}
pub unsafe fn mode_tree_key(
    mut mtd: *mut mode_tree_data,
    mut c: *mut client,
    mut key: *mut key_code,
    mut m: *mut mouse_event,
    mut xp: *mut u_int,
    mut yp: *mut u_int,
) -> ::core::ffi::c_int {
    let mut i: u_int = 0;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut py: u_int = 0;
    let mut sx: u_int = 0;
    let mut choice: ::core::ffi::c_int = 0;
    let mut preview: ::core::ffi::c_int = 0;
    let mut result: prompt_key_result = PROMPT_KEY_NOT_HANDLED;
    let mut redraw: ::core::ffi::c_int = 0;
    if mode_tree_line_count(&*mtd) == 0 as u_int {
        *key = KEYC_NONE as ::core::ffi::c_ulong as key_code;
        return 1 as ::core::ffi::c_int;
    }
    if let Some(prompt) = (*mtd).prompt.clone() {
        let tree = crate::src::shared::rc::downgrade(mtd);
        redraw = 0 as ::core::ffi::c_int;
        let mtp = (*mtd).prompt_data.upgrade();
        if let Some(mtp) = &mtp {
            mtp.borrow_mut().c = if c.is_null() {
                Weak::new()
            } else {
                crate::src::shared::rc::downgrade(c)
            };
        }
        if *key & KEYC_MASK_KEY == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
            || *key & KEYC_MASK_TYPE
                >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int
                && *key & KEYC_MASK_TYPE
                    <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                        << 32 as ::core::ffi::c_int
        {
            if m.is_null()
                || (*m).b & MOUSE_MASK_BUTTONS as u_int != MOUSE_BUTTON_1 as u_int
                || (*m).b & MOUSE_MASK_DRAG as u_int != 0
                || (*m).b & MOUSE_MASK_BUTTONS as u_int == 3 as u_int
                || cmd_mouse_at(
                    (*mtd).wp,
                    m,
                    &raw mut x,
                    &raw mut y,
                    0 as ::core::ffi::c_int,
                ) != 0 as ::core::ffi::c_int
            {
                result = PROMPT_KEY_NOT_HANDLED;
            } else {
                sx = (*mtd).screen.grid().sx;
                if (*mtd).prompt_top != 0 {
                    py = 0 as u_int;
                } else {
                    py = (*mtd).screen.grid().sy.wrapping_sub(1 as u_int);
                }
                if y == py {
                    result = prompt_mouse(
                        &mut prompt.borrow_mut(),
                        x,
                        0 as u_int,
                        sx,
                        Some(&mut redraw),
                    );
                } else {
                    result = PROMPT_KEY_NOT_HANDLED;
                }
            }
        } else {
            result = prompt_key(&prompt, *key, &mut redraw);
        }
        // A prompt can destroy its pane and release the tree during dispatch.
        let Some(_tree_owner) = tree.upgrade() else {
            *key = KEYC_NONE;
            return 0;
        };
        if (*mtd).dead != 0 {
            *key = KEYC_NONE;
            return 0;
        }
        if let Some(mtp) = &mtp {
            if (*mtd).prompt_data.ptr_eq(&Rc::downgrade(mtp)) {
                mtp.borrow_mut().c = Weak::new();
            }
        }
        if (*mtd)
            .prompt
            .as_ref()
            .is_some_and(|current| std::rc::Rc::ptr_eq(current, &prompt))
            && (result as ::core::ffi::c_uint
                == PROMPT_KEY_CLOSE as ::core::ffi::c_int as ::core::ffi::c_uint
                || prompt_closed(&prompt.borrow()) != 0)
        {
            mode_tree_clear_prompt(mtd);
        }
        if (*mtd).dead != 0 {
            *key = KEYC_NONE;
            return 0;
        }
        if redraw != 0
            || !(*mtd)
                .prompt
                .as_ref()
                .is_some_and(|current| std::rc::Rc::ptr_eq(current, &prompt))
        {
            mode_tree_draw(mtd);
            (*(*mtd).wp).flags |= PANE_REDRAW;
        }
        if result as ::core::ffi::c_uint
            != PROMPT_KEY_NOT_HANDLED as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            *key = KEYC_NONE as ::core::ffi::c_ulong as key_code;
            return 0 as ::core::ffi::c_int;
        }
    }
    if (*mtd).help != 0 {
        if *key & KEYC_MASK_KEY == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
            || *key & KEYC_MASK_TYPE
                >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int
                && *key & KEYC_MASK_TYPE
                    <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                        << 32 as ::core::ffi::c_int
        {
            *key = KEYC_NONE as ::core::ffi::c_ulong as key_code;
            return 0 as ::core::ffi::c_int;
        }
        if *key == KEYC_FOCUS_IN as ::core::ffi::c_ulong as key_code
            || *key == KEYC_FOCUS_OUT as ::core::ffi::c_ulong as key_code
        {
            *key = KEYC_NONE as ::core::ffi::c_ulong as key_code;
            return 0 as ::core::ffi::c_int;
        }
        (*mtd).help = 0 as ::core::ffi::c_int;
        mode_tree_draw(mtd);
        *key = KEYC_NONE as ::core::ffi::c_ulong as key_code;
        return 0 as ::core::ffi::c_int;
    }
    if (*key & KEYC_MASK_KEY == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
        || *key & KEYC_MASK_TYPE
            >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                << 32 as ::core::ffi::c_int
            && *key & KEYC_MASK_TYPE
                <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int)
        && !m.is_null()
    {
        if cmd_mouse_at(
            (*mtd).wp,
            m,
            &raw mut x,
            &raw mut y,
            0 as ::core::ffi::c_int,
        ) != 0 as ::core::ffi::c_int
        {
            *key = KEYC_NONE as ::core::ffi::c_ulong as key_code;
            return 0 as ::core::ffi::c_int;
        }
        if !xp.is_null() {
            *xp = x;
        }
        if !yp.is_null() {
            *yp = y;
        }
        if x > (*mtd).width || y > (*mtd).height {
            preview = (*mtd).preview;
            if *key == KEYC_MOUSEDOWN3_PANE as ::core::ffi::c_ulong as key_code {
                mode_tree_display_menu(mtd, c, x, y, 1 as ::core::ffi::c_int);
            }
            if preview == MODE_TREE_PREVIEW_OFF as ::core::ffi::c_int {
                *key = KEYC_NONE as ::core::ffi::c_ulong as key_code;
            }
            return 0 as ::core::ffi::c_int;
        }
        if (*mtd).offset.wrapping_add(y) < mode_tree_line_count(&*mtd) {
            if *key == KEYC_MOUSEDOWN1_PANE as ::core::ffi::c_ulong as key_code
                || *key == KEYC_MOUSEDOWN3_PANE as ::core::ffi::c_ulong as key_code
                || *key == KEYC_DOUBLECLICK1_PANE as ::core::ffi::c_ulong as key_code
            {
                (*mtd).current = (*mtd).offset.wrapping_add(y);
            }
            if *key == KEYC_DOUBLECLICK1_PANE as ::core::ffi::c_ulong as key_code {
                *key = '\r' as i32 as key_code;
            } else {
                if *key == KEYC_MOUSEDOWN3_PANE as ::core::ffi::c_ulong as key_code {
                    mode_tree_display_menu(mtd, c, x, y, 0 as ::core::ffi::c_int);
                }
                *key = KEYC_NONE as ::core::ffi::c_ulong as key_code;
            }
        } else {
            if *key == KEYC_MOUSEDOWN3_PANE as ::core::ffi::c_ulong as key_code {
                mode_tree_display_menu(mtd, c, x, y, 0 as ::core::ffi::c_int);
            }
            *key = KEYC_NONE as ::core::ffi::c_ulong as key_code;
        }
        return 0 as ::core::ffi::c_int;
    }
    let line = (&(*mtd).lines)[(*mtd).current as usize].clone();
    let current = &line.item;
    choice = -(1 as ::core::ffi::c_int);
    i = 0 as u_int;
    while i < mode_tree_line_count(&*mtd) {
        if *key == (&(*mtd).lines)[i as usize].item.borrow().key {
            choice = i as ::core::ffi::c_int;
            break;
        } else {
            i = i.wrapping_add(1);
        }
    }
    if choice != -(1 as ::core::ffi::c_int) {
        if choice as u_int > mode_tree_line_count(&*mtd).wrapping_sub(1 as u_int) {
            *key = KEYC_NONE as ::core::ffi::c_ulong as key_code;
            return 0 as ::core::ffi::c_int;
        }
        (*mtd).current = choice as u_int;
        *key = '\r' as i32 as key_code;
        return 0 as ::core::ffi::c_int;
    }
    match *key {
        113 | 27 | 35184372088923 | 35184372088935 => return 1 as ::core::ffi::c_int,
        8589934600 | 35184372088936 => {
            mode_tree_display_help(mtd);
        }
        8589934619 | 107 | 38654705664 | 35184372088944 => {
            mode_tree_up(mtd, 1 as ::core::ffi::c_int);
        }
        8589934620 | 106 | 34359738368 | 35184372088942 => {
            mode_tree_down(mtd, 1 as ::core::ffi::c_int);
        }
        70377334112283 | 75 => {
            mode_tree_swap(mtd, -(1 as ::core::ffi::c_int));
        }
        70377334112284 | 74 => {
            mode_tree_swap(mtd, 1 as ::core::ffi::c_int);
        }
        8589934617 | 35184372088930 => {
            i = 0 as u_int;
            while i < (*mtd).height {
                if (*mtd).current == 0 as u_int {
                    break;
                }
                mode_tree_up(mtd, 1 as ::core::ffi::c_int);
                i = i.wrapping_add(1);
            }
        }
        8589934616 | 35184372088934 => {
            i = 0 as u_int;
            while i < (*mtd).height {
                if (*mtd).current == mode_tree_line_count(&*mtd).wrapping_sub(1 as u_int) {
                    break;
                }
                mode_tree_down(mtd, 1 as ::core::ffi::c_int);
                i = i.wrapping_add(1);
            }
        }
        103 | 8589934614 => {
            (*mtd).current = 0 as u_int;
            (*mtd).offset = 0 as u_int;
        }
        71 | 8589934615 => {
            (*mtd).current = mode_tree_line_count(&*mtd).wrapping_sub(1 as u_int);
            if (*mtd).current > (*mtd).height.wrapping_sub(1 as u_int) {
                (*mtd).offset = (*mtd)
                    .current
                    .wrapping_sub((*mtd).height)
                    .wrapping_add(1 as u_int);
            } else {
                (*mtd).offset = 0 as u_int;
            }
        }
        116 => {
            let mut row = current.borrow_mut();
            if row.no_tag == 0 {
                if row.tagged == 0 {
                    let mut parent = row.parent.upgrade();
                    while let Some(owner) = parent {
                        let mut row = owner.borrow_mut();
                        row.tagged = 0;
                        parent = row.parent.upgrade();
                    }
                    mode_tree_clear_tagged(&row.children);
                    row.tagged = 1;
                } else {
                    row.tagged = 0;
                }
                drop(row);
                if !m.is_null() {
                    mode_tree_down(mtd, 0);
                }
            }
        }
        84 => {
            for line in &(*mtd).lines {
                line.item.borrow_mut().tagged = 0;
            }
        }
        35184372088948 => {
            for line in &(*mtd).lines {
                let mut row = line.item.borrow_mut();
                row.tagged = if let Some(parent) = row.parent.upgrade() {
                    (parent.borrow().no_tag != 0) as i32
                } else {
                    (row.no_tag == 0) as i32
                };
            }
        }
        79 => {
            sort_next_order(&raw mut (*mtd).sort_crit);
            mode_tree_build(mtd);
        }
        114 => {
            (*mtd).sort_crit.reversed = ((*mtd).sort_crit.reversed == 0) as ::core::ffi::c_int;
            mode_tree_build(mtd);
        }
        8589934621 | 104 | 45 => {
            let target = if line.flat != 0 || current.borrow().expanded == 0 {
                current.borrow().parent.upgrade()
            } else {
                Some(Rc::clone(current))
            };
            if let Some(target) = target {
                let mut row = target.borrow_mut();
                row.expanded = 0;
                (*mtd).current = row.line;
                drop(row);
                mode_tree_build(mtd);
            } else {
                mode_tree_up(mtd, 0);
            }
        }
        8589934622 | 108 | 43 => {
            if line.flat != 0 || current.borrow().expanded != 0 {
                mode_tree_down(mtd, 0);
            } else {
                current.borrow_mut().expanded = 1;
                mode_tree_build(mtd);
            }
        }
        17592186044461 => {
            for item in &(*mtd).children.items {
                item.borrow_mut().expanded = 0;
            }
            mode_tree_build(mtd);
        }
        17592186044459 => {
            for item in &(*mtd).children.items {
                item.borrow_mut().expanded = 1;
            }
            mode_tree_build(mtd);
        }
        63 | 47 | 35184372088947 => {
            (*mtd).search_dir = MODE_TREE_SEARCH_FORWARD;
            mode_tree_set_prompt(
                mtd,
                c,
                c"(search) ",
                Some(c""),
                PROMPT_TYPE_SEARCH,
                PROMPT_NOFORMAT,
                Some(Box::new(move |_, s, key| unsafe {
                    mode_tree_search_callback(mtd, s, key)
                })),
                None,
            );
        }
        110 => {
            (*mtd).search_dir = MODE_TREE_SEARCH_FORWARD;
            mode_tree_search_set(mtd);
        }
        78 => {
            (*mtd).search_dir = MODE_TREE_SEARCH_BACKWARD;
            mode_tree_search_set(mtd);
        }
        102 => {
            mode_tree_set_prompt(
                mtd,
                c,
                c"(filter) ",
                Some((*mtd).filter.as_deref().unwrap_or(c"")),
                PROMPT_TYPE_SEARCH,
                PROMPT_NOFORMAT,
                Some(Box::new(move |_, s, key| unsafe {
                    mode_tree_filter_callback(mtd, s, key)
                })),
                None,
            );
        }
        99 => {
            mode_tree_clear_prompt(mtd);
            mode_tree_clear_filter(mtd);
        }
        118 => {
            match (*mtd).preview {
                0 => {
                    (*mtd).preview = MODE_TREE_PREVIEW_BIG as ::core::ffi::c_int;
                }
                1 => {
                    (*mtd).preview = MODE_TREE_PREVIEW_OFF as ::core::ffi::c_int;
                }
                2 => {
                    (*mtd).preview = MODE_TREE_PREVIEW_NORMAL as ::core::ffi::c_int;
                }
                _ => {}
            }
            mode_tree_build(mtd);
            if (*mtd).preview != MODE_TREE_PREVIEW_OFF as ::core::ffi::c_int {
                mode_tree_check_selected(mtd);
            }
        }
        _ => {}
    }
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn mode_tree_run_command(
    c: *mut client,
    fs: Option<&cmd_find_state>,
    template: &CStr,
    name: &CStr,
) {
    let command = cmd_template_replace_cstring(template, name, 1);
    if command.as_bytes().is_empty() {
        return;
    }
    // The legacy constructor only reads and copies the supplied find state.
    let state = crate::src::shared::rc::take(cmdq_new_state(
        fs.map_or(std::ptr::null_mut(), |fs| std::ptr::from_ref(fs).cast_mut()),
        std::ptr::null_mut(),
        0,
    ));
    if let Err(mut error) =
        cmd_parse_and_append(&command, c, crate::src::shared::rc::as_ptr(&state))
    {
        if !c.is_null() {
            cmd_parse_error_uppercase_first(&mut error);
            status_message_set(c, -1, 1, 0, 0, |out| {
                write_cstr(
                    out,
                    error
                        .as_ref()
                        .map_or(std::ptr::null(), |cause| cause.as_ptr()),
                )
            });
        }
    }
}

#[cfg(test)]
mod mode_tree_tests {
    use super::*;
    use crate::src::shared::sort::sort_criteria;

    #[test]
    fn child_owners_preserve_search_order_and_survive_removal() {
        unsafe {
            let mtd = mode_tree_alloc_data();
            let add = move |parent, tag, name: &CStr| {
                mode_tree_add(mtd, parent, ModeTreeItemData::None, tag, name, None, 1)
            };
            let first = add(None, 1, c"match first");
            let branch = add(None, 2, c"branch");
            let child = add(Some(&branch), 3, c"match child");
            let tail = add(None, 4, c"match tail");
            // Force growth while existing row pointers remain live.
            for tag in 5..100 {
                add(None, tag, c"other");
            }
            branch.borrow_mut().expanded = 0;
            mode_tree_build_lines(mtd, &(*mtd).children.items.clone(), 0);
            (*mtd).search = Some(c"match".to_owned());
            (*mtd).current = first.borrow().line;
            assert!(Rc::ptr_eq(&mode_tree_search_forward(mtd).unwrap(), &child));
            assert!(Rc::ptr_eq(&mode_tree_search_backward(mtd).unwrap(), &tail));
            (*mtd).current = tail.borrow().line;
            assert!(Rc::ptr_eq(&mode_tree_search_backward(mtd).unwrap(), &child));
            assert!(Rc::ptr_eq(&mode_tree_search_forward(mtd).unwrap(), &first));
            (*mtd).search = Some(c"missing".to_owned());
            assert!(mode_tree_search_forward(mtd).is_none());
            assert!(mode_tree_search_backward(mtd).is_none());

            mode_tree_remove(mtd, &child);
            assert!(branch.borrow().children.items.is_empty());
            add(Some(&branch), 100, c"replacement child");
            mode_tree_remove(mtd, &branch);
            mode_tree_remove(mtd, &first);
            let last = (*mtd).children.last().unwrap();
            mode_tree_remove(mtd, &last);
            assert!(Rc::ptr_eq(&(*mtd).children.first().unwrap(), &tail));
            assert!(mode_tree_find_item(&(*mtd).children, 100).is_none());
            mode_tree_clear_lines(mtd);
            mode_tree_build_lines(mtd, &(*mtd).children.items.clone(), 0);
            assert_eq!((*mtd).lines.len(), 95);
            assert_eq!((*mtd).lines.last().unwrap().last, 1);
            mode_tree_remove_ref(mtd);
        }
    }

    #[test]
    fn item_key_label_tracks_repeated_line_builds() {
        unsafe {
            let mtd = mode_tree_alloc_data();
            let mut key = b'x' as key_code;
            let key_ptr = &raw const key;
            (*mtd).keycb = Some(Box::new(move |_, _| *key_ptr));
            let item = mode_tree_add(mtd, None, ModeTreeItemData::None, 1, c"row", None, 1);
            for (next, expected) in [
                (b'x' as key_code, Some(b"x".as_slice())),
                (KEYC_NONE, None),
                (b'y' as key_code, Some(b"y".as_slice())),
            ] {
                key = next;
                mode_tree_clear_lines(mtd);
                mode_tree_build_lines(mtd, &(*mtd).children.items.clone(), 0);
                match expected {
                    Some(expected) => {
                        assert_eq!(
                            CStr::from_ptr(
                                (item.borrow().keystr)
                                    .as_ref()
                                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut())
                            )
                            .to_bytes(),
                            expected
                        );
                        assert_eq!(item.borrow().keylen, expected.len());
                    }
                    None => {
                        assert!(item.borrow().keystr.is_none());
                        assert_eq!(item.borrow().keylen, 0);
                    }
                }
            }
            mode_tree_free_items(&mut (*mtd).children);
            mode_tree_clear_lines(mtd);
            mode_tree_remove_ref(mtd);
        }
    }

    struct NestedBuildState {
        mtd: *mut mode_tree_data,
        empty: bool,
    }

    unsafe fn nested_build(state: &mut NestedBuildState) {
        if state.empty {
            return;
        }
        let parent = mode_tree_add(
            state.mtd,
            None,
            ModeTreeItemData::None,
            1,
            c"parent",
            None,
            1,
        );
        for id in 0..64 {
            mode_tree_add(
                state.mtd,
                Some(&parent),
                ModeTreeItemData::None,
                id as u64 + 2,
                c"child",
                None,
                1,
            );
        }
    }

    #[test]
    fn nested_lines_survive_growth_and_clear_on_empty_rebuild() {
        unsafe {
            let mtd = mode_tree_alloc_data();
            (*mtd).preview = MODE_TREE_PREVIEW_OFF as ::core::ffi::c_int;
            let state = Box::into_raw(Box::new(NestedBuildState { mtd, empty: false }));
            (*mtd).buildcb = Some(Box::new(move |_, _, _| {
                nested_build(&mut *state);
                None
            }));
            (*mtd).screen.grid = Some(crate::src::grid::grid_create(80, 24, 0));

            mode_tree_build(mtd);
            assert_eq!((*mtd).lines.len(), 65);
            assert_eq!((&(*mtd).lines)[0].depth, 0);
            for i in 1..65 {
                assert_eq!((&(*mtd).lines)[i].depth, 1);
                assert_eq!((&(*mtd).lines)[i].item.borrow().line, i as u_int);
            }
            assert_eq!(mode_tree_set_current(mtd, 65), 1);
            assert_eq!((*mtd).current, 64);

            (*state).empty = true;
            mode_tree_build(mtd);
            assert!((*mtd).lines.is_empty());
            mode_tree_free_items(&mut (*mtd).children);
            drop((*mtd).screen.grid.take());
            mode_tree_remove_ref(mtd);
            drop(Box::from_raw(state));
        }
    }

    #[test]
    fn numeric_tags_restore_saved_row_state() {
        unsafe {
            let mtd = mode_tree_alloc_data();
            let prior = mode_tree_add(mtd, None, ModeTreeItemData::None, 7, c"row", None, 1);
            prior.borrow_mut().tagged = 1;
            prior.borrow_mut().expanded = 0;
            (*mtd).saved = std::mem::take(&mut (*mtd).children);

            let restored = mode_tree_add(mtd, None, ModeTreeItemData::None, 7, c"row", None, 1);
            let different = mode_tree_add(mtd, None, ModeTreeItemData::None, 8, c"other", None, 1);
            assert_eq!(
                (restored.borrow().tagged, restored.borrow().expanded),
                (1, 0)
            );
            assert_eq!(
                (different.borrow().tagged, different.borrow().expanded),
                (0, 1)
            );

            mode_tree_free_items(&mut (*mtd).children);
            mode_tree_free_items(&mut (*mtd).saved);
            mode_tree_remove_ref(mtd);
        }
    }
}

#[cfg(test)]
mod mode_prompt_data_tests {
    use super::*;
    use crate::src::shared::rc;
    use crate::src::text::utf8::utf8_fromcstr_vec;
    use std::cell::{Cell, UnsafeCell};

    fn data(tree: &Rc<UnsafeCell<mode_tree_data>>) -> ModeTreePromptRef {
        Rc::new(RefCell::new(mode_tree_prompt {
            mtd: Some(tree.clone()),
            c: Weak::new(),
            inputcb: None,
            freecb: None,
        }))
    }

    #[test]
    fn cleanup_releases_tree_ownership_without_clearing_replacement_data() {
        unsafe {
            let tree = rc::take(mode_tree_alloc_data());
            let mtd = rc::as_ptr(&tree);
            let weak_tree = Rc::downgrade(&tree);
            let old = data(&tree);
            let replacement = data(&tree);
            (*mtd).prompt_data = Rc::downgrade(&replacement);
            let frees = Rc::new(Cell::new(0));
            let count = frees.clone();
            let weak_old = Rc::downgrade(&old);
            let observed = weak_old.clone();
            old.borrow_mut().freecb = Some(Box::new(move || {
                // No record borrow may span the user cleanup callback.
                assert!(observed.upgrade().unwrap().borrow_mut().mtd.is_none());
                count.set(count.get() + 1);
            }));
            assert_eq!(Rc::strong_count(&tree), 3);
            mode_tree_prompt_free_callback(&old);
            assert_eq!(Rc::strong_count(&tree), 2);
            assert!(old.borrow().mtd.is_none());
            assert!((*mtd).prompt_data.ptr_eq(&Rc::downgrade(&replacement)));
            mode_tree_prompt_free_callback(&old);
            assert_eq!(frees.get(), 1);
            assert_eq!(Rc::strong_count(&tree), 2);
            mode_tree_prompt_free_callback(&replacement);
            assert!((*mtd).prompt_data.upgrade().is_none());
            assert_eq!(Rc::strong_count(&tree), 1);
            drop(tree);
            // Retaining the closed callback records does not retain the tree.
            assert!(weak_tree.upgrade().is_none());
            drop(old);
            assert!(weak_old.upgrade().is_none());
        }
    }

    #[test]
    fn active_callback_retains_tree_until_deferred_prompt_cleanup() {
        for (dispatch, retain_tree) in [(false, false), (true, false), (true, true)] {
            unsafe {
                let tree = rc::take(mode_tree_alloc_data());
                let weak_tree = Rc::downgrade(&tree);
                let retained_tree = retain_tree.then(|| tree.clone());
                let mtd = rc::as_ptr(&tree);
                let data = data(&tree);
                let weak_data = Rc::downgrade(&data);
                let tree_slot = Rc::new(RefCell::new(Some(tree)));
                let pr = Rc::new(RefCell::new(prompt {
                    buffer: utf8_fromcstr_vec(c""),
                    flags: PROMPT_SINGLE,
                    ..Default::default()
                }));
                let input_data = data.clone();
                pr.borrow_mut().inputcb = Some(Box::new(move |text, key| {
                    mode_tree_prompt_input_callback(&input_data, text, key)
                }));
                let free_data = data.clone();
                pr.borrow_mut().freecb =
                    Some(Box::new(move || mode_tree_prompt_free_callback(&free_data)));
                (*mtd).prompt = Some(pr.clone());
                (*mtd).prompt_data = Rc::downgrade(&data);
                (*mtd).lines.push(mode_tree_line {
                    item: Rc::new(RefCell::new(mode_tree_item::empty())),
                    depth: 0,
                    last: 0,
                    flat: 0,
                });
                let events = Rc::new(RefCell::new(Vec::new()));
                let input_events = events.clone();
                let slot = tree_slot.clone();
                let observed = weak_tree.clone();
                data.borrow_mut().inputcb = Some(Box::new(move |client, input, key| {
                    assert!(client.is_none());
                    assert_eq!(input, Some(c"x"));
                    assert_eq!(key, PROMPT_KEY_CLOSE);
                    input_events.borrow_mut().push("start");
                    let tree = slot.borrow_mut().take().unwrap();
                    let mtd = rc::as_ptr(&tree);
                    mode_tree_clear_prompt(mtd);
                    (*mtd).dead = 1;
                    (*mtd).lines.clear();
                    drop(tree);
                    assert!(observed.upgrade().is_some());
                    input_events.borrow_mut().push("end");
                    PROMPT_CLOSE
                }));
                let free_events = events.clone();
                let observed = weak_tree.clone();
                data.borrow_mut().freecb = Some(Box::new(move || {
                    assert!(observed.upgrade().is_some());
                    free_events.borrow_mut().push("freed");
                }));
                if dispatch {
                    let mut key = b'x' as key_code;
                    assert_eq!(
                        mode_tree_key(
                            mtd,
                            std::ptr::null_mut(),
                            &mut key,
                            std::ptr::null_mut(),
                            std::ptr::null_mut(),
                            std::ptr::null_mut()
                        ),
                        0
                    );
                    assert_eq!(key, KEYC_NONE);
                } else {
                    assert_eq!(prompt_key(&pr, b'x' as key_code, &mut 0), PROMPT_KEY_CLOSE);
                }
                assert_eq!(&*events.borrow(), &["start", "end", "freed"]);
                assert!(tree_slot.borrow().is_none());
                drop(retained_tree);
                assert!(weak_tree.upgrade().is_none());
                assert!(data.borrow().mtd.is_none());
                assert!(data.borrow().inputcb.is_none());
                assert_eq!(Rc::strong_count(&events), 1);
                drop(pr);
                drop(data);
                assert!(weak_data.upgrade().is_none());
            }
        }
    }
}

#[cfg(test)]
mod queued_prompt_accept_tests {
    use super::*;
    use crate::src::cmd::queue::cmdq_free_detached;
    use crate::src::shared::rc;
    use std::ptr::NonNull;

    #[test]
    fn queued_acceptance_releases_its_tree_when_fired_or_cancelled() {
        for fire in [false, true] {
            unsafe {
                let tree = rc::take(mode_tree_alloc_data());
                let observed = Rc::downgrade(&tree);
                let item = cmdq_get_callback_owned(
                    c"test-mode-accept".as_ptr(),
                    mode_tree_prompt_accept(tree),
                );
                assert!(observed.upgrade().is_some());
                if fire {
                    (*item).flags |= CMDQ_FIRED;
                    let callback = (*item).cb.take().unwrap();
                    assert_eq!(callback(NonNull::new(item).unwrap()), CMD_RETURN_NORMAL);
                    // The item may remain queued, but its fired capture is gone.
                    assert!(observed.upgrade().is_none());
                }
                cmdq_free_detached(item);
                assert!(observed.upgrade().is_none());
            }
        }
    }
}

#[cfg(test)]
mod menu_callback_owner_tests {
    use super::*;
    use crate::src::shared::rc;
    use std::cell::Cell;

    unsafe fn one_line(tree: &Rc<UnsafeCell<mode_tree_data>>) {
        (*rc::as_ptr(tree)).lines.push(mode_tree_line {
            item: Rc::new(RefCell::new(mode_tree_item::empty())),
            depth: 0,
            last: 1,
            flat: 0,
        });
    }

    #[test]
    fn callbacks_release_the_tree_on_selection_cancellation_and_discard() {
        for outcome in 0..6 {
            unsafe {
                let tree = rc::take(mode_tree_alloc_data());
                let observer = Rc::downgrade(&tree);
                one_line(&tree);
                if outcome == 4 {
                    (*rc::as_ptr(&tree)).dead = 1;
                }
                let client = client::new();
                let client_observer = Rc::downgrade(&client);
                let calls = Rc::new(Cell::new(0));
                let callback_calls = Rc::clone(&calls);
                let expected_client = client_observer.clone();
                (*rc::as_ptr(&tree)).menucb = Some(Box::new(move |client, key| {
                    assert!(Rc::ptr_eq(client, &expected_client.upgrade().unwrap()));
                    assert_eq!(key, b't' as key_code);
                    callback_calls.set(callback_calls.get() + 1);
                }));
                let callback = mode_tree_menu_callback(
                    tree,
                    client_observer,
                    if outcome == 3 { 1 } else { 0 },
                );
                assert!(observer.upgrade().is_some());
                assert_eq!(Rc::strong_count(&client), 1);
                if outcome == 0 {
                    drop(callback);
                } else {
                    callback.unwrap()(if outcome == 1 {
                        MenuSelection::Cancelled
                    } else {
                        MenuSelection::Selected {
                            index: 0,
                            key: if outcome == 2 {
                                KEYC_NONE
                            } else {
                                b't' as key_code
                            },
                        }
                    });
                }
                assert_eq!(calls.get(), if outcome == 5 { 1 } else { 0 });
                assert!(observer.upgrade().is_none());
                assert_eq!(Rc::strong_count(&calls), 1);
                assert_eq!(Rc::strong_count(&client), 1);
            }
        }
    }

    #[test]
    fn expired_and_disconnected_clients_cannot_dispatch_menu_actions() {
        for expired in [false, true] {
            unsafe {
                let tree = rc::take(mode_tree_alloc_data());
                one_line(&tree);
                let mtd = rc::as_ptr(&tree);
                (*mtd).current = 7;
                let calls = Rc::new(Cell::new(0));
                let callback_calls = Rc::clone(&calls);
                (*mtd).menucb = Some(Box::new(move |_, _| {
                    callback_calls.set(callback_calls.get() + 1)
                }));
                let mut client = Some(client::new());
                let observer = Rc::downgrade(client.as_ref().unwrap());
                let callback =
                    mode_tree_menu_callback(Rc::clone(&tree), observer.clone(), 0).unwrap();
                if expired {
                    drop(client.take());
                    assert!(observer.upgrade().is_none());
                } else {
                    (*rc::as_ptr(client.as_ref().unwrap())).flags |= CLIENT_DEAD as uint64_t;
                }
                callback(MenuSelection::Selected {
                    index: 0,
                    key: b't' as key_code,
                });
                assert_eq!(calls.get(), 0);
                assert_eq!((*mtd).current, 7);
                drop(tree);
                assert_eq!(Rc::strong_count(&calls), 1);
            }
        }
    }

    #[test]
    fn menu_handlers_can_close_the_mode_or_replace_themselves() {
        for close_mode in [false, true] {
            unsafe {
                let mtd = mode_tree_alloc_data();
                let observer = rc::downgrade(mtd);
                let mut pane = window_pane::empty();
                (*mtd).wp = &mut pane;
                (*mtd).zoomed = 1;
                one_line(&observer.upgrade().unwrap());
                let calls = Rc::new(Cell::new(0));
                let callback_calls = Rc::clone(&calls);
                let callback_tree = observer.clone();
                (*mtd).menucb = Some(Box::new(move |_, _| {
                    let retained = callback_tree.upgrade().unwrap();
                    let mtd = rc::as_ptr(&retained);
                    assert!((*mtd).menucb.is_none());
                    assert_eq!((*mtd).current, 0);
                    callback_calls.set(callback_calls.get() + 1);
                    if close_mode {
                        mode_tree_free(mtd);
                    } else {
                        let replacement_calls = Rc::clone(&callback_calls);
                        (*mtd).menucb = Some(Box::new(move |_, _| {
                            replacement_calls.set(replacement_calls.get() + 10);
                        }));
                    }
                }));
                let client = client::new();
                let callback =
                    mode_tree_menu_callback(observer.upgrade().unwrap(), Rc::downgrade(&client), 0)
                        .unwrap();
                callback(MenuSelection::Selected {
                    index: 0,
                    key: b't' as key_code,
                });
                assert_eq!(calls.get(), 1);
                if !close_mode {
                    mode_tree_menu_callback(observer.upgrade().unwrap(), Rc::downgrade(&client), 0)
                        .unwrap()(MenuSelection::Selected {
                        index: 0,
                        key: b't' as key_code,
                    });
                    assert_eq!(calls.get(), 11);
                    mode_tree_free(mtd);
                }
                assert!(observer.upgrade().is_none());
                assert_eq!(Rc::strong_count(&calls), 1);
            }
        }
    }
}

#[cfg(test)]
mod row_owner_tests {
    use super::*;
    use crate::src::shared::rc;

    #[test]
    fn retained_rows_and_name_snapshots_do_not_keep_parents_alive() {
        unsafe {
            let tree = mode_tree_alloc_data();
            let observer = rc::downgrade(tree);
            let mut pane = window_pane::empty();
            (*tree).wp = &mut pane;
            (*tree).zoomed = 1;
            let parent = mode_tree_add(tree, None, ModeTreeItemData::None, 1, c"parent", None, 1);
            let name = CString::new(b"child \xff".to_vec()).unwrap();
            let text = CString::new("description").unwrap();
            let child = mode_tree_add(
                tree,
                Some(&parent),
                ModeTreeItemData::None,
                2,
                &name,
                Some(&text),
                1,
            );
            drop(name);
            drop(text);
            let parent_observer = Rc::downgrade(&parent);
            let child_observer = Rc::downgrade(&child);
            mode_tree_build_lines(tree, &(*tree).children.items.clone(), 0);
            (*tree).current = 1;
            let selected = (&(*tree).lines)[1].clone();
            let name = mode_tree_get_current_name(&*tree);
            let name_observer = Rc::downgrade(&name);
            drop(parent);
            drop(child);
            mode_tree_free(tree);
            assert!(observer.upgrade().is_none());
            assert!(parent_observer.upgrade().is_none());
            assert!(selected.item.borrow().parent.upgrade().is_none());
            assert_eq!(selected.item.borrow().text.as_deref(), Some(c"description"));
            assert!(child_observer.upgrade().is_some());
            drop(selected);
            assert!(child_observer.upgrade().is_none());
            assert_eq!(name.to_bytes(), b"child \xff");
            drop(name);
            assert!(name_observer.upgrade().is_none());
        }
    }

    #[test]
    fn tag_callbacks_can_clear_rows_without_borrowing_or_lifetime_conflicts() {
        for tagged in [false, true] {
            unsafe {
                let owner = rc::take(mode_tree_alloc_data());
                let tree = rc::as_ptr(&owner);
                let first = mode_tree_add(tree, None, ModeTreeItemData::None, 1, c"first", None, 1);
                first.borrow_mut().tagged = tagged as i32;
                let second = mode_tree_add(tree, None, ModeTreeItemData::None, 2, c"second", None, 1);
                second.borrow_mut().tagged = tagged as i32;
                let first_observer = Rc::downgrade(&first);
                let second_observer = Rc::downgrade(&second);
                mode_tree_build_lines(tree, &(*tree).children.items.clone(), 0);
                drop(first);
                drop(second);
                let mut calls = 0;
                mode_tree_each_tagged(
                    tree,
                    |row, _, _| {
                        calls += 1;
                        row.borrow_mut().tagged = 0;
                        mode_tree_clear_lines(tree);
                        mode_tree_free_items(&mut (*tree).children);
                        assert!(first_observer.upgrade().is_some());
                        assert!(second_observer.upgrade().is_none());
                        assert_eq!(row.borrow().name.as_ref(), c"first");
                    },
                    std::ptr::null_mut(),
                    KEYC_NONE,
                    1,
                );
                assert_eq!(calls, 1);
                assert!(first_observer.upgrade().is_none());
            }
        }
    }
}

#[cfg(test)]
mod payload_owner_tests {
    use super::*;
    use crate::src::shared::rc;
    use crate::src::window_buffer::window_buffer_itemdata;

    fn payload(name: &CStr) -> ModeTreeItemData {
        ModeTreeItemData::Buffer(Rc::new(UnsafeCell::new(window_buffer_itemdata {
            name: name.to_owned(),
            order: 0,
            size: 0,
        })))
    }

    #[test]
    fn selected_payload_survives_replacement_and_mode_cleanup() {
        unsafe {
            let tree = mode_tree_alloc_data();
            let mut pane = window_pane::empty();
            (*tree).wp = &mut pane;
            (*tree).zoomed = 1;
            assert!(matches!(
                mode_tree_get_current(&*tree),
                ModeTreeItemData::None
            ));
            let original = payload(c"original");
            let observer = Rc::downgrade(original.as_buffer().unwrap());
            let row = mode_tree_add(tree, None, original.clone(), 1, c"row", None, 1);
            // Detail rows may share the same record with their parent.
            mode_tree_add(tree, Some(&row), original, 2, c"detail", None, 1);
            drop(row);
            mode_tree_build_lines(tree, &(*tree).children.items.clone(), 0);
            let selected = mode_tree_get_current(&*tree);
            (*tree).current = 1;
            let detail = mode_tree_get_current(&*tree);
            assert!(Rc::ptr_eq(
                selected.as_buffer().unwrap(),
                detail.as_buffer().unwrap()
            ));

            mode_tree_clear_lines(tree);
            mode_tree_free_items(&mut (*tree).children);
            mode_tree_add(tree, None, payload(c"replacement"), 1, c"row", None, 1);
            mode_tree_build_lines(tree, &(*tree).children.items.clone(), 0);
            (*tree).current = 0;
            let replacement = mode_tree_get_current(&*tree);
            let replacement_observer = Rc::downgrade(replacement.as_buffer().unwrap());
            assert!(!Rc::ptr_eq(
                selected.as_buffer().unwrap(),
                replacement.as_buffer().unwrap()
            ));
            drop(replacement);
            mode_tree_free(tree);
            assert!(replacement_observer.upgrade().is_none());
            assert_eq!(
                (*selected.as_buffer().unwrap().get()).name.as_c_str(),
                c"original"
            );
            drop(selected);
            assert!(observer.upgrade().is_some());
            drop(detail);
            assert!(observer.upgrade().is_none());
        }
    }

    #[test]
    fn key_dispatch_retains_payload_without_holding_a_row_borrow() {
        unsafe {
            let owner = rc::take(mode_tree_alloc_data());
            let tree = rc::as_ptr(&owner);
            let original = payload(c"callback snapshot");
            let observer = Rc::downgrade(original.as_buffer().unwrap());
            let row = mode_tree_add(tree, None, original, 1, c"row", None, 1);
            let callback_row = Rc::clone(&row);
            (*tree).keycb = Some(Box::new(move |item, _| {
                callback_row.borrow_mut().itemdata = ModeTreeItemData::None;
                assert_eq!(
                    (*item.as_buffer().unwrap().get()).name.as_c_str(),
                    c"callback snapshot"
                );
                b'x' as key_code
            }));
            mode_tree_build_lines(tree, &(*tree).children.items.clone(), 0);
            assert_eq!(row.borrow().key, b'x' as key_code);
            assert!(matches!(
                mode_tree_get_current(&*tree),
                ModeTreeItemData::None
            ));
            assert!(observer.upgrade().is_none());
        }
    }
}
