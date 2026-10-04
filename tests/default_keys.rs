//! The default prefix is C-a. After it, c inserts a pane in the active pane's
//! directory, h and l select, f toggles the width, H and L reorder, x kills
//! and a second C-a reaches the application.

#![cfg(unix)]

mod common;

use common::{Server, TerminalClient};
use std::fs;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

fn wait_until(what: &str, condition: impl Fn() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !condition() {
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn ids(server: &Server) -> Vec<String> {
    server.panes().into_iter().map(|(id, ..)| id).collect()
}

/// An 80 by 24 window of one shell, shown on a terminal client of that size.
fn attached() -> (Server, TerminalClient) {
    let server = Server::new();
    server.success(&["new-session", "-d", "-x", "80", "-y", "24"]);
    server.success(&["set", "-g", "window-size", "manual"]);
    server.success(&["set", "-g", "status", "off"]);
    let client = server.attach_terminal(80, 24);
    (server, client)
}

#[test]
fn prefix_keys_drive_the_strip() {
    let (server, mut client) = attached();
    assert_eq!(server.success(&["show-options", "-gv", "prefix"]), "C-a\n");
    let prefix = server.success(&["list-keys", "-T", "prefix"]);
    for command in ["split-window", "break-pane"] {
        assert!(!prefix.contains(command), "{command} is still bound");
    }

    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let directory = std::env::temp_dir().join(format!("hmux keys {stamp}"));
    fs::create_dir(&directory).expect("create directory");
    let directory = directory.canonicalize().expect("canonical directory");
    let path = directory.to_str().expect("UTF-8 directory").to_owned();
    client.send(format!("cd '{path}'\n").as_bytes());
    wait_until("the cd", || server.display("#{pane_current_path}") == path);

    client.send(b"\x01c");
    wait_until("the insertion", || ids(&server).len() == 2);
    let panes = ids(&server);
    assert_eq!(server.display("#{pane_id}"), panes[1]);
    wait_until("the new pane's directory", || {
        server.display("#{pane_current_path}") == path
    });

    client.send(b"\x01h");
    wait_until("h", || server.display("#{pane_id}") == panes[0]);
    client.send(b"\x01l");
    wait_until("l", || server.display("#{pane_id}") == panes[1]);

    client.send(b"\x01f");
    wait_until("f", || server.display("#{pane_width}") == "80");
    client.send(b"\x01f");
    wait_until("f again", || server.display("#{pane_width}") == "39");

    // Uppercase moves the active pane where lowercase moved the focus.
    client.send(b"\x01H");
    wait_until("H", || ids(&server) == [panes[1].clone(), panes[0].clone()]);
    assert_eq!(server.display("#{pane_id}"), panes[1]);
    client.send(b"\x01L");
    wait_until("L", || ids(&server) == panes);
    assert_eq!(server.display("#{pane_id}"), panes[1]);

    client.send(b"\x01x");
    wait_until("x", || ids(&server) == [panes[0].clone()]);

    client.send(b"cat -v\n");
    wait_until("cat", || server.display("#{pane_current_command}") == "cat");
    client.send(b"\x01\x01\n");
    wait_until("the literal C-a", || {
        server.success(&["capture-pane", "-p"]).contains("^A")
    });

    let _ = fs::remove_dir(&directory);
}

#[test]
fn configuration_overrides_the_defaults() {
    let (server, mut client) = attached();
    server.success(&["set", "-g", "prefix", "C-b"]);
    server.success(&["bind", "c", "new-window"]);

    // C-a is now an ordinary key for the pane; C-b c runs the override.
    client.send(b"\x01c\x02c");
    wait_until("the new window", || {
        server.display("#{session_windows}") == "2"
    });
    assert_eq!(server.display("#{window_panes}"), "1");
    assert_eq!(
        server.success(&["display-message", "-p", "-t", ":0", "#{window_panes}"]),
        "1\n"
    );
}
