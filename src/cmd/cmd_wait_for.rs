use crate::arguments::{args_has, args_string};
use crate::cmd::cmd_get_args;
use crate::cmd::queue::{cmdq_continue, cmdq_error, cmdq_get_client};
use crate::fmt_args;
use crate::log::log_debug;
use crate::tree::GlobalTree;
pub use crate::types::*;
use ::core::ffi::CStr;
use ::std::collections::btree_map::Entry;
use ::std::collections::{BTreeMap, VecDeque};
use ::std::ffi::CString;
pub const CMD_FIND_PANE: cmd_find_type = 0;
pub const CMD_RETURN_WAIT: cmd_retval = 1;
pub const CMD_RETURN_NORMAL: cmd_retval = 0;
pub const CMD_RETURN_ERROR: cmd_retval = -1;
pub(crate) static cmd_wait_for_entry: cmd_entry = {
    cmd_entry {
        name: c"wait-for",
        alias: Some(c"wait"),
        args: args_parse_t {
            template: c"LSU",
            lower: 1 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-L|-S|-U] channel",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: cmd_wait_for_exec,
    }
};

/// One named channel: whether a `-L` holds it, whether a `-S` has already
/// arrived, and the command queue items blocked on each of the two waits.
struct WaitChannel {
    locked: bool,
    woken: bool,
    waiters: VecDeque<*mut cmdq_item>,
    lockers: VecDeque<*mut cmdq_item>,
}

static WAIT_CHANNELS: GlobalTree<CString, WaitChannel> = GlobalTree::new();

/// Every channel, in name order. tmux runs one command at a time on a single
/// thread, which is what makes handing out the global safe.
fn channels() -> &'static mut BTreeMap<CString, WaitChannel> {
    WAIT_CHANNELS.map()
}

/// The named channel, added empty if this is the first mention of it.
fn channel_for(name: &CStr) -> &'static mut WaitChannel {
    match channels().entry(name.to_owned()) {
        Entry::Occupied(entry) => entry.into_mut(),
        Entry::Vacant(entry) => {
            unsafe { log_debug(c"add wait channel %s".as_ptr(), fmt_args![name.as_ptr()]) };
            entry.insert(WaitChannel {
                locked: false,
                woken: false,
                waiters: VecDeque::new(),
                lockers: VecDeque::new(),
            })
        }
    }
}

/// Drop the channel unless something still needs it: a lock holds it, a
/// blocked waiter holds it, and so does never having been signalled.
fn remove_if_idle(name: &CStr) {
    let channels = channels();
    if channels
        .get(name)
        .is_some_and(|wc| !wc.locked && wc.waiters.is_empty() && wc.woken)
    {
        unsafe { log_debug(c"remove wait channel %s".as_ptr(), fmt_args![name.as_ptr()]) };
        channels.remove(name);
    }
}

unsafe fn cmd_wait_for_exec(self_0: &cmd, item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let args = cmd_get_args(self_0);
        let name = CStr::from_ptr(args_string(args, 0 as u_int));
        if args_has(args, b'S') != 0 {
            return cmd_wait_for_signal(item, name);
        }
        if args_has(args, b'L') != 0 {
            return cmd_wait_for_lock(item, name);
        }
        if args_has(args, b'U') != 0 {
            return cmd_wait_for_unlock(item, name);
        }
        cmd_wait_for_wait(item, name)
    }
}

unsafe fn cmd_wait_for_signal(_item: *mut cmdq_item, name: &CStr) -> cmd_retval {
    unsafe {
        let wc = channel_for(name);
        if wc.waiters.is_empty() && !wc.woken {
            log_debug(
                c"signal wait channel %s, no waiters".as_ptr(),
                fmt_args![name.as_ptr()],
            );
            wc.woken = true;
            return CMD_RETURN_NORMAL;
        }
        log_debug(
            c"signal wait channel %s, with waiters".as_ptr(),
            fmt_args![name.as_ptr()],
        );
        for item in ::core::mem::take(&mut wc.waiters) {
            cmdq_continue(item);
        }
        remove_if_idle(name);
        CMD_RETURN_NORMAL
    }
}

unsafe fn cmd_wait_for_wait(item: *mut cmdq_item, name: &CStr) -> cmd_retval {
    unsafe {
        let c = cmdq_get_client(&*item);
        if c.is_null() {
            cmdq_error(item, c"not able to wait".as_ptr(), fmt_args![]);
            return CMD_RETURN_ERROR;
        }
        if channel_for(name).woken {
            log_debug(
                c"wait channel %s already woken (%p)".as_ptr(),
                fmt_args![name.as_ptr(), c],
            );
            remove_if_idle(name);
            return CMD_RETURN_NORMAL;
        }
        log_debug(
            c"wait channel %s not woken (%p)".as_ptr(),
            fmt_args![name.as_ptr(), c],
        );
        channel_for(name).waiters.push_back(item);
        CMD_RETURN_WAIT
    }
}

unsafe fn cmd_wait_for_lock(item: *mut cmdq_item, name: &CStr) -> cmd_retval {
    unsafe {
        if cmdq_get_client(&*item).is_null() {
            cmdq_error(item, c"not able to lock".as_ptr(), fmt_args![]);
            return CMD_RETURN_ERROR;
        }
        let wc = channel_for(name);
        if wc.locked {
            wc.lockers.push_back(item);
            return CMD_RETURN_WAIT;
        }
        wc.locked = true;
        CMD_RETURN_NORMAL
    }
}

unsafe fn cmd_wait_for_unlock(item: *mut cmdq_item, name: &CStr) -> cmd_retval {
    unsafe {
        let next = match channels().get_mut(name) {
            Some(wc) if wc.locked => match wc.lockers.pop_front() {
                Some(next) => Some(next),
                None => {
                    wc.locked = false;
                    None
                }
            },
            _ => {
                cmdq_error(
                    item,
                    c"channel %s not locked".as_ptr(),
                    fmt_args![name.as_ptr()],
                );
                return CMD_RETURN_ERROR;
            }
        };
        match next {
            Some(next) => cmdq_continue(next),
            None => remove_if_idle(name),
        }
        CMD_RETURN_NORMAL
    }
}

/// Let everything blocked on every channel run again and forget the channels.
/// Releasing a channel's waiters and lockers leaves it woken, unlocked and
/// unwaited, which is exactly the state in which a channel stops being kept,
/// so a flush always empties the whole set.
pub fn cmd_wait_for_flush() {
    unsafe {
        for (name, wc) in ::core::mem::take(channels()) {
            for item in wc.waiters.into_iter().chain(wc.lockers) {
                cmdq_continue(item);
            }
            log_debug(c"remove wait channel %s".as_ptr(), fmt_args![name.as_ptr()]);
        }
    }
}

#[cfg(test)]
#[path = "../tests/test_cmd_wait_for.rs"]
mod tests;
