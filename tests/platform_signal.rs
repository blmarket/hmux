//! Exercise signal callback interoperability between client and process modules.

#[test]
fn signal_callback_abi() {
    use hmux2::src::{client, proc};
    use std::ffi::{c_int, c_void};
    unsafe extern "C" fn handler(signal: c_int) {
        assert_eq!(signal, 15);
    }
    unsafe extern "C" fn info(signal: c_int, data: *mut client::siginfo_t, arg: *mut c_void) {
        assert_eq!(signal, 15);
        assert!(data.is_null() && arg.is_null());
    }
    unsafe extern "C" fn restore() {}
    let client_handler: client::__sighandler_t = Some(handler);
    let proc_handler: proc::__sighandler_t = client_handler;
    let mut action: client::sigaction = Default::default();
    action.__sigaction_handler.sa_handler = proc_handler;
    unsafe {
        action.__sigaction_handler.sa_handler.unwrap()(15);
    }
    action.__sigaction_handler.sa_sigaction = Some(info);
    action.sa_restorer = Some(restore);
    let action: proc::sigaction = action;
    unsafe {
        action.__sigaction_handler.sa_sigaction.unwrap()(
            15,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        action.sa_restorer.unwrap()();
    }
    assert!(client::SIG_DFL.is_none() && proc::SIG_DFL.is_none());
}
