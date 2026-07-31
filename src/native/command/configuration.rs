use super::*;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(in crate::native) enum Command {
    SetEnvironment,
    ShowEnvironment,
    SetOption,
    ShowOptions,
    SetWindowOption,
    ShowWindowOptions,
    SetHook,
    ShowHooks,
}

impl Command {
    pub(super) fn execute(
        self,
        args: &[String],
        context: &mut CommandContext<'_>,
    ) -> CommandResult {
        match self {
            Self::SetEnvironment => set_environment(args, context.state),
            Self::ShowEnvironment => show_environment(args, context.state),
            Self::SetOption => set_option(args, context.state, false),
            Self::ShowOptions => show_options(args, context.state, false),
            Self::SetWindowOption => set_option(args, context.state, true),
            Self::ShowWindowOptions => show_options(args, context.state, true),
            Self::SetHook => set_hook(args, context.state, context.agents, context.client),
            Self::ShowHooks => show_hooks(args, context.state),
        }
    }
}

#[cfg(test)]
pub(super) const ALL: &[Command] = &[
    Command::SetEnvironment,
    Command::ShowEnvironment,
    Command::SetOption,
    Command::ShowOptions,
    Command::SetWindowOption,
    Command::ShowWindowOptions,
    Command::SetHook,
    Command::ShowHooks,
];
