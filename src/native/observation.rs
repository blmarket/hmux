//! Threaded pane-observation driver for the blocking runtime.

use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use crate::server::{ObservationSignal, Server};

pub(super) struct ObservationWorker {
    stop: Arc<AtomicBool>,
    signal: Arc<ObservationSignal>,
    worker: Mutex<Option<JoinHandle<()>>>,
}

impl ObservationWorker {
    pub(super) fn start(server: Server) -> io::Result<Self> {
        let stop = Arc::new(AtomicBool::new(false));
        let signal = server.observation_signal();
        let worker_stop = Arc::clone(&stop);
        let worker_signal = Arc::clone(&signal);
        let worker = thread::Builder::new()
            .name("hmux-pane-observer".to_string())
            .spawn(move || {
                let mut revision = worker_signal.revision();
                while !worker_stop.load(Ordering::Acquire) {
                    if let Err(error) = server.reconcile_event_observations() {
                        tracing::warn!(target: "hmux::native", %error, "pane observation failed");
                    }
                    revision = worker_signal.wait_after(revision, &worker_stop);
                }
            })?;
        Ok(Self {
            stop,
            signal,
            worker: Mutex::new(Some(worker)),
        })
    }
}

impl Drop for ObservationWorker {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        self.signal.notify();
        if let Ok(mut worker) = self.worker.lock() {
            if let Some(worker) = worker.take() {
                let _ = worker.join();
            }
        }
    }
}
