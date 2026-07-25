use super::*;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(in crate::tmux::native) enum Command {
    Set,
    Load,
    Show,
    Save,
    List,
    Delete,
    Paste,
}

impl Command {
    pub(super) fn execute(
        self,
        args: &[String],
        context: &mut CommandContext<'_>,
    ) -> CommandResult {
        match self {
            Self::Set => set_buffer(args, context.state, context.client),
            Self::Load => load_buffer(args, context.state, context.client),
            Self::Show => show_buffer(args, context.state),
            Self::Save => save_buffer(args, context.state),
            Self::List => list_buffers(args, context.state),
            Self::Delete => delete_buffer(args, context.state),
            Self::Paste => paste_buffer(args, context.state),
        }
    }
}

#[cfg(test)]
pub(super) const ALL: &[Command] = &[
    Command::Set,
    Command::Load,
    Command::Show,
    Command::Save,
    Command::List,
    Command::Delete,
    Command::Paste,
];
