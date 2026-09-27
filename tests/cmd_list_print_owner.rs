use hmux2::src::arguments::{
    args_create, args_print, args_push_positional_commands, args_set_flag, args_set_owned_commands,
    args_set_owned_string, args_to_vector, ARGS_ENTRY_OPTIONAL_VALUE,
};
use hmux2::src::cmd::{
    cmd, cmd_list_append, cmd_list_append_all, cmd_list_copy, cmd_list_move,
    cmd_list_new, cmd_list_print, cmd_parse, cmd_print, CMD_LIST_PRINT_ESCAPED,
    CMD_LIST_PRINT_NO_GROUPS,
};
use hmux2::src::shared::arguments::{args_value, ArgumentValue};
use hmux2::src::shared::rc;
use std::ffi::CString;

#[test]
fn argument_printer_preserves_flag_groups_values_and_optional_separator() {
    unsafe {
        let mut args = args_create();
        args_set_flag(&mut *args, b'a', 0);
        args_set_flag(&mut *args, b'a', 0);
        args_set_owned_string(&mut *args, CString::new("two words").unwrap());
        args_set_owned_string(&mut *args, CString::new("").unwrap());
        args_set_flag(&mut *args, b'z', ARGS_ENTRY_OPTIONAL_VALUE);
        assert_eq!(
            args_print(&args).as_bytes(),
            b"-aa -f \"two words\" -f '' -z --"
        );
        drop(args);
    }
}

#[test]
fn argument_printer_preserves_zero_and_non_utf8_flag_bytes() {
    unsafe {
        for (flag, expected) in [(0, b"-".as_slice()), (0xff, b"-\xff".as_slice())] {
            let mut args = args_create();
            args_set_flag(&mut *args, flag, 0);
            assert_eq!(args_print(&args).as_bytes(), expected);
            drop(args);
        }

        let mut args = args_create();
        args_set_flag(&mut *args, 0, ARGS_ENTRY_OPTIONAL_VALUE);
        assert_eq!(args_print(&args).as_bytes(), b"- --");
        drop(args);

        let mut args = args_create();
        args_set_owned_commands(&mut args, 0, cmd_list_new(), 0);
        assert_eq!(args_print(&args).as_bytes(), b"- {  }");
        drop(args);
    }
}

unsafe fn display_message_command() -> Box<cmd> {
    let value = ArgumentValue::borrowed_string(c"display-message");
    cmd_parse(std::slice::from_ref(&value), None, 0, 0).expect("command parse reported an error")
}

#[test]
fn command_printer_keeps_exported_c_buffer_and_empty_arguments() {
    unsafe {
        let command = display_message_command();
        let printed = cmd_print(&*command);
        assert_eq!(printed.as_bytes(), b"display-message");
        drop(command);
    }
}

#[test]
fn list_printer_preserves_empty_and_group_separator_bytes() {
    unsafe {
        let list = cmd_list_new();
        let empty = cmd_list_print(&*rc::as_ptr(&list), 0);
        assert_eq!(empty.as_bytes(), b"");

        for _ in 0..3 {
            let item = display_message_command();
            cmd_list_append(rc::as_ptr(&list), item);
        }
        let commands = &mut (*rc::as_ptr(&list)).list;
        commands[2].group = commands[1].group.wrapping_add(1);

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
            let printed = cmd_list_print(&*rc::as_ptr(&list), flags);
            assert_eq!(printed.as_bytes(), expected);
        }
        let mut args = args_create();
        args_push_positional_commands(&mut args, list);

        let printed = args_print(&args);
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
        drop(args);
    }
}

#[test]
fn list_splice_copy_and_refcount_keep_command_addresses_stable() {
    fn addresses(list: &hmux2::src::shared::command::cmd_list) -> Vec<usize> {
        list.list
            .iter()
            .map(|command| command.as_ref() as *const cmd as usize)
            .collect()
    }
    unsafe {
        let destination = cmd_list_new();
        let first = display_message_command();
        let first_address = first.as_ref() as *const cmd as usize;
        cmd_list_append(rc::as_ptr(&destination), first);
        let source = cmd_list_new();
        let second = display_message_command();
        let second_address = second.as_ref() as *const cmd as usize;
        cmd_list_append(rc::as_ptr(&source), second);
        cmd_list_append_all(rc::as_ptr(&destination), rc::as_ptr(&source));
        assert!((*rc::as_ptr(&source)).list.is_empty());
        assert_eq!(addresses(&*rc::as_ptr(&destination)), [first_address, second_address]);

        let tail = cmd_list_new();
        let third = display_message_command();
        let third_address = third.as_ref() as *const cmd as usize;
        cmd_list_append(rc::as_ptr(&tail), third);
        cmd_list_move(rc::as_ptr(&destination), rc::as_ptr(&tail));
        assert!((*rc::as_ptr(&tail)).list.is_empty());
        let original = vec![first_address, second_address, third_address];
        assert_eq!(addresses(&*rc::as_ptr(&destination)), original);
        // Self append/move remain valid no-ops for storage ownership.
        cmd_list_append_all(rc::as_ptr(&destination), rc::as_ptr(&destination));
        cmd_list_move(rc::as_ptr(&destination), rc::as_ptr(&destination));
        assert_eq!(addresses(&*rc::as_ptr(&destination)), original);

        let copied = cmd_list_copy(&*rc::as_ptr(&destination), &Vec::new());
        let copied_addresses = addresses(&*rc::as_ptr(&copied));
        assert_eq!(copied_addresses.len(), original.len());
        for address in &copied_addresses {
            assert!(!original.contains(address));
        }
        assert_eq!(
            cmd_list_print(&*rc::as_ptr(&copied), 0).as_bytes(),
            b"display-message ; display-message ;; display-message"
        );

        let retained = destination.clone();
        drop(retained);
        assert_eq!(addresses(&*rc::as_ptr(&destination)), original);
        drop(destination);
        drop(source);
        drop(tail);
        drop(copied);
    }
}

#[test]
fn queued_commands_keep_boxed_records_and_nested_arguments_alive() {
    use hmux2::src::cmd::queue::{cmdq_free_detached, cmdq_get_command};
    use hmux2::src::shared::rc;
    unsafe {
        let nested = cmd_list_new();
        let nested_observer = std::rc::Rc::downgrade(&nested);
        let values = vec![
            args_value::string(c"if-shell".to_owned()),
            args_value::string(c"-F".to_owned()),
            args_value::string(c"1".to_owned()),
            args_value::commands(nested),
        ];
        let command = cmd_parse(&values, None, 0, 0).unwrap();
        let address = command.as_ref() as *const cmd as usize;
        drop(values);
        assert_eq!(nested_observer.strong_count(), 1);
        let list = cmd_list_new();
        let list_observer = std::rc::Rc::downgrade(&list);
        cmd_list_append(rc::as_ptr(&list), command);
        // Grow the owning vector after insertion; pointee addresses stay stable.
        for _ in 0..32 {
            cmd_list_append(rc::as_ptr(&list), display_message_command());
        }
        let mut item = cmdq_get_command(&list, None);
        assert_eq!((*item).cmd as usize, address);
        drop(list);
        let mut count = 0;
        while !item.is_null() {
            let next = (*item).next;
            assert!(nested_observer.upgrade().is_some());
            assert!(!cmd_print(&*(*item).cmd).as_bytes().is_empty());
            cmdq_free_detached(item);
            item = next;
            count += 1;
        }
        assert_eq!(count, 33);
        assert!(list_observer.upgrade().is_none());
        assert!(nested_observer.upgrade().is_none());
    }
}
