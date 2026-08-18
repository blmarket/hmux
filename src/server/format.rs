//! A small tmux format-string expander (`#{...}`).
//!
//! tmux's `format.c` is a large engine; this is the slice the prototype's
//! commands need: literal passthrough, `##` → `#`, `#{variable}` lookups, the
//! single-letter shorthands (`#S`, `#I`, …), the ternary conditional
//! (`#{?cond,then,else}`), and the string comparison operators (`#{==:a,b}` /
//! `#{!=:a,b}`). Unknown variables expand to the empty string, exactly as real
//! tmux does (`format_find` returns NULL → nothing is substituted).
//!
//! The variables modeled are the ones the conformance suite exercises against
//! real tmux: `session_name`, `session_id`, `session_windows`, `window_index`,
//! `window_name`, `window_id`, `window_panes`, `window_active`, `window_flags`,
//! `pane_index`, `pane_id`, `pane_active`. This is enough to back `-F`/`-p`/`-P`
//! on `list-sessions`, `list-windows`, `list-panes`, `display-message`,
//! `new-session`, and `new-window`, including conditionals and comparisons over
//! those variables. Every construct here is pinned against real tmux by the
//! differential conformance suite.

use std::collections::HashMap;
use std::ffi::{CStr, CString};

use regex::RegexBuilder;

use super::style::{parse_colour, Colour};
use hmux_vt::codepoint_width;

/// One variable's value: either materialized, or a deferred computation that
/// runs (once) only if a format actually looks the variable up. The deferred
/// form exists for the variables whose value is a syscall away — `/proc`
/// walks behind `#{pane_current_command}`, say — which would otherwise be
/// paid on every command dispatch whether or not the template names them.
#[derive(Clone)]
enum Slot {
    Ready(String),
    Lazy(LazySlot),
}

#[derive(Clone)]
struct LazySlot {
    compute: std::rc::Rc<dyn Fn() -> String>,
    cache: std::cell::OnceCell<String>,
}

impl std::fmt::Debug for Slot {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Slot::Ready(value) => value.fmt(formatter),
            Slot::Lazy(slot) => match slot.cache.get() {
                Some(value) => value.fmt(formatter),
                None => formatter.write_str("<lazy>"),
            },
        }
    }
}

/// About how many variables a fully-populated context ends up holding: the
/// ~150 `vars_full` sets, the exported environment, and the effective options
/// on top of it. Reserving the table up front is what keeps a command off the
/// growth ladder — a `HashMap` that starts empty rehashes eight times on the
/// way to that count, and every rehash re-hashes every key already in it.
const TYPICAL_VARS: usize = 448;

/// Whether expanding `template` can read the variable table at all.
///
/// The expander substitutes only at `#`, and the time pass that runs before it
/// only at `%`; a template with neither is copied through byte for byte, so
/// the table it would have been expanded against is never looked at. tmux
/// gets this for free — `format_find` runs a callback for the name a template
/// actually mentions, so a template naming nothing costs nothing — while hmux
/// builds its table up front. Callers that hold the template ask this first
/// and skip the build.
///
/// `%` is included because the time pass runs first and its output is what the
/// `#` pass then sees. Nothing a `strftime` conversion produces is expected to
/// contain a `#`, but the check costs one byte scan and does not depend on the
/// locale being that well behaved.
pub(crate) fn reads_vars(template: &str) -> bool {
    template.contains(['#', '%'])
}

/// A resolved format context: the variable → value map for one target
/// (session, and optionally a specific window/pane within it).
#[derive(Debug, Clone)]
pub struct Vars {
    /// Keys are tmux's fixed names, so the common ones are `&'static str` and
    /// cost nothing to store. The borrowed names an option or environment
    /// walk produces are copied, as they must be.
    map: HashMap<std::borrow::Cow<'static, str>, Slot>,
    /// The environment layer `format_find` consults only after every option
    /// and format variable has missed, and skips entirely for `t:`. Keeping it
    /// out of `map` is what preserves both of those orderings.
    environment: HashMap<String, String>,
}

/// The command-queue item a format is expanded under, which tmux publishes
/// through `cmdq_merge_formats` as `#{command}` and `#{current_file}`.
#[derive(Clone, Default)]
pub(crate) struct QueueItem {
    pub(crate) command: Option<&'static str>,
    pub(crate) current_file: Option<std::rc::Rc<str>>,
}

thread_local! {
    static QUEUE_ITEM: std::cell::RefCell<QueueItem> =
        const { std::cell::RefCell::new(QueueItem { command: None, current_file: None }) };
}

/// Publish `item` for as long as the returned guard lives, restoring whatever
/// was published before it. The queue installs one around each poll of a
/// command, so an item that suspends does not keep its own formats published
/// while another queue runs.
pub(crate) fn enter_queue_item(item: QueueItem) -> QueueItemGuard {
    QueueItemGuard(QUEUE_ITEM.with(|current| current.replace(item)))
}

pub(crate) struct QueueItemGuard(QueueItem);

impl Drop for QueueItemGuard {
    fn drop(&mut self) {
        QUEUE_ITEM.with(|current| *current.borrow_mut() = std::mem::take(&mut self.0));
    }
}

/// The file the running queue item was compiled from, for a queue that inserts
/// items of its own from within one.
pub(crate) fn queue_item_file() -> Option<std::rc::Rc<str>> {
    QUEUE_ITEM.with(|current| current.borrow().current_file.clone())
}

impl Vars {
    /// The table a template that cannot read one is expanded against.
    ///
    /// [`Vars::new`] reserves for a full context and resolves the
    /// server-global names; this reserves nothing and resolves nothing,
    /// because [`reads_vars`] already said no lookup will reach it.
    pub(crate) fn empty() -> Vars {
        Vars {
            map: HashMap::new(),
            environment: HashMap::new(),
        }
    }

    pub fn new() -> Vars {
        let mut vars = Vars {
            map: HashMap::with_capacity(TYPICAL_VARS),
            environment: HashMap::new(),
        };
        // The daemon's uid cannot change, and resolving the name walks NSS
        // (sockets to nscd, /etc/passwd) — worth doing exactly once.
        static USER: std::sync::OnceLock<(libc::uid_t, Option<String>)> =
            std::sync::OnceLock::new();
        let (uid, user) = USER.get_or_init(|| {
            let uid = unsafe { libc::getuid() };
            (uid, username(uid))
        });
        vars.set("uid", uid.to_string());
        if let Some(user) = user {
            vars.set("user", user.clone());
        }
        // The hostname is server-global too: tmux resolves `#{host}` from the
        // format's global table, so it expands even where the lookup found no
        // session, window or pane to hang the rest of the context off.
        vars.set_lazy("host", hostname)
            .set_lazy("host_short", hostname_short);
        QUEUE_ITEM.with(|item| {
            let item = item.borrow();
            if let Some(command) = item.command {
                vars.set("command", command);
            }
            if let Some(file) = &item.current_file {
                vars.set("current_file", file.to_string());
            }
        });
        vars
    }

    /// Set a variable. Keys are the fixed `&'static str` names tmux uses; a
    /// caller with a borrowed name of its own hands in an owned `String`.
    pub fn set(
        &mut self,
        key: impl Into<std::borrow::Cow<'static, str>>,
        value: impl Into<String>,
    ) -> &mut Vars {
        let key = key.into();
        let mut value = value.into();
        if super::options::option_is_flag(&key) {
            value = match value.as_str() {
                "on" => "1".to_string(),
                "off" => "0".to_string(),
                _ => value,
            };
        }
        self.map.insert(key, Slot::Ready(value));
        self
    }

    /// Set a variable whose value is computed on first lookup. No flag
    /// normalization is applied: deferred variables are never option flags.
    pub fn set_lazy(
        &mut self,
        key: impl Into<std::borrow::Cow<'static, str>>,
        compute: impl Fn() -> String + 'static,
    ) -> &mut Vars {
        self.map.insert(
            key.into(),
            Slot::Lazy(LazySlot {
                compute: std::rc::Rc::new(compute),
                cache: std::cell::OnceCell::new(),
            }),
        );
        self
    }

    /// Set an environment variable. tmux resolves these last of all, so they
    /// live beside the variable table rather than in it.
    pub(crate) fn set_environment(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.environment.insert(key.into(), value.into());
    }

    /// Drop an environment name, so a session's `set-environment -r` hides the
    /// global value the way `environ_find` returning a valueless entry does.
    pub(crate) fn unset_environment(&mut self, key: &str) {
        self.environment.remove(key);
    }

    pub(crate) fn lookup(&self, key: &str) -> Option<&str> {
        self.lookup_variable(key)
            .or_else(|| self.environment.get(key).map(String::as_str))
    }

    /// The environment layer alone, for a context that has its own fallbacks
    /// to consult between the variable table and it.
    pub(crate) fn lookup_environment(&self, key: &str) -> Option<&str> {
        self.environment.get(key).map(String::as_str)
    }

    /// Every variable in the table, name-ordered, as tmux's `format_each`
    /// walks the format tree. The environment layer is not part of it: tmux
    /// reaches that only when a lookup misses the tree.
    pub(crate) fn entries(&self) -> Vec<(&str, &str)> {
        let mut entries: Vec<(&str, &str)> = self
            .map
            .keys()
            .filter_map(|key| Some((key.as_ref(), self.lookup_variable(key)?)))
            .collect();
        entries.sort_unstable_by_key(|(key, _)| *key);
        entries
    }

    /// The lookup that stops before the environment layer.
    pub(crate) fn lookup_variable(&self, key: &str) -> Option<&str> {
        match self.map.get(key)? {
            Slot::Ready(value) => Some(value),
            Slot::Lazy(slot) => Some(slot.cache.get_or_init(|| (slot.compute)())),
        }
    }
}

impl Default for Vars {
    fn default() -> Self {
        Self::new()
    }
}

/// The full hostname (`#{host}`), via `gethostname(3)` — the same source real
/// tmux uses, so it matches on the same machine. Empty on failure.
pub(super) fn hostname() -> String {
    static HOSTNAME: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    HOSTNAME.get_or_init(read_hostname).clone()
}

fn read_hostname() -> String {
    let mut buf = [0 as libc::c_char; 256];
    // SAFETY: `gethostname` writes at most `buf.len()` bytes into `buf`.
    let rc = unsafe { libc::gethostname(buf.as_mut_ptr(), buf.len()) };
    if rc != 0 {
        return String::new();
    }
    // SAFETY: `gethostname` NUL-terminates on success (buffer is large enough).
    let cstr = unsafe { CStr::from_ptr(buf.as_ptr()) };
    cstr.to_string_lossy().into_owned()
}

/// The short hostname (`#{host_short}`): the part before the first `.`.
pub(super) fn hostname_short() -> String {
    let h = hostname();
    h.split('.').next().unwrap_or(&h).to_string()
}

/// The login name `uid` maps to, when the password database has one. Walks
/// NSS, so callers cache the answer rather than resolving it per expansion.
pub(crate) fn username(uid: libc::uid_t) -> Option<String> {
    let suggested = unsafe { libc::sysconf(libc::_SC_GETPW_R_SIZE_MAX) };
    let size = usize::try_from(suggested)
        .ok()
        .filter(|size| *size > 0)
        .unwrap_or(16 * 1024);
    let mut buffer = vec![0_u8; size];
    let mut passwd = unsafe { std::mem::zeroed::<libc::passwd>() };
    let mut result = std::ptr::null_mut();
    let status = unsafe {
        libc::getpwuid_r(
            uid,
            &mut passwd,
            buffer.as_mut_ptr().cast(),
            buffer.len(),
            &mut result,
        )
    };
    if status != 0 || result.is_null() || passwd.pw_name.is_null() {
        return None;
    }
    Some(
        unsafe { CStr::from_ptr(passwd.pw_name) }
            .to_string_lossy()
            .into_owned(),
    )
}

/// Which collection a `#{S:…}` / `#{W:…}` / `#{P:…}` loop iterates.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LoopKind {
    /// `#{S:…}` — every session.
    Session,
    /// `#{W:…}` — the windows of the current session.
    Window,
    /// `#{P:…}` — the panes of the current window.
    Pane,
}

/// Supplies the per-item variable contexts a loop modifier expands its body
/// against. Implemented by the command layer (which owns the session tree); the
/// format engine stays independent of the model.
pub trait LoopSource {
    /// The [`Vars`] context for each item of `kind`, in tmux's display order.
    fn items(&self, kind: LoopKind) -> Vec<Vars>;
}

/// A loop source that reads its session and window from the item the format is
/// currently being expanded against.
///
/// tmux's `format_loop_windows`/`format_loop_panes` walk `ft->s` and `ft->w`,
/// and each loop rebinds those for the body it expands, so a `#{P:…}` written
/// inside `#{W:…}` iterates the panes of the window being listed rather than
/// the panes of the window the format was targeted at. [`LoopSource`] is
/// anchored at one target for its whole expansion and cannot express that, so
/// the internal callers supply this instead.
pub(super) trait ScopedLoopSource {
    /// The [`Vars`] context for each item of `kind` within `vars`'s scope, in
    /// the order `flags` asks for. tmux sorts inside `sort_get_*`, on the
    /// objects themselves rather than on the strings a format entry carries,
    /// so the source owns the ordering.
    fn items_in_scope(&self, kind: FormatLoopKind, flags: &str, vars: &Vars) -> Vec<Vars>;
}

/// A [`LoopSource`] used as a [`ScopedLoopSource`]: the anchor it was built
/// with answers every loop, whatever the enclosing item is.
struct AnchoredLoops<'a>(&'a dyn LoopSource);

impl ScopedLoopSource for AnchoredLoops<'_> {
    fn items_in_scope(&self, kind: FormatLoopKind, _flags: &str, _vars: &Vars) -> Vec<Vars> {
        self.0.items(match kind {
            FormatLoopKind::Session => LoopKind::Session,
            FormatLoopKind::Window => LoopKind::Window,
            FormatLoopKind::Pane => LoopKind::Pane,
            // The public [`LoopSource`] has no client kind of its own.
            FormatLoopKind::Client => return Vec::new(),
        })
    }
}

#[derive(Clone, Debug)]
pub(super) struct FormatLoopItem {
    pub(super) vars: Vars,
    pub(super) active: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum FormatLoopKind {
    Session,
    Window,
    Pane,
    Client,
}

/// Context-specific services used by the single format evaluator.
///
/// tmux keeps parsing in one engine and varies only the format tree supplied by
/// each caller. This internal interface provides the same boundary without
/// changing the existing public [`LoopSource`] compatibility surface.
pub(super) trait FormatContext {
    fn lookup(&self, vars: &Vars, key: &str) -> Option<String> {
        vars.lookup(key).map(str::to_string)
    }

    /// The lookup `t:` performs. tmux's `format_find` skips the environment
    /// under `FORMAT_TIMESTRING`, so a name that only the environment carries
    /// is not a timestamp source.
    fn lookup_variable(&self, vars: &Vars, key: &str) -> Option<String> {
        vars.lookup_variable(key).map(str::to_string)
    }

    fn loop_items(
        &self,
        kind: FormatLoopKind,
        flags: &str,
        vars: &Vars,
    ) -> Option<Vec<FormatLoopItem>>;

    fn job(&self, command: &str, expanded: String, vars: &Vars) -> String;

    fn preserve_double_hash(&self) -> bool {
        false
    }

    fn search_pane(&self, vars: &Vars, term: &str, regex: bool, ignore_case: bool) -> u32;

    fn name_exists(&self, vars: &Vars, scope: NameScope, name: &str) -> bool;
}

/// Runs a `#()` command substitution and returns its cached output. tmux starts
/// jobs from *any* expansion, not only a status redraw, so the caller supplies
/// the job tree the expansion belongs to.
pub(super) trait FormatJobs {
    fn run(&self, command: &str, expanded: String, vars: &Vars) -> String;
}

/// Which tree `#{N:…}` asks about.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum NameScope {
    Window,
    Session,
}

/// The parts of the server a format reaches outside its own variables: tmux's
/// `format_search` over the target pane's screen, and the
/// `format_window_name`/`format_session_name` existence checks.
pub(super) trait FormatTree {
    /// The 1-based row of the target pane's visible screen matching `term`, or
    /// 0 — `window_pane_search`.
    fn search_pane(&self, vars: &Vars, term: &str, regex: bool, ignore_case: bool) -> u32;

    /// Whether a window of that name exists in the format's session, or a
    /// session of that name exists at all.
    fn name_exists(&self, vars: &Vars, scope: NameScope, name: &str) -> bool;
}

struct BasicContext<'a> {
    loops: Option<&'a dyn ScopedLoopSource>,
    jobs: Option<&'a dyn FormatJobs>,
    tree: Option<&'a dyn FormatTree>,
}

impl FormatContext for BasicContext<'_> {
    fn search_pane(&self, vars: &Vars, term: &str, regex: bool, ignore_case: bool) -> u32 {
        self.tree
            .map_or(0, |tree| tree.search_pane(vars, term, regex, ignore_case))
    }

    fn name_exists(&self, vars: &Vars, scope: NameScope, name: &str) -> bool {
        self.tree
            .is_some_and(|tree| tree.name_exists(vars, scope, name))
    }

    fn job(&self, command: &str, expanded: String, vars: &Vars) -> String {
        match self.jobs {
            Some(jobs) => jobs.run(command, expanded, vars),
            None => String::new(),
        }
    }

    fn loop_items(
        &self,
        kind: FormatLoopKind,
        flags: &str,
        _vars: &Vars,
    ) -> Option<Vec<FormatLoopItem>> {
        let items = self
            .loops?
            .items_in_scope(kind, flags, _vars)
            .into_iter()
            .map(|vars| {
                let active_key = match kind {
                    FormatLoopKind::Session => "session_active",
                    FormatLoopKind::Window => "window_active",
                    FormatLoopKind::Pane => "pane_active",
                    // tmux's client loop has no active branch: every client
                    // expands the one body it was given.
                    FormatLoopKind::Client => return FormatLoopItem { vars, active: false },
                };
                let active = vars.lookup(active_key).is_some_and(is_true);
                FormatLoopItem { vars, active }
            })
            .collect::<Vec<_>>();
        Some(items)
    }
}

/// The single-letter shorthand → variable-name mapping tmux exposes (`#S` is the
/// session name, `#I` the window index, …). Only the deterministic,
/// model-backed ones are modeled; volatile ones (`#H` host, `#T` pane title)
/// are intentionally omitted.
fn shorthand(c: u8) -> Option<&'static str> {
    match c {
        b'S' => Some("session_name"),
        b'H' => Some("host"),
        b'h' => Some("host_short"),
        b'I' => Some("window_index"),
        b'P' => Some("pane_index"),
        b'W' => Some("window_name"),
        b'D' => Some("pane_id"),
        b'F' => Some("window_flags"),
        b'T' => Some("pane_title"),
        _ => None,
    }
}

/// Expand a tmux format template against `vars`.
///
/// Grammar handled:
/// - `##` → a literal `#`.
/// - `#{name}` → the value of `name`, or empty if unset/unknown.
/// - `#{?cond,then,else}` → `then` if `cond` is truthy, else `else`. `cond` is a
///   bare variable name (or a nested `#{…}`); truthy means non-empty and not
///   `"0"`, matching tmux's `format_true`.
/// - `#{==:a,b}` / `#{!=:a,b}` → `"1"` or `"0"` from comparing the expansions of
///   `a` and `b`.
/// - `#{t:name}`, `#{t/p:name}`, and `#{t/f/FORMAT:name}` → local-time
///   rendering of an epoch-valued variable.
/// - `#S`, `#I`, `#P`, `#W`, `#D`, `#F` → the corresponding variable.
/// - any other `#x` → the `#` verbatim, then `x` processed normally.
/// - everything else is copied through unchanged.
pub fn expand(template: &str, vars: &Vars) -> String {
    expand_with(template, vars, None)
}

/// [`expand`], but with a [`LoopSource`] so `#{S:…}`/`#{W:…}`/`#{P:…}` loops can
/// enumerate the session tree. The plain [`expand`] passes `None` (loops then
/// expand to empty, as they would with nothing to iterate).
fn expand_with(template: &str, vars: &Vars, ls: Option<&dyn LoopSource>) -> String {
    let anchored = ls.map(AnchoredLoops);
    expand_with_context(
        template,
        vars,
        &BasicContext {
            loops: anchored.as_ref().map(|loops| loops as &dyn ScopedLoopSource),
            jobs: None,
            tree: None,
        },
    )
}

/// [`expand_with`], but able to start `#()` jobs in `jobs`.
pub(super) fn expand_with_jobs(
    template: &str,
    vars: &Vars,
    ls: Option<&dyn ScopedLoopSource>,
    jobs: Option<&dyn FormatJobs>,
    tree: Option<&dyn FormatTree>,
) -> String {
    expand_with_context(
        template,
        vars,
        &BasicContext {
            loops: ls,
            jobs,
            tree,
        },
    )
}

/// [`expand_with_jobs`], but applying current-time directives first.
///
/// tmux applies `strftime` before resolving `#{...}`, so percent sequences
/// introduced by variable values remain literal.
pub(super) fn expand_time_with_jobs(
    template: &str,
    vars: &Vars,
    ls: Option<&dyn ScopedLoopSource>,
    jobs: Option<&dyn FormatJobs>,
    tree: Option<&dyn FormatTree>,
) -> String {
    expand_time_with_context(
        template,
        vars,
        &BasicContext {
            loops: ls,
            jobs,
            tree,
        },
    )
}

pub(super) fn expand_time_with_jobs_verbose(
    template: &str,
    vars: &Vars,
    ls: Option<&dyn ScopedLoopSource>,
    jobs: Option<&dyn FormatJobs>,
    tree: Option<&dyn FormatTree>,
) -> (String, String) {
    let trace = std::cell::RefCell::new(Vec::new());
    let context = BasicContext {
        loops: ls,
        jobs,
        tree,
    };
    let time_expanded = expand_time_string(template);
    let expander = Expander {
        context: &context,
        trace: Some(&trace),
    };
    let result = expander.expand(&time_expanded, vars, 0);
    let mut trace_out = trace.into_inner().join("\n");
    if !trace_out.is_empty() {
        trace_out.push('\n');
    }
    (result, trace_out)
}

pub(super) fn expand_with_context(
    template: &str,
    vars: &Vars,
    context: &dyn FormatContext,
) -> String {
    Expander {
        context,
        trace: None,
    }
    .expand(template, vars, 0)
}

pub(super) fn expand_time_with_context(
    template: &str,
    vars: &Vars,
    context: &dyn FormatContext,
) -> String {
    Expander {
        context,
        trace: None,
    }
    .expand(&expand_time_string(template), vars, 0)
}

struct Expander<'a> {
    context: &'a dyn FormatContext,
    trace: Option<&'a std::cell::RefCell<Vec<String>>>,
}

impl Expander<'_> {
    fn lookup(&self, vars: &Vars, key: &str) -> Option<String> {
        self.context.lookup(vars, key)
    }

    fn lookup_variable(&self, vars: &Vars, key: &str) -> Option<String> {
        self.context.lookup_variable(vars, key)
    }

    fn expand(&self, template: &str, vars: &Vars, depth: usize) -> String {
        if depth >= 100 {
            return String::new();
        }
        if depth == 0 {
            if let Some(t) = self.trace {
                t.borrow_mut()
                    .push(format!("# expanding format: {template}"));
            }
        }
        let mut out = String::with_capacity(template.len());
        let bytes = template.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] != b'#' {
                let character = template[i..]
                    .chars()
                    .next()
                    .expect("format index is at a UTF-8 character boundary");
                out.push(character);
                i += character.len_utf8();
                continue;
            }
            // We're at a '#'. Look at the next byte.
            match bytes.get(i + 1) {
                Some(b'#') => {
                    if self.context.preserve_double_hash() {
                        out.push_str("##");
                    } else {
                        out.push('#');
                    }
                    i += 2;
                }
                Some(b',') | Some(b'}') => {
                    out.push(bytes[i + 1] as char);
                    i += 2;
                }
                Some(b'{') => {
                    // Find the matching (brace-aware) closing brace so nested
                    // `#{…}` inside conditionals/comparisons are captured whole.
                    if let Some(end) = find_close(bytes, i + 2) {
                        let content = &template[i + 2..end];
                        if let Some(t) = self.trace {
                            t.borrow_mut().push(format!("# found #{{}}: {content}"));
                        }
                        let evaluated = self.eval_directive(content, vars, depth + 1);
                        if let Some(t) = self.trace {
                            t.borrow_mut()
                                .push(format!("# replaced '{content}' with '{evaluated}'"));
                        }
                        out.push_str(&evaluated);
                        i = end + 1;
                    } else {
                        // Unterminated `#{` — emit the rest verbatim.
                        out.push_str(&template[i..]);
                        break;
                    }
                }
                Some(b'(') => {
                    if let Some(end) = find_job_close(bytes, i + 2) {
                        let command = &template[i + 2..end];
                        let expanded = self.expand(&strip_format_jobs(command), vars, depth + 1);
                        let output = self.context.job(command, expanded, vars);
                        out.push_str(&self.expand(&strip_format_jobs(&output), vars, depth + 1));
                        i = end + 1;
                    } else {
                        // An unterminated format job is invalid and contributes no
                        // text; the remainder belongs to that invalid expression.
                        break;
                    }
                }
                Some(&c) if shorthand(c).is_some() => {
                    let name = shorthand(c).expect("checked by guard");
                    if let Some(value) = self.lookup(vars, name) {
                        out.push_str(&value);
                    }
                    i += 2;
                }
                _ => {
                    // A lone '#' or an unmodeled shorthand: copy the '#' through.
                    out.push('#');
                    i += 1;
                }
            }
        }
        if depth == 0 {
            if let Some(t) = self.trace {
                t.borrow_mut().push(format!("# result is: {out}"));
            }
        }
        out
    }
}

/// Whether an expanded format string is "true" in tmux's sense (`format_true`):
/// non-empty and not the single character `0`. Used for `-f` row filters.
pub fn is_true(expanded: &str) -> bool {
    !expanded.is_empty() && expanded != "0"
}

/// Whether an expanded `if-shell -F` condition is "true" in tmux's sense: its
/// first byte is neither `0` nor NUL. Unlike [`is_true`], anything starting
/// with `0` — `"00"`, `"0abc"` — is false, while `" 0"` is true.
pub fn is_true_first_byte(expanded: &str) -> bool {
    !matches!(expanded.as_bytes().first(), None | Some(b'0'))
}

/// Evaluate the inside of a `#{…}` directive (the text between the braces).
impl Expander<'_> {
    fn eval_directive(&self, content: &str, vars: &Vars, depth: usize) -> String {
        // Loop modifiers: expand the body once per item, concatenating.
        for (prefix, kind) in [
            ('S', FormatLoopKind::Session),
            ('W', FormatLoopKind::Window),
            ('P', FormatLoopKind::Pane),
            ('L', FormatLoopKind::Client),
        ] {
            if let Some((flags, body)) = parse_loop(content, prefix) {
                return self.expand_loop(body, flags, vars, kind, depth);
            }
        }
        if let Some(rest) = content.strip_prefix("R:") {
            let parts = split_top_level(rest, b',');
            let value = self.expand(parts.first().map(String::as_str).unwrap_or(""), vars, depth);
            let count = self
                .expand(parts.get(1).map(String::as_str).unwrap_or(""), vars, depth)
                .parse::<usize>()
                .unwrap_or(0)
                .min(10_000);
            return value.repeat(count);
        }
        if let Some(rest) = content.strip_prefix('?') {
            return self.eval_conditional(rest, vars, depth);
        }
        if let Some(rest) = content.strip_prefix("==:") {
            return self.eval_comparison(rest, vars, depth, |o| o == std::cmp::Ordering::Equal);
        }
        if let Some(rest) = content.strip_prefix("!=:") {
            return self.eval_comparison(rest, vars, depth, |o| o != std::cmp::Ordering::Equal);
        }
        // Ordering comparisons — string (strcmp) comparison of the operands, matching
        // real tmux (`#{<:10,9}` is `1` because "10" < "9" lexically). `<=`/`>=` must
        // be checked before `<`/`>`.
        if let Some(rest) = content.strip_prefix("<=:") {
            return self.eval_comparison(rest, vars, depth, |o| o != std::cmp::Ordering::Greater);
        }
        if let Some(rest) = content.strip_prefix(">=:") {
            return self.eval_comparison(rest, vars, depth, |o| o != std::cmp::Ordering::Less);
        }
        if let Some(rest) = content.strip_prefix("<:") {
            return self.eval_comparison(rest, vars, depth, |o| o == std::cmp::Ordering::Less);
        }
        if let Some(rest) = content.strip_prefix(">:") {
            return self.eval_comparison(rest, vars, depth, |o| o == std::cmp::Ordering::Greater);
        }
        // Logical operators over the *truthiness* of the two operands.
        if let Some(rest) = content.strip_prefix("||:") {
            return self.eval_logical(rest, vars, depth, true);
        }
        if let Some(rest) = content.strip_prefix("&&:") {
            return self.eval_logical(rest, vars, depth, false);
        }
        if let Some(rest) = content.strip_prefix("!!:") {
            return bool01(is_true(&self.expand(rest, vars, depth)));
        }
        if let Some(rest) = content.strip_prefix("!:") {
            return bool01(!is_true(&self.expand(rest, vars, depth)));
        }
        if let Some((modifiers, name)) = split_modifier_key(content) {
            if modifiers
                .iter()
                .any(|modifier| *modifier == "E" || *modifier == "T")
            {
                // `E`/`T` re-expand what the body resolved to, and a body that
                // is itself a format is expanded on the way in — tmux's
                // "expanding inner format" step, which runs before the
                // modifier's own second pass.
                let mut value = self.resolve_body(name, vars, depth);
                value = self.expand(&value, vars, depth);
                if modifiers.contains(&"T") {
                    value = expand_time_string(&value);
                }
                for modifier in modifiers {
                    if let Some(limit) = modifier.strip_prefix("=/") {
                        let limit = self
                            .expand(limit, vars, depth)
                            .parse::<usize>()
                            .unwrap_or(0);
                        value = trim_left(&value, limit);
                    } else if let Some(limit) = modifier.strip_prefix("=|-") {
                        let limit = self
                            .expand(limit, vars, depth)
                            .parse::<usize>()
                            .unwrap_or(0);
                        value = trim_right(&value, limit);
                    }
                }
                return value;
            }
        }
        // `e|OP:a,b` — arithmetic (`+ - * / m %`) and numeric comparison
        // (`== != < > <= >=`) expressions.
        if let Some(rest) = content.strip_prefix("e|") {
            return self.eval_arith(rest, vars, depth);
        }
        // `m:pattern,string` — fnmatch-style glob match (`*`, `?`), returns "1"/"0".
        if let Some(rest) = content.strip_prefix("m:") {
            return self.eval_match(rest, vars, depth);
        }
        // `m<delimiter>FLAGS:pattern,string` — fnmatch or extended
        // regular-expression matching. tmux accepts any punctuation as the
        // delimiter, not just the documented slash.
        if let Some((flags, operands)) = parse_flagged_modifier(content, b'm') {
            return self.eval_match_with_flags(flags, operands, vars, depth);
        }
        // `t:VAR` / `t/p:VAR` / `t/f/FORMAT:VAR` — render an epoch timestamp
        // using tmux's default, compact, or caller-supplied local-time format.
        if let Some(modifier) = parse_time_modifier(content) {
            return self.eval_time(modifier, vars, depth);
        }
        // `n:BODY` — the length (in characters) of the resolved body.
        if let Some(rest) = content.strip_prefix("n:") {
            return self
                .resolve_body(rest, vars, depth)
                .chars()
                .count()
                .to_string();
        }
        // `C[/flags]:TERM` — the 1-based row of the target pane's visible screen
        // matching the expanded term, or 0. `i` folds case, `r` reads the term
        // as an extended regular expression.
        if let Some((flags, term)) = parse_flagged_modifier(content, b'C') {
            let term = self.expand(term, vars, depth);
            return self
                .context
                .search_pane(vars, &term, flags.contains('r'), flags.contains('i'))
                .to_string();
        }
        // `N[/w|/s]:NAME` — whether a window of that name exists in the
        // format's session, or a session of that name exists. tmux defaults to
        // the window scope.
        if let Some((flags, name)) = parse_flagged_modifier(content, b'N') {
            let name = self.expand(name, vars, depth);
            let scope = if flags.contains('s') && !flags.contains('w') {
                NameScope::Session
            } else {
                NameScope::Window
            };
            return bool01(self.context.name_exists(vars, scope, &name));
        }
        // `w:BODY` — the display width of the resolved body, using the same
        // codepoint-width policy as the terminal grid.
        if let Some(rest) = content.strip_prefix("w:") {
            return display_width(&self.resolve_body(rest, vars, depth)).to_string();
        }
        // `a:N` — the character whose ASCII/Unicode code point is N. Unlike `n:`, the
        // body is format-expanded (a bare number is a literal, not a variable).
        if let Some(rest) = content.strip_prefix("a:") {
            let n = self.expand(rest, vars, depth);
            return match n.parse::<u32>().ok().and_then(char::from_u32) {
                Some(c) => c.to_string(),
                None => String::new(),
            };
        }
        // `l:BODY` — literal: the body is emitted verbatim, *not* expanded.
        if let Some(rest) = content.strip_prefix("l:") {
            return rest.replace("##", "#");
        }
        // `b:BODY` / `d:BODY` — basename / dirname of the resolved body.
        if let Some(rest) = content.strip_prefix("b:") {
            return basename(&self.resolve_body(rest, vars, depth));
        }
        if let Some(rest) = content.strip_prefix("d:") {
            return dirname(&self.resolve_body(rest, vars, depth));
        }
        // `c:COLOUR` — force a tmux colour name, palette index, or RGB literal to
        // a six-digit lower-case RGB value.
        if let Some(rest) = content.strip_prefix("c:") {
            return colour_rgb(&self.expand(rest, vars, depth));
        }
        // `q[:/h|/e|/a]:BODY` — quote shell metacharacters, format-style
        // hashes, or a command argument in the resolved value.
        if let Some((style, body)) = parse_quote_modifier(content) {
            let Some(value) = self.resolve_body_opt(body, vars, depth) else {
                return String::new();
            };
            return match style {
                QuoteStyle::Shell => quote_shell(&value),
                QuoteStyle::Style => quote_style(&value),
                QuoteStyle::Arguments => quote_argument(&value),
            };
        }
        // A `;`-separated modifier list, which tmux's `format_replace` applies
        // in its own order: substitutions first, then the truncation, then the
        // padding — whatever order they were written in.
        if content.contains(';') {
            if let Some((items, body)) = split_modifier_chain(content) {
                if items.len() > 1 && items.iter().all(|item| chain_modifier(item).is_some()) {
                    let mut value = self.resolve_body(body, vars, depth);
                    for item in &items {
                        if let Some(ChainModifier::Substitute(step)) = chain_modifier(item) {
                            value =
                                substitute(&value, step.pattern, step.replacement, step.flags);
                        }
                    }
                    let limit = items.iter().rev().find_map(|item| match chain_modifier(item) {
                        Some(ChainModifier::Limit(limit)) => Some(limit),
                        _ => None,
                    });
                    if let Some(limit) = limit {
                        let limit = self.expand(limit, vars, depth).parse::<isize>().unwrap_or(0);
                        value = truncate(&value, limit, None);
                    }
                    let width = items.iter().rev().find_map(|item| match chain_modifier(item) {
                        Some(ChainModifier::Width(width)) => Some(width),
                        _ => None,
                    });
                    if let Some(width) = width {
                        let width = self.expand(width, vars, depth).parse::<isize>().unwrap_or(0);
                        value = pad(&value, width);
                    }
                    return value;
                }
            }
        }
        // `=N:BODY` / `=-N:BODY` / `=/N/marker:BODY` — truncate the resolved body
        // to N display columns from the start (N>0) or the end (N<0), optionally
        // appending or prepending a marker if truncation occurred.
        if let Some(tmod) = parse_truncate_mod(content) {
            let limit = self
                .expand(tmod.limit_str, vars, depth)
                .parse::<isize>()
                .unwrap_or(0);
            let marker = tmod.marker.map(|m| self.expand(m, vars, depth));
            let value = self.resolve_body(tmod.body, vars, depth);
            return truncate(&value, limit, marker.as_deref());
        }
        // `pN:BODY` / `p-N:BODY` — pad the resolved body to width N with spaces on
        // the right (N>0, left-justified) or the left (N<0, right-justified).
        if let Some((n, body)) = parse_count_mod(content, b'p') {
            return pad(&self.resolve_body(body, vars, depth), n);
        }
        // `s/pattern/replacement/[flags]:BODY` — substitute (all occurrences) in the
        // resolved body. Steps joined by `;` apply left to right.
        if let Some((steps, body)) = parse_subst(content) {
            let mut value = self.resolve_body(body, vars, depth);
            for step in steps {
                value = substitute(&value, step.pattern, step.replacement, step.flags);
            }
            return value;
        }
        // Plain variable lookup (unknown → empty).
        let val = self.lookup(vars, content);
        if let Some(t) = self.trace {
            if let Some(ref v) = val {
                t.borrow_mut()
                    .push(format!("# format '{content}' found: {v}"));
            } else {
                t.borrow_mut()
                    .push(format!("# format '{content}' not found"));
            }
        }
        val.unwrap_or_default()
    }
}

/// Resolve a modifier's body to a value, matching how tmux evaluates the string
/// after a `#{mod:…}` colon: a body containing a nested `#{…}` is expanded as a
/// template, while a bare word is looked up as a variable (unknown → empty).
/// This is why `#{b:/usr/local/bin}` yields empty (no such variable) but
/// `#{=3:session_name}` truncates the *value* of `session_name`.
impl Expander<'_> {
    fn resolve_body(&self, body: &str, vars: &Vars, depth: usize) -> String {
        self.resolve_body_opt(body, vars, depth).unwrap_or_default()
    }

    /// Resolve a modifier body while keeping "no such variable" distinct from
    /// "variable holds an empty value". tmux applies its quoting modifiers
    /// inside the variable lookup, so an unknown name never reaches the quoting
    /// step and the whole replacement expands empty rather than to a quoted
    /// empty string.
    fn resolve_body_opt(&self, body: &str, vars: &Vars, depth: usize) -> Option<String> {
        if body.contains("#{") {
            Some(self.expand(body, vars, depth))
        } else {
            self.lookup(vars, body)
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum TimeStyle {
    Default,
    Pretty,
    Custom(String),
}

#[derive(Debug, PartialEq, Eq)]
struct TimeModifier<'a> {
    style: TimeStyle,
    body: &'a str,
}

/// Parse tmux's time modifier forms. The documented forms use `/` as the
/// argument delimiter, but tmux accepts any punctuation delimiter.
fn parse_time_modifier(content: &str) -> Option<TimeModifier<'_>> {
    let rest = content.strip_prefix('t')?;
    if let Some(body) = rest.strip_prefix(':') {
        return Some(TimeModifier {
            style: TimeStyle::Default,
            body,
        });
    }

    let first = *rest.as_bytes().first()?;
    if first.is_ascii_alphanumeric() || first == b'_' || first == b'-' {
        let colon = find_unescaped(rest.as_bytes(), b':')?;
        return Some(TimeModifier {
            style: time_style(&[&rest[..colon]]),
            body: &rest[colon + 1..],
        });
    }
    if !first.is_ascii_punctuation() || matches!(first, b':' | b';') {
        return None;
    }

    let delimiter = first;
    let bytes = rest.as_bytes();
    let mut args = Vec::new();
    let mut start = 1;
    let mut index = 1;
    let mut depth = 0;
    while index < bytes.len() {
        if bytes[index] == b'#' && bytes.get(index + 1) == Some(&b'{') {
            depth += 1;
            index += 2;
            continue;
        }
        if bytes[index] == b'}' && depth > 0 {
            depth -= 1;
            index += 1;
            continue;
        }
        if bytes[index] == b'#'
            && bytes
                .get(index + 1)
                .is_some_and(|next| b",#{}:".contains(next))
        {
            index += 2;
            continue;
        }
        if depth == 0 && bytes[index] == b':' {
            if index > start {
                args.push(&rest[start..index]);
            }
            return Some(TimeModifier {
                style: time_style(&args),
                body: &rest[index + 1..],
            });
        }
        if depth == 0 && bytes[index] == delimiter {
            if index > start {
                args.push(&rest[start..index]);
            }
            start = index + 1;
        }
        index += 1;
    }
    None
}

fn time_style(args: &[&str]) -> TimeStyle {
    let flags = args.first().copied().unwrap_or_default();
    if flags.contains('p') {
        TimeStyle::Pretty
    } else if flags.contains('f') && args.len() >= 2 {
        TimeStyle::Custom(args[1].to_string())
    } else {
        TimeStyle::Default
    }
}

fn strip_time_format(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut output = String::with_capacity(value.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'#'
            && bytes
                .get(index + 1)
                .is_some_and(|next| b",#{}:".contains(next))
        {
            index += 1;
        }
        let character = value[index..]
            .chars()
            .next()
            .expect("time format index is at a UTF-8 character boundary");
        output.push(character);
        index += character.len_utf8();
    }
    output
}

fn find_unescaped(bytes: &[u8], needle: u8) -> Option<usize> {
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'#'
            && bytes
                .get(index + 1)
                .is_some_and(|next| b",#{}:".contains(next))
        {
            index += 2;
            continue;
        }
        if bytes[index] == needle {
            return Some(index);
        }
        index += 1;
    }
    None
}

impl Expander<'_> {
    fn eval_time(&self, modifier: TimeModifier<'_>, vars: &Vars, depth: usize) -> String {
        // tmux applies time modifiers through format_find(). A nested expression
        // bypasses that lookup and is only expanded, so preserve the same behavior.
        if modifier.body.contains("#{") {
            return self.expand(modifier.body, vars, depth);
        }
        let Some(value) = self.lookup_variable(vars, modifier.body) else {
            return String::new();
        };
        let Ok(timestamp) = value.parse::<i64>() else {
            return String::new();
        };
        if timestamp <= 0 {
            return String::new();
        }

        match modifier.style {
            TimeStyle::Default => ctime(timestamp),
            TimeStyle::Pretty => pretty_time(timestamp, now_epoch()),
            TimeStyle::Custom(format) => {
                let format = strip_time_format(&self.expand(&format, vars, depth));
                strftime_time(timestamp, &format)
            }
        }
    }
}

fn now_epoch() -> i64 {
    // SAFETY: time(NULL) reads the system clock and touches no memory.
    unsafe { libc::time(std::ptr::null_mut()) as i64 }
}

fn expand_time_string(value: &str) -> String {
    // `strftime` copies a string with no conversion in it verbatim, at the
    // cost of a `localtime` call and three allocations. Skipping it where
    // there is nothing to convert is not observable — the interior NUL that
    // `CString` rejects is the one input whose result is not the input, so it
    // stays on the slow path rather than being special-cased here.
    if !value.contains(['%', '\0']) {
        return value.to_string();
    }
    let Ok(format) = CString::new(value) else {
        return String::new();
    };
    unsafe {
        let now = libc::time(std::ptr::null_mut());
        let tm = libc::localtime(&now);
        if tm.is_null() {
            return String::new();
        }
        let mut size = value.len().saturating_mul(4).max(128);
        while size <= 65_536 {
            let mut buffer = vec![0 as libc::c_char; size];
            let written = libc::strftime(buffer.as_mut_ptr(), buffer.len(), format.as_ptr(), tm);
            if written != 0 {
                let bytes = buffer[..written]
                    .iter()
                    .map(|&byte| byte as u8)
                    .collect::<Vec<_>>();
                return String::from_utf8_lossy(&bytes).into_owned();
            }
            size *= 2;
        }
    }
    String::new()
}

fn ctime(timestamp: i64) -> String {
    let timestamp = match libc::time_t::try_from(timestamp) {
        Ok(timestamp) => timestamp,
        Err(_) => return String::new(),
    };
    let mut buffer = [0 as libc::c_char; 64];
    // SAFETY: timestamp and buffer are valid for ctime_r; the buffer is larger
    // than the 26 bytes required by ctime_r.
    let result = unsafe { libc::ctime_r(&timestamp, buffer.as_mut_ptr()) };
    if result.is_null() {
        return String::new();
    }
    // SAFETY: successful ctime_r writes a NUL-terminated string to buffer.
    unsafe { CStr::from_ptr(buffer.as_ptr()) }
        .to_string_lossy()
        .trim_end_matches('\n')
        .to_string()
}

fn local_time(timestamp: i64) -> Option<libc::tm> {
    let timestamp = libc::time_t::try_from(timestamp).ok()?;
    // SAFETY: localtime_r initializes `tm` from the valid timestamp pointer.
    unsafe {
        let mut tm = std::mem::zeroed::<libc::tm>();
        (!libc::localtime_r(&timestamp, &mut tm).is_null()).then_some(tm)
    }
}

fn strftime_time(timestamp: i64, format: &str) -> String {
    let tm = match local_time(timestamp) {
        Some(tm) => tm,
        None => return String::new(),
    };
    let format = match CString::new(format) {
        Ok(format) => format,
        Err(_) => return String::new(),
    };
    let mut buffer = [0 as libc::c_char; 512];
    // SAFETY: all pointers refer to initialized objects and strftime receives
    // the exact buffer capacity.
    let written =
        unsafe { libc::strftime(buffer.as_mut_ptr(), buffer.len(), format.as_ptr(), &tm) };
    if written == 0 {
        return String::new();
    }
    // SAFETY: strftime initialized the first `written` bytes.
    String::from_utf8_lossy(unsafe {
        std::slice::from_raw_parts(buffer.as_ptr().cast::<u8>(), written)
    })
    .into_owned()
}

/// Match tmux 3.7b's format_pretty_time() buckets.
fn pretty_time(timestamp: i64, now: i64) -> String {
    let now = now.max(timestamp);
    let age = now.saturating_sub(timestamp);
    let Some(now_tm) = local_time(now) else {
        return String::new();
    };
    let Some(tm) = local_time(timestamp) else {
        return String::new();
    };

    let format = if age < 24 * 60 * 60 {
        "%H:%M"
    } else if (tm.tm_year == now_tm.tm_year && tm.tm_mon == now_tm.tm_mon)
        || age < 28 * 24 * 60 * 60
    {
        "%a%d"
    } else if (tm.tm_year == now_tm.tm_year && tm.tm_mon < now_tm.tm_mon)
        || (tm.tm_year == now_tm.tm_year - 1 && tm.tm_mon > now_tm.tm_mon)
    {
        "%d%b"
    } else {
        "%h%y"
    };
    strftime_time(timestamp, format)
}

/// Basename of a path-like string (text after the last `/`). Matches tmux's
/// `b:` (which uses `basename(3)`); a trailing slash is ignored.
fn basename(s: &str) -> String {
    let trimmed = s.trim_end_matches('/');
    match trimmed.rsplit_once('/') {
        Some((_, base)) => base.to_string(),
        None => trimmed.to_string(),
    }
}

/// Dirname of a path-like string (text before the last `/`). Matches tmux's `d:`.
/// An empty input yields empty (tmux does not fold it to `.`).
fn dirname(s: &str) -> String {
    let trimmed = s.trim_end_matches('/');
    if trimmed.is_empty() {
        return String::new();
    }
    match trimmed.rsplit_once('/') {
        Some(("", _)) => "/".to_string(),
        Some((dir, _)) => dir.to_string(),
        None => ".".to_string(),
    }
}

/// Take the first `n` display columns (n>0) or last `-n` columns (n<0) of
/// `s`, appending or prepending `marker` if truncation occurred; `n==0` yields
/// the whole string.
fn truncate(s: &str, n: isize, marker: Option<&str>) -> String {
    if n == 0 {
        return s.to_string();
    }
    if n > 0 {
        let trimmed = trim_left(s, n.unsigned_abs());
        if let Some(marker) = marker {
            if trimmed != s {
                return format!("{trimmed}{marker}");
            }
        }
        trimmed
    } else {
        let trimmed = trim_right(s, n.unsigned_abs());
        if let Some(marker) = marker {
            if trimmed != s {
                return format!("{marker}{trimmed}");
            }
        }
        trimmed
    }
}

/// Take up to `limit` display columns from the left without splitting a
/// codepoint. A codepoint wider than the remaining space is omitted together
/// with everything after it, matching the format modifier's scalar boundary
/// behavior.
pub(crate) fn trim_left(s: &str, limit: usize) -> String {
    let tokens = display_tokens(s);
    let width: usize = tokens.clone().map(|(_, width)| width).sum();
    if width <= limit {
        return s.to_string();
    }

    let mut shown = 0usize;
    let mut selected = Vec::new();
    for token in tokens {
        if shown >= limit {
            break;
        }
        if shown + token.1 <= limit {
            selected.push(token);
        }
        shown += token.1;
    }
    serialize_truncated(&selected)
}

/// Take up to `limit` display columns from the right without splitting a
/// codepoint. Style directives are retained so the selected suffix has the same
/// active style state as the source.
pub(crate) fn trim_right(s: &str, limit: usize) -> String {
    let tokens = display_tokens(s);
    let width: usize = tokens.clone().map(|(_, width)| width).sum();
    if width <= limit {
        return s.to_string();
    }

    let skip = width - limit;
    let mut seen = 0usize;
    let mut selected = Vec::new();
    for token in tokens {
        if is_style_token(token) || seen >= skip {
            selected.push(token);
        }
        seen += token.1;
    }
    serialize_truncated(&selected)
}

/// Serialize display tokens after a truncation boundary. Adjacent visible hash
/// tokens need to remain independently escaped: concatenating an escaped `##`
/// token and a literal `#` as `###` would change how a later format pass groups
/// the run. tmux emits both as escaped pairs (`####`) in this case.
fn serialize_truncated(tokens: &[(&str, usize)]) -> String {
    let mut out = String::new();
    let mut index = 0;
    while index < tokens.len() {
        let is_hash = tokens[index].1 == 1 && (tokens[index].0 == "#" || tokens[index].0 == "##");
        if !is_hash {
            out.push_str(tokens[index].0);
            index += 1;
            continue;
        }
        let start = index;
        while index < tokens.len()
            && tokens[index].1 == 1
            && (tokens[index].0 == "#" || tokens[index].0 == "##")
        {
            index += 1;
        }
        if index - start == 1 {
            out.push_str(tokens[start].0);
        } else {
            out.push_str(&"##".repeat(index - start));
        }
    }
    out
}

pub(crate) fn display_width(s: &str) -> usize {
    display_tokens(s).map(|(_, width)| width).sum()
}

/// Tokenize format display content into borrowed source spans and widths.
///
/// Literal hashes are grouped as escaped `##` pairs and style directives are
/// zero-width. Ordinary codepoints use the terminal width policy in
/// [`hmux_vt::codepoint_width`].
pub(crate) fn display_tokens(s: &str) -> DisplayTokens<'_> {
    DisplayTokens {
        source: s,
        index: 0,
    }
}

/// A cloneable, allocation-free traversal of format display content.
///
/// Cloning retains the current cursor, allowing width-sensitive consumers to
/// make a sizing pass before consuming the same spans.
#[derive(Clone, Debug)]
pub(crate) struct DisplayTokens<'a> {
    source: &'a str,
    index: usize,
}

impl<'a> Iterator for DisplayTokens<'a> {
    type Item = (&'a str, usize);

    fn next(&mut self) -> Option<Self::Item> {
        let bytes = self.source.as_bytes();
        if self.index >= bytes.len() {
            return None;
        }

        if bytes[self.index] != b'#' {
            let ch = self.source[self.index..]
                .chars()
                .next()
                .expect("index is at a UTF-8 character boundary");
            let end = self.index + ch.len_utf8();
            let token = (
                &self.source[self.index..end],
                codepoint_width(ch as u32) as usize,
            );
            self.index = end;
            return Some(token);
        }

        if bytes.get(self.index + 1) == Some(&b'#') {
            let start = self.index;
            self.index += 2;
            return Some((&self.source[start..self.index], 1));
        }

        if bytes.get(self.index + 1) == Some(&b'[') {
            if let Some(close) = self.source[self.index + 2..].find(']') {
                let start = self.index;
                let end = self.index + close + 3;
                self.index = end;
                return Some((&self.source[start..end], 0));
            }
        }

        let start = self.index;
        self.index += 1;
        Some((&self.source[start..self.index], 1))
    }
}

impl std::iter::FusedIterator for DisplayTokens<'_> {}

fn is_style_token((raw, width): (&str, usize)) -> bool {
    width == 0 && raw.starts_with("#[") && raw.ends_with(']')
}

/// Pad `s` to width `|n|` with spaces: on the right for n>0 (left-justified) or
/// the left for n<0 (right-justified). A string already at least that wide is
/// returned unchanged.
fn pad(s: &str, n: isize) -> String {
    let width = n.unsigned_abs();
    let len = display_width(s);
    if len >= width {
        return s.to_string();
    }
    let spaces = " ".repeat(width - len);
    if n >= 0 {
        format!("{s}{spaces}")
    } else {
        format!("{spaces}{s}")
    }
}

/// Parse a `<prefix>[-]<digits>:BODY` modifier (used by `=` truncate and `p`
/// pad). Returns the signed count and the body after the `:`. `None` if the
/// content doesn't match this shape (so a variable that merely starts with the
/// prefix letter, like `pane_index`, falls through to a plain lookup).
/// Parse a `X[<sep>flags]:BODY` modifier: the leading letter, an optional
/// argument introduced by a punctuation separator, and the body after the
/// colon. tmux's `format_build_modifiers` shape, for the modifiers whose one
/// argument is a set of flag letters.
fn parse_flagged_modifier(content: &str, prefix: u8) -> Option<(&str, &str)> {
    let bytes = content.as_bytes();
    if bytes.first() != Some(&prefix) {
        return None;
    }
    if bytes.get(1) == Some(&b':') {
        return Some(("", &content[2..]));
    }
    if !bytes.get(1)?.is_ascii_punctuation() {
        return None;
    }
    let colon = content[2..].find(':')? + 2;
    Some((&content[2..colon], &content[colon + 1..]))
}

struct TruncateMod<'a> {
    limit_str: &'a str,
    marker: Option<&'a str>,
    body: &'a str,
}

fn parse_truncate_mod(content: &str) -> Option<TruncateMod<'_>> {
    let bytes = content.as_bytes();
    if bytes.first() != Some(&b'=') || bytes.len() < 2 {
        return None;
    }
    let second = bytes[1];
    if second == b'-' || second.is_ascii_digit() {
        let neg = second == b'-';
        let digits_start = if neg { 2 } else { 1 };
        let mut i = digits_start;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i == digits_start || bytes.get(i) != Some(&b':') {
            return None;
        }
        return Some(TruncateMod {
            limit_str: &content[1..i],
            marker: None,
            body: &content[i + 1..],
        });
    }

    if !second.is_ascii_punctuation() || second == b':' || second == b';' {
        return None;
    }

    let delim = second;
    let mut args = Vec::new();
    let mut cp = 1;
    while cp < bytes.len() && bytes[cp] != b':' && bytes[cp] != b';' {
        if bytes[cp] == delim && bytes.get(cp + 1).is_some_and(|&b| b == b':' || b == b';') {
            cp += 1;
            break;
        }
        if bytes[cp] == delim {
            cp += 1;
        }
        let start = cp;
        let mut depth = 0usize;
        let mut end = start;
        while end < bytes.len() {
            if bytes[end] == b'#' && bytes.get(end + 1) == Some(&b'{') {
                depth += 1;
                end += 2;
                continue;
            }
            if bytes[end] == b'}' && depth > 0 {
                depth -= 1;
                end += 1;
                continue;
            }
            if depth == 0 && (bytes[end] == delim || bytes[end] == b':' || bytes[end] == b';') {
                break;
            }
            end += 1;
        }
        args.push(&content[start..end]);
        cp = end;
    }

    if bytes.get(cp) != Some(&b':') {
        return None;
    }

    let limit_str = args.first().copied()?;
    let marker = args.get(1).copied();
    let body = &content[cp + 1..];

    Some(TruncateMod {
        limit_str,
        marker,
        body,
    })
}

fn parse_count_mod(content: &str, prefix: u8) -> Option<(isize, &str)> {
    let bytes = content.as_bytes();
    if bytes.first() != Some(&prefix) {
        return None;
    }
    let mut i = 1;
    let mut delim = None;
    // tmux's `format_build_modifiers` lets a modifier's arguments follow a
    // separator of their own — any punctuation, which is what spells the
    // documented `=/N` beside the bare `=N`.
    if bytes
        .get(i)
        .is_some_and(|byte| byte.is_ascii_punctuation() && !matches!(byte, b'-' | b':'))
    {
        delim = bytes.get(i).copied();
        i += 1;
    }
    let negative = bytes.get(i) == Some(&b'-');
    if negative {
        i += 1;
    }
    let digits_start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == digits_start {
        return None; // no digits
    }
    let n: isize = content[digits_start..i].parse().ok()?;
    let n = if negative { -n } else { n };
    if let Some(d) = delim {
        if bytes.get(i) == Some(&d) {
            i += 1;
        }
    }
    if bytes.get(i) != Some(&b':') {
        return None;
    }
    Some((n, &content[i + 1..]))
}

/// One `s<delim>pattern<delim>replacement<delim>[flags]` step of a
/// substitution modifier.
struct Subst<'a> {
    pattern: &'a str,
    replacement: &'a str,
    flags: &'a str,
}

/// Parse a chain of `s<delim>pattern<delim>replacement<delim>[flags]` steps
/// separated by `;` and terminated by `:BODY`. Returns the steps in the order
/// they apply plus the body; `None` if the shape doesn't match (so
/// `session_name` etc. fall through to a plain lookup).
fn parse_subst(content: &str) -> Option<(Vec<Subst<'_>>, &str)> {
    let mut steps = Vec::new();
    let mut rest = content;
    loop {
        if !starts_subst(rest) {
            if steps.is_empty() {
                return None;
            }
            // A chain that mixes in other modifiers (`s/o/0/;=3:BODY`): hmux
            // does not model modifier lists yet, so drop the rest of the
            // prefix instead of failing the whole expansion.
            let colon = rest.find(':')?;
            return Some((steps, &rest[colon + 1..]));
        }
        let delim = rest.as_bytes()[1];
        let mut parts = rest[2..].splitn(3, delim as char);
        let pattern = parts.next()?;
        let replacement = parts.next()?;
        let tail = parts.next()?; // "[flags]" then `;` (another step) or `:BODY`
        let stop = tail.find([';', ':'])?;
        steps.push(Subst {
            pattern,
            replacement,
            flags: &tail[..stop],
        });
        if tail.as_bytes()[stop] == b':' {
            return Some((steps, &tail[stop + 1..]));
        }
        rest = &tail[stop + 1..];
    }
}

/// Whether `content` opens an `s<delim>…` substitution. The delimiter must be
/// ASCII punctuation: not part of a variable name like `session_name` (whose
/// second byte is a letter), and not a byte of a multibyte character, which
/// would put the split off a character boundary.
fn starts_subst(content: &str) -> bool {
    let bytes = content.as_bytes();
    bytes.first() == Some(&b's')
        && bytes.get(1).is_some_and(|delim| {
            delim.is_ascii() && !delim.is_ascii_alphanumeric() && *delim != b'_'
        })
}

/// Substitutes every match of `pattern` in `text`, following tmux's `regsub`
/// match loop rather than the regex crate's `replace_all`. The two disagree on
/// the edges the conformance suite pins: an empty pattern is a no-op, an empty
/// match only expands once the scan has moved past the previous match, and a
/// `^`-anchored pattern substitutes at most once.
///
/// The pattern itself is still the regex crate's dialect, not POSIX ERE, so
/// alternation picks the leftmost-first branch where tmux takes the longest.
fn substitute(text: &str, pattern: &str, replacement: &str, flags: &str) -> String {
    // tmux short-circuits both empty inputs before compiling: an empty pattern
    // leaves the value untouched instead of matching at every boundary.
    if text.is_empty() || pattern.is_empty() {
        return text.to_string();
    }
    let regex = match RegexBuilder::new(pattern)
        .case_insensitive(flags.contains('i'))
        .build()
    {
        Ok(regex) => regex,
        // A pattern that will not compile leaves the value untouched, matching
        // `format_sub`'s handling of a NULL `regsub` result.
        Err(_) => return text.to_string(),
    };

    let end = text.len();
    // `start` is where the next match is searched from, `last` the end of the
    // last expanded match — the text between them is copied through verbatim.
    let (mut start, mut last) = (0usize, 0usize);
    let mut empty = false;
    let mut out = String::with_capacity(end);

    while start <= end {
        let Some(captures) = regex.captures(&text[start..]) else {
            // Like tmux, resume the copy at `start`, not at `last`.
            out.push_str(&text[start..end]);
            break;
        };
        let whole = captures.get(0).expect("capture 0 always participates");
        out.push_str(&text[last..start + whole.start()]);

        if empty || start + whole.start() != last || !whole.is_empty() {
            expand_replacement(&mut out, replacement, &captures);
            last = start + whole.end();
            start = last;
            empty = false;
        } else {
            // An empty match butted against the previous one is skipped: move
            // on one character and expand it on the next pass instead. tmux
            // steps one byte, which here has to be one character to keep the
            // slices on UTF-8 boundaries.
            last = start + whole.end();
            start = next_char_boundary(text, last);
            empty = true;
        }

        // A `^`-anchored pattern would otherwise re-anchor at every restart, so
        // tmux stops after the first match and copies the remainder.
        if pattern.starts_with('^') {
            out.push_str(&text[start.min(end)..end]);
            break;
        }
    }
    out
}

/// Appends `replacement` with tmux's `regsub_expand` escape rules: a backslash
/// is always dropped, and `\N` expands the Nth capture only when that group
/// participated *and* matched something — an unmatched or empty group leaves
/// the bare digit behind.
fn expand_replacement(out: &mut String, replacement: &str, captures: &regex::Captures<'_>) {
    let mut chars = replacement.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '\\' || chars.peek().is_none() {
            out.push(ch);
            continue;
        }
        let escaped = chars.next().expect("peeked escaped character");
        if let Some(index) = escaped.to_digit(10) {
            if let Some(group) = captures.get(index as usize) {
                if !group.is_empty() {
                    out.push_str(group.as_str());
                    continue;
                }
            }
        }
        out.push(escaped);
    }
}

/// The offset one character past `index`, or `index + 1` at the end of `text`
/// (which only has to leave the scan loop).
fn next_char_boundary(text: &str, index: usize) -> usize {
    text[index..]
        .chars()
        .next()
        .map_or(index + 1, |ch| index + ch.len_utf8())
}

fn quote_shell(value: &str) -> String {
    const SPECIAL: &str = "|&;<>()$`\\\"'*?[# =%";
    let mut quoted = String::with_capacity(value.len());
    for ch in value.chars() {
        if SPECIAL.contains(ch) {
            quoted.push('\\');
        }
        quoted.push(ch);
    }
    quoted
}

fn quote_style(value: &str) -> String {
    let mut quoted = String::with_capacity(value.len());
    for character in value.chars() {
        if character == '#' {
            quoted.push('#');
        }
        quoted.push(character);
    }
    quoted
}

/// Escape a value for reuse as a command argument, following tmux's
/// `args_escape`: pick the quoting form from the characters present, then
/// visually encode the rest with the C-style escapes `vis(3)` recognises and
/// three-digit octal for every other control byte.
fn quote_argument(value: &str) -> String {
    if value.is_empty() {
        return "''".to_string();
    }

    let quote = if value
        .chars()
        .any(|character| " #';${}%".contains(character))
    {
        Some('"')
    } else if value.chars().any(|character| " \"".contains(character)) {
        Some('\'')
    } else {
        None
    };

    if value.len() == 1 && value != " " && (quote.is_some() || value == "~") {
        return format!("\\{value}");
    }

    let mut escaped = String::with_capacity(value.len());
    let mut characters = value.chars().peekable();
    while let Some(character) = characters.next() {
        match character {
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\u{8}' => escaped.push_str("\\b"),
            '\u{7}' => escaped.push_str("\\a"),
            '\u{b}' => escaped.push_str("\\v"),
            '\u{c}' => escaped.push_str("\\f"),
            '\\' => escaped.push_str("\\\\"),
            '"' if quote == Some('"') => escaped.push_str("\\\""),
            '$' if quote == Some('"')
                && characters.peek().is_some_and(|next| {
                    next.is_ascii_alphabetic() || *next == '_' || *next == '{'
                }) =>
            {
                escaped.push_str("\\$");
            }
            character if character.is_ascii_control() => {
                escaped.push_str(&format!("\\{:03o}", character as u32));
            }
            character => escaped.push(character),
        }
    }

    match quote {
        Some(quote) => format!("{quote}{escaped}{quote}"),
        None if escaped.starts_with('~') => format!("\\{escaped}"),
        None => escaped,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum QuoteStyle {
    Shell,
    Style,
    Arguments,
}

fn parse_quote_modifier(content: &str) -> Option<(QuoteStyle, &str)> {
    if let Some(body) = content.strip_prefix("q:") {
        return Some((QuoteStyle::Shell, body));
    }

    let rest = content.strip_prefix("q/")?;
    let (flags, body) = rest.split_once(':')?;
    let style = if flags.contains('e') || flags.contains('h') {
        QuoteStyle::Style
    } else if flags.contains('a') {
        QuoteStyle::Arguments
    } else {
        return None;
    };
    Some((style, body))
}

fn colour_rgb(value: &str) -> String {
    let Some(colour) = parse_colour(value) else {
        return String::new();
    };
    let (red, green, blue) = match colour {
        Colour::Default => return String::new(),
        Colour::Rgb(red, green, blue) => (red, green, blue),
        Colour::Palette(index) | Colour::Indexed(index) => palette_rgb(index),
    };
    format!("{red:02x}{green:02x}{blue:02x}")
}

fn palette_rgb(index: u8) -> (u8, u8, u8) {
    const ANSI: [(u8, u8, u8); 16] = [
        (0x00, 0x00, 0x00),
        (0x80, 0x00, 0x00),
        (0x00, 0x80, 0x00),
        (0x80, 0x80, 0x00),
        (0x00, 0x00, 0x80),
        (0x80, 0x00, 0x80),
        (0x00, 0x80, 0x80),
        (0xc0, 0xc0, 0xc0),
        (0x80, 0x80, 0x80),
        (0xff, 0x00, 0x00),
        (0x00, 0xff, 0x00),
        (0xff, 0xff, 0x00),
        (0x00, 0x00, 0xff),
        (0xff, 0x00, 0xff),
        (0x00, 0xff, 0xff),
        (0xff, 0xff, 0xff),
    ];
    if index < 16 {
        return ANSI[usize::from(index)];
    }
    if index < 232 {
        const LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];
        let cube = index - 16;
        return (
            LEVELS[usize::from(cube / 36)],
            LEVELS[usize::from((cube / 6) % 6)],
            LEVELS[usize::from(cube % 6)],
        );
    }
    let grey = 8 + (index - 232) * 10;
    (grey, grey, grey)
}

/// `?cond,then,else`: pick a branch on the truthiness of `cond`.
impl Expander<'_> {
    fn eval_conditional(&self, rest: &str, vars: &Vars, depth: usize) -> String {
        let parts = split_top_level(rest, b',');
        if parts.len() == 1 {
            return self.expand(parts.first().map(String::as_str).unwrap_or(""), vars, depth);
        }
        let mut index = 0;
        while index + 1 < parts.len() {
            if self.cond_is_true(&parts[index], vars, depth) {
                return self.expand(&parts[index + 1], vars, depth);
            }
            index += 2;
        }
        if index < parts.len() {
            self.expand(&parts[index], vars, depth)
        } else {
            String::new()
        }
    }

    /// `a,b` for the comparison operators (`==`/`!=`/`<`/`>`/`<=`/`>=`): compare the
    /// *expansions* of `a` and `b` with `strcmp` semantics and reduce the resulting
    /// [`std::cmp::Ordering`] to `"1"`/`"0"` via `keep`.
    fn eval_comparison(
        &self,
        rest: &str,
        vars: &Vars,
        depth: usize,
        keep: fn(std::cmp::Ordering) -> bool,
    ) -> String {
        let parts = split_top_level(rest, b',');
        if parts.len() != 2 {
            return String::new();
        }
        let a = self.expand(parts.first().map(String::as_str).unwrap_or(""), vars, depth);
        let b = self.expand(parts.get(1).map(String::as_str).unwrap_or(""), vars, depth);
        bool01(keep(a.cmp(&b)))
    }

    /// `a,b` for `||`/`&&`: combine the *truthiness* of the operands, stopping
    /// at the one that decides the result. tmux's `format_choose_state` leaves
    /// the rest unexpanded, so a `#()` operand behind a decided `||` never
    /// starts its job.
    fn eval_logical(&self, rest: &str, vars: &Vars, depth: usize, is_or: bool) -> String {
        let parts = split_top_level(rest, b',');
        let mut result = !is_or;
        for part in &parts {
            let value = is_true(&self.expand(part, vars, depth));
            if is_or {
                result |= value;
            } else {
                result &= value;
            }
            if result == is_or {
                break;
            }
        }
        bool01(result)
    }

    /// `OP[|FLAGS[|PRECISION]]:a,b` (following the `e|` prefix): arithmetic and
    /// numeric comparison over two operands. Operands are format-expanded then
    /// parsed as numbers; without the `f` flag both operands are truncated to
    /// integers before the operator runs and the result prints as an integer,
    /// with it the whole computation is floating point. The precision argument
    /// applies in either mode (`f` alone implies 2). A parse error, wrong
    /// operand count, unknown operator, or invalid precision yields empty,
    /// matching real tmux.
    fn eval_arith(&self, rest: &str, vars: &Vars, depth: usize) -> String {
        let (spec, body) = match rest.split_once(':') {
            Some(x) => x,
            None => return String::new(),
        };
        let mut spec_parts = spec.split('|');
        let op = spec_parts.next().unwrap_or(spec);
        let flags = spec_parts.next().unwrap_or_default();
        let use_fp = flags.contains('f');
        let precision = match spec_parts.next() {
            Some(text) => match text.parse::<i64>() {
                // tmux accepts -100..=100 and hands the value to printf's
                // `%.*f`, where a negative precision means the default of 6.
                Ok(value) if (-100..=100).contains(&value) => {
                    if value < 0 {
                        6
                    } else {
                        value as usize
                    }
                }
                _ => return String::new(),
            },
            None => {
                if use_fp {
                    2
                } else {
                    0
                }
            }
        };
        let parts = split_top_level(body, b',');
        if parts.len() != 2 {
            return String::new();
        }
        // strtod semantics: an empty operand is zero, anything else must parse
        // fully as a number.
        let operand = |text: &String| -> Option<f64> {
            let expanded = self.expand(text, vars, depth);
            let trimmed = expanded.trim();
            if trimmed.is_empty() {
                Some(0.0)
            } else {
                trimmed.parse().ok()
            }
        };
        let (Some(mut a), Some(mut b)) = (operand(&parts[0]), operand(&parts[1])) else {
            return String::new();
        };
        if !use_fp {
            a = a.trunc();
            b = b.trunc();
        }
        // Division/modulo by zero: real tmux computes with a long double, so the
        // result is a NaN/inf that casts to `INT64_MIN`. We reproduce that exact
        // value rather than erroring, so conformance matches.
        let truth = |value: bool| if value { 1.0 } else { 0.0 };
        let result = match op {
            "+" => a + b,
            "-" => a - b,
            "*" => a * b,
            "/" => {
                if b == 0.0 {
                    return i64::MIN.to_string();
                }
                a / b
            }
            "m" | "%" => {
                if b == 0.0 {
                    return i64::MIN.to_string();
                }
                a % b
            }
            // Equality within tmux's 1e-9 epsilon.
            "==" => truth((a - b).abs() < 1e-9),
            "!=" => truth((a - b).abs() > 1e-9),
            ">" => truth(a > b),
            "<" => truth(a < b),
            ">=" => truth(a >= b),
            "<=" => truth(a <= b),
            _ => return String::new(),
        };
        if use_fp {
            format!("{result:.precision$}")
        } else {
            format!("{:.precision$}", (result as i64) as f64)
        }
    }

    /// `pattern,string` for `m:`: fnmatch-style glob (supports `*` and `?`), returns
    /// `"1"` on match else `"0"`. Both operands are expanded first.
    fn eval_match(&self, rest: &str, vars: &Vars, depth: usize) -> String {
        let parts = split_top_level(rest, b',');
        let pattern = self.expand(parts.first().map(String::as_str).unwrap_or(""), vars, depth);
        let text = self.expand(parts.get(1).map(String::as_str).unwrap_or(""), vars, depth);
        bool01(glob_match(pattern.as_bytes(), text.as_bytes()))
    }

    fn eval_match_with_flags(
        &self,
        flags: &str,
        operands: &str,
        vars: &Vars,
        depth: usize,
    ) -> String {
        let parts = split_top_level(operands, b',');
        let pattern = self.expand(parts.first().map(String::as_str).unwrap_or(""), vars, depth);
        let text = self.expand(parts.get(1).map(String::as_str).unwrap_or(""), vars, depth);
        if flags.contains('r') {
            let matched = RegexBuilder::new(&pattern)
                .case_insensitive(flags.contains('i'))
                .build()
                .is_ok_and(|regex| regex.is_match(&text));
            bool01(matched)
        } else {
            let (pattern, text) = if flags.contains('i') {
                (pattern.to_lowercase(), text.to_lowercase())
            } else {
                (pattern, text)
            };
            bool01(glob_match(pattern.as_bytes(), text.as_bytes()))
        }
    }
}

/// Minimal fnmatch: `*` matches any run (including empty), `?` any single byte,
/// everything else literal. Enough for the `#{m:...}` cases the suite exercises,
/// and for the `fnmatch` step of target resolution.
pub(super) fn glob_match(pat: &[u8], text: &[u8]) -> bool {
    // Classic iterative backtracking matcher.
    let (mut p, mut t) = (0, 0);
    let (mut star, mut mark): (Option<usize>, usize) = (None, 0);
    while t < text.len() {
        let single = match pat.get(p) {
            Some(b'?') => Some(p + 1),
            Some(b'[') => match match_bracket(pat, p, text[t]) {
                Some((matched, next)) => matched.then_some(next),
                // fnmatch treats an unterminated bracket expression as a
                // literal opening bracket.
                None if text[t] == b'[' => Some(p + 1),
                None => None,
            },
            Some(&byte) if byte == text[t] => Some(p + 1),
            _ => None,
        };
        if let Some(next) = single {
            p = next;
            t += 1;
        } else if pat.get(p) == Some(&b'*') {
            star = Some(p);
            mark = t;
            p += 1;
        } else if let Some(sp) = star {
            p = sp + 1;
            mark += 1;
            t = mark;
        } else {
            return false;
        }
    }
    while p < pat.len() && pat[p] == b'*' {
        p += 1;
    }
    p == pat.len()
}

/// Match one character against the fnmatch bracket expression starting at
/// `start`. Returns whether it matched and the index just past the closing
/// `]`, or `None` when the brackets are unterminated — in which case fnmatch
/// treats the `[` as an ordinary character, which the caller's byte compare
/// has already ruled out.
fn match_bracket(pat: &[u8], start: usize, character: u8) -> Option<(bool, usize)> {
    let mut index = start + 1;
    let negated = matches!(pat.get(index), Some(b'!' | b'^'));
    if negated {
        index += 1;
    }
    let mut matched = false;
    let mut first = true;
    loop {
        match pat.get(index) {
            None => return None,
            // A `]` right after the bracket (or its negation) is a literal.
            Some(b']') if !first => break,
            Some(b'[') if pat.get(index + 1) == Some(&b':') => {
                let rest = &pat[index + 2..];
                let end = rest
                    .windows(2)
                    .position(|pair| pair == b":]")
                    .map(|offset| index + 2 + offset)?;
                let name = std::str::from_utf8(&pat[index + 2..end]).ok()?;
                matched |= character_class_matches(name, character);
                index = end + 2;
            }
            Some(&low) => {
                // `a-z`, unless the `-` is the last character before the `]`.
                if pat.get(index + 1) == Some(&b'-')
                    && pat.get(index + 2).is_some_and(|byte| *byte != b']')
                {
                    let high = pat[index + 2];
                    matched |= (low..=high).contains(&character);
                    index += 3;
                } else {
                    matched |= low == character;
                    index += 1;
                }
            }
        }
        first = false;
    }
    Some((matched != negated, index + 1))
}

/// The POSIX character classes fnmatch accepts inside a bracket expression.
fn character_class_matches(name: &str, character: u8) -> bool {
    match name {
        "alnum" => character.is_ascii_alphanumeric(),
        "alpha" => character.is_ascii_alphabetic(),
        "blank" => matches!(character, b' ' | b'\t'),
        "cntrl" => character.is_ascii_control(),
        "digit" => character.is_ascii_digit(),
        "graph" => character.is_ascii_graphic(),
        "lower" => character.is_ascii_lowercase(),
        "print" => character.is_ascii_graphic() || character == b' ',
        "punct" => character.is_ascii_punctuation(),
        "space" => character.is_ascii_whitespace(),
        "upper" => character.is_ascii_uppercase(),
        "xdigit" => character.is_ascii_hexdigit(),
        _ => false,
    }
}

/// Expand a loop body once per item of `kind`, concatenating the results. Each
/// item's body is expanded in that item's own context. Nested loops inside the
/// body aren't supported (the body expands without a loop source) — enough for
/// the status-line-style loops the conformance suite exercises.
impl Expander<'_> {
    fn expand_loop(
        &self,
        body: &str,
        flags: &str,
        vars: &Vars,
        kind: FormatLoopKind,
        depth: usize,
    ) -> String {
        let Some(mut items) = self.context.loop_items(kind, flags, vars) else {
            return String::new();
        };
        let branches = split_top_level(body, b',');
        let count = items.len();
        let mut output = String::new();
        for (index, item) in items.iter_mut().enumerate() {
            item.vars
                .set("loop_index", index.to_string())
                .set("loop_last_flag", if index + 1 == count { "1" } else { "0" });
            let branch = if item.active {
                branches.get(1).or(branches.first())
            } else {
                branches.first()
            };
            if let Some(branch) = branch {
                output.push_str(&self.expand(branch, &item.vars, depth));
            }
        }
        output
    }

    fn cond_is_true(&self, cond: &str, vars: &Vars, depth: usize) -> bool {
        let value = if cond.contains('#') {
            self.expand(cond, vars, depth)
        } else {
            self.lookup(vars, cond).unwrap_or_default()
        };
        if cond.contains('#') && value == cond {
            return false;
        }
        is_true(&value)
    }
}

/// tmux's boolean rendering: `"1"` for true, `"0"` for false.
fn bool01(b: bool) -> String {
    if b { "1" } else { "0" }.to_string()
}

/// tmux's `format_true`: a condition is true when its value is non-empty and not
/// the single character `0`. A bare `cond` is a variable name; a `cond`
/// containing `#` is itself expanded first (so `#{?#{window_active},…}` works).
/// Split `s` on top-level `sep` bytes, treating text inside `#{…}` as opaque
/// (its commas don't split). Used to separate the arms of a conditional or the
/// operands of a comparison without breaking on commas in nested formats.
fn split_top_level(s: &str, sep: u8) -> Vec<String> {
    let bytes = s.as_bytes();
    let mut parts = Vec::new();
    let mut start = 0;
    let mut depth = 0usize;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'#' && bytes.get(i + 1) == Some(&b'{') && !hash_escaped(bytes, i) {
            depth += 1;
            i += 2;
            continue;
        }
        if bytes[i] == b'}' && depth > 0 {
            depth -= 1;
            i += 1;
            continue;
        }
        if bytes[i] == sep && depth == 0 && !hash_escaped(bytes, i) {
            parts.push(s[start..i].to_string());
            start = i + 1;
        }
        i += 1;
    }
    parts.push(s[start..].to_string());
    parts
}

/// Split a loop directive into its sort flags and its body.
///
/// tmux writes the flags straight after the letter — `#{Wr:…}` — as the
/// "single argument with no wrapper character" case of its modifier parser;
/// the wrapped `#{W/r/:…}` form is accepted too.
fn parse_loop(content: &str, kind: char) -> Option<(&str, &str)> {
    let rest = content.strip_prefix(kind)?;
    if let Some(body) = rest.strip_prefix(':') {
        return Some(("", body));
    }
    if let Some(rest) = rest.strip_prefix('/') {
        let (flags, body) = rest.split_once(':')?;
        return Some((flags.trim_end_matches('/'), body));
    }
    let (flags, body) = rest.split_once(':')?;
    // Only a run of sort letters can precede the colon; anything else is some
    // other directive that happens to start with the same letter.
    flags
        .chars()
        .all(|flag| matches!(flag, 'i' | 'n' | 't' | 'r'))
        .then_some((flags, body))
}

/// One entry of a `;`-separated modifier list.
enum ChainModifier<'a> {
    Substitute(Subst<'a>),
    Limit(&'a str),
    Width(&'a str),
}

/// The modifiers a chain may carry, as tmux's `format_build_modifiers` reads
/// them. A modifier this does not recognize leaves the chain to the
/// single-modifier paths, which is where the rest of them live.
fn chain_modifier(item: &str) -> Option<ChainModifier<'_>> {
    if starts_subst(item) {
        let delim = item.as_bytes()[1] as char;
        let mut parts = item[2..].splitn(3, delim);
        return Some(ChainModifier::Substitute(Subst {
            pattern: parts.next()?,
            replacement: parts.next()?,
            flags: parts.next()?,
        }));
    }
    let count = |rest: &str| -> bool {
        let rest = rest.strip_prefix('-').unwrap_or(rest);
        !rest.is_empty() && rest.bytes().all(|byte| byte.is_ascii_digit())
    };
    if let Some(rest) = item.strip_prefix('=') {
        return count(rest).then_some(ChainModifier::Limit(rest));
    }
    if let Some(rest) = item.strip_prefix('p') {
        return count(rest).then_some(ChainModifier::Width(rest));
    }
    None
}

/// Split a `;`-separated modifier list from the body it applies to. A
/// substitution's own delimiters are skipped, so a pattern may hold either
/// separator.
fn split_modifier_chain(content: &str) -> Option<(Vec<&str>, &str)> {
    let mut items = Vec::new();
    let mut rest = content;
    loop {
        let end = if starts_subst(rest) {
            let delim = rest.as_bytes()[1] as char;
            let mut offset = 2;
            for _ in 0..2 {
                offset = rest[offset..].find(delim)? + offset + delim.len_utf8();
            }
            rest[offset..].find([';', ':'])? + offset
        } else {
            rest.find([';', ':'])?
        };
        items.push(&rest[..end]);
        if rest.as_bytes()[end] == b':' {
            return Some((items, &rest[end + 1..]));
        }
        rest = &rest[end + 1..];
    }
}

fn split_modifier_key(content: &str) -> Option<(Vec<&str>, &str)> {
    let colon = content.find(':')?;
    let modifiers = content[..colon].split(';').collect::<Vec<_>>();
    Some((modifiers, &content[colon + 1..]))
}

/// Index of the `}` closing a `#{` whose body starts at `start`. Brace-aware, so
/// nested `#{…}` (in conditionals/comparisons) are skipped over.
fn find_close(bytes: &[u8], start: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut j = start;
    while j < bytes.len() {
        if bytes[j] == b'#' && bytes.get(j + 1) == Some(&b'{') && !hash_escaped(bytes, j) {
            depth += 1;
            j += 2;
            continue;
        }
        if bytes[j] == b'}' && !hash_escaped(bytes, j) {
            if depth == 0 {
                return Some(j);
            }
            depth -= 1;
        }
        j += 1;
    }
    None
}

fn find_job_close(bytes: &[u8], start: usize) -> Option<usize> {
    let mut depth = 1usize;
    let mut index = start;
    while index < bytes.len() {
        match bytes[index] {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
}

fn strip_format_jobs(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut output = String::with_capacity(value.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'#' && bytes.get(index + 1) == Some(&b'(') {
            let Some(end) = find_job_close(bytes, index + 2) else {
                break;
            };
            index = end + 1;
            continue;
        }
        let character = value[index..]
            .chars()
            .next()
            .expect("format index is at a UTF-8 character boundary");
        output.push(character);
        index += character.len_utf8();
    }
    output
}

fn hash_escaped(bytes: &[u8], index: usize) -> bool {
    let mut hashes = 0usize;
    let mut cursor = index;
    while cursor > 0 && bytes[cursor - 1] == b'#' {
        hashes += 1;
        cursor -= 1;
    }
    hashes % 2 == 1
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vars() -> Vars {
        let mut v = Vars::new();
        v.set("session_name", "work")
            .set("window_index", "1")
            .set("window_name", "editor")
            .set("window_active", "1")
            .set("window_flags", "*")
            .set("pane_index", "0")
            .set("pane_id", "%3");
        v
    }

    /// [`reads_vars`] is what callers trust when they skip building a table,
    /// so it must never answer `false` for a template the expander would have
    /// substituted into. Answering `true` needlessly only costs a build.
    #[test]
    fn reads_vars_covers_every_template_the_expander_substitutes_into() {
        for template in ["", "bench", "no vars here", "a{b}c", "}", "{session_name}"] {
            assert!(!reads_vars(template), "{template:?} named no variable");
            assert_eq!(
                expand(template, &vars()),
                template,
                "{template:?} expanded to something other than itself"
            );
        }

        for template in [
            "#{session_name}",
            "#S",
            "##",
            "#",
            "%H",
            "50%",
            "prefix #{session_name} suffix",
        ] {
            assert!(reads_vars(template), "{template:?} reaches a substitution");
        }
    }

    #[test]
    fn plain_text_passthrough() {
        assert_eq!(expand("hello world", &vars()), "hello world");
    }

    #[test]
    fn unicode_literal_passthrough_preserves_codepoints() {
        assert_eq!(expand("wide: 界é", &vars()), "wide: 界é");
    }

    #[test]
    fn single_variable() {
        assert_eq!(expand("#{session_name}", &vars()), "work");
    }

    #[test]
    fn new_window_template() {
        assert_eq!(
            expand("#{session_name}:#{window_index}.#{pane_index}", &vars()),
            "work:1.0"
        );
    }

    #[test]
    fn new_session_template() {
        assert_eq!(expand("#{session_name}:", &vars()), "work:");
    }

    #[test]
    fn unknown_variable_is_empty() {
        assert_eq!(expand("[#{bogus_var}]", &vars()), "[]");
    }

    #[test]
    fn double_hash_is_literal() {
        assert_eq!(expand("a##b", &vars()), "a#b");
    }

    #[test]
    fn lone_hash_passthrough() {
        // '#b' is not a modeled shorthand, so the '#' is copied and 'b' follows.
        assert_eq!(expand("a#b", &vars()), "a#b");
    }

    #[test]
    fn unterminated_brace_is_verbatim() {
        assert_eq!(expand("x#{oops", &vars()), "x#{oops");
    }

    // ---- shorthands ----

    #[test]
    fn shorthands_expand() {
        assert_eq!(expand("#S:#I.#P", &vars()), "work:1.0");
        assert_eq!(expand("#W", &vars()), "editor");
        assert_eq!(expand("#D", &vars()), "%3");
        assert_eq!(expand("#F", &vars()), "*");
    }

    // ---- conditionals ----

    #[test]
    fn conditional_bare_var_true() {
        assert_eq!(expand("#{?window_active,A,I}", &vars()), "A");
    }

    #[test]
    fn conditional_bare_var_false_when_unset() {
        // tmux treats `1` as a variable name here; it's unset → false branch.
        assert_eq!(expand("#{?1,T,F}", &vars()), "F");
    }

    #[test]
    fn conditional_value_zero_is_false() {
        let mut v = vars();
        v.set("window_active", "0");
        assert_eq!(expand("#{?window_active,A,I}", &v), "I");
    }

    #[test]
    fn conditional_then_branch_expands() {
        assert_eq!(
            expand("#{?window_active,it is #{window_index},no}", &vars()),
            "it is 1"
        );
    }

    #[test]
    fn conditional_nested_cond() {
        assert_eq!(expand("#{?#{window_active},yes,no}", &vars()), "yes");
    }

    #[test]
    fn escaped_comma_stays_inside_a_conditional_branch() {
        let mut v = vars();
        v.set("window_active", "1").set("x", "3").set("y", "4");
        assert_eq!(expand("#{?window_active,[#{x}#,#{y}],no}", &v), "[3,4]");
    }

    #[test]
    fn time_expansion_precedes_variable_expansion() {
        let mut v = vars();
        v.set("value", "%Y");
        let expanded = expand_time_with_jobs("%Y #{value}", &v, None, None, None);
        assert!(expanded[..4].bytes().all(|byte| byte.is_ascii_digit()));
        assert!(expanded.ends_with(" %Y"), "{expanded:?}");
    }

    // ---- comparisons ----

    #[test]
    fn comparison_equal_literals() {
        assert_eq!(expand("#{==:abc,abc}", &vars()), "1");
        assert_eq!(expand("#{==:abc,def}", &vars()), "0");
    }

    #[test]
    fn comparison_not_equal() {
        assert_eq!(expand("#{!=:abc,def}", &vars()), "1");
        assert_eq!(expand("#{!=:a,a}", &vars()), "0");
    }

    #[test]
    fn comparison_expands_operands() {
        assert_eq!(expand("#{==:#{session_name},work}", &vars()), "1");
        assert_eq!(expand("#{==:#{window_index},0}", &vars()), "0");
    }

    // ---- modifiers: literal, basename/dirname, truncate, pad, substitute ----

    #[test]
    fn literal_is_not_expanded() {
        assert_eq!(expand("#{l:#{session_name}}", &vars()), "#{session_name}");
    }

    #[test]
    fn quote_modifiers_escape_styles_and_arguments() {
        let mut v = vars();
        v.set("value", "hash#value with spaces");
        assert_eq!(expand("#{q:value}", &v), "hash\\#value\\ with\\ spaces");
        assert_eq!(expand("#{q/h:value}", &v), "hash##value with spaces");
        assert_eq!(expand("#{q/e:value}", &v), "hash##value with spaces");
        assert_eq!(expand("#{q/a:value}", &v), "\"hash#value with spaces\"");
    }

    #[test]
    fn basename_and_dirname_of_variable_value() {
        let mut v = vars();
        v.set("pane_current_path", "/usr/local/bin");
        assert_eq!(expand("#{b:pane_current_path}", &v), "bin");
        assert_eq!(expand("#{d:pane_current_path}", &v), "/usr/local");
    }

    #[test]
    fn time_modifier_parser_accepts_default_pretty_and_custom_forms() {
        assert_eq!(
            parse_time_modifier("t:value"),
            Some(TimeModifier {
                style: TimeStyle::Default,
                body: "value",
            })
        );
        assert_eq!(
            parse_time_modifier("t/p:value"),
            Some(TimeModifier {
                style: TimeStyle::Pretty,
                body: "value",
            })
        );
        assert_eq!(
            parse_time_modifier("t/f/%H#:%M:value"),
            Some(TimeModifier {
                style: TimeStyle::Custom("%H#:%M".into()),
                body: "value",
            })
        );
    }

    #[test]
    fn time_modifiers_render_epoch_variables() {
        let mut v = vars();
        // Mid-July remains in July in every civil time zone.
        v.set("timestamp", "1752667200").set("time_format", "%Y-%m");

        let default = expand("#{t:timestamp}", &v);
        assert!(
            default.ends_with(" 2025"),
            "unexpected ctime value: {default}"
        );
        assert!(!default.contains('\n'));
        assert_eq!(expand("#{t/f/%Y-%m:timestamp}", &v), "2025-07");
        assert_eq!(
            expand("#{t/f/%H#:%M:timestamp}", &v),
            strftime_time(1_752_667_200, "%H:%M")
        );
        assert_eq!(expand("#{t/f/#{time_format}:timestamp}", &v), "2025-07");
        assert_eq!(expand("#{t/f/literal:timestamp}", &v), "literal");
        assert_eq!(
            expand("#{t/p:timestamp}", &v),
            pretty_time(1_752_667_200, now_epoch())
        );

        v.set("zero", "0").set("invalid", "not-a-time");
        assert_eq!(expand("#{t:zero}", &v), "");
        assert_eq!(expand("#{t:invalid}", &v), "");
        assert_eq!(expand("#{t:missing}", &v), "");
        // tmux expands a nested body without applying the time conversion.
        assert_eq!(expand("#{t:#{timestamp}}", &v), "1752667200");
    }

    #[test]
    fn pretty_time_uses_tmux_age_buckets() {
        let now = 1_752_667_200; // 2025-07-16 12:00:00 UTC
        let hour_ago = now - 60 * 60;
        let two_days_ago = now - 2 * 24 * 60 * 60;
        let thirty_days_ago = now - 30 * 24 * 60 * 60;
        let four_hundred_days_ago = now - 400 * 24 * 60 * 60;

        assert_eq!(pretty_time(hour_ago, now), strftime_time(hour_ago, "%H:%M"));
        assert_eq!(
            pretty_time(two_days_ago, now),
            strftime_time(two_days_ago, "%a%d")
        );
        assert_eq!(
            pretty_time(thirty_days_ago, now),
            strftime_time(thirty_days_ago, "%d%b")
        );
        assert_eq!(
            pretty_time(four_hundred_days_ago, now),
            strftime_time(four_hundred_days_ago, "%h%y")
        );
        assert_eq!(pretty_time(now + 60, now), strftime_time(now + 60, "%H:%M"));
    }

    #[test]
    fn basename_of_nonvariable_is_empty() {
        // The body is not a known variable → resolves to empty (tmux does not
        // treat it as a literal path here).
        assert_eq!(expand("[#{b:/usr/local/bin}]", &vars()), "[]");
        assert_eq!(expand("[#{d:/usr/local/bin}]", &vars()), "[]");
    }

    #[test]
    fn truncate_head_and_tail() {
        let mut v = vars();
        v.set("session_name", "abcdef");
        assert_eq!(expand("#{=3:session_name}", &v), "abc");
        assert_eq!(expand("#{=-3:session_name}", &v), "def");
        // Wider than the value → unchanged.
        assert_eq!(expand("#{=10:session_name}", &v), "abcdef");

        // Truncation markers.
        assert_eq!(expand("#{=/3/...:session_name}", &v), "abc...");
        assert_eq!(expand("#{=/3/.../:session_name}", &v), "abc...");
        assert_eq!(expand("#{=/-3/.../:session_name}", &v), "...def");
        assert_eq!(expand("#{=|-3|...|:session_name}", &v), "...def");
        // When value fits within limit, no marker is added.
        assert_eq!(expand("#{=/10/...:session_name}", &v), "abcdef");
        assert_eq!(expand("#{=/-10/...:session_name}", &v), "abcdef");
    }

    #[test]
    fn truncate_obeys_codepoint_width_boundaries() {
        let mut v = vars();
        v.set("value", "a界b");
        assert_eq!(expand("#{=2:value}", &v), "a");
        assert_eq!(expand("#{=3:value}", &v), "a界");
        assert_eq!(expand("#{=-2:value}", &v), "b");
        assert_eq!(expand("#{=-3:value}", &v), "界b");

        v.set("value", "e\u{301}x");
        assert_eq!(expand("#{=1:value}", &v), "e");
        assert_eq!(expand("#{=-1:value}", &v), "\u{301}x");
    }

    #[test]
    fn truncate_preserves_styles_around_unicode_boundaries() {
        let mut v = vars();
        v.set("value", "a#[fg=red]界b");
        assert_eq!(expand("#{=3:value}", &v), "a#[fg=red]界");
        assert_eq!(expand("#{=-3:value}", &v), "#[fg=red]界b");
    }

    #[test]
    fn display_tokens_stream_exact_source_spans() {
        let input = "a界e\u{301}#####[fg=red]#x";
        let tokens = display_tokens(input);
        assert_eq!(
            tokens.clone().collect::<Vec<_>>(),
            vec![
                ("a", 1),
                ("界", 2),
                ("e", 1),
                ("\u{301}", 0),
                ("##", 1),
                ("##", 1),
                ("#[fg=red]", 0),
                ("#", 1),
                ("x", 1),
            ]
        );
        assert_eq!(tokens.map(|(_, width)| width).sum::<usize>(), 8);
    }

    #[test]
    fn pad_right_and_left() {
        // session_name is "work" (4 chars) in the default vars.
        assert_eq!(expand("[#{p6:session_name}]", &vars()), "[work  ]");
        assert_eq!(expand("[#{p-6:session_name}]", &vars()), "[  work]");
        // Already wide enough → unchanged.
        assert_eq!(expand("[#{p2:session_name}]", &vars()), "[work]");
    }

    #[test]
    fn pad_uses_display_columns() {
        let mut v = vars();
        v.set("value", "界");
        assert_eq!(expand("[#{p3:value}]", &v), "[界 ]");
        assert_eq!(expand("[#{p-3:value}]", &v), "[ 界]");

        v.set("value", "e\u{301}");
        assert_eq!(expand("[#{p2:value}]", &v), "[e\u{301} ]");
    }

    #[test]
    fn substitute_replaces_all() {
        let mut v = vars();
        v.set("session_name", "foobar");
        assert_eq!(expand("#{s/o/0/:session_name}", &v), "f00bar");
        // A trailing flag (e.g. `g`) is accepted; substitution is global anyway.
        assert_eq!(expand("#{s/o/0/g:session_name}", &v), "f00bar");
        assert_eq!(expand("#{s/O/0/i:session_name}", &v), "f00bar");
    }

    // The expectations below are the ones tmux 3.7b prints for the same
    // formats; they are the edges where `regsub`'s scan differs from a plain
    // replace-all.
    #[test]
    fn substitute_leaves_empty_pattern_and_empty_value_alone() {
        let mut v = vars();
        v.set("value", "abABab");
        assert_eq!(expand("#{s//-/:value}", &v), "abABab");
        // An uncompilable pattern is a no-op too.
        assert_eq!(expand("#{s/[/-/:value}", &v), "abABab");

        v.set("value", "");
        assert_eq!(expand("#{s/a/-/:value}", &v), "");
    }

    #[test]
    fn substitute_backreference_falls_back_to_the_digit() {
        let mut v = vars();
        v.set("value", "abABab");
        // No capture group at all, and a group that matched nothing: both emit
        // the bare digit rather than dropping the backreference.
        assert_eq!(expand("#{s/a/\\1/:value}", &v), "1bAB1b");
        assert_eq!(expand("#{s/a/\\9/:value}", &v), "9bAB9b");
        assert_eq!(expand("#{s/(x*)a/[\\1]/:value}", &v), "[1]bAB[1]b");
        // A group that did match expands, and a non-digit escape just loses
        // its backslash.
        assert_eq!(expand("#{s/(a)(b)/[\\2\\1]/:value}", &v), "[ba]AB[ba]");
        assert_eq!(expand("#{s/a/x\\ty/:value}", &v), "xtybABxtyb");
        assert_eq!(expand("#{s/a/x\\\\y/:value}", &v), "x\\ybABx\\yb");
    }

    #[test]
    fn substitute_advances_past_empty_matches() {
        let mut v = vars();
        v.set("value", "abc");
        assert_eq!(expand("#{s/x*/-/:value}", &v), "a-b-c-");
        assert_eq!(expand("#{s/$/-/:value}", &v), "abc-");

        v.set("value", "abcabc");
        assert_eq!(expand("#{s/b*/-/:value}", &v), "a-c-a-c-");

        v.set("value", "aaa");
        assert_eq!(expand("#{s/a*/-/:value}", &v), "-");
    }

    #[test]
    fn substitute_anchored_pattern_stops_after_one_match() {
        let mut v = vars();
        v.set("value", "abcabc");
        assert_eq!(expand("#{s/^abc/-/:value}", &v), "-abc");

        v.set("value", "abc");
        assert_eq!(expand("#{s/^a/-/:value}", &v), "-bc");
        // tmux consumes the character after an anchored empty match; keep the
        // same result rather than a more defensible one.
        assert_eq!(expand("#{s/^/-/:value}", &v), "bc");
    }

    #[test]
    fn substitute_ignores_a_multibyte_delimiter() {
        // A non-ASCII byte after `s` is not a delimiter — splitting there would
        // cut a character in half.
        let mut v = vars();
        v.set("value", "abc");
        assert_eq!(expand("#{s界a界b界:value}", &v), "");
    }

    #[test]
    fn substitute_chains_steps_left_to_right() {
        let mut v = vars();
        v.set("value", "abcabc");
        assert_eq!(expand("#{s/a/b/;s/b/c/:value}", &v), "cccccc");
        assert_eq!(expand("#{s/a/b/g;s/c/d/:value}", &v), "bbdbbd");
        // Each step brings its own delimiter and flags.
        assert_eq!(expand("#{s|a|b|;s/B/e/i:value}", &v), "eeceec");
    }

    #[test]
    fn substitute_steps_over_multibyte_characters() {
        // tmux steps a byte at a time and prints "a-\347-\225-\214-b-", which
        // is not valid UTF-8; stepping whole characters is the documented
        // difference (README.md).
        let mut v = vars();
        v.set("value", "a界b");
        assert_eq!(expand("#{s/x*/-/:value}", &v), "a-界-b-");
    }

    #[test]
    fn width_modifier_is_body_display_width() {
        // `w:` is the width of the *resolved* body: session_name is "work" (4).
        assert_eq!(expand("#{w:session_name}", &vars()), "4");
        assert_eq!(expand("#{w:#{session_name}}", &vars()), "4");
    }

    #[test]
    fn width_modifier_uses_scalar_widths() {
        let mut v = vars();
        v.set("value", "界");
        assert_eq!(expand("#{w:value}", &v), "2");

        v.set("value", "e\u{301}");
        assert_eq!(expand("#{w:value}", &v), "1");

        v.set("value", "👩‍💻");
        assert_eq!(expand("#{w:value}", &v), "4");

        // A conjoining jamo is one column, as tmux's width tables say. What
        // makes it disappear into the syllable before it is the screen's
        // combining rule, which a format expansion never runs.
        v.set("value", "\u{1161}");
        assert_eq!(expand("#{w:value}", &v), "1");
    }

    #[test]
    fn modifier_prefix_letters_do_not_shadow_variables() {
        // `pane_index` and `session_name` start with the pad/subst prefix letters
        // but must still resolve as plain variables.
        assert_eq!(expand("#{pane_index}", &vars()), "0");
        assert_eq!(expand("#{session_name}", &vars()), "work");
    }
}
