use crate::arguments::{args_has, args_string_str};
use crate::cmd::cmd_get_args;
use crate::cmdq::{CmdqItemWeak, cmdq_item_weak_of};
pub use crate::consts::{CMD_FIND_PANE, CMD_RETURN_ERROR, CMD_RETURN_NORMAL, CMD_RETURN_WAIT};
use crate::fmt_args;
use crate::log::log_debug;
pub use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
pub use crate::cmdq::cmdq_item;
pub use crate::types::args_parse_t;
use ::core::ffi::CStr;
use ::std::cell::RefCell;
use ::std::collections::btree_map::Entry;
use ::std::collections::{BTreeMap, VecDeque};
use ::std::ffi::CString;

pub(crate) static cmd_wait_for_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"wait-for",
        alias: Some(c"wait"),
        args: args_parse_t {
            template: c"LSU",
            lower: 1 as core::ffi::c_int,
            upper: 1 as core::ffi::c_int,
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
        flags: 0 as core::ffi::c_int,
        exec: cmd_wait_for_exec,
    }
};

/// One named channel: whether a `-L` holds it, whether a `-S` has already
/// arrived, and the command queue items blocked on each of the two waits —
/// held as observations, so an item whose queue has already given it up is
/// skipped rather than answered.
struct WaitChannel {
    locked: bool,
    woken: bool,
    waiters: VecDeque<CmdqItemWeak>,
    lockers: VecDeque<CmdqItemWeak>,
}

thread_local! {
    static WAIT_CHANNELS: RefCell<BTreeMap<CString, WaitChannel>> = const {
        RefCell::new(BTreeMap::new())
    };
}

/// Accesses the current server thread's channels for one state transition.
fn with_channels<R>(operation: impl FnOnce(&mut BTreeMap<CString, WaitChannel>) -> R) -> R {
    WAIT_CHANNELS.with_borrow_mut(operation)
}

/// The named channel, added empty if this is the first mention of it.
fn channel_for_in<'a>(
    channels: &'a mut BTreeMap<CString, WaitChannel>,
    name: &CStr,
) -> &'a mut WaitChannel {
    match channels.entry(name.to_owned()) {
        Entry::Occupied(entry) => entry.into_mut(),
        Entry::Vacant(entry) => {
            unsafe { log_debug(c"add wait channel %s", fmt_args![name]) };
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
    with_channels(|all| {
        if all
            .get(name)
            .is_some_and(|wc| !wc.locked && wc.waiters.is_empty() && wc.woken)
        {
            unsafe { log_debug(c"remove wait channel %s", fmt_args![name]) };
            all.remove(name);
        }
    });
}

unsafe fn cmd_wait_for_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args = cmd_get_args(self_0);
    let name = unsafe { args_string_str(args, 0).expect("argument count checked") };
    if args_has(args, b'S') != 0 {
        return unsafe { cmd_wait_for_signal(name) };
    }
    if args_has(args, b'L') != 0 {
        return unsafe { cmd_wait_for_lock(item, name) };
    }
    if args_has(args, b'U') != 0 {
        return unsafe { cmd_wait_for_unlock(item, name) };
    }
    unsafe { cmd_wait_for_wait(item, name) }
}

unsafe fn cmd_wait_for_signal(name: &CStr) -> cmd_retval {
    unsafe {
        let waiters = with_channels(|all| {
            let wc = channel_for_in(all, name);
            if wc.waiters.is_empty() && !wc.woken {
                log_debug(c"signal wait channel %s, no waiters", fmt_args![name]);
                wc.woken = true;
                return None;
            }
            log_debug(c"signal wait channel %s, with waiters", fmt_args![name]);
            Some(core::mem::take(&mut wc.waiters))
        });
        let Some(waiters) = waiters else {
            return CMD_RETURN_NORMAL;
        };
        for item in waiters {
            if let Some(item) = item.upgrade() {
                item.resume();
            }
        }
        remove_if_idle(name);
        CMD_RETURN_NORMAL
    }
}

unsafe fn cmd_wait_for_wait(item: &cmdq_item, name: &CStr) -> cmd_retval {
    unsafe {
        let Some(c) = item.client() else {
            item.error(c"not able to wait", fmt_args![]);
            return CMD_RETURN_ERROR;
        };
        let waiting = cmdq_item_weak_of(item).expect("the waiting item is live");
        let woken = with_channels(|all| {
            let wc = channel_for_in(all, name);
            if !wc.woken {
                wc.waiters.push_back(waiting);
            }
            wc.woken
        });
        if woken {
            c.log_wait_channel(name, true);
            remove_if_idle(name);
            return CMD_RETURN_NORMAL;
        }
        c.log_wait_channel(name, false);
        CMD_RETURN_WAIT
    }
}

unsafe fn cmd_wait_for_lock(item: &cmdq_item, name: &CStr) -> cmd_retval {
    unsafe {
        if item.client().is_none() {
            item.error(c"not able to lock", fmt_args![]);
            return CMD_RETURN_ERROR;
        }
        with_channels(|all| {
            let wc = channel_for_in(all, name);
            if wc.locked {
                wc.lockers
                    .push_back(cmdq_item_weak_of(&*item).expect("the locking item is live"));
                return CMD_RETURN_WAIT;
            }
            wc.locked = true;
            CMD_RETURN_NORMAL
        })
    }
}

unsafe fn cmd_wait_for_unlock(item: &cmdq_item, name: &CStr) -> cmd_retval {
    unsafe {
        let next = with_channels(|all| match all.get_mut(name) {
            Some(wc) if wc.locked => {
                let mut next = None;
                while let Some(candidate) = wc.lockers.pop_front() {
                    if let Some(live) = candidate.upgrade() {
                        next = Some(live);
                        break;
                    }
                }
                if next.is_none() {
                    wc.locked = false;
                }
                Ok(next)
            }
            _ => Err(()),
        });
        let next = match next {
            Ok(next) => next,
            Err(()) => {
                item.error(c"channel %s not locked", fmt_args![name]);
                return CMD_RETURN_ERROR;
            }
        };
        match next {
            Some(next) => next.resume(),
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
        let channels = with_channels(core::mem::take);
        for (name, wc) in channels {
            for item in wc.waiters.into_iter().chain(wc.lockers) {
                if let Some(item) = item.upgrade() {
                    item.resume();
                }
            }
            log_debug(c"remove wait channel %s", fmt_args![name.as_c_str()]);
        }
    }
}

#[cfg(test)]
#[path = "../../tests/test_cmd_wait_for.rs"]
mod tests;
