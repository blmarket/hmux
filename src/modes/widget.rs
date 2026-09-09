use crate::WindowPane;
use crate::arguments::{args_get_str, args_has};
use crate::cmdq::CmdqStateRef;
use crate::cmd::cmd_mouse_at;
use crate::cmd::cmd_parse_and_append;

use crate::compat::{tolower, toupper};
use crate::grid::grid_default_cell;
use crate::overlay::{menu_add_items, menu_display};
use crate::overlay::{popup_display, popup_write};
use crate::prompt_history::PromptHistoryType;
use crate::screen::{RustScreenWriteCtx, Screen, ScreenWriteCtx, screen_resize};
use crate::server::client_ref_of;

use crate::sort::{RustSortCriteria, SortCriteria};
use crate::status::{status_message_set, status_prompt_set};
pub use crate::types::*;

#[derive(Default)]
#[repr(C)]
pub struct mode_tree_item {
    /// What the tree names this item by. An id outlives nothing: a line or a
    /// parent that names one the tree has given up finds nothing at all.
    pub id: u_int,
    pub parent: Option<u_int>,
    pub itemdata: ModeTreeItemData,
    pub line: u_int,
    pub key: key_code,
    pub keystr: Option<CString>,
    pub keylen: size_t,
    pub tag: uint64_t,
    pub name: Option<CString>,
    pub text: Option<CString>,
    pub expanded: core::ffi::c_int,
    pub tagged: core::ffi::c_int,
    pub draw_as_parent: core::ffi::c_int,
    pub no_tag: core::ffi::c_int,
    pub align: core::ffi::c_int,
    pub children: mode_tree_list,
}

#[derive(Clone)]
pub(crate) struct ModeTreeItemRef {
    tree: ModeTreeDataRef,
    id: u_int,
}

impl ModeTreeItemRef {
    #[cfg(test)]
    pub(crate) fn id(&self) -> u_int {
        self.id
    }

    pub(crate) fn get(&self) -> Option<std::cell::Ref<'_, mode_tree_item>> {
        std::cell::Ref::filter_map(self.tree.borrow(), |tree| mode_tree_item_ref(tree, self.id))
            .ok()
    }

    pub(crate) fn get_mut(&self) -> Option<std::cell::RefMut<'_, mode_tree_item>> {
        std::cell::RefMut::filter_map(self.tree.borrow_mut(), |tree| {
            mode_tree_item_mut(tree, self.id)
        })
        .ok()
    }
}
use crate::cmd::cmd_template_replace;
use crate::fmt_args;
use crate::log::log_debug;
use crate::overlay::menu_create;
use crate::style::style_apply;
use crate::text::{KeyStringCodec, RustKeyStringCodec};
use crate::text::{RustUtf8VisModel, Utf8VisModel};
use crate::xmalloc::xasprintf;
use ::core::ffi::CStr;
use ::std::ffi::CString;
#[repr(C)]
pub struct mode_tree_data {
    pub zoomed: core::ffi::c_int,
    pane_ref: Option<RustWindowPaneWeak>,
    pub modedata: WindowModeData,
    pub menu: &'static [menu_item<'static>],
    pub sort_crit: sort_criteria_t,
    pub buildcb: mode_tree_build_cb,
    pub(crate) drawcb: mode_tree_draw_cb,
    pub searchcb: mode_tree_search_cb,
    pub menucb: mode_tree_menu_cb,
    pub heightcb: mode_tree_height_cb,
    pub keycb: mode_tree_key_cb,
    pub swapcb: mode_tree_swap_cb,
    pub sortcb: mode_tree_sort_cb,
    pub help: mode_tree_help,
    pub children: mode_tree_list,
    pub saved: mode_tree_list,
    /// The id the next item added to the tree is given.
    pub next_item_id: u_int,
    pub line_list: Vec<mode_tree_line>,
    pub depth: u_int,
    pub maxdepth: u_int,
    pub width: u_int,
    pub height: u_int,
    pub offset: u_int,
    pub current: u_int,
    pub preview: core::ffi::c_int,
    pub search: Option<CString>,
    pub filter: Option<CString>,
    pub no_matches: core::ffi::c_int,
    pub search_dir: mode_tree_search_dir,
    pub search_icase: core::ffi::c_int,
}

impl mode_tree_data {
    /// Resolves the pane while both it and the mode are still live.
    pub(crate) fn pane(&self) -> Option<RustWindowPaneWeak> {
        self.pane_ref
            .as_ref()
            .filter(|pane| pane.is_alive())
            .cloned()
            .filter(|pane| pane.window().is_some())
    }

    fn redraw_pane(&self) {
        if let Some(mut pane) = self.pane()
            && let Some(pane) = unsafe { pane.get_mut() }
        {
            *pane.flags_mut() |= PANE_REDRAW;
        }
    }
}
pub type mode_tree_search_dir = core::ffi::c_uint;
pub const MODE_TREE_SEARCH_BACKWARD: mode_tree_search_dir = 1;
pub const MODE_TREE_SEARCH_FORWARD: mode_tree_search_dir = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mode_tree_line {
    pub item: u_int,
    pub depth: u_int,
    pub last: core::ffi::c_int,
    pub flat: core::ffi::c_int,
}
pub use crate::consts::{
    BOX_LINES_DEFAULT, GRID_ATTR_BRIGHT, KEYC_DOUBLECLICK1_PANE, KEYC_MASK_KEY, KEYC_MASK_TYPE,
    KEYC_META, KEYC_MOUSE, KEYC_MOUSEDOWN1_PANE, KEYC_MOUSEDOWN3_PANE, KEYC_NONE,
    KEYC_TYPE_MOUSEMOVE, KEYC_TYPE_TRIPLECLICK, KEYC_UNKNOWN, PANE_REDRAW, POPUP_CLOSEANYKEY,
    POPUP_NOJOB, PROMPT_NOFORMAT, SORT_NAME, WINDOW_ZOOMED,
};
pub type mode_tree_menu_cb = Option<std::rc::Rc<dyn Fn(&mut client, key_code)>>;
pub(crate) type mode_tree_draw_cb = Option<
    std::rc::Rc<dyn Fn(ModeTreeItemData, &mut crate::screen::RustScreenWriteCtx<'_>, u_int, u_int)>,
>;

pub const MODE_TREE_PREVIEW_BIG: mode_tree_preview = 2;
pub const MODE_TREE_PREVIEW_NORMAL: mode_tree_preview = 1;
pub const MODE_TREE_PREVIEW_OFF: mode_tree_preview = 0;
#[repr(C)]
pub struct mode_tree_menu {
    /// The client the menu is showing on, observed rather than held, so that
    /// a client which goes while the menu is up leaves nothing behind.
    pub(crate) client: ClientWeak,
    pub line: u_int,
    owner: ModeTreeDataWeak,
}

pub type mode_tree_preview = core::ffi::c_uint;
pub const UINT64_MAX: core::ffi::c_ulong = 18446744073709551615 as core::ffi::c_ulong;

static mode_tree_menu_items: [menu_item<'static>; 4] = [
    menu_item {
        name: Some(c"Scroll Left"),
        key: '<' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: Some(c"Scroll Right"),
        key: '>' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: Some(c""),
        key: KEYC_NONE as core::ffi::c_ulong as key_code,
        command: None,
    },
    menu_item {
        name: Some(c"Cancel"),
        key: 'q' as i32 as key_code,
        command: None,
    },
];
static mode_tree_help_start: [&CStr; 20] = [
    c"\r\x1B[1m      Up, k \x1B[0m\x0Ex\x0F \x1B[0mMove cursor up\n",
    c"\r\x1B[1m    Down, j \x1B[0m\x0Ex\x0F \x1B[0mMove cursor down\n",
    c"\r\x1B[1m          g \x1B[0m\x0Ex\x0F \x1B[0mGo to top\n",
    c"\r\x1B[1m          G \x1B[0m\x0Ex\x0F \x1B[0mGo to bottom\n",
    c"\r\x1B[1m PPage, C-b \x1B[0m\x0Ex\x0F \x1B[0mPage up\n",
    c"\r\x1B[1m NPage, C-f \x1B[0m\x0Ex\x0F \x1B[0mPage down\n",
    c"\r\x1B[1m    Left, h \x1B[0m\x0Ex\x0F \x1B[0mCollapse %1\n",
    c"\r\x1B[1m   Right, l \x1B[0m\x0Ex\x0F \x1B[0mExpand %1\n",
    c"\r\x1B[1m        M-- \x1B[0m\x0Ex\x0F \x1B[0mCollapse all %1s\n",
    c"\r\x1B[1m        M-+ \x1B[0m\x0Ex\x0F \x1B[0mExpand all %1s\n",
    c"\r\x1B[1m          t \x1B[0m\x0Ex\x0F \x1B[0mToggle %1 tag\n",
    c"\r\x1B[1m          T \x1B[0m\x0Ex\x0F \x1B[0mUntag all %1s\n",
    c"\r\x1B[1m        C-t \x1B[0m\x0Ex\x0F \x1B[0mTag all %1s\n",
    c"\r\x1B[1m        C-s \x1B[0m\x0Ex\x0F \x1B[0mSearch forward\n",
    c"\r\x1B[1m          n \x1B[0m\x0Ex\x0F \x1B[0mRepeat search forward\n",
    c"\r\x1B[1m          N \x1B[0m\x0Ex\x0F \x1B[0mRepeat search backward\n",
    c"\r\x1B[1m          f \x1B[0m\x0Ex\x0F \x1B[0mFilter %1s\n",
    c"\r\x1B[1m          O \x1B[0m\x0Ex\x0F \x1B[0mChange sort order\n",
    c"\r\x1B[1m          r \x1B[0m\x0Ex\x0F \x1B[0mReverse sort order\n",
    c"\r\x1B[1m          v \x1B[0m\x0Ex\x0F \x1B[0mToggle preview\n",
];
static mode_tree_help_end: [&CStr; 1] =
    [c"\r\x1B[1m  q, Escape \x1B[0m\x0Ex\x0F \x1B[0mExit mode\x1B[H"];
pub const MODE_TREE_HELP_DEFAULT_WIDTH: core::ffi::c_int = 39 as core::ffi::c_int;
fn mode_tree_is_lowercase(s: &CStr) -> core::ffi::c_int {
    for &byte in s.to_bytes() {
        if byte != tolower(byte) {
            return 0 as core::ffi::c_int;
        }
    }
    1 as core::ffi::c_int
}
fn mode_tree_item_mut(mtd: &mut mode_tree_data, id: u_int) -> Option<&mut mode_tree_item> {
    item_of_id(&mut mtd.children, id).or_else(|| item_of_id(&mut mtd.saved, id))
}

fn mode_tree_item_ref(mtd: &mode_tree_data, id: u_int) -> Option<&mode_tree_item> {
    item_ref_of_id(&mtd.children, id).or_else(|| item_ref_of_id(&mtd.saved, id))
}

fn item_ref_of_id(mtl: &mode_tree_list, id: u_int) -> Option<&mode_tree_item> {
    for item in mtl {
        if item.id == id {
            return Some(item);
        }
        if let Some(child) = item_ref_of_id(&item.children, id) {
            return Some(child);
        }
    }
    None
}

fn item_of_id(mtl: &mut mode_tree_list, id: u_int) -> Option<&mut mode_tree_item> {
    for item in mtl {
        if item.id == id {
            return Some(item);
        }
        if let Some(child) = item_of_id(&mut item.children, id) {
            return Some(child);
        }
    }
    None
}

fn line_item_ref(mtd: &mode_tree_data, at: u_int) -> Option<&mode_tree_item> {
    let line = mtd.line_list.get(at as usize)?;
    mode_tree_item_ref(mtd, line.item)
}

fn mode_tree_find_item(mtl: &mode_tree_list, tag: uint64_t) -> Option<&mode_tree_item> {
    for item in mtl {
        if item.tag == tag {
            return Some(item);
        }
        if let Some(child) = mode_tree_find_item(&item.children, tag) {
            return Some(child);
        }
    }
    None
}

unsafe fn mode_tree_check_selected(mtd: &mut mode_tree_data) {
    {
        if mtd.current > mtd.height.wrapping_sub(1 as u_int) {
            mtd.offset = mtd
                .current
                .wrapping_sub(mtd.height)
                .wrapping_add(1 as u_int);
        }
    }
}
/// How many lines the tree currently flattens to.
unsafe fn line_count(mtd: &mode_tree_data) -> u_int {
    mtd.line_list.len() as u_int
}
unsafe fn mode_tree_clear_lines(mtd: &mut mode_tree_data) {
    {
        mtd.line_list = Vec::new();
    }
}

fn mode_tree_clear_tagged(items: &mut mode_tree_list) {
    for item in items {
        item.tagged = 0;
        mode_tree_clear_tagged(&mut item.children);
    }
}

fn mode_tree_toggle_tag(tree: &mut mode_tree_data, id: u_int) -> bool {
    let Some(item) = mode_tree_item_mut(tree, id) else {
        return false;
    };
    if item.no_tag != 0 {
        return false;
    }
    if item.tagged != 0 {
        item.tagged = 0;
        return true;
    }
    mode_tree_clear_tagged(&mut item.children);
    item.tagged = 1;
    let mut parent = item.parent;
    while let Some(id) = parent {
        let Some(item) = mode_tree_item_mut(tree, id) else {
            break;
        };
        item.tagged = 0;
        parent = item.parent;
    }
    true
}

fn mode_tree_set_all_tagged(tree: &mut mode_tree_data, tagged: bool) {
    for at in 0..tree.line_list.len() {
        let id = tree.line_list[at].item;
        let Some(item) = mode_tree_item_ref(tree, id) else {
            continue;
        };
        let parent = item.parent.and_then(|id| mode_tree_item_ref(tree, id));
        let allowed = parent.map_or(item.no_tag == 0, |parent| parent.no_tag != 0);
        mode_tree_item_mut(tree, id)
            .expect("the listed item is live")
            .tagged = (tagged && allowed) as core::ffi::c_int;
    }
}

fn mode_tree_set_expanded(mtd: &mut mode_tree_data, at: u_int, expanded: bool) -> bool {
    let Some(id) = mtd.line_list.get(at as usize).map(|line| line.item) else {
        return false;
    };
    let Some(item) = mode_tree_item_mut(mtd, id) else {
        return false;
    };
    if (item.expanded != 0) == expanded {
        return false;
    }
    item.expanded = expanded as core::ffi::c_int;
    true
}

/// The line carrying `tag`, if the tree has one.
fn mode_tree_get_tag(mtd: &mode_tree_data, tag: uint64_t) -> Option<u_int> {
    mtd.line_list
        .iter()
        .position(|line| mode_tree_item_ref(mtd, line.item).is_some_and(|item| item.tag == tag))
        .map(|at| at as u_int)
}

fn mode_tree_insert_item(
    mtd: &mut mode_tree_data,
    parent: Option<u_int>,
    itemdata: ModeTreeItemData,
    tag: uint64_t,
    name: &CStr,
    text: Option<&CStr>,
    expanded: core::ffi::c_int,
) -> Option<u_int> {
    let parent_expanded = match parent {
        Some(id) => mode_tree_item_ref(mtd, id)?.expanded != 0,
        None => true,
    };
    unsafe {
        log_debug(
            c"%s: %llu, %s %s",
            fmt_args![
                c"mode_tree_add",
                tag as core::ffi::c_ulonglong,
                name,
                text.unwrap_or(c"")
            ],
        );
    }
    let id = mtd.next_item_id;
    mtd.next_item_id = id.checked_add(1).expect("mode tree item IDs exhausted");
    let mut item = Box::new(mode_tree_item {
        id,
        parent,
        itemdata,
        tag,
        name: Some(name.to_owned()),
        text: text.map(CStr::to_owned),
        ..Default::default()
    });
    if let Some(saved) = mode_tree_find_item(&mtd.saved, tag) {
        if parent_expanded {
            item.tagged = saved.tagged;
        }
        item.expanded = saved.expanded;
    } else {
        item.expanded = if expanded == -1 { 1 } else { expanded };
    }
    if let Some(parent) = parent {
        mode_tree_item_mut(mtd, parent)
            .expect("the parent was checked")
            .children
            .push(item);
    } else {
        mtd.children.push(item);
    }
    Some(id)
}

fn mode_tree_remove_item(mtd: &mut mode_tree_data, id: u_int) {
    let Some(item) = mode_tree_item_ref(mtd, id) else {
        return;
    };
    let parent = item.parent;
    let siblings = match parent {
        Some(parent) => {
            &mut mode_tree_item_mut(mtd, parent)
                .expect("an item has its parent")
                .children
        }
        None => &mut mtd.children,
    };
    if let Some(at) = siblings.iter().position(|item| item.id == id) {
        siblings.remove(at);
    }
}

/// Appends what fits of `s` to `dst`, the way `strlcat` fills a buffer of
/// `size` bytes counting its terminator.
fn mode_tree_append(dst: &mut Vec<u8>, s: &[u8], size: usize) {
    let room = size.saturating_sub(dst.len() + 1);
    dst.extend_from_slice(&s[..s.len().min(room)]);
}

fn mode_tree_search_ids(items: &mode_tree_list, ids: &mut Vec<u_int>) {
    for item in items {
        ids.push(item.id);
        mode_tree_search_ids(&item.children, ids);
    }
}

#[allow(clippy::boxed_local)]
unsafe fn mode_tree_menu_callback(_idx: u_int, key: key_code, mtm: Box<mode_tree_menu>) {
    unsafe {
        let Some(owner) = mtm.owner.upgrade() else {
            return;
        };
        let Some(mut client) = mtm.client.upgrade() else {
            return;
        };
        let menucb = {
            let mut tree = owner.borrow_mut();
            if tree.pane().is_none() || key == KEYC_NONE || mtm.line >= line_count(&tree) {
                return;
            }
            tree.current = mtm.line;
            tree.menucb.clone().expect("missing menu callback")
        };
        menucb(client.as_client_mut(), key);
    }
}

unsafe fn mode_tree_display_help(help: mode_tree_help, c: &mut client) {
    unsafe {
        let Some(session) = c.attached_session() else {
            return;
        };

        let mut w: u_int = 0;
        let mut lines: &'static [&'static CStr] = &[];
        let mut item: &'static CStr = c"item";
        if let Some(help) = help {
            (lines, w, item) = help;
        }
        if w < MODE_TREE_HELP_DEFAULT_WIDTH as u_int {
            w = MODE_TREE_HELP_DEFAULT_WIDTH as u_int;
        }
        let h = (mode_tree_help_start.len() + lines.len() + mode_tree_help_end.len()) as u_int;
        if c.tty.sx < w || c.tty.sy < h {
            return;
        }
        let px: u_int = c.tty.sx.wrapping_sub(w).wrapping_div(2 as u_int);
        let py: u_int = c.tty.sy.wrapping_sub(h).wrapping_div(2 as u_int);
        if popup_display(
            POPUP_CLOSEANYKEY | POPUP_NOJOB,
            BOX_LINES_DEFAULT,
            None,
            px,
            py,
            w,
            h,
            None,
            None,
            &[],
            None,
            None,
            &mut *c,
            Some(session.as_session()),
            None,
            None,
            None,
        ) != 0 as core::ffi::c_int
        {
            return;
        }
        popup_write(&mut *c, b"\x1b[H\x1b[?25l\x1b[?7l\x1b)0");
        for line in mode_tree_help_start
            .iter()
            .chain(lines)
            .chain(mode_tree_help_end.iter())
        {
            let new_line = cmd_template_replace(line, item, 1);
            popup_write(&mut *c, new_line.as_bytes());
        }
        popup_write(&mut *c, b"\x1b[H");
    }
}

pub unsafe fn mode_tree_run_command(
    c: Option<&mut client>,
    fs: Option<&cmd_find_state>,
    template: &CStr,
    name: &CStr,
) {
    unsafe {
        let mut error = None;
        let command = cmd_template_replace(template, name, 1 as core::ffi::c_int);
        if !command.as_bytes().is_empty() {
            let state = CmdqStateRef::create(fs, None, 0 as core::ffi::c_int);
            cmd_parse_and_append(
                &command,
                None,
                c.as_deref().and_then(crate::server::client_ref_of).as_ref(),
                &state,
                &mut error,
            );
            if let Some(error) = error.as_mut()
                && let Some(c) = c
            {
                uppercase_first_byte(error);
                status_message_set(
                    Some(c),
                    -(1 as core::ffi::c_int),
                    1 as core::ffi::c_int,
                    0 as core::ffi::c_int,
                    0 as core::ffi::c_int,
                    c"%s",
                    fmt_args![error.as_c_str()],
                );
            }
        }
    }
}

fn uppercase_first_byte(error: &mut CString) {
    let mut bytes = error.as_bytes().to_vec();
    if let Some(first) = bytes.first_mut() {
        *first = toupper(*first);
    }
    *error = CString::new(bytes).expect("parser errors contain no NUL");
}

#[cfg(test)]
#[path = "../tests/test_widget_focused.rs"]
mod focused_tests;
use crate::screen::RustScreen;

impl ModeTreeDataRef {
    unsafe fn build_lines(&self) {
        let mtd = self;

        unsafe {
            let owner = mtd.clone();
            {
                let mut tree = owner.borrow_mut();
                mode_tree_clear_lines(&mut tree);
                tree.maxdepth = 0;
            }
            owner.build_lines_at(None, 0);
            let mut tree = owner.borrow_mut();
            let lines = core::mem::take(&mut tree.line_list);
            for line in lines {
                let at = tree.line_list.len() as u_int;
                if let Some(item) = mode_tree_item_mut(&mut tree, line.item) {
                    item.line = at;
                    tree.line_list.push(line);
                }
            }
        }
    }
    unsafe fn build_lines_at(&self, parent: Option<u_int>, depth: u_int) {
        let mtd = self;

        unsafe {
            let owner = mtd.clone();
            let ids: Vec<u_int> = {
                let mut tree = owner.borrow_mut();
                tree.depth = depth;
                tree.maxdepth = tree.maxdepth.max(depth);
                let items = match parent {
                    Some(id) => {
                        let Some(item) = mode_tree_item_ref(&tree, id) else {
                            return;
                        };
                        &item.children
                    }
                    None => &tree.children,
                };
                items.iter().map(|item| item.id).collect()
            };
            let mut flat = 1;
            for &id in &ids {
                let expanded = {
                    let mut tree = owner.borrow_mut();
                    let line = tree.line_list.len() as u_int;
                    let Some(item) = mode_tree_item_mut(&mut tree, id) else {
                        continue;
                    };
                    item.line = line;
                    if !item.children.is_empty() {
                        flat = 0;
                    }
                    let expanded = item.expanded != 0;
                    tree.line_list.push(mode_tree_line {
                        item: id,
                        depth,
                        last: (Some(&id) == ids.last()) as core::ffi::c_int,
                        flat: 0,
                    });
                    expanded
                };
                if expanded {
                    owner.build_lines_at(Some(id), depth.wrapping_add(1));
                }
                let (keycb, itemdata, line) = {
                    let tree = owner.borrow();
                    let Some(item) = mode_tree_item_ref(&tree, id) else {
                        continue;
                    };
                    (tree.keycb.clone(), item.itemdata.clone(), item.line)
                };
                let key = if let Some(keycb) = keycb {
                    match keycb(itemdata, line) {
                        KEYC_UNKNOWN => KEYC_NONE,
                        key => key,
                    }
                } else if line < 10 {
                    (b'0' as u_int + line) as key_code
                } else if line < 36 {
                    KEYC_META | (b'a' as u_int + line - 10) as key_code
                } else {
                    KEYC_NONE
                };
                let mut tree = owner.borrow_mut();
                if let Some(item) = mode_tree_item_mut(&mut tree, id) {
                    item.key = key;
                    item.keystr =
                        (key != KEYC_NONE).then(|| RustKeyStringCodec.format_key(key, false));
                    item.keylen = item.keystr.as_ref().map_or(0, |key| key.as_bytes().len());
                }
            }
            let mut tree = owner.borrow_mut();
            let live_ids: Vec<_> = ids
                .into_iter()
                .filter(|id| mode_tree_item_ref(&tree, *id).is_some())
                .collect();
            for line in &mut tree.line_list {
                if live_ids.contains(&line.item) {
                    line.flat = flat;
                    line.last = (Some(&line.item) == live_ids.last()) as core::ffi::c_int;
                }
            }
        }
    }
    pub unsafe fn up(&self, wrap: core::ffi::c_int) {
        let mtd = self;

        unsafe {
            let owner = mtd.clone();
            let mut mtd = owner.borrow_mut();
            if line_count(&mtd) == 0 as u_int {
                return;
            }
            if mtd.current == 0 as u_int {
                if wrap != 0 {
                    mtd.current = line_count(&mtd).wrapping_sub(1 as u_int);
                    if line_count(&mtd) >= mtd.height {
                        mtd.offset = line_count(&mtd).wrapping_sub(mtd.height);
                    }
                }
            } else {
                mtd.current = mtd.current.wrapping_sub(1);
                if mtd.current < mtd.offset {
                    mtd.offset = mtd.offset.wrapping_sub(1);
                }
            };
        }
    }
    pub unsafe fn down(&self, wrap: core::ffi::c_int) -> core::ffi::c_int {
        let mtd = self;

        unsafe {
            let owner = mtd.clone();
            let mut mtd = owner.borrow_mut();
            if line_count(&mtd) == 0 as u_int {
                return 0 as core::ffi::c_int;
            }
            if mtd.current == line_count(&mtd).wrapping_sub(1 as u_int) {
                if wrap != 0 {
                    mtd.current = 0 as u_int;
                    mtd.offset = 0 as u_int;
                } else {
                    return 0 as core::ffi::c_int;
                }
            } else {
                mtd.current = mtd.current.wrapping_add(1);
                if mtd.current > mtd.offset.wrapping_add(mtd.height).wrapping_sub(1 as u_int) {
                    mtd.offset = mtd.offset.wrapping_add(1);
                }
            }
            1 as core::ffi::c_int
        }
    }
    unsafe fn swap(&self, direction: core::ffi::c_int) {
        let mtd = self;

        unsafe {
            let owner = mtd.clone();
            let (swapcb, current_item, other_item, sort_crit, swap_with) = {
                let tree = owner.borrow();
                let Some(current) = tree.line_list.get(tree.current as usize) else {
                    return;
                };
                let Some(swapcb) = tree.swapcb.clone() else {
                    return;
                };
                let mut swap_with = tree.current as usize;
                loop {
                    let Some(next) = swap_with.checked_add_signed(direction as isize) else {
                        return;
                    };
                    swap_with = next;
                    let Some(other) = tree.line_list.get(swap_with) else {
                        return;
                    };
                    if other.depth < current.depth {
                        return;
                    }
                    if other.depth == current.depth {
                        break;
                    }
                }
                let Some(current_item) = mode_tree_item_ref(&tree, current.item) else {
                    return;
                };
                let Some(other_item) = line_item_ref(&tree, swap_with as u_int) else {
                    return;
                };
                (
                    swapcb,
                    current_item.itemdata.clone(),
                    other_item.itemdata.clone(),
                    tree.sort_crit.clone(),
                    swap_with as u_int,
                )
            };
            if swapcb(current_item, other_item, &sort_crit) != 0 {
                owner.borrow_mut().current = swap_with;
                owner.build();
            }
        }
    }
    pub unsafe fn current_item(&self) -> ModeTreeItemData {
        let mtd = self;

        let tree = mtd.borrow();
        line_item_ref(&tree, tree.current)
            .map_or(ModeTreeItemData::None, |item| item.itemdata.clone())
    }
    /// Returns an owned copy of the current row's name, or nothing for an unnamed
    /// row or an empty tree. The name remains valid after the tree changes or is freed.
    pub unsafe fn current_name(&self) -> Option<CString> {
        let mtd = self;

        let tree = mtd.borrow();
        line_item_ref(&tree, tree.current)?.name.clone()
    }
    unsafe fn expand_line(&self, at: u_int, expanded: bool) {
        let mtd = self;

        unsafe {
            let changed = {
                let owner = mtd.clone();
                mode_tree_set_expanded(&mut owner.borrow_mut(), at, expanded)
            };
            if changed {
                mtd.build();
            }
        }
    }
    pub unsafe fn expand_current(&self) {
        let mtd = self;

        let current = mtd.borrow().current;
        unsafe { mtd.expand_line(current, true) }
    }
    pub unsafe fn collapse_current(&self) {
        let mtd = self;

        let current = mtd.borrow().current;
        unsafe { mtd.expand_line(current, false) }
    }
    pub unsafe fn expand(&self, tag: uint64_t) {
        let mtd = self;

        unsafe {
            let found = mode_tree_get_tag(&mtd.borrow(), tag);
            if let Some(found) = found {
                mtd.expand_line(found, true);
            }
        }
    }
    pub unsafe fn set_current(&self, tag: uint64_t) -> core::ffi::c_int {
        let mtd = self;

        unsafe {
            let owner = mtd.clone();
            let mut mtd = owner.borrow_mut();
            if let Some(found) = mode_tree_get_tag(&mtd, tag) {
                mtd.current = found;
                if mtd.current > mtd.height.wrapping_sub(1 as u_int) {
                    mtd.offset = mtd
                        .current
                        .wrapping_sub(mtd.height)
                        .wrapping_add(1 as u_int);
                } else {
                    mtd.offset = 0 as u_int;
                }
                return 1 as core::ffi::c_int;
            }
            if mtd.current >= line_count(&mtd) {
                if line_count(&mtd) == 0 as u_int {
                    return 0 as core::ffi::c_int;
                }
                mtd.current = line_count(&mtd).wrapping_sub(1 as u_int);
                if mtd.current > mtd.height.wrapping_sub(1 as u_int) {
                    mtd.offset = mtd
                        .current
                        .wrapping_sub(mtd.height)
                        .wrapping_add(1 as u_int);
                } else {
                    mtd.offset = 0 as u_int;
                }
            }
            0 as core::ffi::c_int
        }
    }
    pub unsafe fn count_tagged(&self) -> u_int {
        let mtd = self;

        let tree = mtd.borrow();
        tree.line_list
            .iter()
            .filter_map(|line| mode_tree_item_ref(&tree, line.item))
            .filter(|item| item.tagged != 0)
            .count() as u_int
    }
    pub unsafe fn each_tagged(
        &self,
        mut cb: impl FnMut(WindowModeData, ModeTreeItemData),
        current: core::ffi::c_int,
    ) {
        let mtd = self;

        {
            let owner = mtd.clone();
            let mut at = 0;
            let mut fired = false;
            loop {
                let callback_item = {
                    let tree = owner.borrow();
                    if at as usize >= tree.line_list.len() {
                        break;
                    }
                    line_item_ref(&tree, at)
                        .filter(|item| item.tagged != 0)
                        .map(|item| (tree.modedata.clone(), item.itemdata.clone()))
                };
                if let Some((modedata, itemdata)) = callback_item {
                    fired = true;
                    cb(modedata, itemdata);
                }
                at = at.wrapping_add(1);
            }
            if !fired && current != 0 {
                let callback_item = {
                    let tree = owner.borrow();
                    line_item_ref(&tree, tree.current)
                        .map(|item| (tree.modedata.clone(), item.itemdata.clone()))
                };
                if let Some((modedata, itemdata)) = callback_item {
                    cb(modedata, itemdata);
                }
            }
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) unsafe fn start(
        wp: &mut impl crate::WindowPane,
        args: Option<&args>,
        buildcb: mode_tree_build_cb,
        drawcb: mode_tree_draw_cb,
        searchcb: mode_tree_search_cb,
        menucb: mode_tree_menu_cb,
        heightcb: mode_tree_height_cb,
        keycb: mode_tree_key_cb,
        swapcb: mode_tree_swap_cb,
        sortcb: mode_tree_sort_cb,
        help: mode_tree_help,
        modedata: WindowModeData,
        menu: &'static [menu_item<'static>],
    ) -> ModeTreeDataRef {
        unsafe {
            let preview = if args.is_some_and(|args| args_has(args, b'N') > 1) {
                MODE_TREE_PREVIEW_BIG as core::ffi::c_int
            } else if args.is_some_and(|args| args_has(args, b'N') != 0) {
                MODE_TREE_PREVIEW_OFF as core::ffi::c_int
            } else {
                MODE_TREE_PREVIEW_NORMAL as core::ffi::c_int
            };
            let sort_crit = RustSortCriteria::new(
                args.map_or(SORT_NAME, |args| {
                    RustSortCriteria::parse_order(args_get_str(args, b'O'))
                }),
                args.is_some_and(|args| args_has(args, b'r') != 0),
            );
            let filter = args
                .and_then(|args| args_get_str(args, b'f'))
                .map(CStr::to_owned);
            let screen_ref = ScreenRef::new(RustScreen::new_with_server_options(
                wp.geometry().width,
                wp.geometry().height,
                0,
            ));

            ModeTreeDataRef::new(
                mode_tree_data {
                    zoomed: 0,
                    pane_ref: crate::window::window_pane_ref_of(wp),
                    modedata,
                    menu,
                    sort_crit,
                    buildcb,
                    drawcb,
                    searchcb,
                    menucb,
                    heightcb,
                    keycb,
                    swapcb,
                    sortcb,
                    help,
                    children: mode_tree_list::new(),
                    saved: mode_tree_list::new(),
                    next_item_id: 0,
                    line_list: Vec::new(),
                    depth: 0,
                    maxdepth: 0,
                    width: 0,
                    height: 0,
                    offset: 0,
                    current: 0,
                    preview,
                    search: None,
                    filter,
                    no_matches: 0,
                    search_dir: MODE_TREE_SEARCH_FORWARD,
                    search_icase: 0,
                },
                screen_ref,
            )
        }
    }
    pub unsafe fn zoom(&self, args: Option<&args>) {
        let mtd = self;

        unsafe {
            let owner = mtd.clone();
            let Some(pane) = owner.borrow().pane() else {
                return;
            };
            if args.is_some_and(|args| args_has(args, b'Z') != 0) {
                let window = pane.window().expect("a live attached pane has a window");
                let zoomed = window.as_window().flags & WINDOW_ZOOMED;
                owner.borrow_mut().zoomed = zoomed;
                if zoomed == 0
                    && window.zoom(
                        &crate::window::window_pane_find_by_id(pane.id())
                            .expect("the selected pane exists"),
                    ) == 0
                {
                    window.redraw();
                }
            } else {
                owner.borrow_mut().zoomed = -1;
            }
        }
    }
    unsafe fn set_height(&self) {
        let mtd = self;

        unsafe {
            let owner = mtd.clone();
            let heightcb = owner.borrow().heightcb.clone();
            let reserved_height = heightcb.map(|heightcb| heightcb());
            let sy = RustScreen::grid(&owner.screen_handle().borrow()).sy;
            let mut tree = owner.borrow_mut();
            if let Some(height) = reserved_height {
                if height < sy {
                    tree.height = sy.wrapping_sub(height);
                }
            } else if tree.preview == MODE_TREE_PREVIEW_NORMAL as core::ffi::c_int {
                tree.height = sy.wrapping_div(3).wrapping_mul(2);
                if tree.height > line_count(&tree) {
                    tree.height = sy.wrapping_div(2);
                }
                if tree.height < 10 {
                    tree.height = sy;
                }
            } else if tree.preview == MODE_TREE_PREVIEW_BIG as core::ffi::c_int {
                tree.height = sy.wrapping_div(4);
                if tree.height > line_count(&tree) {
                    tree.height = line_count(&tree);
                }
                if tree.height < 2 {
                    tree.height = 2;
                }
            } else {
                tree.height = sy;
            }
            if sy.wrapping_sub(tree.height) < 2 {
                tree.height = sy;
            }
        }
    }
    pub unsafe fn build(&self) {
        let mtd = self;

        unsafe {
            let owner = mtd.clone();
            let (mut tag, sortcb, mut sort_crit) = {
                let mut tree = owner.borrow_mut();
                if tree.pane().is_none() {
                    return;
                }
                let tag = line_item_ref(&tree, tree.current).map_or(UINT64_MAX, |item| item.tag);
                let moved = core::mem::take(&mut tree.children);
                tree.saved.extend(moved);
                (tag, tree.sortcb.clone(), tree.sort_crit.clone())
            };
            if let Some(sortcb) = sortcb {
                sortcb(&mut sort_crit);
                owner.borrow_mut().sort_crit = sort_crit;
            }
            let (build, sort_crit, filter) = {
                let tree = owner.borrow();
                if tree.pane().is_none() {
                    return;
                }
                (
                    tree.buildcb.clone().expect("missing build callback"),
                    tree.sort_crit.clone(),
                    tree.filter.clone(),
                )
            };
            build(&sort_crit, &mut tag, filter.as_deref());
            let no_matches = {
                let mut tree = owner.borrow_mut();
                if tree.pane().is_none() {
                    return;
                }
                tree.no_matches = tree.children.is_empty() as core::ffi::c_int;
                tree.no_matches != 0
            };
            if no_matches {
                let sort_crit = owner.borrow().sort_crit.clone();
                build(&sort_crit, &mut tag, None);
                if owner.borrow().pane().is_none() {
                    return;
                }
            }
            owner.borrow_mut().saved.clear();
            owner.build_lines();
            if tag == UINT64_MAX {
                let tree = owner.borrow();
                if let Some(item) = line_item_ref(&tree, tree.current) {
                    tag = item.tag;
                }
            }
            owner.set_current(tag);
            let (sx, sy) = owner.screen_handle().borrow().size();
            let show_preview = {
                let mut tree = owner.borrow_mut();
                tree.width = sx;
                if tree.preview == MODE_TREE_PREVIEW_OFF as core::ffi::c_int {
                    tree.height = sy;
                    false
                } else {
                    true
                }
            };
            if show_preview {
                owner.set_height();
            }
            mode_tree_check_selected(&mut owner.borrow_mut());
        }
    }
    /// Detaches the tree from its pane. The tree itself is released when the
    /// last handle to it goes away, which is normally the owning
    /// `window_mode_entry` moments later.
    pub unsafe fn close(&self) {
        let mtd = self;

        unsafe {
            let owner = mtd.clone();
            let (id, zoomed) = {
                let mut tree = owner.borrow_mut();
                (tree.pane_ref.take(), tree.zoomed)
            };
            if zoomed == 0
                && let Some(pane) = id
                && let Some(window) = pane.window()
            {
                window.unzoom_and_redraw();
            }
        }
    }
    pub unsafe fn resize(&self, sx: u_int, sy: u_int) {
        let mtd = self;

        unsafe {
            let owner = mtd.clone();
            if owner.borrow().pane().is_none() {
                return;
            }
            screen_resize(&mut owner.screen_handle().borrow_mut(), sx, sy, 0);
            owner.build();
            owner.draw();
            owner.borrow().redraw_pane();
        }
    }
    pub(crate) unsafe fn add_item(
        &self,
        parent: Option<&ModeTreeItemRef>,
        itemdata: ModeTreeItemData,
        tag: uint64_t,
        name: &CStr,
        text: Option<&CStr>,
        expanded: core::ffi::c_int,
    ) -> Option<ModeTreeItemRef> {
        let mtd = self;

        if parent.is_some_and(|parent| !parent.tree.ptr_eq(mtd)) {
            return None;
        }
        let tree = mtd.clone();
        let id = mode_tree_insert_item(
            &mut tree.borrow_mut(),
            parent.map(|parent| parent.id),
            itemdata,
            tag,
            name,
            text,
            expanded,
        )?;
        Some(ModeTreeItemRef { tree, id })
    }
    pub unsafe fn draw(&self) {
        let mtd = self;

        unsafe {
            let owner = mtd.clone();
            let (pane, screen_ref) = {
                let tree = owner.borrow();
                let Some(pane) = tree.pane() else {
                    return;
                };
                (pane, owner.screen_handle().clone())
            };
            let options = pane
                .window()
                .expect("the widget pane has a window")
                .options();
            let mut writer = RustScreenWriteCtx::on_shared_screen(&screen_ref);
            let pending_preview = {
                let tree = owner.borrow();
                let mut alignlen = vec![0; tree.maxdepth.wrapping_add(1) as usize];
                if tree.line_list.is_empty() {
                    return;
                }
                let mut gc0 = grid_default_cell;
                let mut gc = grid_default_cell;
                style_apply(&mut gc, &options, c"mode-style", None);
                let (w, h) = (tree.width, tree.height);
                writer.clearscreen(8);
                let mut keylen = 0;
                for line in &tree.line_list {
                    let Some(item) = mode_tree_item_ref(&tree, line.item) else {
                        continue;
                    };
                    if item.key != KEYC_NONE && item.keylen as core::ffi::c_int + 3 > keylen {
                        keylen = item.keylen.wrapping_add(3) as core::ffi::c_int;
                    }
                    let namelen = item.name.as_deref().map_or(0, |name| name.to_bytes().len())
                        as core::ffi::c_int;
                    if item.align != 0 && namelen > alignlen[line.depth as usize] {
                        alignlen[line.depth as usize] = namelen;
                    }
                }
                for (at, line) in tree.line_list.iter().enumerate() {
                    let at = at as u_int;
                    if at < tree.offset {
                        continue;
                    }
                    if at > tree.offset.wrapping_add(h).wrapping_sub(1) {
                        break;
                    }
                    let Some(item) = mode_tree_item_ref(&tree, line.item) else {
                        continue;
                    };
                    writer.cursormove(0, at.wrapping_sub(tree.offset) as core::ffi::c_int, 0);
                    let pad =
                        ((keylen - 2) as size_t).wrapping_sub(item.keylen) as core::ffi::c_int;
                    let key = if item.key != KEYC_NONE {
                        xasprintf(c"(%s)%*s", fmt_args![item.keystr.as_deref(), pad, c""])
                    } else {
                        CString::default()
                    };
                    let symbol = if line.flat != 0 {
                        c""
                    } else if item.children.is_empty() {
                        c"  "
                    } else if item.expanded != 0 {
                        c"- "
                    } else {
                        c"+ "
                    };
                    let mut start = Vec::new();
                    if line.depth == 0 {
                        start.extend_from_slice(symbol.to_bytes());
                    } else {
                        let size = 4u32.wrapping_mul(line.depth).wrapping_add(32) as usize;
                        let parent_last = item
                            .parent
                            .and_then(|id| mode_tree_item_ref(&tree, id))
                            .and_then(|parent| tree.line_list.get(parent.line as usize))
                            .is_some_and(|line| line.last != 0);
                        for _ in 1..line.depth {
                            if parent_last {
                                mode_tree_append(&mut start, b"    ", size);
                            } else {
                                mode_tree_append(&mut start, b"\x01x\x01   ", size);
                            }
                        }
                        if line.last != 0 {
                            mode_tree_append(&mut start, b"\x01mq\x01> ", size);
                        } else {
                            mode_tree_append(&mut start, b"\x01tq\x01> ", size);
                        }
                        mode_tree_append(&mut start, symbol.to_bytes(), size);
                    }
                    let start = CString::new(start).expect("a tree prefix has no NUL");
                    let tag = if item.tagged != 0 { c"*" } else { c"" };
                    let text = xasprintf(
                        c"%-*s%s%*s%s%s",
                        fmt_args![
                            keylen,
                            key.as_c_str(),
                            start.as_c_str(),
                            item.align * alignlen[line.depth as usize],
                            item.name.as_deref(),
                            tag,
                            if item.text.is_some() { c": " } else { c"" },
                        ],
                    );
                    let width = RustUtf8VisModel.width(&text).min(w);
                    if item.tagged != 0 {
                        gc.attr = (gc.attr as core::ffi::c_int ^ GRID_ATTR_BRIGHT) as u_short;
                        gc0.attr = (gc0.attr as core::ffi::c_int ^ GRID_ATTR_BRIGHT) as u_short;
                    }
                    let style = if at == tree.current {
                        writer.clearendofline(gc.bg as u_int);
                        &gc
                    } else {
                        writer.clearendofline(8);
                        &gc0
                    };
                    writer.nputs(w as ssize_t, style, c"%s", fmt_args![text.as_c_str()]);
                    if let Some(text) = item.text.as_deref() {
                        writer.format_draw(style, w.wrapping_sub(width), text.to_bytes(), None, 0);
                    }
                    if item.tagged != 0 {
                        gc.attr = (gc.attr as core::ffi::c_int ^ GRID_ATTR_BRIGHT) as u_short;
                        gc0.attr = (gc0.attr as core::ffi::c_int ^ GRID_ATTR_BRIGHT) as u_short;
                    }
                }
                let sy = writer.size().1;
                let preview_item = line_item_ref(&tree, tree.current).and_then(|item| {
                    if item.draw_as_parent != 0 {
                        item.parent.and_then(|id| mode_tree_item_ref(&tree, id))
                    } else {
                        Some(item)
                    }
                });
                if tree.preview != MODE_TREE_PREVIEW_OFF as core::ffi::c_int
                    && sy > 4
                    && h >= 2
                    && sy.wrapping_sub(h) > 4
                    && w > 4
                    && let Some(item) = preview_item
                {
                    writer.cursormove(0, h as core::ffi::c_int, 0);
                    writer.box_(w, sy.wrapping_sub(h), BOX_LINES_DEFAULT, None, None);
                    let text = if tree.sort_crit.has_cycle() {
                        xasprintf(
                            c" %s (sort: %s%s)",
                            fmt_args![
                                item.name.as_deref(),
                                RustSortCriteria::order_name(tree.sort_crit.order()),
                                if tree.sort_crit.reversed() {
                                    c", reversed"
                                } else {
                                    c""
                                },
                            ],
                        )
                    } else {
                        xasprintf(c" %s", fmt_args![item.name.as_deref()])
                    };
                    if w.wrapping_sub(2) as usize >= text.as_bytes().len() {
                        writer.cursormove(1, h as core::ffi::c_int, 0);
                        writer.puts(&gc0, c"%s", fmt_args![text.as_c_str()]);
                        let n = if tree.no_matches != 0 { 10 } else { 6 };
                        if tree.filter.is_some()
                            && w.wrapping_sub(2) as usize
                                >= text
                                    .as_bytes()
                                    .len()
                                    .wrapping_add(10)
                                    .wrapping_add(n)
                                    .wrapping_add(2)
                        {
                            writer.puts(&gc0, c" (filter: ", fmt_args![]);
                            if tree.no_matches != 0 {
                                writer.puts(&gc, c"no matches", fmt_args![]);
                            } else {
                                writer.puts(&gc0, c"active", fmt_args![]);
                            }
                            writer.puts(&gc0, c") ", fmt_args![]);
                        } else {
                            writer.puts(&gc0, c" ", fmt_args![]);
                        }
                    }
                    let box_x = w.wrapping_sub(4);
                    let box_y = sy.wrapping_sub(h).wrapping_sub(2);
                    if box_x != 0 && box_y != 0 {
                        writer.cursormove(2, h.wrapping_add(1) as core::ffi::c_int, 0);
                        Some((
                            tree.drawcb.clone().expect("a preview has a draw callback"),
                            item.itemdata.clone(),
                            box_x,
                            box_y,
                        ))
                    } else {
                        None
                    }
                } else {
                    None
                }
            };
            if let Some((draw, item, width, height)) = pending_preview {
                draw(item, &mut writer, width, height);
            }
            let tree = owner.borrow();
            writer.cursormove(
                0,
                tree.current.wrapping_sub(tree.offset) as core::ffi::c_int,
                0,
            );
        }
    }
    /// Searches a snapshot of the item order, including collapsed descendants.
    /// Items removed by a search callback are skipped; newly added items wait for
    /// the next search. No item or tree borrow is held across a callback.
    unsafe fn search(&self, direction: mode_tree_search_dir) -> Option<ModeTreeItemRef> {
        let mtd = self;

        {
            let (ids, current, icase) = {
                let tree = mtd.borrow();
                tree.search.as_ref()?;
                let current = line_item_ref(&tree, tree.current)?.id;
                let mut ids = Vec::new();
                mode_tree_search_ids(&tree.children, &mut ids);
                (ids, current, tree.search_icase)
            };
            let start = ids.iter().position(|id| *id == current)?;
            for step in 1..ids.len() {
                let at = if direction == MODE_TREE_SEARCH_FORWARD {
                    (start + step) % ids.len()
                } else {
                    (start + ids.len() - step) % ids.len()
                };
                let id = ids[at];
                let (searchcb, itemdata, search, name) = {
                    let tree = mtd.borrow();
                    let Some(item) = mode_tree_item_ref(&tree, id) else {
                        continue;
                    };
                    (
                        tree.searchcb.clone(),
                        item.itemdata.clone(),
                        tree.search.clone()?,
                        item.name.clone(),
                    )
                };
                let matched = if let Some(searchcb) = searchcb {
                    searchcb(itemdata, &search, icase) != 0
                } else if icase == 0 {
                    cstr_has(name.as_deref().unwrap_or(c""), &search)
                } else {
                    cstr_has_nocase(name.as_deref().unwrap_or(c""), &search)
                };
                let item = ModeTreeItemRef {
                    tree: mtd.clone(),
                    id,
                };
                if matched && item.get().is_some() {
                    return Some(item);
                }
            }
            None
        }
    }
    unsafe fn search_set(&self) {
        let mtd = self;

        unsafe {
            let direction = mtd.borrow().search_dir;
            let Some(item) = mtd.search(direction) else {
                return;
            };
            let Some((tag, mut parent)) = item.get().map(|item| (item.tag, item.parent)) else {
                return;
            };
            {
                let owner = mtd.clone();
                let mut tree = owner.borrow_mut();
                while let Some(id) = parent {
                    let Some(item) = mode_tree_item_mut(&mut tree, id) else {
                        return;
                    };
                    item.expanded = 1;
                    parent = item.parent;
                }
            }
            mtd.build();
            mtd.set_current(tag);
            mtd.draw();
            mtd.borrow().redraw_pane();
        }
    }
    pub(crate) unsafe fn search_input(
        &self,
        _c: &mut client,
        s: Option<&CStr>,
        _done: core::ffi::c_int,
    ) -> core::ffi::c_int {
        let mtd = self;

        unsafe {
            let owner = mtd.clone();
            {
                let mut tree = owner.borrow_mut();
                if tree.pane().is_none() {
                    return 0;
                }
                let Some(s) = s.filter(|s| !s.is_empty()) else {
                    tree.search = None;
                    return 0;
                };
                tree.search = Some(s.to_owned());
                tree.search_icase = mode_tree_is_lowercase(s);
            }
            owner.search_set();
            0
        }
    }
    pub(crate) unsafe fn filter_input(
        &self,
        _c: &mut client,
        s: Option<&CStr>,
        _done: core::ffi::c_int,
    ) -> core::ffi::c_int {
        let mtd = self;

        unsafe {
            let owner = mtd.clone();
            {
                let mut tree = owner.borrow_mut();
                if tree.pane().is_none() {
                    return 0;
                }
                tree.filter = s.filter(|s| !s.is_empty()).map(CStr::to_owned);
            }
            owner.build();
            owner.draw();
            owner.borrow().redraw_pane();
            0
        }
    }
    unsafe fn display_menu(
        &self,
        c: &mut client,
        mut x: u_int,
        y: u_int,
        outside: core::ffi::c_int,
    ) {
        let mtd = self;

        unsafe {
            let (line, items, title) = {
                let tree = mtd.borrow();
                let at = tree.offset.wrapping_add(y);
                let line = if at > line_count(&tree).wrapping_sub(1) {
                    tree.current
                } else {
                    at
                };
                if outside == 0 {
                    let Some(item) = line_item_ref(&tree, line) else {
                        return;
                    };
                    (
                        line,
                        tree.menu,
                        xasprintf(c"#[align=centre]%s", fmt_args![item.name.as_deref()]),
                    )
                } else {
                    (line, &mode_tree_menu_items[..], CString::default())
                }
            };
            let mut menu = menu_create(&title);
            menu_add_items(&mut menu, items, None, c, None);
            let Some(client) = client_ref_of(c).map(|client| client.downgrade()) else {
                return;
            };
            let mtm = Box::new(mode_tree_menu {
                client,
                line,
                owner: mtd.downgrade(),
            });
            x = x.saturating_sub(menu.width.wrapping_add(4).wrapping_div(2));
            menu_display(
                menu,
                0,
                0,
                None,
                x,
                y,
                c,
                BOX_LINES_DEFAULT,
                None,
                None,
                None,
                None,
                Some(Box::new(move |idx, key| {
                    mode_tree_menu_callback(idx, key, mtm)
                })),
            );
        }
    }
    pub unsafe fn key(
        &self,
        c: &mut client,
        key: &mut key_code,
        m: Option<&mouse_event>,
    ) -> (core::ffi::c_int, u_int, u_int) {
        let mtd = self;

        unsafe {
            let owner = mtd.clone();
            let Some(pane) = owner.borrow().pane() else {
                *key = KEYC_NONE;
                return (1, 0, 0);
            };
            if owner.borrow().line_list.is_empty() {
                *key = KEYC_NONE;
                return (1, 0, 0);
            }
            if (*key & KEYC_MASK_KEY == KEYC_MOUSE
                || *key & KEYC_MASK_TYPE >= (KEYC_TYPE_MOUSEMOVE as u64) << 32
                    && *key & KEYC_MASK_TYPE <= (KEYC_TYPE_TRIPLECLICK as u64) << 32)
                && let Some(m) = m
            {
                let Some((x, y)) = pane.get().and_then(|pane| cmd_mouse_at(pane, m, 0)) else {
                    *key = KEYC_NONE;
                    return (0, 0, 0);
                };
                let (width, height, preview, offset, count) = {
                    let tree = owner.borrow();
                    (
                        tree.width,
                        tree.height,
                        tree.preview,
                        tree.offset,
                        line_count(&tree),
                    )
                };
                if x > width || y > height {
                    if *key == KEYC_MOUSEDOWN3_PANE {
                        owner.display_menu(c, x, y, 1);
                    }
                    if preview == MODE_TREE_PREVIEW_OFF as core::ffi::c_int {
                        *key = KEYC_NONE;
                    }
                    return (0, x, y);
                }
                if offset.wrapping_add(y) < count {
                    if matches!(
                        *key,
                        KEYC_MOUSEDOWN1_PANE | KEYC_MOUSEDOWN3_PANE | KEYC_DOUBLECLICK1_PANE
                    ) {
                        owner.borrow_mut().current = offset.wrapping_add(y);
                    }
                    if *key == KEYC_DOUBLECLICK1_PANE {
                        *key = b'\r' as key_code;
                    } else {
                        if *key == KEYC_MOUSEDOWN3_PANE {
                            owner.display_menu(c, x, y, 0);
                        }
                        *key = KEYC_NONE;
                    }
                } else {
                    if *key == KEYC_MOUSEDOWN3_PANE {
                        owner.display_menu(c, x, y, 0);
                    }
                    *key = KEYC_NONE;
                }
                return (0, x, y);
            }
            let (line, choice) = {
                let tree = owner.borrow();
                let Some(line) = tree.line_list.get(tree.current as usize).copied() else {
                    *key = KEYC_NONE;
                    return (1, 0, 0);
                };
                if mode_tree_item_ref(&tree, line.item).is_none() {
                    *key = KEYC_NONE;
                    return (1, 0, 0);
                }
                let choice = tree.line_list.iter().position(|line| {
                    mode_tree_item_ref(&tree, line.item).is_some_and(|item| item.key == *key)
                });
                (line, choice)
            };
            if let Some(choice) = choice {
                owner.borrow_mut().current = choice as u_int;
                *key = b'\r' as key_code;
                return (0, 0, 0);
            }
            match *key {
                113 | 27 | 35184372088923 | 35184372088935 => return (1, 0, 0),
                8589934600 | 35184372088936 => {
                    let help = owner.borrow().help;
                    mode_tree_display_help(help, c);
                }
                8589934619 | 107 | 38654705664 | 35184372088944 => owner.up(1),
                8589934620 | 106 | 34359738368 | 35184372088942 => {
                    owner.down(1);
                }
                70377334112283 | 75 => owner.swap(-1),
                70377334112284 | 74 => owner.swap(1),
                8589934617 | 35184372088930 => {
                    let height = owner.borrow().height;
                    for _ in 0..height {
                        if owner.borrow().current == 0 {
                            break;
                        }
                        owner.up(1);
                    }
                }
                8589934616 | 35184372088934 => {
                    let height = owner.borrow().height;
                    for _ in 0..height {
                        if owner.borrow().current == line_count(&owner.borrow()).wrapping_sub(1) {
                            break;
                        }
                        owner.down(1);
                    }
                }
                103 | 8589934614 => {
                    let mut tree = owner.borrow_mut();
                    tree.current = 0;
                    tree.offset = 0;
                }
                71 | 8589934615 => {
                    let mut tree = owner.borrow_mut();
                    tree.current = line_count(&tree).wrapping_sub(1);
                    tree.offset = if tree.current > tree.height.wrapping_sub(1) {
                        tree.current.wrapping_sub(tree.height).wrapping_add(1)
                    } else {
                        0
                    };
                }
                116 => {
                    if mode_tree_toggle_tag(&mut owner.borrow_mut(), line.item) && m.is_some() {
                        owner.down(0);
                    }
                }
                84 => mode_tree_set_all_tagged(&mut owner.borrow_mut(), false),
                35184372088948 => mode_tree_set_all_tagged(&mut owner.borrow_mut(), true),
                79 => {
                    owner.borrow_mut().sort_crit.advance();
                    owner.build();
                }
                114 => {
                    let reversed = !owner.borrow().sort_crit.reversed();
                    owner.borrow_mut().sort_crit.set_reversed(reversed);
                    owner.build();
                }
                8589934621 | 104 | 45 => {
                    let collapsed = {
                        let mut tree = owner.borrow_mut();
                        let target = mode_tree_item_ref(&tree, line.item).and_then(|item| {
                            if line.flat != 0 || item.expanded == 0 {
                                item.parent
                            } else {
                                Some(item.id)
                            }
                        });
                        let row =
                            target
                                .and_then(|id| mode_tree_item_mut(&mut tree, id))
                                .map(|item| {
                                    item.expanded = 0;
                                    item.line
                                });
                        if let Some(row) = row {
                            tree.current = row;
                            true
                        } else {
                            false
                        }
                    };
                    if collapsed {
                        owner.build();
                    } else {
                        owner.up(0);
                    }
                }
                8589934622 | 108 | 43 => {
                    let expanded = mode_tree_item_ref(&owner.borrow(), line.item)
                        .is_some_and(|item| item.expanded != 0);
                    if line.flat != 0 || expanded {
                        owner.down(0);
                    } else {
                        if let Some(item) = mode_tree_item_mut(&mut owner.borrow_mut(), line.item) {
                            item.expanded = 1;
                        }
                        owner.build();
                    }
                }
                17592186044461 => {
                    for item in &mut owner.borrow_mut().children {
                        item.expanded = 0;
                    }
                    owner.build();
                }
                17592186044459 => {
                    for item in &mut owner.borrow_mut().children {
                        item.expanded = 1;
                    }
                    owner.build();
                }
                63 | 47 | 35184372088947 => {
                    owner.borrow_mut().search_dir = MODE_TREE_SEARCH_FORWARD;
                    status_prompt_set(
                        c,
                        None,
                        c"(search) ",
                        Some(c""),
                        Prompt::ModeTreeSearch,
                        PromptData::ModeTree(Box::new(owner.downgrade())),
                        PROMPT_NOFORMAT,
                        PromptHistoryType::Search,
                    );
                }
                110 => {
                    owner.borrow_mut().search_dir = MODE_TREE_SEARCH_FORWARD;
                    owner.search_set();
                }
                78 => {
                    owner.borrow_mut().search_dir = MODE_TREE_SEARCH_BACKWARD;
                    owner.search_set();
                }
                102 => {
                    let filter = owner.borrow().filter.clone();
                    status_prompt_set(
                        c,
                        None,
                        c"(filter) ",
                        filter.as_deref(),
                        Prompt::ModeTreeFilter,
                        PromptData::ModeTree(Box::new(owner.downgrade())),
                        PROMPT_NOFORMAT,
                        PromptHistoryType::Search,
                    );
                }
                118 => {
                    {
                        let mut tree = owner.borrow_mut();
                        match tree.preview {
                            0 => tree.preview = MODE_TREE_PREVIEW_BIG as core::ffi::c_int,
                            1 => tree.preview = MODE_TREE_PREVIEW_OFF as core::ffi::c_int,
                            2 => tree.preview = MODE_TREE_PREVIEW_NORMAL as core::ffi::c_int,
                            _ => {}
                        }
                    }
                    owner.build();
                    if owner.borrow().preview != MODE_TREE_PREVIEW_OFF as core::ffi::c_int {
                        mode_tree_check_selected(&mut owner.borrow_mut());
                    }
                }
                _ => {}
            }
            (0, 0, 0)
        }
    }
}

impl ModeTreeItemRef {
    pub(crate) unsafe fn remove(&self) {
        let item = self;

        let tree = item.tree.clone();
        mode_tree_remove_item(&mut tree.borrow_mut(), item.id);
    }
}

impl ModeTreeItemRef {
    pub(crate) fn draw_as_parent(&self) -> Option<()> {
        self.get_mut()?.draw_as_parent = 1;
        Some(())
    }
    pub(crate) fn no_tag(&self) -> Option<()> {
        self.get_mut()?.no_tag = 1;
        Some(())
    }
    pub(crate) fn set_align(&self, align: core::ffi::c_int) -> Option<()> {
        self.get_mut()?.align = align;
        Some(())
    }
}

#[cfg(test)]
pub use crate::consts::{
    KEYC_DOWN, KEYC_END, KEYC_HOME, KEYC_LEFT, KEYC_NPAGE, KEYC_PPAGE, KEYC_RIGHT, KEYC_UP,
    SORT_ACTIVITY,
};
