//! The [`Plugin`] contract, exercised only through the trait.
//!
//! The plugin is instantiated by its concrete type and then driven entirely
//! through [`Plugin`], with everything it is told about the server arriving
//! through a [`Host`] double rather than a running one. That is the whole
//! point of the contract — a plugin reads panes through [`Host`] and holds no
//! `*mut window_pane` — so none of this needs a server, and none of it needs
//! the process-wide state a server keeps in statics.

use ::std::cell::RefCell;
use ::std::io;
use ::std::rc::Rc;
use ::std::time::Duration;

use hmux_agent::observability::v1::{
    PaneObservability, PaneProcess, ScreenSource, ScreenTail, ServerObservability,
};

use crate::plugin::agent::AgentPlugin;
use crate::plugin::{Event, Host, PaneId, Plugin};

/// What a [`Host`] double was asked for, so a test can assert the plugin
/// reached the contract rather than around it.
#[derive(Default)]
struct Asked {
    pane_ids: usize,
    resolved: Vec<PaneId>,
    invalidated: Vec<PaneId>,
}

/// A server that has whatever panes a test gives it and nothing else.
struct FakeHost {
    panes: Vec<PaneId>,
    asked: RefCell<Asked>,
}

impl FakeHost {
    fn with_panes(panes: &[u32]) -> Self {
        FakeHost {
            panes: panes.iter().copied().map(PaneId).collect(),
            asked: RefCell::new(Asked::default()),
        }
    }

    fn asked(&self) -> ::std::cell::Ref<'_, Asked> {
        self.asked.borrow()
    }
}

impl ServerObservability for FakeHost {
    fn pane_ids(&self) -> io::Result<Vec<PaneId>> {
        self.asked.borrow_mut().pane_ids += 1;
        Ok(self.panes.clone())
    }

    fn resolve_pane(&self, id: PaneId) -> io::Result<Option<Rc<dyn PaneObservability>>> {
        self.asked.borrow_mut().resolved.push(id);
        if !self.panes.contains(&id) {
            return Ok(None);
        }
        Ok(Some(Rc::new(FakePane)))
    }
}

impl Host for FakeHost {
    fn invalidate(&self, pane: PaneId) {
        self.asked.borrow_mut().invalidated.push(pane);
    }
}

/// A pane with nothing running in it and nothing on screen.
struct FakePane;

impl PaneObservability for FakePane {
    fn process(&self) -> io::Result<PaneProcess> {
        Ok(PaneProcess {
            child_pid: None,
            exited: true,
        })
    }

    fn output_revision(&self) -> io::Result<u64> {
        Ok(1)
    }

    fn screen(&self, _source: ScreenSource, _lines: usize) -> io::Result<ScreenTail> {
        Ok(ScreenTail {
            revision: 1,
            text: String::new(),
            cursor_visible: true,
            cursor_shape: 0,
        })
    }

    fn scrollback_rows(&self) -> io::Result<usize> {
        Ok(0)
    }

    fn title(&self) -> io::Result<Option<String>> {
        Ok(None)
    }
}

/// A pane id no server would have handed out, so nothing is published for it.
const UNKNOWN: PaneId = PaneId(u32::MAX);

/// The name a plugin is enabled by is fixed, and it is not empty.
#[test]
fn a_plugin_answers_to_a_name() {
    fn name(plugin: &impl Plugin) -> &'static str {
        plugin.name()
    }

    let plugin = AgentPlugin::new();
    assert!(!name(&plugin).is_empty());
    assert_eq!(name(&plugin), name(&AgentPlugin::new()), "fixed per plugin");
}

/// The variable list is checked once at registration, so it has to be stable
/// and free of duplicates for a key to route to one place.
#[test]
fn the_variable_list_is_stable_and_has_no_duplicates() {
    fn variables(plugin: &impl Plugin) -> &'static [&'static str] {
        plugin.variables()
    }

    let plugin = AgentPlugin::new();
    let claimed = variables(&plugin);
    assert!(!claimed.is_empty());

    let mut sorted = claimed.to_vec();
    sorted.sort_unstable();
    let mut deduped = sorted.clone();
    deduped.dedup();
    assert_eq!(sorted, deduped, "a name claimed twice");
    assert_eq!(claimed, variables(&AgentPlugin::new()), "stable");
}

/// A plugin that wants a tick says how often, and the answer is a span the
/// loop can actually wait for.
#[test]
fn a_ticking_plugin_names_its_interval() {
    fn interval(plugin: &impl Plugin) -> Option<Duration> {
        plugin.interval()
    }

    let interval = interval(&AgentPlugin::new()).expect("this plugin ticks");
    assert!(interval > Duration::ZERO);
}

/// Option defaults are `(name, value)` pairs, and a plugin that wants none
/// says so with an empty slice rather than by not being asked.
#[test]
fn option_defaults_are_named_pairs() {
    fn defaults(plugin: &impl Plugin) -> &'static [(&'static str, &'static str)] {
        plugin.option_defaults()
    }

    for (name, value) in defaults(&AgentPlugin::new()) {
        assert!(!name.is_empty(), "an unnamed option");
        assert!(!value.is_empty(), "{name} has an empty default");
    }
}

/// A pane the plugin has published nothing for reads as empty metadata in a
/// `none` state — the same answer a pane with no agent in it gives, so a
/// format need not tell the two apart.
#[test]
fn a_pane_with_nothing_published_reads_as_empty_metadata() {
    fn resolve(plugin: &impl Plugin, key: &str) -> Option<String> {
        plugin.resolve(UNKNOWN, key)
    }

    let plugin = AgentPlugin::new();
    assert_eq!(resolve(&plugin, "pane_agent").as_deref(), Some(""));
    assert_eq!(resolve(&plugin, "pane_agent_state").as_deref(), Some("none"));
    assert_eq!(resolve(&plugin, "pane_agent_pid").as_deref(), Some(""));
    assert_eq!(
        resolve(&plugin, "pane_agent_session_id").as_deref(),
        Some("")
    );
    assert_eq!(resolve(&plugin, "pane_agent_model").as_deref(), Some(""));
}

/// A key outside the plugin's list is declined, which is what leaves it to the
/// rest of the format engine.
#[test]
fn a_key_the_plugin_does_not_own_is_declined() {
    fn resolve(plugin: &impl Plugin, key: &str) -> Option<String> {
        plugin.resolve(UNKNOWN, key)
    }

    let plugin = AgentPlugin::new();
    assert_eq!(resolve(&plugin, "window_name"), None);
    assert_eq!(resolve(&plugin, ""), None);
}

/// Everything a plugin reads about the server arrives through [`Host`]: a tick
/// asks the host what panes there are and resolves them, rather than reaching
/// for the server itself.
#[test]
fn a_tick_reads_the_server_through_the_host() {
    fn tick(plugin: &mut impl Plugin, host: &dyn Host) {
        plugin.tick(host);
    }

    let host = FakeHost::with_panes(&[1, 2]);
    let mut plugin = AgentPlugin::new();
    tick(&mut plugin, &host);

    assert!(host.asked().pane_ids >= 1, "the host was asked for panes");
    assert_eq!(
        host.asked().resolved,
        vec![PaneId(1), PaneId(2)],
        "every pane the host named was resolved"
    );
}

/// A server with no panes is not an error, and gives the plugin nothing to
/// redraw.
#[test]
fn a_tick_over_an_empty_server_invalidates_nothing() {
    let host = FakeHost::with_panes(&[]);
    let mut plugin = AgentPlugin::new();

    fn tick(plugin: &mut impl Plugin, host: &dyn Host) {
        plugin.tick(host);
    }
    tick(&mut plugin, &host);

    assert!(host.asked().resolved.is_empty());
    assert!(host.asked().invalidated.is_empty(), "nothing moved");
}

/// Ticking twice over a server nothing changed in redraws nothing the second
/// time: only a pane whose status appeared, changed or went away is stale.
#[test]
fn a_second_tick_over_an_unchanged_server_redraws_nothing() {
    let host = FakeHost::with_panes(&[1]);
    let mut plugin = AgentPlugin::new();

    fn tick(plugin: &mut impl Plugin, host: &dyn Host) {
        plugin.tick(host);
    }
    tick(&mut plugin, &host);
    let after_first = host.asked().invalidated.len();
    tick(&mut plugin, &host);

    assert_eq!(
        host.asked().invalidated.len(),
        after_first,
        "nothing changed, so nothing was invalidated again"
    );
}

/// Values are pulled, not pushed, so a tick does not change what a pane the
/// plugin found nothing in resolves to.
#[test]
fn a_tick_leaves_an_unknown_pane_reading_as_empty() {
    let host = FakeHost::with_panes(&[1]);
    let mut plugin = AgentPlugin::new();
    plugin.tick(&host);

    fn resolve(plugin: &impl Plugin, key: &str) -> Option<String> {
        plugin.resolve(UNKNOWN, key)
    }
    assert_eq!(resolve(&plugin, "pane_agent").as_deref(), Some(""));
    assert_eq!(resolve(&plugin, "pane_agent_state").as_deref(), Some("none"));
}

/// A plugin that does not care about events is not disturbed by them: the
/// default `on_notify` is inert, and what the plugin resolves is unchanged.
#[test]
fn events_a_plugin_ignores_change_nothing() {
    fn notify(plugin: &mut impl Plugin, event: &Event<'_>) {
        plugin.on_notify(event);
    }

    let mut plugin = AgentPlugin::new();
    notify(&mut plugin, &Event::PaneOutput(PaneId(1)));
    notify(
        &mut plugin,
        &Event::Notify {
            name: "window-linked",
            pane: Some(PaneId(1)),
        },
    );
    notify(
        &mut plugin,
        &Event::Notify {
            name: "session-closed",
            pane: None,
        },
    );

    assert_eq!(plugin.resolve(UNKNOWN, "pane_agent").as_deref(), Some(""));
}

/// Registration calls `start` once before anything else; a plugin with
/// nothing to set up is left as it was.
#[test]
fn starting_a_plugin_leaves_it_ready_to_resolve() {
    fn start(plugin: &mut impl Plugin, host: &dyn Host) {
        plugin.start(host);
    }

    let host = FakeHost::with_panes(&[]);
    let mut plugin = AgentPlugin::new();
    start(&mut plugin, &host);

    assert_eq!(
        plugin.resolve(UNKNOWN, "pane_agent_state").as_deref(),
        Some("none")
    );
}
