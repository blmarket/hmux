//! Pi coding-agent detector.
//!
//! Pi keeps a stable `π - <cwd>` terminal title while both resting and working,
//! so lifecycle detection uses the current TUI content instead. A bordered
//! selector or input waiting for user action is blocked, the braille
//! `Working...` status is working, and the stable title is the idle fallback.

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use super::{is_braille, is_uuid, AgentDetector, AgentState, Detection, SessionIdSource};

/// Recognizes Pi coding-agent panes.
pub(crate) struct PiDetector;

impl AgentDetector for PiDetector {
    fn label(&self) -> &'static str {
        "pi"
    }

    fn matches_program(&self, program: &OsStr) -> bool {
        pi_program_name(program)
    }

    fn matches_invocation(&self, arguments: &[OsString]) -> bool {
        arguments
            .iter()
            .take(2)
            .any(|argument| pi_program_name(argument) || pi_runtime_script(argument))
    }

    fn invocation_state(&self, arguments: &[OsString]) -> Option<AgentState> {
        pi_headless_invocation(arguments).then_some(AgentState::Working)
    }

    fn session_id_source(&self) -> Option<SessionIdSource> {
        Some(SessionIdSource::AgentCwdTranscript)
    }

    fn session_dir_for_cwd(&self, cwd: &Path) -> Option<PathBuf> {
        session_dir(cwd)
    }

    fn session_id_from_file_name(&self, name: &OsStr) -> Option<String> {
        session_id_from_file_name(name)
    }

    fn detect(&self, screen: &str, title: Option<&str>) -> Detection {
        detect(screen, title)
    }
}

/// Pi stores default sessions below a cwd-derived directory in its agent home.
pub(crate) fn session_dir(cwd: &Path) -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(
        Path::new(&home)
            .join(".pi/agent/sessions")
            .join(project_slug(cwd)),
    )
}

fn project_slug(cwd: &Path) -> String {
    let path = cwd.to_string_lossy();
    let path = path.trim_start_matches(['/', '\\']);
    let encoded = path
        .chars()
        .map(|c| {
            if matches!(c, '/' | '\\' | ':') {
                '-'
            } else {
                c
            }
        })
        .collect::<String>();
    format!("--{encoded}--")
}

/// Default filenames are `<timestamp>_<session-id>.jsonl`. Accepting a bare
/// UUID stem also covers explicitly named session files without weakening UUID
/// validation.
fn session_id_from_file_name(name: &OsStr) -> Option<String> {
    let stem = name.to_str()?.strip_suffix(".jsonl")?;
    let candidate = stem.rsplit_once('_').map_or(stem, |(_, id)| id);
    is_uuid(candidate).then(|| candidate.to_ascii_lowercase())
}

fn detect(screen: &str, title: Option<&str>) -> Detection {
    let lines = screen.lines().collect::<Vec<_>>();
    let active_region = last_bordered_region(&lines).to_lowercase();

    // Extension selectors and inputs replace Pi's editor. These controls are
    // the point where an active turn waits for a human response, so they outrank
    // a working indicator that may still be visible above the dialog.
    if is_user_input_dialog(&active_region) {
        return Detection::State(AgentState::Blocked);
    }

    // Built-in pickers are user-invoked navigation rather than an agent turn
    // requesting input. Preserve the previous lifecycle state while they are
    // open, matching the treatment of transcript/model pickers in other TUIs.
    if is_transient_picker(&active_region) {
        return Detection::KeepPrevious;
    }

    if has_working_indicator(&lines) {
        return Detection::State(AgentState::Working);
    }

    if title.is_some_and(pi_idle_title) || has_pi_signature(screen) {
        return Detection::State(AgentState::Idle);
    }

    Detection::State(AgentState::Unknown)
}

fn is_user_input_dialog(region: &str) -> bool {
    (region.contains("navigate") && region.contains("select") && region.contains("cancel"))
        || (region.contains("submit") && region.contains("cancel"))
}

fn is_transient_picker(region: &str) -> bool {
    region.contains("resume session")
        || region.contains("select model")
        || region.contains("session tree")
}

fn has_working_indicator(lines: &[&str]) -> bool {
    lines.iter().rev().take(16).any(|line| {
        let trimmed = line.trim_start();
        let mut chars = trimmed.chars();
        matches!(chars.next(), Some(first) if is_braille(first))
            && chars
                .as_str()
                .trim_start()
                .to_lowercase()
                .starts_with("working")
    })
}

fn pi_idle_title(title: &str) -> bool {
    title == "π" || title.starts_with("π - ")
}

fn has_pi_signature(screen: &str) -> bool {
    screen.lines().any(|line| {
        let lower = line.trim_start().to_lowercase();
        lower == "pi" || lower.starts_with("pi v")
    })
}

fn last_bordered_region(lines: &[&str]) -> String {
    let borders = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| is_horizontal_rule(line).then_some(index))
        .collect::<Vec<_>>();
    match borders.as_slice() {
        [.., start, end] => lines[start + 1..*end].join("\n"),
        _ => String::new(),
    }
}

fn is_horizontal_rule(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.chars().count() >= 3
        && trimmed
            .chars()
            .all(|c| matches!(c, '\u{2500}' | '\u{2501}' | '\u{2550}' | '-'))
}

fn pi_program_name(program: &OsStr) -> bool {
    let name = Path::new(program)
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or_default()
        .trim_start_matches('.')
        .to_ascii_lowercase();
    matches!(
        name.as_str(),
        "pi" | "pi.exe" | "pi.cmd" | "pi.js" | "pi.mjs"
    )
}

fn pi_runtime_script(argument: &OsStr) -> bool {
    let path = argument.to_string_lossy().replace('\\', "/").to_lowercase();
    path.ends_with("/dist/cli.js") && path.contains("/pi-coding-agent/")
}

fn pi_headless_invocation(arguments: &[OsString]) -> bool {
    arguments
        .iter()
        .skip(1)
        .any(|argument| matches!(argument.to_str(), Some("--print" | "-p")))
        || arguments
            .windows(2)
            .any(|pair| pair[0] == "--mode" && matches!(pair[1].to_str(), Some("json")))
}

#[cfg(test)]
mod tests {
    use std::ffi::{OsStr, OsString};
    use std::path::Path;

    use super::{
        detect, pi_headless_invocation, pi_program_name, pi_runtime_script, project_slug,
        session_dir, session_id_from_file_name, AgentState, Detection,
    };

    #[test]
    fn resting_editor_reports_idle() {
        let screen = "pi v0.80.6\n────────\n \n────────\n~/work/project";
        assert_eq!(
            detect(screen, Some("π - project")),
            Detection::State(AgentState::Idle)
        );
    }

    #[test]
    fn working_indicator_outranks_static_title() {
        let screen = "previous output\n⠹ Working...\n────────\n────────\n~/work/project";
        assert_eq!(
            detect(screen, Some("π - project")),
            Detection::State(AgentState::Working)
        );
    }

    #[test]
    fn confirmation_dialog_outranks_working_indicator() {
        let screen = "⠹ Working...\n────────\nAllow command?\n→ Yes\n  No\n↑↓ navigate  enter select  esc cancel\n────────";
        assert_eq!(
            detect(screen, Some("π - project")),
            Detection::State(AgentState::Blocked)
        );
    }

    #[test]
    fn extension_input_reports_blocked() {
        let screen = "────────\nEnter a value\n\nenter submit  esc cancel\n────────";
        assert_eq!(
            detect(screen, Some("π - project")),
            Detection::State(AgentState::Blocked)
        );
    }

    #[test]
    fn session_picker_preserves_previous_state() {
        let screen =
            "────────\nResume Session (Current Folder)\nctrl+s sort · ctrl+n named\n────────";
        assert_eq!(detect(screen, Some("π - project")), Detection::KeepPrevious);
    }

    #[test]
    fn recognizes_direct_and_runtime_wrapped_pi() {
        assert!(pi_program_name(OsStr::new("pi")));
        assert!(pi_program_name(OsStr::new("/usr/local/bin/pi.exe")));
        assert!(!pi_program_name(OsStr::new("pilot")));
        assert!(pi_runtime_script(OsStr::new(
            "/opt/node_modules/@earendil-works/pi-coding-agent/dist/cli.js"
        )));
        assert!(!pi_runtime_script(OsStr::new("/opt/other/dist/cli.js")));
    }

    #[test]
    fn recognizes_headless_invocations() {
        assert!(pi_headless_invocation(&[
            OsString::from("pi"),
            OsString::from("--print"),
            OsString::from("do work"),
        ]));
        assert!(pi_headless_invocation(&[
            OsString::from("pi"),
            OsString::from("--mode"),
            OsString::from("json"),
            OsString::from("do work"),
        ]));
        assert!(!pi_headless_invocation(&[
            OsString::from("pi"),
            OsString::from("do work"),
        ]));
    }

    #[test]
    fn cwd_maps_to_pi_session_directory() {
        assert_eq!(
            project_slug(Path::new("/home/hun/srv/pi")),
            "--home-hun-srv-pi--"
        );
        let home = std::env::var_os("HOME").expect("HOME set in test environment");
        assert_eq!(
            session_dir(Path::new("/work/project")),
            Some(Path::new(&home).join(".pi/agent/sessions/--work-project--"))
        );
    }

    #[test]
    fn extracts_session_id_from_timestamped_file_name() {
        assert_eq!(
            session_id_from_file_name(OsStr::new(
                "2026-07-25T03-35-20-915Z_019F9757-7C53-7898-A7D1-C8C780212888.jsonl"
            ))
            .as_deref(),
            Some("019f9757-7c53-7898-a7d1-c8c780212888")
        );
        assert_eq!(
            session_id_from_file_name(OsStr::new("not-a-session.jsonl")),
            None
        );
    }
}
