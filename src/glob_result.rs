//! Owned paths and bookkeeping from libc pathname expansion.

use core::ffi::CStr;
use std::rc::Rc;

/// The paths and public bookkeeping returned by libc pathname expansion.
///
/// ```compile_fail
/// use std::ffi::CStr;
/// use std::rc::Rc;
/// use tmux_c2rs::{GlobPaths, GlobResult};
/// fn escaped() -> &'static CStr {
///     let paths = GlobPaths::from_glob_paths(vec![Some(Rc::from(c"one"))], 0, 0);
///     paths.glob_path(0).unwrap()
/// }
/// ```
pub trait GlobResult: Default {
    /// Retains ownership of the matched path slots and their bookkeeping.
    fn from_glob_paths(paths: Vec<Option<Rc<CStr>>>, offsets: usize, flags: i32) -> Self
    where
        Self: Sized;

    /// Returns the number of matched path slots.
    fn glob_path_count(&self) -> usize;

    /// Borrows one matched path, or none when the index or entry is absent.
    fn glob_path(&self, index: usize) -> Option<&CStr>;

    /// Returns the number of reserved leading slots in the original result.
    fn glob_offsets(&self) -> usize;

    /// Returns libc's result flags.
    fn glob_flags(&self) -> i32;
}

/// Owned matched names, with shared immutable storage for each path.
#[derive(Clone, Default)]
pub struct GlobPaths {
    paths: Vec<Option<Rc<CStr>>>,
    offsets: usize,
    flags: i32,
}

struct NativeGlob(libc::glob_t);

impl Drop for NativeGlob {
    fn drop(&mut self) {
        unsafe { libc::globfree(&raw mut self.0) };
    }
}

impl GlobPaths {
    /// Expands a pathname with libc's default flags and preserves its error code.
    pub fn expand(pattern: &CStr) -> Result<Self, i32> {
        let mut native = NativeGlob(unsafe { std::mem::zeroed() });
        let result = unsafe { libc::glob(pattern.as_ptr(), 0, None, &raw mut native.0) };
        if result != 0 {
            return Err(result);
        }
        let paths = if native.0.gl_pathc == 0 {
            Vec::new()
        } else {
            let paths = unsafe {
                std::slice::from_raw_parts(
                    native.0.gl_pathv.add(native.0.gl_offs),
                    native.0.gl_pathc,
                )
            };
            paths
                .iter()
                .map(|&path| (!path.is_null()).then(|| Rc::from(unsafe { CStr::from_ptr(path) })))
                .collect()
        };
        Ok(Self::from_glob_paths(
            paths,
            native.0.gl_offs,
            native.0.gl_flags,
        ))
    }

    /// Transfers the matched names to their next owner, skipping absent slots.
    pub fn into_paths(self) -> impl Iterator<Item = Rc<CStr>> {
        self.paths.into_iter().flatten()
    }
}

impl GlobResult for GlobPaths {
    fn from_glob_paths(paths: Vec<Option<Rc<CStr>>>, offsets: usize, flags: i32) -> Self {
        Self {
            paths,
            offsets,
            flags,
        }
    }
    fn glob_path_count(&self) -> usize {
        self.paths.len()
    }
    fn glob_path(&self, index: usize) -> Option<&CStr> {
        self.paths.get(index).and_then(Option::as_deref)
    }
    fn glob_offsets(&self) -> usize {
        self.offsets
    }
    fn glob_flags(&self) -> i32 {
        self.flags
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owned_paths_survive_their_original_handles_and_transfer_without_copying() {
        let path: Rc<CStr> = Rc::from(c"one");
        let result = GlobPaths::from_glob_paths(
            vec![Some(path.clone()), None, Some(Rc::from(c"two"))],
            1,
            7,
        );
        assert_eq!(result.glob_path_count(), 3);
        assert_eq!(result.glob_path(0), Some(c"one"));
        assert_eq!(result.glob_path(1), None);
        assert_eq!(result.glob_path(3), None);
        assert_eq!(result.glob_offsets(), 1);
        assert_eq!(result.glob_flags(), 7);
        let retained = result.clone().into_paths().next().unwrap();
        assert!(Rc::ptr_eq(&path, &retained));
        drop(path);
        drop(result);
        assert_eq!(&*retained, c"one");
    }

    #[test]
    fn libc_expansion_owns_sorted_non_utf8_paths_and_reports_no_match() {
        use std::os::unix::ffi::OsStrExt;
        struct TempDir(std::path::PathBuf);
        impl Drop for TempDir {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory =
            std::env::temp_dir().join(format!("hmux-glob-{}-{stamp}", std::process::id()));
        std::fs::create_dir(&directory).unwrap();
        let directory = TempDir(directory);
        for name in [b"b.conf".as_slice(), b"a.conf", b"\xff.conf"] {
            std::fs::write(directory.0.join(std::ffi::OsStr::from_bytes(name)), b"").unwrap();
        }
        let pattern =
            std::ffi::CString::new(directory.0.join("*.conf").as_os_str().as_bytes()).unwrap();
        let result = GlobPaths::expand(&pattern).unwrap();
        assert_eq!(result.glob_path_count(), 3);
        assert_eq!(result.glob_offsets(), 0);
        let names: Vec<_> = result
            .into_paths()
            .map(|path| {
                path.to_bytes()
                    .rsplit(|&b| b == b'/')
                    .next()
                    .unwrap()
                    .to_vec()
            })
            .collect();
        assert_eq!(
            names,
            [
                b"a.conf".to_vec(),
                b"b.conf".to_vec(),
                b"\xff.conf".to_vec()
            ]
        );
        let missing =
            std::ffi::CString::new(directory.0.join("*.missing").as_os_str().as_bytes()).unwrap();
        assert_eq!(GlobPaths::expand(&missing).err(), Some(libc::GLOB_NOMATCH));
    }
}
