//! The lexed view of one command's argv that per-command parsing reads.
//!
//! tmux lexes a command's arguments once, in `args_parse`, and every handler
//! then asks the resulting `struct args` for a flag or an operand. The native
//! engine grew four ad-hoc scanners over the raw argv instead; this type is the
//! single lexer they collapse into.
//!
//! It is built from the *normalized* argv (see `normalize_argv`), so every flag
//! is already its own `-x` token and every flag value its own following token.
//! Unknown flags and missing flag values were already rejected against the
//! command's getopt spec by the parse phase, so lexing here cannot fail: it only
//! shapes input that is known to be valid.

use crate::server::registry;

/// One command's arguments after lexing: its flags in occurrence order, and its
/// operands.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(in crate::server) struct ParsedArgs {
    /// Every flag occurrence in the order it was written. A value-taking flag
    /// carries its value; a boolean one carries `None`. Occurrence order is
    /// kept because tmux's repeatable flags (`-e` on `new-session`) are read as
    /// a list, and its single-valued ones resolve to the last occurrence.
    flags: Vec<(char, Option<String>)>,
    positionals: Vec<String>,
}

impl ParsedArgs {
    /// Lex a normalized argv (index 0 is the command word) for `name`.
    pub(in crate::server) fn lex(name: &str, args: &[String]) -> Self {
        let spec = registry::getopt(name).unwrap_or("");
        let mut flags = Vec::new();
        let mut index = 1;
        while index < args.len() {
            let argument = args[index].as_str();
            if argument == "--" {
                // End of options; `--` itself is not an operand.
                index += 1;
                break;
            }
            let Some(flag) = lone_flag(argument) else {
                // tmux's args_parse stops looking for flags at the first
                // operand, and so does normalize_argv: everything from here on
                // is an operand, option-looking or not.
                break;
            };
            match registry::flag_kind(spec, flag) {
                Some(true) => {
                    // normalize_argv split the value into its own token, and
                    // missing_flag_value rejected the argv that has none.
                    flags.push((flag, args.get(index + 1).cloned()));
                    index += 2;
                }
                Some(false) => {
                    flags.push((flag, None));
                    index += 1;
                }
                // Not a flag of this command: normalize_argv passed it through,
                // so treat it as the first operand rather than inventing a flag.
                None => break,
            }
        }
        Self {
            flags,
            positionals: args[index.min(args.len())..].to_vec(),
        }
    }

    /// Whether a flag was given at all, value-taking or not.
    pub(in crate::server) fn has(&self, flag: char) -> bool {
        self.flags.iter().any(|(letter, _)| *letter == flag)
    }

    /// The flag's value, or `None` if it was not given. tmux's `args_get`
    /// returns the last value recorded for a flag, so a repeated `-t a -t b`
    /// reads as `b`.
    pub(in crate::server) fn value(&self, flag: char) -> Option<&str> {
        self.flags
            .iter()
            .rev()
            .find(|(letter, _)| *letter == flag)
            .and_then(|(_, value)| value.as_deref())
    }

    /// How many times a flag was given. `display-popup` reads its `-E` this
    /// way: one closes the popup when the command exits, two only when it
    /// succeeds.
    pub(in crate::server) fn count(&self, flag: char) -> usize {
        self.flags
            .iter()
            .filter(|(letter, _)| *letter == flag)
            .count()
    }

    /// Every value given for a repeatable flag, in the order written.
    pub(in crate::server) fn values(&self, flag: char) -> impl Iterator<Item = &str> {
        self.flags
            .iter()
            .filter(move |(letter, _)| *letter == flag)
            .filter_map(|(_, value)| value.as_deref())
    }

    /// The operands, in order. For a command whose last operand is another
    /// command line (`new-window /bin/sh -c script`), the option-looking words
    /// after it are operands too, exactly as tmux passes them through.
    pub(in crate::server) fn positionals(&self) -> &[String] {
        &self.positionals
    }
}

/// The flag letter of a bare `-x` token, if that is what this argument is.
fn lone_flag(argument: &str) -> Option<char> {
    let rest = argument.strip_prefix('-')?;
    let mut characters = rest.chars();
    match (characters.next(), characters.next()) {
        (Some(letter), None) => Some(letter),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex(name: &str, words: &[&str]) -> ParsedArgs {
        let args: Vec<String> = words.iter().map(|word| (*word).to_string()).collect();
        let normalized = super::super::normalize_argv(name, &args);
        ParsedArgs::lex(name, &normalized)
    }

    #[test]
    fn separates_flags_from_operands() {
        let args = lex("kill-window", &["kill-window", "-a", "-t", "0"]);
        assert!(args.has('a'));
        assert_eq!(args.value('t'), Some("0"));
        assert!(args.positionals().is_empty());
    }

    #[test]
    fn absorbs_clustered_and_attached_forms() {
        // normalize_argv splits `-ga` and `-t0`; the lexer sees the canonical
        // shape either way.
        let clustered = lex("set-option", &["set-option", "-gt0", "status", "on"]);
        assert!(clustered.has('g'));
        assert_eq!(clustered.value('t'), Some("0"));
        assert_eq!(clustered.positionals(), ["status", "on"]);
    }

    #[test]
    fn single_valued_flag_takes_the_last_occurrence() {
        // tmux's args_get returns the last value recorded for a flag.
        let args = lex("kill-session", &["kill-session", "-t", "a", "-t", "b"]);
        assert_eq!(args.value('t'), Some("b"));
    }

    #[test]
    fn repeatable_flag_keeps_every_value_in_order() {
        let args = lex(
            "new-session",
            &["new-session", "-e", "A=1", "-d", "-e", "B=2"],
        );
        assert_eq!(args.values('e').collect::<Vec<_>>(), ["A=1", "B=2"]);
        assert!(args.has('d'));
    }

    #[test]
    fn options_end_at_the_first_operand() {
        // Everything after the pane command belongs to that command, including
        // its own option-looking words.
        let args = lex("new-window", &["new-window", "-d", "/bin/sh", "-c", "echo"]);
        assert!(args.has('d'));
        assert_eq!(args.positionals(), ["/bin/sh", "-c", "echo"]);
    }

    #[test]
    fn double_dash_ends_options_without_becoming_an_operand() {
        let args = lex("send-keys", &["send-keys", "-l", "--", "-n"]);
        assert!(args.has('l'));
        assert_eq!(args.positionals(), ["-n"]);
    }

    #[test]
    fn missing_flag_is_absent_rather_than_empty() {
        let args = lex("list-sessions", &["list-sessions"]);
        assert!(!args.has('F'));
        assert_eq!(args.value('F'), None);
        assert_eq!(args.values('F').count(), 0);
    }
}
