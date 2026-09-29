use super::*;
use hmux_buffer::SegmentedBuf;

type ClientFileIndex =
    std::collections::BTreeMap<i32, std::rc::Rc<std::cell::UnsafeCell<client_file>>>;

#[derive(Default)]
#[repr(C)]
pub struct client_files {
    /// The stream index owns active file records until completion unlinks them.
    pub(super) storage: Option<refbox::RefBox<ClientFileIndex>>,
}

pub struct client_file {
    pub(super) c: Option<Rc<UnsafeCell<client>>>,
    pub(super) peer: *mut tmuxpeer,
    pub(super) stream: ::core::ffi::c_int,
    pub(super) path: Option<std::ffi::CString>,
    pub(super) buffer: Box<SegmentedBuf>,
    pub(super) event: crate::src::reactor::StreamHandle,
    pub(super) fd: ::core::ffi::c_int,
    pub(super) error: ::core::ffi::c_int,
    pub(super) closed: ::core::ffi::c_int,
    pub(super) cb: client_file_cb,
    pub(super) entry: client_file_entry,
    pub(super) wait_item:
        std::rc::Weak<std::cell::UnsafeCell<crate::src::shared::command::cmdq_item>>,
    pub(super) wait_active: bool,
    pub(super) wait_client: std::rc::Weak<std::cell::UnsafeCell<client>>,
    pub(super) cancel_data: Option<Box<dyn FnOnce()>>,
    pub(super) terminal_scheduled: bool,
    pub(super) read: super::stream::ReadState,
}

pub struct client_file_event<'a> {
    /// Borrowed retained client for this callback; clone to retain it afterward.
    pub client: Option<&'a std::rc::Rc<std::cell::UnsafeCell<client>>>,
    pub path: Option<&'a CStr>,
    pub error: i32,
    pub closed: bool,
    pub buffer: Option<&'a mut SegmentedBuf>,
}

impl client_files {
    pub const fn new() -> Self {
        Self { storage: None }
    }
}

impl client_file {
    pub(super) fn new() -> std::rc::Rc<std::cell::UnsafeCell<Self>> {
        std::rc::Rc::new(std::cell::UnsafeCell::new(Self::empty()))
    }

    pub(super) fn empty() -> Self {
        Self {
            c: Default::default(),
            peer: Default::default(),
            stream: Default::default(),
            path: Default::default(),
            buffer: Default::default(),
            event: Default::default(),
            fd: Default::default(),
            error: Default::default(),
            closed: Default::default(),
            cb: Default::default(),
            entry: client_file_entry {
                owner: refbox::Weak::new(),
            },
            wait_item: Default::default(),
            wait_active: false,
            wait_client: Default::default(),
            cancel_data: Default::default(),
            terminal_scheduled: Default::default(),
            read: Default::default(),
        }
    }
}

#[repr(C)]
pub(super) struct client_file_entry {
    /// Weak traversal handle into the client file index.
    pub(super) owner: refbox::Weak<ClientFileIndex>,
}

pub type client_file_cb = Option<Box<dyn for<'a> FnMut(client_file_event<'a>)>>;
