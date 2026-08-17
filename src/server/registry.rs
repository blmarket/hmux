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

/// Static identity, CLI name, argument contract, and parser for one tmux
/// command: everything the parse phase needs about a command is this one row.
///
/// Rows are compared by identity of the catalog entry, never structurally: a
/// parse hook is a function pointer, whose address says nothing useful.
#[derive(Clone, Copy, Debug)]
pub(in crate::server) struct CommandSpec {
    pub(in crate::server) name: &'static str,
    pub(in crate::server) alias: Option<&'static str>,
    pub(in crate::server) parse: ParseFn,
    /// Every valid flag letter, with a `:` after each letter that takes a
    /// value. Transcribed verbatim from tmux's own `list-commands` usage
    /// strings (tmux 3.7b), so a flag the native engine accepts is exactly a
    /// flag real tmux accepts.
    ///
    /// This is what lets the native engine reproduce tmux's parse-time
    /// `command <name>: unknown flag -X` diagnostic. Like the command table it
    /// is version-specific; the differential suite pins it against the running
    /// tmux.
    pub(in crate::server) getopt: &'static str,
    /// Minimum and maximum positional argument counts from tmux 3.7b's command
    /// entries. `None` as the maximum means unbounded.
    ///
    /// Keeping arity beside the getopt spec lets parse-only config loading use
    /// the same parser as ordinary command execution without invoking handlers
    /// for their side effects.
    pub(in crate::server) arity: (usize, Option<usize>),
    /// The `[-flags] args` portion that `list-commands` prints after the
    /// command's `name (alias)` prefix. Transcribed verbatim from tmux's own
    /// `list-commands` output (tmux 3.7b), so the listing matches real tmux
    /// line for line. Empty for argument-less commands (`kill-server`,
    /// `lock-server`, `start-server`), which tmux still prints with a trailing
    /// space after the name.
    pub(in crate::server) usage: &'static str,
}

impl CommandSpec {
    const fn new(
        name: &'static str,
        alias: Option<&'static str>,
        parse: ParseFn,
        getopt: &'static str,
        arity: (usize, Option<usize>),
        usage: &'static str,
    ) -> Self {
        Self {
            name,
            alias,
            parse,
            getopt,
            arity,
            usage,
        }
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

/// Build the command catalog. One row is
///
/// ```text
/// "name" ("alias") => <parse hook>,
///     getopt: "<flags>", arity: (<min>, <max>),
///     usage: "<usage>";
/// ```
///
/// with the alias omitted for a command that has none. Rows are separated by
/// `;`, so every field of the per-command contract is written in one place and
/// a new command cannot be half-registered.
///
/// The hook is written inline as a closure over the command's lexed arguments;
/// a non-capturing one is just a function pointer, so the table stays a static.
macro_rules! command_specs {
    ($($name:literal $(($alias:literal))? => $parse:expr,
        getopt: $getopt:literal, arity: $arity:expr,
        usage: $usage:literal);* $(;)?) => {
        &[$( CommandSpec::new(
            $name, spec_alias!($($alias)?), $parse, $getopt, $arity, $usage,
        ) ),*]
    };
}

use command::buffers::{self, Command as Buffer};
use command::clients::{self, Command as Client};
use command::configuration::{self, Command as Configuration};
use command::execution::{self, Command as Execution};
use command::keys::{self, Command as Keys};
use command::panes::{self, Command as Pane};
use command::server::{self, Command as Server};
use command::sessions::{self, Command as Session};
use command::windows::{self, Command as Window};

/// A catalog row for a command that reads none of its arguments: the hook just
/// names the variant.
macro_rules! bare {
    ($category:ident, $variant:ident) => {
        |_| Ok(Command::$category($category::$variant))
    };
}

/// A catalog row for a command with typed arguments: the hook hands the lexed
/// argv to that command's own `parse`.
macro_rules! typed {
    ($category:ident, $variant:ident, $arguments:ty) => {
        |args| {
            Ok(Command::$category($category::$variant(
                <$arguments>::parse(args)?,
            )))
        }
    };
}

/// The tmux command catalog, kept in alphabetical order for ambiguity messages.
pub(in crate::server) static COMMAND_SPECS: &[CommandSpec] = command_specs![
    "attach-session" ("attach") => typed!(Session, Attach, sessions::AttachSession),
        getopt: "dErxc:f:t:", arity: (0, Some(0)),
        usage: "[-dErx] [-c working-directory] [-f flags] [-t target-session]";
    "bind-key" ("bind") => typed!(Keys, Bind, keys::BindKey),
        getopt: "nrT:N:", arity: (1, None),
        usage: "[-nr] [-T key-table] [-N note] key [command [argument ...]]";
    "break-pane" ("breakp") => typed!(Pane, Break, panes::BreakPane),
        getopt: "abdPF:n:s:t:", arity: (0, Some(0)),
        usage: "[-abdP] [-F format] [-n window-name] [-s src-pane] [-t dst-window]";
    "capture-pane" ("capturep") => typed!(Pane, Capture, panes::CapturePane),
        getopt: "ab:CeE:FHJLMNpPqS:Tt:", arity: (0, Some(0)),
        usage: "[-aCeFHJLMNpPqT] [-b buffer-name] [-E end-line] [-S start-line] [-t target-pane]";
    "choose-buffer" => typed!(Client, ChooseBuffer, clients::ChooseBuffer),
        getopt: "NrZF:f:K:O:t:", arity: (0, Some(1)),
        usage: "[-NrZ] [-F format] [-f filter] [-K key-format] [-O sort-order] [-t target-pane] [template]";
    "choose-client" => typed!(Client, ChooseClient, clients::ChooseClient),
        getopt: "NrZF:f:K:O:t:", arity: (0, Some(1)),
        usage: "[-NrZ] [-F format] [-f filter] [-K key-format] [-O sort-order] [-t target-pane] [template]";
    "choose-tree" => typed!(Client, ChooseTree, clients::ChooseTree),
        getopt: "GNrswZF:f:K:O:t:", arity: (0, Some(1)),
        usage: "[-GNrswZ] [-F format] [-f filter] [-K key-format] [-O sort-order] [-t target-pane] [template]";
    "clear-history" ("clearhist") => typed!(Pane, ClearHistory, panes::ClearHistory),
        getopt: "Ht:", arity: (0, Some(0)),
        usage: "[-H] [-t target-pane]";
    "clear-prompt-history" ("clearphist") => typed!(Client, ClearPromptHistory, clients::ClearPromptHistory),
        getopt: "T:", arity: (0, Some(0)),
        usage: "[-T prompt-type]";
    "clock-mode" => typed!(Client, ClockMode, clients::ClockMode),
        getopt: "t:", arity: (0, Some(0)),
        usage: "[-t target-pane]";
    "command-prompt" => typed!(Client, Prompt, clients::CommandPrompt),
        getopt: "1CbeFiklI:Np:t:T:", arity: (0, Some(1)),
        usage: "[-1CbeFiklN] [-I inputs] [-p prompts] [-t target-client] [-T prompt-type] [template]";
    "confirm-before" ("confirm") => typed!(Client, ConfirmBefore, clients::ConfirmBefore),
        getopt: "byc:p:t:", arity: (1, Some(1)),
        usage: "[-by] [-c confirm-key] [-p prompt] [-t target-client] command";
    "copy-mode" => typed!(Pane, CopyMode, panes::CopyMode),
        getopt: "deHMqSus:t:", arity: (0, Some(0)),
        usage: "[-deHMqSu] [-s src-pane] [-t target-pane]";
    "customize-mode" => typed!(Client, CustomizeMode, clients::CustomizeMode),
        getopt: "NZF:f:t:", arity: (0, Some(0)),
        usage: "[-NZ] [-F format] [-f filter] [-t target-pane]";
    "delete-buffer" ("deleteb") => typed!(Buffer, Delete, buffers::DeleteBuffer),
        getopt: "b:", arity: (0, Some(0)),
        usage: "[-b buffer-name]";
    "detach-client" ("detach") => typed!(Client, Detach, clients::DetachClient),
        getopt: "aPE:s:t:", arity: (0, Some(0)),
        usage: "[-aP] [-E shell-command] [-s target-session] [-t target-client]";
    "display-menu" ("menu") => typed!(Client, DisplayMenu, clients::DisplayMenu),
        getopt: "MOb:c:C:H:s:S:t:T:x:y:", arity: (1, None),
        usage: "[-MO] [-b border-lines] [-c target-client] [-C starting-choice] [-H selected-style] [-s style] [-S border-style] [-t target-pane] [-T title] [-x position] [-y position] name [key] [command] ...";
    "display-message" ("display") => typed!(Client, DisplayMessage, clients::DisplayMessage),
        getopt: "aCIlNpvc:d:F:t:", arity: (0, Some(1)),
        usage: "[-aCIlNpv] [-c target-client] [-d delay] [-F format] [-t target-pane] [message]";
    "display-popup" ("popup") => typed!(Client, DisplayPopup, clients::DisplayPopup),
        getopt: "BCEkNb:c:d:e:h:s:S:t:T:w:x:y:", arity: (0, None),
        usage: "[-BCEkN] [-b border-lines] [-c target-client] [-d start-directory] [-e environment] [-h height] [-s style] [-S border-style] [-t target-pane] [-T title] [-w width] [-x position] [-y position] [shell-command [argument ...]]";
    "display-panes" ("displayp") => typed!(Client, DisplayPanes, clients::DisplayPanes),
        getopt: "bNd:t:", arity: (0, Some(1)),
        usage: "[-bN] [-d duration] [-t target-client] [template]";
    "find-window" ("findw") => typed!(Window, Find, windows::FindWindow),
        getopt: "CiNrTZt:", arity: (1, Some(1)),
        usage: "[-CiNrTZ] [-t target-pane] match-string";
    "has-session" ("has") => typed!(Session, Has, sessions::HasSession),
        getopt: "t:", arity: (0, Some(0)),
        usage: "[-t target-session]";
    "if-shell" ("if") => typed!(Execution, IfShell, execution::IfShell),
        getopt: "bFt:", arity: (2, Some(3)),
        usage: "[-bF] [-t target-pane] shell-command command [command]";
    "join-pane" ("joinp") => typed!(Pane, Join, panes::MovePane),
        getopt: "bdfhvl:s:t:", arity: (0, Some(0)),
        usage: "[-bdfhv] [-l size] [-s src-pane] [-t dst-pane]";
    "kill-pane" ("killp") => typed!(Pane, Kill, panes::KillPane),
        getopt: "at:", arity: (0, Some(0)),
        usage: "[-a] [-t target-pane]";
    "kill-server" => bare!(Server, Kill),
        getopt: "", arity: (0, Some(0)),
        usage: "";
    "kill-session" => typed!(Session, Kill, sessions::KillSession),
        getopt: "aCgt:", arity: (0, Some(0)),
        usage: "[-aCg] [-t target-session]";
    "kill-window" ("killw") => typed!(Window, Kill, windows::KillWindow),
        getopt: "at:", arity: (0, Some(0)),
        usage: "[-a] [-t target-window]";
    "last-pane" ("lastp") => typed!(Pane, Last, panes::LastPane),
        getopt: "deZt:", arity: (0, Some(0)),
        usage: "[-deZ] [-t target-window]";
    "last-window" ("last") => typed!(Window, Last, windows::LastWindow),
        getopt: "t:", arity: (0, Some(0)),
        usage: "[-t target-session]";
    "link-window" ("linkw") => typed!(Window, Link, windows::LinkWindow),
        getopt: "abdks:t:", arity: (0, Some(0)),
        usage: "[-abdk] [-s src-window] [-t dst-window]";
    "list-buffers" ("lsb") => typed!(Buffer, List, buffers::ListBuffers),
        getopt: "F:f:O:r", arity: (0, Some(0)),
        usage: "[-F format] [-f filter] [-O order]";
    "list-clients" ("lsc") => typed!(Client, List, clients::ListClients),
        getopt: "F:f:O:rt:", arity: (0, Some(0)),
        usage: "[-F format] [-f filter] [-O order][-t target-session]";
    "list-commands" ("lscm") => typed!(Server, ListCommands, server::ListCommands),
        getopt: "F:", arity: (0, Some(1)),
        usage: "[-F format] [command]";
    "list-keys" ("lsk") => typed!(Keys, List, keys::ListKeys),
        getopt: "1aF:NO:P:rT:", arity: (0, Some(1)),
        usage: "[-1aNr] [-F format] [-O order] [-P prefix-string][-T key-table] [key]";
    "list-panes" ("lsp") => typed!(Pane, List, panes::ListPanes),
        getopt: "aF:f:O:rst:", arity: (0, Some(0)),
        usage: "[-asr] [-F format] [-f filter] [-O order][-t target-window]";
    "list-sessions" ("ls") => typed!(Session, List, sessions::ListSessions),
        getopt: "F:f:O:r", arity: (0, Some(0)),
        usage: "[-r] [-F format] [-f filter] [-O order]";
    "list-windows" ("lsw") => typed!(Window, List, windows::ListWindows),
        getopt: "aF:f:O:rt:", arity: (0, Some(0)),
        usage: "[-ar] [-F format] [-f filter] [-O order][-t target-session]";
    "load-buffer" ("loadb") => typed!(Buffer, Load, buffers::LoadBuffer),
        getopt: "b:t:w", arity: (1, Some(1)),
        usage: "[-b buffer-name] [-t target-client] path";
    "lock-client" ("lockc") => typed!(Client, Lock, clients::LockClient),
        getopt: "t:", arity: (0, Some(0)),
        usage: "[-t target-client]";
    "lock-server" ("lock") => bare!(Server, Lock),
        getopt: "", arity: (0, Some(0)),
        usage: "";
    "lock-session" ("locks") => typed!(Server, LockSession, server::LockSession),
        getopt: "t:", arity: (0, Some(0)),
        usage: "[-t target-session]";
    "move-pane" ("movep") => typed!(Pane, Move, panes::MovePane),
        getopt: "bdfhvl:s:t:", arity: (0, Some(0)),
        usage: "[-bdfhv] [-l size] [-s src-pane] [-t dst-pane]";
    "move-window" ("movew") => typed!(Window, Move, windows::MoveWindow),
        getopt: "abdkrs:t:", arity: (0, Some(0)),
        usage: "[-abdkr] [-s src-window] [-t dst-window]";
    "new-pane" ("newp") => typed!(Pane, New, panes::NewPane),
        getopt: "bc:de:EfF:hIkl:Lm:p:PR:s:S:t:vx:X:y:Y:Z", arity: (0, None),
        usage: "[-bdefhIklPvZ] [-c start-directory] [-e environment] [-F format] [-l size] [-m message] [-p percentage] [-s style] [-S active-border-style] [-R inactive-border-style] [-x width] [-y height] [-X x-position] [-Y y-position] [-t target-pane] [shell-command [argument ...]]";
    "new-session" ("new") => typed!(Session, New, sessions::NewSession),
        getopt: "AdDEPXc:e:F:f:n:s:t:x:y:", arity: (0, None),
        usage: "[-AdDEPX] [-c start-directory] [-e environment] [-F format] [-f flags] [-n window-name] [-s session-name] [-t target-session] [-x width] [-y height] [shell-command [argument ...]]";
    "new-window" ("neww") => typed!(Window, New, windows::NewWindow),
        getopt: "abdkPSc:e:F:n:t:", arity: (0, None),
        usage: "[-abdkPS] [-c start-directory] [-e environment] [-F format] [-n window-name] [-t target-window] [shell-command [argument ...]]";
    "next-layout" ("nextl") => typed!(Pane, NextLayout, panes::CycleLayout),
        getopt: "t:", arity: (0, Some(0)),
        usage: "[-t target-window]";
    "next-window" ("next") => typed!(Window, Next, windows::NextWindow),
        getopt: "at:", arity: (0, Some(0)),
        usage: "[-a] [-t target-session]";
    "paste-buffer" ("pasteb") => typed!(Buffer, Paste, buffers::PasteBuffer),
        getopt: "db:prSs:t:", arity: (0, Some(0)),
        usage: "[-dprS] [-s separator] [-b buffer-name] [-t target-pane]";
    "pipe-pane" ("pipep") => typed!(Pane, Pipe, panes::PipePane),
        getopt: "IOot:", arity: (0, Some(1)),
        usage: "[-IOo] [-t target-pane] [shell-command]";
    "previous-layout" ("prevl") => typed!(Pane, PreviousLayout, panes::CycleLayout),
        getopt: "t:", arity: (0, Some(0)),
        usage: "[-t target-window]";
    "previous-window" ("prev") => typed!(Window, Previous, windows::PreviousWindow),
        getopt: "at:", arity: (0, Some(0)),
        usage: "[-a] [-t target-session]";
    "refresh-client" ("refresh") => typed!(Client, Refresh, clients::RefreshClient),
        getopt: "cDlLRSUA:B:C:f:F:r:t:", arity: (0, Some(1)),
        usage: "[-cDlLRSU] [-A pane:state] [-B name:what:format] [-C XxY] [-f flags] [-r pane:report] [-t target-client] [adjustment]";
    "rename-session" ("rename") => typed!(Session, Rename, sessions::RenameSession),
        getopt: "t:", arity: (1, Some(1)),
        usage: "[-t target-session] new-name";
    "rename-window" ("renamew") => typed!(Window, Rename, windows::RenameWindow),
        getopt: "t:", arity: (1, Some(1)),
        usage: "[-t target-window] new-name";
    "resize-pane" ("resizep") => typed!(Pane, Resize, panes::ResizePane),
        getopt: "DLMRTUZx:y:t:", arity: (0, Some(1)),
        usage: "[-DLMRTUZ] [-x width] [-y height] [-t target-pane] [adjustment]";
    "resize-window" ("resizew") => typed!(Pane, ResizeWindow, panes::ResizeWindow),
        getopt: "aADLRUx:y:t:", arity: (0, Some(1)),
        usage: "[-aADLRU] [-x width] [-y height] [-t target-window] [adjustment]";
    "respawn-pane" ("respawnp") => typed!(Pane, Respawn, panes::RespawnPane),
        getopt: "kc:e:t:", arity: (0, None),
        usage: "[-k] [-c start-directory] [-e environment] [-t target-pane] [shell-command [argument ...]]";
    "respawn-window" ("respawnw") => typed!(Window, Respawn, windows::RespawnWindow),
        getopt: "kc:e:t:", arity: (0, None),
        usage: "[-k] [-c start-directory] [-e environment] [-t target-window] [shell-command [argument ...]]";
    "rotate-window" ("rotatew") => typed!(Pane, RotateWindow, panes::RotateWindow),
        getopt: "DUZt:", arity: (0, Some(0)),
        usage: "[-DUZ] [-t target-window]";
    "run-shell" ("run") => typed!(Execution, RunShell, execution::RunShell),
        getopt: "bd:Ct:Es:c:", arity: (0, None),
        usage: "[-bCE] [-c start-directory] [-d delay] [-t target-pane] [shell-command [argument ...]]";
    "save-buffer" ("saveb") => typed!(Buffer, Save, buffers::SaveBuffer),
        getopt: "ab:", arity: (1, Some(1)),
        usage: "[-a] [-b buffer-name] path";
    "select-layout" ("selectl") => typed!(Pane, SelectLayout, panes::SelectLayout),
        getopt: "Enopt:", arity: (0, Some(1)),
        usage: "[-Enop] [-t target-pane] [layout-name]";
    "select-pane" ("selectp") => typed!(Pane, Select, panes::SelectPane),
        getopt: "DdeLlMmRUZT:t:", arity: (0, Some(0)),
        usage: "[-DdeLlMmRUZ] [-T title] [-t target-pane]";
    "select-window" ("selectw") => typed!(Window, Select, windows::SelectWindow),
        getopt: "lnpTt:", arity: (0, Some(0)),
        usage: "[-lnpT] [-t target-window]";
    "send-keys" ("send") => typed!(Keys, Send, keys::SendKeys),
        getopt: "FHKlMRXc:N:t:", arity: (0, None),
        usage: "[-FHKlMRX] [-c target-client] [-N repeat-count] [-t target-pane] [key ...]";
    "send-prefix" => typed!(Keys, SendPrefix, keys::SendPrefix),
        getopt: "2t:", arity: (0, Some(0)),
        usage: "[-2] [-t target-pane]";
    "server-access" => typed!(Server, Access, server::ServerAccess),
        getopt: "adlrwt:", arity: (0, Some(1)),
        usage: "[-adlrw] [-t target-pane] [user]";
    "set-buffer" ("setb") => typed!(Buffer, Set, buffers::SetBuffer),
        getopt: "awb:n:t:", arity: (0, Some(1)),
        usage: "[-aw] [-b buffer-name] [-n new-buffer-name] [-t target-client] [data]";
    "set-environment" ("setenv") => typed!(Configuration, SetEnvironment, configuration::SetEnvironment),
        getopt: "Fhgrut:", arity: (1, Some(2)),
        usage: "[-Fhgru] [-t target-session] variable [value]";
    "set-hook" => typed!(Configuration, SetHook, configuration::SetHook),
        getopt: "agpRuwt:", arity: (1, Some(2)),
        usage: "[-agpRuw] [-t target-pane] hook [command]";
    "set-option" ("set") => typed!(Configuration, SetOption, configuration::SetOption),
        getopt: "aFgopqsuUwt:", arity: (1, Some(2)),
        usage: "[-aFgopqsuUw] [-t target-pane] option [value]";
    "set-window-option" ("setw") => typed!(Configuration, SetWindowOption, configuration::SetOption),
        getopt: "aFgoqut:", arity: (1, Some(2)),
        usage: "[-aFgoqu] [-t target-window] option [value]";
    "show-buffer" ("showb") => typed!(Buffer, Show, buffers::ShowBuffer),
        getopt: "b:", arity: (0, Some(0)),
        usage: "[-b buffer-name]";
    "show-environment" ("showenv") => typed!(Configuration, ShowEnvironment, configuration::ShowEnvironment),
        getopt: "hgst:", arity: (0, Some(1)),
        usage: "[-hgs] [-t target-session] [variable]";
    "show-hooks" => typed!(Configuration, ShowHooks, configuration::ShowHooks),
        getopt: "gpwt:", arity: (0, Some(1)),
        usage: "[-gpw] [-t target-pane] [hook]";
    "show-messages" ("showmsgs") => typed!(Server, ShowMessages, server::ShowMessages),
        getopt: "JTt:", arity: (0, Some(0)),
        usage: "[-JT] [-t target-client]";
    "show-options" ("show") => typed!(Configuration, ShowOptions, configuration::ShowOptions),
        getopt: "AgHpqsvwt:", arity: (0, Some(1)),
        usage: "[-AgHpqsvw] [-t target-pane] [option]";
    "show-prompt-history" ("showphist") => typed!(Client, ShowPromptHistory, clients::ShowPromptHistory),
        getopt: "T:", arity: (0, Some(0)),
        usage: "[-T prompt-type]";
    "show-window-options" ("showw") => typed!(Configuration, ShowWindowOptions, configuration::ShowOptions),
        getopt: "gvt:", arity: (0, Some(1)),
        usage: "[-gv] [-t target-window] [option]";
    "source-file" ("source") => typed!(Execution, SourceFile, execution::SourceFile),
        getopt: "Fnqvt:", arity: (1, None),
        usage: "[-Fnqv] [-t target-pane] path ...";
    "split-window" ("splitw") => typed!(Pane, Split, panes::SplitWindow),
        getopt: "bc:de:EfF:hIkl:m:p:PR:s:S:t:vZ", arity: (0, None),
        usage: "[-bdefhIklPvZ] [-c start-directory] [-e environment] [-F format] [-l size] [-m message] [-p percentage] [-s style] [-S active-border-style] [-R inactive-border-style] [-t target-pane] [shell-command [argument ...]]";
    "start-server" ("start") => bare!(Server, Start),
        getopt: "", arity: (0, Some(0)),
        usage: "";
    "suspend-client" ("suspendc") => typed!(Client, Suspend, clients::SuspendClient),
        getopt: "t:", arity: (0, Some(0)),
        usage: "[-t target-client]";
    "swap-pane" ("swapp") => typed!(Pane, Swap, panes::SwapPane),
        getopt: "dDUZs:t:", arity: (0, Some(0)),
        usage: "[-dDUZ] [-s src-pane] [-t dst-pane]";
    "swap-window" ("swapw") => typed!(Window, Swap, windows::SwapWindow),
        getopt: "ds:t:", arity: (0, Some(0)),
        usage: "[-d] [-s src-window] [-t dst-window]";
    "switch-client" ("switchc") => typed!(Client, Switch, clients::SwitchClient),
        getopt: "c:EFlnO:pt:rT:Z", arity: (0, Some(0)),
        usage: "[-ElnprZ] [-c target-client] [-t target-session] [-T key-table] [-O order]";
    "unbind-key" ("unbind") => typed!(Keys, Unbind, keys::UnbindKey),
        getopt: "anqT:", arity: (0, Some(1)),
        usage: "[-anq] [-T key-table] key";
    "unlink-window" ("unlinkw") => typed!(Window, Unlink, windows::UnlinkWindow),
        getopt: "kt:", arity: (0, Some(0)),
        usage: "[-k] [-t target-window]";
    "wait-for" ("wait") => typed!(Execution, WaitFor, execution::WaitFor),
        getopt: "LSU", arity: (1, Some(1)),
        usage: "[-L|-S|-U] channel";
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

/// The catalog row for a canonical command name, or `None` for a name that is
/// not in the command table.
///
/// The resolver hands most callers a `&CommandSpec` directly; this is for the
/// few that hold only a name. Every per-command contract — [`CommandSpec::getopt`],
/// [`CommandSpec::arity`], [`CommandSpec::usage`] — is read off the returned row,
/// so a name outside the table stays permissive rather than half-validated.
pub(in crate::server) fn spec(name: &str) -> Option<&'static CommandSpec> {
    COMMAND_SPECS.iter().find(|spec| spec.name == name)
}

/// The getopt spec for a canonical command name; `None` means the name isn't in
/// the command table, so flag validation is skipped for it (permissive).
pub fn getopt(name: &str) -> Option<&'static str> {
    spec(name).map(|spec| spec.getopt)
}

/// Format one command's `list-commands` line: `name`, optionally ` (alias)`, then
/// a single space and the row's [`CommandSpec::usage`] string. tmux always emits
/// that space, so an argument-less command (empty usage) yields a trailing space
/// (`kill-server `). `None` for a name not in the command table.
pub fn command_line(name: &str) -> Option<String> {
    let spec = spec(name)?;
    let usage = spec.usage;
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
    fn catalog_names_are_unique() {
        let names = COMMAND_SPECS
            .iter()
            .map(|spec| spec.name)
            .collect::<HashSet<_>>();
        assert_eq!(names.len(), COMMAND_SPECS.len());
    }

    #[test]
    fn every_command_parses_its_minimal_arguments() {
        // A parse hook only shapes arguments the parse phase already validated;
        // it does not add validation of its own. So the smallest argv the arity
        // table admits has to parse for every row in the catalog — a row whose
        // hook rejects it would reject the command outright.
        for spec in COMMAND_SPECS {
            let (minimum, _) = spec.arity;
            let mut argv = vec![spec.name.to_string()];
            argv.extend((0..minimum).map(|index| format!("operand{index}")));
            let lexed = ParsedArgs::lex(spec.name, &argv);
            assert!(
                (spec.parse)(&lexed).is_ok(),
                "{} rejects its minimal argument list",
                spec.name
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
    fn spec_lookup_is_permissive_outside_the_table() {
        // Every canonical command carries its own getopt, arity, and usage by
        // construction; only a name outside the table has none, which is what
        // keeps flag and arity validation permissive for it.
        assert_eq!(spec("kill-server").map(|spec| spec.usage), Some(""));
        assert_eq!(getopt("list-sessions"), Some("F:f:O:r"));
        assert!(spec("frobnicate").is_none());
        assert_eq!(getopt("frobnicate"), None);
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
