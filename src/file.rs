use crate::ffi::{__errno_location, close, dup, open, strcmp, strncmp};
use crate::fmt_args;
use crate::fmt_engine::{FmtArg, format_buf};
use crate::log::{fatalx, log_debug};
use crate::proc::{peer_ptr, proc_send};
use crate::reactor;
use crate::reactor::{Interest, Reactor};
use crate::server::{client_ref_from_ptr, server_client_get_cwd};
use crate::tmux::find_home;
pub use crate::types::*;
use crate::xmalloc::xasprintf;
use ::std::ffi::{CStr, CString, OsStr};
use ::std::fs::{self, File};
use ::std::io::Write;
use ::std::os::unix::ffi::OsStrExt;
pub const MSG_READ_CANCEL: msgtype = 307;
pub const MSG_WRITE_CLOSE: msgtype = 306;
pub const MSG_WRITE_READY: msgtype = 305;
pub const MSG_WRITE: msgtype = 304;
pub const MSG_WRITE_OPEN: msgtype = 303;
pub const MSG_READ_DONE: msgtype = 302;
pub const MSG_READ: msgtype = 301;
pub const MSG_READ_OPEN: msgtype = 300;
pub const MSG_FLAGS: msgtype = 218;
pub const MSG_EXEC: msgtype = 217;
pub const MSG_WAKEUP: msgtype = 216;
pub const MSG_UNLOCK: msgtype = 215;
pub const MSG_SUSPEND: msgtype = 214;
pub const MSG_OLDSTDOUT: msgtype = 213;
pub const MSG_OLDSTDIN: msgtype = 212;
pub const MSG_OLDSTDERR: msgtype = 211;
pub const MSG_SHUTDOWN: msgtype = 210;
pub const MSG_SHELL: msgtype = 209;
pub const MSG_RESIZE: msgtype = 208;
pub const MSG_READY: msgtype = 207;
pub const MSG_LOCK: msgtype = 206;
pub const MSG_EXITING: msgtype = 205;
pub const MSG_EXITED: msgtype = 204;
pub const MSG_EXIT: msgtype = 203;
pub const MSG_DETACHKILL: msgtype = 202;
pub const MSG_DETACH: msgtype = 201;
pub const MSG_COMMAND: msgtype = 200;
pub const MSG_IDENTIFY_TERMINFO: msgtype = 112;
pub const MSG_IDENTIFY_LONGFLAGS: msgtype = 111;
pub const MSG_IDENTIFY_STDOUT: msgtype = 110;
pub const MSG_IDENTIFY_FEATURES: msgtype = 109;
pub const MSG_IDENTIFY_CWD: msgtype = 108;
pub const MSG_IDENTIFY_CLIENTPID: msgtype = 107;
pub const MSG_IDENTIFY_DONE: msgtype = 106;
pub const MSG_IDENTIFY_ENVIRON: msgtype = 105;
pub const MSG_IDENTIFY_STDIN: msgtype = 104;
pub const MSG_IDENTIFY_OLDCWD: msgtype = 103;
pub const MSG_IDENTIFY_TTYNAME: msgtype = 102;
pub const MSG_IDENTIFY_TERM: msgtype = 101;
pub const MSG_IDENTIFY_FLAGS: msgtype = 100;
pub const MSG_VERSION: msgtype = 12;
pub const PANE_LINES_SPACES: pane_lines = 5;
pub const PANE_LINES_NUMBER: pane_lines = 4;
pub const PANE_LINES_SIMPLE: pane_lines = 3;
pub const PANE_LINES_HEAVY: pane_lines = 2;
pub const PANE_LINES_DOUBLE: pane_lines = 1;
pub const PANE_LINES_SINGLE: pane_lines = 0;
pub const PROGRESS_BAR_PAUSED: progress_bar_state = 4;
pub const PROGRESS_BAR_INDETERMINATE: progress_bar_state = 3;
pub const PROGRESS_BAR_ERROR: progress_bar_state = 2;
pub const PROGRESS_BAR_NORMAL: progress_bar_state = 1;
pub const PROGRESS_BAR_HIDDEN: progress_bar_state = 0;
pub const SCREEN_CURSOR_BAR: screen_cursor_style = 3;
pub const SCREEN_CURSOR_UNDERLINE: screen_cursor_style = 2;
pub const SCREEN_CURSOR_BLOCK: screen_cursor_style = 1;
pub const SCREEN_CURSOR_DEFAULT: screen_cursor_style = 0;
pub const STYLE_DEFAULT_SET: style_default_type = 3;
pub const STYLE_DEFAULT_POP: style_default_type = 2;
pub const STYLE_DEFAULT_PUSH: style_default_type = 1;
pub const STYLE_DEFAULT_BASE: style_default_type = 0;
pub const STYLE_RANGE_CONTROL: style_range_type = 7;
pub const STYLE_RANGE_USER: style_range_type = 6;
pub const STYLE_RANGE_SESSION: style_range_type = 5;
pub const STYLE_RANGE_WINDOW: style_range_type = 4;
pub const STYLE_RANGE_PANE: style_range_type = 3;
pub const STYLE_RANGE_RIGHT: style_range_type = 2;
pub const STYLE_RANGE_LEFT: style_range_type = 1;
pub const STYLE_RANGE_NONE: style_range_type = 0;
pub const STYLE_LIST_RIGHT_MARKER: style_list = 4;
pub const STYLE_LIST_LEFT_MARKER: style_list = 3;
pub const STYLE_LIST_FOCUS: style_list = 2;
pub const STYLE_LIST_ON: style_list = 1;
pub const STYLE_LIST_OFF: style_list = 0;
pub const STYLE_ALIGN_ABSOLUTE_CENTRE: style_align = 4;
pub const STYLE_ALIGN_RIGHT: style_align = 3;
pub const STYLE_ALIGN_CENTRE: style_align = 2;
pub const STYLE_ALIGN_LEFT: style_align = 1;
pub const STYLE_ALIGN_DEFAULT: style_align = 0;
pub const THEME_DARK: client_theme = 2;
pub const THEME_LIGHT: client_theme = 1;
pub const THEME_UNKNOWN: client_theme = 0;
pub const LAYOUT_WINDOWPANE: layout_type = 2;
pub const LAYOUT_TOPBOTTOM: layout_type = 1;
pub const LAYOUT_LEFTRIGHT: layout_type = 0;
pub const PROMPT_TYPE_INVALID: prompt_type = 255;
pub const PROMPT_TYPE_WINDOW_TARGET: prompt_type = 3;
pub const PROMPT_TYPE_TARGET: prompt_type = 2;
pub const PROMPT_TYPE_SEARCH: prompt_type = 1;
pub const PROMPT_TYPE_COMMAND: prompt_type = 0;
pub const PROMPT_COMMAND: client_prompt_mode = 1;
pub const PROMPT_ENTRY: client_prompt_mode = 0;
pub const CLIENT_EXIT_DETACH: client_exit_type = 2;
pub const CLIENT_EXIT_SHUTDOWN: client_exit_type = 1;
pub const CLIENT_EXIT_RETURN: client_exit_type = 0;
pub const EIO: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const E2BIG: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const EBADF: ::core::ffi::c_int = 9 as ::core::ffi::c_int;
pub const ENOMEM: ::core::ffi::c_int = 12 as ::core::ffi::c_int;
pub const EINVAL: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const O_RDONLY: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const O_WRONLY: ::core::ffi::c_int = 0o1 as ::core::ffi::c_int;
pub const O_CREAT: ::core::ffi::c_int = 0o100 as ::core::ffi::c_int;
pub const O_APPEND: ::core::ffi::c_int = 0o2000 as ::core::ffi::c_int;
pub const O_NONBLOCK: ::core::ffi::c_int = 0o4000 as ::core::ffi::c_int;
pub const STDIN_FILENO: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const STDOUT_FILENO: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const STDERR_FILENO: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const RB_BLACK: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RB_RED: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const RB_NEGINF: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const EV_TIMEOUT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const EV_READ: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const EV_WRITE: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const STREAM_EVENT_ERROR: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const BUFFER_ERROR: ::core::ffi::c_int = STREAM_EVENT_ERROR;
pub const IMSG_HEADER_SIZE: usize = ::core::mem::size_of::<imsg_hdr>();
pub const MAX_IMSGSIZE: ::core::ffi::c_int = 16384 as ::core::ffi::c_int;
pub const CLIENT_ATTACHED: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const CLIENT_DEAD: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const CLIENT_CONTROL: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
fn file_release_io(cf: &mut client_file) {
    let event = ::core::mem::replace(&mut cf.event, Stream::NONE);
    event.free();
    if cf.fd != -(1 as ::core::ffi::c_int) {
        unsafe { close(cf.fd) };
        cf.fd = -(1 as ::core::ffi::c_int);
    }
}
impl Drop for client_file {
    fn drop(&mut self) {
        file_release_io(self);
        self.path = None;
        self.client_ref = None;
    }
}
static mut file_next_stream: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub(crate) unsafe fn file_find_ref(
    files: *mut client_files_t,
    stream: ::core::ffi::c_int,
) -> Option<ClientFileRef> {
    unsafe { (*files).get(&stream).cloned() }
}

/// Takes `cf` out of the tree it is in, if that tree still holds it. The
/// Removing an entry is logical closure; other strong handles can keep the
/// allocation alive for deferred callbacks.
unsafe fn file_unlink(cf: *mut client_file) {
    unsafe {
        let files = (*cf).tree.tree();
        if files.is_null() {
            return;
        }
        if (*files)
            .get(&(*cf).stream)
            .map(|file| file.as_ptr() == cf)
            .unwrap_or(false)
        {
            (*files).remove(&(*cf).stream);
        }
        (*cf).tree = FileOwner::None;
    }
}
unsafe fn file_get_path(mut c: *mut client, mut file: *const ::core::ffi::c_char) -> CString {
    unsafe {
        let path = if strncmp(file, c"~/".as_ptr(), 2 as size_t) != 0 as ::core::ffi::c_int {
            CStr::from_ptr(file).to_owned()
        } else {
            let home = find_home().unwrap_or(c"");
            xasprintf(
                c"%s%s".as_ptr(),
                fmt_args![home.as_ptr(), file.offset(1 as ::core::ffi::c_int as isize)],
            )
        };
        if path.as_bytes().first() == Some(&{ b'/' }) {
            return path;
        }
        xasprintf(
            c"%s/%s".as_ptr(),
            fmt_args![
                server_client_get_cwd(c, ::core::ptr::null_mut::<session>()),
                path.as_ptr()
            ],
        )
    }
}
pub(crate) unsafe fn file_create_with_peer(
    mut peer: *mut tmuxpeer,
    mut files: *mut client_files_t,
    mut stream: ::core::ffi::c_int,
    mut cb: client_file_cb,
    mut cbdata: ClientFileData,
) -> ClientFileRef {
    unsafe {
        let cf = ClientFileRef::new(client_file {
            client_ref: None,
            peer,
            tree: FileOwner::Held(files),
            stream,
            path: None,
            buffer: Box::new(Buf::new()),
            event: Stream::NONE,
            fd: -(1 as ::core::ffi::c_int),
            error: 0,
            closed: 0,
            done: 0,
            cb,
            data: cbdata,
        });
        (*files).insert(stream, cf.clone());
        cf
    }
}
pub(crate) unsafe fn file_create_with_client(
    mut c: *mut client,
    mut stream: ::core::ffi::c_int,
    mut cb: client_file_cb,
    mut cbdata: ClientFileData,
) -> ClientFileRef {
    unsafe {
        if !c.is_null() && (*c).flags & CLIENT_ATTACHED as uint64_t != 0 {
            c = ::core::ptr::null_mut::<client>();
        }
        let peer = if c.is_null() {
            ::core::ptr::null_mut()
        } else {
            peer_ptr(&(*c).peer)
        };
        let client_ref = client_ref_from_ptr(c);
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
            buffer: Box::new(Buf::new()),
            event: Stream::NONE,
            fd: -(1 as ::core::ffi::c_int),
            error: 0,
            closed: 0,
            done: 0,
            cb,
            data: cbdata,
        });
        if !c.is_null() {
            (*c).files.insert(stream, cf.clone());
        }
        cf
    }
}
pub(crate) unsafe fn file_free(cf: ClientFileRef) {
    unsafe {
        (*cf.as_ptr()).done = 1 as ::core::ffi::c_int;
        file_unlink(cf.as_ptr());
    }
}
pub(crate) unsafe fn file_fire_done(cf: ClientFileRef) {
    unsafe {
        if (*cf.as_ptr()).done != 0 as ::core::ffi::c_int {
            return;
        }
        (*cf.as_ptr()).done = 1 as ::core::ffi::c_int;
        reactor::current().defer(move || {
            let cf_ptr = cf.as_ptr();
            let c = (*cf_ptr).client();
            let call_callback = (*cf_ptr).cb.is_some()
                && ((*cf_ptr).closed != 0
                    || c.is_null()
                    || !(*c).flags & CLIENT_DEAD as uint64_t != 0);
            let data = ::std::mem::take(&mut (*cf_ptr).data);
            if call_callback {
                (*cf_ptr).cb.expect("non-null function pointer")(
                    c,
                    cstr_ptr(&(*cf_ptr).path),
                    (*cf_ptr).error,
                    1 as ::core::ffi::c_int,
                    &raw mut *(*cf_ptr).buffer,
                    data,
                );
            } else {
                file_free(cf.clone());
                drop(data);
                return;
            }
            file_free(cf);
        });
    }
}
/// The path the file was opened under, which every path through
/// [`file_read`] and [`file_write`] has set before it is read back.
unsafe fn file_path(cf: *const client_file) -> &'static CStr {
    unsafe {
        (*cf)
            .path
            .as_deref()
            .expect("the file was opened under a path")
    }
}

pub(crate) unsafe fn file_fire_read(cf: &ClientFileRef) {
    unsafe {
        let cf = cf.as_ptr();
        if (*cf).cb.is_some() {
            (*cf).cb.expect("non-null function pointer")(
                (*cf).client(),
                cstr_ptr(&(*cf).path),
                (*cf).error,
                0 as ::core::ffi::c_int,
                &raw mut *(*cf).buffer,
                (*cf).data.view(),
            );
        }
    }
}
pub(crate) unsafe fn file_can_print(mut c: *mut client) -> ::core::ffi::c_int {
    unsafe {
        if c.is_null()
            || (*c).flags & CLIENT_ATTACHED as uint64_t != 0
            || (*c).flags & CLIENT_CONTROL as uint64_t != 0
        {
            return 0 as ::core::ffi::c_int;
        }
        1 as ::core::ffi::c_int
    }
}
pub(crate) unsafe fn file_print(
    mut c: *mut client,
    mut fmt: *const ::core::ffi::c_char,
    args: &[FmtArg],
) {
    unsafe {
        file_vprint(c, fmt, args);
    }
}
pub(crate) unsafe fn file_vprint(
    mut c: *mut client,
    mut fmt: *const ::core::ffi::c_char,
    args: &[FmtArg],
) {
    unsafe {
        let mut msg = msg_write_open::default();
        if file_can_print(c) == 0 {
            return;
        }
        if let Some(cf) = file_find_ref(&raw mut (*c).files, 1 as ::core::ffi::c_int) {
            let cf_ptr = cf.as_ptr();
            format_buf(&mut (*cf_ptr).buffer, fmt, args);
            file_push(cf);
        } else {
            let cf =
                file_create_with_client(c, 1 as ::core::ffi::c_int, None, ClientFileData::None);
            let cf_ptr = cf.as_ptr();
            (*cf_ptr).path = Some(c"-".to_owned());
            format_buf(&mut (*cf_ptr).buffer, fmt, args);
            msg.stream = 1 as ::core::ffi::c_int;
            msg.fd = STDOUT_FILENO;
            msg.flags = 0 as ::core::ffi::c_int;
            proc_send(
                peer_ptr(&(*c).peer),
                MSG_WRITE_OPEN,
                -(1 as ::core::ffi::c_int),
                &raw mut msg as *const u8,
                ::core::mem::size_of::<msg_write_open>() as size_t,
            );
        };
    }
}
pub(crate) unsafe fn file_print_buffer(mut c: *mut client, data: &[u8]) {
    unsafe {
        let mut msg = msg_write_open::default();
        if file_can_print(c) == 0 {
            return;
        }
        if let Some(cf) = file_find_ref(&raw mut (*c).files, 1 as ::core::ffi::c_int) {
            let cf_ptr = cf.as_ptr();
            (*cf_ptr).buffer.as_mut().append(data);
            file_push(cf);
        } else {
            let cf =
                file_create_with_client(c, 1 as ::core::ffi::c_int, None, ClientFileData::None);
            let cf_ptr = cf.as_ptr();
            (*cf_ptr).path = Some(c"-".to_owned());
            (*cf_ptr).buffer.as_mut().append(data);
            msg.stream = 1 as ::core::ffi::c_int;
            msg.fd = STDOUT_FILENO;
            msg.flags = 0 as ::core::ffi::c_int;
            proc_send(
                peer_ptr(&(*c).peer),
                MSG_WRITE_OPEN,
                -(1 as ::core::ffi::c_int),
                &raw mut msg as *const u8,
                ::core::mem::size_of::<msg_write_open>() as size_t,
            );
        };
    }
}
pub(crate) unsafe fn file_error(
    mut c: *mut client,
    mut fmt: *const ::core::ffi::c_char,
    args: &[FmtArg],
) {
    unsafe {
        let mut msg = msg_write_open::default();
        if file_can_print(c) == 0 {
            return;
        }
        if let Some(cf) = file_find_ref(&raw mut (*c).files, 2 as ::core::ffi::c_int) {
            let cf_ptr = cf.as_ptr();
            format_buf(&mut (*cf_ptr).buffer, fmt, args);
            file_push(cf);
        } else {
            let cf =
                file_create_with_client(c, 2 as ::core::ffi::c_int, None, ClientFileData::None);
            let cf_ptr = cf.as_ptr();
            (*cf_ptr).path = Some(c"-".to_owned());
            format_buf(&mut (*cf_ptr).buffer, fmt, args);
            msg.stream = 2 as ::core::ffi::c_int;
            msg.fd = STDERR_FILENO;
            msg.flags = 0 as ::core::ffi::c_int;
            proc_send(
                peer_ptr(&(*c).peer),
                MSG_WRITE_OPEN,
                -(1 as ::core::ffi::c_int),
                &raw mut msg as *const u8,
                ::core::mem::size_of::<msg_write_open>() as size_t,
            );
        };
    }
}
pub(crate) unsafe fn file_write(
    mut c: *mut client,
    mut path: *const ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
    bdata: &[u8],
    mut cb: client_file_cb,
    mut cbdata: ClientFileData,
) {
    unsafe {
        let mut current_block: u64;
        let cf_ref: ClientFileRef;
        let mut msglen: size_t = 0;
        let mut fd: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
        let fresh0 = file_next_stream;
        file_next_stream += 1;
        let mut stream: u_int = fresh0 as u_int;
        if strcmp(path, c"-".as_ptr()) == 0 as ::core::ffi::c_int {
            cf_ref = file_create_with_client(c, stream as ::core::ffi::c_int, cb, cbdata);
            let cf = cf_ref.as_ptr();
            (*cf).path = Some(c"-".to_owned());
            fd = STDOUT_FILENO;
            if c.is_null()
                || (*c).flags & CLIENT_ATTACHED as uint64_t != 0
                || (*c).flags & CLIENT_CONTROL as uint64_t != 0
            {
                (*cf).error = EBADF;
                current_block = 10126500269645651453;
            } else {
                current_block = 9838574340342979941;
            }
        } else {
            cf_ref = file_create_with_client(c, stream as ::core::ffi::c_int, cb, cbdata);
            let cf = cf_ref.as_ptr();
            (*cf).path = Some(file_get_path(c, path));
            if c.is_null() || (*c).flags & CLIENT_ATTACHED as uint64_t != 0 {
                let append = flags & O_APPEND != 0;
                let opened = File::options()
                    .write(true)
                    .create(true)
                    .append(append)
                    .truncate(!append)
                    .open(OsStr::from_bytes(file_path(cf).to_bytes()));
                match opened {
                    Err(err) => {
                        (*cf).error = err.raw_os_error().unwrap_or(EIO);
                    }
                    Ok(mut file) => {
                        if file.write_all(bdata).is_err() {
                            (*cf).error = EIO;
                        }
                    }
                }
                current_block = 10126500269645651453;
            } else {
                current_block = 9838574340342979941;
            }
        }
        let cf = cf_ref.as_ptr();
        if current_block == 9838574340342979941 {
            (*cf).buffer.as_mut().append(bdata);
            let path = file_path(cf).to_bytes_with_nul();
            msglen = path
                .len()
                .wrapping_add(::core::mem::size_of::<msg_write_open>() as size_t);
            if msglen > (MAX_IMSGSIZE as usize).wrapping_sub(IMSG_HEADER_SIZE) {
                (*cf).error = E2BIG;
            } else {
                let mut msg: Vec<u8> = vec![0_u8; msglen as usize];
                ::core::ptr::write_unaligned(
                    msg.as_mut_ptr() as *mut msg_write_open,
                    msg_write_open {
                        stream: (*cf).stream,
                        fd,
                        flags,
                    },
                );
                msg[::core::mem::size_of::<msg_write_open>()..].copy_from_slice(path);
                if proc_send(
                    (*cf).peer,
                    MSG_WRITE_OPEN,
                    -(1 as ::core::ffi::c_int),
                    msg.as_ptr(),
                    msglen,
                ) != 0 as ::core::ffi::c_int
                {
                    (*cf).error = EINVAL;
                } else {
                    return;
                }
            }
        }
        file_fire_done(cf_ref);
    }
}
pub(crate) unsafe fn file_read(
    mut c: *mut client,
    mut path: *const ::core::ffi::c_char,
    mut cb: client_file_cb,
    mut cbdata: ClientFileData,
) -> Option<ClientFileRef> {
    unsafe {
        let mut current_block: u64;
        let cf_ref: ClientFileRef;
        let mut msglen: size_t = 0;
        let mut fd: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
        let fresh1 = file_next_stream;
        file_next_stream += 1;
        let mut stream: u_int = fresh1 as u_int;
        if strcmp(path, c"-".as_ptr()) == 0 as ::core::ffi::c_int {
            cf_ref = file_create_with_client(c, stream as ::core::ffi::c_int, cb, cbdata);
            let cf = cf_ref.as_ptr();
            (*cf).path = Some(c"-".to_owned());
            fd = STDIN_FILENO;
            if c.is_null()
                || (*c).flags & CLIENT_ATTACHED as uint64_t != 0
                || (*c).flags & CLIENT_CONTROL as uint64_t != 0
            {
                (*cf).error = EBADF;
                current_block = 2435717041892024324;
            } else {
                current_block = 5418638204944806599;
            }
        } else {
            cf_ref = file_create_with_client(c, stream as ::core::ffi::c_int, cb, cbdata);
            let cf = cf_ref.as_ptr();
            (*cf).path = Some(file_get_path(c, path));
            if c.is_null() || (*c).flags & CLIENT_ATTACHED as uint64_t != 0 {
                match fs::read(OsStr::from_bytes(file_path(cf).to_bytes())) {
                    Ok(contents) => {
                        (*cf).buffer.as_mut().append(&contents);
                    }
                    Err(err) => {
                        (*cf).error = err.raw_os_error().unwrap_or(EIO);
                    }
                }
                current_block = 2435717041892024324;
            } else {
                current_block = 5418638204944806599;
            }
        }
        let cf = cf_ref.as_ptr();
        if current_block == 5418638204944806599 {
            let path = file_path(cf).to_bytes_with_nul();
            msglen = path
                .len()
                .wrapping_add(::core::mem::size_of::<msg_read_open>() as size_t);
            if msglen > (MAX_IMSGSIZE as usize).wrapping_sub(IMSG_HEADER_SIZE) {
                (*cf).error = E2BIG;
            } else {
                let mut msg: Vec<u8> = vec![0_u8; msglen as usize];
                ::core::ptr::write_unaligned(
                    msg.as_mut_ptr() as *mut msg_read_open,
                    msg_read_open {
                        stream: (*cf).stream,
                        fd,
                    },
                );
                msg[::core::mem::size_of::<msg_read_open>()..].copy_from_slice(path);
                if proc_send(
                    (*cf).peer,
                    MSG_READ_OPEN,
                    -(1 as ::core::ffi::c_int),
                    msg.as_ptr(),
                    msglen,
                ) != 0 as ::core::ffi::c_int
                {
                    (*cf).error = EINVAL;
                } else {
                    return Some(cf_ref);
                }
            }
        }
        file_fire_done(cf_ref);
        None
    }
}
pub(crate) unsafe fn file_cancel(cf: ClientFileRef) {
    unsafe {
        let cf = cf.as_ptr();
        let mut msg: msg_read_cancel = msg_read_cancel { stream: 0 };
        log_debug(c"read cancel file %d".as_ptr(), fmt_args![(*cf).stream]);
        if (*cf).closed != 0 {
            return;
        }
        (*cf).closed = 1 as ::core::ffi::c_int;
        msg.stream = (*cf).stream;
        proc_send(
            (*cf).peer,
            MSG_READ_CANCEL,
            -(1 as ::core::ffi::c_int),
            &raw mut msg as *const u8,
            ::core::mem::size_of::<msg_read_cancel>() as size_t,
        );
    }
}
pub(crate) unsafe fn file_push(cf: ClientFileRef) {
    unsafe {
        const MAX_DATA: usize =
            (MAX_IMSGSIZE as usize) - IMSG_HEADER_SIZE - ::core::mem::size_of::<msg_write_data>();
        let cf_ptr = cf.as_ptr();
        let mut close_0: msg_write_close = msg_write_close { stream: 0 };
        let mut left: size_t = (*cf_ptr).buffer.as_ref().len();
        while left != 0 as size_t {
            let sent = left.min(MAX_DATA);
            let mut msg: Vec<u8> = vec![0_u8; ::core::mem::size_of::<msg_write_data>()];
            ::core::ptr::write_unaligned(
                msg.as_mut_ptr() as *mut msg_write_data,
                msg_write_data {
                    stream: (*cf_ptr).stream,
                },
            );
            msg.extend_from_slice((*cf_ptr).buffer.as_mut().pullup(sent));
            if proc_send(
                (*cf_ptr).peer,
                MSG_WRITE,
                -(1 as ::core::ffi::c_int),
                msg.as_ptr(),
                msg.len(),
            ) != 0 as ::core::ffi::c_int
            {
                break;
            }
            (*cf_ptr).buffer.as_mut().drain(sent);
            left = (*cf_ptr).buffer.as_ref().len();
            log_debug(
                c"file %d sent %zu, left %zu".as_ptr(),
                fmt_args![(*cf_ptr).stream, sent, left],
            );
        }
        if left != 0 as size_t {
            reactor::current().defer(move || {
                let cf_ptr = cf.as_ptr();
                if (*cf_ptr).client().is_null()
                    || !(*(*cf_ptr).client()).flags & CLIENT_DEAD as uint64_t != 0
                {
                    file_push(cf);
                } else {
                    file_free(cf);
                }
            });
        } else if (*cf_ptr).stream > 2 as ::core::ffi::c_int {
            close_0.stream = (*cf_ptr).stream;
            proc_send(
                (*cf_ptr).peer,
                MSG_WRITE_CLOSE,
                -(1 as ::core::ffi::c_int),
                &raw mut close_0 as *const u8,
                ::core::mem::size_of::<msg_write_close>() as size_t,
            );
            file_fire_done(cf);
        }
    }
}
pub(crate) unsafe fn file_write_left(mut files: *mut client_files_t) -> ::core::ffi::c_int {
    unsafe {
        let mut left: size_t = 0;
        let mut waiting: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        for cf in (*files).values() {
            let cf = cf.as_ptr();
            if !(*cf).event.is_none() {
                left = (*cf).event.output_len();
                if left != 0 as size_t {
                    waiting += 1;
                    log_debug(
                        c"file %u %zu bytes left".as_ptr(),
                        fmt_args![(*cf).stream, left],
                    );
                }
            }
        }
        (waiting != 0 as ::core::ffi::c_int) as ::core::ffi::c_int
    }
}
/// A stream callback that runs `body` on the file it was made for, found
/// again in the tree it belongs to so that a file already given up is not
/// reached at all.
fn on_file(
    tree: FileOwner,
    stream: ::core::ffi::c_int,
    body: unsafe fn(ClientFileRef),
) -> ::std::rc::Rc<dyn Fn(Stream)> {
    ::std::rc::Rc::new(move |_stream| {
        let held = tree.tree();
        if held.is_null() {
            return;
        }
        if let Some(cf) = unsafe { file_find_ref(held, stream) } {
            unsafe { body(cf) };
        }
    })
}

/// The same, for the callback a failed stream makes.
fn on_file_error(
    tree: FileOwner,
    stream: ::core::ffi::c_int,
    body: unsafe fn(ClientFileRef, ::core::ffi::c_short),
) -> ::std::rc::Rc<dyn Fn(Stream, ::core::ffi::c_short)> {
    ::std::rc::Rc::new(move |_stream, what| {
        let held = tree.tree();
        if held.is_null() {
            return;
        }
        if let Some(cf) = unsafe { file_find_ref(held, stream) } {
            unsafe { body(cf, what) };
        }
    })
}

unsafe fn file_write_error_callback(cf_ref: ClientFileRef) {
    unsafe {
        let cf = cf_ref.as_ptr();
        log_debug(c"write error file %d".as_ptr(), fmt_args![(*cf).stream]);
        file_release_io(&mut *cf);
        if (*cf).cb.is_some() {
            (*cf).cb.expect("non-null function pointer")(
                ::core::ptr::null_mut::<client>(),
                ::core::ptr::null::<::core::ffi::c_char>(),
                0 as ::core::ffi::c_int,
                -(1 as ::core::ffi::c_int),
                ::core::ptr::null_mut::<Buf>(),
                (*cf).data.view(),
            );
        }
    }
}
unsafe fn file_write_callback(cf_ref: ClientFileRef) {
    unsafe {
        let cf = cf_ref.as_ptr();
        log_debug(c"write check file %d".as_ptr(), fmt_args![(*cf).stream]);
        if (*cf).cb.is_some() {
            (*cf).cb.expect("non-null function pointer")(
                ::core::ptr::null_mut::<client>(),
                ::core::ptr::null::<::core::ffi::c_char>(),
                0 as ::core::ffi::c_int,
                -(1 as ::core::ffi::c_int),
                ::core::ptr::null_mut::<Buf>(),
                (*cf).data.view(),
            );
        }
        if (*cf).closed != 0 && (*cf).event.output_len() == 0 as size_t {
            file_release_io(&mut *cf);
            file_free(cf_ref);
        }
    }
}
pub(crate) unsafe fn file_write_open(
    mut files: *mut client_files_t,
    mut peer: *mut tmuxpeer,
    mut imsg: *mut imsg,
    mut allow_streams: ::core::ffi::c_int,
    mut close_received: ::core::ffi::c_int,
    mut cb: client_file_cb,
    mut cbdata: ClientFileData,
) {
    unsafe {
        let mut msg: *mut msg_write_open = (*imsg).data as *mut msg_write_open;
        let mut msglen: size_t = ((*imsg).hdr.len as size_t).wrapping_sub(IMSG_HEADER_SIZE);
        let mut path: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut reply = msg_write_ready::default();
        let flags: ::core::ffi::c_int = O_NONBLOCK | O_WRONLY | O_CREAT;
        let mut error: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if msglen < ::core::mem::size_of::<msg_write_open>() as usize {
            fatalx(c"bad MSG_WRITE_OPEN size".as_ptr(), fmt_args![]);
        }
        if msglen == ::core::mem::size_of::<msg_write_open>() as usize {
            path = c"-".as_ptr();
        } else {
            path = msg.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_char;
        }
        log_debug(
            c"open write file %d %s".as_ptr(),
            fmt_args![(*msg).stream, path],
        );
        if file_find_ref(files, (*msg).stream).is_some() {
            error = EBADF;
        } else {
            let cf_ref = file_create_with_peer(peer, files, (*msg).stream, cb, cbdata);
            let cf = cf_ref.as_ptr();
            if (*cf).closed != 0 {
                error = EBADF;
            } else {
                (*cf).fd = -(1 as ::core::ffi::c_int);
                if (*msg).fd == -(1 as ::core::ffi::c_int) {
                    (*cf).fd = open(path, (*msg).flags | flags, 0o644 as ::core::ffi::c_int);
                } else if allow_streams != 0 {
                    if (*msg).fd != STDOUT_FILENO && (*msg).fd != STDERR_FILENO {
                        *__errno_location() = EBADF;
                    } else {
                        (*cf).fd = dup((*msg).fd);
                        if close_received != 0 {
                            close((*msg).fd);
                        }
                    }
                } else {
                    *__errno_location() = EBADF;
                }
                if (*cf).fd == -(1 as ::core::ffi::c_int) {
                    error = *__errno_location();
                } else {
                    (*cf).event = Stream::new(
                        (*cf).fd,
                        None,
                        Some(on_file((*cf).tree.clone(), (*cf).stream, |cf| {
                            file_write_callback(cf)
                        })),
                        Some(on_file_error(
                            (*cf).tree.clone(),
                            (*cf).stream,
                            |cf, _what| file_write_error_callback(cf),
                        )),
                    );
                    if (*cf).event.is_none() {
                        fatalx(c"out of memory".as_ptr(), fmt_args![]);
                    }
                    (*cf).event.enable(Interest::Write);
                }
            }
        }
        reply.stream = (*msg).stream;
        reply.error = error;
        proc_send(
            peer,
            MSG_WRITE_READY,
            -(1 as ::core::ffi::c_int),
            &raw mut reply as *const u8,
            ::core::mem::size_of::<msg_write_ready>() as size_t,
        );
    }
}
pub(crate) unsafe fn file_write_data(mut files: *mut client_files_t, mut imsg: *mut imsg) {
    unsafe {
        let mut msg: *mut msg_write_data = (*imsg).data as *mut msg_write_data;
        let mut msglen: size_t = ((*imsg).hdr.len as size_t).wrapping_sub(IMSG_HEADER_SIZE);
        let mut size: size_t =
            msglen.wrapping_sub(::core::mem::size_of::<msg_write_data>() as size_t);
        if msglen < ::core::mem::size_of::<msg_write_data>() as usize {
            fatalx(c"bad MSG_WRITE size".as_ptr(), fmt_args![]);
        }
        let Some(cf_ref) = file_find_ref(files, (*msg).stream) else {
            fatalx(c"unknown stream number".as_ptr(), fmt_args![]);
        };
        let cf = cf_ref.as_ptr();
        log_debug(
            c"write %zu to file %d".as_ptr(),
            fmt_args![size, (*cf).stream],
        );
        if !(*cf).event.is_none() {
            (*cf).event.write(
                msg.offset(1 as ::core::ffi::c_int as isize) as *const u8,
                size,
            );
        }
    }
}
pub(crate) unsafe fn file_write_close(mut files: *mut client_files_t, mut imsg: *mut imsg) {
    unsafe {
        let mut msg: *mut msg_write_close = (*imsg).data as *mut msg_write_close;
        let mut msglen: size_t = ((*imsg).hdr.len as size_t).wrapping_sub(IMSG_HEADER_SIZE);
        if msglen != ::core::mem::size_of::<msg_write_close>() as usize {
            fatalx(c"bad MSG_WRITE_CLOSE size".as_ptr(), fmt_args![]);
        }
        let Some(cf_ref) = file_find_ref(files, (*msg).stream) else {
            fatalx(c"unknown stream number".as_ptr(), fmt_args![]);
        };
        let cf = cf_ref.as_ptr();
        log_debug(c"close file %d".as_ptr(), fmt_args![(*cf).stream]);
        if (*cf).event.is_none() || (*cf).event.output_len() == 0 as size_t {
            file_release_io(&mut *cf);
            file_free(cf_ref);
        }
    }
}
unsafe fn file_read_error_callback(cf_ref: ClientFileRef, mut what: ::core::ffi::c_short) {
    unsafe {
        let cf = cf_ref.as_ptr();
        let mut msg = msg_read_done::default();
        log_debug(c"read error file %d".as_ptr(), fmt_args![(*cf).stream]);
        msg.stream = (*cf).stream;
        msg.error = if what as ::core::ffi::c_int & BUFFER_ERROR != 0 {
            EIO
        } else {
            0 as ::core::ffi::c_int
        };
        proc_send(
            (*cf).peer,
            MSG_READ_DONE,
            -(1 as ::core::ffi::c_int),
            &raw mut msg as *const u8,
            ::core::mem::size_of::<msg_read_done>() as size_t,
        );
        file_release_io(&mut *cf);
        file_free(cf_ref);
    }
}
unsafe fn file_read_callback(cf_ref: ClientFileRef) {
    unsafe {
        let mut cf: *mut client_file = cf_ref.as_ptr();
        let mut msg: Vec<u8> = vec![0; ::core::mem::size_of::<msg_read_data>()];
        loop {
            let limit = (MAX_IMSGSIZE as usize)
                .wrapping_sub(IMSG_HEADER_SIZE)
                .wrapping_sub(::core::mem::size_of::<msg_read_data>() as usize);
            let data = (*cf).event.with_input(|buffer| buffer.copy_to_bytes(limit));
            let Some(data) = data else {
                break;
            };
            let bsize = data.len();
            if bsize == 0 as size_t {
                break;
            }
            log_debug(
                c"read %zu from file %d".as_ptr(),
                fmt_args![bsize, (*cf).stream],
            );
            msg.truncate(::core::mem::size_of::<msg_read_data>());
            msg.extend_from_slice(&data);
            ::core::ptr::write_unaligned(
                msg.as_mut_ptr() as *mut msg_read_data,
                msg_read_data {
                    stream: (*cf).stream,
                },
            );
            proc_send(
                (*cf).peer,
                MSG_READ,
                -(1 as ::core::ffi::c_int),
                msg.as_ptr(),
                msg.len() as size_t,
            );
        }
    }
}
pub(crate) unsafe fn file_read_open(
    mut files: *mut client_files_t,
    mut peer: *mut tmuxpeer,
    mut imsg: *mut imsg,
    mut allow_streams: ::core::ffi::c_int,
    mut close_received: ::core::ffi::c_int,
    mut cb: client_file_cb,
    mut cbdata: ClientFileData,
) {
    unsafe {
        let mut msg: *mut msg_read_open = (*imsg).data as *mut msg_read_open;
        let mut msglen: size_t = ((*imsg).hdr.len as size_t).wrapping_sub(IMSG_HEADER_SIZE);
        let mut path: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut reply = msg_read_done::default();
        let flags: ::core::ffi::c_int = O_NONBLOCK | O_RDONLY;
        let mut error: ::core::ffi::c_int = 0;
        if msglen < ::core::mem::size_of::<msg_read_open>() as usize {
            fatalx(c"bad MSG_READ_OPEN size".as_ptr(), fmt_args![]);
        }
        if msglen == ::core::mem::size_of::<msg_read_open>() as usize {
            path = c"-".as_ptr();
        } else {
            path = msg.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_char;
        }
        log_debug(
            c"open read file %d %s".as_ptr(),
            fmt_args![(*msg).stream, path],
        );
        if file_find_ref(files, (*msg).stream).is_some() {
            error = EBADF;
        } else {
            let cf_ref = file_create_with_peer(peer, files, (*msg).stream, cb, cbdata);
            let cf = cf_ref.as_ptr();
            if (*cf).closed != 0 {
                error = EBADF;
            } else {
                (*cf).fd = -(1 as ::core::ffi::c_int);
                if (*msg).fd == -(1 as ::core::ffi::c_int) {
                    (*cf).fd = open(path, flags);
                } else if allow_streams != 0 {
                    if (*msg).fd != STDIN_FILENO {
                        *__errno_location() = EBADF;
                    } else {
                        (*cf).fd = dup((*msg).fd);
                        if close_received != 0 {
                            close((*msg).fd);
                        }
                    }
                } else {
                    *__errno_location() = EBADF;
                }
                if (*cf).fd == -(1 as ::core::ffi::c_int) {
                    error = *__errno_location();
                } else {
                    (*cf).event = Stream::new(
                        (*cf).fd,
                        Some(on_file((*cf).tree.clone(), (*cf).stream, |cf| {
                            file_read_callback(cf)
                        })),
                        None,
                        Some(on_file_error(
                            (*cf).tree.clone(),
                            (*cf).stream,
                            |cf, what| file_read_error_callback(cf, what),
                        )),
                    );
                    if (*cf).event.is_none() {
                        fatalx(c"out of memory".as_ptr(), fmt_args![]);
                    }
                    (*cf).event.enable(Interest::Read);
                    return;
                }
            }
        }
        reply.stream = (*msg).stream;
        reply.error = error;
        proc_send(
            peer,
            MSG_READ_DONE,
            -(1 as ::core::ffi::c_int),
            &raw mut reply as *const u8,
            ::core::mem::size_of::<msg_read_done>() as size_t,
        );
    }
}
pub(crate) unsafe fn file_read_cancel(mut files: *mut client_files_t, mut imsg: *mut imsg) {
    unsafe {
        let mut msg: *mut msg_read_cancel = (*imsg).data as *mut msg_read_cancel;
        let mut msglen: size_t = ((*imsg).hdr.len as size_t).wrapping_sub(IMSG_HEADER_SIZE);
        if msglen != ::core::mem::size_of::<msg_read_cancel>() as usize {
            fatalx(c"bad MSG_READ_CANCEL size".as_ptr(), fmt_args![]);
        }
        let Some(cf_ref) = file_find_ref(files, (*msg).stream) else {
            fatalx(c"unknown stream number".as_ptr(), fmt_args![]);
        };
        let cf = cf_ref.as_ptr();
        log_debug(c"cancel file %d".as_ptr(), fmt_args![(*cf).stream]);
        file_read_error_callback(cf_ref, 0 as ::core::ffi::c_short);
    }
}
pub(crate) unsafe fn file_write_ready(
    mut files: *mut client_files_t,
    mut imsg: *mut imsg,
) -> ::core::ffi::c_int {
    unsafe {
        let mut msg: *mut msg_write_ready = (*imsg).data as *mut msg_write_ready;
        let mut msglen: size_t = ((*imsg).hdr.len as size_t).wrapping_sub(IMSG_HEADER_SIZE);
        if msglen != ::core::mem::size_of::<msg_write_ready>() as usize {
            return -(1 as ::core::ffi::c_int);
        }
        let Some(cf_ref) = file_find_ref(files, (*msg).stream) else {
            return 0 as ::core::ffi::c_int;
        };
        let cf = cf_ref.as_ptr();
        if (*msg).error != 0 as ::core::ffi::c_int {
            (*cf).error = (*msg).error;
            file_fire_done(cf_ref);
        } else {
            file_push(cf_ref);
        }
        0 as ::core::ffi::c_int
    }
}
pub(crate) unsafe fn file_read_data(
    mut files: *mut client_files_t,
    mut imsg: *mut imsg,
) -> ::core::ffi::c_int {
    unsafe {
        let mut msg: *mut msg_read_data = (*imsg).data as *mut msg_read_data;
        let mut msglen: size_t = ((*imsg).hdr.len as size_t).wrapping_sub(IMSG_HEADER_SIZE);
        let mut bdata: *mut u8 = msg.offset(1 as ::core::ffi::c_int as isize) as *mut u8;
        let mut bsize: size_t =
            msglen.wrapping_sub(::core::mem::size_of::<msg_read_data>() as size_t);
        if msglen < ::core::mem::size_of::<msg_read_data>() as usize {
            return -(1 as ::core::ffi::c_int);
        }
        let Some(cf_ref) = file_find_ref(files, (*msg).stream) else {
            return 0 as ::core::ffi::c_int;
        };
        let cf = cf_ref.as_ptr();
        log_debug(
            c"file %d read %zu bytes".as_ptr(),
            fmt_args![(*cf).stream, bsize],
        );
        if (*cf).error == 0 as ::core::ffi::c_int && (*cf).closed == 0 {
            (*cf)
                .buffer
                .as_mut()
                .append(::core::slice::from_raw_parts(bdata.cast::<u8>(), bsize));
            file_fire_read(&cf_ref);
        }
        0 as ::core::ffi::c_int
    }
}
pub(crate) unsafe fn file_read_done(
    mut files: *mut client_files_t,
    mut imsg: *mut imsg,
) -> ::core::ffi::c_int {
    unsafe {
        let mut msg: *mut msg_read_done = (*imsg).data as *mut msg_read_done;
        let mut msglen: size_t = ((*imsg).hdr.len as size_t).wrapping_sub(IMSG_HEADER_SIZE);
        if msglen != ::core::mem::size_of::<msg_read_done>() as usize {
            return -(1 as ::core::ffi::c_int);
        }
        let Some(cf_ref) = file_find_ref(files, (*msg).stream) else {
            return 0 as ::core::ffi::c_int;
        };
        let cf = cf_ref.as_ptr();
        log_debug(c"file %d read done".as_ptr(), fmt_args![(*cf).stream]);
        (*cf).error = (*msg).error;
        file_fire_done(cf_ref);
        0 as ::core::ffi::c_int
    }
}
