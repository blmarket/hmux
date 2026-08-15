use super::{
    buffers, clients, configuration, execution, keys, panes, server, sessions, windows,
    CommandContext, CommandResult,
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
    pub(super) fn execute(self, context: &mut CommandContext<'_>) -> CommandResult {
        match self {
            Self::Session(command) => command.execute(context),
            Self::Window(command) => command.execute(context),
            Self::Pane(command) => command.execute(context),
            Self::Keys(command) => command.execute(context),
            Self::Configuration(command) => command.execute(context),
            Self::Buffer(command) => command.execute(context),
            Self::Execution(command) => command.execute(context),
            Self::Client(command) => command.execute(context),
            Self::Server(command) => command.execute(context),
        }
    }
}
