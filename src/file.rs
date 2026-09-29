use crate::src::server_client::server_client_unref_owned;
use crate::src::cmd::queue::{cmdq_clear_wait_file, cmdq_set_wait_file};
use crate::src::compat::imsg::imsg;
use crate::src::compat::imsg::*;
use crate::src::compat::imsg::{IMSG_HEADER_SIZE, MAX_IMSGSIZE};
use crate::src::compat::stdio::CFile;
use crate::src::ffi::libc::{
    __errno_location, close, dup, ferror, fopen, fread, fwrite, memcpy, open, strcmp, strlen,
};
use crate::src::log::{fatalx, log_cstr, log_debug};
use crate::src::proc::proc_send;
use crate::src::reactor::{
    bufferevent_enable, bufferevent_new, bufferevent_write, evbuffer_add,
    evbuffer_add_formatted, evbuffer_drain, evbuffer_get_length, evbuffer_pullup, event_once,
};
use crate::src::server_client::server_client_get_cwd;
use crate::src::shared::abi::ssize_t;
use crate::src::shared::abi::*;
use crate::src::shared::client::{
    client, client_file, client_file_cb, client_file_entry, client_file_event, client_files,
};
use crate::src::shared::client::{CLIENT_ATTACHED, CLIENT_CONTROL, CLIENT_DEAD, CLIENT_WRITE_ACK};
use crate::src::shared::command::cmdq_item;
use crate::src::shared::errno::{E2BIG, EINVAL, ENOMEM};
use crate::src::shared::event::*;
use crate::src::shared::event::{EV_READ, EV_TIMEOUT, EV_WRITE};
use crate::src::shared::posix_io::{
    O_APPEND, O_CREAT, O_NONBLOCK, O_WRONLY, STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO,
};
use crate::src::shared::process::tmuxpeer;
use crate::src::shared::stdio::FILE;
use crate::src::shared::tree::RB_NEGINF;
use crate::src::tmux::find_home_cstr;
use std::cell::UnsafeCell;
use std::ffi::{CStr, CString};
use std::rc::{Rc, Weak};

#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_read_open {
    pub stream: ::core::ffi::c_int,
    pub fd: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_read_data {
    pub stream: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_read_done {
    pub stream: ::core::ffi::c_int,
    pub error: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_read_cancel {
    pub stream: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_write_open {
    pub stream: ::core::ffi::c_int,
    pub fd: ::core::ffi::c_int,
    pub flags: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_write_data {
    pub stream: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_write_ready {
    pub stream: ::core::ffi::c_int,
    pub error: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_write_close {
    pub stream: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_write_done {
    pub stream: ::core::ffi::c_int,
    pub error: ::core::ffi::c_int,
}

pub const EIO: ::core::ffi::c_int = 5 as ::core::ffi::c_int;

pub const EBADF: ::core::ffi::c_int = 9 as ::core::ffi::c_int;

pub const O_RDONLY: ::core::ffi::c_int = 0 as ::core::ffi::c_int;

unsafe fn read_imsg_payload<T: Copy>(imsg: &imsg) -> Option<T> {
    if imsg.data.len() < ::core::mem::size_of::<T>() {
        return None;
    }
    Some(::core::ptr::read_unaligned(imsg.data.as_ptr().cast::<T>()))
}

pub const BEV_EVENT_ERROR: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const EVBUFFER_ERROR: ::core::ffi::c_int = BEV_EVENT_ERROR;

static mut file_next_stream: ::core::ffi::c_int = 3 as ::core::ffi::c_int;

fn file_set_path(cf: &mut client_file, path: CString) {
    cf.path = Default::default();
    cf.path = Some(path);
}

unsafe fn file_set_cmdq_wait(
    file_owner: &Rc<UnsafeCell<client_file>>,
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
    cancel_cb: Option<Box<dyn FnOnce()>>,
) {
    let item = item_handle.get();
    let owner = &mut *file_owner.get();
    assert!(!item.is_null());
    assert!(!owner.wait_active);
    owner.wait_item = (*item).observer.clone();
    owner.wait_active = true;
    owner.wait_client = (*item).client.clone();
    owner.cancel_data = cancel_cb;
    cmdq_set_wait_file(&mut *item, file_owner);
}

/// Stop a file-backed command wait without delivering its file callback.
/// The scheduled terminal event still owns and frees the file itself.
pub(crate) unsafe fn file_cancel_cmdq_wait(file_owner: &Rc<UnsafeCell<client_file>>) {
    let owner = &mut *file_owner.get();
    if !owner.wait_active {
        return;
    }
    owner.wait_active = false;
    if let Some(item) = std::mem::take(&mut owner.wait_item).upgrade() {
        cmdq_clear_wait_file(&mut *item.get(), &owner.observer);
    }
    owner.wait_client = Weak::new();
    owner.cb = None;
    let cancel_cb = owner.cancel_data.take();
    if let Some(cancel_cb) = cancel_cb {
        cancel_cb();
    }
}

unsafe fn file_get_path(c: Option<&client>, file: &CStr) -> CString {
    let file = file.to_bytes();
    let path = if file.starts_with(b"~/") {
        let home = find_home_cstr().map_or(&[][..], CStr::to_bytes);
        [home, &file[1..]].concat()
    } else {
        file.to_vec()
    };
    let full_path = if path.first() == Some(&b'/') {
        path
    } else {
        let cwd = server_client_get_cwd(c, None).expect("client working directory");
        [cwd.as_bytes(), b"/", path.as_slice()].concat()
    };
    CString::new(full_path).expect("C string path fragments contain no NUL")
}
pub unsafe fn file_create_with_peer(
    mut peer: *mut tmuxpeer,
    files: &mut client_files,
    mut stream: ::core::ffi::c_int,
    mut cb: client_file_cb,
) -> Rc<UnsafeCell<client_file>> {
    let owner = client_file::new();
    let cf = &mut *owner.get();
    (*cf).c = None;
    (*cf).stream = stream;
    (*cf).cb = cb;
    (*cf).peer = peer;
    client_files_insert(files, owner.clone());
    return owner;
}
unsafe fn file_create_with_client(
    client_owner: Option<&Rc<UnsafeCell<client>>>,
    stream: ::core::ffi::c_int,
    cb: client_file_cb,
) -> Rc<UnsafeCell<client_file>> {
    let client_owner = client_owner.filter(|owner| (*owner.get()).flags & CLIENT_ATTACHED as uint64_t == 0);
    let owner = client_file::new();
    let cf = &mut *owner.get();
    cf.c = client_owner.cloned();
    cf.stream = stream;
    cf.cb = cb;
    if let Some(client_owner) = client_owner {
        let client = &mut *client_owner.get();
        cf.peer = client.peer;
        client_files_insert(&mut client.files, owner.clone());
    }
    owner
}
unsafe fn file_destroy(cf: &mut client_file) {
    client_files_remove(&mut *cf);
    if let Some(client) = cf.c.take() {
        server_client_unref_owned(client);
    }
    cf.path = Default::default();
}
unsafe fn file_fire_done_cb(owner: &Rc<UnsafeCell<client_file>>) {
    let (client_owner, wait_client, expired_wait) = {
        let file = &*owner.get();
        (
            file.c.clone(),
            (!Weak::ptr_eq(&file.wait_client, &Weak::new())).then(|| file.wait_client.upgrade()),
            file.wait_active && file.wait_item.upgrade().is_none_or(|item| (*item.get()).is_removed()),
        )
    };
    let dead = expired_wait || client_owner.as_ref().is_some_and(|owner| (*owner.get()).flags & CLIENT_DEAD as uint64_t != 0)
        || wait_client.as_ref().is_some_and(|owner| owner.as_ref().is_none_or(|owner| {
            (*owner.get()).flags & CLIENT_DEAD as uint64_t != 0
        }));
    if dead {
        file_cancel_cmdq_wait(owner);
    } else {
        let owner = &mut *owner.get();
        if owner.wait_active {
            owner.wait_active = false;
            if let Some(item) = std::mem::take(&mut owner.wait_item).upgrade() {
                cmdq_clear_wait_file(&mut *item.get(), &owner.observer);
            }
            owner.wait_client = Weak::new();
            owner.cancel_data = None;
        }
    }
    let mut callback = (&mut *owner.get()).cb.take();
    if !dead {
        if let Some(callback) = callback.as_mut() {
            let file = &mut *owner.get();
            callback(client_file_event {
                client: client_owner.as_ref(),
                path: file.path.as_deref(),
                error: file.error,
                closed: true,
                buffer: Some(&mut *file.buffer),
            });
        }
    }
    drop(callback);
    // Completion retires the stream even if a lookup guard still retains its
    // allocation. Final Drop remains an idempotent unlink fallback.
    client_files_remove(&mut *owner.get());
}
/// Own completion until dispatch or cancellation. Both paths retire the index
/// entry while a typed owner still keeps the file and its callback data alive.
struct FileCompletion(Rc<UnsafeCell<client_file>>);

impl Drop for FileCompletion {
    fn drop(&mut self) {
        unsafe { client_files_remove(&mut *self.0.get()); }
    }
}

pub unsafe fn file_fire_done(owner: &Rc<UnsafeCell<client_file>>) {
    let cf = &mut *owner.get();
    if cf.terminal_scheduled {
        return;
    }
    cf.terminal_scheduled = true;
    let mut completion = Some(FileCompletion(owner.clone()));
    event_once(move |_, _| {
        let completion = completion.take().expect("one terminal dispatch");
        file_fire_done_cb(&completion.0);
    });
}
pub unsafe fn file_fire_read(file_owner: &Rc<UnsafeCell<client_file>>) {
    let cf = &mut *file_owner.get();
    let client_owner = cf.c.clone();
    let wait_client =
        (!Weak::ptr_eq(&cf.wait_client, &Weak::new())).then(|| cf.wait_client.upgrade());
    let dead = client_owner.as_ref().is_some_and(|owner| (*owner.get()).flags & CLIENT_DEAD as uint64_t != 0)
        || wait_client.as_ref().is_some_and(|owner| owner.as_ref().is_none_or(|owner| {
            (*owner.get()).flags & CLIENT_DEAD as uint64_t != 0
        }));
    if !dead {
        if let Some(callback) = cf.cb.as_mut() {
            callback(client_file_event {
                client: client_owner.as_ref(),
                path: cf.path.as_deref(),
                error: cf.error,
                closed: false,
                buffer: Some(&mut *cf.buffer),
            });
        }
    }
}
pub fn file_can_print(c: Option<&client>) -> ::core::ffi::c_int {
    c.is_some_and(|c| c.flags & (CLIENT_ATTACHED | CLIENT_DEAD | CLIENT_CONTROL) as uint64_t == 0) as ::core::ffi::c_int
}
pub unsafe fn file_print(
    client_owner: Option<&Rc<UnsafeCell<client>>>,
    write: impl FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>,
) {
    let Some(client_owner) = client_owner else { return; };
    let mut find: client_file = client_file::empty();
    let mut msg: msg_write_open = msg_write_open {
        stream: 0,
        fd: 0,
        flags: 0,
    };
    if file_can_print(Some(&*client_owner.get())) == 0 {
        return;
    }
    find.stream = 1 as ::core::ffi::c_int;
    let file_owner = client_files_find(&(&*client_owner.get()).files, &find);
    if file_owner.is_none() {
        let transfer_owner = file_create_with_client(Some(client_owner), 1 as ::core::ffi::c_int, None);
        let cf = &mut *transfer_owner.get();
        file_set_path(&mut *cf, CString::new("-").unwrap());
        evbuffer_add_formatted(&mut *(*cf).buffer, write);
        msg.stream = 1 as ::core::ffi::c_int;
        msg.fd = STDOUT_FILENO;
        msg.flags = 0 as ::core::ffi::c_int;
        proc_send(
            (&*client_owner.get()).peer,
            MSG_WRITE_OPEN,
            -(1 as ::core::ffi::c_int),
            &raw mut msg as *const ::core::ffi::c_void,
            ::core::mem::size_of::<msg_write_open>() as size_t,
        );
    } else {
        let cf = &mut *file_owner.as_ref().expect("looked-up file").get();
        evbuffer_add_formatted(&mut *(*cf).buffer, write);
        file_push(file_owner.as_ref().expect("looked-up file"));
    };
}
pub unsafe fn file_print_buffer(
    client_owner: Option<&Rc<UnsafeCell<client>>>,
    data: &[u8],
) {
    let Some(client_owner) = client_owner else { return; };
    let mut find: client_file = client_file::empty();
    let mut msg: msg_write_open = msg_write_open {
        stream: 0,
        fd: 0,
        flags: 0,
    };
    if file_can_print(Some(&*client_owner.get())) == 0 {
        return;
    }
    find.stream = 1 as ::core::ffi::c_int;
    let file_owner = client_files_find(&(&*client_owner.get()).files, &find);
    if file_owner.is_none() {
        let transfer_owner = file_create_with_client(Some(client_owner), 1 as ::core::ffi::c_int, None);
        let cf = &mut *transfer_owner.get();
        file_set_path(&mut *cf, CString::new("-").unwrap());
        evbuffer_add(&mut *(*cf).buffer, data.as_ptr().cast(), data.len());
        msg.stream = 1 as ::core::ffi::c_int;
        msg.fd = STDOUT_FILENO;
        msg.flags = 0 as ::core::ffi::c_int;
        proc_send(
            (&*client_owner.get()).peer,
            MSG_WRITE_OPEN,
            -(1 as ::core::ffi::c_int),
            &raw mut msg as *const ::core::ffi::c_void,
            ::core::mem::size_of::<msg_write_open>() as size_t,
        );
    } else {
        let cf = &mut *file_owner.as_ref().expect("looked-up file").get();
        evbuffer_add(&mut *(*cf).buffer, data.as_ptr().cast(), data.len());
        file_push(file_owner.as_ref().expect("looked-up file"));
    };
}
pub unsafe fn file_error(
    client_owner: Option<&Rc<UnsafeCell<client>>>,
    write: impl FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>,
) {
    let Some(client_owner) = client_owner else { return; };
    let mut find: client_file = client_file::empty();
    let mut msg: msg_write_open = msg_write_open {
        stream: 0,
        fd: 0,
        flags: 0,
    };
    if file_can_print(Some(&*client_owner.get())) == 0 {
        return;
    }
    find.stream = 2 as ::core::ffi::c_int;
    let file_owner = client_files_find(&(&*client_owner.get()).files, &find);
    if file_owner.is_none() {
        let transfer_owner = file_create_with_client(Some(client_owner), 2 as ::core::ffi::c_int, None);
        let cf = &mut *transfer_owner.get();
        file_set_path(&mut *cf, CString::new("-").unwrap());
        evbuffer_add_formatted(&mut *(*cf).buffer, write);
        msg.stream = 2 as ::core::ffi::c_int;
        msg.fd = STDERR_FILENO;
        msg.flags = 0 as ::core::ffi::c_int;
        proc_send(
            (&*client_owner.get()).peer,
            MSG_WRITE_OPEN,
            -(1 as ::core::ffi::c_int),
            &raw mut msg as *const ::core::ffi::c_void,
            ::core::mem::size_of::<msg_write_open>() as size_t,
        );
    } else {
        let cf = &mut *file_owner.as_ref().expect("looked-up file").get();
        evbuffer_add_formatted(&mut *(*cf).buffer, write);
        file_push(file_owner.as_ref().expect("looked-up file"));
    };
}

pub(crate) unsafe fn file_write_with_cmdq_wait(
    client_owner: Option<&Rc<UnsafeCell<client>>>,
    path: *const ::core::ffi::c_char,
    flags: ::core::ffi::c_int,
    bdata: *const ::core::ffi::c_void,
    bsize: size_t,
    cb: client_file_cb,
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
) {
    let cancel_cb: Option<Box<dyn FnOnce()>> = None;
    file_write_impl(client_owner, path, flags, bdata, bsize, cb, Some((item_handle, cancel_cb)));
}

unsafe fn file_write_impl(
    client_owner: Option<&Rc<UnsafeCell<client>>>,
    mut path: *const ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
    mut bdata: *const ::core::ffi::c_void,
    mut bsize: size_t,
    mut cb: client_file_cb,
    wait: Option<(&Rc<UnsafeCell<cmdq_item>>, Option<Box<dyn FnOnce()>>)>,
) {
    let mut current_block: u64;
    let mut msglen: size_t = 0;
    let mut fd: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    let fresh0 = file_next_stream;
    file_next_stream = file_next_stream + 1;
    let mut stream: u_int = fresh0 as u_int;
    let mut f: *mut FILE = ::core::ptr::null_mut::<FILE>();
    let mut mode: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let transfer_owner = file_create_with_client(client_owner, stream as ::core::ffi::c_int, cb);
    if let Some((item_handle, cancel_cb)) = wait {
        file_set_cmdq_wait(&transfer_owner, item_handle, cancel_cb);
    }
    let cf = &mut *transfer_owner.get();
    if strcmp(path, b"-\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int {
        file_set_path(cf, CString::new("-").unwrap());
        fd = STDOUT_FILENO;
        if client_owner.is_none_or(|owner| {
            let client = &*owner.get();
            client.flags & (CLIENT_ATTACHED | CLIENT_CONTROL) as uint64_t != 0
        }) {
            cf.error = EBADF;
            current_block = 4636144702248558238;
        } else {
            current_block = 8821498768635335055;
        }
    } else {
        file_set_path(
            cf,
            file_get_path(client_owner.map(|owner| &*owner.get()), CStr::from_ptr(path)),
        );
        if client_owner.is_none_or(|owner| {
            let client = &*owner.get();
            client.flags & CLIENT_ATTACHED as uint64_t != 0
        }) {
            if flags & O_APPEND != 0 {
                mode = b"ab\0" as *const u8 as *const ::core::ffi::c_char;
            } else {
                mode = b"wb\0" as *const u8 as *const ::core::ffi::c_char;
            }
            f = fopen(
                cf.path
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                mode,
            ) as *mut FILE;
            if f.is_null() {
                cf.error = *__errno_location();
            } else {
                let file = CFile::from_raw(f).expect("fopen returned a non-null stream");
                let write_failed =
                    fwrite(bdata, 1 as size_t, bsize, file.as_ptr()) as size_t != bsize;
                drop(file);
                if write_failed {
                    cf.error = EIO;
                }
            }
            current_block = 4636144702248558238;
        } else {
            current_block = 8821498768635335055;
        }
    }
    match current_block {
        8821498768635335055 => {
            evbuffer_add(&mut *cf.buffer, bdata, bsize);
            msglen = strlen(
                cf.path
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            )
            .wrapping_add(1 as size_t)
            .wrapping_add(::core::mem::size_of::<msg_write_open>() as size_t);
            if msglen > (MAX_IMSGSIZE as usize).wrapping_sub(IMSG_HEADER_SIZE) {
                cf.error = E2BIG;
            } else {
                let mut msg = vec![0_u8; msglen];
                let header = msg_write_open {
                    stream: cf.stream,
                    fd,
                    flags,
                };
                memcpy(
                    msg.as_mut_ptr().cast(),
                    (&raw const header).cast(),
                    ::core::mem::size_of::<msg_write_open>(),
                );
                memcpy(
                    msg.as_mut_ptr()
                        .add(::core::mem::size_of::<msg_write_open>())
                        .cast(),
                    cf.path
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut())
                        as *const ::core::ffi::c_void,
                    msglen.wrapping_sub(::core::mem::size_of::<msg_write_open>() as size_t),
                );
                if proc_send(
                    cf.peer,
                    MSG_WRITE_OPEN,
                    -(1 as ::core::ffi::c_int),
                    msg.as_ptr().cast(),
                    msglen,
                ) != 0 as ::core::ffi::c_int
                {
                    cf.error = EINVAL;
                } else {
                    return;
                }
            }
        }
        _ => {}
    }
    file_fire_done(&transfer_owner);
}

pub(crate) unsafe fn file_read_with_cmdq_wait(
    client_owner: Option<&Rc<UnsafeCell<client>>>,
    path: *const ::core::ffi::c_char,
    cb: client_file_cb,
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
    cancel_cb: Option<Box<dyn FnOnce()>>,
) {
    file_read_with_cmdq_wait_init(client_owner, path, |_| cb, item_handle, cancel_cb)
}

/// Build the callback after allocating its file, before opening or scheduling
/// any events. This lets a callback own its state without sharing startup data.
pub(crate) unsafe fn file_read_with_cmdq_wait_init(
    client_owner: Option<&Rc<UnsafeCell<client>>>,
    mut path: *const ::core::ffi::c_char,
    callback: impl FnOnce(std::rc::Weak<std::cell::UnsafeCell<client_file>>) -> client_file_cb,
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
    cancel_cb: Option<Box<dyn FnOnce()>>,
) {
    let mut current_block: u64;
    let mut fd: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    let fresh1 = file_next_stream;
    file_next_stream = file_next_stream + 1;
    let mut stream: u_int = fresh1 as u_int;
    let mut f: *mut FILE = ::core::ptr::null_mut::<FILE>();
    let mut file_owner: Option<CFile> = None;
    let mut size: size_t = 0;
    let mut buffer: [::core::ffi::c_char; 8192] = [0; 8192];
    let transfer_owner = file_create_with_client(client_owner, stream as ::core::ffi::c_int, None);
    file_set_cmdq_wait(&transfer_owner, item_handle, cancel_cb);
    let cb = callback(Rc::downgrade(&transfer_owner));
    let cf = &mut *transfer_owner.get();
    cf.cb = cb;
    if strcmp(path, b"-\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int {
        file_set_path(cf, CString::new("-").unwrap());
        fd = STDIN_FILENO;
        if client_owner.is_none_or(|owner| {
            let client = &*owner.get();
            client.flags & (CLIENT_ATTACHED | CLIENT_CONTROL) as uint64_t != 0
        }) {
            cf.error = EBADF;
            current_block = 17369485759464587280;
        } else {
            current_block = 17710118112003399050;
        }
    } else {
        file_set_path(
            cf,
            file_get_path(client_owner.map(|owner| &*owner.get()), CStr::from_ptr(path)),
        );
        if client_owner.is_none_or(|owner| {
            let client = &*owner.get();
            client.flags & CLIENT_ATTACHED as uint64_t != 0
        }) {
            f = fopen(
                cf.path
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                b"rb\0" as *const u8 as *const ::core::ffi::c_char,
            ) as *mut FILE;
            if f.is_null() {
                cf.error = *__errno_location();
            } else {
                file_owner = CFile::from_raw(f);
                let file = file_owner
                    .as_ref()
                    .expect("fopen returned a non-null stream");
                loop {
                    size = fread(
                        &raw mut buffer as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                        1 as size_t,
                        ::core::mem::size_of::<[::core::ffi::c_char; 8192]>() as size_t,
                        file.as_ptr(),
                    ) as size_t;
                    if ferror(file.as_ptr()) != 0 {
                        cf.error = *__errno_location();
                        current_block = 17369485759464587280;
                        break;
                    } else if evbuffer_add(
                        &mut *cf.buffer,
                        &raw mut buffer as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
                        size,
                    ) != 0 as ::core::ffi::c_int
                    {
                        cf.error = ENOMEM;
                        current_block = 17369485759464587280;
                        break;
                    } else if size != ::core::mem::size_of::<[::core::ffi::c_char; 8192]>() as usize
                    {
                        current_block = 4808432441040389987;
                        break;
                    }
                }
                match current_block {
                    17369485759464587280 => {}
                    _ => {
                        if ferror(file.as_ptr()) != 0 {
                            cf.error = EIO;
                        }
                    }
                }
            }
            current_block = 17369485759464587280;
        } else {
            current_block = 17710118112003399050;
        }
    }
    match current_block {
        17710118112003399050 => {
            let path = cf.path.as_deref().expect("file path").to_bytes_with_nul();
            let header_len = ::core::mem::size_of::<msg_read_open>();
            let msglen = header_len + path.len();
            if msglen > (MAX_IMSGSIZE as usize).wrapping_sub(IMSG_HEADER_SIZE) {
                cf.error = E2BIG;
            } else {
                let header = msg_read_open {
                    stream: cf.stream,
                    fd,
                };
                let mut msg = vec![0; msglen];
                memcpy(
                    msg.as_mut_ptr().cast(),
                    (&raw const header).cast(),
                    header_len,
                );
                msg[header_len..].copy_from_slice(path);
                if proc_send(
                    cf.peer,
                    MSG_READ_OPEN,
                    -(1 as ::core::ffi::c_int),
                    msg.as_ptr().cast(),
                    msglen,
                ) != 0 as ::core::ffi::c_int
                {
                    cf.error = EINVAL;
                } else {
                    return;
                }
            }
        }
        _ => {}
    }
    drop(file_owner);
    file_fire_done(&transfer_owner);

}
pub unsafe fn file_cancel(cf: &mut client_file) {
    let mut msg: msg_read_cancel = msg_read_cancel { stream: 0 };
    log_debug(format_args!("read cancel file {}", ((*cf).stream) as i32));
    if (*cf).closed != 0 {
        return;
    }
    (*cf).closed = 1 as ::core::ffi::c_int;
    msg.stream = (*cf).stream;
    proc_send(
        (*cf).peer,
        MSG_READ_CANCEL,
        -(1 as ::core::ffi::c_int),
        &raw mut msg as *const ::core::ffi::c_void,
        ::core::mem::size_of::<msg_read_cancel>() as size_t,
    );
}
unsafe fn file_push_cb(owner: &Rc<UnsafeCell<client_file>>) {
    let cf = &*owner.get();
    if cf.c.as_ref().is_none_or(|owner| {
        let client = &*owner.get();
        client.flags & CLIENT_DEAD as uint64_t == 0
    }) {
        file_push(owner);
    }
}
pub unsafe fn file_push(file_owner: &Rc<UnsafeCell<client_file>>) {
    let cf = &mut *file_owner.get();
    let mut msg = Vec::<u8>::new();
    let header_len = ::core::mem::size_of::<msg_write_data>();
    let mut sent: size_t = 0;
    let mut left: size_t = 0;
    let mut close_0: msg_write_close = msg_write_close { stream: 0 };
    left = evbuffer_get_length(&*(cf.buffer));
    while left != 0 as size_t {
        sent = left;
        if sent
            > (MAX_IMSGSIZE as usize)
                .wrapping_sub(IMSG_HEADER_SIZE)
                .wrapping_sub(::core::mem::size_of::<msg_write_data>() as usize)
        {
            sent = (MAX_IMSGSIZE as usize)
                .wrapping_sub(IMSG_HEADER_SIZE)
                .wrapping_sub(::core::mem::size_of::<msg_write_data>() as usize)
                as size_t;
        }
        let msglen = header_len + sent;
        msg.resize(msglen, 0);
        let header = msg_write_data {
            stream: cf.stream,
        };
        memcpy(
            msg.as_mut_ptr().cast(),
            (&raw const header).cast(),
            header_len,
        );
        memcpy(
            msg.as_mut_ptr().add(header_len).cast(),
            evbuffer_pullup(&mut *cf.buffer, sent as ssize_t)
                .map_or(std::ptr::null_mut(), |bytes| bytes.as_mut_ptr()) as *const ::core::ffi::c_void,
            sent,
        );
        if proc_send(
            cf.peer,
            MSG_WRITE,
            -(1 as ::core::ffi::c_int),
            msg.as_ptr().cast(),
            msglen,
        ) != 0 as ::core::ffi::c_int
        {
            break;
        }
        evbuffer_drain(&mut *cf.buffer, sent);
        left = evbuffer_get_length(&*(cf.buffer));
        log_debug(format_args!(
            "file {} sent {}, left {}",
            (cf.stream) as i32,
            (sent) as usize,
            (left) as usize
        ));
    }
    if left != 0 as size_t {
        let owner = file_owner.clone();
        event_once(move |_, _| unsafe { file_push_cb(&owner) });
    } else if cf.stream > 2 as ::core::ffi::c_int {
        close_0.stream = cf.stream;
        proc_send(
            cf.peer,
            MSG_WRITE_CLOSE,
            -(1 as ::core::ffi::c_int),
            &raw mut close_0 as *const ::core::ffi::c_void,
            ::core::mem::size_of::<msg_write_close>() as size_t,
        );
        if cf.c.as_ref().is_none_or(|owner| {
            let client = &*owner.get();
            client.flags & CLIENT_WRITE_ACK as uint64_t == 0
        }) {
            file_fire_done(file_owner);
        }
    }
}
pub unsafe fn file_write_left(files: &client_files) -> ::core::ffi::c_int {
    let mut left: size_t = 0;
    let mut waiting: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut next = client_files_minmax(files);
    while let Some(file) = next {
        let cf = &*file.get();
        if let Some(remaining) = cf.event.with_ptr(|stream| unsafe {
            evbuffer_get_length(&*(*stream).output)
        }) {
            left = remaining;
            if left != 0 as size_t {
                waiting += 1;
                log_debug(format_args!(
                    "file {} {} bytes left",
                    (cf.stream) as u32,
                    (left) as usize
                ));
            }
        }
        next = client_files_next(&*cf);
    }
    return (waiting != 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
unsafe fn file_write_finished(owner: &Rc<UnsafeCell<client_file>>) {
    let cf = &mut *owner.get();
    let mut msg: msg_write_done = msg_write_done {
        stream: 0,
        error: 0,
    };
    cf.event.free();
    if cf.fd != -(1 as ::core::ffi::c_int) {
        if close(cf.fd) != 0 as ::core::ffi::c_int && cf.error == 0 as ::core::ffi::c_int {
            cf.error = *__errno_location();
        }
        cf.fd = -(1 as ::core::ffi::c_int);
    }
    msg.stream = cf.stream;
    msg.error = cf.error;
    proc_send(
        cf.peer,
        MSG_WRITE_DONE,
        -(1 as ::core::ffi::c_int),
        &raw mut msg as *const ::core::ffi::c_void,
        ::core::mem::size_of::<msg_write_done>() as size_t,
    );
    if let Some(callback) = cf.cb.as_mut() {
        callback(client_file_event {
            client: None,
            path: None,
            error: 0,
            closed: true,
            buffer: None,
        });
    }
    client_files_remove(&mut *owner.get());
}
unsafe fn file_write_error_callback(
    mut what: ::core::ffi::c_short,
    owner: &Rc<UnsafeCell<client_file>>,
) {
    let cf = &mut *owner.get();
    let mut error: ::core::ffi::c_int = 0;
    if what as ::core::ffi::c_int & EVBUFFER_ERROR != 0 {
        error = *__errno_location();
    } else {
        error = EIO;
    }
    if error == 0 as ::core::ffi::c_int {
        error = EIO;
    }
    log_debug(format_args!("write error file {}", (cf.stream) as i32));
    cf.error = error;
    cf.event.free();
    close(cf.fd);
    cf.fd = -(1 as ::core::ffi::c_int);
    if cf.closed != 0 {
        file_write_finished(owner);
    } else if let Some(callback) = cf.cb.as_mut() {
        callback(client_file_event {
            client: None,
            path: None,
            error: 0,
            closed: true,
            buffer: None,
        });
    }
}
unsafe fn file_write_callback(owner: &Rc<UnsafeCell<client_file>>) {
    let cf = &mut *owner.get();
    log_debug(format_args!("write check file {}", (cf.stream) as i32));
    let remaining = cf.event.with_ptr(|stream| unsafe {
        evbuffer_get_length(&*(*stream).output)
    }).unwrap_or(0);
    if cf.closed != 0 && remaining == 0 as size_t {
        file_write_finished(owner);
    } else if let Some(callback) = cf.cb.as_mut() {
        callback(client_file_event {
            client: None,
            path: None,
            error: 0,
            closed: true,
            buffer: None,
        });
    }
}
pub unsafe fn file_write_open(
    files: &mut client_files,
    mut peer: *mut tmuxpeer,
    imsg: &imsg,
    mut close_received: ::core::ffi::c_int,
    mut cb: client_file_cb,
) {
    let transfer_owner;
    let msglen = imsg.data.len();
    if msglen < ::core::mem::size_of::<msg_write_open>() {
        fatalx(|out| out.write_all(b"bad MSG_WRITE_OPEN size"));
    }
    let msg = read_imsg_payload::<msg_write_open>(imsg).unwrap();
    let mut path: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut reply: msg_write_ready = msg_write_ready {
        stream: 0,
        error: 0,
    };
    let mut find: client_file = client_file::empty();
    let flags: ::core::ffi::c_int = O_NONBLOCK | O_WRONLY | O_CREAT;
    let mut error: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if msglen == ::core::mem::size_of::<msg_write_open>() as usize {
        path = b"-\0" as *const u8 as *const ::core::ffi::c_char;
    } else {
        path = imsg.data[::core::mem::size_of::<msg_write_open>()..]
            .as_ptr()
            .cast::<::core::ffi::c_char>();
    }
    log_debug(format_args!(
        "open write file {} {}",
        (msg.stream) as i32,
        log_cstr((path) as *const _)
    ));
    find.stream = msg.stream;
    if client_files_find(files, &find).is_some() {
        error = EBADF;
    } else {
        transfer_owner = file_create_with_peer(peer, files, msg.stream, cb);
        let cf = &mut *transfer_owner.get();
        if cf.closed != 0 {
            error = EBADF;
        } else {
            cf.fd = -(1 as ::core::ffi::c_int);
            if msg.fd == -(1 as ::core::ffi::c_int) {
                cf.fd = open(path, msg.flags | flags, 0o644 as ::core::ffi::c_int);
            } else {
                if msg.fd != STDOUT_FILENO && msg.fd != STDERR_FILENO {
                    *__errno_location() = EBADF;
                } else {
                    cf.fd = dup(msg.fd);
                    if close_received != 0 {
                        close(msg.fd);
                    }
                }
            }
            if cf.fd == -(1 as ::core::ffi::c_int) {
                error = *__errno_location();
            } else {
                let data_observer = Rc::downgrade(&transfer_owner);
                let error_observer = data_observer.clone();
                let stream = bufferevent_new(
                    cf.fd,
                    None,
                    bufferevent_data_callback(move |_| unsafe {
                        if let Some(owner) = data_observer.upgrade() { file_write_callback(&owner); }
                    }),
                    bufferevent_event_callback(move |_, flags| unsafe {
                        if let Some(owner) = error_observer.upgrade() { file_write_error_callback(flags, &owner); }
                    }),
                );
                if stream.is_null() {
                    fatalx(|out| out.write_all(b"out of memory"));
                }
                cf.event = crate::src::reactor::StreamHandle::from_ptr(stream);
                bufferevent_enable(stream, EV_WRITE as ::core::ffi::c_short);
            }
        }
    }
    reply.stream = msg.stream;
    reply.error = error;
    proc_send(
        peer,
        MSG_WRITE_READY,
        -(1 as ::core::ffi::c_int),
        &raw mut reply as *const ::core::ffi::c_void,
        ::core::mem::size_of::<msg_write_ready>() as size_t,
    );
}
pub unsafe fn file_write_data(files: &client_files, imsg: &imsg) {
    let msglen = imsg.data.len();
    if msglen < ::core::mem::size_of::<msg_write_data>() {
        fatalx(|out| out.write_all(b"bad MSG_WRITE size"));
    }
    let msg = read_imsg_payload::<msg_write_data>(imsg).unwrap();
    let mut find: client_file = client_file::empty();
    let size = msglen - ::core::mem::size_of::<msg_write_data>();
    find.stream = msg.stream;
    let Some(file_owner) = client_files_find(files, &find) else {
        fatalx(|out| out.write_all(b"unknown stream number"));
    };
    let cf = &*file_owner.get();
    log_debug(format_args!(
        "write {} to file {}",
        (size) as usize,
        (cf.stream) as i32
    ));
    let _ = cf.event.with_ptr(|stream| unsafe {
        bufferevent_write(
            stream,
            imsg.data[::core::mem::size_of::<msg_write_data>()..]
                .as_ptr()
                .cast::<::core::ffi::c_void>(),
            size,
        )
    });
}
pub unsafe fn file_write_close(files: &client_files, imsg: &imsg) {
    let msglen = imsg.data.len();
    if msglen != ::core::mem::size_of::<msg_write_close>() {
        fatalx(|out| out.write_all(b"bad MSG_WRITE_CLOSE size"));
    }
    let msg = read_imsg_payload::<msg_write_close>(imsg).unwrap();
    let mut find: client_file = client_file::empty();
    find.stream = msg.stream;
    let Some(file_owner) = client_files_find(files, &find) else {
        fatalx(|out| out.write_all(b"unknown stream number"));
    };
    let cf = &mut *file_owner.get();
    log_debug(format_args!("close file {}", (cf.stream) as i32));
    cf.closed = 1 as ::core::ffi::c_int;
    let remaining = cf.event.with_ptr(|stream| unsafe {
        evbuffer_get_length(&*(*stream).output)
    }).unwrap_or(0);
    if remaining == 0 as size_t {
        file_write_finished(&file_owner);
    }
}
unsafe fn file_read_error_callback(
    mut what: ::core::ffi::c_short,
    owner: &Rc<UnsafeCell<client_file>>,
) {
    let cf = &mut *owner.get();
    let mut msg: msg_read_done = msg_read_done {
        stream: 0,
        error: 0,
    };
    log_debug(format_args!("read error file {}", (cf.stream) as i32));
    msg.stream = cf.stream;
    msg.error = if what as ::core::ffi::c_int & EVBUFFER_ERROR != 0 {
        EIO
    } else {
        0 as ::core::ffi::c_int
    };
    proc_send(
        cf.peer,
        MSG_READ_DONE,
        -(1 as ::core::ffi::c_int),
        &raw mut msg as *const ::core::ffi::c_void,
        ::core::mem::size_of::<msg_read_done>() as size_t,
    );
    cf.event.free();
    close(cf.fd);
    client_files_remove(&mut *cf);
}
unsafe fn file_read_callback(owner: &Rc<UnsafeCell<client_file>>) {
    let cf = &mut *owner.get();
    let mut msg = Vec::<u8>::new();
    let header_len = ::core::mem::size_of::<msg_read_data>();
    loop {
        let Some(chunk) = cf.event.with_ptr(|stream| unsafe {
            let input = &mut *(*stream).input;
            let bsize = evbuffer_get_length(input).min(
                (MAX_IMSGSIZE as usize)
                    .wrapping_sub(IMSG_HEADER_SIZE)
                    .wrapping_sub(header_len),
            );
            evbuffer_pullup(input, bsize as ssize_t)
                .map_or_else(Vec::new, |bytes| bytes[..bsize].to_vec())
        }) else { break };
        let bsize = chunk.len();
        if bsize == 0 { break; }
        log_debug(format_args!(
            "read {} from file {}",
            (bsize) as usize,
            (cf.stream) as i32
        ));
        let msglen = header_len + bsize;
        msg.resize(msglen, 0);
        let header = msg_read_data {
            stream: cf.stream,
        };
        memcpy(
            msg.as_mut_ptr().cast(),
            (&raw const header).cast(),
            header_len,
        );
        msg[header_len..].copy_from_slice(&chunk);
        proc_send(
            cf.peer,
            MSG_READ,
            -(1 as ::core::ffi::c_int),
            msg.as_ptr().cast(),
            msglen,
        );
        let _ = cf.event.with_ptr(|stream| unsafe {
            evbuffer_drain(&mut *(*stream).input, bsize)
        });
    }
}
pub unsafe fn file_read_open(
    files: &mut client_files,
    mut peer: *mut tmuxpeer,
    imsg: &imsg,
    mut close_received: ::core::ffi::c_int,
    mut cb: client_file_cb,
) {
    let transfer_owner;
    let msglen = imsg.data.len();
    if msglen < ::core::mem::size_of::<msg_read_open>() {
        fatalx(|out| out.write_all(b"bad MSG_READ_OPEN size"));
    }
    let msg = read_imsg_payload::<msg_read_open>(imsg).unwrap();
    let mut path: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut reply: msg_read_done = msg_read_done {
        stream: 0,
        error: 0,
    };
    let mut find: client_file = client_file::empty();
    let flags: ::core::ffi::c_int = O_NONBLOCK | O_RDONLY;
    let mut error: ::core::ffi::c_int = 0;
    if msglen == ::core::mem::size_of::<msg_read_open>() as usize {
        path = b"-\0" as *const u8 as *const ::core::ffi::c_char;
    } else {
        path = imsg.data[::core::mem::size_of::<msg_read_open>()..]
            .as_ptr()
            .cast::<::core::ffi::c_char>();
    }
    log_debug(format_args!(
        "open read file {} {}",
        (msg.stream) as i32,
        log_cstr((path) as *const _)
    ));
    find.stream = msg.stream;
    if client_files_find(files, &find).is_some() {
        error = EBADF;
    } else {
        transfer_owner = file_create_with_peer(peer, files, msg.stream, cb);
        let cf = &mut *transfer_owner.get();
        if (*cf).closed != 0 {
            error = EBADF;
        } else {
            (*cf).fd = -(1 as ::core::ffi::c_int);
            if msg.fd == -(1 as ::core::ffi::c_int) {
                (*cf).fd = open(path, flags);
            } else {
                if msg.fd != STDIN_FILENO {
                    *__errno_location() = EBADF;
                } else {
                    (*cf).fd = dup(msg.fd);
                    if close_received != 0 {
                        close(msg.fd);
                    }
                }
            }
            if (*cf).fd == -(1 as ::core::ffi::c_int) {
                error = *__errno_location();
            } else {
                let data_observer = Rc::downgrade(&transfer_owner);
                let error_observer = data_observer.clone();
                let stream = bufferevent_new(
                    (*cf).fd,
                    bufferevent_data_callback(move |_| unsafe {
                        if let Some(owner) = data_observer.upgrade() { file_read_callback(&owner); }
                    }),
                    None,
                    bufferevent_event_callback(move |_, flags| unsafe {
                        if let Some(owner) = error_observer.upgrade() { file_read_error_callback(flags, &owner); }
                    }),
                );
                if stream.is_null() {
                    fatalx(|out| out.write_all(b"out of memory"));
                }
                (*cf).event = crate::src::reactor::StreamHandle::from_ptr(stream);
                bufferevent_enable(stream, EV_READ as ::core::ffi::c_short);
                return;
            }
        }
    }
    reply.stream = msg.stream;
    reply.error = error;
    proc_send(
        peer,
        MSG_READ_DONE,
        -(1 as ::core::ffi::c_int),
        &raw mut reply as *const ::core::ffi::c_void,
        ::core::mem::size_of::<msg_read_done>() as size_t,
    );
}
pub unsafe fn file_read_cancel(files: &client_files, imsg: &imsg) {
    let msglen = imsg.data.len();
    if msglen != ::core::mem::size_of::<msg_read_cancel>() {
        fatalx(|out| out.write_all(b"bad MSG_READ_CANCEL size"));
    }
    let msg = read_imsg_payload::<msg_read_cancel>(imsg).unwrap();
    let mut find: client_file = client_file::empty();
    find.stream = msg.stream;
    let Some(file_owner) = client_files_find(files, &find) else {
        fatalx(|out| out.write_all(b"unknown stream number"));
    };
    let cf = &*file_owner.get();
    log_debug(format_args!("cancel file {}", (cf.stream) as i32));
    file_read_error_callback(0, &file_owner);
}
pub unsafe fn file_write_ready(files: &client_files, imsg: &imsg) -> ::core::ffi::c_int {
    let msglen = imsg.data.len();
    if msglen != ::core::mem::size_of::<msg_write_ready>() {
        return -1;
    }
    let msg = read_imsg_payload::<msg_write_ready>(imsg).unwrap();
    let mut find: client_file = client_file::empty();
    find.stream = msg.stream;
    let Some(file_owner) = client_files_find(files, &find) else {
        return 0 as ::core::ffi::c_int;
    };
    let cf = &mut *file_owner.get();
    if msg.error != 0 as ::core::ffi::c_int {
        cf.error = msg.error;
        file_fire_done(&file_owner);
    } else {
        file_push(&file_owner);
    }
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn file_write_done(files: &client_files, imsg: &imsg) -> ::core::ffi::c_int {
    let msglen = imsg.data.len();
    if msglen != ::core::mem::size_of::<msg_write_done>() {
        return -1;
    }
    let msg = read_imsg_payload::<msg_write_done>(imsg).unwrap();
    let mut find: client_file = client_file::empty();
    find.stream = msg.stream;
    let Some(file_owner) = client_files_find(files, &find) else {
        return 0 as ::core::ffi::c_int;
    };
    let cf = &mut *file_owner.get();
    if cf.c.as_ref().is_none_or(|owner| {
        let client = &*owner.get();
        client.flags & CLIENT_WRITE_ACK as uint64_t == 0
    }) {
        return 0 as ::core::ffi::c_int;
    }
    log_debug(format_args!("file {} write done", (cf.stream) as i32));
    cf.error = msg.error;
    file_fire_done(&file_owner);
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn file_read_data(files: &client_files, imsg: &imsg) -> ::core::ffi::c_int {
    let msglen = imsg.data.len();
    if msglen < ::core::mem::size_of::<msg_read_data>() {
        return -1;
    }
    let msg = read_imsg_payload::<msg_read_data>(imsg).unwrap();
    let mut find: client_file = client_file::empty();
    let bdata = imsg.data[::core::mem::size_of::<msg_read_data>()..]
        .as_ptr()
        .cast::<::core::ffi::c_void>();
    let bsize = msglen - ::core::mem::size_of::<msg_read_data>();
    find.stream = msg.stream;
    let Some(file_owner) = client_files_find(files, &find) else {
        return 0 as ::core::ffi::c_int;
    };
    let cf = &mut *file_owner.get();
    log_debug(format_args!(
        "file {} read {} bytes",
        (cf.stream) as i32,
        (bsize) as usize
    ));
    if cf.error == 0 as ::core::ffi::c_int && cf.closed == 0 {
        if evbuffer_add(&mut *cf.buffer, bdata, bsize) != 0 as ::core::ffi::c_int {
            cf.error = ENOMEM;
            file_fire_done(&file_owner);
        } else {
            file_fire_read(&file_owner);
        }
    }
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn file_read_done(files: &client_files, imsg: &imsg) -> ::core::ffi::c_int {
    let msglen = imsg.data.len();
    if msglen != ::core::mem::size_of::<msg_read_done>() {
        return -1;
    }
    let msg = read_imsg_payload::<msg_read_done>(imsg).unwrap();
    let mut find: client_file = client_file::empty();
    find.stream = msg.stream;
    let Some(file_owner) = client_files_find(files, &find) else {
        return 0 as ::core::ffi::c_int;
    };
    let cf = &mut *file_owner.get();
    log_debug(format_args!("file {} read done", (cf.stream) as i32));
    cf.error = msg.error;
    file_fire_done(&file_owner);
    return 0 as ::core::ffi::c_int;
}

fn client_files_key(elm: &client_file) -> i32 {
    elm.stream
}
/// Lookups clone the indexed Rc to keep a file alive during an operation.
pub fn client_files_find(
    head: &client_files,
    elm: &client_file,
) -> Option<Rc<UnsafeCell<client_file>>> {
    let owner = head.storage.as_ref()?;
    let map = owner
        .try_borrow_mut()
        .expect("client file index already borrowed");
    map.get(&client_files_key(elm)).cloned()
}

pub unsafe fn client_files_insert(
    head: &mut client_files,
    file: Rc<UnsafeCell<client_file>>,
) -> Option<Rc<UnsafeCell<client_file>>> {
    let key = client_files_key(&*file.get());
    let owner = head.storage.get_or_insert_with(refbox::RefBox::default);
    let observer = owner.downgrade();
    let mut map = owner
        .try_borrow_mut()
        .expect("client file index already borrowed");
    if let Some(existing) = map.get(&key).cloned() {
        return Some(existing);
    }
    (&mut *file.get()).entry.owner = observer;
    map.insert(key, file);
    None
}

/// Unlink by identity, including during final Rc Drop when upgrade cannot work.
/// The head retains its empty index until it is dropped or replaced.
pub fn client_files_remove(elm: &mut client_file) {
    let owner = std::mem::take(&mut elm.entry.owner);
    if owner.is_empty() {
        return;
    }
    let mut map = match owner.try_borrow_mut() {
        Ok(map) => map,
        Err(refbox::BorrowError::Dropped) => return,
        Err(refbox::BorrowError::Borrowed) => panic!("client file index already borrowed"),
    };
    let key = client_files_key(&*elm);
    if map
        .get(&key)
        .is_some_and(|file| Rc::downgrade(file).ptr_eq(&elm.observer))
    {
        let file = map.remove(&key);
        drop(map);
        drop(file);
    }
}

pub fn client_files_minmax(head: &client_files) -> Option<Rc<UnsafeCell<client_file>>> {
    let owner = head.storage.as_ref()?;
    let map = owner
        .try_borrow_mut()
        .expect("client file index already borrowed");
    map.values().next().cloned()
}

pub fn client_files_next(elm: &client_file) -> Option<Rc<UnsafeCell<client_file>>> {
    let owner = &elm.entry.owner;
    let map = match owner.try_borrow_mut() {
        Ok(map) => map,
        Err(refbox::BorrowError::Dropped) => return None,
        Err(refbox::BorrowError::Borrowed) => panic!("client file index already borrowed"),
    };
    let key = client_files_key(elm);
    map.range((std::ops::Bound::Excluded(&key), std::ops::Bound::Unbounded))
        .next().map(|(_, file)| file.clone())
}

impl Drop for client_file {
    fn drop(&mut self) {
        unsafe { file_destroy(self) }
    }
}

#[cfg(test)]
mod file_index_ownership_tests {
    use super::*;
    use crate::src::reactor::{event_loop, shutdown_runtime};

    #[test]
    fn pending_write_check_skips_a_freed_stream() {
        unsafe {
            let mut files = client_files::default();
            let owner = file_create_with_peer(std::ptr::null_mut(), &mut files, 7, None);
            let stream = bufferevent_new(-1, None, None, None);
            (*owner.get()).event = crate::src::reactor::StreamHandle::from_ptr(stream);
            assert_eq!(file_write_left(&files), 0);
            bufferevent_write(stream, b"pending".as_ptr().cast(), 7);
            assert_eq!(file_write_left(&files), 1);
            (*owner.get()).event.free();
            assert_eq!(file_write_left(&files), 0);
            client_files_remove(&mut *owner.get());
            drop(owner);
            drop(files);
            shutdown_runtime();
        }
    }

    #[test]
    fn terminal_completion_unlinks_before_lookup_guards_release_the_allocation() {
        unsafe {
            let mut files = client_files::default();
            let file = file_create_with_peer(std::ptr::null_mut(), &mut files, 7, None);
            let observed = Rc::downgrade(&file);
            assert_eq!(Rc::strong_count(&file), 2, "caller and index each own the file");
            let guard = client_files_minmax(&files).unwrap();
            file_fire_done(&file);
            drop(file);
            event_loop();

            assert!(observed.upgrade().is_some());
            assert!(client_files_minmax(&files).is_none());
            assert!((&*guard.get()).entry.owner.is_empty());
            drop(guard);
            assert!(observed.upgrade().is_none());
            shutdown_runtime();
        }
    }

    #[test]
    fn final_drop_unlinks_by_identity_and_tolerates_a_replaced_index() {
        unsafe {
            let mut files = client_files::default();
            let first = file_create_with_peer(std::ptr::null_mut(), &mut files, 7, None);
            client_files_remove(&mut *first.get());
            drop(first);
            assert!(
                files
                    .storage
                    .as_ref()
                    .unwrap()
                    .try_borrow_mut()
                    .unwrap()
                    .is_empty()
            );

            let old = file_create_with_peer(std::ptr::null_mut(), &mut files, 7, None);
            let old_index = files.storage.as_ref().unwrap().downgrade();
            files.storage = None;
            assert!(!old_index.is_alive());
            let replacement = file_create_with_peer(std::ptr::null_mut(), &mut files, 7, None);
            drop(old);
            let found = client_files_minmax(&files).unwrap();
            assert!(Rc::ptr_eq(&found, &replacement));
            drop(found);
            client_files_remove(&mut *replacement.get());
            drop(replacement);
            assert!(client_files_minmax(&files).is_none());
        }
    }
}

#[cfg(test)]
mod completion_cancellation_tests {
    use super::*;
    use crate::src::cmd::queue::{cmdq_get_callback_owned};
    use crate::src::reactor::event_loop;

    #[test]
    fn cancelled_completion_releases_index_and_client_owners() {
        unsafe {
            let client = client::new();
            let client_observer = Rc::downgrade(&client);
            let file = file_create_with_client(Some(&client), 7, None);
            let file_observer = Rc::downgrade(&file);
            file_fire_done(&file);
            drop(file);
            drop(client);
            assert!(file_observer.upgrade().is_some());
            assert!(client_observer.upgrade().is_some());

            crate::src::reactor::shutdown_runtime();

            assert!(file_observer.upgrade().is_none());
            assert!(client_observer.upgrade().is_none());
        }
    }

    #[test]
    fn completion_callback_can_retain_its_client_after_file_retirement() {
        unsafe {
            let client = client::new();
            let observed = Rc::downgrade(&client);
            let saved = Rc::new(std::cell::RefCell::new(None));
            let callback_saved = saved.clone();
            let file = file_create_with_client(Some(&client), 7, Some(Box::new(move |event| {
                assert!(event.closed);
                *callback_saved.borrow_mut() = event.client.cloned();
            })));
            let file_observed = Rc::downgrade(&file);
            file_fire_done(&file);
            drop(file);
            drop(client);
            event_loop();
            assert!(file_observed.upgrade().is_none());
            assert!(Rc::ptr_eq(saved.borrow().as_ref().unwrap(), &observed.upgrade().unwrap()));
            drop(saved.borrow_mut().take());
            assert!(observed.upgrade().is_none());
        }
    }

}
