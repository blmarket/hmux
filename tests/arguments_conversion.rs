use std::ffi::{CStr, CString};
use std::ptr;

use hmux2::src::arguments::{
    args_copy, args_create, args_escape, args_first_value, args_free, args_get, args_has,
    args_next_value, args_parse as parse_args, args_percentage_result, args_print,
    args_push_positional_commands, args_push_positional_string, args_set_flag,
    args_set_owned_commands, args_set_owned_string, args_string,
    args_string_percentage_and_expand_result, args_strtonum_and_expand_result,
    args_strtonum_result, parse_number, parse_percentage, ArgumentValueError,
};
use hmux2::src::cmd::{cmd_list_free, cmd_list_new, cmd_list_print};
use hmux2::src::cmd_queue::{
    cmdq_free_detached, cmdq_get_callback1, cmdq_get_error, cmdq_get_name,
};
use hmux2::src::ffi::libc::snprintf;
use hmux2::src::shared::arguments::{
    args, args_parse, args_value, ARGS_COMMANDS, ARGS_PARSE_COMMANDS, ARGS_STRING,
};

fn cstring(value: &str) -> CString {
    CString::new(value).expect("test input contains no NUL")
}

#[test]
fn printing_options_uses_formatted_flag_and_string_fragments() {
    unsafe {
        let args = args_create();
        args_set_flag(args, b'v', 0);
        args_set_owned_string(args, b'n', c"hello".to_owned(), 0);
        let printed = args_print(args);
        assert_eq!(printed.as_bytes(), b"-v -n hello");
        args_free(args);
    }
}

#[test]
fn argument_escape_keeps_byte_output() {
    unsafe {
        for (input, expected) in [
            (b"\xff".as_slice(), b"\\377".as_slice()),
            (b"\"".as_slice(), b"\\\"".as_slice()),
        ] {
            let input = CString::new(input).unwrap();
            let escaped = args_escape(input.as_ptr());
            assert_eq!(escaped.as_bytes(), expected);
        }
    }
}

fn error_message(error: ArgumentValueError) -> &'static [u8] {
    error.message().to_bytes()
}

unsafe fn borrowed_string_value(value: &CStr) -> args_value {
    args_value::borrowed_string(value.as_ptr())
}

unsafe fn borrowed_commands_value(
    cmdlist: *mut hmux2::src::shared::command::cmd_list,
) -> args_value {
    args_value::borrowed_commands(cmdlist)
}

#[test]
fn number_helper_preserves_syntax_bounds_and_diagnostics() {
    let minimum = cstring("-9223372036854775808");
    let maximum = cstring("9223372036854775807");
    let below = cstring("-9223372036854775809");
    let above = cstring("9223372036854775808");
    let signed = cstring("  +17");
    let malformed = cstring("17x");
    let empty = cstring("");

    assert_eq!(
        parse_number(minimum.as_c_str(), i64::MIN, i64::MAX),
        Ok(i64::MIN)
    );
    assert_eq!(
        parse_number(maximum.as_c_str(), i64::MIN, i64::MAX),
        Ok(i64::MAX)
    );
    assert_eq!(
        parse_number(below.as_c_str(), i64::MIN, i64::MAX),
        Err(ArgumentValueError::TooSmall)
    );
    assert_eq!(
        parse_number(above.as_c_str(), i64::MIN, i64::MAX),
        Err(ArgumentValueError::TooLarge)
    );
    assert_eq!(parse_number(signed.as_c_str(), 0, 20), Ok(17));
    assert_eq!(
        parse_number(malformed.as_c_str(), 0, 20),
        Err(ArgumentValueError::Invalid)
    );
    assert_eq!(
        parse_number(empty.as_c_str(), 0, 20),
        Err(ArgumentValueError::Invalid)
    );

    assert_eq!(error_message(ArgumentValueError::TooSmall), b"too small");
    assert_eq!(error_message(ArgumentValueError::TooLarge), b"too large");
    assert_eq!(error_message(ArgumentValueError::Invalid), b"invalid");
}

#[test]
fn percentage_helper_preserves_literal_and_computed_bounds() {
    let zero = cstring("0%");
    let full = cstring("100%");
    let maximum = cstring("1000%");
    let percentage_too_large = cstring("1001%");
    let computed_too_large = cstring("101%");
    let computed_too_small = cstring("0%");
    let malformed = cstring("bad%");
    let empty = cstring("");

    assert_eq!(parse_percentage(zero.as_c_str(), 0, 100, 80), Ok(0));
    assert_eq!(parse_percentage(full.as_c_str(), 0, 100, 80), Ok(80));
    assert_eq!(parse_percentage(maximum.as_c_str(), 0, 100, 10), Ok(100));
    assert_eq!(
        parse_percentage(percentage_too_large.as_c_str(), 0, 100, 10),
        Err(ArgumentValueError::TooLarge)
    );
    assert_eq!(
        parse_percentage(computed_too_large.as_c_str(), 0, 100, 100),
        Err(ArgumentValueError::TooLarge)
    );
    assert_eq!(
        parse_percentage(computed_too_small.as_c_str(), 1, 100, 100),
        Err(ArgumentValueError::TooSmall)
    );
    assert_eq!(
        parse_percentage(malformed.as_c_str(), 0, 100, 100),
        Err(ArgumentValueError::Invalid)
    );
    assert_eq!(
        parse_percentage(empty.as_c_str(), 0, 100, 100),
        Err(ArgumentValueError::Empty)
    );
    assert_eq!(error_message(ArgumentValueError::Empty), b"empty");
}

#[test]
fn expanded_helpers_convert_formatted_values_and_keep_errors_typed() {
    unsafe {
        let item_name = cstring("arguments-conversion");
        let item = cmdq_get_callback1(item_name.as_ptr(), None, ptr::null_mut());
        let number = cstring("17");
        let malformed = cstring("17x");
        let percentage = cstring("1000%");
        let empty = cstring("");

        let args = hmux2::src::arguments::args_create();
        args_set_owned_string(args, b'n', number.clone(), 0);
        assert_eq!(
            args_strtonum_and_expand_result(args, b'n', 0, 20, item),
            Ok(17)
        );
        args_set_owned_string(args, b'n', malformed.clone(), 0);
        assert_eq!(
            args_strtonum_and_expand_result(args, b'n', 0, 20, item),
            Err(ArgumentValueError::Invalid)
        );
        args_free(args);

        assert_eq!(
            args_string_percentage_and_expand_result(percentage.as_ptr(), 0, 1000, 1, item),
            Ok(10)
        );
        assert_eq!(
            args_string_percentage_and_expand_result(empty.as_ptr(), 0, 1000, 1, item),
            Err(ArgumentValueError::Invalid)
        );
        cmdq_free_detached(item);
    }
}

#[test]
fn callback_queue_name_keeps_non_utf8_label_and_item_pointer() {
    unsafe {
        let label = CString::new(b"raw\xff".as_slice()).unwrap();
        let item = cmdq_get_callback1(label.as_ptr(), None, ptr::null_mut());
        assert!(!item.is_null());

        let mut expected = [0_i8; 128];
        let written = snprintf(
            expected.as_mut_ptr(),
            expected.len(),
            c"[%s/%p]".as_ptr(),
            label.as_ptr(),
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

#[test]
fn detached_error_callback_keeps_its_message_after_input_changes() {
    unsafe {
        let mut source = b"raw\xff error\0".to_vec();
        let item = cmdq_get_error(source.as_ptr().cast());
        source.fill(0);

        assert_eq!(
            CStr::from_ptr((*item).data.cast()).to_bytes(),
            b"raw\xff error"
        );
        cmdq_free_detached(item);
    }
}

#[test]
fn repeated_flags_keep_order_and_numeric_helpers_use_the_last_value() {
    unsafe {
        let args = args_create();
        let first_text = cstring("12");
        let second_text = cstring("34");
        args_set_owned_string(args, b'v', first_text.clone(), 0);
        args_set_owned_string(args, b'v', second_text.clone(), 0);

        assert_eq!(args_has(args, b'v'), 2);
        let first = args_first_value(args, b'v');
        let second = args_next_value(first);
        assert_eq!(
            CStr::from_ptr((*first).string_ptr()).to_bytes(),
            b"12"
        );
        assert_eq!(
            CStr::from_ptr((*second).string_ptr()).to_bytes(),
            b"34"
        );
        assert_eq!(args_strtonum_result(args, b'v', 0, 100), Ok(34));
        assert_eq!(args_percentage_result(args, b'v', 0, 100, 100), Ok(34));
        args_free(args);
    }
}

#[test]
fn flag_value_collection_keeps_addresses_stable_and_returns_the_last_value() {
    unsafe {
        let args = args_create();
        let first_text = cstring("first");
        args_set_owned_string(args, b'n', first_text.clone(), 0);
        let first = args_first_value(args, b'n');

        for index in 1..512 {
            let text = CString::new(index.to_string()).unwrap();
            args_set_owned_string(args, b'n', text, 0);
        }

        assert_eq!(args_first_value(args, b'n'), first);
        assert_eq!(
            CStr::from_ptr((*first).string_ptr()).to_bytes(),
            b"first"
        );

        let mut value = first;
        for _ in 0..512 {
            assert!(!value.is_null());
            value = args_next_value(value);
        }
        assert!(value.is_null());
        assert_eq!(CStr::from_ptr(args_get(args, b'n')).to_bytes(), b"511");
        args_free(args);
    }
}

#[test]
fn c_boundary_translates_typed_errors_without_changing_success_values() {
    unsafe {
        let missing = args_create();
        assert_eq!(
            args_strtonum_result(missing, b'n', 0, 50),
            Err(ArgumentValueError::Missing)
        );
        args_set_flag(missing, b'p', 0);
        assert_eq!(
            args_percentage_result(missing, b'p', 0, 50, 50),
            Err(ArgumentValueError::Empty)
        );
        args_free(missing);

        let args = args_create();
        let value = cstring("42");
        args_set_owned_string(args, b'n', value.clone(), 0);

        assert_eq!(args_strtonum_result(args, b'n', 0, 50), Ok(42));
        assert_eq!(
            args_strtonum_result(args, b'n', 0, 40),
            Err(ArgumentValueError::TooLarge)
        );
        args_free(args);
    }
}

#[test]
fn command_values_and_cached_strings_keep_their_storage_ownership() {
    unsafe {
        let args = args_create();
        let cmdlist = cmd_list_new();
        args_set_owned_commands(args, b'c', cmdlist, 0);
        let value = args_first_value(args, b'c');

        assert_eq!(
            args_strtonum_result(args, b'c', 0, 100),
            Err(ArgumentValueError::Missing)
        );
        assert_eq!(
            args_percentage_result(args, b'c', 0, 100, 100),
            Err(ArgumentValueError::Missing)
        );

        let rendered = cmd_list_print((*value).cmdlist(), 0);
        assert_eq!(rendered.as_bytes(), b"");
        args_free(args);

        let args = args_create();
        args_push_positional_commands(args, cmd_list_new());
        let first = args_string(args, 0);
        let second = args_string(args, 0);
        assert_eq!(first, second);
        assert_eq!(CStr::from_ptr(first).to_bytes(), b"");
        args_free(args);
    }
}

#[test]
fn borrowed_parser_command_retains_only_while_stored() {
    unsafe extern "C" fn commands(
        _: *mut args,
        _: core::ffi::c_uint,
        _: *mut *mut core::ffi::c_char,
    ) -> core::ffi::c_uint {
        ARGS_PARSE_COMMANDS
    }

    unsafe {
        let cmdlist = cmd_list_new();
        let mut values = [
            args_value::borrowed_string(c"command".as_ptr()),
            args_value::borrowed_commands(cmdlist),
        ];
        let mut spec = args_parse {
            template: c"".as_ptr(),
            lower: 1,
            upper: -1,
            cb: Some(commands),
        };
        let stored = parse_args(&spec, values.as_mut_ptr(), 2).expect("valid command argument");
        assert_eq!((*cmdlist).references, 2);
        args_free(stored);
        assert_eq!((*cmdlist).references, 1);

        spec.lower = 2;
        assert!(parse_args(&spec, values.as_mut_ptr(), 2).is_err());
        assert_eq!((*cmdlist).references, 1);
        cmd_list_free(cmdlist);
    }
}

#[test]
fn positional_command_cache_survives_array_growth_and_copy() {
    unsafe extern "C" fn command_argument(
        args: *mut args,
        count: core::ffi::c_uint,
        _: *mut *mut core::ffi::c_char,
    ) -> core::ffi::c_uint {
        if count == 1 {
            args_string(args, 0);
        }
        ARGS_PARSE_COMMANDS
    }

    unsafe {
        let command = cstring("command");
        let command_lists = [cmd_list_new(), cmd_list_new()];
        let mut values = [
            borrowed_string_value(command.as_c_str()),
            borrowed_commands_value(command_lists[0]),
            borrowed_commands_value(command_lists[1]),
        ];
        let spec = args_parse {
            template: c"".as_ptr(),
            lower: 0,
            upper: -1,
            cb: Some(command_argument),
        };
        let args = parse_args(&spec, values.as_mut_ptr(), values.len() as u32)
            .expect("valid command arguments");
        for cmdlist in command_lists {
            cmd_list_free(cmdlist);
        }

        let first = (&(*args).values)[0].cached.as_ref().unwrap().as_ptr();
        assert!(!first.is_null());
        assert_eq!(CStr::from_ptr(first).to_bytes(), b"");
        assert_eq!(args_string(args, 0), first);
        let second = args_string(args, 1);
        assert_ne!(first, second);

        let copied = args_copy(args, &Vec::new());
        let copied_first = args_string(copied, 0);
        let copied_second = args_string(copied, 1);
        assert_ne!(copied_first, first);
        assert_ne!(copied_second, second);
        assert_eq!(CStr::from_ptr(copied_first).to_bytes(), b"");
        assert_eq!(CStr::from_ptr(copied_second).to_bytes(), b"");
        args_free(args);
        assert_eq!(CStr::from_ptr(copied_first).to_bytes(), b"");
        args_free(copied);
    }
}

#[test]
fn copied_argument_templates_own_intermediate_and_final_strings() {
    unsafe {
        let source = args_create();
        args_set_owned_string(source, b'n', c"left-%1-right-%2".to_owned(), 0);
        args_set_owned_string(source, b'q', c"%%".to_owned(), 0);
        args_push_positional_string(source, c"%2:%1".to_owned());

        let argv = vec![c"A'B".to_owned(), c"X Y".to_owned()];
        let copied = args_copy(source, &argv);
        let source_named = args_first_value(source, b'n');
        let copied_named = args_first_value(copied, b'n');
        assert_ne!(
            (*source_named).string_ptr(),
            (*copied_named).string_ptr()
        );
        args_free(source);

        assert_eq!(
            CStr::from_ptr((*copied_named).string_ptr()).to_bytes(),
            b"left-A'B-right-X Y"
        );
        let quoted = args_first_value(copied, b'q');
        assert_eq!(
            CStr::from_ptr((*quoted).string_ptr()).to_bytes(),
            b"A'\\''B"
        );
        assert_eq!(
            CStr::from_ptr((&(*copied).values)[0].string_ptr()).to_bytes(),
            b"X Y:A'B"
        );
        args_free(copied);
    }
}

#[test]
fn rejected_command_argument_keeps_source_value_ownership() {
    unsafe {
        let command = cstring("command");
        let command_list = cmd_list_new();
        let mut values = [
            borrowed_string_value(command.as_c_str()),
            borrowed_commands_value(command_list),
        ];
        let spec = args_parse {
            template: c"".as_ptr(),
            lower: 0,
            upper: -1,
            cb: None,
        };
        let error = parse_args(&spec, values.as_mut_ptr(), values.len() as u32)
            .expect_err("command value must be rejected");
        let error = match error {
            hmux2::src::arguments::ArgsParseError::Message(error) => error,
            hmux2::src::arguments::ArgsParseError::Usage => panic!("expected a diagnostic"),
        };
        assert_eq!(error.to_bytes(), b"argument 1 must be \"string\"");
        cmd_list_free(command_list);
    }
}
