//! Codex agent detector.
//!
//! The rules are a compact port of Herdr's Codex detector. The window title
//! (OSC 0/2) is the primary channel — Codex reports "Action Required", a braille
//! working spinner, or a resting title there — and it outranks the screen
//! heuristics: live approval prompts, current status markers identifying work,
//! and a current `›` prompt identifying idle Codex.

use std::ffi::{OsStr, OsString};
use std::path::Path;

use super::{
    AgentDetector, AgentState, CursorEvidence, Detection, SessionIdSource, is_title_spinner,
    is_uuid, title_working_spinner,
};

/// Recognizes OpenAI Codex panes.
pub(crate) struct CodexDetector;

impl AgentDetector for CodexDetector {
    fn label(&self) -> &'static str {
        "codex"
    }

    fn matches_program(&self, program: &OsStr) -> bool {
        codex_program_name(program)
    }

    fn invocation_state(&self, arguments: &[OsString]) -> Option<AgentState> {
        codex_headless_invocation(arguments).then_some(AgentState::Working)
    }

    fn session_id_source(&self) -> Option<SessionIdSource> {
        Some(SessionIdSource::ProcessTreeOpenFiles)
    }

    fn session_id_from_open_file(&self, path: &Path) -> Option<String> {
        session_id_from_rollout_path(path)
    }

    fn detect(&self, screen: &str, title: Option<&str>) -> Detection {
        detect(screen, title, None)
    }

    fn detect_with_cursor(
        &self,
        screen: &str,
        title: Option<&str>,
        cursor: CursorEvidence,
    ) -> Detection {
        detect(screen, title, Some(cursor))
    }
}

/// Codex keeps the active rollout open for the lifetime of a thread. Its
/// filename ends in the thread UUID, including when the thread was resumed.
fn session_id_from_rollout_path(path: &Path) -> Option<String> {
    let name = path.file_name()?.to_str()?;
    let stem = name.strip_prefix("rollout-")?.strip_suffix(".jsonl")?;
    let session_id = stem.get(stem.len().checked_sub(36)?..)?;
    is_uuid(session_id).then(|| session_id.to_ascii_lowercase())
}

fn detect(screen: &str, title: Option<&str>, cursor: Option<CursorEvidence>) -> Detection {
    // Highest-priority signals come from the window title (OSC 0/2), which Codex
    // sets to its live status. These mirror Herdr's `osc_title` manifest rules,
    // which outrank every screen heuristic below: an approval request in the
    // title, then a leading braille spinner meaning work in progress.
    if let Some(title) = title {
        if title_indicates_blocked(title) {
            return Detection::State(AgentState::Blocked);
        }
        if title_working_spinner(title) {
            return Detection::State(AgentState::Working);
        }
    }

    let lower = screen.to_lowercase();
    let lines = screen.lines().map(str::trim_start).collect::<Vec<_>>();
    let prompt = current_prompt_index(&lines);
    let after_prompt = lines
        .iter()
        .rposition(|line| is_prompt(line))
        .map(|index| lines[index + 1..].join("\n"))
        .unwrap_or_else(|| screen.to_string())
        .to_lowercase();

    if is_transcript_viewer(&after_prompt) {
        return Detection::KeepPrevious;
    }
    if has_strong_blocker(&after_prompt) || has_weak_blocker(&lower) {
        return Detection::State(AgentState::Blocked);
    }

    if let Some(prompt) = prompt {
        if let Some(marker) = lines[..prompt]
            .iter()
            .rposition(|line| is_block_marker(line))
        {
            let current_block = lines[marker..].join("\n").to_lowercase();
            if has_working_status(&current_block) {
                return Detection::State(AgentState::Working);
            }
        }
        return Detection::State(AgentState::Idle);
    }

    if has_working_status(&lower) {
        return Detection::State(AgentState::Working);
    }
    // Lowest-priority title signal (Herdr `osc_title_idle`): a non-empty title
    // that is neither a spinner nor an approval request means Codex is sitting
    // at its prompt, even when the screen tail carries no other evidence.
    if let Some(title) = title {
        if title_indicates_idle(title) {
            return Detection::State(AgentState::Idle);
        }
    }
    if has_codex_signature(&lower) {
        return Detection::State(AgentState::Idle);
    }
    // Codex shows the terminal cursor only while its composer accepts input.
    // Normal editing requests the terminal's default shape (DECSCUSR 0); Vim
    // insert mode requests a steady bar (6). This is deliberately last-resort
    // evidence and is supplied only after the process tree identified Codex, so
    // an ordinary shell cursor cannot claim an agent pane.
    if cursor.is_some_and(|cursor| cursor.visible && matches!(cursor.shape, 0 | 6)) {
        return Detection::State(AgentState::Idle);
    }
    Detection::State(AgentState::Unknown)
}

fn current_prompt_index(lines: &[&str]) -> Option<usize> {
    let prompt = lines.iter().rposition(|line| is_prompt(line))?;
    if lines[prompt + 1..].iter().any(|line| is_block_marker(line)) {
        None
    } else {
        Some(prompt)
    }
}

fn is_prompt(line: &str) -> bool {
    line == "›" || line.starts_with("› ")
}

fn is_block_marker(line: &str) -> bool {
    line.starts_with(['•', '■', '✗', '✓'])
}

fn is_transcript_viewer(text: &str) -> bool {
    text.contains("↑/↓ to scroll")
        && text.contains("pgup/pgdn to")
        && text.contains("home/end to jump")
        && text.contains("q to quit")
        && (text.contains("esc to edit prev") || text.contains("esc/← to edit prev"))
}

fn has_strong_blocker(text: &str) -> bool {
    [
        "action required",
        "press enter to confirm or esc to cancel",
        "enter to submit answer",
        "enter to submit all",
        "allow command?",
    ]
    .iter()
    .any(|pattern| text.contains(pattern))
}

fn has_weak_blocker(text: &str) -> bool {
    text.contains("[y/n]")
        || text.contains("yes (y)")
        || ((text.contains("do you want to") || text.contains("would you like to"))
            && (text.contains("yes") || text.contains('❯')))
}

fn has_working_status(text: &str) -> bool {
    [
        "queued follow-up inputs",
        "messages to be submitted after next tool call",
        "waiting for background terminal",
        "background terminal running",
        "/ps to view",
        "/stop to close",
        "working (",
        "booting mcp server:",
        "reviewing approval request (",
        "approval requests (",
        "esc to interrupt",
        "ctrl+c to interrupt",
        "press esc to interrupt",
    ]
    .iter()
    .any(|pattern| text.contains(pattern))
}

fn has_codex_signature(text: &str) -> bool {
    text.contains("openai codex") || text.contains("codex cli")
}

/// Herdr `osc_title_blocked`: the title advertises a pending approval.
fn title_indicates_blocked(title: &str) -> bool {
    title.to_lowercase().contains("action required")
}

/// Herdr `osc_title_idle`: the title has visible text but is neither a spinner
/// (a leading spinner cell) nor an approval request.
fn title_indicates_idle(title: &str) -> bool {
    title.chars().any(|c| !c.is_whitespace())
        && !title.starts_with(is_title_spinner)
        && !title.to_lowercase().contains("action required")
}

fn codex_program_name(program: &OsStr) -> bool {
    let name = Path::new(program)
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or_default()
        .trim_start_matches('.')
        .to_ascii_lowercase();
    matches!(
        name.as_str(),
        "codex" | "codex.exe" | "codex.cmd" | "codex.js" | "codex.mjs" | "codex-wrapped"
    )
}

/// `codex exec` (alias `e`) and `codex review` do not run the interactive TUI:
/// they emit a transcript but no prompt or OSC lifecycle title. The process is
/// therefore working for its entire lifetime; its exit is observed separately
/// by the pane process lifecycle.
fn codex_headless_invocation(arguments: &[OsString]) -> bool {
    arguments
        .iter()
        .skip(1)
        .any(|argument| matches!(argument.to_str(), Some("exec" | "e" | "review")))
}

#[cfg(test)]
mod tests {
    use std::ffi::{OsStr, OsString};
    use std::path::Path;

    use super::{
        AgentState, CursorEvidence, Detection, codex_headless_invocation, codex_program_name,
        detect, session_id_from_rollout_path,
    };

    fn detect_without_cursor(screen: &str, title: Option<&str>) -> Detection {
        detect(screen, title, None)
    }

    #[test]
    fn current_prompt_overrides_stale_working_text() {
        let screen = "OpenAI Codex\nold output: esc to interrupt\n› ";
        assert_eq!(
            detect_without_cursor(screen, None),
            Detection::State(AgentState::Idle)
        );
    }

    #[test]
    fn current_status_marker_reports_working() {
        let screen = "OpenAI Codex\n• Working (2s) • esc to interrupt\n› queued follow-up";
        assert_eq!(
            detect_without_cursor(screen, None),
            Detection::State(AgentState::Working)
        );
    }

    #[test]
    fn live_confirmation_reports_blocked() {
        let screen = "OpenAI Codex\n› change files\nAllow command?";
        assert_eq!(
            detect_without_cursor(screen, None),
            Detection::State(AgentState::Blocked)
        );
    }

    #[test]
    fn transcript_viewer_preserves_previous_state() {
        let screen = "›\n↑/↓ to scroll · pgup/pgdn to move · home/end to jump · q to quit · esc to edit prev";
        assert_eq!(detect_without_cursor(screen, None), Detection::KeepPrevious);
    }

    #[test]
    fn title_spinner_reports_working_over_idle_screen() {
        // Codex sets a braille-spinner title while working even when the screen
        // tail still shows the resting prompt. The title outranks the screen.
        let screen = "OpenAI Codex\n› ";
        assert_eq!(
            detect_without_cursor(screen, Some("⠹ Working")),
            Detection::State(AgentState::Working)
        );
    }

    #[test]
    fn title_action_required_reports_blocked_over_prompt() {
        let screen = "OpenAI Codex\n› ";
        assert_eq!(
            detect_without_cursor(screen, Some("Action Required")),
            Detection::State(AgentState::Blocked)
        );
    }

    #[test]
    fn title_outranks_transcript_viewer() {
        // A working title takes precedence even while the transcript viewer is
        // open (which would otherwise preserve the previous state).
        let screen = "›\n↑/↓ to scroll · pgup/pgdn to move · home/end to jump · q to quit · esc to edit prev";
        assert_eq!(
            detect_without_cursor(screen, Some("⠿ Working (12s)")),
            Detection::State(AgentState::Working)
        );
    }

    #[test]
    fn plain_title_is_a_last_resort_idle_signal() {
        // No screen evidence at all, but a non-spinner, non-approval title means
        // Codex is idle at its prompt.
        assert_eq!(
            detect_without_cursor("", Some("codex — my-project")),
            Detection::State(AgentState::Idle)
        );
        // An empty/whitespace title provides nothing.
        assert_eq!(
            detect_without_cursor("", Some("   ")),
            Detection::State(AgentState::Unknown)
        );
    }

    #[test]
    fn visible_composer_cursor_is_last_resort_idle_evidence() {
        for shape in [0, 6] {
            assert_eq!(
                detect(
                    "",
                    None,
                    Some(CursorEvidence {
                        visible: true,
                        shape,
                    }),
                ),
                Detection::State(AgentState::Idle)
            );
        }

        assert_eq!(
            detect(
                "",
                None,
                Some(CursorEvidence {
                    visible: false,
                    shape: 0,
                }),
            ),
            Detection::State(AgentState::Unknown)
        );
        assert_eq!(
            detect(
                "",
                None,
                Some(CursorEvidence {
                    visible: true,
                    shape: 2,
                }),
            ),
            Detection::State(AgentState::Unknown)
        );
    }

    #[test]
    fn stronger_codex_signals_override_visible_cursor() {
        let cursor = Some(CursorEvidence {
            visible: true,
            shape: 0,
        });
        assert_eq!(
            detect("Allow command?", None, cursor),
            Detection::State(AgentState::Blocked)
        );
        assert_eq!(
            detect("", Some("⠹ Working"), cursor),
            Detection::State(AgentState::Working)
        );
    }

    #[test]
    fn recognizes_direct_and_runtime_wrapped_codex_programs() {
        assert!(codex_program_name(OsStr::new("codex")));
        assert!(codex_program_name(OsStr::new("/opt/openai/bin/codex.js")));
        assert!(codex_program_name(OsStr::new(".codex-wrapped")));
        assert!(!codex_program_name(OsStr::new("my-codex-helper")));
    }

    #[test]
    fn recognizes_non_interactive_invocations() {
        let args = |values: &[&str]| values.iter().map(OsString::from).collect::<Vec<_>>();

        assert!(codex_headless_invocation(&args(&[
            "codex", "e", "--yolo", "prompt"
        ])));
        assert!(codex_headless_invocation(&args(&[
            "codex", "--yolo", "exec", "prompt"
        ])));
        assert!(codex_headless_invocation(&args(&[
            "codex",
            "review",
            "--uncommitted"
        ])));
        assert!(!codex_headless_invocation(&args(&[
            "codex", "--yolo", "prompt"
        ])));
    }

    #[test]
    fn extracts_session_id_from_open_rollout_path() {
        let path = Path::new(
            "/home/me/.codex/sessions/2026/07/17/rollout-2026-07-17T11-12-23-019f7147-8F68-7F11-8ECA-FA67874BFEFD.jsonl",
        );
        assert_eq!(
            session_id_from_rollout_path(path).as_deref(),
            Some("019f7147-8f68-7f11-8eca-fa67874bfefd")
        );
        assert_eq!(
            session_id_from_rollout_path(Path::new("state_5.sqlite")),
            None
        );
        assert_eq!(
            session_id_from_rollout_path(Path::new("rollout-not-a-uuid.jsonl")),
            None
        );
    }
}
