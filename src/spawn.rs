use crate::WindowPane as _;
use crate::compat::error_message;

use crate::cmd::CmdqItemRef;
use crate::cmd::cmd_log_argv;

use crate::compat::fdforkpty;
use crate::compat::systemd_move_to_new_cgroup;
use crate::environ::EnvironmentStore;
use crate::environ::{environment_for_session, log_environment, push_environment_to_process};
use crate::ffi::{
    __errno_location, _exit, chdir, close, closefrom, execl, execvp, getcwd, getpid, kill,
    sigfillset, sigprocmask, tcgetattr, tcsetattr, utempter_add_record,
};
use crate::fmt_args;
use crate::format::{format_create_for_client, format_defaults_for_handles, format_expand};

use crate::layout::LayoutCellPath;
use crate::log::{log_close, log_debug};
use crate::names::default_window_name;
use crate::notify::{notify_session_window, notify_window};

use crate::pane_command::PaneCommandState;
use crate::proc::proc_clear_signals;
use crate::resize::default_window_size;
use crate::screen::Screen;
use crate::screen::screen_reinit;
use crate::server::server_process;
use crate::server::{server_client_get_cwd, server_client_remove_pane};

pub use crate::consts::{
    _PATH_BSHELL, CLIENT_EXIT_DETACH, CLIENT_EXIT_RETURN, CLIENT_EXIT_SHUTDOWN,
    LAYOUT_CELL_FLOATING, LAYOUT_LEFTRIGHT, LAYOUT_TOPBOTTOM, LAYOUT_WINDOWPANE, MODE_CRLF,
    MODE_CURSOR, MSG_COMMAND, MSG_DETACH, MSG_DETACHKILL, MSG_EXEC, MSG_EXIT, MSG_EXITED,
    MSG_EXITING, MSG_FLAGS, MSG_IDENTIFY_CLIENTPID, MSG_IDENTIFY_CWD, MSG_IDENTIFY_DONE,
    MSG_IDENTIFY_ENVIRON, MSG_IDENTIFY_FEATURES, MSG_IDENTIFY_FLAGS, MSG_IDENTIFY_LONGFLAGS,
    MSG_IDENTIFY_OLDCWD, MSG_IDENTIFY_STDIN, MSG_IDENTIFY_STDOUT, MSG_IDENTIFY_TERM,
    MSG_IDENTIFY_TERMINFO, MSG_IDENTIFY_TTYNAME, MSG_LOCK, MSG_OLDSTDERR, MSG_OLDSTDIN,
    MSG_OLDSTDOUT, MSG_READ, MSG_READ_CANCEL, MSG_READ_DONE, MSG_READ_OPEN, MSG_READY, MSG_RESIZE,
    MSG_SHELL, MSG_SHUTDOWN, MSG_SUSPEND, MSG_UNLOCK, MSG_VERSION, MSG_WAKEUP, MSG_WRITE,
    MSG_WRITE_CLOSE, MSG_WRITE_OPEN, MSG_WRITE_READY, PANE_EMPTY, PANE_EXITED, PANE_LINES_DOUBLE,
    PANE_LINES_HEAVY, PANE_LINES_NUMBER, PANE_LINES_SIMPLE, PANE_LINES_SINGLE, PANE_LINES_SPACES,
    PANE_STATUSDRAWN, PANE_STATUSREADY, PROGRESS_BAR_ERROR, PROGRESS_BAR_HIDDEN,
    PROGRESS_BAR_INDETERMINATE, PROGRESS_BAR_NORMAL, PROGRESS_BAR_PAUSED, PROMPT_COMMAND,
    PROMPT_ENTRY, PROMPT_TYPE_COMMAND, PROMPT_TYPE_INVALID, PROMPT_TYPE_SEARCH, PROMPT_TYPE_TARGET,
    PROMPT_TYPE_WINDOW_TARGET, SCREEN_CURSOR_BAR, SCREEN_CURSOR_BLOCK, SCREEN_CURSOR_DEFAULT,
    SCREEN_CURSOR_UNDERLINE, SIG_BLOCK, SIG_SETMASK, SIGCHLD, SPAWN_DETACHED, SPAWN_EMPTY,
    SPAWN_FLOATING, SPAWN_KILL, SPAWN_RESPAWN, SPAWN_ZOOM, STDERR_FILENO, STDIN_FILENO,
    STYLE_ALIGN_ABSOLUTE_CENTRE, STYLE_ALIGN_CENTRE, STYLE_ALIGN_DEFAULT, STYLE_ALIGN_LEFT,
    STYLE_ALIGN_RIGHT, STYLE_DEFAULT_BASE, STYLE_DEFAULT_POP, STYLE_DEFAULT_PUSH,
    STYLE_DEFAULT_SET, STYLE_LIST_FOCUS, STYLE_LIST_LEFT_MARKER, STYLE_LIST_OFF, STYLE_LIST_ON,
    STYLE_LIST_RIGHT_MARKER, STYLE_RANGE_CONTROL, STYLE_RANGE_LEFT, STYLE_RANGE_NONE,
    STYLE_RANGE_PANE, STYLE_RANGE_RIGHT, STYLE_RANGE_SESSION, STYLE_RANGE_USER, STYLE_RANGE_WINDOW,
    TCSANOW, THEME_DARK, THEME_LIGHT, THEME_UNKNOWN, VERASE, WINDOW_ZOOMED, WINLINK_ACTIVITY,
    WINLINK_ALERTFLAGS, WINLINK_BELL, WINLINK_SILENCE,
};
use crate::tmux::{checkshell, find_home};
use crate::tmux::{global_options, ptm_fd};
pub use crate::types::*;
use crate::window::{WinlinkRef, window_set_latest};
use crate::window::{
    window_add_pane, window_panes_insert_head, winlink_remove, winlink_stack_remove,
};
use crate::xmalloc::xasprintf;
use crate::{CommandTextCodec, RustCommandTextCodec};
use ::core::ffi::CStr;
use ::std::ffi::CString;

pub const IUTF8: core::ffi::c_int = 0o40000 as core::ffi::c_int;

pub const _PATH_DEFPATH: &CStr = c"/usr/bin:/bin";

pub const SPAWN_NONOTIFY: core::ffi::c_int = 0x10 as core::ffi::c_int;

pub(crate) fn spawn_shell_name(shell: &CStr) -> &CStr {
    let bytes = shell.to_bytes();
    match bytes.iter().rposition(|&byte| byte == b'/') {
        Some(at) if at + 1 < bytes.len() => {
            CStr::from_bytes_with_nul(&shell.to_bytes_with_nul()[at + 1..])
                .expect("a shell name ends with its path")
        }
        _ => shell,
    }
}

fn spawn_log(from: &CStr, sc: &spawn_context) {
    {
        let s = sc.s.as_ref().expect("a spawn context has a session owner");
        let wl = sc.wl;
        let item = spawn_item(sc);
        log_debug(
            c"%s: %s, flags=%#x",
            fmt_args![from, (item.read()).name(), sc.flags],
        );
        let tmp = match (wl, sc.wp0.as_ref().map(|pane| pane.id())) {
            (Some(wl), Some(id)) => xasprintf(c"wl=%d wp0=%%%u", fmt_args![wl, id]),
            (Some(wl), None) => xasprintf(c"wl=%d wp0=none", fmt_args![wl]),
            (None, Some(id)) => xasprintf(c"wl=none wp0=%%%u", fmt_args![id]),
            (None, None) => xasprintf(c"wl=none wp0=none", fmt_args![]),
        };
        log_debug(
            c"%s: s=$%u %s idx=%d",
            fmt_args![from, s.id(), tmp.as_c_str(), sc.idx],
        );
        log_debug(c"%s: name=%s", fmt_args![from, sc.name.unwrap_or(c"none")]);
    }
}
/// The queue item the spawn was asked from, which is waiting on it.
fn spawn_item(sc: &spawn_context) -> CmdqItemRef {
    sc.item
        .as_ref()
        .and_then(|item| item.upgrade())
        .expect("a spawn context has a live queue item")
}

/// The client the spawn was asked for, or none.
fn spawn_client(sc: &spawn_context) -> Option<ClientRef> {
    sc.tc.as_ref().and_then(ClientWeak::upgrade)
}

pub(crate) unsafe fn spawn_window(
    sc: &mut spawn_context,
    cause: &mut Option<CString>,
) -> Option<WinlinkRef> {
    unsafe {
        let mut session = sc.s.clone().expect("a spawn context has a session owner");
        let window;
        let index;
        spawn_log(c"spawn_window", sc);
        if sc.flags & SPAWN_RESPAWN != 0 {
            let Some(existing) = sc
                .wl
                .filter(|index| session.as_session().windows.contains_key(index))
            else {
                *cause = Some(c"window no longer exists".to_owned());
                return None;
            };
            index = existing;
            window = session
                .as_session()
                .windows
                .get(&index)
                .and_then(|link| link.window_handle())
                .expect("a link owns its window")
                .clone();
            if sc.flags & SPAWN_KILL == 0
                && window
                    .as_window()
                    .panes
                    .iter()
                    .any(|pane| pane.as_pane().process_active())
            {
                *cause = Some(xasprintf(
                    c"window %s:%d still active",
                    fmt_args![session.name().as_deref(), index],
                ));
                return None;
            }
            let kept = window.as_window_mut().panes.remove(0);
            window.free_layout();
            window.destroy_panes();
            let retained_pane = window_panes_insert_head(&mut window.as_window_mut(), kept);
            let pane_id = retained_pane.id();
            sc.wp0 = Some(retained_pane);
            let size = window.dimensions().size;
            let mut payload = window.as_window_mut();
            let pane = payload
                .panes
                .iter_mut()
                .find(|pane| pane.pane_id() == pane_id)
                .expect("the retained pane belongs to its window")
                .as_pane_mut();
            pane.resize(size);
            drop(payload);
            window.init_layout(
                &crate::window::window_pane_find_by_id(pane_id).expect("the layout pane exists"),
            );
            window.as_window_mut().active = None;
            window.set_active_pane(
                &crate::window::window_pane_find_by_id(pane_id).expect("the selected pane exists"),
                0,
            );
        } else {
            let mut requested = sc.idx;
            if requested != -1 && session.as_session().windows.contains_key(&requested) {
                if sc.flags & SPAWN_KILL == 0 {
                    *cause = Some(xasprintf(c"index %d in use", fmt_args![requested]));
                    return None;
                }
                let was_current = session.curw().is_some_and(|link| link.index() == requested);
                let replaced = {
                    let link = session
                        .as_session_mut()
                        .windows
                        .get_mut(&requested)
                        .unwrap();
                    link.flags &= !WINLINK_ALERTFLAGS;
                    link.window_handle()
                        .expect("a link owns its window")
                        .clone()
                };
                notify_session_window(c"window-unlinked", session.as_session(), &replaced);
                drop(replaced);
                {
                    let s = session.as_session_mut();
                    winlink_stack_remove(
                        &mut s.lastw,
                        s.windows.get_mut(&requested).map(Box::as_mut),
                    );
                    winlink_remove(&mut s.windows, requested);
                    if was_current {
                        s.curw = None;
                        sc.flags &= !SPAWN_DETACHED;
                    }
                }
            }
            if requested == -1 {
                requested = (-1 - session.options().number(c"base-index")) as core::ffi::c_int;
            }
            sc.wl =
                crate::window::winlink_insert(&mut session.as_session_mut().windows, requested)
                    .map(|link| link.idx);
            let Some(created) = sc.wl else {
                *cause = Some(xasprintf(c"couldn't add window %d", fmt_args![requested]));
                return None;
            };
            index = created;
            let client = spawn_client(sc);
            let (sx, sy, xpixel, ypixel) = default_window_size(
                client.as_ref().map(|client| client.as_client()),
                session.as_session(),
                None,
                -1,
            );
            window = WindowRef::create(sx, sy, xpixel, ypixel);
            if session.curw().is_none() {
                session.as_session_mut().curw = Some(index);
            }
            window_set_latest(
                &mut window.as_window_mut(),
                client.as_ref().map(|client| client.as_client()),
            );
            let observer = session.downgrade();
            let link = session
                .as_session_mut()
                .windows
                .get_mut(&index)
                .expect("the new link remains registered");
            link.session = Some(observer);
            link.set_window(window.clone());
        }
        sc.flags |= SPAWN_NONOTIFY;
        if spawn_pane(sc, None, cause).is_none() {
            if sc.flags & SPAWN_RESPAWN == 0 {
                let s = session.as_session_mut();
                if s.curw().is_some_and(|link| link.idx == index) {
                    s.curw = None;
                }
                winlink_remove(&mut s.windows, index);
                sc.wl = None;
            }
            return None;
        }
        if sc.flags & SPAWN_RESPAWN == 0 {
            if sc.name.is_none_or(|name| name.is_empty()) {
                let name = default_window_name(&window);
                window.set_window_name(Some(&name));
            } else {
                window.set_window_name(sc.name);
                window.options().set_number(c"automatic-rename", 0);
            }
        }
        if sc.flags & SPAWN_DETACHED == 0 {
            session.select(index);
        }
        if sc.flags & SPAWN_RESPAWN == 0 {
            notify_session_window(c"window-linked", session.as_session(), &window);
        }
        session.synchronize_group_from();
        WinlinkRef::new(session, index)
    }
}

pub(crate) unsafe fn spawn_pane(
    sc: &mut spawn_context,
    lc: Option<&LayoutCellPath>,
    cause: &mut Option<CString>,
) -> Option<RustWindowPaneWeak> {
    unsafe {
        let item_ref = spawn_item(sc);
        let (mut c, target_session) =
            item_ref.with_item(|item| (item.client(), item.target.session()));

        let session = sc.s.clone().expect("a spawn context has a session owner");
        let Some(link) = sc
            .wl
            .and_then(|index| WinlinkRef::new(session.clone(), index))
        else {
            *cause = Some(c"window no longer exists".to_owned());
            return None;
        };
        let window = link
            .get()
            .and_then(winlink::window_handle)
            .expect("a link owns its window")
            .clone();
        let previous = sc.wp0.as_ref().filter(|pane| pane.is_alive()).cloned();
        if previous.is_none() && (sc.wp0.is_some() || sc.flags & SPAWN_RESPAWN != 0) {
            *cause = Some(c"pane no longer exists".to_owned());
            return None;
        }
        let mut cwd: Option<CString>;
        let mut path: [core::ffi::c_char; 4096] = [0; 4096];
        let home_path = find_home();
        let home = home_path.as_deref();
        let mut actual_cwd: Option<&CStr> = None;
        let mut now: termios = core::mem::zeroed();

        let mut ws: winsize;
        let mut set: sigset_t = __sigset_t { __val: [0; 16] };
        let mut oldset: sigset_t = __sigset_t { __val: [0; 16] };
        let key: key_code;
        spawn_log(c"spawn_pane", sc);
        if let Some(sc_cwd) = sc.cwd {
            let expanded = item_ref.with_item(|item| {
                let mut ft = format_create_for_client(item.client().as_ref(), Some(item), 0, 0);
                format_defaults_for_handles(
                    &mut ft,
                    c.as_ref(),
                    target_session.as_ref(),
                    None,
                    None,
                );
                format_expand(&mut ft, sc_cwd)
            });
            if !expanded.as_bytes().starts_with(b"/") {
                cwd = Some(xasprintf(
                    c"%s%s%s",
                    fmt_args![
                        server_client_get_cwd(
                            c.as_ref().map(|client| client.as_client()),
                            target_session.as_ref().map(|session| session.as_session())
                        )
                        .as_c_str(),
                        if expanded.is_empty() { c"" } else { c"/" },
                        expanded.as_c_str()
                    ],
                ));
            } else {
                cwd = Some(expanded);
            }
        } else if !sc.flags & SPAWN_RESPAWN != 0 {
            cwd = Some(server_client_get_cwd(
                c.as_ref().map(|client| client.as_client()),
                target_session.as_ref().map(|session| session.as_session()),
            ));
        } else {
            cwd = None;
        }
        let hlimit: u_int = (session.options()).number(c"history-limit") as u_int;
        let new_id;
        let mut new_pane;
        if sc.flags & SPAWN_RESPAWN != 0 {
            new_pane = previous.expect("respawning requires an existing pane");
            let previous = &mut new_pane;
            new_id = previous.id();
            if previous.get().expect("the respawn pane is present").process_active()
                && sc.flags & SPAWN_KILL == 0
            {
                let index = crate::window::window_pane_index(
                    &window.as_window(),
                    previous.get().expect("the respawn pane exists"),
                )
                .1;
                *cause = Some(xasprintf(
                    c"pane %s:%d.%u still active",
                    fmt_args![session.name().as_deref(), link.index(), index],
                ));
                return None;
            }
            let pane = previous.get_mut().expect("the respawn pane is present");
            pane.prepare_respawn();
        } else {
            if let Some(lc) = lc {
                new_pane = window_add_pane(
                    &mut window.as_window_mut(),
                    sc.wp0.as_ref(),
                    hlimit,
                    sc.flags,
                );
                new_id = new_pane.id();
                window.assign_pane_layout(
                    lc,
                    &new_pane,
                    (sc.flags & SPAWN_ZOOM != 0) as core::ffi::c_int,
                );
            } else {
                new_pane = window_add_pane(&mut window.as_window_mut(), None, hlimit, sc.flags);
                new_id = new_pane.id();
                window.init_layout(&new_pane);
            }
            if sc.flags & SPAWN_FLOATING != 0 {
                let mut payload = window.as_window_mut();
                let root = payload
                    .layout_root
                    .as_deref_mut()
                    .expect("the spawned pane has a layout");
                let path = LayoutCellPath::for_pane(
                    root,
                    &crate::window::window_pane_find_by_id(new_id)
                        .expect("the pane allocation exists"),
                )
                .expect("the spawned pane has a cell");
                path.get_mut(root).unwrap().flags |= LAYOUT_CELL_FLOATING;
            }
        }
        let pixels = window.dimensions().pixels;
        let _window_id = window.window_id();
        let new_wp = new_pane.get_mut().expect("the spawned pane is present");
        let argv = if sc.argv.is_empty() && sc.flags & SPAWN_RESPAWN == 0 {
            let command = session.options().string_ref(c"default-command");
            if !command.is_empty() {
                vec![command.as_ref().to_owned()]
            } else {
                Vec::new()
            }
        } else {
            core::mem::take(&mut sc.argv)
        };
        let mut pane_command = new_wp.pane_command();
        if cwd.is_some() {
            pane_command.cwd = cwd.take();
        }
        if !argv.is_empty() {
            pane_command.argv = argv;
        }
        let mut child = environment_for_session(Some(session.as_session()), 0 as core::ffi::c_int);
        if let Some(environ) = sc.environ.as_deref() {
            child.copy_from(environ);
        }
        let tmux_pane = xasprintf(c"%%%u", fmt_args![new_id]);
        child.set(c"TMUX_PANE", 0, &tmux_pane);
        if let Some(client) = c.as_mut()
            && client.attached_session().is_none()
        {
            let path = client
                .environ_mut()
                .find(c"PATH")
                .and_then(|entry| entry.value);
            if let Some(path) = path {
                child.set(c"PATH", 0, path);
            }
        }
        if child.find(c"PATH").is_none() {
            child.set(c"PATH", 0, _PATH_DEFPATH);
        }
        if !sc.flags & SPAWN_RESPAWN != 0 {
            let configured = session.options().string_ref(c"default-shell");
            let shell = if checkshell(Some(&configured)) == 0 {
                _PATH_BSHELL
            } else {
                &configured
            };
            pane_command.shell = Some(shell.to_owned());
        }
        new_wp.set_pane_command(&pane_command);
        if let Some(shell) = pane_command.shell.as_deref() {
            child.set(c"SHELL", 0, shell);
        }
        log_debug(
            c"%s: shell=%s",
            fmt_args![c"spawn_pane", pane_command.shell.as_deref()],
        );
        if !pane_command.argv.is_empty() {
            let command = RustCommandTextCodec.stringify(&pane_command.argv);
            log_debug(c"%s: cmd=%s", fmt_args![c"spawn_pane", command.as_c_str()]);
        }
        log_debug(
            c"%s: cwd=%s",
            fmt_args![c"spawn_pane", pane_command.cwd.as_deref()],
        );
        cmd_log_argv(&pane_command.argv, c"%s", fmt_args![c"spawn_pane"]);
        log_environment(&child, c"%s: environment ", fmt_args![c"spawn_pane"]);
        ws = core::mem::zeroed();
        ws.ws_col = new_wp.base().grid().sx as core::ffi::c_ushort;
        ws.ws_row = new_wp.base().grid().sy as core::ffi::c_ushort;
        ws.ws_xpixel = pixels.width.wrapping_mul(ws.ws_col as u_int) as core::ffi::c_ushort;
        ws.ws_ypixel = pixels.height.wrapping_mul(ws.ws_row as u_int) as core::ffi::c_ushort;
        sigfillset(&raw mut set);
        sigprocmask(SIG_BLOCK, &raw mut set, &raw mut oldset);
        if sc.flags & SPAWN_EMPTY != 0 {
            *new_wp.flags_mut() |= PANE_EMPTY;
            let pane_screen_mode = (new_wp.base().mode() & !MODE_CURSOR) | MODE_CRLF;
            new_wp.base_mut().set_mode(pane_screen_mode);
        } else {
            if !getcwd(path.as_mut_ptr(), path.len()).is_null() {
                if let Some(cwd) = pane_command.cwd.as_deref()
                    && chdir(cwd.as_ptr()) == 0
                {
                    actual_cwd = Some(cwd);
                } else if let Some(home) = home
                    && chdir(home.as_ptr()) == 0
                {
                    actual_cwd = Some(home);
                } else if chdir(c"/".as_ptr()) == 0 {
                    actual_cwd = Some(c"/");
                }
            }
            let pid = new_wp.fork_process(ptm_fd.get(), &ws);
            if pid == -(1 as core::ffi::c_int) {
                *cause = Some(xasprintf(
                    c"fork failed: %s",
                    fmt_args![error_message(*__errno_location()).as_c_str()],
                ));
                if !sc.flags & SPAWN_RESPAWN != 0 {
                    server_client_remove_pane(new_wp);
                    let pane_id = new_id;
                    window.close_pane_layout(
                        &crate::window::window_pane_find_by_id(pane_id)
                            .expect("the layout pane exists"),
                    );
                    window.remove_pane(&new_pane);
                }
                sigprocmask(
                    SIG_SETMASK,
                    &raw mut oldset,
                    core::ptr::null_mut::<sigset_t>(),
                );
                return None;
            }
            if pid != 0 as core::ffi::c_int {
                if actual_cwd.is_some()
                    && chdir(path.as_ptr()) != 0
                    && home.is_none_or(|home| chdir(home.as_ptr()) != 0)
                {
                    chdir(c"/".as_ptr());
                }
            } else {
                let mut cgroup_cause = None;
                if systemd_move_to_new_cgroup(&mut cgroup_cause) < 0 as core::ffi::c_int
                    && let Some(cgroup_cause) = cgroup_cause
                {
                    log_debug(
                        c"%s: moving pane to new cgroup failed: %s",
                        fmt_args![c"spawn_pane", cgroup_cause.as_c_str()],
                    );
                }
                if let Some(actual_cwd) = actual_cwd {
                    child.set(c"PWD", 0, actual_cwd);
                }
                if tcgetattr(STDIN_FILENO, &raw mut now) != 0 as core::ffi::c_int {
                    _exit(1 as core::ffi::c_int);
                }
                if let Some(tio) = &session.tio() {
                    now.c_cc = tio.c_cc;
                }
                key = (global_options
                    .get()
                    .as_ref()
                    .expect("global options are initialized"))
                .number(c"backspace") as key_code;
                if key >= 0x7f as key_code {
                    now.c_cc[VERASE as usize] = '\u{7f}' as i32 as cc_t;
                } else {
                    now.c_cc[VERASE as usize] = key as cc_t;
                }
                now.c_iflag |= IUTF8 as tcflag_t;
                if tcsetattr(STDIN_FILENO, TCSANOW, &raw mut now) != 0 as core::ffi::c_int {
                    _exit(1 as core::ffi::c_int);
                }
                proc_clear_signals(
                    &mut server_process
                        .get()
                        .as_ref()
                        .expect("server process is initialized")
                        .borrow_mut(),
                    1 as core::ffi::c_int,
                );
                closefrom(STDERR_FILENO + 1 as core::ffi::c_int);
                sigprocmask(
                    SIG_SETMASK,
                    &raw mut oldset,
                    core::ptr::null_mut::<sigset_t>(),
                );
                log_close();
                push_environment_to_process(&child);
                if pane_command.argv.len() > 1 {
                    let argvp: Vec<*mut core::ffi::c_char> = pane_command
                        .argv
                        .iter()
                        .map(|arg| arg.as_ptr() as *mut core::ffi::c_char)
                        .chain(core::iter::once(core::ptr::null_mut()))
                        .collect();
                    execvp(argvp[0], argvp.as_ptr());
                    _exit(1 as core::ffi::c_int);
                }
                let shell = pane_command.shell.as_deref().expect("a pane has a shell");
                let shell_name = spawn_shell_name(shell);
                if pane_command.argv.len() == 1 {
                    execl(
                        shell.as_ptr(),
                        shell_name.as_ptr(),
                        c"-c".as_ptr(),
                        pane_command.argv[0].as_ptr(),
                        core::ptr::null::<core::ffi::c_char>(),
                    );
                    _exit(1 as core::ffi::c_int);
                }
                let argv0 = xasprintf(c"-%s", fmt_args![shell_name]);
                execl(
                    shell.as_ptr(),
                    argv0.as_ptr(),
                    core::ptr::null::<core::ffi::c_char>(),
                );
                _exit(1 as core::ffi::c_int);
            }
        }
        new_wp.activate_spawned_process(&oldset);
        if sc.flags & SPAWN_RESPAWN != 0 {
            return Some(new_pane);
        }
        let has_active = window.active_pane_id().is_some_and(|id| {
            window
                .as_window()
                .panes
                .iter()
                .any(|pane| pane.pane_id() == id)
        });
        if sc.flags & SPAWN_DETACHED == 0 || !has_active {
            window.set_active_pane(
                &crate::window::window_pane_find_by_id(new_id).expect("the selected pane exists"),
                (sc.flags & SPAWN_NONOTIFY == 0) as core::ffi::c_int,
            );
        }
        if sc.flags & SPAWN_NONOTIFY == 0 {
            notify_window(c"window-layout-changed", Some(&window));
        }
        Some(new_pane)
    }
}
