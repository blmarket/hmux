//! Window size is the `window-size` policy result; layout may extend past it
//! without feeding back into it.

#![cfg(unix)]

use std::fs;
use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

struct Server {
    directory: PathBuf,
    socket: PathBuf,
}

impl Server {
    fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        let directory =
            std::env::temp_dir().join(format!("hmux-window-sizes-{}-{stamp}", std::process::id()));
        fs::create_dir(&directory).expect("create socket directory");
        let socket = directory.join("socket");
        Self { directory, socket }
    }

    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_hmux"));
        command
            .args(["-f", "/dev/null", "-S"])
            .arg(&self.socket)
            .env("TERM", "xterm-256color")
            .env("SHELL", "/bin/sh")
            .env("TMUX", "");
        command
    }

    fn run(&self, args: &[&str]) -> Output {
        self.command().args(args).output().expect("run hmux")
    }

    fn success(&self, args: &[&str]) -> String {
        let output = self.run(args);
        assert!(
            output.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).expect("UTF-8 command output")
    }

    fn display(&self, format: &str) -> String {
        self.success(&["display-message", "-p", format])
            .trim_end()
            .to_owned()
    }

    /// Start a manually sized window with `panes` side by side (or stacked).
    fn start(&self, sx: u32, sy: u32, panes: usize, split: &str) {
        self.success(&[
            "new-session",
            "-d",
            "-x",
            &sx.to_string(),
            "-y",
            &sy.to_string(),
            "sleep 60",
        ]);
        self.success(&["set", "-g", "window-size", "manual"]);
        for _ in 1..panes {
            self.success(&["split-window", split, "sleep 60"]);
        }
    }

    fn window_size(&self) -> (u32, u32) {
        let size = self.display("#{window_width}x#{window_height}");
        let (sx, sy) = size.split_once('x').expect("size format");
        (sx.parse().unwrap(), sy.parse().unwrap())
    }

    /// The tiled root's size, as layout strings report it.
    fn root_size(&self) -> (u32, u32) {
        let layout = self.display("#{window_layout}");
        let root = &layout[layout.find("\"L\":{").expect("layout root")..];
        (field(root, "\"w\":"), field(root, "\"h\":"))
    }

    /// Count `window-layout-changed` and `window-resized` from here on.
    fn count_events(&self) {
        for (hook, option) in [
            ("window-layout-changed", "@layout_changed"),
            ("window-resized", "@resized"),
        ] {
            self.success(&["set", "-g", option, "0"]);
            let command = format!("set -gF {option} '#{{e|+:#{{{option}}},1}}'");
            self.success(&["set-hook", "-g", hook, &command]);
        }
    }

    fn events(&self) -> (u32, u32) {
        let events = self.display("#{@layout_changed} #{@resized}");
        let (layout, resized) = events.split_once(' ').expect("event counts");
        (layout.parse().unwrap(), resized.parse().unwrap())
    }

    fn wait_for_size(&self, expected: (u32, u32)) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while self.window_size() != expected {
            assert!(
                Instant::now() < deadline,
                "window stayed {:?}, expected {expected:?}",
                self.window_size()
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    /// Attach a control client and size it; it stays attached until dropped.
    fn attach_control(&self, sx: u32, sy: u32) -> ControlClient {
        let mut child = self
            .command()
            .args(["-C", "attach"])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn control client");
        writeln!(
            child.stdin.as_mut().expect("control stdin"),
            "refresh-client -C {sx}x{sy}"
        )
        .expect("size control client");
        ControlClient(child)
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.run(&["kill-server"]);
        let _ = fs::remove_dir_all(&self.directory);
    }
}

struct ControlClient(Child);

impl Drop for ControlClient {
    fn drop(&mut self) {
        drop(self.0.stdin.take());
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn field(text: &str, key: &str) -> u32 {
    let start = text.find(key).expect("layout field") + key.len();
    let digits: String = text[start..]
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    digits.parse().expect("numeric layout field")
}

#[test]
fn narrow_window_keeps_policy_size_and_recalculation_is_silent() {
    let server = Server::new();
    server.start(20, 10, 4, "-h");
    server.success(&["resize-window", "-x", "5"]);
    assert_eq!(server.window_size(), (5, 10));
    // Four one-column panes and three borders cannot fit in five columns.
    assert_eq!(server.root_size(), (7, 10));

    server.count_events();
    // Each option change recalculates sizes; the refit cannot shrink further.
    for _ in 0..3 {
        server.success(&["set", "-w", "window-size", "manual"]);
        server.success(&["set", "-w", "pane-border-status", "off"]);
    }
    assert_eq!(server.window_size(), (5, 10));
    assert_eq!(server.root_size(), (7, 10));
    assert_eq!(server.events(), (0, 0), "recalculation was not silent");
}

#[test]
fn closing_a_pane_refits_without_resizing() {
    let server = Server::new();
    server.start(20, 10, 4, "-h");
    server.success(&["resize-window", "-x", "5"]);
    server.count_events();

    server.success(&["kill-pane"]);
    assert_eq!(server.window_size(), (5, 10));
    assert_eq!(server.root_size(), (5, 10));
    let (layout_changed, resized) = server.events();
    assert!(layout_changed > 0, "refit fired no window-layout-changed");
    assert_eq!(resized, 0);
}

#[test]
fn border_status_change_refits_without_resizing() {
    let server = Server::new();
    server.start(20, 12, 3, "-v");
    server.success(&["set", "-w", "pane-border-status", "top"]);
    server.success(&["resize-window", "-y", "5"]);
    assert_eq!(server.window_size(), (20, 5));
    assert_eq!(server.root_size(), (20, 6));
    server.count_events();

    server.success(&["set", "-w", "pane-border-status", "off"]);
    assert_eq!(server.window_size(), (20, 5));
    assert_eq!(server.root_size(), (20, 5));
    assert_eq!(server.events().1, 0);
}

#[test]
fn imported_layout_is_arranged_to_window_size() {
    let server = Server::new();
    server.start(30, 10, 2, "-h");
    let wide = server.display("#{window_layout}");
    server.success(&["resize-window", "-x", "20"]);
    server.count_events();

    server.success(&["select-layout", &wide]);
    assert_eq!(server.window_size(), (20, 10));
    assert_eq!(server.root_size(), (20, 10));
    assert_eq!(server.events().1, 0, "import resized the window");
}

#[test]
fn zoom_never_changes_window_size() {
    let server = Server::new();
    server.start(30, 12, 2, "-h");
    server.count_events();

    server.success(&["resize-pane", "-Z"]);
    assert_eq!(server.display("#{window_zoomed_flag}"), "1");
    assert_eq!(server.window_size(), (30, 12));
    assert_eq!(server.events().1, 0);

    server.success(&["resize-window", "-x", "25"]);
    assert_eq!(server.display("#{window_zoomed_flag}"), "1");
    assert_eq!(server.window_size(), (25, 12));

    server.success(&["resize-pane", "-Z"]);
    assert_eq!(server.display("#{window_zoomed_flag}"), "0");
    assert_eq!(server.window_size(), (25, 12));
    assert_eq!(server.root_size(), (25, 12));
    assert_eq!(
        server.events().1,
        1,
        "only resize-window fired window-resized"
    );
}

#[test]
fn window_size_follows_policy_across_clients() {
    let server = Server::new();
    server.success(&["new-session", "-d", "-x", "40", "-y", "10", "sleep 60"]);
    let _small = server.attach_control(30, 10);
    let _large = server.attach_control(50, 12);
    server.success(&["set", "-w", "window-size", "largest"]);
    server.wait_for_size((50, 12));

    server.success(&["set", "-w", "window-size", "smallest"]);
    server.wait_for_size((30, 10));

    // Crowding the smallest client extends the layout, not the window.
    for _ in 0..3 {
        server.success(&["split-window", "-h", "sleep 60"]);
    }
    server.success(&["select-layout", "even-horizontal"]);
    server.success(&["resize-pane", "-t", "{left}", "-x", "20"]);
    assert_eq!(server.window_size(), (30, 10));

    server.success(&["set", "-w", "window-size", "manual"]);
    server.success(&["resize-window", "-x", "35", "-y", "11"]);
    assert_eq!(server.window_size(), (35, 11));
    server.success(&["set", "-w", "window-size", "largest"]);
    server.wait_for_size((50, 12));
}
