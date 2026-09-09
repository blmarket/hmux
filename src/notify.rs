use crate::cmd::{CmdListRef, cmd_retval, cmdq_item};
use crate::cmd::CMD_RETURN_NORMAL;
use crate::cmd::{CMD_PARSE_SUCCESS, cmd_parse_from_string};
use crate::cmd::{CmdqItemRef, CmdqStateRef, cmdq_append, cmdq_item_ref_of, cmdq_running};
use crate::cmd::{
    cmd_find_clear_state, cmd_find_copy_state, cmd_find_empty_state, cmd_find_from_client,
    cmd_find_from_nothing, cmd_find_from_pane, cmd_find_from_session, cmd_find_from_winlink,
    cmd_find_valid_state,
};
use crate::control::{
    control_notify_client_detached, control_notify_client_session_changed,
    control_notify_pane_mode_changed, control_notify_paste_buffer_changed,
    control_notify_paste_buffer_deleted, control_notify_session_closed,
    control_notify_session_created, control_notify_session_renamed,
    control_notify_session_window_changed, control_notify_window_layout_changed,
    control_notify_window_linked, control_notify_window_pane_changed,
    control_notify_window_renamed, control_notify_window_unlinked,
};
use crate::fmt_args;
use crate::format::{format_add, format_create, format_log_debug};
use crate::log::{log_debug, log_get_level};
use crate::options::{OptionsEngine, RustOptionsEngine};

use crate::server::client_ref_of;
use crate::session::session_ref_of;
use crate::tmux::global_s_options;
pub use crate::types::*;
use ::core::ffi::{CStr, c_int};

pub use crate::consts::{CMDQ_STATE_NOHOOKS, FORMAT_NOJOBS};

/// What a notification carries until the command queue gets round to it: the
/// name of the hook, the target it was raised against, the format tree its
/// commands read, and strong handles for the session state it needs.
#[repr(C)]
pub struct notify_entry {
    pub name: Option<std::ffi::CString>,
    pub fs: cmd_find_state,
    pub formats: Box<format_tree>,
    pub(crate) client_ref: Option<ClientRef>,
    pub(crate) session_ref: Option<SessionRef>,
    pub(crate) window_ref: Option<WindowRef>,
    pub pane: c_int,
    pub pbname: Option<std::ffi::CString>,
}

impl notify_entry {
    /// The session the notification was raised against, or nothing for none.
    ///
    /// # Safety
    /// No owner or callback may mutate the session while the view is in use.
    pub(crate) unsafe fn session(&self) -> Option<&session> {
        unsafe {
            self.session_ref
                .as_ref()
                .map(|reference| reference.as_session())
        }
    }

    /// Returns the window handle retained by this notification, if any.
    pub(crate) fn window(&self) -> Option<&WindowRef> {
        self.window_ref.as_ref()
    }
}

/// Puts the commands of one hook on the queue behind `item`, and answers the
/// item the next one should go behind.
fn notify_insert_one_hook(
    item: &CmdqItemRef,
    ne: &notify_entry,
    cmdlist: Option<&CmdListRef>,
    state: &CmdqStateRef,
) -> CmdqItemRef {
    unsafe {
        let Some(cmdlist) = cmdlist else {
            return item.clone();
        };
        if log_get_level() != 0 {
            let s = cmdlist.print(0);
            log_debug(
                c"%s: hook %s is: %s",
                fmt_args![
                    c"notify_insert_one_hook".as_ptr(),
                    ne.name.as_deref(),
                    s.as_ptr()
                ],
            );
        }
        item.insert_after(cmdlist.queue_items(Some(state)))
    }
}

/// Looks the hook `ne` names up against the target it was raised for and puts
/// whatever it finds on the queue behind `item`.
///
/// The hook is looked for on the session first, then on the pane and then on
/// the window the target names, which is how a hook set at a narrower scope
/// wins. A name beginning with `@` is a user option holding one command line
/// rather than an array of them.
fn notify_insert_hook(item: &CmdqItemRef, ne: &mut notify_entry) {
    unsafe {
        let mut item = item.clone();
        let name = ne.name.as_deref().expect("a notification has a name");
        log_debug(
            c"%s: inserting hook %s",
            fmt_args![c"notify_insert_hook".as_ptr(), name],
        );

        let mut fs = cmd_find_state::default();
        cmd_find_clear_state(&mut fs, 0);
        if cmd_find_empty_state(&ne.fs) != 0 || cmd_find_valid_state(&ne.fs) == 0 {
            cmd_find_from_nothing(&mut fs, 0);
        } else {
            cmd_find_copy_state(&mut fs, &ne.fs);
        }

        let session = fs.session();
        let session_options = match &session {
            Some(s) => s.options(),
            None => global_s_options
                .as_ref()
                .expect("global options are initialized")
                .clone(),
        };
        let pane_options = fs
            .pane_list_ref()
            .and_then(|pane| pane.get().map(|pane| pane.options_ref().clone()));
        let window_options = fs.window().map(|window| window.options());
        let hook = [Some(session_options), pane_options, window_options]
            .into_iter()
            .flatten()
            .find_map(|store| {
                store.with_entry(name, false, |entry| {
                    let entry = entry?;
                    if name.to_bytes().starts_with(b"@") {
                        Some((
                            Some(
                                RustOptionsEngine.value_string_ref(RustOptionsEngine.value(entry)),
                            ),
                            Vec::new(),
                        ))
                    } else {
                        let commands = RustOptionsEngine
                            .array_indices(entry)
                            .into_iter()
                            .map(|index| {
                                RustOptionsEngine
                                    .array_get(entry, index)
                                    .and_then(|value| RustOptionsEngine.value_command(value))
                            })
                            .collect::<Vec<_>>();
                        Some((None, commands))
                    }
                })
            });
        let Some((text, commands)) = hook else {
            log_debug(
                c"%s: hook %s not found",
                fmt_args![c"notify_insert_hook".as_ptr(), name],
            );
            return;
        };

        let state = CmdqStateRef::create(Some(&fs), None, CMDQ_STATE_NOHOOKS);
        state.add_formats(&mut ne.formats);

        if let Some(value) = text {
            let mut pr = cmd_parse_from_string(&value, None);
            if pr.status == CMD_PARSE_SUCCESS {
                let cmdlist = pr.cmdlist.take();
                notify_insert_one_hook(&item, ne, cmdlist.as_ref(), &state);
            } else {
                let error = pr.error.take().unwrap();
                log_debug(
                    c"%s: can't parse hook %s: %s",
                    fmt_args![c"notify_insert_hook".as_ptr(), name, error.as_ptr()],
                );
            }
        } else {
            for cmdlist in commands {
                item = notify_insert_one_hook(&item, ne, cmdlist.as_ref(), &state);
            }
        }
    }
}

/// Runs a notification once the command queue reaches it: tells the control
/// clients about it, puts its hook's commands on the queue, and gives back
/// everything the entry was holding.
fn notify_callback(item: &CmdqItemRef, mut ne: Box<notify_entry>) -> cmd_retval {
    unsafe {
        log_debug(
            c"%s: %s",
            fmt_args![c"notify_callback".as_ptr(), ne.name.as_deref()],
        );

        match ne.name.as_deref().map(|s| s.to_bytes()) {
            Some(b"pane-mode-changed") => control_notify_pane_mode_changed(ne.pane),
            Some(b"window-layout-changed") => {
                control_notify_window_layout_changed(ne.window().expect("the hook names a window"))
            }
            Some(b"window-pane-changed") => {
                control_notify_window_pane_changed(ne.window().expect("the hook names a window"))
            }
            Some(b"window-unlinked") => control_notify_window_unlinked(
                ne.session(),
                ne.window().expect("the hook names a window"),
            ),
            Some(b"window-linked") => control_notify_window_linked(
                ne.session(),
                ne.window().expect("the hook names a window"),
            ),
            Some(b"window-renamed") => {
                control_notify_window_renamed(ne.window().expect("the hook names a window"))
            }
            Some(b"client-session-changed") => control_notify_client_session_changed(
                ne.client_ref
                    .as_mut()
                    .expect("the hook names a client")
                    .as_client_mut(),
            ),
            Some(b"client-detached") => control_notify_client_detached(
                ne.client_ref
                    .as_mut()
                    .expect("the hook names a client")
                    .as_client_mut(),
            ),
            Some(b"session-renamed") => {
                control_notify_session_renamed(ne.session().expect("the hook names a session"))
            }
            Some(b"session-created") => {
                control_notify_session_created(ne.session().expect("the hook names a session"))
            }
            Some(b"session-closed") => {
                control_notify_session_closed(ne.session().expect("the hook names a session"))
            }
            Some(b"session-window-changed") => control_notify_session_window_changed(
                ne.session().expect("the hook names a session"),
            ),
            Some(b"paste-buffer-changed") => {
                control_notify_paste_buffer_changed(ne.pbname.as_deref())
            }
            Some(b"paste-buffer-deleted") => {
                control_notify_paste_buffer_deleted(ne.pbname.as_deref())
            }
            _ => {}
        }
        crate::plugin::note_notification(ne.name.as_deref(), ne.pane);

        notify_insert_hook(item, &mut ne);
        CMD_RETURN_NORMAL
    }
}

/// Builds a notification and puts it at the end of the queue, taking a
/// reference on everything it names so that it is still there when the queue
/// gets to it. A command that asked for no hooks raises nothing.
unsafe fn notify_add(
    name: &CStr,
    fs: &cmd_find_state,
    c: Option<&client>,
    s: Option<&session>,
    w: Option<&WindowRef>,
    wp: Option<&impl crate::WindowPane>,
    pbname: Option<&CStr>,
) {
    unsafe {
        if cmdq_running(None).is_some_and(|item| item.flags() & CMDQ_STATE_NOHOOKS != 0) {
            return;
        }

        let window_ref = w.cloned();
        let session_ref = match s {
            Some(s) => match session_ref_of(s) {
                Some(session_ref) => Some(session_ref),
                None => return,
            },
            None => None,
        };
        let mut ne = Box::new(notify_entry {
            name: Some(name.to_owned()),
            fs: cmd_find_state::default(),
            formats: format_create(None, None, 0, FORMAT_NOJOBS),
            client_ref: c.and_then(client_ref_of),
            session_ref,
            window_ref,
            pane: wp.map_or(-1, |wp| wp.pane_id() as c_int),
            pbname: pbname.map(CStr::to_owned),
        });

        format_add(&mut ne.formats, c"hook", c"%s", fmt_args![name]);
        if let Some(c) = c {
            format_add(
                &mut ne.formats,
                c"hook_client",
                c"%s",
                fmt_args![c.name.as_deref()],
            );
        }
        if let Some(s) = s {
            format_add(
                &mut ne.formats,
                c"hook_session",
                c"$%u",
                fmt_args![crate::SessionIdentity::session_id(s)],
            );
            format_add(
                &mut ne.formats,
                c"hook_session_name",
                c"%s",
                fmt_args![crate::SessionNameState::session_name(s)],
            );
        }
        if let Some(w) = w {
            format_add(
                &mut ne.formats,
                c"hook_window",
                c"@%u",
                fmt_args![w.window_id()],
            );
            format_add(
                &mut ne.formats,
                c"hook_window_name",
                c"%s",
                fmt_args![w.window_name().as_deref()],
            );
        }
        if let Some(wp) = wp {
            format_add(
                &mut ne.formats,
                c"hook_pane",
                c"%%%d",
                fmt_args![wp.pane_id()],
            );
            if let Some(window) = wp.window_context() {
                let w = &window;
                format_add(
                    &mut ne.formats,
                    c"hook_window",
                    c"@%u",
                    fmt_args![w.window_id()],
                );
                format_add(
                    &mut ne.formats,
                    c"hook_window_name",
                    c"%s",
                    fmt_args![w.window_name().as_deref()],
                );
            }
        }
        format_log_debug(&mut ne.formats, c"notify_add");

        cmd_find_copy_state(&mut ne.fs, fs);

        cmdq_append(
            None,
            CmdqItemRef::callback_items(c"notify_callback", move |item| notify_callback(item, ne)),
        );
    }
}

/// Runs the hook `name` for the command `item` is running, without going
/// through the queue: the entry lives on the stack and nothing takes a
/// reference on it.
pub unsafe fn notify_hook(item: &cmdq_item, name: &CStr) {
    unsafe {
        let item_ref = cmdq_item_ref_of(item).expect("a running hook item is on its queue");
        let target = item.target();
        let mut ne = notify_entry {
            name: Some(name.to_owned()),
            fs: cmd_find_state::default(),
            formats: format_create(None, None, 0, FORMAT_NOJOBS),
            client_ref: None,
            session_ref: None,
            window_ref: None,
            pane: target
                .pane_list_ref()
                .map_or(-1, |pane| pane.pane_id() as c_int),
            pbname: None,
        };
        cmd_find_copy_state(&mut ne.fs, target);
        format_add(&mut ne.formats, c"hook", c"%s", fmt_args![name]);
        format_log_debug(&mut ne.formats, c"notify_hook");
        notify_insert_hook(&item_ref, &mut ne);
    }
}

/// Raises `name` against a client.
pub unsafe fn notify_client(name: &CStr, c: Option<&mut client>) {
    unsafe {
        let mut fs = cmd_find_state::default();
        match c {
            Some(c) => {
                cmd_find_from_client(&mut fs, crate::server::client_ref_of(c).as_ref(), 0);
                notify_add(
                    name,
                    &fs,
                    Some(c),
                    None,
                    None,
                    None::<&crate::types::window_pane>,
                    None,
                );
            }
            None => {
                cmd_find_from_client(&mut fs, None, 0);
                notify_add(
                    name,
                    &fs,
                    None,
                    None,
                    None,
                    None::<&crate::types::window_pane>,
                    None,
                );
            }
        }
    }
}

/// Raises `name` against a session. A session already gone has no target of
/// its own, so one is found from nothing.
pub unsafe fn notify_session(name: &CStr, s: Option<&session>) {
    unsafe {
        let mut fs = cmd_find_state::default();
        if s.and_then(crate::session::session_ref_of)
            .is_some_and(|session| session.is_registered())
        {
            cmd_find_from_session(&mut fs, s.expect("the hook names a session"), 0);
        } else {
            cmd_find_from_nothing(&mut fs, 0);
        }
        notify_add(
            name,
            &fs,
            None,
            s,
            None,
            None::<&crate::types::window_pane>,
            None,
        );
    }
}

/// Raises `name` against a window in a session.
pub unsafe fn notify_winlink(name: &CStr, wl: &winlink) {
    unsafe {
        let mut fs = cmd_find_state::default();
        cmd_find_from_winlink(&mut fs, wl, 0);
        notify_add(
            name,
            &fs,
            None,
            (*wl)
                .session()
                .as_ref()
                .map(|reference| reference.as_session()),
            wl.window_handle(),
            None::<&crate::types::window_pane>,
            None,
        );
    }
}

/// Raises `name` against a window that a session no longer has to be showing.
pub unsafe fn notify_session_window(name: &CStr, s: &session, w: &WindowRef) {
    unsafe {
        let mut fs = cmd_find_state::default();
        crate::window::window_find_from_session(&mut fs, s, w, 0);
        notify_add(
            name,
            &fs,
            None,
            Some(s),
            Some(w),
            None::<&crate::types::window_pane>,
            None,
        );
    }
}

/// Raises `name` against a window.
pub unsafe fn notify_window(name: &CStr, w: Option<&WindowRef>) {
    unsafe {
        let mut fs = cmd_find_state::default();
        crate::window::window_find_from_window(&mut fs, w.expect("the hook names a window"), 0);
        notify_add(
            name,
            &fs,
            None,
            None,
            w,
            None::<&crate::types::window_pane>,
            None,
        );
    }
}

/// Raises `name` against a pane.
pub unsafe fn notify_pane(name: &CStr, wp: Option<&impl crate::WindowPane>) {
    unsafe {
        let mut fs = cmd_find_state::default();
        cmd_find_from_pane(&mut fs, wp.expect("the hook names a pane"), 0);
        notify_add(name, &fs, None, None, None, wp, None);
    }
}

pub(crate) unsafe fn notify_pane_in_window(
    name: &CStr,
    wp: &impl crate::WindowPane,
    window: &WindowRef,
) {
    unsafe {
        let mut fs = cmd_find_state::default();
        if crate::window::window_find_from_window(&mut fs, window, 0) == 0 {
            fs.set_pane(Some(wp));
            crate::cmd::cmd_find_log_state_with_window(
                c"cmd_find_from_pane",
                &fs,
                Some((window.window_id(), window.window_name())),
            );
        }
        notify_add(name, &fs, None, None, None, Some(wp), None);
    }
}

/// Raises the paste-buffer notification for `pbname`, against no target at all.
pub unsafe fn notify_paste_buffer(pbname: &CStr, deleted: c_int) {
    unsafe {
        let mut fs = cmd_find_state::default();
        cmd_find_clear_state(&mut fs, 0);
        let name = if deleted != 0 {
            c"paste-buffer-deleted"
        } else {
            c"paste-buffer-changed"
        };
        notify_add(
            name,
            &fs,
            None,
            None,
            None,
            None::<&crate::types::window_pane>,
            Some(pbname),
        );
    }
}
#[cfg(test)]
#[path = "tests/test_notify.rs"]
mod tests;
