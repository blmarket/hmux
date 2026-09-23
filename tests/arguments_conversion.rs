use std::ffi::{CStr, CString};
use std::mem::size_of;
use std::ptr;

use hmux2::src::arguments::{
    args_copy, args_create, args_escape, args_first_value, args_free, args_free_values, args_has,
    args_next_value, args_parse as parse_args, args_percentage_result, args_print, args_set,
    args_string, args_string_percentage_and_expand_result, args_strtonum,
    args_strtonum_and_expand_result, args_strtonum_result, parse_number, parse_percentage,
    ArgumentValueError,
};
use hmux2::src::cmd::{cmd_list_new, cmd_list_print};
use hmux2::src::cmd_queue::{cmdq_free_state, cmdq_get_callback1};
use hmux2::src::ffi::libc::free;
use hmux2::src::shared::arguments::{
    args, args_parse, args_value, ARGS_COMMANDS, ARGS_PARSE_COMMANDS, ARGS_STRING,
};
use hmux2::src::xmalloc::{xcalloc, xstrdup};

fn cstring(value: &str) -> CString {
    CString::new(value).expect("test input contains no NUL")
}

#[test]
fn printing_options_uses_formatted_flag_and_string_fragments() {
    unsafe {
        let args = args_create();
        args_set(args, b'v', ptr::null_mut(), 0);
        args_set(args, b'n', string_value(c"hello"), 0);
        let printed = args_print(args);
        assert_eq!(CStr::from_ptr(printed).to_bytes(), b"-v -n hello");
        free(printed.cast());
        args_free(args);
    }
}

#[test]
fn exported_argument_escape_keeps_c_owned_byte_output() {
    unsafe {
        for (input, expected) in [
            (b"\xff".as_slice(), b"\\377".as_slice()),
            (b"\"".as_slice(), b"\\\"".as_slice()),
        ] {
            let input = CString::new(input).unwrap();
            let escaped = args_escape(input.as_ptr());
            assert_eq!(CStr::from_ptr(escaped).to_bytes(), expected);
            free(escaped.cast());
        }
    }
}

fn error_message(error: ArgumentValueError) -> &'static [u8] {
    error.message().to_bytes()
}

unsafe fn string_value(value: &CStr) -> *mut args_value {
    let result = Box::into_raw(Box::new(std::mem::zeroed::<args_value>()));
    (*result).type_0 = ARGS_STRING;
    (*result).c2rust_unnamed.string = xstrdup(value.as_ptr());
    result
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
        args_set(args, b'n', string_value(number.as_c_str()), 0);
        assert_eq!(
            args_strtonum_and_expand_result(args, b'n', 0, 20, item),
            Ok(17)
        );
        args_set(args, b'n', string_value(malformed.as_c_str()), 0);
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
        cmdq_free_state((*item).state);
        free((*item).name.cast());
        drop(Box::from_raw(item));
    }
}

#[test]
fn repeated_flags_keep_order_and_numeric_helpers_use_the_last_value() {
    unsafe {
        let args = args_create();
        let first_text = cstring("12");
        let second_text = cstring("34");
        args_set(args, b'v', string_value(first_text.as_c_str()), 0);
        args_set(args, b'v', string_value(second_text.as_c_str()), 0);

        assert_eq!(args_has(args, b'v'), 2);
        let first = args_first_value(args, b'v');
        let second = args_next_value(first);
        assert_eq!(
            CStr::from_ptr((*first).c2rust_unnamed.string).to_bytes(),
            b"12"
        );
        assert_eq!(
            CStr::from_ptr((*second).c2rust_unnamed.string).to_bytes(),
            b"34"
        );
        assert_eq!(args_strtonum_result(args, b'v', 0, 100), Ok(34));
        assert_eq!(args_percentage_result(args, b'v', 0, 100, 100), Ok(34));
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
        args_set(missing, b'p', ptr::null_mut(), 0);
        assert_eq!(
            args_percentage_result(missing, b'p', 0, 50, 50),
            Err(ArgumentValueError::Empty)
        );
        args_free(missing);

        let args = args_create();
        let value = cstring("42");
        args_set(args, b'n', string_value(value.as_c_str()), 0);

        let mut cause = ptr::null_mut();
        assert_eq!(args_strtonum(args, b'n', 0, 50, &mut cause), 42);
        assert!(cause.is_null());

        assert_eq!(args_strtonum(args, b'n', 0, 40, &mut cause), 0);
        assert_eq!(CStr::from_ptr(cause).to_bytes(), b"too large");
        free(cause.cast());
        args_free(args);
    }
}

#[test]
fn command_values_and_cached_strings_keep_their_storage_ownership() {
    unsafe {
        let args = args_create();
        let value = Box::into_raw(Box::new(std::mem::zeroed::<args_value>()));
        (*value).type_0 = ARGS_COMMANDS;
        (*value).c2rust_unnamed.cmdlist = cmd_list_new();
        args_set(args, b'c', value, 0);

        assert_eq!(
            args_strtonum_result(args, b'c', 0, 100),
            Err(ArgumentValueError::Missing)
        );
        assert_eq!(
            args_percentage_result(args, b'c', 0, 100, 100),
            Err(ArgumentValueError::Missing)
        );

        let rendered = cmd_list_print((*value).c2rust_unnamed.cmdlist, 0);
        assert_eq!(CStr::from_ptr(rendered).to_bytes(), b"");
        free(rendered.cast());
        args_free(args);

        let args = args_create();
        (*args).count = 1;
        (*args).values = xcalloc(1, size_of::<args_value>()) as *mut args_value;
        (*(*args).values).type_0 = ARGS_COMMANDS;
        (*(*args).values).c2rust_unnamed.cmdlist = cmd_list_new();
        let first = args_string(args, 0);
        let second = args_string(args, 0);
        assert_eq!(first, second);
        assert_eq!(CStr::from_ptr(first).to_bytes(), b"");
        args_free(args);
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
        let values = xcalloc(3, size_of::<args_value>()) as *mut args_value;
        (*values).type_0 = ARGS_STRING;
        (*values).c2rust_unnamed.string = xstrdup(c"command".as_ptr());
        for index in 1..3 {
            let value = values.add(index);
            (*value).type_0 = ARGS_COMMANDS;
            (*value).c2rust_unnamed.cmdlist = cmd_list_new();
        }
        let spec = args_parse {
            template: c"".as_ptr(),
            lower: 0,
            upper: -1,
            cb: Some(command_argument),
        };
        let mut cause = ptr::null_mut();
        let args = parse_args(&spec, values, 3, &mut cause);
        assert!(!args.is_null());
        assert!(cause.is_null());
        args_free_values(values, 3);
        free(values.cast());

        let first = (*(*args).values).cached as *const core::ffi::c_char;
        assert!(!first.is_null());
        assert_eq!(CStr::from_ptr(first).to_bytes(), b"");
        assert_eq!(args_string(args, 0), first);
        let second = args_string(args, 1);
        assert_ne!(first, second);

        let copied = args_copy(args, 0, ptr::null_mut());
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
fn rejected_command_argument_keeps_source_value_ownership() {
    unsafe {
        let values = xcalloc(2, size_of::<args_value>()) as *mut args_value;
        (*values).type_0 = ARGS_STRING;
        (*values).c2rust_unnamed.string = xstrdup(c"command".as_ptr());
        let command_value = values.add(1);
        (*command_value).type_0 = ARGS_COMMANDS;
        (*command_value).c2rust_unnamed.cmdlist = cmd_list_new();
        let spec = args_parse {
            template: c"".as_ptr(),
            lower: 0,
            upper: -1,
            cb: None,
        };
        let mut cause = ptr::null_mut();
        assert!(parse_args(&spec, values, 2, &mut cause).is_null());
        assert_eq!(
            CStr::from_ptr(cause).to_bytes(),
            b"argument 1 must be \"string\""
        );
        free(cause.cast());
        args_free_values(values, 2);
        free(values.cast());
    }
}
