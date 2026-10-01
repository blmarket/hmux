//! Shared declarations retain their types across former translation-unit boundaries.
use hmux::src::{arguments, client, screen, tty, window};
use std::ptr::null_mut;

fn parse(
    _: &mut arguments::args,
    index: hmux::src::shared::abi::u_int,
) -> Result<
    hmux::src::shared::arguments::args_parse_type,
    hmux::src::shared::arguments::ArgsParseError,
> {
    if index == 7 {
        Ok(hmux::src::shared::arguments::ARGS_PARSE_STRING)
    } else {
        Err(hmux::src::shared::arguments::ArgsParseError::Usage)
    }
}

unsafe fn mode_screen(_: refbox::Weak<window::window_mode_entry>) -> *mut screen::screen {
    null_mut()
}

#[test]
fn callbacks_cross_original_module_paths_without_conversion() {
    let parse_callback: arguments::args_parse_cb = Some(parse);
    let parser_callback: hmux::src::shared::arguments::args_parse_cb = parse_callback;
    let tty_callback: tty::tty_ctx_set_client_cb = Some(Box::new(|_, _| 17));
    let draw_callback: hmux::src::popup::tty_ctx_set_client_cb = tty_callback;
    let overlay_callback: client::overlay_mode_cb = Some(Box::new(|_| None));
    let server_callback: hmux::src::server_client::overlay_mode_cb = overlay_callback;
    let mut mode = window::window_mode::default();
    assert!(mode.get_screen.is_none());
    mode.get_screen = Some(mode_screen);
    let copy_mode: hmux::src::window_copy::window_mode = mode;
    let mut parsed_args = arguments::args::empty();
    assert_eq!(
        parser_callback.unwrap()(&mut parsed_args, 7).unwrap(),
        hmux::src::shared::arguments::ARGS_PARSE_STRING
    );
    assert!(draw_callback.is_some());
    assert!(server_callback.is_some());
    unsafe {
        assert!(copy_mode.get_screen.unwrap()(refbox::Weak::new()).is_null());
    }
}
