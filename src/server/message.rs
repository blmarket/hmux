use super::client::{server_client_remove_pane, server_client_set_session};
use super::run::client_walk;
use super::run::marked_pane;
use crate::WindowPane;
use crate::ffi::{close, getpid, kill, utempter_remove_record};
use crate::fmt_args;
use crate::format::format_single;
use crate::grid::grid_default_cell;

use crate::notify::{notify_pane, notify_session_window};

use crate::pane_exit::PaneExitState;

use crate::resize::recalculate_sizes;
use crate::screen::Screen;
use crate::screen::{ScreenWriteCtx, screen_write_ctx_on_pane_base};

pub use crate::consts::{
    CLIENT_ALLREDRAWFLAGS, CLIENT_CONTROL, CLIENT_EXIT, CLIENT_NO_DETACH_ON_DESTROY,
    CLIENT_REDRAWBORDERS, CLIENT_REDRAWSTATUS, CLIENT_SUSPENDED, IMSG_HEADER_SIZE, MAX_IMSGSIZE,
    MODE_CURSOR, MSG_LOCK, PANE_REDRAW, PANE_STATUSDRAWN, PANE_STATUSREADY, SIGCHLD, SORT_NAME,
    TTYC_CLEAR, TTYC_SMCUP, WINLINK_ALERTFLAGS,
};
use crate::session::SESSIONS;
use crate::sort::{RustSortCriteria, SortCriteria};
use crate::terminfo::{TerminalCapabilities, tty_term_of};
use crate::tty::{tty_raw, tty_stop_tty};
pub use crate::types::*;
use crate::window::{
    WinlinkRef, window_count_panes, winlink_find_by_window, winlink_remove, winlink_stack_remove,
};
use crate::xmalloc::xasprintf;
use ::std::ffi::CString;

pub const TTYC_E3: tty_code_code = 36;

pub fn server_redraw_client(c: &mut client) {
    {
        c.flags = (c.flags as core::ffi::c_ulonglong | CLIENT_ALLREDRAWFLAGS) as uint64_t;
    }
}
pub fn server_status_client(c: &mut client) {
    {
        c.flags |= CLIENT_REDRAWSTATUS as uint64_t;
    }
}
pub unsafe fn server_redraw_session(s: &session) {
    unsafe {
        for mut c in client_walk() {
            if c.attached_session()
                .is_some_and(|session| session.points_to(s))
            {
                server_redraw_client(c.as_client_mut());
            }
        }
    }
}
pub unsafe fn server_redraw_session_group(s: &session) {
    unsafe {
        let grouped = crate::session::session_ref_of(s).and_then(|session| {
            session.with_group(|group| {
                for reference in group.sessions.iter().filter_map(SessionWeak::upgrade) {
                    server_redraw_session(reference.as_session());
                }
            })
        });
        if grouped.is_none() {
            server_redraw_session(s);
        }
    }
}
pub unsafe fn server_status_session(s: &session) {
    unsafe {
        for mut c in client_walk() {
            if c.attached_session()
                .is_some_and(|session| session.points_to(s))
            {
                server_status_client(c.as_client_mut());
            }
        }
    }
}
pub unsafe fn server_status_session_group(s: &session) {
    unsafe {
        let grouped = crate::session::session_ref_of(s).and_then(|session| {
            session.with_group(|group| {
                for reference in group.sessions.iter().filter_map(SessionWeak::upgrade) {
                    server_status_session(reference.as_session());
                }
            })
        });
        if grouped.is_none() {
            server_status_session(s);
        }
    }
}

pub fn server_lock() {
    unsafe {
        for mut c in client_walk() {
            if !c.attached_session().is_none() {
                server_lock_client(c.as_client_mut());
            }
        }
    }
}
pub unsafe fn server_lock_session(s: &mut session) {
    unsafe {
        for mut c in client_walk() {
            if c.attached_session()
                .is_some_and(|session| session.points_to(s))
            {
                server_lock_client(c.as_client_mut());
            }
        }
    }
}
pub unsafe fn server_lock_client(c: &mut client) {
    unsafe {
        if c.flags & CLIENT_CONTROL as uint64_t != 0 {
            return;
        }
        if c.flags & CLIENT_SUSPENDED as uint64_t != 0 {
            return;
        }
        let Some(session) = c.attached_session() else {
            return;
        };
        let cmd = session.options().string_ref(c"lock-command");
        if cmd.is_empty()
            || cmd.to_bytes_with_nul().len()
                > (MAX_IMSGSIZE as usize).wrapping_sub(IMSG_HEADER_SIZE)
        {
            return;
        }
        let tty = &mut c.tty;
        tty_stop_tty(tty);
        let smcup = tty_term_of(tty).string(TTYC_SMCUP).to_owned();
        let clear = tty_term_of(tty).string(TTYC_CLEAR).to_owned();
        let e3 = tty_term_of(tty).string(TTYC_E3).to_owned();
        tty_raw(tty, &smcup);
        tty_raw(tty, &clear);
        tty_raw(tty, &e3);
        c.flags |= CLIENT_SUSPENDED as uint64_t;
        ((*c).peer_handle()).send(MSG_LOCK, -(1 as core::ffi::c_int), cmd.to_bytes_with_nul());
    }
}

impl ClientRef {
    /// Requests locking of this retained client using its attached session's command.
    ///
    /// Control, suspended or unattached clients are unchanged, as are clients with
    /// an empty or oversized lock command. Otherwise stops the TTY, writes the
    /// existing alternate-screen and clearing sequences, marks the client suspended,
    /// then sends `MSG_LOCK` for asynchronous execution by the peer. Send failures
    /// remain unreported. Client registration is not required; size recalculation
    /// remains the caller's responsibility.
    ///
    /// # Safety
    /// Run on the initialized server thread. Exclude conflicting client, TTY, peer
    /// and attached-session/options access during this call. No payload borrow
    /// escapes; retaining this handle does not enforce exclusive payload access.
    pub(crate) unsafe fn lock(&mut self) {
        unsafe { server_lock_client(self.as_client_mut()) };
    }
}

pub unsafe fn server_kill_pane(pane: &RustWindowPaneWeak) {
    unsafe {
        let Some(owner) = pane.window() else {
            return;
        };
        if !owner
            .as_window()
            .panes
            .iter()
            .any(|candidate| candidate.downgrade().ptr_eq(pane))
        {
            return;
        }
        if window_count_panes(&owner.as_window(), 1) == 1 {
            (owner.clone()).kill(1);
            recalculate_sizes();
        } else {
            owner.unzoom_and_redraw();
            server_client_remove_pane(pane.as_pane());
            owner.close_pane_layout(
                &crate::window::window_pane_find_by_id(pane.id()).expect("the layout pane exists"),
            );
            owner.remove_pane(pane);
            owner.redraw();
        }
    }
}

pub fn server_renumber_all() {
    unsafe {
        for s_ref in SESSIONS.read().values() {
            s_ref.renumber_group_if_enabled();
        }
    }
}
pub(crate) unsafe fn server_link_window(
    source: &WinlinkRef,
    destination: &mut SessionRef,
    mut dstidx: core::ffi::c_int,
    killflag: core::ffi::c_int,
    mut selectflag: core::ffi::c_int,
    cause: &mut Option<CString>,
) -> core::ffi::c_int {
    unsafe {
        if !source.session().ptr_eq(destination) && (source.session()).shares_group(destination) {
            *cause = Some(c"sessions are grouped".to_owned());
            return -1;
        }
        let Some(window) = source.get().and_then(winlink::window_handle).cloned() else {
            *cause = Some(c"source window no longer exists".to_owned());
            return -1;
        };
        let replaced = if dstidx != -1 {
            destination
                .as_session()
                .windows
                .get(&dstidx)
                .and_then(|link| link.window_handle())
                .cloned()
        } else {
            None
        };
        if let Some(replaced) = replaced {
            if replaced.ptr_eq(&window) {
                *cause = Some(xasprintf(c"same index: %d", fmt_args![dstidx]));
                return -1;
            }
            if killflag != 0 {
                notify_session_window(c"window-unlinked", destination.as_session(), &replaced);
                let dst = destination.as_session_mut();
                let was_current = dst.curw_idx == Some(dstidx);
                if let Some(link) = dst.windows.get_mut(&dstidx) {
                    link.flags &= !WINLINK_ALERTFLAGS;
                    winlink_stack_remove(&mut dst.lastw, Some(link));
                }
                winlink_remove(&mut dst.windows, dstidx);
                if was_current {
                    selectflag = 1;
                    dst.curw_idx = None;
                }
            }
        }
        if dstidx == -1 {
            dstidx = (-1_i64 - destination.options().number(c"base-index")) as core::ffi::c_int;
        }
        let Some(index) = destination.attach(window, dstidx, cause) else {
            return -1;
        };
        if marked_pane.wl_idx == Some(source.index())
            && marked_pane
                .session()
                .is_some_and(|session| session.ptr_eq(source.session()))
        {
            marked_pane.set_winlink(
                destination
                    .as_session()
                    .windows
                    .get(&index)
                    .map(Box::as_ref),
            );
        }
        if selectflag != 0 {
            destination.select(index);
        }
        server_redraw_session_group(destination.as_session_mut());
        0
    }
}
pub unsafe fn server_unlink_window(s: &mut session, index: core::ffi::c_int) {
    unsafe {
        if crate::session::session_ref_of(s)
            .expect("session owner")
            .detach(index)
            != 0
        {
            server_destroy_session_group(s);
        } else {
            server_redraw_session_group(s);
        };
    }
}
pub unsafe fn server_destroy_pane(pane: &RustWindowPaneWeak, notify: core::ffi::c_int) {
    unsafe {
        let mut observed = pane.clone();
        let Some(owner) = pane.window() else {
            return;
        };
        let Some(wp) = observed.get_mut() else { return };
        let gc;

        let sx = wp.base().grid().sx;
        let sy = wp.base().grid().sy;
        if *wp.fd() != -(1 as core::ffi::c_int) {
            utempter_remove_record(*wp.fd());
            kill(getpid(), SIGCHLD);
            wp.event().free();
            *wp.event_mut() = Stream::NONE;
            close(*wp.fd());
            *wp.fd_mut() = -(1 as core::ffi::c_int);
        }
        let remain_on_exit: core::ffi::c_int =
            ((*wp).options_ref()).number(c"remain-on-exit") as core::ffi::c_int;
        if remain_on_exit != 0 as core::ffi::c_int && !*wp.flags() & PANE_STATUSREADY != 0 {
            return;
        }
        let current_block_31: u64;
        match remain_on_exit {
            2 => {
                let status = (*wp).exit_status();
                if status & 0x7f as core::ffi::c_int == 0 as core::ffi::c_int
                    && (status & 0xff00 as core::ffi::c_int) >> 8 as core::ffi::c_int
                        == 0 as core::ffi::c_int
                {
                    current_block_31 = 13550086250199790493;
                } else {
                    current_block_31 = 622960851218599991;
                }
            }
            1 | 3 => {
                current_block_31 = 622960851218599991;
            }
            _ => {
                current_block_31 = 13550086250199790493;
            }
        }
        match current_block_31 {
            13550086250199790493 => {}
            _ => {
                if *wp.flags() & PANE_STATUSDRAWN != 0 {
                    return;
                }
                *wp.flags_mut() |= PANE_STATUSDRAWN;
                let death_time = timeval::now();
                (*wp).set_death_time(death_time);
                if notify != 0 {
                    notify_pane(c"pane-died", Some(&*wp));
                }
                let s = (*wp).options_ref().string_ref(c"remain-on-exit-format");
                if !s.is_empty() {
                    let expanded = format_single(None, &s, None, None, None, Some(wp));
                    let mut writer = screen_write_ctx_on_pane_base(wp);
                    writer.scrollregion(0 as u_int, sy.wrapping_sub(1 as u_int));
                    writer.cursormove(
                        0 as core::ffi::c_int,
                        sy.wrapping_sub(1 as u_int) as core::ffi::c_int,
                        0 as core::ffi::c_int,
                    );
                    writer.linefeed(1 as core::ffi::c_int, 8 as u_int);
                    gc = grid_default_cell;
                    writer.format_draw(&gc, sx, expanded.as_bytes(), None, 0 as core::ffi::c_int);
                    writer.finish();
                }
                let pane_screen_mode = wp.base().mode() & !MODE_CURSOR;
                wp.base_mut().set_mode(pane_screen_mode);
                *wp.flags_mut() |= PANE_REDRAW;
                return;
            }
        }
        if notify != 0 {
            notify_pane(c"pane-exited", Some(&*wp));
        }
        owner.unzoom_and_redraw();
        server_client_remove_pane(wp);
        owner.close_pane_layout(
            &crate::window::window_pane_find_by_id(pane.id()).expect("the layout pane exists"),
        );
        owner.remove_pane(pane);
        if owner.as_window().panes.is_empty() {
            (owner.clone()).kill(1);
        } else {
            owner.redraw();
        };
    }
}
unsafe fn server_destroy_session_group(s: &mut session) {
    unsafe {
        let members =
            crate::session::session_ref_of(s).and_then(|session| session.group_walk_safe());
        if let Some(members) = members {
            for mut reference in members {
                let member = if reference.points_to(s) {
                    &mut *s
                } else {
                    reference.as_session_mut()
                };
                server_destroy_session(member);
                reference.destroy(1 as core::ffi::c_int, c"server_destroy_session_group");
            }
        } else {
            server_destroy_session(s);
            crate::session::session_ref_of(&*s)
                .expect("session owner")
                .destroy(1 as core::ffi::c_int, c"server_destroy_session_group");
        };
    }
}
unsafe fn server_find_session(
    s: &mut session,
    f: impl Fn(&session, Option<&session>) -> core::ffi::c_int,
) -> Option<SessionRef> {
    unsafe {
        let mut s_out: Option<SessionRef> = None;
        for s_loop in SESSIONS.read().values() {
            if !core::ptr::eq(s_loop.as_ptr(), s)
                && f(
                    s_loop.as_session(),
                    s_out.as_ref().map(|reference| reference.as_session()),
                ) != 0
            {
                s_out = Some(s_loop.clone());
            }
        }
        s_out
    }
}
fn server_newer_session(s_loop: &session, s_out: Option<&session>) -> core::ffi::c_int {
    let Some(s_out) = s_out else {
        return 1 as core::ffi::c_int;
    };
    if crate::SessionTimestampState::session_timestamps(s_loop)
        .activity
        .tv_sec
        == crate::SessionTimestampState::session_timestamps(s_out)
            .activity
            .tv_sec
    {
        (crate::SessionTimestampState::session_timestamps(s_loop)
            .activity
            .tv_usec
            > crate::SessionTimestampState::session_timestamps(s_out)
                .activity
                .tv_usec) as core::ffi::c_int
    } else {
        (crate::SessionTimestampState::session_timestamps(s_loop)
            .activity
            .tv_sec
            > crate::SessionTimestampState::session_timestamps(s_out)
                .activity
                .tv_sec) as core::ffi::c_int
    }
}
fn server_newer_detached_session(s_loop: &session, s_out: Option<&session>) -> core::ffi::c_int {
    if crate::SessionAttachmentState::session_attached(s_loop) != 0 {
        return 0 as core::ffi::c_int;
    }
    server_newer_session(s_loop, s_out)
}
pub unsafe fn server_destroy_session(s: &mut session) {
    unsafe {
        let mut s_new: Option<SessionRef> = None;
        let mut cs_new: Option<SessionRef> = None;
        let sort_crit = RustSortCriteria::new(SORT_NAME, false);

        let detach_on_destroy: core::ffi::c_int =
            (s.options_ref().clone()).number(c"detach-on-destroy") as core::ffi::c_int;
        if detach_on_destroy == 0 as core::ffi::c_int {
            s_new = server_find_session(s, server_newer_session);
        } else if detach_on_destroy == 2 as core::ffi::c_int {
            s_new = server_find_session(s, server_newer_detached_session);
        } else if detach_on_destroy == 3 as core::ffi::c_int {
            s_new = crate::session::session_ref_of(&*s)
                .and_then(|session| session.previous_session(&sort_crit));
        } else if detach_on_destroy == 4 as core::ffi::c_int {
            s_new = crate::session::session_ref_of(&*s)
                .and_then(|session| session.next_session(&sort_crit));
        }
        if s_new.as_ref().is_some_and(|found| found.points_to(s)) {
            s_new = None;
        }
        if s_new.is_none()
            && (detach_on_destroy == 1 as core::ffi::c_int
                || detach_on_destroy == 2 as core::ffi::c_int)
        {
            cs_new = server_find_session(s, server_newer_session);
        }
        for mut c in client_walk() {
            if c.attached_session()
                .is_some_and(|session| session.points_to(s))
            {
                let mut use_s = s_new.as_ref();
                if use_s.is_none()
                    && c.flags() as core::ffi::c_ulonglong & CLIENT_NO_DETACH_ON_DESTROY != 0
                {
                    use_s = cs_new.as_ref();
                }
                c.set_attached_session(None);
                c.as_client_mut().last_session = None;
                server_client_set_session(c.as_client_mut(), use_s);
                if use_s.is_none() {
                    *c.flags_mut() |= CLIENT_EXIT as uint64_t;
                }
            }
        }
        recalculate_sizes();
    }
}
pub fn server_check_unattached() {
    unsafe {
        for mut owner in SESSIONS.walk_safe() {
            if owner.attached() != 0 {
                continue;
            }
            let destroy = match owner.options().number(c"destroy-unattached") {
                0 => false,
                2 => owner
                    .with_group(crate::session::session_group_count)
                    .is_some_and(|count| count > 1),
                3 => owner
                    .with_group(crate::session::session_group_count)
                    .is_none_or(|count| count != 1),
                _ => true,
            };
            if destroy {
                let s = owner.as_session_mut();
                server_destroy_session(s);
                owner.destroy(1, c"server_check_unattached");
            }
        }
    }
}

#[cfg(test)]
#[path = "../tests/test_server_message_focused.rs"]
mod focused_tests;

impl WindowRef {
    pub unsafe fn redraw(&self) {
        let w = self;

        unsafe {
            for mut c in client_walk() {
                let session = c.attached_session();
                let current_window = session
                    .as_ref()
                    .and_then(|session| session.curw())
                    .and_then(|link| link.window());
                if current_window.is_some_and(|window| window.ptr_eq(w)) {
                    server_redraw_client(c.as_client_mut());
                }
            }
        }
    }
    pub unsafe fn redraw_borders(&self) {
        let w = self;

        unsafe {
            for mut c in client_walk() {
                let session = c.attached_session();
                let current_window = session
                    .as_ref()
                    .and_then(|session| session.curw())
                    .and_then(|link| link.window());
                if current_window.is_some_and(|window| window.ptr_eq(w)) {
                    *c.flags_mut() |= CLIENT_REDRAWBORDERS as uint64_t;
                }
            }
        }
    }
    pub unsafe fn redraw_status(&self) {
        let w = self;

        unsafe {
            for session in SESSIONS.read().values() {
                if w.winlinks().any(|link| link.session().ptr_eq(session)) {
                    server_status_session(session.as_session());
                }
            }
        }
    }
    pub unsafe fn kill(self, renumber: core::ffi::c_int) {
        let window = self;

        unsafe {
            for mut session in SESSIONS.walk_safe() {
                if !session.has(&window) {
                    continue;
                }
                window.unzoom_and_redraw();
                loop {
                    let Some(index) =
                        winlink_find_by_window(&session.as_session().windows, &window.as_window())
                            .map(|link| link.idx)
                    else {
                        break;
                    };
                    if session.detach(index) != 0 {
                        server_destroy_session_group(session.as_session_mut());
                        break;
                    }
                    server_redraw_session_group(session.as_session_mut());
                }
                if renumber != 0 {
                    session.renumber_group_if_enabled();
                }
            }
            recalculate_sizes();
        }
    }
    pub unsafe fn unzoom_and_redraw(&self) {
        let owner = self;

        unsafe {
            if owner.unzoom(1 as core::ffi::c_int) == 0 as core::ffi::c_int {
                owner.redraw();
            }
        }
    }
}

impl SessionRef {
    /// Requests locking of registered clients attached to this retained session.
    ///
    /// The existing traversal retains client owners and checks each client's current
    /// attachment by allocation identity. The session need not be registered.
    /// Each matching client follows the existing lock guards and TTY/peer ordering;
    /// size recalculation remains the caller's responsibility.
    ///
    /// # Safety
    /// Run on the initialized server thread. Exclude conflicting session/options
    /// and registered-client attachment access, plus matching-client, TTY or peer
    /// access during this call. No payload borrow escapes; the mutable handle does
    /// not enforce exclusive payload access.
    pub(crate) unsafe fn lock(&mut self) {
        unsafe { server_lock_session(self.as_session_mut()) };
    }

    pub(crate) unsafe fn renumber_group_if_enabled(&self) {
        let s_ref = self;

        unsafe {
            if s_ref.options().number(c"renumber-windows") != 0
                && s_ref
                    .with_group(|group| {
                        for member in group.sessions.iter().filter_map(SessionWeak::upgrade) {
                            member.renumber_windows();
                        }
                    })
                    .is_none()
            {
                s_ref.clone().renumber_windows();
            }
        }
    }
}

#[cfg(test)]
pub use crate::consts::CLIENT_REDRAWWINDOW;
