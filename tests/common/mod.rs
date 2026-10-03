//! Server harness shared by integration tests that drive a real hmux server.

#![allow(dead_code)] // Each test binary uses a different subset.

use std::fs;
use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub struct Server {
    directory: PathBuf,
    socket: PathBuf,
}

impl Server {
    pub fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        let directory =
            std::env::temp_dir().join(format!("hmux-test-{}-{stamp}", std::process::id()));
        fs::create_dir(&directory).expect("create socket directory");
        let socket = directory.join("socket");
        Self { directory, socket }
    }

    pub fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_hmux"));
        command
            .args(["-f", "/dev/null", "-S"])
            .arg(&self.socket)
            .env("TERM", "xterm-256color")
            .env("SHELL", "/bin/sh")
            .env("TMUX", "");
        command
    }

    pub fn run(&self, args: &[&str]) -> Output {
        self.command().args(args).output().expect("run hmux")
    }

    pub fn success(&self, args: &[&str]) -> String {
        let output = self.run(args);
        assert!(
            output.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).expect("UTF-8 command output")
    }

    pub fn display(&self, format: &str) -> String {
        self.success(&["display-message", "-p", format])
            .trim_end()
            .to_owned()
    }

    /// Start a manually sized window with `panes` in its strip.
    pub fn start(&self, sx: u32, sy: u32, panes: usize) {
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
            self.success(&["new-pane", "sleep 60"]);
        }
    }

    pub fn window_size(&self) -> (u32, u32) {
        let size = self.display("#{window_width}x#{window_height}");
        let (sx, sy) = size.split_once('x').expect("size format");
        (sx.parse().unwrap(), sy.parse().unwrap())
    }

    /// The layout root's size: the strip, or a lone pane's cell.
    pub fn root_size(&self) -> (u32, u32) {
        let layout = self.display("#{window_layout}");
        let root = &layout[layout.find("\"L\":{").expect("layout root")..];
        (field(root, "\"w\":"), field(root, "\"h\":"))
    }

    /// Each pane's id, first column, width, top row and height, in strip order.
    pub fn panes(&self) -> Vec<(String, u32, u32, u32, u32)> {
        self.success(&[
            "list-panes",
            "-F",
            "#{pane_id} #{pane_left} #{pane_width} #{pane_top} #{pane_height}",
        ])
        .lines()
        .map(|line| {
            let fields: Vec<_> = line.split(' ').collect();
            let number = |index: usize| fields[index].parse().expect("numeric pane field");
            (
                fields[0].to_owned(),
                number(1),
                number(2),
                number(3),
                number(4),
            )
        })
        .collect()
    }

    /// Count `window-layout-changed` and `window-resized` from here on.
    pub fn count_events(&self) {
        for (hook, option) in [
            ("window-layout-changed", "@layout_changed"),
            ("window-resized", "@resized"),
        ] {
            self.success(&["set", "-g", option, "0"]);
            let command = format!("set -gF {option} '#{{e|+:#{{{option}}},1}}'");
            self.success(&["set-hook", "-g", hook, &command]);
        }
    }

    pub fn events(&self) -> (u32, u32) {
        let events = self.display("#{@layout_changed} #{@resized}");
        let (layout, resized) = events.split_once(' ').expect("event counts");
        (layout.parse().unwrap(), resized.parse().unwrap())
    }

    pub fn wait_for_size(&self, expected: (u32, u32)) {
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
    pub fn attach_control(&self, sx: u32, sy: u32) -> ControlClient {
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

pub struct ControlClient(Child);

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
