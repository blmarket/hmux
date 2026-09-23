use std::ffi::CStr;
use std::mem::size_of;

use hmux2::src::arguments::{args_create, args_free, args_print, args_to_vector};
use hmux2::src::cmd::entries::display_message::cmd_display_message_entry;
use hmux2::src::cmd::{
    cmd, cmd_free_argv, cmd_list_append, cmd_list_new, cmd_list_print, CMD_LIST_PRINT_ESCAPED,
    CMD_LIST_PRINT_NO_GROUPS,
};
use hmux2::src::ffi::libc::free;
use hmux2::src::shared::arguments::{args_value, ARGS_COMMANDS};
use hmux2::src::xmalloc::xcalloc;

#[test]
fn list_printer_preserves_empty_and_group_separator_bytes() {
    unsafe {
        let list = cmd_list_new();
        let empty = cmd_list_print(list, 0);
        assert_eq!(CStr::from_ptr(empty).to_bytes(), b"");
        free(empty.cast());

        for _ in 0..3 {
            let item = xcalloc(1, size_of::<cmd>()) as *mut cmd;
            (*item).entry = &raw const cmd_display_message_entry;
            (*item).args = args_create();
            cmd_list_append(list, item);
        }
        let first = (*(*list).list).tqh_first;
        let second = (*first).qentry.tqe_next;
        let third = (*second).qentry.tqe_next;
        (*third).group = (*second).group.wrapping_add(1);

        let name = b"display-message";
        for (flags, separator1, separator2) in [
            (0, b" ; ".as_slice(), b" ;; ".as_slice()),
            (
                CMD_LIST_PRINT_ESCAPED,
                b" \\; ".as_slice(),
                b" \\;\\; ".as_slice(),
            ),
            (
                CMD_LIST_PRINT_NO_GROUPS,
                b" ; ".as_slice(),
                b" ; ".as_slice(),
            ),
            (
                CMD_LIST_PRINT_ESCAPED | CMD_LIST_PRINT_NO_GROUPS,
                b" \\; ".as_slice(),
                b" \\; ".as_slice(),
            ),
        ] {
            let mut expected = Vec::new();
            expected.extend_from_slice(name);
            expected.extend_from_slice(separator1);
            expected.extend_from_slice(name);
            expected.extend_from_slice(separator2);
            expected.extend_from_slice(name);
            let printed = cmd_list_print(list, flags);
            assert_eq!(CStr::from_ptr(printed).to_bytes(), expected);
            free(printed.cast());
        }
        let args = args_create();
        (*args).count = 1;
        (*args).values = xcalloc(1, size_of::<args_value>()) as *mut args_value;
        (*(*args).values).type_0 = ARGS_COMMANDS;
        (*(*args).values).c2rust_unnamed.cmdlist = list;

        let printed = args_print(args);
        assert_eq!(
            CStr::from_ptr(printed).to_bytes(),
            b"{ display-message ; display-message ;; display-message }"
        );
        free(printed.cast());

        let mut argc = 0;
        let mut argv = std::ptr::null_mut();
        args_to_vector(args, &mut argc, &mut argv);
        assert_eq!(argc, 1);
        assert_eq!(
            CStr::from_ptr(*argv).to_bytes(),
            b"display-message ; display-message ;; display-message"
        );
        cmd_free_argv(argc, argv);
        args_free(args);
    }
}
