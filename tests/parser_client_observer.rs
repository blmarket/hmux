use hmux2::src::cmd::{cmd_list_print, parse::cmd_parse_from_string};
use hmux2::src::options::{options_create, options_free};
use hmux2::src::shared::client::client;
use hmux2::src::shared::command::{cmd_parse_input, CMD_PARSE_NOALIAS, CMD_PARSE_SUCCESS};
use hmux2::src::tmux::{global_environ, global_options, global_s_options, global_w_options};
use std::rc::Rc;

#[test]
fn format_condition_handles_expired_parser_client() {
    unsafe {
        global_options = options_create(std::ptr::null_mut());
        global_s_options = options_create(std::ptr::null_mut());
        global_w_options = options_create(std::ptr::null_mut());
        global_environ = Some(hmux2::src::environ::environ_create());
        let client = client::new();
        let mut input = cmd_parse_input {
            c: Rc::downgrade(&client),
            flags: CMD_PARSE_NOALIAS,
            ..Default::default()
        };
        drop(client);
        assert!(input.c.upgrade().is_none());
        let result = cmd_parse_from_string(
            c"%if #{client_name}\ndisplay-message stale\n%else\ndisplay-message absent\n%endif\n",
            &mut input,
        );
        assert_eq!(result.status, CMD_PARSE_SUCCESS, "{:?}", result.error);
        assert_eq!(
            cmd_list_print(&result.cmdlist.as_ref().unwrap().borrow(), 0).as_c_str(),
            c"display-message absent",
        );
        drop(result);
        options_free(global_options);
        options_free(global_s_options);
        options_free(global_w_options);
        global_options = std::ptr::null_mut();
        global_s_options = std::ptr::null_mut();
        global_w_options = std::ptr::null_mut();
        global_environ = None;
    }
}
