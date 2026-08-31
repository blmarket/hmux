// tmux.c itself lives in the library, as src/tmux.rs; this binary is only
// the entry point that forwards argv to it.
use std::os::unix::ffi::OsStringExt;

fn main() {
    let mut args_strings: Vec<Vec<u8>> = ::std::env::args_os()
        .map(|arg| {
            ::std::ffi::CString::new(arg.into_vec())
                .expect("Failed to convert argument into CString.")
                .into_bytes_with_nul()
        })
        .collect();
    let mut args_ptrs: Vec<*mut ::core::ffi::c_char> = args_strings
        .iter_mut()
        .map(|arg| arg.as_mut_ptr() as *mut ::core::ffi::c_char)
        .chain(::core::iter::once(::core::ptr::null_mut()))
        .collect();
    unsafe { ::std::process::exit(::tmux_c2rs::tmux::main_0(&mut args_ptrs) as i32) }
}
