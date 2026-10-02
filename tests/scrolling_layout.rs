//! Exercise scrolling through isolated servers, including real terminal clients.
use std::fs;
use std::path::PathBuf;
use std::process::{Child, Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

const TIMEOUT: Duration = Duration::from_secs(8);

struct Server {
    root: PathBuf,
    child: Child,
}

impl Server {
    fn new(config: &str) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "hmux-scrolling-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        fs::write(root.join("config"), config).unwrap();
        let mut command = Self::command_at(&root);
        let child = command
            .env("HOME", &root)
            .args(["-D", "-f"])
            .arg(root.join("config"))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let mut server = Self { root, child };
        let deadline = Instant::now() + TIMEOUT;
        while !server.root.join("socket").exists() {
            assert!(server.child.try_wait().unwrap().is_none(), "server exited");
            assert!(Instant::now() < deadline, "server did not start");
            thread::sleep(Duration::from_millis(10));
        }
        server.run(&[
            "new-session",
            "-d",
            "-s",
            "scroll",
            "-x80",
            "-y24",
            "/bin/sh",
        ]);
        server
    }

    fn command_at(root: &std::path::Path) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_hmux"));
        command
            .arg("-S")
            .arg(root.join("socket"))
            .env_remove("TMUX")
            .env_remove("TMUX_PANE")
            .env("TMUX_C2RS_PLUGINS", "none")
            .env("SHELL", "/bin/sh")
            .env("TERM", "xterm-256color");
        command
    }

    fn command(&self) -> Command {
        Self::command_at(&self.root)
    }

    fn output(&self, args: &[&str]) -> Output {
        self.command().args(args).output().unwrap()
    }

    fn run(&self, args: &[&str]) -> String {
        let output = self.output(args);
        assert!(
            output.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout)
            .unwrap()
            .trim_end_matches('\n')
            .to_owned()
    }

    fn reject(&self, args: &[&str], diagnostic: &str) {
        let before = self.layout();
        let zoom = self.format("#{window_zoomed_flag}");
        let output = self.output(args);
        assert!(!output.status.success(), "unexpected success: {args:?}");
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains(diagnostic), "{args:?}: {error}");
        assert_eq!(
            self.layout(),
            before,
            "rejected command changed layout: {args:?}"
        );
        assert_eq!(self.format("#{window_zoomed_flag}"), zoom);
    }

    fn format(&self, format: &str) -> String {
        self.run(&["display-message", "-p", "-t", "scroll:0", format])
    }

    fn layout(&self) -> String {
        self.format("#{window_layout}")
    }

    fn panes(&self) -> Vec<(u32, u32, u32, u32, bool, bool)> {
        self.run(&["list-panes", "-t", "scroll:0", "-F",
            "#{pane_id} #{pane_width} #{pane_height} #{pane_left} #{pane_active} #{pane_floating_flag}"])
            .lines().map(|line| {
                let v: Vec<u32> = line.split_whitespace().map(|v| v.trim_start_matches('%').parse().unwrap()).collect();
                (v[0], v[1], v[2], v[3], v[4] != 0, v[5] != 0)
            }).collect()
    }

    fn widths(&self) -> Vec<(u32, u32, u32)> {
        self.panes()
            .into_iter()
            .filter(|p| !p.5)
            .map(|p| (p.0, p.1, p.3))
            .collect()
    }

    fn insert(&self, target: &str, before: bool) -> String {
        let mut args = vec!["new-pane", "-L", "-t", target, "-P", "-F", "#{pane_id}"];
        if before {
            args.push("-b");
        }
        self.run(&args)
    }

    fn await_value(&self, mut value: impl FnMut() -> String, expected: &str) {
        let deadline = Instant::now() + TIMEOUT;
        loop {
            let actual = value();
            if actual == expected {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "expected {expected:?}, got {actual:?}"
            );
            thread::sleep(Duration::from_millis(20));
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self
            .command()
            .arg("kill-server")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline && self.child.try_wait().ok().flatten().is_none() {
            thread::sleep(Duration::from_millis(10));
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn strip_creation_removal_widths_and_window_sizes() {
    let s = Server::new("");
    assert_eq!(s.widths(), [(0, 39, 0)]);
    assert_eq!(s.format("#{window_width}"), "80");
    assert_eq!(s.run(&["show-options", "-gv", "prefix"]), "C-a");
    s.run(&["resize-pane", "-W"]);
    assert_eq!(s.widths(), [(0, 80, 0)]);
    s.run(&["resize-pane", "-W"]);
    s.insert("%0", false);
    assert_eq!(s.widths(), [(0, 39, 0), (1, 39, 40)]);
    s.insert("%0", false);
    s.insert("%0", true);
    assert_eq!(
        s.widths(),
        [(3, 39, 0), (0, 39, 40), (2, 39, 80), (1, 39, 120)]
    );
    s.run(&["resize-pane", "-W", "-t", "%2"]);
    assert_eq!(
        s.widths(),
        [(3, 39, 0), (0, 39, 40), (2, 80, 80), (1, 39, 161)]
    );
    assert_eq!(s.format("#{pane_id}"), "%3");
    for (width, half) in [(81, 40), (1, 1), (101, 50), (80, 39)] {
        s.run(&["resize-window", "-x", &width.to_string(), "-y", "24"]);
        assert_eq!(
            s.widths().iter().map(|p| p.1).collect::<Vec<_>>(),
            [half, half, width, half]
        );
    }
    for (closed, remaining) in [
        ("%3", vec![(0, 39, 0), (2, 80, 40), (1, 39, 121)]),
        ("%2", vec![(0, 39, 0), (1, 39, 40)]),
        ("%1", vec![(0, 39, 0)]),
    ] {
        s.run(&["kill-pane", "-t", closed]);
        assert_eq!(s.widths(), remaining);
    }
    s.run(&["new-window", "-d", "-P", "-F", "#{pane_width}"]);
    assert_eq!(
        s.run(&["display-message", "-p", "-t", "scroll:1", "#{pane_width}"]),
        "39"
    );
}

#[test]
fn rejected_splits_and_resizes_preserve_zoom_layout_and_focus() {
    let s = Server::new("");
    s.insert("%0", false);
    s.run(&["resize-pane", "-Z"]);
    for command in [
        vec!["split-window"],
        vec!["splitw", "-h"],
        vec!["split-window", "-v", "exec sleep 60"],
        vec!["new-pane", "-L", "-x", "50%"],
        vec!["new-pane", "-L", "-h"],
    ] {
        s.reject(&command, "scrolling");
    }
    for command in [
        vec!["resize-pane"],
        vec!["resize-pane", "-x", "100%"],
        vec!["resize-pane", "-y", "10"],
        vec!["resize-pane", "-R", "5"],
        vec!["resize-pane", "-L"],
        vec!["resize-pane", "-M"],
        vec!["select-layout", "-E"],
    ] {
        s.reject(&command, "scrolling");
    }
    s.reject(&["resize-pane", "-W", "-Z"], "cannot be combined");
    s.run(&["resize-pane", "-Z"]);
    s.run(&["select-pane", "-t", "%0"]);
    s.run(&["select-pane", "-L"]);
    assert_eq!(s.format("#{pane_id}"), "%0");
    s.run(&["select-pane", "-R"]);
    s.run(&["select-pane", "-R"]);
    assert_eq!(s.format("#{pane_id}"), "%1");
}

#[test]
fn layout_restoration_zoom_and_classic_layouts_keep_their_policies() {
    let s = Server::new("");
    s.run(&["resize-pane", "-Z"]);
    assert_eq!(s.format("#{pane_width}:#{window_width}"), "80:80");
    s.run(&["resize-pane", "-Z"]);
    let lone = s.layout();
    s.run(&["select-layout", "even-horizontal"]);
    assert_eq!(s.widths(), [(0, 80, 0)]);
    s.run(&["select-layout", "-o"]);
    assert_eq!(s.layout(), lone);
    s.insert("%0", false);
    s.insert("%1", false);
    s.run(&["resize-pane", "-W", "-t", "%1"]);
    let mixed = s.layout();
    for name in [
        "even-horizontal",
        "even-vertical",
        "main-horizontal",
        "main-vertical",
        "tiled",
    ] {
        s.run(&["select-layout", name]);
        assert!(!s.layout().contains("\"scrolling\""));
        s.run(&["select-layout", "-o"]);
        assert_eq!(s.layout(), mixed);
    }
    s.run(&["select-layout", "even-horizontal"]);
    s.run(&["select-layout", "scrolling"]);
    assert_eq!(s.widths(), [(0, 39, 0), (1, 80, 40), (2, 39, 121)]);
    s.run(&["select-layout", "even-horizontal"]);
    s.run(&["select-layout", &mixed]);
    assert_eq!(s.layout(), mixed);
    s.run(&["resize-pane", "-Z", "-t", "%1"]);
    s.run(&["resize-window", "-x101", "-y30"]);
    assert_eq!(
        s.format("#{pane_width}:#{window_width}:#{window_zoomed_flag}"),
        "101:101:1"
    );
    s.run(&["resize-pane", "-Z"]);
    assert_eq!(s.widths(), [(0, 50, 0), (1, 101, 51), (2, 50, 153)]);
    s.run(&["select-layout", "even-horizontal"]);
    s.run(&["resize-window", "-x80"]);
    s.run(&["split-window", "-h"]);
    s.run(&["resize-pane", "-L", "2"]);
    assert!(!s.layout().contains("\"scrolling\""));
    s.run(&["respawn-window", "-k"]);
    assert_eq!(s.widths().len(), 1);
    assert_eq!(s.widths()[0].1, 80);
    assert!(!s.layout().contains("\"scrolling\""));
    s.run(&["select-layout", "scrolling"]);
    s.run(&["next-layout"]);
    assert!(!s.layout().contains("\"scrolling\""));
    s.run(&["previous-layout"]);
    assert!(s.layout().contains("\"scrolling\""));
}

#[test]
fn invalid_metadata_is_atomic_and_old_json_still_loads() {
    let s = Server::new("");
    s.insert("%0", false);
    s.run(&["resize-pane", "-Z"]);
    let layout = s.layout();
    for (broken, diagnostic) in [
        (
            layout.replace("\"width\":80", "\"width\":0"),
            "invalid scrolling width",
        ),
        (
            layout.replace("\"height\":24", "\"height\":10001"),
            "invalid scrolling height",
        ),
        (
            layout.replace("\"full\":false", "\"full\":3"),
            "expected a boolean",
        ),
        (
            layout.replace(
                "\"scrolling\":{\"width\":80,\"height\":24}",
                "\"scrolling\":true",
            ),
            "expected an object",
        ),
    ] {
        s.reject(&["select-layout", &broken], diagnostic);
    }
    s.run(&["select-layout", r#"{"V":2,"L":{"t":"h","w":80,"h":24,"x":0,"y":0,"c":[{"t":"p","w":40,"h":24,"x":0,"y":0,"i":0},{"t":"p","w":39,"h":24,"x":41,"y":0,"i":1}]}}"#]);
    assert_eq!(s.widths(), [(0, 40, 0), (1, 39, 41)]);
    s.run(&["split-window", "-h"]);
}

#[test]
fn reordering_moving_and_floating_panes_preserve_width_preferences() {
    let s = Server::new("");
    s.insert("%0", false);
    s.insert("%1", false);
    s.run(&["resize-pane", "-W", "-t", "%1"]);
    s.run(&["select-pane", "-t", "%1"]);
    s.run(&["swap-pane", "-U"]);
    assert_eq!(s.widths(), [(1, 80, 0), (0, 39, 81), (2, 39, 121)]);
    assert_eq!(s.format("#{pane_id}"), "%1");
    let edge = s.layout();
    s.run(&["swap-pane", "-U"]);
    assert_eq!(s.layout(), edge);
    s.run(&["swap-pane", "-D"]);
    s.run(&["swap-pane", "-D"]);
    assert_eq!(s.widths(), [(0, 39, 0), (2, 39, 40), (1, 80, 80)]);
    let edge = s.layout();
    s.run(&["swap-pane", "-D"]);
    assert_eq!(s.layout(), edge);
    s.run(&["rotate-window"]);
    assert!(s.widths().iter().any(|p| *p == (1, 80, 40)));
    s.run(&["break-pane", "-W", "-s", "%1"]);
    assert!(s.panes().iter().find(|p| p.0 == 1).unwrap().5);
    s.run(&["resize-pane", "-t", "%1", "-x20", "-y10"]);
    let tiles = s.widths();
    s.run(&["new-pane", "-d", "-t", "%0", "-x15", "-y8"]);
    assert_eq!(s.widths(), tiles);
    s.run(&["join-pane", "-s", "%1", "-t", "%1"]);
    assert_eq!(s.panes().iter().find(|p| p.0 == 1).unwrap().1, 80);
    s.run(&["break-pane", "-d", "-s", "%1"]);
    assert_eq!(s.widths().len(), 2);
    s.run(&["join-pane", "-s", "%1", "-t", "%0"]);
    assert_eq!(s.widths().iter().find(|p| p.0 == 1).unwrap().1, 80);
    s.run(&["move-pane", "-s", "%1", "-t", "%2", "-b"]);
    let order = s.widths();
    assert_eq!(
        order.iter().position(|p| p.0 == 1).unwrap() + 1,
        order.iter().position(|p| p.0 == 2).unwrap()
    );
}

#[test]
fn pane_status_scrollbars_and_spawn_environment() {
    let s = Server::new("");
    s.insert("%0", false);
    s.run(&["set-option", "-w", "pane-border-status", "top"]);
    assert!(s.panes().iter().all(|p| p.2 == 23));
    s.run(&["set-option", "-w", "pane-border-status", "bottom"]);
    assert!(s.panes().iter().all(|p| p.2 == 23));
    s.run(&["set-option", "-w", "pane-scrollbars", "on"]);
    let widths = s.widths();
    assert!(widths.iter().all(|p| p.1 < 39));
    assert_eq!(widths[1].2, 40);
    s.run(&["resize-window", "-x1", "-y1"]);
    assert!(s.panes().iter().all(|p| p.1 >= 1 && p.2 >= 1));
    s.run(&["resize-pane", "-Z"]);
    assert_eq!(s.format("#{window_height}"), "2");
    s.run(&["resize-pane", "-Z"]);
    s.run(&["resize-window", "-x80", "-y24"]);
    assert_eq!(s.widths(), widths);
    s.run(&["set-option", "-w", "pane-scrollbars", "off"]);
    let cwd = s.root.join("directory with spaces");
    fs::create_dir(&cwd).unwrap();
    s.run(&[
        "send-keys",
        "-t",
        "%0",
        &format!("cd '{}'", cwd.display()),
        "Enter",
    ]);
    s.await_value(
        || s.run(&["display-message", "-p", "-t", "%0", "#{pane_current_path}"]),
        cwd.to_str().unwrap(),
    );
    s.run(&["select-pane", "-t", "%0"]);
    s.run(&["set-environment", "SCROLL_SESSION", "session-value"]);
    let output = s.root.join("environment");
    let script = format!("printf '%s\\n' \"$PWD\" \"$(pwd)\" \"$SCROLL_SESSION\" \"$SCROLL_OVERRIDE\" \"$TMUX_PANE\" > '{}'; exec sleep 60", output.display());
    s.run(&[
        "new-pane",
        "-L",
        "-t",
        "%0",
        "-c",
        "#{pane_current_path}",
        "-e",
        "SCROLL_OVERRIDE=first",
        "-e",
        "SCROLL_OVERRIDE=second",
        &script,
    ]);
    let expected = format!(
        "{}\n{}\nsession-value\nsecond\n%2\n",
        cwd.display(),
        cwd.display()
    );
    s.await_value(
        || fs::read_to_string(&output).unwrap_or_default(),
        &expected,
    );
    let fallback = s.root.join("fallback directory");
    fs::create_dir(&fallback).unwrap();
    let cwd_output = s.root.join("fallback-cwd");
    let script = format!(
        "printf '%s\\n' \"$PWD\" \"$(pwd)\" > '{}'; exec sleep 60",
        cwd_output.display()
    );
    let result = s
        .command()
        .current_dir(&fallback)
        .args(["new-pane", "-L", "-t", "%0", &script])
        .output()
        .unwrap();
    assert!(result.status.success());
    let expected = format!("{}\n{}\n", fallback.display(), fallback.display());
    s.await_value(
        || fs::read_to_string(&cwd_output).unwrap_or_default(),
        &expected,
    );
    // A missing requested directory retains the existing HOME fallback.
    let expected = format!("{}\n{}\n", s.root.display(), s.root.display());
    fs::remove_file(&cwd_output).unwrap();
    s.run(&[
        "new-pane",
        "-L",
        "-t",
        "%0",
        "-c",
        s.root.join("missing").to_str().unwrap(),
        &script,
    ]);
    s.await_value(
        || fs::read_to_string(&cwd_output).unwrap_or_default(),
        &expected,
    );
}

// A small terminal recorder for the cursor movement and erasure emitted by
// xterm-256color. Tests check the final cells as well as command-side geometry.
mod terminal {
    use super::*;
    use std::fs::File;
    use std::io::{ErrorKind, Read, Write};
    use std::os::fd::{AsRawFd, FromRawFd};
    use std::os::unix::process::CommandExt;

    pub struct Screen {
        pub cells: Vec<Vec<char>>,
        pub x: usize,
        pub y: usize,
        pub cursor_visible: bool,
        pending: Vec<u8>,
        saved: (usize, usize),
        region: (usize, usize),
    }

    impl Screen {
        fn new(width: usize, height: usize) -> Self {
            Self {
                cells: vec![vec![' '; width]; height],
                x: 0,
                y: 0,
                cursor_visible: true,
                pending: Vec::new(),
                saved: (0, 0),
                region: (0, height - 1),
            }
        }
        pub fn row(&self, y: usize) -> String {
            self.cells[y].iter().collect()
        }
        fn newline(&mut self) {
            if self.y == self.region.1 {
                let width = self.cells[0].len();
                self.cells.remove(self.region.0);
                self.cells.insert(self.region.1, vec![' '; width]);
            } else {
                self.y = (self.y + 1).min(self.cells.len() - 1);
            }
        }
        fn csi(&mut self, bytes: &[u8], command: u8) {
            let private = bytes.first() == Some(&b'?');
            let params = String::from_utf8_lossy(bytes);
            let values: Vec<usize> = params
                .trim_start_matches('?')
                .split(';')
                .map(|n| n.parse().unwrap_or(0))
                .collect();
            let get = |i: usize, default: usize| {
                values
                    .get(i)
                    .copied()
                    .filter(|n| *n != 0)
                    .unwrap_or(default)
            };
            let w = self.cells[0].len();
            let h = self.cells.len();
            match command {
                b'H' | b'f' => {
                    self.y = get(0, 1).saturating_sub(1).min(h - 1);
                    self.x = get(1, 1).saturating_sub(1).min(w - 1);
                }
                b'A' => self.y = self.y.saturating_sub(get(0, 1)),
                b'B' | b'e' => self.y = (self.y + get(0, 1)).min(h - 1),
                b'C' | b'a' => self.x = (self.x + get(0, 1)).min(w - 1),
                b'D' => self.x = self.x.saturating_sub(get(0, 1)),
                b'G' | b'`' => self.x = get(0, 1).saturating_sub(1).min(w - 1),
                b'd' => self.y = get(0, 1).saturating_sub(1).min(h - 1),
                b'E' => {
                    self.y = (self.y + get(0, 1)).min(h - 1);
                    self.x = 0;
                }
                b'F' => {
                    self.y = self.y.saturating_sub(get(0, 1));
                    self.x = 0;
                }
                b'J' => match values[0] {
                    0 => {
                        self.cells[self.y][self.x.min(w)..].fill(' ');
                        for row in &mut self.cells[self.y + 1..] {
                            row.fill(' ');
                        }
                    }
                    1 => {
                        for row in &mut self.cells[..self.y] {
                            row.fill(' ');
                        }
                        self.cells[self.y][..=self.x.min(w - 1)].fill(' ');
                    }
                    2 | 3 => {
                        for row in &mut self.cells {
                            row.fill(' ');
                        }
                    }
                    _ => (),
                },
                b'K' => match values[0] {
                    0 => self.cells[self.y][self.x.min(w)..].fill(' '),
                    1 => self.cells[self.y][..=self.x.min(w - 1)].fill(' '),
                    2 => self.cells[self.y].fill(' '),
                    _ => (),
                },
                b'X' => {
                    let end = (self.x + get(0, 1)).min(w);
                    self.cells[self.y][self.x.min(w)..end].fill(' ');
                }
                b'P' => {
                    let n = get(0, 1).min(w - self.x.min(w));
                    self.cells[self.y].drain(self.x.min(w)..self.x.min(w) + n);
                    self.cells[self.y].resize(w, ' ');
                }
                b'@' => {
                    for _ in 0..get(0, 1).min(w - self.x.min(w)) {
                        self.cells[self.y].insert(self.x, ' ');
                        self.cells[self.y].pop();
                    }
                }
                b'b' => {
                    let ch = self.cells[self.y][self.x.saturating_sub(1).min(w - 1)];
                    for _ in 0..get(0, 1) {
                        self.put(ch);
                    }
                }
                b'r' => {
                    self.region = (
                        get(0, 1).saturating_sub(1).min(h - 1),
                        get(1, h).saturating_sub(1).min(h - 1),
                    )
                }
                b'L' => {
                    for _ in 0..get(0, 1).min(h - self.y) {
                        self.cells.remove(self.region.1);
                        self.cells.insert(self.y, vec![' '; w]);
                    }
                }
                b'M' => {
                    for _ in 0..get(0, 1).min(h - self.y) {
                        self.cells.remove(self.y);
                        self.cells.insert(self.region.1, vec![' '; w]);
                    }
                }
                b'h' | b'l' if private => {
                    if values.contains(&25) {
                        self.cursor_visible = command == b'h';
                    }
                    if values.contains(&1049) && command == b'h' {
                        for row in &mut self.cells {
                            row.fill(' ');
                        }
                        self.x = 0;
                        self.y = 0;
                    }
                }
                _ => (),
            }
        }
        fn put(&mut self, ch: char) {
            if self.x == self.cells[0].len() {
                self.x = 0;
                self.newline();
            }
            self.cells[self.y][self.x] = ch;
            self.x += 1;
        }
        fn feed(&mut self, bytes: &[u8]) {
            self.pending.extend_from_slice(bytes);
            let bytes = std::mem::take(&mut self.pending);
            let mut i = 0;
            while i < bytes.len() {
                let start = i;
                if bytes[i] == 27 {
                    if i + 1 == bytes.len() {
                        break;
                    }
                    match bytes[i + 1] {
                        b'[' => {
                            let Some(end) =
                                (i + 2..bytes.len()).find(|j| (0x40..=0x7e).contains(&bytes[*j]))
                            else {
                                break;
                            };
                            self.csi(&bytes[i + 2..end], bytes[end]);
                            i = end + 1;
                        }
                        b']' | b'P' | b'_' => {
                            let Some(end) = (i + 2..bytes.len()).find(|j| {
                                bytes[*j] == 7
                                    || (bytes[*j] == 27 && bytes.get(*j + 1) == Some(&b'\\'))
                            }) else {
                                break;
                            };
                            i = end + if bytes[end] == 7 { 1 } else { 2 };
                        }
                        b'(' | b')' | b'%' => {
                            if i + 2 == bytes.len() {
                                break;
                            }
                            i += 3;
                        }
                        b'7' => {
                            self.saved = (self.x, self.y);
                            i += 2;
                        }
                        b'8' => {
                            (self.x, self.y) = self.saved;
                            i += 2;
                        }
                        b'D' => {
                            self.newline();
                            i += 2;
                        }
                        b'M' => {
                            self.y = self.y.saturating_sub(1);
                            i += 2;
                        }
                        _ => i += 2,
                    }
                } else {
                    let byte = bytes[i];
                    i += 1;
                    match byte {
                        b'\r' => self.x = 0,
                        b'\n' => self.newline(),
                        b'\x08' => self.x = self.x.saturating_sub(1),
                        b'\t' => self.x = ((self.x / 8 + 1) * 8).min(self.cells[0].len() - 1),
                        0..=31 | 127 => (),
                        32..=126 => self.put(byte as char),
                        _ => {
                            let n = if byte < 0xe0 {
                                2
                            } else if byte < 0xf0 {
                                3
                            } else {
                                4
                            };
                            if start + n > bytes.len() {
                                i = start;
                                break;
                            }
                            self.put(
                                std::str::from_utf8(&bytes[start..start + n])
                                    .unwrap()
                                    .chars()
                                    .next()
                                    .unwrap(),
                            );
                            i = start + n;
                        }
                    }
                }
            }
            self.pending.extend_from_slice(&bytes[i..]);
        }
    }

    pub struct Client {
        master: File,
        child: Child,
        pub screen: Screen,
        pub tty: String,
    }

    impl Client {
        pub fn new(server: &Server, width: u16, height: u16) -> Self {
            let mut master = -1;
            let mut slave = -1;
            let size = libc::winsize {
                ws_row: height,
                ws_col: width,
                ws_xpixel: 0,
                ws_ypixel: 0,
            };
            assert_eq!(
                unsafe {
                    libc::openpty(
                        &mut master,
                        &mut slave,
                        std::ptr::null_mut(),
                        std::ptr::null(),
                        &size,
                    )
                },
                0
            );
            let master = unsafe { File::from_raw_fd(master) };
            let slave = unsafe { File::from_raw_fd(slave) };
            let mut command = server.command();
            command
                .args(["attach-session", "-t", "scroll"])
                .stdin(slave.try_clone().unwrap())
                .stdout(slave.try_clone().unwrap())
                .stderr(slave);
            unsafe {
                command.pre_exec(|| {
                    if libc::setsid() == -1 || libc::ioctl(0, libc::TIOCSCTTY, 0) == -1 {
                        return Err(std::io::Error::last_os_error());
                    }
                    Ok(())
                });
            }
            let child = command.spawn().unwrap();
            assert_ne!(
                unsafe { libc::fcntl(master.as_raw_fd(), libc::F_SETFL, libc::O_NONBLOCK) },
                -1
            );
            let mut client = Self {
                master,
                child,
                screen: Screen::new(width as usize, height as usize),
                tty: String::new(),
            };
            let deadline = Instant::now() + TIMEOUT;
            loop {
                let clients = server.run(&["list-clients", "-F", "#{client_pid} #{client_tty}"]);
                if let Some(line) = clients
                    .lines()
                    .find(|line| line.starts_with(&format!("{} ", client.child.id())))
                {
                    client.tty = line.split_once(' ').unwrap().1.to_owned();
                    break;
                }
                assert!(Instant::now() < deadline, "client did not attach");
                client.drain();
                thread::sleep(Duration::from_millis(10));
            }
            client
        }
        pub fn drain(&mut self) {
            loop {
                let mut bytes = [0; 65536];
                match self.master.read(&mut bytes) {
                    Ok(0) => return,
                    Ok(n) => self.screen.feed(&bytes[..n]),
                    Err(e) if e.kind() == ErrorKind::WouldBlock => return,
                    Err(e) => panic!("terminal read: {e}"),
                }
            }
        }
        pub fn send(&mut self, bytes: &[u8]) {
            self.master.write_all(bytes).unwrap();
        }
        pub fn wait_screen(&mut self, condition: impl Fn(&Screen) -> bool) {
            let deadline = Instant::now() + TIMEOUT;
            loop {
                self.drain();
                if condition(&self.screen) {
                    return;
                }
                assert!(
                    Instant::now() < deadline,
                    "terminal mismatch: cursor {},{}; rows {:?}",
                    self.screen.x,
                    self.screen.y,
                    self.screen
                        .cells
                        .iter()
                        .map(|r| r.iter().collect::<String>())
                        .collect::<Vec<_>>()
                );
                thread::sleep(Duration::from_millis(10));
            }
        }
        pub fn resize(&mut self, width: u16, height: u16) {
            let size = libc::winsize {
                ws_row: height,
                ws_col: width,
                ws_xpixel: 0,
                ws_ypixel: 0,
            };
            assert_eq!(
                unsafe { libc::ioctl(self.master.as_raw_fd(), libc::TIOCSWINSZ, &size) },
                0
            );
            self.screen = Screen::new(width as usize, height as usize);
        }
        pub fn format(&self, server: &Server, format: &str) -> String {
            server.run(&["display-message", "-p", "-c", &self.tty, format])
        }
    }
    impl Drop for Client {
        fn drop(&mut self) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

#[test]
fn prefix_keys_current_directory_passthrough_and_configuration_overrides() {
    let s = Server::new("set -g status off\n");
    let mut c = terminal::Client::new(&s, 80, 24);
    let cwd = s.root.join("key binding directory");
    fs::create_dir(&cwd).unwrap();
    c.send(format!("cd '{}'\n", cwd.display()).as_bytes());
    s.await_value(|| s.format("#{pane_current_path}"), cwd.to_str().unwrap());
    c.send(b"\x01c");
    s.await_value(|| s.format("#{window_panes}"), "2");
    assert_eq!(s.format("#{pane_current_path}"), cwd.to_str().unwrap());
    assert_eq!(s.format("#{pane_id}"), "%1");
    c.send(b"\x01h");
    s.await_value(|| s.format("#{pane_id}"), "%0");
    c.send(b"\x01h");
    c.send(b"\x01l");
    s.await_value(|| s.format("#{pane_id}"), "%1");
    c.send(b"\x01f");
    s.await_value(|| s.format("#{pane_width}"), "80");
    c.send(b"\x01H");
    s.await_value(|| s.format("#{pane_left}"), "0");
    assert_eq!(s.format("#{pane_id}"), "%1");
    c.send(b"\x01L");
    s.await_value(|| s.format("#{pane_left}"), "40");
    c.send(b"\x01x");
    s.await_value(|| s.format("#{window_panes}"), "1");
    let output = s.root.join("prefix-byte");
    c.send(
        format!(
            "stty -echo -icanon min 1 time 0; dd bs=1 count=1 of='{}' 2>/dev/null; stty sane\n",
            output.display()
        )
        .as_bytes(),
    );
    // Wait until dd has created the destination before sending the prefix.
    s.await_value(|| output.exists().to_string(), "true");
    c.send(b"\x01\x01");
    s.await_value(
        || {
            fs::read(&output)
                .unwrap_or_default()
                .first()
                .copied()
                .unwrap_or(0)
                .to_string()
        },
        "1",
    );

    let configured =
        Server::new("set -g prefix C-b\nbind c new-window\nbind f display-message 'custom f'\n");
    assert_eq!(configured.run(&["show-options", "-gv", "prefix"]), "C-b");
    assert!(configured
        .run(&["list-keys", "-T", "prefix", "c"])
        .contains("new-window"));
    assert!(configured
        .run(&["list-keys", "-T", "prefix", "f"])
        .contains("custom f"));
    configured.run(&["select-layout", "even-horizontal"]);
    let mut customized = terminal::Client::new(&configured, 60, 24);
    configured.await_value(|| configured.format("#{pane_width}"), "60");
    assert!(!configured.layout().contains("\"scrolling\""));
    customized.send(b"\x02c");
    configured.await_value(|| customized.format(&configured, "#{window_index}"), "1");
    assert_eq!(customized.format(&configured, "#{pane_width}"), "29");
}

#[test]
fn focus_pan_resize_and_multiple_clients_keep_independent_viewports() {
    let s = Server::new("set -g status off\nset -g mouse on\n");
    let mut c = terminal::Client::new(&s, 80, 24);
    s.insert("%0", false);
    s.insert("%1", false);
    s.insert("%2", false);
    s.await_value(|| c.format(&s, "#{window_offset_x}"), "79");
    s.run(&["select-pane", "-t", "%2"]);
    s.await_value(|| c.format(&s, "#{window_offset_x}"), "79");
    s.run(&["select-pane", "-t", "%1"]);
    s.await_value(|| c.format(&s, "#{window_offset_x}"), "39");
    s.run(&["select-pane", "-t", "%0"]);
    s.await_value(|| c.format(&s, "#{window_offset_x}"), "0");
    s.run(&["select-pane", "-L"]);
    assert_eq!(c.format(&s, "#{window_offset_x}"), "0");
    s.run(&["refresh-client", "-t", &c.tty, "-R", "20"]);
    assert_eq!(c.format(&s, "#{window_offset_x}"), "20");
    s.run(&["select-pane", "-t", "%1"]);
    assert_eq!(c.format(&s, "#{window_offset_x}"), "20");
    s.run(&["select-pane", "-t", "%0"]);
    assert_eq!(c.format(&s, "#{window_offset_x}"), "0");
    s.run(&["refresh-client", "-t", &c.tty, "-R", "50"]);
    s.run(&["refresh-client", "-t", &c.tty, "-c"]);
    assert_eq!(c.format(&s, "#{window_offset_x}"), "0");
    // Cursor output in a visible pane does not move the viewport.
    s.run(&[
        "send-keys",
        "-t",
        "%0",
        "printf '\\033[20Gx\\033[2Gy'",
        "Enter",
    ]);
    c.wait_screen(|screen| screen.cells.iter().any(|row| row.contains(&'y')));
    assert_eq!(c.format(&s, "#{window_offset_x}"), "0");
    s.run(&["set-option", "-w", "window-size", "largest"]);
    s.run(&["resize-pane", "-W", "-t", "%2"]);
    s.run(&["select-pane", "-t", "%2"]);
    let mut narrow = terminal::Client::new(&s, 40, 24);
    s.await_value(|| s.format("#{pane_width}"), "80");
    s.run(&[
        "send-keys",
        "-t",
        "%2",
        "printf '\\033[1;79H!'; sleep 60",
        "Enter",
    ]);
    s.await_value(|| narrow.format(&s, "#{window_offset_x}"), "120");
    narrow.wait_screen(|screen| screen.cells[0][38] == '!' && screen.x == 39);
    assert_eq!(c.format(&s, "#{window_offset_x}"), "80");
    let narrow_offset: u32 = narrow.format(&s, "#{window_offset_x}").parse().unwrap();
    assert!((80..=120).contains(&narrow_offset));
    assert_ne!(narrow_offset, 80, "narrow client follows the far cursor");
    assert_eq!(s.format("#{pane_width}"), "80");
    s.run(&[
        "respawn-pane",
        "-k",
        "-t",
        "%2",
        "printf '\\033[1;80H@'; exec sleep 60",
    ]);
    narrow.wait_screen(|screen| {
        screen.cells[0][39] == '@' && screen.cursor_visible && screen.x == 39
    });
    assert_eq!(narrow.format(&s, "#{window_offset_x}"), "120");
    drop(narrow);
    c.resize(101, 30);
    s.await_value(|| s.format("#{pane_width}"), "101");
    c.resize(61, 20);
    s.await_value(|| s.format("#{pane_width}"), "61");
    assert_eq!(
        s.widths().iter().map(|p| p.1).collect::<Vec<_>>(),
        [30, 30, 61, 30]
    );
}

#[test]
fn scrolled_rendering_clips_neighbors_cursor_and_mouse_coordinates() {
    let s = Server::new("set -g status off\nset -g mouse on\n");
    let mut c = terminal::Client::new(&s, 80, 24);
    s.insert("%0", false);
    s.insert("%1", false);
    let paint = |id: &str, ch: char| {
        let script = format!(
            "printf '\\033[2J\\033[2;1H{}\\033[4;5H'; exec sleep 60",
            ch.to_string().repeat(39)
        );
        s.run(&["respawn-pane", "-k", "-t", id, &script]);
        s.await_value(
            || {
                s.run(&["capture-pane", "-p", "-t", id])
                    .lines()
                    .nth(1)
                    .unwrap_or("")
                    .to_owned()
            },
            &ch.to_string().repeat(39),
        );
    };
    paint("%0", 'A');
    paint("%1", 'B');
    paint("%2", 'C');
    s.await_value(|| c.format(&s, "#{window_offset_x}"), "39");
    c.wait_screen(|screen| {
        screen.row(1).contains(&"B".repeat(39))
            && screen.row(1).ends_with(&"C".repeat(39))
            && screen.x == 45
            && screen.y == 3
    });
    assert_ne!(c.screen.cells[1][0], ' ');
    assert_ne!(c.screen.cells[1][40], ' ');
    paint("%0", 'D');
    c.drain();
    assert!(
        !c.screen.row(1).contains('D'),
        "offscreen output leaked into the viewport"
    );
    assert_eq!(c.format(&s, "#{window_offset_x}"), "39");
    s.run(&["select-pane", "-t", "%0"]);
    s.run(&["refresh-client", "-t", &c.tty, "-R", "10"]);
    c.wait_screen(|screen| {
        screen.row(1).starts_with(&"D".repeat(29))
            && screen.row(1).contains(&"B".repeat(39))
            && screen.row(1).ends_with(&"C".repeat(10))
            && !screen.cursor_visible
    });
    // Screen column 51 is canvas column 60 after panning, inside pane 1.
    c.send(b"\x1b[<0;51;2M\x1b[<0;51;2m");
    s.await_value(|| s.format("#{pane_id}"), "%1");
    assert_eq!(c.format(&s, "#{window_offset_x}"), "10");
    c.wait_screen(|screen| screen.cursor_visible && screen.x == 34 && screen.y == 3);
    let before = s.layout();
    c.send(b"\x1b[<0;30;5M\x1b[<32;35;5M\x1b[<0;35;5m");
    // A following key gives a command-queue barrier after the mouse events.
    s.run(&[
        "bind-key",
        "-n",
        "F12",
        "set-option",
        "-w",
        "@drag-done",
        "yes",
    ]);
    c.send(b"\x1b[24~");
    s.await_value(|| s.format("#{@drag-done}"), "yes");
    assert_eq!(s.layout(), before);
}

#[test]
fn large_canvas_round_trips_and_limits_are_checked() {
    let s = Server::new("");
    s.run(&["resize-window", "-x10000", "-y1"]);
    s.insert("%0", false);
    s.insert("%1", false);
    s.run(&["resize-pane", "-W", "-t", "%1"]);
    assert_eq!(
        s.widths(),
        [(0, 4999, 0), (1, 10000, 5000), (2, 4999, 15001)]
    );
    assert_eq!(s.format("#{window_width}"), "20000");
    let layout = s.layout();
    s.run(&["select-layout", "even-horizontal"]);
    s.run(&["select-layout", &layout]);
    assert_eq!(s.layout(), layout);
    s.reject(
        &[
            "select-layout",
            &layout.replace("\"width\":10000", "\"width\":10001"),
        ],
        "invalid scrolling width",
    );
    s.reject(&["resize-window", "-x10001"], "too large");
}

#[test]
fn legacy_control_output_keeps_internal_previous_layout_metadata() {
    use std::io::{BufRead, BufReader, Write};
    use std::sync::mpsc;
    let s = Server::new("set -g status off\n");
    s.insert("%0", false);
    s.insert("%1", false);
    s.run(&["resize-pane", "-W", "-t", "%1"]);
    let expected = s.widths();
    let mut child = s
        .command()
        .args(["-C", "attach-session", "-t", "scroll"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let stdout = child.stdout.take().unwrap();
    let (sender, receiver) = mpsc::channel();
    let reader = thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            if sender.send(line).is_err() {
                break;
            }
        }
    });
    let wait = |prefix: &str| -> String {
        let deadline = Instant::now() + TIMEOUT;
        loop {
            let line = receiver
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                .unwrap();
            if let Some(value) = line.strip_prefix(prefix) {
                return value.to_owned();
            }
        }
    };
    writeln!(
        child.stdin.as_mut().unwrap(),
        "display-message -p 'LEGACY=#{{window_layout}}'"
    )
    .unwrap();
    let legacy = wait("LEGACY=");
    assert!(!legacy.starts_with('{'));
    assert!(legacy.contains("160x24"), "{legacy}");
    writeln!(
        child.stdin.as_mut().unwrap(),
        "select-layout even-horizontal\nselect-layout -o\ndisplay-message -p RESTORED"
    )
    .unwrap();
    wait("RESTORED");
    assert_eq!(s.widths(), expected);
    // Changing an unrelated built-in option recalculates window sizes. The
    // unchanged 80-cell sizing basis must not resize the 160-cell canvas again.
    while receiver.recv_timeout(Duration::from_millis(50)).is_ok() {}
    writeln!(
        child.stdin.as_mut().unwrap(),
        "set-option -w aggressive-resize off\ndisplay-message -p SIZE-CHECKED"
    )
    .unwrap();
    let deadline = Instant::now() + TIMEOUT;
    loop {
        let line = receiver
            .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .unwrap();
        assert!(
            !line.starts_with("%layout-change"),
            "unchanged sizing basis triggered a resize: {line}"
        );
        if line == "SIZE-CHECKED" {
            break;
        }
    }
    while let Ok(line) = receiver.recv_timeout(Duration::from_millis(50)) {
        assert!(
            !line.starts_with("%layout-change"),
            "redundant deferred resize: {line}"
        );
    }
    // Behavioral proof that -o retained scrolling mode despite legacy output.
    s.run(&["resize-pane", "-W", "-t", "%1"]);
    assert_eq!(
        s.widths().iter().map(|p| p.1).collect::<Vec<_>>(),
        [39, 39, 39]
    );
    let _ = child.kill();
    let _ = child.wait();
    reader.join().unwrap();
    s.run(&["select-layout", &legacy]);
    assert!(!s.layout().contains("\"scrolling\""));
    s.run(&["split-window", "-h", "-t", "%1"]);
}

#[cfg(target_os = "linux")]
#[test]
fn failed_fork_restores_strip_focus_and_zoom() {
    if unsafe { libc::geteuid() } == 0 {
        return; // RLIMIT_NPROC is not enforced for root.
    }
    let s = Server::new("");
    s.insert("%0", false);
    s.run(&["resize-pane", "-W", "-t", "%0"]);
    for zoom in [false, true] {
        if zoom {
            s.run(&["resize-pane", "-Z"]);
        }
        let before = s.layout();
        let geometry = s.format("#{window_width}:#{pane_id}:#{window_zoomed_flag}");
        let mut previous = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        assert_eq!(
            unsafe {
                libc::prlimit(
                    s.child.id() as i32,
                    libc::RLIMIT_NPROC,
                    std::ptr::null(),
                    &mut previous,
                )
            },
            0
        );
        let blocked = libc::rlimit {
            rlim_cur: 0,
            rlim_max: previous.rlim_max,
        };
        assert_eq!(
            unsafe {
                libc::prlimit(
                    s.child.id() as i32,
                    libc::RLIMIT_NPROC,
                    &blocked,
                    std::ptr::null_mut(),
                )
            },
            0
        );
        let output = s.output(&["new-pane", "-L"]);
        assert_eq!(
            unsafe {
                libc::prlimit(
                    s.child.id() as i32,
                    libc::RLIMIT_NPROC,
                    &previous,
                    std::ptr::null_mut(),
                )
            },
            0
        );
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("fork failed"));
        assert_eq!(s.layout(), before);
        assert_eq!(
            s.format("#{window_width}:#{pane_id}:#{window_zoomed_flag}"),
            geometry
        );
    }
    s.run(&["new-pane", "-L"]);
    assert_eq!(s.widths().len(), 3);
}

#[test]
fn floating_overlays_survive_rotation_zoom_and_all_floating_restoration() {
    let s = Server::new("");
    s.insert("%0", false);
    s.run(&["new-pane", "-d", "-x20", "-y10", "-X10", "-Y3"]);
    let floating = || {
        s.run(&[
            "display-message",
            "-p",
            "-t",
            "%2",
            "#{pane_width}:#{pane_height}:#{pane_left}:#{pane_top}:#{pane_floating_flag}",
        ])
    };
    let original = floating();
    s.run(&["rotate-window"]);
    assert_eq!(floating(), original);
    s.run(&["select-pane", "-t", "%2"]);
    s.run(&["rotate-window", "-D"]);
    assert_eq!(s.format("#{pane_id}"), "%2");
    assert_eq!(floating(), original);
    s.run(&["resize-pane", "-Z"]);
    s.reject(&["resize-pane", "-W"], "requires a tiled pane");
    s.run(&["resize-pane", "-x15"]);
    assert_eq!(
        s.format("#{window_zoomed_flag}:#{pane_width}:#{pane_floating_flag}"),
        "0:13:1"
    );
    for pane in ["%0", "%1"] {
        s.run(&["break-pane", "-W", "-s", pane]);
    }
    assert!(s.widths().is_empty());
    let all_floating = s.layout();
    s.run(&["select-layout", "even-horizontal"]);
    s.run(&["select-layout", "-o"]);
    assert_eq!(s.layout(), all_floating);
    s.run(&["kill-pane", "-t", "%1"]);
    s.run(&["kill-pane", "-t", "%2"]);
    let lone = s.layout();
    s.run(&["select-layout", &lone]);
    assert_eq!(s.layout(), lone);
    s.run(&["join-pane", "-s", "%0", "-t", "%0"]);
    assert_eq!(s.widths(), [(0, 39, 0)]);
    s.run(&["new-pane", "-L", "-Z"]);
    assert_eq!(s.format("#{window_zoomed_flag}:#{pane_width}"), "1:80");
    s.run(&["resize-pane", "-Z"]);
    assert_eq!(s.widths().iter().map(|p| p.1).collect::<Vec<_>>(), [39, 39]);
}

#[test]
fn cross_window_swap_and_pane_exit_reflow_using_the_destination_basis() {
    let s = Server::new("");
    s.insert("%0", false);
    s.run(&["resize-pane", "-W", "-t", "%1"]);
    s.run(&["new-window", "-d"]);
    s.run(&["resize-window", "-t", "scroll:1", "-x120", "-y15"]);
    s.run(&["swap-pane", "-s", "%1", "-t", "%2"]);
    assert_eq!(s.widths(), [(0, 39, 0), (2, 39, 40)]);
    assert_eq!(
        s.run(&[
            "display-message",
            "-p",
            "-t",
            "%1",
            "#{pane_width}:#{pane_height}"
        ]),
        "120:15"
    );
    s.run(&["respawn-pane", "-k", "-t", "%2", "exit 0"]);
    s.await_value(|| s.format("#{window_panes}"), "1");
    assert_eq!(s.widths(), [(0, 39, 0)]);
    for target in ["%0", "%3", "%4"] {
        s.insert(target, false);
    }
    s.run(&["resize-pane", "-W", "-t", "%3"]);
    s.run(&["break-pane", "-d", "-s", "%0"]);
    assert_eq!(
        s.run(&[
            "display-message",
            "-p",
            "-t",
            "%0",
            "#{pane_width}:#{window_width}"
        ]),
        "39:80"
    );
}

#[test]
fn closing_the_active_pane_keeps_the_previous_viewport_as_its_starting_point() {
    let s = Server::new("set -g status off\n");
    let c = terminal::Client::new(&s, 80, 24);
    for target in ["%0", "%1", "%2", "%3"] {
        s.insert(target, false);
    }
    s.run(&["select-pane", "-t", "%1"]);
    s.run(&["select-pane", "-t", "%2"]);
    assert_eq!(c.format(&s, "#{window_offset_x}"), "40");
    s.run(&["kill-pane", "-t", "%2"]);
    assert_eq!(s.format("#{pane_id}"), "%1");
    assert_eq!(c.format(&s, "#{window_offset_x}"), "39");
}
