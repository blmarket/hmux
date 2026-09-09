use crate::arguments::{args_count, args_has, args_string_str};
use crate::cmd::cmd_get_args;

pub use crate::consts::{
    CMD_AFTERHOOK, CMD_FIND_PANE, CMD_RETURN_ERROR, CMD_RETURN_NORMAL, FORMAT_NONE,
};
use crate::fmt_args;
use crate::format::{format_create_for_client, format_defaults_for_handles, format_expand_time};
pub use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval, cmdq_item};
pub use crate::types::args_parse_t;

pub(crate) static cmd_pipe_pane_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"pipe-pane",
        alias: Some(c"pipep"),
        args: args_parse_t {
            template: c"IOot:",
            lower: 0 as core::ffi::c_int,
            upper: 1 as core::ffi::c_int,
            cb: None,
        },
        usage: c"[-IOo] [-t target-pane] [shell-command]",
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
        exec: cmd_pipe_pane_exec,
    }
};
unsafe fn cmd_pipe_pane_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    unsafe {
        let args = cmd_get_args(self_0);
        let tc = item.target_client();
        let target = &item.target;
        let session = target.session().expect("pipe target session");
        let link = target.winlink_ref().expect("pipe target link");
        let pane = target.pane_ref().expect("pipe target pane is present");
        if pane.has_exited() != Some(false) {
            item.error(c"target pane has exited", fmt_args![]);
            return CMD_RETURN_ERROR;
        }
        let Some(closed) = pane.close_pipe() else {
            return CMD_RETURN_NORMAL;
        };
        if closed.removed
            || args_count(args) == 0
            || args_string_str(args, 0).is_none_or(|value| value.is_empty())
        {
            return CMD_RETURN_NORMAL;
        }
        if args_has(args, b'o') != 0 && closed.was_open {
            return CMD_RETURN_NORMAL;
        }
        let (input, output) = if args_has(args, b'I') != 0 {
            (1, args_has(args, b'O'))
        } else {
            (0, 1)
        };
        let pair = match crate::pane_handle::PanePipePair::open() {
            Ok(pair) => pair,
            Err(cause) => {
                item.error(c"%s", fmt_args![cause.as_c_str()]);
                return CMD_RETURN_ERROR;
            }
        };
        let mut ft = format_create_for_client(item.client().as_ref(), Some(item), FORMAT_NONE, 0);
        format_defaults_for_handles(
            &mut ft,
            tc.as_ref(),
            Some(&session),
            Some(&link),
            Some(&pane),
        );
        let command = format_expand_time(
            &mut ft,
            args_string_str(args, 0).expect("argument count checked"),
        );
        match pane.connect_pipe(pair, &command, input, output) {
            Ok(()) => CMD_RETURN_NORMAL,
            Err(cause) => {
                item.error(c"%s", fmt_args![cause.as_c_str()]);
                CMD_RETURN_ERROR
            }
        }
    }
}
