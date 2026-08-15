//! tmux command-name resolution: exact names, aliases, and unambiguous prefix
//! matching, with tmux's exact ambiguity diagnostics.
//!
//! Real tmux lets a client abbreviate any command to an unambiguous prefix
//! (`list-sess` → `list-sessions`) or use its short alias (`ls`), and reports
//! `ambiguous command: <x>, could be: <names>` when a prefix matches several.
//! The native engine reproduces this against the *full* tmux command table (not
//! just the commands it implements), so the resolver's diagnostics match real
//! tmux even for commands the interpreter doesn't yet handle. This mirrors
//! `cmd.c:cmd_find` in tmux:
//!
//! - an exact alias match wins immediately (no ambiguity);
//! - otherwise every command whose *name* has the typed prefix is a candidate;
//! - an exact name match wins;
//! - two or more prefix candidates → ambiguous, listing all candidate names in
//!   table order (which is alphabetical);
//! - no candidate → unknown command.

use super::command::args::ParsedArgs;
use super::command::{self, Command};

/// How one command's lexed arguments become the command itself.
///
/// The parse phase has already rejected an unknown flag, a flag missing its
/// value, and an out-of-range operand count against this command's getopt and
/// arity entries, so a hook only shapes input that is known to be valid. An
/// `Err` is a parse error like any other: it aborts the whole command line
/// before anything runs.
pub(in crate::server) type ParseFn = fn(&ParsedArgs) -> Result<Command, String>;

/// Static identity, CLI name, and argument parser for one tmux command.
///
/// Rows are compared by identity of the catalog entry, never structurally: a
/// parse hook is a function pointer, whose address says nothing useful.
#[derive(Clone, Copy, Debug)]
pub(in crate::server) struct CommandSpec {
    pub(in crate::server) name: &'static str,
    pub(in crate::server) alias: Option<&'static str>,
    pub(in crate::server) parse: ParseFn,
}

impl CommandSpec {
    const fn new(name: &'static str, alias: Option<&'static str>, parse: ParseFn) -> Self {
        Self { name, alias, parse }
    }
}

/// `Some(alias)` for a row that names one, `None` for a row that doesn't.
macro_rules! spec_alias {
    () => {
        None
    };
    ($alias:literal) => {
        Some($alias)
    };
}

/// Build the command catalog: `"name" ("alias") => <parse expression>`, with the
/// alias omitted for a command that has none.
///
/// Each row gets its own parse hook. A command whose arguments are not typed yet
/// names its bare [`Command`] variant, which the generated hook returns while
/// ignoring the lexed arguments.
macro_rules! command_specs {
    ($($name:literal $(($alias:literal))? => $variant:expr),* $(,)?) => {
        &[$({
            fn parse(_args: &ParsedArgs) -> Result<Command, String> {
                Ok($variant)
            }
            CommandSpec::new($name, spec_alias!($($alias)?), parse)
        }),*]
    };
}

use command::buffers::Command as Buffer;
use command::clients::Command as Client;
use command::configuration::Command as Configuration;
use command::execution::Command as Execution;
use command::keys::Command as Keys;
use command::panes::Command as Pane;
use command::server::Command as Server;
use command::sessions::Command as Session;
use command::windows::Command as Window;

/// The tmux command catalog, kept in alphabetical order for ambiguity messages.
pub(in crate::server) static COMMAND_SPECS: &[CommandSpec] = command_specs![
    "attach-session" ("attach") => Command::Session(Session::Attach),
    "bind-key" ("bind") => Command::Keys(Keys::Bind),
    "break-pane" ("breakp") => Command::Pane(Pane::Break),
    "capture-pane" ("capturep") => Command::Pane(Pane::Capture),
    "choose-buffer" => Command::Client(Client::ChooseBuffer),
    "choose-client" => Command::Client(Client::ChooseClient),
    "choose-tree" => Command::Client(Client::ChooseTree),
    "clear-history" ("clearhist") => Command::Pane(Pane::ClearHistory),
    "clear-prompt-history" ("clearphist") => Command::Client(Client::ClearPromptHistory),
    "clock-mode" => Command::Client(Client::ClockMode),
    "command-prompt" => Command::Client(Client::Prompt),
    "confirm-before" ("confirm") => Command::Client(Client::ConfirmBefore),
    "copy-mode" => Command::Pane(Pane::CopyMode),
    "customize-mode" => Command::Client(Client::CustomizeMode),
    "delete-buffer" ("deleteb") => Command::Buffer(Buffer::Delete),
    "detach-client" ("detach") => Command::Client(Client::Detach),
    "display-menu" ("menu") => Command::Client(Client::DisplayMenu),
    "display-message" ("display") => Command::Client(Client::DisplayMessage),
    "display-popup" ("popup") => Command::Client(Client::DisplayPopup),
    "display-panes" ("displayp") => Command::Client(Client::DisplayPanes),
    "find-window" ("findw") => Command::Window(Window::Find),
    "has-session" ("has") => Command::Session(Session::Has),
    "if-shell" ("if") => Command::Execution(Execution::IfShell),
    "join-pane" ("joinp") => Command::Pane(Pane::Join),
    "kill-pane" ("killp") => Command::Pane(Pane::Kill),
    "kill-server" => Command::Server(Server::Kill),
    "kill-session" => Command::Session(Session::Kill),
    "kill-window" ("killw") => Command::Window(Window::Kill),
    "last-pane" ("lastp") => Command::Pane(Pane::Last),
    "last-window" ("last") => Command::Window(Window::Last),
    "link-window" ("linkw") => Command::Window(Window::Link),
    "list-buffers" ("lsb") => Command::Buffer(Buffer::List),
    "list-clients" ("lsc") => Command::Client(Client::List),
    "list-commands" ("lscm") => Command::Server(Server::ListCommands),
    "list-keys" ("lsk") => Command::Keys(Keys::List),
    "list-panes" ("lsp") => Command::Pane(Pane::List),
    "list-sessions" ("ls") => Command::Session(Session::List),
    "list-windows" ("lsw") => Command::Window(Window::List),
    "load-buffer" ("loadb") => Command::Buffer(Buffer::Load),
    "lock-client" ("lockc") => Command::Client(Client::Lock),
    "lock-server" ("lock") => Command::Server(Server::Lock),
    "lock-session" ("locks") => Command::Server(Server::LockSession),
    "move-pane" ("movep") => Command::Pane(Pane::Move),
    "move-window" ("movew") => Command::Window(Window::Move),
    "new-pane" ("newp") => Command::Pane(Pane::New),
    "new-session" ("new") => Command::Session(Session::New),
    "new-window" ("neww") => Command::Window(Window::New),
    "next-layout" ("nextl") => Command::Pane(Pane::NextLayout),
    "next-window" ("next") => Command::Window(Window::Next),
    "paste-buffer" ("pasteb") => Command::Buffer(Buffer::Paste),
    "pipe-pane" ("pipep") => Command::Pane(Pane::Pipe),
    "previous-layout" ("prevl") => Command::Pane(Pane::PreviousLayout),
    "previous-window" ("prev") => Command::Window(Window::Previous),
    "refresh-client" ("refresh") => Command::Client(Client::Refresh),
    "rename-session" ("rename") => Command::Session(Session::Rename),
    "rename-window" ("renamew") => Command::Window(Window::Rename),
    "resize-pane" ("resizep") => Command::Pane(Pane::Resize),
    "resize-window" ("resizew") => Command::Pane(Pane::ResizeWindow),
    "respawn-pane" ("respawnp") => Command::Pane(Pane::Respawn),
    "respawn-window" ("respawnw") => Command::Window(Window::Respawn),
    "rotate-window" ("rotatew") => Command::Pane(Pane::RotateWindow),
    "run-shell" ("run") => Command::Execution(Execution::RunShell),
    "save-buffer" ("saveb") => Command::Buffer(Buffer::Save),
    "select-layout" ("selectl") => Command::Pane(Pane::SelectLayout),
    "select-pane" ("selectp") => Command::Pane(Pane::Select),
    "select-window" ("selectw") => Command::Window(Window::Select),
    "send-keys" ("send") => Command::Keys(Keys::Send),
    "send-prefix" => Command::Keys(Keys::SendPrefix),
    "server-access" => Command::Server(Server::Access),
    "set-buffer" ("setb") => Command::Buffer(Buffer::Set),
    "set-environment" ("setenv") => Command::Configuration(Configuration::SetEnvironment),
    "set-hook" => Command::Configuration(Configuration::SetHook),
    "set-option" ("set") => Command::Configuration(Configuration::SetOption),
    "set-window-option" ("setw") => Command::Configuration(Configuration::SetWindowOption),
    "show-buffer" ("showb") => Command::Buffer(Buffer::Show),
    "show-environment" ("showenv") => Command::Configuration(Configuration::ShowEnvironment),
    "show-hooks" => Command::Configuration(Configuration::ShowHooks),
    "show-messages" ("showmsgs") => Command::Server(Server::ShowMessages),
    "show-options" ("show") => Command::Configuration(Configuration::ShowOptions),
    "show-prompt-history" ("showphist") => Command::Client(Client::ShowPromptHistory),
    "show-window-options" ("showw") => Command::Configuration(Configuration::ShowWindowOptions),
    "source-file" ("source") => Command::Execution(Execution::SourceFile),
    "split-window" ("splitw") => Command::Pane(Pane::Split),
    "start-server" ("start") => Command::Server(Server::Start),
    "suspend-client" ("suspendc") => Command::Client(Client::Suspend),
    "swap-pane" ("swapp") => Command::Pane(Pane::Swap),
    "swap-window" ("swapw") => Command::Window(Window::Swap),
    "switch-client" ("switchc") => Command::Client(Client::Switch),
    "unbind-key" ("unbind") => Command::Keys(Keys::Unbind),
    "unlink-window" ("unlinkw") => Command::Window(Window::Unlink),
    "wait-for" ("wait") => Command::Execution(Execution::WaitFor),
];

/// The outcome of resolving a typed command word.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolution {
    /// Resolved to this canonical command name.
    Name(&'static str),
    /// A prefix matched several commands; `error` is tmux's diagnostic line.
    Ambiguous { error: String },
    /// No command matched; `error` is tmux's diagnostic line.
    Unknown { error: String },
}

#[derive(Debug, Clone)]
pub(in crate::server) enum SpecResolution {
    Spec(&'static CommandSpec),
    Ambiguous { error: String },
    Unknown { error: String },
}

/// Resolve a typed command word to a canonical name, matching tmux's `cmd_find`.
pub fn resolve(word: &str) -> Resolution {
    match resolve_spec(word) {
        SpecResolution::Spec(spec) => Resolution::Name(spec.name),
        SpecResolution::Ambiguous { error } => Resolution::Ambiguous { error },
        SpecResolution::Unknown { error } => Resolution::Unknown { error },
    }
}

/// Resolve directly to the catalog row used by parsing and execution.
pub(in crate::server) fn resolve_spec(word: &str) -> SpecResolution {
    let mut found: Option<&'static CommandSpec> = None;
    let mut ambiguous = false;

    for spec in COMMAND_SPECS {
        // An exact alias match wins outright, cancelling any pending ambiguity.
        if spec.alias == Some(word) {
            return SpecResolution::Spec(spec);
        }
        if !spec.name.starts_with(word) {
            continue;
        }
        if found.is_some() {
            ambiguous = true;
        }
        found = Some(spec);
        if spec.name == word {
            // Exact name match: tmux breaks here, so any earlier prefix
            // candidates don't make this ambiguous.
            return SpecResolution::Spec(spec);
        }
    }

    if ambiguous {
        let candidates: Vec<&str> = COMMAND_SPECS
            .iter()
            .filter(|spec| spec.name.starts_with(word))
            .map(|spec| spec.name)
            .collect();
        return SpecResolution::Ambiguous {
            error: format!(
                "ambiguous command: {word}, could be: {}\n",
                candidates.join(", ")
            ),
        };
    }

    match found {
        Some(spec) => SpecResolution::Spec(spec),
        None => SpecResolution::Unknown {
            error: format!("unknown command: {word}\n"),
        },
    }
}

/// The getopt spec for a canonical command name: every valid flag letter, with a
/// `:` after each letter that takes a value. Transcribed verbatim from tmux's own
/// `list-commands` usage strings (tmux 3.x), so a flag the native engine accepts
/// is exactly a flag real tmux accepts. `None` means the command's flag set isn't
/// modeled, so flag validation is skipped for it (permissive).
///
/// This is what lets the native engine reproduce tmux's parse-time
/// `command <name>: unknown flag -X` diagnostic. Like the command table, it is
/// version-specific; the differential suite pins it against the running tmux.
pub fn getopt(name: &str) -> Option<&'static str> {
    Some(match name {
        "attach-session" => "dErxc:f:t:",
        "bind-key" => "nrT:N:",
        "break-pane" => "abdPF:n:s:t:",
        "capture-pane" => "ab:CeE:FHJLMNpPqS:Tt:",
        "choose-buffer" => "NrZF:f:K:O:t:",
        "choose-client" => "NrZF:f:K:O:t:",
        "choose-tree" => "GNrswZF:f:K:O:t:",
        "clear-history" => "Ht:",
        "clear-prompt-history" => "T:",
        "clock-mode" => "t:",
        "command-prompt" => "1CbeFiklI:Np:t:T:",
        "confirm-before" => "byc:p:t:",
        "copy-mode" => "deHMqSus:t:",
        "customize-mode" => "NZF:f:t:",
        "delete-buffer" => "b:",
        "detach-client" => "aPE:s:t:",
        "display-menu" => "MOb:c:C:H:s:S:t:T:x:y:",
        "display-message" => "aCIlNpvc:d:F:t:",
        "display-popup" => "BCEkNb:c:d:e:h:s:S:t:T:w:x:y:",
        "display-panes" => "bNd:t:",
        "find-window" => "CiNrTZt:",
        "has-session" => "t:",
        "if-shell" => "bFt:",
        "join-pane" => "bdfhvl:s:t:",
        "kill-pane" => "at:",
        "kill-server" => "",
        "kill-session" => "aCgt:",
        "kill-window" => "at:",
        "last-pane" => "deZt:",
        "last-window" => "t:",
        "link-window" => "abdks:t:",
        "list-buffers" => "F:f:O:r",
        "list-clients" => "F:f:O:rt:",
        "list-commands" => "F:",
        "list-keys" => "1aF:NO:P:rT:",
        "list-panes" => "aF:f:O:rst:",
        "list-sessions" => "F:f:O:r",
        "list-windows" => "aF:f:O:rt:",
        "load-buffer" => "b:t:w",
        "lock-client" => "t:",
        "lock-server" => "",
        "lock-session" => "t:",
        "move-pane" => "bdfhvl:s:t:",
        "move-window" => "abdkrs:t:",
        "new-pane" => "bc:de:EfF:hIkl:Lm:p:PR:s:S:t:vx:X:y:Y:Z",
        "new-session" => "AdDEPXc:e:F:f:n:s:t:x:y:",
        "new-window" => "abdkPSc:e:F:n:t:",
        "next-layout" => "t:",
        "next-window" => "at:",
        "paste-buffer" => "db:prSs:t:",
        "pipe-pane" => "IOot:",
        "previous-layout" => "t:",
        "previous-window" => "at:",
        "refresh-client" => "cDlLRSUA:B:C:f:F:r:t:",
        "rename-session" => "t:",
        "rename-window" => "t:",
        "resize-pane" => "DLMRTUZx:y:t:",
        "resize-window" => "aADLRUx:y:t:",
        "respawn-pane" => "kc:e:t:",
        "respawn-window" => "kc:e:t:",
        "rotate-window" => "DUZt:",
        "run-shell" => "bd:Ct:Es:c:",
        "save-buffer" => "ab:",
        "select-layout" => "Enopt:",
        "select-pane" => "DdeLlMmRUZT:t:",
        "select-window" => "lnpTt:",
        "send-keys" => "FHKlMRXc:N:t:",
        "send-prefix" => "2t:",
        "server-access" => "adlrwt:",
        "set-buffer" => "awb:n:t:",
        "set-environment" => "Fhgrut:",
        "set-hook" => "agpRuwt:",
        "set-option" => "aFgopqsuUwt:",
        "set-window-option" => "aFgoqut:",
        "show-buffer" => "b:",
        "show-environment" => "hgst:",
        "show-hooks" => "gpwt:",
        "show-messages" => "JTt:",
        "show-options" => "AgHpqsvwt:",
        "show-prompt-history" => "T:",
        "show-window-options" => "gvt:",
        "source-file" => "Fnqvt:",
        "split-window" => "bc:de:EfF:hIkl:m:p:PR:s:S:t:vZ",
        "start-server" => "",
        "suspend-client" => "t:",
        "swap-pane" => "dDUZs:t:",
        "swap-window" => "ds:t:",
        "switch-client" => "c:EFlnO:pt:rT:Z",
        "unbind-key" => "anqT:",
        "unlink-window" => "kt:",
        "wait-for" => "LSU",
        _ => return None,
    })
}

/// Minimum and maximum positional argument counts from tmux 3.7b's command
/// entries. `None` as the maximum means unbounded.
///
/// Keeping arity beside the getopt metadata lets parse-only config loading use
/// the same parser as ordinary command execution without invoking handlers for
/// their side effects.
pub fn argument_limits(name: &str) -> Option<(usize, Option<usize>)> {
    Some(match name {
        "display-popup" | "new-session" | "new-window" | "respawn-pane" | "respawn-window"
        | "run-shell" | "send-keys" | "new-pane" | "split-window" => (0, None),

        "attach-session"
        | "break-pane"
        | "capture-pane"
        | "clear-history"
        | "customize-mode"
        | "copy-mode"
        | "clock-mode"
        | "detach-client"
        | "suspend-client"
        | "join-pane"
        | "move-pane"
        | "kill-pane"
        | "kill-server"
        | "start-server"
        | "kill-session"
        | "kill-window"
        | "unlink-window"
        | "list-buffers"
        | "list-clients"
        | "list-panes"
        | "list-sessions"
        | "list-windows"
        | "lock-server"
        | "lock-session"
        | "lock-client"
        | "move-window"
        | "link-window"
        | "has-session"
        | "paste-buffer"
        | "rotate-window"
        | "show-buffer"
        | "next-layout"
        | "previous-layout"
        | "select-pane"
        | "last-pane"
        | "select-window"
        | "next-window"
        | "previous-window"
        | "last-window"
        | "send-prefix"
        | "delete-buffer"
        | "show-messages"
        | "show-prompt-history"
        | "clear-prompt-history"
        | "swap-pane"
        | "swap-window"
        | "switch-client" => (0, Some(0)),

        "choose-tree"
        | "choose-client"
        | "choose-buffer"
        | "command-prompt"
        | "display-message"
        | "display-panes"
        | "list-commands"
        | "list-keys"
        | "pipe-pane"
        | "refresh-client"
        | "resize-pane"
        | "resize-window"
        | "select-layout"
        | "server-access"
        | "set-buffer"
        | "show-environment"
        | "show-options"
        | "show-window-options"
        | "show-hooks"
        | "unbind-key" => (0, Some(1)),

        "bind-key" | "display-menu" | "source-file" => (1, None),

        "confirm-before" | "find-window" | "load-buffer" | "rename-session" | "rename-window"
        | "save-buffer" | "wait-for" => (1, Some(1)),

        "set-environment" | "set-option" | "set-window-option" | "set-hook" => (1, Some(2)),

        "if-shell" => (2, Some(3)),
        _ => return None,
    })
}

/// The usage string for a canonical command name: the `[-flags] args` portion
/// that `list-commands` prints after the command's `name (alias)` prefix.
/// Transcribed verbatim from tmux's own `list-commands` output (tmux 3.7b), so
/// the listing matches real tmux line for line. Empty for argument-less commands
/// (`kill-server`, `lock-server`, `start-server`), which tmux still prints with a
/// trailing space after the name. `None` for a name not in the command table.
///
/// Like the getopt catalog this is version-specific; the differential suite pins
/// it against the running tmux.
pub fn usage(name: &str) -> Option<&'static str> {
    Some(match name {
        "attach-session" => "[-dErx] [-c working-directory] [-f flags] [-t target-session]",
        "bind-key" => "[-nr] [-T key-table] [-N note] key [command [argument ...]]",
        "break-pane" => "[-abdP] [-F format] [-n window-name] [-s src-pane] [-t dst-window]",
        "capture-pane" => "[-aCeFHJLMNpPqT] [-b buffer-name] [-E end-line] [-S start-line] [-t target-pane]",
        "choose-buffer" => "[-NrZ] [-F format] [-f filter] [-K key-format] [-O sort-order] [-t target-pane] [template]",
        "choose-client" => "[-NrZ] [-F format] [-f filter] [-K key-format] [-O sort-order] [-t target-pane] [template]",
        "choose-tree" => "[-GNrswZ] [-F format] [-f filter] [-K key-format] [-O sort-order] [-t target-pane] [template]",
        "clear-history" => "[-H] [-t target-pane]",
        "clear-prompt-history" => "[-T prompt-type]",
        "clock-mode" => "[-t target-pane]",
        "command-prompt" => "[-1CbeFiklN] [-I inputs] [-p prompts] [-t target-client] [-T prompt-type] [template]",
        "confirm-before" => "[-by] [-c confirm-key] [-p prompt] [-t target-client] command",
        "copy-mode" => "[-deHMqSu] [-s src-pane] [-t target-pane]",
        "customize-mode" => "[-NZ] [-F format] [-f filter] [-t target-pane]",
        "delete-buffer" => "[-b buffer-name]",
        "detach-client" => "[-aP] [-E shell-command] [-s target-session] [-t target-client]",
        "display-menu" => "[-MO] [-b border-lines] [-c target-client] [-C starting-choice] [-H selected-style] [-s style] [-S border-style] [-t target-pane] [-T title] [-x position] [-y position] name [key] [command] ...",
        "display-message" => "[-aCIlNpv] [-c target-client] [-d delay] [-F format] [-t target-pane] [message]",
        "display-popup" => "[-BCEkN] [-b border-lines] [-c target-client] [-d start-directory] [-e environment] [-h height] [-s style] [-S border-style] [-t target-pane] [-T title] [-w width] [-x position] [-y position] [shell-command [argument ...]]",
        "display-panes" => "[-bN] [-d duration] [-t target-client] [template]",
        "find-window" => "[-CiNrTZ] [-t target-pane] match-string",
        "has-session" => "[-t target-session]",
        "if-shell" => "[-bF] [-t target-pane] shell-command command [command]",
        "join-pane" => "[-bdfhv] [-l size] [-s src-pane] [-t dst-pane]",
        "kill-pane" => "[-a] [-t target-pane]",
        "kill-server" => "",
        "kill-session" => "[-aCg] [-t target-session]",
        "kill-window" => "[-a] [-t target-window]",
        "last-pane" => "[-deZ] [-t target-window]",
        "last-window" => "[-t target-session]",
        "link-window" => "[-abdk] [-s src-window] [-t dst-window]",
        "list-buffers" => "[-F format] [-f filter] [-O order]",
        "list-clients" => "[-F format] [-f filter] [-O order][-t target-session]",
        "list-commands" => "[-F format] [command]",
        "list-keys" => "[-1aNr] [-F format] [-O order] [-P prefix-string][-T key-table] [key]",
        "list-panes" => "[-asr] [-F format] [-f filter] [-O order][-t target-window]",
        "list-sessions" => "[-r] [-F format] [-f filter] [-O order]",
        "list-windows" => "[-ar] [-F format] [-f filter] [-O order][-t target-session]",
        "load-buffer" => "[-b buffer-name] [-t target-client] path",
        "lock-client" => "[-t target-client]",
        "lock-server" => "",
        "lock-session" => "[-t target-session]",
        "move-pane" => "[-bdfhv] [-l size] [-s src-pane] [-t dst-pane]",
        "move-window" => "[-abdkr] [-s src-window] [-t dst-window]",
        "new-pane" => "[-bdefhIklPvZ] [-c start-directory] [-e environment] [-F format] [-l size] [-m message] [-p percentage] [-s style] [-S active-border-style] [-R inactive-border-style] [-x width] [-y height] [-X x-position] [-Y y-position] [-t target-pane] [shell-command [argument ...]]",
        "new-session" => "[-AdDEPX] [-c start-directory] [-e environment] [-F format] [-f flags] [-n window-name] [-s session-name] [-t target-session] [-x width] [-y height] [shell-command [argument ...]]",
        "new-window" => "[-abdkPS] [-c start-directory] [-e environment] [-F format] [-n window-name] [-t target-window] [shell-command [argument ...]]",
        "next-layout" => "[-t target-window]",
        "next-window" => "[-a] [-t target-session]",
        "paste-buffer" => "[-dprS] [-s separator] [-b buffer-name] [-t target-pane]",
        "pipe-pane" => "[-IOo] [-t target-pane] [shell-command]",
        "previous-layout" => "[-t target-window]",
        "previous-window" => "[-a] [-t target-session]",
        "refresh-client" => "[-cDlLRSU] [-A pane:state] [-B name:what:format] [-C XxY] [-f flags] [-r pane:report] [-t target-client] [adjustment]",
        "rename-session" => "[-t target-session] new-name",
        "rename-window" => "[-t target-window] new-name",
        "resize-pane" => "[-DLMRTUZ] [-x width] [-y height] [-t target-pane] [adjustment]",
        "resize-window" => "[-aADLRU] [-x width] [-y height] [-t target-window] [adjustment]",
        "respawn-pane" => "[-k] [-c start-directory] [-e environment] [-t target-pane] [shell-command [argument ...]]",
        "respawn-window" => "[-k] [-c start-directory] [-e environment] [-t target-window] [shell-command [argument ...]]",
        "rotate-window" => "[-DUZ] [-t target-window]",
        "run-shell" => "[-bCE] [-c start-directory] [-d delay] [-t target-pane] [shell-command [argument ...]]",
        "save-buffer" => "[-a] [-b buffer-name] path",
        "select-layout" => "[-Enop] [-t target-pane] [layout-name]",
        "select-pane" => "[-DdeLlMmRUZ] [-T title] [-t target-pane]",
        "select-window" => "[-lnpT] [-t target-window]",
        "send-keys" => "[-FHKlMRX] [-c target-client] [-N repeat-count] [-t target-pane] [key ...]",
        "send-prefix" => "[-2] [-t target-pane]",
        "server-access" => "[-adlrw] [-t target-pane] [user]",
        "set-buffer" => "[-aw] [-b buffer-name] [-n new-buffer-name] [-t target-client] [data]",
        "set-environment" => "[-Fhgru] [-t target-session] variable [value]",
        "set-hook" => "[-agpRuw] [-t target-pane] hook [command]",
        "set-option" => "[-aFgopqsuUw] [-t target-pane] option [value]",
        "set-window-option" => "[-aFgoqu] [-t target-window] option [value]",
        "show-buffer" => "[-b buffer-name]",
        "show-environment" => "[-hgs] [-t target-session] [variable]",
        "show-hooks" => "[-gpw] [-t target-pane] [hook]",
        "show-messages" => "[-JT] [-t target-client]",
        "show-options" => "[-AgHpqsvw] [-t target-pane] [option]",
        "show-prompt-history" => "[-T prompt-type]",
        "show-window-options" => "[-gv] [-t target-window] [option]",
        "source-file" => "[-Fnqv] [-t target-pane] path ...",
        "split-window" => "[-bdefhIklPvZ] [-c start-directory] [-e environment] [-F format] [-l size] [-m message] [-p percentage] [-s style] [-S active-border-style] [-R inactive-border-style] [-t target-pane] [shell-command [argument ...]]",
        "start-server" => "",
        "suspend-client" => "[-t target-client]",
        "swap-pane" => "[-dDUZ] [-s src-pane] [-t dst-pane]",
        "swap-window" => "[-d] [-s src-window] [-t dst-window]",
        "switch-client" => "[-ElnprZ] [-c target-client] [-t target-session] [-T key-table] [-O order]",
        "unbind-key" => "[-anq] [-T key-table] key",
        "unlink-window" => "[-k] [-t target-window]",
        "wait-for" => "[-L|-S|-U] channel",
        _ => return None,
    })
}

/// Format one command's `list-commands` line: `name`, optionally ` (alias)`, then
/// a single space and the [`usage`] string. tmux always emits that space, so an
/// argument-less command (empty usage) yields a trailing space (`kill-server `).
/// `None` for a name not in the command table.
pub fn command_line(name: &str) -> Option<String> {
    let spec = COMMAND_SPECS.iter().find(|spec| spec.name == name)?;
    let usage = usage(spec.name)?;
    Some(match spec.alias {
        Some(alias) => format!("{} ({alias}) {usage}", spec.name),
        None => format!("{} {usage}", spec.name),
    })
}

/// Whether flag letter `c` is valid in getopt `spec`, and if so whether it takes
/// a value (a `:` follows it). `None` if `c` isn't a valid flag.
pub fn flag_kind(spec: &str, c: char) -> Option<bool> {
    let b = spec.as_bytes();
    let mut k = 0;
    while k < b.len() {
        if b[k] != b':' && b[k] as char == c {
            return Some(b.get(k + 1) == Some(&b':'));
        }
        k += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn catalog_names_and_command_identities_are_unique_and_complete() {
        let names = COMMAND_SPECS
            .iter()
            .map(|spec| spec.name)
            .collect::<HashSet<_>>();
        assert_eq!(names.len(), COMMAND_SPECS.len());

        // Every row's parse hook still ignores its arguments, so the empty
        // lexed argv is enough to recover the identity each row registers.
        let commands = COMMAND_SPECS
            .iter()
            .map(|spec| (spec.parse)(&ParsedArgs::default()).expect("identity parse"))
            .collect::<HashSet<_>>();
        assert_eq!(commands.len(), COMMAND_SPECS.len());

        let all_commands = command::all_commands();
        assert_eq!(commands.len(), all_commands.len());
        for command in all_commands {
            assert!(
                commands.contains(&command),
                "unregistered command: {command:?}"
            );
        }
    }

    #[test]
    fn exact_name() {
        assert_eq!(resolve("list-sessions"), Resolution::Name("list-sessions"));
    }

    #[test]
    fn exact_alias() {
        assert_eq!(resolve("ls"), Resolution::Name("list-sessions"));
        // `set` is an alias of set-option even though it's also a prefix of
        // several set-* commands: the exact alias wins.
        assert_eq!(resolve("set"), Resolution::Name("set-option"));
    }

    #[test]
    fn unambiguous_prefix() {
        assert_eq!(resolve("list-sess"), Resolution::Name("list-sessions"));
        assert_eq!(resolve("new-w"), Resolution::Name("new-window"));
        // A prefix that names exactly one command resolves even if longer than
        // the alias.
        assert_eq!(resolve("kill-ser"), Resolution::Name("kill-server"));
    }

    #[test]
    fn ambiguous_prefix() {
        match resolve("ne") {
            Resolution::Ambiguous { error } => assert_eq!(
                error,
                "ambiguous command: ne, could be: new-pane, new-session, new-window, next-layout, next-window\n"
            ),
            other => panic!("expected ambiguous, got {other:?}"),
        }
    }

    #[test]
    fn ambiguous_list_prefix() {
        match resolve("l") {
            Resolution::Ambiguous { error } => assert_eq!(
                error,
                "ambiguous command: l, could be: last-pane, last-window, link-window, list-buffers, list-clients, list-commands, list-keys, list-panes, list-sessions, list-windows, load-buffer, lock-client, lock-server, lock-session\n"
            ),
            other => panic!("expected ambiguous, got {other:?}"),
        }
    }

    #[test]
    fn unknown() {
        assert_eq!(
            resolve("frobnicate"),
            Resolution::Unknown {
                error: "unknown command: frobnicate\n".into()
            }
        );
    }

    #[test]
    fn getopt_covers_every_command() {
        // Every canonical command has a modeled flag spec (validation is only
        // skipped for names not in the table, and the table is the command table).
        for spec in COMMAND_SPECS {
            assert!(
                getopt(spec.name).is_some(),
                "missing getopt spec for {}",
                spec.name
            );
        }
    }

    #[test]
    fn usage_covers_every_command() {
        // Every canonical command has a usage string, so `list-commands` can print
        // a line for each entry in the table.
        for spec in COMMAND_SPECS {
            assert!(
                usage(spec.name).is_some(),
                "missing usage string for {}",
                spec.name
            );
        }
        assert_eq!(usage("frobnicate"), None);
    }

    #[test]
    fn argument_limits_cover_every_command() {
        for spec in COMMAND_SPECS {
            assert!(
                argument_limits(spec.name).is_some(),
                "missing argument limits for {}",
                spec.name
            );
        }
        assert_eq!(argument_limits("frobnicate"), None);
    }

    #[test]
    fn command_line_formats_name_alias_and_usage() {
        assert_eq!(
            command_line("has-session").as_deref(),
            Some("has-session (has) [-t target-session]"),
        );
        // No alias, empty usage: name then a single trailing space.
        assert_eq!(command_line("kill-server").as_deref(), Some("kill-server "));
        // Alias, empty usage: `name (alias) ` with a trailing space.
        assert_eq!(
            command_line("lock-server").as_deref(),
            Some("lock-server (lock) ")
        );
        assert_eq!(command_line("frobnicate"), None);
    }

    #[test]
    fn flag_kind_distinguishes_value_and_boolean() {
        let spec = getopt("list-sessions").unwrap(); // "F:f:O:r"
        assert_eq!(flag_kind(spec, 'F'), Some(true)); // takes a value
        assert_eq!(flag_kind(spec, 'f'), Some(true));
        assert_eq!(flag_kind(spec, 'Z'), None); // not a flag
        let spec = getopt("kill-window").unwrap(); // "at:"
        assert_eq!(flag_kind(spec, 'a'), Some(false)); // boolean
        assert_eq!(flag_kind(spec, 't'), Some(true));
    }
}
