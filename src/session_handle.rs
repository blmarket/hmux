use crate::types::*;
use std::ffi::CStr;

/// Session operations keep compatibility payload access inside the session engine.
impl SessionRef {
    /// # Safety
    /// Exclude mutable payload access during this query.
    pub(crate) unsafe fn current_link(&self) -> Option<crate::window::WinlinkRef> {
        let index = unsafe { self.as_session().curw_idx }?;
        crate::window::WinlinkRef::new(self.clone(), index)
    }

    /// Marks attached clients of this exact session for a full redraw.
    ///
    /// This only updates client flags; it neither draws nor invokes callbacks.
    ///
    /// # Safety
    /// Run on the server thread with its client registry initialized. Exclude
    /// mutable session access and all client payload access during this operation.
    pub(crate) unsafe fn request_redraw(&self) {
        unsafe { crate::server::server_redraw_session(self.as_session()) };
    }

    /// Marks attached clients of this exact session for a status refresh.
    ///
    /// This only updates client flags; it neither draws nor invokes callbacks.
    ///
    /// # Safety
    /// Run on the server thread with its client registry initialized. Exclude
    /// mutable session access and all client payload access during this operation.
    pub(crate) unsafe fn request_status(&self) {
        unsafe { crate::server::server_status_session(self.as_session()) };
    }

    /// Queues a session notification using the current name and target context.
    ///
    /// An unregistered session uses a target resolved from nothing. Notification
    /// construction respects hook suppression, retains the named session, and
    /// snapshots hook formats and target observations. Control notifications and
    /// hook callbacks do not run here.
    ///
    /// # Safety
    /// Run on the server thread with registries and command queues initialized.
    /// Exclude mutable session and target payload access during construction.
    pub(crate) unsafe fn notify(&self, name: &CStr) {
        unsafe { crate::notify::notify_session(name, Some(self.as_session())) };
    }

    /// Clears alert flags on this session's links and their windows.
    ///
    /// Links in other sessions retain their flags even when sharing a window.
    /// The retained session need not be registered; its current links are used.
    /// This does not reset alert timers, request redraw/status, or invoke callbacks.
    ///
    /// # Safety
    /// Exclude other session payload access during this synchronous operation.
    pub(crate) unsafe fn clear_alert_flags(&mut self) {
        for link in unsafe { self.as_session_mut().windows.values_mut() } {
            link.window_handle()
                .expect("linked window")
                .clear_alert_flags();
            link.flags &= !crate::window::WINLINK_ALERTFLAGS;
        }
    }
}

impl SessionRef {
    /// # Safety
    /// Exclude mutation of the session's links during this query.
    pub(crate) unsafe fn window_count(&self) -> u_int {
        unsafe { self.as_session().windows.len() as u_int }
    }

    /// # Safety
    /// Exclude mutation of the session's links during this query.
    pub(crate) unsafe fn current_index(&self) -> Option<core::ffi::c_int> {
        unsafe { self.current_link().map(|link| link.index()) }
    }

    /// Returns the smallest linked-window index, or `None` when no links remain.
    /// This synchronous query invokes no callbacks and returns no payload borrow.
    ///
    /// # Safety
    /// Exclude mutable session payload access, including link mutation, during this query.
    pub(crate) unsafe fn first_index(&self) -> Option<core::ffi::c_int> {
        unsafe {
            self.as_session()
                .windows
                .first_key_value()
                .map(|(_, link)| link.idx)
        }
    }

    /// # Safety
    /// Exclude mutation of the session's links during this query.
    pub(crate) unsafe fn links_to(&self, window: &WindowRef) -> usize {
        unsafe {
            self.as_session()
                .windows
                .values()
                .filter(|link| link.window_handle().is_some_and(|held| held.ptr_eq(window)))
                .count()
        }
    }

    /// Retains the first window in link-index order other than `window`.
    /// Query again after removing a window, since removal can change every link.
    ///
    /// # Safety
    /// Exclude mutation of the session's links during this query.
    pub(crate) unsafe fn first_other_window(&self, window: &WindowRef) -> Option<WindowRef> {
        unsafe {
            self.as_session().windows.values().find_map(|link| {
                let other = link.window_handle()?;
                (!other.ptr_eq(window)).then(|| other.clone())
            })
        }
    }

    /// Returns the sole link index whose window has exactly `name`.
    /// Returns `Ok(None)` for no match and `Err(())` for multiple matching links,
    /// including multiple links to the same window.
    ///
    /// # Safety
    /// Exclude mutation of session links and window names during this query.
    pub(crate) unsafe fn unique_window_index_named(
        &self,
        name: &CStr,
    ) -> Result<Option<core::ffi::c_int>, ()> {
        unsafe {
            let mut found = None;
            for (&index, link) in &self.as_session().windows {
                if !link
                    .window_handle()
                    .is_some_and(|window| window.window_name().as_deref() == Some(name))
                {
                    continue;
                }
                if found.replace(index).is_some() {
                    return Err(());
                }
            }
            Ok(found)
        }
    }
}

impl SessionRef {
    /// Unlinks an index, redrawing its group or destroying it if this session is empty.
    ///
    /// Detaching a present link updates selection when needed, queues unlink
    /// notifications, and synchronizes the group. An empty result triggers client
    /// reassignment or detach and session destruction for every group member (or
    /// this session alone when ungrouped). A missing index still follows this
    /// empty/nonempty decision. Retaining the session does not preserve registration
    /// or links.
    ///
    /// # Safety
    /// Run on the initialized server thread. Exclude conflicting session/group,
    /// window, pane and client payload access throughout this operation, including
    /// through callbacks. Teardown may run synchronous lifecycle callbacks; do
    /// not retain payload borrows across the call.
    pub(crate) unsafe fn unlink_window(&mut self, index: core::ffi::c_int) {
        unsafe { crate::server::server_unlink_window(self.as_session_mut(), index) }
    }

    /// Requests a full redraw for this session's group, or this session alone.
    ///
    /// Snapshots group members in join order and marks their attached clients.
    /// This only updates client flags; it neither draws nor invokes callbacks.
    ///
    /// # Safety
    /// Run on the server thread with group and client registries initialized.
    /// Exclude mutable session/group-member access and all client payload access.
    pub(crate) unsafe fn redraw_group(&self) {
        unsafe { crate::server::server_redraw_session_group(self.as_session()) }
    }

    /// Requests only a status refresh for this session's group, or this session alone.
    ///
    /// Snapshots group members in join order and marks their attached clients.
    /// This only updates client flags; it neither draws nor invokes callbacks.
    ///
    /// # Safety
    /// Run on the server thread with group and client registries initialized.
    /// Exclude mutable session/group-member access and all client payload access.
    pub(crate) unsafe fn status_group(&self) {
        unsafe { crate::server::server_status_session_group(self.as_session()) }
    }

    /// # Safety
    /// Exclude conflicting session and linked-window access throughout this operation.
    pub(crate) unsafe fn shuffle_windows(
        &mut self,
        around: Option<core::ffi::c_int>,
        before: core::ffi::c_int,
    ) -> Option<crate::window::WinlinkShuffle> {
        unsafe { crate::window::winlink_shuffle_up(self.as_session_mut(), around, before) }
    }
}

impl SessionRef {
    /// # Safety
    /// Exclude mutation of session links while taking this snapshot.
    pub(crate) unsafe fn links(&self) -> Vec<crate::window::WinlinkRef> {
        unsafe {
            self.as_session()
                .windows
                .keys()
                .filter_map(|index| crate::window::WinlinkRef::new(self.clone(), *index))
                .collect()
        }
    }

    pub(crate) fn link(&self, index: core::ffi::c_int) -> Option<crate::window::WinlinkRef> {
        crate::window::WinlinkRef::new(self.clone(), index)
    }

    /// # Safety
    /// Exclude mutation of session links during this lookup.
    pub(crate) unsafe fn first_link_to(
        &self,
        window: &WindowRef,
    ) -> Option<crate::window::WinlinkRef> {
        unsafe {
            self.links()
                .into_iter()
                .find(|link| link.window().is_some_and(|linked| linked.ptr_eq(window)))
        }
    }
}

impl SessionRef {
    /// Reads the environment without copying its entries.
    ///
    /// Invokes `read` once synchronously; its result cannot borrow the environment.
    /// The retained session need not be registered.
    ///
    /// # Safety
    /// The session must have an initialized environment. Exclude mutable session
    /// payload and environment access until `read` returns, including any reentry
    /// from the reader. The reader must finish before dispatching commands or hooks.
    pub(crate) unsafe fn with_environment<R>(
        &self,
        read: impl FnOnce(&crate::environ::RustEnvironment) -> R,
    ) -> R {
        read(unsafe { self.as_session().environ_ref() })
    }
}
