//! OSC 8 hyperlinks: tmux's `hyperlinks.c`.
//!
//! A cell cannot hold a URI, so it holds a small number instead and the screen
//! keeps the table those numbers index. tmux calls that number the *inner* id
//! and starts it at one, which is what lets zero mean "this cell is not part of
//! a link".
//!
//! The other id in play is the one the sequence itself may carry, `id=…`.
//! tmux calls it the *internal* id and uses it for identity: two OSC 8s naming
//! the same URI under the same internal id are the same link and share an inner
//! id, so a link split across rows stays one link. A sequence with no `id=` is
//! anonymous, and every anonymous OSC 8 is a distinct link even when the URI
//! repeats — which is the behaviour a terminal wants, because two separate
//! mentions of the same address are not one hyperlink.

use std::collections::HashMap;

use crate::vt::vis;

/// tmux's `MAX_HYPERLINKS`. tmux enforces this across the whole server; here it
/// bounds one screen's table, which is the same protection against a pane that
/// emits an unbounded number of anonymous links.
const MAX_HYPERLINKS: usize = 5000;

/// One entry in the table.
#[derive(Clone, Debug)]
struct Entry {
    uri: String,
    /// The `id=` the sequence carried, empty when it was anonymous.
    internal_id: String,
}

/// A screen's hyperlink table: tmux's `struct hyperlinks`.
#[derive(Debug)]
pub(crate) struct Hyperlinks {
    next_inner: u32,
    entries: HashMap<u32, Entry>,
    /// `(internal_id, uri)` to inner id, for the named links only. Anonymous
    /// links are deliberately absent so they never match each other.
    by_id: HashMap<(String, String), u32>,
    /// Inner ids in the order they were added, so the oldest can be dropped
    /// when the table is full.
    order: Vec<u32>,
}

impl Default for Hyperlinks {
    fn default() -> Hyperlinks {
        Hyperlinks {
            // tmux's `hyperlinks_init` starts here so that zero can mean "no
            // link" in a cell.
            next_inner: 1,
            entries: HashMap::new(),
            by_id: HashMap::new(),
            order: Vec::new(),
        }
    }
}

impl Hyperlinks {
    /// tmux's `hyperlinks_put`: the inner id for this URI, reusing the existing
    /// one when the sequence named an `id=` that has been seen with this URI.
    ///
    /// Both the URI and the id are stored escaped, and are compared in that
    /// form. The escaping is why the URI arrives as bytes rather than as text:
    /// nothing has vouched for it being valid UTF-8, and a byte that is not
    /// stops the link no more than a control byte does — it comes back as an
    /// octal escape.
    pub(crate) fn put(&mut self, uri: &[u8], internal_id: Option<&[u8]>) -> u32 {
        let uri = vis::escape(uri, vis::KEEP_WHITESPACE);
        let internal_id = vis::escape(internal_id.unwrap_or_default(), vis::KEEP_WHITESPACE);
        let named = !internal_id.is_empty();
        let key = (internal_id.clone(), uri.clone());
        if named {
            if let Some(&inner) = self.by_id.get(&key) {
                return inner;
            }
        }

        let inner = self.next_inner;
        self.next_inner += 1;
        self.entries.insert(inner, Entry { uri, internal_id });
        if named {
            self.by_id.insert(key, inner);
        }
        self.order.push(inner);
        if self.order.len() > MAX_HYPERLINKS {
            let oldest = self.order.remove(0);
            if let Some(entry) = self.entries.remove(&oldest) {
                self.by_id.remove(&(entry.internal_id, entry.uri));
            }
        }
        inner
    }

    /// tmux's `hyperlinks_get`: the URI and the `id=` for an inner id.
    ///
    /// The internal id comes back only when the sequence actually carried one;
    /// an anonymous link reports its URI and no id.
    pub(crate) fn get(&self, inner: u32) -> Option<(&str, Option<&str>)> {
        let entry = self.entries.get(&inner)?;
        let internal_id = if entry.internal_id.is_empty() {
            None
        } else {
            Some(entry.internal_id.as_str())
        };
        Some((entry.uri.as_str(), internal_id))
    }

    // `clear-history -H` is what calls this, once that flag is plumbed through.
    #[allow(dead_code)]
    /// tmux's `hyperlinks_reset`, which a screen clear performs.
    pub(crate) fn reset(&mut self) {
        self.entries.clear();
        self.by_id.clear();
        self.order.clear();
    }
}

/// Parse an OSC 8 body — the part after `8;` — into its `id=` and its URI.
///
/// This is tmux's `input_osc_8`. The parameters are separated by `:` and end at
/// the first `;`, after which everything is the URI. An empty URI closes the
/// open link rather than opening a new one, which is why the URI comes back as
/// an `Option`. A malformed body is rejected outright, leaving whatever link is
/// open alone.
pub(crate) fn parse(body: &[u8]) -> Option<(Option<&[u8]>, Option<&[u8]>)> {
    let mut id: Option<&[u8]> = None;
    let mut rest = body;
    let uri = loop {
        let Some(end) = rest.iter().position(|byte| matches!(byte, b':' | b';')) else {
            // No `;` at all: tmux treats this as a bad sequence.
            return None;
        };
        let (parameter, tail) = rest.split_at(end);
        if let Some(value) = parameter.strip_prefix(b"id=") {
            if !value.is_empty() {
                if id.is_some() {
                    // A second `id=` is malformed.
                    return None;
                }
                id = Some(value);
            }
        }
        // The first `;` ends the parameters and begins the URI.
        if tail[0] == b';' {
            break &tail[1..];
        }
        rest = &tail[1..];
    };

    if uri.is_empty() {
        return Some((None, None));
    }
    Some((id, Some(uri)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_id_ties_two_sequences_into_one_link() {
        let mut links = Hyperlinks::default();
        let first = links.put(b"https://example.test", Some(b"x1"));
        let second = links.put(b"https://example.test", Some(b"x1"));
        assert_eq!(first, second, "same id and URI is the same link");
        assert_eq!(links.get(first), Some(("https://example.test", Some("x1"))));
    }

    #[test]
    fn anonymous_links_never_match_each_other() {
        let mut links = Hyperlinks::default();
        let first = links.put(b"https://example.test", None);
        let second = links.put(b"https://example.test", None);
        assert_ne!(
            first, second,
            "two mentions of one address are two links, not one"
        );
        assert_eq!(links.get(first), Some(("https://example.test", None)));
    }

    #[test]
    fn the_same_id_against_a_different_uri_is_a_different_link() {
        let mut links = Hyperlinks::default();
        let first = links.put(b"https://one.test", Some(b"x1"));
        let second = links.put(b"https://two.test", Some(b"x1"));
        assert_ne!(first, second);
    }

    #[test]
    fn inner_ids_start_at_one_so_zero_can_mean_no_link() {
        let mut links = Hyperlinks::default();
        assert_eq!(links.put(b"https://example.test", None), 1);
        assert_eq!(links.get(0), None);
    }

    #[test]
    fn a_reset_empties_the_table() {
        let mut links = Hyperlinks::default();
        let inner = links.put(b"https://example.test", Some(b"x1"));
        links.reset();
        assert_eq!(links.get(inner), None);
    }

    /// A URI is stored the way `utf8_stravis` leaves it, and identity is
    /// decided on that form rather than on the bytes that arrived.
    #[test]
    fn a_uri_is_stored_escaped() {
        let mut links = Hyperlinks::default();
        let inner = links.put(b"http\x7f\xffx", Some(b"x1"));
        assert_eq!(links.get(inner), Some((r"http\177\377x", Some("x1"))));
    }

    #[test]
    fn parsing_splits_the_parameters_from_the_uri() {
        assert_eq!(
            parse(b"id=x1;https://example.test"),
            Some((Some(&b"x1"[..]), Some(&b"https://example.test"[..])))
        );
        assert_eq!(
            parse(b";https://example.test"),
            Some((None, Some(&b"https://example.test"[..])))
        );
        // Parameters are colon separated and only the first `;` matters.
        assert_eq!(
            parse(b"a=b:id=x1;https://example.test/a;b"),
            Some((Some(&b"x1"[..]), Some(&b"https://example.test/a;b"[..])))
        );
    }

    #[test]
    fn an_empty_uri_closes_the_open_link() {
        assert_eq!(parse(b";"), Some((None, None)));
        assert_eq!(parse(b"id=x1;"), Some((None, None)));
    }

    #[test]
    fn a_body_with_no_semicolon_is_rejected() {
        assert_eq!(parse(b"id=x1"), None);
        assert_eq!(parse(b""), None);
    }
}
