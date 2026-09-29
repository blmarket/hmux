//! Session-owned cache publication used by creation and the sizing pass.
use super::*;
use crate::src::server::clients;
use crate::src::server_client::Client;
use crate::src::shared::client::CLIENT_UNATTACHEDFLAGS;

/// Publish the Session-owned state used by the subsequent client/window sizing
/// passes. A removed Session can still be retained by a closing client: preserve
/// the legacy accumulation for that Session without refreshing its cached layout.
/// Neither registry traversal nor numeric option lookup invokes callbacks.
pub(crate) unsafe fn recalculate_size_state() {
    let mut cursor = sessions_minmax(&sessions);
    while let Some(owner) = cursor {
        (*owner.get()).attached = 0;
        status_update_cache(&mut *owner.get());
        cursor = sessions_next(&*owner.get());
    }
    let mut cursor = clients.first();
    while let Some(owner) = cursor {
        if owner.counts_as_attached() {
            if let Some(session) = owner.attached_session().upgrade() {
                let state = &mut *session.get();
                state.attached = state.attached.wrapping_add(1);
            }
        }
        cursor = clients.next(&owner);
    }
}

pub(super) unsafe fn status_update_cache(s_value: &mut session) {
    let s: *mut session = s_value as *mut _;
    (*s).statuslines = options_get_number(
        options_owner_ptr(&mut (*s).options).map_or(std::ptr::null_mut(), |options| options),
        b"status\0" as *const u8 as *const ::core::ffi::c_char,
    ) as u_int;
    if (*s).statuslines == 0 as u_int {
        (*s).statusat = -(1 as ::core::ffi::c_int);
    } else if options_get_number(
        options_owner_ptr(&mut (*s).options).map_or(std::ptr::null_mut(), |options| options),
        b"status-position\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_longlong
    {
        (*s).statusat = 0 as ::core::ffi::c_int;
    } else {
        (*s).statusat = 1 as ::core::ffi::c_int;
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::src::options::{options_create, options_default, options_set_number};
    use crate::src::server_client::ClientRegistry;
    use crate::src::shared::client::{client, CLIENT_CONTROL, CLIENT_IGNORESIZE, CLIENT_SUSPENDED};

    unsafe fn new_session(name: &CStr, status: i64) -> Rc<UnsafeCell<session>> {
        let owner = session::new();
        (*owner.get()).name = name.to_owned();
        let mut options = options_create(std::ptr::null_mut());
        for key in [c"status", c"status-position"] {
            let definition = crate::src::options_table::options_table
                .iter()
                .find(|definition| definition.name == Some(key))
                .unwrap();
            options_default(&mut *options, definition);
        }
        options_set_number(&mut *options, c"status".as_ptr(), status);
        (*owner.get()).options = Some(options);
        owner
    }

    unsafe fn add_client(session: &Rc<UnsafeCell<session>>, flags: u64) {
        let owner = client::with_session_for_test(Some(session));
        owner.update_flags(flags, 0);
        clients.push_back(owner);
    }

    #[test]
    fn sizing_resets_registered_sessions_and_preserves_removed_session_accounting() {
        unsafe {
            let saved_sessions = std::mem::replace(
                &mut sessions,
                crate::src::shared::session::sessions { storage: None },
            );
            let saved_clients = std::mem::replace(&mut clients, ClientRegistry::new());
            let registered = new_session(c"registered", 2);
            let empty = new_session(c"empty", 0);
            let removed = new_session(c"removed", 1);
            for owner in [&registered, &empty, &removed] {
                (*owner.get()).attached = 3;
                sessions_insert(&mut sessions, owner.clone());
            }
            status_update_cache(&mut *removed.get());
            let removed_cache = removed.status_layout();
            // This is the interval after destruction removes the Session from
            // the registry but before closing clients switch to their fallback.
            let removed_registry_owner = sessions_remove(&mut sessions, &removed).unwrap();
            removed.with_options_mut(|options| {
                options_set_number(options, c"status".as_ptr(), 0);
            });
            add_client(&registered, CLIENT_IGNORESIZE as u64);
            add_client(&registered, CLIENT_SUSPENDED as u64);
            add_client(&removed, 0);
            add_client(&removed, CLIENT_CONTROL as u64);

            recalculate_size_state();
            assert_eq!((*registered.get()).attached, 1);
            assert_eq!(registered.status_layout().1, 2);
            assert_eq!((*empty.get()).attached, 0);
            assert_eq!(empty.status_layout(), (-1, 0));
            assert_eq!((*removed.get()).attached, 5);
            assert_eq!(removed.status_layout(), removed_cache);
            recalculate_size_state();
            assert_eq!((*registered.get()).attached, 1);
            assert_eq!((*removed.get()).attached, 7);
            assert_eq!(removed.status_layout(), removed_cache);

            drop(std::mem::replace(&mut clients, saved_clients));
            for owner in [&registered, &empty] {
                drop(sessions_remove(&mut sessions, owner).unwrap());
            }
            sessions = saved_sessions;
            drop(removed_registry_owner);
        }
    }
}
