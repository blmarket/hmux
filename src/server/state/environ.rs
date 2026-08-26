//! The environment tmux keeps for spawned panes: the global environment, each
//! session's overrides, and the `#{?...}` variables derived from them.
//!
//! `set-environment`/`unset-environment` write here; a pane spawn folds the
//! global set, the session set, and the per-spawn `-e` values into the child's
//! environment.

use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::rc::Rc;

use super::{
    DEFAULT_PATH, PANE_ID_PLACEHOLDER, RenderInvalidation, SESSION_ID_PLACEHOLDER, ServerState,
    SpawnSession, environment_entries,
};
use crate::server::format::glob_match;
use crate::server::options::{OptionsView, option_default};

impl ServerState {
    /// Set a global environment variable (`set-environment -g VAR VALUE`).
    pub fn set_env(&mut self, key: &str, value: &str) {
        self.environment_generation += 1;
        let changed = self.environment.get(key).is_none_or(|old| old != value)
            || self.hidden_environment.contains(key)
            || self.removed_environment.contains(key);
        self.environment.insert(key.to_string(), value.to_string());
        self.hidden_environment.remove(key);
        self.removed_environment.remove(key);
        self.seeded_environment.remove(key);
        if changed {
            self.invalidate_all_clients(RenderInvalidation::STATUS);
        }
    }

    pub(crate) fn set_session_env(
        &mut self,
        target: &str,
        key: &str,
        value: &str,
        hidden: bool,
    ) -> io::Result<()> {
        self.environment_generation += 1;
        let session = self
            .find_mut(target)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "can't find session"))?;
        session
            .environment
            .insert(key.to_string(), value.to_string());
        session.removed_environment.remove(key);
        if hidden {
            session.hidden_environment.insert(key.to_string());
        } else {
            session.hidden_environment.remove(key);
        }
        Ok(())
    }

    /// The environment a process spawned for `session` runs with, as
    /// `KEY=VALUE` entries.
    ///
    /// tmux's `environ_for_session` layers the session's environment over the
    /// global one and pushes the result with `environ_push`, which skips a
    /// hidden entry and drops one marked removed. hmux's daemon is not forked
    /// from a client, so where tmux's global environment starts out as the
    /// first client's, hmux uses the environment of the client that asked for
    /// the spawn; the two `set-environment` layers then apply over it exactly as
    /// tmux's do. That difference is characterized in README.md.
    pub(crate) fn spawn_environment(
        &self,
        session: Option<&str>,
        client_environment: &[String],
        extra: &[&str],
    ) -> Vec<String> {
        environment_entries(self.session_environment(session, client_environment, extra))
    }

    /// The `TMUX` variable's value for a session named by `session` — the
    /// socket, the server's pid, and the session id, as `environ_for_session`
    /// formats them.
    fn tmux_variable(&self, session: &str) -> String {
        format!(
            "{},{},{}",
            self.socket_path.display(),
            std::process::id(),
            session,
        )
    }

    /// The environment tmux's `spawn_pane` gives a pane: everything
    /// [`Self::spawn_environment`] builds, plus the pane's own name, the `PATH`
    /// an unattached client would have run with, and the session's shell.
    ///
    /// `unattached_client` is tmux's `c != NULL && c->session == NULL` — a
    /// command client, which is why `tmux new myprogram` finds `myprogram` on
    /// the caller's path rather than the session's.
    pub(crate) fn pane_environment(
        &self,
        session: SpawnSession<'_>,
        client_environment: &[String],
        unattached_client: bool,
        extra: &[&str],
    ) -> Vec<String> {
        let target = match session {
            SpawnSession::Existing(target) => Some(target),
            SpawnSession::Pending => None,
        };
        let mut environment = self.session_environment(target, client_environment, extra);
        if matches!(session, SpawnSession::Pending) {
            // The session is created before its pane is spawned, so its id
            // replaces the placeholder rather than staying at the `-1` a
            // sessionless `environ_for_session` would report.
            environment.insert(
                "TMUX".to_owned(),
                self.tmux_variable(SESSION_ID_PLACEHOLDER),
            );
        }
        environment.insert("TMUX_PANE".to_owned(), PANE_ID_PLACEHOLDER.to_owned());
        if unattached_client {
            if let Some(path) = client_environment
                .iter()
                .filter_map(|entry| entry.split_once('='))
                .find_map(|(name, value)| (name == "PATH").then_some(value))
            {
                environment.insert("PATH".to_owned(), path.to_owned());
            }
        }
        environment
            .entry("PATH".to_owned())
            .or_insert_with(|| DEFAULT_PATH.to_owned());
        environment.insert("SHELL".to_owned(), self.default_shell(target).to_owned());
        environment_entries(environment)
    }

    /// The shell a pane spawned for `target` runs: the `default-shell` option,
    /// which is both the program a command-less pane execs and the `SHELL` its
    /// process is given.
    pub(crate) fn default_shell(&self, target: Option<&str>) -> &str {
        let shell = target
            .and_then(|target| self.option_for_target(target, "default-shell"))
            .or_else(|| self.global_options().session().get("default-shell"))
            .unwrap_or("/bin/sh");
        if Path::new(shell)
            .metadata()
            .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        {
            shell
        } else {
            "/bin/sh"
        }
    }

    fn session_environment(
        &self,
        session: Option<&str>,
        client_environment: &[String],
        extra: &[&str],
    ) -> BTreeMap<String, String> {
        // The daemon's own environment is the base, as tmux's `global_environ`
        // is; the requesting client's overrides it, and an explicit
        // `set-environment -g` overrides that in turn.
        let (seeded, assigned): (BTreeMap<_, _>, BTreeMap<_, _>) = self
            .environment
            .iter()
            .map(|(name, value)| (name.clone(), value.clone()))
            .partition(|(name, _)| self.seeded_environment.contains(name));
        let mut environment = seeded;
        environment.extend(
            client_environment
                .iter()
                .filter_map(|entry| entry.split_once('='))
                .map(|(name, value)| (name.to_owned(), value.to_owned())),
        );
        let session = session.and_then(|target| self.resolve_session(target));
        let mut layer = |values: &BTreeMap<String, String>,
                         removed: &BTreeSet<String>,
                         hidden: &BTreeSet<String>| {
            for (name, value) in values {
                environment.insert(name.clone(), value.clone());
            }
            for name in removed.iter().chain(hidden.iter()) {
                environment.remove(name);
            }
        };
        layer(
            &assigned,
            &self.removed_environment,
            &self.hidden_environment,
        );
        if let Some(session) = session {
            layer(
                &session.environment,
                &session.removed_environment,
                &session.hidden_environment,
            );
        }
        // The variables `environ_for_session` always writes. `TMUX` is what
        // lets a process tell it is inside a server, and which one.
        environment.insert(
            "TERM".to_owned(),
            self.server_options()
                .get("default-terminal")
                .or_else(|| option_default("default-terminal"))
                .unwrap_or("tmux-256color")
                .to_owned(),
        );
        environment.insert("TERM_PROGRAM".to_owned(), "tmux".to_owned());
        environment.insert(
            "TERM_PROGRAM_VERSION".to_owned(),
            crate::TMUX_VERSION.to_owned(),
        );
        environment.insert("COLORTERM".to_owned(), "truecolor".to_owned());
        // A systemd build clears the socket-activation variables, so a pane
        // never inherits fds the server was handed.
        for name in ["LISTEN_PID", "LISTEN_FDS", "LISTEN_FDNAMES"] {
            environment.remove(name);
        }
        environment.insert(
            "TMUX".to_owned(),
            self.tmux_variable(&session.map_or(-1, |session| session.id as i64).to_string()),
        );
        for entry in extra {
            if let Some((name, value)) = entry.split_once('=') {
                environment.insert(name.to_owned(), value.to_owned());
            }
        }
        environment
    }

    /// The environment a process started by the server — a `#()` job, a
    /// `run-shell`, a `copy-pipe` — runs with: `environ_for_session` and
    /// nothing else, since none of those spawns is a pane and tmux gives a job
    /// no view of the requesting client.
    ///
    /// Cached against [`ServerState::environment_generation`], because a
    /// command builds its job runner whether or not it expands a `#()`.
    pub(crate) fn job_environment(&self, session: Option<&str>) -> Rc<Vec<String>> {
        let session_id = session
            .and_then(|target| self.resolve_session(target))
            .map(|session| session.id);
        let generation = self.environment_generation;
        if let Some((cached_generation, cached_session, environment)) =
            self.job_environment_cache.borrow().as_ref()
        {
            if *cached_generation == generation && *cached_session == session_id {
                return Rc::clone(environment);
            }
        }
        // Built outside the borrow: `spawn_environment` walks the rest of the
        // state and must not run with the cache borrowed.
        let environment = Rc::new(self.spawn_environment(session, &[], &[]));
        *self.job_environment_cache.borrow_mut() =
            Some((generation, session_id, Rc::clone(&environment)));
        environment
    }

    /// tmux's `environ_update`: copy the variables `update-environment` names
    /// from a client's environment into a session's.
    ///
    /// Each entry is an fnmatch pattern. A pattern that matches nothing is
    /// *removed* for the session rather than left to fall through to the global
    /// environment, which is what `show-environment` reports as `-NAME`.
    pub(crate) fn update_session_environment(&mut self, target: &str, client_env: &[String]) {
        let patterns = self
            .resolve_session(target)
            .map(|session| session.options(&self.global_options))
            .unwrap_or_else(|| OptionsView::one(self.global_options.session()))
            .array_values("update-environment")
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        for pattern in patterns {
            let matched = client_env
                .iter()
                .filter_map(|entry| entry.split_once('='))
                .filter(|(name, _)| glob_match(pattern.as_bytes(), name.as_bytes()))
                .map(|(name, value)| (name.to_owned(), value.to_owned()))
                .collect::<Vec<_>>();
            if matched.is_empty() {
                let _ = self.unset_session_env(target, &pattern, true);
                continue;
            }
            for (name, value) in matched {
                let _ = self.set_session_env(target, &name, &value, false);
            }
        }
    }

    pub(crate) fn unset_session_env(
        &mut self,
        target: &str,
        key: &str,
        remove: bool,
    ) -> io::Result<()> {
        self.environment_generation += 1;
        let session = self
            .find_mut(target)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "can't find session"))?;
        session.environment.remove(key);
        session.hidden_environment.remove(key);
        if remove {
            session.removed_environment.insert(key.to_string());
        } else {
            session.removed_environment.remove(key);
        }
        Ok(())
    }

    pub(crate) fn session_env(
        &self,
        target: &str,
    ) -> io::Result<(
        &BTreeMap<String, String>,
        &BTreeSet<String>,
        &BTreeSet<String>,
    )> {
        let session = self
            .resolve_session(target)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "can't find session"))?;
        Ok((
            &session.environment,
            &session.removed_environment,
            &session.hidden_environment,
        ))
    }

    /// Set a hidden global environment variable (`set-environment -g -h`).
    pub fn set_hidden_env(&mut self, key: &str, value: &str) {
        self.environment_generation += 1;
        let changed = self.environment.get(key).is_none_or(|old| old != value)
            || !self.hidden_environment.contains(key)
            || self.removed_environment.contains(key);
        self.environment.insert(key.to_string(), value.to_string());
        self.hidden_environment.insert(key.to_string());
        self.removed_environment.remove(key);
        self.seeded_environment.remove(key);
        if changed {
            self.invalidate_all_clients(RenderInvalidation::STATUS);
        }
    }

    /// Mark a global environment variable for removal from child processes.
    pub fn remove_env(&mut self, key: &str) {
        self.environment_generation += 1;
        let had_value = self.environment.remove(key).is_some();
        let was_hidden = self.hidden_environment.remove(key);
        let newly_removed = self.removed_environment.insert(key.to_string());
        let changed = had_value || was_hidden || newly_removed;
        if changed {
            self.invalidate_all_clients(RenderInvalidation::STATUS);
        }
    }

    /// Unset a global environment variable (`set-environment -g -u VAR`).
    pub fn unset_env(&mut self, key: &str) {
        self.environment_generation += 1;
        let had_value = self.environment.remove(key).is_some();
        let was_hidden = self.hidden_environment.remove(key);
        let was_removed = self.removed_environment.remove(key);
        let changed = had_value || was_hidden || was_removed;
        if changed {
            self.invalidate_all_clients(RenderInvalidation::STATUS);
        }
    }

    /// Look up a global environment variable.
    pub fn get_env(&self, key: &str) -> Option<&str> {
        self.environment.get(key).map(String::as_str)
    }

    /// Whether a global environment variable is hidden.
    pub fn env_is_hidden(&self, key: &str) -> bool {
        self.hidden_environment.contains(key)
    }

    /// Whether a global environment variable is marked for removal.
    pub fn env_is_removed(&self, key: &str) -> bool {
        self.removed_environment.contains(key)
    }

    /// Seed a format table's environment layer the way tmux's `format_find`
    /// reads it: the target session's own entries over the global ones, and a
    /// name the session removed hiding the global value rather than exposing
    /// it.
    pub(crate) fn seed_format_environment(
        &self,
        vars: &mut crate::server::format::Vars,
        session: Option<&super::Session>,
    ) {
        for (name, value) in &self.environment {
            vars.set_environment(name.clone(), value.clone());
        }
        let Some(session) = session else {
            return;
        };
        for (name, value) in &session.environment {
            vars.set_environment(name.clone(), value.clone());
        }
        for name in &session.removed_environment {
            vars.unset_environment(name);
        }
    }

    /// Iterate the global environment in sorted order.
    pub fn env_iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.environment.iter()
    }

    /// Iterate names marked for removal in sorted order.
    pub fn removed_env_iter(&self) -> impl Iterator<Item = &String> {
        self.removed_environment.iter()
    }
}
