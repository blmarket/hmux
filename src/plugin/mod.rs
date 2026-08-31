//! The server's plugin layer.
//!
//! A plugin is a bundle of format variables the server does not have of its
//! own, plus whatever work it takes to keep them current. It publishes an
//! id-keyed dictionary — pane id and variable name in, string out — which the
//! format engine consults after its own static table, so a plugin's variables
//! expand anywhere a built-in one does: status formats, `list-panes -F`,
//! `display-message`, and control-mode `refresh-client -B` subscriptions.
//!
//! Everything a plugin reads about the server arrives through [`Host`], which
//! is the pane observability contract shared with the hmux daemon plus one
//! call to say a pane's values have changed. Nothing in a plugin holds a
//! `*mut window_pane`: panes are named by id and resolved per call, so a pane
//! destroyed between two ticks cannot be read through stale state.
//!
//! Adding a plugin is a matter of implementing [`Plugin`] and naming it in
//! [`builtins`]. Which of them run is read from `TMUX_C2RS_PLUGINS` at server
//! start — see [`enabled_names`]. The default is [`DEFAULT_PLUGINS`];
//! `TMUX_C2RS_PLUGINS=none` turns them all off, which is how the server is
//! compared against tmux, since a plugin adds variables and option defaults
//! tmux does not have.

use std::cell::{Cell, RefCell};
use std::ffi::{CStr, CString};
use std::time::{Duration, Instant};

use crate::reactor::{Timer, TimerHandle};
use crate::types::{timeval, u_int};

pub mod agent;
pub mod git;
pub mod host;

pub use hmux_agent::observability::v1::{PaneId, ServerObservability};
pub use host::ServerHost;

/// What a plugin is told about the server while it runs.
///
/// The read side is [`ServerObservability`] — the same contract the hmux
/// daemon implements over its own panes, which is what lets a detector written
/// against one server run unchanged on the other.
pub trait Host: ServerObservability {
    /// Report that the values this plugin publishes for `pane` have changed,
    /// so anything drawing them is redrawn. Cheap and idempotent: it marks the
    /// pane's window for a status redraw and returns.
    fn invalidate(&self, pane: PaneId);
}

/// Something that happened in the server, for a plugin that would rather react
/// than wait for its next tick.
#[derive(Clone, Copy, Debug)]
pub enum Event<'a> {
    /// A pane parsed some output. Raised from the pane input path as the bytes
    /// arrive, ahead of any notification the queue would carry.
    PaneOutput(PaneId),
    /// A server notification, under the name its hook has.
    Notify { name: &'a str, pane: Option<PaneId> },
}

/// A source of format variables the server does not have of its own.
///
/// Values are pulled, not pushed: [`resolve`](Plugin::resolve) is called only
/// when an expansion actually names one of the plugin's variables, so an
/// expensive value costs nothing in a format that never mentions it.
pub trait Plugin {
    /// The name this plugin is enabled by.
    fn name(&self) -> &'static str;

    /// Every format variable this plugin answers for. Checked once at
    /// registration; a key outside this set is never routed here.
    fn variables(&self) -> &'static [&'static str];

    /// How often [`tick`](Plugin::tick) should run, or `None` for a plugin
    /// that only ever answers lookups and events.
    fn interval(&self) -> Option<Duration> {
        None
    }

    /// Option defaults this plugin wants in place of the server's own, as
    /// `(name, value)`. Applied at server start and only to options still
    /// holding their built-in default, so a configuration file — which is read
    /// later — always wins.
    fn option_defaults(&self) -> &'static [(&'static str, &'static str)] {
        &[]
    }

    /// Called once, when the plugin is registered at server start.
    fn start(&mut self, host: &dyn Host) {
        let _ = host;
    }

    /// Called every [`interval`](Plugin::interval).
    fn tick(&mut self, host: &dyn Host) {
        let _ = host;
    }

    /// The value of `key` for `pane`, or `None` to expand to nothing.
    fn resolve(&self, pane: PaneId, key: &str) -> Option<String>;

    /// Called as things happen, for a plugin that wants to react promptly.
    fn on_notify(&mut self, event: &Event<'_>) {
        let _ = event;
    }
}

/// Add a plugin to the running server.
///
/// Everything else follows from the plugin itself: its variables start
/// expanding, its tick is scheduled, and its option defaults go in. This is
/// what [`init`] calls for each enabled built-in, and it is what an embedder
/// calls for one of its own.
pub fn register(mut plugin: Box<dyn Plugin>) {
    crate::server::server_apply_option_defaults(plugin.option_defaults());
    plugin.start(&ServerHost);
    let interval = plugin.interval();
    PLUGINS.with(|plugins| {
        plugins.borrow_mut().push(Slot {
            plugin: RefCell::new(plugin),
            interval,
            due: Cell::new(Instant::now() + interval.unwrap_or_default()),
        })
    });
    arm_next();
}

/// The plugins this build carries. A new plugin is added here and enabled by
/// name; nothing else in the server needs to know about it.
fn builtins() -> Vec<Box<dyn Plugin>> {
    vec![
        Box::new(agent::AgentPlugin::new()),
        Box::new(git::GitPlugin::new()),
    ]
}

/// The plugins that run when `TMUX_C2RS_PLUGINS` says nothing.
const DEFAULT_PLUGINS: &[&str] = &["agent", "git"];

/// The plugin names `TMUX_C2RS_PLUGINS` asks for: a comma-separated list, or
/// `all`, or `none`.
///
/// Unset means [`DEFAULT_PLUGINS`], because a server nobody has configured
/// should be the one worth running. `none` — and an empty value, which is what
/// a shell writes for a variable it wants cleared — turns every plugin off,
/// and that is the setting to compare against tmux under: it leaves the
/// format hooks reading one flag and no option default touched.
fn enabled_names() -> Vec<String> {
    let Ok(value) = std::env::var("TMUX_C2RS_PLUGINS") else {
        return DEFAULT_PLUGINS
            .iter()
            .map(|name| name.to_string())
            .collect();
    };
    let names: Vec<String> = value
        .split(',')
        .map(|name| name.trim().to_ascii_lowercase())
        .filter(|name| !name.is_empty())
        .collect();
    match names.iter().any(|name| name == "none") {
        true => Vec::new(),
        false => names,
    }
}

/// One registered plugin and the schedule it runs on.
struct Slot {
    plugin: RefCell<Box<dyn Plugin>>,
    interval: Option<Duration>,
    /// When this plugin is next due. Measured from the end of the last tick,
    /// so a sweep that takes as long as its own interval leaves the gap
    /// between sweeps whole instead of running back to back.
    due: Cell<Instant>,
}

thread_local! {
    static PLUGINS: RefCell<Vec<Slot>> = const { RefCell::new(Vec::new()) };
    static TICK: RefCell<TimerHandle> = const { RefCell::new(TimerHandle::ZERO) };
}

/// Whether any plugin is registered. The lookup hooks in the format engine ask
/// this first so that a server running none pays a single flag read, and the
/// server's own option defaults ask it because they draw on plugin variables.
pub(crate) fn any_registered() -> bool {
    PLUGINS.with(|plugins| !plugins.borrow().is_empty())
}

/// Register the enabled built-in plugins and arm their tick.
///
/// Called from `server_start` once the reactor has been rebuilt on the far
/// side of the daemon fork, so the timer armed here belongs to the loop that
/// will run it.
pub(crate) fn init() {
    let wanted = enabled_names();
    if wanted.is_empty() {
        return;
    }
    let all = wanted.iter().any(|name| name == "all");
    for plugin in builtins() {
        if all || wanted.iter().any(|name| name == plugin.name()) {
            register(plugin);
        }
    }
}

/// Arm the shared tick at whichever plugin is due first.
fn arm_next() {
    let now = Instant::now();
    let soonest = PLUGINS.with(|plugins| {
        plugins
            .borrow()
            .iter()
            .filter(|slot| slot.interval.is_some())
            .map(|slot| slot.due.get().saturating_duration_since(now))
            .min()
    });
    let Some(after) = soonest else {
        return;
    };
    TICK.with(|timer| {
        let mut timer = timer.borrow_mut();
        if !timer.is_set() {
            timer.set_callback(tick);
        }
        timer.arm(timeval {
            tv_sec: after.as_secs() as _,
            tv_usec: after.subsec_micros() as _,
        });
    });
}

/// Run every plugin that has come due, then re-arm.
fn tick() {
    let host = ServerHost;
    let now = Instant::now();
    let due: Vec<usize> = PLUGINS.with(|plugins| {
        plugins
            .borrow()
            .iter()
            .enumerate()
            .filter(|(_, slot)| slot.interval.is_some() && slot.due.get() <= now)
            .map(|(at, _)| at)
            .collect()
    });
    for at in due {
        // The plugin is borrowed for the length of its own tick only: it may
        // reach back into the server, and the server may expand a format,
        // which reads every other plugin.
        PLUGINS.with(|plugins| {
            let plugins = plugins.borrow();
            let Some(slot) = plugins.get(at) else {
                return;
            };
            if let Ok(mut plugin) = slot.plugin.try_borrow_mut() {
                plugin.tick(&host);
            }
            slot.due
                .set(Instant::now() + slot.interval.unwrap_or_default());
        });
    }
    host::prune_revisions();
    arm_next();
}

/// Note that a pane parsed output: bump its revision, then tell any plugin
/// that would rather react than wait for its next tick.
pub(crate) fn note_pane_output(id: u_int) {
    if !any_registered() {
        return;
    }
    host::note_output(id);
    on_event(&Event::PaneOutput(PaneId(id)));
}

/// Pass a server notification on to the plugins, under the name its hook has.
pub(crate) fn note_notification(name: Option<&CStr>, pane: ::core::ffi::c_int) {
    if !any_registered() {
        return;
    }
    let Some(name) = name.and_then(|name| name.to_str().ok()) else {
        return;
    };
    on_event(&Event::Notify {
        name,
        pane: (pane >= 0).then(|| PaneId(pane as u_int)),
    });
}

/// Hand an event to every plugin.
pub(crate) fn on_event(event: &Event<'_>) {
    if !any_registered() {
        return;
    }
    let count = PLUGINS.with(|plugins| plugins.borrow().len());
    for at in 0..count {
        PLUGINS.with(|plugins| {
            let plugins = plugins.borrow();
            let Some(slot) = plugins.get(at) else {
                return;
            };
            if let Ok(mut plugin) = slot.plugin.try_borrow_mut() {
                plugin.on_notify(event);
            }
        });
    }
}

/// The value a plugin gives `key` for the pane `wp_id`, or `None` when no
/// plugin owns the key.
///
/// This is what the format engine calls after its own static table misses.
pub(crate) fn find(wp_id: Option<u_int>, key: &CStr) -> Option<CString> {
    if !any_registered() {
        return None;
    }
    let wp_id = wp_id?;
    let key = key.to_str().ok()?;
    let value = PLUGINS.with(|plugins| {
        for slot in plugins.borrow().iter() {
            // A plugin borrowed for its own tick cannot answer; the rest still
            // can, and the key belongs to at most one of them anyway.
            let Ok(plugin) = slot.plugin.try_borrow() else {
                continue;
            };
            if !plugin.variables().contains(&key) {
                continue;
            }
            return plugin.resolve(PaneId(wp_id), key);
        }
        None
    })?;
    CString::new(value).ok()
}

/// Every plugin variable for the pane `wp_id`, for the walks that enumerate a
/// format tree rather than look one key up.
pub(crate) fn each(wp_id: Option<u_int>) -> Vec<(CString, CString)> {
    if !any_registered() {
        return Vec::new();
    }
    let Some(wp_id) = wp_id else {
        return Vec::new();
    };
    let mut out = Vec::new();
    PLUGINS.with(|plugins| {
        for slot in plugins.borrow().iter() {
            let Ok(plugin) = slot.plugin.try_borrow() else {
                continue;
            };
            for key in plugin.variables() {
                let Some(value) = plugin.resolve(PaneId(wp_id), key) else {
                    continue;
                };
                let (Ok(key), Ok(value)) = (CString::new(*key), CString::new(value)) else {
                    continue;
                };
                out.push((key, value));
            }
        }
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::test_fixtures::globals;

    /// A plugin with nothing behind it: two variables, one of which it
    /// declines to answer for.
    struct Fake;

    impl Plugin for Fake {
        fn name(&self) -> &'static str {
            "fake"
        }

        fn variables(&self) -> &'static [&'static str] {
            &["fake_pane", "fake_silent"]
        }

        fn resolve(&self, pane: PaneId, key: &str) -> Option<String> {
            match key {
                "fake_pane" => Some(::std::format!("pane-{}", pane.0)),
                _ => None,
            }
        }
    }

    /// Registration is the whole of what a plugin has to do to be reachable:
    /// its variables answer for a pane, a key it does not own is left to the
    /// rest of the format engine, and a pane-less lookup has nothing to ask.
    #[test]
    fn a_registered_plugin_answers_for_its_own_variables() {
        let _guard = globals();
        PLUGINS.with(|plugins| plugins.borrow_mut().clear());
        register(Box::new(Fake));

        assert_eq!(
            find(Some(7), c"fake_pane").as_deref(),
            Some(c"pane-7"),
            "the plugin's own variable"
        );
        assert_eq!(
            find(Some(7), c"fake_silent"),
            None,
            "declined by the plugin"
        );
        assert_eq!(find(Some(7), c"pane_id"), None, "not the plugin's variable");
        assert_eq!(find(None, c"fake_pane"), None, "no pane to answer for");

        // The enumeration skips what `resolve` declines, exactly as a lookup
        // of the same key does.
        assert_eq!(
            each(Some(7)),
            vec![(c"fake_pane".to_owned(), c"pane-7".to_owned())]
        );

        PLUGINS.with(|plugins| plugins.borrow_mut().clear());
    }

    /// With nothing registered the hooks are inert, which is what keeps a
    /// server running no plugins byte-identical to tmux.
    #[test]
    fn no_plugins_means_no_lookups() {
        let _guard = globals();
        PLUGINS.with(|plugins| plugins.borrow_mut().clear());
        assert_eq!(find(Some(1), c"fake_pane"), None);
        assert!(each(Some(1)).is_empty());
    }

    /// `TMUX_C2RS_PLUGINS` is a list of names, and the spelling is forgiving
    /// about spacing and case because it is typed by hand. Saying nothing
    /// leaves the defaults in place; `none` is how they are turned off, and an
    /// empty value means the same, since that is what a shell leaves behind
    /// for a variable someone wanted cleared.
    #[test]
    fn the_enabled_list_is_read_by_name() {
        let _guard = globals();
        // SAFETY: the fixture guard serialises the tests that touch process
        // state, and nothing else in this process reads the variable.
        unsafe {
            std::env::remove_var("TMUX_C2RS_PLUGINS");
            assert_eq!(enabled_names(), DEFAULT_PLUGINS, "unset means the default");

            std::env::set_var("TMUX_C2RS_PLUGINS", " Agent , ,all ");
            assert_eq!(
                enabled_names(),
                vec!["agent".to_string(), "all".to_string()]
            );

            std::env::set_var("TMUX_C2RS_PLUGINS", "none");
            assert!(enabled_names().is_empty(), "none turns them off");
            std::env::set_var("TMUX_C2RS_PLUGINS", "");
            assert!(enabled_names().is_empty(), "so does an empty value");
            // `none` wins over anything named alongside it, so a list that
            // ends up with both is off rather than half on.
            std::env::set_var("TMUX_C2RS_PLUGINS", "agent,none");
            assert!(enabled_names().is_empty());

            std::env::remove_var("TMUX_C2RS_PLUGINS");
        }
    }
}
