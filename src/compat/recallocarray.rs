use crate::ffi::{__errno_location, calloc, explicit_bzero, free, getpagesize, malloc};
pub use crate::types::*;
pub const ENOMEM: ::core::ffi::c_int = 12 as ::core::ffi::c_int;
pub const EINVAL: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const MUL_NO_OVERFLOW: size_t = (1 as ::core::ffi::c_int as size_t)
    << (::core::mem::size_of::<size_t>() as usize).wrapping_mul(4_usize);
/// What resizing an existing (non-null) block amounts to.
enum Resize {
    /// The requested element count times element size overflows; fail with
    /// this `errno` value.
    Fail(::core::ffi::c_int),
    /// The block shrank little enough to keep in place; just zero `len`
    /// bytes starting at `offset`.
    ZeroTail { offset: size_t, len: size_t },
    /// A fresh block of `newsize` bytes is needed, carrying over the
    /// contents of the `oldsize` bytes already there.
    Reallocate { oldsize: size_t, newsize: size_t },
}

/// Whether `nmemb * size` can overflow, using the same cheap guard as
/// OpenBSD: only pay for the division when either side is large.
fn mul_overflows(nmemb: size_t, size: size_t) -> bool {
    (nmemb >= MUL_NO_OVERFLOW || size >= MUL_NO_OVERFLOW)
        && nmemb > 0
        && (SIZE_MAX as size_t).wrapping_div(nmemb) < size
}

/// Decide how to resize a non-null block, given the page size used as the
/// cutoff for keeping a shrunk block in place.
fn plan_resize(oldnmemb: size_t, newnmemb: size_t, size: size_t, pagesize: size_t) -> Resize {
    if mul_overflows(newnmemb, size) {
        return Resize::Fail(ENOMEM);
    }
    let newsize = newnmemb.wrapping_mul(size);
    if mul_overflows(oldnmemb, size) {
        return Resize::Fail(EINVAL);
    }
    let oldsize = oldnmemb.wrapping_mul(size);
    if newsize <= oldsize {
        let d = oldsize.wrapping_sub(newsize);
        if d < oldsize.wrapping_div(2) && d < pagesize {
            return Resize::ZeroTail {
                offset: newsize,
                len: d,
            };
        }
    }
    Resize::Reallocate { oldsize, newsize }
}

pub unsafe fn recallocarray(
    ptr: *mut u8,
    oldnmemb: size_t,
    newnmemb: size_t,
    size: size_t,
) -> *mut u8 {
    unsafe {
        if ptr.is_null() {
            return calloc(newnmemb, size) as *mut u8;
        }
        match plan_resize(oldnmemb, newnmemb, size, getpagesize() as size_t) {
            Resize::Fail(err) => {
                *__errno_location() = err;
                ::core::ptr::null_mut()
            }
            Resize::ZeroTail { offset, len } => {
                ::core::ptr::write_bytes(ptr.add(offset), 0, len);
                ptr
            }
            Resize::Reallocate { oldsize, newsize } => {
                let newptr = malloc(newsize) as *mut u8;
                if newptr.is_null() {
                    return ::core::ptr::null_mut();
                }
                if newsize > oldsize {
                    ::core::ptr::copy_nonoverlapping(ptr, newptr, oldsize);
                    ::core::ptr::write_bytes(newptr.add(oldsize), 0, newsize.wrapping_sub(oldsize));
                } else {
                    ::core::ptr::copy_nonoverlapping(ptr, newptr, newsize);
                }
                explicit_bzero(ptr as *mut ::core::ffi::c_void, oldsize);
                free(ptr as *mut ::core::ffi::c_void);
                newptr
            }
        }
    }
}

#[cfg(test)]
#[path = "../tests/test_compat_recallocarray.rs"]
mod tests;
