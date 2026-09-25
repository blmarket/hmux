use crate::src::cfg::{cfg_finished, cfg_test_take_causes, CFG_TEST_LOCK};
use crate::src::cmd::parse::cmd_parse_from_string;
use crate::src::cmd::queue::{cmdq_free_detached, cmdq_get_command, cmdq_get_name};
use crate::src::cmd::{cmd_list_free, cmdq_item};
use crate::src::ffi::libc::snprintf;
use crate::src::options::{options_create, options_free};
use crate::src::shared::command::{cmd_retval, CMD_PARSE_SUCCESS, CMD_RETURN_ERROR};
use crate::src::shared::options::options;
use crate::src::tmux::{global_options, global_s_options, global_w_options};
use std::ffi::{CStr, CString};

struct OptionGlobalsGuard {
    previous_global_options: *mut options,
    previous_global_s_options: *mut options,
    previous_global_w_options: *mut options,
    previous_cfg_finished: ::core::ffi::c_int,
    created_global_options: bool,
    created_global_s_options: bool,
    created_global_w_options: bool,
}

impl OptionGlobalsGuard {
    unsafe fn new() -> Self {
        let mut guard = Self {
            previous_global_options: global_options,
            previous_global_s_options: global_s_options,
            previous_global_w_options: global_w_options,
            previous_cfg_finished: cfg_finished,
            created_global_options: false,
            created_global_s_options: false,
            created_global_w_options: false,
        };
        if global_options.is_null() {
            global_options = options_create(std::ptr::null_mut());
            guard.created_global_options = true;
        }
        if global_s_options.is_null() {
            global_s_options = options_create(std::ptr::null_mut());
            guard.created_global_s_options = true;
        }
        if global_w_options.is_null() {
            global_w_options = options_create(std::ptr::null_mut());
            guard.created_global_w_options = true;
        }
        guard
    }
}

impl Drop for OptionGlobalsGuard {
    fn drop(&mut self) {
        unsafe {
            if self.created_global_options {
                options_free(global_options);
            }
            if self.created_global_s_options {
                options_free(global_s_options);
            }
            if self.created_global_w_options {
                options_free(global_w_options);
            }
            global_options = self.previous_global_options;
            global_s_options = self.previous_global_s_options;
            global_w_options = self.previous_global_w_options;
            cfg_finished = self.previous_cfg_finished;
        }
    }
}

unsafe fn run_option_command(command: &str) -> (cmd_retval, Vec<Vec<u8>>) {
    let _globals = OptionGlobalsGuard::new();
    let _ = cfg_test_take_causes();
    cfg_finished = 0;

    let command = CString::new(command).expect("command has no embedded NUL");
    let parsed = cmd_parse_from_string(command.as_c_str(), std::ptr::null_mut());
    assert_eq!(parsed.status, CMD_PARSE_SUCCESS, "command={command:?}");

    let command_list = parsed.cmdlist;
    let item: *mut cmdq_item = cmdq_get_command(command_list, std::ptr::null_mut());
    assert!(!item.is_null(), "command={command:?}");
    cmd_list_free(command_list);

    let cmd = (*item).cmd;
    let exec = (*(*cmd).entry)
        .exec
        .expect("option command has an execution callback");
    let retval = exec(cmd, item);
    let causes = cfg_test_take_causes();

    cmdq_free_detached(item);
    (retval, causes)
}

#[test]
fn command_queue_name_keeps_entry_label_and_item_pointer() {
    let _guard = CFG_TEST_LOCK.lock().unwrap();
    unsafe {
        let _globals = OptionGlobalsGuard::new();
        let command = c"set-option status on";
        let parsed = cmd_parse_from_string(command, std::ptr::null_mut());
        assert_eq!(parsed.status, CMD_PARSE_SUCCESS);

        let command_list = parsed.cmdlist;
        let item = cmdq_get_command(command_list, std::ptr::null_mut());
        assert!(!item.is_null());
        cmd_list_free(command_list);

        let mut expected = [0_i8; 128];
        let written = snprintf(
            expected.as_mut_ptr(),
            expected.len(),
            c"[%s/%p]".as_ptr(),
            c"set-option".as_ptr(),
            item.cast::<::core::ffi::c_void>(),
        );
        assert!(written >= 0 && (written as usize) < expected.len());
        assert_eq!(
            CStr::from_ptr(cmdq_get_name(item)).to_bytes(),
            CStr::from_ptr(expected.as_ptr()).to_bytes()
        );
        cmdq_free_detached(item);
    }
}

fn assert_error_cases(command: &str, cases: &[(&str, &[u8])]) {
    for &(argument, expected) in cases {
        let command_line = format!("{command} {argument}");
        let (retval, causes) = unsafe { run_option_command(&command_line) };
        assert_eq!(retval, CMD_RETURN_ERROR, "command={command_line:?}");
        assert_eq!(causes, vec![expected.to_vec()], "command={command_line:?}");
    }
}

#[test]
fn set_option_reports_match_errors_through_cmdq_error() {
    let _guard = CFG_TEST_LOCK.lock().unwrap();
    assert_error_cases(
        "set-option",
        &[
            ("status-", b"ambiguous option: status-"),
            ("status[]", b"invalid option: status[]"),
            (
                "status-format[4294967296]",
                b"invalid option: status-format[4294967296]",
            ),
            ("not-an-option", b"invalid option: not-an-option"),
        ],
    );
}

#[test]
fn show_options_reports_match_errors_through_cmdq_error() {
    let _guard = CFG_TEST_LOCK.lock().unwrap();
    assert_error_cases(
        "show-options",
        &[
            ("status-", b"ambiguous option: status-"),
            ("status[]", b"invalid option: status[]"),
            (
                "status-format[4294967296]",
                b"invalid option: status-format[4294967296]",
            ),
            ("not-an-option", b"invalid option: not-an-option"),
        ],
    );
}
