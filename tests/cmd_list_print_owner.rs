use hmux2::src::arguments::{
    args_create, args_free, args_print, args_push_positional_commands, args_to_vector,
};
use hmux2::src::cmd::{
    cmd, cmd_free, cmd_list_append, cmd_list_append_all, cmd_list_copy, cmd_list_first,
    cmd_list_free, cmd_list_move, cmd_list_new, cmd_list_next, cmd_list_print, cmd_parse,
    cmd_print, CMD_LIST_PRINT_ESCAPED, CMD_LIST_PRINT_NO_GROUPS,
};
use hmux2::src::shared::arguments::args_value;

unsafe fn display_message_command() -> *mut cmd {
    let mut value = args_value::borrowed_string(c"display-message".as_ptr());
    cmd_parse(&mut value, 1, None, 0, 0).expect("command parse reported an error")
}

#[test]
fn command_printer_keeps_exported_c_buffer_and_empty_arguments() {
    unsafe {
        let command = display_message_command();
        let printed = cmd_print(&*command);
        assert_eq!(printed.as_bytes(), b"display-message");
        cmd_free(command);
    }
}

#[test]
fn list_printer_preserves_empty_and_group_separator_bytes() {
    unsafe {
        let list = cmd_list_new();
        let empty = cmd_list_print(&*list, 0);
        assert_eq!(empty.as_bytes(), b"");

        for _ in 0..3 {
            let item = display_message_command();
            cmd_list_append(list, item);
        }
        let first = cmd_list_first(list);
        let second = cmd_list_next(first);
        let third = cmd_list_next(second);
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
            let printed = cmd_list_print(&*list, flags);
            assert_eq!(printed.as_bytes(), expected);
        }
        let args = args_create();
        args_push_positional_commands(args, list);

        let printed = args_print(args);
        assert_eq!(
            printed.as_bytes(),
            b"{ display-message ; display-message ;; display-message }"
        );

        let argv = args_to_vector(&*args);
        assert_eq!(argv.len(), 1);
        assert_eq!(
            argv[0].as_bytes(),
            b"display-message ; display-message ;; display-message"
        );
        args_free(args);
    }
}

#[test]
fn list_splice_copy_and_refcount_keep_command_pointers_stable() {
    unsafe {
        let destination = cmd_list_new();
        let first = display_message_command();
        cmd_list_append(destination, first);

        let source = cmd_list_new();
        let second = display_message_command();
        cmd_list_append(source, second);
        cmd_list_append_all(destination, source);
        assert!(cmd_list_first(source).is_null());
        assert_eq!(cmd_list_first(destination), first);
        assert_eq!(cmd_list_next(first), second);

        let tail = cmd_list_new();
        let third = display_message_command();
        cmd_list_append(tail, third);
        cmd_list_move(destination, tail);
        assert!(cmd_list_first(tail).is_null());
        assert_eq!(cmd_list_next(second), third);
        assert!(cmd_list_next(third).is_null());

        let copied = cmd_list_copy(&*destination, &Vec::new());
        let copied_first = cmd_list_first(copied);
        let copied_second = cmd_list_next(copied_first);
        let copied_third = cmd_list_next(copied_second);
        assert!(!copied_first.is_null());
        assert_ne!(copied_first, first);
        assert_ne!(copied_second, second);
        assert_ne!(copied_third, third);
        assert!(cmd_list_next(copied_third).is_null());
        let printed = cmd_list_print(&*copied, 0);
        assert_eq!(
            printed.as_bytes(),
            b"display-message ; display-message ;; display-message"
        );

        (*destination).references += 1;
        cmd_list_free(destination);
        assert_eq!(cmd_list_first(destination), first);
        assert_eq!(cmd_list_next(second), third);
        cmd_list_free(destination);
        cmd_list_free(source);
        cmd_list_free(tail);
        cmd_list_free(copied);
    }
}
