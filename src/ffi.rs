#![allow(unused_imports)]
pub use crate::types::*;
pub use libc::{accept, bind, connect, getsockname, readv, writev};

#[link(name = "resolv", kind = "dylib", modifiers = "-as-needed")]
unsafe extern "C" {
    pub fn __b64_ntop(
        _: *const core::ffi::c_uchar,
        _: size_t,
        _: *mut core::ffi::c_char,
        _: size_t,
    ) -> core::ffi::c_int;
    pub fn __b64_pton(
        _: *const core::ffi::c_char,
        _: *mut core::ffi::c_uchar,
        _: size_t,
    ) -> core::ffi::c_int;
}

unsafe extern "C" {
    pub static mut cur_term: *mut TERMINAL;
    pub static mut environ: *mut *mut core::ffi::c_char;
    pub static mut program_invocation_short_name: *mut core::ffi::c_char;
    pub static mut stderr: *mut FILE;
    pub static mut stdin: *mut FILE;
    pub static mut stdout: *mut FILE;
    pub fn __ctype_get_mb_cur_max() -> size_t;
    pub fn __errno_location() -> *mut core::ffi::c_int;
    pub fn __xpg_basename(__path: *mut core::ffi::c_char) -> *mut core::ffi::c_char;
    pub fn _exit(__status: core::ffi::c_int) -> !;
    pub fn abort() -> !;
    pub fn abs(__x: core::ffi::c_int) -> core::ffi::c_int;
    pub fn access(__name: *const core::ffi::c_char, __type: core::ffi::c_int) -> core::ffi::c_int;
    pub fn calloc(__nmemb: size_t, __size: size_t) -> *mut core::ffi::c_void;
    pub fn cfgetispeed(__termios_p: *const termios) -> speed_t;
    pub fn cfgetospeed(__termios_p: *const termios) -> speed_t;
    pub fn cfmakeraw(__termios_p: *mut termios);
    pub fn cfsetispeed(__termios_p: *mut termios, __speed: speed_t) -> core::ffi::c_int;
    pub fn cfsetospeed(__termios_p: *mut termios, __speed: speed_t) -> core::ffi::c_int;
    pub fn chdir(__path: *const core::ffi::c_char) -> core::ffi::c_int;
    pub fn close(__fd: core::ffi::c_int) -> core::ffi::c_int;
    pub fn closefrom(__lowfd: core::ffi::c_int);
    pub fn ctime_r(__timer: *const time_t, __buf: *mut core::ffi::c_char)
    -> *mut core::ffi::c_char;
    pub fn daemon(__nochdir: core::ffi::c_int, __noclose: core::ffi::c_int) -> core::ffi::c_int;
    pub fn del_curterm(_: *mut TERMINAL) -> core::ffi::c_int;
    pub fn dirname(__path: *mut core::ffi::c_char) -> *mut core::ffi::c_char;
    pub fn dup(__fd: core::ffi::c_int) -> core::ffi::c_int;
    pub fn dup2(__fd: core::ffi::c_int, __fd2: core::ffi::c_int) -> core::ffi::c_int;
    pub fn err(_: core::ffi::c_int, _: *const core::ffi::c_char, ...);
    pub fn errx(_: core::ffi::c_int, _: *const core::ffi::c_char, ...);
    pub fn execl(
        __path: *const core::ffi::c_char,
        __arg: *const core::ffi::c_char,
        ...
    ) -> core::ffi::c_int;
    pub fn execvp(
        __file: *const core::ffi::c_char,
        __argv: *const *mut core::ffi::c_char,
    ) -> core::ffi::c_int;
    pub fn exit(__status: core::ffi::c_int) -> !;
    pub fn explicit_bzero(__s: *mut core::ffi::c_void, __n: size_t);
    pub fn fabs(__x: core::ffi::c_double) -> core::ffi::c_double;
    pub fn fclose(__stream: *mut FILE) -> core::ffi::c_int;
    pub fn fcntl(__fd: core::ffi::c_int, __cmd: core::ffi::c_int, ...) -> core::ffi::c_int;
    pub fn flock(__fd: core::ffi::c_int, __operation: core::ffi::c_int) -> core::ffi::c_int;
    pub fn fmod(__x: core::ffi::c_double, __y: core::ffi::c_double) -> core::ffi::c_double;
    pub fn fnmatch(
        __pattern: *const core::ffi::c_char,
        __name: *const core::ffi::c_char,
        __flags: core::ffi::c_int,
    ) -> core::ffi::c_int;
    pub fn fopen(
        __filename: *const core::ffi::c_char,
        __modes: *const core::ffi::c_char,
    ) -> *mut FILE;
    pub fn fork() -> __pid_t;
    pub fn forkpty(
        __amaster: *mut core::ffi::c_int,
        __name: *mut core::ffi::c_char,
        __termp: *const termios,
        __winp: *const winsize,
    ) -> core::ffi::c_int;
    pub fn free(__ptr: *mut core::ffi::c_void);
    pub fn getc(__stream: *mut FILE) -> core::ffi::c_int;
    pub fn getcwd(__buf: *mut core::ffi::c_char, __size: size_t) -> *mut core::ffi::c_char;
    pub fn getenv(__name: *const core::ffi::c_char) -> *mut core::ffi::c_char;
    pub fn gethostname(__name: *mut core::ffi::c_char, __len: size_t) -> core::ffi::c_int;
    pub fn getpagesize() -> core::ffi::c_int;
    pub fn getpid() -> __pid_t;
    pub fn getppid() -> __pid_t;
    pub fn getsockopt(
        __fd: core::ffi::c_int,
        __level: core::ffi::c_int,
        __optname: core::ffi::c_int,
        __optval: *mut core::ffi::c_void,
        __optlen: *mut socklen_t,
    ) -> core::ffi::c_int;
    pub fn getuid() -> __uid_t;
    pub fn ioctl(__fd: core::ffi::c_int, __request: core::ffi::c_ulong, ...) -> core::ffi::c_int;
    pub fn isatty(__fd: core::ffi::c_int) -> core::ffi::c_int;
    pub fn kill(__pid: __pid_t, __sig: core::ffi::c_int) -> core::ffi::c_int;
    pub fn killpg(__pgrp: __pid_t, __sig: core::ffi::c_int) -> core::ffi::c_int;
    pub fn listen(__fd: core::ffi::c_int, __n: core::ffi::c_int) -> core::ffi::c_int;
    pub fn malloc(__size: size_t) -> *mut core::ffi::c_void;
    pub fn malloc_trim(__pad: size_t) -> core::ffi::c_int;
    pub fn mbtowc(
        __pwc: *mut wchar_t,
        __s: *const core::ffi::c_char,
        __n: size_t,
    ) -> core::ffi::c_int;
    pub fn mkstemp(__template: *mut core::ffi::c_char) -> core::ffi::c_int;
    pub fn nl_langinfo(__item: nl_item) -> *mut core::ffi::c_char;
    pub fn open(
        __file: *const core::ffi::c_char,
        __oflag: core::ffi::c_int,
        ...
    ) -> core::ffi::c_int;
    pub fn prctl(__option: core::ffi::c_int, ...) -> core::ffi::c_int;
    pub fn printf(__format: *const core::ffi::c_char, ...) -> core::ffi::c_int;
    pub fn sd_is_socket_unix(
        fd: core::ffi::c_int,
        type_0: core::ffi::c_int,
        listening: core::ffi::c_int,
        path: *const core::ffi::c_char,
        length: size_t,
    ) -> core::ffi::c_int;
    pub fn sd_listen_fds(unset_environment: core::ffi::c_int) -> core::ffi::c_int;
    pub fn sd_pid_get_unit(pid: pid_t, ret_unit: *mut *mut core::ffi::c_char) -> core::ffi::c_int;
    pub fn sd_pid_get_user_slice(
        pid: pid_t,
        ret_slice: *mut *mut core::ffi::c_char,
    ) -> core::ffi::c_int;
    pub fn sd_pid_get_user_unit(
        pid: pid_t,
        ret_unit: *mut *mut core::ffi::c_char,
    ) -> core::ffi::c_int;
    pub fn setenv(
        __name: *const core::ffi::c_char,
        __value: *const core::ffi::c_char,
        __replace: core::ffi::c_int,
    ) -> core::ffi::c_int;
    pub fn setlocale(
        __category: core::ffi::c_int,
        __locale: *const core::ffi::c_char,
    ) -> *mut core::ffi::c_char;
    pub fn setpgid(__pid: __pid_t, __pgid: __pid_t) -> core::ffi::c_int;
    pub fn setupterm(
        _: *const core::ffi::c_char,
        _: core::ffi::c_int,
        _: *mut core::ffi::c_int,
    ) -> core::ffi::c_int;
    pub fn setvbuf(
        __stream: *mut FILE,
        __buf: *mut core::ffi::c_char,
        __modes: core::ffi::c_int,
        __n: size_t,
    ) -> core::ffi::c_int;
    pub fn shutdown(__fd: core::ffi::c_int, __how: core::ffi::c_int) -> core::ffi::c_int;
    pub fn sigaction(
        __sig: core::ffi::c_int,
        __act: *const libc::sigaction,
        __oact: *mut libc::sigaction,
    ) -> core::ffi::c_int;
    pub fn sigemptyset(__set: *mut libc::sigset_t) -> core::ffi::c_int;
    pub fn sigfillset(__set: *mut sigset_t) -> core::ffi::c_int;
    pub fn sigprocmask(
        __how: core::ffi::c_int,
        __set: *const sigset_t,
        __oset: *mut sigset_t,
    ) -> core::ffi::c_int;
    pub fn snprintf(
        __s: *mut core::ffi::c_char,
        __maxlen: size_t,
        __format: *const core::ffi::c_char,
        ...
    ) -> core::ffi::c_int;
    pub fn socket(
        __domain: core::ffi::c_int,
        __type: core::ffi::c_int,
        __protocol: core::ffi::c_int,
    ) -> core::ffi::c_int;
    pub fn socketpair(
        __domain: core::ffi::c_int,
        __type: core::ffi::c_int,
        __protocol: core::ffi::c_int,
        __fds: *mut core::ffi::c_int,
    ) -> core::ffi::c_int;
    pub fn sscanf(
        __s: *const core::ffi::c_char,
        __format: *const core::ffi::c_char,
        ...
    ) -> core::ffi::c_int;
    pub fn strchr(__s: *const core::ffi::c_char, __c: core::ffi::c_int) -> *mut core::ffi::c_char;
    pub fn strcmp(
        __s1: *const core::ffi::c_char,
        __s2: *const core::ffi::c_char,
    ) -> core::ffi::c_int;
    pub fn strcspn(
        __s: *const core::ffi::c_char,
        __reject: *const core::ffi::c_char,
    ) -> core::ffi::c_ulong;
    pub fn strlcat(
        __dest: *mut core::ffi::c_char,
        __src: *const core::ffi::c_char,
        __n: size_t,
    ) -> core::ffi::c_ulong;
    pub fn strlcpy(
        __dest: *mut core::ffi::c_char,
        __src: *const core::ffi::c_char,
        __n: size_t,
    ) -> core::ffi::c_ulong;
    pub fn strlen(__s: *const core::ffi::c_char) -> size_t;
    pub fn strncmp(
        __s1: *const core::ffi::c_char,
        __s2: *const core::ffi::c_char,
        __n: size_t,
    ) -> core::ffi::c_int;
    pub fn strpbrk(
        __s: *const core::ffi::c_char,
        __accept: *const core::ffi::c_char,
    ) -> *mut core::ffi::c_char;
    pub fn strrchr(__s: *const core::ffi::c_char, __c: core::ffi::c_int) -> *mut core::ffi::c_char;
    pub fn strsep(
        __stringp: *mut *mut core::ffi::c_char,
        __delim: *const core::ffi::c_char,
    ) -> *mut core::ffi::c_char;
    pub fn strtod(
        __nptr: *const core::ffi::c_char,
        __endptr: *mut *mut core::ffi::c_char,
    ) -> core::ffi::c_double;
    pub fn strtol(
        __nptr: *const core::ffi::c_char,
        __endptr: *mut *mut core::ffi::c_char,
        __base: core::ffi::c_int,
    ) -> core::ffi::c_long;
    pub fn strtoll(
        __nptr: *const core::ffi::c_char,
        __endptr: *mut *mut core::ffi::c_char,
        __base: core::ffi::c_int,
    ) -> core::ffi::c_longlong;
    pub fn strtoul(
        __nptr: *const core::ffi::c_char,
        __endptr: *mut *mut core::ffi::c_char,
        __base: core::ffi::c_int,
    ) -> core::ffi::c_ulong;
    pub fn strtoull(
        __nptr: *const core::ffi::c_char,
        __endptr: *mut *mut core::ffi::c_char,
        __base: core::ffi::c_int,
    ) -> core::ffi::c_ulonglong;
    pub fn system(__command: *const core::ffi::c_char) -> core::ffi::c_int;
    pub fn tcflush(__fd: core::ffi::c_int, __queue_selector: core::ffi::c_int) -> core::ffi::c_int;
    pub fn tcgetattr(__fd: core::ffi::c_int, __termios_p: *mut termios) -> core::ffi::c_int;
    pub fn tcgetpgrp(__fd: core::ffi::c_int) -> __pid_t;
    pub fn tcsetattr(
        __fd: core::ffi::c_int,
        __optional_actions: core::ffi::c_int,
        __termios_p: *const termios,
    ) -> core::ffi::c_int;
    pub fn tigetflag(_: *const core::ffi::c_char) -> core::ffi::c_int;
    pub fn tigetnum(_: *const core::ffi::c_char) -> core::ffi::c_int;
    pub fn tigetstr(_: *const core::ffi::c_char) -> *mut core::ffi::c_char;
    pub fn time(__timer: *mut time_t) -> time_t;
    pub fn tiparm_s(
        _: core::ffi::c_int,
        _: core::ffi::c_int,
        _: *const core::ffi::c_char,
        ...
    ) -> *mut core::ffi::c_char;
    pub fn tzset();
    pub fn umask(__mask: __mode_t) -> __mode_t;
    pub fn uname(__name: *mut utsname) -> core::ffi::c_int;
    #[cfg(test)]
    pub fn unsetenv(__name: *const core::ffi::c_char) -> core::ffi::c_int;
    pub fn usleep(__useconds: __useconds_t) -> core::ffi::c_int;
    pub fn utempter_add_record(
        master_fd: core::ffi::c_int,
        hostname: *const core::ffi::c_char,
    ) -> core::ffi::c_int;
    pub fn utempter_remove_record(master_fd: core::ffi::c_int) -> core::ffi::c_int;
    pub fn utf8proc_version() -> *const core::ffi::c_char;
    pub fn waitpid(
        __pid: __pid_t,
        __stat_loc: *mut core::ffi::c_int,
        __options: core::ffi::c_int,
    ) -> __pid_t;
    pub fn warnx(_: *const core::ffi::c_char, ...);
    pub fn wctomb(__s: *mut core::ffi::c_char, __wchar: wchar_t) -> core::ffi::c_int;
    pub fn write(__fd: core::ffi::c_int, __buf: *const core::ffi::c_void, __n: size_t) -> ssize_t;
}
