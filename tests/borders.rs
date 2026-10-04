//! The strip draws only vertical separators, as tall as the panes. Whatever no
//! pane or separator covers shows `fill-character`: the inside fill within the
//! window's extent, such as the blank extent beside the last pane, and the
//! outside fill beyond it, such as the rows below a shorter window.

#![cfg(unix)]

mod common;

use common::{Screen, Server};

const BORDER: char = '│';
/// The default `fill-character` outside the extent, an ACS bullet.
const FILL: char = '·';
/// The default `fill-character` inside the extent, a space on a dark grey
/// background.
const INSIDE: char = ' ';
const RED: Option<u8> = Some(1);
const GREEN: Option<u8> = Some(2);

/// A server whose manually sized `sx` by `sy` window holds `panes` half
/// panes, with the status line off so the window starts at the top row.
fn strip(sx: u32, sy: u32, panes: usize) -> Server {
    let server = Server::new();
    server.start(sx, sy, panes);
    server.success(&["set", "-g", "status", "off"]);
    server
}

fn ids(server: &Server) -> Vec<String> {
    server.panes().into_iter().map(|(id, ..)| id).collect()
}

/// Whether every cell of `rows` in `columns` shows `ch`.
fn shows(
    screen: &Screen,
    rows: std::ops::Range<usize>,
    columns: std::ops::Range<usize>,
    ch: char,
) -> bool {
    rows.into_iter()
        .all(|y| columns.clone().all(|x| screen.cells[y][x].ch == ch))
}

/// Whether no cell of the screen shows `ch`.
fn nowhere(screen: &Screen, ch: char) -> bool {
    (0..screen.cells.len()).all(|y| !screen.row(y).contains(ch))
}

#[test]
fn a_lone_half_pane_has_a_right_border_beside_the_inside_fill() {
    let server = strip(80, 24, 1);
    let mut client = server.attach_terminal(80, 24);
    client.wait_screen(|screen| {
        shows(screen, 0..24, 39..40, BORDER)
            && shows(screen, 0..24, 40..80, INSIDE)
            && nowhere(screen, FILL)
    });
}

#[test]
fn fill_character_tells_inside_from_outside() {
    let server = strip(80, 10, 1);
    server.success(&["set", "-gw", "fill-character", "#{?is_inside,I,O}"]);
    let mut client = server.attach_terminal(80, 24);
    client.wait_screen(|screen| {
        shows(screen, 0..10, 40..80, 'I') && shows(screen, 10..24, 0..80, 'O')
    });
}

#[test]
fn two_halves_end_in_the_last_border_on_an_even_width() {
    let server = strip(80, 24, 2);
    let mut client = server.attach_terminal(80, 24);
    // The spare column holds the last pane's right border.
    client.wait_screen(|screen| {
        shows(screen, 0..24, 39..40, BORDER)
            && shows(screen, 0..24, 79..80, BORDER)
            && nowhere(screen, FILL)
    });
    // On an odd width the two halves fill the view and the last border lies
    // past it.
    let server = strip(81, 24, 2);
    let mut client = server.attach_terminal(81, 24);
    client.wait_screen(|screen| {
        shows(screen, 0..24, 40..41, BORDER)
            && screen.column(80).trim().is_empty()
            && nowhere(screen, FILL)
    });
}

#[test]
fn borders_are_never_horizontal() {
    // A window shorter than its client: the separators stop with the panes
    // and the rows below show the outside fill.
    let server = strip(80, 10, 3);
    let mut client = server.attach_terminal(80, 24);
    client.wait_offset(&server, 40);
    client.wait_screen(|screen| {
        shows(screen, 0..10, 39..40, BORDER)
            && shows(screen, 0..10, 79..80, BORDER)
            && shows(screen, 10..24, 0..80, FILL)
    });
    // Panned to the end, the last pane's border sits beside the blank extent,
    // which shows the inside fill.
    server.success(&["refresh-client", "-t", &client.tty, "-R", "999"]);
    assert_eq!(client.offset(&server), 80);
    client.wait_screen(|screen| {
        shows(screen, 0..10, 39..40, BORDER)
            && shows(screen, 0..10, 40..80, INSIDE)
            && shows(screen, 10..24, 0..80, FILL)
    });
    assert!(nowhere(&client.screen, '─'));
}

#[test]
fn separators_beside_the_active_pane_take_its_style() {
    let server = strip(81, 24, 3);
    server.success(&["set", "-gw", "pane-border-style", "fg=red"]);
    server.success(&["set", "-gw", "pane-active-border-style", "fg=green"]);
    let ids = ids(&server);
    // Wide enough for the whole strip: separators at 40 and 81, and the last
    // pane's right border at 122.
    let mut client = server.attach_terminal(123, 24);
    let colours = |screen: &Screen, x: usize| {
        (0..24)
            .map(|y| (screen.cells[y][x].ch == BORDER).then_some(screen.cells[y][x].fg))
            .collect::<Vec<_>>()
    };
    let solid = |fg| vec![Some(fg); 24];
    for (active, expected) in [
        (0, [GREEN, RED, RED]),
        (1, [GREEN, GREEN, RED]),
        (2, [RED, GREEN, GREEN]),
    ] {
        server.success(&["select-pane", "-t", &ids[active]]);
        client.wait_screen(|screen| {
            [40, 81, 122]
                .iter()
                .zip(expected)
                .all(|(&x, fg)| colours(screen, x) == solid(fg))
        });
    }
    // With two panes their one separator is split: the top half takes the
    // left pane's style and the bottom half the right pane's.
    server.success(&["kill-pane", "-t", &ids[2]]);
    server.success(&["select-pane", "-t", &ids[0]]);
    let split = |top, bottom| {
        (0..24)
            .map(|y| Some(if y <= 12 { top } else { bottom }))
            .collect::<Vec<_>>()
    };
    client.wait_screen(|screen| colours(screen, 40) == split(GREEN, RED));
    server.success(&["select-pane", "-t", &ids[1]]);
    client.wait_screen(|screen| colours(screen, 40) == split(RED, GREEN));
}
