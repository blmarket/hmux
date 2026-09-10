//! All mutable application globals belong to this thread's process owner.
//! Fields have independent cells so callbacks may access unrelated state without
//! borrowing the entire server. The owner also exists during client startup.
use crate::client::client_exit_reason;
use crate::environ::{EnvironmentStore, RustEnvironment};
use crate::reactor::{IoWatch, Timer};
use crate::types::*;
use std::cell::{Cell, LazyCell, OnceCell, RefCell};
use std::collections::{BTreeMap, HashMap};
use std::ffi::{CString, c_int};
use std::fs::File;
use std::rc::Rc;
use std::time::Instant;

thread_local! {
    pub(crate) static server_proc: ServerState = ServerState::new();
}

pub(crate) struct ServerState {
    #[cfg(test)]
    pub(crate) file_test_events: RefCell<Vec<(i32, Vec<u8>, Vec<u8>)>>,
    #[cfg(test)]
    pub(crate) prompt_test_answers: RefCell<Vec<(String, c_int)>>,
    pub(crate) citem_pool: Rc<RefCell<crate::screen::write::CItemPool>>,
    pub(crate) clients: RefCell<clients_t>,
    pub(crate) process: RefCell<Option<ProcessRef>>,
    pub(crate) server_fd: RefCell<c_int>,
    pub(crate) server_client_flags: RefCell<uint64_t>,
    pub(crate) server_exit: RefCell<c_int>,
    pub(crate) server_ev_accept: RefCell<IoHandle>,
    pub(crate) server_ev_accept_timer: RefCell<TimerHandle>,
    pub(crate) server_ev_tidy: RefCell<TimerHandle>,
    pub(crate) marked_pane: RefCell<cmd_find_state>,
    pub(crate) current_time: RefCell<time_t>,
    pub(crate) server_last_attached: RefCell<c_int>,
    pub(crate) server_acl: Rc<RefCell<crate::server::acl::RustServerAclStore>>,
    pub(crate) client_flags_timer: RefCell<TimerHandle>,
    pub(crate) paste_buffers: RefCell<crate::paste::RustPasteBufferStore>,
    pub(crate) input_buffer_size: RefCell<size_t>,
    pub(crate) input_cell_buffer: RefCell<utf8_data>,
    pub(crate) input_keys: OnceCell<Rc<BTreeMap<key_code, Rc<CString>>>>,
    pub(crate) windows: crate::handle_registry::HandleRegistry<WindowWeak>,
    pub(crate) global_pane_index: crate::window::GlobalPaneIndex,
    pub(crate) next_window_pane_id: Cell<Option<u_int>>,
    pub(crate) next_window_id: Cell<Option<u_int>>,
    pub(crate) next_active_point: Cell<u_int>,
    pub(crate) file_next_stream: RefCell<c_int>,
    pub(crate) alerts_fired: RefCell<c_int>,
    pub(crate) alerts_list: Rc<crate::tree::GlobalQueue<WindowRef>>,
    pub(crate) runtime_control: crate::reactor::registry::RuntimeControl,
    pub(crate) stream_registry: crate::reactor::stream::StreamRegistry,
    pub(crate) runtime_host: RefCell<crate::reactor::runtime::RuntimeHost>,
    pub(crate) client_proc: RefCell<Option<ProcessRef>>,
    pub(crate) client_peer: RefCell<Option<PeerRef>>,
    pub(crate) client_flags: RefCell<uint64_t>,
    pub(crate) client_suspended: RefCell<c_int>,
    pub(crate) client_exitreason: RefCell<client_exit_reason>,
    pub(crate) client_exitflag: RefCell<c_int>,
    pub(crate) client_exitval: RefCell<c_int>,
    pub(crate) client_exittype: RefCell<msgtype>,
    pub(crate) client_exitsession: RefCell<Option<CString>>,
    pub(crate) client_exitmessage: RefCell<Option<CString>>,
    pub(crate) client_execshell: RefCell<Option<CString>>,
    pub(crate) client_execcmd: RefCell<Option<CString>>,
    pub(crate) client_attached: RefCell<c_int>,
    pub(crate) client_files: Rc<crate::tree::GlobalTree<c_int, ClientFileRef>>,
    pub(crate) tty_terms: crate::terminfo::term::TerminalRegistry,
    pub(crate) global_environment: RefCell<RustEnvironment>,
    pub(crate) bsdopterr: RefCell<c_int>,
    pub(crate) bsdoptind: RefCell<c_int>,
    pub(crate) bsdoptopt: RefCell<c_int>,
    pub(crate) bsdoptreset: RefCell<c_int>,
    pub(crate) optarg: RefCell<Option<crate::compat::getopt_long::ArgumentPosition>>,
    pub(crate) place: RefCell<Option<crate::compat::getopt_long::ArgumentPosition>>,
    pub(crate) nonopt_start: RefCell<c_int>,
    pub(crate) nonopt_end: RefCell<c_int>,
    pub(crate) getopt_posixly_correct: RefCell<c_int>,
    pub(crate) cmd_list_next_group: Cell<u32>,
    pub(crate) wait_channels:
        RefCell<BTreeMap<CString, crate::cmd::entries::cmd_wait_for::WaitChannel>>,
    pub(crate) cmd_source_file_depth: Cell<u32>,
    pub(crate) global_queue: Rc<crate::tree::GlobalQueue<CmdqListRef>>,
    pub(crate) cmdq_next_number: Cell<u32>,
    pub(crate) cached_user: OnceCell<CString>,
    pub(crate) format_jobs: Rc<
        crate::tree::GlobalTree<
            crate::format::expand::format_job_key,
            Box<crate::format::expand::format_job>,
        >,
    >,
    pub(crate) all_jobs: Rc<crate::tree::GlobalQueue<Box<crate::job::job>>>,
    pub(crate) next_job_id: Cell<Option<u_int>>,
    pub(crate) log_file: Rc<RefCell<Option<File>>>,
    pub(crate) log_level: Cell<i32>,
    pub(crate) message_log: Rc<RefCell<crate::message_log::RustMessageLog>>,
    pub(crate) sessions: Rc<crate::tree::GlobalTree<CString, SessionRef>>,
    pub(crate) next_session_id: Cell<Option<u_int>>,
    pub(crate) session_groups: RefCell<session_groups_t>,
    pub(crate) plugin_revisions: RefCell<HashMap<u_int, u64>>,
    pub(crate) plugins: RefCell<Vec<crate::plugin::Slot>>,
    pub(crate) plugin_tick: RefCell<TimerHandle>,
    pub(crate) timer_start: LazyCell<Instant>,
    pub(crate) cached_home: OnceCell<CString>,
    pub(crate) global_options: RefCell<Option<RustOptionsRef>>,
    pub(crate) global_s_options: RefCell<Option<RustOptionsRef>>,
    pub(crate) global_w_options: RefCell<Option<RustOptionsRef>>,
    pub(crate) start_time: RefCell<timeval>,
    pub(crate) socket_path: RefCell<Option<CString>>,
    pub(crate) ptm_fd: RefCell<c_int>,
    pub(crate) shell_command: RefCell<Option<CString>>,
    pub(crate) next_client_id: Cell<u64>,
    pub(crate) utf8_width_cache: RefCell<BTreeMap<wchar_t, u_int>>,
    pub(crate) utf8_no_width: Cell<bool>,
    pub(crate) utf8_store: Rc<RefCell<crate::text::utf8::Utf8Store>>,
    pub(crate) key_tables: RefCell<BTreeMap<CString, crate::key_bindings::KeyTableRef>>,
    pub(crate) prompt_history: Rc<RefCell<crate::prompt_history::RustPromptHistoryStore>>,
    pub(crate) hyperlink_registry: Rc<RefCell<crate::grid::links::HyperlinkRegistry>>,
    pub(crate) tty_log_fd: RefCell<c_int>,
    pub(crate) tty_string_buffer: RefCell<[u_char; 500]>,
}
impl ServerState {
    fn new() -> Self {
        let runtime_control = crate::reactor::registry::RuntimeControl::new();
        let stream_registry = crate::reactor::stream::StreamRegistry::new();
        let runtime_host = RefCell::new(crate::reactor::runtime::RuntimeHost::new(
            runtime_control.clone(),
            stream_registry.clone(),
        ));
        Self {
            #[cfg(test)]
            file_test_events: RefCell::new(Vec::new()),
            #[cfg(test)]
            prompt_test_answers: RefCell::new(Vec::new()),
            citem_pool: Rc::new(RefCell::new(crate::screen::write::CItemPool::new())),
            clients: const { RefCell::new(Vec::new()) },
            process: RefCell::new(None),
            server_fd: RefCell::new(-1),
            server_client_flags: RefCell::new(0),
            server_exit: RefCell::new(0),
            server_ev_accept: RefCell::new(IoHandle::ZERO),
            server_ev_accept_timer: RefCell::new(TimerHandle::ZERO),
            server_ev_tidy: RefCell::new(TimerHandle::ZERO),
            marked_pane: RefCell::new(cmd_find_state {
                flags: 0,
                s_ref: None,
                wl_idx: None,
                w_ref: None,
                wp_ref: None,
                idx: 0,
            }),
            current_time: RefCell::new(0),
            server_last_attached: RefCell::new(-1),
            server_acl: Rc::new(RefCell::new(crate::server::acl::RustServerAclStore::empty())),
            client_flags_timer: RefCell::new(TimerHandle::ZERO),
            paste_buffers: const { RefCell::new(crate::paste::RustPasteBufferStore::server()) },
            input_buffer_size: RefCell::new(crate::input::parser::INPUT_BUF_DEFAULT_SIZE as size_t),
            input_cell_buffer: RefCell::new(utf8_data {
                data: *b"\xEF\xBF\xBD\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                have: 3 as u_char,
                size: 3 as u_char,
                width: 1 as u_char,
            }),
            input_keys: OnceCell::new(),
            windows: crate::handle_registry::HandleRegistry::new(),
            global_pane_index: crate::window::GlobalPaneIndex::new(),
            next_window_pane_id: const { Cell::new(Some(0)) },
            next_window_id: const { Cell::new(Some(0)) },
            next_active_point: const { Cell::new(0) },
            file_next_stream: RefCell::new(3),
            alerts_fired: RefCell::new(0),
            alerts_list: Rc::new(crate::tree::GlobalQueue::new()),
            runtime_control,
            stream_registry,
            runtime_host,
            client_proc: RefCell::new(None),
            client_peer: RefCell::new(None),
            client_flags: RefCell::new(0),
            client_suspended: RefCell::new(0),
            client_exitreason: RefCell::new(crate::client::CLIENT_EXIT_NONE),
            client_exitflag: RefCell::new(0),
            client_exitval: RefCell::new(0),
            client_exittype: RefCell::new(0),
            client_exitsession: RefCell::new(None),
            client_exitmessage: RefCell::new(None),
            client_execshell: RefCell::new(None),
            client_execcmd: RefCell::new(None),
            client_attached: RefCell::new(0),
            client_files: Rc::new(crate::tree::GlobalTree::new()),
            tty_terms: crate::terminfo::term::TerminalRegistry::default(),
            global_environment: RefCell::new(RustEnvironment::empty()),
            bsdopterr: RefCell::new(1),
            bsdoptind: RefCell::new(1),
            bsdoptopt: RefCell::new('?' as c_int),
            bsdoptreset: RefCell::new(0),
            optarg: RefCell::new(None),
            place: RefCell::new(None),
            nonopt_start: RefCell::new(-1),
            nonopt_end: RefCell::new(-1),
            getopt_posixly_correct: RefCell::new(-1),
            cmd_list_next_group: Cell::new(1),
            wait_channels: const { RefCell::new(BTreeMap::new()) },
            cmd_source_file_depth: Cell::new(0),
            global_queue: Rc::new(crate::tree::GlobalQueue::new()),
            cmdq_next_number: Cell::new(0),
            cached_user: OnceCell::new(),
            format_jobs: Rc::new(crate::tree::GlobalTree::new()),
            all_jobs: Rc::new(crate::tree::GlobalQueue::new()),
            next_job_id: const { Cell::new(Some(0)) },
            log_file: Rc::new(RefCell::new(None)),
            log_level: Cell::new(0),
            message_log: Rc::new(RefCell::new(crate::message_log::RustMessageLog::empty())),
            sessions: Rc::new(crate::tree::GlobalTree::new()),
            next_session_id: const { Cell::new(Some(0)) },
            session_groups: const { RefCell::new(BTreeMap::new()) },
            plugin_revisions: RefCell::new(HashMap::new()),
            plugins: const { RefCell::new(Vec::new()) },
            plugin_tick: const { RefCell::new(TimerHandle::ZERO) },
            timer_start: LazyCell::new(Instant::now),
            cached_home: OnceCell::new(),
            global_options: RefCell::new(None),
            global_s_options: RefCell::new(None),
            global_w_options: RefCell::new(None),
            start_time: RefCell::new(timeval {
                tv_sec: 0,
                tv_usec: 0,
            }),
            socket_path: RefCell::new(None),
            ptm_fd: RefCell::new(-1),
            shell_command: RefCell::new(None),
            next_client_id: const { Cell::new(0) },
            utf8_width_cache: const { RefCell::new(BTreeMap::new()) },
            utf8_no_width: const { Cell::new(false) },
            utf8_store: Rc::new(RefCell::new(crate::text::utf8::Utf8Store::new())),
            key_tables: const { RefCell::new(BTreeMap::new()) },
            prompt_history: Rc::new(RefCell::new(
                crate::prompt_history::RustPromptHistoryStore::empty(),
            )),
            hyperlink_registry: Rc::new(RefCell::new(crate::grid::links::HyperlinkRegistry::new())),
            tty_log_fd: RefCell::new(-1),
            tty_string_buffer: RefCell::new([0; 500]),
        }
    }
}

/// A storage-free accessor for one field of the process owner.
pub struct LocalField<T: 'static> {
    field: fn(&ServerState) -> &T,
}
impl<T> Copy for LocalField<T> {}
impl<T> Clone for LocalField<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> LocalField<T> {
    pub(crate) const fn new(field: fn(&ServerState) -> &T) -> Self {
        Self { field }
    }
    pub fn try_with<R>(&self, f: impl FnOnce(&T) -> R) -> Result<R, std::thread::AccessError> {
        server_proc.try_with(|state| f((self.field)(state)))
    }
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        server_proc.with(|state| f((self.field)(state)))
    }
}
impl<T> LocalField<Rc<T>> {
    pub fn get(&self) -> Rc<T> {
        self.with(Clone::clone)
    }
}
impl<T> LocalField<RefCell<T>> {
    pub fn with_borrow<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        self.with(|v| f(&v.borrow()))
    }
    pub fn with_borrow_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        self.with(|v| f(&mut v.borrow_mut()))
    }
    pub fn replace(&self, value: T) -> T {
        self.with(|v| v.replace(value))
    }
    pub fn set(&self, value: T) {
        drop(self.replace(value));
    }
}
impl<T: Copy> LocalField<Cell<T>> {
    pub fn get(&self) -> T {
        self.with(Cell::get)
    }
    pub fn set(&self, value: T) {
        self.with(|v| v.set(value));
    }
    pub fn replace(&self, value: T) -> T {
        self.with(|v| v.replace(value))
    }
}

/// An accessor for a former `static mut`, with checked, short-lived borrows.
pub struct Value<T: 'static>(LocalField<RefCell<T>>);
impl<T> Value<T> {
    pub(crate) const fn new(field: fn(&ServerState) -> &RefCell<T>) -> Self {
        Self(LocalField::new(field))
    }
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.0.with_borrow(Clone::clone)
    }
    pub fn set(&self, value: T) {
        self.0.set(value);
    }
    pub fn replace(&self, value: T) -> T {
        self.0.replace(value)
    }
    pub fn take(&self) -> T
    where
        T: Default,
    {
        self.0.with_borrow_mut(std::mem::take)
    }
    pub fn try_with<R>(&self, f: impl FnOnce(&T) -> R) -> Result<R, std::thread::AccessError> {
        self.0.try_with(|value| f(&value.borrow()))
    }
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        self.0.with_borrow(f)
    }
    pub fn with_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        self.0.with_borrow_mut(f)
    }
}

#[cfg(test)]
mod tests {
    use super::server_proc;

    #[test]
    fn each_process_thread_has_independent_mutable_state() {
        server_proc.with(|state| {
            *state.client_flags.borrow_mut() = 17;
            *state.bsdoptind.borrow_mut() = 4;
            state.log_level.set(2);
            // A callback may reach a different field while this one is borrowed.
            let _clients = state.clients.borrow_mut();
            crate::client::client_flags.set(18);
        });

        std::thread::spawn(|| {
            server_proc.with(|state| {
                assert_eq!(*state.client_flags.borrow(), 0);
                assert_eq!(*state.bsdoptind.borrow(), 1);
                assert_eq!(state.log_level.get(), 0);
                *state.client_flags.borrow_mut() = 99;
                *state.bsdoptind.borrow_mut() = 9;
                state.log_level.set(3);
            });
        })
        .join()
        .unwrap();

        server_proc.with(|state| {
            assert_eq!(*state.client_flags.borrow(), 18);
            assert_eq!(*state.bsdoptind.borrow(), 4);
            assert_eq!(state.log_level.get(), 2);
        });
    }
}
