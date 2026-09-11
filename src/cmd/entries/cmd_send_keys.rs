use crate::args::arguments_trait::Arguments as _;
use crate::args::RustArguments;
use crate::args::args_parse_t;
use crate::args::args_strtonum_and_expand;
use crate::cmd::{CmdqItemRef, cmdq_item_ref_of};
use crate::cmd::{cmd_get_args, cmd_get_entry, cmd_mouse_pane};
use crate::ffi::strtol;
use crate::fmt_args;
use crate::key_bindings::key_bindings_get_table_ref;

use crate::cmd::cmdq_item;
use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
use crate::consts::{
    CLIENT_READONLY, CMD_AFTERHOOK, CMD_CLIENT_CANFAIL, CMD_CLIENT_CFLAG, CMD_FIND_PANE,
    CMD_READONLY, CMD_RETURN_ERROR, CMD_RETURN_NORMAL, KEYC_LITERAL, KEYC_MASK_FLAGS, KEYC_NONE,
    KEYC_SENT, KEYC_UNKNOWN, UINT_MAX, UTF8_DONE,
};
use crate::text::{KeyStringCodec, RustKeyStringCodec};
use crate::text::{utf8_from_data, utf8_fromcstr};
use crate::types::{
    ClientRef, OptionsRef, key_code, key_event, mouse_event, u_char, u_int, uint64_t,
};

pub(crate) static cmd_send_keys_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"send-keys",
        alias: Some(c"send"),
        args: args_parse_t {
            template: c"c:FHKlMN:Rt:X",
            lower: 0 as core::ffi::c_int,
            upper: -(1 as core::ffi::c_int),
            cb: None,
        },
        usage: c"[-FHKlMRX] [-c target-client] [-N repeat-count] [-t target-pane] [key ...]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as core::ffi::c_int,
        },
        flags: CMD_AFTERHOOK | CMD_CLIENT_CFLAG | CMD_CLIENT_CANFAIL | CMD_READONLY,
        exec: cmd_send_keys_exec,
    }
};
pub(crate) static cmd_send_prefix_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"send-prefix",
        alias: None,
        args: args_parse_t {
            template: c"2t:",
            lower: 0 as core::ffi::c_int,
            upper: 0 as core::ffi::c_int,
            cb: None,
        },
        usage: c"[-2] [-t target-pane]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as core::ffi::c_int,
        },
        flags: CMD_AFTERHOOK,
        exec: cmd_send_keys_exec,
    }
};
unsafe fn cmd_send_keys_inject_key(
    item: &CmdqItemRef,
    mut after: Option<CmdqItemRef>,
    args: &RustArguments,
    key: key_code,
) -> Option<CmdqItemRef> {
    unsafe {
        let (mut tc, target) = (item.target_client(), item.target());

        if ({
            let flag = 'K' as i32 as u_char;
            args.argument_flag_count(flag)
        }) != 0
        {
            if tc.is_none() {
                return Some(item.clone());
            }
            let event = Box::new(key_event {
                key: (key as core::ffi::c_ulonglong | KEYC_SENT) as key_code,
                m: mouse_event::default(),
                buf: Vec::new(),
            });
            tc.as_mut()
                .expect("the command has a client")
                .handle_key(event);
            return Some(item.clone());
        }
        let pane = target.pane_ref()?;
        let mode_table = pane.mode_key_table();
        let Some(mode_table) = mode_table else {
            if ClientRef::send_key_to_pane(pane, tc.as_mut(), key, None) != 0 as core::ffi::c_int {
                return None;
            }
            return Some(item.clone());
        };
        let table_ref = key_bindings_get_table_ref(mode_table, 1 as core::ffi::c_int)
            .expect("mode key table creation requested");
        let binding = table_ref.binding(key & !KEYC_MASK_FLAGS);
        if let Some(binding) = binding {
            after = ClientRef::dispatch_binding(
                &binding,
                after.as_ref(),
                tc.as_mut(),
                None,
                Some(&target),
            );
        }
        after
    }
}
unsafe fn cmd_send_keys_inject_string(
    item: &CmdqItemRef,
    mut after: Option<CmdqItemRef>,
    args: &RustArguments,
    i: core::ffi::c_int,
) -> Option<CmdqItemRef> {
    unsafe {
        let s = {
            let idx = i as u_int;
            args.argument_string(idx)
        }
        .expect("argument index checked");
        let mut key: key_code;
        let mut endptr: *mut core::ffi::c_char = core::ptr::null_mut::<core::ffi::c_char>();
        let n: core::ffi::c_long;
        let mut literal: core::ffi::c_int;
        if ({
            let flag = 'H' as i32 as u_char;
            args.argument_flag_count(flag)
        }) != 0
        {
            n = strtol(s.as_ptr(), &raw mut endptr, 16 as core::ffi::c_int);
            if s.to_bytes().is_empty()
                || n < 0 as core::ffi::c_long
                || n > 0xff as core::ffi::c_long
                || *endptr as core::ffi::c_int != '\0' as i32
            {
                return Some(item.clone());
            }
            return cmd_send_keys_inject_key(item, after, args, KEYC_LITERAL | n as key_code);
        }
        literal = {
            let flag = 'l' as i32 as u_char;
            args.argument_flag_count(flag)
        };
        if literal == 0 {
            key = RustKeyStringCodec.parse_key(s);
            if key != KEYC_NONE as core::ffi::c_ulong as key_code
                && key != KEYC_UNKNOWN as core::ffi::c_ulong as key_code
            {
                after = cmd_send_keys_inject_key(item, after, args, key);
                if after.is_some() {
                    return after;
                }
            }
            literal = 1 as core::ffi::c_int;
        }
        if literal != 0 {
            let ud = utf8_fromcstr(s);
            for item_ud in &ud {
                if item_ud.size == 1 && item_ud.data[0] <= 0x7f {
                    key = item_ud.data[0] as key_code;
                    after = cmd_send_keys_inject_key(item, after, args, key);
                } else if let (UTF8_DONE, uc) = utf8_from_data(item_ud) {
                    key = uc as key_code;
                    after = cmd_send_keys_inject_key(item, after, args, key);
                }
            }
        }
        after
    }
}
unsafe fn cmd_send_keys_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args: &RustArguments = cmd_get_args(self_0);
    let mut tc = item.target_client();
    let session = item.target.session();
    let link = item.target.winlink_ref();
    let pane = item.target.pane_list_ref();
    let event_state_ref = item.state_ref();
    let item_ref = cmdq_item_ref_of(item).expect("a running send-keys item is live");
    let mut after = Some(item_ref.clone());
    let key: key_code;
    let mut i: u_int;
    let mut np: u_int = 1 as u_int;
    let count: u_int = args.argument_count();
    let mut cause = None;
    if unsafe {
        tc.is_some()
            && tc.as_ref().expect("the command has a client").flags() & CLIENT_READONLY as uint64_t
                != 0
            && ({
                let flag = 'X' as i32 as u_char;
                args.argument_flag_count(flag)
            }) == 0
    } {
        unsafe { item.error(c"client is read-only", fmt_args![]) };
        return CMD_RETURN_ERROR;
    }
    if ({
        let flag = 'N' as i32 as u_char;
        args.argument_flag_count(flag)
    }) != 0
    {
        unsafe {
            np = args_strtonum_and_expand(
                args,
                'N' as i32 as u_char,
                1 as core::ffi::c_longlong,
                UINT_MAX as core::ffi::c_longlong,
                item,
                &mut cause,
            ) as u_int
        };
        if let Some(cause) = cause.as_ref() {
            unsafe { item.error(c"repeat count %s", fmt_args![cause.as_c_str()]) };
            return CMD_RETURN_ERROR;
        }
        if (args.argument_flag_count(b'X') != 0 || count == 0)
            && pane
                .as_ref()
                .and_then(|pane| unsafe { pane.set_mode_prefix(np) })
                == Some(false)
        {
            unsafe { item.error(c"not in a mode", fmt_args![]) };
            return CMD_RETURN_ERROR;
        }
    }
    if args.argument_flag_count(b'X') != 0 {
        let dispatched = event_state_ref.with_mouse_event(|mouse| {
            let mouse = if mouse.valid == 0 { None } else { Some(mouse) };
            pane.as_ref().is_some_and(|pane| unsafe {
                pane.mode_command(tc.as_mut(), session.as_ref(), link.as_ref(), args, mouse)
            })
        });
        if !dispatched {
            unsafe { item.error(c"not in a mode", fmt_args![]) };
            return CMD_RETURN_ERROR;
        }
        return CMD_RETURN_NORMAL;
    }
    if ({
        let flag = 'M' as i32 as u_char;
        args.argument_flag_count(flag)
    }) != 0
    {
        return event_state_ref.with_mouse_event(|m| {
            let Some((_, _, mouse_wp)) = (unsafe { cmd_mouse_pane(m) }) else {
                unsafe { item.error(c"no mouse target", fmt_args![]) };
                return CMD_RETURN_ERROR;
            };
            unsafe { ClientRef::send_key_to_pane(mouse_wp, tc.as_mut(), m.key, Some(m)) };
            CMD_RETURN_NORMAL
        });
    }
    if core::ptr::eq(cmd_get_entry(self_0), &cmd_send_prefix_entry) {
        let session = session
            .as_ref()
            .expect("a send-prefix target has a session");
        if ({
            let flag = '2' as i32 as u_char;
            args.argument_flag_count(flag)
        }) != 0
        {
            {
                key = session.options().number(c"prefix2") as key_code
            };
        } else {
            {
                key = session.options().number(c"prefix") as key_code
            };
        }
        unsafe { cmd_send_keys_inject_key(&item_ref, Some(item_ref.clone()), args, key) };
        return CMD_RETURN_NORMAL;
    }
    if args.argument_flag_count(b'R') != 0
        && let Some(pane) = pane.as_ref()
    {
        unsafe { pane.reset_terminal() };
    }
    if count == 0 as u_int {
        if ({
            let flag = 'N' as i32 as u_char;
            args.argument_flag_count(flag)
        }) != 0
            || ({
                let flag = 'R' as i32 as u_char;
                args.argument_flag_count(flag)
            }) != 0
        {
            return CMD_RETURN_NORMAL;
        }
        let event_key = event_state_ref.event_snapshot().key;
        while np != 0 as u_int {
            unsafe { cmd_send_keys_inject_key(&item_ref, None, args, event_key) };
            np = np.wrapping_sub(1);
        }
        return CMD_RETURN_NORMAL;
    }
    while np != 0 as u_int {
        i = 0 as u_int;
        while i < count {
            unsafe {
                after = cmd_send_keys_inject_string(&item_ref, after, args, i as core::ffi::c_int)
            };
            i = i.wrapping_add(1);
        }
        np = np.wrapping_sub(1);
    }
    CMD_RETURN_NORMAL
}
