use crate::WindowPane as _;
use crate::cmd::CmdqStateRef;
use crate::cmd::cmd_retval;
use crate::cmd::cmdq_item;
use crate::cmd::{CmdqItemRef, CmdqItemWeak, cmdq_append};
use crate::cmd::{cmd_parse_from_buffer, cmd_parse_from_file};
use crate::compat::error_message;
use crate::control::control_write;
use crate::fmt_args;
use crate::fmt_engine::{FmtArg, format_alloc};
use crate::log::log_debug;
use crate::modes::window_copy_add;
use crate::server::client_ref_of;
use crate::server::first_client;
use crate::session::sessions_first;

pub use crate::consts::{
    CLIENT_CONTROL, CLIENT_EXIT_DETACH, CLIENT_EXIT_RETURN, CLIENT_EXIT_SHUTDOWN, CMD_PARSE_ERROR,
    CMD_PARSE_PARSEONLY, CMD_PARSE_QUIET, CMD_PARSE_SUCCESS, CMD_RETURN_ERROR, CMD_RETURN_NORMAL,
    CMD_RETURN_STOP, CMD_RETURN_WAIT, ENOENT, LAYOUT_LEFTRIGHT, LAYOUT_TOPBOTTOM,
    LAYOUT_WINDOWPANE, MSG_COMMAND, MSG_DETACH, MSG_DETACHKILL, MSG_EXEC, MSG_EXIT, MSG_EXITED,
    MSG_EXITING, MSG_FLAGS, MSG_IDENTIFY_CLIENTPID, MSG_IDENTIFY_CWD, MSG_IDENTIFY_DONE,
    MSG_IDENTIFY_ENVIRON, MSG_IDENTIFY_FEATURES, MSG_IDENTIFY_FLAGS, MSG_IDENTIFY_LONGFLAGS,
    MSG_IDENTIFY_OLDCWD, MSG_IDENTIFY_STDIN, MSG_IDENTIFY_STDOUT, MSG_IDENTIFY_TERM,
    MSG_IDENTIFY_TERMINFO, MSG_IDENTIFY_TTYNAME, MSG_LOCK, MSG_OLDSTDERR, MSG_OLDSTDIN,
    MSG_OLDSTDOUT, MSG_READ, MSG_READ_CANCEL, MSG_READ_DONE, MSG_READ_OPEN, MSG_READY, MSG_RESIZE,
    MSG_SHELL, MSG_SHUTDOWN, MSG_SUSPEND, MSG_UNLOCK, MSG_VERSION, MSG_WAKEUP, MSG_WRITE,
    MSG_WRITE_CLOSE, MSG_WRITE_OPEN, MSG_WRITE_READY, PANE_LINES_DOUBLE, PANE_LINES_HEAVY,
    PANE_LINES_NUMBER, PANE_LINES_SIMPLE, PANE_LINES_SINGLE, PANE_LINES_SPACES, PROGRESS_BAR_ERROR,
    PROGRESS_BAR_HIDDEN, PROGRESS_BAR_INDETERMINATE, PROGRESS_BAR_NORMAL, PROGRESS_BAR_PAUSED,
    PROMPT_COMMAND, PROMPT_ENTRY, PROMPT_TYPE_COMMAND, PROMPT_TYPE_INVALID, PROMPT_TYPE_SEARCH,
    PROMPT_TYPE_TARGET, PROMPT_TYPE_WINDOW_TARGET, RB_NEGINF, SCREEN_CURSOR_BAR,
    SCREEN_CURSOR_BLOCK, SCREEN_CURSOR_DEFAULT, SCREEN_CURSOR_UNDERLINE,
    STYLE_ALIGN_ABSOLUTE_CENTRE, STYLE_ALIGN_CENTRE, STYLE_ALIGN_DEFAULT, STYLE_ALIGN_LEFT,
    STYLE_ALIGN_RIGHT, STYLE_DEFAULT_BASE, STYLE_DEFAULT_POP, STYLE_DEFAULT_PUSH,
    STYLE_DEFAULT_SET, STYLE_LIST_FOCUS, STYLE_LIST_LEFT_MARKER, STYLE_LIST_OFF, STYLE_LIST_ON,
    STYLE_LIST_RIGHT_MARKER, STYLE_RANGE_CONTROL, STYLE_RANGE_LEFT, STYLE_RANGE_NONE,
    STYLE_RANGE_PANE, STYLE_RANGE_RIGHT, STYLE_RANGE_SESSION, STYLE_RANGE_USER, STYLE_RANGE_WINDOW,
    THEME_DARK, THEME_LIGHT, THEME_UNKNOWN,
};
use crate::status::status_prompt_load_history;
pub use crate::types::*;
use crate::window::window_pane_set_mode;
use ::std::ffi::{CStr, CString, OsStr};
use ::std::fs::File;
use ::std::io::Read;
use ::std::os::unix::ffi::OsStrExt;

/// The client the config load is running for, held for as long as the caller
/// uses it, or nothing once the load has no client or that client has gone.
pub fn cfg_client() -> Option<ClientRef> {
    with_config(|config| config.client.as_ref().and_then(ClientWeak::upgrade))
}
pub(crate) struct ConfigState {
    causes: Vec<CString>,
    finished: bool,
    client: Option<ClientWeak>,
    item: Option<CmdqItemWeak>,
    pub(crate) quiet: bool,
    pub(crate) files: Vec<CString>,
}

impl Default for ConfigState {
    fn default() -> Self {
        Self {
            causes: Vec::new(),
            finished: false,
            client: None,
            item: None,
            quiet: true,
            files: Vec::new(),
        }
    }
}

pub(crate) fn configuration_files() -> Vec<CString> {
    with_config(|config| config.files.clone())
}

fn with_config<R>(visit: impl FnOnce(&mut ConfigState) -> R) -> R {
    {
        let process = crate::server::server_process
            .get()
            .expect("server process is initialized");
        let mut process = process.borrow_mut();
        visit(&mut process.config)
    }
}

fn take_causes() -> Vec<CString> {
    with_config(|config| core::mem::take(&mut config.causes))
}
fn cfg_client_done(_item: &CmdqItemRef) -> cmd_retval {
    {
        if !configuration_finished() {
            return CMD_RETURN_WAIT;
        }
        CMD_RETURN_NORMAL
    }
}
fn cfg_done(_item: &CmdqItemRef) -> cmd_retval {
    unsafe {
        if configuration_finished() {
            return CMD_RETURN_NORMAL;
        }
        with_config(|config| config.finished = true);
        cfg_show_causes(None);
        if let Some(item) =
            with_config(|config| config.item.as_ref().and_then(CmdqItemWeak::upgrade))
        {
            item.resume();
        }
        status_prompt_load_history();
        CMD_RETURN_NORMAL
    }
}
pub fn start_cfg() {
    unsafe {
        let mut flags: core::ffi::c_int = 0 as core::ffi::c_int;
        let c = first_client();
        with_config(|config| config.client = c.as_ref().map(ClientRef::downgrade));
        if c.is_some() {
            let item = cmdq_append(
                c.as_ref(),
                CmdqItemRef::callback_items(c"cfg_client_done", cfg_client_done),
            )
            .map(|item| item.downgrade());
            with_config(|config| config.item = item);
        }
        if with_config(|config| config.quiet) {
            flags = CMD_PARSE_QUIET;
        }
        for file in &configuration_files() {
            load_cfg(
                file,
                c.as_ref().map(|reference| reference.as_client()),
                None,
                None,
                flags,
                None,
            );
        }
        cmdq_append(None, CmdqItemRef::callback_items(c"cfg_done", cfg_done));
    }
}
pub unsafe fn load_cfg(
    path: &CStr,
    c: Option<&client>,
    item: Option<&CmdqItemRef>,
    current: Option<&cmd_find_state>,
    flags: core::ffi::c_int,
    mut new_item: Option<&mut Option<CmdqItemRef>>,
) -> core::ffi::c_int {
    unsafe {
        let mut pi = cmd_parse_input::default();
        if let Some(new_item) = new_item.as_deref_mut() {
            *new_item = None;
        }
        log_debug(c"loading %s", fmt_args![path]);
        let mut file = match File::open(OsStr::from_bytes(path.to_bytes())) {
            Ok(file) => file,
            Err(err) => {
                let errno = err.raw_os_error().unwrap_or(0);
                if errno == ENOENT && flags & CMD_PARSE_QUIET != 0 {
                    return 0 as core::ffi::c_int;
                }
                cfg_add_cause(c"%s: %s", fmt_args![path, error_message(errno).as_c_str()]);
                return -(1 as core::ffi::c_int);
            }
        };
        let mut contents: Vec<u8> = Vec::new();
        let _ = file.read_to_end(&mut contents);
        pi.flags = flags;
        pi.file = Some(path.to_owned());
        pi.line = 1 as u_int;
        pi.item = item.map(CmdqItemRef::downgrade);
        pi.c = c.and_then(client_ref_of).map(|c| c.downgrade().into());
        let mut pr = cmd_parse_from_file(contents, Some(&mut pi));
        if pr.status as core::ffi::c_uint
            == CMD_PARSE_ERROR as core::ffi::c_int as core::ffi::c_uint
        {
            let error = pr.error.take().unwrap();
            cfg_add_cause(c"%s", fmt_args![error.as_ptr()]);
            return -(1 as core::ffi::c_int);
        }
        if flags & CMD_PARSE_PARSEONLY != 0 {
            let _ = pr.cmdlist.take();
            return 0 as core::ffi::c_int;
        }
        let state = match item {
            Some(anchor) => (anchor.state_ref()).copy_with_current(current),
            None => CmdqStateRef::create(None, None, 0 as core::ffi::c_int),
        };
        state.add_format(c"current_file", c"%s", fmt_args![pi.file()]);
        let cmdlist = pr.cmdlist.take().unwrap();
        let queued = cmdlist.queue_items(Some(&state));
        let last = match item {
            Some(anchor) => Some(anchor.insert_after(queued)),
            None => cmdq_append(None, queued),
        };
        if let Some(new_item) = new_item {
            *new_item = last;
        }
        0 as core::ffi::c_int
    }
}
pub unsafe fn load_cfg_from_buffer(
    buf: &[u8],
    path: &CStr,
    c: Option<&client>,
    item: Option<&CmdqItemRef>,
    current: Option<&cmd_find_state>,
    flags: core::ffi::c_int,
    mut new_item: Option<&mut Option<CmdqItemRef>>,
) -> core::ffi::c_int {
    unsafe {
        let mut pi = cmd_parse_input::default();
        if let Some(new_item) = new_item.as_deref_mut() {
            *new_item = None;
        }
        log_debug(c"loading %s", fmt_args![path]);
        pi.flags = flags;
        pi.file = Some(path.to_owned());
        pi.line = 1 as u_int;
        pi.item = item.map(CmdqItemRef::downgrade);
        pi.c = c.and_then(client_ref_of).map(|c| c.downgrade().into());
        let mut pr = cmd_parse_from_buffer(buf, Some(&mut pi));
        if pr.status as core::ffi::c_uint
            == CMD_PARSE_ERROR as core::ffi::c_int as core::ffi::c_uint
        {
            let error = pr.error.take().unwrap();
            cfg_add_cause(c"%s", fmt_args![error.as_ptr()]);
            return -(1 as core::ffi::c_int);
        }
        if flags & CMD_PARSE_PARSEONLY != 0 {
            let _ = pr.cmdlist.take();
            return 0 as core::ffi::c_int;
        }
        let state = match item {
            Some(anchor) => (anchor.state_ref()).copy_with_current(current),
            None => CmdqStateRef::create(None, None, 0 as core::ffi::c_int),
        };
        state.add_format(c"current_file", c"%s", fmt_args![pi.file()]);
        let cmdlist = pr.cmdlist.take().unwrap();
        let queued = cmdlist.queue_items(Some(&state));
        let last = match item {
            Some(anchor) => Some(anchor.insert_after(queued)),
            None => cmdq_append(None, queued),
        };
        if let Some(new_item) = new_item {
            *new_item = last;
        }
        0 as core::ffi::c_int
    }
}
pub fn cfg_add_cause(fmt: &CStr, args: &[FmtArg]) {
    let msg = format_alloc(fmt, args);
    with_config(|config| config.causes.push(msg));
}
pub unsafe fn cfg_print_causes(item: &cmdq_item) {
    unsafe {
        let mut c = item.client();
        for msg in take_causes() {
            if let Some(c) = c.as_mut()
                && c.flags() & CLIENT_CONTROL as uint64_t != 0
            {
                control_write(
                    c.as_client_mut(),
                    c"%%config-error %s",
                    fmt_args![msg.as_ptr()],
                );
            } else {
                item.print(c"%s", fmt_args![msg.as_ptr()]);
            }
        }
    }
}
pub unsafe fn cfg_show_causes(s: Option<&session>) {
    unsafe {
        let mut c = first_client();
        if with_config(|config| config.causes.is_empty()) {
            return;
        }
        if let Some(c) = c.as_mut()
            && c.flags() & CLIENT_CONTROL as uint64_t != 0
        {
            for msg in take_causes() {
                control_write(
                    c.as_client_mut(),
                    c"%%config-error %s",
                    fmt_args![msg.as_ptr()],
                );
            }
        } else {
            let first_session = sessions_first();
            let attached = c
                .as_ref()
                .and_then(|reference| reference.attached_session());
            let Some(s) = s
                .or_else(|| attached.as_ref().map(|session| session.as_session()))
                .or(first_session
                    .as_ref()
                    .map(|reference| reference.as_session()))
            else {
                return;
            };
            if crate::SessionAttachmentState::session_attached(s) == 0 as u_int {
                return;
            }
            let Some(current) = s.curw() else {
                return;
            };
            let Some(window) = current.window_handle().cloned() else {
                return;
            };
            let Some(mut pane) = window.active_pane() else {
                return;
            };
            let Some(active) = pane.get() else {
                return;
            };
            if active
                .modes()
                .first()
                .is_none_or(|current| current.mode() != WindowMode::View)
            {
                window_pane_set_mode(pane.as_pane_mut(), None, WindowMode::View, None, None);
            }
            for msg in take_causes() {
                window_copy_add(pane.as_pane_mut(), 0, c"%s", fmt_args![msg.as_ptr()]);
            }
        }
    }
}

#[cfg(test)]
#[path = "tests/test_cfg.rs"]
mod tests;

/// Parses and queues configuration input with the existing client context,
/// error handling and insertion order; returns the existing completion result.
///
/// # Safety
/// Run on the server thread without conflicting client, TTY, session or queue
/// payload access. Existing replacement/completion callbacks may run inline;
/// exclude conflicting callback state access. No payload reference escapes.
pub(crate) unsafe fn load_cfg_buffer_for_client(
    buf: &[u8],
    path: &CStr,
    c: Option<&ClientRef>,
    item: Option<&CmdqItemRef>,
    current: Option<&cmd_find_state>,
    flags: core::ffi::c_int,
    new_item: Option<&mut Option<CmdqItemRef>>,
) -> core::ffi::c_int {
    unsafe {
        load_cfg_from_buffer(
            buf,
            path,
            c.map(|c| c.as_client()),
            item,
            current,
            flags,
            new_item,
        )
    }
}

/// Delivers configuration causes using existing control-client and session fallbacks.
///
/// # Safety
/// Run on the server thread without conflicting client, TTY, session or queue
/// payload access. Existing replacement/completion callbacks may run inline;
/// exclude conflicting callback state access. No payload reference escapes.
pub(crate) unsafe fn cfg_show_causes_for_session(s: Option<&SessionRef>) {
    unsafe { cfg_show_causes(s.map(|s| s.as_session())) }
}

/// Reports whether initial configuration loading has completed.
///
/// Query on the server thread without concurrent configuration-state mutation.
pub(crate) fn configuration_finished() -> bool {
    with_config(|config| config.finished)
}

#[cfg(test)]
pub(crate) fn replace_configuration_finished(finished: bool) -> bool {
    with_config(|config| core::mem::replace(&mut config.finished, finished))
}
