//! Window-owned alert flags, queue membership and silence timer.
use super::*;
use crate::src::reactor::{event_add, event_del, event_initialized, event_set};
use crate::src::shared::window::WindowRef;
use crate::src::shared::window::{WINDOW_ACTIVITY, WINDOW_ALERTFLAGS, WINDOW_BELL, WINDOW_SILENCE};

pub(super) unsafe fn reset_timer(owner: &WindowRef) {
    let w = owner.get();
    if event_initialized(&(*w).alerts_timer) == 0 {
        let observer = Rc::downgrade(owner);
        event_set(&raw mut (*w).alerts_timer, -1, 0, move |_, _| {
            if let Some(owner) = observer.upgrade() {
                log_debug(format_args!("@{} alerts timer expired", owner.id()));
                crate::src::alerts::alerts_queue(&owner, WINDOW_SILENCE);
            }
        });
    }
    (*w).flags &= !WINDOW_SILENCE;
    event_del(&raw mut (*w).alerts_timer);
    let mut timeout = timeval {
        tv_sec: owner
            .with_options_mut(|options| options_get_number(options, c"monitor-silence".as_ptr()))
            as _,
        tv_usec: 0,
    };
    log_debug(format_args!(
        "@{} alerts timer reset {}",
        owner.id(),
        timeout.tv_sec as u32
    ));
    if timeout.tv_sec != 0 {
        event_add(&raw mut (*w).alerts_timer, &mut timeout);
    }
}

pub(super) unsafe fn queue(owner: &WindowRef, flags: i32) -> Option<bool> {
    assert_eq!(flags & !WINDOW_ALERTFLAGS, 0, "alert flags only");
    owner.reset_alert_timer();
    let w = owner.get();
    if (*w).flags & flags != flags {
        (*w).flags |= flags;
        log_debug(format_args!(
            "@{} alerts flags added {}",
            owner.id(),
            crate::src::log::log_hex(flags as u32 as u64)
        ));
    }
    let enabled = owner.with_options_mut(|options| {
        [
            (WINDOW_BELL, c"monitor-bell"),
            (WINDOW_ACTIVITY, c"monitor-activity"),
            (WINDOW_SILENCE, c"monitor-silence"),
        ]
        .into_iter()
        .any(|(flag, name)| flags & flag != 0 && options_get_number(options, name.as_ptr()) != 0)
    });
    if !enabled {
        return None;
    }
    if (*w).alerts_queued != 0 {
        return Some(false);
    }
    (*w).alerts_queued = 1;
    Some(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::src::options::{options_create, options_default, options_set_number};

    #[test]
    fn queue_membership_covers_delivery_and_disabled_alerts_still_record_flags() {
        unsafe {
            let window = window::new();
            let mut options = options_create(None);
            for name in [c"monitor-bell", c"monitor-activity", c"monitor-silence"] {
                let definition = crate::src::options_table::options_table
                    .iter()
                    .find(|entry| entry.name == Some(name))
                    .unwrap();
                options_default(&mut *options, definition);
                options_set_number(&mut *options, name.as_ptr(), 0);
            }
            (*window.get()).options = Some(options);
            assert_eq!(window.queue_alerts(WINDOW_ACTIVITY), None);
            assert_ne!(window.pending_alerts() & WINDOW_ACTIVITY, 0);
            window.with_options_mut(|options| {
                options_set_number(options, c"monitor-bell".as_ptr(), 1)
            });
            let count = Rc::strong_count(&window);
            assert_eq!(window.queue_alerts(WINDOW_BELL), Some(true));
            // Reentrant delivery can set another flag, but cannot claim another owner.
            assert_eq!(window.queue_alerts(WINDOW_BELL), Some(false));
            assert_eq!(Rc::strong_count(&window), count);
            window.finish_alerts();
            assert_eq!(window.pending_alerts(), 0);
            assert_eq!(window.queue_alerts(WINDOW_BELL), Some(true));
            window.finish_alerts();
            window.release(c"alert membership test");
            crate::src::reactor::shutdown_runtime();
        }
    }
}
