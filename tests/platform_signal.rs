//! Frozen measurements from every original translation-unit copy (Linux x86_64).
use std::mem::{align_of, offset_of, size_of};
#[test]
fn platform_layouts() {
    let mut records = Vec::new();
    macro_rules! layout {
    ($label:literal, $ty:path, [$($field:ident),*]) => {
        records.push(format!(concat!($label, " {} {}", $(" ", stringify!($field), "={}"),*),
            size_of::<$ty>(), align_of::<$ty>() $(, offset_of!($ty, $field))*));
    };
}
    macro_rules! constant {
        ($label:literal, $ty:ty, $value:path) => {{
            let value: $ty = $value;
            records.push(format!(
                concat!($label, " {:?} {} {}"),
                value,
                size_of::<$ty>(),
                align_of::<$ty>()
            ));
        }};
    }

    constant!(
        "src/client.rs::SA_RESTART",
        ::core::ffi::c_int,
        hmux2::src::client::SA_RESTART
    );
    constant!(
        "src/proc.rs::SA_RESTART",
        ::core::ffi::c_int,
        hmux2::src::proc::SA_RESTART
    );
    constant!(
        "src/client.rs::SIGCHLD",
        ::core::ffi::c_int,
        hmux2::src::client::SIGCHLD
    );
    constant!(
        "src/proc.rs::SIGCHLD",
        ::core::ffi::c_int,
        hmux2::src::proc::SIGCHLD
    );
    constant!(
        "src/server.rs::SIGCHLD",
        ::core::ffi::c_int,
        hmux2::src::server::SIGCHLD
    );
    constant!(
        "src/server_fn.rs::SIGCHLD",
        ::core::ffi::c_int,
        hmux2::src::server_fn::SIGCHLD
    );
    constant!(
        "src/spawn.rs::SIGCHLD",
        ::core::ffi::c_int,
        hmux2::src::spawn::SIGCHLD
    );
    constant!(
        "src/window.rs::SIGCHLD",
        ::core::ffi::c_int,
        hmux2::src::window::SIGCHLD
    );
    constant!(
        "src/client.rs::SIGCONT",
        ::core::ffi::c_int,
        hmux2::src::client::SIGCONT
    );
    constant!(
        "src/job.rs::SIGCONT",
        ::core::ffi::c_int,
        hmux2::src::job::SIGCONT
    );
    constant!(
        "src/proc.rs::SIGCONT",
        ::core::ffi::c_int,
        hmux2::src::proc::SIGCONT
    );
    constant!(
        "src/server.rs::SIGCONT",
        ::core::ffi::c_int,
        hmux2::src::server::SIGCONT
    );
    constant!(
        "src/client.rs::SIGHUP",
        ::core::ffi::c_int,
        hmux2::src::client::SIGHUP
    );
    constant!(
        "src/popup.rs::SIGHUP",
        ::core::ffi::c_int,
        hmux2::src::popup::SIGHUP
    );
    constant!(
        "src/proc.rs::SIGHUP",
        ::core::ffi::c_int,
        hmux2::src::proc::SIGHUP
    );
    constant!(
        "src/spawn.rs::SIGHUP",
        ::core::ffi::c_int,
        hmux2::src::spawn::SIGHUP
    );
    constant!(
        "src/proc.rs::SIGINT",
        ::core::ffi::c_int,
        hmux2::src::proc::SIGINT
    );
    constant!(
        "src/server.rs::SIGINT",
        ::core::ffi::c_int,
        hmux2::src::server::SIGINT
    );
    constant!(
        "src/client.rs::SIGTERM",
        ::core::ffi::c_int,
        hmux2::src::client::SIGTERM
    );
    constant!(
        "src/cmd_kill_server.rs::SIGTERM",
        ::core::ffi::c_int,
        hmux2::src::cmd_kill_server::SIGTERM
    );
    constant!(
        "src/job.rs::SIGTERM",
        ::core::ffi::c_int,
        hmux2::src::job::SIGTERM
    );
    constant!(
        "src/proc.rs::SIGTERM",
        ::core::ffi::c_int,
        hmux2::src::proc::SIGTERM
    );
    constant!(
        "src/server.rs::SIGTERM",
        ::core::ffi::c_int,
        hmux2::src::server::SIGTERM
    );
    constant!(
        "src/client.rs::SIGTSTP",
        ::core::ffi::c_int,
        hmux2::src::client::SIGTSTP
    );
    constant!(
        "src/proc.rs::SIGTSTP",
        ::core::ffi::c_int,
        hmux2::src::proc::SIGTSTP
    );
    constant!(
        "src/job.rs::SIGTTIN",
        ::core::ffi::c_int,
        hmux2::src::job::SIGTTIN
    );
    constant!(
        "src/proc.rs::SIGTTIN",
        ::core::ffi::c_int,
        hmux2::src::proc::SIGTTIN
    );
    constant!(
        "src/server.rs::SIGTTIN",
        ::core::ffi::c_int,
        hmux2::src::server::SIGTTIN
    );
    constant!(
        "src/job.rs::SIGTTOU",
        ::core::ffi::c_int,
        hmux2::src::job::SIGTTOU
    );
    constant!(
        "src/proc.rs::SIGTTOU",
        ::core::ffi::c_int,
        hmux2::src::proc::SIGTTOU
    );
    constant!(
        "src/server.rs::SIGTTOU",
        ::core::ffi::c_int,
        hmux2::src::server::SIGTTOU
    );
    constant!(
        "src/proc.rs::SIGUSR1",
        ::core::ffi::c_int,
        hmux2::src::proc::SIGUSR1
    );
    constant!(
        "src/server.rs::SIGUSR1",
        ::core::ffi::c_int,
        hmux2::src::server::SIGUSR1
    );
    constant!(
        "src/proc.rs::SIGUSR2",
        ::core::ffi::c_int,
        hmux2::src::proc::SIGUSR2
    );
    constant!(
        "src/server.rs::SIGUSR2",
        ::core::ffi::c_int,
        hmux2::src::server::SIGUSR2
    );
    constant!(
        "src/client.rs::SIGWINCH",
        ::core::ffi::c_int,
        hmux2::src::client::SIGWINCH
    );
    constant!(
        "src/proc.rs::SIGWINCH",
        ::core::ffi::c_int,
        hmux2::src::proc::SIGWINCH
    );
    constant!(
        "src/cmd_pipe_pane.rs::SIG_BLOCK",
        ::core::ffi::c_int,
        hmux2::src::cmd_pipe_pane::SIG_BLOCK
    );
    constant!(
        "src/job.rs::SIG_BLOCK",
        ::core::ffi::c_int,
        hmux2::src::job::SIG_BLOCK
    );
    constant!(
        "src/server.rs::SIG_BLOCK",
        ::core::ffi::c_int,
        hmux2::src::server::SIG_BLOCK
    );
    constant!(
        "src/spawn.rs::SIG_BLOCK",
        ::core::ffi::c_int,
        hmux2::src::spawn::SIG_BLOCK
    );
    constant!(
        "src/client.rs::SIG_DFL",
        hmux2::src::client::__sighandler_t,
        hmux2::src::client::SIG_DFL
    );
    constant!(
        "src/proc.rs::SIG_DFL",
        hmux2::src::proc::__sighandler_t,
        hmux2::src::proc::SIG_DFL
    );
    constant!(
        "src/cmd_pipe_pane.rs::SIG_SETMASK",
        ::core::ffi::c_int,
        hmux2::src::cmd_pipe_pane::SIG_SETMASK
    );
    constant!(
        "src/job.rs::SIG_SETMASK",
        ::core::ffi::c_int,
        hmux2::src::job::SIG_SETMASK
    );
    constant!(
        "src/server.rs::SIG_SETMASK",
        ::core::ffi::c_int,
        hmux2::src::server::SIG_SETMASK
    );
    constant!(
        "src/spawn.rs::SIG_SETMASK",
        ::core::ffi::c_int,
        hmux2::src::spawn::SIG_SETMASK
    );
    layout!(
        "src/client.rs::__sighandler_t",
        hmux2::src::client::__sighandler_t,
        []
    );
    layout!(
        "src/proc.rs::__sighandler_t",
        hmux2::src::proc::__sighandler_t,
        []
    );
    layout!(
        "src/client.rs::__sigset_t",
        hmux2::src::client::__sigset_t,
        [__val]
    );
    layout!(
        "src/cmd_pipe_pane.rs::__sigset_t",
        hmux2::src::cmd_pipe_pane::__sigset_t,
        [__val]
    );
    layout!(
        "src/job.rs::__sigset_t",
        hmux2::src::job::__sigset_t,
        [__val]
    );
    layout!(
        "src/proc.rs::__sigset_t",
        hmux2::src::proc::__sigset_t,
        [__val]
    );
    layout!(
        "src/server.rs::__sigset_t",
        hmux2::src::server::__sigset_t,
        [__val]
    );
    layout!(
        "src/spawn.rs::__sigset_t",
        hmux2::src::spawn::__sigset_t,
        [__val]
    );
    layout!(
        "src/client.rs::__sigval_t",
        hmux2::src::client::__sigval_t,
        []
    );
    layout!("src/proc.rs::__sigval_t", hmux2::src::proc::__sigval_t, []);
    layout!(
        "src/client.rs::sigaction",
        hmux2::src::client::sigaction,
        [__sigaction_handler, sa_mask, sa_flags, sa_restorer]
    );
    layout!(
        "src/proc.rs::sigaction",
        hmux2::src::proc::sigaction,
        [__sigaction_handler, sa_mask, sa_flags, sa_restorer]
    );
    layout!(
        "src/client.rs::C2RustUnnamed_10",
        hmux2::src::client::sigaction___sigaction_handler,
        [sa_handler, sa_sigaction]
    );
    layout!(
        "src/proc.rs::C2RustUnnamed_9",
        hmux2::src::proc::sigaction___sigaction_handler,
        [sa_handler, sa_sigaction]
    );
    layout!(
        "src/client.rs::siginfo_t",
        hmux2::src::client::siginfo_t,
        [si_signo, si_errno, si_code, __pad0, _sifields]
    );
    layout!(
        "src/proc.rs::siginfo_t",
        hmux2::src::proc::siginfo_t,
        [si_signo, si_errno, si_code, __pad0, _sifields]
    );
    layout!(
        "src/client.rs::C2RustUnnamed_0",
        hmux2::src::client::siginfo_t__sifields,
        [_pad, _kill, _timer, _rt, _sigchld, _sigfault, _sigpoll, _sigsys]
    );
    layout!(
        "src/proc.rs::C2RustUnnamed",
        hmux2::src::proc::siginfo_t__sifields,
        [_pad, _kill, _timer, _rt, _sigchld, _sigfault, _sigpoll, _sigsys]
    );
    layout!(
        "src/client.rs::C2RustUnnamed_9",
        hmux2::src::client::siginfo_t__sifields__kill,
        [si_pid, si_uid]
    );
    layout!(
        "src/proc.rs::C2RustUnnamed_8",
        hmux2::src::proc::siginfo_t__sifields__kill,
        [si_pid, si_uid]
    );
    layout!(
        "src/client.rs::C2RustUnnamed_7",
        hmux2::src::client::siginfo_t__sifields__rt,
        [si_pid, si_uid, si_sigval]
    );
    layout!(
        "src/proc.rs::C2RustUnnamed_6",
        hmux2::src::proc::siginfo_t__sifields__rt,
        [si_pid, si_uid, si_sigval]
    );
    layout!(
        "src/client.rs::C2RustUnnamed_6",
        hmux2::src::client::siginfo_t__sifields__sigchld,
        [si_pid, si_uid, si_status, si_utime, si_stime]
    );
    layout!(
        "src/proc.rs::C2RustUnnamed_5",
        hmux2::src::proc::siginfo_t__sifields__sigchld,
        [si_pid, si_uid, si_status, si_utime, si_stime]
    );
    layout!(
        "src/client.rs::C2RustUnnamed_3",
        hmux2::src::client::siginfo_t__sifields__sigfault,
        [si_addr, si_addr_lsb, _bounds]
    );
    layout!(
        "src/proc.rs::C2RustUnnamed_2",
        hmux2::src::proc::siginfo_t__sifields__sigfault,
        [si_addr, si_addr_lsb, _bounds]
    );
    layout!(
        "src/client.rs::C2RustUnnamed_4",
        hmux2::src::client::siginfo_t__sifields__sigfault__bounds,
        [_addr_bnd, _pkey]
    );
    layout!(
        "src/proc.rs::C2RustUnnamed_3",
        hmux2::src::proc::siginfo_t__sifields__sigfault__bounds,
        [_addr_bnd, _pkey]
    );
    layout!(
        "src/client.rs::C2RustUnnamed_5",
        hmux2::src::client::siginfo_t__sifields__sigfault__bounds__addr_bnd,
        [_lower, _upper]
    );
    layout!(
        "src/proc.rs::C2RustUnnamed_4",
        hmux2::src::proc::siginfo_t__sifields__sigfault__bounds__addr_bnd,
        [_lower, _upper]
    );
    layout!(
        "src/client.rs::C2RustUnnamed_2",
        hmux2::src::client::siginfo_t__sifields__sigpoll,
        [si_band, si_fd]
    );
    layout!(
        "src/proc.rs::C2RustUnnamed_1",
        hmux2::src::proc::siginfo_t__sifields__sigpoll,
        [si_band, si_fd]
    );
    layout!(
        "src/client.rs::C2RustUnnamed_1",
        hmux2::src::client::siginfo_t__sifields__sigsys,
        [_call_addr, _syscall, _arch]
    );
    layout!(
        "src/proc.rs::C2RustUnnamed_0",
        hmux2::src::proc::siginfo_t__sifields__sigsys,
        [_call_addr, _syscall, _arch]
    );
    layout!(
        "src/client.rs::C2RustUnnamed_8",
        hmux2::src::client::siginfo_t__sifields__timer,
        [si_tid, si_overrun, si_sigval]
    );
    layout!(
        "src/proc.rs::C2RustUnnamed_7",
        hmux2::src::proc::siginfo_t__sifields__timer,
        [si_tid, si_overrun, si_sigval]
    );
    layout!("src/client.rs::sigset_t", hmux2::src::client::sigset_t, []);
    layout!(
        "src/cmd_pipe_pane.rs::sigset_t",
        hmux2::src::cmd_pipe_pane::sigset_t,
        []
    );
    layout!("src/job.rs::sigset_t", hmux2::src::job::sigset_t, []);
    layout!("src/proc.rs::sigset_t", hmux2::src::proc::sigset_t, []);
    layout!("src/server.rs::sigset_t", hmux2::src::server::sigset_t, []);
    layout!("src/spawn.rs::sigset_t", hmux2::src::spawn::sigset_t, []);
    layout!(
        "src/client.rs::sigval",
        hmux2::src::client::sigval,
        [sival_int, sival_ptr]
    );
    layout!(
        "src/proc.rs::sigval",
        hmux2::src::proc::sigval,
        [sival_int, sival_ptr]
    );
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/platform-signal.txt"));
}

#[test]
fn signal_callback_abi() {
    use hmux2::src::{client, proc};
    use std::ffi::{c_int, c_void};
    unsafe extern "C" fn handler(signal: c_int) {
        assert_eq!(signal, 15);
    }
    unsafe extern "C" fn info(signal: c_int, data: *mut client::siginfo_t, arg: *mut c_void) {
        assert_eq!(signal, 15);
        assert!(data.is_null() && arg.is_null());
    }
    unsafe extern "C" fn restore() {}
    let client_handler: client::__sighandler_t = Some(handler);
    let proc_handler: proc::__sighandler_t = client_handler;
    let mut action: client::sigaction = unsafe { std::mem::zeroed() };
    action.__sigaction_handler.sa_handler = proc_handler;
    unsafe {
        action.__sigaction_handler.sa_handler.unwrap()(15);
    }
    action.__sigaction_handler.sa_sigaction = Some(info);
    action.sa_restorer = Some(restore);
    let action: proc::sigaction = action;
    unsafe {
        action.__sigaction_handler.sa_sigaction.unwrap()(
            15,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        action.sa_restorer.unwrap()();
    }
    assert!(client::SIG_DFL.is_none() && proc::SIG_DFL.is_none());
}
