fn unreviewed_function() {
    static mut unreviewed_scratch: usize = 0;
    let _ = unsafe { unreviewed_scratch };
}
