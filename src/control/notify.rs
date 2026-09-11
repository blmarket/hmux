use super::state::control_write;
use crate::fmt_args;
use crate::format::format_single;
use crate::server::{client_ref_of, client_walk};

pub use crate::consts::CLIENT_CONTROL;
pub use crate::types::*;
use crate::window::winlink_find_by_window_id;
pub fn control_notify_pane_mode_changed(pane: core::ffi::c_int) {
    unsafe {
        for mut owner in client_walk() {
            let c = owner.as_client_mut();
            if c.flags & CLIENT_CONTROL as uint64_t != 0 && c.control_state.is_some() {
                control_write(c, c"%%pane-mode-changed %%%u", fmt_args![pane]);
            }
        }
    }
}
pub unsafe fn control_notify_window_layout_changed(w: &WindowRef) {
    unsafe {
        let Some(held) = w.winlinks().next() else {
            return;
        };
        let Some(wl) = held.get() else {
            return;
        };
        if w.as_window().layout().size().is_none() {
            return;
        }
        let template = c"%layout-change #{window_id} #{window_layout} #{window_visible_layout} #{window_raw_flags}";
        let cp = format_single(
            None,
            template,
            None,
            None,
            Some(wl),
            None::<&dyn crate::WindowPane>,
        );
        for mut owner in client_walk() {
            let Some(session) = owner.attached_session() else {
                continue;
            };
            let c = owner.as_client_mut();
            if c.flags & CLIENT_CONTROL as uint64_t != 0
                && c.control_state.is_some()
                && winlink_find_by_window_id(&session.as_session().windows, w.window_id()).is_some()
            {
                control_write(c, c"%s", fmt_args![cp.as_c_str()]);
            }
        }
    }
}
pub unsafe fn control_notify_window_pane_changed(w: &WindowRef) {
    unsafe {
        let Some(pane_id) = w.active_pane().and_then(|pane| pane.pane_id()) else {
            return;
        };
        for mut owner in client_walk() {
            let c = owner.as_client_mut();
            if c.flags & CLIENT_CONTROL as uint64_t != 0 && c.control_state.is_some() {
                control_write(
                    c,
                    c"%%window-pane-changed @%u %%%u",
                    fmt_args![w.window_id(), pane_id],
                );
            }
        }
    }
}
pub unsafe fn control_notify_window_unlinked(_s: Option<&session>, w: &WindowRef) {
    unsafe {
        for mut owner in client_walk() {
            let Some(session) = owner.attached_session() else {
                continue;
            };
            let c = owner.as_client_mut();
            if c.flags & CLIENT_CONTROL as uint64_t != 0 && c.control_state.is_some() {
                if winlink_find_by_window_id(&session.as_session().windows, w.window_id()).is_some()
                {
                    control_write(c, c"%%window-close @%u", fmt_args![w.window_id()]);
                } else {
                    control_write(c, c"%%unlinked-window-close @%u", fmt_args![w.window_id()]);
                }
            }
        }
    }
}
pub unsafe fn control_notify_window_linked(_s: Option<&session>, w: &WindowRef) {
    unsafe {
        for mut owner in client_walk() {
            let Some(session) = owner.attached_session() else {
                continue;
            };
            let c = owner.as_client_mut();
            if c.flags & CLIENT_CONTROL as uint64_t != 0 && c.control_state.is_some() {
                if winlink_find_by_window_id(&session.as_session().windows, w.window_id()).is_some()
                {
                    control_write(c, c"%%window-add @%u", fmt_args![w.window_id()]);
                } else {
                    control_write(c, c"%%unlinked-window-add @%u", fmt_args![w.window_id()]);
                }
            }
        }
    }
}
pub unsafe fn control_notify_window_renamed(w: &WindowRef) {
    unsafe {
        for mut owner in client_walk() {
            let Some(session) = owner.attached_session() else {
                continue;
            };
            let c = owner.as_client_mut();
            if c.flags & CLIENT_CONTROL as uint64_t != 0 && c.control_state.is_some() {
                if winlink_find_by_window_id(&session.as_session().windows, w.window_id()).is_some()
                {
                    control_write(
                        c,
                        c"%%window-renamed @%u %s",
                        fmt_args![w.window_id(), w.window_name().as_deref()],
                    );
                } else {
                    control_write(
                        c,
                        c"%%unlinked-window-renamed @%u %s",
                        fmt_args![w.window_id(), w.window_name().as_deref()],
                    );
                }
            }
        }
    }
}
pub unsafe fn control_notify_client_session_changed(cc: &mut client) {
    unsafe {
        let Some(session) = cc.attached_session() else {
            return;
        };
        let source = client_ref_of(cc);
        let name = cc.name.clone();
        for mut owner in client_walk() {
            let is_source = source.as_ref().is_some_and(|source| owner.ptr_eq(source));
            let c = if is_source {
                &mut *cc
            } else {
                owner.as_client_mut()
            };
            if c.flags & CLIENT_CONTROL as uint64_t != 0
                && c.control_state.is_some()
                && c.attached_session().is_some()
            {
                if is_source {
                    control_write(
                        c,
                        c"%%session-changed $%u %s",
                        fmt_args![session.id(), session.name().as_deref()],
                    );
                } else {
                    control_write(
                        c,
                        c"%%client-session-changed %s $%u %s",
                        fmt_args![name.as_deref(), session.id(), session.name().as_deref()],
                    );
                }
            }
        }
    }
}
pub unsafe fn control_notify_client_detached(cc: &mut client) {
    unsafe {
        let source = client_ref_of(cc);
        let name = cc.name.clone();
        for mut owner in client_walk() {
            let is_source = source.as_ref().is_some_and(|source| owner.ptr_eq(source));
            let c = if is_source {
                &mut *cc
            } else {
                owner.as_client_mut()
            };
            if c.flags & CLIENT_CONTROL as uint64_t != 0 && c.control_state.is_some() {
                control_write(c, c"%%client-detached %s", fmt_args![name.as_deref()]);
            }
        }
    }
}
pub unsafe fn control_notify_session_renamed(s: &session) {
    unsafe {
        for mut owner in client_walk() {
            let c = owner.as_client_mut();
            if c.flags & CLIENT_CONTROL as uint64_t != 0 && c.control_state.is_some() {
                control_write(
                    c,
                    c"%%session-renamed $%u %s",
                    fmt_args![
                        crate::SessionIdentity::session_id(s),
                        crate::SessionNameState::session_name(s)
                    ],
                );
            }
        }
    }
}
pub unsafe fn control_notify_session_created(_s: &session) {
    unsafe {
        for mut owner in client_walk() {
            let c = owner.as_client_mut();
            if c.flags & CLIENT_CONTROL as uint64_t != 0 && c.control_state.is_some() {
                control_write(c, c"%%sessions-changed", fmt_args![]);
            }
        }
    }
}
pub unsafe fn control_notify_session_closed(_s: &session) {
    unsafe {
        for mut owner in client_walk() {
            let c = owner.as_client_mut();
            if c.flags & CLIENT_CONTROL as uint64_t != 0 && c.control_state.is_some() {
                control_write(c, c"%%sessions-changed", fmt_args![]);
            }
        }
    }
}
pub unsafe fn control_notify_session_window_changed(s: &session) {
    unsafe {
        let window = (s).current_window();
        let window_id = window.as_ref().map_or(0, |window| window.window_id());
        for mut owner in client_walk() {
            let c = owner.as_client_mut();
            if c.flags & CLIENT_CONTROL as uint64_t != 0 && c.control_state.is_some() {
                control_write(
                    c,
                    c"%%session-window-changed $%u @%u",
                    fmt_args![crate::SessionIdentity::session_id(s), window_id],
                );
            }
        }
    }
}
pub unsafe fn control_notify_paste_buffer_changed(name: Option<&core::ffi::CStr>) {
    unsafe {
        for mut owner in client_walk() {
            let c = owner.as_client_mut();
            if c.flags & CLIENT_CONTROL as uint64_t != 0 && c.control_state.is_some() {
                control_write(c, c"%%paste-buffer-changed %s", fmt_args![name]);
            }
        }
    }
}
pub unsafe fn control_notify_paste_buffer_deleted(name: Option<&core::ffi::CStr>) {
    unsafe {
        for mut owner in client_walk() {
            let c = owner.as_client_mut();
            if c.flags & CLIENT_CONTROL as uint64_t != 0 && c.control_state.is_some() {
                control_write(c, c"%%paste-buffer-deleted %s", fmt_args![name]);
            }
        }
    }
}
