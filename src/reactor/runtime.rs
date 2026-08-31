use std::cell::RefCell;
use std::time::Duration;

use hmux_rt::TaskRuntime;

use super::Reactor;
use super::registry::RuntimeControl;
use super::stream::StreamRegistry;

const DISPATCH_BUDGET: usize = 64;
const TURN_TIMEOUT: Duration = Duration::from_millis(10);

pub(crate) struct RuntimeHost {
    runtime: Option<TaskRuntime>,
    control: RuntimeControl,
    streams: StreamRegistry,
    pid: libc::pid_t,
}

impl RuntimeHost {
    fn new() -> Self {
        let control = CONTROL.with(Clone::clone);
        let streams = STREAMS.with(Clone::clone);
        Self {
            runtime: None,
            control,
            streams,
            pid: unsafe { libc::getpid() },
        }
    }

    fn activate(&mut self) -> std::io::Result<()> {
        self.activate_inner(false)
    }

    fn rebuild(&mut self) -> std::io::Result<()> {
        self.activate_inner(true)
    }

    fn activate_inner(&mut self, force: bool) -> std::io::Result<()> {
        let pid = unsafe { libc::getpid() };
        if !force && self.runtime.is_some() && self.pid == pid {
            return Ok(());
        }
        self.runtime = None;
        let runtime = TaskRuntime::new()?;
        let handle = runtime.handle();
        self.control.respawn_active(&handle);
        self.streams.respawn_active(&handle);
        self.runtime = Some(runtime);
        self.pid = pid;
        Ok(())
    }

    fn run_once(&mut self) -> std::io::Result<()> {
        self.activate()?;
        let epoch = self.control.begin_epoch();
        let runtime = self.runtime.as_mut().expect("runtime activated");
        runtime.dispatch(DISPATCH_BUDGET)?;

        for deferred in self.control.take_ready_deferred(epoch) {
            (deferred.callback)();
        }

        if runtime.pending() == 0 {
            runtime.poll(Some(TURN_TIMEOUT))?;
        }
        runtime.dispatch(DISPATCH_BUDGET)?;
        Ok(())
    }
}

thread_local! {
    static CONTROL: RuntimeControl = RuntimeControl::new();
    static STREAMS: StreamRegistry = StreamRegistry::new();
    static HOST: RefCell<RuntimeHost> = RefCell::new(RuntimeHost::new());
}

#[derive(Copy, Clone)]
pub struct Base;

pub(crate) fn runtime_control() -> RuntimeControl {
    CONTROL.with(Clone::clone)
}

pub(crate) fn stream_registry() -> StreamRegistry {
    STREAMS.with(Clone::clone)
}

/// Drops everything the reactor holds for the rest of the crate, and then the
/// runtime underneath it, leaving the reactor itself the last thing standing.
///
/// Call this on the way out of the process, before `exit`. Client, job and
/// buffer teardown reaches back into the reactor from its own `Drop`, and the
/// reactor lives in thread-local storage, which `exit` destroys before it runs
/// anything else — so a teardown left queued at that point would be asking a
/// reactor that is already half gone to disarm its timers. Running it here
/// keeps every one of those drops in a healthy reactor, and leaves the
/// thread-local with nothing that can call back into it.
pub fn shutdown() {
    runtime_control().shutdown();
    stream_registry().shutdown();
    let runtime = HOST.with(|host| host.borrow_mut().runtime.take());
    drop(runtime);
}

/// The loop this process runs on. `Base` carries no state — every handle
/// reaches the runtime through thread-local storage — so this is the one way
/// to name it, whether the caller is starting the loop up or reaching it from
/// somewhere the tree does not thread it through.
pub fn current() -> Base {
    Base
}

impl Reactor for Base {
    fn run_once(&mut self) {
        HOST.with(|host| {
            host.borrow_mut()
                .run_once()
                .expect("hmux-rt reactor turn failed");
        });
    }

    fn reinit(&mut self) -> bool {
        HOST.with(|host| host.borrow_mut().rebuild().is_ok())
    }

    fn defer(&mut self, callback: impl FnOnce() + 'static) {
        runtime_control().defer(callback);
    }

    fn describe(&self) -> String {
        "hmux-rt (mio)".to_owned()
    }
}
