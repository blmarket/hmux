//! With `sticky-layout` on, a selected preset stays in force: panes are
//! arranged again when they are added or removed, when the window is resized
//! and when an option the preset reads changes.

#![cfg(unix)]

mod common;

use common::Server;
use std::time::{Duration, Instant};

const PRESETS: [&str; 7] = [
    "even-horizontal",
    "even-vertical",
    "main-horizontal",
    "main-horizontal-mirrored",
    "main-vertical",
    "main-vertical-mirrored",
    "tiled",
];

fn pane_widths(server: &Server, target: &str) -> Vec<u32> {
    server
        .success(&["list-panes", "-t", target, "-F", "#{pane_width}"])
        .lines()
        .map(|width| width.parse().expect("numeric pane width"))
        .collect()
}

fn widths(server: &Server) -> Vec<u32> {
    pane_widths(server, "")
}

fn layout(server: &Server, target: &str) -> String {
    server
        .success(&["display-message", "-p", "-t", target, "#{window_layout}"])
        .trim_end()
        .to_owned()
}

/// Reselecting `preset` leaves the layout of the window at `target` unchanged.
fn assert_matches(server: &Server, target: &str, preset: &str) {
    let before = layout(server, target);
    server.success(&["select-layout", "-t", target, preset]);
    assert_eq!(layout(server, target), before, "{preset} was not in force");
}

/// Three side by side panes in an 83-column window with `sticky-layout` on.
fn sticky_server(panes: usize, preset: &str) -> Server {
    let server = Server::new();
    server.start(83, 24, panes, "-h");
    server.success(&["set", "-g", "sticky-layout", "on"]);
    server.success(&["select-layout", preset]);
    server
}

fn split(server: &Server) {
    server.success(&["split-window", "sleep 60"]);
}

#[test]
fn option_off_keeps_free_layouts() {
    let server = Server::new();
    server.start(83, 24, 3, "-h");
    server.success(&["select-layout", "even-horizontal"]);
    server.success(&["kill-pane"]);
    let widths = widths(&server);
    assert_eq!(widths.len(), 2);
    assert_ne!(widths, [41, 41], "a neighbor should absorb the space");
}

#[test]
fn panes_stay_arranged_as_they_come_and_go() {
    let server = sticky_server(3, "even-horizontal");
    assert_eq!(widths(&server), [27, 27, 27]);
    server.success(&["kill-pane"]);
    assert_eq!(widths(&server), [41, 41]);
    split(&server);
    split(&server);
    assert_eq!(widths(&server), [20, 20, 20, 20]);
}

#[test]
fn split_is_arranged_before_the_next_command() {
    let server = sticky_server(3, "even-horizontal");
    let output = server.success(&[
        "split-window",
        "sleep 60",
        ";",
        "display-message",
        "-p",
        "#{pane_width}",
    ]);
    assert_eq!(output.trim_end(), "20");
}

#[test]
fn resize_arranges_for_the_new_size() {
    let server = sticky_server(3, "even-horizontal");
    server.count_events();
    server.success(&["resize-window", "-x", "62"]);
    assert_eq!(widths(&server), [20, 20, 20]);
    assert_eq!(server.events().1, 1, "one window-resized");
}

#[test]
fn repeated_recalculation_is_silent_for_every_preset() {
    for preset in PRESETS {
        let server = sticky_server(4, preset);
        server.count_events();
        for _ in 0..3 {
            server.success(&["set", "-w", "window-size", "manual"]);
            server.success(&["set", "-w", "pane-border-status", "off"]);
        }
        assert_eq!(
            server.events(),
            (0, 0),
            "{preset} recalculation was not silent"
        );
    }
}

#[test]
fn adding_and_removing_panes_keeps_the_preset() {
    for preset in ["tiled", "main-vertical"] {
        let server = sticky_server(2, preset);
        for _ in 0..4 {
            split(&server);
            assert_matches(&server, "", preset);
        }
        server.success(&["kill-pane", "-t", "{top-left}"]);
        assert_matches(&server, "", preset);
        server.success(&["kill-pane"]);
        assert_matches(&server, "", preset);
    }
}

#[test]
fn pane_exiting_on_its_own_keeps_the_preset() {
    let server = sticky_server(4, "tiled");
    server.success(&["split-window", "sleep 0.3"]);
    let deadline = Instant::now() + Duration::from_secs(5);
    while widths(&server).len() != 4 {
        assert!(Instant::now() < deadline, "pane did not exit");
        std::thread::sleep(Duration::from_millis(20));
    }
    assert_matches(&server, "", "tiled");
}

#[test]
fn option_change_is_applied_without_reselecting() {
    let server = sticky_server(3, "main-vertical");
    server.success(&["set", "-w", "main-pane-width", "30"]);
    assert_eq!(widths(&server)[0], 30);
    assert_matches(&server, "", "main-vertical");
}

#[test]
fn swap_and_rotate_keep_the_preset_without_extra_events() {
    let free = Server::new();
    free.start(83, 24, 4, "-h");
    free.success(&["select-layout", "main-vertical"]);
    free.count_events();
    free.success(&["swap-pane", "-s", "{top-left}", "-t", "{bottom-right}"]);
    free.success(&["rotate-window"]);
    let free_events = free.events();

    let server = sticky_server(4, "main-vertical");
    server.count_events();
    server.success(&["swap-pane", "-s", "{top-left}", "-t", "{bottom-right}"]);
    assert_matches(&server, "", "main-vertical");
    server.count_events();
    server.success(&["swap-pane", "-s", "{top-left}", "-t", "{bottom-right}"]);
    server.success(&["rotate-window"]);
    assert_eq!(server.events(), free_events);
    assert_matches(&server, "", "main-vertical");
}

#[test]
fn join_pane_keeps_both_presets() {
    let server = sticky_server(3, "even-horizontal");
    server.success(&["new-window", "sleep 60"]);
    split(&server);
    split(&server);
    server.success(&["select-layout", "even-vertical"]);
    server.success(&["join-pane", "-h", "-s", ":1.0", "-t", ":0.0"]);
    assert_eq!(pane_widths(&server, ":0").len(), 4);
    assert_eq!(pane_widths(&server, ":1").len(), 2);
    assert_matches(&server, ":1", "even-vertical");
    assert_matches(&server, ":0", "even-horizontal");
}

#[test]
fn break_pane_keeps_the_source_and_frees_the_new_window() {
    let server = sticky_server(3, "even-horizontal");
    server.success(&["break-pane", "-d", "-s", ":0.1", "-t", ":1"]);
    assert_eq!(pane_widths(&server, ":0"), [41, 41]);
    server.success(&["split-window", "-h", "-t", ":1", "sleep 60"]);
    server.success(&["split-window", "-h", "-t", ":1", "sleep 60"]);
    server.success(&["kill-pane", "-t", ":1.0"]);
    assert_ne!(
        pane_widths(&server, ":1"),
        [41, 41],
        "new window was sticky"
    );
}

#[test]
fn manual_geometry_is_refused() {
    let server = sticky_server(3, "even-horizontal");
    let before = layout(&server, "");
    for args in [
        &["resize-pane", "-x", "10"][..],
        &["resize-pane", "-L"],
        &["select-layout", "-E"],
    ] {
        let output = server.run(args);
        assert!(!output.status.success(), "{args:?} succeeded");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("layout is sticky"),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(layout(&server, ""), before);
    }
}

#[test]
fn layout_string_frees_the_layout() {
    let server = sticky_server(3, "even-horizontal");
    let even = layout(&server, "");
    server.success(&["select-layout", &even]);
    server.success(&["kill-pane"]);
    assert_ne!(widths(&server), [41, 41], "layout string left it sticky");
}

#[test]
fn failed_layout_string_stays_sticky() {
    let server = sticky_server(3, "even-horizontal");
    assert!(!server
        .run(&["select-layout", "bogus,layout"])
        .status
        .success());
    server.success(&["kill-pane"]);
    assert_eq!(widths(&server), [41, 41]);
}

#[test]
fn option_is_read_only_at_selection() {
    let server = sticky_server(3, "even-horizontal");
    server.success(&["set", "-g", "sticky-layout", "off"]);
    server.success(&["kill-pane"]);
    assert_eq!(widths(&server), [41, 41], "turning the option off freed it");

    let server = Server::new();
    server.start(83, 24, 3, "-h");
    server.success(&["select-layout", "even-horizontal"]);
    server.success(&["set", "-g", "sticky-layout", "on"]);
    server.success(&["kill-pane"]);
    assert_ne!(
        widths(&server),
        [41, 41],
        "turning the option on captured it"
    );
}

#[test]
fn cycling_makes_each_preset_sticky() {
    // Tall enough that every preset leaves room to split the active pane.
    let server = Server::new();
    server.start(83, 80, 3, "-h");
    server.success(&["set", "-g", "sticky-layout", "on"]);
    for preset in PRESETS {
        server.success(&["next-layout"]);
        split(&server);
        assert_matches(&server, "", preset);
        server.success(&["kill-pane"]);
        assert_matches(&server, "", preset);
    }
}

#[test]
fn lone_pane_fills_the_window() {
    let server = sticky_server(2, "even-horizontal");
    server.success(&["kill-pane"]);
    server.success(&["resize-window", "-x", "62"]);
    assert_eq!(widths(&server), [62]);
}
