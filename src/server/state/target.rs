//! Target resolution — parsing tmux's `-t` argument into the session, window,
//! and pane it names.
//!
//! [`Target`] is the parsed form; [`TargetParts`] splits the raw string on
//! tmux's `session:window.pane` grammar and expands the `{next}`/`{last}`
//! token spellings. Everything here is pure parsing over borrowed state — the
//! lookups that walk `ServerState` live with the state itself.

use std::io;

use super::{Session, Window};

/// A resolved `target` (`session[:window]`): indices into the session/window
/// tree plus the active pane index of that window.
#[derive(Clone, Copy)]
pub struct Target {
    pub session: usize,
    pub window: usize,
    pub pane: usize,
}

/// What a `-t` target names, mirroring the type argument of tmux's
/// `cmd_find_target`. It decides how a target that is only a bare word is read:
/// for a pane target `1` is a pane index in the current window, for a window
/// target a window index in the current session. Either falls back to the other
/// (and to a session name) when that first reading finds nothing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum TargetKind {
    Window,
    Pane,
}

/// A target that did not resolve: tmux's diagnostic for the part that went
/// missing, plus the state its lookup had reached when it gave up. Commands
/// whose `-t` is allowed to fail (`display-message`) run against that residual
/// state instead of failing.
pub(super) struct TargetMiss {
    pub(super) error: io::Error,
    pub(super) residual: Option<Target>,
}

impl TargetMiss {
    pub(super) fn new(error: io::Error, residual: Option<Target>) -> Self {
        Self { error, residual }
    }
}

/// The `session:window.pane` parts of a target, split by tmux's rules.
pub(super) struct TargetParts<'a> {
    pub(super) session: Option<&'a str>,
    pub(super) window: Option<&'a str>,
    pub(super) pane: Option<&'a str>,
    /// The window part was spelled out with a `:` (or a `.`), so a window that
    /// doesn't resolve is an error rather than a session name to try next.
    pub(super) window_only: bool,
    /// The pane part was spelled out with a `.`, likewise.
    pub(super) pane_only: bool,
    pub(super) exact_session: bool,
    pub(super) exact_window: bool,
}

impl<'a> TargetParts<'a> {
    /// Split a target the way tmux's `cmd_find_target` does: the session part
    /// ends at the first `:`, the pane part starts at the first `.` after it (or
    /// at the first `.` at all when the target has no `:`). A target with
    /// neither separator is a session, window or pane part depending on its `$`,
    /// `@` or `%` marker, and on the target kind when it has none.
    pub(super) fn parse(target: &'a str, kind: TargetKind) -> Self {
        let colon = target.find(':');
        let tail = match colon {
            Some(at) => &target[at + 1..],
            None => target,
        };
        let period = tail.find('.');
        let (mut session, mut window, mut pane) = (None, None, None);
        let (mut window_only, mut pane_only) = (false, false);
        match (colon, period) {
            (Some(at), Some(dot)) => {
                session = Some(&target[..at]);
                window = Some(&tail[..dot]);
                window_only = true;
                pane = Some(&tail[dot + 1..]);
                pane_only = true;
            }
            (Some(at), None) => {
                session = Some(&target[..at]);
                window = Some(tail);
                window_only = true;
            }
            (None, Some(dot)) => {
                window = Some(&tail[..dot]);
                pane = Some(&tail[dot + 1..]);
                pane_only = true;
            }
            (None, None) => match target.as_bytes().first() {
                Some(b'$') => session = Some(target),
                Some(b'@') => window = Some(target),
                Some(b'%') => pane = Some(target),
                _ => match kind {
                    TargetKind::Window => window = Some(target),
                    TargetKind::Pane => pane = Some(target),
                },
            },
        }
        // `=name` asks for an exact match on that part; the marker is not part
        // of the name.
        let exact_session = strip_exact(&mut session);
        let exact_window = strip_exact(&mut window);
        // tmux rewrites the long token names to their short forms before it
        // resolves anything (`cmd_find_map_table`), so a diagnostic about one
        // names the short form.
        let window = window.map(|part| map_token(part, WINDOW_TOKENS));
        let pane = pane.map(|part| map_token(part, PANE_TOKENS));
        Self {
            // An empty part is the same as an absent one, so `:win` is the
            // current session's `win` and `sess:` its current window.
            session: session.filter(|part| !part.is_empty()),
            window: window.filter(|part| !part.is_empty()),
            pane: pane.filter(|part| !part.is_empty()),
            window_only,
            pane_only,
            exact_session,
            exact_window,
        }
    }
}

/// tmux's `cmd_find_window_table`: the long names of the window tokens and the
/// short forms it rewrites them to.
const WINDOW_TOKENS: &[(&str, &str)] = &[
    ("{start}", "^"),
    ("{last}", "!"),
    ("{end}", "$"),
    ("{next}", "+"),
    ("{previous}", "-"),
];

/// tmux's `cmd_find_pane_table`. Only these three have a short form; the rest of
/// the pane tokens (`{top}`, `{left-of}`, …) are already their own spelling.
const PANE_TOKENS: &[(&str, &str)] = &[("{last}", "!"), ("{next}", "+"), ("{previous}", "-")];

/// Rewrite a target part through one of the token tables, leaving anything that
/// isn't a token alone.
fn map_token<'a>(part: &'a str, table: &[(&str, &'a str)]) -> &'a str {
    table
        .iter()
        .find_map(|(token, short)| (*token == part).then_some(*short))
        .unwrap_or(part)
}

/// Strip a target part's `=` exact-match marker, reporting whether it was there.
fn strip_exact(part: &mut Option<&str>) -> bool {
    match part.and_then(|part| part.strip_prefix('=')) {
        Some(stripped) => {
            *part = Some(stripped);
            true
        }
        None => false,
    }
}

/// The one position an iterator of `(position, value)` matches, or `None` when
/// it matched nothing — or, as tmux treats an ambiguous target, more than one.
pub(super) fn unique<T>(mut matches: impl Iterator<Item = (usize, T)>) -> Option<usize> {
    let (position, _) = matches.next()?;
    matches.next().is_none().then_some(position)
}

/// How a target's window part matched the windows of a session.
pub(super) enum WindowMatch {
    Found(usize),
    /// More than one window matched the name, prefix or pattern. tmux reports
    /// such a target as unresolvable, but its lookup keeps the first match, so
    /// that is the residual state a can-fail command formats against.
    Ambiguous(usize),
    None,
}

impl WindowMatch {
    pub(super) fn of(position: Option<usize>) -> Self {
        match position {
            Some(position) => Self::Found(position),
            None => Self::None,
        }
    }

    pub(super) fn single<T>(mut matches: impl Iterator<Item = (usize, T)>) -> Self {
        let Some((position, _)) = matches.next() else {
            return Self::None;
        };
        match matches.next() {
            Some(_) => Self::Ambiguous(position),
            None => Self::Found(position),
        }
    }

    /// Where an ambiguous match's first candidate was, if this is one.
    pub(super) fn ambiguous(&self) -> Option<usize> {
        match self {
            Self::Ambiguous(position) => Some(*position),
            _ => None,
        }
    }
}

/// Split a pane target `session[:window][.pane]` into its window-target part and
/// the optional pane part (the text after the last `.`).
pub(super) fn split_pane_target(target: &str) -> (&str, Option<&str>) {
    match target.rsplit_once('.') {
        Some((win, pane)) => (win, Some(pane)),
        None => (target, None),
    }
}

/// Parse a `[session][:index]` destination into `(session, index)`. A bare
/// number with no colon is an index in the current session.
pub(super) fn parse_index_target(target: &str) -> (Option<&str>, Option<u32>) {
    match target.split_once(':') {
        Some((sess, idx)) => {
            let session = if sess.is_empty() { None } else { Some(sess) };
            (session, idx.parse().ok())
        }
        None => match target.parse::<u32>() {
            Ok(n) => (None, Some(n)),
            Err(_) => (Some(target), None),
        },
    }
}

/// Resolve one of tmux's special window tokens within a session to a `Vec`
/// position, relative to the session's active/last window. Returns `None` if the
/// token isn't a special form (the caller then tries a numeric index).
///
/// Handled: `{start}`/`^` (first), `{end}`/`$` (last), `{last}`/`!` (previous),
/// `{next}`/`+` (next by order, wrapping), `{previous}`/`-` (previous by order).
pub(super) fn window_special(sess: &Session, spec: &str) -> Option<usize> {
    let n = sess.windows.len();
    if n == 0 {
        return None;
    }
    match spec {
        "{start}" | "^" => Some(0),
        "{end}" | "$" => Some(n - 1),
        "{last}" | "!" => sess.last_active().filter(|&p| p < n),
        "{next}" | "+" => Some((sess.active + 1) % n),
        "{previous}" | "-" => Some((sess.active + n - 1) % n),
        _ => None,
    }
}

/// Resolve a `+N`/`-N` window offset within a session: `N` windows after or
/// before the session's active one, wrapping like tmux's
/// `winlink_next_by_number`. A bare `+`/`-` moves by one. `None` if the spec
/// isn't an offset.
pub(super) fn window_offset(sess: &Session, spec: &str) -> Option<usize> {
    let n = sess.windows.len();
    let (sign, count) = match spec.split_at_checked(1)? {
        ("+", rest) => (1isize, rest),
        ("-", rest) => (-1, rest),
        _ => return None,
    };
    let count = if count.is_empty() {
        1
    } else {
        count.parse::<usize>().ok()?
    };
    if n == 0 {
        return None;
    }
    let step = count % n;
    Some(match sign {
        1 => (sess.active + step) % n,
        _ => (sess.active + n - step) % n,
    })
}

/// Resolve a pane spec within `win` to a `Vec` position: a numeric index, a
/// `%id`, or a special token (`{top}`, `{bottom}`, `{last}`/`!`, `{next}`/`+`,
/// `{previous}`/`-`). Returns `None` on a miss.
pub(super) fn pane_pos_in(win: &Window, spec: &str, base: usize) -> Option<usize> {
    let n = win.panes.len();
    if n == 0 {
        return None;
    }
    if let Some(id) = spec.strip_prefix('%') {
        let id: u32 = id.parse().ok()?;
        return win.panes.iter().position(|p| p.id == id);
    }
    match spec {
        "{top}" => return Some(0),
        "{bottom}" => return Some(n - 1),
        "{last}" | "!" => return win.last_pane.filter(|&p| p < n),
        "{next}" | "+" => return Some((win.active + 1) % n),
        "{previous}" | "-" => return Some((win.active + n - 1) % n),
        _ => {}
    }
    let idx: usize = spec.parse().ok()?;
    let idx = idx.checked_sub(base)?;
    (idx < n).then_some(idx)
}

/// tmux's `can't find pane: <part>` error.
pub(super) fn pane_not_found(part: &str) -> io::Error {
    io::Error::new(io::ErrorKind::NotFound, format!("can't find pane: {part}"))
}

/// tmux's `can't find window: <part>` error.
pub(super) fn window_not_found(part: &str) -> io::Error {
    io::Error::new(
        io::ErrorKind::NotFound,
        format!("can't find window: {part}"),
    )
}

/// tmux's `can't find session: <part>` error.
pub(super) fn session_not_found(part: &str) -> io::Error {
    io::Error::new(
        io::ErrorKind::NotFound,
        format!("can't find session: {part}"),
    )
}
