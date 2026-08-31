//! `fgetln`, which BSD has and Linux does not: the next line of a file, given
//! back as a run of bytes rather than a C string.
//!
//! The bytes are the module's own buffer, not the caller's, so they last only
//! until the next call and must not be freed; the newline that ended the line
//! is part of them, and the length is written to the caller's `len` because
//! there is no terminator to look for. A last line with no newline comes back
//! as it stands, and only the read after it answers nothing.
//!
//! The buffer is the C allocator's rather than this crate's, because a growth
//! that could not be made is answered with nothing rather than by ending the
//! process the way `xmalloc` would. It is never given back except on that
//! failure, and `fgetln` is not reentrant: two readers of two files share the
//! one buffer.
//!
//! Coverage exemptions: the two allocator arms — the first buffer that could
//! not be made and the doubling that could not be done. The C allocator is
//! what hands those out, and a test cannot make it refuse. The one line the
//! test module leaves uncovered besides is the arm that takes a poisoned mutex
//! back, which is only read once another test has already failed.
use crate::ffi::{__errno_location, calloc, free, getc, reallocarray};
pub use crate::types::*;
use ::core::ffi::{c_char, c_int, c_void};
use ::core::ptr::{NonNull, null_mut};

const BUFSIZ: c_int = 8192;
const EOF: c_int = -1;
const EINVAL: c_int = 22;

/// The one buffer every line is read into, which starts at `BUFSIZ` bytes and
/// doubles whenever a line fills it.
static mut BUFFER: Buffer = Buffer { at: None, size: 0 };

struct Buffer {
    at: Option<NonNull<c_char>>,
    size: size_t,
}

impl Buffer {
    /// The bytes the buffer holds.
    fn bytes(&mut self) -> &mut [c_char] {
        let at = self.at.expect("an allocated buffer has storage");
        unsafe { ::core::slice::from_raw_parts_mut(at.as_ptr(), self.size) }
    }

    /// Doubles the buffer, or answers nothing if it could not be doubled. One
    /// that could not be is given back and forgotten, with the allocator's own
    /// errno left standing, so that the next call starts again from nothing.
    fn double(&mut self) -> Option<()> {
        unsafe {
            let at = self.at.expect("an allocated buffer has storage");
            let bigger =
                NonNull::new(reallocarray(at.as_ptr().cast::<c_void>(), 2, self.size).cast());
            let Some(bigger) = bigger else {
                let refused = *__errno_location();
                free(at.as_ptr().cast::<c_void>());
                *__errno_location() = refused;
                self.at = None;
                self.size = 0;
                return None;
            };
            self.at = Some(bigger);
            self.size = self.size.wrapping_mul(2);
            Some(())
        }
    }
}

/// The one buffer, made at `BUFSIZ` bytes if there is not one yet, or nothing
/// if the allocator would not make it.
fn buffer() -> Option<&'static mut Buffer> {
    unsafe {
        let buffer = &mut BUFFER;
        if buffer.at.is_none() {
            buffer.at = NonNull::new(calloc(1, BUFSIZ as size_t).cast());
            buffer.at?;
            buffer.size = BUFSIZ as size_t;
        }
        Some(buffer)
    }
}

/// The next line of `fp` in the shared buffer, its length written to `len`;
/// nothing at all if the buffer could not be made or grown, and a null pointer
/// with a length of zero at the end of the file.
unsafe fn read_line(fp: NonNull<FILE>, len: &mut size_t) -> Option<*mut c_char> {
    unsafe {
        let buffer = buffer()?;
        let mut read: size_t = 0;
        loop {
            let c = getc(fp.as_ptr());
            if c == EOF {
                break;
            }
            buffer.bytes()[read] = c as c_char;
            read = read.wrapping_add(1);
            if read == buffer.size {
                buffer.double()?;
            }
            if c == '\n' as c_int {
                break;
            }
        }
        *len = read;
        Some(if read != 0 {
            buffer.at.expect("an allocated buffer has storage").as_ptr()
        } else {
            null_mut()
        })
    }
}

pub unsafe fn fgetln(fp: *mut FILE, len: *mut size_t) -> *mut c_char {
    unsafe {
        let (Some(fp), Some(mut len)) = (NonNull::new(fp), NonNull::new(len)) else {
            *__errno_location() = EINVAL;
            return null_mut();
        };
        read_line(fp, len.as_mut()).unwrap_or(null_mut())
    }
}

#[cfg(test)]
#[path = "../tests/test_compat_fgetln.rs"]
mod tests;
