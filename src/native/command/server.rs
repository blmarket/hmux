use super::*;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(in crate::native) enum Command {
    ListCommands,
    Start,
    Kill,
    Access,
    ShowMessages,
    Lock,
    LockSession,
}

impl Command {
    pub(super) fn execute(
        self,
        args: &[String],
        context: &mut CommandContext<'_>,
    ) -> CommandResult {
        match self {
            Self::ListCommands => list_commands(args),
            Self::Start => CommandResult::ok(""),
            Self::Kill => {
                context.state.kill_server();
                CommandResult::ok("")
            }
            Self::Access => CommandResult::err("server-access is not supported\n"),
            Self::ShowMessages => show_messages(args, context.state),
            Self::Lock => lock_server(context.state),
            Self::LockSession => lock_session(args, context.state),
        }
    }
}

#[cfg(test)]
pub(super) const ALL: &[Command] = &[
    Command::ListCommands,
    Command::Start,
    Command::Kill,
    Command::Access,
    Command::ShowMessages,
    Command::Lock,
    Command::LockSession,
];
