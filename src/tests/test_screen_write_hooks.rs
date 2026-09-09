use super::*;

pub(super) unsafe fn initctx(
    ctx: &mut screen_write_ctx,
    ttyctx: &mut tty_ctx,
    is_sync: c_int,
    check_obscured: c_int,
) {
    unsafe { screen_write_initctx(ctx, ttyctx, is_sync, check_obscured) }
}

pub(super) unsafe fn collect_flush(
    ctx: &mut screen_write_ctx,
    scroll_only: c_int,
    from: &core::ffi::CStr,
) {
    unsafe { screen_write_collect_flush(ctx, scroll_only, from) }
}

pub(crate) fn set_client_cb() -> unsafe fn(&mut tty_ctx, &mut client) -> c_int {
    screen_write_set_client_cb
}
