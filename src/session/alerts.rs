//! Session alert delivery preserves association order and per-cycle deduplication.
use super::*;
use crate::src::events::events_fire_winlink;
use crate::src::server::clients;
use crate::src::server_fn::server_status_session;
use crate::src::shared::alerts::{ALERT_ANY, ALERT_CURRENT, ALERT_OTHER, VISUAL_BOTH, VISUAL_OFF};
use crate::src::shared::client::{client, CLIENT_CONTROL};
use crate::src::shared::tty::TTYC_BEL;
use crate::src::shared::window::{
    WINDOW_ACTIVITY, WINDOW_BELL, WINDOW_SILENCE, WINLINK_ACTIVITY, WINLINK_BELL, WINLINK_SILENCE,
};
use crate::src::status::status_message_set;
use crate::src::tty::tty_putcode;

const SESSION_ALERTED: ::core::ffi::c_int = 0x1;

unsafe fn alerts_action_applies(
    mut wl: refbox::Weak<winlink>,
    mut name: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let Some(session_owner) = wl.get_unchecked().session.upgrade() else {
        return 0;
    };
    let s = session_owner.get();
    let mut action: ::core::ffi::c_int = 0;
    action = options_get_number(
        options_owner_ptr(&mut (*s).options).map_or(std::ptr::null_mut(), |options| options),
        name,
    ) as ::core::ffi::c_int;
    if action == ALERT_ANY {
        return 1 as ::core::ffi::c_int;
    }
    if action == ALERT_CURRENT {
        return (wl == (*s).current_winlink()) as ::core::ffi::c_int;
    }
    if action == ALERT_OTHER {
        return (wl != (*s).current_winlink()) as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}

pub(crate) unsafe fn alerts_check_all(w_owner: &Rc<UnsafeCell<window>>) -> ::core::ffi::c_int {
    let mut alerts: ::core::ffi::c_int = 0;
    alerts = alerts_check_bell(w_owner);
    alerts |= alerts_check_activity(w_owner);
    alerts |= alerts_check_silence(w_owner);
    return alerts;
}

unsafe fn alerts_check_bell(w_owner: &Rc<UnsafeCell<window>>) -> ::core::ffi::c_int {
    let w = w_owner.get();
    let mut wl: refbox::Weak<winlink> = refbox::Weak::new();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    if !(*w).flags & WINDOW_BELL != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if options_get_number(
        options_owner_ptr(&mut (*w).options).map_or(std::ptr::null_mut(), |options| options),
        b"monitor-bell\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0
    {
        return 0 as ::core::ffi::c_int;
    }
    wl = window_winlinks_first((w).as_ref());
    while wl.is_alive() {
        if let Some(session_owner) = wl.get_unchecked().session.upgrade() {
            (*session_owner.get()).flags &= !SESSION_ALERTED;
        }
        wl = window_winlinks_next((w).as_ref(), wl.clone());
    }
    wl = window_winlinks_first((w).as_ref());
    while wl.is_alive() {
        let Some(session_owner) = wl.get_unchecked().session.upgrade() else {
            wl = window_winlinks_next((w).as_ref(), wl.clone());
            continue;
        };
        s = session_owner.get();
        if (*s).current_winlink() != wl || (*s).attached == 0 as u_int {
            wl.get_mut_unchecked().flags |= WINLINK_BELL;
            server_status_session(&*(s));
        }
        if !(alerts_action_applies(
            wl.clone(),
            b"bell-action\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0)
        {
            events_fire_winlink(
                b"alert-bell\0" as *const u8 as *const ::core::ffi::c_char,
                wl.clone(),
            );
            if !((*s).flags & SESSION_ALERTED != 0) {
                (*s).flags |= SESSION_ALERTED;
                alerts_set_message(
                    wl.clone(),
                    b"Bell\0" as *const u8 as *const ::core::ffi::c_char,
                    b"visual-bell\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
        }
        wl = window_winlinks_next((w).as_ref(), wl.clone());
    }
    return 0x1 as ::core::ffi::c_int;
}

unsafe fn alerts_check_activity(w_owner: &Rc<UnsafeCell<window>>) -> ::core::ffi::c_int {
    let w = w_owner.get();
    let mut wl: refbox::Weak<winlink> = refbox::Weak::new();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    if !(*w).flags & WINDOW_ACTIVITY != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if options_get_number(
        options_owner_ptr(&mut (*w).options).map_or(std::ptr::null_mut(), |options| options),
        b"monitor-activity\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0
    {
        return 0 as ::core::ffi::c_int;
    }
    wl = window_winlinks_first((w).as_ref());
    while wl.is_alive() {
        if let Some(session_owner) = wl.get_unchecked().session.upgrade() {
            (*session_owner.get()).flags &= !SESSION_ALERTED;
        }
        wl = window_winlinks_next((w).as_ref(), wl.clone());
    }
    wl = window_winlinks_first((w).as_ref());
    while wl.is_alive() {
        if !(wl.get_unchecked().flags & WINLINK_ACTIVITY != 0) {
            let Some(session_owner) = wl.get_unchecked().session.upgrade() else {
                wl = window_winlinks_next((w).as_ref(), wl.clone());
                continue;
            };
            s = session_owner.get();
            if (*s).current_winlink() != wl || (*s).attached == 0 as u_int {
                wl.get_mut_unchecked().flags |= WINLINK_ACTIVITY;
                server_status_session(&*(s));
            }
            if !(alerts_action_applies(
                wl.clone(),
                b"activity-action\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0)
            {
                events_fire_winlink(
                    b"alert-activity\0" as *const u8 as *const ::core::ffi::c_char,
                    wl.clone(),
                );
                if !((*s).flags & SESSION_ALERTED != 0) {
                    (*s).flags |= SESSION_ALERTED;
                    alerts_set_message(
                        wl.clone(),
                        b"Activity\0" as *const u8 as *const ::core::ffi::c_char,
                        b"visual-activity\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                }
            }
        }
        wl = window_winlinks_next((w).as_ref(), wl.clone());
    }
    return 0x2 as ::core::ffi::c_int;
}

unsafe fn alerts_check_silence(w_owner: &Rc<UnsafeCell<window>>) -> ::core::ffi::c_int {
    let w = w_owner.get();
    let mut wl: refbox::Weak<winlink> = refbox::Weak::new();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    if !(*w).flags & WINDOW_SILENCE != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if options_get_number(
        options_owner_ptr(&mut (*w).options).map_or(std::ptr::null_mut(), |options| options),
        b"monitor-silence\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_longlong
    {
        return 0 as ::core::ffi::c_int;
    }
    wl = window_winlinks_first((w).as_ref());
    while wl.is_alive() {
        if let Some(session_owner) = wl.get_unchecked().session.upgrade() {
            (*session_owner.get()).flags &= !SESSION_ALERTED;
        }
        wl = window_winlinks_next((w).as_ref(), wl.clone());
    }
    wl = window_winlinks_first((w).as_ref());
    while wl.is_alive() {
        if !(wl.get_unchecked().flags & WINLINK_SILENCE != 0) {
            let Some(session_owner) = wl.get_unchecked().session.upgrade() else {
                wl = window_winlinks_next((w).as_ref(), wl.clone());
                continue;
            };
            s = session_owner.get();
            if (*s).current_winlink() != wl || (*s).attached == 0 as u_int {
                wl.get_mut_unchecked().flags |= WINLINK_SILENCE;
                server_status_session(&*(s));
            }
            if !(alerts_action_applies(
                wl.clone(),
                b"silence-action\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0)
            {
                events_fire_winlink(
                    b"alert-silence\0" as *const u8 as *const ::core::ffi::c_char,
                    wl.clone(),
                );
                if !((*s).flags & SESSION_ALERTED != 0) {
                    (*s).flags |= SESSION_ALERTED;
                    alerts_set_message(
                        wl.clone(),
                        b"Silence\0" as *const u8 as *const ::core::ffi::c_char,
                        b"visual-silence\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                }
            }
        }
        wl = window_winlinks_next((w).as_ref(), wl.clone());
    }
    return 0x4 as ::core::ffi::c_int;
}

unsafe fn alerts_set_message(
    mut wl: refbox::Weak<winlink>,
    mut type_0: *const ::core::ffi::c_char,
    mut option: *const ::core::ffi::c_char,
) {
    let Some(session_owner) = wl.get_unchecked().session.upgrade() else {
        return;
    };
    let s = session_owner.get();
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut visual: ::core::ffi::c_int = 0;
    visual = options_get_number(
        options_owner_ptr(&mut (*s).options).map_or(std::ptr::null_mut(), |options| options),
        option,
    ) as ::core::ffi::c_int;
    let mut registry_c_owner = clients.first();
    c = registry_c_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    while !c.is_null() {
        if !((*c)
            .session_handle()
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get())
            != s
            || (*c).flags & CLIENT_CONTROL as uint64_t != 0)
        {
            if visual == VISUAL_OFF || visual == VISUAL_BOTH {
                tty_putcode(&raw mut (*c).tty, TTYC_BEL);
            }
            if !(visual == VISUAL_OFF) {
                if (*(*c)
                    .session_handle()
                    .as_ref()
                    .map_or(std::ptr::null_mut(), |owner| owner.get()))
                .current_winlink()
                    == wl
                {
                    status_message_set(
                        (c).as_ref()
                            .and_then(|model| model.observer.upgrade())
                            .as_ref(),
                        -(1 as ::core::ffi::c_int),
                        1 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        |out| {
                            write_cstr(out, type_0)?;
                            out.write_all(b" in current window")
                        },
                    );
                } else {
                    status_message_set(
                        (c).as_ref()
                            .and_then(|model| model.observer.upgrade())
                            .as_ref(),
                        -(1 as ::core::ffi::c_int),
                        1 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        |out| {
                            write_cstr(out, type_0)?;
                            write!(out, " in window {}", (wl.get_unchecked().idx) as i32)
                        },
                    );
                }
            }
        }
        registry_c_owner =
            clients.next(registry_c_owner.as_ref().expect("current registry client"));
        c = registry_c_owner
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
    }
}
