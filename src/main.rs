// tmux.c itself lives in the library, as src/tmux.rs; this binary is only
// the entry point that forwards argv to it.
use std::os::unix::ffi::OsStringExt;

fn main() {
    let mut args: Vec<std::ffi::CString> = std::env::args_os()
        .map(|arg| {
            std::ffi::CString::new(arg.into_vec())
                .expect("Failed to convert argument into CString.")
        })
        .collect();
    unsafe { std::process::exit(tmux_c2rs::tmux::main_0(&mut args) as i32) }
}
