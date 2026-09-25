// Private job-integration implementation.  This module owns the process-wide
// format-job cache, per-client cache interaction, job callbacks, and tidy
// lifecycle.  It calls the parent facade for expansion, logging, and allocation.
// Job addresses remain stable for process callbacks.
use super::*;
use std::ffi::{CStr, CString};

fn format_job_set_expanded(fj: &mut format_job, value: CString) {
    fj.expanded = Default::default();
    fj.expanded = Some(value);
}

fn format_job_set_out(fj: &mut format_job, value: CString) {
    fj.out = Default::default();
    fj.out = Some(value);
}

// Match C-string visibility while keeping the line's Rust allocation local.
unsafe fn format_job_set_out_from_line(fj: *mut format_job, value: &[u8]) {
    let visible = value
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(value.len());
    let owned = CString::new(&value[..visible]).expect("visible job output contains no NUL");
    format_job_set_out(&mut *fj, owned);
}

fn format_job_message(fj: &format_job, suffix: &[u8]) -> CString {
    let cmd = fj.cmd.as_bytes();
    let mut bytes = Vec::with_capacity(2 + cmd.len() + suffix.len());
    bytes.extend_from_slice(b"<'");
    bytes.extend_from_slice(cmd);
    bytes.extend_from_slice(suffix);
    CString::new(bytes).expect("C command and literal suffix contain no NUL")
}

static mut format_jobs: format_job_tree = format_job_tree {
    entries: std::collections::BTreeMap::new(),
};
pub(super) unsafe extern "C" fn format_job_update(mut job: *mut job) {
    let mut fj: *mut format_job = job_get_data(job) as *mut format_job;
    let mut evb: *mut evbuffer = (*job_get_event(job)).input;
    let mut line: Option<Vec<u8>> = None;
    let mut t: time_t = 0;
    loop {
        let Some(next) = evbuffer_readline(evb) else {
            break;
        };
        line = Some(next);
    }
    let Some(line) = line else {
        return;
    };
    (*fj).updated = 1 as ::core::ffi::c_int;
    format_job_set_out_from_line(fj, &line);
    log_debug(
        b"%s: %p %s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"format_job_update\0" as *const u8 as *const ::core::ffi::c_char,
        fj,
        ((*fj).cmd).as_ptr().cast_mut(),
        ((*fj).out)
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
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
    (*fj).job = ::core::ptr::null_mut::<job>();
    let line = evbuffer_readline(evb);
    let output = if let Some(line) = line {
        let visible = line
            .iter()
            .position(|&byte| byte == 0)
            .unwrap_or(line.len());
        CString::new(&line[..visible]).expect("visible job output contains no NUL")
    } else {
        let len = evbuffer_get_length(evb);
        let bytes = if len == 0 {
            &[][..]
        } else {
            std::slice::from_raw_parts(
                evbuffer_pullup(evb, -(1 as ::core::ffi::c_int) as ssize_t),
                len,
            )
        };
        // The old malloc buffer was treated as a C string after copying all
        // bytes, so only bytes before the first NUL became visible output.
        let visible = bytes.iter().position(|&byte| byte == 0).unwrap_or(len);
        CString::new(&bytes[..visible]).expect("visible job output contains no NUL")
    };
    log_debug(
        b"%s: %p %s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"format_job_complete\0" as *const u8 as *const ::core::ffi::c_char,
        fj,
        ((*fj).cmd).as_ptr().cast_mut(),
        output.as_ptr(),
    );
    if !output.as_bytes().is_empty() || (*fj).updated == 0 {
        format_job_set_out(&mut *fj, output);
    }
    if (*fj).status != 0 {
        if !(*fj).client.is_null() {
            server_status_client((*fj).client);
        }
        (*fj).status = 0 as ::core::ffi::c_int;
    }
}
pub(super) unsafe fn format_job_get(
    mut es: *mut format_expand_state,
    mut cmd: *const ::core::ffi::c_char,
) -> CString {
    let mut ft: *mut format_tree = (*es).ft;
    let mut jobs: *mut format_job_tree = ::core::ptr::null_mut::<format_job_tree>();
    let mut fj: *mut format_job = ::core::ptr::null_mut::<format_job>();
    let mut t: time_t = 0;
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
    let expanded = format_expand1_cstring(&raw mut next, cmd);
    if (*fj).expanded.is_none()
        || strcmp(
            expanded.as_ptr(),
            ((*fj).expanded)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        ) != 0 as ::core::ffi::c_int
    {
        format_job_set_expanded(&mut *fj, expanded.clone());
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
            Some(expanded.as_c_str()),
            &Vec::new(),
            ::core::ptr::null_mut::<environ>(),
            ::core::ptr::null_mut::<session>(),
            {
                let cwd = server_client_get_cwd((*ft).client, ::core::ptr::null_mut::<session>());
                (!cwd.is_null()).then(|| CStr::from_ptr(cwd))
            },
            Some(format_job_update as unsafe extern "C" fn(*mut job) -> ()),
            Some(format_job_complete as unsafe extern "C" fn(*mut job) -> ()),
            None,
            fj as *mut ::core::ffi::c_void,
            JOB_NOWAIT,
            -(1 as ::core::ffi::c_int),
            -(1 as ::core::ffi::c_int),
        );
        if (*fj).job.is_null() {
            let message = format_job_message(&*fj, b"' didn't start>");
            format_job_set_out(&mut *fj, message);
        }
        (*fj).last = t;
        (*fj).updated = 0 as ::core::ffi::c_int;
    } else if !(*fj).job.is_null() && t - (*fj).last > 1 as time_t && (*fj).out.is_none() {
        let message = format_job_message(&*fj, b"' not ready>");
        format_job_set_out(&mut *fj, message);
    }
    if (*ft).flags & FORMAT_STATUS != 0 {
        (*fj).status = 1 as ::core::ffi::c_int;
    }
    if (*fj).out.is_none() {
        return CString::default();
    }
    return format_expand1_cstring(
        &raw mut next,
        ((*fj).out)
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
    );
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
        let command = CStr::from_ptr(cmd).to_owned();
        let node = format_job {
            client,
            tag,
            cmd: ::std::ffi::CStr::from_ptr(command.as_ptr()).to_owned(),
            expanded: Default::default(),
            last: 0,
            out: Default::default(),
            updated: 0,
            job: std::ptr::null_mut(),
            status: 0,
        };
        Box::into_raw(Box::new(format_job {
            cmd: command,
            expanded: None,
            out: None,
            ..node
        }))
        .cast::<format_job>()
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
            ((*fj).cmd).as_ptr().cast_mut(),
        );
        if !(*fj).job.is_null() {
            job_free((*fj).job);
        }
        drop(Box::from_raw(fj));
    }
}
#[no_mangle]
pub unsafe extern "C" fn format_tidy_jobs() {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    format_job_tidy(&raw mut format_jobs, 0 as ::core::ffi::c_int);
    c = clients.first();
    while !c.is_null() {
        if !(*c).jobs.is_null() {
            format_job_tidy((*c).jobs, 0 as ::core::ffi::c_int);
        }
        c = clients.next(c);
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
                    (*a).tag < (*b).tag
                        || ((*a).tag == (*b).tag
                            && strcmp(
                                ((*a).cmd).as_ptr().cast_mut(),
                                ((*b).cmd).as_ptr().cast_mut()
                            ) < 0)
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
            let mut c: client = client::empty();
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
                format_job_set_expanded(&mut *fj, CStr::from_ptr(cmd.as_ptr()).to_owned());
                format_job_set_out(&mut *fj, CStr::from_ptr(cmd.as_ptr()).to_owned());
                if last > now || now - last < 3600 {
                    survivors.push((cmd, fj));
                }
            }
            format_job_tidy_at(&mut cache, 0, now);
            assert_eq!(cache.entries.len(), 3);
            for (cmd, fj) in survivors {
                assert_eq!(cache.entries.get(&(0, cmd.as_bytes().to_vec())), Some(&fj));
                assert_eq!(
                    std::ffi::CStr::from_ptr(
                        ((*fj).out)
                            .as_ref()
                            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut())
                    )
                    .to_bytes(),
                    cmd.as_bytes()
                );
            }
            format_job_tidy_at(&mut cache, 1, now);
            assert!(cache.entries.is_empty());
            format_job_tidy_at(&mut cache, 0, now);
        }
    }

    #[test]
    fn line_output_crosses_into_job_owner_and_replacement_stays_address_stable() {
        unsafe {
            let mut cache = format_job_tree::default();
            let cmd = CString::new(b"printf '\xff'".to_vec()).unwrap();
            let fj = format_job_find_or_insert(&mut cache, null_mut(), 1, cmd.as_ptr());
            format_job_set_out_from_line(fj, b"first\0ignored");
            assert_eq!(
                CStr::from_ptr(
                    ((*fj).out)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut())
                )
                .to_bytes(),
                b"first"
            );
            format_job_set_expanded(&mut *fj, CString::new(b"expanded\xff".to_vec()).unwrap());
            let message = format_job_message(&*fj, b"' not ready>");
            format_job_set_out(&mut *fj, message);
            assert_eq!(
                CStr::from_ptr(
                    ((*fj).out)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut())
                )
                .to_bytes(),
                b"<'printf '\xff'' not ready>"
            );
            assert_eq!(
                CStr::from_ptr(
                    ((*fj).expanded)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut())
                )
                .to_bytes(),
                b"expanded\xff"
            );
            assert_eq!(
                fj,
                format_job_find_or_insert(&mut cache, null_mut(), 1, cmd.as_ptr())
            );
            format_job_tidy_at(&mut cache, 1, 0);
            assert!(cache.entries.is_empty());
        }
    }
}
