//! Application-owned futures with explicit and drop cancellation.
use super::ensure_runtime;
use hmux_rt::Handle as _;
use std::future::Future;
use std::io;

/// Replace the owner's task. Dropping the runtime task cancels it.
/// Owners that use the slot to track active work must clear it on completion.
pub fn task_start<F>(
    task: &mut Option<hmux_rt::mio::Task>,
    initialize: impl FnOnce() -> io::Result<F>,
) -> io::Result<()>
where
    F: Future<Output = ()> + 'static,
{
    drop(task.take());
    ensure_runtime();
    *task = Some(hmux_rt::mio::Handle::current().spawn(initialize()?)?);
    Ok(())
}
