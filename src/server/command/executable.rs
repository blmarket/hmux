//! The compiled form of a tmux command line: hmux's `struct cmd_list`.
//!
//! Everything above the command queue trades in [`ExecutableCommand`]. A string
//! (or an already split argv) becomes work by being compiled once —
//! tokenized, split on `;`, its command names and `command-alias` entries
//! resolved, its flags and operand counts validated, and each command shaped by
//! its own parse hook — and the compiled value is then handed to the queue.
//! Storage sites keep it, print it, and clone it; they never keep the text and
//! re-parse it.
//!
//! Below this line sit [`ParsedCommand`], the per-module `Command` enums, and
//! [`CommandSpec`]: implementation detail of the interpreter, not the currency
//! the rest of the server speaks.

use std::collections::BTreeMap;

use super::args::ParsedArgs;
use super::{
    Command, display_command, missing_flag_value, normalize_argv, tokenize_line,
    tokenized_command_groups, unknown_bind_key_flag, unknown_flag,
};
use crate::server::registry::{self, CommandSpec, SpecResolution};
use crate::server::state::SharedState;

/// A command line that may still be waiting to be compiled: hmux's
/// `struct args_command_state`.
///
/// Deferred work — a `-b` command, a key binding the attached client hands
/// back to the queue — carries this instead of an argv, so the moment of
/// compilation is part of the value rather than a property of the caller:
///
/// * [`LazyCommand::Compiled`] was parsed where it was written or stored — a
///   hook body at `set-hook` time, a binding body at `bind-key` time — and
///   fires exactly as it was compiled, whatever `command-alias` says later.
/// * [`LazyCommand::Line`] is still text, so its aliases, its command names,
///   and its parse errors all resolve when it fires. `run-shell -b` and
///   `if-shell -b` branches are this, as they are in tmux.
#[derive(Clone, Debug)]
pub(crate) enum LazyCommand {
    Compiled(ExecutableCommand),
    Line(String),
}

impl LazyCommand {
    /// The compiled line, compiling the text form against the server's current
    /// `command-alias` table if that is what this still is.
    pub(crate) fn compile(self, state: &SharedState) -> Result<ExecutableCommand, String> {
        match self {
            Self::Compiled(command) => Ok(command),
            Self::Line(line) => {
                let aliases = {
                    let state = state.borrow_mut();
                    state.command_aliases()
                };
                ExecutableCommand::compile(&line, &aliases)
            }
        }
    }
}

/// One command of a command line, ready to run.
///
/// The raw (normalized) argv is retained beside the typed command: the queue
/// logs it, the `hook_*` format variables are built from it, and the name-keyed
/// hook planning needs it.
#[derive(Clone, Debug)]
pub(super) struct ParsedCommand {
    pub(super) spec: &'static CommandSpec,
    pub(super) command: Command,
    pub(super) args: Vec<String>,
}

impl ParsedCommand {
    /// This command as tmux's `cmd_print` renders it: the canonical name, the
    /// value-less flags as one sorted cluster, then each valued flag with its
    /// value, then the operands.
    fn print(&self) -> String {
        let lexed = ParsedArgs::lex(self.spec.name, &self.args);
        let mut valueless: BTreeMap<char, usize> = BTreeMap::new();
        let mut valued: BTreeMap<char, Vec<&str>> = BTreeMap::new();
        for (flag, value) in lexed.flags() {
            match value {
                Some(value) => valued.entry(flag).or_default().push(value),
                None => *valueless.entry(flag).or_default() += 1,
            }
        }

        let mut words = vec![self.spec.name.to_string()];
        if !valueless.is_empty() {
            let mut cluster = String::from("-");
            for (flag, count) in &valueless {
                for _ in 0..*count {
                    cluster.push(*flag);
                }
            }
            words.push(cluster);
        }
        for (flag, values) in &valued {
            for value in values {
                words.push(format!("-{flag}"));
                words.push((*value).to_string());
            }
        }
        words.extend(lexed.positionals().iter().cloned());
        display_command(&words)
    }
}

/// A whole command line, compiled once and ready to run any number of times.
///
/// `a ; b ; c` is one value holding three commands in order; chaining is
/// internal to it, and the queue runs the members with tmux's
/// keep-going-on-error sequencing. It is plain data — `Clone`, `Debug`, no
/// captured environment — so a key binding or a command-typed option can hold
/// one, print it, and hand a clone to the queue whenever it fires.
#[derive(Clone, Debug)]
pub(crate) struct ExecutableCommand {
    commands: Vec<ParsedCommand>,
}

impl ExecutableCommand {
    /// Compile a command line written as text: the form config files, control
    /// mode, prompts, menus and command-typed options all carry.
    pub(crate) fn compile(
        source: &str,
        aliases: &[(String, String)],
    ) -> Result<ExecutableCommand, String> {
        let tokens = tokenize_line(source);
        let groups = tokenized_command_groups(&tokens);
        let groups = groups.iter().map(Vec::as_slice).collect::<Vec<_>>();
        Ok(ExecutableCommand {
            commands: expand_aliases_and_parse(groups, aliases)?,
        })
    }

    /// Compile a command line that arrived already split into words: a command
    /// client's argv, or the operands `bind-key` was given.
    ///
    /// tmux's legacy trailing-semicolon form (`neww \;` as one word) is expanded
    /// into the standalone separator token first, so both entry points agree on
    /// where one command ends and the next begins.
    pub(crate) fn compile_argv(
        args: &[String],
        aliases: &[(String, String)],
    ) -> Result<ExecutableCommand, String> {
        let expanded = expand_attached_separators(args);
        Ok(ExecutableCommand {
            commands: expand_aliases_and_parse(split_commands(&expanded), aliases)?,
        })
    }

    /// Compile a line that a *different* lexer produced.
    ///
    /// Configuration files have a lexical layer of their own — `%if`, `%hidden`,
    /// assignments, `{ }` blocks, and stricter escapes — so `source-file` settles
    /// on the words itself and hands them over already split into commands.
    /// Everything from alias expansion on is the same pipeline the other two
    /// entry points use.
    pub(super) fn compile_groups(
        groups: Vec<&[String]>,
        aliases: &[(String, String)],
    ) -> Result<ExecutableCommand, String> {
        Ok(ExecutableCommand {
            commands: expand_aliases_and_parse(groups, aliases)?,
        })
    }

    /// The command line re-printed with every command's canonical name and
    /// argument shape, the way tmux's `cmd_list_print` does for a stored
    /// command option.
    pub(crate) fn print(&self) -> String {
        self.commands
            .iter()
            .map(ParsedCommand::print)
            .collect::<Vec<_>>()
            .join(" ; ")
    }

    /// Whether the line holds no commands at all — empty text, or an alias
    /// whose replacement was empty.
    pub(crate) fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// The whole line back as one flat argv, each command by its canonical name
    /// and normalized arguments, with a standalone `;` between commands.
    ///
    /// The attached-client key path reads a binding this way: it answers a few
    /// keys itself (`detach-client`, `send-prefix`, `copy-mode`) instead of
    /// queueing them, and decides from the words. Recompiling this argv yields
    /// the same commands.
    pub(crate) fn argv(&self) -> Vec<String> {
        let mut argv = Vec::new();
        for command in &self.commands {
            if !argv.is_empty() {
                argv.push(";".to_string());
            }
            argv.push(command.spec.name.to_string());
            argv.extend(command.args.iter().skip(1).cloned());
        }
        argv
    }

    /// Whether every command of the line is one a read-only client may run —
    /// tmux's `cmd_list_all_have(cmdlist, CMD_READONLY)`. An empty line
    /// qualifies, as it does there.
    pub(crate) fn all_read_only(&self) -> bool {
        self.commands.iter().all(|command| command.spec.read_only)
    }

    /// The compiled commands, in order, for the queue to run.
    pub(super) fn into_commands(self) -> Vec<ParsedCommand> {
        self.commands
    }

    /// Append another compiled line's commands to this one.
    ///
    /// A prompt's answer and the words the client typed after it are compiled
    /// separately — they arrive as text and as argv — but run as one line.
    pub(super) fn extend(&mut self, other: ExecutableCommand) {
        self.commands.extend(other.commands);
    }

    /// One compiled value per command of the line.
    ///
    /// The sites that run each member as a queue item of its own — control-mode
    /// granularity, hook bodies — need the members separately, and still hand
    /// the queue nothing but compiled values.
    pub(crate) fn split(self) -> Vec<ExecutableCommand> {
        self.commands
            .into_iter()
            .map(|command| ExecutableCommand {
                commands: vec![command],
            })
            .collect()
    }
}

/// Expand `command-alias` entries over the split groups, then parse each one.
///
/// An alias replaces the whole command word with a command *line*, so its own
/// `;`-separated members are spliced in and the invocation's remaining
/// arguments are appended to the last of them.
fn expand_aliases_and_parse(
    groups: Vec<&[String]>,
    aliases: &[(String, String)],
) -> Result<Vec<ParsedCommand>, String> {
    let mut expanded = Vec::new();
    for group in groups {
        let Some(name) = group.first() else {
            continue;
        };
        let Some((_, replacement)) = aliases.iter().find(|(alias, _)| alias == name) else {
            expanded.push(group.to_vec());
            continue;
        };
        let tokens = tokenize_line(replacement);
        let mut replacements = tokenized_command_groups(&tokens);
        if replacements.is_empty() {
            // A matched alias whose value holds no commands expands to an empty
            // command list, so the invocation succeeds and drops its arguments.
            // Falling back to the alias name would report it as unknown.
            continue;
        }
        replacements
            .last_mut()
            .expect("nonempty replacement")
            .extend(group.iter().skip(1).cloned());
        expanded.extend(replacements);
    }
    let groups = expanded.iter().map(Vec::as_slice).collect::<Vec<_>>();
    parse_groups(groups)
}

/// The parse phase proper: resolve, validate, and shape every command of the
/// line before any of them runs.
fn parse_groups(groups: Vec<&[String]>) -> Result<Vec<ParsedCommand>, String> {
    // Resolve every command's name up front. A bad name aborts the whole line
    // with no output, exactly like tmux's cmd_parse.
    let mut resolved: Vec<(&'static CommandSpec, &[String])> = Vec::with_capacity(groups.len());
    for group in &groups {
        let word = match group.first() {
            Some(w) => w.as_str(),
            None => continue, // empty group (e.g. trailing ';'): tmux ignores it
        };
        match registry::resolve_spec(word) {
            SpecResolution::Spec(spec) => resolved.push((spec, group)),
            SpecResolution::Ambiguous { error } | SpecResolution::Unknown { error } => {
                return Err(error);
            }
        }
    }

    // Still in the parse phase: validate each command's flags against tmux's
    // getopt spec. An unknown flag is a *parse* error too, so (like a bad command
    // name) it aborts the entire line with no output before anything runs.
    for (command, group) in &resolved {
        let getopt = command.getopt;
        let bad = if command.name == "bind-key" {
            unknown_bind_key_flag(group, getopt)
        } else {
            unknown_flag(group, getopt)
        };
        if let Some(bad) = bad {
            return Err(format!("command {}: unknown flag -{bad}\n", command.name));
        }
        if let Some(flag) = missing_flag_value(group, getopt) {
            return Err(format!(
                "command {}: -{flag} expects an argument\n",
                command.name
            ));
        }
    }

    let mut parsed = Vec::with_capacity(resolved.len());
    for (spec, group) in resolved {
        let args = normalize_argv(spec.name, group);
        // Lex the validated argv once: the arity check and the command's own
        // parse hook both read the same operands.
        let lexed = ParsedArgs::lex(spec.name, &args);
        let (minimum, maximum) = spec.arity;
        let count = lexed.positionals().len();
        if count < minimum {
            return Err(format!(
                "command {}: too few arguments (need at least {minimum})\n",
                spec.name
            ));
        }
        if let Some(maximum) = maximum.filter(|maximum| count > *maximum) {
            return Err(format!(
                "command {}: too many arguments (need at most {maximum})\n",
                spec.name
            ));
        }
        // Last parse step: let the command shape itself from its arguments. A
        // rejection here is a parse error like any other, so it aborts the whole
        // line before anything runs.
        let command = (spec.parse)(&lexed)?;
        parsed.push(ParsedCommand {
            spec,
            command,
            args,
        });
    }
    Ok(parsed)
}

/// Expand tmux's legacy trailing-semicolon argv form into the standalone token
/// consumed by [`split_commands`].
fn expand_attached_separators(args: &[String]) -> Vec<String> {
    let mut expanded = Vec::with_capacity(args.len());
    for arg in args {
        if arg.ends_with(r"\;") {
            expanded.push(arg.clone());
            continue;
        }
        if arg != ";" {
            if let Some(word) = arg.strip_suffix(';') {
                if !word.is_empty() {
                    expanded.push(word.to_string());
                }
                expanded.push(";".to_string());
                continue;
            }
        }
        expanded.push(arg.clone());
    }
    expanded
}

/// Split an argv into `;`-separated command groups.
fn split_commands(args: &[String]) -> Vec<&[String]> {
    let mut groups = Vec::new();
    let mut start = 0;
    for (i, a) in args.iter().enumerate() {
        if a == ";" {
            if start < i {
                groups.push(&args[start..i]);
            }
            start = i + 1;
        }
    }
    if start < args.len() {
        groups.push(&args[start..]);
    }
    groups
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compile(source: &str) -> Result<ExecutableCommand, String> {
        ExecutableCommand::compile(source, &[])
    }

    fn words(words: &[&str]) -> Vec<String> {
        words.iter().map(|word| (*word).to_string()).collect()
    }

    fn names(compiled: &ExecutableCommand) -> Vec<&'static str> {
        compiled
            .commands
            .iter()
            .map(|command| command.spec.name)
            .collect()
    }

    #[test]
    fn a_line_compiles_to_one_value_holding_every_command() {
        let compiled = compile("new-window -d ; list-windows ; kill-pane").expect("compiles");
        assert_eq!(
            names(&compiled),
            ["new-window", "list-windows", "kill-pane"]
        );
    }

    #[test]
    fn a_line_is_read_only_only_when_every_command_of_it_is() {
        // tmux's `cmd_list_all_have`: one command outside the read-only set
        // takes the whole line with it, and an empty line qualifies.
        assert!(compile("detach-client").expect("compiles").all_read_only());
        assert!(
            compile("list-clients ; copy-mode")
                .expect("compiles")
                .all_read_only()
        );
        assert!(
            !compile("detach-client ; kill-window")
                .expect("compiles")
                .all_read_only()
        );
        assert!(!compile("new-window").expect("compiles").all_read_only());
        assert!(compile("").expect("compiles").all_read_only());
    }

    #[test]
    fn an_escaped_separator_stays_inside_its_command() {
        // `\;` is a literal argument, not a command separator.
        let compiled = compile(r"send-keys \; ; list-windows").expect("compiles");
        assert_eq!(names(&compiled), ["send-keys", "list-windows"]);
        assert_eq!(compiled.commands[0].args, words(&["send-keys", ";"]));
    }

    #[test]
    fn a_trailing_separator_contributes_no_command() {
        assert_eq!(
            names(&compile("list-windows ;").expect("compiles")),
            ["list-windows"]
        );
        assert!(compile("").expect("compiles").is_empty());
    }

    #[test]
    fn print_canonicalizes_names_flags_and_quoting() {
        // tmux's cmd_print: canonical name, value-less flags as one sorted
        // cluster, then each valued flag, then the operands.
        assert_eq!(
            compile(r#"neww -dP -n "x y""#).expect("compiles").print(),
            r#"new-window -Pd -n "x y""#
        );
        assert_eq!(
            compile("kill-pane -t %1 -a").expect("compiles").print(),
            "kill-pane -a -t %1"
        );
        // A repeated flag prints once per occurrence, as tmux's args_print does.
        assert_eq!(
            compile("display-popup -E -E").expect("compiles").print(),
            "display-popup -EE"
        );
        assert_eq!(
            compile("neww -d ; lsw").expect("compiles").print(),
            "new-window -d ; list-windows"
        );
    }

    #[test]
    fn print_round_trips_through_compile() {
        let once = compile(r#"neww -dP -n "x y" -e A=1"#)
            .expect("compiles")
            .print();
        let twice = compile(&once).expect("recompiles").print();
        assert_eq!(once, twice);
    }

    #[test]
    fn a_bad_line_aborts_before_any_command_is_kept() {
        assert_eq!(
            compile("list-windows ; frobnicate ; kill-pane").unwrap_err(),
            "unknown command: frobnicate\n"
        );
        assert!(
            compile("ne")
                .unwrap_err()
                .starts_with("ambiguous command: ne, could be:")
        );
        assert_eq!(
            compile("kill-pane -Q").unwrap_err(),
            "command kill-pane: unknown flag -Q\n"
        );
        assert_eq!(
            compile("kill-pane -t").unwrap_err(),
            "command kill-pane: -t expects an argument\n"
        );
        assert_eq!(
            compile("rename-window").unwrap_err(),
            "command rename-window: too few arguments (need at least 1)\n"
        );
        assert_eq!(
            compile("kill-pane extra").unwrap_err(),
            "command kill-pane: too many arguments (need at most 0)\n"
        );
    }

    #[test]
    fn an_alias_expands_to_a_whole_command_line_at_compile_time() {
        let aliases = vec![("two".to_string(), "neww -d ; lsw".to_string())];
        let compiled = ExecutableCommand::compile("two -t 0", &aliases).expect("compiles");
        assert_eq!(names(&compiled), ["new-window", "list-windows"]);
        // The invocation's remaining arguments land on the last member.
        assert_eq!(compiled.commands[1].args, words(&["lsw", "-t", "0"]));
        // An alias whose value holds no commands drops the invocation entirely.
        let empty = vec![("nothing".to_string(), String::new())];
        assert!(
            ExecutableCommand::compile("nothing arg", &empty)
                .expect("compiles")
                .is_empty()
        );
        // The table is a snapshot: without it the same line is unknown.
        assert_eq!(compile("two -t 0").unwrap_err(), "unknown command: two\n");
    }

    #[test]
    fn split_and_extend_keep_the_line_in_order() {
        let compiled = compile("neww -d ; lsw ; kill-pane").expect("compiles");
        let members = compiled.clone().split();
        assert_eq!(
            members
                .iter()
                .map(ExecutableCommand::print)
                .collect::<Vec<_>>(),
            ["new-window -d", "list-windows", "kill-pane"]
        );
        let mut rejoined = ExecutableCommand::compile("", &[]).expect("empty compiles");
        for member in members {
            rejoined.extend(member);
        }
        assert_eq!(rejoined.print(), compiled.print());
    }

    #[test]
    fn compile_and_compile_argv_agree_on_equivalent_input() {
        let line = compile("neww -dP ; lsw -t 0").expect("compiles");
        let argv =
            ExecutableCommand::compile_argv(&words(&["neww", "-dP", ";", "lsw", "-t", "0"]), &[])
                .expect("compiles");
        assert_eq!(line.print(), argv.print());
        // tmux's legacy attached separator is the same split.
        let attached =
            ExecutableCommand::compile_argv(&words(&["neww", "-dP;", "lsw", "-t", "0"]), &[])
                .expect("compiles");
        assert_eq!(line.print(), attached.print());
    }
}
