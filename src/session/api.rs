//! Operations on retained session holders. Cleanup remains explicit.
use super::*;
use crate::src::shared::spawn::spawn_context;

/// The caller must preserve the legacy single-threaded borrow and logical
/// lifetime rules. No model/component reference may survive a callback.
pub trait Session {
    unsafe fn id(&self) -> u32;
    unsafe fn name(&self) -> CString;
    unsafe fn rename(&self, name: &CStr) -> Result<(), CString>;
    unsafe fn activity_time(&self) -> timeval;
    unsafe fn is_attached(&self) -> bool;
    unsafe fn current_winlink(&self) -> refbox::Weak<winlink>;
    /// Observe MRU history without selecting it (notably command target `!`).
    unsafe fn last_winlink(&self) -> refbox::Weak<winlink>;
    /// Read only. Do not mutate entries, reenter, or let references/pointers escape.
    unsafe fn with_winlinks<R>(&self, read: impl FnOnce(&winlinks) -> R) -> R;
    unsafe fn select_winlink(&self, link: refbox::Weak<winlink>) -> i32;
    unsafe fn detach_window(&self, link: refbox::Weak<winlink>) -> i32;
    /// Adopt an existing window, including break-pane's already-running pane.
    unsafe fn attach_window(
        &self,
        window: &Rc<UnsafeCell<window>>,
        index: i32,
    ) -> Result<refbox::Weak<winlink>, CString>;
    unsafe fn link_window(
        &self,
        source: &Rc<UnsafeCell<session>>,
        link: refbox::Weak<winlink>,
        index: i32,
        replace: bool,
        select: bool,
    ) -> Result<(), CString>;
    unsafe fn spawn_window(
        &self,
        context: &mut spawn_context,
    ) -> Result<refbox::Weak<winlink>, CString>;
    unsafe fn renumber_windows(&self);
    unsafe fn update_activity(&self, from: Option<timeval>);
    unsafe fn on_attached(&self);
    unsafe fn status_layout(&self) -> (i32, u32);
    unsafe fn join_group(&self, name: &CStr);
    /// Scopes cannot reenter model code, run callbacks, change component parents,
    /// free components, or allow references/pointers to escape.
    unsafe fn with_options_mut<R>(&self, edit: impl FnOnce(&mut options) -> R) -> R;
    unsafe fn with_environment_mut<R>(&self, edit: impl FnOnce(&mut environ) -> R) -> R;
    unsafe fn cwd(&self) -> Option<CString>;
    unsafe fn termios(&self) -> Option<termios>;
    /// Evaluate a session builtin in a context already targeting this holder.
    unsafe fn format_value(
        &self,
        key: &CStr,
        context: &mut crate::src::shared::format::format_tree,
    ) -> Option<crate::src::format::FormatValue>;
    unsafe fn destroy(&self, notify: bool, from: &CStr);
}

impl Session for Rc<UnsafeCell<session>> {
    unsafe fn id(&self) -> u32 {
        (*self.get()).id
    }
    unsafe fn name(&self) -> CString {
        (*self.get()).name.clone()
    }

    unsafe fn rename(&self, name: &CStr) -> Result<(), CString> {
        use crate::src::tmux::{check_name, clean_name_cstring};
        if !check_name(name) {
            let mut message = b"invalid session name: ".to_vec();
            message.extend_from_slice(name.to_bytes());
            return Err(CString::new(message).expect("C string name"));
        }
        let new_name = clean_name_cstring(name, 0).expect("validated session name");
        let old_name = self.name();
        if old_name == new_name {
            return Ok(());
        }
        if session_find(&new_name).is_some() {
            let mut message = b"duplicate session: ".to_vec();
            message.extend_from_slice(new_name.as_bytes());
            return Err(CString::new(message).expect("C string name"));
        }
        let mut find = cmd_find_state {
            flags: 0,
            s: Default::default(),
            wl: Default::default(),
            w: Default::default(),
            wp: Default::default(),
            idx: 0,
        };
        cmd_find_from_session(&mut find, self, 0);
        let mut payload = event_payload_create();
        event_payload_set_target(&mut payload, &find);
        event_payload_set_session(&mut payload, c"session".as_ptr(), self.clone());
        event_payload_set_string(&mut payload, c"old_name".as_ptr(), |out| {
            out.write_all(old_name.as_bytes())
        });
        event_payload_set_string(&mut payload, c"new_name".as_ptr(), |out| {
            out.write_all(new_name.as_bytes())
        });
        let owner = sessions_remove(&mut sessions, self).expect("registered session owner");
        drop(session_replace_name(&mut *self.get(), new_name));
        sessions_insert(&mut sessions, owner);
        crate::src::server_fn::server_status_session(&*self.get());
        events_fire(c"session-renamed".as_ptr(), payload);
        Ok(())
    }

    unsafe fn activity_time(&self) -> timeval {
        (*self.get()).activity_time
    }
    unsafe fn is_attached(&self) -> bool {
        (*self.get()).attached != 0
    }
    unsafe fn current_winlink(&self) -> refbox::Weak<winlink> {
        (*self.get()).curw.clone()
    }
    unsafe fn last_winlink(&self) -> refbox::Weak<winlink> {
        (*self.get()).lastw.front().cloned().unwrap_or_default()
    }
    unsafe fn with_winlinks<R>(&self, read: impl FnOnce(&winlinks) -> R) -> R {
        read(&(*self.get()).windows)
    }
    unsafe fn select_winlink(&self, link: refbox::Weak<winlink>) -> i32 {
        session_set_current(self, link)
    }
    unsafe fn detach_window(&self, link: refbox::Weak<winlink>) -> i32 {
        session_detach(self, link)
    }
    unsafe fn attach_window(
        &self,
        window: &Rc<UnsafeCell<window>>,
        index: i32,
    ) -> Result<refbox::Weak<winlink>, CString> {
        session_attach(self, window, index)
    }
    unsafe fn link_window(
        &self,
        source: &Rc<UnsafeCell<session>>,
        link: refbox::Weak<winlink>,
        index: i32,
        replace: bool,
        select: bool,
    ) -> Result<(), CString> {
        crate::src::server_fn::server_link_window(
            source,
            link,
            self,
            index,
            replace as i32,
            select as i32,
        )
    }
    unsafe fn spawn_window(
        &self,
        context: &mut spawn_context,
    ) -> Result<refbox::Weak<winlink>, CString> {
        assert!(
            context.s.ptr_eq(&Rc::downgrade(self)),
            "spawn context session must match receiver"
        );
        let mut cause = None;
        let link = super::spawn::spawn_window(context, &mut cause);
        if link.is_alive() {
            Ok(link)
        } else {
            Err(cause.expect("failed window spawn supplies a cause"))
        }
    }
    unsafe fn renumber_windows(&self) {
        session_renumber_windows(self);
    }
    unsafe fn update_activity(&self, from: Option<timeval>) {
        session_update_activity(&mut *self.get(), from);
    }
    unsafe fn on_attached(&self) {
        self.update_activity(None);
        session_theme_changed(Some(&*self.get()));
        gettimeofday(
            &raw mut (*self.get()).last_attached_time,
            std::ptr::null_mut(),
        );
        self.current_winlink().get_mut_unchecked().flags &= !WINLINK_ALERTFLAGS;
        crate::src::alerts::alerts_check_session(self);
    }
    unsafe fn status_layout(&self) -> (i32, u32) {
        ((*self.get()).statusat, (*self.get()).statuslines)
    }
    unsafe fn join_group(&self, name: &CStr) {
        let group = session_group_new(name.as_ptr());
        session_group_add(group, self);
        session_group_synchronize_to(self);
    }
    unsafe fn with_options_mut<R>(&self, edit: impl FnOnce(&mut options) -> R) -> R {
        edit(
            (*self.get())
                .options
                .as_deref_mut()
                .expect("session options"),
        )
    }
    unsafe fn with_environment_mut<R>(&self, edit: impl FnOnce(&mut environ) -> R) -> R {
        edit(
            (*self.get())
                .environ
                .as_deref_mut()
                .expect("session environment"),
        )
    }
    unsafe fn cwd(&self) -> Option<CString> {
        (*self.get()).cwd.clone()
    }
    unsafe fn termios(&self) -> Option<termios> {
        (*self.get()).tio.as_deref().copied()
    }
    unsafe fn format_value(
        &self,
        key: &CStr,
        context: &mut crate::src::shared::format::format_tree,
    ) -> Option<crate::src::format::FormatValue> {
        super::format::format_value(self, key, context)
    }
    unsafe fn destroy(&self, notify: bool, from: &CStr) {
        session_destroy(self, notify as i32, from.as_ptr());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rename_rejects_invalid_utf8_before_touching_the_registry() {
        unsafe {
            let session = session::new();
            let invalid = CString::new(vec![0xff]).unwrap();
            assert_eq!(
                session.rename(&invalid).unwrap_err().as_bytes(),
                b"invalid session name: \xff"
            );
            assert_eq!(session.name().as_bytes(), b"");
        }
    }

    #[test]
    fn status_layout_is_cached_until_the_sizing_pass_publishes_it() {
        unsafe {
            let session = session::new();
            let mut options = crate::src::options::options_create(std::ptr::null_mut());
            for key in [c"status", c"status-position"] {
                let definition = crate::src::options_table::options_table
                    .iter()
                    .find(|definition| definition.name == Some(key))
                    .unwrap();
                crate::src::options::options_default(&mut *options, definition);
            }
            (*session.get()).options = Some(options);
            status_update_cache(&mut *session.get());
            let before = session.status_layout();
            session.with_options_mut(|options| {
                crate::src::options::options_set_number(options, c"status".as_ptr(), 0);
            });
            assert_eq!(
                session.status_layout(),
                before,
                "configuration edits do not publish the sizing cache"
            );
            status_update_cache(&mut *session.get());
            assert_eq!(session.status_layout(), (-1, 0));
        }
    }
}
