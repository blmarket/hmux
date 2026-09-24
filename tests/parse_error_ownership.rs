use hmux2::src::cmd::parse::cmd_parse_from_buffer;
use hmux2::src::shared::command::{cmd_parse_input, CMD_PARSE_ERROR};

#[test]
fn lexer_error_retains_formatted_message_after_temporary_drops() {
    let input = b"display-message \"\\400\"";
    let file = b"source\xff.conf\0";
    let mut pi: cmd_parse_input = unsafe { std::mem::zeroed() };
    pi.file = file.as_ptr().cast();
    pi.line = 7;

    unsafe {
        let result = cmd_parse_from_buffer(input.as_ptr().cast(), input.len(), &mut pi);
        assert_eq!(result.status, CMD_PARSE_ERROR);
        assert_eq!(
            result.error.as_ref().unwrap().as_bytes(),
            b"source\xff.conf:7: invalid octal escape"
        );
    }
}
