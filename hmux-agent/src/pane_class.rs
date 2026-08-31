//! What a pane is doing, as the compact `#{pane_state_emoji}` label reports it.
//!
//! This is the non-agent half of that variable, kept beside the agent
//! detectors because the two halves share the glyph contract: every label is a
//! single codepoint two columns wide. A host builds a [`PaneProcessProbe`]
//! from its own pty descriptor and asks [`PaneClass::classify`] what to draw.

use std::path::Path;

use crate::platform::{CurrentPlatform, Platform};

/// A pane's foreground-process identity, captured while the pane was at hand.
///
/// Holds only owned data (pids and the spawn command line), so a deferred
/// format callback can resolve the process-derived variables long after the
/// pane borrow ended. A pid whose process has exited simply fails its
/// `/proc` read and falls through, exactly as the live path always has.
#[derive(Clone)]
pub struct PaneProcessProbe {
    foreground: Option<libc::pid_t>,
    session_leader: Option<libc::pid_t>,
    fallback_command: Option<String>,
}

impl PaneProcessProbe {
    /// Build a probe from what the host already knows about the pane's pty:
    /// the foreground process group and session leader it reports, and the
    /// command line the pane was spawned with as the fallback for a group
    /// whose leader has exited.
    pub fn new(
        foreground: Option<libc::pid_t>,
        session_leader: Option<libc::pid_t>,
        fallback_command: Option<String>,
    ) -> Self {
        PaneProcessProbe {
            foreground,
            session_leader,
            fallback_command,
        }
    }

    /// The working directory of the pane's foreground process group
    /// (`#{pane_current_path}`).
    ///
    /// Mirrors tmux's `osdep_get_cwd`: prefer the foreground group, then fall
    /// back to the session leader — the group id is only a pid while the
    /// group's leader lives, and a shell pipeline whose first member exited
    /// leaves a group that names no process.
    pub fn current_path(&self) -> Option<String> {
        [self.foreground, self.session_leader]
            .into_iter()
            .flatten()
            .find_map(|pid| CurrentPlatform::process_cwd(pid as u32))
            .map(|path| path.to_string_lossy().into_owned())
    }

    /// The program occupying the pane's foreground process group
    /// (`#{pane_current_command}`).
    ///
    /// Mirrors tmux's `format_cb_current_command`, which tries the foreground
    /// group's `argv[0]` and then falls back to the pane's own command line —
    /// the fallback is what answers for a leaderless group.
    pub fn current_command(&self) -> Option<String> {
        // tmux's Linux osdep_get_name reads argv[0] from /proc/PID/cmdline.
        // Keep the executable-name candidates as a fallback for platforms or
        // processes where the argument vector is unavailable.
        let foreground = self
            .foreground
            .and_then(|pid| {
                CurrentPlatform::process_arguments(pid as u32)
                    .into_iter()
                    .next()
                    .or_else(|| {
                        CurrentPlatform::process_programs(pid as u32)
                            .into_iter()
                            .next()
                    })
            })
            .map(|program| program.to_string_lossy().into_owned());

        foreground
            .filter(|program| !program.is_empty())
            .or_else(|| self.fallback_command.clone())
            .map(|command| parse_window_name(&command))
            .filter(|name| !name.is_empty())
    }

    /// Whether the shell holding the terminal is sitting at a prompt with
    /// nothing running in front of it.
    ///
    /// The program holding the terminal is the pane's foreground group, which
    /// is the pane's own shell until something it launched takes over — and a
    /// shell started from that shell is a shell at a prompt just as much as the
    /// pane's own is, so what the test asks is what the foreground program is
    /// rather than whether it is the process the pane forked.
    ///
    /// The program has to be a shell, and a shell handed work of its own
    /// (`sh -c 'while :; do :; done'`, or a script to run) is working rather
    /// than prompting. A pane launched straight into a program
    /// (`new-window -- tail -f log`) is not a shell at all, so it never gets
    /// here.
    ///
    /// What this cannot do is ask the shell whether it is currently at its
    /// prompt: bash and dash sit in a `poll` loop there, which is
    /// indistinguishable from any other wait. Reading the invocation is what
    /// is left, and it is right for the panes that matter — an interactive
    /// shell has no work on its command line, and a shell given some never
    /// returns to a prompt.
    fn shell_at_prompt(&self) -> bool {
        self.current_command()
            .is_some_and(|command| is_shell(&command))
            && !self.runs_a_command_string()
    }

    /// Whether the foreground process was invoked with work of its own — `-c`,
    /// or a script to run — rather than started interactively.
    ///
    /// An unreadable argument vector reads as no work, leaving the shell to be
    /// treated as interactive — which is what a pane's shell usually is.
    fn runs_a_command_string(&self) -> bool {
        let Some(pid) = self.foreground else {
            return false;
        };
        let arguments = CurrentPlatform::process_arguments(pid as u32);
        invoked_with_command_string(arguments.iter().filter_map(|argument| argument.to_str()))
    }

    /// Whether the foreground command is parked in a terminal read.
    ///
    /// The foreground process *group* id is a pid only while the group's
    /// leader lives, so this answers for the leader and reads a leaderless
    /// group — a pipeline outliving its first member — as not waiting. Which
    /// is the conservative direction: it never claims a pane wants you.
    fn waiting_for_tty(&self) -> bool {
        self.foreground
            .and_then(|pid| CurrentPlatform::process_waiting_for_tty(pid as u32))
            .unwrap_or(false)
    }
}

/// Whether a shell's argument vector carries work of its own: a `-c` command
/// string, or a script operand.
///
/// Only the leading option arguments are scanned for `-c`: everything from the
/// first non-option onwards is the command string and its own arguments, which
/// may contain anything at all. `--` ends the options, and whatever follows it
/// is a script to run.
///
/// Within the options, only single-dash arguments are short-option bundles, so
/// `-lc` counts while a long option merely spelled with a `c` in it — `--norc`
/// — does not.
fn invoked_with_command_string<'a>(arguments: impl Iterator<Item = &'a str>) -> bool {
    let mut rest = arguments.skip(1).peekable();
    let mut options = Vec::new();
    while let Some(argument) = rest.peek() {
        if !argument.starts_with('-') {
            break;
        }
        let argument = rest.next().expect("the peeked option");
        if argument == "--" {
            break;
        }
        options.push(argument);
    }
    options
        .into_iter()
        .filter(|argument| !argument.starts_with("--"))
        .any(|argument| argument.contains('c'))
        || rest.next().is_some()
}

/// Whether a program name — as [`parse_window_name`] reduces it, so already
/// stripped of its path and of the leading `-` a login shell carries — is an
/// interactive shell.
///
/// Used only to decide whether a pane holding its own terminal is at a prompt.
/// An unlisted shell reads as a foreground program instead, which is the same
/// answer the pane would give while running anything else.
fn is_shell(command: &str) -> bool {
    const SHELLS: [&str; 13] = [
        "sh", "bash", "zsh", "fish", "dash", "ash", "ksh", "mksh", "csh", "tcsh", "elvish", "nu",
        "xonsh",
    ];
    SHELLS.contains(&command)
}

/// What a pane is doing, as the compact `#{pane_state_emoji}` label reports it.
///
/// This is the non-agent half of that variable: a pane running a recognized
/// agent reports the agent's lifecycle state instead. Every other pane lands in
/// exactly one of these, which is what keeps the variable from ever being
/// empty.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PaneClass {
    /// No live child: reaped with `remain-on-exit` holding the pane open, or
    /// never started.
    Dead,
    /// The pane is on its alternate screen, which is what a full-screen
    /// application switches to and an ordinary command line never does.
    Tui,
    /// The pane's shell is at its prompt, waiting for you.
    ShellPrompt,
    /// A foreground command has stopped to read from the terminal, waiting for
    /// you.
    WaitingForTty,
    /// A foreground command is working.
    Running,
}

impl PaneClass {
    /// Classify a pane from what it is running.
    ///
    /// Order matters. A dead pane is dead whatever its last screen was. The
    /// alternate screen then outranks the shell/command split, because a pane
    /// launched straight into a full-screen application (`new-window htop`)
    /// has that application as *both* its session leader and its foreground
    /// group, and would otherwise read as a shell sitting at a prompt.
    pub fn classify(probe: Option<&PaneProcessProbe>, alternate_on: bool, dead: bool) -> Self {
        let Some(probe) = probe.filter(|_| !dead) else {
            return PaneClass::Dead;
        };
        if alternate_on {
            PaneClass::Tui
        } else if probe.shell_at_prompt() {
            PaneClass::ShellPrompt
        } else if probe.waiting_for_tty() {
            PaneClass::WaitingForTty
        } else {
            PaneClass::Running
        }
    }

    /// The status-bar glyph for this class.
    ///
    /// Every glyph here — and every one [`AgentState::emoji`] returns — is a
    /// single codepoint two columns wide. That is a hard requirement, not a
    /// preference: the status renderer measures per codepoint, so a glyph
    /// needing U+FE0F to reach emoji presentation would be counted as one
    /// column while the terminal drew two, and the whole status line would
    /// drift. Check any replacement against `codepoint_width` first.
    ///
    /// [`AgentState::emoji`]: crate::integration::AgentState::emoji
    pub fn emoji(self) -> &'static str {
        match self {
            PaneClass::Dead => "🛑",
            PaneClass::Tui => "🪟",
            PaneClass::ShellPrompt => "💲",
            PaneClass::WaitingForTty => "⌛",
            PaneClass::Running => "🔧",
        }
    }
}

/// Render a pane's argument vector the way tmux's `cmd_stringify_argv` does,
/// for the callers that reduce the result with [`parse_window_name`].
///
/// tmux escapes each argument; that step is skipped because it cannot change
/// what the caller sees. `parse_window_name` cuts at the first space, and it
/// does so after resolving quotes, so both a quoted and an unquoted argument
/// reduce to the same leading word either way.
pub fn stringify_argv<S: AsRef<str>>(argv: &[S]) -> String {
    argv.iter().map(AsRef::as_ref).collect::<Vec<_>>().join(" ")
}

/// Reduce a command line to the program name tmux displays for it, following
/// tmux's `parse_window_name`: take the first quoted or whitespace-delimited
/// word, drop an `exec` prefix and any leading dashes (a login shell's `-bash`
/// is reported as `bash`), and keep only the last component of an absolute
/// path.
///
/// tmux additionally runs the result through `clean_name`, which escapes
/// non-printable bytes for display. That step is not reproduced: every name
/// reaching this function is a program name, and the trailing-byte trim below
/// already removes the control characters `clean_name` would have escaped.
pub fn parse_window_name(input: &str) -> String {
    let mut name = input.strip_prefix('"').unwrap_or(input);
    if let Some(quote) = name.find('"') {
        name = &name[..quote];
    }
    name = name.strip_prefix("exec ").unwrap_or(name);
    name = name.trim_start_matches([' ', '-']);
    if let Some(space) = name.find(' ') {
        name = &name[..space];
    }
    // tmux keeps trailing bytes only while they are alphanumeric or
    // punctuation, which together are exactly the printable ASCII characters.
    let trimmed = name.trim_end_matches(|ch: char| !ch.is_ascii_graphic());
    if trimmed.starts_with('/') {
        return Path::new(trimmed).file_name().map_or_else(
            || trimmed.to_string(),
            |base| base.to_string_lossy().into_owned(),
        );
    }
    trimmed.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A pane with no live child has no foreground group to compare, so the
    /// classes that describe a running process cannot apply to it.
    #[test]
    fn a_pane_with_no_process_is_dead_whatever_its_screen_says() {
        assert_eq!(PaneClass::classify(None, false, false), PaneClass::Dead);
        assert_eq!(PaneClass::classify(None, true, false), PaneClass::Dead);
    }

    #[test]
    fn a_probed_pane_is_classified_by_what_holds_its_terminal() {
        // The pids are deliberately ones `/proc` cannot answer for, so the
        // program name comes from the spawn command the way it does for a
        // leaderless group — and the tty probe reads as "cannot tell", which
        // is also what every non-Linux platform reports.
        let probe = |foreground, session_leader, command: &str| PaneProcessProbe {
            foreground,
            session_leader,
            fallback_command: Some(command.to_string()),
        };

        // The shell still owns the terminal: nothing runs in front of it.
        assert_eq!(
            PaneClass::classify(Some(&probe(Some(0), Some(0), "bash")), false, false),
            PaneClass::ShellPrompt
        );
        // A shell started from the pane's own shell holds the terminal in its
        // place, and is a shell at a prompt just the same.
        assert_eq!(
            PaneClass::classify(Some(&probe(Some(0), Some(1), "zsh")), false, false),
            PaneClass::ShellPrompt
        );
        // A command took the terminal away from the shell.
        assert_eq!(
            PaneClass::classify(Some(&probe(Some(0), Some(1), "make")), false, false),
            PaneClass::Running
        );
        // A pane launched straight into a program is its own session leader,
        // so the group comparison alone would call this a prompt.
        assert_eq!(
            PaneClass::classify(Some(&probe(Some(0), Some(0), "tail -f log")), false, false),
            PaneClass::Running
        );
        // The alternate screen outranks the rest: a full-screen program is
        // likewise both leader and foreground group.
        assert_eq!(
            PaneClass::classify(Some(&probe(Some(0), Some(0), "bash")), true, false),
            PaneClass::Tui
        );
        // Death outranks everything, including the screen it died on.
        assert_eq!(
            PaneClass::classify(Some(&probe(Some(0), Some(0), "bash")), true, true),
            PaneClass::Dead
        );
    }

    /// A shell handed work — a command string or a script — is working, not
    /// prompting, and the option scan has to stop before that work, which can
    /// hold anything.
    #[test]
    fn a_shell_given_work_is_not_at_a_prompt() {
        let invoked = |arguments: &[&str]| invoked_with_command_string(arguments.iter().copied());

        assert!(invoked(&["sh", "-c", "while :; do :; done"]));
        assert!(invoked(&["bash", "-lc", "make"]));
        assert!(!invoked(&["bash", "--norc", "-i"]));
        assert!(!invoked(&["-bash"]));
        assert!(!invoked(&["zsh"]));
        // A shell given a script to run is running it, whether the operand
        // stands on its own or behind the `--` that ends the options.
        assert!(invoked(&["sh", "-i", "script.sh"]));
        assert!(invoked(&["sh", "--", "script.sh"]));
        // Only single-dash arguments bundle short options, so a long option
        // that merely contains a `c` is not one.
        assert!(!invoked(&["bash", "--noprofile", "--norc"]));
        assert!(invoked(&["bash", "--norc", "-c", "make"]));
    }

    #[test]
    fn a_login_shells_argv0_still_reads_as_a_shell() {
        // A login shell is spelled `-bash`, and an absolute path is common in
        // `default-shell`; both reduce to the bare program name.
        assert!(is_shell(&parse_window_name("-bash")));
        assert!(is_shell(&parse_window_name("/usr/bin/zsh")));
        assert!(is_shell(&parse_window_name("fish")));
        assert!(!is_shell(&parse_window_name("/usr/bin/tail -f log")));
        assert!(!is_shell(&parse_window_name("htop")));
    }

    #[test]
    fn parse_window_name_reduces_a_command_line_to_its_program() {
        // The plain cases: a bare name survives, an absolute path loses its
        // directories, and arguments are dropped.
        assert_eq!(parse_window_name("bash"), "bash");
        assert_eq!(parse_window_name("/usr/bin/sleep"), "sleep");
        assert_eq!(parse_window_name("sleep 30"), "sleep");
        // A relative path keeps its directories; tmux only takes the basename
        // of a name that starts at the root.
        assert_eq!(parse_window_name("bin/sleep"), "bin/sleep");
        // A login shell announces itself with a leading dash.
        assert_eq!(parse_window_name("-zsh"), "zsh");
        assert_eq!(parse_window_name("exec vim file"), "vim");
        // The stringified argv of the leaderless-pipeline fixture.
        assert_eq!(parse_window_name(r#"bash -mc "echo x | sleep 30""#), "bash");
        // Quotes are resolved before the cut at the first space, so a quoted
        // argv[0] containing one is still cut there.
        assert_eq!(parse_window_name(r#""my program" -x"#), "my");
        assert_eq!(parse_window_name("sleep\r\n"), "sleep");
        assert_eq!(parse_window_name(""), "");
    }

    #[test]
    fn stringify_argv_reduces_to_the_program_the_pane_was_given() {
        let argv = ["bash", "-mc", "echo x | sleep 30"].map(String::from);
        assert_eq!(parse_window_name(&stringify_argv(&argv)), "bash");
        // A pane spawned with no command carries just the shell.
        let shell = ["/bin/zsh"].map(String::from);
        assert_eq!(parse_window_name(&stringify_argv(&shell)), "zsh");
    }
}
