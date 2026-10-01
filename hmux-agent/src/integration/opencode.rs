//! OpenCode agent detector.
//!
//! OpenCode keeps its lifecycle state in the TUI rather than its window title.
//! A visible interrupt hint or progress bar means that a turn is running, and
//! its permission/question controls mean that the turn is waiting for input.
//! The footer identifies a settled TUI prompt as idle.

use std::ffi::{OsStr, OsString};

use super::{AgentDetector, AgentState, Detection, SessionIdSource};

const LIVE_ROWS: usize = 24;

/// Recognizes OpenCode panes.
pub(crate) struct OpencodeDetector;

impl AgentDetector for OpencodeDetector {
    fn label(&self) -> &'static str {
        "opencode"
    }

    fn matches_program(&self, program: &OsStr) -> bool {
        opencode_program_name(program)
    }

    fn invocation_state(&self, arguments: &[OsString]) -> Option<AgentState> {
        opencode_headless_invocation(arguments).then_some(AgentState::Working)
    }

    fn session_id_source(&self) -> Option<SessionIdSource> {
        None
    }

    fn detect(&self, screen: &str, _title: Option<&str>) -> Detection {
        detect(screen)
    }
}

fn detect(screen: &str) -> Detection {
    let lower = screen.to_ascii_lowercase();

    if is_permission_prompt(&lower) {
        return Detection::State(AgentState::Blocked);
    }

    if has_interrupt_hint(&lower) || has_progress_bar(screen) {
        return Detection::State(AgentState::Working);
    }

    if has_idle_signature(&lower) {
        return Detection::State(AgentState::Idle);
    }

    Detection::State(AgentState::Unknown)
}

fn is_permission_prompt(text: &str) -> bool {
    if text.contains("△ permission required") {
        return true;
    }

    text.contains("esc dismiss")
        && (text.contains("enter confirm")
            || text.contains("enter submit")
            || text.contains("enter toggle"))
        && (text.contains("↑↓ select") || text.contains("⇆ tab"))
}

fn has_interrupt_hint(text: &str) -> bool {
    text.lines().rev().take(LIVE_ROWS).any(|line| {
        line.contains("esc to interrupt")
            || line.contains("ctrl+c to interrupt")
            || line.contains("esc interrupt")
            || line.contains("esc again to interrupt")
            || (line.contains("opencode") && line.contains("esc") && line.contains("interrupt"))
    })
}

fn has_progress_bar(text: &str) -> bool {
    let mut run = 0;
    for character in text.chars() {
        if matches!(character, '■' | '⬝') {
            run += 1;
            if run >= 4 {
                return true;
            }
        } else {
            run = 0;
        }
    }
    false
}

fn has_idle_signature(text: &str) -> bool {
    text.lines()
        .any(|line| line.contains("opencode ") && (line.contains('•') || line.contains('·')))
}

fn opencode_program_name(program: &OsStr) -> bool {
    let name = program
        .to_string_lossy()
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or_default()
        .trim_start_matches('.')
        .to_ascii_lowercase();
    matches!(
        name.as_str(),
        "opencode"
            | "opencode.exe"
            | "opencode.cmd"
            | "opencode.js"
            | "opencode.mjs"
            | "opencode-wrapped"
            | "opencode-wrapp"
    )
}

fn opencode_headless_invocation(arguments: &[OsString]) -> bool {
    let mut takes_value = false;
    for argument in arguments.iter().skip(1) {
        let Some(argument) = argument.to_str() else {
            continue;
        };

        if takes_value {
            takes_value = false;
            continue;
        }

        if argument == "--" {
            return false;
        }

        if argument.starts_with('-') {
            takes_value = matches!(
                argument,
                "--agent"
                    | "-a"
                    | "--attach"
                    | "--config"
                    | "--dir"
                    | "--format"
                    | "--hostname"
                    | "--log-level"
                    | "--model"
                    | "-m"
                    | "--port"
                    | "--prompt"
                    | "--session"
                    | "-s"
            );
            continue;
        }

        return argument == "run";
    }
    false
}

#[cfg(test)]
mod tests {
    use std::ffi::{OsStr, OsString};

    use super::{
        AgentState, Detection, detect, has_idle_signature, opencode_headless_invocation,
        opencode_program_name,
    };

    #[test]
    fn recognizes_direct_and_wrapped_opencode_programs() {
        assert!(opencode_program_name(OsStr::new("opencode")));
        assert!(opencode_program_name(OsStr::new(
            "/run/current-system/sw/bin/opencode"
        )));
        assert!(opencode_program_name(OsStr::new(".opencode-wrapp")));
        assert!(opencode_program_name(OsStr::new(
            r"C:\\Users\\me\\opencode.cmd"
        )));
        assert!(!opencode_program_name(OsStr::new("opencoder")));
        assert!(!opencode_program_name(OsStr::new("opencode2")));
    }

    #[test]
    fn recognizes_the_noninteractive_run_command() {
        assert!(opencode_headless_invocation(&[
            OsString::from("opencode"),
            OsString::from("--yolo"),
            OsString::from("run"),
            OsString::from("fix the bug"),
        ]));
        assert!(!opencode_headless_invocation(&[
            OsString::from("opencode"),
            OsString::from("--prompt"),
            OsString::from("run"),
        ]));
        assert!(!opencode_headless_invocation(&[
            OsString::from("opencode"),
            OsString::from("--yolo"),
        ]));
    }

    #[test]
    fn permission_prompt_reports_blocked() {
        assert_eq!(
            detect("△ Permission required\nShell command\nenter once  esc dismiss"),
            Detection::State(AgentState::Blocked)
        );
        assert_eq!(
            detect("↑↓ select   enter submit   esc dismiss"),
            Detection::State(AgentState::Blocked)
        );
        assert_eq!(
            detect("old output: permission required"),
            Detection::State(AgentState::Unknown)
        );
    }

    #[test]
    fn interrupt_hints_report_working() {
        for screen in [
            "esc to interrupt",
            "ctrl+c to interrupt",
            "OpenCode · esc interrupt",
            "esc again to interrupt",
        ] {
            assert_eq!(
                detect(screen),
                Detection::State(AgentState::Working),
                "screen: {screen}"
            );
        }
    }

    #[test]
    fn progress_bar_reports_working() {
        assert_eq!(
            detect("building ■⬝■■ now"),
            Detection::State(AgentState::Working)
        );
    }

    #[test]
    fn footer_reports_idle() {
        let screen = "done\nctrl+p commands  • OpenCode 1.18.23";
        assert!(has_idle_signature(&screen.to_ascii_lowercase()));
        assert_eq!(detect(screen), Detection::State(AgentState::Idle));
    }

    #[test]
    fn unrelated_screen_is_unknown() {
        assert_eq!(
            detect("OpenCode is ready"),
            Detection::State(AgentState::Unknown)
        );
    }
}
