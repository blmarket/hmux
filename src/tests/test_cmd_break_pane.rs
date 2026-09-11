use super::*;
use crate::cmd::cmd_find_from_winlink;
use crate::options::OptionsRef;
use crate::tests::test_fixtures::{
    Item, Pane, Registry, Session, Window, ensure_reactor, globals, link,
};
use crate::window::window_count_panes;
use crate::window::winlink_count;
use crate::window::winlink_find_by_window;
use crate::window_pane::RustWindowPaneWeak;
use ::core::ffi::c_int;
use ::std::ffi::CString;

/// Where the fixture windows' ids start, clear of anything
/// `window_create` hands out from the server's own counter.
const WINDOW_ID_BASE: u_int = 700_000;

/// Where the fixture panes' ids start.
const PANE_ID_BASE: u_int = 800_000;

/// Windows the exec hook built with `window_create` and left in the
/// server's tree, held until the test ends.
struct Created(Vec<WindowRef>);

impl Created {
    fn new() -> Created {
        Created(Vec::new())
    }

    fn keep(&mut self, window: &WindowRef) {
        self.0.push(window.clone());
    }
}

/// Registered sessions holding linked windows of real panes, in the
/// server's trees the way a prepared command queue item's find states
/// expect to walk them. Every pane gets a shell string so
/// `default_window_name` reads something deterministic when the pane
/// becomes the active pane of a fresh window. All links remaining in the
/// fixture's sessions are unlinked on the way out, including links created
/// or renumbered by commands.
struct World {
    registry: Registry,
    sessions: Vec<Session>,
    windows: Vec<Window>,
    panes: Vec<Pane>,
    shells: Vec<CString>,
}

impl World {
    fn new(name: &str) -> World {
        let mut w = World {
            registry: Registry::new(),
            sessions: Vec::new(),
            windows: Vec::new(),
            panes: Vec::new(),
            shells: Vec::new(),
        };
        w.add_session(name);
        w
    }

    fn add_session(&mut self, name: &str) -> usize {
        self.sessions
            .push(Session::new(self.sessions.len() as u_int, name));
        self.registry
            .add_session(self.sessions.last_mut().expect("a session"));
        self.sessions.len() - 1
    }

    /// Links a fresh window of `panes` panes at index `idx` behind session
    /// `sidx`, answering its winlink and its panes in creation order; the
    /// first pane is the window's active one.
    fn add_window(
        &mut self,
        sidx: usize,
        idx: c_int,
        panes: usize,
    ) -> (*mut winlink, Vec<RustWindowPaneWeak>) {
        let wid = WINDOW_ID_BASE + self.windows.len() as u_int * 17 + sidx as u_int;
        let mut w = Window::new(wid, "world", 80, 24);
        let mut made = Vec::new();
        for _ in 0..panes {
            let pid = PANE_ID_BASE + self.panes.len() as u_int + 1;
            let mut p = Pane::new(pid, 80, 24, 100);
            self.shells
                .push(CString::new("/bin/sh").expect("a shell path has no NUL"));
            w.add_pane(&mut p);
            let mut owner = {
                w.handle()
                    .as_window()
                    .panes
                    .last()
                    .map(|pane| pane.downgrade())
                    .unwrap()
            };
            unsafe {
                let pane = owner.as_pane_mut();
                let mut command = pane.pane_command();
                command.shell = Some(self.shells.last().expect("a shell").clone());
                pane.set_pane_command(&command);
            };
            made.push(owner);
            self.panes.push(p);
        }
        self.registry.add_window(&mut w);
        let wl = link(&mut self.sessions[sidx], &mut w, idx);
        self.windows.push(w);
        (wl, made)
    }

    fn session(&self, i: usize) -> SessionRef {
        self.sessions[i].reference()
    }
}

impl Drop for World {
    fn drop(&mut self) {
        for fixture in &self.sessions {
            let mut owner = fixture.handle().clone();
            unsafe {
                let session = owner.as_session_mut();
                session.curw = None;
                session.lastw.clear();
                while let Some(index) = session.windows.keys().next().copied() {
                    crate::window::winlink_remove(&mut session.windows, index);
                }
            }
        }
    }
}

/// Runs the item's parsed command through the entry's exec hook, the way
/// the command queue would.
fn run(item: &mut Item) -> cmd_retval {
    unsafe { item.with_command(|command, item| (cmd_break_pane_entry.exec)(command, item)) }
}

/// Points the item's target, source and current states where the test
/// wants them, as a prepared item's resolved find states would be.
fn aim(item: &mut Item, target: cmd_find_state, source: cmd_find_state) {
    {
        let mut item = item.item_mut();
        item.target = target.clone();
        item.source = source;
        *item.current() = target;
    }
}

/// The find state of `wl` with `idx` filled in by hand, since resolution
/// is the command queue's job and this hook reads the states as given.
unsafe fn fs_of(wl: *mut winlink, idx: c_int) -> cmd_find_state {
    let mut fs = cmd_find_state::default();
    unsafe { cmd_find_from_winlink(&mut fs, &*wl, 0) };
    fs.idx = idx;
    fs
}

/// A target state naming only a session and an index, which is what
/// `cmd_find_target` leaves behind for a `-t` window index that no window
/// holds — `CMD_FIND_WINDOW_INDEX` fills in `idx` and keeps `wl` null.
fn fs_index(s: &SessionRef, idx: c_int) -> cmd_find_state {
    let mut fs = cmd_find_state::default();
    fs.set_session_ref(Some(s));
    fs.idx = idx;
    fs
}

#[test]
fn an_index_only_target_shuffles_up_from_the_sessions_current_window() {
    let _guard = globals();
    ensure_reactor();
    let mut created = Created::new();
    let mut world = World::new("0");
    let (wl_cur, panes) = world.add_window(0, 0, 2);
    let (wl_next, _) = world.add_window(0, 1, 1);
    let w_cur = unsafe { (*wl_cur).window_handle().unwrap().clone() };
    let moved = panes[0].clone();

    let mut item = Item::new().with_args(c"break-pane -a");
    unsafe {
        let s = world.session(0);
        assert!(
            core::ptr::eq(
                s.curw().unwrap().get().expect("the current link is live"),
                wl_cur
            ),
            "the first linked window is current"
        );
        aim(&mut item, fs_index(&s, 9), fs_of(wl_cur, -1));

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        assert_eq!(
            (*wl_next).idx,
            2,
            "the window above the current one shuffled up"
        );
        let wl_new = s
            .as_session()
            .windows
            .get(&1)
            .map(Box::as_ref)
            .expect("the indexed window is linked");

        let nw = wl_new.window_handle().unwrap().clone();
        created.keep(&nw);
        assert!(
            moved.window().unwrap().ptr_eq(&nw),
            "the pane moved into the new window"
        );
        assert!(!nw.ptr_eq(&w_cur));
        assert!(
            s.as_session()
                .windows
                .get(&0)
                .map(Box::as_ref)
                .is_some_and(|link| core::ptr::eq(link, wl_cur)),
            "the current window kept its own index"
        );
        assert_eq!(winlink_count(&s.as_session().windows), 3);
        assert!(
            core::ptr::eq(
                s.curw().unwrap().get().expect("the current link is live"),
                wl_new
            ),
            "without -d the new window is selected"
        );
        assert_eq!(window_count_panes(&w_cur.as_window(), 1), 1);
    }
}

#[test]
fn a_target_at_the_last_index_refuses_to_shuffle_up() {
    let _guard = globals();
    ensure_reactor();
    let mut world = World::new("0");
    let (wl_src, panes) = world.add_window(0, 0, 2);
    let (wl_last, _) = world.add_window(0, c_int::MAX, 1);
    let w_src = unsafe { (*wl_src).window_handle().unwrap().clone() };

    let mut item = Item::new().with_args(c"break-pane -b");
    unsafe {
        aim(&mut item, fs_of(wl_last, c_int::MAX), fs_of(wl_src, -1));

        assert_eq!(
            run(&mut item),
            CMD_RETURN_ERROR,
            "there is no index above the last one to shuffle into"
        );

        let s = world.session(0);
        assert_eq!(
            winlink_count(&s.as_session().windows),
            2,
            "nothing was linked"
        );
        assert!(
            s.as_session()
                .windows
                .get(&c_int::MAX)
                .map(Box::as_ref)
                .is_some_and(|link| core::ptr::eq(link, wl_last)),
            "the target stayed at the last index"
        );
        assert_eq!(
            window_count_panes(&w_src.as_window(), 1),
            2,
            "the source kept its panes"
        );
        assert!(
            w_src
                .as_window()
                .panes
                .get(0)
                .is_some_and(|owner| { owner.downgrade().ptr_eq(&panes[0]) })
        );
        assert!(
            w_src
                .as_window()
                .panes
                .get(1)
                .is_some_and(|owner| { owner.downgrade().ptr_eq(&panes[1]) })
        );
        assert!(core::ptr::eq(
            s.curw().unwrap().get().expect("the current link is live"),
            wl_src
        ));
    }
}

#[test]
fn a_single_pane_window_relinked_without_n_keeps_the_name_it_had() {
    let _guard = globals();
    ensure_reactor();
    let mut world = World::new("src");
    let (wl_src, _) = world.add_window(0, 0, 1);
    world.add_window(0, 1, 1);
    let w_src = unsafe { (*wl_src).window_handle().unwrap().clone() };
    let dst = world.add_session("dst");
    let (wl_dst, _) = world.add_window(dst, 0, 1);

    let mut item = Item::new().with_args(c"break-pane -d");
    unsafe {
        aim(&mut item, fs_of(wl_dst, 4), fs_of(wl_src, -1));

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        let s_dst = world.session(dst);
        let wl_new = s_dst
            .as_session()
            .windows
            .get(&4)
            .map(Box::as_ref)
            .expect("the indexed window is linked");

        assert!(wl_new.window_handle().unwrap().ptr_eq(&w_src));
        assert!(
            core::ptr::eq(
                s_dst
                    .curw()
                    .unwrap()
                    .get()
                    .expect("the current link is live"),
                wl_dst
            ),
            "-d keeps the destination's current window"
        );
        assert_eq!(
            w_src
                .window_name()
                .as_deref()
                .expect("a window has a name")
                .to_str()
                .unwrap(),
            "world",
            "the relinked window keeps the name it came with"
        );
        assert_eq!(
            (w_src.options()).number(c"automatic-rename"),
            1,
            "and its automatic renaming is left alone"
        );
    }
}

#[test]
fn p_takes_the_format_from_f_instead_of_the_default_template() {
    let _guard = globals();
    ensure_reactor();
    let mut created = Created::new();
    let mut world = World::new("0");
    let (wl0, panes) = world.add_window(0, 0, 2);
    let w0 = unsafe { (*wl0).window_handle().unwrap().clone() };
    let moved = panes[0].clone();

    let mut item = Item::new().with_args(c"break-pane -d -P -F '#{window_name}'");
    unsafe {
        let args = item.args();
        assert_eq!(args.argument_flag_count(b'P'), 1);
        assert_eq!(args.argument_flag_string(b'F'), Some(c"#{window_name}"));
        drop(args);
        aim(&mut item, fs_of(wl0, -1), fs_of(wl0, -1));

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        assert_eq!(window_count_panes(&w0.as_window(), 1), 1);
        let nw = moved.window().unwrap();
        created.keep(&nw);
        let s = world.session(0);
        assert!(winlink_find_by_window(&s.as_session().windows, &nw.as_window()).is_some());
        assert!(!nw.ptr_eq(&w0));
        assert_eq!(
            nw.window_name()
                .as_deref()
                .expect("a window has a name")
                .to_str()
                .unwrap(),
            "sh"
        );
    }
}

#[test]
fn a_single_pane_window_is_relinked_into_the_destination_session_and_n_renames_it() {
    let _guard = globals();
    ensure_reactor();
    let mut world = World::new("src");
    let (wl_src, panes) = world.add_window(0, 0, 1);
    let (wl_keep, _) = world.add_window(0, 1, 1);
    let w_src = unsafe { (*wl_src).window_handle().unwrap().clone() };
    let dst = world.add_session("dst");
    let (wl_dst, _) = world.add_window(dst, 0, 1);

    let mut item = Item::new().with_args(c"break-pane -n moved");
    unsafe {
        aim(&mut item, fs_of(wl_dst, 5), fs_of(wl_src, -1));

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        let s_src = world.session(0);
        let s_dst = world.session(dst);
        assert_eq!(
            winlink_count(&s_src.as_session().windows),
            1,
            "the source session gave the window up"
        );
        assert!(s_src.as_session().windows.get(&0).is_none());
        assert!(
            core::ptr::eq(
                s_src
                    .curw()
                    .unwrap()
                    .get()
                    .expect("the current link is live"),
                wl_keep
            ),
            "the source moved on to what is left"
        );

        let wl_new = s_dst
            .as_session()
            .windows
            .get(&5)
            .map(Box::as_ref)
            .expect("the indexed window is linked");

        assert!(
            wl_new.window_handle().unwrap().ptr_eq(&w_src),
            "it is the very same window"
        );
        assert!(
            winlink_find_by_window(&s_dst.as_session().windows, &w_src.as_window())
                .is_some_and(|link| core::ptr::eq(link, wl_new))
        );
        assert_eq!(winlink_count(&s_dst.as_session().windows), 2);
        assert!(
            core::ptr::eq(
                s_dst
                    .curw()
                    .unwrap()
                    .get()
                    .expect("the current link is live"),
                wl_new
            ),
            "without -d the destination selects it"
        );
        assert!(
            s_dst
                .as_session()
                .lastw
                .first()
                .and_then(|index| s_dst.as_session().windows.get(index))
                .is_some_and(|link| core::ptr::eq(link.as_ref(), wl_dst))
        );

        assert_eq!(
            window_count_panes(&w_src.as_window(), 1),
            1,
            "the window kept its pane"
        );
        assert!(
            w_src
                .as_window()
                .panes
                .get(0)
                .is_some_and(|owner| { owner.downgrade().ptr_eq(&panes[0]) })
        );
        assert!(
            panes[0].window().unwrap().ptr_eq(&w_src),
            "no new window was built"
        );
        assert_eq!(
            w_src
                .window_name()
                .as_deref()
                .expect("a window has a name")
                .to_str()
                .unwrap(),
            "moved"
        );
        assert_eq!(
            (w_src.options()).number(c"automatic-rename"),
            0,
            "-n switches automatic renaming off"
        );
    }
}

#[test]
fn breaking_the_last_pane_hands_both_list_tails_to_the_one_in_front() {
    let _guard = globals();
    ensure_reactor();
    let mut created = Created::new();
    let mut world = World::new("0");
    let (wl0, panes) = world.add_window(0, 0, 3);
    let w0 = unsafe { (*wl0).window_handle().unwrap().clone() };
    let (first, front, moved) = (panes[0].clone(), panes[1].clone(), panes[2].clone());

    let mut item = Item::new().with_args(c"break-pane -d");
    unsafe {
        aim(&mut item, fs_of(wl0, -1), fs_of(wl0, -1));
        item.item_mut().source.set_pane(moved.get());

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        assert_eq!(window_count_panes(&w0.as_window(), 1), 2);
        assert!(
            w0.as_window()
                .panes
                .get(0)
                .is_some_and(|owner| owner.downgrade().ptr_eq(&first))
        );
        assert!(
            w0.as_window()
                .panes
                .get(1)
                .is_some_and(|owner| owner.downgrade().ptr_eq(&front))
        );
        assert!(
            w0.as_window()
                .panes
                .last()
                .is_some_and(|owner| owner.downgrade().ptr_eq(&front)),
            "the pane in front took over the pane list's tail"
        );
        assert_eq!(
            w0.as_window()
                .z_index
                .iter()
                .map(|pane| pane.id())
                .collect::<Vec<_>>(),
            vec![first.id(), front.id()],
            "and the z-order list's tail"
        );
        assert!(w0.as_window().panes.get(2).is_none());

        let nw = moved.window().unwrap();
        created.keep(&nw);
        assert!(!nw.ptr_eq(&w0));
        assert!(
            nw.as_window()
                .panes
                .first()
                .is_some_and(|owner| owner.downgrade().ptr_eq(&moved))
        );
        assert!(
            nw.as_window()
                .panes
                .last()
                .is_some_and(|owner| owner.downgrade().ptr_eq(&moved))
        );
        assert_eq!(
            nw.as_window()
                .z_index
                .iter()
                .map(|pane| pane.id())
                .collect::<Vec<_>>(),
            vec![moved.id()]
        );
        assert!(nw.active_pane().is_some_and(|pane| { pane.ptr_eq(&moved) }));
        assert_eq!(window_count_panes(&nw.as_window(), 1), 1);

        let s = world.session(0);
        let wl_new = s
            .as_session()
            .windows
            .get(&1)
            .map(Box::as_ref)
            .expect("the indexed window is linked");

        assert!(wl_new.window_handle().unwrap().ptr_eq(&nw));
        assert!(
            core::ptr::eq(
                s.curw().unwrap().get().expect("the current link is live"),
                wl0
            ),
            "-d keeps the current window selected"
        );
    }
}
