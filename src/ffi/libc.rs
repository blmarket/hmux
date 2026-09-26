//! Foreign declarations supplied by libc.

use crate::src::shared::abi::{
    __compar_fn_t, __gid_t, __int32_t, __mode_t, __off_t, __pid_t, __uid_t, clockid_t, size_t,
    socklen_t, speed_t, ssize_t, time_t, timeval,
};
use crate::src::shared::account::{group, passwd};
use crate::src::shared::posix_io::{glob_t, stat};
use crate::src::shared::posix_terminal::winsize;
use crate::src::shared::regex::{regex_t, regmatch_t};
use crate::src::shared::signal::{sigaction, sigset_t};
use crate::src::shared::socket::{__CONST_SOCKADDR_ARG, __SOCKADDR_ARG};
use crate::src::shared::stdio::FILE;
use crate::src::shared::terminal::termios;
use crate::src::shared::time::{timespec, tm};
use crate::src::shared::utf8::wchar_t;
pub type __useconds_t = ::core::ffi::c_uint;

pub type nl_item = ::core::ffi::c_int;

pub type __ssize_t = ::core::ffi::c_long;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct utsname {
    pub sysname: [::core::ffi::c_char; 65],
    pub nodename: [::core::ffi::c_char; 65],
    pub release: [::core::ffi::c_char; 65],
    pub version: [::core::ffi::c_char; 65],
    pub machine: [::core::ffi::c_char; 65],
    pub domainname: [::core::ffi::c_char; 65],
}

pub type nfds_t = ::core::ffi::c_ulong;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct pollfd {
    pub fd: ::core::ffi::c_int,
    pub events: ::core::ffi::c_short,
    pub revents: ::core::ffi::c_short,
}

pub use ::libc::msghdr;

extern "C" {
    pub type _IO_codecvt;
    pub type _IO_marker;
    pub type _IO_wide_data;
    pub fn __ctype_b_loc() -> *mut *const ::core::ffi::c_ushort;
    pub fn __ctype_get_mb_cur_max() -> size_t;
    pub fn __ctype_tolower_loc() -> *mut *const __int32_t;
    pub fn __ctype_toupper_loc() -> *mut *const __int32_t;
    pub fn __errno_location() -> *mut ::core::ffi::c_int;
    pub fn __getdelim(
        __lineptr: *mut *mut ::core::ffi::c_char,
        __n: *mut size_t,
        __delimiter: ::core::ffi::c_int,
        __stream: *mut FILE,
    ) -> __ssize_t;
    pub fn __xpg_basename(__path: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    pub fn _exit(__status: ::core::ffi::c_int) -> !;
    pub fn abort() -> !;
    pub fn abs(__x: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn accept(
        __fd: ::core::ffi::c_int,
        __addr: __SOCKADDR_ARG,
        __addr_len: *mut socklen_t,
    ) -> ::core::ffi::c_int;
    pub fn access(
        __name: *const ::core::ffi::c_char,
        __type: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    pub fn bind(
        __fd: ::core::ffi::c_int,
        __addr: __CONST_SOCKADDR_ARG,
        __len: socklen_t,
    ) -> ::core::ffi::c_int;
    pub fn calloc(__nmemb: size_t, __size: size_t) -> *mut ::core::ffi::c_void;
    pub fn cfgetispeed(__termios_p: *const termios) -> speed_t;
    pub fn cfgetospeed(__termios_p: *const termios) -> speed_t;
    pub fn cfmakeraw(__termios_p: *mut termios);
    pub fn cfsetispeed(__termios_p: *mut termios, __speed: speed_t) -> ::core::ffi::c_int;
    pub fn cfsetospeed(__termios_p: *mut termios, __speed: speed_t) -> ::core::ffi::c_int;
    pub fn chdir(__path: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    pub fn chmod(__file: *const ::core::ffi::c_char, __mode: __mode_t) -> ::core::ffi::c_int;
    pub fn clock_gettime(__clock_id: clockid_t, __tp: *mut timespec) -> ::core::ffi::c_int;
    pub fn close(__fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn closefrom(__lowfd: ::core::ffi::c_int);
    pub fn connect(
        __fd: ::core::ffi::c_int,
        __addr: __CONST_SOCKADDR_ARG,
        __len: socklen_t,
    ) -> ::core::ffi::c_int;
    pub fn ctime_r(
        __timer: *const time_t,
        __buf: *mut ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    pub fn daemon(
        __nochdir: ::core::ffi::c_int,
        __noclose: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    pub type dirent;
    pub fn dirname(__path: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    pub fn dup(__fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn dup2(__fd: ::core::ffi::c_int, __fd2: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub static mut environ: *mut *mut ::core::ffi::c_char;
    pub fn err(_: ::core::ffi::c_int, _: *const ::core::ffi::c_char, ...);
    pub fn errx(_: ::core::ffi::c_int, _: *const ::core::ffi::c_char, ...);
    pub fn execl(
        __path: *const ::core::ffi::c_char,
        __arg: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    pub fn execvp(
        __file: *const ::core::ffi::c_char,
        __argv: *const *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    pub fn exit(__status: ::core::ffi::c_int) -> !;
    pub fn explicit_bzero(__s: *mut ::core::ffi::c_void, __n: size_t);
    pub fn fclose(__stream: *mut FILE) -> ::core::ffi::c_int;
    pub fn fcntl(__fd: ::core::ffi::c_int, __cmd: ::core::ffi::c_int, ...) -> ::core::ffi::c_int;
    pub fn fdopen(__fd: ::core::ffi::c_int, __modes: *const ::core::ffi::c_char) -> *mut FILE;
    pub fn ferror(__stream: *mut FILE) -> ::core::ffi::c_int;
    pub fn fflush(__stream: *mut FILE) -> ::core::ffi::c_int;
    pub fn fgetc(__stream: *mut FILE) -> ::core::ffi::c_int;
    pub fn flock(__fd: ::core::ffi::c_int, __operation: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn fnmatch(
        __pattern: *const ::core::ffi::c_char,
        __name: *const ::core::ffi::c_char,
        __flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    pub fn fopen(
        __filename: *const ::core::ffi::c_char,
        __modes: *const ::core::ffi::c_char,
    ) -> *mut FILE;
    pub fn fork() -> __pid_t;
    pub fn fprintf(
        __stream: *mut FILE,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    pub fn fputc(__c: ::core::ffi::c_int, __stream: *mut FILE) -> ::core::ffi::c_int;
    pub fn fputs(__s: *const ::core::ffi::c_char, __stream: *mut FILE) -> ::core::ffi::c_int;
    pub fn fread(
        __ptr: *mut ::core::ffi::c_void,
        __size: size_t,
        __n: size_t,
        __stream: *mut FILE,
    ) -> ::core::ffi::c_ulong;
    pub fn free(__ptr: *mut ::core::ffi::c_void);
    pub fn fseeko(
        __stream: *mut FILE,
        __off: __off_t,
        __whence: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    pub fn ftello(__stream: *mut FILE) -> __off_t;
    pub fn fwrite(
        __ptr: *const ::core::ffi::c_void,
        __size: size_t,
        __n: size_t,
        __s: *mut FILE,
    ) -> ::core::ffi::c_ulong;
    pub fn getc(__stream: *mut FILE) -> ::core::ffi::c_int;
    pub fn getcwd(__buf: *mut ::core::ffi::c_char, __size: size_t) -> *mut ::core::ffi::c_char;
    pub fn getenv(__name: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    pub fn getgrgid(__gid: __gid_t) -> *mut group;
    pub fn getgrnam(__name: *const ::core::ffi::c_char) -> *mut group;
    pub fn gethostname(__name: *mut ::core::ffi::c_char, __len: size_t) -> ::core::ffi::c_int;
    pub fn getpagesize() -> ::core::ffi::c_int;
    pub fn getpid() -> __pid_t;
    pub fn getppid() -> __pid_t;
    pub fn getpwnam(__name: *const ::core::ffi::c_char) -> *mut passwd;
    pub fn getpwuid(__uid: __uid_t) -> *mut passwd;
    pub fn getsockname(
        __fd: ::core::ffi::c_int,
        __addr: __SOCKADDR_ARG,
        __len: *mut socklen_t,
    ) -> ::core::ffi::c_int;
    pub fn getsockopt(
        __fd: ::core::ffi::c_int,
        __level: ::core::ffi::c_int,
        __optname: ::core::ffi::c_int,
        __optval: *mut ::core::ffi::c_void,
        __optlen: *mut socklen_t,
    ) -> ::core::ffi::c_int;
    pub fn gettimeofday(__tv: *mut timeval, __tz: *mut ::core::ffi::c_void) -> ::core::ffi::c_int;
    pub fn getuid() -> __uid_t;
    pub fn glob(
        __pattern: *const ::core::ffi::c_char,
        __flags: ::core::ffi::c_int,
        __errfunc: Option<
            unsafe extern "C" fn(
                *const ::core::ffi::c_char,
                ::core::ffi::c_int,
            ) -> ::core::ffi::c_int,
        >,
        __pglob: *mut glob_t,
    ) -> ::core::ffi::c_int;
    pub fn globfree(__pglob: *mut glob_t);
    pub fn gmtime_r(__timer: *const time_t, __tp: *mut tm) -> *mut tm;
    pub fn ioctl(
        __fd: ::core::ffi::c_int,
        __request: ::core::ffi::c_ulong,
        ...
    ) -> ::core::ffi::c_int;
    pub fn isatty(__fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn kill(__pid: __pid_t, __sig: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn killpg(__pgrp: __pid_t, __sig: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn listen(__fd: ::core::ffi::c_int, __n: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn llabs(__x: ::core::ffi::c_longlong) -> ::core::ffi::c_longlong;
    pub fn localtime(__timer: *const time_t) -> *mut tm;
    pub fn localtime_r(__timer: *const time_t, __tp: *mut tm) -> *mut tm;
    pub fn lstat(__file: *const ::core::ffi::c_char, __buf: *mut stat) -> ::core::ffi::c_int;
    pub fn malloc(__size: size_t) -> *mut ::core::ffi::c_void;
    pub fn malloc_trim(__pad: size_t) -> ::core::ffi::c_int;
    pub fn mbtowc(
        __pwc: *mut wchar_t,
        __s: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    pub fn memchr(
        __s: *const ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    pub fn memcmp(
        __s1: *const ::core::ffi::c_void,
        __s2: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    pub fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    pub fn memmove(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    pub fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    pub fn mkdir(__path: *const ::core::ffi::c_char, __mode: __mode_t) -> ::core::ffi::c_int;
    pub fn mkstemp(__template: *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
    pub fn nl_langinfo(__item: nl_item) -> *mut ::core::ffi::c_char;
    pub fn open(
        __file: *const ::core::ffi::c_char,
        __oflag: ::core::ffi::c_int,
        ...
    ) -> ::core::ffi::c_int;
    pub fn poll(
        __fds: *mut pollfd,
        __nfds: nfds_t,
        __timeout: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    pub fn prctl(__option: ::core::ffi::c_int, ...) -> ::core::ffi::c_int;
    pub fn printf(__format: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
    pub static mut program_invocation_short_name: *mut ::core::ffi::c_char;
    pub fn qsort(
        __base: *mut ::core::ffi::c_void,
        __nmemb: size_t,
        __size: size_t,
        __compar: __compar_fn_t,
    );
    pub type re_dfa_t;
    pub fn readlink(
        __path: *const ::core::ffi::c_char,
        __buf: *mut ::core::ffi::c_char,
        __len: size_t,
    ) -> ssize_t;
    pub fn readv(
        __fd: ::core::ffi::c_int,
        __iovec: *const ::libc::iovec,
        __count: ::core::ffi::c_int,
    ) -> ssize_t;
    pub fn realloc(__ptr: *mut ::core::ffi::c_void, __size: size_t) -> *mut ::core::ffi::c_void;
    pub fn reallocarray(
        __ptr: *mut ::core::ffi::c_void,
        __nmemb: size_t,
        __size: size_t,
    ) -> *mut ::core::ffi::c_void;
    pub fn realpath(
        __name: *const ::core::ffi::c_char,
        __resolved: *mut ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    pub fn recvmsg(
        __fd: ::core::ffi::c_int,
        __message: *mut msghdr,
        __flags: ::core::ffi::c_int,
    ) -> ssize_t;
    pub fn regcomp(
        __preg: *mut regex_t,
        __pattern: *const ::core::ffi::c_char,
        __cflags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    pub fn regexec(
        __preg: *const regex_t,
        __String: *const ::core::ffi::c_char,
        __nmatch: size_t,
        __pmatch: *mut regmatch_t,
        __eflags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    pub fn regfree(__preg: *mut regex_t);
    pub fn sendmsg(
        __fd: ::core::ffi::c_int,
        __message: *const msghdr,
        __flags: ::core::ffi::c_int,
    ) -> ssize_t;
    pub fn setenv(
        __name: *const ::core::ffi::c_char,
        __value: *const ::core::ffi::c_char,
        __replace: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    pub fn setlocale(
        __category: ::core::ffi::c_int,
        __locale: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    pub fn setpgid(__pid: __pid_t, __pgid: __pid_t) -> ::core::ffi::c_int;
    pub fn setvbuf(
        __stream: *mut FILE,
        __buf: *mut ::core::ffi::c_char,
        __modes: ::core::ffi::c_int,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    pub fn shutdown(__fd: ::core::ffi::c_int, __how: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn sigaction(
        __sig: ::core::ffi::c_int,
        __act: *const sigaction,
        __oact: *mut sigaction,
    ) -> ::core::ffi::c_int;
    pub fn sigemptyset(__set: *mut sigset_t) -> ::core::ffi::c_int;
    pub fn sigfillset(__set: *mut sigset_t) -> ::core::ffi::c_int;
    pub fn sigprocmask(
        __how: ::core::ffi::c_int,
        __set: *const sigset_t,
        __oset: *mut sigset_t,
    ) -> ::core::ffi::c_int;
    pub fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    pub type sockaddr_at;
    pub type sockaddr_ax25;
    pub type sockaddr_dl;
    pub type sockaddr_eon;
    pub type sockaddr_inarp;
    pub type sockaddr_ipx;
    pub type sockaddr_iso;
    pub type sockaddr_ns;
    pub type sockaddr_x25;
    pub fn socket(
        __domain: ::core::ffi::c_int,
        __type: ::core::ffi::c_int,
        __protocol: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    pub fn socketpair(
        __domain: ::core::ffi::c_int,
        __type: ::core::ffi::c_int,
        __protocol: ::core::ffi::c_int,
        __fds: *mut ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    pub fn sscanf(
        __s: *const ::core::ffi::c_char,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    pub fn stat(__file: *const ::core::ffi::c_char, __buf: *mut stat) -> ::core::ffi::c_int;
    pub static mut stderr: *mut FILE;
    pub static mut stdout: *mut FILE;
    pub fn strcasecmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    pub fn strcasestr(
        __haystack: *const ::core::ffi::c_char,
        __needle: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    pub fn strchr(
        __s: *const ::core::ffi::c_char,
        __c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    pub fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    pub fn strcspn(
        __s: *const ::core::ffi::c_char,
        __reject: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_ulong;
    pub fn strdup(__s: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    pub fn strerror(__errnum: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    pub fn strftime(
        __s: *mut ::core::ffi::c_char,
        __maxsize: size_t,
        __format: *const ::core::ffi::c_char,
        __tp: *const tm,
    ) -> size_t;
    pub fn strlcat(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_ulong;
    pub fn strlcpy(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_ulong;
    pub fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    pub fn strncasecmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    pub fn strncmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    pub fn strndup(__string: *const ::core::ffi::c_char, __n: size_t) -> *mut ::core::ffi::c_char;
    pub fn strpbrk(
        __s: *const ::core::ffi::c_char,
        __accept: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    pub fn strrchr(
        __s: *const ::core::ffi::c_char,
        __c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    pub fn strsep(
        __stringp: *mut *mut ::core::ffi::c_char,
        __delim: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    pub fn strsignal(__sig: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    pub fn strspn(
        __s: *const ::core::ffi::c_char,
        __accept: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_ulong;
    pub fn strstr(
        __haystack: *const ::core::ffi::c_char,
        __needle: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    pub fn strtod(
        __nptr: *const ::core::ffi::c_char,
        __endptr: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_double;
    pub fn strtol(
        __nptr: *const ::core::ffi::c_char,
        __endptr: *mut *mut ::core::ffi::c_char,
        __base: ::core::ffi::c_int,
    ) -> ::core::ffi::c_long;
    pub fn strtoll(
        __nptr: *const ::core::ffi::c_char,
        __endptr: *mut *mut ::core::ffi::c_char,
        __base: ::core::ffi::c_int,
    ) -> ::core::ffi::c_longlong;
    pub fn strtoul(
        __nptr: *const ::core::ffi::c_char,
        __endptr: *mut *mut ::core::ffi::c_char,
        __base: ::core::ffi::c_int,
    ) -> ::core::ffi::c_ulong;
    pub fn strtoull(
        __nptr: *const ::core::ffi::c_char,
        __endptr: *mut *mut ::core::ffi::c_char,
        __base: ::core::ffi::c_int,
    ) -> ::core::ffi::c_ulonglong;
    pub fn system(__command: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    pub fn tcflush(
        __fd: ::core::ffi::c_int,
        __queue_selector: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    pub fn tcgetattr(__fd: ::core::ffi::c_int, __termios_p: *mut termios) -> ::core::ffi::c_int;
    pub fn tcgetpgrp(__fd: ::core::ffi::c_int) -> __pid_t;
    pub fn tcsetattr(
        __fd: ::core::ffi::c_int,
        __optional_actions: ::core::ffi::c_int,
        __termios_p: *const termios,
    ) -> ::core::ffi::c_int;
    pub fn time(__timer: *mut time_t) -> time_t;
    pub fn ttyname(__fd: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    pub fn tzset();
    pub fn umask(__mask: __mode_t) -> __mode_t;
    pub fn uname(__name: *mut utsname) -> ::core::ffi::c_int;
    pub fn ungetc(__c: ::core::ffi::c_int, __stream: *mut FILE) -> ::core::ffi::c_int;
    pub fn unlink(__name: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    pub fn unsetenv(__name: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    pub fn usleep(__useconds: __useconds_t) -> ::core::ffi::c_int;
    pub fn vasprintf(
        __ptr: *mut *mut ::core::ffi::c_char,
        __f: *const ::core::ffi::c_char,
        __arg: ::core::ffi::VaList,
    ) -> ::core::ffi::c_int;
    pub fn vsnprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        __arg: ::core::ffi::VaList,
    ) -> ::core::ffi::c_int;
    pub fn waitpid(
        __pid: __pid_t,
        __stat_loc: *mut ::core::ffi::c_int,
        __options: ::core::ffi::c_int,
    ) -> __pid_t;
    pub fn warnx(_: *const ::core::ffi::c_char, ...);
    pub fn wctomb(__s: *mut ::core::ffi::c_char, __wchar: wchar_t) -> ::core::ffi::c_int;
    pub fn write(
        __fd: ::core::ffi::c_int,
        __buf: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> ssize_t;
    pub fn writev(
        __fd: ::core::ffi::c_int,
        __iovec: *const ::libc::iovec,
        __count: ::core::ffi::c_int,
    ) -> ssize_t;
    pub fn forkpty(
        __amaster: *mut ::core::ffi::c_int,
        __name: *mut ::core::ffi::c_char,
        __termp: *const termios,
        __winp: *const winsize,
    ) -> ::core::ffi::c_int;
}
