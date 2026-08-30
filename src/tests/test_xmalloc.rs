use super::*;
use crate::ffi::free;
use ::core::ffi::{c_char, c_int, c_void};

/// The bytes `p` points at, freeing it afterwards.
unsafe fn bytes(p: *mut u8, len: usize) -> Vec<u8> {
    unsafe {
        let v = ::core::slice::from_raw_parts(p, len).to_vec();
        free(p as *mut c_void);
        v
    }
}

#[test]
fn xmalloc_hands_out_memory_that_can_be_written() {
    unsafe {
        let p = xmalloc(4);
        assert!(!p.is_null());
        ::core::ptr::write_bytes(p, 0xab, 4);
        assert_eq!(bytes(p, 4), vec![0xab; 4]);
    }
}

#[test]
fn xcalloc_hands_out_zeroed_memory() {
    unsafe {
        let p = xcalloc(3, 8);
        assert!(!p.is_null());
        assert_eq!(bytes(p, 24), vec![0u8; 24]);
    }
}

#[test]
fn xrealloc_keeps_what_was_there() {
    unsafe {
        let p = xmalloc(4);
        ::core::ptr::write_bytes(p, 0x11, 4);
        let q = xrealloc(p, 16);
        assert!(!q.is_null());
        assert_eq!(::core::slice::from_raw_parts(q, 4), &[0x11u8; 4]);
        free(q as *mut c_void);
    }
}

#[test]
fn xreallocarray_sizes_by_the_product() {
    unsafe {
        let p = xreallocarray(::core::ptr::null_mut::<u8>(), 4, 8);
        assert!(!p.is_null());
        ::core::ptr::write_bytes(p, 0x22, 32);
        let q = xreallocarray(p, 8, 8);
        assert_eq!(::core::slice::from_raw_parts(q, 32), &[0x22u8; 32]);
        free(q as *mut c_void);
    }
}

#[test]
fn xrecallocarray_zeroes_what_it_added() {
    unsafe {
        let p = xrecallocarray(::core::ptr::null_mut::<u8>(), 0, 2, 4);
        assert_eq!(bytes(p, 8), vec![0u8; 8]);

        let p = xmalloc(8);
        ::core::ptr::write_bytes(p, 0x33, 8);
        let q = xrecallocarray(p, 2, 4, 4);
        let got = bytes(q, 16);
        assert_eq!(&got[..8], &[0x33u8; 8]);
        assert_eq!(&got[8..], &[0u8; 8]);
    }
}

#[test]
fn xasprintf_allocates_the_formatted_string() {
    unsafe {
        let s = xasprintf(c"%s-%d".as_ptr(), fmt_args![c"a".as_ptr(), 42 as c_int]);
        assert_eq!(s.as_bytes(), b"a-42");
    }
}

#[test]
fn xsnprintf_writes_into_the_buffer_given() {
    unsafe {
        let mut buf = [0 as c_char; 16];
        let n = xsnprintf(
            buf.as_mut_ptr(),
            buf.len() as size_t,
            c"%s%d".as_ptr(),
            fmt_args![c"n=".as_ptr(), 7 as c_int],
        );
        assert_eq!(n, 3);
        assert_eq!(
            ::core::ffi::CStr::from_ptr(buf.as_ptr()).to_str().unwrap(),
            "n=7"
        );
    }
}
