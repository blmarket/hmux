use std::ffi::CStr;
use std::ptr;

use hmux2::src::cmd::{cmd_copy, cmd_free, cmd_parse};
use hmux2::src::shared::arguments::{args_value, ARGS_STRING};

unsafe fn parse_display_message(file: *const std::ffi::c_char) -> *mut hmux2::src::cmd::cmd {
    let command_name = c"display-message";
    let mut value: args_value = std::mem::zeroed();
    value.type_0 = ARGS_STRING;
    value.c2rust_unnamed.string = command_name.as_ptr().cast_mut();
    let mut cause = ptr::null_mut();
    let command = cmd_parse(&mut value, 1, file, 37, 0, &mut cause);
    assert!(cause.is_null(), "command parse reported an error");
    assert!(!command.is_null());
    command
}

#[test]
fn filename_survives_input_mutation_and_original_command_free() {
    unsafe {
        let mut filename = *b"source-\xff.conf\0";
        let command = parse_display_message(filename.as_ptr().cast());
        let copied = cmd_copy(command, 0, ptr::null_mut());

        assert!(!copied.is_null());
        assert_ne!(
            (*command).file.as_ref().unwrap().as_ptr(),
            filename.as_ptr().cast()
        );
        assert_ne!(
            (*copied).file.as_ref().unwrap().as_ptr(),
            (*command).file.as_ref().unwrap().as_ptr()
        );

        // The parser must retain its own bytes, including non-UTF-8 bytes.
        filename[0] = b'X';
        assert_eq!(
            CStr::from_ptr(((*command).file).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut())).to_bytes(),
            b"source-\xff.conf"
        );
        assert_eq!(
            CStr::from_ptr(((*copied).file).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut())).to_bytes(),
            b"source-\xff.conf"
        );
        assert_eq!((*copied).line, 37);

        cmd_free(command);
        assert_eq!(
            CStr::from_ptr(((*copied).file).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut())).to_bytes(),
            b"source-\xff.conf"
        );
        cmd_free(copied);
    }
}

#[test]
fn absent_filename_stays_null_in_copy() {
    unsafe {
        let command = parse_display_message(ptr::null());
        let copied = cmd_copy(command, 0, ptr::null_mut());
        assert!((*command).file.is_none());
        assert!((*copied).file.is_none());
        cmd_free(command);
        cmd_free(copied);
    }
}
