use crate::cmd::{CMD_PARSE_SUCCESS, cmd_parse_from_string};
use crate::cmd::{CMD_RETURN_ERROR, CMD_RETURN_NORMAL, cmd_list_print};
use crate::cmd::{
    CmdqStateRef, cmdq_add_formats, cmdq_append, cmdq_get_callback1, cmdq_get_command,
    cmdq_get_flags, cmdq_get_target, cmdq_insert_after, cmdq_new_state, cmdq_running,
};
use crate::cmd::{
    cmd_find_clear_state, cmd_find_copy_state, cmd_find_empty_state, cmd_find_from_client,
    cmd_find_from_nothing, cmd_find_from_pane, cmd_find_from_session, cmd_find_from_session_window,
    cmd_find_from_window, cmd_find_from_winlink, cmd_find_valid_state,
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
use crate::options::options_get_ptr;
use crate::options::{
    options_array_first, options_array_item_command, options_array_next, options_get_string,
};
use crate::server::client_ref_from_ptr;
use crate::session::{
    session_alive, session_id, session_name, session_options, session_ref_from_ptr,
};
use crate::tmux::global_s_options;
pub use crate::types::*;
use crate::window::window_ref_from_ptr;
use ::core::ffi::{CStr, c_char, c_int};
use ::core::ptr::{null, null_mut};

pub const CMDQ_STATE_NOHOOKS: c_int = 0x4;
pub const FORMAT_NOJOBS: c_int = 0x4;

/// What a notification carries until the command queue gets round to it: the
/// name of the hook, the target it was raised against, the format tree its
/// commands read, and strong handles for the session state it needs.
#[repr(C)]
pub struct notify_entry {
    pub name: Option<::std::ffi::CString>,
    pub fs: cmd_find_state,
    pub formats: Box<format_tree>,
    pub(crate) client_ref: Option<ClientRef>,
    pub(crate) session_ref: Option<SessionRef>,
    pub(crate) window_ref: Option<WindowRef>,
    pub pane: c_int,
    pub pbname: Option<::std::ffi::CString>,
}

impl notify_entry {
    /// The client the notification was raised against, or null for none.
    pub(crate) fn client(&self) -> *mut client {
        self.client_ref
            .as_ref()
            .map_or(null_mut(), ClientRef::as_ptr)
    }

    /// The session the notification was raised against, or null for none.
    pub(crate) fn session(&self) -> *mut session {
        self.session_ref
            .as_ref()
            .map_or(null_mut(), SessionRef::as_ptr)
    }

    /// The window the notification was raised against, or null for none.
    pub(crate) fn window(&self) -> *mut window {
        self.window_ref
            .as_ref()
            .map_or(null_mut(), WindowRef::as_ptr)
    }
}

/// Puts the commands of one hook on the queue behind `item`, and answers the
/// item the next one should go behind.
unsafe fn notify_insert_one_hook(
    item: *mut cmdq_item,
    ne: &notify_entry,
    cmdlist: Option<&CmdListRef>,
    state: &CmdqStateRef,
) -> *mut cmdq_item {
    unsafe {
        let Some(cmdlist) = cmdlist else {
            return item;
        };
        if log_get_level() != 0 {
            let s = cmd_list_print(cmdlist.as_ptr(), 0);
            log_debug(
                c"%s: hook %s is: %s".as_ptr(),
                fmt_args![
                    c"notify_insert_one_hook".as_ptr(),
                    ne.name.as_deref(),
                    s.as_ptr()
                ],
            );
        }
        cmdq_insert_after(item, cmdq_get_command(cmdlist, Some(state)))
    }
}

/// Looks the hook `ne` names up against the target it was raised for and puts
/// whatever it finds on the queue behind `item`.
///
/// The hook is looked for on the session first, then on the pane and then on
/// the window the target names, which is how a hook set at a narrower scope
/// wins. A name beginning with `@` is a user option holding one command line
/// rather than an array of them.
unsafe fn notify_insert_hook(mut item: *mut cmdq_item, ne: &mut notify_entry) {
    unsafe {
        let name_ptr = cstr_ptr(&ne.name);
        log_debug(
            c"%s: inserting hook %s".as_ptr(),
            fmt_args![c"notify_insert_hook".as_ptr(), name_ptr],
        );

        let mut fs = cmd_find_state::default();
        cmd_find_clear_state(&mut fs, 0);
        if cmd_find_empty_state(&ne.fs) != 0 || cmd_find_valid_state(&ne.fs) == 0 {
            cmd_find_from_nothing(&mut fs, 0);
        } else {
            cmd_find_copy_state(&mut fs, &ne.fs);
        }

        let mut oo = if fs.session().is_null() {
            global_s_options
        } else {
            session_options(fs.session())
        };
        let mut o = options_get_ptr(oo, name_ptr);
        if o.is_null() && !fs.pane().is_null() {
            oo = (*fs.pane()).options_ptr();
            o = options_get_ptr(oo, name_ptr);
        }
        if o.is_null() && !fs.winlink().is_null() {
            oo = (*(*fs.winlink()).window()).options_ptr();
            o = options_get_ptr(oo, name_ptr);
        }
        if o.is_null() {
            log_debug(
                c"%s: hook %s not found".as_ptr(),
                fmt_args![c"notify_insert_hook".as_ptr(), name_ptr],
            );
            return;
        }

        let state = cmdq_new_state(&raw mut fs, null_mut::<key_event>(), CMDQ_STATE_NOHOOKS);
        cmdq_add_formats(state.as_ptr(), &mut ne.formats);

        if ne
            .name
            .as_deref()
            .is_some_and(|n| n.to_bytes().starts_with(b"@"))
        {
            let value = options_get_string(oo, name_ptr);
            let mut pr = cmd_parse_from_string(value, null_mut::<cmd_parse_input>());
            if pr.status == CMD_PARSE_SUCCESS {
                let cmdlist = pr.cmdlist.take();
                notify_insert_one_hook(item, ne, cmdlist.as_ref(), &state);
            } else {
                let error = pr.error.take().unwrap();
                log_debug(
                    c"%s: can't parse hook %s: %s".as_ptr(),
                    fmt_args![c"notify_insert_hook".as_ptr(), name_ptr, error.as_ptr()],
                );
            }
        } else {
            let mut a = options_array_first(o);
            while !a.is_null() {
                let cmdlist = options_array_item_command(a);
                item = notify_insert_one_hook(item, ne, cmdlist.as_ref(), &state);
                a = options_array_next(o, a);
            }
        }
    }
}

/// Runs a notification once the command queue reaches it: tells the control
/// clients about it, puts its hook's commands on the queue, and gives back
/// everything the entry was holding.
unsafe fn notify_callback(item: *mut cmdq_item, data: CmdqCallbackData) -> cmd_retval {
    unsafe {
        let CmdqCallbackData::NotifyEntry(mut ne) = data else {
            return CMD_RETURN_ERROR;
        };
        log_debug(
            c"%s: %s".as_ptr(),
            fmt_args![c"notify_callback".as_ptr(), ne.name.as_deref()],
        );

        match ne.name.as_deref().map(|s| s.to_bytes()) {
            Some(b"pane-mode-changed") => control_notify_pane_mode_changed(ne.pane),
            Some(b"window-layout-changed") => control_notify_window_layout_changed(ne.window()),
            Some(b"window-pane-changed") => control_notify_window_pane_changed(ne.window()),
            Some(b"window-unlinked") => control_notify_window_unlinked(ne.session(), ne.window()),
            Some(b"window-linked") => control_notify_window_linked(ne.session(), ne.window()),
            Some(b"window-renamed") => control_notify_window_renamed(ne.window()),
            Some(b"client-session-changed") => control_notify_client_session_changed(ne.client()),
            Some(b"client-detached") => control_notify_client_detached(ne.client()),
            Some(b"session-renamed") => control_notify_session_renamed(ne.session()),
            Some(b"session-created") => control_notify_session_created(ne.session()),
            Some(b"session-closed") => control_notify_session_closed(ne.session()),
            Some(b"session-window-changed") => control_notify_session_window_changed(ne.session()),
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
    name: *const c_char,
    fs: &cmd_find_state,
    c: *mut client,
    s: *mut session,
    w: *mut window,
    wp: *mut window_pane,
    pbname: *const c_char,
) {
    unsafe {
        let item = cmdq_running(null_mut::<client>());
        if !item.is_null() && cmdq_get_flags(&*item) & CMDQ_STATE_NOHOOKS != 0 {
            return;
        }

        let window_ref = if w.is_null() {
            None
        } else {
            let Some(window_ref) = window_ref_from_ptr(w) else {
                return;
            };
            Some(window_ref)
        };
        let session_ref = if s.is_null() {
            None
        } else {
            let Some(session_ref) = session_ref_from_ptr(s) else {
                return;
            };
            Some(session_ref)
        };
        let mut ne = Box::new(notify_entry {
            name: if name.is_null() {
                None
            } else {
                Some(CStr::from_ptr(name).to_owned())
            },
            fs: cmd_find_state::default(),
            formats: format_create(null_mut(), null_mut(), 0, FORMAT_NOJOBS),
            client_ref: client_ref_from_ptr(c),
            session_ref,
            window_ref,
            pane: if wp.is_null() { -1 } else { (*wp).id as c_int },
            pbname: if pbname.is_null() {
                None
            } else {
                Some(CStr::from_ptr(pbname).to_owned())
            },
        });

        format_add(&mut ne.formats, c"hook", c"%s".as_ptr(), fmt_args![name]);
        if !c.is_null() {
            format_add(
                &mut ne.formats,
                c"hook_client",
                c"%s".as_ptr(),
                fmt_args![(*c).name.as_deref()],
            );
        }
        if !s.is_null() {
            format_add(
                &mut ne.formats,
                c"hook_session",
                c"$%u".as_ptr(),
                fmt_args![session_id(s)],
            );
            format_add(
                &mut ne.formats,
                c"hook_session_name",
                c"%s".as_ptr(),
                fmt_args![session_name(s)],
            );
        }
        if !w.is_null() {
            format_add(
                &mut ne.formats,
                c"hook_window",
                c"@%u".as_ptr(),
                fmt_args![(*w).id],
            );
            format_add(
                &mut ne.formats,
                c"hook_window_name",
                c"%s".as_ptr(),
                fmt_args![(*w).name.as_deref()],
            );
        }
        if !wp.is_null() {
            format_add(
                &mut ne.formats,
                c"hook_pane",
                c"%%%d".as_ptr(),
                fmt_args![(*wp).id],
            );
            format_add(
                &mut ne.formats,
                c"hook_window",
                c"@%u".as_ptr(),
                fmt_args![(*(*wp).window).id],
            );
            format_add(
                &mut ne.formats,
                c"hook_window_name",
                c"%s".as_ptr(),
                fmt_args![(*(*wp).window).name.as_deref()],
            );
        }
        format_log_debug(&mut ne.formats, c"notify_add");

        cmd_find_copy_state(&mut ne.fs, fs);

        cmdq_append(
            null_mut::<client>(),
            cmdq_get_callback1(
                c"notify_callback".as_ptr(),
                Some(notify_callback),
                CmdqCallbackData::NotifyEntry(ne),
            ),
        );
    }
}

/// Runs the hook `name` for the command `item` is running, without going
/// through the queue: the entry lives on the stack and nothing takes a
/// reference on it.
pub unsafe fn notify_hook(item: *mut cmdq_item, name: &CStr) {
    unsafe {
        let target = cmdq_get_target(item);
        let mut ne = notify_entry {
            name: Some(name.to_owned()),
            fs: cmd_find_state::default(),
            formats: format_create(null_mut(), null_mut(), 0, FORMAT_NOJOBS),
            client_ref: None,
            session_ref: None,
            window_ref: None,
            pane: if (*target).pane().is_null() {
                -1
            } else {
                (*(*target).pane()).id as c_int
            },
            pbname: None,
        };
        cmd_find_copy_state(&mut ne.fs, &*target);
        format_add(
            &mut ne.formats,
            c"hook",
            c"%s".as_ptr(),
            fmt_args![name.as_ptr()],
        );
        format_log_debug(&mut ne.formats, c"notify_hook");
        notify_insert_hook(item, &mut ne);
    }
}

/// Raises `name` against a client.
pub unsafe fn notify_client(name: *const c_char, c: *mut client) {
    unsafe {
        let mut fs = cmd_find_state::default();
        cmd_find_from_client(&mut fs, c, 0);
        notify_add(
            name,
            &fs,
            c,
            null_mut::<session>(),
            null_mut::<window>(),
            null_mut::<window_pane>(),
            null::<c_char>(),
        );
    }
}

/// Raises `name` against a session. A session already gone has no target of
/// its own, so one is found from nothing.
pub unsafe fn notify_session(name: *const c_char, s: *mut session) {
    unsafe {
        let mut fs = cmd_find_state::default();
        if session_alive(s) != 0 {
            cmd_find_from_session(&mut fs, s, 0);
        } else {
            cmd_find_from_nothing(&mut fs, 0);
        }
        notify_add(
            name,
            &fs,
            null_mut::<client>(),
            s,
            null_mut::<window>(),
            null_mut::<window_pane>(),
            null::<c_char>(),
        );
    }
}

/// Raises `name` against a window in a session.
pub unsafe fn notify_winlink(name: *const c_char, wl: *mut winlink) {
    unsafe {
        let mut fs = cmd_find_state::default();
        cmd_find_from_winlink(&mut fs, wl, 0);
        notify_add(
            name,
            &fs,
            null_mut::<client>(),
            (*wl).session(),
            (*wl).window(),
            null_mut::<window_pane>(),
            null::<c_char>(),
        );
    }
}

/// Raises `name` against a window that a session no longer has to be showing.
pub unsafe fn notify_session_window(name: *const c_char, s: *mut session, w: *mut window) {
    unsafe {
        let mut fs = cmd_find_state::default();
        cmd_find_from_session_window(&mut fs, s, w, 0);
        notify_add(
            name,
            &fs,
            null_mut::<client>(),
            s,
            w,
            null_mut::<window_pane>(),
            null::<c_char>(),
        );
    }
}

/// Raises `name` against a window.
pub unsafe fn notify_window(name: *const c_char, w: *mut window) {
    unsafe {
        let mut fs = cmd_find_state::default();
        cmd_find_from_window(&mut fs, w, 0);
        notify_add(
            name,
            &fs,
            null_mut::<client>(),
            null_mut::<session>(),
            w,
            null_mut::<window_pane>(),
            null::<c_char>(),
        );
    }
}

/// Raises `name` against a pane.
pub unsafe fn notify_pane(name: *const c_char, wp: *mut window_pane) {
    unsafe {
        let mut fs = cmd_find_state::default();
        cmd_find_from_pane(&mut fs, wp, 0);
        notify_add(
            name,
            &fs,
            null_mut::<client>(),
            null_mut::<session>(),
            null_mut::<window>(),
            wp,
            null::<c_char>(),
        );
    }
}

/// Raises the paste-buffer notification for `pbname`, against no target at all.
pub unsafe fn notify_paste_buffer(pbname: *const c_char, deleted: c_int) {
    unsafe {
        let mut fs = cmd_find_state::default();
        cmd_find_clear_state(&mut fs, 0);
        let name = if deleted != 0 {
            c"paste-buffer-deleted"
        } else {
            c"paste-buffer-changed"
        };
        notify_add(
            name.as_ptr(),
            &fs,
            null_mut::<client>(),
            null_mut::<session>(),
            null_mut::<window>(),
            null_mut::<window_pane>(),
            pbname,
        );
    }
}
#[cfg(test)]
#[path = "tests/test_notify.rs"]
mod tests;
