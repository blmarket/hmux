//! The compositor's repaint decision for scrolling output.
//!
//! When a pane's output scrolls most of the screen, tmux prefers one deferred
//! repaint of the pane over drawing each moved row. Deciding that means
//! recognizing the operations that scroll a vertical region and knowing the
//! region they would move — screen state the tokens change. [`ScrollRedraw`]
//! watches the same tokens the screen applies and answers only that question,
//! so the caller never reads the tokens itself.

use crate::parser::{Param, Token, TokenKind};

#[derive(Clone, Copy)]
struct ScrollRegion {
    top: u16,
    bottom: u16,
}

/// Where in its scrolling region an operation moves rows, which decides whether
/// the cursor has to be inside it for the move to be visible.
#[derive(Clone, Copy)]
enum ScrollEdge {
    /// Reverse index: only at the region's top row.
    Top,
    /// Line feed and index: only at the region's bottom row.
    Bottom,
    /// Insert/delete line: anywhere inside the region.
    Inside,
    /// Scroll up/down: the whole region moves regardless of the cursor.
    Any,
}

/// Recognizes the operations that scroll a vertical region, so the compositor
/// can choose between repainting rows and repainting the pane.
///
/// This reads the pane's tokens rather than its bytes: the framing is already
/// settled by the time it sees them, so all that is left is the scrolling
/// region, which it tracks for itself.
pub struct ScrollRedraw {
    rows: u16,
    explicit_region: Option<ScrollRegion>,
}

impl ScrollRedraw {
    pub fn new(rows: u16) -> Self {
        Self {
            rows: rows.max(1),
            explicit_region: None,
        }
    }

    pub fn resize(&mut self, rows: u16) {
        self.rows = rows.max(1);
        self.explicit_region = None;
    }

    /// The scroll region as `#{scroll_region_upper}`/`#{lower}` report it: the
    /// DECSTBM region when one is set, else the whole screen.
    pub fn region(&self) -> (u16, u16) {
        let region = self.current_region();
        (region.top, region.bottom)
    }

    /// Classify one token, applying any change it makes to the region first:
    /// whether tmux would prefer one deferred repaint of the pane over drawing
    /// the moved rows. That takes a region of at least half the pane, with the
    /// cursor somewhere the operation actually moves rows — so `cursor_y` is
    /// the cursor *before* the token applies, because a scroll is only visible
    /// where the cursor already was.
    pub fn scan(&mut self, token: &Token, cursor_y: u16) -> bool {
        let edge = match &token.kind {
            TokenKind::Control(0x0a..=0x0c) => ScrollEdge::Bottom,
            TokenKind::Esc {
                intermediates,
                final_byte,
            } if intermediates.is_empty() => match final_byte {
                b'D' => ScrollEdge::Bottom,
                b'M' => ScrollEdge::Top,
                // RIS puts the region back to the whole screen.
                b'c' => {
                    self.explicit_region = None;
                    return false;
                }
                _ => return false,
            },
            TokenKind::Csi {
                private: None,
                params,
                intermediates,
                final_byte,
            } => match (intermediates.as_slice(), final_byte) {
                // DECSTR, which also resets the region.
                ([b'!'], b'p') => {
                    self.explicit_region = None;
                    return false;
                }
                ([], b'r') => {
                    self.set_region(params);
                    return false;
                }
                ([], b'S' | b'T') => ScrollEdge::Any,
                ([], b'L' | b'M') => ScrollEdge::Inside,
                _ => return false,
            },
            _ => return false,
        };
        let region = self.current_region();
        if region.bottom.saturating_sub(region.top) < self.rows / 2 {
            return false;
        }
        match edge {
            ScrollEdge::Top => cursor_y == region.top,
            ScrollEdge::Bottom => cursor_y == region.bottom,
            ScrollEdge::Inside => (region.top..=region.bottom).contains(&cursor_y),
            ScrollEdge::Any => true,
        }
    }

    fn current_region(&self) -> ScrollRegion {
        self.explicit_region.unwrap_or(ScrollRegion {
            top: 0,
            bottom: self.rows.saturating_sub(1),
        })
    }

    /// DECSTBM. A region that is empty or runs off the screen is refused, as
    /// tmux refuses it.
    fn set_region(&mut self, params: &[Param]) {
        let read = |index: usize, default: u16| -> u16 {
            params
                .get(index)
                .and_then(|param: &Param| param.value)
                .and_then(|value| u16::try_from(value).ok())
                .filter(|value| *value != 0)
                .unwrap_or(default)
        };
        let top = read(0, 1);
        let bottom = read(1, self.rows);
        if top < bottom && bottom <= self.rows {
            self.explicit_region = Some(ScrollRegion {
                top: top - 1,
                bottom: bottom - 1,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::tokenize;

    fn scan_all(redraw: &mut ScrollRedraw, bytes: &[u8], cursor_y: u16) -> bool {
        tokenize(bytes)
            .into_iter()
            .map(|token| redraw.scan(&token, cursor_y))
            .any(|large| large)
    }

    #[test]
    fn a_line_feed_at_the_bottom_of_a_full_screen_wants_a_repaint() {
        let mut redraw = ScrollRedraw::new(24);
        assert!(scan_all(&mut redraw, b"\n", 23));
        assert!(!scan_all(&mut redraw, b"\n", 10));
    }

    #[test]
    fn a_reverse_index_wants_one_only_at_the_region_top() {
        let mut redraw = ScrollRedraw::new(24);
        assert!(scan_all(&mut redraw, b"\x1bM", 0));
        assert!(!scan_all(&mut redraw, b"\x1bM", 5));
    }

    #[test]
    fn a_small_decstbm_region_never_wants_a_repaint() {
        let mut redraw = ScrollRedraw::new(24);
        assert!(!scan_all(&mut redraw, b"\x1b[1;5r\n", 4));
        assert_eq!(redraw.region(), (0, 4));
    }

    #[test]
    fn scroll_up_moves_the_whole_region_wherever_the_cursor_is() {
        let mut redraw = ScrollRedraw::new(24);
        assert!(scan_all(&mut redraw, b"\x1b[2S", 3));
    }

    #[test]
    fn ris_puts_the_region_back_to_the_whole_screen() {
        let mut redraw = ScrollRedraw::new(24);
        assert!(!scan_all(&mut redraw, b"\x1b[1;5r\n", 23));
        assert!(scan_all(&mut redraw, b"\x1bc\n", 23));
        assert_eq!(redraw.region(), (0, 23));
    }

    #[test]
    fn an_invalid_region_is_refused_as_tmux_refuses_it() {
        let mut redraw = ScrollRedraw::new(24);
        scan_all(&mut redraw, b"\x1b[7;3r", 0);
        assert_eq!(redraw.region(), (0, 23));
    }

    #[test]
    fn resize_drops_the_explicit_region() {
        let mut redraw = ScrollRedraw::new(24);
        scan_all(&mut redraw, b"\x1b[1;5r", 0);
        redraw.resize(30);
        assert_eq!(redraw.region(), (0, 29));
    }
}
