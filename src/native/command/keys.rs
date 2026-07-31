use super::*;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(in crate::native) enum Command {
    Bind,
    Unbind,
    List,
    Send,
    SendPrefix,
}

impl Command {
    pub(super) fn execute(
        self,
        args: &[String],
        context: &mut CommandContext<'_>,
    ) -> CommandResult {
        match self {
            Self::Bind => bind_key(args, context.state),
            Self::Unbind => unbind_key(args, context.state),
            Self::List => list_keys(args, context.state),
            Self::Send => super::super::cmd_send_keys::exec(
                args,
                context.state,
                context.agents,
                context.client,
            ),
            Self::SendPrefix => {
                super::super::cmd_send_keys::exec_prefix(args, context.state, context.client)
            }
        }
    }
}

#[cfg(test)]
pub(super) const ALL: &[Command] = &[
    Command::Bind,
    Command::Unbind,
    Command::List,
    Command::Send,
    Command::SendPrefix,
];
