//! Antigravity CLI (`agy`) agent detector.
//!
//! `agy` never sets a terminal title, so detection is screen-only — not even the
//! resting-title fallback other agents offer. Its main TUI renders inline: a
//! bordered `>` composer above a footer carrying `? for shortcuts` on the left
//! and `<Model> · <effort>` on the right. That footer is present in every state
//! and doubles as the "this really is agy" signature.
//!
//! What makes the classification more than a status-line match: while a tool
//! runs, agy hands the composer and the resting footer straight back and tracks
//! the tool in a task panel below. The turn is still in progress there, so the
//! panel's running entry is read as working — otherwise a pane would read idle
//! mid-turn and a loop runner would end the run on top of a live tool.
//!
//! Observable difference from the other agents: `agy` keeps its conversation in
//! a sqlite database (`~/.gemini/antigravity-cli/conversations/<uuid>.db`)
//! rather than a JSONL transcript. The open database still names the session, so
//! `#{pane_agent_session_id}` is reported, but the line-oriented session scan
//! cannot read a model out of it — `#{pane_agent_model}` stays empty for agy
//! panes.

use std::ffi::{OsStr, OsString};
use std::path::Path;

use super::{is_braille, is_uuid, AgentDetector, AgentState, Detection, SessionIdSource};

/// Rows from the bottom of the screen tail that carry live UI. The composer,
/// the status line and every dialog sit here; bounding the scan keeps an
/// answered permission dialog still resting in scrollback from being read as a
/// live one.
const LIVE_ROWS: usize = 16;

/// Status verbs accepted without the leading spinner cell.
const SPINNER_STATUSES: [&str; 2] = ["generating...", "loading..."];

/// Recognizes Antigravity CLI panes.
pub(crate) struct AgyDetector;

impl AgentDetector for AgyDetector {
    fn label(&self) -> &'static str {
        "agy"
    }

    fn matches_program(&self, program: &OsStr) -> bool {
        agy_program_name(program)
    }

    fn invocation_state(&self, arguments: &[OsString]) -> Option<AgentState> {
        agy_headless_invocation(arguments).then_some(AgentState::Working)
    }

    fn session_id_source(&self) -> Option<SessionIdSource> {
        Some(SessionIdSource::ProcessTreeOpenFiles)
    }

    fn session_id_from_open_file(&self, path: &Path) -> Option<String> {
        session_id_from_conversation_path(path)
    }

    fn detect(&self, screen: &str, _title: Option<&str>) -> Detection {
        detect(screen)
    }
}

/// `agy` holds its conversation database — and the sqlite sidecars beside it —
/// open for the life of the conversation, and names it after the conversation
/// id. Nothing is open before the first prompt, so a freshly started pane
/// reports no session until then.
fn session_id_from_conversation_path(path: &Path) -> Option<String> {
    let name = path.file_name()?.to_str()?;
    let stem = ["-shm", "-wal", ""]
        .iter()
        .find_map(|suffix| name.strip_suffix(suffix)?.strip_suffix(".db"))?;
    let conversations = path.parent()?;
    if conversations.file_name()? != "conversations"
        || conversations.parent()?.file_name()? != "antigravity-cli"
    {
        return None;
    }
    is_uuid(stem).then(|| stem.to_ascii_lowercase())
}

fn detect(screen: &str) -> Detection {
    let all = screen.lines().collect::<Vec<_>>();
    let live = &all[all.len().saturating_sub(LIVE_ROWS)..];
    let text = live.join("\n").to_lowercase();

    // A dialog owns the keyboard until it is answered, so it outranks the
    // status line still visible above it.
    if is_permission_dialog(&text) || is_trust_dialog(&text) {
        return Detection::State(AgentState::Blocked);
    }

    // User-invoked overlays carry no lifecycle signal of their own. The
    // slash-command menu additionally flips the left footer to `esc to cancel`,
    // which is why that footer alone is never read as work in progress.
    if is_shortcuts_overlay(&text) || is_slash_menu(&text) {
        return Detection::KeepPrevious;
    }

    // A running background task is work in progress even though agy hands the
    // composer back while it waits: the turn is not over, and the idle footer
    // it shows meanwhile must not end a run.
    if has_spinner_status(live) || has_running_task(live) {
        return Detection::State(AgentState::Working);
    }

    if has_idle_composer(live) || has_agy_signature(live) {
        return Detection::State(AgentState::Idle);
    }

    Detection::State(AgentState::Unknown)
}

fn is_permission_dialog(text: &str) -> bool {
    text.contains("requesting permission for:") && text.contains("do you want to proceed?")
}

fn is_trust_dialog(text: &str) -> bool {
    text.contains("do you trust the contents of this project?")
}

fn is_shortcuts_overlay(text: &str) -> bool {
    text.contains("navigate") && text.contains("esc close")
}

fn is_slash_menu(text: &str) -> bool {
    text.contains("enter select") && text.contains("tab complete")
}

/// agy's status line is one verb followed by an ellipsis — `Generating...`
/// while the model streams, `Loading...` while it takes a tool result back —
/// normally behind a braille spinner cell, which the first frame can be drawn
/// without. Requiring a single word keeps prose ending in an ellipsis out.
fn has_spinner_status(lines: &[&str]) -> bool {
    lines.iter().any(|line| {
        let trimmed = line.trim_start();
        match trimmed.chars().next() {
            // Behind the spinner, take any single-word ellipsis status, so a
            // verb not seen yet still reports work in progress.
            Some(first) if is_braille(first) => {
                let status = trimmed[first.len_utf8()..].trim();
                status.ends_with("...")
                    && !status.trim_end_matches('.').contains(char::is_whitespace)
            }
            // Without it, only the known verbs: transcript prose ending in an
            // ellipsis would otherwise pin the pane to working forever.
            _ => {
                let lower = trimmed.to_lowercase();
                SPINNER_STATUSES
                    .iter()
                    .any(|status| lower.starts_with(status))
            }
        }
    })
}

/// A background task agy is still running, listed under the composer as
/// `● [HH:MM:SS] <command> running`. agy returns the composer and the resting
/// footer while such a task runs, so without this the pane would read idle in
/// the middle of a turn.
fn has_running_task(lines: &[&str]) -> bool {
    lines.iter().any(|line| {
        let trimmed = line.trim();
        trimmed.starts_with('●') && trimmed.to_lowercase().ends_with(" running")
    })
}

/// The resting composer: a `>` prompt line together with the idle left footer.
/// The prompt is not required to be inside the *last* bordered region — a
/// background-task panel draws its own box below the composer.
fn has_idle_composer(lines: &[&str]) -> bool {
    let prompt = lines.iter().any(|line| {
        let trimmed = line.trim_start();
        trimmed == ">" || trimmed.starts_with("> ")
    });
    prompt
        && lines
            .iter()
            .any(|line| line.to_lowercase().contains("? for shortcuts"))
}

/// Evidence the pane is running agy at all: the startup banner, or the
/// `<Model> · <effort>` right footer that every state carries.
fn has_agy_signature(lines: &[&str]) -> bool {
    lines.iter().any(|line| {
        let lower = line.trim_end().to_lowercase();
        lower.contains("antigravity cli") || is_model_effort_footer(&lower)
    })
}

/// agy's right footer is `<Model> · <effort>`, with further `·`-separated
/// fields appended while background tasks exist (`· 1 task(s) · /tasks`).
/// Looking for an effort level in its own field tolerates both, and the left
/// footer sharing the row, while keeping ordinary `·`-separated hint rows out.
fn is_model_effort_footer(lower: &str) -> bool {
    let mut fields = lower.split(" · ");
    let leading = fields.next().unwrap_or_default();
    !leading.trim().is_empty()
        && fields.any(|field| matches!(field.trim(), "low" | "medium" | "high"))
}

fn agy_program_name(program: &OsStr) -> bool {
    let name = Path::new(program)
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or_default()
        .trim_start_matches('.')
        .to_ascii_lowercase();
    matches!(name.as_str(), "agy" | "agy.exe")
}

/// `agy -p/--print` prints one answer instead of running the TUI, so there is
/// no prompt or status line to read: the process is working for its whole
/// lifetime. `-i/--prompt-interactive` seeds a prompt and stays interactive, so
/// it is deliberately not matched here.
fn agy_headless_invocation(arguments: &[OsString]) -> bool {
    arguments
        .iter()
        .skip(1)
        .any(|argument| matches!(argument.to_str(), Some("-p" | "--print" | "--prompt")))
}

#[cfg(test)]
mod tests {
    use std::ffi::{OsStr, OsString};
    use std::path::Path;

    use super::{
        agy_headless_invocation, agy_program_name, detect, session_id_from_conversation_path,
        AgentState, Detection,
    };

    const RULE: &str = "────────────────────────────────────────";

    fn screen(body: &str) -> String {
        format!("Antigravity CLI 1.1.10\n\n{body}")
    }

    #[test]
    fn resting_composer_reports_idle() {
        let text = screen(&format!(
            "{RULE}\n>\n{RULE}\n? for shortcuts                    Gemini 3.6 Flash · high"
        ));
        assert_eq!(detect(&text), Detection::State(AgentState::Idle));
    }

    #[test]
    fn spinner_status_reports_working() {
        // `Loading...` is what agy shows taking a tool result back, and the
        // spinner cell can be missing on the first frame of either verb.
        for status in [
            "⣷  Generating...",
            "   Generating...",
            "⣽  Loading...",
            "   Loading...",
        ] {
            let text = screen(&format!(
                "● Bash(git status) (ctrl+o to expand)\n{status}\n{RULE}\n>\n{RULE}\n\
                 esc to cancel                      Gemini 3.6 Flash · high"
            ));
            assert_eq!(detect(&text), Detection::State(AgentState::Working));
        }
    }

    #[test]
    fn a_running_background_task_reports_working_despite_the_resting_footer() {
        // agy returns the composer and the `? for shortcuts` footer while a
        // tool runs, tracking it in a panel below. The turn is not over.
        let text = screen(&format!(
            "{RULE}\n>\n{RULE}\n  ● [22:37:14] sleep 12 && echo finished running\n{RULE}\n\
             ? for shortcuts                Gemini 3.6 Flash · high · 1 task(s) · /tasks"
        ));
        assert_eq!(detect(&text), Detection::State(AgentState::Working));
    }

    #[test]
    fn a_finished_tool_in_the_transcript_does_not_report_working() {
        // The transcript keeps the tool entry and prose that can end in an
        // ellipsis; neither is a live status line.
        let text = screen(&format!(
            "● Bash(git status) (ctrl+o to expand)\n  Analyzing the working tree...\n\
             {RULE}\n>\n{RULE}\n? for shortcuts                    Gemini 3.6 Flash · high"
        ));
        assert_eq!(detect(&text), Detection::State(AgentState::Idle));
    }

    #[test]
    fn permission_dialog_outranks_the_status_line() {
        let text = screen(
            "⣷  Generating...\n● Bash(git status) (ctrl+o to expand)\nRequesting permission for:\n\
             git status\nDo you want to proceed?\n1. Yes\n2. Yes, and don't ask again\n\
             3. Yes, and don't ask again this session\n4. No\n\
             ↑/↓ Navigate · tab Amend · ctrl+g edit/expand command",
        );
        assert_eq!(detect(&text), Detection::State(AgentState::Blocked));
    }

    #[test]
    fn trust_dialog_reports_blocked() {
        let text = screen(
            "Do you trust the contents of this project?\n> Yes, I trust this folder\n  No, exit\n\
             ↑/↓ Navigate · enter Confirm\n                    Gemini 3.6 Flash · high",
        );
        assert_eq!(detect(&text), Detection::State(AgentState::Blocked));
    }

    #[test]
    fn shortcuts_overlay_preserves_previous_state() {
        let text = screen(
            "Shortcuts\n  ctrl+o  expand tool output\n\
             Keyboard: ↑/↓ Navigate  ←/→ Switch View  esc Close",
        );
        assert_eq!(detect(&text), Detection::KeepPrevious);
    }

    #[test]
    fn slash_menu_preserves_previous_state_despite_cancel_footer() {
        // `esc to cancel` shows while the menu is open, so the footer alone must
        // not be read as work in progress.
        let text = screen(&format!(
            "/help   Show help\n/model  Switch model\n\
             ↑/↓ Navigate · enter Select · tab Complete\n{RULE}\n> /\n{RULE}\n\
             esc to cancel                      Gemini 3.6 Flash · high"
        ));
        assert_eq!(detect(&text), Detection::KeepPrevious);
    }

    #[test]
    fn model_effort_footer_is_the_idle_fallback() {
        // No banner and no composer left on screen, but the right footer is
        // still agy's own.
        assert_eq!(
            detect("some scrolled output\n                     Gemini 3.6 Flash · high"),
            Detection::State(AgentState::Idle)
        );
        assert_eq!(
            detect("some scrolled output\n↑/↓ Navigate · enter Confirm"),
            Detection::State(AgentState::Unknown)
        );
    }

    #[test]
    fn recognizes_agy_programs() {
        assert!(agy_program_name(OsStr::new("agy")));
        assert!(agy_program_name(OsStr::new(
            "/run/current-system/sw/bin/agy"
        )));
        assert!(agy_program_name(OsStr::new("AGY.EXE")));
        assert!(!agy_program_name(OsStr::new("agyness")));
    }

    #[test]
    fn recognizes_headless_invocations() {
        let args = |values: &[&str]| values.iter().map(OsString::from).collect::<Vec<_>>();

        assert!(agy_headless_invocation(&args(&["agy", "-p", "do work"])));
        assert!(agy_headless_invocation(&args(&[
            "agy", "--print", "do work"
        ])));
        // The interactive seed prompt keeps the TUI, so it is not headless.
        assert!(!agy_headless_invocation(&args(&[
            "agy",
            "--prompt-interactive",
            "do work"
        ])));
        assert!(!agy_headless_invocation(&args(&["agy"])));
    }

    #[test]
    fn extracts_session_id_from_open_conversation_database() {
        let base = "/home/me/.gemini/antigravity-cli/conversations";
        let id = "019F9757-7C53-7898-A7D1-C8C780212888";
        for suffix in [".db", ".db-wal", ".db-shm"] {
            assert_eq!(
                session_id_from_conversation_path(&Path::new(base).join(format!("{id}{suffix}")))
                    .as_deref(),
                Some("019f9757-7c53-7898-a7d1-c8c780212888")
            );
        }
        assert_eq!(
            session_id_from_conversation_path(&Path::new(base).join("settings.json")),
            None
        );
        // The same file name outside agy's conversation directory is not a
        // session of ours.
        assert_eq!(
            session_id_from_conversation_path(Path::new(
                "/home/me/other/conversations/019f9757-7c53-7898-a7d1-c8c780212888.db"
            )),
            None
        );
    }
}
