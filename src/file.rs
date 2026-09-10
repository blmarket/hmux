use crate::ImsgMessage;
use crate::ffi::{__errno_location, close, dup, open};
use crate::fmt_args;
use crate::fmt_engine::{FmtArg, format_buf};
use crate::log::{fatalx, log_debug};

pub use crate::consts::{
    CLIENT_ATTACHED, CLIENT_CONTROL, CLIENT_DEAD, CLIENT_EXIT_DETACH, CLIENT_EXIT_RETURN,
    CLIENT_EXIT_SHUTDOWN, E2BIG, EINVAL, ENOMEM, EV_TIMEOUT, IMSG_HEADER_SIZE, LAYOUT_LEFTRIGHT,
    LAYOUT_TOPBOTTOM, LAYOUT_WINDOWPANE, MAX_IMSGSIZE, MSG_COMMAND, MSG_DETACH, MSG_DETACHKILL,
    MSG_EXEC, MSG_EXIT, MSG_EXITED, MSG_EXITING, MSG_FLAGS, MSG_IDENTIFY_CLIENTPID,
    MSG_IDENTIFY_CWD, MSG_IDENTIFY_DONE, MSG_IDENTIFY_ENVIRON, MSG_IDENTIFY_FEATURES,
    MSG_IDENTIFY_FLAGS, MSG_IDENTIFY_LONGFLAGS, MSG_IDENTIFY_OLDCWD, MSG_IDENTIFY_STDIN,
    MSG_IDENTIFY_STDOUT, MSG_IDENTIFY_TERM, MSG_IDENTIFY_TERMINFO, MSG_IDENTIFY_TTYNAME, MSG_LOCK,
    MSG_OLDSTDERR, MSG_OLDSTDIN, MSG_OLDSTDOUT, MSG_READ, MSG_READ_CANCEL, MSG_READ_DONE,
    MSG_READ_OPEN, MSG_READY, MSG_RESIZE, MSG_SHELL, MSG_SHUTDOWN, MSG_SUSPEND, MSG_UNLOCK,
    MSG_VERSION, MSG_WAKEUP, MSG_WRITE, MSG_WRITE_CLOSE, MSG_WRITE_OPEN, MSG_WRITE_READY, O_APPEND,
    O_CREAT, O_NONBLOCK, O_WRONLY, PANE_LINES_DOUBLE, PANE_LINES_HEAVY, PANE_LINES_NUMBER,
    PANE_LINES_SIMPLE, PANE_LINES_SINGLE, PANE_LINES_SPACES, PROGRESS_BAR_ERROR,
    PROGRESS_BAR_HIDDEN, PROGRESS_BAR_INDETERMINATE, PROGRESS_BAR_NORMAL, PROGRESS_BAR_PAUSED,
    PROMPT_COMMAND, PROMPT_ENTRY, PROMPT_TYPE_COMMAND, PROMPT_TYPE_INVALID, PROMPT_TYPE_SEARCH,
    PROMPT_TYPE_TARGET, PROMPT_TYPE_WINDOW_TARGET, RB_BLACK, RB_NEGINF, RB_RED, SCREEN_CURSOR_BAR,
    SCREEN_CURSOR_BLOCK, SCREEN_CURSOR_DEFAULT, SCREEN_CURSOR_UNDERLINE, STDERR_FILENO,
    STDIN_FILENO, STDOUT_FILENO, STYLE_ALIGN_ABSOLUTE_CENTRE, STYLE_ALIGN_CENTRE,
    STYLE_ALIGN_DEFAULT, STYLE_ALIGN_LEFT, STYLE_ALIGN_RIGHT, STYLE_DEFAULT_BASE,
    STYLE_DEFAULT_POP, STYLE_DEFAULT_PUSH, STYLE_DEFAULT_SET, STYLE_LIST_FOCUS,
    STYLE_LIST_LEFT_MARKER, STYLE_LIST_OFF, STYLE_LIST_ON, STYLE_LIST_RIGHT_MARKER,
    STYLE_RANGE_CONTROL, STYLE_RANGE_LEFT, STYLE_RANGE_NONE, STYLE_RANGE_PANE, STYLE_RANGE_RIGHT,
    STYLE_RANGE_SESSION, STYLE_RANGE_USER, STYLE_RANGE_WINDOW, THEME_DARK, THEME_LIGHT,
    THEME_UNKNOWN,
};
use crate::reactor;
use crate::reactor::{Interest, Reactor};
use crate::server::{client_ref_of, server_client_get_cwd};
use crate::tmux::find_home;
pub use crate::types::*;
use crate::xmalloc::xasprintf;
use ::std::ffi::{CStr, CString, OsStr};
use ::std::fs::{self, File};
use ::std::io::Write;
use ::std::os::unix::ffi::OsStrExt;

pub const EIO: core::ffi::c_int = 5 as core::ffi::c_int;

pub const EBADF: core::ffi::c_int = 9 as core::ffi::c_int;

pub const O_RDONLY: core::ffi::c_int = 0 as core::ffi::c_int;

pub use crate::consts::EV_READ;
pub use crate::consts::EV_WRITE;
pub const STREAM_EVENT_ERROR: core::ffi::c_int = 0x20 as core::ffi::c_int;
pub const BUFFER_ERROR: core::ffi::c_int = STREAM_EVENT_ERROR;

fn file_release_io(cf: &mut client_file) {
    let event = core::mem::replace(&mut cf.event, Stream::NONE);
    event.free();
    if cf.fd != -(1 as core::ffi::c_int) {
        unsafe { close(cf.fd) };
        cf.fd = -(1 as core::ffi::c_int);
    }
}
impl Drop for client_file {
    fn drop(&mut self) {
        file_release_io(self);
        self.path = None;
        self.client_ref = None;
    }
}
const file_next_stream: crate::server_state::Value<core::ffi::c_int> =
    crate::server_state::Value::new(|state| &state.file_next_stream);
pub(crate) fn file_find_ref(
    files: &client_files_t,
    stream: core::ffi::c_int,
) -> Option<ClientFileRef> {
    files.get(&stream).cloned()
}

unsafe fn file_get_path(c: Option<&client>, file: &CStr) -> CString {
    unsafe {
        let path = if !file.to_bytes().starts_with(b"~/") {
            file.to_owned()
        } else {
            let home = find_home().unwrap_or_else(|| c"".to_owned());
            let tail = CStr::from_bytes_with_nul(&file.to_bytes_with_nul()[1..])
                .expect("a path suffix remains NUL-terminated");
            xasprintf(c"%s%s", fmt_args![home.as_ptr(), tail.as_ptr()])
        };
        if path.as_bytes().first() == Some(&{ b'/' }) {
            return path;
        }
        xasprintf(
            c"%s/%s",
            fmt_args![server_client_get_cwd(c, None).as_ptr(), path.as_ptr()],
        )
    }
}

/// The path set when the file operation was opened.
fn file_path(cf: &client_file) -> &CStr {
    cf.path
        .as_deref()
        .expect("the file was opened under a path")
}

pub(crate) fn file_can_print(c: Option<&client>) -> core::ffi::c_int {
    let Some(c) = c else {
        return 0 as core::ffi::c_int;
    };
    if c.flags & CLIENT_ATTACHED as uint64_t != 0 || c.flags & CLIENT_CONTROL as uint64_t != 0 {
        return 0 as core::ffi::c_int;
    }
    1 as core::ffi::c_int
}

pub(crate) unsafe fn file_print(c: Option<&mut client>, fmt: &CStr, args: &[FmtArg]) {
    unsafe {
        let mut msg = msg_write_open::default();
        if file_can_print(c.as_deref()) == 0 {
            return;
        }
        let c = c.expect("a client that can be printed to");
        if let Some(cf) = file_find_ref(&c.files, 1 as core::ffi::c_int) {
            let mut file_guard = cf.borrow_mut();
            let cf_ptr = &mut *file_guard;
            format_buf(&mut cf_ptr.buffer, fmt, args);
            drop(file_guard);
            cf.push();
        } else {
            let cf = ClientFileRef::create_with_client(
                Some(&mut *c),
                1 as core::ffi::c_int,
                None,
                ClientFileData::None,
            );
            let mut file_guard = cf.borrow_mut();
            let cf_ptr = &mut *file_guard;
            cf_ptr.path = Some(c"-".to_owned());
            format_buf(&mut cf_ptr.buffer, fmt, args);
            msg.stream = 1 as core::ffi::c_int;
            msg.fd = STDOUT_FILENO;
            msg.flags = 0 as core::ffi::c_int;
            (c.peer_handle()).send(
                MSG_WRITE_OPEN,
                -(1 as core::ffi::c_int),
                [msg.stream, msg.fd, msg.flags]
                    .map(|field| field.to_ne_bytes())
                    .as_flattened(),
            );
        };
    }
}
pub(crate) unsafe fn file_print_buffer(c: Option<&mut client>, data: &[u8]) {
    unsafe {
        let mut msg = msg_write_open::default();
        if file_can_print(c.as_deref()) == 0 {
            return;
        }
        let c = c.expect("a client that can be printed to");
        if let Some(cf) = file_find_ref(&c.files, 1 as core::ffi::c_int) {
            let mut file_guard = cf.borrow_mut();
            let cf_ptr = &mut *file_guard;
            cf_ptr.buffer.as_mut().append(data);
            drop(file_guard);
            cf.push();
        } else {
            let cf = ClientFileRef::create_with_client(
                Some(&mut *c),
                1 as core::ffi::c_int,
                None,
                ClientFileData::None,
            );
            let mut file_guard = cf.borrow_mut();
            let cf_ptr = &mut *file_guard;
            cf_ptr.path = Some(c"-".to_owned());
            cf_ptr.buffer.as_mut().append(data);
            msg.stream = 1 as core::ffi::c_int;
            msg.fd = STDOUT_FILENO;
            msg.flags = 0 as core::ffi::c_int;
            (c.peer_handle()).send(
                MSG_WRITE_OPEN,
                -(1 as core::ffi::c_int),
                [msg.stream, msg.fd, msg.flags]
                    .map(|field| field.to_ne_bytes())
                    .as_flattened(),
            );
        };
    }
}
pub(crate) unsafe fn file_error(c: Option<&mut client>, fmt: &CStr, args: &[FmtArg]) {
    unsafe {
        let mut msg = msg_write_open::default();
        if file_can_print(c.as_deref()) == 0 {
            return;
        }
        let c = c.expect("a client that can be printed to");
        if let Some(cf) = file_find_ref(&c.files, 2 as core::ffi::c_int) {
            let mut file_guard = cf.borrow_mut();
            let cf_ptr = &mut *file_guard;
            format_buf(&mut cf_ptr.buffer, fmt, args);
            drop(file_guard);
            cf.push();
        } else {
            let cf = ClientFileRef::create_with_client(
                Some(&mut *c),
                2 as core::ffi::c_int,
                None,
                ClientFileData::None,
            );
            let mut file_guard = cf.borrow_mut();
            let cf_ptr = &mut *file_guard;
            cf_ptr.path = Some(c"-".to_owned());
            format_buf(&mut cf_ptr.buffer, fmt, args);
            msg.stream = 2 as core::ffi::c_int;
            msg.fd = STDERR_FILENO;
            msg.flags = 0 as core::ffi::c_int;
            (c.peer_handle()).send(
                MSG_WRITE_OPEN,
                -(1 as core::ffi::c_int),
                [msg.stream, msg.fd, msg.flags]
                    .map(|field| field.to_ne_bytes())
                    .as_flattened(),
            );
        };
    }
}
pub(crate) unsafe fn file_write(
    mut c: Option<&mut client>,
    path: &CStr,
    flags: core::ffi::c_int,
    bdata: &[u8],
    cb: client_file_cb,
    cbdata: ClientFileData,
) {
    unsafe {
        let current_block: u64;
        let cf_ref: ClientFileRef;
        let msglen: size_t;
        let mut fd: core::ffi::c_int = -(1 as core::ffi::c_int);
        let fresh0 = file_next_stream.get();
        {
            let value = 1;
            file_next_stream.with_mut(|current| *current += value)
        };
        let stream: u_int = fresh0 as u_int;
        if path.to_bytes() == b"-" {
            cf_ref = ClientFileRef::create_with_client(
                c.as_deref_mut(),
                stream as core::ffi::c_int,
                cb,
                cbdata,
            );
            let mut file_guard = cf_ref.borrow_mut();
            let cf = &mut *file_guard;
            cf.path = Some(c"-".to_owned());
            fd = STDOUT_FILENO;
            if c.as_deref().is_none_or(|c| {
                c.flags & CLIENT_ATTACHED as uint64_t != 0
                    || c.flags & CLIENT_CONTROL as uint64_t != 0
            }) {
                cf.error = EBADF;
                current_block = 10126500269645651453;
            } else {
                current_block = 9838574340342979941;
            }
        } else {
            cf_ref = ClientFileRef::create_with_client(
                c.as_deref_mut(),
                stream as core::ffi::c_int,
                cb,
                cbdata,
            );
            let mut file_guard = cf_ref.borrow_mut();
            let cf = &mut *file_guard;
            cf.path = Some(file_get_path(c.as_deref(), path));
            if c.as_deref()
                .is_none_or(|c| c.flags & CLIENT_ATTACHED as uint64_t != 0)
            {
                let append = flags & O_APPEND != 0;
                let opened = File::options()
                    .write(true)
                    .create(true)
                    .append(append)
                    .truncate(!append)
                    .open(OsStr::from_bytes(file_path(&*cf).to_bytes()));
                match opened {
                    Err(err) => {
                        cf.error = err.raw_os_error().unwrap_or(EIO);
                    }
                    Ok(mut file) => {
                        if file.write_all(bdata).is_err() {
                            cf.error = EIO;
                        }
                    }
                }
                current_block = 10126500269645651453;
            } else {
                current_block = 9838574340342979941;
            }
        }
        let mut file_guard = cf_ref.borrow_mut();
        let cf = &mut *file_guard;
        if current_block == 9838574340342979941 {
            cf.buffer.as_mut().append(bdata);
            let path = file_path(&*cf).to_bytes_with_nul();
            msglen = path
                .len()
                .wrapping_add(size_of::<msg_write_open>() as size_t);
            if msglen > (MAX_IMSGSIZE as usize).wrapping_sub(IMSG_HEADER_SIZE) {
                cf.error = E2BIG;
            } else {
                let mut msg: Vec<u8> = vec![0_u8; msglen as usize];
                msg[..size_of::<msg_write_open>()].copy_from_slice(
                    [cf.stream, fd, flags]
                        .map(|field| field.to_ne_bytes())
                        .as_flattened(),
                );
                msg[size_of::<msg_write_open>()..].copy_from_slice(path);
                if (cf.peer.as_ref().expect("file peer is retained")).send(
                    MSG_WRITE_OPEN,
                    -(1 as core::ffi::c_int),
                    &msg,
                ) != 0 as core::ffi::c_int
                {
                    cf.error = EINVAL;
                } else {
                    return;
                }
            }
        }
        drop(file_guard);
        cf_ref.fire_done();
    }
}
pub(crate) unsafe fn file_read(
    mut c: Option<&mut client>,
    path: &CStr,
    cb: client_file_cb,
    cbdata: ClientFileData,
) -> Option<ClientFileRef> {
    unsafe {
        let current_block: u64;
        let cf_ref: ClientFileRef;
        let msglen: size_t;
        let mut fd: core::ffi::c_int = -(1 as core::ffi::c_int);
        let fresh1 = file_next_stream.get();
        {
            let value = 1;
            file_next_stream.with_mut(|current| *current += value)
        };
        let stream: u_int = fresh1 as u_int;
        if path.to_bytes() == b"-" {
            cf_ref = ClientFileRef::create_with_client(
                c.as_deref_mut(),
                stream as core::ffi::c_int,
                cb,
                cbdata,
            );
            let mut file_guard = cf_ref.borrow_mut();
            let cf = &mut *file_guard;
            cf.path = Some(c"-".to_owned());
            fd = STDIN_FILENO;
            if c.as_deref().is_none_or(|c| {
                c.flags & CLIENT_ATTACHED as uint64_t != 0
                    || c.flags & CLIENT_CONTROL as uint64_t != 0
            }) {
                cf.error = EBADF;
                current_block = 2435717041892024324;
            } else {
                current_block = 5418638204944806599;
            }
        } else {
            cf_ref = ClientFileRef::create_with_client(
                c.as_deref_mut(),
                stream as core::ffi::c_int,
                cb,
                cbdata,
            );
            let mut file_guard = cf_ref.borrow_mut();
            let cf = &mut *file_guard;
            cf.path = Some(file_get_path(c.as_deref(), path));
            if c.as_deref()
                .is_none_or(|c| c.flags & CLIENT_ATTACHED as uint64_t != 0)
            {
                match fs::read(OsStr::from_bytes(file_path(&*cf).to_bytes())) {
                    Ok(contents) => {
                        cf.buffer.as_mut().append(&contents);
                    }
                    Err(err) => {
                        cf.error = err.raw_os_error().unwrap_or(EIO);
                    }
                }
                current_block = 2435717041892024324;
            } else {
                current_block = 5418638204944806599;
            }
        }
        let mut file_guard = cf_ref.borrow_mut();
        let cf = &mut *file_guard;
        if current_block == 5418638204944806599 {
            let path = file_path(&*cf).to_bytes_with_nul();
            msglen = path
                .len()
                .wrapping_add(size_of::<msg_read_open>() as size_t);
            if msglen > (MAX_IMSGSIZE as usize).wrapping_sub(IMSG_HEADER_SIZE) {
                cf.error = E2BIG;
            } else {
                let mut msg: Vec<u8> = vec![0_u8; msglen as usize];
                msg[..size_of::<msg_read_open>()].copy_from_slice(
                    [cf.stream, fd]
                        .map(|field| field.to_ne_bytes())
                        .as_flattened(),
                );
                msg[size_of::<msg_read_open>()..].copy_from_slice(path);
                if (cf.peer.as_ref().expect("file peer is retained")).send(
                    MSG_READ_OPEN,
                    -(1 as core::ffi::c_int),
                    &msg,
                ) != 0 as core::ffi::c_int
                {
                    cf.error = EINVAL;
                } else {
                    drop(file_guard);
                    return Some(cf_ref);
                }
            }
        }
        drop(file_guard);
        cf_ref.fire_done();
        None
    }
}

pub(crate) fn file_write_left(files: &client_files_t) -> core::ffi::c_int {
    {
        let mut left: size_t;
        let mut waiting: core::ffi::c_int = 0 as core::ffi::c_int;
        for cf in files.values() {
            let cf = cf.borrow();
            if !cf.event.is_none() {
                left = cf.event.output_len();
                if left != 0 as size_t {
                    waiting += 1;
                    log_debug(c"file %u %zu bytes left", fmt_args![cf.stream, left]);
                }
            }
        }
        (waiting != 0 as core::ffi::c_int) as core::ffi::c_int
    }
}
/// A stream callback that runs `body` on the file it was made for, found
/// again in the tree it belongs to so that a file already given up is not
/// reached at all.
fn on_file(
    tree: FileOwner,
    stream: core::ffi::c_int,
    body: impl Fn(ClientFileRef) + 'static,
) -> std::rc::Rc<dyn Fn(Stream)> {
    std::rc::Rc::new(move |_stream| {
        let (_owner, file) = unsafe { tree.with_tree(|files| file_find_ref(files, stream)) };
        if let Some(Some(cf)) = file {
            body(cf);
        }
    })
}

/// The same, for the callback a failed stream makes.
fn on_file_error(
    tree: FileOwner,
    stream: core::ffi::c_int,
    body: impl Fn(ClientFileRef, core::ffi::c_short) + 'static,
) -> std::rc::Rc<dyn Fn(Stream, core::ffi::c_short)> {
    std::rc::Rc::new(move |_stream, what| {
        let (_owner, file) = unsafe { tree.with_tree(|files| file_find_ref(files, stream)) };
        if let Some(Some(cf)) = file {
            body(cf, what);
        }
    })
}

pub(crate) unsafe fn file_write_open(
    files: &FileOwner,
    peer: &PeerRef,
    imsg: &imsg,
    allow_streams: core::ffi::c_int,
    close_received: core::ffi::c_int,
    cb: client_file_cb,
    cbdata: ClientFileData,
) {
    unsafe {
        let payload = imsg.imsg_message_data();
        let msglen = payload.len();
        let mut reply = msg_write_ready::default();
        let flags: core::ffi::c_int = O_NONBLOCK | O_WRONLY | O_CREAT;
        let mut error: core::ffi::c_int = 0 as core::ffi::c_int;
        if msglen < size_of::<msg_write_open>() {
            fatalx(c"bad MSG_WRITE_OPEN size", fmt_args![]);
        }
        let msg = msg_write_open {
            stream: core::ffi::c_int::from_ne_bytes(
                payload[0..4].try_into().expect("message size checked"),
            ),
            fd: core::ffi::c_int::from_ne_bytes(
                payload[4..8].try_into().expect("message size checked"),
            ),
            flags: core::ffi::c_int::from_ne_bytes(
                payload[8..12].try_into().expect("message size checked"),
            ),
        };
        let path = if msglen == size_of::<msg_write_open>() {
            c"-"
        } else {
            CStr::from_bytes_until_nul(&payload[size_of::<msg_write_open>()..])
                .unwrap_or_else(|_| fatalx(c"bad MSG_WRITE_OPEN string", fmt_args![]))
        };
        log_debug(c"open write file %d %s", fmt_args![msg.stream, path]);
        if files
            .with_tree(|files| file_find_ref(files, msg.stream))
            .1
            .flatten()
            .is_some()
        {
            error = EBADF;
        } else {
            let cf_ref =
                ClientFileRef::create_with_peer(Some(peer.clone()), files, msg.stream, cb, cbdata);
            let mut file_guard = cf_ref.borrow_mut();
            let cf = &mut *file_guard;
            if cf.closed != 0 {
                error = EBADF;
            } else {
                cf.fd = -(1 as core::ffi::c_int);
                if msg.fd == -(1 as core::ffi::c_int) {
                    cf.fd = open(path.as_ptr(), msg.flags | flags, 0o644 as core::ffi::c_int);
                } else if allow_streams != 0 {
                    if msg.fd != STDOUT_FILENO && msg.fd != STDERR_FILENO {
                        *__errno_location() = EBADF;
                    } else {
                        cf.fd = dup(msg.fd);
                        if close_received != 0 {
                            close(msg.fd);
                        }
                    }
                } else {
                    *__errno_location() = EBADF;
                }
                if cf.fd == -(1 as core::ffi::c_int) {
                    error = *__errno_location();
                } else {
                    cf.event = Stream::new(
                        cf.fd,
                        None,
                        Some(on_file(cf.tree.clone(), cf.stream, |cf| cf.on_write())),
                        Some(on_file_error(cf.tree.clone(), cf.stream, |cf, _what| {
                            cf.on_write_error()
                        })),
                    );
                    if cf.event.is_none() {
                        fatalx(c"out of memory", fmt_args![]);
                    }
                    cf.event.enable(Interest::Write);
                }
            }
        }
        reply.stream = msg.stream;
        reply.error = error;
        peer.send(
            MSG_WRITE_READY,
            -(1 as core::ffi::c_int),
            [reply.stream, reply.error]
                .map(|field| field.to_ne_bytes())
                .as_flattened(),
        );
    }
}
pub(crate) fn file_write_data(files: &client_files_t, imsg: &imsg) {
    {
        let payload = imsg.imsg_message_data();
        let msglen = payload.len();
        let size: size_t = msglen.wrapping_sub(size_of::<msg_write_data>() as size_t);
        if msglen < size_of::<msg_write_data>() {
            fatalx(c"bad MSG_WRITE size", fmt_args![]);
        }
        let msg = msg_write_data {
            stream: core::ffi::c_int::from_ne_bytes(
                payload[0..4].try_into().expect("message size checked"),
            ),
        };
        let Some(cf_ref) = file_find_ref(files, msg.stream) else {
            fatalx(c"unknown stream number", fmt_args![]);
        };
        let mut file_guard = cf_ref.borrow_mut();
        let cf = &mut *file_guard;
        log_debug(c"write %zu to file %d", fmt_args![size, cf.stream]);
        if !cf.event.is_none() {
            let data = &payload[size_of::<msg_write_data>()..];
            cf.event.write(data);
        }
    }
}
pub(crate) unsafe fn file_write_close(files: &FileOwner, imsg: &imsg) {
    unsafe {
        let payload = imsg.imsg_message_data();
        let msglen = payload.len();
        if msglen != size_of::<msg_write_close>() {
            fatalx(c"bad MSG_WRITE_CLOSE size", fmt_args![]);
        }
        let msg = msg_write_close {
            stream: core::ffi::c_int::from_ne_bytes(
                payload[0..4].try_into().expect("message size checked"),
            ),
        };
        let Some(cf_ref) = files
            .with_tree(|files| file_find_ref(files, msg.stream))
            .1
            .flatten()
        else {
            fatalx(c"unknown stream number", fmt_args![]);
        };
        let mut file_guard = cf_ref.borrow_mut();
        let cf = &mut *file_guard;
        log_debug(c"close file %d", fmt_args![cf.stream]);
        if cf.event.is_none() || cf.event.output_len() == 0 as size_t {
            file_release_io(&mut *cf);
            drop(file_guard);
            cf_ref.close();
        }
    }
}

pub(crate) unsafe fn file_read_open(
    files: &FileOwner,
    peer: &PeerRef,
    imsg: &imsg,
    allow_streams: core::ffi::c_int,
    close_received: core::ffi::c_int,
    cb: client_file_cb,
    cbdata: ClientFileData,
) {
    unsafe {
        let payload = imsg.imsg_message_data();
        let msglen = payload.len();
        let mut reply = msg_read_done::default();
        let flags: core::ffi::c_int = O_NONBLOCK | O_RDONLY;
        let error: core::ffi::c_int;
        if msglen < size_of::<msg_read_open>() {
            fatalx(c"bad MSG_READ_OPEN size", fmt_args![]);
        }
        let msg = msg_read_open {
            stream: core::ffi::c_int::from_ne_bytes(
                payload[0..4].try_into().expect("message size checked"),
            ),
            fd: core::ffi::c_int::from_ne_bytes(
                payload[4..8].try_into().expect("message size checked"),
            ),
        };
        let path = if msglen == size_of::<msg_read_open>() {
            c"-"
        } else {
            CStr::from_bytes_until_nul(&payload[size_of::<msg_read_open>()..])
                .unwrap_or_else(|_| fatalx(c"bad MSG_READ_OPEN string", fmt_args![]))
        };
        log_debug(c"open read file %d %s", fmt_args![msg.stream, path]);
        if files
            .with_tree(|files| file_find_ref(files, msg.stream))
            .1
            .flatten()
            .is_some()
        {
            error = EBADF;
        } else {
            let cf_ref =
                ClientFileRef::create_with_peer(Some(peer.clone()), files, msg.stream, cb, cbdata);
            let mut file_guard = cf_ref.borrow_mut();
            let cf = &mut *file_guard;
            if cf.closed != 0 {
                error = EBADF;
            } else {
                cf.fd = -(1 as core::ffi::c_int);
                if msg.fd == -(1 as core::ffi::c_int) {
                    cf.fd = open(path.as_ptr(), flags);
                } else if allow_streams != 0 {
                    if msg.fd != STDIN_FILENO {
                        *__errno_location() = EBADF;
                    } else {
                        cf.fd = dup(msg.fd);
                        if close_received != 0 {
                            close(msg.fd);
                        }
                    }
                } else {
                    *__errno_location() = EBADF;
                }
                if cf.fd == -(1 as core::ffi::c_int) {
                    error = *__errno_location();
                } else {
                    cf.event = Stream::new(
                        cf.fd,
                        Some(on_file(cf.tree.clone(), cf.stream, |cf| cf.on_read())),
                        None,
                        Some(on_file_error(cf.tree.clone(), cf.stream, |cf, what| {
                            cf.on_read_error(what)
                        })),
                    );
                    if cf.event.is_none() {
                        fatalx(c"out of memory", fmt_args![]);
                    }
                    cf.event.enable(Interest::Read);
                    return;
                }
            }
        }
        reply.stream = msg.stream;
        reply.error = error;
        peer.send(
            MSG_READ_DONE,
            -(1 as core::ffi::c_int),
            [reply.stream, reply.error]
                .map(|field| field.to_ne_bytes())
                .as_flattened(),
        );
    }
}
pub(crate) unsafe fn file_read_cancel(files: &FileOwner, imsg: &imsg) {
    unsafe {
        let payload = imsg.imsg_message_data();
        let msglen = payload.len();
        if msglen != size_of::<msg_read_cancel>() {
            fatalx(c"bad MSG_READ_CANCEL size", fmt_args![]);
        }
        let msg = msg_read_cancel {
            stream: core::ffi::c_int::from_ne_bytes(
                payload[0..4].try_into().expect("message size checked"),
            ),
        };
        let Some(cf_ref) = files
            .with_tree(|files| file_find_ref(files, msg.stream))
            .1
            .flatten()
        else {
            fatalx(c"unknown stream number", fmt_args![]);
        };
        let mut file_guard = cf_ref.borrow_mut();
        let cf = &mut *file_guard;
        log_debug(c"cancel file %d", fmt_args![cf.stream]);
        drop(file_guard);
        cf_ref.on_read_error(0 as core::ffi::c_short);
    }
}

#[cfg(test)]
#[path = "file_focused_tests.rs"]
mod focused_tests;

impl ClientFileRef {
    /// Takes `cf` out of the tree it is in, if that tree still holds it. The
    /// Removing an entry is logical closure; other strong handles can keep the
    /// allocation alive for deferred callbacks.
    unsafe fn unlink(&self) {
        let owner = self;

        let (tree, stream) = {
            let mut file = owner.borrow_mut();
            (std::mem::take(&mut file.tree), file.stream)
        };
        let (_held, removed) = unsafe {
            tree.with_tree(|files| {
                if files.get(&stream).is_some_and(|file| file.ptr_eq(owner)) {
                    files.remove(&stream)
                } else {
                    None
                }
            })
        };
        drop(removed);
    }
    pub(crate) unsafe fn create_with_peer(
        peer: Option<PeerRef>,
        files: &FileOwner,
        stream: core::ffi::c_int,
        cb: client_file_cb,
        cbdata: ClientFileData,
    ) -> ClientFileRef {
        unsafe {
            let cf = ClientFileRef::new(client_file {
                client_ref: None,
                peer,
                tree: files.clone(),
                stream,
                path: None,
                buffer: Box::new(ByteBuffer::new()),
                event: Stream::NONE,
                fd: -(1 as core::ffi::c_int),
                error: 0,
                closed: 0,
                done: 0,
                cb,
                data: cbdata,
            });
            files
                .with_tree(|files| files.insert(stream, cf.clone()))
                .1
                .expect("a file set is present during insertion");
            cf
        }
    }
    pub(crate) fn create_with_client(
        mut c: Option<&mut client>,
        stream: core::ffi::c_int,
        cb: client_file_cb,
        cbdata: ClientFileData,
    ) -> ClientFileRef {
        if c.as_deref()
            .is_some_and(|c| c.flags & CLIENT_ATTACHED as uint64_t != 0)
        {
            c = None;
        }
        let client_ref = c.as_deref().and_then(client_ref_of);
        let peer = c.as_deref().and_then(|c| c.peer.clone());
        let tree = match client_ref.as_ref() {
            Some(held) => FileOwner::Client(held.downgrade()),
            None => FileOwner::None,
        };
        let cf = ClientFileRef::new(client_file {
            client_ref,
            peer,
            tree,
            stream,
            path: None,
            buffer: Box::new(ByteBuffer::new()),
            event: Stream::NONE,
            fd: -(1 as core::ffi::c_int),
            error: 0,
            closed: 0,
            done: 0,
            cb,
            data: cbdata,
        });
        if let Some(c) = c {
            c.files.insert(stream, cf.clone());
        }
        cf
    }
    pub(crate) unsafe fn close(self) {
        let cf = self;

        cf.borrow_mut().done = 1;
        unsafe { cf.unlink() };
    }
    pub(crate) unsafe fn fire_done(self) {
        let cf = self;

        {
            let mut file = cf.borrow_mut();
            if file.done != 0 {
                return;
            }
            file.done = 1;
        }
        reactor::current().defer(move || unsafe {
            let (callback, event, discarded) = {
                let mut file = cf.borrow_mut();
                let client = file.client();
                let deliver = file.cb.is_some()
                    && (file.closed != 0
                        || client
                            .as_ref()
                            .map(|reference| reference.as_client())
                            .is_none_or(|client| client.flags & CLIENT_DEAD as uint64_t == 0));
                let data = std::mem::take(&mut file.data);
                if deliver {
                    let callback = file.cb.clone();
                    let event = ClientFileEvent::Done {
                        client: file.client_ref.clone(),
                        path: file
                            .path
                            .take()
                            .expect("a completed file operation has a path"),
                        error: file.error,
                        buffer: std::mem::take(file.buffer.as_mut()),
                        data,
                    };
                    (callback, Some(event), None)
                } else {
                    (None, None, Some(data))
                }
            };
            if let (Some(callback), Some(event)) = (callback, event) {
                callback(event);
            }
            cf.close();
            drop(discarded);
        });
    }
    pub(crate) fn fire_read(&self) {
        let cf = self;

        let (callback, client, error, mut buffer, data) = {
            let mut file = cf.borrow_mut();
            let Some(callback) = file.cb.clone() else {
                return;
            };
            (
                callback,
                file.client_ref.clone(),
                file.error,
                std::mem::take(file.buffer.as_mut()),
                file.data.clone(),
            )
        };
        callback(ClientFileEvent::Read {
            client: client.as_ref(),
            error,
            buffer: &mut buffer,
            data: &data,
        });
        let mut file = cf.borrow_mut();
        buffer.append_buf(file.buffer.as_mut());
        *file.buffer = buffer;
    }
    pub(crate) unsafe fn cancel(self) {
        let cf = self;

        unsafe {
            let mut file_guard = cf.borrow_mut();
            let cf = &mut *file_guard;
            let mut msg: msg_read_cancel = msg_read_cancel { stream: 0 };
            log_debug(c"read cancel file %d", fmt_args![cf.stream]);
            if cf.closed != 0 {
                return;
            }
            cf.closed = 1 as core::ffi::c_int;
            msg.stream = cf.stream;
            (cf.peer.as_ref().expect("file peer is retained")).send(
                MSG_READ_CANCEL,
                -(1 as core::ffi::c_int),
                &msg.stream.to_ne_bytes(),
            );
        }
    }
    pub(crate) unsafe fn push(self) {
        let cf = self;

        unsafe {
            const MAX_DATA: usize =
                (MAX_IMSGSIZE as usize) - IMSG_HEADER_SIZE - size_of::<msg_write_data>();
            let mut file_guard = cf.borrow_mut();
            let cf_ptr = &mut *file_guard;
            let mut close_0: msg_write_close = msg_write_close { stream: 0 };
            let mut left: size_t = cf_ptr.buffer.as_ref().len();
            while left != 0 as size_t {
                let sent = left.min(MAX_DATA);
                let mut msg: Vec<u8> = vec![0_u8; size_of::<msg_write_data>()];
                msg.copy_from_slice(&cf_ptr.stream.to_ne_bytes());
                msg.extend_from_slice(cf_ptr.buffer.as_mut().pullup(sent));
                if (cf_ptr.peer.as_ref().expect("file peer is retained")).send(
                    MSG_WRITE,
                    -(1 as core::ffi::c_int),
                    &msg,
                ) != 0 as core::ffi::c_int
                {
                    break;
                }
                cf_ptr.buffer.as_mut().drain(sent);
                left = cf_ptr.buffer.as_ref().len();
                log_debug(
                    c"file %d sent %zu, left %zu",
                    fmt_args![cf_ptr.stream, sent, left],
                );
            }
            if left != 0 as size_t {
                drop(file_guard);
                reactor::current().defer(move || {
                    let mut file_guard = cf.borrow_mut();
                    let cf_ptr = &mut *file_guard;
                    let c = (*cf_ptr).client();
                    if c.as_ref()
                        .map(|reference| reference.as_client())
                        .is_none_or(|c| !c.flags & CLIENT_DEAD as uint64_t != 0)
                    {
                        drop(file_guard);
                        cf.push();
                    } else {
                        drop(file_guard);
                        cf.close();
                    }
                });
            } else if cf_ptr.stream > 2 as core::ffi::c_int {
                close_0.stream = cf_ptr.stream;
                (cf_ptr.peer.as_ref().expect("file peer is retained")).send(
                    MSG_WRITE_CLOSE,
                    -(1 as core::ffi::c_int),
                    &close_0.stream.to_ne_bytes(),
                );
                drop(file_guard);
                cf.fire_done();
            }
        }
    }
    fn on_write_error(self) {
        let cf_ref = self;

        let callback = {
            let mut cf = cf_ref.borrow_mut();
            log_debug(c"write error file %d", fmt_args![cf.stream]);
            file_release_io(&mut cf);
            cf.cb.clone()
        };
        if let Some(callback) = callback {
            callback(ClientFileEvent::CheckExit);
        }
    }
    unsafe fn on_write(self) {
        let cf_ref = self;

        let callback = {
            let cf = cf_ref.borrow();
            log_debug(c"write check file %d", fmt_args![cf.stream]);
            cf.cb.clone()
        };
        if let Some(callback) = callback {
            callback(ClientFileEvent::CheckExit);
        }
        let close = {
            let mut cf = cf_ref.borrow_mut();
            if cf.closed != 0 && cf.event.output_len() == 0 {
                file_release_io(&mut cf);
                true
            } else {
                false
            }
        };
        if close {
            unsafe { cf_ref.close() };
        }
    }
    unsafe fn on_read_error(self, what: core::ffi::c_short) {
        let cf_ref = self;

        unsafe {
            let mut file_guard = cf_ref.borrow_mut();
            let cf = &mut *file_guard;
            let mut msg = msg_read_done::default();
            log_debug(c"read error file %d", fmt_args![cf.stream]);
            msg.stream = cf.stream;
            msg.error = if what as core::ffi::c_int & BUFFER_ERROR != 0 {
                EIO
            } else {
                0 as core::ffi::c_int
            };
            (cf.peer.as_ref().expect("file peer is retained")).send(
                MSG_READ_DONE,
                -(1 as core::ffi::c_int),
                [msg.stream, msg.error]
                    .map(|field| field.to_ne_bytes())
                    .as_flattened(),
            );
            file_release_io(&mut *cf);
            drop(file_guard);
            cf_ref.close();
        }
    }
    unsafe fn on_read(self) {
        let cf_ref = self;

        unsafe {
            let mut file_guard = cf_ref.borrow_mut();
            let cf = &mut *file_guard;
            let mut msg: Vec<u8> = vec![0; size_of::<msg_read_data>()];
            loop {
                let limit = (MAX_IMSGSIZE as usize)
                    .wrapping_sub(IMSG_HEADER_SIZE)
                    .wrapping_sub(size_of::<msg_read_data>());
                let data = cf.event.with_input(|buffer| buffer.copy_to_bytes(limit));
                let Some(data) = data else {
                    break;
                };
                let bsize = data.len();
                if bsize == 0 as size_t {
                    break;
                }
                log_debug(c"read %zu from file %d", fmt_args![bsize, cf.stream]);
                msg.truncate(size_of::<msg_read_data>());
                msg.extend_from_slice(&data);
                msg[..size_of::<msg_read_data>()].copy_from_slice(&cf.stream.to_ne_bytes());
                (cf.peer.as_ref().expect("file peer is retained")).send(
                    MSG_READ,
                    -(1 as core::ffi::c_int),
                    &msg,
                );
            }
        }
    }
}

impl ClientRef {
    pub(crate) unsafe fn handle_file_write_ready(&self, imsg: &imsg) -> core::ffi::c_int {
        let client = self;

        unsafe {
            let payload = imsg.imsg_message_data();
            let msglen = payload.len();
            if msglen != size_of::<msg_write_ready>() {
                return -(1 as core::ffi::c_int);
            }
            let msg = msg_write_ready {
                stream: core::ffi::c_int::from_ne_bytes(
                    payload[0..4].try_into().expect("message size checked"),
                ),
                error: core::ffi::c_int::from_ne_bytes(
                    payload[4..8].try_into().expect("message size checked"),
                ),
            };
            let Some(cf_ref) = file_find_ref(&client.as_client().files, msg.stream) else {
                return 0 as core::ffi::c_int;
            };
            let mut file_guard = cf_ref.borrow_mut();
            let cf = &mut *file_guard;
            if msg.error != 0 as core::ffi::c_int {
                cf.error = msg.error;
                drop(file_guard);
                cf_ref.fire_done();
            } else {
                drop(file_guard);
                cf_ref.push();
            }
            0 as core::ffi::c_int
        }
    }
    pub(crate) unsafe fn handle_file_read_data(&self, imsg: &imsg) -> core::ffi::c_int {
        let client = self;

        unsafe {
            let payload = imsg.imsg_message_data();
            let msglen = payload.len();
            let bsize: size_t = msglen.wrapping_sub(size_of::<msg_read_data>() as size_t);
            if msglen < size_of::<msg_read_data>() {
                return -(1 as core::ffi::c_int);
            }
            let msg = msg_read_data {
                stream: core::ffi::c_int::from_ne_bytes(
                    payload[0..4].try_into().expect("message size checked"),
                ),
            };
            let Some(cf_ref) = file_find_ref(&client.as_client().files, msg.stream) else {
                return 0 as core::ffi::c_int;
            };
            let mut file_guard = cf_ref.borrow_mut();
            let cf = &mut *file_guard;
            log_debug(c"file %d read %zu bytes", fmt_args![cf.stream, bsize]);
            if cf.error == 0 as core::ffi::c_int && cf.closed == 0 {
                cf.buffer
                    .as_mut()
                    .append(&payload[size_of::<msg_read_data>()..]);
                drop(file_guard);
                cf_ref.fire_read();
            }
            0 as core::ffi::c_int
        }
    }
    pub(crate) unsafe fn handle_file_read_done(&self, imsg: &imsg) -> core::ffi::c_int {
        let client = self;

        unsafe {
            let payload = imsg.imsg_message_data();
            let msglen = payload.len();
            if msglen != size_of::<msg_read_done>() {
                return -(1 as core::ffi::c_int);
            }
            let msg = msg_read_done {
                stream: core::ffi::c_int::from_ne_bytes(
                    payload[0..4].try_into().expect("message size checked"),
                ),
                error: core::ffi::c_int::from_ne_bytes(
                    payload[4..8].try_into().expect("message size checked"),
                ),
            };
            let Some(cf_ref) = file_find_ref(&client.as_client().files, msg.stream) else {
                return 0 as core::ffi::c_int;
            };
            let mut file_guard = cf_ref.borrow_mut();
            let cf = &mut *file_guard;
            log_debug(c"file %d read done", fmt_args![cf.stream]);
            cf.error = msg.error;
            drop(file_guard);
            cf_ref.fire_done();
            0 as core::ffi::c_int
        }
    }
}

/// Starts file input using the existing client/file ownership and completion path.
/// An absent client retains the existing server-side path policy. Completion may
/// be immediate on errors; callback data is consumed exactly once.
///
/// # Safety
/// Run on the server thread without conflicting client, TTY, session or queue
/// payload access. Existing replacement/completion callbacks may run inline;
/// exclude conflicting callback state access. No payload reference escapes.
pub(crate) unsafe fn file_read_for_client(
    c: Option<&mut ClientRef>,
    path: &CStr,
    cb: client_file_cb,
    cbdata: ClientFileData,
) -> Option<ClientFileRef> {
    unsafe { file_read(c.map(|c| c.as_client_mut()), path, cb, cbdata) }
}

/// Starts file output using existing stream ownership, flags and callback delivery.
/// An absent client retains the existing server-side path policy. Data is copied
/// by the file implementation; completion may be immediate on errors.
///
/// # Safety
/// Run on the server thread without conflicting client, TTY, session or queue
/// payload access. Existing replacement/completion callbacks may run inline;
/// exclude conflicting callback state access. No payload reference escapes.
pub(crate) unsafe fn file_write_for_client(
    c: Option<&mut ClientRef>,
    path: &CStr,
    flags: core::ffi::c_int,
    bdata: &[u8],
    cb: client_file_cb,
    cbdata: ClientFileData,
) {
    unsafe { file_write(c.map(|c| c.as_client_mut()), path, flags, bdata, cb, cbdata) }
}
