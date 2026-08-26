use super::{
    ExecContext, SharedCommandExecution, buffers, clients, configuration, execution, keys, panes,
    server, sessions, windows,
};

/// Closed identity for every tmux command understood by the native server, with
/// the arguments that command parsed out of its argv.
///
/// The outer variant records the feature owner. Each feature module owns the
/// smaller enum that identifies its commands and the argument types they carry.
#[derive(Clone, Debug)]
pub(in crate::server) enum Command {
    Session(sessions::Command),
    Window(windows::Command),
    Pane(panes::Command),
    Keys(keys::Command),
    Configuration(configuration::Command),
    Buffer(buffers::Command),
    Execution(execution::Command),
    Client(clients::Command),
    Server(server::Command),
}

impl Command {
    /// Run the command, waiting wherever it has to.
    ///
    /// The categories whose commands all finish on their own take the state
    /// borrow for exactly their own call; the four that hold a command which
    /// waits — for a shell, a file, a channel, or a client — drive that wait
    /// themselves.
    pub(super) async fn execute(self, context: &mut ExecContext<'_>) -> SharedCommandExecution {
        match self {
            Self::Session(command) => context.sync(|inner| command.execute(inner)),
            Self::Window(command) => context.sync(|inner| command.execute(inner)),
            Self::Pane(command) => context.sync(|inner| command.execute(inner)),
            Self::Keys(command) => context.sync(|inner| command.execute(inner)),
            Self::Server(command) => context.sync(|inner| command.execute(inner)),
            Self::Configuration(command) => command.execute(context).await,
            Self::Buffer(command) => command.execute(context).await,
            Self::Execution(command) => command.execute(context).await,
            Self::Client(command) => command.execute(context).await,
        }
    }
}
