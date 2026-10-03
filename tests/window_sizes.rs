//! Window size is the `window-size` policy result; layout may extend past it
//! without feeding back into it.

#![cfg(unix)]

mod common;

use common::Server;

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
fn zoom_flags_are_accepted_and_change_nothing() {
    let server = Server::new();
    server.start(30, 12, 2, "-h");
    let layout = server.display("#{window_layout}");
    server.count_events();

    server.success(&["resize-pane", "-Z"]);
    server.success(&["select-pane", "-Z", "-L"]);
    server.success(&["select-pane", "-Z", "-R"]);
    assert_eq!(server.display("#{window_zoomed_flag}"), "0");
    assert_eq!(server.display("#{pane_zoomed_flag}"), "0");
    assert_eq!(server.display("#{window_flags}"), "*");
    assert_eq!(server.display("#{window_layout}"), layout);
    assert_eq!(server.window_size(), (30, 12));
    assert_eq!(server.events(), (0, 0));
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
