//! The agent plugin: which coding agent runs in a pane, and what it is doing.
//!
//! The detection itself is not here. It lives in `hmux-agent`, behind the same
//! pane observability contract the hmux daemon hosts it through, so both
//! servers classify panes with one implementation rather than two that drift.
//! What this module is, is the wiring: a 200 ms tick, a status hub read back
//! by the six format variables of `PROTOCOL.md`, and the status-line redraw a
//! changed pane asks for.

use crate::WindowPane;
use std::ffi::CString;
use std::time::Duration;

use hmux_agent::integration::AgentObserver;
use hmux_agent::integration::status::{AgentStatus, PaneAgents, StatusHub};
use hmux_agent::pane_class::{PaneClass, PaneProcessProbe, stringify_argv};

use crate::spawn::PANE_STATUSREADY;
use crate::window::window_pane_find_by_id;

use super::{Host, PaneId, Plugin};

/// The variables this plugin answers for, in the spelling `PROTOCOL.md` fixes.
const VARIABLES: &[&str] = &[
    "pane_agent",
    "pane_agent_state",
    "pane_agent_pid",
    "pane_agent_session_id",
    "pane_agent_model",
    "pane_state_emoji",
];

/// The agent observer, its published status, and the copy of that status the
/// last redraw was issued for.
pub struct AgentPlugin {
    observer: AgentObserver,
    hub: StatusHub,
    published: PaneAgents,
}

impl AgentPlugin {
    pub fn new() -> Self {
        let hub = StatusHub::new();
        AgentPlugin {
            observer: AgentObserver::new(hub.clone()),
            hub,
            published: PaneAgents::new(),
        }
    }

    /// The status published for a pane, if the observer found an agent in it.
    fn status(&self, pane: PaneId) -> Option<&AgentStatus> {
        self.published.get(&pane)
    }
}

impl Default for AgentPlugin {
    fn default() -> Self {
        AgentPlugin::new()
    }
}

impl Plugin for AgentPlugin {
    fn name(&self) -> &'static str {
        "agent"
    }

    fn variables(&self) -> &'static [&'static str] {
        VARIABLES
    }

    fn interval(&self) -> Option<Duration> {
        Some(AgentObserver::INTERVAL)
    }

    fn tick(&mut self, host: &dyn Host) {
        self.observer.tick(host);
        let snapshot = self.hub.snapshot().panes;
        // Redraw only what moved: every pane whose status appeared, changed or
        // went away is one whose window is now drawing something stale.
        for (pane, status) in &snapshot {
            if self.published.get(pane) != Some(status) {
                host.invalidate(*pane);
            }
        }
        for pane in self.published.keys() {
            if !snapshot.contains_key(pane) {
                host.invalidate(*pane);
            }
        }
        self.published = snapshot;
    }

    fn resolve(&self, pane: PaneId, key: &str) -> Option<CString> {
        let status = self.status(pane);
        // A pane the observer has nothing for reads as empty metadata in a
        // "none" state — the same answer a pane with no agent in it gives, so
        // a format need not tell the two apart.
        match key {
            "pane_agent" => Some(
                CString::new(status.map_or("", |status| status.agent))
                    .expect("an agent name has no NUL"),
            ),
            "pane_agent_state" => Some(
                CString::new(status.map_or("none", |status| status.state.wire_str()))
                    .expect("an agent state has no NUL"),
            ),
            "pane_agent_pid" => Some(
                status
                    .and_then(|status| status.pid)
                    .map(|pid| CString::new(pid.to_string()).expect("a pid has no NUL"))
                    .unwrap_or_default(),
            ),
            "pane_agent_session_id" => Some(
                status
                    .and_then(|status| status.session_id.clone())
                    .unwrap_or_default(),
            ),
            "pane_agent_model" => Some(
                status
                    .and_then(|status| status.model.clone())
                    .unwrap_or_default(),
            ),
            "pane_state_emoji" => Some(state_emoji(pane, status)),
            _ => None,
        }
    }
}

/// The compact glyph every pane gets.
///
/// An agent pane reports its lifecycle state; any other pane reports what it
/// is running, so this is never empty. The agent's *label* is what decides
/// which half applies, not its emoji: the observer reports a state for every
/// pane it watches — an ordinary shell that exits is `exited` just as an agent
/// is — and only a pane that named an agent should be labelled as one.
fn state_emoji(pane: PaneId, status: Option<&AgentStatus>) -> CString {
    if let Some(status) = status
        && !status.agent.is_empty()
        && !status.state.emoji().is_empty()
    {
        return CString::new(status.state.emoji()).expect("an emoji has no NUL");
    }
    let Some(pane) = window_pane_find_by_id(pane.0) else {
        return CString::new(PaneClass::Dead.emoji()).expect("an emoji has no NUL");
    };
    unsafe {
        let wp = pane.as_pane();
        let dead = !(*wp).process_active() && *(*wp).flags() & PANE_STATUSREADY != 0;
        let alternate_on = (*wp).base().is_alternate();
        let emoji = PaneClass::classify(pane_probe(wp).as_ref(), alternate_on, dead).emoji();
        CString::new(emoji).expect("an emoji has no NUL")
    }
}

/// What the pane's pty says about the process holding it: the foreground group
/// and the session leader, with the pane's own command line as the fallback
/// for a group whose leader has already exited.
fn pane_probe(wp: &(impl crate::WindowPane + ?Sized)) -> Option<PaneProcessProbe> {
    let (foreground, session_leader) = wp.process_groups()?;
    let command = wp.pane_command();
    let fallback = match command.argv.is_empty() {
        true => command.shell,
        false => Some(stringify_argv(&command.argv)),
    };
    Some(PaneProcessProbe::new(
        foreground,
        session_leader,
        fallback,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    use hmux_agent::integration::AgentState;

    use crate::tests::test_fixtures::globals;
    use crate::text::{RustUtf8VisModel, Utf8VisModel};
    use crate::types::u_int;

    /// Every glyph the status bar can put in a pane's slot has to occupy the
    /// same two columns, or window entries stop lining up as panes change
    /// state. Two things can break that, and this covers both: a glyph the
    /// width table calls one column, and a glyph that only reaches emoji
    /// presentation through U+FE0F — which the renderer measures as zero,
    /// leaving the cell one column wide while the terminal draws two.
    #[test]
    fn every_pane_state_glyph_is_exactly_two_columns() {
        let _guard = globals();
        let classes = [
            PaneClass::Dead,
            PaneClass::Tui,
            PaneClass::ShellPrompt,
            PaneClass::WaitingForTty,
            PaneClass::Running,
        ]
        .map(PaneClass::emoji);
        let agents = [
            AgentState::Idle,
            AgentState::Working,
            AgentState::Blocked,
            AgentState::Exited,
        ]
        .map(AgentState::emoji);

        for emoji in classes.iter().chain(agents.iter()) {
            let mut codepoints = emoji.chars();
            codepoints.next().expect("a glyph");
            assert_eq!(
                codepoints.next(),
                None,
                "{emoji:?} is more than one codepoint"
            );
            let owned = std::ffi::CString::new(*emoji).expect("a glyph has no NUL");
            assert_eq!(
                RustUtf8VisModel.width(&owned),
                2,
                "{emoji:?} is not 2 columns"
            );
        }

        // Unknown stays empty on purpose: it is what lets the state emoji fall
        // through to the pane's own class instead of labelling it as an agent.
        assert_eq!(AgentState::Unknown.emoji(), "");
    }

    /// `pane_state_emoji` is the one variable that reaches past [`Host`] into
    /// the server's own pane tree, for the half of it that reports what a
    /// non-agent pane is running. A pane the server has never heard of is
    /// dead, and that lookup is what needs the guard here; the rest of the
    /// contract is covered through the trait alone.
    #[test]
    fn a_pane_the_server_does_not_have_is_dead() {
        let _guard = globals();
        let plugin = AgentPlugin::new();

        assert_eq!(
            plugin
                .resolve(PaneId(u_int::MAX), "pane_state_emoji")
                .as_deref(),
            Some(c"🛑")
        );
    }

    /// The variables the plugin claims are exactly the ones it answers for; a
    /// name in one list and not the other would expand to nothing at all.
    #[test]
    fn every_claimed_variable_is_answered() {
        let _guard = globals();
        let plugin = AgentPlugin::new();
        for key in plugin.variables() {
            assert!(
                plugin.resolve(PaneId(u_int::MAX), key).is_some(),
                "{key} is claimed but not answered"
            );
        }
    }
}
