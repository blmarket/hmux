use super::*;
use crate::cmd::cmd_find_from_winlink;

use crate::tests::test_fixtures::{
    Item, Pane, Registry, Session, Window, ensure_reactor, globals, link,
};
use crate::window::{WinlinkRef, winlink_count};
use ::core::ffi::c_int;

/// Where the fixture windows' ids start, far above anything production
/// hands out from its own counters.
const WINDOW_ID_BASE: u_int = 810_000;

/// Where the fixture panes' ids start.
const PANE_ID_BASE: u_int = 910_000;

/// One registered session holding registered windows, the way
/// `cmd_find_target` walks them. Winlinks the fixture linked are unlinked
/// again on the way out by sweeping the session's current link collection.
struct Chain {
    registry: Registry,
    session: Session,
    windows: Vec<Window>,
    panes: Vec<Pane>,
}

impl Chain {
    fn new(name: &str) -> Chain {
        let mut c = Chain {
            registry: Registry::new(),
            session: Session::new(0, name),
            windows: Vec::new(),
            panes: Vec::new(),
        };
        c.registry.add_session(&mut c.session);
        c
    }

    fn add_window(&mut self, idx: c_int) -> (WinlinkRef, WindowRef) {
        let mut w = Window::new(
            WINDOW_ID_BASE + self.windows.len() as u_int * 11,
            "chain",
            80,
            24,
        );
        let mut p = Pane::new(PANE_ID_BASE + self.panes.len() as u_int, 80, 24, 100);
        w.add_pane(&mut p);
        self.registry.add_window(&mut w);
        let window = w.reference();
        link(&mut self.session, &mut w, idx);
        let wl = WinlinkRef::new(self.session.reference(), idx).unwrap();
        self.windows.push(w);
        self.panes.push(p);
        (wl, window)
    }
}

impl Drop for Chain {
    fn drop(&mut self) {
        let mut owner = self.session.handle().clone();
        unsafe {
            let session = owner.as_session_mut();
            session.curw_idx = None;
            session.lastw.clear();
            while let Some(index) = session.windows.keys().next().copied() {
                crate::window::winlink_remove(&mut session.windows, index);
            }
        }
    }
}

/// Runs the item's parsed command through its own entry's exec hook, the
/// way the command queue would.
fn run(item: &mut Item) -> cmd_retval {
    unsafe { item.with_command(|command, item| (command.entry.exec)(command, item)) }
}

/// Points the item's source and current states at `wl`, as a prepared
/// item's resolved find states would be; the target state this hook builds
/// for itself out of `-t`.
unsafe fn aim(item: &mut Item, wl: &WinlinkRef) {
    unsafe {
        let mut fs = cmd_find_state::default();
        cmd_find_from_winlink(&mut fs, wl.get().unwrap(), 0);
        let mut item = item.item_mut();
        item.source = fs.clone();
        item.target = fs.clone();
        *item.current() = fs;
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
        assert!(
            core::ptr::eq(
                chain
                    .session
                    .handle()
                    .curw()
                    .unwrap()
                    .get()
                    .expect("the current link is live"),
                wl0.get().unwrap()
            ),
            "the first window is current"
        );
        aim(&mut item, &wl0);

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        let s = chain.session.handle().as_session();
        let at_one = s
            .windows
            .get(&1)
            .map(Box::as_ref)
            .expect("the indexed window is linked");
        let at_two = s
            .windows
            .get(&2)
            .map(Box::as_ref)
            .expect("the indexed window is linked");

        assert_eq!(winlink_count(&s.windows), 2);
        assert!(s.windows.get(&0).is_none(), "the source slot was given up");

        assert!(
            at_one.window_handle().unwrap().ptr_eq(&w0),
            "the moved window took the index freed above the current one"
        );
        assert!(
            at_two.window_handle().unwrap().ptr_eq(&w1),
            "which the window standing there shuffled up for"
        );
        assert!(
            s.windows.get(&9).is_none(),
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
        aim(&mut item, &wl0);

        assert_eq!(
            run(&mut item),
            CMD_RETURN_ERROR,
            "there is no index above the last one to shuffle into"
        );

        let s = chain.session.handle().as_session();
        assert_eq!(winlink_count(&s.windows), 2);
        assert!(
            s.windows
                .get(&0)
                .map(Box::as_ref)
                .is_some_and(|link| core::ptr::eq(link, wl0.get().unwrap()))
        );
        assert!(wl0.get().unwrap().window_handle().unwrap().ptr_eq(&w0));
        assert!(
            s.windows
                .get(&c_int::MAX)
                .map(Box::as_ref)
                .is_some_and(|link| core::ptr::eq(link, wl_last.get().unwrap()))
        );
        assert!(
            wl_last
                .get()
                .unwrap()
                .window_handle()
                .unwrap()
                .ptr_eq(&w_last)
        );
        assert!(core::ptr::eq(s.curw().unwrap(), wl0.get().unwrap()));
    }
}
