use super::*;
use crate::cmd::cmd_find_from_winlink;
use crate::cmd::cmdq_get_current;
use crate::session::session_get_curw;
use crate::tests::test_fixtures::{
    Item, Pane, Registry, Session, Window, ensure_reactor, globals, link, unlink,
};
use crate::window::{winlink_count, winlink_find_by_index};
use ::core::ffi::c_int;

/// Where the fixture windows' ids start, far above anything production
/// hands out from its own counters.
const WINDOW_ID_BASE: u_int = 810_000;

/// Where the fixture panes' ids start.
const PANE_ID_BASE: u_int = 910_000;

/// One registered session holding registered windows, the way
/// `cmd_find_target` walks them. Winlinks the fixture linked are unlinked
/// again on the way out; a run that frees or makes winlinks hands the
/// cleanup list back with [`Chain::tracking`].
struct Chain {
    registry: Registry,
    session: Session,
    windows: Vec<Window>,
    panes: Vec<Pane>,
    tracked: Vec<*mut winlink>,
}

impl Chain {
    fn new(name: &str) -> Chain {
        let mut c = Chain {
            registry: Registry::new(),
            session: Session::new(0, name),
            windows: Vec::new(),
            panes: Vec::new(),
            tracked: Vec::new(),
        };
        let s = &raw mut c.session;
        unsafe { c.registry.add_session(&mut *s) };
        c
    }

    fn add_window(&mut self, idx: c_int) -> (*mut winlink, *mut window) {
        let mut w = Window::new(
            WINDOW_ID_BASE + self.windows.len() as u_int * 11,
            "chain",
            80,
            24,
        );
        let mut p = Pane::new(PANE_ID_BASE + self.panes.len() as u_int, 80, 24, 100);
        w.add_pane(&mut p);
        self.registry.add_window(&mut w);
        self.registry.add_pane(&mut p);
        let wptr = w.ptr();
        let wl = link(&mut self.session, &mut w, idx);
        self.tracked.push(wl);
        self.windows.push(w);
        self.panes.push(p);
        (wl, wptr)
    }

    fn sptr(&mut self) -> *mut session {
        self.session.ptr()
    }

    /// Replaces the cleanup list with exactly `winlinks`, for a run after
    /// which the session's winlinks are not the ones the fixture linked.
    fn tracking(&mut self, winlinks: &[*mut winlink]) {
        self.tracked = winlinks.to_vec();
    }
}

impl Drop for Chain {
    fn drop(&mut self) {
        for wl in ::std::mem::take(&mut self.tracked).into_iter().rev() {
            unlink(&mut self.session, wl);
        }
    }
}

/// Runs the item's parsed command through its own entry's exec hook, the
/// way the command queue would.
fn run(item: &mut Item) -> cmd_retval {
    unsafe {
        let e = (*item.cmd()).entry;
        (e.exec)(&*item.cmd(), item.ptr())
    }
}

/// Points the item's source and current states at `wl`, as a prepared
/// item's resolved find states would be; the target state this hook builds
/// for itself out of `-t`.
unsafe fn aim(item: &mut Item, wl: *mut winlink) {
    unsafe {
        let mut fs = *Box::new(cmd_find_state::default());
        cmd_find_from_winlink(&mut fs, wl, 0);
        let p = item.ptr();
        (*p).source = fs.clone();
        (*p).target = fs.clone();
        *cmdq_get_current(p) = fs.clone();
    }
}

/// A `-t` naming an index no window holds. `move-window` resolves its own
/// target with `CMD_FIND_WINDOW_INDEX`, which answers such a `-t` with the
/// index alone and a null winlink, so `-a` shuffles up from the
/// destination session's *current* window instead of the target's.
#[test]
fn an_index_only_target_shuffles_up_from_the_sessions_current_window() {
    let _guard = globals();
    ensure_reactor();
    let mut chain = Chain::new("0");
    let (wl0, w0) = chain.add_window(0);
    let (_, w1) = chain.add_window(1);

    let mut item = Item::new().with_args(c"move-window -a -d -t 9");
    unsafe {
        assert_eq!(
            session_get_curw(chain.sptr()),
            wl0,
            "the first window is current"
        );
        aim(&mut item, wl0);

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        let s = chain.sptr();
        let at_one = winlink_find_by_index(&mut (*s).windows, 1);
        let at_two = winlink_find_by_index(&mut (*s).windows, 2);
        chain.tracking(&[at_one, at_two]);

        assert_eq!(winlink_count(&(*s).windows), 2);
        assert!(
            winlink_find_by_index(&mut (*s).windows, 0).is_null(),
            "the source slot was given up"
        );
        assert!(!at_one.is_null() && !at_two.is_null());
        assert_eq!(
            (*at_one).window(),
            w0,
            "the moved window took the index freed above the current one"
        );
        assert_eq!(
            (*at_two).window(),
            w1,
            "which the window standing there shuffled up for"
        );
        assert!(
            winlink_find_by_index(&mut (*s).windows, 9).is_null(),
            "the -t index itself was never used"
        );
    }
}

/// A `-t` naming the last index there is. `winlink_shuffle_up` looks for a
/// free index at or above the one it is given and gives up at `INT_MAX`,
/// so `-b` there refuses before anything is linked or unlinked.
#[test]
fn a_target_at_the_last_index_refuses_to_shuffle_up() {
    let _guard = globals();
    ensure_reactor();
    let mut chain = Chain::new("0");
    let (wl0, w0) = chain.add_window(0);
    let (wl_last, w_last) = chain.add_window(c_int::MAX);

    let mut item = Item::new().with_args(c"move-window -b -d -t 2147483647");
    unsafe {
        aim(&mut item, wl0);

        assert_eq!(
            run(&mut item),
            CMD_RETURN_ERROR,
            "there is no index above the last one to shuffle into"
        );

        let s = chain.sptr();
        assert_eq!(winlink_count(&(*s).windows), 2);
        assert_eq!(winlink_find_by_index(&mut (*s).windows, 0), wl0);
        assert_eq!((*wl0).window(), w0);
        assert_eq!(
            winlink_find_by_index(&mut (*s).windows, c_int::MAX),
            wl_last
        );
        assert_eq!((*wl_last).window(), w_last);
        assert_eq!(session_get_curw(s), wl0);
    }
}
