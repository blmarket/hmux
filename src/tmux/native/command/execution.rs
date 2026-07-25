use super::*;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(in crate::tmux::native) enum Command {
    RunShell,
    IfShell,
    SourceFile,
    WaitFor,
}

impl Command {
    pub(super) fn execute(
        self,
        args: &[String],
        context: &mut CommandContext<'_>,
    ) -> CommandResult {
        match self {
            Self::RunShell => run_shell(args, context.state, context.agents, context.client),
            Self::IfShell => if_shell(args, context.state, context.agents, context.client),
            Self::SourceFile => source_file(args, context.state, context.agents, context.client),
            Self::WaitFor => CommandResult::err("not able to wait\n"),
        }
    }
}

#[cfg(test)]
pub(super) const ALL: &[Command] = &[
    Command::RunShell,
    Command::IfShell,
    Command::SourceFile,
    Command::WaitFor,
];

pub(super) fn wait_for(args: &[String], registry: &WaitRegistry) -> CommandResult {
    let channel = positionals(args, &[])
        .into_iter()
        .next()
        .unwrap_or_default();
    if has_flag(args, "-S") {
        registry.signal(channel);
    } else if has_flag(args, "-L") {
        registry.lock(channel);
    } else if has_flag(args, "-U") {
        if !registry.unlock(channel) {
            return CommandResult::err(format!("channel {channel} not locked\n"));
        }
    } else {
        registry.wait(channel);
    }
    CommandResult::ok("")
}
