use crate::args::RustArguments;
use crate::cmd::cmd_retval;
use super::widget::{ModeTreeItemRef, mode_tree_run_command};
use crate::WindowPane;
use crate::args::{args_get_str, args_has, args_string_str};
use crate::cmd::{CmdqItemRef, cmdq_append};
use crate::cmd::{cmd_find_clear_state, cmd_find_from_winlink_pane};
use crate::compat::tolower;
use crate::fmt_args;
use crate::format::format_true;
use crate::format::{format_add, format_create, format_defaults, format_expand, format_single};
use crate::grid::grid_default_cell;
#[cfg(test)]
use crate::window::window_pane_current_mode_mut;

use crate::osdep_linux::osdep_get_name;
use crate::prompt_history::PromptHistoryType;
use crate::resize::recalculate_sizes;
use crate::screen::ScreenWriteCtx;
use crate::server::server_clear_marked;
use crate::server::server_renumber_all;
use crate::server::server_set_marked;
use crate::server::{server_destroy_session, server_kill_pane, server_redraw_session_group};

pub use crate::consts::{
    BOX_LINES_DEFAULT, CMD_RETURN_NORMAL, FORMAT_NONE, FORMAT_PANE, FORMAT_WINDOW, KEYC_MASK_KEY,
    KEYC_MASK_TYPE, KEYC_MOUSE, KEYC_MOUSEDOWN1_PANE, KEYC_NONE, KEYC_RIGHT, KEYC_TYPE_MOUSEMOVE,
    KEYC_TYPE_TRIPLECLICK, PANE_REDRAW, PROMPT_ACCEPT, PROMPT_NOFORMAT, PROMPT_SINGLE,
    SORT_ACTIVITY, SORT_END, SORT_INDEX, SORT_NAME, SORT_Z,
};
use crate::sort::{SortCriteria, sort_get_sessions, sort_would_window_tree_swap};
use crate::status::status_prompt_set;
use crate::style::style_apply;
use crate::text::{KeyStringCodec, RustKeyStringCodec};
pub use crate::types::*;
use crate::window::{WinlinkRef, winlink_is};
use crate::window::{window_pane_find_by_id, window_pane_index, window_pane_reset_mode};
use crate::xmalloc::xasprintf;
use crate::{FormatText, RustFormatText};
use ::core::ffi::CStr;
use ::std::ffi::CString;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum WindowTreeTag {
    Session(u_int),
    Window(u_int, core::ffi::c_int),
    Pane(u_int),
}

impl WindowTreeTag {
    fn is_live(self) -> bool {
        unsafe {
            match self {
                Self::Session(id) => SessionRef::find_by_id(id).is_some(),
                Self::Window(id, index) => SessionRef::find_by_id(id)
                    .is_some_and(|session| session.as_session().windows.contains_key(&index)),
                Self::Pane(id) => window_pane_find_by_id(id).is_some(),
            }
        }
    }
}

#[derive(Default)]
#[repr(C)]
pub struct window_tree_modedata {
    /// The pane the mode is running in.
    pub wp_ref: Option<RustWindowPaneWeak>,
    pub(crate) data: Option<ModeTreeDataWeak>,
    pub format: Option<CString>,
    pub key_format: Option<CString>,
    pub command: Option<CString>,
    pub squash_groups: core::ffi::c_int,
    pub prompt_flags: core::ffi::c_int,
    pub item_list: Vec<window_tree_itemdata>,
    pub entered: Option<CString>,
    pub fs: cmd_find_state,
    pub type_0: window_tree_type,
    pub offset: core::ffi::c_int,
    pub left: core::ffi::c_int,
    pub right: core::ffi::c_int,
    pub start: u_int,
    pub end: u_int,
    pub each: u_int,
    pub(crate) owner: Option<WindowTreeModeDataWeak>,
    tags: std::collections::BTreeMap<WindowTreeTag, uint64_t>,
    next_tag: uint64_t,
}

impl window_tree_modedata {
    fn tag_for(&mut self, key: WindowTreeTag) -> uint64_t {
        if let Some(tag) = self.tags.get(&key) {
            return *tag;
        }
        self.next_tag = self
            .next_tag
            .checked_add(1)
            .filter(|tag| *tag != uint64_t::MAX)
            .expect("tree selection tags exhausted");
        self.tags.insert(key, self.next_tag);
        self.next_tag
    }

    fn prune_tags(&mut self) {
        self.tags.retain(|key, _| key.is_live());
    }

    /// The pane the mode is running in, retained while it is still owned by
    /// its window.
    pub(crate) fn pane(&self) -> Option<RustWindowPaneWeak> {
        self.wp_ref.as_ref().filter(|pane| pane.is_alive()).cloned()
    }

    /// The mode tree the mode is showing, as a handle. Only ever asked of a
    /// mode that is showing one.
    pub(crate) fn tree_ref(&self) -> ModeTreeDataRef {
        self.data
            .as_ref()
            .and_then(ModeTreeDataWeak::upgrade)
            .expect("the mode is showing a tree")
    }
}
pub type window_tree_type = core::ffi::c_uint;
pub const WINDOW_TREE_PANE: window_tree_type = 3;
pub const WINDOW_TREE_WINDOW: window_tree_type = 2;
pub const WINDOW_TREE_SESSION: window_tree_type = 1;
pub const WINDOW_TREE_NONE: window_tree_type = 0;
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct window_tree_itemdata {
    pub type_0: window_tree_type,
    pub session: core::ffi::c_int,
    pub winlink: core::ffi::c_int,
    pub pane: core::ffi::c_int,
}

pub const WINDOW_TREE_DEFAULT_COMMAND: &CStr = c"switch-client -Zt '%%'";
pub static WINDOW_TREE_DEFAULT_FORMAT: &CStr = c"#{?pane_format,#{?pane_marked,#[reverse],}#{?pane_floating_flag,#[italics],}#{pane_current_command}#{pane_flags}#{?#{&&:#{pane_title},#{!=:#{pane_title},#{host_short}}},: \"#{pane_title}\",},window_format,#{?window_marked_flag,#[reverse],}#{window_name}#{window_flags}#{?#{&&:#{==:#{window_panes},1},#{&&:#{pane_title},#{!=:#{pane_title},#{host_short}}}},: \"#{pane_title}\",},#{session_windows} windows#{?session_grouped, (group #{session_group}: #{session_group_list}),}#{?session_attached, (attached),}}";
pub const WINDOW_TREE_DEFAULT_KEY_FORMAT: &CStr =
    c"#{?#{e|<:#{line},10},#{line},#{e|<:#{line},36},M-#{a:#{e|+:97,#{e|-:#{line},10}}}}";
static window_tree_menu_items: [menu_item<'static>; 12] = [
    menu_item {
        name: Some(c"Select"),
        key: '\r' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: Some(c"Expand"),
        key: KEYC_RIGHT as core::ffi::c_ulong as key_code,
        command: None,
    },
    menu_item {
        name: Some(c"Mark"),
        key: 'm' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: Some(c""),
        key: KEYC_NONE as core::ffi::c_ulong as key_code,
        command: None,
    },
    menu_item {
        name: Some(c"Tag"),
        key: 't' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: Some(c"Tag All"),
        key: '\u{14}' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: Some(c"Tag None"),
        key: 'T' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: Some(c""),
        key: KEYC_NONE as core::ffi::c_ulong as key_code,
        command: None,
    },
    menu_item {
        name: Some(c"Kill"),
        key: 'x' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: Some(c"Kill Tagged"),
        key: 'X' as i32 as key_code,
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
static window_tree_order_seq: [sort_order; 4] = [SORT_INDEX, SORT_NAME, SORT_ACTIVITY, SORT_Z];
struct WindowTreeTarget {
    link: WinlinkRef,
    window: WindowRef,
    pane: Option<RustWindowPaneWeak>,
}

/// Resolves an item's IDs while retaining its session and window. Session and
/// window items may have no active pane; pane items must still belong to the
/// named window.
unsafe fn window_tree_resolve_item(item: &window_tree_itemdata) -> Option<WindowTreeTarget> {
    {
        let session = SessionRef::find_by_id(item.session as u_int)?;
        let index = if item.type_0 == WINDOW_TREE_SESSION {
            session.curw()?.index()
        } else {
            item.winlink
        };
        let link = WinlinkRef::new(session, index)?;
        let window = link.get()?.window_handle()?.clone();
        let pane = if matches!(item.type_0, WINDOW_TREE_SESSION | WINDOW_TREE_WINDOW) {
            window.active_pane_id().and_then(|id| window.pane_by_id(id))
        } else {
            Some(window.pane_by_id(item.pane as u_int)?)
        };
        Some(WindowTreeTarget { link, window, pane })
    }
}

fn window_tree_add_item(data: &mut window_tree_modedata) -> &mut window_tree_itemdata {
    data.item_list.push(window_tree_itemdata::default());
    data.item_list.last_mut().unwrap()
}
unsafe fn window_tree_build_pane(
    s: &mut session,
    wl: &mut winlink,
    wp: &mut impl crate::WindowPane,
    modedata: WindowModeData,
    parent: Option<&ModeTreeItemRef>,
) {
    unsafe {
        let held = modedata.tree().expect("the mode holds its state");
        let mut data_guard = held.borrow_mut();
        let data = &mut *data_guard;
        let idx: u_int;
        let window = wl
            .window_handle()
            .expect("the pane's link has a window")
            .clone();
        (_, idx) = window_pane_index(&window.as_window(), wp);
        let item = window_tree_add_item(&mut *data);
        item.type_0 = WINDOW_TREE_PANE;
        item.session = crate::SessionIdentity::session_id(&*s) as core::ffi::c_int;
        item.winlink = wl.idx;
        item.pane = (*wp).pane_id() as core::ffi::c_int;
        let item = *item;
        let mut ft = format_create(
            None,
            None,
            (FORMAT_PANE | (*wp).pane_id()) as core::ffi::c_int,
            0 as core::ffi::c_int,
        );
        format_defaults(&mut ft, None, Some(&*s), Some(&*wl), Some(&*wp));
        let text = format_expand(&mut ft, data.format.as_deref().unwrap_or(c""));
        let name = xasprintf(c"%u", fmt_args![idx]);
        let mti = ((*data).tree_ref())
            .add_item(
                parent,
                ModeTreeItemData::Tree(item),
                data.tag_for(WindowTreeTag::Pane(wp.pane_id())),
                &name,
                Some(&text),
                -(1 as core::ffi::c_int),
            )
            .expect("the parent belongs to this tree");
        mti.set_align(1).expect("the new item is live");
    }
}
unsafe fn window_tree_filter_pane(
    s: &mut session,
    wl: &mut winlink,
    wp: &mut impl crate::WindowPane,
    filter: Option<&CStr>,
) -> core::ffi::c_int {
    unsafe {
        let Some(filter) = filter else {
            return 1 as core::ffi::c_int;
        };
        let cp = format_single(None, filter, None, Some(&*s), Some(&*wl), Some(&*wp));
        let result: core::ffi::c_int = format_true(Some(&cp));
        result
    }
}
unsafe fn window_tree_build_window(
    s: &mut session,
    wl: &mut winlink,
    modedata: WindowModeData,
    sort_crit: &sort_criteria_t,
    parent: Option<&ModeTreeItemRef>,
    filter: Option<&CStr>,
) -> core::ffi::c_int {
    unsafe {
        let held = modedata.tree().expect("the mode holds its state");
        let mut data_guard = held.borrow_mut();
        let data = &mut *data_guard;

        let item = window_tree_add_item(&mut *data);
        item.type_0 = WINDOW_TREE_WINDOW;
        item.session = crate::SessionIdentity::session_id(&*s) as core::ffi::c_int;
        item.winlink = wl.idx;
        item.pane = -(1 as core::ffi::c_int);
        let item = *item;
        let window = wl.window_handle().expect("a link has a window").clone();
        let active_id = window
            .active_pane_id()
            .expect("the window has an active pane");
        let mut ft = format_create(None, None, (FORMAT_PANE | active_id) as core::ffi::c_int, 0);
        format_defaults(
            &mut ft,
            None,
            Some(&*s),
            Some(&*wl),
            None::<&crate::types::window_pane>,
        );
        let text = format_expand(&mut ft, data.format.as_deref().unwrap_or(c""));
        let name = xasprintf(c"%u", fmt_args![wl.idx]);
        let expanded: core::ffi::c_int = if data.type_0 as core::ffi::c_uint
            == WINDOW_TREE_SESSION as core::ffi::c_int as core::ffi::c_uint
            || data.type_0 as core::ffi::c_uint
                == WINDOW_TREE_WINDOW as core::ffi::c_int as core::ffi::c_uint
        {
            0 as core::ffi::c_int
        } else {
            1 as core::ffi::c_int
        };
        let mti = ((*data).tree_ref())
            .add_item(
                parent,
                ModeTreeItemData::Tree(item),
                data.tag_for(WindowTreeTag::Window(
                    crate::SessionIdentity::session_id(s),
                    wl.idx,
                )),
                &name,
                Some(&text),
                expanded,
            )
            .expect("the parent belongs to this tree");
        mti.set_align(1).expect("the new item is live");
        drop(data_guard);
        let pane_count = window.pane_count();
        let show_window = if pane_count == 1 {
            let pane_id = window.as_window().panes[0].pane_id();
            window.pane_by_id(pane_id).is_some_and(|mut pane| {
                pane.get_mut()
                    .is_some_and(|pane| window_tree_filter_pane(s, wl, pane, filter) != 0)
            })
        } else {
            pane_count != 0
        };
        if show_window {
            let panes = window.sorted_panes(sort_crit);
            if !panes.is_empty() {
                for mut pane in panes {
                    if pane
                        .get_mut()
                        .is_some_and(|pane| window_tree_filter_pane(s, wl, pane, filter) != 0)
                    {
                        let Some(pane) = pane.get_mut() else {
                            continue;
                        };
                        window_tree_build_pane(s, wl, pane, modedata.clone(), Some(&mti));
                    }
                }
                return 1;
            }
        }
        held.borrow_mut().item_list.pop();
        mti.remove();
        0 as core::ffi::c_int
    }
}
unsafe fn window_tree_build_session(
    s: &mut session,
    modedata: WindowModeData,
    sort_crit: &sort_criteria_t,
    filter: Option<&CStr>,
) {
    unsafe {
        let held = modedata.tree().expect("the mode holds its state");
        let mut data_guard = held.borrow_mut();
        let data = &mut *data_guard;
        let wl = s.curw().expect("the session has a current window");
        let mut empty: u_int;

        let item = window_tree_add_item(&mut *data);
        item.type_0 = WINDOW_TREE_SESSION;
        item.session = crate::SessionIdentity::session_id(&*s) as core::ffi::c_int;
        item.winlink = -(1 as core::ffi::c_int);
        item.pane = -(1 as core::ffi::c_int);
        let item = *item;
        let window = wl.window_handle().expect("a link has a window").clone();
        let active_id = window
            .active_pane_id()
            .expect("the window has an active pane");
        let mut ft = format_create(None, None, (FORMAT_PANE | active_id) as core::ffi::c_int, 0);
        format_defaults(
            &mut ft,
            None,
            Some(s),
            None,
            None::<&crate::types::window_pane>,
        );
        let text = format_expand(&mut ft, data.format.as_deref().unwrap_or(c""));
        let expanded: core::ffi::c_int = if data.type_0 as core::ffi::c_uint
            == WINDOW_TREE_SESSION as core::ffi::c_int as core::ffi::c_uint
        {
            0 as core::ffi::c_int
        } else {
            1 as core::ffi::c_int
        };
        let mti = ((*data).tree_ref())
            .add_item(
                None,
                ModeTreeItemData::Tree(item),
                data.tag_for(WindowTreeTag::Session(crate::SessionIdentity::session_id(
                    s,
                ))),
                crate::SessionNameState::session_name(&*s).expect("the session has a name"),
                Some(&text),
                expanded,
            )
            .expect("the parent belongs to this tree");
        drop(data_guard);
        let session = crate::session::session_ref_of(s).expect("the session has an owner");
        let mut l = session.sorted_winlinks(sort_crit);
        empty = 0 as u_int;
        for link in &mut l {
            let Some(wl) = link.get_mut() else {
                empty = empty.wrapping_add(1);
                continue;
            };
            if window_tree_build_window(s, wl, modedata.clone(), sort_crit, Some(&mti), filter) == 0
            {
                empty = empty.wrapping_add(1);
            }
        }
        if empty == l.len() as u_int {
            held.borrow_mut().item_list.pop();
            mti.remove();
        }
    }
}

fn window_tree_draw_label(
    writer: &mut impl ScreenWriteCtx,
    px: u_int,
    py: u_int,
    sx: u_int,
    sy: u_int,
    gc: &grid_cell,
    label: &[u8],
) {
    let mut width: u_int;

    let new_label: Option<CString>;
    if sx < 5 as u_int || sy < 3 as u_int {
        return;
    }
    let mut label = label;
    width = RustFormatText.width(label);
    if width > sx.wrapping_sub(4 as u_int) {
        new_label = Some(RustFormatText.trim_left(label, sx.wrapping_sub(4 as u_int)));
        label = new_label.as_ref().unwrap().to_bytes();
        width = RustFormatText.width(label);
    }
    if width == 0 as u_int {
        return;
    }
    let ox: u_int = sx
        .wrapping_sub(width)
        .wrapping_add(1 as u_int)
        .wrapping_div(2 as u_int);
    let oy: u_int = sy.wrapping_add(1 as u_int).wrapping_div(2 as u_int);
    writer.cursormove(
        px.wrapping_add(ox).wrapping_sub(2 as u_int) as core::ffi::c_int,
        py.wrapping_add(oy).wrapping_sub(1 as u_int) as core::ffi::c_int,
        0 as core::ffi::c_int,
    );
    writer.box_(
        width.wrapping_add(4 as u_int),
        3 as u_int,
        BOX_LINES_DEFAULT,
        None,
        None,
    );
    writer.cursormove(
        px.wrapping_add(ox).wrapping_sub(1 as u_int) as core::ffi::c_int,
        py.wrapping_add(oy) as core::ffi::c_int,
        0 as core::ffi::c_int,
    );
    writer.clearcharacter(width.wrapping_add(2 as u_int), 8 as u_int);
    writer.cursormove(
        px.wrapping_add(ox) as core::ffi::c_int,
        py.wrapping_add(oy) as core::ffi::c_int,
        0 as core::ffi::c_int,
    );
    writer.format_draw(gc, width, label, None, 0 as core::ffi::c_int);
}
unsafe fn window_tree_draw_session(
    data: &mut window_tree_modedata,
    s: &SessionRef,
    writer: &mut impl ScreenWriteCtx,
    sx: u_int,
    sy: u_int,
) {
    unsafe {
        let links: Vec<_> = crate::window::winlinks_in(s).collect();
        let (cx, cy) = writer.cursor_position();
        let mut visible: u_int;
        let each: u_int;
        let mut width: u_int;
        let mut offset: u_int;
        let mut start: u_int;
        let mut end: u_int;

        let mut gc;
        let mut left: core::ffi::c_int;
        let mut right: core::ffi::c_int;
        let total = links.len() as u_int;
        if total == 0 {
            return;
        }
        if sx.wrapping_div(total) < 24 as u_int {
            visible = sx.wrapping_div(24 as u_int);
            if visible == 0 as u_int {
                visible = 1 as u_int;
            }
        } else {
            visible = total;
        }
        let active_index = s.curw().map(|link| link.index());
        let current = links
            .iter()
            .position(|link| Some(link.index()) == active_index)
            .map_or(total, |index| index as u_int);
        if current < visible {
            start = 0 as u_int;
            end = visible;
        } else if current >= total.wrapping_sub(visible) {
            start = total.wrapping_sub(visible);
            end = total;
        } else {
            start = current.wrapping_sub(visible.wrapping_div(2 as u_int));
            end = start.wrapping_add(visible);
        }
        if data.offset < -(start as core::ffi::c_int) {
            data.offset = -(start as core::ffi::c_int);
        }
        if data.offset > total.wrapping_sub(end) as core::ffi::c_int {
            data.offset = total.wrapping_sub(end) as core::ffi::c_int;
        }
        start = start.wrapping_add(data.offset as u_int);
        end = end.wrapping_add(data.offset as u_int);
        left = (start != 0 as u_int) as core::ffi::c_int;
        right = (end != total) as core::ffi::c_int;
        if left != 0 && right != 0 && sx <= 6 as u_int
            || (left != 0 || right != 0) && sx <= 3 as u_int
        {
            right = 0 as core::ffi::c_int;
            left = right;
        }
        let remaining: u_int = if left != 0 && right != 0 {
            each = sx.wrapping_sub(6 as u_int).wrapping_div(visible);
            sx.wrapping_sub(6 as u_int)
                .wrapping_sub(visible.wrapping_mul(each))
        } else if left != 0 || right != 0 {
            each = sx.wrapping_sub(3 as u_int).wrapping_div(visible);
            sx.wrapping_sub(3 as u_int)
                .wrapping_sub(visible.wrapping_mul(each))
        } else {
            each = sx.wrapping_div(visible);
            sx.wrapping_sub(visible.wrapping_mul(each))
        };
        if each == 0 as u_int {
            return;
        }
        if left != 0 {
            data.left = cx.wrapping_add(2 as u_int) as core::ffi::c_int;
            writer.cursormove(
                cx.wrapping_add(2 as u_int) as core::ffi::c_int,
                cy as core::ffi::c_int,
                0 as core::ffi::c_int,
            );
            writer.vline(sy, 0 as core::ffi::c_int, 0 as core::ffi::c_int);
            writer.cursormove(
                cx as core::ffi::c_int,
                cy.wrapping_add(sy.wrapping_div(2 as u_int)) as core::ffi::c_int,
                0 as core::ffi::c_int,
            );
            writer.puts(&grid_default_cell, c"<", fmt_args![]);
        } else {
            data.left = -(1 as core::ffi::c_int);
        }
        if right != 0 {
            data.right = cx.wrapping_add(sx).wrapping_sub(3 as u_int) as core::ffi::c_int;
            writer.cursormove(
                cx.wrapping_add(sx).wrapping_sub(3 as u_int) as core::ffi::c_int,
                cy as core::ffi::c_int,
                0 as core::ffi::c_int,
            );
            writer.vline(sy, 0 as core::ffi::c_int, 0 as core::ffi::c_int);
            writer.cursormove(
                cx.wrapping_add(sx).wrapping_sub(1 as u_int) as core::ffi::c_int,
                cy.wrapping_add(sy.wrapping_div(2 as u_int)) as core::ffi::c_int,
                0 as core::ffi::c_int,
            );
            writer.puts(&grid_default_cell, c">", fmt_args![]);
        } else {
            data.right = -(1 as core::ffi::c_int);
        }
        data.start = start;
        data.end = end;
        data.each = each;
        for (position, link) in links
            .iter()
            .enumerate()
            .take(end as usize)
            .skip(start as usize)
        {
            let loop_0 = position as u_int;
            let i = loop_0.wrapping_sub(start);
            let Some(window) = link.get().and_then(|link| link.window_handle().cloned()) else {
                continue;
            };
            let oo = window.options();
            let mut ft = format_create(
                None,
                None,
                (FORMAT_WINDOW | window.window_id()) as core::ffi::c_int,
                0 as core::ffi::c_int,
            );
            format_defaults(
                &mut ft,
                None,
                Some(s.as_session()),
                link.get(),
                None::<&crate::types::window_pane>,
            );
            gc = grid_default_cell;
            style_apply(&mut gc, &oo, c"tree-mode-preview-style", Some(&mut ft));
            if left != 0 {
                offset = (3 as u_int).wrapping_add(i.wrapping_mul(each));
            } else {
                offset = i.wrapping_mul(each);
            }
            if loop_0 == end.wrapping_sub(1 as u_int) {
                width = each.wrapping_add(remaining);
            } else {
                width = each.wrapping_sub(1 as u_int);
            }
            writer.cursormove(
                cx.wrapping_add(offset) as core::ffi::c_int,
                cy as core::ffi::c_int,
                0 as core::ffi::c_int,
            );
            let active = window.active_pane_id().and_then(|id| window.pane_by_id(id));
            if let Some(pane) = active.as_ref().and_then(|pane| pane.get()) {
                writer.preview(pane.base(), width, sy);
            }
            let format = oo.string_ref(c"tree-mode-preview-format");
            if !format.is_empty() {
                let label = format_expand(&mut ft, &format);
                if !label.as_bytes().is_empty() {
                    window_tree_draw_label(
                        writer,
                        cx.wrapping_add(offset),
                        cy,
                        width,
                        sy,
                        &gc,
                        label.as_bytes(),
                    );
                }
            }
            if loop_0 != end.wrapping_sub(1 as u_int) {
                writer.cursormove(
                    cx.wrapping_add(offset).wrapping_add(width) as core::ffi::c_int,
                    cy as core::ffi::c_int,
                    0 as core::ffi::c_int,
                );
                writer.vline(sy, 0 as core::ffi::c_int, 0 as core::ffi::c_int);
            }
        }
    }
}
unsafe fn window_tree_draw_window(
    data: &mut window_tree_modedata,
    target: &WindowTreeTarget,
    writer: &mut impl ScreenWriteCtx,
    sx: u_int,
    sy: u_int,
) {
    unsafe {
        let panes: Vec<_> = target
            .window
            .as_window()
            .panes
            .iter()
            .map(|pane| {
                target
                    .window
                    .pane_by_id(pane.pane_id())
                    .expect("the window owns this pane")
            })
            .collect();
        let (cx, cy) = writer.cursor_position();
        let mut visible: u_int;
        let each: u_int;
        let mut width: u_int;
        let mut offset: u_int;
        let mut start: u_int;
        let mut end: u_int;

        let mut gc;
        let mut left: core::ffi::c_int;
        let mut right: core::ffi::c_int;
        let total = panes.len() as u_int;
        if total == 0 {
            return;
        }
        if sx.wrapping_div(total) < 24 as u_int {
            visible = sx.wrapping_div(24 as u_int);
            if visible == 0 as u_int {
                visible = 1 as u_int;
            }
        } else {
            visible = total;
        }
        let active_id = target.window.active_pane_id();
        let current = panes
            .iter()
            .position(|pane| Some(pane.id()) == active_id)
            .map_or(total, |index| index as u_int);
        if current < visible {
            start = 0 as u_int;
            end = visible;
        } else if current >= total.wrapping_sub(visible) {
            start = total.wrapping_sub(visible);
            end = total;
        } else {
            start = current.wrapping_sub(visible.wrapping_div(2 as u_int));
            end = start.wrapping_add(visible);
        }
        if data.offset < -(start as core::ffi::c_int) {
            data.offset = -(start as core::ffi::c_int);
        }
        if data.offset > total.wrapping_sub(end) as core::ffi::c_int {
            data.offset = total.wrapping_sub(end) as core::ffi::c_int;
        }
        start = start.wrapping_add(data.offset as u_int);
        end = end.wrapping_add(data.offset as u_int);
        left = (start != 0 as u_int) as core::ffi::c_int;
        right = (end != total) as core::ffi::c_int;
        if left != 0 && right != 0 && sx <= 6 as u_int
            || (left != 0 || right != 0) && sx <= 3 as u_int
        {
            right = 0 as core::ffi::c_int;
            left = right;
        }
        let remaining: u_int = if left != 0 && right != 0 {
            each = sx.wrapping_sub(6 as u_int).wrapping_div(visible);
            sx.wrapping_sub(6 as u_int)
                .wrapping_sub(visible.wrapping_mul(each))
        } else if left != 0 || right != 0 {
            each = sx.wrapping_sub(3 as u_int).wrapping_div(visible);
            sx.wrapping_sub(3 as u_int)
                .wrapping_sub(visible.wrapping_mul(each))
        } else {
            each = sx.wrapping_div(visible);
            sx.wrapping_sub(visible.wrapping_mul(each))
        };
        if each == 0 as u_int {
            return;
        }
        if left != 0 {
            data.left = cx.wrapping_add(2 as u_int) as core::ffi::c_int;
            writer.cursormove(
                cx.wrapping_add(2 as u_int) as core::ffi::c_int,
                cy as core::ffi::c_int,
                0 as core::ffi::c_int,
            );
            writer.vline(sy, 0 as core::ffi::c_int, 0 as core::ffi::c_int);
            writer.cursormove(
                cx as core::ffi::c_int,
                cy.wrapping_add(sy.wrapping_div(2 as u_int)) as core::ffi::c_int,
                0 as core::ffi::c_int,
            );
            writer.puts(&grid_default_cell, c"<", fmt_args![]);
        } else {
            data.left = -(1 as core::ffi::c_int);
        }
        if right != 0 {
            data.right = cx.wrapping_add(sx).wrapping_sub(3 as u_int) as core::ffi::c_int;
            writer.cursormove(
                cx.wrapping_add(sx).wrapping_sub(3 as u_int) as core::ffi::c_int,
                cy as core::ffi::c_int,
                0 as core::ffi::c_int,
            );
            writer.vline(sy, 0 as core::ffi::c_int, 0 as core::ffi::c_int);
            writer.cursormove(
                cx.wrapping_add(sx).wrapping_sub(1 as u_int) as core::ffi::c_int,
                cy.wrapping_add(sy.wrapping_div(2 as u_int)) as core::ffi::c_int,
                0 as core::ffi::c_int,
            );
            writer.puts(&grid_default_cell, c">", fmt_args![]);
        } else {
            data.right = -(1 as core::ffi::c_int);
        }
        data.start = start;
        data.end = end;
        data.each = each;
        for (position, pane) in panes
            .into_iter()
            .enumerate()
            .take(end as usize)
            .skip(start as usize)
        {
            let loop_0 = position as u_int;
            let i = loop_0.wrapping_sub(start);
            let Some(oo) = pane.get().map(|pane| pane.options_ref().clone()) else {
                continue;
            };
            let mut ft = format_create(
                None,
                None,
                (FORMAT_PANE | pane.id()) as core::ffi::c_int,
                0 as core::ffi::c_int,
            );
            format_defaults(
                &mut ft,
                None,
                Some(target.link.session().as_session()),
                target.link.get(),
                pane.get(),
            );
            gc = grid_default_cell;
            style_apply(&mut gc, &oo, c"tree-mode-preview-style", Some(&mut ft));
            if left != 0 {
                offset = (3 as u_int).wrapping_add(i.wrapping_mul(each));
            } else {
                offset = i.wrapping_mul(each);
            }
            if loop_0 == end.wrapping_sub(1 as u_int) {
                width = each.wrapping_add(remaining);
            } else {
                width = each.wrapping_sub(1 as u_int);
            }
            writer.cursormove(
                cx.wrapping_add(offset) as core::ffi::c_int,
                cy as core::ffi::c_int,
                0 as core::ffi::c_int,
            );
            if let Some(pane) = pane.get() {
                writer.preview(pane.base(), width, sy);
            }
            let format = oo.string_ref(c"tree-mode-preview-format");
            if !format.is_empty() {
                let label = format_expand(&mut ft, &format);
                if !label.as_bytes().is_empty() {
                    window_tree_draw_label(
                        writer,
                        cx.wrapping_add(offset),
                        cy,
                        width,
                        sy,
                        &gc,
                        label.as_bytes(),
                    );
                }
            }
            if loop_0 != end.wrapping_sub(1 as u_int) {
                writer.cursormove(
                    cx.wrapping_add(offset).wrapping_add(width) as core::ffi::c_int,
                    cy as core::ffi::c_int,
                    0 as core::ffi::c_int,
                );
                writer.vline(sy, 0 as core::ffi::c_int, 0 as core::ffi::c_int);
            }
        }
    }
}

unsafe fn window_tree_search(
    itemdata: ModeTreeItemData,
    ss: &CStr,
    icase: core::ffi::c_int,
) -> core::ffi::c_int {
    unsafe {
        let Some(item) = itemdata.tree() else {
            return 0;
        };
        let Some(target) = window_tree_resolve_item(&item) else {
            return 0;
        };
        let holds = |haystack: &CStr| match icase {
            0 => cstr_has(haystack, ss),
            _ => cstr_has_nocase(haystack, ss),
        };
        match item.type_0 {
            WINDOW_TREE_NONE => return 0 as core::ffi::c_int,
            WINDOW_TREE_SESSION => {
                return holds(
                    target
                        .link
                        .session()
                        .name()
                        .as_deref()
                        .expect("the session has a name"),
                ) as core::ffi::c_int;
            }
            WINDOW_TREE_WINDOW => {
                return holds(
                    target
                        .window
                        .window_name()
                        .as_deref()
                        .expect("a live window has a name"),
                ) as core::ffi::c_int;
            }
            WINDOW_TREE_PANE => {
                let Some(pane) = target.pane.as_ref().and_then(|pane| pane.get()) else {
                    return 0;
                };
                let cmd = osdep_get_name(*pane.fd()).filter(|cmd| !cmd.as_bytes().is_empty());
                let Some(cmd) = cmd else {
                    return 0 as core::ffi::c_int;
                };
                return holds(&cmd) as core::ffi::c_int;
            }
            _ => {}
        }
        0 as core::ffi::c_int
    }
}

unsafe fn window_tree_swap(
    cur_itemdata: ModeTreeItemData,
    other_itemdata: ModeTreeItemData,
    sort_crit: &sort_criteria_t,
) -> core::ffi::c_int {
    unsafe {
        let (Some(cur), Some(other)) = (cur_itemdata.tree(), other_itemdata.tree()) else {
            return 0;
        };
        if cur.type_0 != WINDOW_TREE_WINDOW || other.type_0 != WINDOW_TREE_WINDOW {
            return 0;
        }
        let Some(mut current) = window_tree_resolve_item(&cur) else {
            return 0;
        };
        let Some(mut other) = window_tree_resolve_item(&other) else {
            return 0;
        };
        if !current.link.session().ptr_eq(other.link.session())
            || current.link.index() == other.link.index()
        {
            return 0;
        }
        if sort_would_window_tree_swap(
            sort_crit,
            current.link.get().unwrap(),
            other.link.get().unwrap(),
        ) != 0
        {
            return 0;
        }
        let key_cur = (current.link.get().unwrap()).key();
        let key_other = (other.link.get().unwrap()).key();
        current
            .window
            .as_window_mut()
            .winlinks
            .retain(|held| !winlink_is(held, current.link.get().unwrap()));
        other
            .window
            .as_window_mut()
            .winlinks
            .retain(|held| !winlink_is(held, other.link.get().unwrap()));
        current.link.get_mut().unwrap().window_ref = Some(other.window.clone());
        other.link.get_mut().unwrap().window_ref = Some(current.window.clone());
        if let Some(key) = key_other {
            current.window.as_window_mut().winlinks.push(key);
        }
        if let Some(key) = key_cur {
            other.window.as_window_mut().winlinks.push(key);
        }
        let mut session = current.link.session().clone();
        let active_index = session.curw().map(|link| link.index());
        if active_index == Some(current.link.index()) {
            session.set_current(Some(other.link.index()));
        } else if active_index == Some(other.link.index()) {
            session.set_current(Some(current.link.index()));
        }
        session.synchronize_group_from();
        server_redraw_session_group(session.as_session_mut());
        recalculate_sizes();
        1
    }
}
fn window_tree_sort(sort_crit: &mut sort_criteria_t) {
    sort_crit.set_cycle(&window_tree_order_seq);
    if sort_crit.order() == SORT_END {
        sort_crit.set_order(window_tree_order_seq[0]);
    }
}
static window_tree_help_lines: [&CStr; 12] = [
    c"\r\x1B[1m      Enter \x1B[0m\x0Ex\x0F \x1B[0mChoose selected item\n",
    c"\r\x1B[1m       S-Up \x1B[0m\x0Ex\x0F \x1B[0mSwap current and previous window\n",
    c"\r\x1B[1m     S-Down \x1B[0m\x0Ex\x0F \x1B[0mSwap current and next window\n",
    c"\r\x1B[1m          x \x1B[0m\x0Ex\x0F \x1B[0mKill selected item\n",
    c"\r\x1B[1m          X \x1B[0m\x0Ex\x0F \x1B[0mKill tagged items\n",
    c"\r\x1B[1m          < \x1B[0m\x0Ex\x0F \x1B[0mScroll previews left\n",
    c"\r\x1B[1m          > \x1B[0m\x0Ex\x0F \x1B[0mScroll previews right\n",
    c"\r\x1B[1m          m \x1B[0m\x0Ex\x0F \x1B[0mSet the marked pane\n",
    c"\r\x1B[1m          M \x1B[0m\x0Ex\x0F \x1B[0mClear the marked pane\n",
    c"\r\x1B[1m          : \x1B[0m\x0Ex\x0F \x1B[0mRun a command for each tagged item\n",
    c"\r\x1B[1m          f \x1B[0m\x0Ex\x0F \x1B[0mEnter a format\n",
    c"\r\x1B[1m          H \x1B[0m\x0Ex\x0F \x1B[0mJump to the starting pane\n",
];
fn window_tree_help() -> (&'static [&'static CStr], u_int, &'static CStr) {
    (&window_tree_help_lines, 51 as u_int, c"item")
}
pub(crate) unsafe fn window_tree_init(
    wme: &mut window_mode_entry,
    mut pane: crate::window::RustWindowPaneWeak,
    fs: Option<&cmd_find_state>,
    args: Option<&RustArguments>,
) {
    unsafe {
        let data_ref = WindowTreeModeDataRef::new(window_tree_modedata::default());
        let mut data_guard = data_ref.borrow_mut();
        let data = &mut *data_guard;
        data.wp_ref = Some(pane.clone());
        wme.state = WindowModeState::Tree(data_ref.clone());
        if args.is_some_and(|args| args_has(args, b's') != 0) {
            data.type_0 = WINDOW_TREE_SESSION;
        } else if args.is_some_and(|args| args_has(args, b'w') != 0) {
            data.type_0 = WINDOW_TREE_WINDOW;
        } else {
            data.type_0 = WINDOW_TREE_PANE;
        }
        data.fs = fs.expect("a choose mode opens from a target").clone();
        data.format = Some(match args.and_then(|args| args_get_str(args, b'F')) {
            Some(value) => value.to_owned(),
            None => WINDOW_TREE_DEFAULT_FORMAT.to_owned(),
        });
        data.key_format = Some(match args.and_then(|args| args_get_str(args, b'K')) {
            Some(value) => value.to_owned(),
            None => WINDOW_TREE_DEFAULT_KEY_FORMAT.to_owned(),
        });
        data.command = Some(match args.and_then(|args| args_string_str(args, 0)) {
            Some(value) => value.to_owned(),
            None => WINDOW_TREE_DEFAULT_COMMAND.to_owned(),
        });
        data.squash_groups =
            !args.is_some_and(|args| args_has(args, b'G') != 0) as core::ffi::c_int;
        if args.is_some_and(|args| args_has(args, b'y') != 0) {
            data.prompt_flags = PROMPT_ACCEPT;
        }
        drop(data_guard);
        let build_data = data_ref.downgrade();
        let menu_data = data_ref.downgrade();
        let key_data = data_ref.downgrade();
        let draw_data = data_ref.downgrade();
        let mtd = ModeTreeDataRef::start(
            pane.get_mut().expect("the initializing pane still exists"),
            args,
            Some(std::rc::Rc::new(move |sort, tag, filter| {
                if let Some(data) = build_data.upgrade() {
                    data.build(sort, tag, filter);
                }
            })),
            Some(std::rc::Rc::new(move |itemdata, writer, sx, sy| {
                if let Some(data) = draw_data.upgrade() {
                    data.draw(itemdata, writer, sx, sy);
                }
            })),
            Some(std::rc::Rc::new(|itemdata, search, icase| {
                window_tree_search(itemdata, search, icase)
            })),
            Some(std::rc::Rc::new(move |c, key| {
                if let Some(data) = menu_data.upgrade() {
                    data.menu(c, key);
                }
            })),
            None,
            Some(std::rc::Rc::new(move |itemdata, line| {
                key_data
                    .upgrade()
                    .map_or(KEYC_NONE, |data| data.get_key(itemdata, line))
            })),
            Some(std::rc::Rc::new(|current, other, sort| {
                window_tree_swap(current, other, sort)
            })),
            Some(std::rc::Rc::new(window_tree_sort)),
            Some(window_tree_help()),
            WindowModeData::Tree(data_ref.downgrade()),
            &window_tree_menu_items,
        );
        data_ref.borrow_mut().data = Some(mtd.downgrade());
        wme.mode_tree_ref = Some(mtd);
        (data_ref.tree_ref()).zoom(args);
        (data_ref.tree_ref()).build();
        (data_ref.tree_ref()).draw();
        data_ref.borrow_mut().type_0 = WINDOW_TREE_NONE;
    }
}
pub(crate) unsafe fn window_tree_free(wme: &mut window_mode_entry) {
    unsafe {
        let Some(data) = wme.state.tree() else {
            return;
        };
        (data.tree_ref()).close();
    }
}
pub(crate) unsafe fn window_tree_resize(wme: &mut window_mode_entry, sx: u_int, sy: u_int) {
    unsafe {
        let data = wme.state.tree().expect("tree mode state");
        (data.tree_ref()).resize(sx, sy);
    }
}

unsafe fn window_tree_get_target(
    item: &window_tree_itemdata,
    fs: &mut cmd_find_state,
) -> Option<CString> {
    unsafe {
        let Some(target) = window_tree_resolve_item(item) else {
            cmd_find_clear_state(fs, 0);
            return None;
        };
        let name = target.link.session().name();
        let formatted = match item.type_0 {
            WINDOW_TREE_SESSION => Some(xasprintf(c"=%s:", fmt_args![name.as_deref()])),
            WINDOW_TREE_WINDOW => Some(xasprintf(
                c"=%s:%u.",
                fmt_args![name.as_deref(), target.link.index()],
            )),
            WINDOW_TREE_PANE => target.pane.as_ref().map(|pane| {
                xasprintf(
                    c"=%s:%u.%%%u",
                    fmt_args![name.as_deref(), target.link.index(), pane.id()],
                )
            }),
            _ => None,
        };
        if let Some(pane) = target.pane.as_ref().and_then(|pane| pane.get())
            && formatted.is_some()
        {
            cmd_find_from_winlink_pane(fs, target.link.get().unwrap(), pane, 0);
            formatted
        } else {
            cmd_find_clear_state(fs, 0);
            None
        }
    }
}
unsafe fn window_tree_command_each(
    modedata: WindowModeData,
    itemdata: ModeTreeItemData,
    c: &mut client,
) {
    unsafe {
        let held = modedata.tree().expect("the mode holds its state");
        let entered = held.borrow().entered.clone();
        let Some(item) = itemdata.tree() else {
            return;
        };
        let mut fs = cmd_find_state::default();
        if let Some(name) = window_tree_get_target(&item, &mut fs) {
            mode_tree_run_command(
                Some(c),
                Some(&fs),
                entered
                    .as_deref()
                    .expect("a tree mode has an entered command"),
                &name,
            );
        }
    }
}

fn window_tree_command_done(_item: &CmdqItemRef, data_weak: WindowTreeModeDataWeak) -> cmd_retval {
    unsafe {
        if let Some(data_ref) = data_weak.upgrade() {
            let data = &data_ref;
            (data.tree_ref()).build();
            (data.tree_ref()).draw();
            if let Some(mut pane) = data.pane()
                && let Some(pane) = pane.get_mut()
            {
                *pane.flags_mut() |= PANE_REDRAW;
            }
        }
        CMD_RETURN_NORMAL
    }
}

unsafe fn window_tree_kill_each(itemdata: ModeTreeItemData) {
    unsafe {
        let Some(item) = itemdata.tree() else {
            return;
        };
        let Some(target) = window_tree_resolve_item(&item) else {
            return;
        };
        match item.type_0 {
            WINDOW_TREE_SESSION => {
                let mut session = target.link.session().clone();
                server_destroy_session(session.as_session_mut());
                session.destroy(1, c"window_tree_kill_each");
            }
            WINDOW_TREE_WINDOW => {
                (target.window.clone()).kill(0);
            }
            WINDOW_TREE_PANE => {
                if let Some(pane) = target.pane {
                    server_kill_pane(&pane);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
#[path = "tree_focused_tests.rs"]
mod focused_tests;

impl WindowTreeModeDataRef {
    fn tree_ref(&self) -> ModeTreeDataRef {
        self.borrow().tree_ref()
    }
    fn pane(&self) -> Option<RustWindowPaneWeak> {
        self.borrow().pane()
    }
}

impl WindowTreeModeDataRef {
    unsafe fn build(self, sort_crit: &sort_criteria_t, tag: &mut uint64_t, filter: Option<&CStr>) {
        let held = self;

        unsafe {
            let modedata = WindowModeData::Tree(held.downgrade());
            let mut data_guard = held.borrow_mut();
            let data = &mut *data_guard;
            let fs = data.fs.clone();
            let type_0 = data.type_0;
            let squash_groups = data.squash_groups;
            data.item_list.clear();
            data.prune_tags();
            drop(data_guard);
            let current = fs.session();
            let l = sort_get_sessions(sort_crit);
            if l.is_empty() {
                return;
            }
            for mut s in l {
                let mut include = true;
                if squash_groups != 0 {
                    include = s
                        .with_group(|group| {
                            if let Some(current) = current.as_ref()
                                && group
                                    .sessions
                                    .iter()
                                    .any(|member| member.ptr_eq(&current.downgrade()))
                            {
                                s.ptr_eq(current)
                            } else {
                                group
                                    .sessions
                                    .iter()
                                    .find_map(SessionWeak::upgrade)
                                    .is_some_and(|member| member.ptr_eq(&s))
                            }
                        })
                        .unwrap_or(true);
                }
                if include {
                    window_tree_build_session(
                        s.as_session_mut(),
                        modedata.clone(),
                        sort_crit,
                        filter,
                    );
                }
            }
            let session_key = current.as_ref().map(|session| session.id());
            let window_key = session_key
                .zip(fs.wl_idx)
                .map(|(id, index)| WindowTreeTag::Window(id, index));
            let selection = match type_0 {
                WINDOW_TREE_SESSION => session_key.map(WindowTreeTag::Session),
                WINDOW_TREE_WINDOW => window_key,
                WINDOW_TREE_PANE => {
                    if fs.window().is_some_and(|window| window.pane_count() == 1) {
                        window_key
                    } else {
                        fs.wp_ref
                            .as_ref()
                            .map(|pane| pane.id())
                            .map(WindowTreeTag::Pane)
                    }
                }
                _ => None,
            };
            if let Some(selection) = selection {
                *tag = held.borrow_mut().tag_for(selection);
            }
        }
    }
    unsafe fn draw(
        self,
        itemdata: ModeTreeItemData,
        writer: &mut impl ScreenWriteCtx,
        sx: u_int,
        sy: u_int,
    ) {
        let held = self;

        unsafe {
            let mut data_guard = held.borrow_mut();
            let data = &mut *data_guard;
            let Some(item) = itemdata.tree() else {
                return;
            };
            let Some(target) = window_tree_resolve_item(&item) else {
                return;
            };
            let Some(pane) = target.pane.as_ref().and_then(|pane| pane.get()) else {
                return;
            };
            match item.type_0 {
                WINDOW_TREE_SESSION => {
                    window_tree_draw_session(data, target.link.session(), writer, sx, sy);
                }
                WINDOW_TREE_WINDOW => {
                    window_tree_draw_window(data, &target, writer, sx, sy);
                }
                WINDOW_TREE_PANE => {
                    writer.preview(pane.base(), sx, sy);
                }
                _ => {}
            };
        }
    }
    unsafe fn menu(self, c: &mut client, key: key_code) {
        let held = self;

        unsafe {
            let Some(pane) = held.pane() else {
                return;
            };
            let current = pane
                .get()
                .and_then(|pane| crate::window::window_pane_current_mode(pane))
                .is_some_and(
                    |mode| matches!(&mode.state, WindowModeState::Tree(data) if data.ptr_eq(&held)),
                );
            if current {
                held.key(c, key, None);
            }
        }
    }
    unsafe fn get_key(self, itemdata: ModeTreeItemData, line: u_int) -> key_code {
        let held = self;

        unsafe {
            let key_format = held.borrow().key_format.clone();
            let item = itemdata.tree().unwrap_or_default();

            let mut ft = format_create(None, None, FORMAT_NONE, 0 as core::ffi::c_int);
            let target = window_tree_resolve_item(&item);
            let session = target.as_ref().map(|target| target.link.session().clone());
            let link = target.as_ref().map(|target| target.link.clone());
            if item.type_0 as core::ffi::c_uint
                == WINDOW_TREE_SESSION as core::ffi::c_int as core::ffi::c_uint
            {
                format_defaults(
                    &mut ft,
                    None,
                    session.as_ref().map(|s| s.as_session()),
                    None,
                    None::<&crate::types::window_pane>,
                );
            } else if item.type_0 as core::ffi::c_uint
                == WINDOW_TREE_WINDOW as core::ffi::c_int as core::ffi::c_uint
            {
                format_defaults(
                    &mut ft,
                    None,
                    session.as_ref().map(|s| s.as_session()),
                    link.as_ref().and_then(WinlinkRef::get),
                    None::<&crate::types::window_pane>,
                );
            } else {
                format_defaults(
                    &mut ft,
                    None,
                    session.as_ref().map(|s| s.as_session()),
                    link.as_ref().and_then(WinlinkRef::get),
                    target
                        .as_ref()
                        .and_then(|target| target.pane.as_ref())
                        .and_then(|pane| pane.get()),
                );
            }
            format_add(&mut ft, c"line", c"%u", fmt_args![line]);
            let expanded = format_expand(&mut ft, key_format.as_deref().unwrap_or(c""));
            let key: key_code = RustKeyStringCodec.parse_key(&expanded);
            key
        }
    }
    pub(crate) unsafe fn update(self) {
        let data = self;

        unsafe {
            (data.tree_ref()).build();
            (data.tree_ref()).draw();
            if let Some(mut pane) = data.pane()
                && let Some(pane) = pane.get_mut()
            {
                *pane.flags_mut() |= PANE_REDRAW;
            }
        }
    }
    unsafe fn owner(&self) -> Option<WindowTreeModeDataWeak> {
        let data = self;

        Some(data.downgrade())
    }
    pub(crate) unsafe fn command(
        &self,
        c: &mut client,
        s: Option<&CStr>,
        _done: core::ffi::c_int,
    ) -> core::ffi::c_int {
        let data = self;

        unsafe {
            let Some(s) = s.filter(|s| !s.is_empty()) else {
                return 0 as core::ffi::c_int;
            };
            data.borrow_mut().entered = Some(s.to_owned());
            (data.tree_ref()).each_tagged(
                |modedata, itemdata| window_tree_command_each(modedata, itemdata, c),
                1 as core::ffi::c_int,
            );
            data.borrow_mut().entered = None;
            let Some(data_ref) = data.owner() else {
                return 0 as core::ffi::c_int;
            };
            cmdq_append(
                crate::server::client_ref_of(c).as_ref(),
                CmdqItemRef::callback_items(c"window_tree_command_done", move |item| {
                    window_tree_command_done(item, data_ref)
                }),
            );
            0 as core::ffi::c_int
        }
    }
    pub(crate) unsafe fn kill_current(
        &self,
        c: &mut client,
        s: Option<&CStr>,
        _done: core::ffi::c_int,
    ) -> core::ffi::c_int {
        let data = self;

        unsafe {
            let tree = data.tree_ref();
            let Some(s) = s.filter(|s| !s.is_empty()) else {
                return 0 as core::ffi::c_int;
            };
            if s.to_bytes().len() != 1 || tolower(s.to_bytes()[0]) != b'y' {
                return 0 as core::ffi::c_int;
            }
            window_tree_kill_each(tree.current_item());
            server_renumber_all();
            let Some(data_ref) = data.owner() else {
                return 0 as core::ffi::c_int;
            };
            cmdq_append(
                crate::server::client_ref_of(c).as_ref(),
                CmdqItemRef::callback_items(c"window_tree_command_done", move |item| {
                    window_tree_command_done(item, data_ref)
                }),
            );
            0 as core::ffi::c_int
        }
    }
    pub(crate) unsafe fn kill_tagged(
        &self,
        c: &mut client,
        s: Option<&CStr>,
        _done: core::ffi::c_int,
    ) -> core::ffi::c_int {
        let modedata = self;

        unsafe {
            let tree = modedata.tree_ref();
            let Some(s) = s.filter(|s| !s.is_empty()) else {
                return 0 as core::ffi::c_int;
            };
            if s.to_bytes().len() != 1 || tolower(s.to_bytes()[0]) != b'y' {
                return 0 as core::ffi::c_int;
            }
            tree.each_tagged(
                |_modedata, itemdata| window_tree_kill_each(itemdata),
                1 as core::ffi::c_int,
            );
            server_renumber_all();
            let Some(data_ref) = modedata.owner() else {
                return 0 as core::ffi::c_int;
            };
            cmdq_append(
                crate::server::client_ref_of(c).as_ref(),
                CmdqItemRef::callback_items(c"window_tree_command_done", move |item| {
                    window_tree_command_done(item, data_ref)
                }),
            );
            0 as core::ffi::c_int
        }
    }
    unsafe fn mouse(&self, key: key_code, mut x: u_int, item: &window_tree_itemdata) -> key_code {
        let owner = self;

        unsafe {
            let data = owner.borrow();
            let (left, right, start, end, each) =
                (data.left, data.right, data.start, data.end, data.each);
            drop(data);
            if key != KEYC_MOUSEDOWN1_PANE as core::ffi::c_ulong as key_code {
                return KEYC_NONE as core::ffi::c_ulong as key_code;
            }
            if left != -(1 as core::ffi::c_int) && x <= left as u_int {
                return '<' as i32 as key_code;
            }
            if right != -(1 as core::ffi::c_int) && x >= right as u_int {
                return '>' as i32 as key_code;
            }
            if left != -(1 as core::ffi::c_int) {
                x = x.wrapping_sub(left as u_int);
            } else if x != 0 as u_int {
                x = x.wrapping_sub(1);
            }
            if x == 0 as u_int || end == 0 as u_int {
                x = 0 as u_int;
            } else {
                x = x.wrapping_div(each);
                if start.wrapping_add(x) >= end {
                    x = end.wrapping_sub(1 as u_int);
                }
            }
            let Some(target) = window_tree_resolve_item(item) else {
                return KEYC_NONE;
            };
            match item.type_0 {
                WINDOW_TREE_SESSION => {
                    (owner.tree_ref()).expand_current();
                    if let Some(link) = target
                        .link
                        .session()
                        .as_session()
                        .windows
                        .values()
                        .nth(start.wrapping_add(x) as usize)
                    {
                        let key = WindowTreeTag::Window(target.link.session().id(), link.idx);
                        let tag = owner.borrow_mut().tag_for(key);
                        (owner.tree_ref()).set_current(tag);
                    }
                    '\r' as key_code
                }
                WINDOW_TREE_WINDOW => {
                    (owner.tree_ref()).expand_current();
                    if let Some(pane) = target
                        .window
                        .as_window()
                        .panes
                        .get(start.wrapping_add(x) as usize)
                    {
                        let tag = owner
                            .borrow_mut()
                            .tag_for(WindowTreeTag::Pane(pane.pane_id()));
                        (owner.tree_ref()).set_current(tag);
                    }
                    '\r' as key_code
                }
                _ => KEYC_NONE,
            }
        }
    }
    pub(crate) unsafe fn key(self, c: &mut client, mut key: key_code, m: Option<&mouse_event>) {
        let data = self;

        unsafe {
            let Some(mut pane) = data.pane() else {
                return;
            };
            let prompt_flags = data.borrow().prompt_flags;
            let tree = data.tree_ref();
            let mut fs = cmd_find_state::default();
            let fsp = data.borrow().fs.clone();
            let mut finished: core::ffi::c_int;
            let tagged: u_int;
            let x: u_int;
            let mut item = tree.current_item().tree();
            (finished, x, _) = tree.key(c, &mut key, m);
            loop {
                let new_item = tree.current_item().tree();
                if item != new_item {
                    item = new_item;
                    data.borrow_mut().offset = 0 as core::ffi::c_int;
                }
                if !((key as core::ffi::c_ulonglong & KEYC_MASK_KEY
                    == KEYC_MOUSE as core::ffi::c_ulong as core::ffi::c_ulonglong
                    || key as core::ffi::c_ulonglong & KEYC_MASK_TYPE
                        >= (KEYC_TYPE_MOUSEMOVE as core::ffi::c_int as core::ffi::c_ulonglong)
                            << 32 as core::ffi::c_int
                        && key as core::ffi::c_ulonglong & KEYC_MASK_TYPE
                            <= (KEYC_TYPE_TRIPLECLICK as core::ffi::c_int
                                as core::ffi::c_ulonglong)
                                << 32 as core::ffi::c_int)
                    && m.is_some())
                {
                    break;
                }
                key = item
                    .as_ref()
                    .map_or(KEYC_NONE, |item| data.mouse(key, x, item));
            }
            match key {
                60 => {
                    data.borrow_mut().offset -= 1;
                }
                62 => {
                    data.borrow_mut().offset += 1;
                }
                72 => {
                    let session_id = fsp.session().map(|session| session.id());
                    if let Some(id) = session_id {
                        let tag = data.borrow_mut().tag_for(WindowTreeTag::Session(id));
                        tree.expand(tag);
                    }
                    let window_tag = session_id.zip(fsp.wl_idx).map(|(id, index)| {
                        data.borrow_mut().tag_for(WindowTreeTag::Window(id, index))
                    });
                    if let Some(tag) = window_tag {
                        tree.expand(tag);
                    }
                    let pane_tag = data.borrow_mut().tag_for(WindowTreeTag::Pane(pane.id()));
                    if tree.set_current(pane_tag) == 0
                        && let Some(tag) = window_tag
                    {
                        tree.set_current(tag);
                    }
                }
                109 => {
                    let target = item
                        .as_ref()
                        .and_then(|item| window_tree_resolve_item(item));
                    server_set_marked(
                        target
                            .as_ref()
                            .map(|target| target.link.session().as_session()),
                        target.as_ref().and_then(|target| target.link.get()),
                        target
                            .as_ref()
                            .and_then(|target| target.pane.as_ref())
                            .and_then(|pane| pane.get()),
                    );
                    tree.build();
                }
                77 => {
                    server_clear_marked();
                    tree.build();
                }
                120 => {
                    let target = item
                        .as_ref()
                        .and_then(|item| window_tree_resolve_item(item));
                    let prompt = match item.map(|item| item.type_0).unwrap_or(WINDOW_TREE_NONE) {
                        WINDOW_TREE_SESSION => target.as_ref().map(|target| {
                            xasprintf(
                                c"Kill session %s? ",
                                fmt_args![target.link.session().name().as_deref()],
                            )
                        }),
                        WINDOW_TREE_WINDOW => target.as_ref().map(|target| {
                            xasprintf(c"Kill window %u? ", fmt_args![target.link.index()])
                        }),
                        WINDOW_TREE_PANE => target
                            .as_ref()
                            .and_then(|target| target.pane.as_ref())
                            .and_then(|pane| {
                                let window = pane.window()?;
                                let (found, index) =
                                    window_pane_index(&window.as_window(), pane.get()?);
                                (found == 0).then(|| xasprintf(c"Kill pane %u? ", fmt_args![index]))
                            }),
                        _ => None,
                    };
                    if let Some(prompt) = prompt {
                        let data_ref = data.owner().expect("window tree owner");
                        status_prompt_set(
                            c,
                            None,
                            &prompt,
                            Some(c""),
                            Prompt::WindowTreeKillCurrent,
                            PromptData::WindowTree(Box::new(data_ref)),
                            PROMPT_SINGLE | PROMPT_NOFORMAT | prompt_flags,
                            PromptHistoryType::Command,
                        );
                    }
                }
                88 => {
                    tagged = tree.count_tagged();
                    if !(tagged == 0 as u_int) {
                        let prompt = xasprintf(c"Kill %u tagged? ", fmt_args![tagged]);
                        let data_ref = data.owner().expect("window tree owner");
                        status_prompt_set(
                            c,
                            None,
                            &prompt,
                            Some(c""),
                            Prompt::WindowTreeKillTagged,
                            PromptData::WindowTree(Box::new(data_ref)),
                            PROMPT_SINGLE | PROMPT_NOFORMAT | prompt_flags,
                            PromptHistoryType::Command,
                        );
                    }
                }
                58 => {
                    tagged = tree.count_tagged();
                    let prompt = if tagged != 0 as u_int {
                        xasprintf(c"(%u tagged) ", fmt_args![tagged])
                    } else {
                        xasprintf(c"(current) ", fmt_args![])
                    };
                    let data_ref = data.owner().expect("window tree owner");
                    status_prompt_set(
                        c,
                        None,
                        &prompt,
                        Some(c""),
                        Prompt::WindowTreeCommand,
                        PromptData::WindowTree(Box::new(data_ref)),
                        PROMPT_NOFORMAT,
                        PromptHistoryType::Command,
                    );
                }
                13 => {
                    if let Some(name) = item
                        .as_ref()
                        .and_then(|item| window_tree_get_target(item, &mut fs))
                    {
                        let command = data.borrow().command.clone();
                        mode_tree_run_command(
                            Some(&mut *c),
                            None,
                            command.as_deref().expect("a tree mode has a command"),
                            &name,
                        );
                    }
                    finished = 1 as core::ffi::c_int;
                }
                _ => {}
            }
            if finished != 0 {
                if let Some(pane) = pane.get_mut() {
                    window_pane_reset_mode(pane);
                }
            } else {
                tree.draw();
                if let Some(pane) = pane.get_mut() {
                    *pane.flags_mut() |= PANE_REDRAW;
                }
            };
        }
    }
}

#[cfg(test)]
pub use crate::consts::{KEYC_DOWN, KEYC_END, KEYC_HOME, KEYC_NPAGE, KEYC_PPAGE, KEYC_UP};
