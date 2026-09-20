//! Frozen measurements of every original copy; no fixture update mode.
use std::mem::{align_of, offset_of, size_of};
#[test]
fn original_declarations_match() {
    let mut records = Vec::new();
    let v: ::core::ffi::c_int = hmux2::src::format::FNM_CASEFOLD;
    records.push(format!(
        "src/format.rs::FNM_CASEFOLD {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::window::FNM_CASEFOLD;
    records.push(format!(
        "src/window.rs::FNM_CASEFOLD {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::cmd_save_buffer::O_APPEND;
    records.push(format!(
        "src/cmd_save_buffer.rs::O_APPEND {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::file::O_APPEND;
    records.push(format!(
        "src/file.rs::O_APPEND {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::client::O_CREAT;
    records.push(format!(
        "src/client.rs::O_CREAT {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::file::O_CREAT;
    records.push(format!(
        "src/file.rs::O_CREAT {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::tty::O_CREAT;
    records.push(format!(
        "src/tty.rs::O_CREAT {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::file::O_NONBLOCK;
    records.push(format!(
        "src/file.rs::O_NONBLOCK {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::tmux::O_NONBLOCK;
    records.push(format!(
        "src/tmux.rs::O_NONBLOCK {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::cmd_save_buffer::O_TRUNC;
    records.push(format!(
        "src/cmd_save_buffer.rs::O_TRUNC {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::tty::O_TRUNC;
    records.push(format!(
        "src/tty.rs::O_TRUNC {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::client::O_WRONLY;
    records.push(format!(
        "src/client.rs::O_WRONLY {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::cmd_pipe_pane::O_WRONLY;
    records.push(format!(
        "src/cmd_pipe_pane.rs::O_WRONLY {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::file::O_WRONLY;
    records.push(format!(
        "src/file.rs::O_WRONLY {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::tty::O_WRONLY;
    records.push(format!(
        "src/tty.rs::O_WRONLY {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::client::STDERR_FILENO;
    records.push(format!(
        "src/client.rs::STDERR_FILENO {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::cmd_pipe_pane::STDERR_FILENO;
    records.push(format!(
        "src/cmd_pipe_pane.rs::STDERR_FILENO {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::file::STDERR_FILENO;
    records.push(format!(
        "src/file.rs::STDERR_FILENO {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::job::STDERR_FILENO;
    records.push(format!(
        "src/job.rs::STDERR_FILENO {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::server_client::STDERR_FILENO;
    records.push(format!(
        "src/server_client.rs::STDERR_FILENO {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::spawn::STDERR_FILENO;
    records.push(format!(
        "src/spawn.rs::STDERR_FILENO {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::client::STDIN_FILENO;
    records.push(format!(
        "src/client.rs::STDIN_FILENO {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::cmd_pipe_pane::STDIN_FILENO;
    records.push(format!(
        "src/cmd_pipe_pane.rs::STDIN_FILENO {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::file::STDIN_FILENO;
    records.push(format!(
        "src/file.rs::STDIN_FILENO {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::job::STDIN_FILENO;
    records.push(format!(
        "src/job.rs::STDIN_FILENO {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::server_client::STDIN_FILENO;
    records.push(format!(
        "src/server_client.rs::STDIN_FILENO {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::spawn::STDIN_FILENO;
    records.push(format!(
        "src/spawn.rs::STDIN_FILENO {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::client::STDOUT_FILENO;
    records.push(format!(
        "src/client.rs::STDOUT_FILENO {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::cmd_pipe_pane::STDOUT_FILENO;
    records.push(format!(
        "src/cmd_pipe_pane.rs::STDOUT_FILENO {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::file::STDOUT_FILENO;
    records.push(format!(
        "src/file.rs::STDOUT_FILENO {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::job::STDOUT_FILENO;
    records.push(format!(
        "src/job.rs::STDOUT_FILENO {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::server_client::STDOUT_FILENO;
    records.push(format!(
        "src/server_client.rs::STDOUT_FILENO {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::server::S_IRWXU;
    records.push(format!(
        "src/server.rs::S_IRWXU {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::tmux::S_IRWXU;
    records.push(format!(
        "src/tmux.rs::S_IRWXU {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::client::WAIT_ANY;
    records.push(format!(
        "src/client.rs::WAIT_ANY {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::server::WAIT_ANY;
    records.push(format!(
        "src/server.rs::WAIT_ANY {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::client::WNOHANG;
    records.push(format!(
        "src/client.rs::WNOHANG {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::server::WNOHANG;
    records.push(format!(
        "src/server.rs::WNOHANG {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::server_client::X_OK;
    records.push(format!(
        "src/server_client.rs::X_OK {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::tmux::X_OK;
    records.push(format!(
        "src/tmux.rs::X_OK {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: [::core::ffi::c_char; 8] = hmux2::src::cmd_display_menu::_PATH_BSHELL;
    records.push(format!(
        "src/cmd_display_menu.rs::_PATH_BSHELL {:?} {} {}",
        v,
        size_of::<[::core::ffi::c_char; 8]>(),
        align_of::<[::core::ffi::c_char; 8]>()
    ));
    let v: [::core::ffi::c_char; 8] = hmux2::src::cmd_pipe_pane::_PATH_BSHELL;
    records.push(format!(
        "src/cmd_pipe_pane.rs::_PATH_BSHELL {:?} {} {}",
        v,
        size_of::<[::core::ffi::c_char; 8]>(),
        align_of::<[::core::ffi::c_char; 8]>()
    ));
    let v: [::core::ffi::c_char; 8] = hmux2::src::job::_PATH_BSHELL;
    records.push(format!(
        "src/job.rs::_PATH_BSHELL {:?} {} {}",
        v,
        size_of::<[::core::ffi::c_char; 8]>(),
        align_of::<[::core::ffi::c_char; 8]>()
    ));
    let v: [::core::ffi::c_char; 8] = hmux2::src::options_table::_PATH_BSHELL;
    records.push(format!(
        "src/options_table.rs::_PATH_BSHELL {:?} {} {}",
        v,
        size_of::<[::core::ffi::c_char; 8]>(),
        align_of::<[::core::ffi::c_char; 8]>()
    ));
    let v: [::core::ffi::c_char; 8] = hmux2::src::server_client::_PATH_BSHELL;
    records.push(format!(
        "src/server_client.rs::_PATH_BSHELL {:?} {} {}",
        v,
        size_of::<[::core::ffi::c_char; 8]>(),
        align_of::<[::core::ffi::c_char; 8]>()
    ));
    let v: [::core::ffi::c_char; 8] = hmux2::src::spawn::_PATH_BSHELL;
    records.push(format!(
        "src/spawn.rs::_PATH_BSHELL {:?} {} {}",
        v,
        size_of::<[::core::ffi::c_char; 8]>(),
        align_of::<[::core::ffi::c_char; 8]>()
    ));
    let v: [::core::ffi::c_char; 8] = hmux2::src::tmux::_PATH_BSHELL;
    records.push(format!(
        "src/tmux.rs::_PATH_BSHELL {:?} {} {}",
        v,
        size_of::<[::core::ffi::c_char; 8]>(),
        align_of::<[::core::ffi::c_char; 8]>()
    ));
    let v: [::core::ffi::c_char; 10] = hmux2::src::cmd_pipe_pane::_PATH_DEVNULL;
    records.push(format!(
        "src/cmd_pipe_pane.rs::_PATH_DEVNULL {:?} {} {}",
        v,
        size_of::<[::core::ffi::c_char; 10]>(),
        align_of::<[::core::ffi::c_char; 10]>()
    ));
    let v: [::core::ffi::c_char; 10] = hmux2::src::job::_PATH_DEVNULL;
    records.push(format!(
        "src/job.rs::_PATH_DEVNULL {:?} {} {}",
        v,
        size_of::<[::core::ffi::c_char; 10]>(),
        align_of::<[::core::ffi::c_char; 10]>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::server::__S_IEXEC;
    records.push(format!(
        "src/server.rs::__S_IEXEC {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::tmux::__S_IEXEC;
    records.push(format!(
        "src/tmux.rs::__S_IEXEC {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::server::__S_IREAD;
    records.push(format!(
        "src/server.rs::__S_IREAD {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::tmux::__S_IREAD;
    records.push(format!(
        "src/tmux.rs::__S_IREAD {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::server::__S_IWRITE;
    records.push(format!(
        "src/server.rs::__S_IWRITE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let v: ::core::ffi::c_int = hmux2::src::tmux::__S_IWRITE;
    records.push(format!(
        "src/tmux.rs::__S_IWRITE {:?} {} {}",
        v,
        size_of::<::core::ffi::c_int>(),
        align_of::<::core::ffi::c_int>()
    ));
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/subjects-posix_io.txt"));
}
