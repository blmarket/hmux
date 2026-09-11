use crate::cmd::CmdqStateRef;
use crate::cmd::cmd_parse_and_append;
use crate::cmd::cmd_retval;
use crate::cmd::{CmdqItemRef, cmdq_append};
use crate::ffi::close;
use crate::fmt_args;
use crate::fmt_engine::{FmtArg, format_alloc, format_buf};
use crate::format::{format_create_defaults, format_expand};
use crate::log::{fatalx, log_debug};
use crate::pane_output::RustPaneOutputOffset;
use crate::reactor::{Interest, Timer};
use crate::server::client_ref_of;

use crate::tmux::{get_timer, setblocking};
pub use crate::types::*;
use crate::window::{RustWindowPaneWeak, window_pane_find_by_id, winlinks_in};
use ::core::ffi::CStr;
use ::std::ffi::CString;
use std::cell::RefCell;
use std::rc::Rc;
#[derive(Default)]
#[repr(C)]
pub struct control_state {
    pub panes: control_panes,
    /// The panes with output waiting, in the order it arrived. The panes
    /// themselves belong to `panes`.
    pub pending_list: control_pending_list,
    pub pending_count: u_int,
    /// Every block this client has, whether a line or pane output, oldest
    /// first. `control_free_block` removes a block from this owner.
    pub all_blocks: control_blocks,
    pub next_block_id: ControlBlockId,
    pub read_event: Stream,
    pub write_event: Stream,
    pub subs: control_subs,
    pub subs_timer: TimerHandle,
}
/// The subscriptions of one control client, by name. The order decides the
/// order `%subscription-changed` lines come out in within a tick.
pub type control_subs = std::collections::BTreeMap<CString, Rc<RefCell<control_sub>>>;

#[repr(C)]
pub struct control_sub {
    pub name: CString,
    pub format: CString,
    pub type_0: control_sub_type,
    pub id: u_int,
    pub last: Option<CString>,
    pub panes: control_sub_panes,
    pub windows: control_sub_windows,
}

/// The last value a subscription sent for a pane at a window index, which is
/// all that is kept about it.
pub type control_sub_panes = std::collections::BTreeMap<(u_int, u_int), CString>;

/// The last value a subscription sent for a window at an index.
pub type control_sub_windows = std::collections::BTreeMap<(u_int, u_int), CString>;
pub use crate::consts::{
    CLIENT_CONTROL_NOOUTPUT, CLIENT_CONTROL_PAUSEAFTER, CLIENT_CONTROLCONTROL, CLIENT_EXIT,
    CLIENT_UNATTACHEDFLAGS, CMD_RETURN_NORMAL, CMDQ_STATE_CONTROL, CONTROL_SUB_ALL_PANES,
    CONTROL_SUB_ALL_WINDOWS, CONTROL_SUB_PANE, CONTROL_SUB_SESSION, CONTROL_SUB_WINDOW, SIZE_MAX,
};

#[repr(C)]
pub struct control_block {
    pub id: ControlBlockId,
    pub size: size_t,
    pub line: Option<CString>,
    pub t: uint64_t,
}

/// Blocks in the order they were made, which is the order they go out in.
/// This list owns them; a pane's list stores their IDs.
pub type control_blocks = Vec<Box<control_block>>;

pub type ControlBlockId = u64;

/// One pane's share of the blocks its client owns in [`control_blocks`].
pub type control_pane_blocks = Vec<ControlBlockId>;

/// Panes with output waiting, in the order it arrived.
pub type control_pending_list = Vec<std::ptr::NonNull<control_pane>>;

#[repr(C)]
pub struct control_pane {
    pub pane: u_int,
    pub offset: RustPaneOutputOffset,
    pub queued: RustPaneOutputOffset,
    pub flags: core::ffi::c_int,
    pub pending_flag: core::ffi::c_int,
    /// This pane's own output blocks, which the client's `all_blocks` owns.
    pub blocks: control_pane_blocks,
}

/// The panes a control client is watching, by pane id.
pub type control_panes = std::collections::BTreeMap<u_int, Box<control_pane>>;

pub const CONTROL_PANE_OFF: core::ffi::c_int = 0x1 as core::ffi::c_int;
pub const CONTROL_PANE_PAUSED: core::ffi::c_int = 0x2 as core::ffi::c_int;
pub const CONTROL_BUFFER_LOW: core::ffi::c_int = 512 as core::ffi::c_int;
pub const CONTROL_BUFFER_HIGH: core::ffi::c_int = 8192 as core::ffi::c_int;
pub const CONTROL_WRITE_MINIMUM: core::ffi::c_int = 32 as core::ffi::c_int;
pub const CONTROL_MAXIMUM_AGE: core::ffi::c_int = 300000 as core::ffi::c_int;
pub const CONTROL_IGNORE_FLAGS: core::ffi::c_int = CLIENT_CONTROL_NOOUTPUT | CLIENT_UNATTACHEDFLAGS;
/// Takes `cb` off `blocks`, the pane list that only points at it.
fn control_unlink_block(blocks: &mut control_pane_blocks, cb: ControlBlockId) {
    if let Some(at) = blocks.iter().position(|&block| block == cb) {
        blocks.remove(at);
    }
}

fn control_free_block(cs: &mut control_state, cb: ControlBlockId) {
    if let Some(at) = cs.all_blocks.iter().position(|block| block.id == cb) {
        cs.all_blocks.remove(at);
    }
}

fn control_insert_block(cs: &mut control_state, mut block: Box<control_block>) -> ControlBlockId {
    let id = cs.next_block_id;
    cs.next_block_id = cs.next_block_id.wrapping_add(1);
    block.id = id;
    cs.all_blocks.push(block);
    id
}
fn control_get_pane<'a>(
    cs: &'a mut control_state,
    wp: &(impl crate::WindowPane + ?Sized),
) -> Option<&'a mut control_pane> {
    cs.panes.get_mut(&wp.pane_id()).map(Box::as_mut)
}

fn control_add_pane<'a>(
    cs: &'a mut control_state,
    wp: &(impl crate::WindowPane + ?Sized),
) -> &'a mut control_pane {
    cs.panes.entry(wp.pane_id()).or_insert_with(|| {
        Box::new(control_pane {
            pane: wp.pane_id(),
            offset: wp.output_position(),
            queued: wp.output_position(),
            flags: 0,
            pending_flag: 0,
            blocks: control_pane_blocks::new(),
        })
    })
}

fn control_discard_pane(cs: &mut control_state, pane: u_int) {
    let cp = cs
        .panes
        .get_mut(&pane)
        .expect("the control pane is present");
    for block in core::mem::take(&mut cp.blocks) {
        control_free_block(cs, block);
    }
}
unsafe fn control_window_pane(c: &client, pane: u_int) -> Option<RustWindowPaneWeak> {
    unsafe {
        let session = c.attached_session()?;
        let pane = window_pane_find_by_id(pane)?;
        let owner = pane.window()?;
        session
            .as_session()
            .windows
            .values()
            .any(|link| {
                link.window_handle()
                    .is_some_and(|window| window.ptr_eq(&owner))
            })
            .then_some(pane)
    }
}
pub fn control_reset_offsets(c: &mut client) {
    let cs = c
        .control_state
        .as_deref_mut()
        .expect("the client has control state");
    cs.pending_list.clear();
    cs.pending_count = 0;
    for cp in core::mem::take(&mut cs.panes).into_values() {
        for block in cp.blocks {
            control_free_block(cs, block);
        }
    }
}
/// Where a control client's reader stands in a pane's output, and whether
/// what it has already queued is enough to stop reading more.
pub fn control_pane_offset<'a>(
    c: &'a client,
    wp: &(impl crate::WindowPane + ?Sized),
) -> (Option<&'a RustPaneOutputOffset>, core::ffi::c_int) {
    if c.flags & CLIENT_CONTROL_NOOUTPUT as uint64_t != 0 {
        return (None, 0);
    }
    let cs = c
        .control_state
        .as_deref()
        .expect("the client has control state");
    let Some(cp) = cs.panes.get(&wp.pane_id()) else {
        return (None, 0);
    };
    if cp.flags & CONTROL_PANE_PAUSED != 0 {
        return (None, 0);
    }
    if cp.flags & CONTROL_PANE_OFF != 0 {
        return (None, 1);
    }
    let off = (cs.write_event.output_len() >= CONTROL_BUFFER_LOW as size_t) as core::ffi::c_int;
    (Some(&cp.offset), off)
}

/// Mutably borrows an active control offset with the same output-limit status.
pub fn control_pane_offset_mut<'a>(
    c: &'a mut client,
    wp: &(impl crate::WindowPane + ?Sized),
) -> (Option<&'a mut RustPaneOutputOffset>, core::ffi::c_int) {
    let (offset, off) = control_pane_offset(c, wp);
    if offset.is_none() {
        return (None, off);
    }
    let cp = c
        .control_state
        .as_deref_mut()
        .expect("the client has control state")
        .panes
        .get_mut(&wp.pane_id())
        .expect("the active control pane is present");
    (Some(&mut cp.offset), off)
}
pub fn control_set_pane_on(c: &mut client, wp: &(impl crate::WindowPane + ?Sized)) {
    if let Some(cp) = control_get_pane(control_state_mut(c), wp)
        && cp.flags & CONTROL_PANE_OFF != 0
    {
        cp.flags &= !CONTROL_PANE_OFF;
        cp.offset = wp.output_position();
        cp.queued = wp.output_position();
    }
}
pub fn control_set_pane_off(c: &mut client, wp: &(impl crate::WindowPane + ?Sized)) {
    let cs = control_state_mut(c);
    let cp = control_add_pane(cs, wp);
    cp.offset = wp.output_position();
    cp.queued = wp.output_position();
    cp.flags |= CONTROL_PANE_OFF;
    control_discard_pane(cs, wp.pane_id());
}
pub fn control_continue_pane(c: &mut client, wp: &(impl crate::WindowPane + ?Sized)) {
    if let Some(cp) = control_get_pane(control_state_mut(c), wp)
        && cp.flags & CONTROL_PANE_PAUSED != 0
    {
        cp.flags &= !CONTROL_PANE_PAUSED;
        cp.offset = wp.output_position();
        cp.queued = wp.output_position();
        control_write(c, c"%%continue %%%u", fmt_args![wp.pane_id()]);
    }
}
pub fn control_pause_pane(c: &mut client, wp: &(impl crate::WindowPane + ?Sized)) {
    let cs = control_state_mut(c);
    let cp = control_add_pane(cs, wp);
    if cp.flags & CONTROL_PANE_PAUSED == 0 {
        cp.flags |= CONTROL_PANE_PAUSED;
        control_discard_pane(cs, wp.pane_id());
        control_write(c, c"%%pause %%%u", fmt_args![wp.pane_id()]);
    }
}
fn control_vwrite(c: &mut client, fmt: &CStr, args: &[FmtArg]) {
    {
        let cs = c
            .control_state
            .as_deref_mut()
            .expect("the client has control state");
        let s = format_alloc(fmt, args);
        log_debug(
            c"%s: %s: writing line: %s",
            fmt_args![c"control_vwrite", c.name.as_deref(), s.as_c_str()],
        );
        cs.write_event.write(s.as_bytes());
        cs.write_event.write(b"\n");
        cs.write_event.enable(Interest::Write);
    }
}
pub fn control_write(c: &mut client, fmt: &CStr, args: &[FmtArg]) {
    {
        let cs = c
            .control_state
            .as_deref_mut()
            .expect("the client has control state");
        if cs.all_blocks.is_empty() {
            control_vwrite(c, fmt, args);
            return;
        }
        let cb = Box::new(control_block {
            id: 0,
            size: 0,
            line: Some(format_alloc(fmt, args)),
            t: get_timer(),
        });
        log_debug(
            c"%s: %s: storing line: %s",
            fmt_args![c"control_write", c.name.as_deref(), cb.line.as_deref()],
        );
        control_insert_block(cs, cb);
        cs.write_event.enable(Interest::Write);
    }
}
fn control_check_age(c: &mut client, wp: &(impl crate::WindowPane + ?Sized)) -> core::ffi::c_int {
    {
        let cs = control_state_mut(c);
        let cp = cs
            .panes
            .get(&wp.pane_id())
            .expect("the control pane is present");
        let Some(&cb) = cp.blocks.first() else {
            return 0 as core::ffi::c_int;
        };
        let cb = cs
            .all_blocks
            .iter()
            .find(|block| block.id == cb)
            .expect("control pane block missing");
        let t = get_timer();
        if cb.t >= t {
            return 0 as core::ffi::c_int;
        }
        let age = t.wrapping_sub(cb.t);
        log_debug(
            c"%s: %s: %%%u is %llu behind",
            fmt_args![
                c"control_check_age",
                c.name.as_deref(),
                wp.pane_id(),
                age as core::ffi::c_ulonglong
            ],
        );
        if c.flags as core::ffi::c_ulonglong & CLIENT_CONTROL_PAUSEAFTER != 0 {
            if age < c.pause_age as uint64_t {
                return 0 as core::ffi::c_int;
            }
            let cs = control_state_mut(c);
            cs.panes
                .get_mut(&wp.pane_id())
                .expect("the control pane is present")
                .flags |= CONTROL_PANE_PAUSED;
            control_discard_pane(cs, wp.pane_id());
            control_write(c, c"%%pause %%%u", fmt_args![wp.pane_id()]);
        } else {
            if age < CONTROL_MAXIMUM_AGE as uint64_t {
                return 0 as core::ffi::c_int;
            }
            c.exit_message = Some(c"too far behind".to_owned());
            c.flags |= CLIENT_EXIT as uint64_t;
            control_discard(&mut *c);
        }
        1 as core::ffi::c_int
    }
}
pub unsafe fn control_write_output(c: &mut client, wp: &(impl crate::WindowPane + ?Sized)) {
    unsafe {
        let Some(session) = c.attached_session() else {
            return;
        };
        let Some(window) = wp.window_context() else {
            return;
        };
        if !session.as_session().windows.values().any(|link| {
            link.window_handle()
                .is_some_and(|held| held.ptr_eq(&window))
        }) {
            return;
        }
        let cs = c
            .control_state
            .as_deref_mut()
            .expect("the client has control state");
        let cp = if c.flags & CONTROL_IGNORE_FLAGS as uint64_t != 0 {
            let Some(cp) = control_get_pane(cs, wp) else {
                return;
            };
            cp
        } else {
            control_add_pane(cs, wp)
        };
        if cp.flags & (CONTROL_PANE_OFF | CONTROL_PANE_PAUSED) != 0 {
            log_debug(
                c"%s: %s: ignoring pane %%%u",
                fmt_args![c"control_write_output", c.name.as_deref(), wp.pane_id()],
            );
            wp.advance_output(&mut cp.offset, SIZE_MAX as size_t);
            wp.advance_output(&mut cp.queued, SIZE_MAX as size_t);
            return;
        }
        if control_check_age(c, wp) != 0 {
            return;
        }
        let cs = c
            .control_state
            .as_deref_mut()
            .expect("the client has control state");
        let cp = cs
            .panes
            .get_mut(&wp.pane_id())
            .expect("the control pane is present");
        let new_size = wp.unread_output_len(&cp.queued);
        if new_size == 0 {
            return;
        }
        wp.advance_output(&mut cp.queued, new_size);
        let block = control_insert_block(
            cs,
            Box::new(control_block {
                id: 0,
                size: new_size,
                line: None,
                t: get_timer(),
            }),
        );
        let cp = cs
            .panes
            .get_mut(&wp.pane_id())
            .expect("the control pane is present");
        cp.blocks.push(block);
        log_debug(
            c"%s: %s: new output block of %zu for %%%u",
            fmt_args![
                c"control_write_output",
                c.name.as_deref(),
                new_size,
                wp.pane_id()
            ],
        );
        if cp.pending_flag == 0 {
            log_debug(
                c"%s: %s: %%%u now pending",
                fmt_args![c"control_write_output", c.name.as_deref(), wp.pane_id()],
            );
            cs.pending_list.push(std::ptr::NonNull::from(cp.as_mut()));
            cp.pending_flag = 1;
            cs.pending_count = cs.pending_count.wrapping_add(1);
        }
        cs.write_event.enable(Interest::Write);
    }
}
fn control_error(item: &CmdqItemRef, error: CString) -> cmd_retval {
    unsafe {
        let item = item.read();
        let mut client = item.client().expect("control error without a client");
        let c = client.as_client_mut();
        item.guard(c"begin", 1 as core::ffi::c_int);
        control_write(&mut *c, c"parse error: %s", fmt_args![error.as_c_str()]);
        item.guard(c"error", 1 as core::ffi::c_int);
        CMD_RETURN_NORMAL
    }
}
/// A stream callback that runs `body` on the client it was made for, and
/// does nothing at all once that client has gone.
fn on_client(
    watching: &Option<ClientWeak>,
    body: impl Fn(&mut client) + 'static,
) -> std::rc::Rc<dyn Fn(Stream)> {
    let watching = watching.clone();
    std::rc::Rc::new(move |_stream| {
        if let Some(mut c) = watching.as_ref().and_then(ClientWeak::upgrade) {
            unsafe { body(c.as_client_mut()) };
        }
    })
}

/// The same, for the callback a failed stream makes.
fn on_client_error(
    watching: &Option<ClientWeak>,
    body: impl Fn(&mut client) + 'static,
) -> std::rc::Rc<dyn Fn(Stream, core::ffi::c_short)> {
    let watching = watching.clone();
    std::rc::Rc::new(move |_stream, _what| {
        if let Some(mut c) = watching.as_ref().and_then(ClientWeak::upgrade) {
            unsafe { body(c.as_client_mut()) };
        }
    })
}

fn control_error_callback(c: &mut client) {
    c.flags |= CLIENT_EXIT as uint64_t;
}
unsafe fn control_read_callback(c: &mut client) {
    unsafe {
        let read_event = c
            .control_state
            .as_deref()
            .expect("the client has control state")
            .read_event;
        let mut error = None;
        loop {
            let Some(line) = read_event.with_input(|buffer| buffer.read_line()).flatten() else {
                break;
            };
            let mut line_data = line.to_vec();
            line_data.push(0);
            let line =
                CStr::from_bytes_until_nul(&line_data).expect("the control line is terminated");
            log_debug(
                c"%s: %s: %s",
                fmt_args![c"control_read_callback", c.name.as_deref(), line],
            );
            if line.to_bytes().is_empty() {
                c.flags |= CLIENT_EXIT as uint64_t;
                break;
            } else {
                let state = CmdqStateRef::create(None, None, CMDQ_STATE_CONTROL);
                cmd_parse_and_append(
                    line,
                    None,
                    crate::server::client_ref_of(&*c).as_ref(),
                    &state,
                    &mut error,
                );
                if let Some(error) = error.take() {
                    cmdq_append(
                        crate::server::client_ref_of(c).as_ref(),
                        CmdqItemRef::callback_items(c"control_error", move |item| {
                            control_error(item, error)
                        }),
                    );
                }
            }
        }
    }
}
pub fn control_all_done(c: &client) -> core::ffi::c_int {
    let cs = c
        .control_state
        .as_deref()
        .expect("the client has control state");
    (cs.all_blocks.is_empty() && cs.write_event.output_len() == 0) as core::ffi::c_int
}
fn control_flush_all_blocks(c: &mut client) {
    {
        let cs = c
            .control_state
            .as_deref_mut()
            .expect("the client has control state");
        while let Some(cb) = cs.all_blocks.first() {
            if cb.size != 0 as size_t {
                break;
            }
            let line = cb.line.as_deref().expect("a line block contains a line");
            log_debug(
                c"%s: %s: flushing line: %s",
                fmt_args![c"control_flush_all_blocks", c.name.as_deref(), line],
            );
            cs.write_event.write(line.to_bytes());
            cs.write_event.write(b"\n");
            cs.all_blocks.remove(0);
        }
    }
}
fn control_append_data(
    client_flags: uint64_t,
    cp: &mut control_pane,
    age: uint64_t,
    message: Option<Box<ByteBuffer>>,
    wp: &(impl crate::WindowPane + ?Sized),
    size: size_t,
) -> Option<Box<ByteBuffer>> {
    {
        let mut message = match message {
            Some(message) => message,
            None => {
                let mut message = Box::new(ByteBuffer::new());
                if client_flags as core::ffi::c_ulonglong & CLIENT_CONTROL_PAUSEAFTER != 0 {
                    format_buf(
                        &mut message,
                        c"%%extended-output %%%u %llu : ",
                        fmt_args![wp.pane_id(), age as core::ffi::c_ulonglong],
                    );
                } else {
                    format_buf(&mut message, c"%%output %%%u ", fmt_args![wp.pane_id()]);
                }
                message
            }
        };
        let mut new_data = wp.unread_output(&cp.offset);
        let new_size = new_data.len();
        if new_size < size {
            fatalx(c"not enough data: %zu < %zu", fmt_args![new_size, size]);
        }
        let mut remaining = size;
        while remaining != 0 {
            let chunk = bytes::Buf::chunk(&new_data);
            let take = remaining.min(chunk.len());
            let mut i = 0;
            while i < take {
                if chunk[i] < b' ' || chunk[i] == b'\\' {
                    format_buf(
                        &mut message,
                        c"\\%03o",
                        fmt_args![chunk[i] as core::ffi::c_int],
                    );
                    i += 1;
                    continue;
                }
                let start = i;
                i += 1;
                while i < take && chunk[i] >= b' ' && chunk[i] != b'\\' {
                    i += 1;
                }
                message.append(&chunk[start..i]);
            }
            new_data.advance(take);
            remaining -= take;
        }
        wp.advance_output(&mut cp.offset, size);
        Some(message)
    }
}
fn control_write_data(c: &mut client, mut message: Box<ByteBuffer>) {
    {
        let cs = c
            .control_state
            .as_deref_mut()
            .expect("the client has control state");
        let data = message.as_slice();
        log_debug(
            c"%s: %s: %.*s",
            fmt_args![
                c"control_write_data",
                c.name.as_deref(),
                data.len() as core::ffi::c_int,
                data
            ],
        );
        message.append(b"\n");
        cs.write_event.write_buffer(&mut message);
        drop(message);
    }
}
unsafe fn control_write_pending(
    c: &mut client,
    mut pending: std::ptr::NonNull<control_pane>,
    limit: size_t,
) -> core::ffi::c_int {
    unsafe {
        let pane = pending.as_ref().pane;
        let mut message: Option<Box<ByteBuffer>> = None;
        let mut used: size_t = 0 as size_t;
        let mut size: size_t;
        let mut age: uint64_t;
        let t: uint64_t = get_timer();
        let held = control_window_pane(c, pane);
        let Some(wp) = held
            .as_ref()
            .and_then(|pane| pane.get())
            .filter(|pane| pane.process_active())
        else {
            control_discard_pane(control_state_mut(c), pane);
            control_flush_all_blocks(c);
            return 0 as core::ffi::c_int;
        };
        while used != limit && !pending.as_ref().blocks.is_empty() {
            if control_check_age(c, wp) != 0 {
                message = None;
                break;
            } else {
                let cs = c
                    .control_state
                    .as_deref_mut()
                    .expect("the client has control state");
                let cp = pending.as_mut();
                let cb_id = cp.blocks[0];
                let cb = cs
                    .all_blocks
                    .iter_mut()
                    .find(|block| block.id == cb_id)
                    .expect("control pane block missing");
                if cb.t < t {
                    age = t.wrapping_sub(cb.t);
                } else {
                    age = 0 as uint64_t;
                }
                log_debug(
                    c"%s: %s: output block %zu (age %llu) for %%%u (used %zu/%zu)",
                    fmt_args![
                        c"control_write_pending",
                        c.name.as_deref(),
                        cb.size,
                        age as core::ffi::c_ulonglong,
                        cp.pane,
                        used,
                        limit
                    ],
                );
                size = cb.size;
                if size > limit.wrapping_sub(used) {
                    size = limit.wrapping_sub(used);
                }
                used = used.wrapping_add(size);
                message = control_append_data(c.flags, cp, age, message, wp, size);
                cb.size = cb.size.wrapping_sub(size);
                if cb.size == 0 as size_t {
                    control_unlink_block(&mut cp.blocks, cb_id);
                    control_free_block(&mut *cs, cb_id);
                    if cs
                        .all_blocks
                        .first()
                        .is_some_and(|block| block.size == 0 as size_t)
                    {
                        if let Some(message) = message.take() {
                            control_write_data(c, message);
                        }
                        control_flush_all_blocks(c);
                    }
                }
            }
        }
        if let Some(message) = message {
            control_write_data(c, message);
        }
        !pending.as_ref().blocks.is_empty() as core::ffi::c_int
    }
}
unsafe fn control_write_callback(c: &mut client) {
    unsafe {
        let mut space: size_t;
        let mut limit: size_t;
        control_flush_all_blocks(c);
        loop {
            let cs = c
                .control_state
                .as_deref_mut()
                .expect("the client has control state");
            if cs.write_event.output_len() >= CONTROL_BUFFER_HIGH as size_t {
                break;
            }
            if cs.pending_count == 0 as u_int {
                break;
            }
            space = (CONTROL_BUFFER_HIGH as size_t).wrapping_sub(cs.write_event.output_len());
            log_debug(
                c"%s: %s: %zu bytes available, %u panes",
                fmt_args![
                    c"control_write_callback",
                    c.name.as_deref(),
                    space,
                    cs.pending_count
                ],
            );
            limit = space
                .wrapping_div(cs.pending_count as size_t)
                .wrapping_div(3 as size_t);
            if limit < CONTROL_WRITE_MINIMUM as size_t {
                limit = CONTROL_WRITE_MINIMUM as size_t;
            }
            let pending_panes = cs.pending_list.clone();
            for mut pending in pending_panes {
                if control_state_mut(c).write_event.output_len() >= CONTROL_BUFFER_HIGH as size_t {
                    break;
                }
                let still_pending = control_write_pending(c, pending, limit) != 0;
                if !still_pending {
                    let cs = control_state_mut(c);
                    if let Some(at) = cs
                        .pending_list
                        .iter()
                        .position(|&waiting| waiting == pending)
                    {
                        cs.pending_list.remove(at);
                    }
                    pending.as_mut().pending_flag = 0;
                    cs.pending_count = cs.pending_count.wrapping_sub(1);
                }
            }
        }
        let cs = control_state_mut(c);
        if cs.write_event.output_len() == 0 as size_t {
            cs.write_event.disable(Interest::Write);
        }
    }
}
/// Mutably borrows the state of a control client.
fn control_state_mut(c: &mut client) -> &mut control_state {
    c.control_state
        .as_deref_mut()
        .expect("the client has control state")
}
pub unsafe fn control_start(c: &mut client) {
    unsafe {
        if c.flags & CLIENT_CONTROLCONTROL as uint64_t != 0 {
            close(c.out_fd);
            c.out_fd = -(1 as core::ffi::c_int);
        } else {
            setblocking(c.out_fd, 0 as core::ffi::c_int);
        }
        setblocking(c.fd, 0 as core::ffi::c_int);
        let watching = client_ref_of(c).map(|held| held.downgrade());
        let cs = c.control_state.insert(Box::new(control_state::default()));
        cs.read_event = Stream::new(
            c.fd,
            Some(on_client(&watching, |c| control_read_callback(c))),
            Some(on_client(&watching, |c| control_write_callback(c))),
            Some(on_client_error(&watching, |c| {
                control_error_callback(&mut *c)
            })),
        );
        if cs.read_event.is_none() {
            fatalx(c"out of memory", fmt_args![]);
        }
        if c.flags & CLIENT_CONTROLCONTROL as uint64_t != 0 {
            cs.write_event = cs.read_event;
        } else {
            cs.write_event = Stream::new(
                c.out_fd,
                None,
                Some(on_client(&watching, |c| control_write_callback(c))),
                Some(on_client_error(&watching, |c| {
                    control_error_callback(&mut *c)
                })),
            );
            if cs.write_event.is_none() {
                fatalx(c"out of memory", fmt_args![]);
            }
        }
        cs.write_event
            .set_write_watermark(CONTROL_BUFFER_LOW as size_t, 0 as size_t);
        if c.flags & CLIENT_CONTROLCONTROL as uint64_t != 0 {
            cs.write_event.write(b"\x1BP1000p");
            cs.write_event.enable(Interest::Write);
        }
    }
}
pub fn control_ready(c: &client) {
    let cs = c
        .control_state
        .as_deref()
        .expect("the client has control state");
    cs.read_event.enable(Interest::Read);
}
pub fn control_discard(c: &mut client) {
    let cs = c
        .control_state
        .as_deref_mut()
        .expect("the client has control state");
    let blocks: Vec<_> = cs
        .panes
        .values_mut()
        .flat_map(|cp| core::mem::take(&mut cp.blocks))
        .collect();
    for block in blocks {
        control_free_block(cs, block);
    }
    cs.read_event.disable(Interest::Read);
}
pub fn control_stop(c: &mut client) {
    {
        let Some(cs) = c.control_state.as_deref_mut() else {
            return;
        };
        if !c.flags & CLIENT_CONTROLCONTROL as uint64_t != 0 {
            cs.write_event.free();
        }
        cs.read_event.free();
        cs.subs.clear();
        cs.subs_timer.disarm();
        control_reset_offsets(c);
        c.control_state = None;
    }
}
unsafe fn control_check_subs_session(c: &mut client, csub: &mut control_sub, ft: &mut format_tree) {
    unsafe {
        let Some(session) = c.attached_session() else {
            return;
        };
        let session_id = session.id();
        let value = format_expand(ft, &csub.format);
        if csub.last.as_deref() == Some(value.as_c_str()) {
            return;
        }
        control_write(
            c,
            c"%%subscription-changed %s $%u - - - : %s",
            fmt_args![csub.name.as_c_str(), session_id, value.as_c_str()],
        );
        csub.last = Some(value);
    }
}
unsafe fn control_check_subs_pane(c: &mut client, csub: &mut control_sub) {
    unsafe {
        let Some(session) = c.attached_session() else {
            return;
        };
        let Some(mut pane) = window_pane_find_by_id(csub.id) else {
            return;
        };
        if pane.get().is_none_or(|pane| !pane.process_active()) {
            return;
        }
        let Some(window) = pane.window() else { return };
        let session_id = session.id();
        let window_id = window.window_id();
        let pane_id = pane.id();
        for held in window.winlinks() {
            if !held.session().ptr_eq(&session) {
                continue;
            }
            let Some(wl) = held.get() else { continue };
            let Some(wp) = pane.get_mut() else { break };
            let index = held.index();
            let mut ft = format_create_defaults(
                None,
                Some(c),
                Some(session.as_session()),
                Some(wl),
                Some(wp),
            );
            let value = format_expand(&mut ft, &csub.format);
            let key = (pane_id, index as u_int);
            if csub.panes.get(&key).map(CString::as_c_str) != Some(value.as_c_str()) {
                control_write(
                    c,
                    c"%%subscription-changed %s $%u @%u %u %%%u : %s",
                    fmt_args![
                        csub.name.as_c_str(),
                        session_id,
                        window_id,
                        index,
                        pane_id,
                        value.as_c_str()
                    ],
                );
                csub.panes.insert(key, value);
            }
        }
    }
}
unsafe fn control_check_subs_all_panes_one(
    c: &mut client,
    csub: &mut control_sub,
    ft: &mut format_tree,
    wl: &winlink,
    wp: &(impl crate::WindowPane + ?Sized),
) {
    unsafe {
        let Some(session) = c.attached_session() else {
            return;
        };
        let session_id = session.id();
        let window_id = wl
            .window_handle()
            .expect("the link has a window")
            .window_id();
        let index = wl.idx;
        let pane_id = wp.pane_id();
        let value = format_expand(ft, &csub.format);
        let key = (pane_id, index as u_int);
        let last = csub.panes.get(&key);
        if last.map(CString::as_c_str) == Some(value.as_c_str()) {
            return;
        }
        control_write(
            c,
            c"%%subscription-changed %s $%u @%u %u %%%u : %s",
            fmt_args![
                csub.name.as_c_str(),
                session_id,
                window_id,
                index,
                pane_id,
                value.as_c_str()
            ],
        );
        csub.panes.insert(key, value);
    }
}
unsafe fn control_check_subs_window(c: &mut client, csub: &mut control_sub) {
    unsafe {
        let Some(session) = c.attached_session() else {
            return;
        };
        let Some(window) = WindowRef::find_by_id(csub.id) else {
            return;
        };
        let session_id = session.id();
        let window_id = window.window_id();
        for held in window.winlinks() {
            if !held.session().ptr_eq(&session) {
                continue;
            }
            let Some(wl) = held.get() else { continue };
            let index = held.index();
            let mut ft = format_create_defaults(
                None,
                Some(c),
                Some(session.as_session()),
                Some(wl),
                None::<&dyn crate::WindowPane>,
            );
            let value = format_expand(&mut ft, &csub.format);
            let key = (window_id, index as u_int);
            if csub.windows.get(&key).map(CString::as_c_str) != Some(value.as_c_str()) {
                control_write(
                    c,
                    c"%%subscription-changed %s $%u @%u %u - : %s",
                    fmt_args![
                        csub.name.as_c_str(),
                        session_id,
                        window_id,
                        index,
                        value.as_c_str()
                    ],
                );
                csub.windows.insert(key, value);
            }
        }
    }
}
unsafe fn control_check_subs_all_windows_one(
    c: &mut client,
    csub: &mut control_sub,
    ft: &mut format_tree,
    wl: &winlink,
) {
    unsafe {
        let Some(session) = c.attached_session() else {
            return;
        };
        let session_id = session.id();
        let window_id = wl
            .window_handle()
            .expect("the link has a window")
            .window_id();
        let index = wl.idx;
        let value = format_expand(ft, &csub.format);
        let key = (window_id, index as u_int);
        let last = csub.windows.get(&key);
        if last.map(CString::as_c_str) == Some(value.as_c_str()) {
            return;
        }
        control_write(
            c,
            c"%%subscription-changed %s $%u @%u %u - : %s",
            fmt_args![
                csub.name.as_c_str(),
                session_id,
                window_id,
                index,
                value.as_c_str()
            ],
        );
        csub.windows.insert(key, value);
    }
}
fn control_subs_snapshot(c: &client) -> Vec<Rc<RefCell<control_sub>>> {
    c.control_state
        .as_deref()
        .map(|state| state.subs.values().cloned().collect())
        .unwrap_or_default()
}

unsafe fn control_check_subs_timer(c: &mut client) {
    unsafe {
        let session = c.attached_session();
        let cs = c
            .control_state
            .as_deref_mut()
            .expect("the client has control state");
        let tv = timeval::from_secs(1 as __time_t);
        let mut have_session: core::ffi::c_int = 0 as core::ffi::c_int;
        let mut have_all_panes: core::ffi::c_int = 0 as core::ffi::c_int;
        let mut have_all_windows: core::ffi::c_int = 0 as core::ffi::c_int;
        log_debug(c"%s: timer fired", fmt_args![c"control_check_subs_timer"]);
        cs.subs_timer.arm(tv);
        let Some(session) = session else { return };
        for csub in cs.subs.values() {
            let csub = csub.borrow();
            match csub.type_0 {
                CONTROL_SUB_SESSION => {
                    have_session = 1 as core::ffi::c_int;
                }
                CONTROL_SUB_ALL_PANES => {
                    have_all_panes = 1 as core::ffi::c_int;
                }
                CONTROL_SUB_ALL_WINDOWS => {
                    have_all_windows = 1 as core::ffi::c_int;
                }
                _ => {}
            }
        }
        if have_session != 0 {
            let mut ft = format_create_defaults(
                None,
                Some(c),
                Some(session.as_session()),
                None,
                None::<&dyn crate::WindowPane>,
            );
            for csub in control_subs_snapshot(c) {
                let mut csub = csub.borrow_mut();
                if csub.type_0 as core::ffi::c_uint
                    == CONTROL_SUB_SESSION as core::ffi::c_int as core::ffi::c_uint
                {
                    control_check_subs_session(c, &mut csub, &mut ft);
                }
            }
        }
        for csub in control_subs_snapshot(c) {
            let mut csub = csub.borrow_mut();
            match csub.type_0 {
                CONTROL_SUB_PANE => {
                    control_check_subs_pane(c, &mut csub);
                }
                CONTROL_SUB_WINDOW => {
                    control_check_subs_window(c, &mut csub);
                }
                _ => {}
            }
        }
        if have_all_panes != 0 {
            for link in winlinks_in(&session) {
                let Some(window) = link.get().and_then(winlink::window_handle).cloned() else {
                    continue;
                };
                let pane_ids: Vec<_> = window
                    .as_window()
                    .panes
                    .iter()
                    .map(|pane| pane.pane_id())
                    .collect();
                for pane_id in pane_ids {
                    let Some(mut pane) = window.pane_by_id(pane_id) else {
                        continue;
                    };
                    let Some(wl) = link.get() else { break };
                    let Some(wp) = pane.get_mut() else { continue };
                    let mut ft = format_create_defaults(
                        None,
                        Some(c),
                        Some(session.as_session()),
                        Some(wl),
                        Some(wp),
                    );
                    for csub in control_subs_snapshot(c) {
                        let mut csub = csub.borrow_mut();
                        if csub.type_0 == CONTROL_SUB_ALL_PANES {
                            let (Some(wl), Some(wp)) = (link.get(), pane.get()) else {
                                break;
                            };
                            control_check_subs_all_panes_one(c, &mut csub, &mut ft, wl, wp);
                        }
                    }
                }
            }
        }
        if have_all_windows != 0 {
            for link in winlinks_in(&session) {
                let Some(wl) = link.get() else { continue };
                let mut ft = format_create_defaults(
                    None,
                    Some(c),
                    Some(session.as_session()),
                    Some(wl),
                    None::<&dyn crate::WindowPane>,
                );
                for csub in control_subs_snapshot(c) {
                    let mut csub = csub.borrow_mut();
                    if csub.type_0 == CONTROL_SUB_ALL_WINDOWS {
                        let Some(wl) = link.get() else { break };
                        control_check_subs_all_windows_one(c, &mut csub, &mut ft, wl);
                    }
                }
            }
        }
    }
}
pub unsafe fn control_add_sub(
    c: &mut client,
    name: &CStr,
    type_0: control_sub_type,
    id: core::ffi::c_int,
    format: &CStr,
) {
    unsafe {
        let watching = client_ref_of(c).map(|held| held.downgrade());
        let cs = c
            .control_state
            .as_deref_mut()
            .expect("the client has control state");
        let tv = timeval::from_secs(1 as __time_t);
        cs.subs.remove(name);
        let csub = control_sub {
            name: name.to_owned(),
            format: format.to_owned(),
            type_0,
            id: id as u_int,
            last: None,
            panes: control_sub_panes::new(),
            windows: control_sub_windows::new(),
        };
        cs.subs
            .insert(csub.name.clone(), Rc::new(RefCell::new(csub)));
        if !cs.subs_timer.is_set() {
            cs.subs_timer.set_callback(move || {
                if let Some(mut c) = watching.as_ref().and_then(ClientWeak::upgrade) {
                    control_check_subs_timer(c.as_client_mut());
                }
            });
        }
        if !cs.subs_timer.is_armed() {
            cs.subs_timer.arm(tv);
        }
    }
}
pub fn control_remove_sub(c: &mut client, name: &CStr) {
    {
        let cs = c
            .control_state
            .as_deref_mut()
            .expect("the client has control state");
        cs.subs.remove(name);
        if cs.subs.is_empty() {
            cs.subs_timer.disarm();
        }
    }
}

#[cfg(test)]
mod focused_tests {
    use super::*;
    use crate::pane_identity::PaneIdentity;
    use crate::pane_output::PaneOutputOffset;
    use crate::tests::test_fixtures::{StreamBuffer, globals, zeroed_client, zeroed_pane};

    struct ControlCtx {
        client: ClientRef,
        stream: StreamBuffer,
        _guard: crate::tests::test_fixtures::GlobalsGuard,
    }

    impl ControlCtx {
        fn new() -> Self {
            let guard = globals();
            let stream = StreamBuffer::new();
            let mut client = zeroed_client();
            (unsafe { client.as_client_mut() }).name = Some(c"focused-control".to_owned());
            let mut state = Box::new(control_state::default());
            state.write_event = stream.ptr();
            (unsafe { client.as_client_mut() }).control_state = Some(state);
            Self {
                client,
                stream,
                _guard: guard,
            }
        }

        fn client(&mut self) -> &mut client {
            unsafe { self.client.as_client_mut() }
        }

        fn state(&mut self) -> &mut control_state {
            (unsafe { self.client.as_client_mut() })
                .control_state
                .as_deref_mut()
                .unwrap()
        }
    }

    impl Drop for ControlCtx {
        fn drop(&mut self) {
            self.state().subs_timer.disarm();
            self.state().read_event = Stream::NONE;
            self.state().write_event = Stream::NONE;
        }
    }

    fn data_block(size: usize, time: u64) -> Box<control_block> {
        Box::new(control_block {
            id: 0,
            size,
            line: None,
            t: time,
        })
    }

    fn line_block(line: &CStr) -> Box<control_block> {
        Box::new(control_block {
            id: 0,
            size: 0,
            line: Some(line.to_owned()),
            t: 0,
        })
    }

    #[test]
    fn block_ownership_and_pane_links_remove_only_matching_entries() {
        let mut cs = control_state::default();
        let first = control_insert_block(&mut cs, data_block(3, 1));
        let second = control_insert_block(&mut cs, data_block(5, 2));
        let mut links = vec![first, second];
        control_unlink_block(&mut links, first);
        control_unlink_block(&mut links, first);
        assert_eq!(links, [second]);
        control_free_block(&mut cs, first);
        assert_eq!(cs.all_blocks.len(), 1);
        control_free_block(&mut cs, first);
        control_free_block(&mut cs, second);
        assert!(cs.all_blocks.is_empty());
    }

    #[test]
    fn pane_lifecycle_covers_add_get_offset_pause_off_and_discard() {
        let mut ctx = ControlCtx::new();
        let mut pane = zeroed_pane();
        pane.set_pane_id(71);
        pane.set_output_position(90);
        {
            assert!(control_get_pane(ctx.state(), &*pane).is_none());
            control_add_pane(ctx.state(), &*pane);
            assert_eq!(control_add_pane(ctx.state(), &*pane).offset.position(), 90);
            assert_eq!(
                ctx.state()
                    .panes
                    .get_mut(&pane.pane_id())
                    .unwrap()
                    .offset
                    .position(),
                90
            );
            let (offset, full) = control_pane_offset(ctx.client(), &*pane);
            assert_eq!(offset.unwrap().position(), 90);
            assert_eq!(full, 0);

            ctx.client().flags |= CLIENT_CONTROL_NOOUTPUT as u64;
            assert!(control_pane_offset(ctx.client(), &*pane).0.is_none());
            ctx.client().flags &= !(CLIENT_CONTROL_NOOUTPUT as u64);
            control_set_pane_off(ctx.client(), &*pane);
            assert_ne!(
                ctx.state().panes.get_mut(&pane.pane_id()).unwrap().flags & CONTROL_PANE_OFF,
                0
            );
            assert_eq!(control_pane_offset(ctx.client(), &*pane).1, 1);
            pane.set_output_position(101);
            control_set_pane_on(ctx.client(), &*pane);
            assert_eq!(
                ctx.state()
                    .panes
                    .get_mut(&pane.pane_id())
                    .unwrap()
                    .offset
                    .position(),
                101
            );
            control_pause_pane(ctx.client(), &*pane);
            assert_ne!(
                ctx.state().panes.get_mut(&pane.pane_id()).unwrap().flags & CONTROL_PANE_PAUSED,
                0
            );
            control_pause_pane(ctx.client(), &*pane);
            pane.set_output_position(110);
            control_continue_pane(ctx.client(), &*pane);
            assert_eq!(
                ctx.state()
                    .panes
                    .get_mut(&pane.pane_id())
                    .unwrap()
                    .offset
                    .position(),
                110
            );
            control_continue_pane(ctx.client(), &*pane);
            control_reset_offsets(ctx.client());
            assert!(ctx.state().panes.is_empty());
        }
    }

    #[test]
    fn line_flush_stops_at_data_then_resumes_and_write_data_appends_newline() {
        let mut ctx = ControlCtx::new();
        {
            control_insert_block(ctx.state(), line_block(c"first"));
            control_insert_block(ctx.state(), data_block(4, 0));
            control_insert_block(ctx.state(), line_block(c"last"));
            control_flush_all_blocks(ctx.client());
            assert_eq!(ctx.stream.written(), b"first\n");
            assert_eq!(ctx.state().all_blocks.len(), 2);
            ctx.state().all_blocks.remove(0);
            control_flush_all_blocks(ctx.client());
            assert_eq!(ctx.stream.written(), b"last\n");
            control_write_data(
                ctx.client(),
                Box::new(ByteBuffer::from(b"payload".to_vec())),
            );
            assert_eq!(ctx.stream.written(), b"payload\n");
        }
    }

    #[test]
    fn pausing_an_aged_pane_discards_only_its_blocks_before_queuing_the_notice() {
        let mut ctx = ControlCtx::new();
        let mut pane = zeroed_pane();
        pane.set_pane_id(91);
        let mut other = zeroed_pane();
        other.set_pane_id(92);
        let aged = control_insert_block(ctx.state(), data_block(12, get_timer()));
        control_add_pane(ctx.state(), &*pane).blocks.push(aged);
        let retained = control_insert_block(ctx.state(), data_block(7, get_timer()));
        control_add_pane(ctx.state(), &*other).blocks.push(retained);
        ctx.client().flags |= CLIENT_CONTROL_PAUSEAFTER;
        ctx.client().pause_age = 0;
        std::thread::sleep(std::time::Duration::from_millis(2));
        assert_eq!({ control_check_age(ctx.client(), &*pane) }, 1);
        assert!(ctx.state().panes[&91].blocks.is_empty());
        assert_ne!(ctx.state().panes[&91].flags & CONTROL_PANE_PAUSED, 0);
        assert_eq!(ctx.state().panes[&92].blocks, [retained]);
        assert_eq!(ctx.state().all_blocks.len(), 2);
        assert_eq!(ctx.state().all_blocks[0].id, retained);
        assert_eq!(
            ctx.state().all_blocks[1].line.as_deref(),
            Some(c"%pause %91")
        );
        control_pause_pane(ctx.client(), &*pane);
        assert_eq!(ctx.state().all_blocks.len(), 2);
    }

    #[test]
    fn pending_writer_discards_blocks_for_a_client_without_a_session() {
        let mut ctx = ControlCtx::new();
        let mut cp = Box::new(control_pane {
            pane: 99,
            offset: RustPaneOutputOffset::default(),
            queued: RustPaneOutputOffset::default(),
            flags: 0,
            pending_flag: 1,
            blocks: Vec::new(),
        });
        let block = control_insert_block(ctx.state(), data_block(12, 0));
        cp.blocks.push(block);
        let pending = std::ptr::NonNull::from(cp.as_mut());
        ctx.state().panes.insert(99, cp);
        unsafe {
            assert_eq!(control_write_pending(ctx.client(), pending, 32), 0);
            assert!(ctx.state().panes[&99].blocks.is_empty());
            assert!(ctx.state().all_blocks.is_empty());
        }
    }

    #[test]
    fn callbacks_observe_live_owners_and_ignore_dropped_ones() {
        let _guard = globals();
        let mut client = zeroed_client();
        let weak = Some(client.downgrade());
        let callback = on_client(&weak, |c| control_error_callback(c));
        callback(Stream::NONE);
        assert_ne!(unsafe { client.flags() } & CLIENT_EXIT as u64, 0);
        let error = on_client_error(&weak, |c| control_error_callback(c));
        *unsafe { client.flags_mut() } &= !(CLIENT_EXIT as u64);
        error(Stream::NONE, 0);
        assert_ne!(unsafe { client.flags() } & CLIENT_EXIT as u64, 0);
        drop(client);
        callback(Stream::NONE);
        error(Stream::NONE, 0);
    }

    #[test]
    fn subscription_timer_handles_no_session_and_removal_disarms_it() {
        let mut ctx = ControlCtx::new();
        unsafe {
            control_add_sub(ctx.client(), c"one", CONTROL_SUB_SESSION, 0, c"literal");
            assert!(ctx.state().subs_timer.is_armed());
            control_check_subs_timer(ctx.client());
            assert!(ctx.state().subs_timer.is_armed());
            control_remove_sub(ctx.client(), c"missing");
            control_remove_sub(ctx.client(), c"one");
            assert!(!ctx.state().subs_timer.is_armed());
        }
    }

    #[test]
    fn control_read_treats_a_nul_prefix_as_an_empty_command() {
        let mut ctx = ControlCtx::new();
        ctx.state().read_event = ctx.stream.ptr();
        ctx.state()
            .read_event
            .with_input(|input| input.append(b"\0ignored\ntrailing"));
        unsafe { control_read_callback(ctx.client()) };
        assert_ne!(ctx.client().flags & CLIENT_EXIT as u64, 0);
        assert_eq!(ctx.state().read_event.input_len(), b"trailing".len());
    }

    #[test]
    fn direct_and_queued_writes_preserve_order_and_done_state() {
        let mut ctx = ControlCtx::new();
        {
            assert_eq!(control_all_done(ctx.client()), 1);
            control_write(ctx.client(), c"%%notice %s", fmt_args![c"first"]);
            assert_eq!(ctx.stream.written(), b"%notice first\n");
            assert_eq!(control_all_done(ctx.client()), 0);
            ctx.stream.written();

            control_insert_block(ctx.state(), data_block(2, 0));
            control_write(ctx.client(), c"%%queued %d", fmt_args![7]);
            control_write(ctx.client(), c"%%queued %d", fmt_args![8]);
            assert_eq!(ctx.state().all_blocks.len(), 3);
            assert!(ctx.stream.written().is_empty());
            ctx.state().all_blocks.remove(0);
            control_flush_all_blocks(ctx.client());
            assert_eq!(ctx.stream.written(), b"%queued 7\n%queued 8\n");
            assert!(ctx.state().all_blocks.is_empty());
        }
    }

    #[test]
    fn borrowed_control_offset_rebases_without_changing_the_pane() {
        let mut ctx = ControlCtx::new();
        let mut pane = zeroed_pane();
        pane.set_pane_id(89);
        pane.set_output_position(90);
        control_add_pane(ctx.state(), &*pane);
        let (offset, full) = control_pane_offset_mut(ctx.client(), &*pane);
        assert_eq!(full, 0);
        offset.unwrap().rebase(40);
        assert_eq!(
            control_pane_offset(ctx.client(), &*pane)
                .0
                .unwrap()
                .position(),
            50
        );
        assert_eq!(pane.output_position().position(), 90);
        control_set_pane_off(ctx.client(), &*pane);
        let (offset, full) = control_pane_offset_mut(ctx.client(), &*pane);
        assert!(offset.is_none());
        assert_eq!(full, 1);
    }

    #[test]
    fn pane_offset_backpressure_and_pending_reset_cover_bookkeeping() {
        let mut ctx = ControlCtx::new();
        let mut pane = zeroed_pane();
        pane.set_pane_id(88);
        {
            control_add_pane(ctx.state(), &*pane);
            ctx.state()
                .write_event
                .write(&vec![b'x'; CONTROL_BUFFER_LOW as usize]);
            assert_eq!(control_pane_offset(ctx.client(), &*pane).1, 1);
            ctx.state().panes.get_mut(&pane.pane_id()).unwrap().flags = CONTROL_PANE_PAUSED;
            assert!(control_pane_offset(ctx.client(), &*pane).0.is_none());
            ctx.state()
                .panes
                .get_mut(&pane.pane_id())
                .unwrap()
                .pending_flag = 1;
            let pending = std::ptr::NonNull::from(
                ctx.state().panes.get_mut(&pane.pane_id()).unwrap().as_mut(),
            );
            ctx.state().pending_list.push(pending);
            ctx.state().pending_count = 1;
            let block = control_insert_block(ctx.state(), data_block(1, 0));
            ctx.state()
                .panes
                .get_mut(&pane.pane_id())
                .unwrap()
                .blocks
                .push(block);
            control_reset_offsets(ctx.client());
            assert_eq!(ctx.state().pending_count, 0);
            assert!(ctx.state().pending_list.is_empty());
            assert!(ctx.state().panes.is_empty());
            assert!(ctx.state().all_blocks.is_empty());
        }
    }

    #[test]
    fn subscription_targets_match_session_owners_and_skip_removed_links() {
        use crate::tests::test_fixtures::{Registry, Session, Window, link, unlink_all};
        let mut ctx = ControlCtx::new();
        let mut registry = Registry::new();
        let mut session = Session::new(61, "subscription-session");
        let mut other = Session::new(62, "other-session");
        let mut window = Window::new(81, "shared", 80, 24);
        registry.add_window(&mut window);
        link(&mut session, &mut window, 3);
        link(&mut other, &mut window, 7);
        ctx.client()
            .set_attached_session(Some(&session.reference()));
        unsafe {
            control_add_sub(
                ctx.client(),
                c"one",
                CONTROL_SUB_WINDOW,
                81,
                c"#{session_id}/#{window_id}/#{window_index}",
            );
            control_add_sub(
                ctx.client(),
                c"all",
                CONTROL_SUB_ALL_WINDOWS,
                0,
                c"#{session_id}/#{window_id}/#{window_index}",
            );
            control_check_subs_timer(ctx.client());
            assert_eq!(ctx.stream.written(), b"%subscription-changed one $61 @81 3 - : $61/@81/3\n%subscription-changed all $61 @81 3 - : $61/@81/3\n");
            control_check_subs_timer(ctx.client());
            assert!(ctx.stream.written().is_empty());
            unlink_all(&mut session);
            control_check_subs_timer(ctx.client());
            assert!(ctx.stream.written().is_empty());
        }
        ctx.client().set_attached_session(None);
        unlink_all(&mut other);
    }

    #[test]
    fn subscription_snapshot_survives_replacement_without_retaining_the_registry() {
        let mut ctx = ControlCtx::new();
        unsafe { control_add_sub(ctx.client(), c"same", CONTROL_SUB_SESSION, 1, c"old") };
        let snapshot = control_subs_snapshot(ctx.client());
        let watched = Rc::downgrade(&snapshot[0]);
        {
            let mut old = snapshot[0].borrow_mut();
            unsafe { control_add_sub(ctx.client(), c"same", CONTROL_SUB_SESSION, 2, c"new") };
            old.last = Some(c"old result".to_owned());
            let replacement = control_subs_snapshot(ctx.client());
            assert!(!Rc::ptr_eq(&snapshot[0], &replacement[0]));
            assert_eq!(replacement[0].borrow().format.as_c_str(), c"new");
            assert!(replacement[0].borrow().last.is_none());
            control_remove_sub(ctx.client(), c"same");
            assert!(control_subs_snapshot(ctx.client()).is_empty());
            assert_eq!(old.format.as_c_str(), c"old");
        }
        drop(snapshot);
        assert!(watched.upgrade().is_none());
    }

    #[test]
    fn subscriptions_replace_by_name_and_keep_independent_types() {
        let mut ctx = ControlCtx::new();
        unsafe {
            control_add_sub(ctx.client(), c"same", CONTROL_SUB_SESSION, 1, c"one");
            control_add_sub(ctx.client(), c"same", CONTROL_SUB_PANE, 2, c"two");
            control_add_sub(ctx.client(), c"window", CONTROL_SUB_WINDOW, 3, c"three");
            assert_eq!(ctx.state().subs.len(), 2);
            let same = ctx.state().subs.get(c"same").unwrap().clone();
            let same = same.borrow();
            assert_eq!(same.type_0, CONTROL_SUB_PANE);
            assert_eq!(same.id, 2);
            assert_eq!(same.format.as_bytes(), b"two");
            drop(same);
            control_check_subs_timer(ctx.client());
            assert!(ctx.state().subs_timer.is_armed());
            control_remove_sub(ctx.client(), c"same");
            assert!(ctx.state().subs_timer.is_armed());
            control_remove_sub(ctx.client(), c"window");
            assert!(!ctx.state().subs_timer.is_armed());
        }
    }
}

#[cfg(test)]
pub const BUFFER_EOL_NUL: core::ffi::c_uint = 4;
#[cfg(test)]
pub const BUFFER_EOL_LF: core::ffi::c_uint = 3;
#[cfg(test)]
pub const BUFFER_EOL_CRLF_STRICT: core::ffi::c_uint = 2;
#[cfg(test)]
pub const BUFFER_EOL_CRLF: core::ffi::c_uint = 1;
#[cfg(test)]
pub const BUFFER_EOL_ANY: core::ffi::c_uint = 0;
#[cfg(test)]
pub use crate::consts::{
    CLIENT_EXIT_DETACH, CLIENT_EXIT_RETURN, CLIENT_EXIT_SHUTDOWN, LAYOUT_LEFTRIGHT,
    LAYOUT_TOPBOTTOM, LAYOUT_WINDOWPANE, MSG_COMMAND, MSG_DETACH, MSG_DETACHKILL, MSG_EXEC,
    MSG_EXIT, MSG_EXITED, MSG_EXITING, MSG_FLAGS, MSG_IDENTIFY_CLIENTPID, MSG_IDENTIFY_CWD,
    MSG_IDENTIFY_DONE, MSG_IDENTIFY_ENVIRON, MSG_IDENTIFY_FEATURES, MSG_IDENTIFY_FLAGS,
    MSG_IDENTIFY_LONGFLAGS, MSG_IDENTIFY_OLDCWD, MSG_IDENTIFY_STDIN, MSG_IDENTIFY_STDOUT,
    MSG_IDENTIFY_TERM, MSG_IDENTIFY_TERMINFO, MSG_IDENTIFY_TTYNAME, MSG_LOCK, MSG_OLDSTDERR,
    MSG_OLDSTDIN, MSG_OLDSTDOUT, MSG_READ, MSG_READ_CANCEL, MSG_READ_DONE, MSG_READ_OPEN,
    MSG_READY, MSG_RESIZE, MSG_SHELL, MSG_SHUTDOWN, MSG_SUSPEND, MSG_UNLOCK, MSG_VERSION,
    MSG_WAKEUP, MSG_WRITE, MSG_WRITE_CLOSE, MSG_WRITE_OPEN, MSG_WRITE_READY, PANE_LINES_DOUBLE,
    PANE_LINES_HEAVY, PANE_LINES_NUMBER, PANE_LINES_SIMPLE, PANE_LINES_SINGLE, PANE_LINES_SPACES,
    PROGRESS_BAR_ERROR, PROGRESS_BAR_HIDDEN, PROGRESS_BAR_INDETERMINATE, PROGRESS_BAR_NORMAL,
    PROGRESS_BAR_PAUSED, PROMPT_COMMAND, PROMPT_ENTRY, PROMPT_TYPE_COMMAND, PROMPT_TYPE_INVALID,
    PROMPT_TYPE_SEARCH, PROMPT_TYPE_TARGET, PROMPT_TYPE_WINDOW_TARGET, SCREEN_CURSOR_BAR,
    SCREEN_CURSOR_BLOCK, SCREEN_CURSOR_DEFAULT, SCREEN_CURSOR_UNDERLINE,
    STYLE_ALIGN_ABSOLUTE_CENTRE, STYLE_ALIGN_CENTRE, STYLE_ALIGN_DEFAULT, STYLE_ALIGN_LEFT,
    STYLE_ALIGN_RIGHT, STYLE_DEFAULT_BASE, STYLE_DEFAULT_POP, STYLE_DEFAULT_PUSH,
    STYLE_DEFAULT_SET, STYLE_LIST_FOCUS, STYLE_LIST_LEFT_MARKER, STYLE_LIST_OFF, STYLE_LIST_ON,
    STYLE_LIST_RIGHT_MARKER, STYLE_RANGE_CONTROL, STYLE_RANGE_LEFT, STYLE_RANGE_NONE,
    STYLE_RANGE_PANE, STYLE_RANGE_RIGHT, STYLE_RANGE_SESSION, STYLE_RANGE_USER, STYLE_RANGE_WINDOW,
    THEME_DARK, THEME_LIGHT, THEME_UNKNOWN,
};

#[cfg(test)]
pub use crate::consts::{CLIENT_DEAD, CLIENT_SUSPENDED};
