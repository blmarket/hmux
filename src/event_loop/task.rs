//! Worker helper for operations without a portable readiness interface.

use std::io;
use std::thread;

use crate::server::command::{CommandRuntime, CommandSuspension, CommandSuspensionResult};
use crate::server::task::{completion_pair, drive_blocking, Completion, Coroutine, TaskState};

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
        let (completion, sender) = completion_pair()?;
        thread::Builder::new()
            .name("hmux-event-command".to_string())
            .spawn(move || sender.complete(suspension.resolve_blocking()))?;
        Ok(completion)
    }
}
