//! Worker helper for operations without a portable readiness interface.

use std::io;
use std::thread;

use crate::server::command::{CommandRuntime, CommandSuspension, CommandSuspensionResult};
use crate::server::task::{completion_pair, drive_blocking, Completion, Coroutine, TaskState};

use super::suspend::SuspensionExecutorHandle;

/// Command runtime for clients served by the event loop.
///
/// Suspensions the loop's [`SuspensionExecutor`](super::suspend::SuspensionExecutor)
/// owns are driven on the loop itself; the remaining variants still resolve on
/// a worker thread until the executor grows their jobs.
#[derive(Clone)]
pub(crate) struct EventCommandRuntime {
    executor: Option<SuspensionExecutorHandle>,
}

impl EventCommandRuntime {
    pub(crate) fn new(executor: SuspensionExecutorHandle) -> Self {
        Self {
            executor: Some(executor),
        }
    }

    /// A runtime with no loop behind it: every suspension takes the worker
    /// thread. Test scaffolding for control clients driven outside a loop.
    #[cfg(test)]
    pub(crate) fn detached() -> Self {
        Self { executor: None }
    }

    pub(crate) fn spawn_coroutine<T, F>(mut task: TaskState<T>, complete: F) -> io::Result<()>
    where
        T: Coroutine + Send + 'static,
        T::Output: Send + 'static,
        F: FnOnce(T::Output) + Send + 'static,
    {
        thread::Builder::new()
            .name("hmux-event-coroutine".to_string())
            .spawn(move || {
                drive_blocking(&mut task);
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
        let suspension = match self.executor.as_ref() {
            Some(executor) => match executor.submit(suspension) {
                Ok(completion) => return completion,
                Err(suspension) => suspension,
            },
            None => suspension,
        };
        let (completion, sender) = completion_pair()?;
        thread::Builder::new()
            .name("hmux-event-command".to_string())
            .spawn(move || sender.complete(suspension.resolve_blocking()))?;
        Ok(completion)
    }
}
