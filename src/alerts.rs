use crate::src::options::options_owner_ptr;
use crate::src::events::events_fire_winlink;
use crate::src::format::bytes::write_cstr;
use crate::src::log::{log_debug, log_hex};
use crate::src::options::options_get_number;
use crate::src::reactor::{event_add, event_del, event_initialized, event_once, event_set};
use crate::src::server::clients;
use crate::src::server_fn::server_status_session;
use crate::src::shared::abi::*;
use crate::src::shared::alerts::{ALERT_ANY, ALERT_CURRENT, ALERT_OTHER, VISUAL_BOTH, VISUAL_OFF};
use crate::src::shared::client::client;
use crate::src::shared::client::CLIENT_CONTROL;
use crate::src::shared::event::EV_TIMEOUT;
use crate::src::shared::session::session;
use crate::src::shared::tree::RB_NEGINF;
use crate::src::shared::tty::*;
use crate::src::shared::window::{window, winlink};
use crate::src::shared::window::{
    WINDOW_ACTIVITY, WINDOW_ALERTFLAGS, WINDOW_BELL, WINDOW_SILENCE, WINLINK_ACTIVITY,
    WINLINK_BELL, WINLINK_SILENCE,
};
use crate::src::status::status_message_set;
use crate::src::tty::tty_putcode;
use crate::src::window::windows;
use crate::src::window::{
    window_add_ref, window_remove_ref, window_winlinks_first, window_winlinks_next, windows_minmax,
    windows_next, winlinks_minmax, winlinks_next,
};
use std::collections::VecDeque;

pub const SESSION_ALERTED: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
static mut alerts_fired: ::core::ffi::c_int = 0;
static mut alerts_list: VecDeque<*mut window> = VecDeque::new();

fn alerts_pop_front<T>(queue: &mut VecDeque<T>) -> Option<(T, bool)> {
    let item = queue.pop_front()?;
    // Leave appends made while processing an empty tail for the next callback.
    Some((item, !queue.is_empty()))
}

fn alerts_enqueue<T>(queue: &mut VecDeque<T>, queued: &mut ::core::ffi::c_int, item: T) -> bool {
    if *queued != 0 {
        return false;
    }
    *queued = 1;
    queue.push_back(item);
    true
}

unsafe fn alerts_timer(mut arg: *mut ::core::ffi::c_void) {
    let mut w: *mut window = arg as *mut window;
    log_debug(format_args!("@{} alerts timer expired", ((*w).id) as u32));
    alerts_queue(w, WINDOW_SILENCE);
}
unsafe fn alerts_callback() {
    let mut alerts: ::core::ffi::c_int = 0;
    loop {
        let next = {
            let queue = &mut *::core::ptr::addr_of_mut!(alerts_list);
            alerts_pop_front(queue)
        };
        let Some((w, has_next)) = next else {
            break;
        };
        // Keep the membership flag set during checks to suppress duplicate requeues.
        alerts = alerts_check_all(w);
        log_debug(format_args!(
            "@{} alerts check, alerts {}",
            ((*w).id) as u32,
            log_hex(((alerts) as u32) as u64)
        ));
        (*w).alerts_queued = 0 as ::core::ffi::c_int;
        (*w).flags &= !WINDOW_ALERTFLAGS;
        window_remove_ref(
            w,
            b"alerts_callback\0" as *const u8 as *const ::core::ffi::c_char,
        );
        if !has_next {
            break;
        }
    }
    alerts_fired = 0 as ::core::ffi::c_int;
}
unsafe fn alerts_action_applies(
    mut wl: *mut winlink,
    mut name: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut action: ::core::ffi::c_int = 0;
    action = options_get_number(options_owner_ptr(&mut (*(*wl).session).options), name) as ::core::ffi::c_int;
    if action == ALERT_ANY {
        return 1 as ::core::ffi::c_int;
    }
    if action == ALERT_CURRENT {
        return (wl == (*(*wl).session).curw) as ::core::ffi::c_int;
    }
    if action == ALERT_OTHER {
        return (wl != (*(*wl).session).curw) as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe fn alerts_check_all(mut w: *mut window) -> ::core::ffi::c_int {
    let mut alerts: ::core::ffi::c_int = 0;
    alerts = alerts_check_bell(w);
    alerts |= alerts_check_activity(w);
    alerts |= alerts_check_silence(w);
    return alerts;
}
pub unsafe fn alerts_check_session(mut s: *mut session) {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    wl = winlinks_minmax(&(*s).windows, RB_NEGINF);
    while !wl.is_null() {
        alerts_check_all((*wl).window_ptr());
        wl = winlinks_next(&*wl);
    }
}
unsafe fn alerts_enabled(mut w: *mut window, mut flags: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if flags & WINDOW_BELL != 0 {
        if options_get_number(
            options_owner_ptr(&mut (*w).options),
            b"monitor-bell\0" as *const u8 as *const ::core::ffi::c_char,
        ) != 0
        {
            return 1 as ::core::ffi::c_int;
        }
    }
    if flags & WINDOW_ACTIVITY != 0 {
        if options_get_number(
            options_owner_ptr(&mut (*w).options),
            b"monitor-activity\0" as *const u8 as *const ::core::ffi::c_char,
        ) != 0
        {
            return 1 as ::core::ffi::c_int;
        }
    }
    if flags & WINDOW_SILENCE != 0 {
        if options_get_number(
            options_owner_ptr(&mut (*w).options),
            b"monitor-silence\0" as *const u8 as *const ::core::ffi::c_char,
        ) != 0 as ::core::ffi::c_longlong
        {
            return 1 as ::core::ffi::c_int;
        }
    }
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn alerts_reset_all() {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    w = windows_minmax(&*std::ptr::addr_of!(windows));
    while !w.is_null() {
        alerts_reset(w);
        w = windows_next(&*w);
    }
}
unsafe fn alerts_reset(mut w: *mut window) {
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    if event_initialized(&(*w).alerts_timer) == 0 {
        event_set(
            &raw mut (*w).alerts_timer,
            -(1 as ::core::ffi::c_int),
            0 as ::core::ffi::c_short,
            move |_, _| unsafe { alerts_timer(w as *mut ::core::ffi::c_void) },
        );
    }
    (*w).flags &= !WINDOW_SILENCE;
    event_del(&raw mut (*w).alerts_timer);
    tv.tv_usec = 0 as __suseconds_t;
    tv.tv_sec = tv.tv_usec as __time_t;
    tv.tv_sec = options_get_number(
        options_owner_ptr(&mut (*w).options),
        b"monitor-silence\0" as *const u8 as *const ::core::ffi::c_char,
    ) as __time_t;
    log_debug(format_args!(
        "@{} alerts timer reset {}",
        ((*w).id) as u32,
        tv.tv_sec as u_int
    ));
    if tv.tv_sec != 0 as __time_t {
        event_add(&raw mut (*w).alerts_timer, &raw mut tv);
    }
}
pub unsafe fn alerts_queue(mut w: *mut window, mut flags: ::core::ffi::c_int) {
    alerts_reset(w);
    if (*w).flags & flags != flags {
        (*w).flags |= flags;
        log_debug(format_args!(
            "@{} alerts flags added {}",
            ((*w).id) as u32,
            log_hex(((flags) as u32) as u64)
        ));
    }
    if alerts_enabled(w, flags) != 0 {
        let enqueued = {
            let queue = &mut *::core::ptr::addr_of_mut!(alerts_list);
            alerts_enqueue(queue, &mut (*w).alerts_queued, w)
        };
        if enqueued {
            window_add_ref(
                w,
                b"alerts_queue\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        if alerts_fired == 0 {
            log_debug(format_args!(
                "alerts check queued (by @{})",
                ((*w).id) as u32
            ));
            event_once(move |_, _| unsafe { alerts_callback() });
            alerts_fired = 1 as ::core::ffi::c_int;
        }
    }
}
unsafe fn alerts_check_bell(mut w: *mut window) -> ::core::ffi::c_int {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    if !(*w).flags & WINDOW_BELL != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if options_get_number(
        options_owner_ptr(&mut (*w).options),
        b"monitor-bell\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0
    {
        return 0 as ::core::ffi::c_int;
    }
    wl = window_winlinks_first(w);
    while !wl.is_null() {
        (*(*wl).session).flags &= !SESSION_ALERTED;
        wl = window_winlinks_next(w, wl);
    }
    wl = window_winlinks_first(w);
    while !wl.is_null() {
        s = (*wl).session;
        if (*s).curw != wl || (*s).attached == 0 as u_int {
            (*wl).flags |= WINLINK_BELL;
            server_status_session(s);
        }
        if !(alerts_action_applies(
            wl,
            b"bell-action\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0)
        {
            events_fire_winlink(
                b"alert-bell\0" as *const u8 as *const ::core::ffi::c_char,
                wl,
            );
            if !((*s).flags & SESSION_ALERTED != 0) {
                (*s).flags |= SESSION_ALERTED;
                alerts_set_message(
                    wl,
                    b"Bell\0" as *const u8 as *const ::core::ffi::c_char,
                    b"visual-bell\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
        }
        wl = window_winlinks_next(w, wl);
    }
    return 0x1 as ::core::ffi::c_int;
}
unsafe fn alerts_check_activity(mut w: *mut window) -> ::core::ffi::c_int {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    if !(*w).flags & WINDOW_ACTIVITY != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if options_get_number(
        options_owner_ptr(&mut (*w).options),
        b"monitor-activity\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0
    {
        return 0 as ::core::ffi::c_int;
    }
    wl = window_winlinks_first(w);
    while !wl.is_null() {
        (*(*wl).session).flags &= !SESSION_ALERTED;
        wl = window_winlinks_next(w, wl);
    }
    wl = window_winlinks_first(w);
    while !wl.is_null() {
        if !((*wl).flags & WINLINK_ACTIVITY != 0) {
            s = (*wl).session;
            if (*s).curw != wl || (*s).attached == 0 as u_int {
                (*wl).flags |= WINLINK_ACTIVITY;
                server_status_session(s);
            }
            if !(alerts_action_applies(
                wl,
                b"activity-action\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0)
            {
                events_fire_winlink(
                    b"alert-activity\0" as *const u8 as *const ::core::ffi::c_char,
                    wl,
                );
                if !((*s).flags & SESSION_ALERTED != 0) {
                    (*s).flags |= SESSION_ALERTED;
                    alerts_set_message(
                        wl,
                        b"Activity\0" as *const u8 as *const ::core::ffi::c_char,
                        b"visual-activity\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                }
            }
        }
        wl = window_winlinks_next(w, wl);
    }
    return 0x2 as ::core::ffi::c_int;
}
unsafe fn alerts_check_silence(mut w: *mut window) -> ::core::ffi::c_int {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    if !(*w).flags & WINDOW_SILENCE != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if options_get_number(
        options_owner_ptr(&mut (*w).options),
        b"monitor-silence\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_longlong
    {
        return 0 as ::core::ffi::c_int;
    }
    wl = window_winlinks_first(w);
    while !wl.is_null() {
        (*(*wl).session).flags &= !SESSION_ALERTED;
        wl = window_winlinks_next(w, wl);
    }
    wl = window_winlinks_first(w);
    while !wl.is_null() {
        if !((*wl).flags & WINLINK_SILENCE != 0) {
            s = (*wl).session;
            if (*s).curw != wl || (*s).attached == 0 as u_int {
                (*wl).flags |= WINLINK_SILENCE;
                server_status_session(s);
            }
            if !(alerts_action_applies(
                wl,
                b"silence-action\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0)
            {
                events_fire_winlink(
                    b"alert-silence\0" as *const u8 as *const ::core::ffi::c_char,
                    wl,
                );
                if !((*s).flags & SESSION_ALERTED != 0) {
                    (*s).flags |= SESSION_ALERTED;
                    alerts_set_message(
                        wl,
                        b"Silence\0" as *const u8 as *const ::core::ffi::c_char,
                        b"visual-silence\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                }
            }
        }
        wl = window_winlinks_next(w, wl);
    }
    return 0x4 as ::core::ffi::c_int;
}
unsafe fn alerts_set_message(
    mut wl: *mut winlink,
    mut type_0: *const ::core::ffi::c_char,
    mut option: *const ::core::ffi::c_char,
) {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut visual: ::core::ffi::c_int = 0;
    visual = options_get_number(options_owner_ptr(&mut (*(*wl).session).options), option) as ::core::ffi::c_int;
    c = clients.first();
    while !c.is_null() {
        if !((*c).session != (*wl).session || (*c).flags & CLIENT_CONTROL as uint64_t != 0) {
            if visual == VISUAL_OFF || visual == VISUAL_BOTH {
                tty_putcode(&raw mut (*c).tty, TTYC_BEL);
            }
            if !(visual == VISUAL_OFF) {
                if (*(*c).session).curw == wl {
                    status_message_set(
                        c,
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
                        c,
                        -(1 as ::core::ffi::c_int),
                        1 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        |out| {
                            write_cstr(out, type_0)?;
                            write!(out, " in window {}", ((*wl).idx) as i32)
                        },
                    );
                }
            }
        }
        c = clients.next(c);
    }
}

#[cfg(test)]
mod alerts_list_tests {
    use super::{alerts_enqueue, alerts_pop_front};
    use std::collections::VecDeque;

    fn run_callback<T>(queue: &mut VecDeque<T>, mut process: impl FnMut(T, &mut VecDeque<T>)) {
        loop {
            let Some((item, has_next)) = alerts_pop_front(queue) else {
                break;
            };
            process(item, queue);
            if !has_next {
                break;
            }
        }
    }

    #[test]
    fn processes_items_appended_before_the_current_tail() {
        let mut queue = VecDeque::from([1, 2]);
        let mut seen = Vec::new();

        run_callback(&mut queue, |item, queue| {
            seen.push(item);
            if item == 1 {
                queue.push_back(3);
            }
        });

        assert_eq!(seen, [1, 2, 3]);
        assert!(queue.is_empty());
    }

    #[test]
    fn leaves_items_appended_while_processing_the_tail_for_next_callback() {
        let mut queue = VecDeque::from([1]);
        let mut seen = Vec::new();

        run_callback(&mut queue, |item, queue| {
            seen.push(item);
            if item == 1 {
                queue.push_back(2);
            }
        });

        assert_eq!(seen, [1]);
        assert_eq!(queue, VecDeque::from([2]));
    }

    #[test]
    fn queue_flag_prevents_duplicate_membership_until_cleared() {
        let mut queue = VecDeque::new();
        let mut queued = 0;

        assert!(alerts_enqueue(&mut queue, &mut queued, 1));
        assert!(!alerts_enqueue(&mut queue, &mut queued, 2));
        assert_eq!(queue, VecDeque::from([1]));
        assert_eq!(queued, 1);

        let (item, has_next) = alerts_pop_front(&mut queue).unwrap();
        assert_eq!(item, 1);
        assert!(!has_next);
        assert!(!alerts_enqueue(&mut queue, &mut queued, 2));
        assert!(queue.is_empty());

        queued = 0;
        assert!(alerts_enqueue(&mut queue, &mut queued, 3));
        assert_eq!(queue, VecDeque::from([3]));
    }
}
