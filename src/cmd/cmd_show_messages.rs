use crate::arguments::args_has;
use crate::cmd::cmd_get_args;

pub use crate::consts::{
    CMD_AFTERHOOK, CMD_CLIENT_CANFAIL, CMD_CLIENT_TFLAG, CMD_FIND_PANE, CMD_RETURN_NORMAL,
};
use crate::fmt_args;
use crate::format::{format_add, format_add_tv, format_create_from_target, format_expand};
use crate::job::job_print_summary;
use crate::message_log::{MessageLogStore, with_message_log};
use crate::terminfo::tty_term_snapshots_for_client;
pub use crate::types::{
    RustCommandEntry, args, args_parse_t, cmd, cmd_entry_flag, cmd_retval, cmdq_item, u_char, u_int,
};

pub const SHOW_MESSAGES_TEMPLATE: &core::ffi::CStr = c"#{t/p:message_time}: #{message_text}";
pub(crate) static cmd_show_messages_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"show-messages",
        alias: Some(c"showmsgs"),
        args: args_parse_t {
            template: c"JTt:",
            lower: 0 as core::ffi::c_int,
            upper: 0 as core::ffi::c_int,
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
        exec: cmd_show_messages_exec,
    }
};
unsafe fn cmd_show_messages_terminals(
    self_0: &cmd,
    item: &cmdq_item,
    mut blank: core::ffi::c_int,
) -> core::ffi::c_int {
    unsafe {
        let args: &args = cmd_get_args(self_0);
        let target_client = item.target_client();
        let mut n: u_int;
        n = 0 as u_int;
        let target = (args_has(args, b't') != 0)
            .then_some(target_client.as_ref())
            .flatten();
        for terminal in tty_term_snapshots_for_client(target) {
            if blank != 0 {
                item.print(c"%s", fmt_args![c""]);
                blank = 0 as core::ffi::c_int;
            }
            item.print(
                c"Terminal %u: %s for %s, flags=0x%x:",
                fmt_args![
                    n,
                    terminal.name.as_c_str(),
                    terminal.client_name.as_deref(),
                    terminal.flags
                ],
            );
            n = n.wrapping_add(1);
            for capability in terminal.capabilities {
                item.print(c"%s", fmt_args![capability.as_c_str()]);
            }
        }
        (n != 0 as u_int) as core::ffi::c_int
    }
}
unsafe fn cmd_show_messages_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args: &args = cmd_get_args(self_0);
    let mut done: core::ffi::c_int;
    let mut blank: core::ffi::c_int;
    blank = 0 as core::ffi::c_int;
    done = blank;
    if args_has(args, 'T' as i32 as u_char) != 0 {
        unsafe { blank = cmd_show_messages_terminals(self_0, item, blank) };
        done = 1 as core::ffi::c_int;
    }
    if args_has(args, 'J' as i32 as u_char) != 0 {
        unsafe { job_print_summary(item, blank) };
        done = 1 as core::ffi::c_int;
    }
    if done != 0 {
        return CMD_RETURN_NORMAL;
    }
    let messages = with_message_log(|log| {
        log.entries()
            .map(|entry| (entry.text.to_owned(), entry.number, entry.time))
            .collect::<Vec<_>>()
    });
    let mut ft = unsafe { format_create_from_target(item) };
    for (text, number, time) in messages {
        format_add(&mut ft, c"message_text", c"%s", fmt_args![text.as_c_str()]);
        format_add(&mut ft, c"message_number", c"%u", fmt_args![number]);
        format_add_tv(&mut ft, c"message_time", &time.as_timeval());
        let s = unsafe { format_expand(&mut ft, SHOW_MESSAGES_TEMPLATE) };
        unsafe { item.print(c"%s", fmt_args![s.as_c_str()]) };
    }
    CMD_RETURN_NORMAL
}
