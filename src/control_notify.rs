pub use crate::src::shared::events::{event_payload, events_cb, events_sink};
pub use crate::src::shared::client::{clients};
pub use crate::src::shared::arguments::{args};
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
pub use crate::src::shared::command::{cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds};
pub use crate::src::shared::control::{control_state};
pub use crate::src::shared::format::{format_job_tree, format_tree};
pub use crate::src::shared::input::{input_ctx, input_request, input_requests};
pub use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
pub use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
pub use crate::src::shared::menu::{menu_data};
pub use crate::src::shared::options::{options};
pub use crate::src::shared::pane::{
    window_pane, window_pane_entry, window_pane_modes, window_pane_prompt, window_pane_sentry,
    window_pane_tree_entry, window_pane_zentry, window_panes,
};
pub use crate::src::shared::process::{tmuxpeer};
pub use crate::src::shared::prompt::{prompt};
pub use crate::src::shared::redraw::{redraw_scene};
pub use crate::src::shared::session::{session, session_entry, session_gentry};
pub use crate::src::shared::spawn::{spawn_editor_state};
pub use crate::src::shared::status::{status_line};
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
pub use crate::src::shared::environment::{environ};
pub use crate::src::shared::format::{FORMAT_NONE};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::client::{CLIENT_CONTROL, CLIENT_EXIT};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{mouse_event};
use crate::src::shared::client::*;
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

    fn free(__ptr: *mut ::core::ffi::c_void);
    fn format_create(
        _: *mut client,
        _: *mut cmdq_item,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    ) -> *mut format_tree;
    fn format_free(_: *mut format_tree);
    fn format_expand(
        _: *mut format_tree,
        _: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn format_defaults(
        _: *mut format_tree,
        _: *mut client,
        _: *mut session,
        _: *mut winlink,
        _: *mut window_pane,
    );
    fn event_payload_get_string(
        _: *mut event_payload,
        _: *const ::core::ffi::c_char,
    ) -> *const ::core::ffi::c_char;
    fn event_payload_print(
        _: *mut event_payload,
        _: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn event_payload_get_client(
        _: *mut event_payload,
        _: *const ::core::ffi::c_char,
    ) -> *mut client;
    fn event_payload_get_session(
        _: *mut event_payload,
        _: *const ::core::ffi::c_char,
    ) -> *mut session;
    fn event_payload_get_window(
        _: *mut event_payload,
        _: *const ::core::ffi::c_char,
    ) -> *mut window;
    fn event_payload_get_pane(
        _: *mut event_payload,
        _: *const ::core::ffi::c_char,
    ) -> *mut window_pane;
    fn events_add_sink(
        _: *const ::core::ffi::c_char,
        _: events_cb,
        _: *mut ::core::ffi::c_void,
    ) -> *mut events_sink;
    static mut clients: clients;
    fn winlink_find_by_window_id(_: *mut winlinks, _: u_int) -> *mut winlink;
    fn control_notify_write(_: *mut client, _: *const ::core::ffi::c_char, ...);
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

#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_35 {
    pub name: *const ::core::ffi::c_char,
    pub cb: events_cb,
}

unsafe extern "C" fn control_pane_mode_changed_cb(
    mut name: *const ::core::ffi::c_char,
    mut ep: *mut event_payload,
    mut sink_data: *mut ::core::ffi::c_void,
) {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    wp = event_payload_get_pane(ep, b"pane\0" as *const u8 as *const ::core::ffi::c_char);
    if !wp.is_null() {
        c = clients.tqh_first;
        while !c.is_null() {
            if !c.is_null()
                && (*c).flags & CLIENT_CONTROL as uint64_t != 0
                && !(*c).flags & CLIENT_EXIT as uint64_t != 0
                && !(*c).control_state.is_null()
            {
                control_notify_write(
                    c,
                    b"%%pane-mode-changed %%%u\0" as *const u8 as *const ::core::ffi::c_char,
                    (*wp).id,
                );
            }
            c = (*c).entry.tqe_next;
        }
        return;
    }
    value = event_payload_print(ep, b"pane\0" as *const u8 as *const ::core::ffi::c_char);
    if value.is_null() {
        return;
    }
    c = clients.tqh_first;
    while !c.is_null() {
        if !c.is_null()
            && (*c).flags & CLIENT_CONTROL as uint64_t != 0
            && !(*c).flags & CLIENT_EXIT as uint64_t != 0
            && !(*c).control_state.is_null()
        {
            control_notify_write(
                c,
                b"%%pane-mode-changed %s\0" as *const u8 as *const ::core::ffi::c_char,
                value,
            );
        }
        c = (*c).entry.tqe_next;
    }
    free(value as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn control_window_layout_changed_cb(
    mut name: *const ::core::ffi::c_char,
    mut ep: *mut event_payload,
    mut sink_data: *mut ::core::ffi::c_void,
) {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut w: *mut window =
        event_payload_get_window(ep, b"window\0" as *const u8 as *const ::core::ffi::c_char);
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut template: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if w.is_null() {
        return;
    }
    template = b"%layout-change #{window_id} #{window_layout} #{window_visible_layout} #{window_raw_flags}\0"
        as *const u8 as *const ::core::ffi::c_char;
    if (*w).winlinks.tqh_first.is_null() || (*w).layout_root.is_null() {
        return;
    }
    c = clients.tqh_first;
    while !c.is_null() {
        if !(!(!c.is_null()
            && (*c).flags & CLIENT_CONTROL as uint64_t != 0
            && !(*c).flags & CLIENT_EXIT as uint64_t != 0
            && !(*c).control_state.is_null())
            || (*c).session.is_null())
        {
            s = (*c).session;
            wl = winlink_find_by_window_id(&raw mut (*s).windows, (*w).id);
            if !wl.is_null() {
                ft = format_create(
                    c,
                    ::core::ptr::null_mut::<cmdq_item>(),
                    FORMAT_NONE,
                    0 as ::core::ffi::c_int,
                );
                format_defaults(ft, c, s, wl, ::core::ptr::null_mut::<window_pane>());
                cp = format_expand(ft, template);
                format_free(ft);
                control_notify_write(c, b"%s\0" as *const u8 as *const ::core::ffi::c_char, cp);
                free(cp as *mut ::core::ffi::c_void);
            }
        }
        c = (*c).entry.tqe_next;
    }
}
unsafe extern "C" fn control_window_pane_changed_cb(
    mut name: *const ::core::ffi::c_char,
    mut ep: *mut event_payload,
    mut sink_data: *mut ::core::ffi::c_void,
) {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut w: *mut window =
        event_payload_get_window(ep, b"window\0" as *const u8 as *const ::core::ffi::c_char);
    if w.is_null() || (*w).active.is_null() {
        return;
    }
    c = clients.tqh_first;
    while !c.is_null() {
        if !c.is_null()
            && (*c).flags & CLIENT_CONTROL as uint64_t != 0
            && !(*c).flags & CLIENT_EXIT as uint64_t != 0
            && !(*c).control_state.is_null()
        {
            control_notify_write(
                c,
                b"%%window-pane-changed @%u %%%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*w).id,
                (*(*w).active).id,
            );
        }
        c = (*c).entry.tqe_next;
    }
}
unsafe extern "C" fn control_window_unlinked_cb(
    mut name: *const ::core::ffi::c_char,
    mut ep: *mut event_payload,
    mut sink_data: *mut ::core::ffi::c_void,
) {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut cs: *mut session = ::core::ptr::null_mut::<session>();
    let mut w: *mut window =
        event_payload_get_window(ep, b"window\0" as *const u8 as *const ::core::ffi::c_char);
    if w.is_null() {
        return;
    }
    c = clients.tqh_first;
    while !c.is_null() {
        if !(!(!c.is_null()
            && (*c).flags & CLIENT_CONTROL as uint64_t != 0
            && !(*c).flags & CLIENT_EXIT as uint64_t != 0
            && !(*c).control_state.is_null())
            || (*c).session.is_null())
        {
            cs = (*c).session;
            if !winlink_find_by_window_id(&raw mut (*cs).windows, (*w).id).is_null() {
                control_notify_write(
                    c,
                    b"%%window-close @%u\0" as *const u8 as *const ::core::ffi::c_char,
                    (*w).id,
                );
            } else {
                control_notify_write(
                    c,
                    b"%%unlinked-window-close @%u\0" as *const u8 as *const ::core::ffi::c_char,
                    (*w).id,
                );
            }
        }
        c = (*c).entry.tqe_next;
    }
}
unsafe extern "C" fn control_window_linked_cb(
    mut name: *const ::core::ffi::c_char,
    mut ep: *mut event_payload,
    mut sink_data: *mut ::core::ffi::c_void,
) {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut cs: *mut session = ::core::ptr::null_mut::<session>();
    let mut w: *mut window =
        event_payload_get_window(ep, b"window\0" as *const u8 as *const ::core::ffi::c_char);
    if w.is_null() {
        return;
    }
    c = clients.tqh_first;
    while !c.is_null() {
        if !(!(!c.is_null()
            && (*c).flags & CLIENT_CONTROL as uint64_t != 0
            && !(*c).flags & CLIENT_EXIT as uint64_t != 0
            && !(*c).control_state.is_null())
            || (*c).session.is_null())
        {
            cs = (*c).session;
            if !winlink_find_by_window_id(&raw mut (*cs).windows, (*w).id).is_null() {
                control_notify_write(
                    c,
                    b"%%window-add @%u\0" as *const u8 as *const ::core::ffi::c_char,
                    (*w).id,
                );
            } else {
                control_notify_write(
                    c,
                    b"%%unlinked-window-add @%u\0" as *const u8 as *const ::core::ffi::c_char,
                    (*w).id,
                );
            }
        }
        c = (*c).entry.tqe_next;
    }
}
unsafe extern "C" fn control_window_renamed_cb(
    mut name: *const ::core::ffi::c_char,
    mut ep: *mut event_payload,
    mut sink_data: *mut ::core::ffi::c_void,
) {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut cs: *mut session = ::core::ptr::null_mut::<session>();
    let mut w: *mut window =
        event_payload_get_window(ep, b"window\0" as *const u8 as *const ::core::ffi::c_char);
    if w.is_null() {
        return;
    }
    c = clients.tqh_first;
    while !c.is_null() {
        if !(!(!c.is_null()
            && (*c).flags & CLIENT_CONTROL as uint64_t != 0
            && !(*c).flags & CLIENT_EXIT as uint64_t != 0
            && !(*c).control_state.is_null())
            || (*c).session.is_null())
        {
            cs = (*c).session;
            if !winlink_find_by_window_id(&raw mut (*cs).windows, (*w).id).is_null() {
                control_notify_write(
                    c,
                    b"%%window-renamed @%u %s\0" as *const u8 as *const ::core::ffi::c_char,
                    (*w).id,
                    (*w).name,
                );
            } else {
                control_notify_write(
                    c,
                    b"%%unlinked-window-renamed @%u %s\0" as *const u8
                        as *const ::core::ffi::c_char,
                    (*w).id,
                    (*w).name,
                );
            }
        }
        c = (*c).entry.tqe_next;
    }
}
unsafe extern "C" fn control_client_session_changed_cb(
    mut name: *const ::core::ffi::c_char,
    mut ep: *mut event_payload,
    mut sink_data: *mut ::core::ffi::c_void,
) {
    let mut cc: *mut client =
        event_payload_get_client(ep, b"client\0" as *const u8 as *const ::core::ffi::c_char);
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    if cc.is_null() || (*cc).session.is_null() {
        return;
    }
    s = (*cc).session;
    c = clients.tqh_first;
    while !c.is_null() {
        if !(!(!c.is_null()
            && (*c).flags & CLIENT_CONTROL as uint64_t != 0
            && !(*c).flags & CLIENT_EXIT as uint64_t != 0
            && !(*c).control_state.is_null())
            || (*c).session.is_null())
        {
            if cc == c {
                control_notify_write(
                    c,
                    b"%%session-changed $%u %s\0" as *const u8 as *const ::core::ffi::c_char,
                    (*s).id,
                    (*s).name,
                );
            } else {
                control_notify_write(
                    c,
                    b"%%client-session-changed %s $%u %s\0" as *const u8
                        as *const ::core::ffi::c_char,
                    (*cc).name,
                    (*s).id,
                    (*s).name,
                );
            }
        }
        c = (*c).entry.tqe_next;
    }
}
unsafe extern "C" fn control_client_detached_cb(
    mut name: *const ::core::ffi::c_char,
    mut ep: *mut event_payload,
    mut sink_data: *mut ::core::ffi::c_void,
) {
    let mut cc: *mut client =
        event_payload_get_client(ep, b"client\0" as *const u8 as *const ::core::ffi::c_char);
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    if cc.is_null() {
        return;
    }
    c = clients.tqh_first;
    while !c.is_null() {
        if !c.is_null()
            && (*c).flags & CLIENT_CONTROL as uint64_t != 0
            && !(*c).flags & CLIENT_EXIT as uint64_t != 0
            && !(*c).control_state.is_null()
        {
            control_notify_write(
                c,
                b"%%client-detached %s\0" as *const u8 as *const ::core::ffi::c_char,
                (*cc).name,
            );
        }
        c = (*c).entry.tqe_next;
    }
}
unsafe extern "C" fn control_session_renamed_cb(
    mut name: *const ::core::ffi::c_char,
    mut ep: *mut event_payload,
    mut sink_data: *mut ::core::ffi::c_void,
) {
    let mut s: *mut session =
        event_payload_get_session(ep, b"session\0" as *const u8 as *const ::core::ffi::c_char);
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    if s.is_null() {
        return;
    }
    c = clients.tqh_first;
    while !c.is_null() {
        if !c.is_null()
            && (*c).flags & CLIENT_CONTROL as uint64_t != 0
            && !(*c).flags & CLIENT_EXIT as uint64_t != 0
            && !(*c).control_state.is_null()
        {
            control_notify_write(
                c,
                b"%%session-renamed $%u %s\0" as *const u8 as *const ::core::ffi::c_char,
                (*s).id,
                (*s).name,
            );
        }
        c = (*c).entry.tqe_next;
    }
}
unsafe extern "C" fn control_session_created_cb(
    mut name: *const ::core::ffi::c_char,
    mut ep: *mut event_payload,
    mut sink_data: *mut ::core::ffi::c_void,
) {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    c = clients.tqh_first;
    while !c.is_null() {
        if !c.is_null()
            && (*c).flags & CLIENT_CONTROL as uint64_t != 0
            && !(*c).flags & CLIENT_EXIT as uint64_t != 0
            && !(*c).control_state.is_null()
        {
            control_notify_write(
                c,
                b"%%sessions-changed\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        c = (*c).entry.tqe_next;
    }
}
unsafe extern "C" fn control_session_closed_cb(
    mut name: *const ::core::ffi::c_char,
    mut ep: *mut event_payload,
    mut sink_data: *mut ::core::ffi::c_void,
) {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    c = clients.tqh_first;
    while !c.is_null() {
        if !c.is_null()
            && (*c).flags & CLIENT_CONTROL as uint64_t != 0
            && !(*c).flags & CLIENT_EXIT as uint64_t != 0
            && !(*c).control_state.is_null()
        {
            control_notify_write(
                c,
                b"%%sessions-changed\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        c = (*c).entry.tqe_next;
    }
}
unsafe extern "C" fn control_session_window_changed_cb(
    mut name: *const ::core::ffi::c_char,
    mut ep: *mut event_payload,
    mut sink_data: *mut ::core::ffi::c_void,
) {
    let mut s: *mut session =
        event_payload_get_session(ep, b"session\0" as *const u8 as *const ::core::ffi::c_char);
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    if s.is_null() || (*s).curw.is_null() {
        return;
    }
    c = clients.tqh_first;
    while !c.is_null() {
        if !c.is_null()
            && (*c).flags & CLIENT_CONTROL as uint64_t != 0
            && !(*c).flags & CLIENT_EXIT as uint64_t != 0
            && !(*c).control_state.is_null()
        {
            control_notify_write(
                c,
                b"%%session-window-changed $%u @%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*s).id,
                (*(*(*s).curw).window).id,
            );
        }
        c = (*c).entry.tqe_next;
    }
}
unsafe extern "C" fn control_paste_buffer_changed_cb(
    mut name: *const ::core::ffi::c_char,
    mut ep: *mut event_payload,
    mut sink_data: *mut ::core::ffi::c_void,
) {
    let mut pbname: *const ::core::ffi::c_char = event_payload_get_string(
        ep,
        b"paste_buffer\0" as *const u8 as *const ::core::ffi::c_char,
    );
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    if pbname.is_null() {
        return;
    }
    c = clients.tqh_first;
    while !c.is_null() {
        if !c.is_null()
            && (*c).flags & CLIENT_CONTROL as uint64_t != 0
            && !(*c).flags & CLIENT_EXIT as uint64_t != 0
            && !(*c).control_state.is_null()
        {
            control_notify_write(
                c,
                b"%%paste-buffer-changed %s\0" as *const u8 as *const ::core::ffi::c_char,
                pbname,
            );
        }
        c = (*c).entry.tqe_next;
    }
}
unsafe extern "C" fn control_paste_buffer_deleted_cb(
    mut name: *const ::core::ffi::c_char,
    mut ep: *mut event_payload,
    mut sink_data: *mut ::core::ffi::c_void,
) {
    let mut pbname: *const ::core::ffi::c_char = event_payload_get_string(
        ep,
        b"paste_buffer\0" as *const u8 as *const ::core::ffi::c_char,
    );
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    if pbname.is_null() {
        return;
    }
    c = clients.tqh_first;
    while !c.is_null() {
        if !c.is_null()
            && (*c).flags & CLIENT_CONTROL as uint64_t != 0
            && !(*c).flags & CLIENT_EXIT as uint64_t != 0
            && !(*c).control_state.is_null()
        {
            control_notify_write(
                c,
                b"%%paste-buffer-deleted %s\0" as *const u8 as *const ::core::ffi::c_char,
                pbname,
            );
        }
        c = (*c).entry.tqe_next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn control_build_events() {
    static mut events: [C2RustUnnamed_35; 14] = unsafe {
        [
            C2RustUnnamed_35 {
                name: b"pane-mode-changed\0" as *const u8 as *const ::core::ffi::c_char,
                cb: Some(
                    control_pane_mode_changed_cb
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_char,
                            *mut event_payload,
                            *mut ::core::ffi::c_void,
                        ) -> (),
                ),
            },
            C2RustUnnamed_35 {
                name: b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
                cb: Some(
                    control_window_layout_changed_cb
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_char,
                            *mut event_payload,
                            *mut ::core::ffi::c_void,
                        ) -> (),
                ),
            },
            C2RustUnnamed_35 {
                name: b"window-pane-changed\0" as *const u8 as *const ::core::ffi::c_char,
                cb: Some(
                    control_window_pane_changed_cb
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_char,
                            *mut event_payload,
                            *mut ::core::ffi::c_void,
                        ) -> (),
                ),
            },
            C2RustUnnamed_35 {
                name: b"window-unlinked\0" as *const u8 as *const ::core::ffi::c_char,
                cb: Some(
                    control_window_unlinked_cb
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_char,
                            *mut event_payload,
                            *mut ::core::ffi::c_void,
                        ) -> (),
                ),
            },
            C2RustUnnamed_35 {
                name: b"window-linked\0" as *const u8 as *const ::core::ffi::c_char,
                cb: Some(
                    control_window_linked_cb
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_char,
                            *mut event_payload,
                            *mut ::core::ffi::c_void,
                        ) -> (),
                ),
            },
            C2RustUnnamed_35 {
                name: b"window-renamed\0" as *const u8 as *const ::core::ffi::c_char,
                cb: Some(
                    control_window_renamed_cb
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_char,
                            *mut event_payload,
                            *mut ::core::ffi::c_void,
                        ) -> (),
                ),
            },
            C2RustUnnamed_35 {
                name: b"client-session-changed\0" as *const u8 as *const ::core::ffi::c_char,
                cb: Some(
                    control_client_session_changed_cb
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_char,
                            *mut event_payload,
                            *mut ::core::ffi::c_void,
                        ) -> (),
                ),
            },
            C2RustUnnamed_35 {
                name: b"client-detached\0" as *const u8 as *const ::core::ffi::c_char,
                cb: Some(
                    control_client_detached_cb
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_char,
                            *mut event_payload,
                            *mut ::core::ffi::c_void,
                        ) -> (),
                ),
            },
            C2RustUnnamed_35 {
                name: b"session-renamed\0" as *const u8 as *const ::core::ffi::c_char,
                cb: Some(
                    control_session_renamed_cb
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_char,
                            *mut event_payload,
                            *mut ::core::ffi::c_void,
                        ) -> (),
                ),
            },
            C2RustUnnamed_35 {
                name: b"session-created\0" as *const u8 as *const ::core::ffi::c_char,
                cb: Some(
                    control_session_created_cb
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_char,
                            *mut event_payload,
                            *mut ::core::ffi::c_void,
                        ) -> (),
                ),
            },
            C2RustUnnamed_35 {
                name: b"session-closed\0" as *const u8 as *const ::core::ffi::c_char,
                cb: Some(
                    control_session_closed_cb
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_char,
                            *mut event_payload,
                            *mut ::core::ffi::c_void,
                        ) -> (),
                ),
            },
            C2RustUnnamed_35 {
                name: b"session-window-changed\0" as *const u8 as *const ::core::ffi::c_char,
                cb: Some(
                    control_session_window_changed_cb
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_char,
                            *mut event_payload,
                            *mut ::core::ffi::c_void,
                        ) -> (),
                ),
            },
            C2RustUnnamed_35 {
                name: b"paste-buffer-changed\0" as *const u8 as *const ::core::ffi::c_char,
                cb: Some(
                    control_paste_buffer_changed_cb
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_char,
                            *mut event_payload,
                            *mut ::core::ffi::c_void,
                        ) -> (),
                ),
            },
            C2RustUnnamed_35 {
                name: b"paste-buffer-deleted\0" as *const u8 as *const ::core::ffi::c_char,
                cb: Some(
                    control_paste_buffer_deleted_cb
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_char,
                            *mut event_payload,
                            *mut ::core::ffi::c_void,
                        ) -> (),
                ),
            },
        ]
    };
    let mut i: u_int = 0;
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[C2RustUnnamed_35; 14]>() as usize)
            .wrapping_div(::core::mem::size_of::<C2RustUnnamed_35>() as usize)
    {
        events_add_sink(events[i as usize].name, events[i as usize].cb, NULL);
        i = i.wrapping_add(1);
    }
}
