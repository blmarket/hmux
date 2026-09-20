// Private job-integration implementation.  This module owns the process-wide
// format-job cache, per-client cache interaction, job callbacks, and tidy
// lifecycle.  It calls the parent facade for expansion, logging, allocation,
// and the tree-storage helpers; the cache and callback signatures are kept
// exactly as they were.
use super::*;

static mut format_jobs: format_job_tree = format_job_tree {
    rbh_root: ::core::ptr::null::<format_job>() as *mut format_job,
};
pub(super) unsafe extern "C" fn format_job_update(mut job: *mut job) {
    let mut fj: *mut format_job = job_get_data(job) as *mut format_job;
    let mut evb: *mut evbuffer = (*job_get_event(job)).input;
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut next: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut t: time_t = 0;
    loop {
        next = evbuffer_readline(evb);
        if next.is_null() {
            break;
        }
        free(line as *mut ::core::ffi::c_void);
        line = next;
    }
    if line.is_null() {
        return;
    }
    (*fj).updated = 1 as ::core::ffi::c_int;
    free((*fj).out as *mut ::core::ffi::c_void);
    (*fj).out = line;
    log_debug(
        b"%s: %p %s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"format_job_update\0" as *const u8 as *const ::core::ffi::c_char,
        fj,
        (*fj).cmd,
        (*fj).out,
    );
    t = time(::core::ptr::null_mut::<time_t>());
    if (*fj).status != 0 && (*fj).last != t {
        if !(*fj).client.is_null() {
            server_status_client((*fj).client);
        }
        (*fj).last = t;
    }
}
pub(super) unsafe extern "C" fn format_job_complete(mut job: *mut job) {
    let mut fj: *mut format_job = job_get_data(job) as *mut format_job;
    let mut evb: *mut evbuffer = (*job_get_event(job)).input;
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: size_t = 0;
    (*fj).job = ::core::ptr::null_mut::<job>();
    buf = ::core::ptr::null_mut::<::core::ffi::c_char>();
    line = evbuffer_readline(evb);
    if line.is_null() {
        len = evbuffer_get_length(evb);
        buf = xmalloc(len.wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
        if len != 0 as size_t {
            memcpy(
                buf as *mut ::core::ffi::c_void,
                evbuffer_pullup(evb, -(1 as ::core::ffi::c_int) as ssize_t)
                    as *const ::core::ffi::c_void,
                len,
            );
        }
        *buf.offset(len as isize) = '\0' as i32 as ::core::ffi::c_char;
    } else {
        buf = line;
    }
    log_debug(
        b"%s: %p %s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"format_job_complete\0" as *const u8 as *const ::core::ffi::c_char,
        fj,
        (*fj).cmd,
        buf,
    );
    if *buf as ::core::ffi::c_int != '\0' as i32 || (*fj).updated == 0 {
        free((*fj).out as *mut ::core::ffi::c_void);
        (*fj).out = buf;
    } else {
        free(buf as *mut ::core::ffi::c_void);
    }
    if (*fj).status != 0 {
        if !(*fj).client.is_null() {
            server_status_client((*fj).client);
        }
        (*fj).status = 0 as ::core::ffi::c_int;
    }
}
pub(super) unsafe extern "C" fn format_job_get(
    mut es: *mut format_expand_state,
    mut cmd: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut ft: *mut format_tree = (*es).ft;
    let mut jobs: *mut format_job_tree = ::core::ptr::null_mut::<format_job_tree>();
    let mut fj0: format_job = format_job {
        client: ::core::ptr::null_mut::<client>(),
        tag: 0,
        cmd: ::core::ptr::null::<::core::ffi::c_char>(),
        expanded: ::core::ptr::null::<::core::ffi::c_char>(),
        last: 0,
        out: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        updated: 0,
        job: ::core::ptr::null_mut::<job>(),
        status: 0,
        entry: format_job_entry {
            rbe_left: ::core::ptr::null_mut::<format_job>(),
            rbe_right: ::core::ptr::null_mut::<format_job>(),
            rbe_parent: ::core::ptr::null_mut::<format_job>(),
            rbe_color: 0,
        },
    };
    let mut fj: *mut format_job = ::core::ptr::null_mut::<format_job>();
    let mut t: time_t = 0;
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut force: ::core::ffi::c_int = 0;
    let mut next: format_expand_state = format_expand_state {
        ft: ::core::ptr::null_mut::<format_tree>(),
        loop_0: 0,
        start_time: 0,
        flags: 0,
        time: 0,
        tm: tm {
            tm_sec: 0,
            tm_min: 0,
            tm_hour: 0,
            tm_mday: 0,
            tm_mon: 0,
            tm_year: 0,
            tm_wday: 0,
            tm_yday: 0,
            tm_isdst: 0,
            tm_gmtoff: 0,
            tm_zone: ::core::ptr::null::<::core::ffi::c_char>(),
        },
    };
    if (*ft).client.is_null() {
        jobs = &raw mut format_jobs as *mut format_job_tree;
    } else if !(*(*ft).client).jobs.is_null() {
        jobs = (*(*ft).client).jobs;
    } else {
        (*(*ft).client).jobs =
            xmalloc(::core::mem::size_of::<format_job_tree>() as size_t) as *mut format_job_tree;
        jobs = (*(*ft).client).jobs;
        (*jobs).rbh_root = ::core::ptr::null_mut::<format_job>();
    }
    fj0.tag = (*ft).tag;
    fj0.cmd = cmd;
    fj = format_job_tree_RB_FIND(jobs, &raw mut fj0);
    if fj.is_null() {
        fj =
            xcalloc(1 as size_t, ::core::mem::size_of::<format_job>() as size_t) as *mut format_job;
        (*fj).client = (*ft).client;
        (*fj).tag = (*ft).tag;
        (*fj).cmd = xstrdup(cmd);
        format_job_tree_RB_INSERT(jobs, fj);
    }
    format_copy_state(
        &raw mut next,
        es,
        FORMAT_EXPAND_NOJOBS | FORMAT_EXPAND_NOCYCLE,
    );
    next.flags &= !FORMAT_EXPAND_TIME;
    expanded = format_expand1(&raw mut next, cmd);
    if (*fj).expanded.is_null() || strcmp(expanded, (*fj).expanded) != 0 as ::core::ffi::c_int {
        free((*fj).expanded as *mut ::core::ffi::c_void);
        (*fj).expanded = xstrdup(expanded);
        force = 1 as ::core::ffi::c_int;
    } else {
        force = (*ft).flags & FORMAT_FORCE;
    }
    t = time(::core::ptr::null_mut::<time_t>());
    if force != 0 && !(*fj).job.is_null() {
        job_free((*fj).job);
    }
    if force != 0 || (*fj).job.is_null() && (*fj).last != t {
        (*fj).job = job_run(
            expanded,
            0 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<environ>(),
            ::core::ptr::null_mut::<session>(),
            server_client_get_cwd((*ft).client, ::core::ptr::null_mut::<session>()),
            Some(format_job_update as unsafe extern "C" fn(*mut job) -> ()),
            Some(format_job_complete as unsafe extern "C" fn(*mut job) -> ()),
            None,
            fj as *mut ::core::ffi::c_void,
            JOB_NOWAIT,
            -(1 as ::core::ffi::c_int),
            -(1 as ::core::ffi::c_int),
        );
        if (*fj).job.is_null() {
            free((*fj).out as *mut ::core::ffi::c_void);
            xasprintf(
                &raw mut (*fj).out,
                b"<'%s' didn't start>\0" as *const u8 as *const ::core::ffi::c_char,
                (*fj).cmd,
            );
        }
        (*fj).last = t;
        (*fj).updated = 0 as ::core::ffi::c_int;
    } else if !(*fj).job.is_null() && t - (*fj).last > 1 as time_t && (*fj).out.is_null() {
        xasprintf(
            &raw mut (*fj).out,
            b"<'%s' not ready>\0" as *const u8 as *const ::core::ffi::c_char,
            (*fj).cmd,
        );
    }
    free(expanded as *mut ::core::ffi::c_void);
    if (*ft).flags & FORMAT_STATUS != 0 {
        (*fj).status = 1 as ::core::ffi::c_int;
    }
    if (*fj).out.is_null() {
        return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    }
    return format_expand1(&raw mut next, (*fj).out);
}
pub(super) unsafe extern "C" fn format_job_tidy(
    mut jobs: *mut format_job_tree,
    mut force: ::core::ffi::c_int,
) {
    let mut fj: *mut format_job = ::core::ptr::null_mut::<format_job>();
    let mut fj1: *mut format_job = ::core::ptr::null_mut::<format_job>();
    let mut now: time_t = 0;
    now = time(::core::ptr::null_mut::<time_t>());
    fj = format_job_tree_RB_MINMAX(jobs, RB_NEGINF);
    while !fj.is_null() && {
        fj1 = format_job_tree_RB_NEXT(fj);
        1 as ::core::ffi::c_int != 0
    } {
        if !(force == 0 && ((*fj).last > now || now - (*fj).last < 3600 as time_t)) {
            format_job_tree_RB_REMOVE(jobs, fj);
            log_debug(
                b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
                b"format_job_tidy\0" as *const u8 as *const ::core::ffi::c_char,
                (*fj).cmd,
            );
            if !(*fj).job.is_null() {
                job_free((*fj).job);
            }
            free((*fj).expanded as *mut ::core::ffi::c_void);
            free((*fj).cmd as *mut ::core::ffi::c_void);
            free((*fj).out as *mut ::core::ffi::c_void);
            free(fj as *mut ::core::ffi::c_void);
        }
        fj = fj1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn format_tidy_jobs() {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    format_job_tidy(&raw mut format_jobs, 0 as ::core::ffi::c_int);
    c = clients.tqh_first;
    while !c.is_null() {
        if !(*c).jobs.is_null() {
            format_job_tidy((*c).jobs, 0 as ::core::ffi::c_int);
        }
        c = (*c).entry.tqe_next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn format_lost_client(mut c: *mut client) {
    if !(*c).jobs.is_null() {
        format_job_tidy((*c).jobs, 1 as ::core::ffi::c_int);
    }
    free((*c).jobs as *mut ::core::ffi::c_void);
}
