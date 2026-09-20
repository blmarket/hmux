unsafe fn invalid_owner(ptr: *mut u8) {
    let _ = Box::from_raw(ptr);
}
