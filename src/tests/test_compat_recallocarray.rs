use super::*;

fn errno() -> ::core::ffi::c_int {
    unsafe { *__errno_location() }
}

fn set_errno(v: ::core::ffi::c_int) {
    unsafe { *__errno_location() = v };
}

fn filled(n: usize, byte: u8) -> *mut u8 {
    unsafe {
        let p = malloc(n as size_t);
        assert!(!p.is_null());
        ::core::ptr::write_bytes(p as *mut u8, byte, n);
        p as *mut u8
    }
}

unsafe fn bytes(p: *mut u8, n: usize) -> Vec<u8> {
    unsafe { ::core::slice::from_raw_parts(p, n).to_vec() }
}

#[test]
fn null_pointer_callocs_zeroed_memory() {
    unsafe {
        let p = recallocarray(::core::ptr::null_mut::<u8>(), 0, 4, 8);
        assert!(!p.is_null());
        assert_eq!(bytes(p, 32), vec![0u8; 32]);
        free(p as *mut ::core::ffi::c_void);
    }
}

#[test]
fn new_count_overflow_reports_enomem() {
    unsafe {
        let p = filled(8, 0xaa);
        set_errno(0);
        let r = recallocarray(p, 1, 1usize << 32, 1usize << 32);
        assert!(r.is_null());
        assert_eq!(errno(), ENOMEM);
        free(p as *mut ::core::ffi::c_void);
    }
}

#[test]
fn old_count_overflow_reports_einval() {
    unsafe {
        let p = filled(8, 0xaa);
        set_errno(0);
        let r = recallocarray(p, 1usize << 32, 1, 1usize << 32);
        assert!(r.is_null());
        assert_eq!(errno(), EINVAL);
        free(p as *mut ::core::ffi::c_void);
    }
}

#[test]
fn small_shrink_zeroes_the_tail_in_place() {
    unsafe {
        let p = filled(100, 0xaa);
        let r = recallocarray(p, 100, 99, 1);
        assert_eq!(r, p);
        let b = bytes(r, 100);
        assert_eq!(&b[..99], &[0xaau8; 99][..]);
        assert_eq!(b[99], 0);
        free(r as *mut ::core::ffi::c_void);
    }
}

#[test]
fn same_size_returns_the_same_pointer() {
    unsafe {
        let p = filled(100, 0xaa);
        let r = recallocarray(p, 100, 100, 1);
        assert_eq!(r, p);
        assert_eq!(bytes(r, 100), vec![0xaau8; 100]);
        free(r as *mut ::core::ffi::c_void);
    }
}

#[test]
fn large_shrink_reallocates_and_copies() {
    unsafe {
        let p = filled(100, 0xaa);
        let r = recallocarray(p, 100, 10, 1);
        assert!(!r.is_null());
        assert_ne!(r, p);
        assert_eq!(bytes(r, 10), vec![0xaau8; 10]);
        free(r as *mut ::core::ffi::c_void);
    }
}

#[test]
fn shrink_of_at_least_a_page_reallocates() {
    unsafe {
        let p = filled(100000, 0xaa);
        let r = recallocarray(p, 100000, 90000, 1);
        assert!(!r.is_null());
        assert_ne!(r, p);
        assert_eq!(bytes(r, 90000), vec![0xaau8; 90000]);
        free(r as *mut ::core::ffi::c_void);
    }
}

#[test]
fn growth_copies_and_zeroes_the_new_tail() {
    unsafe {
        let p = filled(32, 0xaa);
        let r = recallocarray(p, 4, 8, 8);
        assert!(!r.is_null());
        let b = bytes(r, 64);
        assert_eq!(&b[..32], &[0xaau8; 32][..]);
        assert_eq!(&b[32..], &[0u8; 32][..]);
        free(r as *mut ::core::ffi::c_void);
    }
}

#[test]
fn zero_sized_old_block_takes_the_reallocation_path() {
    unsafe {
        let p = filled(8, 0xaa);
        let r = recallocarray(p, 0, 0, 8);
        assert!(!r.is_null());
    }
}

#[test]
fn allocation_failure_returns_null() {
    unsafe {
        let p = filled(1, 0xaa);
        let r = recallocarray(p, 1, (SIZE_MAX / 2) as size_t, 1);
        assert!(r.is_null());
        free(p as *mut ::core::ffi::c_void);
    }
}
