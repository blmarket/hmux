//! Worker helper for operations without a portable readiness interface.

use std::io;
use std::os::fd::AsRawFd;
use std::thread;
use std::time::Instant;

use crate::server::command::{CommandRuntime, CommandSuspension, CommandSuspensionResult};
use crate::server::task::{completion_pair, Completion, Coroutine, ReadySet, TaskState};

#[derive(Clone, Default)]
pub(crate) struct EventCommandRuntime;

impl EventCommandRuntime {
    pub(crate) fn spawn_coroutine<T, F>(mut task: TaskState<T>, complete: F) -> io::Result<()>
    where
        T: Coroutine + Send + 'static,
        T::Output: Send + 'static,
        F: FnOnce(T::Output) + Send + 'static,
    {
        thread::Builder::new()
            .name("hmux-event-coroutine".to_string())
            .spawn(move || {
                task.poll(&ReadySet::default());
                while task.wait().is_some() {
                    let ready = {
                        let wait = task.wait().expect("pending task has a wait request");
                        assert!(
                            wait.sources().len() <= 1,
                            "event worker supports one descriptor per coroutine"
                        );
                        let mut descriptor = wait.sources().first().map(|source| libc::pollfd {
                            fd: source.fd().as_raw_fd(),
                            events: libc::POLLIN,
                            revents: 0,
                        });
                        let timeout = wait.deadline().map_or(-1, |deadline| {
                            let duration = deadline.saturating_duration_since(Instant::now());
                            let millis = duration.as_nanos().saturating_add(999_999) / 1_000_000;
                            i32::try_from(millis).unwrap_or(i32::MAX)
                        });
                        let (pointer, count) = descriptor
                            .as_mut()
                            .map_or((std::ptr::null_mut(), 0), |descriptor| {
                                (descriptor as *mut libc::pollfd, 1)
                            });
                        let source_ready = loop {
                            let result = unsafe { libc::poll(pointer, count, timeout) };
                            if result >= 0
                                || io::Error::last_os_error().kind() != io::ErrorKind::Interrupted
                            {
                                break result > 0;
                            }
                        };
                        ReadySet::after_single_source_wakeup(&wait, source_ready, Instant::now())
                    };
                    task.poll(&ready);
                }
                if let Some(output) = task.take_output() {
                    complete(output);
                }
            })?;
        Ok(())
    }

    pub(crate) fn spawn_blocking<T, W, F>(work: W, complete: F) -> io::Result<()>
    where
        T: Send + 'static,
        W: FnOnce() -> T + Send + 'static,
        F: FnOnce(T) + Send + 'static,
    {
        thread::Builder::new()
            .name("hmux-event-helper".to_string())
            .spawn(move || complete(work()))?;
        Ok(())
    }
}

impl CommandRuntime for EventCommandRuntime {
    fn submit(
        &self,
        suspension: CommandSuspension,
    ) -> io::Result<Completion<CommandSuspensionResult>> {
        let (completion, sender) = completion_pair()?;
        thread::Builder::new()
            .name("hmux-event-command".to_string())
            .spawn(move || sender.complete(suspension.resolve_blocking()))?;
        Ok(completion)
    }
}
