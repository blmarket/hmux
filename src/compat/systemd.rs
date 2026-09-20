use crate::src::shared::abi::*;
extern "C" {
    pub type sd_bus;
    pub type sd_bus_message;
    pub type sd_bus_slot;
    pub type sockaddr_x25;
    pub type sockaddr_ns;
    pub type sockaddr_iso;
    pub type sockaddr_ipx;
    pub type sockaddr_inarp;
    pub type sockaddr_eon;
    pub type sockaddr_dl;
    pub type sockaddr_ax25;
    pub type sockaddr_at;
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strerror(__errnum: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn __errno_location() -> *mut ::core::ffi::c_int;
    fn getpid() -> __pid_t;
    fn getppid() -> __pid_t;
    fn sd_id128_randomize(ret: *mut sd_id128_t) -> ::core::ffi::c_int;
    fn sd_bus_default_user(ret: *mut *mut sd_bus) -> ::core::ffi::c_int;
    fn sd_bus_unref(p: *mut sd_bus) -> *mut sd_bus;
    fn sd_bus_call(
        bus: *mut sd_bus,
        m: *mut sd_bus_message,
        usec: uint64_t,
        reterr_error: *mut sd_bus_error,
        ret_reply: *mut *mut sd_bus_message,
    ) -> ::core::ffi::c_int;
    fn sd_bus_process(bus: *mut sd_bus, ret: *mut *mut sd_bus_message) -> ::core::ffi::c_int;
    fn sd_bus_wait(bus: *mut sd_bus, timeout_usec: uint64_t) -> ::core::ffi::c_int;
    fn sd_bus_slot_unref(p: *mut sd_bus_slot) -> *mut sd_bus_slot;
    fn sd_bus_message_new_method_call(
        bus: *mut sd_bus,
        ret: *mut *mut sd_bus_message,
        destination: *const ::core::ffi::c_char,
        path: *const ::core::ffi::c_char,
        interface: *const ::core::ffi::c_char,
        member: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn sd_bus_message_unref(p: *mut sd_bus_message) -> *mut sd_bus_message;
    fn sd_bus_message_append(
        m: *mut sd_bus_message,
        types: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn sd_bus_message_open_container(
        m: *mut sd_bus_message,
        type_0: ::core::ffi::c_char,
        contents: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn sd_bus_message_close_container(m: *mut sd_bus_message) -> ::core::ffi::c_int;
    fn sd_bus_message_read(
        m: *mut sd_bus_message,
        types: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn sd_bus_match_signal(
        bus: *mut sd_bus,
        ret: *mut *mut sd_bus_slot,
        sender: *const ::core::ffi::c_char,
        path: *const ::core::ffi::c_char,
        interface: *const ::core::ffi::c_char,
        member: *const ::core::ffi::c_char,
        callback: sd_bus_message_handler_t,
        userdata: *mut ::core::ffi::c_void,
    ) -> ::core::ffi::c_int;
    fn sd_bus_error_free(e: *mut sd_bus_error);
    fn getsockname(
        __fd: ::core::ffi::c_int,
        __addr: __SOCKADDR_ARG,
        __len: *mut socklen_t,
    ) -> ::core::ffi::c_int;
    fn sd_listen_fds(unset_environment: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn sd_is_socket_unix(
        fd: ::core::ffi::c_int,
        type_0: ::core::ffi::c_int,
        listening: ::core::ffi::c_int,
        path: *const ::core::ffi::c_char,
        length: size_t,
    ) -> ::core::ffi::c_int;
    fn sd_pid_get_unit(pid: pid_t, ret_unit: *mut *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn sd_pid_get_user_unit(
        pid: pid_t,
        ret_unit: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn sd_pid_get_user_slice(
        pid: pid_t,
        ret_slice: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn gettimeofday(__tv: *mut timeval, __tz: *mut ::core::ffi::c_void) -> ::core::ffi::c_int;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn xasprintf(
        _: *mut *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    static mut socket_path: *const ::core::ffi::c_char;
    fn server_create_socket(_: uint64_t, _: *mut *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
}
pub type __uint16_t = u16;
pub type __uint32_t = u32;
pub type __socklen_t = ::core::ffi::c_uint;
pub type sa_family_t = ::core::ffi::c_ushort;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sockaddr_un {
    pub sun_family: sa_family_t,
    pub sun_path: [::core::ffi::c_char; 108],
}
pub type uint16_t = __uint16_t;
pub type uint32_t = __uint32_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sd_bus_error {
    pub name: *const ::core::ffi::c_char,
    pub message: *const ::core::ffi::c_char,
    pub _need_free: ::core::ffi::c_int,
}
pub type sd_bus_message_handler_t = Option<
    unsafe extern "C" fn(
        *mut sd_bus_message,
        *mut ::core::ffi::c_void,
        *mut sd_bus_error,
    ) -> ::core::ffi::c_int,
>;
pub type socklen_t = __socklen_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub union sd_id128 {
    pub bytes: [uint8_t; 16],
    pub qwords: [uint64_t; 2],
}
pub type sd_id128_t = sd_id128;
pub type __socket_type = ::core::ffi::c_uint;
pub const SOCK_NONBLOCK: __socket_type = 2048;
pub const SOCK_CLOEXEC: __socket_type = 524288;
pub const SOCK_PACKET: __socket_type = 10;
pub const SOCK_DCCP: __socket_type = 6;
pub const SOCK_SEQPACKET: __socket_type = 5;
pub const SOCK_RDM: __socket_type = 4;
pub const SOCK_RAW: __socket_type = 3;
pub const SOCK_DGRAM: __socket_type = 2;
pub const SOCK_STREAM: __socket_type = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sockaddr {
    pub sa_family: sa_family_t,
    pub sa_data: [::core::ffi::c_char; 14],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union __SOCKADDR_ARG {
    pub __sockaddr__: *mut sockaddr,
    pub __sockaddr_at__: *mut sockaddr_at,
    pub __sockaddr_ax25__: *mut sockaddr_ax25,
    pub __sockaddr_dl__: *mut sockaddr_dl,
    pub __sockaddr_eon__: *mut sockaddr_eon,
    pub __sockaddr_in__: *mut sockaddr_in,
    pub __sockaddr_in6__: *mut sockaddr_in6,
    pub __sockaddr_inarp__: *mut sockaddr_inarp,
    pub __sockaddr_ipx__: *mut sockaddr_ipx,
    pub __sockaddr_iso__: *mut sockaddr_iso,
    pub __sockaddr_ns__: *mut sockaddr_ns,
    pub __sockaddr_un__: *mut sockaddr_un,
    pub __sockaddr_x25__: *mut sockaddr_x25,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sockaddr_in6 {
    pub sin6_family: sa_family_t,
    pub sin6_port: in_port_t,
    pub sin6_flowinfo: uint32_t,
    pub sin6_addr: in6_addr,
    pub sin6_scope_id: uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct in6_addr {
    pub __in6_u: C2RustUnnamed,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed {
    pub __u6_addr8: [uint8_t; 16],
    pub __u6_addr16: [uint16_t; 8],
    pub __u6_addr32: [uint32_t; 4],
}
pub type in_port_t = uint16_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sockaddr_in {
    pub sin_family: sa_family_t,
    pub sin_port: in_port_t,
    pub sin_addr: in_addr,
    pub sin_zero: [::core::ffi::c_uchar; 8],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct in_addr {
    pub s_addr: in_addr_t,
}
pub type in_addr_t = uint32_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct systemd_job_watch {
    pub path: *const ::core::ffi::c_char,
    pub done: ::core::ffi::c_int,
}
pub const EPFNOSUPPORT: ::core::ffi::c_int = 96 as ::core::ffi::c_int;
pub const E2BIG: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const SD_BUS_ERROR_NULL: sd_bus_error = sd_bus_error {
    name: ::core::ptr::null::<::core::ffi::c_char>(),
    message: ::core::ptr::null::<::core::ffi::c_char>(),
    _need_free: 0 as ::core::ffi::c_int,
};
pub const SD_LISTEN_FDS_START: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn systemd_activated() -> ::core::ffi::c_int {
    return (sd_listen_fds(0 as ::core::ffi::c_int) >= 1 as ::core::ffi::c_int)
        as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn systemd_create_socket(
    mut flags: ::core::ffi::c_int,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut fds: ::core::ffi::c_int = 0;
    let mut fd: ::core::ffi::c_int = 0;
    let mut sa: sockaddr_un = sockaddr_un {
        sun_family: 0,
        sun_path: [0; 108],
    };
    let mut addrlen: socklen_t = ::core::mem::size_of::<sockaddr_un>() as socklen_t;
    fds = sd_listen_fds(0 as ::core::ffi::c_int);
    if fds > 1 as ::core::ffi::c_int {
        *__errno_location() = E2BIG;
    } else if fds == 1 as ::core::ffi::c_int {
        fd = SD_LISTEN_FDS_START;
        if sd_is_socket_unix(
            fd,
            SOCK_STREAM as ::core::ffi::c_int,
            1 as ::core::ffi::c_int,
            ::core::ptr::null::<::core::ffi::c_char>(),
            0 as size_t,
        ) == 0
        {
            *__errno_location() = EPFNOSUPPORT;
        } else if !(getsockname(
            fd,
            __SOCKADDR_ARG {
                __sockaddr__: &raw mut sa as *mut sockaddr,
            },
            &raw mut addrlen,
        ) == -(1 as ::core::ffi::c_int))
        {
            socket_path = xstrdup(&raw mut sa.sun_path as *mut ::core::ffi::c_char);
            return fd;
        }
    } else {
        return server_create_socket(flags as uint64_t, cause);
    }
    if !cause.is_null() {
        xasprintf(
            cause,
            b"systemd socket error (%s)\0" as *const u8 as *const ::core::ffi::c_char,
            strerror(*__errno_location()),
        );
    }
    return -(1 as ::core::ffi::c_int);
}
unsafe extern "C" fn job_removed_handler(
    mut m: *mut sd_bus_message,
    mut userdata: *mut ::core::ffi::c_void,
    mut ret_error: *mut sd_bus_error,
) -> ::core::ffi::c_int {
    let mut watch: *mut systemd_job_watch = userdata as *mut systemd_job_watch;
    let mut path: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut id: uint32_t = 0;
    let mut r: ::core::ffi::c_int = 0;
    if (*watch).path.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    r = sd_bus_message_read(
        m,
        b"uo\0" as *const u8 as *const ::core::ffi::c_char,
        &raw mut id,
        &raw mut path,
    );
    if r < 0 as ::core::ffi::c_int {
        return r;
    }
    if strcmp(path, (*watch).path) == 0 as ::core::ffi::c_int {
        (*watch).done = 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn systemd_move_to_new_cgroup(
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut current_block: u64;
    let mut error: sd_bus_error = SD_BUS_ERROR_NULL;
    let mut m: *mut sd_bus_message = ::core::ptr::null_mut::<sd_bus_message>();
    let mut reply: *mut sd_bus_message = ::core::ptr::null_mut::<sd_bus_message>();
    let mut bus: *mut sd_bus = ::core::ptr::null_mut::<sd_bus>();
    let mut slot: *mut sd_bus_slot = ::core::ptr::null_mut::<sd_bus_slot>();
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut desc: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut slice: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut unit: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut uuid: sd_id128_t = sd_id128 { bytes: [0; 16] };
    let mut r: ::core::ffi::c_int = 0;
    let mut elapsed_usec: uint64_t = 0;
    let mut pid: pid_t = 0;
    let mut parent_pid: pid_t = 0;
    let mut start: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    let mut now: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    let mut watch: systemd_job_watch = systemd_job_watch {
        path: ::core::ptr::null::<::core::ffi::c_char>(),
        done: 0,
    };
    gettimeofday(&raw mut start, NULL);
    r = sd_bus_default_user(&raw mut bus);
    if r < 0 as ::core::ffi::c_int {
        xasprintf(
            cause,
            b"failed to connect to session bus: %s\0" as *const u8 as *const ::core::ffi::c_char,
            strerror(-r),
        );
    } else {
        r = sd_bus_match_signal(
            bus,
            &raw mut slot,
            b"org.freedesktop.systemd1\0" as *const u8 as *const ::core::ffi::c_char,
            b"/org/freedesktop/systemd1\0" as *const u8 as *const ::core::ffi::c_char,
            b"org.freedesktop.systemd1.Manager\0" as *const u8 as *const ::core::ffi::c_char,
            b"JobRemoved\0" as *const u8 as *const ::core::ffi::c_char,
            Some(
                job_removed_handler
                    as unsafe extern "C" fn(
                        *mut sd_bus_message,
                        *mut ::core::ffi::c_void,
                        *mut sd_bus_error,
                    ) -> ::core::ffi::c_int,
            ),
            &raw mut watch as *mut ::core::ffi::c_void,
        );
        if r < 0 as ::core::ffi::c_int {
            xasprintf(
                cause,
                b"failed to create match signal: %s\0" as *const u8 as *const ::core::ffi::c_char,
                strerror(-r),
            );
        } else {
            r = sd_bus_message_new_method_call(
                bus,
                &raw mut m,
                b"org.freedesktop.systemd1\0" as *const u8 as *const ::core::ffi::c_char,
                b"/org/freedesktop/systemd1\0" as *const u8 as *const ::core::ffi::c_char,
                b"org.freedesktop.systemd1.Manager\0" as *const u8 as *const ::core::ffi::c_char,
                b"StartTransientUnit\0" as *const u8 as *const ::core::ffi::c_char,
            );
            if r < 0 as ::core::ffi::c_int {
                xasprintf(
                    cause,
                    b"failed to create bus message: %s\0" as *const u8
                        as *const ::core::ffi::c_char,
                    strerror(-r),
                );
            } else {
                r = sd_id128_randomize(&raw mut uuid);
                if r < 0 as ::core::ffi::c_int {
                    xasprintf(
                        cause,
                        b"failed to generate uuid: %s\0" as *const u8 as *const ::core::ffi::c_char,
                        strerror(-r),
                    );
                } else {
                    xasprintf(
                        &raw mut name,
                        b"tmux-spawn-%02x%02x%02x%02x-%02x%02x-%02x%02x-%02x%02x-%02x%02x%02x%02x%02x%02x.scope\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        uuid.bytes[0 as ::core::ffi::c_int as usize]
                            as ::core::ffi::c_int,
                        uuid.bytes[1 as ::core::ffi::c_int as usize]
                            as ::core::ffi::c_int,
                        uuid.bytes[2 as ::core::ffi::c_int as usize]
                            as ::core::ffi::c_int,
                        uuid.bytes[3 as ::core::ffi::c_int as usize]
                            as ::core::ffi::c_int,
                        uuid.bytes[4 as ::core::ffi::c_int as usize]
                            as ::core::ffi::c_int,
                        uuid.bytes[5 as ::core::ffi::c_int as usize]
                            as ::core::ffi::c_int,
                        uuid.bytes[6 as ::core::ffi::c_int as usize]
                            as ::core::ffi::c_int,
                        uuid.bytes[7 as ::core::ffi::c_int as usize]
                            as ::core::ffi::c_int,
                        uuid.bytes[8 as ::core::ffi::c_int as usize]
                            as ::core::ffi::c_int,
                        uuid.bytes[9 as ::core::ffi::c_int as usize]
                            as ::core::ffi::c_int,
                        uuid.bytes[10 as ::core::ffi::c_int as usize]
                            as ::core::ffi::c_int,
                        uuid.bytes[11 as ::core::ffi::c_int as usize]
                            as ::core::ffi::c_int,
                        uuid.bytes[12 as ::core::ffi::c_int as usize]
                            as ::core::ffi::c_int,
                        uuid.bytes[13 as ::core::ffi::c_int as usize]
                            as ::core::ffi::c_int,
                        uuid.bytes[14 as ::core::ffi::c_int as usize]
                            as ::core::ffi::c_int,
                        uuid.bytes[15 as ::core::ffi::c_int as usize]
                            as ::core::ffi::c_int,
                    );
                    r = sd_bus_message_append(
                        m,
                        b"s\0" as *const u8 as *const ::core::ffi::c_char,
                        name,
                    );
                    free(name as *mut ::core::ffi::c_void);
                    if r < 0 as ::core::ffi::c_int {
                        xasprintf(
                            cause,
                            b"failed to append to bus message: %s\0" as *const u8
                                as *const ::core::ffi::c_char,
                            strerror(-r),
                        );
                    } else {
                        r = sd_bus_message_append(
                            m,
                            b"s\0" as *const u8 as *const ::core::ffi::c_char,
                            b"fail\0" as *const u8 as *const ::core::ffi::c_char,
                        );
                        if r < 0 as ::core::ffi::c_int {
                            xasprintf(
                                cause,
                                b"failed to append to bus message: %s\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                strerror(-r),
                            );
                        } else {
                            r = sd_bus_message_open_container(
                                m,
                                'a' as i32 as ::core::ffi::c_char,
                                b"(sv)\0" as *const u8 as *const ::core::ffi::c_char,
                            );
                            if r < 0 as ::core::ffi::c_int {
                                xasprintf(
                                    cause,
                                    b"failed to start properties array: %s\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    strerror(-r),
                                );
                            } else {
                                pid = getpid() as pid_t;
                                parent_pid = getppid() as pid_t;
                                xasprintf(
                                    &raw mut desc,
                                    b"tmux child pane %ld launched by process %ld\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    pid as ::core::ffi::c_long,
                                    parent_pid as ::core::ffi::c_long,
                                );
                                r = sd_bus_message_append(
                                    m,
                                    b"(sv)\0" as *const u8 as *const ::core::ffi::c_char,
                                    b"Description\0" as *const u8 as *const ::core::ffi::c_char,
                                    b"s\0" as *const u8 as *const ::core::ffi::c_char,
                                    desc,
                                );
                                free(desc as *mut ::core::ffi::c_void);
                                if r < 0 as ::core::ffi::c_int {
                                    xasprintf(
                                        cause,
                                        b"failed to append to properties: %s\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                        strerror(-r),
                                    );
                                } else {
                                    r = sd_bus_message_append(
                                        m,
                                        b"(sv)\0" as *const u8 as *const ::core::ffi::c_char,
                                        b"SendSIGHUP\0" as *const u8 as *const ::core::ffi::c_char,
                                        b"b\0" as *const u8 as *const ::core::ffi::c_char,
                                        1 as ::core::ffi::c_int,
                                    );
                                    if r < 0 as ::core::ffi::c_int {
                                        xasprintf(
                                            cause,
                                            b"failed to append to properties: %s\0" as *const u8
                                                as *const ::core::ffi::c_char,
                                            strerror(-r),
                                        );
                                    } else {
                                        r = sd_pid_get_user_slice(parent_pid, &raw mut slice);
                                        if r < 0 as ::core::ffi::c_int {
                                            slice = xstrdup(
                                                b"app-tmux.slice\0" as *const u8
                                                    as *const ::core::ffi::c_char,
                                            );
                                        }
                                        r = sd_bus_message_append(
                                            m,
                                            b"(sv)\0" as *const u8 as *const ::core::ffi::c_char,
                                            b"Slice\0" as *const u8 as *const ::core::ffi::c_char,
                                            b"s\0" as *const u8 as *const ::core::ffi::c_char,
                                            slice,
                                        );
                                        free(slice as *mut ::core::ffi::c_void);
                                        if r < 0 as ::core::ffi::c_int {
                                            xasprintf(
                                                cause,
                                                b"failed to append to properties: %s\0" as *const u8
                                                    as *const ::core::ffi::c_char,
                                                strerror(-r),
                                            );
                                        } else {
                                            r = sd_bus_message_append(
                                                m,
                                                b"(sv)\0" as *const u8
                                                    as *const ::core::ffi::c_char,
                                                b"PIDs\0" as *const u8
                                                    as *const ::core::ffi::c_char,
                                                b"au\0" as *const u8 as *const ::core::ffi::c_char,
                                                1 as ::core::ffi::c_int,
                                                pid,
                                            );
                                            if r < 0 as ::core::ffi::c_int {
                                                xasprintf(
                                                    cause,
                                                    b"failed to append to properties: %s\0"
                                                        as *const u8
                                                        as *const ::core::ffi::c_char,
                                                    strerror(-r),
                                                );
                                            } else {
                                                r = sd_bus_message_append(
                                                    m,
                                                    b"(sv)\0" as *const u8
                                                        as *const ::core::ffi::c_char,
                                                    b"CollectMode\0" as *const u8
                                                        as *const ::core::ffi::c_char,
                                                    b"s\0" as *const u8
                                                        as *const ::core::ffi::c_char,
                                                    b"inactive-or-failed\0" as *const u8
                                                        as *const ::core::ffi::c_char,
                                                );
                                                if r < 0 as ::core::ffi::c_int {
                                                    xasprintf(
                                                        cause,
                                                        b"failed to append to properties: %s\0"
                                                            as *const u8
                                                            as *const ::core::ffi::c_char,
                                                        strerror(-r),
                                                    );
                                                } else {
                                                    if sd_pid_get_user_unit(
                                                        parent_pid,
                                                        &raw mut unit,
                                                    ) == 0 as ::core::ffi::c_int
                                                        || sd_pid_get_unit(
                                                            parent_pid,
                                                            &raw mut unit,
                                                        ) == 0 as ::core::ffi::c_int
                                                    {
                                                        r = sd_bus_message_append(
                                                            m,
                                                            b"(sv)\0" as *const u8
                                                                as *const ::core::ffi::c_char,
                                                            b"Before\0" as *const u8
                                                                as *const ::core::ffi::c_char,
                                                            b"as\0" as *const u8
                                                                as *const ::core::ffi::c_char,
                                                            1 as ::core::ffi::c_int,
                                                            unit,
                                                        );
                                                        if r >= 0 as ::core::ffi::c_int {
                                                            r = sd_bus_message_append(
                                                                m,
                                                                b"(sv)\0" as *const u8
                                                                    as *const ::core::ffi::c_char,
                                                                b"PartOf\0" as *const u8
                                                                    as *const ::core::ffi::c_char,
                                                                b"as\0" as *const u8
                                                                    as *const ::core::ffi::c_char,
                                                                1 as ::core::ffi::c_int,
                                                                unit,
                                                            );
                                                        }
                                                        free(unit as *mut ::core::ffi::c_void);
                                                        if r < 0 as ::core::ffi::c_int {
                                                            xasprintf(
                                                                cause,
                                                                b"failed to append to properties: %s\0" as *const u8
                                                                    as *const ::core::ffi::c_char,
                                                                strerror(-r),
                                                            );
                                                            current_block = 3315597219737674933;
                                                        } else {
                                                            current_block = 1423531122933789233;
                                                        }
                                                    } else {
                                                        current_block = 1423531122933789233;
                                                    }
                                                    match current_block {
                                                        3315597219737674933 => {}
                                                        _ => {
                                                            r = sd_bus_message_close_container(m);
                                                            if r < 0 as ::core::ffi::c_int {
                                                                xasprintf(
                                                                    cause,
                                                                    b"failed to end properties array: %s\0" as *const u8
                                                                        as *const ::core::ffi::c_char,
                                                                    strerror(-r),
                                                                );
                                                            } else {
                                                                r = sd_bus_message_append(
                                                                    m,
                                                                    b"a(sa(sv))\0" as *const u8 as *const ::core::ffi::c_char,
                                                                    0 as ::core::ffi::c_int,
                                                                );
                                                                if r < 0 as ::core::ffi::c_int {
                                                                    xasprintf(
                                                                        cause,
                                                                        b"failed to append to bus message: %s\0" as *const u8
                                                                            as *const ::core::ffi::c_char,
                                                                        strerror(-r),
                                                                    );
                                                                } else {
                                                                    r = sd_bus_call(
                                                                        bus,
                                                                        m,
                                                                        1000000 as uint64_t,
                                                                        &raw mut error,
                                                                        &raw mut reply,
                                                                    );
                                                                    if r < 0 as ::core::ffi::c_int {
                                                                        if !error.message.is_null()
                                                                        {
                                                                            xasprintf(
                                                                                cause,
                                                                                b"StartTransientUnit call failed: %s\0" as *const u8
                                                                                    as *const ::core::ffi::c_char,
                                                                                error.message,
                                                                            );
                                                                        } else {
                                                                            xasprintf(
                                                                                cause,
                                                                                b"StartTransientUnit call failed: %s\0" as *const u8
                                                                                    as *const ::core::ffi::c_char,
                                                                                strerror(-r),
                                                                            );
                                                                        }
                                                                    } else {
                                                                        r = sd_bus_message_read(
                                                                            reply,
                                                                            b"o\0" as *const u8 as *const ::core::ffi::c_char,
                                                                            &raw mut watch.path,
                                                                        );
                                                                        if r < 0
                                                                            as ::core::ffi::c_int
                                                                        {
                                                                            xasprintf(
                                                                                cause,
                                                                                b"failed to parse method reply: %s\0" as *const u8
                                                                                    as *const ::core::ffi::c_char,
                                                                                strerror(-r),
                                                                            );
                                                                        } else {
                                                                            while watch.done == 0 {
                                                                                r = sd_bus_process(
                                                                                    bus,
                                                                                    ::core::ptr::null_mut::<*mut sd_bus_message>(),
                                                                                );
                                                                                if r < 0 as ::core::ffi::c_int {
                                                                                    xasprintf(
                                                                                        cause,
                                                                                        b"failed waiting for cgroup allocation: %s\0" as *const u8
                                                                                            as *const ::core::ffi::c_char,
                                                                                        strerror(-r),
                                                                                    );
                                                                                    break;
                                                                                } else {
                                                                                    if r > 0 as ::core::ffi::c_int {
                                                                                        continue;
                                                                                    }
                                                                                    gettimeofday(&raw mut now, NULL);
                                                                                    elapsed_usec = ((now.tv_sec as __suseconds_t
                                                                                        - start.tv_sec as __suseconds_t) * 1000000 as __suseconds_t
                                                                                        + now.tv_usec - start.tv_usec) as uint64_t;
                                                                                    if elapsed_usec >= 1000000 as uint64_t {
                                                                                        xasprintf(
                                                                                            cause,
                                                                                            b"timeout waiting for cgroup allocation\0" as *const u8
                                                                                                as *const ::core::ffi::c_char,
                                                                                        );
                                                                                        break;
                                                                                    } else {
                                                                                        r = sd_bus_wait(
                                                                                            bus,
                                                                                            (1000000 as uint64_t).wrapping_sub(elapsed_usec),
                                                                                        );
                                                                                        if !(r < 0 as ::core::ffi::c_int) {
                                                                                            continue;
                                                                                        }
                                                                                        xasprintf(
                                                                                            cause,
                                                                                            b"failed waiting for cgroup allocation: %s\0" as *const u8
                                                                                                as *const ::core::ffi::c_char,
                                                                                            strerror(-r),
                                                                                        );
                                                                                        break;
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    sd_bus_error_free(&raw mut error);
    sd_bus_message_unref(m);
    sd_bus_message_unref(reply);
    sd_bus_slot_unref(slot);
    sd_bus_unref(bus);
    return r;
}
