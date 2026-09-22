//! Recognition of bracketed-paste boundary sequences.

/// The two control sequences used to delimit a bracketed paste.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BracketedPasteBoundary {
    Start,
    End,
}

/// The result of looking for a bracketed-paste boundary at the beginning of a
/// byte slice.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BracketedPasteBoundaryMatch {
    /// A complete boundary was found. Bytes after `consumed` belong to the
    /// caller and must be left for the next parse operation.
    Match {
        boundary: BracketedPasteBoundary,
        consumed: usize,
    },
    /// The supplied bytes are a prefix of a boundary. `consumed` is the
    /// number of bytes in that prefix.
    Incomplete { consumed: usize },
    /// No boundary starts at the beginning of the slice. Nothing was
    /// consumed, so the caller can apply its normal key parsing rules.
    NoMatch { consumed: usize },
}

pub const BRACKETED_PASTE_START: &[u8] = b"\x1b[200~";
pub const BRACKETED_PASTE_END: &[u8] = b"\x1b[201~";

/// Match one bracketed-paste boundary without inspecting or changing caller
/// state.
pub fn match_bracketed_paste_boundary(input: &[u8]) -> BracketedPasteBoundaryMatch {
    for (boundary, marker) in [
        (BracketedPasteBoundary::Start, BRACKETED_PASTE_START),
        (BracketedPasteBoundary::End, BRACKETED_PASTE_END),
    ] {
        let compared = input.len().min(marker.len());
        if input.get(..compared) != Some(&marker[..compared]) {
            continue;
        }
        if input.len() >= marker.len() {
            return BracketedPasteBoundaryMatch::Match {
                boundary,
                consumed: marker.len(),
            };
        }
        return BracketedPasteBoundaryMatch::Incomplete {
            consumed: input.len(),
        };
    }

    BracketedPasteBoundaryMatch::NoMatch { consumed: 0 }
}

/// Whether the input is still a possible prefix of the end marker.
pub fn is_incomplete_bracketed_paste_end(input: &[u8]) -> bool {
    input.len() < BRACKETED_PASTE_END.len()
        && input.starts_with(&BRACKETED_PASTE_END[..input.len()])
}

#[cfg(test)]
mod tests {
    use super::{
        is_incomplete_bracketed_paste_end, match_bracketed_paste_boundary, BracketedPasteBoundary,
        BracketedPasteBoundaryMatch, BRACKETED_PASTE_END, BRACKETED_PASTE_START,
    };

    #[test]
    fn every_marker_split_point_is_incomplete_until_the_final_byte() {
        for (boundary, marker) in [
            (BracketedPasteBoundary::Start, BRACKETED_PASTE_START),
            (BracketedPasteBoundary::End, BRACKETED_PASTE_END),
        ] {
            for split in 0..marker.len() {
                assert_eq!(
                    match_bracketed_paste_boundary(&marker[..split]),
                    BracketedPasteBoundaryMatch::Incomplete { consumed: split },
                    "{boundary:?} split at {split}"
                );
            }
            assert_eq!(
                match_bracketed_paste_boundary(marker),
                BracketedPasteBoundaryMatch::Match {
                    boundary,
                    consumed: marker.len(),
                }
            );
        }
    }

    #[test]
    fn misleading_prefixes_are_not_boundaries() {
        for input in [
            b"x\x1b[200~".as_slice(),
            b"\x1b]200~".as_slice(),
            b"\x1b[202~".as_slice(),
            b"\x1b[20x".as_slice(),
            b"\x1b[200x".as_slice(),
        ] {
            assert_eq!(
                match_bracketed_paste_boundary(input),
                BracketedPasteBoundaryMatch::NoMatch { consumed: 0 },
                "input {input:?}"
            );
        }
    }

    #[test]
    fn adjacent_markers_and_trailing_bytes_are_left_for_the_caller() {
        let adjacent = [BRACKETED_PASTE_START, BRACKETED_PASTE_END].concat();
        assert_eq!(
            match_bracketed_paste_boundary(&adjacent),
            BracketedPasteBoundaryMatch::Match {
                boundary: BracketedPasteBoundary::Start,
                consumed: BRACKETED_PASTE_START.len(),
            }
        );
        assert_eq!(
            match_bracketed_paste_boundary(&adjacent[BRACKETED_PASTE_START.len()..]),
            BracketedPasteBoundaryMatch::Match {
                boundary: BracketedPasteBoundary::End,
                consumed: BRACKETED_PASTE_END.len(),
            }
        );

        let trailing = [BRACKETED_PASTE_END, b"payload"].concat();
        assert_eq!(
            match_bracketed_paste_boundary(&trailing),
            BracketedPasteBoundaryMatch::Match {
                boundary: BracketedPasteBoundary::End,
                consumed: BRACKETED_PASTE_END.len(),
            }
        );
    }

    #[test]
    fn only_end_prefixes_extend_the_paste_end_delay() {
        for split in 0..BRACKETED_PASTE_END.len() {
            assert!(is_incomplete_bracketed_paste_end(
                &BRACKETED_PASTE_END[..split]
            ));
        }
        assert!(!is_incomplete_bracketed_paste_end(BRACKETED_PASTE_END));
        assert!(!is_incomplete_bracketed_paste_end(b"\x1b[200"));
        assert!(!is_incomplete_bracketed_paste_end(b"\x1b[201x"));
    }
}
