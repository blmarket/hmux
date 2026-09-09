use crate::arguments::{
    args_count, args_flags, args_get_str, args_print, args_string_str, args_value_list,
};
use crate::cfg::cfg_add_cause;
use crate::cfg::cfg_finished;
use crate::cmd::find::{
    cmd_find_clear_state, cmd_find_client, cmd_find_copy_state, cmd_find_from_client,
    cmd_find_target, cmd_find_valid_state,
};
use crate::cmd::{cmd_get_args, cmd_get_entry, cmd_get_group, cmd_get_source, cmd_print};
use crate::command_entry::{CommandEntry, RustCommandContext};
use crate::compat::toupper;
use crate::control::control_write;
use crate::ffi::{getuid, time};
use crate::file::file_error;
use crate::fmt_args;
use crate::fmt_engine::{FmtArg, format_alloc, format_buf};
use crate::format::{format_add, format_create, format_merge};
use crate::log::{log_debug, log_get_level};
use crate::options::{OptionsEngine, RustOptionsEngine};

use crate::server::server_add_message;
use crate::server::server_client_print;

use crate::status::status_message_set;
use crate::text::{KeyStringCodec, RustKeyStringCodec};
use crate::text::{RustUtf8VisModel, Utf8VisModel};
use crate::tmux::global_s_options;
use crate::tree::GlobalQueue;
pub use crate::types::*;
use crate::xmalloc::xasprintf;
use crate::{UserAccount, UserAccountRecord};
use ::std::cell::{RefCell, RefMut};
use ::std::ffi::{CStr, CString};
use ::std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicU32, Ordering};
#[repr(C)]
pub struct cmdq_list {
    /// Whether the queue is part-way through running the item at the front
    /// of its list. The item itself is never held: the running one is always
    /// the front one.
    running: bool,
    list: cmdq_item_list,
}

/// A command queue retained independently of its client while work runs.
#[derive(Clone)]
pub struct CmdqListRef(Rc<RefCell<cmdq_list>>);

/// An item's observation of its queue, without keeping that queue alive.
#[derive(Clone)]
pub struct CmdqListWeak(Weak<RefCell<cmdq_list>>);

impl CmdqListRef {
    pub(crate) fn downgrade(&self) -> CmdqListWeak {
        CmdqListWeak(Rc::downgrade(&self.0))
    }
}

impl CmdqListWeak {
    pub(crate) fn upgrade(&self) -> Option<CmdqListRef> {
        self.0.upgrade().map(CmdqListRef)
    }
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "Queue inspection and reset operations are used by unit tests"
    )
)]
pub(crate) trait CmdqListOps {
    fn is_empty(&self) -> bool;
    fn len(&self) -> usize;
    fn items(&self) -> impl Iterator<Item = CmdqItemRef>;
    fn item_at(&self, at: usize) -> Option<CmdqItemRef>;
    fn append_item(&self, item: CmdqItemRef);
    fn insert_item_after(&self, at: usize, item: CmdqItemRef);
    fn remove_item_at(&self, at: usize) -> Option<CmdqItemRef>;
    fn clear(&self);
    fn truncate(&self, len: usize);
    fn is_running(&self) -> bool;
    fn set_running(&self, running: bool);
}

impl CmdqListOps for CmdqListRef {
    fn is_empty(&self) -> bool {
        self.0.borrow().list.is_empty()
    }
    fn len(&self) -> usize {
        self.0.borrow().list.len()
    }
    fn items(&self) -> impl Iterator<Item = CmdqItemRef> {
        self.0.borrow().list.clone().into_iter()
    }
    fn item_at(&self, at: usize) -> Option<CmdqItemRef> {
        self.0.borrow().list.get(at).cloned()
    }
    fn append_item(&self, item: CmdqItemRef) {
        self.0.borrow_mut().list.push_back(item);
    }
    fn insert_item_after(&self, at: usize, item: CmdqItemRef) {
        self.0.borrow_mut().list.insert(at + 1, item);
    }
    fn remove_item_at(&self, at: usize) -> Option<CmdqItemRef> {
        self.0.borrow_mut().list.remove(at)
    }
    fn clear(&self) {
        let removed = std::mem::take(&mut self.0.borrow_mut().list);
        drop(removed);
    }
    fn truncate(&self, len: usize) {
        let removed = {
            let mut queue = self.0.borrow_mut();
            let len = len.min(queue.list.len());
            queue.list.split_off(len)
        };
        drop(removed);
    }
    fn is_running(&self) -> bool {
        self.0.borrow().running
    }
    fn set_running(&self, running: bool) {
        self.0.borrow_mut().running = running;
    }
}

/// The items waiting on one queue, front to back, and the owner of each.
/// [`cmdq_remove`] is what gives one up.
pub type cmdq_item_list = std::collections::VecDeque<CmdqItemRef>;
/// A run of items a caller has had made but not yet put on a queue, in the
/// order they are to run. [`cmdq_append`] and [`CmdqItemRef::insert_after`] are what
/// take one.
pub type cmdq_items = Vec<CmdqItemRef>;
type CmdqCallback = Box<dyn FnOnce(&CmdqItemRef) -> cmd_retval>;

/// What an item runs when the queue fires it, and the fields only an item of
/// that type has: a command out of a parsed command list, or a callback with
/// the data it was queued with.
pub(crate) enum CmdqType {
    Command {
        cmdlist: Option<CmdListRef>,
        /// Where the command sits in `cmdlist`, which is what names it: the
        /// item never holds a pointer into the list it shares.
        at: usize,
    },
    Callback {
        callback: Option<CmdqCallback>,
    },
}

#[repr(C)]
pub struct cmdq_item {
    pub name: Option<CString>,
    /// The queue the item is waiting on, if that queue still exists.
    pub queue: Option<CmdqListWeak>,
    pub(crate) client: Option<ClientRef>,
    pub(crate) target_client: Option<ClientWeak>,
    pub(crate) type_0: CmdqType,
    pub group: u_int,
    pub number: u_int,
    pub time: time_t,
    pub flags: core::ffi::c_int,
    state_ref: Option<CmdqStateRef>,
    owner: Option<CmdqItemWeak>,
    pub source: cmd_find_state,
    pub target: cmd_find_state,
}

/// A strong owner of a queue item. The raw pointer from [`CmdqItemRef::as_ptr`]
/// is only a borrowed compatibility view; the handle must remain alive for
/// every use of that pointer.
#[derive(Clone)]
pub struct CmdqItemRef(Rc<RefCell<cmdq_item>>);

/// A non-owning observation of a queue item. A command that answers later
/// holds the item it is to answer this way: the item stays on its queue for
/// as long as it is waiting, and one whose queue has given it up is found as
/// nothing rather than as a freed item.
#[derive(Clone)]
pub struct CmdqItemWeak(Weak<RefCell<cmdq_item>>);

/// Upgrades the payload's owner without borrowing its storage.
pub(crate) fn cmdq_item_ref_of(item: &cmdq_item) -> Option<CmdqItemRef> {
    item.owner
        .as_ref()?
        .upgrade()
        .filter(|owner| owner.points_to(item))
}

/// The same as an observation, which is what a command stores while it waits.
pub(crate) fn cmdq_item_weak_of(item: &cmdq_item) -> Option<CmdqItemWeak> {
    cmdq_item_ref_of(item).map(|reference| reference.downgrade())
}

impl CmdqItemWeak {
    /// Upgrades the observation while an owner still retains the item.
    pub(crate) fn upgrade(&self) -> Option<CmdqItemRef> {
        self.0.upgrade().map(CmdqItemRef)
    }
}

impl CmdqItemRef {
    fn new(value: cmdq_item) -> Self {
        let reference = Self(Rc::new(RefCell::new(value)));
        reference.0.borrow_mut().owner = Some(reference.downgrade());
        reference
    }

    /// Makes a non-owning observation of this item.
    pub(crate) fn downgrade(&self) -> CmdqItemWeak {
        CmdqItemWeak(Rc::downgrade(&self.0))
    }

    pub(crate) fn as_ptr(&self) -> *mut cmdq_item {
        self.0.as_ptr()
    }

    pub(crate) fn points_to(&self, item: &cmdq_item) -> bool {
        core::ptr::eq(self.0.as_ptr(), item)
    }

    pub(crate) fn read(&self) -> std::cell::Ref<'_, cmdq_item> {
        self.0.borrow()
    }

    pub(crate) fn with_item<R>(&self, read: impl FnOnce(&cmdq_item) -> R) -> R {
        read(&self.read())
    }

    fn fire_callback(&self) -> cmd_retval {
        let callback = {
            let mut item = self.0.borrow_mut();
            match &mut item.type_0 {
                CmdqType::Callback { callback } => callback.take(),
                CmdqType::Command { .. } => return CMD_RETURN_ERROR,
            }
            .expect("a command queue callback only fires once")
        };
        callback(self)
    }

    pub(crate) fn item(&self) -> RefMut<'_, cmdq_item> {
        self.0.borrow_mut()
    }
}

impl cmdq_item {
    pub(crate) fn state_ref(&self) -> CmdqStateRef {
        self.state_ref
            .clone()
            .expect("a queue item without a state")
    }

    pub(crate) fn command_location(&self) -> Option<(&CmdListRef, usize)> {
        match &self.type_0 {
            CmdqType::Command { cmdlist, at } => cmdlist.as_ref().map(|list| (list, *at)),
            CmdqType::Callback { .. } => None,
        }
    }

    pub(crate) fn command(&self) -> Option<std::cell::Ref<'_, cmd>> {
        let (list, at) = self.command_location()?;
        list.command(at)
    }
}
pub use crate::consts::{
    CLIENT_CONTROL, CLIENT_UTF8, CMD_AFTERHOOK, CMD_CLIENT_CANFAIL, CMD_CLIENT_CFLAG,
    CMD_CLIENT_TFLAG, CMD_RETURN_ERROR, CMD_RETURN_NORMAL, CMD_RETURN_WAIT, CMDQ_STATE_CONTROL,
    CMDQ_STATE_NOHOOKS, FORMAT_NONE, KEYC_NONE,
};

#[repr(C)]
pub struct cmdq_state {
    pub flags: core::ffi::c_int,
    pub formats: Option<Box<format_tree>>,
    pub event: key_event,
    pub current: cmd_find_state,
}

#[derive(Clone)]
pub struct CmdqStateRef(Rc<RefCell<cmdq_state>>);

impl CmdqStateRef {
    fn new(value: cmdq_state) -> Self {
        Self(Rc::new(RefCell::new(value)))
    }

    pub(crate) fn state(&self) -> RefMut<'_, cmdq_state> {
        self.0.borrow_mut()
    }
}

pub const CMDQ_FIRED: core::ffi::c_int = 0x1 as core::ffi::c_int;
pub const CMDQ_WAITING: core::ffi::c_int = 0x2 as core::ffi::c_int;
/// How a client is named in the queue's log lines, as the caller's own
/// string.
unsafe fn cmdq_name(c: Option<&ClientRef>) -> CString {
    let Some(c) = c else {
        return c"<global>".to_owned();
    };
    if unsafe { c.name() }.is_some() {
        format_alloc(c"<%s>", fmt_args![unsafe { c.name() }])
    } else {
        format_alloc(c"<%p>", fmt_args![c.as_ptr()])
    }
}
/// The queue the items with no client behind them wait on, made when the
/// first one is queued and held by the server for as long as it runs.
static GLOBAL_QUEUE: GlobalQueue<CmdqListRef> = GlobalQueue::new();

unsafe fn cmdq_get(c: Option<&ClientRef>) -> CmdqListRef {
    let Some(c) = c else {
        let mut held = GLOBAL_QUEUE.queue();
        if held.is_empty() {
            held.push_back(CmdqListRef::empty());
        }
        return held
            .front()
            .expect("the global queue was just made")
            .clone();
    };
    c.command_queue()
}

pub unsafe fn cmdq_append(c: Option<&ClientRef>, items: cmdq_items) -> Option<CmdqItemRef> {
    unsafe { cmdq_get(c).append(c.cloned(), &cmdq_name(c), items) }
}

fn cmdq_remove(item: &mut cmdq_item) {
    let _ = item.client.take();
    let _ = item.state_ref.take();
    let queue = item.queue.as_ref().and_then(CmdqListWeak::upgrade);
    item.name = None;
    if let Some(queue) = queue
        && let Some(at) = queue.position_of(item)
    {
        queue.remove_item_at(at);
    }
    item.queue = None;
}
fn cmdq_remove_group(item: &mut cmdq_item) {
    if item.group == 0 as u_int {
        return;
    }
    let Some(queue) = item.queue.as_ref().and_then(CmdqListWeak::upgrade) else {
        return;
    };
    let Some(mut at) = queue.position_of(item) else {
        return;
    };
    while let Some(this) = queue.item_at(at + 1) {
        if this.item().group == item.group {
            cmdq_remove(&mut this.item());
        } else {
            at += 1;
        }
    }
}
fn cmdq_empty_command(_item: &CmdqItemRef) -> cmd_retval {
    CMD_RETURN_NORMAL
}

unsafe fn cmdq_find_flag(item: &cmdq_item, flag: &cmd_entry_flag) -> (cmd_retval, cmd_find_state) {
    unsafe {
        let mut fs = cmd_find_state::default();
        if flag.flag as core::ffi::c_int == 0 as core::ffi::c_int {
            let target_client = item.target_client();
            cmd_find_from_client(&mut fs, target_client.as_ref(), 0 as core::ffi::c_int);
            return (CMD_RETURN_NORMAL, fs);
        }
        let (list, at) = item
            .command_location()
            .expect("a target belongs to a command");
        let list = list.clone();
        let command = list.command(at).expect("the target command is in its list");
        let value = args_get_str(cmd_get_args(&command), flag.flag as u_char);
        if cmd_find_target(&mut fs, item, value, flag.type_0, flag.flags) != 0 as core::ffi::c_int {
            cmd_find_clear_state(&mut fs, 0 as core::ffi::c_int);
            return (CMD_RETURN_ERROR, fs);
        }
        (CMD_RETURN_NORMAL, fs)
    }
}
unsafe fn cmdq_add_message(item: &cmdq_item) {
    unsafe {
        let c = item.client();
        let event_key = item.state_ref().state().event.key;
        let uid: uid_t;
        let user: Option<CString>;
        let tmp = cmd_print(&item.command().expect("a message describes a command"));
        if let Some(c) = c {
            uid = (c.peer_handle()).uid();
            if uid != -(1 as core::ffi::c_int) as uid_t && uid != getuid() {
                if let Some(account) = UserAccountRecord::lookup_uid(uid) {
                    user = Some(xasprintf(c"[%s]", fmt_args![account.account_name()]));
                } else {
                    user = Some(c"[unknown]".to_owned());
                }
            } else {
                user = Some(c"".to_owned());
            }
            if !c.attached_session().is_none()
                && event_key != KEYC_NONE as core::ffi::c_ulong as key_code
            {
                let key = RustKeyStringCodec.format_key(event_key, false);
                server_add_message(
                    c"%s%s key %s: %s",
                    fmt_args![c.name(), user.as_deref(), key.as_c_str(), tmp.as_c_str()],
                );
            } else {
                server_add_message(
                    c"%s%s command: %s",
                    fmt_args![c.name(), user.as_deref(), tmp.as_c_str()],
                );
            }
        } else {
            server_add_message(c"command: %s", fmt_args![tmp.as_c_str()]);
        }
    }
}

fn cmdq_error_callback(item: &CmdqItemRef, error: CString) -> cmd_retval {
    unsafe {
        let item = item.item();
        item.error(c"%s", fmt_args![error.as_c_str()]);
        CMD_RETURN_NORMAL
    }
}

pub unsafe fn cmdq_next(c: Option<&ClientRef>) -> u_int {
    unsafe { cmdq_get(c).run(&cmdq_name(c)) }
}
pub unsafe fn cmdq_running(c: Option<&ClientRef>) -> Option<CmdqItemRef> {
    unsafe { cmdq_get(c).running_item() }
}

#[cfg(test)]
mod focused_boundary_tests {
    use super::*;

    fn callback(_item: &CmdqItemRef) -> cmd_retval {
        CMD_RETURN_NORMAL
    }

    #[test]
    fn item_owner_lookup_preserves_an_active_mutable_borrow() {
        let item = CmdqItemRef::callback_items(c"borrowed", callback).remove(0);
        let mut borrow = item.item();
        let recovered = cmdq_item_ref_of(&borrow).unwrap();
        borrow.number = 42;
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                recovered.read();
            }))
            .is_err()
        );
        drop(borrow);
        assert_eq!(recovered.read().number, 42);
    }

    #[test]
    fn item_owner_expires_before_callback_data_is_dropped() {
        struct OnDrop {
            owner: Rc<RefCell<Option<CmdqItemWeak>>>,
            observed: Rc<std::cell::Cell<bool>>,
        }
        impl Drop for OnDrop {
            fn drop(&mut self) {
                assert!(self.owner.borrow().as_ref().unwrap().upgrade().is_none());
                let nested = CmdqItemRef::callback_items(c"nested", callback).remove(0);
                assert!(nested.with_item(cmdq_item_ref_of).is_some());
                drop(nested);
                self.observed.set(true);
            }
        }
        std::thread::spawn(|| {
            let owner = Rc::new(RefCell::new(None));
            let observed = Rc::new(std::cell::Cell::new(false));
            let data = OnDrop {
                owner: owner.clone(),
                observed: observed.clone(),
            };
            let item = CmdqItemRef::callback_items(c"outer", move |_| {
                drop(data);
                CMD_RETURN_NORMAL
            })
            .remove(0);
            *owner.borrow_mut() = Some(item.downgrade());
            let retained = item.clone();
            let weak = item.downgrade();
            drop(item);
            assert!(!observed.get());
            assert!(weak.upgrade().is_some());
            drop(retained);
            assert!(observed.get());
            assert!(weak.upgrade().is_none());
        })
        .join()
        .unwrap();
    }

    #[test]
    fn item_owner_cleanup_works_during_thread_local_teardown() {
        thread_local! {
            static LAST_ITEM: RefCell<Option<CmdqItemRef>> = const { RefCell::new(None) };
        }
        std::thread::spawn(|| {
            LAST_ITEM.with_borrow_mut(|last| {
                let item = CmdqItemRef::callback_items(c"retained", callback).remove(0);
                assert!(item.with_item(cmdq_item_ref_of).is_some());
                *last = Some(item);
            });
        })
        .join()
        .unwrap();
    }

    #[test]
    fn callbacks_can_borrow_their_item_insert_work_and_resume_without_refiring() {
        let _guard = crate::tests::test_fixtures::globals();
        let mut client = ClientRef::new(client::default());
        unsafe { client.as_client_mut() }.queue = Some(CmdqListRef::empty());
        let calls = Rc::new(std::cell::Cell::new(0));
        let observed = calls.clone();
        let callback = CmdqItemRef::callback_items(c"waiting", move |item| {
            observed.set(observed.get() + 1);
            item.item().group = 42;
            let observed = observed.clone();
            unsafe {
                item.insert_after(CmdqItemRef::callback_items(c"inserted", move |item| {
                    assert_eq!(item.item().flags & CMDQ_WAITING, 0);
                    observed.set(observed.get() + 10);
                    CMD_RETURN_NORMAL
                }));
            }
            CMD_RETURN_WAIT
        });
        unsafe {
            let waiting = cmdq_append(Some(&client), callback).unwrap();
            assert_eq!(cmdq_next(Some(&client)), 0);
            assert_eq!(calls.get(), 1);
            assert_eq!(waiting.item().group, 42);
            assert_eq!(cmdq_next(Some(&client)), 0);
            waiting.resume();
            assert_eq!(cmdq_next(Some(&client)), 1);
            assert_eq!(calls.get(), 11);
            assert!(client.as_client().queue.as_ref().unwrap().is_empty());
        }
    }

    #[test]
    fn retained_items_do_not_keep_discarded_queues_alive() {
        let queue = CmdqListRef::empty();
        let item = CmdqItemRef::callback_items(c"retained", callback).remove(0);
        item.item().queue = Some(queue.downgrade());
        queue.append_item(item.clone());
        let weak = queue.downgrade();
        drop(queue);
        assert!(weak.upgrade().is_none());
        assert!(item.item().queue.as_ref().unwrap().upgrade().is_none());
        cmdq_remove(&mut item.item());
        assert!(item.item().queue.is_none());
    }

    #[test]
    fn callbacks_can_release_the_clients_queue_while_it_runs() {
        let _guard = crate::tests::test_fixtures::globals();
        let mut client = ClientRef::new(client::default());
        unsafe { client.as_client_mut() }.queue = Some(CmdqListRef::empty());
        let weak = unsafe { client.as_client() }
            .queue
            .as_ref()
            .unwrap()
            .downgrade();
        let observed = weak.clone();
        let items = CmdqItemRef::callback_items(c"detach", move |item| {
            let mut client = item.client().unwrap();
            unsafe { client.as_client_mut() }.queue = None;
            assert!(observed.upgrade().is_some());
            CMD_RETURN_NORMAL
        });
        unsafe {
            cmdq_append(Some(&client), items);
            assert_eq!(cmdq_next(Some(&client)), 1);
        }
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn discarding_callback_data_releases_queue_borrows_before_drop() {
        struct AppendOnDrop(CmdqListWeak);
        impl Drop for AppendOnDrop {
            fn drop(&mut self) {
                if let Some(queue) = self.0.upgrade() {
                    queue.append_item(CmdqItemRef::callback_items(c"cleanup", callback).remove(0));
                }
            }
        }
        for truncate in [false, true] {
            let queue = CmdqListRef::empty();
            let data = AppendOnDrop(queue.downgrade());
            let item = CmdqItemRef::callback_items(c"discarded", move |_| {
                drop(data);
                CMD_RETURN_NORMAL
            })
            .remove(0);
            queue.append_item(item);
            if truncate {
                queue.truncate(0);
            } else {
                queue.clear();
            }
            assert_eq!(queue.len(), 1);
            assert!(
                queue
                    .item_at(0)
                    .unwrap()
                    .item()
                    .name
                    .as_ref()
                    .unwrap()
                    .to_bytes()
                    .starts_with(b"[cleanup/")
            );
            queue.clear();
        }
    }

    #[test]
    fn target_lookup_returns_independent_source_and_failed_target_states() {
        use crate::tests::test_fixtures::{Item, Target, globals};
        let _guard = globals();
        let mut target = Target::new(80, 24);
        let mut item = Item::new()
            .with_args(c"move-pane -s %0 -t missing:")
            .targeting(&mut target);
        let source_flag = cmd_entry_flag {
            flag: b's' as _,
            type_0: CMD_FIND_PANE,
            flags: 0,
        };
        let target_flag = cmd_entry_flag {
            flag: b't' as _,
            type_0: CMD_FIND_PANE,
            flags: crate::cmd::find::CMD_FIND_QUIET,
        };
        unsafe {
            let (result, source) = cmdq_find_flag(&item.read(), &source_flag);
            assert_eq!(result, CMD_RETURN_NORMAL);
            assert!(source.pane_ref().is_some());
            item.item_mut().source = source;
            let (result, target) = cmdq_find_flag(&item.read(), &target_flag);
            assert_eq!(result, CMD_RETURN_ERROR);
            assert!(target.pane_ref().is_none());
            assert!(item.item_mut().source.pane_ref().is_some());
        }
    }

    #[test]
    fn inserting_work_does_not_require_exclusive_access_to_the_anchor() {
        let queue = CmdqListRef::empty();
        let anchor = CmdqItemRef::callback_items(c"anchor", callback).remove(0);
        anchor.item().queue = Some(queue.downgrade());
        queue.append_item(anchor.clone());
        let original = anchor.0.borrow();
        let inserted =
            unsafe { anchor.insert_after(CmdqItemRef::callback_items(c"inserted", callback)) };
        assert_eq!(queue.len(), 2);
        assert!(queue.item_at(0).unwrap().points_to(&original));
        assert!(queue.item_at(1).unwrap().points_to(&inserted.item()));
    }

    #[test]
    fn queue_boundary_owns_inspects_inserts_removes_and_transitions() {
        let mut queue = CmdqListRef::empty();
        let first = CmdqItemRef::callback_items(c"first", callback).remove(0);
        let second = CmdqItemRef::callback_items(c"second", callback).remove(0);
        let third = CmdqItemRef::callback_items(c"third", callback).remove(0);
        queue.append_item(first.clone());
        queue.append_item(third.clone());
        queue.insert_item_after(0, second.clone());
        assert_eq!(queue.len(), 3);
        assert_eq!(queue.item_at(0).unwrap().as_ptr(), first.as_ptr());
        assert_eq!(queue.item_at(1).unwrap().as_ptr(), second.as_ptr());
        assert_eq!(queue.items().count(), 3);
        queue.set_running(true);
        assert!(queue.is_running());
        assert_eq!(queue.remove_item_at(1).unwrap().as_ptr(), second.as_ptr());
        queue.truncate(1);
        assert_eq!(queue.len(), 1);
        queue.truncate(0);
        assert!(queue.is_empty());
        queue.append_item(first);
        queue.clear();
        queue.set_running(false);
        assert!(!queue.is_running());
        assert!(queue.is_empty());
        drop(queue);
    }
}

impl CmdqStateRef {
    pub(crate) unsafe fn create(
        current: Option<&cmd_find_state>,
        event: Option<&key_event>,
        flags: core::ffi::c_int,
    ) -> CmdqStateRef {
        unsafe {
            let state = CmdqStateRef::new(cmdq_state {
                flags,
                formats: None,
                event: key_event::default(),
                current: cmd_find_state::default(),
            });
            {
                let mut state_data = state.state();
                match event {
                    Some(event) => state_data.event = event.clone(),
                    None => state_data.event.key = KEYC_NONE as core::ffi::c_ulong as key_code,
                }
                if let Some(current) = current
                    && cmd_find_valid_state(current) != 0
                {
                    cmd_find_copy_state(&mut state_data.current, current);
                } else {
                    cmd_find_clear_state(&mut state_data.current, 0 as core::ffi::c_int);
                }
            }
            state
        }
    }
    pub(crate) unsafe fn copy_with_current(
        &self,
        current: Option<&cmd_find_state>,
    ) -> CmdqStateRef {
        let state = self;

        unsafe {
            let (current, event, flags) = {
                let state = state.state();
                (
                    current.cloned().unwrap_or_else(|| state.current.clone()),
                    state.event.clone(),
                    state.flags,
                )
            };
            CmdqStateRef::create(Some(&current), Some(&event), flags)
        }
    }
    /// The formats `state` carries, made when it has none yet.
    unsafe fn formats_mut(&self) -> RefMut<'_, format_tree> {
        let state = self;

        RefMut::map(state.state(), |state| {
            state
                .formats
                .get_or_insert_with(|| format_create(None, None, FORMAT_NONE, 0))
                .as_mut()
        })
    }
    pub unsafe fn add_format(&self, key: &CStr, fmt: &CStr, args: &[FmtArg]) {
        let state = self;

        unsafe {
            let value = format_alloc(fmt, args);
            let mut formats = state.formats_mut();
            format_add(&mut formats, key, c"%s", fmt_args![value.as_c_str()]);
        }
    }
    pub unsafe fn add_formats(&self, ft: &mut format_tree) {
        let state = self;

        unsafe {
            let mut formats = state.formats_mut();
            format_merge(&mut formats, ft);
        }
    }
}

impl CmdqItemRef {
    /// A fresh item of `type_0` sharing `state` with the rest of its queue.
    pub(crate) fn from_type(type_0: CmdqType, state: CmdqStateRef) -> CmdqItemRef {
        CmdqItemRef::new(cmdq_item {
            name: None,
            queue: None,
            client: None,
            target_client: None,
            type_0,
            group: 0,
            number: 0,
            time: 0,
            flags: 0,
            state_ref: Some(state),
            owner: None,
            source: cmd_find_state::default(),
            target: cmd_find_state::default(),
        })
    }
    /// The item's share of its queue's state, as a handle a new item can be given
    /// so that the two run under the same one.
    pub(crate) fn state_ref(&self) -> CmdqStateRef {
        let item = self;

        item.with_item(cmdq_item::state_ref)
    }
    pub unsafe fn insert_after(&self, items: cmdq_items) -> CmdqItemRef {
        let anchor = self;

        unsafe {
            let mut after = anchor.clone();
            if items.is_empty() {
                return after;
            }
            let (client, queue) = anchor.with_item(|item| {
                let queue = item
                    .queue
                    .as_ref()
                    .and_then(CmdqListWeak::upgrade)
                    .expect("the anchor has a live queue");
                (item.client(), queue)
            });
            for item in items {
                item.item().client = client.clone();
                item.item().queue = Some(queue.downgrade());
                after.with_item(|after| {
                    log_debug(
                        c"%s %s: %s after %s",
                        fmt_args![
                            c"cmdq_insert_after",
                            cmdq_name(client.as_ref()).as_c_str(),
                            item.item().name.as_deref(),
                            after.name.as_deref()
                        ],
                    )
                });
                let at = after
                    .with_item(|after| queue.position_of(after))
                    .expect("the anchor is on this queue");
                after = item.clone();
                queue.insert_item_after(at, item);
            }
            after
        }
    }
    /// Lets a parked item run again. Taking the strong handle makes a dead
    /// waiter unrepresentable here: whoever answers later holds a
    /// [`CmdqItemWeak`] and reaches this only through a successful upgrade.
    pub fn resume(&self) {
        let item = self;

        item.item().flags &= !CMDQ_WAITING;
    }
}

impl CmdqListRef {
    pub fn empty() -> CmdqListRef {
        CmdqListRef(Rc::new(RefCell::new(cmdq_list {
            running: false,
            list: cmdq_item_list::new(),
        })))
    }
    /// Where `item` sits on `queue`, which is wherever it was put.
    fn position_of(&self, item: &cmdq_item) -> Option<usize> {
        let queue = self;

        queue
            .0
            .borrow()
            .list
            .iter()
            .position(|waiting| waiting.points_to(item))
    }
}

impl CmdListRef {
    pub(crate) unsafe fn queue_items(&self, state: Option<&CmdqStateRef>) -> cmdq_items {
        let cmdlist = self;

        unsafe {
            let mut items = cmdq_items::new();
            let state = state
                .cloned()
                .unwrap_or_else(|| CmdqStateRef::create(None, None, 0));
            let commands = cmdlist.map(|cmd| (cmd_get_entry(cmd), cmd_get_group(cmd)));
            if commands.is_empty() {
                return CmdqItemRef::callback_items(c"cmdq_empty_command", cmdq_empty_command);
            }
            for (at, (entry, group)) in commands.into_iter().enumerate() {
                let new = CmdqItemRef::from_type(
                    CmdqType::Command {
                        cmdlist: Some(cmdlist.clone()),
                        at,
                    },
                    state.clone(),
                );
                {
                    let mut item = new.item();
                    item.name = Some(xasprintf(c"[%s/%p]", fmt_args![entry.name(), new.as_ptr()]));
                    item.group = group;
                    log_debug(
                        c"%s: %s group %u",
                        fmt_args![c"cmdq_get_command", item.name.as_deref(), item.group],
                    );
                }
                items.push(new);
            }
            items
        }
    }
}

impl CmdqListRef {
    pub unsafe fn append(
        &self,
        held: Option<ClientRef>,
        name: &CStr,
        items: cmdq_items,
    ) -> Option<CmdqItemRef> {
        unsafe {
            let queue = self;
            for item in items {
                item.item().client = held.clone();
                item.item().queue = Some(queue.downgrade());
                log_debug(
                    c"%s %s: %s",
                    fmt_args![c"cmdq_append", name, item.item().name.as_deref()],
                );
                queue.append_item(item);
            }
            queue.item_at(queue.len().saturating_sub(1))
        }
    }
    pub unsafe fn run(&self, name: &CStr) -> u_int {
        unsafe {
            let queue = self;
            let mut items: u_int = 0;
            static NUMBER: AtomicU32 = AtomicU32::new(0);
            if queue.is_empty() {
                log_debug(c"%s %s: empty", fmt_args![c"cmdq_next", name]);
                return 0;
            }
            if queue
                .item_at(0)
                .expect("the queue is not empty")
                .item()
                .flags
                & CMDQ_WAITING
                != 0
            {
                log_debug(c"%s %s: waiting", fmt_args![c"cmdq_next", name]);
                return 0;
            }
            log_debug(c"%s %s: enter", fmt_args![c"cmdq_next", name]);
            loop {
                let Some(fired) = queue.item_at(0) else {
                    queue.set_running(false);
                    log_debug(c"%s %s: exit (empty)", fmt_args![c"cmdq_next", name]);
                    return items;
                };
                queue.set_running(true);
                let (is_command, flags) = fired.with_item(|item| {
                    let is_command = matches!(item.type_0, CmdqType::Command { .. });
                    log_debug(
                        c"%s %s: %s (%d), flags %x",
                        fmt_args![
                            c"cmdq_next",
                            name,
                            item.name.as_deref(),
                            if is_command { 0u32 } else { 1u32 },
                            item.flags
                        ],
                    );
                    (is_command, item.flags)
                });
                if flags & CMDQ_WAITING != 0 {
                    break;
                }
                if flags & CMDQ_FIRED == 0 {
                    {
                        let mut item = fired.item();
                        item.time = time(core::ptr::null_mut::<time_t>());
                        item.number = NUMBER.fetch_add(1, Ordering::Relaxed).wrapping_add(1);
                    }
                    let retval = if is_command {
                        let retval = fired.fire_command();
                        if retval == CMD_RETURN_ERROR {
                            cmdq_remove_group(&mut fired.item());
                        }
                        retval
                    } else {
                        fired.fire_callback()
                    };
                    let mut item = fired.item();
                    item.flags |= CMDQ_FIRED;
                    if retval == CMD_RETURN_WAIT {
                        item.flags |= CMDQ_WAITING;
                        break;
                    }
                    items = items.wrapping_add(1);
                }
                cmdq_remove(&mut fired.item());
            }
            log_debug(c"%s %s: exit (wait)", fmt_args![c"cmdq_next", name]);
            items
        }
    }
    pub fn running_item(&self) -> Option<CmdqItemRef> {
        let queue = self;
        if !queue.is_running() {
            return None;
        }
        let item = queue.item_at(0)?;
        if item.with_item(|item| item.flags & CMDQ_WAITING != 0) {
            return None;
        }
        Some(item)
    }
}

impl CmdqItemRef {
    pub unsafe fn insert_hook(
        &self,
        s: Option<&session>,
        current: Option<&cmd_find_state>,
        fmt: &CStr,
        args: &[FmtArg],
    ) {
        let item = self;

        unsafe {
            let mut after = item.clone();
            let (state, list, at) = item.with_item(|item| {
                let (list, at) = item
                    .command_location()
                    .expect("a hook parent runs a command");
                (item.state_ref(), list.clone(), at)
            });
            let cmd = list
                .command(at)
                .expect("the hook parent command is in its list");
            let args_0: &args = cmd_get_args(&cmd);
            let mut i: u_int;
            let event = state.state().event.clone();
            let new_state = CmdqStateRef::create(current, Some(&event), CMDQ_STATE_NOHOOKS);
            if state.state().flags & CMDQ_STATE_NOHOOKS != 0 {
                return;
            }
            let oo = match s {
                Some(s) => (s).options_ref().clone(),
                None => global_s_options
                    .as_ref()
                    .expect("global options are initialized")
                    .clone(),
            };
            let name = format_alloc(fmt, args);
            let commands = oo.with_entry(&name, false, |entry| {
                let entry = entry?;
                Some(
                    RustOptionsEngine
                        .array_indices(entry)
                        .into_iter()
                        .filter_map(|index| {
                            RustOptionsEngine
                                .array_get(entry, index)
                                .and_then(|value| RustOptionsEngine.value_command(value))
                        })
                        .collect::<Vec<_>>(),
                )
            });
            let Some(commands) = commands else { return };
            log_debug(
                c"running hook %s (parent %p)",
                fmt_args![name.as_c_str(), item.as_ptr()],
            );
            new_state.add_format(c"hook", c"%s", fmt_args![name.as_c_str()]);
            let arguments = args_print(args_0);
            new_state.add_format(c"hook_arguments", c"%s", fmt_args![arguments.as_c_str()]);
            i = 0 as u_int;
            while i < args_count(args_0) {
                let tmp = xasprintf(c"hook_argument_%d", fmt_args![i]);
                new_state.add_format(
                    &tmp,
                    c"%s",
                    fmt_args![args_string_str(args_0, i).expect("argument index checked")],
                );
                i = i.wrapping_add(1);
            }
            for flag in args_flags(args_0).map(|flag| flag as core::ffi::c_char) {
                let tmp = xasprintf(c"hook_flag_%c", fmt_args![flag as core::ffi::c_int]);
                match args_get_str(args_0, flag as u_char) {
                    None => new_state.add_format(&tmp, c"1", fmt_args![]),
                    Some(value) => new_state.add_format(&tmp, c"%s", fmt_args![value]),
                }
                i = 0 as u_int;
                for av in args_value_list(args_0, flag as u_char) {
                    let tmp = xasprintf(c"hook_flag_%c_%d", fmt_args![flag as core::ffi::c_int, i]);
                    new_state.add_format(&tmp, c"%s", fmt_args![av.value.string()]);
                    i = i.wrapping_add(1);
                }
            }
            for cmdlist in commands {
                let queued = cmdlist.queue_items(Some(&new_state));
                after = after.insert_after(queued);
            }
        }
    }
}

impl cmdq_item {
    /// The name the item was queued under, or nothing before it has been given
    /// one.
    pub fn name(&self) -> Option<&CStr> {
        let item = self;

        item.name.as_deref()
    }
    pub fn client(&self) -> Option<ClientRef> {
        let item = self;

        item.client.clone()
    }
    /// Makes `tc` the client the item's target was found against.
    pub fn set_target_client(&mut self, tc: Option<&ClientRef>) {
        let item = self;

        item.target_client = tc.map(ClientRef::downgrade);
    }
    pub fn target_client(&self) -> Option<ClientRef> {
        let item = self;

        item.target_client.as_ref().and_then(ClientWeak::upgrade)
    }
    pub fn target(&self) -> &cmd_find_state {
        let item = self;

        &item.target
    }
    pub fn event(&self) -> RefMut<'_, key_event> {
        let item = self;

        let state = item
            .state_ref
            .as_ref()
            .expect("a queue item without a state");
        state.event()
    }
    pub fn current(&self) -> RefMut<'_, cmd_find_state> {
        let item = self;

        let state = item
            .state_ref
            .as_ref()
            .expect("a queue item without a state");
        state.current()
    }
    pub fn flags(&self) -> core::ffi::c_int {
        let item = self;

        item.state_ref().state().flags
    }
    pub fn merge_formats(&self, ft: &mut format_tree) {
        let item = self;

        {
            if let Some(cmd) = item.command() {
                let entry = cmd_get_entry(&cmd);
                format_add(ft, c"command", c"%s", fmt_args![entry.name]);
            }
            if let Some(state) = item.state_ref.as_ref()
                && let Some(formats) = state.state().formats.as_deref()
            {
                format_merge(ft, formats);
            }
        }
    }
    pub unsafe fn guard(&self, guard: &CStr, flags: core::ffi::c_int) {
        let item = self;

        unsafe {
            let mut c = item.client();
            let t: core::ffi::c_long = item.time as core::ffi::c_long;
            let number: u_int = item.number;
            if let Some(c) = c.as_mut()
                && c.flags() & CLIENT_CONTROL as uint64_t != 0
            {
                control_write(
                    c.as_client_mut(),
                    c"%%%s %ld %u %d",
                    fmt_args![guard, t, number, flags],
                );
            }
        }
    }
    pub unsafe fn print_data(&self, evb: &mut ByteBuffer) {
        let item = self;

        unsafe {
            server_client_print(
                item.client()
                    .as_mut()
                    .map(|reference| reference.as_client_mut()),
                1 as core::ffi::c_int,
                evb,
            );
        }
    }
    pub unsafe fn print(&self, fmt: &CStr, args: &[FmtArg]) {
        let item = self;

        unsafe {
            let mut evb = ByteBuffer::new();
            format_buf(&mut evb, fmt, args);
            item.print_data(&mut evb);
        }
    }
    pub unsafe fn error(&self, fmt: &CStr, args: &[FmtArg]) {
        let item = self;

        unsafe {
            let mut c = item.client();
            let mut msg = format_alloc(fmt, args);
            log_debug(c"%s: %s", fmt_args![c"cmdq_error", msg.as_c_str()]);
            let Some(c) = c.as_mut() else {
                let cmd = item
                    .command()
                    .expect("a configuration error belongs to a command");
                let (file, line) = cmd_get_source(&cmd);
                cfg_add_cause(c"%s:%u: %s", fmt_args![file, line, msg.as_c_str()]);
                return;
            };
            if c.attached_session().is_none() || c.flags() & CLIENT_CONTROL as uint64_t != 0 {
                server_add_message(c"%s message: %s", fmt_args![c.name(), msg.as_c_str()]);
                if !c.flags() & CLIENT_UTF8 as uint64_t != 0 {
                    msg = RustUtf8VisModel.sanitize(&msg);
                }
                if c.flags() & CLIENT_CONTROL as uint64_t != 0 {
                    control_write(c.as_client_mut(), c"%s", fmt_args![msg.as_c_str()]);
                } else {
                    file_error(Some(c.as_client_mut()), c"%s\n", fmt_args![msg.as_c_str()]);
                }
                *c.retval_mut() = 1 as core::ffi::c_int;
            } else {
                let mut bytes = msg.into_bytes();
                if let Some(first) = bytes.first_mut() {
                    *first = toupper(*first);
                }
                msg = CString::new(bytes).expect("a message holds no NUL");
                status_message_set(
                    Some(c.as_client_mut()),
                    -(1 as core::ffi::c_int),
                    1 as core::ffi::c_int,
                    0 as core::ffi::c_int,
                    0 as core::ffi::c_int,
                    c"%s",
                    fmt_args![msg.as_c_str()],
                );
            }
        }
    }
}

impl CmdqItemRef {
    unsafe fn hook_target(&self) -> Option<cmd_find_state> {
        let item = self;

        unsafe {
            let (target, state, client) =
                item.with_item(|item| (item.target.clone(), item.state_ref(), item.client()));
            let current = state.state().current.clone();
            if cmd_find_valid_state(&target) != 0 {
                return Some(target);
            }
            if cmd_find_valid_state(&current) != 0 {
                return Some(current);
            }
            let mut target = cmd_find_state::default();
            (cmd_find_from_client(&mut target, client.as_ref(), 0) == 0).then_some(target)
        }
    }
    /// Prepares a command and routes its hooks through checked item borrows.
    unsafe fn fire_command(&self) -> cmd_retval {
        let fired = self;

        unsafe {
            let (list, at, saved, group) = fired.with_item(|item| {
                let (list, at) = item
                    .command_location()
                    .expect("the fired item runs a command");
                (list.clone(), at, item.client.clone(), item.group)
            });
            let name = cmdq_name(saved.as_ref());
            let cmd = list.command(at).expect("the fired command is in its list");
            let args = cmd_get_args(&cmd);
            let entry = cmd_get_entry(&cmd);
            if cfg_finished != 0 {
                fired.with_item(|item| cmdq_add_message(item));
            }
            if log_get_level() > 1 {
                let printed = cmd_print(&cmd);
                log_debug(
                    c"%s %s: (%u) %s",
                    fmt_args![
                        c"cmdq_fire_command",
                        name.as_c_str(),
                        group,
                        printed.as_c_str()
                    ],
                );
            }
            let flags = (fired.flags() & CMDQ_STATE_CONTROL != 0) as core::ffi::c_int;
            fired.guard(c"begin", flags);
            let mut retval = {
                let mut item = fired.item();
                if item.client.is_none() {
                    item.client = cmd_find_client(Some(&item), None, 1);
                }
                let quiet = (entry.flags() & CMD_CLIENT_CANFAIL != 0) as core::ffi::c_int;
                let target_client = if entry.flags() & CMD_CLIENT_CFLAG != 0 {
                    cmd_find_client(Some(&item), args_get_str(args, b'c'), quiet)
                } else if entry.flags() & CMD_CLIENT_TFLAG != 0 {
                    cmd_find_client(Some(&item), args_get_str(args, b't'), quiet)
                } else {
                    cmd_find_client(Some(&item), None, 1)
                };
                if target_client.is_none()
                    && quiet == 0
                    && entry.flags() & (CMD_CLIENT_CFLAG | CMD_CLIENT_TFLAG) != 0
                {
                    CMD_RETURN_ERROR
                } else {
                    item.set_target_client(target_client.as_ref());
                    let (result, source) = cmdq_find_flag(&item, &entry.source());
                    item.source = source;
                    if result == CMD_RETURN_ERROR {
                        result
                    } else {
                        let (result, target) = cmdq_find_flag(&item, &entry.target());
                        item.target = target;
                        result
                    }
                }
            };
            if retval != CMD_RETURN_ERROR {
                retval = {
                    let item = fired.read();
                    let mut context = RustCommandContext::new(&cmd, &item);
                    CommandEntry::execute(entry, &mut context).into_raw()
                };
                if retval != CMD_RETURN_ERROR
                    && entry.flags() & CMD_AFTERHOOK != 0
                    && let Some(target) = fired.hook_target()
                {
                    let session = target.session();
                    fired.insert_hook(
                        session.as_ref().map(|session| session.as_session()),
                        Some(&target),
                        c"after-%s",
                        fmt_args![entry.name()],
                    );
                }
            }
            fired.item().client = saved;
            if retval == CMD_RETURN_ERROR {
                let target = fired.hook_target();
                let session = target.as_ref().and_then(|target| target.session());
                fired.insert_hook(
                    session.as_ref().map(|session| session.as_session()),
                    target.as_ref(),
                    c"command-error",
                    fmt_args![],
                );
                fired.guard(c"error", flags);
            } else {
                fired.guard(c"end", flags);
            }
            retval
        }
    }
    pub fn callback_items(
        name: &CStr,
        callback: impl FnOnce(&CmdqItemRef) -> cmd_retval + 'static,
    ) -> cmdq_items {
        unsafe {
            let state = CmdqStateRef::create(None, None, 0 as core::ffi::c_int);
            let item = CmdqItemRef::from_type(
                CmdqType::Callback {
                    callback: Some(Box::new(callback)),
                },
                state,
            );
            item.item().name = Some(xasprintf(c"[%s/%p]", fmt_args![name, item.as_ptr()]));
            item.item().group = 0 as u_int;
            vec![item]
        }
    }
    pub fn error_items(error: &CStr) -> cmdq_items {
        let error = error.to_owned();
        CmdqItemRef::callback_items(c"cmdq_error_callback", move |item| {
            cmdq_error_callback(item, error)
        })
    }
}

impl CmdqStateRef {
    pub fn event(&self) -> RefMut<'_, key_event> {
        RefMut::map(self.state(), |state| &mut state.event)
    }
    pub fn current(&self) -> RefMut<'_, cmd_find_state> {
        RefMut::map(self.state(), |state| &mut state.current)
    }
}
impl CmdqItemRef {
    pub fn name(&self) -> Option<CString> {
        self.with_item(|item| item.name().map(CStr::to_owned))
    }
    pub fn client(&self) -> Option<ClientRef> {
        self.with_item(cmdq_item::client)
    }
    pub fn target_client(&self) -> Option<ClientRef> {
        self.with_item(cmdq_item::target_client)
    }
    pub fn target(&self) -> cmd_find_state {
        self.with_item(|item| item.target().clone())
    }
    pub fn flags(&self) -> core::ffi::c_int {
        self.with_item(cmdq_item::flags)
    }
    pub fn set_target_client(&self, client: Option<&ClientRef>) {
        self.item().set_target_client(client)
    }
    pub fn event_snapshot(&self) -> key_event {
        self.state_ref().event().clone()
    }
    pub fn current_snapshot(&self) -> cmd_find_state {
        self.state_ref().current().clone()
    }
    pub fn merge_formats(&self, ft: &mut format_tree) {
        self.with_item(|item| item.merge_formats(ft))
    }
    pub unsafe fn guard(&self, guard: &CStr, flags: core::ffi::c_int) {
        unsafe { self.with_item(|item| item.guard(guard, flags)) }
    }
    pub unsafe fn print_data(&self, buffer: &mut ByteBuffer) {
        unsafe { self.with_item(|item| item.print_data(buffer)) }
    }
    pub unsafe fn print(&self, fmt: &CStr, args: &[FmtArg]) {
        unsafe { self.with_item(|item| item.print(fmt, args)) }
    }
    pub unsafe fn error(&self, fmt: &CStr, args: &[FmtArg]) {
        unsafe { self.with_item(|item| item.error(fmt, args)) }
    }
}

impl CmdqStateRef {
    /// Returns an owned event observation without retaining queue storage access.
    pub(crate) fn event_snapshot(&self) -> key_event {
        self.state().event.clone()
    }

    /// Returns the current resolved target without retaining queue storage access.
    pub(crate) fn current_snapshot(&self) -> cmd_find_state {
        self.state().current.clone()
    }

    /// Replaces the current target with the caller's resolved snapshot.
    pub(crate) fn replace_current(&self, current: cmd_find_state) {
        self.state().current = current;
    }

    /// Resolves and records the session's current target at the point of this call.
    ///
    /// # Safety
    /// The session must have a current live link/pane. Exclude conflicting target,
    /// session, window and queue state access. No callbacks are dispatched.
    pub(crate) unsafe fn update_current_session(
        &self,
        session: &SessionRef,
        flags: core::ffi::c_int,
    ) {
        unsafe {
            super::find::cmd_find_from_session_ref(&mut self.state().current, session, flags)
        };
    }

    /// Records a live link target, using its active pane if no pane is specified.
    ///
    /// # Safety
    /// The link and optional pane must be live and mutually consistent. Exclude
    /// conflicting target/session/window/pane/queue access. No callbacks run.
    pub(crate) unsafe fn update_current_link(
        &self,
        link: &crate::window::WinlinkRef,
        pane: Option<&RustWindowPaneWeak>,
        flags: core::ffi::c_int,
    ) {
        unsafe {
            super::find::cmd_find_from_link_ref(&mut self.state().current, link, pane, flags)
        };
    }

    /// Gives one synchronous command operation access to its mutable mouse input.
    /// The callback must not borrow this queue state again or retain the reference.
    pub(crate) fn with_mouse_event<R>(&self, operation: impl FnOnce(&mut mouse_event) -> R) -> R {
        operation(&mut self.state().event.m)
    }
}

impl CmdqItemRef {
    /// Inserts a session hook using the existing order, target snapshot and nested
    /// suppression rules. An absent session retains the global hook fallback.
    ///
    /// # Safety
    /// Run on the server thread without conflicting session/option/queue storage
    /// access. The current target must satisfy the existing hook target contract;
    /// hook execution is queued, not dispatched inline.
    pub(crate) unsafe fn insert_session_hook(
        &self,
        session: Option<&SessionRef>,
        current: Option<&cmd_find_state>,
        fmt: &CStr,
        args: &[FmtArg],
    ) {
        unsafe {
            self.insert_hook(
                session.map(|session| session.as_session()),
                current,
                fmt,
                args,
            )
        };
    }
}

#[cfg(test)]
pub use crate::consts::CMD_FIND_PANE;
