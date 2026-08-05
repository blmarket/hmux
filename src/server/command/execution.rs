use super::*;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(in crate::server) enum Command {
    RunShell,
    IfShell,
    SourceFile,
    WaitFor,
}

impl Command {
    pub(super) fn execute(
        self,
        _args: &[String],
        _context: &mut CommandContext<'_>,
    ) -> CommandResult {
        match self {
            // Every command here waits on something the loop polls, so the
            // command queue lifts it into a job of its own before dispatch
            // ever reaches this table. Reaching it would mean waiting under
            // the state borrow, which is what the queue exists to avoid.
            Self::RunShell | Self::IfShell | Self::SourceFile | Self::WaitFor => {
                CommandResult::err("not able to wait\n")
            }
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

/// What a `wait-for` did: either it finished, or it is queued behind another
/// client and resumes when the returned completion fires.
pub(super) enum WaitForOutcome {
    Done(CommandResult),
    Pending(Completion<()>),
}

pub(super) fn wait_for(args: &[String], registry: &WaitRegistry) -> WaitForOutcome {
    let channel = positionals(args, &[])
        .into_iter()
        .next()
        .unwrap_or_default();
    let outcome = if has_flag(args, "-S") {
        registry.signal(channel);
        WaitOutcome::Ready
    } else if has_flag(args, "-L") {
        registry.lock(channel)
    } else if has_flag(args, "-U") {
        if !registry.unlock(channel) {
            return WaitForOutcome::Done(CommandResult::err(format!(
                "channel {channel} not locked\n"
            )));
        }
        WaitOutcome::Ready
    } else {
        registry.wait(channel)
    };
    match outcome {
        WaitOutcome::Ready => WaitForOutcome::Done(CommandResult::ok("")),
        WaitOutcome::Pending(completion) => WaitForOutcome::Pending(completion),
    }
}
