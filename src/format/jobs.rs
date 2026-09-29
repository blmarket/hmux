use crate::src::log::{log_cstr, log_pointer};
use crate::src::server_client::Client as _;
use crate::src::shared::client::{ClientRef, ClientWeak};
// Private job-integration implementation.  This module owns the process-wide
// format-job cache, per-client cache interaction, job callbacks, and tidy
// lifecycle.  It calls the parent facade for expansion, logging, and allocation.
// All cache borrows finish before format expansion or process callbacks.
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
fn format_job_set_out_from_line(fj: &mut format_job, value: &[u8]) {
    let visible = value
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(value.len());
    let owned = CString::new(&value[..visible]).expect("visible job output contains no NUL");
    format_job_set_out(fj, owned);
}

fn format_job_message(fj: &format_job, suffix: &[u8]) -> CString {
    let cmd = fj.cmd.as_bytes();
    let mut bytes = Vec::with_capacity(2 + cmd.len() + suffix.len());
    bytes.extend_from_slice(b"<'");
    bytes.extend_from_slice(cmd);
    bytes.extend_from_slice(suffix);
    CString::new(bytes).expect("C command and literal suffix contain no NUL")
}

// Cache storage is owned by Client or by the process. Callbacks carry a weak
// Client and entry identity, never a reference or pointer into either cache.
static mut format_jobs: Option<Box<format_job_tree>> = None;

#[derive(Clone)]
enum JobCache {
    Global,
    Client(ClientWeak),
}

impl JobCache {
    fn for_client(client: Option<&ClientRef>) -> Self {
        client.map_or(Self::Global, |client| {
            Self::Client(std::rc::Rc::downgrade(client))
        })
    }

    /// Only immediate cache edits. The closure must not expand formats, launch
    /// or free processes, notify clients, or return borrowed components.
    unsafe fn with_cache<R>(
        &self,
        edit: impl FnOnce(&mut Option<Box<format_job_tree>>) -> R,
    ) -> Option<R> {
        match self {
            Self::Global => Some(edit(&mut *(&raw mut format_jobs))),
            Self::Client(observer) => {
                let client = observer.upgrade()?;
                let mut cache = client.borrow_format_jobs_mut();
                Some(edit(&mut cache))
            }
        }
    }

    unsafe fn entry(&self, client: Option<&ClientRef>, tag: u_int, command: &CStr) -> JobEntry {
        let key = (tag, command.to_bytes().to_vec());
        let identity = self
            .with_cache(|cache| {
                let record = format_job_find_or_insert(
                    cache.get_or_insert_with(Default::default),
                    client,
                    tag,
                    command,
                );
                std::rc::Rc::downgrade(&record.identity)
            })
            .expect("format context retains the cache client");
        JobEntry {
            cache: self.clone(),
            key,
            identity,
        }
    }
}

#[derive(Clone)]
struct JobEntry {
    cache: JobCache,
    key: (u_int, Vec<u8>),
    identity: std::rc::Weak<()>,
}

impl JobEntry {
    unsafe fn with_record<R>(&self, edit: impl FnOnce(&mut format_job) -> R) -> Option<R> {
        self.cache
            .with_cache(|cache| {
                let record = cache.as_deref_mut()?.get_mut(&self.key)?;
                if !std::rc::Rc::downgrade(&record.identity).ptr_eq(&self.identity) {
                    return None;
                }
                Some(edit(record))
            })
            .flatten()
    }
}

unsafe fn format_job_update(job: &refbox::Weak<job>, entry: &JobEntry) {
    let line = {
        let buffer = crate::src::reactor::bufferevent_get_input(&mut *job_get_event(job));
        let mut line = None;
        while let Some(next) = evbuffer_readline(buffer) {
            line = Some(next);
        }
        line
    };
    let Some(line) = line else { return };
    let Some((client, status, last)) = entry.with_record(|record| {
        record.updated = 1;
        format_job_set_out_from_line(record, &line);
        log_debug(format_args!(
            "format_job_update: {} {}: {}",
            log_pointer(std::ptr::from_ref(record).cast()),
            log_cstr(record.cmd.as_ptr()),
            log_cstr(record.out.as_ref().unwrap().as_ptr()),
        ));
        (record.client.clone(), record.status, record.last)
    }) else {
        return;
    };
    let now = time(std::ptr::null_mut());
    if status != 0 && last != now {
        if let Some(client) = client.upgrade() {
            server_status_client(&client);
        }
        // Keep publication after the notification, as before. A notification
        // may have retired this entry, so reacquire and check its identity.
        entry.with_record(|record| record.last = now);
    }
}

unsafe fn format_job_complete(completion: JobCompletion, entry: &JobEntry) {
    let mut buffer = evbuffer_new();
    if !completion.output.is_empty() {
        evbuffer_add(
            &mut buffer,
            completion.output.as_ptr().cast(),
            completion.output.len(),
        );
    }
    if entry
        .with_record(|record| record.job = refbox::Weak::new())
        .is_none()
    {
        return;
    }
    let output = if let Some(line) = evbuffer_readline(&mut buffer) {
        let visible = line
            .iter()
            .position(|&byte| byte == 0)
            .unwrap_or(line.len());
        CString::new(&line[..visible]).expect("visible job output contains no NUL")
    } else {
        let bytes = evbuffer_pullup(&mut buffer, -1).unwrap_or_default();
        let visible = bytes
            .iter()
            .position(|&byte| byte == 0)
            .unwrap_or(bytes.len());
        CString::new(&bytes[..visible]).expect("visible job output contains no NUL")
    };
    let Some((client, status)) = entry.with_record(|record| {
        log_debug(format_args!(
            "format_job_complete: {} {}: {}",
            log_pointer(std::ptr::from_ref(record).cast()),
            log_cstr(record.cmd.as_ptr()),
            log_cstr(output.as_ptr()),
        ));
        if !output.is_empty() || record.updated == 0 {
            format_job_set_out(record, output);
        }
        (record.client.clone(), record.status)
    }) else {
        return;
    };
    if status != 0 {
        if let Some(client) = client.upgrade() {
            server_status_client(&client);
        }
        entry.with_record(|record| record.status = 0);
    }
}

pub(super) unsafe fn format_job_get(
    es: *mut format_expand_state,
    command: *const ::core::ffi::c_char,
) -> CString {
    let ft = (*es).ft;
    let cache = JobCache::for_client((*ft).client.as_ref());
    // Publish the entry before expansion, preserving recursive lookup ordering.
    let entry = cache.entry((*ft).client.as_ref(), (*ft).tag, CStr::from_ptr(command));
    let mut next = format_expand_state::default();
    format_copy_state(&mut next, es, FORMAT_EXPAND_NOJOBS | FORMAT_EXPAND_NOCYCLE);
    next.flags &= !FORMAT_EXPAND_TIME;
    let expanded = format_expand1_cstring(&mut next, command);
    let Some((force, running)) = entry.with_record(|record| {
        let force = if record.expanded.as_deref() != Some(expanded.as_c_str()) {
            format_job_set_expanded(record, expanded.clone());
            1
        } else {
            (*ft).flags & FORMAT_FORCE
        };
        (force, record.job.clone())
    }) else {
        return CString::default();
    };
    let now = time(std::ptr::null_mut());
    if force != 0 && !running.is_empty() {
        job_free(&running);
    }
    let Some(start) =
        entry.with_record(|record| force != 0 || record.job.is_empty() && record.last != now)
    else {
        return CString::default();
    };
    if start {
        let cwd = server_client_get_cwd((*ft).client.as_ref(), None);
        let update_entry = entry.clone();
        let complete_entry = entry.clone();
        let started = job_run(
            Some(&expanded),
            &Vec::new(),
            None,
            None,
            cwd.as_deref(),
            job_update_callback(move |job| unsafe { format_job_update(job, &update_entry) }),
            Some(Box::new(move |completion| unsafe {
                format_job_complete(completion, &complete_entry)
            })),
            None,
            JOB_NOWAIT,
            -1,
            -1,
        );
        if entry
            .with_record(|record| {
                record.job = started.clone();
                if record.job.is_empty() {
                    let message = format_job_message(record, b"' didn't start>");
                    format_job_set_out(record, message);
                }
                record.last = now;
                record.updated = 0;
            })
            .is_none()
        {
            // A removed cache cannot own a newly started process. Keep explicit
            // cancellation outside the borrow even on this reentrant path.
            job_free(&started);
            return CString::default();
        }
    } else {
        entry.with_record(|record| {
            if !record.job.is_empty() && now - record.last > 1 && record.out.is_none() {
                let message = format_job_message(record, b"' not ready>");
                format_job_set_out(record, message);
            }
        });
    }
    let output = entry
        .with_record(|record| {
            if (*ft).flags & FORMAT_STATUS != 0 {
                record.status = 1;
            }
            record.out.clone()
        })
        .flatten();
    output.map_or_else(CString::default, |output| {
        format_expand1_cstring(&mut next, output.as_ptr())
    })
}

// Component-only lookup: callers keep this reference inside their cache borrow.
fn format_job_find_or_insert<'a>(
    jobs: &'a mut format_job_tree,
    client: Option<&ClientRef>,
    tag: u_int,
    command: &CStr,
) -> &'a mut format_job {
    jobs.entry((tag, command.to_bytes().to_vec()))
        .or_insert_with(|| {
            Box::new(format_job {
                identity: std::rc::Rc::new(()),
                client: client.map_or_else(std::rc::Weak::new, std::rc::Rc::downgrade),
                tag,
                cmd: command.to_owned(),
                expanded: None,
                last: 0,
                out: None,
                updated: 0,
                job: refbox::Weak::new(),
                status: 0,
            })
        })
}

unsafe fn format_job_tidy_at(cache: &JobCache, force: i32, now: time_t) {
    let expired = cache
        .with_cache(|cache| {
            cache.as_deref().map_or_else(Vec::new, |jobs| {
                jobs.iter()
                    .filter_map(|(key, record)| {
                        if force == 0 && (record.last > now || now - record.last < 3600) {
                            None
                        } else {
                            Some((key.clone(), std::rc::Rc::downgrade(&record.identity)))
                        }
                    })
                    .collect()
            })
        })
        .unwrap_or_default();
    for (key, identity) in expired {
        let removed = cache
            .with_cache(|cache| {
                let jobs = cache.as_deref_mut()?;
                let record = jobs.get(&key)?;
                if !std::rc::Rc::downgrade(&record.identity).ptr_eq(&identity) {
                    return None;
                }
                jobs.remove(&key)
            })
            .flatten();
        let Some(record) = removed else { continue };
        log_debug(format_args!(
            "format_job_tidy: {}",
            log_cstr(record.cmd.as_ptr())
        ));
        if !record.job.is_empty() {
            job_free(&record.job);
        }
        drop(record);
    }
}

pub unsafe fn format_tidy_jobs() {
    format_job_tidy_at(&JobCache::Global, 0, time(std::ptr::null_mut()));
    let mut current = clients.first();
    while let Some(client) = current {
        format_job_tidy_at(
            &JobCache::for_client(Some(&client)),
            0,
            time(std::ptr::null_mut()),
        );
        current = clients.next(&client);
    }
}

pub unsafe fn format_lost_client(client: &ClientRef) {
    let cache = JobCache::for_client(Some(client));
    // Keep the cache installed throughout explicit process cancellation.
    format_job_tidy_at(&cache, 1, time(std::ptr::null_mut()));
    let removed = client.borrow_format_jobs_mut().take();
    drop(removed);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    #[test]
    fn cancellation_reenters_the_installed_cache_after_entry_removal() {
        unsafe {
            let owner = client::new();
            let cache = JobCache::for_client(Some(&owner));
            let expired = cache.entry(Some(&owner), 0, c"expired");
            let kept = cache.entry(Some(&owner), 0, c"kept");
            kept.with_record(|record| record.last = 10_000);
            let observer = Rc::downgrade(&owner);
            let calls = Rc::new(std::cell::Cell::new(0));
            let called = calls.clone();
            let callback_cache = cache.clone();
            let process =
                crate::src::job::job_with_free_callback_for_test(Some(Box::new(move || {
                    let owner = observer.upgrade().unwrap();
                    {
                        let jobs = owner.borrow_format_jobs_mut();
                        let jobs = jobs
                            .as_ref()
                            .expect("cache remains installed during cancellation");
                        assert!(!jobs.contains_key(&(0, b"expired".to_vec())));
                        assert!(jobs.contains_key(&(0, b"kept".to_vec())));
                    }
                    callback_cache.entry(Some(&owner), 0, c"from callback");
                    called.set(called.get() + 1);
                })));
            expired.with_record(|record| record.job = process.clone());
            assert_eq!(Rc::strong_count(&owner), 1);
            format_job_tidy_at(&cache, 0, 10_000);
            assert_eq!(calls.get(), 1);
            assert!(!process.is_alive());
            assert!(expired.identity.upgrade().is_none());
            assert!(kept.identity.upgrade().is_some());
            assert_eq!(
                cache.with_cache(|cache| cache.as_ref().unwrap().len()),
                Some(2)
            );
            format_lost_client(&owner);
            assert!(owner.borrow_format_jobs_mut().is_none());
        }
    }

    #[test]
    fn completion_skips_expired_notification_client_without_retaining_it() {
        unsafe {
            let owner = client::new();
            let notification_client = client::new();
            let cache = JobCache::for_client(Some(&owner));
            let entry = cache.entry(Some(&notification_client), 0, c"expired-client");
            entry.with_record(|record| record.status = 1);
            assert_eq!(Rc::strong_count(&notification_client), 1);
            drop(notification_client);
            assert!(entry
                .with_record(|record| record.client.upgrade().is_none())
                .unwrap());
            format_job_complete(
                JobCompletion {
                    status: crate::src::shared::job::JobExitStatus::Exited(0),
                    output: b"completed\n".to_vec(),
                },
                &entry,
            );
            entry
                .with_record(|record| {
                    assert_eq!(record.out.as_deref(), Some(c"completed"));
                    assert_eq!(record.status, 0);
                })
                .unwrap();
            format_lost_client(&owner);
        }
    }

    #[test]
    fn cache_preserves_identity_and_c_comparator_order() {
        unsafe {
            let owner = client::new();
            let other_owner = client::new();
            let cache = JobCache::for_client(Some(&owner));
            let other = JobCache::for_client(Some(&other_owner));
            let command = CString::new(b"cmd\xff".to_vec()).unwrap();
            let original = cache.entry(Some(&owner), 7, &command);
            original.with_record(|record| record.updated = 42);
            let duplicate = CString::new(command.as_bytes()).unwrap();
            assert!(original
                .identity
                .ptr_eq(&cache.entry(Some(&owner), 7, &duplicate).identity));
            assert!(!original
                .identity
                .ptr_eq(&other.entry(Some(&other_owner), 7, &duplicate).identity));
            assert!(!original
                .identity
                .ptr_eq(&cache.entry(Some(&owner), 8, &duplicate).identity));
            for tag in [0, 7, 8, u_int::MAX] {
                for byte in 1..=255u8 {
                    cache.entry(Some(&owner), tag, &CString::new(vec![byte]).unwrap());
                }
            }
            assert!(original
                .identity
                .ptr_eq(&cache.entry(Some(&owner), 7, &duplicate).identity));
            assert_eq!(original.with_record(|record| record.updated), Some(42));
            let keys = cache
                .with_cache(|cache| {
                    cache
                        .as_ref()
                        .unwrap()
                        .values()
                        .map(|record| (record.tag, record.cmd.clone()))
                        .collect::<Vec<_>>()
                })
                .unwrap();
            for pair in keys.windows(2) {
                let (a, b) = (&pair[0], &pair[1]);
                assert!(a.0 < b.0 || a.0 == b.0 && strcmp(a.1.as_ptr(), b.1.as_ptr()) < 0);
            }
            format_lost_client(&owner);
            format_lost_client(&other_owner);
            assert!(owner.borrow_format_jobs_mut().is_none());
            assert!(other_owner.borrow_format_jobs_mut().is_none());
        }
    }

    #[test]
    fn client_teardown_releases_cache_and_late_callbacks_skip_replacement_entries() {
        unsafe {
            let owner = client::new();
            let cache = JobCache::for_client(Some(&owner));
            let retired = cache.entry(Some(&owner), 0, c"job");
            retired.with_record(|record| record.last = time(std::ptr::null_mut()) + 3600);
            assert_eq!(Rc::strong_count(&owner), 1);
            format_lost_client(&owner);
            assert!(owner.borrow_format_jobs_mut().is_none());
            assert!(retired.identity.upgrade().is_none());
            let replacement = cache.entry(Some(&owner), 0, c"job");
            replacement.with_record(|record| record.out = Some(c"replacement".to_owned()));
            format_job_complete(
                JobCompletion {
                    status: crate::src::shared::job::JobExitStatus::Exited(0),
                    output: b"stale completion\n".to_vec(),
                },
                &retired,
            );
            assert_eq!(
                replacement.with_record(|record| record.out.clone()),
                Some(Some(c"replacement".to_owned()))
            );
            format_lost_client(&owner);
            format_lost_client(&owner);
            let observer = Rc::downgrade(&owner);
            drop(owner);
            assert!(observer.upgrade().is_none());
            assert!(retired.with_record(|_| ()).is_none());
            format_job_complete(
                JobCompletion {
                    status: crate::src::shared::job::JobExitStatus::Exited(0),
                    output: Vec::new(),
                },
                &replacement,
            );
        }
    }

    #[test]
    fn tidy_preserves_expiration_boundary_and_survivor_identity() {
        unsafe {
            let owner = client::new();
            let cache = JobCache::for_client(Some(&owner));
            let now = 10_000;
            let mut survivors = Vec::new();
            for (index, last) in [now - 3600, now - 3599, now - 7200, now + 1, now]
                .into_iter()
                .enumerate()
            {
                let command = CString::new(format!("job-{index}")).unwrap();
                let entry = cache.entry(Some(&owner), 0, &command);
                entry.with_record(|record| {
                    record.last = last;
                    format_job_set_expanded(record, command.clone());
                    format_job_set_out(record, command.clone());
                });
                if last > now || now - last < 3600 {
                    survivors.push((command, entry));
                }
            }
            format_job_tidy_at(&cache, 0, now);
            assert_eq!(
                cache.with_cache(|cache| cache.as_ref().unwrap().len()),
                Some(3)
            );
            for (command, entry) in survivors {
                assert!(entry
                    .identity
                    .ptr_eq(&cache.entry(Some(&owner), 0, &command).identity));
                assert_eq!(
                    entry.with_record(|record| record.out.clone()),
                    Some(Some(command))
                );
            }
            format_job_tidy_at(&cache, 1, now);
            assert!(cache
                .with_cache(|cache| cache.as_ref().unwrap().is_empty())
                .unwrap());
            format_job_tidy_at(&cache, 0, now);
            format_lost_client(&owner);
        }
    }

    #[test]
    fn line_output_is_owned_and_empty_completion_preserves_an_updated_line() {
        unsafe {
            let owner = client::new();
            let cache = JobCache::for_client(Some(&owner));
            let command = CString::new(b"printf '\xff'".to_vec()).unwrap();
            let entry = cache.entry(Some(&owner), 1, &command);
            entry.with_record(|record| {
                format_job_set_out_from_line(record, b"first\0ignored");
                record.updated = 1;
                format_job_set_expanded(record, CString::new(b"expanded\xff".to_vec()).unwrap());
            });
            let output = entry
                .with_record(|record| record.out.clone())
                .flatten()
                .unwrap();
            format_job_complete(
                JobCompletion {
                    status: crate::src::shared::job::JobExitStatus::Exited(0),
                    output: Vec::new(),
                },
                &entry,
            );
            assert_eq!(
                entry.with_record(|record| record.out.clone()),
                Some(Some(c"first".to_owned()))
            );
            entry.with_record(|record| {
                let message = format_job_message(record, b"' not ready>");
                format_job_set_out(record, message);
                assert_eq!(
                    record.out.as_deref().unwrap().to_bytes(),
                    b"<'printf '\xff'' not ready>"
                );
                assert_eq!(
                    record.expanded.as_deref().unwrap().to_bytes(),
                    b"expanded\xff"
                );
            });
            assert!(entry
                .identity
                .ptr_eq(&cache.entry(Some(&owner), 1, &command).identity));
            format_lost_client(&owner);
            assert_eq!(output, c"first");
        }
    }
}
