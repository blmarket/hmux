use crate::compat::error_message;
use crate::ffi::{
    __errno_location, free, getpid, getppid, getsockname, sd_is_socket_unix, sd_listen_fds,
    sd_pid_get_unit, sd_pid_get_user_slice, sd_pid_get_user_unit,
};
use crate::server::server_create_socket;
use crate::tmux::socket_path;
pub use crate::types::*;
use crate::xmalloc::xasprintf;
use crate::{SystemdBusError, SystemdJobWatch, fmt_args};
use ::std::ffi::{CStr, CString};

unsafe extern "C" {
    pub type sd_bus;
    pub type sd_bus_message;
    pub type sd_bus_slot;
    fn sd_id128_randomize(ret: *mut sd_id128_t) -> core::ffi::c_int;
    fn sd_bus_default_user(ret: *mut *mut sd_bus) -> core::ffi::c_int;
    fn sd_bus_unref(p: *mut sd_bus) -> *mut sd_bus;
    fn sd_bus_call(
        bus: *mut sd_bus,
        m: *mut sd_bus_message,
        usec: uint64_t,
        reterr_error: *mut NativeBusError,
        ret_reply: *mut *mut sd_bus_message,
    ) -> core::ffi::c_int;
    fn sd_bus_process(bus: *mut sd_bus, ret: *mut *mut sd_bus_message) -> core::ffi::c_int;
    fn sd_bus_wait(bus: *mut sd_bus, timeout_usec: uint64_t) -> core::ffi::c_int;
    fn sd_bus_slot_unref(p: *mut sd_bus_slot) -> *mut sd_bus_slot;
    fn sd_bus_message_new_method_call(
        bus: *mut sd_bus,
        ret: *mut *mut sd_bus_message,
        destination: *const core::ffi::c_char,
        path: *const core::ffi::c_char,
        interface: *const core::ffi::c_char,
        member: *const core::ffi::c_char,
    ) -> core::ffi::c_int;
    fn sd_bus_message_unref(p: *mut sd_bus_message) -> *mut sd_bus_message;
    fn sd_bus_message_append(
        m: *mut sd_bus_message,
        types: *const core::ffi::c_char,
        ...
    ) -> core::ffi::c_int;
    fn sd_bus_message_open_container(
        m: *mut sd_bus_message,
        type_0: core::ffi::c_char,
        contents: *const core::ffi::c_char,
    ) -> core::ffi::c_int;
    fn sd_bus_message_close_container(m: *mut sd_bus_message) -> core::ffi::c_int;
    fn sd_bus_message_read(
        m: *mut sd_bus_message,
        types: *const core::ffi::c_char,
        ...
    ) -> core::ffi::c_int;
    fn sd_bus_match_signal(
        bus: *mut sd_bus,
        ret: *mut *mut sd_bus_slot,
        sender: *const core::ffi::c_char,
        path: *const core::ffi::c_char,
        interface: *const core::ffi::c_char,
        member: *const core::ffi::c_char,
        callback: sd_bus_message_handler_t,
        userdata: *mut core::ffi::c_void,
    ) -> core::ffi::c_int;
    fn sd_bus_error_free(e: *mut NativeBusError);
    #[cfg(test)]
    fn sd_bus_error_set(
        e: *mut NativeBusError,
        name: *const core::ffi::c_char,
        message: *const core::ffi::c_char,
    ) -> core::ffi::c_int;
}
#[derive(Clone, Default)]
pub struct sd_bus_error {
    pub name: Option<CString>,
    pub message: Option<CString>,
    pub _need_free: core::ffi::c_int,
}

#[derive(Default)]
#[repr(C)]
struct NativeBusError {
    name: *const core::ffi::c_char,
    message: *const core::ffi::c_char,
    need_free: core::ffi::c_int,
}

impl NativeBusError {
    fn snapshot(&self) -> sd_bus_error {
        unsafe {
            sd_bus_error::from_systemd_bus_error(
                (!self.name.is_null()).then(|| CStr::from_ptr(self.name)),
                (!self.message.is_null()).then(|| CStr::from_ptr(self.message)),
                self.need_free,
            )
        }
    }
}

impl Drop for NativeBusError {
    fn drop(&mut self) {
        unsafe { sd_bus_error_free(self) };
    }
}

pub type sd_bus_message_handler_t = Option<
    unsafe extern "C" fn(
        *mut sd_bus_message,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> core::ffi::c_int,
>;
#[derive(Copy, Clone)]
#[repr(C)]
pub union sd_id128 {
    pub bytes: [uint8_t; 16],
    pub qwords: [uint64_t; 2],
}
pub type sd_id128_t = sd_id128;
pub use crate::consts::{E2BIG, SOCK_STREAM};
#[derive(Clone, Default)]
pub struct systemd_job_watch {
    pub path: Option<CString>,
    pub done: core::ffi::c_int,
}
pub const EPFNOSUPPORT: core::ffi::c_int = 96 as core::ffi::c_int;

#[cfg(test)]
pub const SD_BUS_ERROR_NULL: sd_bus_error = sd_bus_error {
    name: None,
    message: None,
    _need_free: 0 as core::ffi::c_int,
};
pub const SD_LISTEN_FDS_START: core::ffi::c_int = 3 as core::ffi::c_int;
pub fn systemd_activated() -> core::ffi::c_int {
    unsafe { (sd_listen_fds(0 as core::ffi::c_int) >= 1 as core::ffi::c_int) as core::ffi::c_int }
}
pub unsafe fn systemd_create_socket(
    flags: core::ffi::c_int,
    cause: &mut Option<CString>,
) -> core::ffi::c_int {
    unsafe {
        let fd: core::ffi::c_int;
        let mut sa = sockaddr_un::default();
        let mut addrlen: socklen_t = size_of::<sockaddr_un>() as socklen_t;
        let fds: core::ffi::c_int = sd_listen_fds(0 as core::ffi::c_int);
        if fds > 1 as core::ffi::c_int {
            *__errno_location() = E2BIG;
        } else if fds == 1 as core::ffi::c_int {
            fd = SD_LISTEN_FDS_START;
            if sd_is_socket_unix(
                fd,
                SOCK_STREAM as core::ffi::c_int,
                1 as core::ffi::c_int,
                core::ptr::null::<core::ffi::c_char>(),
                0 as size_t,
            ) == 0
            {
                *__errno_location() = EPFNOSUPPORT;
            } else if !(getsockname(fd, (&raw mut sa).cast(), &raw mut addrlen)
                == -(1 as core::ffi::c_int))
            {
                let length = sa
                    .sun_path
                    .iter()
                    .position(|byte| *byte == 0)
                    .unwrap_or(sa.sun_path.len());
                socket_path = Some(
                    CString::new(&sa.sun_path[..length])
                        .expect("the socket path ends before its first terminator"),
                );
                return fd;
            }
        } else {
            return server_create_socket(flags as uint64_t, cause);
        }
        *cause = Some(xasprintf(
            c"systemd socket error (%s)",
            fmt_args![error_message(*__errno_location()).as_c_str()],
        ));
        -(1 as core::ffi::c_int)
    }
}
pub(crate) unsafe extern "C" fn job_removed_handler(
    m: *mut sd_bus_message,
    userdata: *mut core::ffi::c_void,
    _ret_error: *mut core::ffi::c_void,
) -> core::ffi::c_int {
    unsafe {
        let watch = &mut *userdata.cast::<systemd_job_watch>();
        let mut path: *const core::ffi::c_char = core::ptr::null::<core::ffi::c_char>();
        let mut id: uint32_t = 0;

        let Some(watched_path) = watch.systemd_job_path() else {
            return 0;
        };
        let r: core::ffi::c_int =
            sd_bus_message_read(m, c"uo".as_ptr(), &raw mut id, &raw mut path);
        if r < 0 as core::ffi::c_int {
            return r;
        }
        if !path.is_null() && CStr::from_ptr(path) == watched_path {
            watch.set_systemd_job_done(1);
        }
        0 as core::ffi::c_int
    }
}
pub unsafe fn systemd_move_to_new_cgroup(cause: &mut Option<CString>) -> core::ffi::c_int {
    unsafe {
        let current_block: u64;
        let mut error = NativeBusError::default();
        let mut m: *mut sd_bus_message = core::ptr::null_mut::<sd_bus_message>();
        let mut reply: *mut sd_bus_message = core::ptr::null_mut::<sd_bus_message>();
        let mut bus: *mut sd_bus = core::ptr::null_mut::<sd_bus>();
        let mut slot: *mut sd_bus_slot = core::ptr::null_mut::<sd_bus_slot>();
        let mut slice: *mut core::ffi::c_char = core::ptr::null_mut::<core::ffi::c_char>();
        let mut unit: *mut core::ffi::c_char = core::ptr::null_mut::<core::ffi::c_char>();
        let mut uuid: sd_id128_t = sd_id128 { bytes: [0; 16] };
        let mut r: core::ffi::c_int;
        let mut elapsed_usec: uint64_t;
        let pid: pid_t;
        let parent_pid: pid_t;
        let mut watch = systemd_job_watch::default();
        let start = timeval::now();
        r = sd_bus_default_user(&raw mut bus);
        if r < 0 as core::ffi::c_int {
            *cause = Some(xasprintf(
                c"failed to connect to session bus: %s",
                fmt_args![error_message(-r).as_c_str()],
            ));
        } else {
            r = sd_bus_match_signal(
                bus,
                &raw mut slot,
                c"org.freedesktop.systemd1".as_ptr(),
                c"/org/freedesktop/systemd1".as_ptr(),
                c"org.freedesktop.systemd1.Manager".as_ptr(),
                c"JobRemoved".as_ptr(),
                Some(job_removed_handler),
                (&raw mut watch).cast(),
            );
            if r < 0 as core::ffi::c_int {
                *cause = Some(xasprintf(
                    c"failed to create match signal: %s",
                    fmt_args![error_message(-r).as_c_str()],
                ));
            } else {
                r = sd_bus_message_new_method_call(
                    bus,
                    &raw mut m,
                    c"org.freedesktop.systemd1".as_ptr(),
                    c"/org/freedesktop/systemd1".as_ptr(),
                    c"org.freedesktop.systemd1.Manager".as_ptr(),
                    c"StartTransientUnit".as_ptr(),
                );
                if r < 0 as core::ffi::c_int {
                    *cause = Some(xasprintf(
                        c"failed to create bus message: %s",
                        fmt_args![error_message(-r).as_c_str()],
                    ));
                } else {
                    r = sd_id128_randomize(&raw mut uuid);
                    if r < 0 as core::ffi::c_int {
                        *cause = Some(xasprintf(
                            c"failed to generate uuid: %s",
                            fmt_args![error_message(-r).as_c_str()],
                        ));
                    } else {
                        let name = xasprintf(c"tmux-spawn-%02x%02x%02x%02x-%02x%02x-%02x%02x-%02x%02x-%02x%02x%02x%02x%02x%02x.scope", fmt_args![uuid.bytes[0 as core::ffi::c_int as usize]
                            as core::ffi::c_int, uuid.bytes[1 as core::ffi::c_int as usize]
                            as core::ffi::c_int, uuid.bytes[2 as core::ffi::c_int as usize]
                            as core::ffi::c_int, uuid.bytes[3 as core::ffi::c_int as usize]
                            as core::ffi::c_int, uuid.bytes[4 as core::ffi::c_int as usize]
                            as core::ffi::c_int, uuid.bytes[5 as core::ffi::c_int as usize]
                            as core::ffi::c_int, uuid.bytes[6 as core::ffi::c_int as usize]
                            as core::ffi::c_int, uuid.bytes[7 as core::ffi::c_int as usize]
                            as core::ffi::c_int, uuid.bytes[8 as core::ffi::c_int as usize]
                            as core::ffi::c_int, uuid.bytes[9 as core::ffi::c_int as usize]
                            as core::ffi::c_int, uuid.bytes[10 as core::ffi::c_int as usize]
                            as core::ffi::c_int, uuid.bytes[11 as core::ffi::c_int as usize]
                            as core::ffi::c_int, uuid.bytes[12 as core::ffi::c_int as usize]
                            as core::ffi::c_int, uuid.bytes[13 as core::ffi::c_int as usize]
                            as core::ffi::c_int, uuid.bytes[14 as core::ffi::c_int as usize]
                            as core::ffi::c_int, uuid.bytes[15 as core::ffi::c_int as usize]
                            as core::ffi::c_int]);
                        r = sd_bus_message_append(m, c"s".as_ptr(), name.as_ptr());
                        if r < 0 as core::ffi::c_int {
                            *cause = Some(xasprintf(
                                c"failed to append to bus message: %s",
                                fmt_args![error_message(-r).as_c_str()],
                            ));
                        } else {
                            r = sd_bus_message_append(m, c"s".as_ptr(), c"fail".as_ptr());
                            if r < 0 as core::ffi::c_int {
                                *cause = Some(xasprintf(
                                    c"failed to append to bus message: %s",
                                    fmt_args![error_message(-r).as_c_str()],
                                ));
                            } else {
                                r = sd_bus_message_open_container(
                                    m,
                                    'a' as i32 as core::ffi::c_char,
                                    c"(sv)".as_ptr(),
                                );
                                if r < 0 as core::ffi::c_int {
                                    *cause = Some(xasprintf(
                                        c"failed to start properties array: %s",
                                        fmt_args![error_message(-r).as_c_str()],
                                    ));
                                } else {
                                    pid = getpid() as pid_t;
                                    parent_pid = getppid() as pid_t;
                                    let desc = xasprintf(
                                        c"tmux child pane %ld launched by process %ld",
                                        fmt_args![
                                            pid as core::ffi::c_long,
                                            parent_pid as core::ffi::c_long
                                        ],
                                    );
                                    r = sd_bus_message_append(
                                        m,
                                        c"(sv)".as_ptr(),
                                        c"Description".as_ptr(),
                                        c"s".as_ptr(),
                                        desc.as_ptr(),
                                    );
                                    if r < 0 as core::ffi::c_int {
                                        *cause = Some(xasprintf(
                                            c"failed to append to properties: %s",
                                            fmt_args![error_message(-r).as_c_str()],
                                        ));
                                    } else {
                                        r = sd_bus_message_append(
                                            m,
                                            c"(sv)".as_ptr(),
                                            c"SendSIGHUP".as_ptr(),
                                            c"b".as_ptr(),
                                            1 as core::ffi::c_int,
                                        );
                                        if r < 0 as core::ffi::c_int {
                                            *cause = Some(xasprintf(
                                                c"failed to append to properties: %s",
                                                fmt_args![error_message(-r).as_c_str()],
                                            ));
                                        } else {
                                            let slice_from_systemd =
                                                sd_pid_get_user_slice(parent_pid, &raw mut slice)
                                                    >= 0 as core::ffi::c_int;
                                            let slice = if slice_from_systemd {
                                                slice as *const core::ffi::c_char
                                            } else {
                                                c"app-tmux.slice".as_ptr()
                                            };
                                            r = sd_bus_message_append(
                                                m,
                                                c"(sv)".as_ptr(),
                                                c"Slice".as_ptr(),
                                                c"s".as_ptr(),
                                                slice,
                                            );
                                            if slice_from_systemd {
                                                free(slice as *mut core::ffi::c_void);
                                            }
                                            if r < 0 as core::ffi::c_int {
                                                *cause = Some(xasprintf(
                                                    c"failed to append to properties: %s",
                                                    fmt_args![error_message(-r).as_c_str()],
                                                ));
                                            } else {
                                                r = sd_bus_message_append(
                                                    m,
                                                    c"(sv)".as_ptr(),
                                                    c"PIDs".as_ptr(),
                                                    c"au".as_ptr(),
                                                    1 as core::ffi::c_int,
                                                    pid,
                                                );
                                                if r < 0 as core::ffi::c_int {
                                                    *cause = Some(xasprintf(
                                                        c"failed to append to properties: %s",
                                                        fmt_args![error_message(-r).as_c_str()],
                                                    ));
                                                } else {
                                                    r = sd_bus_message_append(
                                                        m,
                                                        c"(sv)".as_ptr(),
                                                        c"CollectMode".as_ptr(),
                                                        c"s".as_ptr(),
                                                        c"inactive-or-failed".as_ptr(),
                                                    );
                                                    if r < 0 as core::ffi::c_int {
                                                        *cause = Some(xasprintf(
                                                            c"failed to append to properties: %s",
                                                            fmt_args![error_message(-r).as_c_str()],
                                                        ));
                                                    } else {
                                                        if sd_pid_get_user_unit(
                                                            parent_pid,
                                                            &raw mut unit,
                                                        ) == 0 as core::ffi::c_int
                                                            || sd_pid_get_unit(
                                                                parent_pid,
                                                                &raw mut unit,
                                                            ) == 0 as core::ffi::c_int
                                                        {
                                                            r = sd_bus_message_append(
                                                                m,
                                                                c"(sv)".as_ptr(),
                                                                c"Before".as_ptr(),
                                                                c"as".as_ptr(),
                                                                1 as core::ffi::c_int,
                                                                unit,
                                                            );
                                                            if r >= 0 as core::ffi::c_int {
                                                                r = sd_bus_message_append(
                                                                    m,
                                                                    c"(sv)".as_ptr(),
                                                                    c"PartOf".as_ptr(),
                                                                    c"as".as_ptr(),
                                                                    1 as core::ffi::c_int,
                                                                    unit,
                                                                );
                                                            }
                                                            free(unit as *mut core::ffi::c_void);
                                                            if r < 0 as core::ffi::c_int {
                                                                *cause = Some(xasprintf(c"failed to append to properties: %s", fmt_args![error_message(-r).as_c_str()]));
                                                                current_block =
                                                                    17941589424253864687;
                                                            } else {
                                                                current_block = 1423531122933789233;
                                                            }
                                                        } else {
                                                            current_block = 1423531122933789233;
                                                        }
                                                        match current_block {
                                                            17941589424253864687 => {}
                                                            _ => {
                                                                r = sd_bus_message_close_container(
                                                                    m,
                                                                );
                                                                if r < 0 as core::ffi::c_int {
                                                                    *cause = Some(xasprintf(c"failed to end properties array: %s", fmt_args![error_message(-r).as_c_str()]));
                                                                } else {
                                                                    r = sd_bus_message_append(
                                                                        m,
                                                                        c"a(sa(sv))".as_ptr(),
                                                                        0 as core::ffi::c_int,
                                                                    );
                                                                    if r < 0 as core::ffi::c_int {
                                                                        *cause = Some(xasprintf(c"failed to append to bus message: %s", fmt_args![error_message(-r).as_c_str()]));
                                                                    } else {
                                                                        r = sd_bus_call(
                                                                            bus,
                                                                            m,
                                                                            1000000 as uint64_t,
                                                                            &raw mut error,
                                                                            &raw mut reply,
                                                                        );
                                                                        if r < 0 as core::ffi::c_int
                                                                        {
                                                                            let snapshot =
                                                                                error.snapshot();
                                                                            if let Some(message) = snapshot.systemd_bus_error_message() {
                                                                                *cause = Some(xasprintf(c"StartTransientUnit call failed: %s", fmt_args![message]));
                                                                            } else {
                                                                                *cause = Some(xasprintf(c"StartTransientUnit call failed: %s", fmt_args![error_message(-r).as_c_str()]));
                                                                            }
                                                                        } else {
                                                                            let mut path = core::ptr::null::<core::ffi::c_char>();
                                                                            r = sd_bus_message_read(
                                                                                reply,
                                                                                c"o".as_ptr(),
                                                                                &raw mut path,
                                                                            );
                                                                            if r < 0
                                                                                as core::ffi::c_int
                                                                            {
                                                                                *cause = Some(xasprintf(c"failed to parse method reply: %s", fmt_args![error_message(-r).as_c_str()]));
                                                                            } else {
                                                                                watch.set_systemd_job_path((!path.is_null()).then(|| CStr::from_ptr(path)));
                                                                                #[allow(clippy::while_immutable_condition)]
                                                                                while watch.done
                                                                                    == 0
                                                                                {
                                                                                    r = sd_bus_process(
                                                                                    bus,
                                                                                    core::ptr::null_mut::<*mut sd_bus_message>(),
                                                                                );
                                                                                    if r < 0 as core::ffi::c_int {
                                                                                    *cause = Some(xasprintf(c"failed waiting for cgroup allocation: %s", fmt_args![error_message(-r).as_c_str()]));
                                                                                    break;
                                                                                } else {
                                                                                    if r > 0 as core::ffi::c_int {
                                                                                        continue;
                                                                                    }
                                                                                    let now = timeval::now();
                                                                                    elapsed_usec = ((now.tv_sec as __suseconds_t
                                                                                        - start.tv_sec as __suseconds_t) * 1000000 as __suseconds_t
                                                                                        + now.tv_usec - start.tv_usec) as uint64_t;
                                                                                    if elapsed_usec >= 1000000 as uint64_t {
                                                                                        *cause = Some(xasprintf(c"timeout waiting for cgroup allocation", fmt_args![]));
                                                                                        break;
                                                                                    } else {
                                                                                        r = sd_bus_wait(
                                                                                            bus,
                                                                                            (1000000 as uint64_t).wrapping_sub(elapsed_usec),
                                                                                        );
                                                                                        if !(r < 0 as core::ffi::c_int) {
                                                                                            continue;
                                                                                        }
                                                                                        *cause = Some(xasprintf(c"failed waiting for cgroup allocation: %s", fmt_args![error_message(-r).as_c_str()]));
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
        drop(error);
        sd_bus_message_unref(m);
        sd_bus_message_unref(reply);
        sd_bus_slot_unref(slot);
        sd_bus_unref(bus);
        r
    }
}

#[cfg(test)]
#[path = "../tests/test_compat_systemd.rs"]
mod tests;

#[cfg(test)]
pub use crate::consts::{
    SOCK_CLOEXEC, SOCK_DCCP, SOCK_DGRAM, SOCK_NONBLOCK, SOCK_PACKET, SOCK_RAW, SOCK_RDM,
    SOCK_SEQPACKET,
};
