pub use crate::src::shared::errno::{EINVAL, ENOMEM};
pub use crate::src::shared::limits::SIZE_MAX;
use crate::src::shared::abi::*;
extern "C" {
    fn __errno_location() -> *mut ::core::ffi::c_int;
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn explicit_bzero(__s: *mut ::core::ffi::c_void, __n: size_t);
    fn getpagesize() -> ::core::ffi::c_int;
    fn malloc(__size: size_t) -> *mut ::core::ffi::c_void;
    fn calloc(__nmemb: size_t, __size: size_t) -> *mut ::core::ffi::c_void;
    fn free(__ptr: *mut ::core::ffi::c_void);
}

pub const MUL_NO_OVERFLOW: size_t = (1 as ::core::ffi::c_int as size_t)
    << (::core::mem::size_of::<size_t>() as usize).wrapping_mul(4 as usize);
#[no_mangle]
pub unsafe extern "C" fn recallocarray(
    mut ptr: *mut ::core::ffi::c_void,
    mut oldnmemb: size_t,
    mut newnmemb: size_t,
    mut size: size_t,
) -> *mut ::core::ffi::c_void {
    let mut oldsize: size_t = 0;
    let mut newsize: size_t = 0;
    let mut newptr: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
    if ptr.is_null() {
        return calloc(newnmemb, size);
    }
    if (newnmemb >= MUL_NO_OVERFLOW || size >= MUL_NO_OVERFLOW)
        && newnmemb > 0 as size_t
        && (SIZE_MAX as size_t).wrapping_div(newnmemb) < size
    {
        *__errno_location() = ENOMEM;
        return NULL;
    }
    newsize = newnmemb.wrapping_mul(size);
    if (oldnmemb >= MUL_NO_OVERFLOW || size >= MUL_NO_OVERFLOW)
        && oldnmemb > 0 as size_t
        && (SIZE_MAX as size_t).wrapping_div(oldnmemb) < size
    {
        *__errno_location() = EINVAL;
        return NULL;
    }
    oldsize = oldnmemb.wrapping_mul(size);
    if newsize <= oldsize {
        let mut d: size_t = oldsize.wrapping_sub(newsize);
        if d < oldsize.wrapping_div(2 as size_t) && d < getpagesize() as size_t {
            memset(
                (ptr as *mut ::core::ffi::c_char).offset(newsize as isize)
                    as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                d,
            );
            return ptr;
        }
    }
    newptr = malloc(newsize);
    if newptr.is_null() {
        return NULL;
    }
    if newsize > oldsize {
        memcpy(newptr, ptr, oldsize);
        memset(
            (newptr as *mut ::core::ffi::c_char).offset(oldsize as isize)
                as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            newsize.wrapping_sub(oldsize),
        );
    } else {
        memcpy(newptr, ptr, newsize);
    }
    explicit_bzero(ptr, oldsize);
    free(ptr);
    return newptr;
}
