use crate::src::arguments::args_has;
use crate::src::cmd::cmd_get_args_mut;
use crate::src::cmd::queue::{cmdq_get_target_client, cmdq_print};
use crate::src::format::bytes::write_cstr;
use crate::src::format::{
    format_add, format_add_tv, format_create_from_target, format_expand_cstring, format_free,
};
use crate::src::job::job_print_summary;
use crate::src::server::message_log;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::client::client;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmdq_item};
use crate::src::shared::command::{CMD_AFTERHOOK, CMD_CLIENT_CANFAIL, CMD_CLIENT_TFLAG};
use crate::src::shared::format::format_tree;
use crate::src::shared::tty::tty_term;
use crate::src::shared::tty::*;
use crate::src::tty_term::tty_term_owner_ptr;
use crate::src::tty_term::tty_terms;
use crate::src::tty_term::{tty_term_describe, tty_term_ncodes};

pub const SHOW_MESSAGES_TEMPLATE: [::core::ffi::c_char; 37] = unsafe {
    ::core::mem::transmute::<[u8; 37], [::core::ffi::c_char; 37]>(
        *b"#{t/p:message_time}: #{message_text}\0",
    )
};
pub static cmd_show_messages_entry: cmd_entry = {
    cmd_entry {
        name: c"show-messages",
        alias: Some(c"showmsgs"),
        args: args_parse {
            template: c"JTt:",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-JT] [-t target-client]",
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
        flags: CMD_AFTERHOOK | CMD_CLIENT_TFLAG | CMD_CLIENT_CANFAIL,
        exec: Some(cmd_show_messages_exec),
    }
};
unsafe fn cmd_show_messages_terminals(
    mut self_0: refbox::Weak<cmd>,
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
    mut blank: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let item = item_handle.get();
    let mut args: *mut args =
        cmd_get_args_mut(self_0.get_mut_unchecked()).map_or(std::ptr::null_mut(), |args| args);
    let tc_owner = cmdq_get_target_client((item).as_ref());
    let mut tc: *mut client = tc_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    let mut term: *const tty_term = ::core::ptr::null::<tty_term>();
    let mut i: u_int = 0;
    let mut n: u_int = 0;
    n = 0 as u_int;
    term = tty_terms.lh_first;
    while !term.is_null() {
        if !(args_has(args, 't' as i32 as u_char) != 0
            && !tc.is_null()
            && term != tty_term_owner_ptr(&(*tc).tty.term).map_or(std::ptr::null(), |term| term))
        {
            if blank != 0 {
                cmdq_print(item_handle, |out| {
                    write_cstr(out, b"\0" as *const u8 as *const ::core::ffi::c_char)
                });
                blank = 0 as ::core::ffi::c_int;
            }
            cmdq_print(item_handle, |out| {
                let owner = (*term).client.upgrade().expect("terminal client");
                write!(out, "Terminal {}: ", (n) as u32)?;
                write_cstr(out, ((*term).name).as_ptr().cast_mut())?;
                out.write_all(b" for ")?;
                write_cstr(
                    out,
                    ((*owner.get()).name)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                )?;
                write!(out, ", flags=0x{:x}:", ((*term).flags) as u32)
            });
            n = n.wrapping_add(1);
            i = 0 as u_int;
            while i < tty_term_ncodes() {
                cmdq_print(item_handle, |out| {
                    out.write_all(tty_term_describe(term, i as tty_code_code).as_bytes())
                });
                i = i.wrapping_add(1);
            }
        }
        term = (*term).entry.le_next;
    }
    return (n != 0 as u_int) as ::core::ffi::c_int;
}
unsafe fn cmd_show_messages_exec(
    mut self_0: refbox::Weak<cmd>,
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
) -> cmd_retval {
    let mut args: *mut args =
        cmd_get_args_mut(self_0.get_mut_unchecked()).map_or(std::ptr::null_mut(), |args| args);
    let mut done: ::core::ffi::c_int = 0;
    let mut blank: ::core::ffi::c_int = 0;
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    blank = 0 as ::core::ffi::c_int;
    done = blank;
    if args_has(args, 'T' as i32 as u_char) != 0 {
        blank = cmd_show_messages_terminals(self_0.clone(), item_handle, blank);
        done = 1 as ::core::ffi::c_int;
    }
    if args_has(args, 'J' as i32 as u_char) != 0 {
        job_print_summary(item_handle, blank);
        done = 1 as ::core::ffi::c_int;
    }
    if done != 0 {
        return CMD_RETURN_NORMAL;
    }
    let mut ft_owner = format_create_from_target(item_handle);
    ft = &raw mut *ft_owner;
    for msg in message_log.iter_rev() {
        format_add(
            ft,
            b"message_text\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write_cstr(out, msg.msg.as_ptr()),
        );
        format_add(
            ft,
            b"message_number\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write!(out, "{}", (msg.msg_num) as u32),
        );
        let mut msg_time = msg.msg_time;
        format_add_tv(
            ft,
            b"message_time\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut msg_time,
        );
        let s = format_expand_cstring(ft, SHOW_MESSAGES_TEMPLATE.as_ptr());
        cmdq_print(item_handle, |out| write_cstr(out, s.as_ptr()));
    }
    format_free(ft_owner);
    return CMD_RETURN_NORMAL;
}
