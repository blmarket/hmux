use super::{
    buffers, clients, configuration, execution, keys, panes, server, sessions, windows,
    CommandContext, CommandResult,
};

/// Closed identity for every tmux command understood by the native server.
///
/// The outer variant records the feature owner. Each feature module owns the
/// smaller enum that identifies its commands.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(in crate::native) enum Command {
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
    pub(super) fn execute(
        self,
        args: &[String],
        context: &mut CommandContext<'_>,
    ) -> CommandResult {
        match self {
            Self::Session(command) => command.execute(args, context),
            Self::Window(command) => command.execute(args, context),
            Self::Pane(command) => command.execute(args, context),
            Self::Keys(command) => command.execute(args, context),
            Self::Configuration(command) => command.execute(args, context),
            Self::Buffer(command) => command.execute(args, context),
            Self::Execution(command) => command.execute(args, context),
            Self::Client(command) => command.execute(args, context),
            Self::Server(command) => command.execute(args, context),
        }
    }
}

#[cfg(test)]
pub(in crate::native) fn all() -> Vec<Command> {
    sessions::ALL
        .iter()
        .copied()
        .map(Command::Session)
        .chain(windows::ALL.iter().copied().map(Command::Window))
        .chain(panes::ALL.iter().copied().map(Command::Pane))
        .chain(keys::ALL.iter().copied().map(Command::Keys))
        .chain(
            configuration::ALL
                .iter()
                .copied()
                .map(Command::Configuration),
        )
        .chain(buffers::ALL.iter().copied().map(Command::Buffer))
        .chain(execution::ALL.iter().copied().map(Command::Execution))
        .chain(clients::ALL.iter().copied().map(Command::Client))
        .chain(server::ALL.iter().copied().map(Command::Server))
        .collect()
}
