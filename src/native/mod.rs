//! Blocking/threaded runtime adapter for the shared server engine.

use std::io;
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::integration::status::StatusHub;
use crate::observability::v1::{PaneId, PaneObservability, ServerObservability};
use crate::server::{state::ServerState, Server};
use crate::tmux::codec::{split_stream, ImsgReader, ImsgWriter};
use crate::tmux::traits::TmuxServer;

mod attach;
mod observation;
pub(crate) mod pane;
mod protocol;
pub(crate) mod task;

/// The shared server engine hosted by the blocking/threaded runtime.
#[derive(Clone)]
pub struct NativeServer {
    server: Server,
    observation: Arc<observation::ObservationWorker>,
}

impl NativeServer {
    pub fn new() -> io::Result<Self> {
        Self::from_server(Server::new()?)
    }

    pub fn from_server(server: Server) -> io::Result<Self> {
        server
            .state()
            .lock()
            .map_err(|_| io::Error::other("server state mutex poisoned"))?
            .set_pane_io_mode(crate::server::pane::PaneIoMode::Threaded(
                pane::spawn_reader,
            ));
        let observation = Arc::new(observation::ObservationWorker::start(server.clone())?);
        Ok(Self {
            server,
            observation,
        })
    }

    pub fn state(&self) -> Arc<Mutex<ServerState>> {
        self.server.state()
    }

    pub fn status_hub(&self) -> StatusHub {
        self.server.status_hub()
    }

    pub fn shutdown_requested(&self) -> bool {
        self.server
            .state()
            .lock()
            .map(|mut state| {
                state.reap_exited_panes();
                state.shutdown_requested()
            })
            .unwrap_or(true)
    }

    fn spawn_protocol_connection(&self) -> io::Result<UnixStream> {
        let (client, runtime) = UnixStream::pair()?;
        let (reader, writer) = split_stream(runtime)?;
        let server = self.server.clone();
        let observation = Arc::clone(&self.observation);
        thread::spawn(move || {
            let _observation = observation;
            let _ = protocol::handle(reader, writer, server.state(), server.status_hub());
        });
        Ok(client)
    }
}

impl ServerObservability for NativeServer {
    fn pane_ids(&self) -> io::Result<Vec<PaneId>> {
        self.server.pane_ids()
    }

    fn resolve_pane(&self, id: PaneId) -> io::Result<Option<Arc<dyn PaneObservability>>> {
        self.server.resolve_pane(id)
    }
}

impl TmuxServer for NativeServer {
    type Reader = ImsgReader;
    type Writer = ImsgWriter;

    fn connect(&self) -> io::Result<(Self::Reader, Self::Writer)> {
        // SCM_RIGHTS descriptor passing works across the socket pair, so the
        // identify handshake matches a listener connection.
        split_stream(self.spawn_protocol_connection()?)
    }
}
