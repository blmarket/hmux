//! Stable access to systemd job-completion watch records.

use core::ffi::{CStr, c_int};

/// The object path and completion state of one pending systemd job.
pub trait SystemdJobWatch {
    /// Builds a systemd job watch owning a copy of the optional path.
    fn from_systemd_job_watch(path: Option<&CStr>, done: c_int) -> Self
    where
        Self: Sized;

    /// Returns the watched systemd object path.
    fn systemd_job_path(&self) -> Option<&CStr>;

    /// Replaces the watched path with an owned copy.
    fn set_systemd_job_path(&mut self, path: Option<&CStr>);

    /// Returns whether the watched job has completed.
    fn systemd_job_done(&self) -> c_int;

    /// Replaces the completion state.
    fn set_systemd_job_done(&mut self, done: c_int);
}

impl SystemdJobWatch for crate::compat::systemd_job_watch {
    fn from_systemd_job_watch(path: Option<&CStr>, done: c_int) -> Self {
        Self {
            path: path.map(CStr::to_owned),
            done,
        }
    }
    fn systemd_job_path(&self) -> Option<&CStr> {
        self.path.as_deref()
    }
    fn set_systemd_job_path(&mut self, path: Option<&CStr>) {
        self.path = path.map(CStr::to_owned);
    }
    fn systemd_job_done(&self) -> c_int {
        self.done
    }
    fn set_systemd_job_done(&mut self, done: c_int) {
        self.done = done;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compat::systemd_job_watch;

    #[test]
    fn job_watch_retains_paths_after_the_source_is_dropped() {
        let source = c"/job/1".to_owned();
        let mut watch = systemd_job_watch::from_systemd_job_watch(Some(&source), 0);
        drop(source);
        assert_eq!(watch.systemd_job_path(), Some(c"/job/1"));
        let replacement = c"/job/2".to_owned();
        watch.set_systemd_job_path(Some(&replacement));
        drop(replacement);
        assert_eq!(watch.systemd_job_path(), Some(c"/job/2"));
    }

    #[test]
    fn job_watch_exposes_path_and_completion() {
        let mut watch = systemd_job_watch::from_systemd_job_watch(Some(c"/job/1"), 0);
        assert_eq!(watch.systemd_job_path(), Some(c"/job/1"));
        assert_eq!(watch.systemd_job_done(), 0);
        watch.set_systemd_job_path(Some(c"/job/2"));
        watch.set_systemd_job_done(1);
        assert_eq!(watch.systemd_job_path(), Some(c"/job/2"));
        assert_eq!(watch.systemd_job_done(), 1);
        watch.set_systemd_job_path(None);
        assert_eq!(watch.systemd_job_path(), None);
    }
}
