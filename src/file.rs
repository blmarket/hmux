use crate::src::cmd::queue::{cmdq_clear_wait_file, cmdq_set_wait_file};
use crate::src::compat::stdio::CFile;
use crate::src::ffi::libc::{
    __errno_location, close, dup, ferror, fopen, fread, free, fwrite, memcpy, open, strcmp, strlen,
};
use crate::src::log::{fatalx, log_debug};
use crate::src::proc::proc_send;
use crate::src::reactor::{
    bufferevent_enable, bufferevent_free, bufferevent_new, bufferevent_write, evbuffer_add,
    evbuffer_add_vprintf, evbuffer_drain, evbuffer_free, evbuffer_get_length, evbuffer_new,
    evbuffer_pullup, event_once,
};
use crate::src::server_client::{server_client_get_cwd, server_client_unref};
use crate::src::shared::abi::*;
use crate::src::shared::abi::{__off64_t, __off_t, __uint32_t, ssize_t, uint32_t};
use crate::src::shared::arguments::args;
use crate::src::shared::client::*;
use crate::src::shared::client::{
    client, client_file, client_file_cb, client_file_entry, client_files, overlay_check_cb,
    overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb, overlay_resize_cb,
};
use crate::src::shared::client::{
    CLIENT_ATTACHED, CLIENT_CONTROL, CLIENT_DEAD, CLIENT_WRITE_ACK,
};
use crate::src::shared::colour::*;
use crate::src::shared::command::{cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds};
use crate::src::shared::control::control_state;
use crate::src::shared::display::*;
use crate::src::shared::display::{visible_range, visible_ranges};
use crate::src::shared::environment::environ;
use crate::src::shared::errno::{E2BIG, EINVAL, ENOMEM};
use crate::src::shared::event::*;
use crate::src::shared::event::{EV_READ, EV_TIMEOUT, EV_WRITE};
use crate::src::shared::format::{format_job_tree, format_tree};
use crate::src::shared::grid::*;
use crate::src::shared::hyperlinks::hyperlinks;
use crate::src::shared::input::{input_ctx, input_request, input_requests};
use crate::src::shared::key::*;
use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
use crate::src::shared::layout::layout_geometry;
use crate::src::shared::layout::*;
use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
use crate::src::shared::menu::menu_data;
use crate::src::shared::message::*;
use crate::src::shared::message::{ibuf, imsg};
use crate::src::shared::message::{imsg_hdr, IMSG_HEADER_SIZE, MAX_IMSGSIZE};
use crate::src::shared::mouse::mouse_event;
use crate::src::shared::options::options;
use crate::src::shared::pane::{
    window_pane, window_pane_modes, window_pane_prompt, window_pane_tree_entry, window_panes,
};
use crate::src::shared::pane::{window_pane_offset, window_pane_resize, window_pane_resizes};
use crate::src::shared::posix_io::{
    O_APPEND, O_CREAT, O_NONBLOCK, O_WRONLY, STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO,
};
use crate::src::shared::process::tmuxpeer;
use crate::src::shared::prompt::prompt;
use crate::src::shared::redraw::redraw_scene;
use crate::src::shared::screen::{screen, screen_sel, screen_titles};
use crate::src::shared::screen_write::screen_write_cline;
use crate::src::shared::session::{session, session_entry};
use crate::src::shared::spawn::spawn_editor_state;
use crate::src::shared::status::status_line;
use crate::src::shared::stdio::{
    _IO_codecvt, _IO_lock_t, _IO_marker, _IO_wide_data, _IO_FILE, FILE,
};
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
use crate::src::shared::tree::{RB_BLACK, RB_NEGINF, RB_RED};
use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
use crate::src::shared::variadic::{__builtin_va_list, __gnuc_va_list, __va_list_tag, va_list};
use crate::src::shared::window::{
    window, window_entry, window_mode, window_mode_entry, window_winlinks, winlink, winlink_entry,
    winlink_stack, winlinks,
};
use crate::src::tmux::find_home_cstr;
use std::any::Any;
use std::ffi::{CStr, CString};

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

use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_14;
use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_13;

pub const EIO: ::core::ffi::c_int = 5 as ::core::ffi::c_int;

pub const EBADF: ::core::ffi::c_int = 9 as ::core::ffi::c_int;

pub const O_RDONLY: ::core::ffi::c_int = 0 as ::core::ffi::c_int;

pub const BEV_EVENT_ERROR: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const EVBUFFER_ERROR: ::core::ffi::c_int = BEV_EVENT_ERROR;

static mut file_next_stream: ::core::ffi::c_int = 3 as ::core::ffi::c_int;

unsafe fn file_create_owner() -> *mut client_file {
    Box::into_raw(Box::new(client_file {
        path: None,
        callback_data: None,
        wait_item: std::ptr::null_mut(),
        wait_client: std::ptr::null_mut(),
        cancel_data: None,
        terminal_scheduled: false,
        ..client_file::empty()
    }))
    .cast()
}

fn file_set_path(cf: &mut client_file, path: CString) {
    cf.path = Default::default();
    cf.path = Some(path);
}

unsafe fn file_set_cmdq_wait(
    cf: *mut client_file,
    item: *mut cmdq_item,
    cancel_data: Option<unsafe fn(*mut ::core::ffi::c_void)>,
) {
    let owner = &mut *cf;
    assert!(!item.is_null());
    assert!(owner.wait_item.is_null());
    owner.wait_item = item;
    owner.wait_client = (*item).client;
    owner.cancel_data = cancel_data;
    cmdq_set_wait_file(&mut *item, cf);
}

/// Stop a file-backed command wait without delivering its file callback.
/// The scheduled terminal event still owns and frees the file itself.
pub(crate) unsafe fn file_cancel_cmdq_wait(cf: *mut client_file) {
    let owner = &mut *cf;
    if owner.wait_item.is_null() {
        return;
    }
    cmdq_clear_wait_file(&mut *owner.wait_item, cf);
    owner.wait_item = std::ptr::null_mut();
    owner.wait_client = std::ptr::null_mut();
    (*cf).cb = None;
    let data = std::mem::replace(&mut (*cf).data, std::ptr::null_mut());
    let callback_data = owner.callback_data.take();
    let cancel_data = owner.cancel_data.take();
    drop(callback_data);
    if let Some(cancel_data) = cancel_data {
        cancel_data(data);
    }
}

unsafe fn file_get_path(c: *mut client, file: &CStr) -> CString {
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
        let cwd = CStr::from_ptr(server_client_get_cwd(c, std::ptr::null_mut())).to_bytes();
        [cwd, b"/", path.as_slice()].concat()
    };
    CString::new(full_path).expect("C string path fragments contain no NUL")
}
#[no_mangle]
pub unsafe extern "C" fn file_cmp(
    mut cf1: *mut client_file,
    mut cf2: *mut client_file,
) -> ::core::ffi::c_int {
    if (*cf1).stream < (*cf2).stream {
        return -(1 as ::core::ffi::c_int);
    }
    if (*cf1).stream > (*cf2).stream {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn file_create_with_peer(
    mut peer: *mut tmuxpeer,
    mut files: *mut client_files,
    mut stream: ::core::ffi::c_int,
    mut cb: client_file_cb,
    mut cbdata: *mut ::core::ffi::c_void,
) -> *mut client_file {
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    cf = file_create_owner();
    (*cf).c = ::core::ptr::null_mut::<client>();
    (*cf).references = 1 as ::core::ffi::c_int;
    (*cf).stream = stream;
    (*cf).buffer = evbuffer_new();
    if (*cf).buffer.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    (*cf).cb = cb;
    (*cf).data = cbdata;
    (*cf).peer = peer;
    (*cf).tree = files as *mut client_files;
    client_files_insert(files, cf);
    return cf;
}
#[no_mangle]
pub unsafe extern "C" fn file_create_with_client(
    mut c: *mut client,
    mut stream: ::core::ffi::c_int,
    mut cb: client_file_cb,
    mut cbdata: *mut ::core::ffi::c_void,
) -> *mut client_file {
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    if !c.is_null() && (*c).flags & CLIENT_ATTACHED as uint64_t != 0 {
        c = ::core::ptr::null_mut::<client>();
    }
    cf = file_create_owner();
    (*cf).c = c;
    (*cf).references = 1 as ::core::ffi::c_int;
    (*cf).stream = stream;
    (*cf).buffer = evbuffer_new();
    if (*cf).buffer.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    (*cf).cb = cb;
    (*cf).data = cbdata;
    if !(*cf).c.is_null() {
        (*cf).peer = (*(*cf).c).peer;
        (*cf).tree = &raw mut (*(*cf).c).files as *mut client_files;
        client_files_insert(&raw mut (*(*cf).c).files, cf);
        (*(*cf).c).references += 1;
    }
    return cf;
}
#[no_mangle]
pub unsafe extern "C" fn file_free(mut cf: *mut client_file) {
    (*cf).references -= 1;
    if (*cf).references != 0 as ::core::ffi::c_int {
        return;
    }
    evbuffer_free((*cf).buffer);
    if !(*cf).tree.is_null() {
        client_files_remove((*cf).tree as *mut client_files, cf);
    }
    if !(*cf).c.is_null() {
        server_client_unref((*cf).c);
    }
    (*cf).path = Default::default();
    drop(Box::from_raw(cf));
}
unsafe extern "C" fn file_fire_done_cb(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut cf: *mut client_file = arg as *mut client_file;
    let c: *mut client = (*cf).c;
    let wait_client = (*cf).wait_client;
    let dead = (!c.is_null() && (*c).flags & CLIENT_DEAD as uint64_t != 0)
        || (!wait_client.is_null() && (*wait_client).flags & CLIENT_DEAD as uint64_t != 0);
    if dead {
        file_cancel_cmdq_wait(cf);
    } else {
        let owner = &mut *cf;
        if !owner.wait_item.is_null() {
            cmdq_clear_wait_file(&mut *owner.wait_item, cf);
            owner.wait_item = std::ptr::null_mut();
            owner.wait_client = std::ptr::null_mut();
            owner.cancel_data = None;
        }
    }
    // The callback borrows this payload. Keep it alive through delivery.
    let callback_data = (*cf).callback_data.take();
    if !dead && (*cf).cb.is_some() {
        (*cf).cb.expect("non-null function pointer")(
            c,
            ((*cf).path).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            (*cf).error,
            1 as ::core::ffi::c_int,
            (*cf).buffer,
            (*cf).data,
        );
    }
    drop(callback_data);
    file_free(cf);
}
#[no_mangle]
pub unsafe extern "C" fn file_fire_done(mut cf: *mut client_file) {
    // The file stays in its stream index until this event runs. A read-done
    // message and client teardown can both request completion before then.
    // Only the first event may consume the callback data and free the owner.
    let owner = &mut *cf;
    if owner.terminal_scheduled {
        return;
    }
    owner.terminal_scheduled = true;
    event_once(
        -(1 as ::core::ffi::c_int),
        EV_TIMEOUT as ::core::ffi::c_short,
        Some(
            file_fire_done_cb
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    ::core::ffi::c_short,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        cf as *mut ::core::ffi::c_void,
        ::core::ptr::null::<timeval>(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn file_fire_read(mut cf: *mut client_file) {
    let c = (*cf).c;
    let wait_client = (*cf).wait_client;
    let dead = (!c.is_null() && (*c).flags & CLIENT_DEAD as uint64_t != 0)
        || (!wait_client.is_null() && (*wait_client).flags & CLIENT_DEAD as uint64_t != 0);
    if !dead && (*cf).cb.is_some() {
        (*cf).cb.expect("non-null function pointer")(
            (*cf).c,
            ((*cf).path).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            (*cf).error,
            0 as ::core::ffi::c_int,
            (*cf).buffer,
            (*cf).data,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn file_can_print(mut c: *mut client) -> ::core::ffi::c_int {
    if c.is_null()
        || (*c).flags & CLIENT_ATTACHED as uint64_t != 0
        || (*c).flags & CLIENT_DEAD as uint64_t != 0
        || (*c).flags & CLIENT_CONTROL as uint64_t != 0
    {
        return 0 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn file_print(
    mut c: *mut client,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut ap: ::core::ffi::VaList;
    ap = args.clone();
    file_vprint(c, fmt, ap);
}
#[no_mangle]
pub unsafe extern "C" fn file_vprint(
    mut c: *mut client,
    mut fmt: *const ::core::ffi::c_char,
    mut ap: ::core::ffi::VaList,
) {
    let mut find: client_file = client_file {
        c: ::core::ptr::null_mut::<client>(),
        peer: ::core::ptr::null_mut::<tmuxpeer>(),
        tree: ::core::ptr::null_mut::<client_files>(),
        references: 0,
        stream: 0,
        path: Default::default(),
        buffer: ::core::ptr::null_mut::<evbuffer>(),
        event: ::core::ptr::null_mut::<bufferevent>(),
        fd: 0,
        error: 0,
        closed: 0,
        cb: None,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        entry: client_file_entry {
            owner: std::ptr::null_mut(),
        },
     ..client_file::empty() };
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let mut msg: msg_write_open = msg_write_open {
        stream: 0,
        fd: 0,
        flags: 0,
    };
    if file_can_print(c) == 0 {
        return;
    }
    find.stream = 1 as ::core::ffi::c_int;
    cf = client_files_find(&raw mut (*c).files, &raw mut find);
    if cf.is_null() {
        cf = file_create_with_client(c, 1 as ::core::ffi::c_int, None, NULL);
        file_set_path(&mut *cf, CString::new("-").unwrap());
        evbuffer_add_vprintf((*cf).buffer, fmt, ap);
        msg.stream = 1 as ::core::ffi::c_int;
        msg.fd = STDOUT_FILENO;
        msg.flags = 0 as ::core::ffi::c_int;
        proc_send(
            (*c).peer,
            MSG_WRITE_OPEN,
            -(1 as ::core::ffi::c_int),
            &raw mut msg as *const ::core::ffi::c_void,
            ::core::mem::size_of::<msg_write_open>() as size_t,
        );
    } else {
        evbuffer_add_vprintf((*cf).buffer, fmt, ap);
        file_push(cf);
    };
}
#[no_mangle]
pub unsafe extern "C" fn file_print_buffer(
    mut c: *mut client,
    mut data: *mut ::core::ffi::c_void,
    mut size: size_t,
) {
    let mut find: client_file = client_file {
        c: ::core::ptr::null_mut::<client>(),
        peer: ::core::ptr::null_mut::<tmuxpeer>(),
        tree: ::core::ptr::null_mut::<client_files>(),
        references: 0,
        stream: 0,
        path: Default::default(),
        buffer: ::core::ptr::null_mut::<evbuffer>(),
        event: ::core::ptr::null_mut::<bufferevent>(),
        fd: 0,
        error: 0,
        closed: 0,
        cb: None,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        entry: client_file_entry {
            owner: std::ptr::null_mut(),
        },
     ..client_file::empty() };
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let mut msg: msg_write_open = msg_write_open {
        stream: 0,
        fd: 0,
        flags: 0,
    };
    if file_can_print(c) == 0 {
        return;
    }
    find.stream = 1 as ::core::ffi::c_int;
    cf = client_files_find(&raw mut (*c).files, &raw mut find);
    if cf.is_null() {
        cf = file_create_with_client(c, 1 as ::core::ffi::c_int, None, NULL);
        file_set_path(&mut *cf, CString::new("-").unwrap());
        evbuffer_add((*cf).buffer, data, size);
        msg.stream = 1 as ::core::ffi::c_int;
        msg.fd = STDOUT_FILENO;
        msg.flags = 0 as ::core::ffi::c_int;
        proc_send(
            (*c).peer,
            MSG_WRITE_OPEN,
            -(1 as ::core::ffi::c_int),
            &raw mut msg as *const ::core::ffi::c_void,
            ::core::mem::size_of::<msg_write_open>() as size_t,
        );
    } else {
        evbuffer_add((*cf).buffer, data, size);
        file_push(cf);
    };
}
#[no_mangle]
pub unsafe extern "C" fn file_error(
    mut c: *mut client,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut find: client_file = client_file {
        c: ::core::ptr::null_mut::<client>(),
        peer: ::core::ptr::null_mut::<tmuxpeer>(),
        tree: ::core::ptr::null_mut::<client_files>(),
        references: 0,
        stream: 0,
        path: Default::default(),
        buffer: ::core::ptr::null_mut::<evbuffer>(),
        event: ::core::ptr::null_mut::<bufferevent>(),
        fd: 0,
        error: 0,
        closed: 0,
        cb: None,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        entry: client_file_entry {
            owner: std::ptr::null_mut(),
        },
     ..client_file::empty() };
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let mut msg: msg_write_open = msg_write_open {
        stream: 0,
        fd: 0,
        flags: 0,
    };
    let mut ap: ::core::ffi::VaList;
    if file_can_print(c) == 0 {
        return;
    }
    ap = args.clone();
    find.stream = 2 as ::core::ffi::c_int;
    cf = client_files_find(&raw mut (*c).files, &raw mut find);
    if cf.is_null() {
        cf = file_create_with_client(c, 2 as ::core::ffi::c_int, None, NULL);
        file_set_path(&mut *cf, CString::new("-").unwrap());
        evbuffer_add_vprintf((*cf).buffer, fmt, ap);
        msg.stream = 2 as ::core::ffi::c_int;
        msg.fd = STDERR_FILENO;
        msg.flags = 0 as ::core::ffi::c_int;
        proc_send(
            (*c).peer,
            MSG_WRITE_OPEN,
            -(1 as ::core::ffi::c_int),
            &raw mut msg as *const ::core::ffi::c_void,
            ::core::mem::size_of::<msg_write_open>() as size_t,
        );
    } else {
        evbuffer_add_vprintf((*cf).buffer, fmt, ap);
        file_push(cf);
    };
}
#[no_mangle]
pub unsafe extern "C" fn file_write(
    c: *mut client,
    path: *const ::core::ffi::c_char,
    flags: ::core::ffi::c_int,
    bdata: *const ::core::ffi::c_void,
    bsize: size_t,
    cb: client_file_cb,
    cbdata: *mut ::core::ffi::c_void,
) {
    file_write_impl(c, path, flags, bdata, bsize, cb, cbdata, None);
}

pub(crate) unsafe fn file_write_with_cmdq_wait(
    c: *mut client,
    path: *const ::core::ffi::c_char,
    flags: ::core::ffi::c_int,
    bdata: *const ::core::ffi::c_void,
    bsize: size_t,
    cb: client_file_cb,
    cbdata: *mut ::core::ffi::c_void,
    item: *mut cmdq_item,
    cancel_data: Option<unsafe fn(*mut ::core::ffi::c_void)>,
) {
    file_write_impl(
        c,
        path,
        flags,
        bdata,
        bsize,
        cb,
        cbdata,
        Some((item, cancel_data)),
    );
}

unsafe fn file_write_impl(
    mut c: *mut client,
    mut path: *const ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
    mut bdata: *const ::core::ffi::c_void,
    mut bsize: size_t,
    mut cb: client_file_cb,
    mut cbdata: *mut ::core::ffi::c_void,
    wait: Option<(*mut cmdq_item, Option<unsafe fn(*mut ::core::ffi::c_void)>)>,
) {
    let mut current_block: u64;
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let mut msglen: size_t = 0;
    let mut fd: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    let fresh0 = file_next_stream;
    file_next_stream = file_next_stream + 1;
    let mut stream: u_int = fresh0 as u_int;
    let mut f: *mut FILE = ::core::ptr::null_mut::<FILE>();
    let mut mode: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if strcmp(path, b"-\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int {
        cf = file_create_with_client(c, stream as ::core::ffi::c_int, cb, cbdata);
        if let Some((item, cancel_data)) = wait {
            file_set_cmdq_wait(cf, item, cancel_data);
        }
        file_set_path(&mut *cf, CString::new("-").unwrap());
        fd = STDOUT_FILENO;
        if c.is_null()
            || (*c).flags & CLIENT_ATTACHED as uint64_t != 0
            || (*c).flags & CLIENT_CONTROL as uint64_t != 0
        {
            (*cf).error = EBADF;
            current_block = 4636144702248558238;
        } else {
            current_block = 8821498768635335055;
        }
    } else {
        cf = file_create_with_client(c, stream as ::core::ffi::c_int, cb, cbdata);
        if let Some((item, cancel_data)) = wait {
            file_set_cmdq_wait(cf, item, cancel_data);
        }
        file_set_path(&mut *cf, file_get_path(c, CStr::from_ptr(path)));
        if c.is_null() || (*c).flags & CLIENT_ATTACHED as uint64_t != 0 {
            if flags & O_APPEND != 0 {
                mode = b"ab\0" as *const u8 as *const ::core::ffi::c_char;
            } else {
                mode = b"wb\0" as *const u8 as *const ::core::ffi::c_char;
            }
            f = fopen(((*cf).path).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()), mode) as *mut FILE;
            if f.is_null() {
                (*cf).error = *__errno_location();
            } else {
                let file = CFile::from_raw(f).expect("fopen returned a non-null stream");
                let write_failed = fwrite(bdata, 1 as size_t, bsize, file.as_ptr()) as size_t
                    != bsize;
                drop(file);
                if write_failed {
                    (*cf).error = EIO;
                }
            }
            current_block = 4636144702248558238;
        } else {
            current_block = 8821498768635335055;
        }
    }
    match current_block {
        8821498768635335055 => {
            evbuffer_add((*cf).buffer, bdata, bsize);
            msglen = strlen(((*cf).path).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                .wrapping_add(1 as size_t)
                .wrapping_add(::core::mem::size_of::<msg_write_open>() as size_t);
            if msglen > (MAX_IMSGSIZE as usize).wrapping_sub(IMSG_HEADER_SIZE) {
                (*cf).error = E2BIG;
            } else {
                let mut msg = vec![0_u8; msglen];
                let header = msg_write_open {
                    stream: (*cf).stream,
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
                    ((*cf).path).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()) as *const ::core::ffi::c_void,
                    msglen.wrapping_sub(::core::mem::size_of::<msg_write_open>() as size_t),
                );
                if proc_send(
                    (*cf).peer,
                    MSG_WRITE_OPEN,
                    -(1 as ::core::ffi::c_int),
                    msg.as_ptr().cast(),
                    msglen,
                ) != 0 as ::core::ffi::c_int
                {
                    (*cf).error = EINVAL;
                } else {
                    return;
                }
            }
        }
        _ => {}
    }
    file_fire_done(cf);
}
#[no_mangle]
pub unsafe extern "C" fn file_read(
    mut c: *mut client,
    mut path: *const ::core::ffi::c_char,
    mut cb: client_file_cb,
    mut cbdata: *mut ::core::ffi::c_void,
) -> *mut client_file {
    file_read_impl(c, path, cb, cbdata, None, None)
}

pub(crate) unsafe fn file_read_with_cmdq_wait(
    c: *mut client,
    path: *const ::core::ffi::c_char,
    cb: client_file_cb,
    cbdata: *mut ::core::ffi::c_void,
    item: *mut cmdq_item,
    cancel_data: Option<unsafe fn(*mut ::core::ffi::c_void)>,
) -> *mut client_file {
    file_read_impl(c, path, cb, cbdata, None, Some((item, cancel_data)))
}

/// The file owns the payload until its terminal event. `client_file.data` is
/// only a borrowed pointer for the existing callback ABI, including progress.
pub(crate) unsafe fn file_read_with_owned_data<T: 'static>(
    c: *mut client,
    path: *const ::core::ffi::c_char,
    cb: client_file_cb,
    mut data: Box<T>,
) -> *mut client_file {
    let borrowed = (&mut *data as *mut T).cast();
    file_read_impl(c, path, cb, borrowed, Some(data), None)
}

pub(crate) unsafe fn file_read_with_owned_data_and_cmdq_wait<T: 'static>(
    c: *mut client,
    path: *const ::core::ffi::c_char,
    cb: client_file_cb,
    mut data: Box<T>,
    item: *mut cmdq_item,
) -> *mut client_file {
    let borrowed = (&mut *data as *mut T).cast();
    file_read_impl(c, path, cb, borrowed, Some(data), Some((item, None)))
}

unsafe fn file_read_impl(
    mut c: *mut client,
    mut path: *const ::core::ffi::c_char,
    mut cb: client_file_cb,
    mut cbdata: *mut ::core::ffi::c_void,
    callback_data: Option<Box<dyn Any>>,
    wait: Option<(*mut cmdq_item, Option<unsafe fn(*mut ::core::ffi::c_void)>)>,
) -> *mut client_file {
    let mut current_block: u64;
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let mut fd: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    let fresh1 = file_next_stream;
    file_next_stream = file_next_stream + 1;
    let mut stream: u_int = fresh1 as u_int;
    let mut f: *mut FILE = ::core::ptr::null_mut::<FILE>();
    let mut file_owner: Option<CFile> = None;
    let mut size: size_t = 0;
    let mut buffer: [::core::ffi::c_char; 8192] = [0; 8192];
    cf = file_create_with_client(c, stream as ::core::ffi::c_int, cb, cbdata);
    (*cf).callback_data = callback_data;
    if let Some((item, cancel_data)) = wait {
        file_set_cmdq_wait(cf, item, cancel_data);
    }
    if strcmp(path, b"-\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int {
        file_set_path(&mut *cf, CString::new("-").unwrap());
        fd = STDIN_FILENO;
        if c.is_null()
            || (*c).flags & CLIENT_ATTACHED as uint64_t != 0
            || (*c).flags & CLIENT_CONTROL as uint64_t != 0
        {
            (*cf).error = EBADF;
            current_block = 17369485759464587280;
        } else {
            current_block = 17710118112003399050;
        }
    } else {
        file_set_path(&mut *cf, file_get_path(c, CStr::from_ptr(path)));
        if c.is_null() || (*c).flags & CLIENT_ATTACHED as uint64_t != 0 {
            f = fopen(
                ((*cf).path).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                b"rb\0" as *const u8 as *const ::core::ffi::c_char,
            ) as *mut FILE;
            if f.is_null() {
                (*cf).error = *__errno_location();
            } else {
                file_owner = CFile::from_raw(f);
                let file = file_owner.as_ref().expect("fopen returned a non-null stream");
                loop {
                    size = fread(
                        &raw mut buffer as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                        1 as size_t,
                        ::core::mem::size_of::<[::core::ffi::c_char; 8192]>() as size_t,
                        file.as_ptr(),
                    ) as size_t;
                    if ferror(file.as_ptr()) != 0 {
                        (*cf).error = *__errno_location();
                        current_block = 17369485759464587280;
                        break;
                    } else if evbuffer_add(
                        (*cf).buffer,
                        &raw mut buffer as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
                        size,
                    ) != 0 as ::core::ffi::c_int
                    {
                        (*cf).error = ENOMEM;
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
                            (*cf).error = EIO;
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
            let path = CStr::from_ptr(((*cf).path).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut())).to_bytes_with_nul();
            let header_len = ::core::mem::size_of::<msg_read_open>();
            let msglen = header_len + path.len();
            if msglen > (MAX_IMSGSIZE as usize).wrapping_sub(IMSG_HEADER_SIZE) {
                (*cf).error = E2BIG;
            } else {
                let header = msg_read_open {
                    stream: (*cf).stream,
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
                    (*cf).peer,
                    MSG_READ_OPEN,
                    -(1 as ::core::ffi::c_int),
                    msg.as_ptr().cast(),
                    msglen,
                ) != 0 as ::core::ffi::c_int
                {
                    (*cf).error = EINVAL;
                } else {
                    return cf;
                }
            }
        }
        _ => {}
    }
    drop(file_owner);
    file_fire_done(cf);
    return ::core::ptr::null_mut::<client_file>();
}
#[no_mangle]
pub unsafe extern "C" fn file_cancel(mut cf: *mut client_file) {
    let mut msg: msg_read_cancel = msg_read_cancel { stream: 0 };
    log_debug(
        b"read cancel file %d\0" as *const u8 as *const ::core::ffi::c_char,
        (*cf).stream,
    );
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
unsafe extern "C" fn file_push_cb(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut cf: *mut client_file = arg as *mut client_file;
    if (*cf).c.is_null() || !(*(*cf).c).flags & CLIENT_DEAD as uint64_t != 0 {
        file_push(cf);
    }
    file_free(cf);
}
#[no_mangle]
pub unsafe extern "C" fn file_push(mut cf: *mut client_file) {
    let mut msg = Vec::<u8>::new();
    let header_len = ::core::mem::size_of::<msg_write_data>();
    let mut sent: size_t = 0;
    let mut left: size_t = 0;
    let mut close_0: msg_write_close = msg_write_close { stream: 0 };
    left = evbuffer_get_length(&*((*cf).buffer));
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
            stream: (*cf).stream,
        };
        memcpy(
            msg.as_mut_ptr().cast(),
            (&raw const header).cast(),
            header_len,
        );
        memcpy(
            msg.as_mut_ptr().add(header_len).cast(),
            evbuffer_pullup((*cf).buffer, sent as ssize_t) as *const ::core::ffi::c_void,
            sent,
        );
        if proc_send(
            (*cf).peer,
            MSG_WRITE,
            -(1 as ::core::ffi::c_int),
            msg.as_ptr().cast(),
            msglen,
        ) != 0 as ::core::ffi::c_int
        {
            break;
        }
        evbuffer_drain((*cf).buffer, sent);
        left = evbuffer_get_length(&*((*cf).buffer));
        log_debug(
            b"file %d sent %zu, left %zu\0" as *const u8 as *const ::core::ffi::c_char,
            (*cf).stream,
            sent,
            left,
        );
    }
    if left != 0 as size_t {
        (*cf).references += 1;
        event_once(
            -(1 as ::core::ffi::c_int),
            EV_TIMEOUT as ::core::ffi::c_short,
            Some(
                file_push_cb
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        ::core::ffi::c_short,
                        *mut ::core::ffi::c_void,
                    ) -> (),
            ),
            cf as *mut ::core::ffi::c_void,
            ::core::ptr::null::<timeval>(),
        );
    } else if (*cf).stream > 2 as ::core::ffi::c_int {
        close_0.stream = (*cf).stream;
        proc_send(
            (*cf).peer,
            MSG_WRITE_CLOSE,
            -(1 as ::core::ffi::c_int),
            &raw mut close_0 as *const ::core::ffi::c_void,
            ::core::mem::size_of::<msg_write_close>() as size_t,
        );
        if (*cf).c.is_null()
            || !(*(*cf).c).flags as ::core::ffi::c_ulonglong & CLIENT_WRITE_ACK != 0
        {
            file_fire_done(cf);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn file_write_left(mut files: *mut client_files) -> ::core::ffi::c_int {
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let mut left: size_t = 0;
    let mut waiting: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    cf = client_files_minmax(files, RB_NEGINF);
    while !cf.is_null() {
        if !(*cf).event.is_null() {
            left = evbuffer_get_length(&*((*(*cf).event).output));
            if left != 0 as size_t {
                waiting += 1;
                log_debug(
                    b"file %u %zu bytes left\0" as *const u8 as *const ::core::ffi::c_char,
                    (*cf).stream,
                    left,
                );
            }
        }
        cf = client_files_next(&*cf);
    }
    return (waiting != 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
unsafe extern "C" fn file_write_finished(mut cf: *mut client_file) {
    let mut msg: msg_write_done = msg_write_done {
        stream: 0,
        error: 0,
    };
    if !(*cf).event.is_null() {
        bufferevent_free((*cf).event);
        (*cf).event = ::core::ptr::null_mut::<bufferevent>();
    }
    if (*cf).fd != -(1 as ::core::ffi::c_int) {
        if close((*cf).fd) != 0 as ::core::ffi::c_int && (*cf).error == 0 as ::core::ffi::c_int {
            (*cf).error = *__errno_location();
        }
        (*cf).fd = -(1 as ::core::ffi::c_int);
    }
    msg.stream = (*cf).stream;
    msg.error = (*cf).error;
    proc_send(
        (*cf).peer,
        MSG_WRITE_DONE,
        -(1 as ::core::ffi::c_int),
        &raw mut msg as *const ::core::ffi::c_void,
        ::core::mem::size_of::<msg_write_done>() as size_t,
    );
    if (*cf).cb.is_some() {
        (*cf).cb.expect("non-null function pointer")(
            ::core::ptr::null_mut::<client>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
            0 as ::core::ffi::c_int,
            -(1 as ::core::ffi::c_int),
            ::core::ptr::null_mut::<evbuffer>(),
            (*cf).data,
        );
    }
    file_free(cf);
}
unsafe extern "C" fn file_write_error_callback(
    mut bev: *mut bufferevent,
    mut what: ::core::ffi::c_short,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut cf: *mut client_file = arg as *mut client_file;
    let mut error: ::core::ffi::c_int = 0;
    if what as ::core::ffi::c_int & EVBUFFER_ERROR != 0 {
        error = *__errno_location();
    } else {
        error = EIO;
    }
    if error == 0 as ::core::ffi::c_int {
        error = EIO;
    }
    log_debug(
        b"write error file %d\0" as *const u8 as *const ::core::ffi::c_char,
        (*cf).stream,
    );
    (*cf).error = error;
    bufferevent_free((*cf).event);
    (*cf).event = ::core::ptr::null_mut::<bufferevent>();
    close((*cf).fd);
    (*cf).fd = -(1 as ::core::ffi::c_int);
    if (*cf).closed != 0 {
        file_write_finished(cf);
    } else if (*cf).cb.is_some() {
        (*cf).cb.expect("non-null function pointer")(
            ::core::ptr::null_mut::<client>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
            0 as ::core::ffi::c_int,
            -(1 as ::core::ffi::c_int),
            ::core::ptr::null_mut::<evbuffer>(),
            (*cf).data,
        );
    }
}
unsafe extern "C" fn file_write_callback(
    mut bev: *mut bufferevent,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut cf: *mut client_file = arg as *mut client_file;
    log_debug(
        b"write check file %d\0" as *const u8 as *const ::core::ffi::c_char,
        (*cf).stream,
    );
    if (*cf).closed != 0 && evbuffer_get_length(&*((*(*cf).event).output)) == 0 as size_t {
        file_write_finished(cf);
    } else if (*cf).cb.is_some() {
        (*cf).cb.expect("non-null function pointer")(
            ::core::ptr::null_mut::<client>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
            0 as ::core::ffi::c_int,
            -(1 as ::core::ffi::c_int),
            ::core::ptr::null_mut::<evbuffer>(),
            (*cf).data,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn file_write_open(
    mut files: *mut client_files,
    mut peer: *mut tmuxpeer,
    mut imsg: *mut imsg,
    mut allow_streams: ::core::ffi::c_int,
    mut close_received: ::core::ffi::c_int,
    mut cb: client_file_cb,
    mut cbdata: *mut ::core::ffi::c_void,
) {
    let mut msg: *mut msg_write_open = (*imsg).data as *mut msg_write_open;
    let mut msglen: size_t = ((*imsg).hdr.len as size_t).wrapping_sub(IMSG_HEADER_SIZE);
    let mut path: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut reply: msg_write_ready = msg_write_ready {
        stream: 0,
        error: 0,
    };
    let mut find: client_file = client_file {
        c: ::core::ptr::null_mut::<client>(),
        peer: ::core::ptr::null_mut::<tmuxpeer>(),
        tree: ::core::ptr::null_mut::<client_files>(),
        references: 0,
        stream: 0,
        path: Default::default(),
        buffer: ::core::ptr::null_mut::<evbuffer>(),
        event: ::core::ptr::null_mut::<bufferevent>(),
        fd: 0,
        error: 0,
        closed: 0,
        cb: None,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        entry: client_file_entry {
            owner: std::ptr::null_mut(),
        },
     ..client_file::empty() };
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let flags: ::core::ffi::c_int = O_NONBLOCK | O_WRONLY | O_CREAT;
    let mut error: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if msglen < ::core::mem::size_of::<msg_write_open>() as usize {
        fatalx(b"bad MSG_WRITE_OPEN size\0" as *const u8 as *const ::core::ffi::c_char);
    }
    if msglen == ::core::mem::size_of::<msg_write_open>() as usize {
        path = b"-\0" as *const u8 as *const ::core::ffi::c_char;
    } else {
        path = msg.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_char;
    }
    log_debug(
        b"open write file %d %s\0" as *const u8 as *const ::core::ffi::c_char,
        (*msg).stream,
        path,
    );
    find.stream = (*msg).stream;
    if !client_files_find(files, &raw mut find).is_null() {
        error = EBADF;
    } else {
        cf = file_create_with_peer(peer, files, (*msg).stream, cb, cbdata);
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
                (*cf).event = bufferevent_new(
                    (*cf).fd,
                    None,
                    Some(
                        file_write_callback
                            as unsafe extern "C" fn(
                                *mut bufferevent,
                                *mut ::core::ffi::c_void,
                            ) -> (),
                    ),
                    Some(
                        file_write_error_callback
                            as unsafe extern "C" fn(
                                *mut bufferevent,
                                ::core::ffi::c_short,
                                *mut ::core::ffi::c_void,
                            ) -> (),
                    ),
                    cf as *mut ::core::ffi::c_void,
                );
                if (*cf).event.is_null() {
                    fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
                }
                bufferevent_enable((*cf).event, EV_WRITE as ::core::ffi::c_short);
            }
        }
    }
    reply.stream = (*msg).stream;
    reply.error = error;
    proc_send(
        peer,
        MSG_WRITE_READY,
        -(1 as ::core::ffi::c_int),
        &raw mut reply as *const ::core::ffi::c_void,
        ::core::mem::size_of::<msg_write_ready>() as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn file_write_data(mut files: *mut client_files, mut imsg: *mut imsg) {
    let mut msg: *mut msg_write_data = (*imsg).data as *mut msg_write_data;
    let mut msglen: size_t = ((*imsg).hdr.len as size_t).wrapping_sub(IMSG_HEADER_SIZE);
    let mut find: client_file = client_file {
        c: ::core::ptr::null_mut::<client>(),
        peer: ::core::ptr::null_mut::<tmuxpeer>(),
        tree: ::core::ptr::null_mut::<client_files>(),
        references: 0,
        stream: 0,
        path: Default::default(),
        buffer: ::core::ptr::null_mut::<evbuffer>(),
        event: ::core::ptr::null_mut::<bufferevent>(),
        fd: 0,
        error: 0,
        closed: 0,
        cb: None,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        entry: client_file_entry {
            owner: std::ptr::null_mut(),
        },
     ..client_file::empty() };
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let mut size: size_t = msglen.wrapping_sub(::core::mem::size_of::<msg_write_data>() as size_t);
    if msglen < ::core::mem::size_of::<msg_write_data>() as usize {
        fatalx(b"bad MSG_WRITE size\0" as *const u8 as *const ::core::ffi::c_char);
    }
    find.stream = (*msg).stream;
    cf = client_files_find(files, &raw mut find);
    if cf.is_null() {
        fatalx(b"unknown stream number\0" as *const u8 as *const ::core::ffi::c_char);
    }
    log_debug(
        b"write %zu to file %d\0" as *const u8 as *const ::core::ffi::c_char,
        size,
        (*cf).stream,
    );
    if !(*cf).event.is_null() {
        bufferevent_write(
            (*cf).event,
            msg.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
            size,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn file_write_close(mut files: *mut client_files, mut imsg: *mut imsg) {
    let mut msg: *mut msg_write_close = (*imsg).data as *mut msg_write_close;
    let mut msglen: size_t = ((*imsg).hdr.len as size_t).wrapping_sub(IMSG_HEADER_SIZE);
    let mut find: client_file = client_file {
        c: ::core::ptr::null_mut::<client>(),
        peer: ::core::ptr::null_mut::<tmuxpeer>(),
        tree: ::core::ptr::null_mut::<client_files>(),
        references: 0,
        stream: 0,
        path: Default::default(),
        buffer: ::core::ptr::null_mut::<evbuffer>(),
        event: ::core::ptr::null_mut::<bufferevent>(),
        fd: 0,
        error: 0,
        closed: 0,
        cb: None,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        entry: client_file_entry {
            owner: std::ptr::null_mut(),
        },
     ..client_file::empty() };
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    if msglen != ::core::mem::size_of::<msg_write_close>() as usize {
        fatalx(b"bad MSG_WRITE_CLOSE size\0" as *const u8 as *const ::core::ffi::c_char);
    }
    find.stream = (*msg).stream;
    cf = client_files_find(files, &raw mut find);
    if cf.is_null() {
        fatalx(b"unknown stream number\0" as *const u8 as *const ::core::ffi::c_char);
    }
    log_debug(
        b"close file %d\0" as *const u8 as *const ::core::ffi::c_char,
        (*cf).stream,
    );
    (*cf).closed = 1 as ::core::ffi::c_int;
    if (*cf).event.is_null() || evbuffer_get_length(&*((*(*cf).event).output)) == 0 as size_t {
        file_write_finished(cf);
    }
}
unsafe extern "C" fn file_read_error_callback(
    mut bev: *mut bufferevent,
    mut what: ::core::ffi::c_short,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut cf: *mut client_file = arg as *mut client_file;
    let mut msg: msg_read_done = msg_read_done {
        stream: 0,
        error: 0,
    };
    log_debug(
        b"read error file %d\0" as *const u8 as *const ::core::ffi::c_char,
        (*cf).stream,
    );
    msg.stream = (*cf).stream;
    msg.error = if what as ::core::ffi::c_int & EVBUFFER_ERROR != 0 {
        EIO
    } else {
        0 as ::core::ffi::c_int
    };
    proc_send(
        (*cf).peer,
        MSG_READ_DONE,
        -(1 as ::core::ffi::c_int),
        &raw mut msg as *const ::core::ffi::c_void,
        ::core::mem::size_of::<msg_read_done>() as size_t,
    );
    bufferevent_free((*cf).event);
    close((*cf).fd);
    client_files_remove((*cf).tree as *mut client_files, cf);
    file_free(cf);
}
unsafe extern "C" fn file_read_callback(
    mut bev: *mut bufferevent,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut cf: *mut client_file = arg as *mut client_file;
    let mut bdata: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut bsize: size_t = 0;
    let mut msg = Vec::<u8>::new();
    let header_len = ::core::mem::size_of::<msg_read_data>();
    loop {
        bsize = evbuffer_get_length(&*((*(*cf).event).input));
        if bsize == 0 as size_t {
            break;
        }
        if bsize
            > (MAX_IMSGSIZE as usize)
                .wrapping_sub(IMSG_HEADER_SIZE)
                .wrapping_sub(::core::mem::size_of::<msg_read_data>() as usize)
        {
            bsize = (MAX_IMSGSIZE as usize)
                .wrapping_sub(IMSG_HEADER_SIZE)
                .wrapping_sub(::core::mem::size_of::<msg_read_data>() as usize)
                as size_t;
        }
        bdata = evbuffer_pullup((*(*cf).event).input, bsize as ssize_t) as *mut ::core::ffi::c_void;
        log_debug(
            b"read %zu from file %d\0" as *const u8 as *const ::core::ffi::c_char,
            bsize,
            (*cf).stream,
        );
        let msglen = header_len + bsize;
        msg.resize(msglen, 0);
        let header = msg_read_data {
            stream: (*cf).stream,
        };
        memcpy(
            msg.as_mut_ptr().cast(),
            (&raw const header).cast(),
            header_len,
        );
        memcpy(msg.as_mut_ptr().add(header_len).cast(), bdata, bsize);
        proc_send(
            (*cf).peer,
            MSG_READ,
            -(1 as ::core::ffi::c_int),
            msg.as_ptr().cast(),
            msglen,
        );
        evbuffer_drain((*(*cf).event).input, bsize);
    }
}
#[no_mangle]
pub unsafe extern "C" fn file_read_open(
    mut files: *mut client_files,
    mut peer: *mut tmuxpeer,
    mut imsg: *mut imsg,
    mut allow_streams: ::core::ffi::c_int,
    mut close_received: ::core::ffi::c_int,
    mut cb: client_file_cb,
    mut cbdata: *mut ::core::ffi::c_void,
) {
    let mut msg: *mut msg_read_open = (*imsg).data as *mut msg_read_open;
    let mut msglen: size_t = ((*imsg).hdr.len as size_t).wrapping_sub(IMSG_HEADER_SIZE);
    let mut path: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut reply: msg_read_done = msg_read_done {
        stream: 0,
        error: 0,
    };
    let mut find: client_file = client_file {
        c: ::core::ptr::null_mut::<client>(),
        peer: ::core::ptr::null_mut::<tmuxpeer>(),
        tree: ::core::ptr::null_mut::<client_files>(),
        references: 0,
        stream: 0,
        path: Default::default(),
        buffer: ::core::ptr::null_mut::<evbuffer>(),
        event: ::core::ptr::null_mut::<bufferevent>(),
        fd: 0,
        error: 0,
        closed: 0,
        cb: None,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        entry: client_file_entry {
            owner: std::ptr::null_mut(),
        },
     ..client_file::empty() };
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let flags: ::core::ffi::c_int = O_NONBLOCK | O_RDONLY;
    let mut error: ::core::ffi::c_int = 0;
    if msglen < ::core::mem::size_of::<msg_read_open>() as usize {
        fatalx(b"bad MSG_READ_OPEN size\0" as *const u8 as *const ::core::ffi::c_char);
    }
    if msglen == ::core::mem::size_of::<msg_read_open>() as usize {
        path = b"-\0" as *const u8 as *const ::core::ffi::c_char;
    } else {
        path = msg.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_char;
    }
    log_debug(
        b"open read file %d %s\0" as *const u8 as *const ::core::ffi::c_char,
        (*msg).stream,
        path,
    );
    find.stream = (*msg).stream;
    if !client_files_find(files, &raw mut find).is_null() {
        error = EBADF;
    } else {
        cf = file_create_with_peer(peer, files, (*msg).stream, cb, cbdata);
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
                (*cf).event = bufferevent_new(
                    (*cf).fd,
                    Some(
                        file_read_callback
                            as unsafe extern "C" fn(
                                *mut bufferevent,
                                *mut ::core::ffi::c_void,
                            ) -> (),
                    ),
                    None,
                    Some(
                        file_read_error_callback
                            as unsafe extern "C" fn(
                                *mut bufferevent,
                                ::core::ffi::c_short,
                                *mut ::core::ffi::c_void,
                            ) -> (),
                    ),
                    cf as *mut ::core::ffi::c_void,
                );
                if (*cf).event.is_null() {
                    fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
                }
                bufferevent_enable((*cf).event, EV_READ as ::core::ffi::c_short);
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
        &raw mut reply as *const ::core::ffi::c_void,
        ::core::mem::size_of::<msg_read_done>() as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn file_read_cancel(mut files: *mut client_files, mut imsg: *mut imsg) {
    let mut msg: *mut msg_read_cancel = (*imsg).data as *mut msg_read_cancel;
    let mut msglen: size_t = ((*imsg).hdr.len as size_t).wrapping_sub(IMSG_HEADER_SIZE);
    let mut find: client_file = client_file {
        c: ::core::ptr::null_mut::<client>(),
        peer: ::core::ptr::null_mut::<tmuxpeer>(),
        tree: ::core::ptr::null_mut::<client_files>(),
        references: 0,
        stream: 0,
        path: Default::default(),
        buffer: ::core::ptr::null_mut::<evbuffer>(),
        event: ::core::ptr::null_mut::<bufferevent>(),
        fd: 0,
        error: 0,
        closed: 0,
        cb: None,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        entry: client_file_entry {
            owner: std::ptr::null_mut(),
        },
     ..client_file::empty() };
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    if msglen != ::core::mem::size_of::<msg_read_cancel>() as usize {
        fatalx(b"bad MSG_READ_CANCEL size\0" as *const u8 as *const ::core::ffi::c_char);
    }
    find.stream = (*msg).stream;
    cf = client_files_find(files, &raw mut find);
    if cf.is_null() {
        fatalx(b"unknown stream number\0" as *const u8 as *const ::core::ffi::c_char);
    }
    log_debug(
        b"cancel file %d\0" as *const u8 as *const ::core::ffi::c_char,
        (*cf).stream,
    );
    file_read_error_callback(
        ::core::ptr::null_mut::<bufferevent>(),
        0 as ::core::ffi::c_short,
        cf as *mut ::core::ffi::c_void,
    );
}
#[no_mangle]
pub unsafe extern "C" fn file_write_ready(
    mut files: *mut client_files,
    mut imsg: *mut imsg,
) -> ::core::ffi::c_int {
    let mut msg: *mut msg_write_ready = (*imsg).data as *mut msg_write_ready;
    let mut msglen: size_t = ((*imsg).hdr.len as size_t).wrapping_sub(IMSG_HEADER_SIZE);
    let mut find: client_file = client_file {
        c: ::core::ptr::null_mut::<client>(),
        peer: ::core::ptr::null_mut::<tmuxpeer>(),
        tree: ::core::ptr::null_mut::<client_files>(),
        references: 0,
        stream: 0,
        path: Default::default(),
        buffer: ::core::ptr::null_mut::<evbuffer>(),
        event: ::core::ptr::null_mut::<bufferevent>(),
        fd: 0,
        error: 0,
        closed: 0,
        cb: None,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        entry: client_file_entry {
            owner: std::ptr::null_mut(),
        },
     ..client_file::empty() };
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    if msglen != ::core::mem::size_of::<msg_write_ready>() as usize {
        return -(1 as ::core::ffi::c_int);
    }
    find.stream = (*msg).stream;
    cf = client_files_find(files, &raw mut find);
    if cf.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if (*msg).error != 0 as ::core::ffi::c_int {
        (*cf).error = (*msg).error;
        file_fire_done(cf);
    } else {
        file_push(cf);
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn file_write_done(
    mut files: *mut client_files,
    mut imsg: *mut imsg,
) -> ::core::ffi::c_int {
    let mut msg: *mut msg_write_done = (*imsg).data as *mut msg_write_done;
    let mut msglen: size_t = ((*imsg).hdr.len as size_t).wrapping_sub(IMSG_HEADER_SIZE);
    let mut find: client_file = client_file {
        c: ::core::ptr::null_mut::<client>(),
        peer: ::core::ptr::null_mut::<tmuxpeer>(),
        tree: ::core::ptr::null_mut::<client_files>(),
        references: 0,
        stream: 0,
        path: Default::default(),
        buffer: ::core::ptr::null_mut::<evbuffer>(),
        event: ::core::ptr::null_mut::<bufferevent>(),
        fd: 0,
        error: 0,
        closed: 0,
        cb: None,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        entry: client_file_entry {
            owner: std::ptr::null_mut(),
        },
     ..client_file::empty() };
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    if msglen != ::core::mem::size_of::<msg_write_done>() as usize {
        return -(1 as ::core::ffi::c_int);
    }
    find.stream = (*msg).stream;
    cf = client_files_find(files, &raw mut find);
    if cf.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if (*cf).c.is_null() || !(*(*cf).c).flags as ::core::ffi::c_ulonglong & CLIENT_WRITE_ACK != 0 {
        return 0 as ::core::ffi::c_int;
    }
    log_debug(
        b"file %d write done\0" as *const u8 as *const ::core::ffi::c_char,
        (*cf).stream,
    );
    (*cf).error = (*msg).error;
    file_fire_done(cf);
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn file_read_data(
    mut files: *mut client_files,
    mut imsg: *mut imsg,
) -> ::core::ffi::c_int {
    let mut msg: *mut msg_read_data = (*imsg).data as *mut msg_read_data;
    let mut msglen: size_t = ((*imsg).hdr.len as size_t).wrapping_sub(IMSG_HEADER_SIZE);
    let mut find: client_file = client_file {
        c: ::core::ptr::null_mut::<client>(),
        peer: ::core::ptr::null_mut::<tmuxpeer>(),
        tree: ::core::ptr::null_mut::<client_files>(),
        references: 0,
        stream: 0,
        path: Default::default(),
        buffer: ::core::ptr::null_mut::<evbuffer>(),
        event: ::core::ptr::null_mut::<bufferevent>(),
        fd: 0,
        error: 0,
        closed: 0,
        cb: None,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        entry: client_file_entry {
            owner: std::ptr::null_mut(),
        },
     ..client_file::empty() };
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let mut bdata: *mut ::core::ffi::c_void =
        msg.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void;
    let mut bsize: size_t = msglen.wrapping_sub(::core::mem::size_of::<msg_read_data>() as size_t);
    if msglen < ::core::mem::size_of::<msg_read_data>() as usize {
        return -(1 as ::core::ffi::c_int);
    }
    find.stream = (*msg).stream;
    cf = client_files_find(files, &raw mut find);
    if cf.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    log_debug(
        b"file %d read %zu bytes\0" as *const u8 as *const ::core::ffi::c_char,
        (*cf).stream,
        bsize,
    );
    if (*cf).error == 0 as ::core::ffi::c_int && (*cf).closed == 0 {
        if evbuffer_add((*cf).buffer, bdata, bsize) != 0 as ::core::ffi::c_int {
            (*cf).error = ENOMEM;
            file_fire_done(cf);
        } else {
            file_fire_read(cf);
        }
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn file_read_done(
    mut files: *mut client_files,
    mut imsg: *mut imsg,
) -> ::core::ffi::c_int {
    let mut msg: *mut msg_read_done = (*imsg).data as *mut msg_read_done;
    let mut msglen: size_t = ((*imsg).hdr.len as size_t).wrapping_sub(IMSG_HEADER_SIZE);
    let mut find: client_file = client_file {
        c: ::core::ptr::null_mut::<client>(),
        peer: ::core::ptr::null_mut::<tmuxpeer>(),
        tree: ::core::ptr::null_mut::<client_files>(),
        references: 0,
        stream: 0,
        path: Default::default(),
        buffer: ::core::ptr::null_mut::<evbuffer>(),
        event: ::core::ptr::null_mut::<bufferevent>(),
        fd: 0,
        error: 0,
        closed: 0,
        cb: None,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        entry: client_file_entry {
            owner: std::ptr::null_mut(),
        },
     ..client_file::empty() };
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    if msglen != ::core::mem::size_of::<msg_read_done>() as usize {
        return -(1 as ::core::ffi::c_int);
    }
    find.stream = (*msg).stream;
    cf = client_files_find(files, &raw mut find);
    if cf.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    log_debug(
        b"file %d read done\0" as *const u8 as *const ::core::ffi::c_char,
        (*cf).stream,
    );
    (*cf).error = (*msg).error;
    file_fire_done(cf);
    return 0 as ::core::ffi::c_int;
}

fn client_files_key(elm: &client_file) -> i32 {
    elm.stream
}
pub unsafe fn client_files_find(
    head: *mut client_files,
    elm: *mut client_file,
) -> *mut client_file {
    let Some(map) = (*head).storage.as_deref() else {
        return std::ptr::null_mut();
    };
    let key = client_files_key(&*elm);
    map.get(&key).copied().unwrap_or(std::ptr::null_mut())
}
pub unsafe fn client_files_nfind(
    head: *mut client_files,
    elm: *mut client_file,
) -> *mut client_file {
    let Some(map) = (*head).storage.as_deref() else {
        return std::ptr::null_mut();
    };
    let key = client_files_key(&*elm);
    map.range((std::ops::Bound::Included(&key), std::ops::Bound::Unbounded))
        .next()
        .map_or(std::ptr::null_mut(), |(_, node)| *node)
}
pub unsafe fn client_files_insert(
    head: *mut client_files,
    elm: *mut client_file,
) -> *mut client_file {
    let key = client_files_key(&*elm);
    let map = (*head)
        .storage
        .get_or_insert_with(|| Box::new(std::collections::BTreeMap::new()))
        .as_mut();
    match map.entry(key) {
        std::collections::btree_map::Entry::Occupied(entry) => return *entry.get(),
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(elm);
        }
    }
    (*elm).entry.owner = map as *mut _;
    std::ptr::null_mut()
}
pub unsafe fn client_files_remove(
    head: *mut client_files,
    elm: *mut client_file,
) -> *mut client_file {
    if elm.is_null() {
        return std::ptr::null_mut();
    }
    let key = client_files_key(&*elm);
    let Some(map) = (*head).storage.as_deref_mut() else {
        return std::ptr::null_mut();
    };
    if map.get(&key).copied() != Some(elm) {
        return std::ptr::null_mut();
    }
    map.remove(&key);
    (*elm).entry.owner = std::ptr::null_mut();
    if map.is_empty() {
        (*head).storage = None;
    }
    elm
}
pub unsafe fn client_files_minmax(
    head: *mut client_files,
    direction: ::core::ffi::c_int,
) -> *mut client_file {
    let Some(map) = (*head).storage.as_deref() else {
        return std::ptr::null_mut();
    };
    let pair = if direction < 0 {
        map.first_key_value()
    } else {
        map.last_key_value()
    };
    pair.map_or(std::ptr::null_mut(), |(_, node)| *node)
}
pub unsafe fn client_files_next(elm: &client_file) -> *mut client_file {
    let Some(map) = elm.entry.owner.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = client_files_key(elm);
    map.range((std::ops::Bound::Excluded(&key), std::ops::Bound::Unbounded))
        .next()
        .map_or(std::ptr::null_mut(), |(_, node)| *node)
}
pub unsafe fn client_files_prev(elm: &client_file) -> *mut client_file {
    let Some(map) = elm.entry.owner.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = client_files_key(elm);
    map.range((std::ops::Bound::Unbounded, std::ops::Bound::Excluded(&key)))
        .next_back()
        .map_or(std::ptr::null_mut(), |(_, node)| *node)
}
