// Private job-integration implementation.  This module owns the process-wide
// format-job cache, per-client cache interaction, job callbacks, and tidy
// lifecycle.  It calls the parent facade for expansion, logging, and allocation.
// Job addresses remain stable for process callbacks.
use super::*;

static mut format_jobs: format_job_tree = format_job_tree {
    entries: std::collections::BTreeMap::new(),
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
        (*(*ft).client).jobs = Box::into_raw(Box::new(format_job_tree::default()));
        jobs = (*(*ft).client).jobs;
    }
    fj = format_job_find_or_insert(jobs, (*ft).client, (*ft).tag, cmd);
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
// Do not retain a map borrow across format expansion or process callbacks.
unsafe fn format_job_find_or_insert(
    jobs: *mut format_job_tree,
    client: *mut client,
    tag: u_int,
    cmd: *const ::core::ffi::c_char,
) -> *mut format_job {
    let key = (tag, std::ffi::CStr::from_ptr(cmd).to_bytes().to_vec());
    *(*jobs).entries.entry(key).or_insert_with(|| {
        let fj = xcalloc(1, ::core::mem::size_of::<format_job>()) as *mut format_job;
        (*fj).client = client;
        (*fj).tag = tag;
        (*fj).cmd = xstrdup(cmd);
        fj
    })
}

pub(super) unsafe extern "C" fn format_job_tidy(
    jobs: *mut format_job_tree,
    force: ::core::ffi::c_int,
) {
    format_job_tidy_at(jobs, force, time(::core::ptr::null_mut()));
}

unsafe fn format_job_tidy_at(jobs: *mut format_job_tree, force: ::core::ffi::c_int, now: time_t) {
    // Snapshot keys in tree order, then remove before cleanup, just like the
    // old traversal. No iterator or map borrow survives a call to job_free.
    let expired: Vec<_> = (*jobs)
        .entries
        .iter()
        .filter_map(|(key, &fj)| {
            if force == 0 && ((*fj).last > now || now - (*fj).last < 3600) {
                None
            } else {
                Some(key.clone())
            }
        })
        .collect();
    for key in expired {
        let fj = (*jobs)
            .entries
            .remove(&key)
            .expect("format job still cached");
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
        drop(Box::from_raw((*c).jobs));
        (*c).jobs = ::core::ptr::null_mut();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::ptr::null_mut;

    #[test]
    fn cache_preserves_identity_and_c_comparator_order() {
        unsafe {
            let mut cache = format_job_tree::default();
            let mut other = format_job_tree::default();
            let command = CString::new(b"cmd\xff".to_vec()).unwrap();
            let original = format_job_find_or_insert(&mut cache, null_mut(), 7, command.as_ptr());
            (*original).updated = 42;
            let duplicate = CString::new(command.as_bytes()).unwrap();
            assert_eq!(
                original,
                format_job_find_or_insert(&mut cache, null_mut(), 7, duplicate.as_ptr())
            );
            assert_ne!(
                original,
                format_job_find_or_insert(&mut other, null_mut(), 7, duplicate.as_ptr())
            );
            assert_ne!(
                original,
                format_job_find_or_insert(&mut cache, null_mut(), 8, duplicate.as_ptr())
            );

            // Force tree growth with unsigned tags and non-UTF-8 command bytes.
            for tag in [0, 7, 8, u_int::MAX] {
                for byte in 1..=255u8 {
                    let cmd = CString::new(vec![byte]).unwrap();
                    format_job_find_or_insert(&mut cache, null_mut(), tag, cmd.as_ptr());
                }
            }
            assert_eq!(
                original,
                format_job_find_or_insert(&mut cache, null_mut(), 7, command.as_ptr())
            );
            assert_eq!((*original).updated, 42);
            let jobs: Vec<_> = cache.entries.values().copied().collect();
            for pair in jobs.windows(2) {
                let (a, b) = (pair[0], pair[1]);
                assert!(
                    (*a).tag < (*b).tag || ((*a).tag == (*b).tag && strcmp((*a).cmd, (*b).cmd) < 0)
                );
            }
            format_job_tidy_at(&mut cache, 1, 0);
            format_job_tidy_at(&mut other, 1, 0);
            assert!(cache.entries.is_empty());
            assert!(other.entries.is_empty());
        }
    }

    #[test]
    fn client_teardown_releases_the_rust_cache() {
        unsafe {
            let mut c: client = std::mem::zeroed();
            c.jobs = Box::into_raw(Box::new(format_job_tree::default()));
            let cmd = CString::new("job").unwrap();
            let fj = format_job_find_or_insert(c.jobs, &mut c, 0, cmd.as_ptr());
            (*fj).last = time(std::ptr::null_mut()) + 3600;
            format_lost_client(&mut c);
            assert!(c.jobs.is_null());
            format_lost_client(&mut c);
        }
    }

    #[test]
    fn tidy_preserves_expiration_boundary_and_survivor_addresses() {
        unsafe {
            let mut cache = format_job_tree::default();
            let now = 10_000;
            let mut survivors = Vec::new();
            // Interleave expired and retained entries in traversal order.
            for (index, last) in [now - 3600, now - 3599, now - 7200, now + 1, now]
                .into_iter()
                .enumerate()
            {
                let cmd = CString::new(format!("job-{index}")).unwrap();
                let fj = format_job_find_or_insert(&mut cache, null_mut(), 0, cmd.as_ptr());
                (*fj).last = last;
                (*fj).expanded = xstrdup(cmd.as_ptr());
                (*fj).out = xstrdup(cmd.as_ptr());
                if last > now || now - last < 3600 {
                    survivors.push((cmd, fj));
                }
            }
            format_job_tidy_at(&mut cache, 0, now);
            assert_eq!(cache.entries.len(), 3);
            for (cmd, fj) in survivors {
                assert_eq!(cache.entries.get(&(0, cmd.as_bytes().to_vec())), Some(&fj));
                assert_eq!(
                    std::ffi::CStr::from_ptr((*fj).out).to_bytes(),
                    cmd.as_bytes()
                );
            }
            format_job_tidy_at(&mut cache, 1, now);
            assert!(cache.entries.is_empty());
            format_job_tidy_at(&mut cache, 0, now);
        }
    }
}
