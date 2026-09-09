use super::widget::mode_tree_run_command;
use crate::WindowPane;
use crate::arguments::{args_get_str, args_string_str};
use crate::fmt_args;
use crate::format::format_true;
use crate::format::{format_add, format_create, format_defaults, format_expand, format_single};
#[cfg(test)]
use crate::pane_identity::PaneIdentity;
#[cfg(test)]
use crate::screen::RustScreen;
use crate::screen::ScreenWriteCtx;
use crate::server::server_client_how_many;
use crate::server::{server_client_detach, server_client_suspend};

pub use crate::consts::{
    BOX_LINES_DEFAULT, CLIENT_UNATTACHEDFLAGS, FORMAT_NONE, KEYC_NONE, MSG_DETACH, MSG_DETACHKILL,
    PANE_REDRAW, SORT_ACTIVITY, SORT_CREATION, SORT_END, SORT_NAME, SORT_SIZE,
};
use crate::sort::{SortCriteria, sort_get_clients};
use crate::status::{status_at_line, status_line_size};
use crate::text::{KeyStringCodec, RustKeyStringCodec};
pub use crate::types::*;
use crate::window::RustWindowPaneWeak;
use crate::window::window_pane_reset_mode;
use ::core::ffi::CStr;

#[repr(C)]
pub struct window_client_modedata {
    /// The pane the mode is running in.
    pub wp_ref: Option<RustWindowPaneWeak>,
    pub(crate) data: Option<ModeTreeDataWeak>,
    pub format: Option<std::ffi::CString>,
    pub key_format: Option<std::ffi::CString>,
    pub command: Option<std::ffi::CString>,
    pub item_list: Vec<std::rc::Rc<window_client_itemdata>>,
    /// The mode's observation of itself, which the tree it shows holds it by.
    pub(crate) owner: Option<WindowClientModeDataWeak>,
}

impl window_client_modedata {
    /// The pane the mode is running in, retained directly until this reference is dropped.
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
#[repr(C)]
pub struct window_client_itemdata {
    pub(crate) client_ref: ClientRef,
}

impl window_client_itemdata {
    /// The client retained by this row.
    pub(crate) fn client(&self) -> ClientRef {
        self.client_ref.clone()
    }
}

pub const WINDOW_CLIENT_DEFAULT_COMMAND: &CStr = c"detach-client -t '%%'";
pub static WINDOW_CLIENT_DEFAULT_FORMAT: &CStr = c"#{t/p:client_activity}: session #{session_name}";
pub const WINDOW_CLIENT_DEFAULT_KEY_FORMAT: &CStr =
    c"#{?#{e|<:#{line},10},#{line},#{e|<:#{line},36},M-#{a:#{e|+:97,#{e|-:#{line},10}}}}";
static window_client_menu_items: [menu_item<'static>; 8] = [
    menu_item {
        name: Some(c"Detach"),
        key: 'd' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: Some(c"Detach Tagged"),
        key: 'D' as i32 as key_code,
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
        name: Some(c"Cancel"),
        key: 'q' as i32 as key_code,
        command: None,
    },
];
static window_client_order_seq: [sort_order; 4] =
    [SORT_NAME, SORT_SIZE, SORT_CREATION, SORT_ACTIVITY];
fn window_client_add_item(
    data: &mut window_client_modedata,
    client: ClientRef,
) -> std::rc::Rc<window_client_itemdata> {
    let item = std::rc::Rc::new(window_client_itemdata { client_ref: client });
    data.item_list.push(item.clone());
    item
}

unsafe fn window_client_draw(
    itemdata: ModeTreeItemData,
    writer: &mut impl ScreenWriteCtx,
    sx: u_int,
    sy: u_int,
) {
    unsafe {
        let Some(item) = itemdata.client() else {
            return;
        };
        let c = item.client();
        let (cx, cy) = writer.cursor_position();
        let mut lines: u_int;

        if c.flags() & CLIENT_UNATTACHEDFLAGS as uint64_t != 0 {
            return;
        }
        let Some(session) = c.attached_session() else {
            return;
        };
        let Some(window) = session.current_window() else {
            return;
        };
        let Some(active) = window.active_pane_id() else {
            return;
        };
        let Some(pane) = window.pane_by_id(active) else {
            return;
        };
        lines = status_line_size(c.as_client());
        if lines >= sy {
            lines = 0 as u_int;
        }
        let at: u_int = if status_at_line(c.as_client()) == 0 as core::ffi::c_int {
            lines
        } else {
            0 as u_int
        };
        writer.cursormove(
            cx as core::ffi::c_int,
            cy.wrapping_add(at) as core::ffi::c_int,
            0 as core::ffi::c_int,
        );
        writer.preview(
            pane.get().expect("the preview pane is live").base(),
            sx,
            sy.wrapping_sub(2 as u_int).wrapping_sub(lines),
        );
        if at != 0 as u_int {
            writer.cursormove(
                cx as core::ffi::c_int,
                cy.wrapping_add(2 as u_int) as core::ffi::c_int,
                0 as core::ffi::c_int,
            );
        } else {
            writer.cursormove(
                cx as core::ffi::c_int,
                cy.wrapping_add(sy)
                    .wrapping_sub(1 as u_int)
                    .wrapping_sub(lines) as core::ffi::c_int,
                0 as core::ffi::c_int,
            );
        }
        writer.hline(
            sx,
            0 as core::ffi::c_int,
            0 as core::ffi::c_int,
            BOX_LINES_DEFAULT,
            None,
        );
        if at != 0 as u_int {
            writer.cursormove(
                cx as core::ffi::c_int,
                cy as core::ffi::c_int,
                0 as core::ffi::c_int,
            );
        } else {
            writer.cursormove(
                cx as core::ffi::c_int,
                cy.wrapping_add(sy).wrapping_sub(lines) as core::ffi::c_int,
                0 as core::ffi::c_int,
            );
        }
        writer.fast_copy(
            &c.as_client().status.screen,
            0 as u_int,
            0 as u_int,
            sx,
            lines,
        );
    }
}

fn window_client_sort(sort_crit: &mut sort_criteria_t) {
    sort_crit.set_cycle(&window_client_order_seq);
    if sort_crit.order() == SORT_END {
        sort_crit.set_order(window_client_order_seq[0]);
    }
}
static window_client_help_lines: [&CStr; 8] = [
    c"\r\x1B[1m      Enter \x1B[0m\x0Ex\x0F \x1B[0mChoose selected %1\n",
    c"\r\x1B[1m          d \x1B[0m\x0Ex\x0F \x1B[0mDetach selected %1\n",
    c"\r\x1B[1m          D \x1B[0m\x0Ex\x0F \x1B[0mDetach tagged %1s\n",
    c"\r\x1B[1m          x \x1B[0m\x0Ex\x0F \x1B[0mDetach selected %1\n",
    c"\r\x1B[1m          X \x1B[0m\x0Ex\x0F \x1B[0mDetach tagged %1s\n",
    c"\r\x1B[1m          z \x1B[0m\x0Ex\x0F \x1B[0mSuspend selected %1\n",
    c"\r\x1B[1m          Z \x1B[0m\x0Ex\x0F \x1B[0mSuspend tagged %1s\n",
    c"\r\x1B[1m          f \x1B[0m\x0Ex\x0F \x1B[0mEnter a filter\n",
];
fn window_client_help() -> (&'static [&'static CStr], u_int, &'static CStr) {
    (&window_client_help_lines, 0 as u_int, c"client")
}
pub(crate) unsafe fn window_client_init(
    wme: &mut window_mode_entry,
    mut pane: crate::window::RustWindowPaneWeak,
    _fs: Option<&cmd_find_state>,
    args: Option<&args>,
) {
    unsafe {
        let format = match args.and_then(|args| args_get_str(args, b'F')) {
            Some(value) => value.to_owned(),
            None => WINDOW_CLIENT_DEFAULT_FORMAT.to_owned(),
        };
        let key_format = match args.and_then(|args| args_get_str(args, b'K')) {
            Some(value) => value.to_owned(),
            None => WINDOW_CLIENT_DEFAULT_KEY_FORMAT.to_owned(),
        };
        let command = match args.and_then(|args| args_string_str(args, 0)) {
            Some(value) => value.to_owned(),
            None => WINDOW_CLIENT_DEFAULT_COMMAND.to_owned(),
        };
        let data_ref = WindowClientModeDataRef::new(window_client_modedata {
            wp_ref: Some(pane.clone()),
            data: None,
            format: Some(format),
            key_format: Some(key_format),
            command: Some(command),
            item_list: Vec::new(),
            owner: None,
        });
        wme.state = WindowModeState::Client(data_ref.clone());
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
                window_client_draw(itemdata, writer, sx, sy)
            })),
            None,
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
            Some(std::rc::Rc::new(window_client_sort)),
            Some(window_client_help()),
            WindowModeData::Client(data_ref.downgrade()),
            &window_client_menu_items,
        );
        data_ref.borrow_mut().data = Some(mtd.downgrade());
        mtd.zoom(args);
        mtd.build();
        mtd.draw();
        wme.mode_tree_ref = Some(mtd);
    }
}
pub(crate) unsafe fn window_client_free(wme: &mut window_mode_entry) {
    let Some(owner) = wme.state.client() else {
        return;
    };
    let tree = owner.borrow().tree_ref();
    unsafe { tree.close() };
}
pub(crate) unsafe fn window_client_resize(wme: &mut window_mode_entry, sx: u_int, sy: u_int) {
    let owner = wme.state.client().expect("client mode state");
    let tree = owner.borrow().tree_ref();
    unsafe { tree.resize(sx, sy) };
}

unsafe fn window_client_do_detach(
    modedata: WindowModeData,
    itemdata: ModeTreeItemData,
    key: key_code,
) {
    unsafe {
        let held = modedata.client().expect("the mode holds its state");
        let tree = held.borrow().tree_ref();
        let Some(item) = itemdata.client() else {
            return;
        };
        if tree
            .current_item()
            .client()
            .is_some_and(|current| std::rc::Rc::ptr_eq(&item, &current))
        {
            tree.down(0 as core::ffi::c_int);
        }
        let mut c = item.client();
        if key == 'd' as i32 as key_code || key == 'D' as i32 as key_code {
            server_client_detach(c.as_client_mut(), MSG_DETACH);
        } else if key == 'x' as i32 as key_code || key == 'X' as i32 as key_code {
            server_client_detach(c.as_client_mut(), MSG_DETACHKILL);
        } else if key == 'z' as i32 as key_code || key == 'Z' as i32 as key_code {
            server_client_suspend(c.as_client_mut());
        }
    }
}

#[cfg(test)]
#[path = "../tests/test_modes_client_focused.rs"]
mod focused_tests;

impl WindowClientModeDataRef {
    unsafe fn build(self, sort_crit: &sort_criteria_t, _tag: &mut uint64_t, filter: Option<&CStr>) {
        let held = self;

        unsafe {
            let mut data = held.borrow_mut();
            data.item_list.clear();
            for client in sort_get_clients(sort_crit) {
                if client.attached_session().is_none()
                    || client.flags() & CLIENT_UNATTACHEDFLAGS as uint64_t != 0
                {
                    continue;
                }
                window_client_add_item(&mut data, client);
            }
            for item in &data.item_list {
                let client = item.client();
                if let Some(filter) = filter {
                    let value = format_single(
                        None,
                        filter,
                        Some(client.as_client()),
                        None,
                        None,
                        None::<&crate::types::window_pane>,
                    );
                    if format_true(Some(&value)) == 0 {
                        continue;
                    }
                }
                let text = format_single(
                    None,
                    data.format.as_deref().unwrap_or(c""),
                    Some(client.as_client()),
                    None,
                    None,
                    None::<&crate::types::window_pane>,
                );
                (data.tree_ref())
                    .add_item(
                        None,
                        ModeTreeItemData::Client(item.clone()),
                        client.id(),
                        &client.name().map(std::ffi::CStr::to_owned).expect("a name"),
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
                    |mode| matches!(&mode.state, WindowModeState::Client(data) if data.ptr_eq(&held)),
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
            let Some(item) = itemdata.client() else {
                return KEYC_NONE;
            };

            let mut ft = format_create(None, None, FORMAT_NONE, 0 as core::ffi::c_int);
            let c = item.client();
            format_defaults(
                &mut ft,
                Some(c.as_client()),
                None,
                None,
                None::<&crate::types::window_pane>,
            );
            format_add(&mut ft, c"line", c"%u", fmt_args![line]);
            let expanded = format_expand(&mut ft, key_format.as_deref().unwrap_or(c""));
            let key: key_code = RustKeyStringCodec.parse_key(&expanded);
            key
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
            (finished, _, _) = tree.key(c, &mut key, m);
            match key {
                100 | 120 | 122 => {
                    window_client_do_detach(
                        WindowModeData::Client(owner.downgrade()),
                        tree.current_item(),
                        key,
                    );
                    tree.build();
                }
                68 | 88 | 90 => {
                    tree.each_tagged(
                        |modedata, itemdata| window_client_do_detach(modedata, itemdata, key),
                        0 as core::ffi::c_int,
                    );
                    tree.build();
                }
                13 => {
                    let Some(item) = tree.current_item().client() else {
                        return;
                    };
                    let selected = item.client();
                    let command = owner
                        .borrow()
                        .command
                        .clone()
                        .expect("a client mode has a command");
                    mode_tree_run_command(
                        Some(&mut *c),
                        None,
                        &command,
                        selected
                            .as_client()
                            .ttyname
                            .as_deref()
                            .expect("an attached client has a tty name"),
                    );
                    finished = 1 as core::ffi::c_int;
                }
                _ => {}
            }
            if finished != 0 || server_client_how_many() == 0 as u_int {
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
