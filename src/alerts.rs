use std::{cell::UnsafeCell, rc::Rc};
use crate::src::options::options_owner_ptr;
use crate::src::session::{alerts_check_all, Session};
use crate::src::log::{log_debug, log_hex};
use crate::src::options::options_get_number;
use crate::src::reactor::{event_add, event_del, event_initialized, event_once, event_set};
use crate::src::shared::abi::*;
use crate::src::shared::event::EV_TIMEOUT;
use crate::src::shared::session::session;
use crate::src::shared::tree::RB_NEGINF;
use crate::src::shared::window::window;
use crate::src::shared::window::{
    WINDOW_ACTIVITY, WINDOW_ALERTFLAGS, WINDOW_BELL, WINDOW_SILENCE,
};
use crate::src::window::windows;
use crate::src::window::{
    window_remove_ref, windows_minmax,
    windows_next, winlinks_minmax, winlinks_next,
};
use std::collections::VecDeque;

static mut alerts_fired: ::core::ffi::c_int = 0;
static mut alerts_list: VecDeque<std::rc::Rc<std::cell::UnsafeCell<window>>> = VecDeque::new();

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

unsafe fn alerts_timer(owner: &std::rc::Rc<std::cell::UnsafeCell<window>>) {
    let w = owner.get();
    log_debug(format_args!("@{} alerts timer expired", ((*w).id) as u32));
    alerts_queue(owner, WINDOW_SILENCE);
}
unsafe fn alerts_callback() {
    let mut alerts: ::core::ffi::c_int = 0;
    loop {
        let next = {
            let queue = &mut alerts_list;
            alerts_pop_front(queue)
        };
        let Some((owner, has_next)) = next else {
            break;
        };
        let w = crate::src::shared::rc::as_ptr(&owner);
        // Keep the membership flag set during checks to suppress duplicate requeues.
        alerts = alerts_check_all(&owner);
        log_debug(format_args!(
            "@{} alerts check, alerts {}",
            ((*w).id) as u32,
            log_hex(((alerts) as u32) as u64)
        ));
        (*w).alerts_queued = 0 as ::core::ffi::c_int;
        (*w).flags &= !WINDOW_ALERTFLAGS;
        window_remove_ref(owner, b"alerts_callback\0" as *const u8 as *const ::core::ffi::c_char);
        if !has_next {
            break;
        }
    }
    alerts_fired = 0 as ::core::ffi::c_int;
}

pub unsafe fn alerts_check_session(s_owner: &Rc<UnsafeCell<session>>) {
    let mut wl = s_owner.with_winlinks(|links| winlinks_minmax(links, RB_NEGINF));
    while wl.is_alive() {
        alerts_check_all(&(*(wl.get_unchecked().window_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get()))).observer.upgrade().expect("live window"));
        wl = winlinks_next(wl.get_unchecked());
    }
}
unsafe fn alerts_enabled(w_owner: &Rc<UnsafeCell<window>>, mut flags: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let w = w_owner.get();
    if flags & WINDOW_BELL != 0 {
        if options_get_number(
            options_owner_ptr(&mut (*w).options).map_or(std::ptr::null_mut(), |options| options),
            b"monitor-bell\0" as *const u8 as *const ::core::ffi::c_char,
        ) != 0
        {
            return 1 as ::core::ffi::c_int;
        }
    }
    if flags & WINDOW_ACTIVITY != 0 {
        if options_get_number(
            options_owner_ptr(&mut (*w).options).map_or(std::ptr::null_mut(), |options| options),
            b"monitor-activity\0" as *const u8 as *const ::core::ffi::c_char,
        ) != 0
        {
            return 1 as ::core::ffi::c_int;
        }
    }
    if flags & WINDOW_SILENCE != 0 {
        if options_get_number(
            options_owner_ptr(&mut (*w).options).map_or(std::ptr::null_mut(), |options| options),
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
    let mut window_cursor = windows_minmax(&windows);
    while let Some(window_owner) = window_cursor.take() {
        w = window_owner.get();
        alerts_reset(&window_owner);
        window_cursor = windows_next(&*w);
        crate::src::window::window_remove_ref(window_owner, c"window traversal".as_ptr());
    }
}
unsafe fn alerts_reset(w_owner: &Rc<UnsafeCell<window>>) {
    let w = w_owner.get();
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    if event_initialized(&(*w).alerts_timer) == 0 {
        let observer = (*w).observer.clone();
        event_set(
            &raw mut (*w).alerts_timer,
            -(1 as ::core::ffi::c_int),
            0 as ::core::ffi::c_short,
            move |_, _| unsafe {
                if let Some(owner) = observer.upgrade() {
                    alerts_timer(&owner);
                }
            },
        );
    }
    (*w).flags &= !WINDOW_SILENCE;
    event_del(&raw mut (*w).alerts_timer);
    tv.tv_usec = 0 as __suseconds_t;
    tv.tv_sec = tv.tv_usec as __time_t;
    tv.tv_sec = options_get_number(
        options_owner_ptr(&mut (*w).options).map_or(std::ptr::null_mut(), |options| options),
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
pub unsafe fn alerts_queue(w_owner: &Rc<UnsafeCell<window>>, mut flags: ::core::ffi::c_int) {
    let w = w_owner.get();
    alerts_reset(w_owner);
    if (*w).flags & flags != flags {
        (*w).flags |= flags;
        log_debug(format_args!(
            "@{} alerts flags added {}",
            ((*w).id) as u32,
            log_hex(((flags) as u32) as u64)
        ));
    }
    if alerts_enabled(w_owner, flags) != 0 {
        if (*w).alerts_queued == 0 {
            let owner = Rc::clone(w_owner);
            let queue = &mut alerts_list;
            alerts_enqueue(queue, &mut (*w).alerts_queued, owner);
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
