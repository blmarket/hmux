use super::*;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(in crate::server) enum Command {
    Attach,
    Has,
    New,
    Kill,
    Rename,
    List,
}

impl Command {
    pub(super) fn execute(
        self,
        args: &[String],
        context: &mut CommandContext<'_>,
    ) -> CommandResult {
        let st = &mut *context.state;
        match self {
            Self::List => list_sessions(args, st, context.agents),
            Self::Has => has_session(args, st),
            Self::New => new_session(args, st, context.client),
            Self::Kill if has_flag(args, "-C") => {
                let target = flag_value(args, "-t")
                    .map(|target| target.split(':').next().unwrap_or(target).to_string())
                    .or_else(|| current_session(st));
                match target {
                    Some(name) => match st.clear_session_alerts(&name) {
                        Ok(()) => CommandResult::ok(""),
                        Err(_) => CommandResult::err(format!("can't find session: {name}\n")),
                    },
                    None => CommandResult::err("can't establish current session\n"),
                }
            }
            Self::Kill if has_flag(args, "-a") => {
                let keep = flag_value(args, "-t")
                    .map(|target| target.split(':').next().unwrap_or(target).to_string())
                    .or_else(|| current_session(st));
                match keep {
                    Some(name) => match st.kill_other_sessions(&name) {
                        Ok(()) => CommandResult::ok(""),
                        Err(error) => CommandResult::err(format!("{error}\n")),
                    },
                    None => CommandResult::err("can't establish current session\n"),
                }
            }
            Self::Kill if has_flag(args, "-g") => {
                let target = flag_value(args, "-t")
                    .map(|target| target.split(':').next().unwrap_or(target).to_string())
                    .or_else(|| current_session(st));
                match target {
                    Some(name) if st.kill_session_group(&name) => CommandResult::ok(""),
                    Some(name) => CommandResult::err(format!("can't find session: {name}\n")),
                    None => CommandResult::err("can't establish current session\n"),
                }
            }
            Self::Kill => match flag_value(args, "-t") {
                Some(name) if st.kill_session(name) => CommandResult::ok(""),
                Some(name) => CommandResult::err(format!("can't find session: {name}\n")),
                None => match current_session(st) {
                    Some(name) if st.kill_session(&name) => CommandResult::ok(""),
                    _ => CommandResult::err("can't establish current session\n"),
                },
            },
            Self::Rename => rename_session(args, st),
            Self::Attach => {
                let target = flag_value(args, "-t")
                    .or_else(|| positionals(args, &["-t"]).into_iter().next())
                    .unwrap_or("0");
                if st.find(target).is_none() {
                    CommandResult::err(format!("can't find session: {target}\n"))
                } else {
                    CommandResult::err("open terminal failed: not a terminal\n")
                }
            }
        }
    }
}

#[cfg(test)]
pub(super) const ALL: &[Command] = &[
    Command::Attach,
    Command::Has,
    Command::New,
    Command::Kill,
    Command::Rename,
    Command::List,
];
