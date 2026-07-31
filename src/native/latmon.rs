//! Optional keystroke→screen latency probe for the interactive attach loop.
//!
//! In native mode there is no network between the client and the server: the
//! tmux client hands its tty fds to the daemon (`SCM_RIGHTS`) and the daemon
//! reads keystrokes from / writes frames to that tty *directly* from a single
//! thread (`attach::run_attach`). Any lag a user perceives while typing is
//! therefore one of two things:
//!
//!   1. the network between their terminal and the host running hmux (e.g. the
//!      SSH hop) — invisible to hmux, and
//!   2. the time hmux itself spends turning a keypress into a visible frame.
//!
//! This probe measures (2) so the user can tell the two apart: if the logged
//! in-daemon latency is small but typing still feels laggy, the delay is the
//! network. The whole measurement lives in the attach-loop thread — it stamps
//! the moment input is offered to the pane, the moment subsequent pane output
//! is parsed, the moment that output wakes the attach loop, and the moment the
//! resulting frame is written back to the tty.
//!
//! Disabled by default (zero overhead). Enable by setting the `HMUX_LATENCY`
//! environment variable **for the daemon process**:
//!
//!   - `HMUX_LATENCY=1`         → append to `/tmp/hmux-latency-<uid>.log`
//!   - `HMUX_LATENCY=/path/log` → append to that path
//!
//! Each keystroke-to-frame cycle emits one line, plus a rolling summary every
//! [`SUMMARY_EVERY`] samples:
//!
//! ```text
//! 14:22:01.123 sess=0 pid=1234 fmt=2 in=3B echo=1.20ms pane=1.10ms wake=0.10ms render=0.31ms total=1.51ms inputs=1 span=0.00ms quiet=1.10ms queued=0B dropped=0B late=0
//! 14:22:05.880 sess=0 pid=1234 fmt=2 SUMMARY n=100 avg=1.44ms p50=1.10ms p95=3.90ms max=12.30ms late=0
//! ```
//!
//! `pid` identifies the daemon which emitted an append-only log entry; `fmt`
//! identifies the field layout so samples left by older binaries are obvious.
//!
//! - `echo`   = input accepted by hmux → the pane's next output wakes the loop
//!   (possible PTY queueing + shell/app response + libghostty VT parse). It is
//!   the bulk of any real hmux-side cost, but does not itself prove causality.
//! - `pane`   = input offered → pane output parsed. A large value points at PTY
//!   queueing or the foreground application rather than the attach renderer.
//! - `wake`   = pane output parsed → attach loop observes its notification. A
//!   large value points at attach-loop scheduling or an hmux-side stall.
//! - `render` = output wake → frame written to the client tty (compose + diff).
//! - `total`  = input accepted → frame on the wire back to the client.
//! - `inputs` / `span` show how many attach-loop input batches were folded into
//!   the sample and the time from the first to the last. `quiet` is the remaining
//!   last-input → pane-output interval. This distinguishes a long typing burst
//!   from a long wait after the final key.
//! - `queued` / `dropped` report PTY backpressure. Queued bytes were accepted by
//!   hmux but not yet written to the PTY when input handling returned.
//! - `late=1` means the next changed output arrived after [`STALE_AFTER`]. Such
//!   a sample is intentionally retained, but its attribution is ambiguous: it
//!   may be a real stall, or a silent input later paired with unrelated output.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// How many samples between rolling summary lines.
const SUMMARY_EVERY: usize = 100;

/// Version of the per-sample text fields. Bump when their meaning or layout
/// changes so an append-only file cannot silently mix incompatible samples.
const LOG_FORMAT: u8 = 2;

/// A pending input whose on-screen echo has not yet been rendered. `input_at`
/// marks the start of the current typing burst (kept at the *earliest* un-echoed
/// keystroke so `total` reflects the full time the user waited).
struct Pending {
    input_at: Instant,
    last_input_at: Instant,
    wall: SystemTime,
    bytes_in: usize,
    input_events: usize,
    queued: usize,
    dropped: usize,
}

/// A pending burst whose subsequent pane output has arrived, but whose frame
/// has not yet been composed. Keeping this separate prevents input read in the
/// same poll iteration from being attributed to the older output event.
struct Ready {
    pending: Pending,
    pane_output_at: Instant,
    output_at: Instant,
}

/// If a pending input is older than this when the next keystroke arrives, the
/// old burst is replaced. A completed sample this old is retained but marked
/// `late=1`, because a silent key followed by unrelated output cannot be
/// distinguished here from a real application/PTY stall.
const STALE_AFTER: Duration = Duration::from_secs(2);

/// Per-attach latency recorder. All methods are cheap no-ops unless enabled via
/// the `HMUX_LATENCY` env var, so the attach loop can call them unconditionally.
pub struct LatMon {
    sink: Option<File>,
    label: String,
    pid: u32,
    pending: Option<Pending>,
    ready: Option<Ready>,
    /// Recent `total` values (ms) for the rolling summary.
    window: Vec<f64>,
    /// Samples in `window` whose input-to-output attribution is ambiguous.
    late_in_window: usize,
}

impl LatMon {
    /// Create a recorder for one attached client. `label` identifies the client
    /// in the log (e.g. `"sess=0"`) so concurrent attaches sharing one file are
    /// distinguishable. Reads `HMUX_LATENCY` to decide whether to log at all.
    pub fn new(label: impl Into<String>) -> Self {
        let sink = match std::env::var("HMUX_LATENCY") {
            Ok(v) if !v.is_empty() && v != "0" => open_sink(&v),
            _ => None,
        };
        LatMon {
            sink,
            label: label.into(),
            pid: std::process::id(),
            pending: None,
            ready: None,
            window: Vec::new(),
            late_in_window: 0,
        }
    }

    /// Record that `bytes` of real input were accepted for the pane's PTY.
    /// Starts a new typing burst if none is in flight; otherwise extends the
    /// current one (keeping the earliest timestamp), resetting it if the prior
    /// burst has gone stale without ever producing a frame.
    pub fn on_input(&mut self, now: Instant, bytes: usize, queued: usize, dropped: usize) {
        if self.sink.is_none() {
            return;
        }
        match self.pending.as_mut() {
            Some(p) if now.saturating_duration_since(p.input_at) < STALE_AFTER => {
                p.last_input_at = now;
                p.bytes_in += bytes;
                p.input_events += 1;
                p.queued += queued;
                p.dropped += dropped;
            }
            _ => {
                self.pending = Some(Pending {
                    input_at: now,
                    last_input_at: now,
                    wall: SystemTime::now(),
                    bytes_in: bytes,
                    input_events: 1,
                    queued,
                    dropped,
                });
            }
        }
    }

    /// Record that the active pane's output just woke the loop (its echo is now
    /// in the grid, about to be composed). Only the first wake per burst counts.
    pub fn on_output(&mut self, pane_output_at: Option<Instant>) {
        if self.sink.is_none() {
            return;
        }
        if self.ready.is_some() {
            return;
        }
        let output_at = Instant::now();
        let Some(pane_output_at) = pane_output_at else {
            return;
        };
        let Some(pending) = self.pending.as_ref() else {
            return;
        };
        // A pane wakeup can already be readable when input arrives. Its timestamp
        // makes that older output distinguishable from output caused by (or at
        // least subsequent to) the pending input.
        if pane_output_at < pending.input_at {
            return;
        }
        self.ready = self.pending.take().map(|pending| Ready {
            pending,
            pane_output_at,
            output_at,
        });
    }

    /// Record that a changed frame was just written to the client tty, closing
    /// the current burst and emitting a sample line.
    pub fn on_render(&mut self) {
        if self.sink.is_none() {
            return;
        }
        let now = Instant::now();
        let Some(ready) = self.ready.take() else {
            return;
        };
        let p = &ready.pending;
        let pane = ms(ready.pane_output_at.saturating_duration_since(p.input_at));
        let wake = ms(ready
            .output_at
            .saturating_duration_since(ready.pane_output_at));
        let echo = ms(ready.output_at.saturating_duration_since(p.input_at));
        let render = ms(now.saturating_duration_since(ready.output_at));
        let total = ms(now.saturating_duration_since(p.input_at));
        let span = ms(p.last_input_at.saturating_duration_since(p.input_at));
        let quiet = ms(ready
            .pane_output_at
            .saturating_duration_since(p.last_input_at));
        self.emit(p, echo, pane, wake, render, total, span, quiet);
    }

    /// The loop considered rendering but the frame was unchanged: the pending
    /// input produced no visible output, so drop it rather than blame a later
    /// unrelated frame for its latency.
    pub fn discard(&mut self) {
        if self.sink.is_some() {
            self.ready = None;
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn emit(
        &mut self,
        p: &Pending,
        echo: f64,
        pane: f64,
        wake: f64,
        render: f64,
        total: f64,
        span: f64,
        quiet: f64,
    ) {
        let late = usize::from(echo >= ms(STALE_AFTER));
        let line = format!(
            "{} {} pid={} fmt={} in={}B echo={:.2}ms pane={:.2}ms wake={:.2}ms render={:.2}ms total={:.2}ms inputs={} span={:.2}ms quiet={:.2}ms queued={}B dropped={}B late={}\n",
            fmt_wall(p.wall),
            self.label,
            self.pid,
            LOG_FORMAT,
            p.bytes_in,
            echo,
            pane,
            wake,
            render,
            total,
            p.input_events,
            span,
            quiet,
            p.queued,
            p.dropped,
            late,
        );
        self.write(&line);

        self.window.push(total);
        self.late_in_window += late;
        if self.window.len() >= SUMMARY_EVERY {
            self.emit_summary();
            self.window.clear();
            self.late_in_window = 0;
        }
    }

    fn emit_summary(&mut self) {
        let mut v = self.window.clone();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let n = v.len();
        if n == 0 {
            return;
        }
        let pct = |q: f64| v[(((n - 1) as f64) * q).round() as usize];
        let avg = v.iter().sum::<f64>() / n as f64;
        let line = format!(
            "{} {} pid={} fmt={} SUMMARY n={} avg={:.2}ms p50={:.2}ms p95={:.2}ms max={:.2}ms late={}\n",
            fmt_wall(SystemTime::now()),
            self.label,
            self.pid,
            LOG_FORMAT,
            n,
            avg,
            pct(0.50),
            pct(0.95),
            v[n - 1],
            self.late_in_window,
        );
        self.write(&line);
    }

    fn write(&mut self, line: &str) {
        if let Some(f) = self.sink.as_mut() {
            // Best effort: a full short line via a single append write is atomic
            // enough for eyeballing; drop the sample rather than disturb attach.
            let _ = f.write_all(line.as_bytes());
        }
    }
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

#[cfg(test)]
impl LatMon {
    /// Build a recorder writing to `path`, bypassing the env-var gate, for tests.
    fn for_test(path: &std::path::Path) -> Self {
        let sink = OpenOptions::new().create(true).append(true).open(path).ok();
        LatMon {
            sink,
            label: "sess=0".to_string(),
            pid: std::process::id(),
            pending: None,
            ready: None,
            window: Vec::new(),
            late_in_window: 0,
        }
    }
}

/// Resolve `HMUX_LATENCY` to an append handle: `"1"` → the default per-uid path
/// in `/tmp`, anything else → that literal path.
fn open_sink(value: &str) -> Option<File> {
    let path = if value == "1" {
        let uid = unsafe { libc::getuid() };
        format!("/tmp/hmux-latency-{uid}.log")
    } else {
        value.to_string()
    };
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .ok()
}

/// Format a wall-clock instant as local `HH:MM:SS.mmm` (no chrono dependency).
fn fmt_wall(t: SystemTime) -> String {
    let dur = t.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = dur.as_secs() as libc::time_t;
    let millis = dur.subsec_millis();
    // SAFETY: `tm` is a plain C struct fully written by localtime_r; the call
    // only reads `secs` and writes `tm`.
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    unsafe {
        libc::localtime_r(&secs, &mut tm);
    }
    format!(
        "{:02}:{:02}:{:02}.{:03}",
        tm.tm_hour, tm.tm_min, tm.tm_sec, millis
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(path: &std::path::Path) -> String {
        std::fs::read_to_string(path).unwrap_or_default()
    }

    #[test]
    fn full_cycle_emits_one_wellformed_line() {
        let path = std::env::temp_dir().join(format!("hmux-lat-cycle-{}.log", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let mut m = LatMon::for_test(&path);

        let input_at = Instant::now();
        m.on_input(input_at, 3, 2, 1);
        m.on_output(Some(Instant::now()));
        m.on_render();

        let out = read(&path);
        assert_eq!(out.lines().count(), 1, "one input→render cycle → one line");
        let line = out.trim();
        // `HH:MM:SS.mmm sess=0 pid=123 fmt=2 in=3B echo=..ms ...`
        assert!(line.contains("sess=0"), "labelled: {line}");
        assert!(line.contains("fmt=2"), "format version: {line}");
        assert!(line.contains("in=3B"), "byte count: {line}");
        assert!(line.contains("inputs=1"), "input events: {line}");
        assert!(line.contains("queued=2B"), "pty queue: {line}");
        assert!(line.contains("dropped=1B"), "pty drops: {line}");
        assert!(line.contains("late=0"), "late marker: {line}");
        for field in ["echo=", "pane=", "wake=", "render=", "total="] {
            assert!(
                line.contains(field) && line.contains("ms"),
                "{field} in: {line}"
            );
        }
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn discard_and_spurious_hooks_write_nothing() {
        let path =
            std::env::temp_dir().join(format!("hmux-lat-discard-{}.log", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let mut m = LatMon::for_test(&path);

        // An unchanged frame drops the input whose output was being rendered.
        m.on_input(Instant::now(), 1, 0, 0);
        m.on_output(Some(Instant::now()));
        m.discard();
        // Nothing is ready after the unchanged frame; later output/render hooks
        // with no input in flight are ignored too.
        m.on_render();
        m.on_output(Some(Instant::now()));
        m.on_render();

        assert_eq!(
            read(&path),
            "",
            "no samples without a completed keystroke cycle"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn disabled_recorder_is_inert() {
        // No sink (env-gated off in production): every hook is a no-op.
        let mut m = LatMon {
            sink: None,
            label: "sess=0".to_string(),
            pid: std::process::id(),
            pending: None,
            ready: None,
            window: Vec::new(),
            late_in_window: 0,
        };
        m.on_input(Instant::now(), 5, 0, 0);
        m.on_output(Some(Instant::now()));
        m.on_render();
        assert!(m.pending.is_none());
    }

    #[test]
    fn late_output_is_marked_as_ambiguous() {
        let path = std::env::temp_dir().join(format!("hmux-lat-late-{}.log", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let mut m = LatMon::for_test(&path);
        let old = Instant::now() - STALE_AFTER - Duration::from_millis(1);
        m.pending = Some(Pending {
            input_at: old,
            last_input_at: old,
            wall: SystemTime::now(),
            bytes_in: 1,
            input_events: 1,
            queued: 0,
            dropped: 0,
        });

        m.on_output(Some(Instant::now()));
        m.on_render();

        let out = read(&path);
        assert!(out.contains("late=1"), "late attribution marker: {out}");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn input_after_output_wake_starts_the_next_sample() {
        let path =
            std::env::temp_dir().join(format!("hmux-lat-overlap-{}.log", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let mut m = LatMon::for_test(&path);

        m.on_input(Instant::now(), 1, 0, 0);
        m.on_output(Some(Instant::now()));
        m.on_input(Instant::now(), 2, 0, 0);
        m.on_render();
        m.on_output(Some(Instant::now()));
        m.on_render();

        let out = read(&path);
        let lines: Vec<_> = out.lines().collect();
        assert_eq!(lines.len(), 2, "one sample per output generation: {out}");
        assert!(lines[0].contains("in=1B"), "first burst: {out}");
        assert!(lines[1].contains("in=2B"), "overlapping next burst: {out}");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn output_older_than_input_cannot_close_the_sample() {
        let path =
            std::env::temp_dir().join(format!("hmux-lat-preinput-{}.log", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let mut m = LatMon::for_test(&path);
        let input_at = Instant::now();

        m.on_input(input_at, 1, 0, 0);
        m.on_output(Some(input_at - Duration::from_millis(1)));
        m.on_render();
        assert_eq!(read(&path), "", "older output must not claim newer input");
        assert!(m.pending.is_some(), "new input remains pending");

        m.on_output(Some(Instant::now()));
        m.on_render();
        assert!(read(&path).contains("in=1B"));
        let _ = std::fs::remove_file(&path);
    }
}
