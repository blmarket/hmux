//! C callbacks retain their signatures and can cross former translation-unit boundaries.
use hmux2::src::{arguments, client, cmd_choose_tree, screen, tty, window};
use std::ptr::null_mut;

unsafe extern "C" fn parse(
    _: *mut arguments::args,
    index: hmux2::src::shared::abi::u_int,
    _: *mut *mut core::ffi::c_char,
) -> hmux2::src::shared::arguments::args_parse_type {
    if index == 7 {
        hmux2::src::shared::arguments::ARGS_PARSE_STRING
    } else {
        hmux2::src::shared::arguments::ARGS_PARSE_INVALID
    }
}

unsafe extern "C" fn set_client(_: *mut tty::tty_ctx, _: *mut client::client) -> core::ffi::c_int {
    17
}

unsafe extern "C" fn overlay(
    _: *mut client::client,
    _: *mut core::ffi::c_void,
    _: *mut hmux2::src::shared::abi::u_int,
    _: *mut hmux2::src::shared::abi::u_int,
) -> *mut screen::screen {
    null_mut()
}

unsafe extern "C" fn mode_screen(_: *mut window::window_mode_entry) -> *mut screen::screen {
    null_mut()
}

#[test]
fn callbacks_cross_original_module_paths_without_conversion() {
    let parse_callback: arguments::args_parse_cb = Some(parse);
    let parser_callback: cmd_choose_tree::args_parse_cb = parse_callback;
    let tty_callback: tty::tty_ctx_set_client_cb = Some(set_client);
    let draw_callback: hmux2::src::popup::tty_ctx_set_client_cb = tty_callback;
    let overlay_callback: client::overlay_mode_cb = Some(overlay);
    let server_callback: hmux2::src::server_client::overlay_mode_cb = overlay_callback;
    // The structure consists only of integer and nullable pointer fields.
    let mut mode: window::window_mode = unsafe { std::mem::zeroed() };
    assert!(mode.get_screen.is_none());
    mode.get_screen = Some(mode_screen);
    let copy_mode: hmux2::src::window_copy::window_mode = mode;
    unsafe {
        assert_eq!(
            parser_callback.unwrap()(null_mut(), 7, null_mut()),
            hmux2::src::shared::arguments::ARGS_PARSE_STRING
        );
        assert_eq!(draw_callback.unwrap()(null_mut(), null_mut()), 17);
        assert!(server_callback.unwrap()(null_mut(), null_mut(), null_mut(), null_mut()).is_null());
        assert!(copy_mode.get_screen.unwrap()(null_mut()).is_null());
    }
}
