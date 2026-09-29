use crate::src::control::control_notify_write;
use crate::src::events::events_add_sink;
use crate::src::events_payload::{
    event_payload_get_client, event_payload_get_pane, event_payload_get_session,
    event_payload_get_string, event_payload_get_window, event_payload_print_owned,
};
use crate::src::format::bytes::write_cstr;
use crate::src::format::{format_create, format_defaults, format_expand_cstring, format_free};
use crate::src::server::clients;
use crate::src::shared::events::{event_payload, events_callback};
use crate::src::window::{window_winlinks_first, winlink_find_by_window_id};
use std::ffi::CStr;

use crate::src::shared::abi::*;
use crate::src::shared::client::client;
use crate::src::shared::client::{CLIENT_CONTROL, CLIENT_EXIT};
use crate::src::shared::command::cmdq_item;
use crate::src::shared::format::format_tree;
use crate::src::shared::format::FORMAT_NONE;
use crate::src::shared::pane::window_pane;
use crate::src::shared::session::session;
use crate::src::shared::window::{window, winlink};

#[derive(Copy, Clone)]
pub struct C2RustUnnamed_35 {
    pub name: &'static CStr,
    pub cb: unsafe fn(&CStr, &mut event_payload),
}

unsafe fn control_pane_mode_changed_cb(_name: &CStr, payload: &mut event_payload) {
    let ep = &*payload;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    wp = event_payload_get_pane(ep).map_or(std::ptr::null_mut(), |owner| owner.get());
    if !wp.is_null() {
        let mut registry_c_owner = clients.first();
        c = registry_c_owner
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
        while !c.is_null() {
            if !c.is_null()
                && (*c).flags & CLIENT_CONTROL as uint64_t != 0
                && !(*c).flags & CLIENT_EXIT as uint64_t != 0
                && (*c).control_state.is_some()
            {
                control_notify_write(&(*(c)).observer.upgrade().expect("live client"), |out| {
                    write!(out, "%pane-mode-changed %{}", ((*wp).id) as u32)
                });
            }
            registry_c_owner =
                clients.next(registry_c_owner.as_ref().expect("current registry client"));
            c = registry_c_owner
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get());
        }
        return;
    }
    let Some(value) = event_payload_print_owned(ep) else {
        return;
    };
    let mut registry_c_owner = clients.first();
    c = registry_c_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    while !c.is_null() {
        if !c.is_null()
            && (*c).flags & CLIENT_CONTROL as uint64_t != 0
            && !(*c).flags & CLIENT_EXIT as uint64_t != 0
            && (*c).control_state.is_some()
        {
            control_notify_write(&(*(c)).observer.upgrade().expect("live client"), |out| {
                out.write_all(b"%pane-mode-changed ")?;
                write_cstr(out, value.as_ptr().cast::<::core::ffi::c_char>())
            });
        }
        registry_c_owner =
            clients.next(registry_c_owner.as_ref().expect("current registry client"));
        c = registry_c_owner
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
    }
}
unsafe fn control_window_layout_changed_cb(_name: &CStr, payload: &mut event_payload) {
    let ep = &*payload;
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut wl: refbox::Weak<winlink> = refbox::Weak::new();
    let mut w: *mut window =
        event_payload_get_window(ep).map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut template: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if w.is_null() {
        return;
    }
    template = b"%layout-change #{window_id} #{window_layout} #{window_visible_layout} #{window_raw_flags}\0"
        as *const u8 as *const ::core::ffi::c_char;
    if !window_winlinks_first((w).as_ref()).is_alive()
        || (*w)
            .layout_root_ptr()
            .map_or(std::ptr::null_mut(), |root| root)
            .is_null()
    {
        return;
    }
    let mut registry_c_owner = clients.first();
    c = registry_c_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    while !c.is_null() {
        if !(!(!c.is_null()
            && (*c).flags & CLIENT_CONTROL as uint64_t != 0
            && !(*c).flags & CLIENT_EXIT as uint64_t != 0
            && (*c).control_state.is_some())
            || (*c).session_handle().is_none())
        {
            s = (*c)
                .session_handle()
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get());
            wl = winlink_find_by_window_id(&(*s).windows, (*w).id);
            if wl.is_alive() {
                let mut ft_owner = format_create(
                    (c).as_ref()
                        .and_then(|model| model.observer.upgrade())
                        .as_ref(),
                    None,
                    FORMAT_NONE,
                    0 as ::core::ffi::c_int,
                );
                ft = &raw mut *ft_owner;
                format_defaults(
                    ft,
                    (c).as_ref()
                        .and_then(|model| model.observer.upgrade())
                        .as_ref(),
                    (s).as_ref()
                        .and_then(|model| model.observer.upgrade())
                        .as_ref(),
                    wl.clone(),
                    None,
                );
                let cp = format_expand_cstring(ft, template);
                format_free(ft_owner);
                control_notify_write(&(*(c)).observer.upgrade().expect("live client"), |out| {
                    write_cstr(out, cp.as_ptr())
                });
            }
        }
        registry_c_owner =
            clients.next(registry_c_owner.as_ref().expect("current registry client"));
        c = registry_c_owner
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
    }
}
unsafe fn control_window_pane_changed_cb(_name: &CStr, payload: &mut event_payload) {
    let ep = &*payload;
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut w: *mut window =
        event_payload_get_window(ep).map_or(std::ptr::null_mut(), |owner| owner.get());
    if w.is_null() || (*w).active_pane().is_none() {
        return;
    }
    let mut registry_c_owner = clients.first();
    c = registry_c_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    while !c.is_null() {
        if !c.is_null()
            && (*c).flags & CLIENT_CONTROL as uint64_t != 0
            && !(*c).flags & CLIENT_EXIT as uint64_t != 0
            && (*c).control_state.is_some()
        {
            control_notify_write(&(*(c)).observer.upgrade().expect("live client"), |out| {
                write!(
                    out,
                    "%window-pane-changed @{} %{}",
                    ((*w).id) as u32,
                    ((*(*w)
                        .active_pane()
                        .as_ref()
                        .map_or(std::ptr::null_mut(), |owner| owner.get()))
                    .id) as u32
                )
            });
        }
        registry_c_owner =
            clients.next(registry_c_owner.as_ref().expect("current registry client"));
        c = registry_c_owner
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
    }
}
unsafe fn control_window_unlinked_cb(_name: &CStr, payload: &mut event_payload) {
    let ep = &*payload;
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut cs: *mut session = ::core::ptr::null_mut::<session>();
    let mut w: *mut window =
        event_payload_get_window(ep).map_or(std::ptr::null_mut(), |owner| owner.get());
    if w.is_null() {
        return;
    }
    let mut registry_c_owner = clients.first();
    c = registry_c_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    while !c.is_null() {
        if !(!(!c.is_null()
            && (*c).flags & CLIENT_CONTROL as uint64_t != 0
            && !(*c).flags & CLIENT_EXIT as uint64_t != 0
            && (*c).control_state.is_some())
            || (*c).session_handle().is_none())
        {
            cs = (*c)
                .session_handle()
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get());
            if winlink_find_by_window_id(&(*cs).windows, (*w).id).is_alive() {
                control_notify_write(&(*(c)).observer.upgrade().expect("live client"), |out| {
                    write!(out, "%window-close @{}", ((*w).id) as u32)
                });
            } else {
                control_notify_write(&(*(c)).observer.upgrade().expect("live client"), |out| {
                    write!(out, "%unlinked-window-close @{}", ((*w).id) as u32)
                });
            }
        }
        registry_c_owner =
            clients.next(registry_c_owner.as_ref().expect("current registry client"));
        c = registry_c_owner
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
    }
}
unsafe fn control_window_linked_cb(_name: &CStr, payload: &mut event_payload) {
    let ep = &*payload;
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut cs: *mut session = ::core::ptr::null_mut::<session>();
    let mut w: *mut window =
        event_payload_get_window(ep).map_or(std::ptr::null_mut(), |owner| owner.get());
    if w.is_null() {
        return;
    }
    let mut registry_c_owner = clients.first();
    c = registry_c_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    while !c.is_null() {
        if !(!(!c.is_null()
            && (*c).flags & CLIENT_CONTROL as uint64_t != 0
            && !(*c).flags & CLIENT_EXIT as uint64_t != 0
            && (*c).control_state.is_some())
            || (*c).session_handle().is_none())
        {
            cs = (*c)
                .session_handle()
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get());
            if winlink_find_by_window_id(&(*cs).windows, (*w).id).is_alive() {
                control_notify_write(&(*(c)).observer.upgrade().expect("live client"), |out| {
                    write!(out, "%window-add @{}", ((*w).id) as u32)
                });
            } else {
                control_notify_write(&(*(c)).observer.upgrade().expect("live client"), |out| {
                    write!(out, "%unlinked-window-add @{}", ((*w).id) as u32)
                });
            }
        }
        registry_c_owner =
            clients.next(registry_c_owner.as_ref().expect("current registry client"));
        c = registry_c_owner
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
    }
}
unsafe fn control_window_renamed_cb(_name: &CStr, payload: &mut event_payload) {
    let ep = &*payload;
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut cs: *mut session = ::core::ptr::null_mut::<session>();
    let mut w: *mut window =
        event_payload_get_window(ep).map_or(std::ptr::null_mut(), |owner| owner.get());
    if w.is_null() {
        return;
    }
    let mut registry_c_owner = clients.first();
    c = registry_c_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    while !c.is_null() {
        if !(!(!c.is_null()
            && (*c).flags & CLIENT_CONTROL as uint64_t != 0
            && !(*c).flags & CLIENT_EXIT as uint64_t != 0
            && (*c).control_state.is_some())
            || (*c).session_handle().is_none())
        {
            cs = (*c)
                .session_handle()
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get());
            if winlink_find_by_window_id(&(*cs).windows, (*w).id).is_alive() {
                control_notify_write(&(*(c)).observer.upgrade().expect("live client"), |out| {
                    write!(out, "%window-renamed @{} ", ((*w).id) as u32)?;
                    write_cstr(out, (*w).name.as_ptr())
                });
            } else {
                control_notify_write(&(*(c)).observer.upgrade().expect("live client"), |out| {
                    write!(out, "%unlinked-window-renamed @{} ", ((*w).id) as u32)?;
                    write_cstr(out, (*w).name.as_ptr())
                });
            }
        }
        registry_c_owner =
            clients.next(registry_c_owner.as_ref().expect("current registry client"));
        c = registry_c_owner
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
    }
}
unsafe fn control_client_session_changed_cb(_name: &CStr, payload: &mut event_payload) {
    let ep = &*payload;
    let mut cc: *mut client =
        event_payload_get_client(ep).map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    if cc.is_null() || (*cc).session_handle().is_none() {
        return;
    }
    s = (*cc)
        .session_handle()
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut registry_c_owner = clients.first();
    c = registry_c_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    while !c.is_null() {
        if !(!(!c.is_null()
            && (*c).flags & CLIENT_CONTROL as uint64_t != 0
            && !(*c).flags & CLIENT_EXIT as uint64_t != 0
            && (*c).control_state.is_some())
            || (*c).session_handle().is_none())
        {
            if cc == c {
                control_notify_write(&(*(c)).observer.upgrade().expect("live client"), |out| {
                    write!(out, "%session-changed ${} ", ((*s).id) as u32)?;
                    write_cstr(out, ((*s).name).as_ptr().cast_mut())
                });
            } else {
                control_notify_write(&(*(c)).observer.upgrade().expect("live client"), |out| {
                    out.write_all(b"%client-session-changed ")?;
                    write_cstr(
                        out,
                        ((*cc).name)
                            .as_ref()
                            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                    )?;
                    write!(out, " ${} ", ((*s).id) as u32)?;
                    write_cstr(out, ((*s).name).as_ptr().cast_mut())
                });
            }
        }
        registry_c_owner =
            clients.next(registry_c_owner.as_ref().expect("current registry client"));
        c = registry_c_owner
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
    }
}
unsafe fn control_client_detached_cb(_name: &CStr, payload: &mut event_payload) {
    let ep = &*payload;
    let mut cc: *mut client =
        event_payload_get_client(ep).map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    if cc.is_null() {
        return;
    }
    let mut registry_c_owner = clients.first();
    c = registry_c_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    while !c.is_null() {
        if !c.is_null()
            && (*c).flags & CLIENT_CONTROL as uint64_t != 0
            && !(*c).flags & CLIENT_EXIT as uint64_t != 0
            && (*c).control_state.is_some()
        {
            control_notify_write(&(*(c)).observer.upgrade().expect("live client"), |out| {
                out.write_all(b"%client-detached ")?;
                write_cstr(
                    out,
                    ((*cc).name)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                )
            });
        }
        registry_c_owner =
            clients.next(registry_c_owner.as_ref().expect("current registry client"));
        c = registry_c_owner
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
    }
}
unsafe fn control_session_renamed_cb(_name: &CStr, payload: &mut event_payload) {
    let ep = &*payload;
    let mut s: *mut session =
        event_payload_get_session(ep).map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    if s.is_null() {
        return;
    }
    let mut registry_c_owner = clients.first();
    c = registry_c_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    while !c.is_null() {
        if !c.is_null()
            && (*c).flags & CLIENT_CONTROL as uint64_t != 0
            && !(*c).flags & CLIENT_EXIT as uint64_t != 0
            && (*c).control_state.is_some()
        {
            control_notify_write(&(*(c)).observer.upgrade().expect("live client"), |out| {
                write!(out, "%session-renamed ${} ", ((*s).id) as u32)?;
                write_cstr(out, ((*s).name).as_ptr().cast_mut())
            });
        }
        registry_c_owner =
            clients.next(registry_c_owner.as_ref().expect("current registry client"));
        c = registry_c_owner
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
    }
}
unsafe fn control_session_created_cb(_name: &CStr, _payload: &mut event_payload) {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut registry_c_owner = clients.first();
    c = registry_c_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    while !c.is_null() {
        if !c.is_null()
            && (*c).flags & CLIENT_CONTROL as uint64_t != 0
            && !(*c).flags & CLIENT_EXIT as uint64_t != 0
            && (*c).control_state.is_some()
        {
            control_notify_write(&(*(c)).observer.upgrade().expect("live client"), |out| {
                out.write_all(b"%sessions-changed")
            });
        }
        registry_c_owner =
            clients.next(registry_c_owner.as_ref().expect("current registry client"));
        c = registry_c_owner
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
    }
}
unsafe fn control_session_closed_cb(_name: &CStr, _payload: &mut event_payload) {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut registry_c_owner = clients.first();
    c = registry_c_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    while !c.is_null() {
        if !c.is_null()
            && (*c).flags & CLIENT_CONTROL as uint64_t != 0
            && !(*c).flags & CLIENT_EXIT as uint64_t != 0
            && (*c).control_state.is_some()
        {
            control_notify_write(&(*(c)).observer.upgrade().expect("live client"), |out| {
                out.write_all(b"%sessions-changed")
            });
        }
        registry_c_owner =
            clients.next(registry_c_owner.as_ref().expect("current registry client"));
        c = registry_c_owner
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
    }
}
unsafe fn control_session_window_changed_cb(_name: &CStr, payload: &mut event_payload) {
    let ep = &*payload;
    let mut s: *mut session =
        event_payload_get_session(ep).map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    if s.is_null() || !(*s).current_winlink().is_alive() {
        return;
    }
    let mut registry_c_owner = clients.first();
    c = registry_c_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    while !c.is_null() {
        if !c.is_null()
            && (*c).flags & CLIENT_CONTROL as uint64_t != 0
            && !(*c).flags & CLIENT_EXIT as uint64_t != 0
            && (*c).control_state.is_some()
        {
            control_notify_write(&(*(c)).observer.upgrade().expect("live client"), |out| {
                write!(
                    out,
                    "%session-window-changed ${} @{}",
                    ((*s).id) as u32,
                    ((*((*s).current_winlink())
                        .get_unchecked()
                        .window_handle()
                        .as_ref()
                        .map_or(std::ptr::null_mut(), |owner| owner.get()))
                    .id) as u32
                )
            });
        }
        registry_c_owner =
            clients.next(registry_c_owner.as_ref().expect("current registry client"));
        c = registry_c_owner
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
    }
}
unsafe fn control_paste_buffer_changed_cb(_name: &CStr, payload: &mut event_payload) {
    let ep = &*payload;
    let Some(pbname) = event_payload_get_string(ep) else {
        return;
    };
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut registry_c_owner = clients.first();
    c = registry_c_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    while !c.is_null() {
        if !c.is_null()
            && (*c).flags & CLIENT_CONTROL as uint64_t != 0
            && !(*c).flags & CLIENT_EXIT as uint64_t != 0
            && (*c).control_state.is_some()
        {
            control_notify_write(&(*(c)).observer.upgrade().expect("live client"), |out| {
                out.write_all(b"%paste-buffer-changed ")?;
                write_cstr(out, pbname.as_ptr())
            });
        }
        registry_c_owner =
            clients.next(registry_c_owner.as_ref().expect("current registry client"));
        c = registry_c_owner
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
    }
}
unsafe fn control_paste_buffer_deleted_cb(_name: &CStr, payload: &mut event_payload) {
    let ep = &*payload;
    let Some(pbname) = event_payload_get_string(ep) else {
        return;
    };
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut registry_c_owner = clients.first();
    c = registry_c_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    while !c.is_null() {
        if !c.is_null()
            && (*c).flags & CLIENT_CONTROL as uint64_t != 0
            && !(*c).flags & CLIENT_EXIT as uint64_t != 0
            && (*c).control_state.is_some()
        {
            control_notify_write(&(*(c)).observer.upgrade().expect("live client"), |out| {
                out.write_all(b"%paste-buffer-deleted ")?;
                write_cstr(out, pbname.as_ptr())
            });
        }
        registry_c_owner =
            clients.next(registry_c_owner.as_ref().expect("current registry client"));
        c = registry_c_owner
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
    }
}
pub unsafe fn control_build_events() {
    static events: [C2RustUnnamed_35; 14] = {
        [
            C2RustUnnamed_35 {
                name: c"pane-mode-changed",
                cb: control_pane_mode_changed_cb,
            },
            C2RustUnnamed_35 {
                name: c"window-layout-changed",
                cb: control_window_layout_changed_cb,
            },
            C2RustUnnamed_35 {
                name: c"window-pane-changed",
                cb: control_window_pane_changed_cb,
            },
            C2RustUnnamed_35 {
                name: c"window-unlinked",
                cb: control_window_unlinked_cb,
            },
            C2RustUnnamed_35 {
                name: c"window-linked",
                cb: control_window_linked_cb,
            },
            C2RustUnnamed_35 {
                name: c"window-renamed",
                cb: control_window_renamed_cb,
            },
            C2RustUnnamed_35 {
                name: c"client-session-changed",
                cb: control_client_session_changed_cb,
            },
            C2RustUnnamed_35 {
                name: c"client-detached",
                cb: control_client_detached_cb,
            },
            C2RustUnnamed_35 {
                name: c"session-renamed",
                cb: control_session_renamed_cb,
            },
            C2RustUnnamed_35 {
                name: c"session-created",
                cb: control_session_created_cb,
            },
            C2RustUnnamed_35 {
                name: c"session-closed",
                cb: control_session_closed_cb,
            },
            C2RustUnnamed_35 {
                name: c"session-window-changed",
                cb: control_session_window_changed_cb,
            },
            C2RustUnnamed_35 {
                name: c"paste-buffer-changed",
                cb: control_paste_buffer_changed_cb,
            },
            C2RustUnnamed_35 {
                name: c"paste-buffer-deleted",
                cb: control_paste_buffer_deleted_cb,
            },
        ]
    };
    let mut i: u_int = 0;
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[C2RustUnnamed_35; 14]>() as usize)
            .wrapping_div(::core::mem::size_of::<C2RustUnnamed_35>() as usize)
    {
        let callback = events[i as usize].cb;
        events_add_sink(
            events[i as usize].name,
            events_callback(move |name, payload| unsafe { callback(name, payload) }),
        );
        i = i.wrapping_add(1);
    }
}
