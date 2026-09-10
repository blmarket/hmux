use super::widget::mode_tree_run_command;
use crate::WindowPane;
use crate::args::RustArguments;
use crate::cmd::{cmd_find_copy_state, cmd_find_valid_state};
pub use crate::consts::{
    FORMAT_NONE, KEYC_NONE, PANE_REDRAW, SORT_CREATION, SORT_END, SORT_NAME, SORT_SIZE, VIS_CSTYLE,
    VIS_OCTAL, VIS_TAB,
};
use crate::fmt_args;
use crate::format::format_true;
use crate::format::{
    format_add, format_create, format_defaults, format_defaults_paste_buffer, format_expand,
};
use crate::grid::grid_default_cell;
use crate::overlay::popup_editor;
#[cfg(test)]
use crate::pane_identity::PaneIdentity;
use crate::paste::{PasteBufferStore, with_paste_buffers, with_paste_buffers_mut};
use crate::screen::ScreenWriteCtx;
use crate::sort::{SortCriteria, sort_get_buffers};
use crate::text::{KeyStringCodec, RustKeyStringCodec};
use crate::text::{RustUtf8VisModel, Utf8VisModel};
pub use crate::types::*;
use crate::window::window_pane_current_mode;
use crate::window::{
    RustWindowPaneWeak, WinlinkRef, window_pane_find_by_id, window_pane_reset_mode,
};
use ::core::ffi::CStr;

#[repr(C)]
pub struct window_buffer_modedata {
    /// The pane the mode is running in.
    pub wp: Option<RustWindowPaneWeak>,
    pub fs: cmd_find_state,
    pub(crate) data: Option<ModeTreeDataRef>,
    pub command: Option<std::ffi::CString>,
    pub format: Option<std::ffi::CString>,
    pub key_format: Option<std::ffi::CString>,
    pub item_list: Vec<std::rc::Rc<window_buffer_itemdata>>,
    /// The mode's observation of itself, which the tree it shows holds it by.
    pub(crate) owner: Option<WindowBufferModeDataWeak>,
}

impl window_buffer_modedata {
    /// The pane the mode is running in, retained directly until this reference is dropped.
    pub(crate) fn pane(&self) -> Option<RustWindowPaneWeak> {
        self.wp.as_ref().filter(|pane| pane.is_alive()).cloned()
    }

    /// The mode tree the mode is showing, as a handle. Only ever asked of a
    /// mode that is showing one.
    pub(crate) fn tree_ref(&self) -> ModeTreeDataRef {
        self.data
            .as_ref()
            .cloned()
            .expect("the mode is showing a tree")
    }
}
#[repr(C)]
pub struct window_buffer_editdata {
    pub wp_id: u_int,
    pub name: Option<std::ffi::CString>,
    /// The order of the buffer the editor was opened on. A buffer keeps the
    /// order it was made with for as long as it lives, so the buffer standing
    /// under the name when the editor closes is the same one only if it
    /// carries this order.
    pub order: u_int,
}

pub const WINDOW_BUFFER_DEFAULT_COMMAND: &CStr = c"paste-buffer -p -b '%%'";
pub static WINDOW_BUFFER_DEFAULT_FORMAT: &CStr = c"#{t/p:buffer_created}: #{buffer_sample}";
pub const WINDOW_BUFFER_DEFAULT_KEY_FORMAT: &CStr =
    c"#{?#{e|<:#{line},10},#{line},#{e|<:#{line},36},M-#{a:#{e|+:97,#{e|-:#{line},10}}}}";
static window_buffer_menu_items: [menu_item<'static>; 11] = [
    menu_item {
        name: Some(c"Paste"),
        key: 'p' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: Some(c"Paste Tagged"),
        key: 'P' as i32 as key_code,
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
        name: Some(c"Delete"),
        key: 'd' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: Some(c"Delete Tagged"),
        key: 'D' as i32 as key_code,
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
static window_buffer_order_seq: [sort_order; 3] = [SORT_CREATION, SORT_NAME, SORT_SIZE];
unsafe fn window_buffer_format_context(
    fs: &cmd_find_state,
) -> (
    Option<SessionRef>,
    Option<WinlinkRef>,
    Option<RustWindowPaneWeak>,
) {
    if unsafe { cmd_find_valid_state(fs) } == 0 {
        return (None, None, None);
    }
    (fs.session(), fs.winlink_ref(), fs.pane_ref())
}

fn window_buffer_draw(
    itemdata: ModeTreeItemData,
    writer: &mut impl ScreenWriteCtx,
    sx: u_int,
    sy: u_int,
) {
    {
        let Some(item) = itemdata.buffer() else {
            return;
        };
        let (cx, cy) = writer.cursor_position();
        let Some(bytes) = with_paste_buffers(|buffers| {
            item.name
                .as_deref()
                .and_then(|name| buffers.get(name))
                .map(|buffer| buffer.data.to_vec())
        }) else {
            return;
        };
        // One row per line of the buffer, so the walk is over the newlines
        // rather than a pointer stepped to each of them.
        for (i, line) in bytes
            .split(|&byte| byte as core::ffi::c_int == '\n' as i32)
            .take(sy as usize)
            .enumerate()
        {
            let visible = RustUtf8VisModel.encode_utf8(line, VIS_OCTAL | VIS_CSTYLE | VIS_TAB);
            if !visible.as_bytes().is_empty() {
                writer.cursormove(
                    cx as core::ffi::c_int,
                    cy.wrapping_add(i as u_int) as core::ffi::c_int,
                    0 as core::ffi::c_int,
                );
                writer.nputs(
                    sx as ssize_t,
                    &grid_default_cell,
                    c"%s",
                    fmt_args![visible.as_c_str()],
                );
            }
        }
    }
}
/// Whether the buffer's bytes hold `find`. Unlike [`bytes_have`], nothing
/// matches an empty pattern here: a buffer is searched for something.
fn window_buffer_find(data: &[u8], find: &[u8], icase: core::ffi::c_int) -> core::ffi::c_int {
    if find.is_empty() {
        return 0 as core::ffi::c_int;
    }
    let found = match icase {
        0 => bytes_have(data, find),
        _ => bytes_have_nocase(data, find),
    };
    found as core::ffi::c_int
}
fn window_buffer_search(
    itemdata: ModeTreeItemData,
    ss: &CStr,
    icase: core::ffi::c_int,
) -> core::ffi::c_int {
    {
        let Some(item) = itemdata.buffer() else {
            return 0;
        };
        let name = item.name.as_deref().expect("a buffer item has a name");
        let Some(bytes) =
            with_paste_buffers(|buffers| buffers.get(name).map(|buffer| buffer.data.to_vec()))
        else {
            return 0 as core::ffi::c_int;
        };
        let named = match icase {
            0 => cstr_has(name, ss),
            _ => cstr_has_nocase(name, ss),
        };
        if named {
            return 1 as core::ffi::c_int;
        }
        window_buffer_find(&bytes, ss.to_bytes(), icase)
    }
}

fn window_buffer_sort(sort_crit: &mut sort_criteria_t) {
    sort_crit.set_cycle(&window_buffer_order_seq);
    if sort_crit.order() == SORT_END {
        sort_crit.set_order(window_buffer_order_seq[0]);
    }
}
static window_buffer_help_lines: [&CStr; 7] = [
    c"\r\x1B[1m      Enter \x1B[0m\x0Ex\x0F \x1B[0mPaste selected %1\n",
    c"\r\x1B[1m          p \x1B[0m\x0Ex\x0F \x1B[0mPaste selected %1\n",
    c"\r\x1B[1m          P \x1B[0m\x0Ex\x0F \x1B[0mPaste tagged %1s\n",
    c"\r\x1B[1m          d \x1B[0m\x0Ex\x0F \x1B[0mDelete selected %1\n",
    c"\r\x1B[1m          D \x1B[0m\x0Ex\x0F \x1B[0mDelete tagged %1s\n",
    c"\r\x1B[1m          e \x1B[0m\x0Ex\x0F \x1B[0mOpen %1 in editor\n",
    c"\r\x1B[1m          f \x1B[0m\x0Ex\x0F \x1B[0mEnter a filter\n",
];
fn window_buffer_help() -> (&'static [&'static CStr], u_int, &'static CStr) {
    (&window_buffer_help_lines, 0 as u_int, c"buffer")
}
pub(crate) unsafe fn window_buffer_init(
    wme: &mut window_mode_entry,
    mut pane: crate::window::RustWindowPaneWeak,
    fs: Option<&cmd_find_state>,
    args: Option<&RustArguments>,
) {
    unsafe {
        let mut state = cmd_find_state::default();
        cmd_find_copy_state(&mut state, fs.expect("a choose mode opens from a target"));
        let format = match args.and_then(|args| args.argument_flag_string(b'F')) {
            Some(value) => value.to_owned(),
            None => WINDOW_BUFFER_DEFAULT_FORMAT.to_owned(),
        };
        let key_format = match args.and_then(|args| args.argument_flag_string(b'K')) {
            Some(value) => value.to_owned(),
            None => WINDOW_BUFFER_DEFAULT_KEY_FORMAT.to_owned(),
        };
        let command = match args.and_then(|args| args.argument_string(0)) {
            Some(value) => value.to_owned(),
            None => WINDOW_BUFFER_DEFAULT_COMMAND.to_owned(),
        };
        let data_ref = WindowBufferModeDataRef::new(window_buffer_modedata {
            wp: Some(pane.clone()),
            fs: state,
            data: None,
            command: Some(command),
            format: Some(format),
            key_format: Some(key_format),
            item_list: Vec::new(),
            owner: None,
        });
        wme.state = WindowModeState::Buffer(data_ref.clone());
        let build_data = data_ref.downgrade();
        let menu_data = data_ref.downgrade();
        let key_data = data_ref.downgrade();
        let mtd = ModeTreeDataRef::start(
            pane.get_mut().expect("the initializing pane still exists"),
            args,
            Some(std::rc::Rc::new(move |sort, tag, filter| {
                if let Some(data) = build_data.upgrade() {
                    data.build(sort, tag, filter);
                }
            })),
            Some(std::rc::Rc::new(|itemdata, writer, sx, sy| {
                window_buffer_draw(itemdata, writer, sx, sy)
            })),
            Some(std::rc::Rc::new(|itemdata, search, icase| {
                window_buffer_search(itemdata, search, icase)
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
            None,
            Some(std::rc::Rc::new(window_buffer_sort)),
            Some(window_buffer_help()),
            WindowModeData::Buffer(data_ref.downgrade()),
            &window_buffer_menu_items,
        );
        data_ref.borrow_mut().data = Some(mtd.clone());
        mtd.zoom(args);
        mtd.build();
        mtd.draw();
    }
}
pub(crate) unsafe fn window_buffer_free(wme: &mut window_mode_entry) {
    let Some(owner) = wme.state.buffer() else {
        return;
    };
    let tree = owner.borrow_mut().data.take().expect("the mode is showing a tree");
    unsafe { tree.close() };
}
pub(crate) unsafe fn window_buffer_resize(wme: &mut window_mode_entry, sx: u_int, sy: u_int) {
    let owner = wme.state.buffer().expect("buffer mode state");
    let tree = owner.borrow().tree_ref();
    unsafe { tree.resize(sx, sy) };
}

fn window_buffer_do_delete(modedata: WindowModeData, itemdata: ModeTreeItemData) {
    {
        let held = modedata.buffer().expect("the mode holds its state");
        let tree = held.borrow().tree_ref();
        let Some(item) = itemdata.buffer() else {
            return;
        };
        if tree
            .current_item()
            .buffer()
            .is_some_and(|current| std::rc::Rc::ptr_eq(&item, &current))
            && tree.down(0 as core::ffi::c_int) == 0
        {
            tree.up(0 as core::ffi::c_int);
        }
        if let Some(name) = item.name.as_deref() {
            with_paste_buffers_mut(|buffers| buffers.remove(name));
        }
    }
}
unsafe fn window_buffer_do_paste(
    modedata: WindowModeData,
    itemdata: ModeTreeItemData,
    c: &mut client,
) {
    unsafe {
        let held = modedata.buffer().expect("the mode holds its state");
        let command = held
            .borrow()
            .command
            .clone()
            .expect("a buffer mode has a command");
        let Some(item) = itemdata.buffer() else {
            return;
        };
        let name = item.name.as_deref().expect("a buffer item has a name");
        if with_paste_buffers(|buffers| buffers.get(name).is_some()) {
            mode_tree_run_command(Some(c), None, &command, name);
        }
    }
}
unsafe fn window_buffer_edit_close_cb(mut buf: Vec<u8>, ed: Box<window_buffer_editdata>) {
    unsafe {
        if buf.is_empty() {
            drop(ed);
            return;
        }
        let Some(name) = ed.name.as_deref() else {
            drop(ed);
            return;
        };
        let Some(oldbuf) = with_paste_buffers(|buffers| {
            buffers
                .get(name)
                .and_then(|buffer| (buffer.order == ed.order).then(|| buffer.data.to_vec()))
        }) else {
            drop(ed);
            return;
        };
        if !oldbuf.is_empty() && oldbuf[oldbuf.len() - 1] != b'\n' && buf.ends_with(b"\n") {
            buf.pop();
        }
        if !buf.is_empty() {
            with_paste_buffers_mut(|buffers| buffers.replace(name, buf));
        }
        if let Some(mut pane) = window_pane_find_by_id(ed.wp_id) {
            let tree = pane
                .get()
                .and_then(|pane| window_pane_current_mode(pane))
                .filter(|mode| mode.mode() == WindowMode::Buffer)
                .and_then(|mode| mode.state.buffer())
                .map(|owner| owner.borrow().tree_ref());
            if let Some(tree) = tree {
                tree.build();
                tree.draw();
            }
            if let Some(pane) = pane.get_mut() {
                *pane.flags_mut() |= PANE_REDRAW;
            }
        }
        drop(ed);
    }
}
unsafe fn window_buffer_start_edit(wp_id: u_int, item: &window_buffer_itemdata, c: &mut client) {
    unsafe {
        let Some((name, order, bytes)) = with_paste_buffers(|buffers| {
            item.name
                .as_deref()
                .and_then(|name| buffers.get(name))
                .map(|buffer| (buffer.name.to_owned(), buffer.order, buffer.data.to_vec()))
        }) else {
            return;
        };
        let ed = Box::new(window_buffer_editdata {
            wp_id,
            name: Some(name),
            order,
        });
        popup_editor(
            &mut *c,
            &bytes,
            Box::new(move |buf| window_buffer_edit_close_cb(buf, ed)),
        );
    }
}

#[cfg(test)]
#[path = "../tests/test_window_buffer.rs"]
mod tests;

impl WindowBufferModeDataRef {
    unsafe fn build(self, sort_crit: &sort_criteria_t, _tag: &mut uint64_t, filter: Option<&CStr>) {
        let held = self;

        unsafe {
            let mut data = held.borrow_mut();
            data.item_list.clear();
            for buffer in sort_get_buffers(sort_crit) {
                data.item_list
                    .push(std::rc::Rc::new(window_buffer_itemdata {
                        name: Some(buffer.name),
                        size: buffer.size as size_t,
                        order: buffer.order,
                    }));
            }
            let (s, wl, wp) = window_buffer_format_context(&data.fs);
            for item in &data.item_list {
                let Some(name) = item.name.as_deref() else {
                    continue;
                };
                if with_paste_buffers(|buffers| buffers.get(name).is_none()) {
                    continue;
                }
                let mut ft = format_create(None, None, FORMAT_NONE, 0);
                format_defaults(
                    &mut ft,
                    None,
                    s.as_ref().map(|session| session.as_session()),
                    wl.as_ref().and_then(WinlinkRef::get),
                    wp.as_ref().and_then(|pane| pane.get()),
                );
                format_defaults_paste_buffer(&mut ft, name);
                if let Some(filter) = filter {
                    let value = format_expand(&mut ft, filter);
                    if format_true(Some(&value)) == 0 {
                        continue;
                    }
                }
                let text = format_expand(&mut ft, data.format.as_deref().unwrap_or(c""));
                (data.tree_ref())
                    .add_item(
                        None,
                        ModeTreeItemData::Buffer(item.clone()),
                        item.order as uint64_t,
                        name,
                        Some(&text),
                        -1,
                    )
                    .expect("a widget row can be inserted");
            }
        }
    }
    unsafe fn menu(self, c: &mut client, key: key_code) {
        let held = self;

        unsafe {
            let Some(pane) = held.borrow().pane() else {
                return;
            };
            let current = pane
                .get()
                .and_then(|pane| crate::window::window_pane_current_mode(pane))
                .is_some_and(
                    |mode| matches!(&mode.state, WindowModeState::Buffer(data) if data.ptr_eq(&held)),
                );
            if current {
                held.key(c, key, None);
            }
        }
    }
    unsafe fn get_key(self, itemdata: ModeTreeItemData, line: u_int) -> key_code {
        let held = self;

        unsafe {
            let data = held.borrow();
            let Some(item) = itemdata.buffer() else {
                return KEYC_NONE;
            };
            let (s, wl, wp) = window_buffer_format_context(&data.fs);
            let Some(name) = item.name.as_deref() else {
                return KEYC_NONE as core::ffi::c_ulong as key_code;
            };
            if with_paste_buffers(|buffers| buffers.get(name).is_none()) {
                return KEYC_NONE as core::ffi::c_ulong as key_code;
            }
            let mut ft = format_create(None, None, FORMAT_NONE, 0 as core::ffi::c_int);
            format_defaults(
                &mut ft,
                None,
                None,
                None,
                None::<&crate::types::window_pane>,
            );
            format_defaults(
                &mut ft,
                None,
                s.as_ref().map(|session| session.as_session()),
                wl.as_ref().and_then(WinlinkRef::get),
                wp.as_ref().and_then(|pane| pane.get()),
            );
            format_defaults_paste_buffer(&mut ft, name);
            format_add(&mut ft, c"line", c"%u", fmt_args![line]);
            let expanded = format_expand(&mut ft, data.key_format.as_deref().unwrap_or(c""));
            RustKeyStringCodec.parse_key(&expanded)
        }
    }
    pub(crate) unsafe fn update(self) {
        let owner = self;

        let tree = owner.borrow().tree_ref();
        unsafe {
            tree.build();
            tree.draw();
            if let Some(mut pane) = owner.borrow().pane()
                && let Some(pane) = pane.get_mut()
            {
                *pane.flags_mut() |= PANE_REDRAW;
            }
        }
    }
    pub(crate) unsafe fn key(self, c: &mut client, mut key: key_code, m: Option<&mouse_event>) {
        let owner = self;

        unsafe {
            let tree = owner.borrow().tree_ref();
            let mut finished: core::ffi::c_int;
            if with_paste_buffers(PasteBufferStore::is_empty) {
                finished = 1 as core::ffi::c_int;
            } else {
                (finished, _, _) = tree.key(c, &mut key, m);
                match key {
                    101 => {
                        if let Some(item) = tree.current_item().buffer() {
                            window_buffer_start_edit(
                                owner.borrow().wp.as_ref().unwrap().id(),
                                &item,
                                c,
                            );
                        }
                    }
                    100 => {
                        window_buffer_do_delete(
                            WindowModeData::Buffer(owner.downgrade()),
                            tree.current_item(),
                        );
                        tree.build();
                    }
                    68 => {
                        tree.each_tagged(
                            |modedata, itemdata| window_buffer_do_delete(modedata, itemdata),
                            0 as core::ffi::c_int,
                        );
                        tree.build();
                    }
                    80 => {
                        tree.each_tagged(
                            |modedata, itemdata| window_buffer_do_paste(modedata, itemdata, c),
                            0 as core::ffi::c_int,
                        );
                        finished = 1 as core::ffi::c_int;
                    }
                    112 | 13 => {
                        window_buffer_do_paste(
                            WindowModeData::Buffer(owner.downgrade()),
                            tree.current_item(),
                            c,
                        );
                        finished = 1 as core::ffi::c_int;
                    }
                    _ => {}
                }
            }
            if finished != 0 || with_paste_buffers(PasteBufferStore::is_empty) {
                let pane = tree.borrow().pane();
                if let Some(mut pane) = pane
                    && let Some(pane) = pane.get_mut()
                {
                    window_pane_reset_mode(pane);
                }
            } else {
                tree.draw();
                let pane = tree.borrow().pane();
                if let Some(mut pane) = pane
                    && let Some(pane) = pane.get_mut()
                {
                    *pane.flags_mut() |= PANE_REDRAW;
                }
            };
        }
    }
}
