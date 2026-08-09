//! Claude Code agent detector.
//!
//! A compact port of Herdr's Claude detector (`claude.toml`), using the same
//! rule priorities. Where Herdr parses structured regions (the bordered prompt
//! box, the last horizontal rule), this prototype approximates them from the
//! plain-text screen tail plus the window title. Signals, high priority first:
//!
//! - a braille working spinner in the title (OSC 0/2) → working;
//! - the detailed-transcript viewer → keep previous;
//! - a live selection form / dynamic-workflow prompt → blocked;
//! - a live prompt box (`❯`) with no pending question → idle;
//! - the model picker → keep previous;
//! - bash / generic permission prompts → blocked;
//! - legacy permission phrasings → blocked;
//! - a resting `✳ ` title → idle.
//!
//! Herdr's `osc_progress` idle rule has no analogue here: the detector reads
//! only the screen tail and the window title, not OSC 9;4 progress reports.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use super::{
    is_uuid, title_working_spinner, AgentDetector, AgentState, Detection, SessionEnvStamp,
    SessionIdSource,
};

/// A resting-title marker: Claude sets `✳ …` (U+2733) at its idle prompt.
const IDLE_TITLE_MARK: char = '\u{2733}';

/// Recognizes Anthropic Claude Code panes.
pub(crate) struct ClaudeDetector;

impl AgentDetector for ClaudeDetector {
    fn label(&self) -> &'static str {
        "claude"
    }

    fn matches_program(&self, program: &OsStr) -> bool {
        claude_program_name(program)
    }

    fn session_id_source(&self) -> Option<SessionIdSource> {
        Some(SessionIdSource::AgentCwdTranscript)
    }

    fn session_dir_for_cwd(&self, cwd: &Path) -> Option<PathBuf> {
        transcript_dir(cwd)
    }

    fn session_id_from_file_name(&self, name: &OsStr) -> Option<String> {
        session_id_from_transcript_name(name)
    }

    fn session_env_stamp(&self) -> Option<SessionEnvStamp> {
        Some(SessionEnvStamp {
            session_id: "CLAUDE_CODE_SESSION_ID",
            owner_pid: "CLAUDE_PID",
        })
    }

    fn session_file_for_id(&self, cwd: &Path, session_id: &str) -> Option<PathBuf> {
        is_uuid(session_id)
            .then(|| transcript_dir(cwd))?
            .map(|dir| dir.join(format!("{}.jsonl", session_id.to_ascii_lowercase())))
    }

    fn detect(&self, screen: &str, title: Option<&str>) -> Detection {
        detect(screen, title)
    }
}

/// Claude Code records each session's transcript at
/// `~/.claude/projects/<slug>/<session-id>.jsonl`, where `<slug>` is the
/// project's working directory with every non-alphanumeric character replaced by
/// `-`. The active session is the most recently modified transcript in that
/// directory, so attribution reads the agent's cwd and lists this directory.
///
/// The directory is shared by every agent run from the same project, and Claude
/// Code does not create a transcript until its first turn completes, so "newest"
/// alone can name a neighbour's session during that window. Candidates are
/// therefore dated against the agent's start time before being considered.
pub(crate) fn transcript_dir(cwd: &Path) -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(
        Path::new(&home)
            .join(".claude/projects")
            .join(project_slug(cwd)),
    )
}

/// Encode a working directory the way Claude Code names its project directory:
/// every character that is not an ASCII letter or digit becomes `-`.
fn project_slug(cwd: &Path) -> String {
    cwd.to_string_lossy()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect()
}

/// A transcript file is `<session-id>.jsonl`; the stem is the session UUID.
fn session_id_from_transcript_name(name: &OsStr) -> Option<String> {
    let stem = name.to_str()?.strip_suffix(".jsonl")?;
    is_uuid(stem).then(|| stem.to_ascii_lowercase())
}

fn detect(screen: &str, title: Option<&str>) -> Detection {
    // 1100 — a braille spinner in the title means a turn is in progress.
    if let Some(title) = title {
        if title_working_spinner(title) {
            return Detection::State(AgentState::Working);
        }
    }

    let lines = screen.lines().collect::<Vec<_>>();
    let lower = screen.to_lowercase();

    // 1000 — the detailed-transcript viewer is transient UI; keep prior state.
    let bottom = bottom_non_empty_lines(&lines, 3).to_lowercase();
    if bottom.contains("showing detailed transcript")
        && (contains_all(&bottom, &["ctrl+o", "to toggle"])
            || contains_all(&bottom, &["ctrl+e", "show all"])
            || contains_all(&bottom, &["ctrl+e", "collapse"])
            || bottom.contains("↑↓ scroll")
            || bottom.contains("? for shortcuts"))
    {
        return Detection::KeepPrevious;
    }

    let after_rule = after_last_horizontal_rule(&lines).to_lowercase();

    // 980 — a live selection form (arrow-key navigable, enter/esc) is blocking.
    if after_rule.contains("enter to select")
        && after_rule.contains("esc to cancel")
        && [
            "tab/arrow keys to navigate",
            "arrow keys to navigate",
            "arrows to navigate",
            "↑/↓ to navigate",
            "↑↓ to navigate",
        ]
        .iter()
        .any(|hint| after_rule.contains(hint))
    {
        return Detection::State(AgentState::Blocked);
    }

    // 980 — the dynamic-workflow confirmation prompt.
    if lower.contains("run a dynamic workflow?") && lower.contains("esc to cancel") {
        return Detection::State(AgentState::Blocked);
    }

    // 950 — a live prompt box (`❯`) with no pending question means idle. This
    // outranks the permission/model rules below, so its exclusions must hold.
    if has_prompt_marker(&lines)
        && !lower.contains("enter to select")
        && !lower.contains("esc to cancel")
        && !lower.contains("tab/arrow keys")
        && !lower.contains("arrow keys to navigate")
        && !lower.contains("↑/↓ to navigate")
    {
        return Detection::State(AgentState::Idle);
    }

    // 900 — the model picker is a transient menu, not a lifecycle state.
    if lower.contains("select model")
        && lower.contains("enter to set as default")
        && lower.contains("esc to cancel")
        && !lower.contains("do you want to proceed?")
        && !lower.contains("enter to select")
    {
        return Detection::KeepPrevious;
    }

    // 850 — a bash-command permission prompt.
    if lower.contains("do you want to proceed?")
        && [
            "bash command",
            "bash(",
            "contains expansion",
            "tab to amend",
            "ctrl+e to explain",
        ]
        .iter()
        .any(|hint| lower.contains(hint))
        && (has_choice_line(&lines, "yes") || has_choice_line(&lines, "no"))
    {
        return Detection::State(AgentState::Blocked);
    }

    // 840 — a generic permission prompt with numbered yes/no options.
    if after_rule.contains("do you want to proceed?")
        && after_rule.contains("esc to cancel")
        && (has_choice_line(&lines, "yes") || has_choice_line(&lines, "no"))
    {
        return Detection::State(AgentState::Blocked);
    }

    // 300 — legacy permission phrasings, unless the pane is at a bare `❯`.
    if !has_lone_prompt_line(&lines) && has_legacy_blocker(&lower, screen) {
        return Detection::State(AgentState::Blocked);
    }

    // 250 — a resting `✳ ` title is the last-resort idle signal.
    if let Some(title) = title {
        let mut chars = title.chars();
        if chars.next() == Some(IDLE_TITLE_MARK) && chars.next() == Some(' ') {
            return Detection::State(AgentState::Idle);
        }
    }

    Detection::State(AgentState::Unknown)
}

fn has_legacy_blocker(lower: &str, screen: &str) -> bool {
    let wants_confirmation =
        |verb: &str| lower.contains(verb) && (lower.contains("yes") || screen.contains('❯'));
    wants_confirmation("do you want to")
        || wants_confirmation("would you like to")
        || lower.contains("waiting for permission")
        || lower.contains("do you want to allow this connection?")
        || lower.contains("tab to amend")
        || lower.contains("ctrl+e to explain")
        || (lower.contains("do you want to proceed?") && lower.contains("esc to cancel"))
        || lower.contains("review your answers")
        || lower.contains("skip interview and plan immediately")
}

/// The last `n` non-empty screen lines joined with newlines (Herdr's
/// `bottom_non_empty_lines(n)` region).
fn bottom_non_empty_lines(lines: &[&str], n: usize) -> String {
    let mut tail = lines
        .iter()
        .rev()
        .filter(|line| !line.trim().is_empty())
        .take(n)
        .copied()
        .collect::<Vec<_>>();
    tail.reverse();
    tail.join("\n")
}

/// The text after the last horizontal-rule line (Herdr's
/// `after_last_horizontal_rule` region), or the whole screen if there is none.
fn after_last_horizontal_rule(lines: &[&str]) -> String {
    match lines.iter().rposition(|line| is_horizontal_rule(line)) {
        Some(index) => lines[index + 1..].join("\n"),
        None => lines.join("\n"),
    }
}

/// A box-drawing horizontal divider (e.g. Claude's `────` separators): a
/// non-empty line made entirely of horizontal rule glyphs, at least 3 wide.
fn is_horizontal_rule(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.chars().count() >= 3
        && trimmed
            .chars()
            .all(|c| matches!(c, '\u{2500}' | '\u{2501}' | '\u{2550}' | '-' | '—'))
}

/// Whether any line is the live *input* prompt (`❯`), optionally indented.
///
/// A selected menu option such as `❯ 1. Yes` also begins with `❯`; Herdr
/// distinguishes these by region (the input box vs. an option list). Lacking
/// regions here, we exclude lines whose `❯` is followed by a yes/no or numbered
/// choice, so a permission prompt is not mistaken for a resting input box.
fn has_prompt_marker(lines: &[&str]) -> bool {
    lines.iter().any(|line| {
        let rest = line.trim_start();
        rest.starts_with('❯') && !looks_like_choice(rest.trim_start_matches('❯').trim_start())
    })
}

/// Whether `text` looks like a menu choice: a `yes`/`no` option or a `N.` item.
fn looks_like_choice(text: &str) -> bool {
    let lower = text.trim_start().to_lowercase();
    lower.starts_with("yes") || lower.starts_with("no") || {
        let after_digits = lower.trim_start_matches(|c: char| c.is_ascii_digit());
        after_digits.len() < lower.len() && after_digits.trim_start().starts_with('.')
    }
}

/// Whether any line is a bare `❯` prompt (Herdr's lone-prompt `not` guard).
fn has_lone_prompt_line(lines: &[&str]) -> bool {
    lines.iter().any(|line| line.trim() == "❯")
}

/// Whether any line is a numbered/`❯`-prefixed choice for `keyword`
/// (e.g. `❯ 1. Yes`, `2. No`) — an approximation of Herdr's option `line_regex`.
fn has_choice_line(lines: &[&str], keyword: &str) -> bool {
    lines.iter().any(|line| {
        let rest = line.trim_start().to_lowercase();
        let rest = rest.trim_start_matches('❯').trim_start();
        let rest = strip_leading_number(rest);
        rest.trim_start().starts_with(keyword)
    })
}

/// Strip a leading `N.` list marker (e.g. `"1. yes"` → `" yes"`).
fn strip_leading_number(text: &str) -> &str {
    let digits = text.trim_start();
    let after_digits = digits.trim_start_matches(|c: char| c.is_ascii_digit());
    if after_digits.len() < digits.len() {
        after_digits.strip_prefix('.').unwrap_or(after_digits)
    } else {
        text
    }
}

fn contains_all(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().all(|needle| haystack.contains(needle))
}

fn claude_program_name(program: &OsStr) -> bool {
    let name = Path::new(program)
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or_default()
        .trim_start_matches('.')
        .to_ascii_lowercase();
    matches!(
        name.as_str(),
        "claude" | "claude-code" | "claude.exe" | "claude.cmd" | "claude.js" | "claude.mjs"
    )
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;
    use std::path::Path;

    use super::{
        claude_program_name, detect, project_slug, session_id_from_transcript_name, transcript_dir,
        AgentState, Detection,
    };

    #[test]
    fn title_spinner_reports_working() {
        // The braille spinner in the title outranks a resting prompt box.
        let screen = "───────\n❯ ";
        assert_eq!(
            detect(screen, Some("⠹ Claude")),
            Detection::State(AgentState::Working)
        );
    }

    #[test]
    fn live_prompt_box_reports_idle() {
        let screen = "some earlier output\n─────────────\n❯ ";
        assert_eq!(detect(screen, None), Detection::State(AgentState::Idle));
    }

    #[test]
    fn selection_form_reports_blocked() {
        let screen = "\
Do you want to make this edit?
─────────────────────────
❯ 1. Yes
  2. No
Enter to select · Esc to cancel · ↑/↓ to navigate";
        assert_eq!(detect(screen, None), Detection::State(AgentState::Blocked));
    }

    #[test]
    fn bash_permission_prompt_reports_blocked() {
        let screen = "\
Bash command
  rm -rf build/
Do you want to proceed?
❯ 1. Yes
  2. No";
        assert_eq!(detect(screen, None), Detection::State(AgentState::Blocked));
    }

    #[test]
    fn generic_permission_prompt_reports_blocked() {
        let screen = "\
Edit file src/main.rs?
──────────────────────
❯ 1. Yes
  2. No
Do you want to proceed? · Esc to cancel";
        assert_eq!(detect(screen, None), Detection::State(AgentState::Blocked));
    }

    #[test]
    fn transcript_viewer_preserves_previous_state() {
        let screen = "Showing detailed transcript · Ctrl+O to toggle";
        assert_eq!(detect(screen, None), Detection::KeepPrevious);
    }

    #[test]
    fn model_picker_is_transient() {
        let screen = "Select model\n  Sonnet\n  Opus\nEnter to set as default · Esc to cancel";
        assert_eq!(detect(screen, None), Detection::KeepPrevious);
    }

    #[test]
    fn resting_title_is_last_resort_idle() {
        assert_eq!(
            detect("", Some("✳ my-project")),
            Detection::State(AgentState::Idle)
        );
        assert_eq!(detect("", None), Detection::State(AgentState::Unknown));
    }

    #[test]
    fn recognizes_direct_and_wrapped_claude_programs() {
        assert!(claude_program_name(OsStr::new("claude")));
        assert!(claude_program_name(OsStr::new(
            "/usr/local/bin/claude-code"
        )));
        assert!(claude_program_name(OsStr::new(".claude.js")));
        assert!(!claude_program_name(OsStr::new("claude-helper")));
    }

    #[test]
    fn project_slug_replaces_non_alphanumeric_with_dash() {
        assert_eq!(
            project_slug(Path::new("/home/hun/srv/hmux")),
            "-home-hun-srv-hmux"
        );
        // A leading dot after a separator yields a double dash, matching Claude.
        assert_eq!(
            project_slug(Path::new("/home/hun/.claude")),
            "-home-hun--claude"
        );
        // Case is preserved; only non-alphanumerics are rewritten.
        assert_eq!(project_slug(Path::new("/home/hun/AAI")), "-home-hun-AAI");
    }

    #[test]
    fn extracts_session_id_from_transcript_name() {
        assert_eq!(
            session_id_from_transcript_name(OsStr::new(
                "713EE853-A358-4DC4-A306-90F1F2F4070D.jsonl"
            ))
            .as_deref(),
            Some("713ee853-a358-4dc4-a306-90f1f2f4070d")
        );
        assert_eq!(
            session_id_from_transcript_name(OsStr::new("not-a-uuid.jsonl")),
            None
        );
        assert_eq!(
            session_id_from_transcript_name(OsStr::new(
                "713ee853-a358-4dc4-a306-90f1f2f4070d.json"
            )),
            None
        );
    }

    #[test]
    fn transcript_dir_is_under_the_home_projects_directory() {
        let home = std::env::var_os("HOME").expect("HOME set in test environment");
        assert_eq!(
            transcript_dir(Path::new("/proj/app")),
            Some(Path::new(&home).join(".claude/projects/-proj-app"))
        );
    }
}
