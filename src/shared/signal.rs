//! Authoritative signal declarations from the translated Linux C ABI.
use super::abi::{__clock_t, __pid_t, __uid_t, __uint32_t};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ProcessSignal {
    Interrupt,
    Hangup,
    Child,
    Continue,
    Terminate,
    User1,
    User2,
    WindowChange,
    Other(::core::ffi::c_int),
}

impl ProcessSignal {
    pub fn from_raw(signal: ::core::ffi::c_int) -> Self {
        match signal {
            SIGINT => Self::Interrupt,
            SIGHUP => Self::Hangup,
            SIGCHLD => Self::Child,
            SIGCONT => Self::Continue,
            SIGTERM => Self::Terminate,
            SIGUSR1 => Self::User1,
            SIGUSR2 => Self::User2,
            SIGWINCH => Self::WindowChange,
            signal => Self::Other(signal),
        }
    }

    pub fn as_raw(self) -> ::core::ffi::c_int {
        match self {
            Self::Interrupt => SIGINT,
            Self::Hangup => SIGHUP,
            Self::Child => SIGCHLD,
            Self::Continue => SIGCONT,
            Self::Terminate => SIGTERM,
            Self::User1 => SIGUSR1,
            Self::User2 => SIGUSR2,
            Self::WindowChange => SIGWINCH,
            Self::Other(signal) => signal,
        }
    }
}

#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct __sigset_t {
    pub __val: [::core::ffi::c_ulong; 16],
}

pub type sigset_t = __sigset_t;

#[derive(Copy, Clone)]
#[repr(C)]
pub union sigval {
    pub sival_int: ::core::ffi::c_int,
    pub sival_ptr: *mut ::core::ffi::c_void,
}

pub type __sigval_t = sigval;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct siginfo_t {
    pub si_signo: ::core::ffi::c_int,
    pub si_errno: ::core::ffi::c_int,
    pub si_code: ::core::ffi::c_int,
    pub __pad0: ::core::ffi::c_int,
    pub _sifields: siginfo_t__sifields,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union siginfo_t__sifields {
    pub _pad: [::core::ffi::c_int; 28],
    pub _kill: siginfo_t__sifields__kill,
    pub _timer: siginfo_t__sifields__timer,
    pub _rt: siginfo_t__sifields__rt,
    pub _sigchld: siginfo_t__sifields__sigchld,
    pub _sigfault: siginfo_t__sifields__sigfault,
    pub _sigpoll: siginfo_t__sifields__sigpoll,
    pub _sigsys: siginfo_t__sifields__sigsys,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct siginfo_t__sifields__sigsys {
    pub _call_addr: *mut ::core::ffi::c_void,
    pub _syscall: ::core::ffi::c_int,
    pub _arch: ::core::ffi::c_uint,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct siginfo_t__sifields__sigpoll {
    pub si_band: ::core::ffi::c_long,
    pub si_fd: ::core::ffi::c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct siginfo_t__sifields__sigfault {
    pub si_addr: *mut ::core::ffi::c_void,
    pub si_addr_lsb: ::core::ffi::c_short,
    pub _bounds: siginfo_t__sifields__sigfault__bounds,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union siginfo_t__sifields__sigfault__bounds {
    pub _addr_bnd: siginfo_t__sifields__sigfault__bounds__addr_bnd,
    pub _pkey: __uint32_t,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct siginfo_t__sifields__sigfault__bounds__addr_bnd {
    pub _lower: *mut ::core::ffi::c_void,
    pub _upper: *mut ::core::ffi::c_void,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct siginfo_t__sifields__sigchld {
    pub si_pid: __pid_t,
    pub si_uid: __uid_t,
    pub si_status: ::core::ffi::c_int,
    pub si_utime: __clock_t,
    pub si_stime: __clock_t,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct siginfo_t__sifields__rt {
    pub si_pid: __pid_t,
    pub si_uid: __uid_t,
    pub si_sigval: __sigval_t,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct siginfo_t__sifields__timer {
    pub si_tid: ::core::ffi::c_int,
    pub si_overrun: ::core::ffi::c_int,
    pub si_sigval: __sigval_t,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct siginfo_t__sifields__kill {
    pub si_pid: __pid_t,
    pub si_uid: __uid_t,
}

pub type __sighandler_t = Option<unsafe extern "C" fn(::core::ffi::c_int) -> ()>;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct sigaction {
    pub __sigaction_handler: sigaction___sigaction_handler,
    pub sa_mask: __sigset_t,
    pub sa_flags: ::core::ffi::c_int,
    pub sa_restorer: Option<unsafe extern "C" fn() -> ()>,
}

impl Default for sigaction {
    fn default() -> Self {
        Self {
            __sigaction_handler: Default::default(),
            sa_mask: Default::default(),
            sa_flags: 0,
            sa_restorer: None,
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union sigaction___sigaction_handler {
    pub sa_handler: __sighandler_t,
    pub sa_sigaction: Option<
        unsafe extern "C" fn(::core::ffi::c_int, *mut siginfo_t, *mut ::core::ffi::c_void) -> (),
    >,
}

impl Default for sigaction___sigaction_handler {
    fn default() -> Self {
        Self { sa_handler: None }
    }
}

pub const SIG_DFL: __sighandler_t = None;

pub const SIGTERM: ::core::ffi::c_int = 15;

pub const SIGHUP: ::core::ffi::c_int = 1 as ::core::ffi::c_int;

pub const SIGTSTP: ::core::ffi::c_int = 20 as ::core::ffi::c_int;

pub const SIGCONT: ::core::ffi::c_int = 18;

pub const SIGCHLD: ::core::ffi::c_int = 17 as ::core::ffi::c_int;

pub const SIGWINCH: ::core::ffi::c_int = 28;

pub const SA_RESTART: ::core::ffi::c_int = 0x10000000 as ::core::ffi::c_int;

pub const SIG_BLOCK: ::core::ffi::c_int = 0 as ::core::ffi::c_int;

pub const SIG_SETMASK: ::core::ffi::c_int = 2 as ::core::ffi::c_int;

pub const SIGTTIN: ::core::ffi::c_int = 21 as ::core::ffi::c_int;

pub const SIGTTOU: ::core::ffi::c_int = 22 as ::core::ffi::c_int;

pub const SIGINT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;

pub const SIGUSR1: ::core::ffi::c_int = 10 as ::core::ffi::c_int;

pub const SIGUSR2: ::core::ffi::c_int = 12 as ::core::ffi::c_int;
