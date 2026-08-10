//! The width policy: how many cells a character occupies.
//!
//! Width is its own concern, kept out of [`crate::PaneScreen`]. tmux exposes it
//! as options (`codepoint-widths`, `variation-selector-always-wide`), so the
//! server sets the policy here; the emulator reads it where it combines a
//! character, which is why the getter is crate-visible and the setters are
//! not.
//!
//! The oracle is the pinned tmux 3.7b, which resolves a width in two steps: it
//! consults its width cache first, and falls back to `utf8proc_wcwidth`. We
//! reproduce both halves. The private `defaults` module is tmux's built-in
//! cache, transcribed; `table` is generated from the same utf8proc release the
//! pinned tmux links against, so the fallback agrees codepoint for codepoint
//! without a C dependency in the build.

mod defaults;
mod table;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::RwLock;

/// tmux's `variation-selector-always-wide`, whose default is on.
///
/// This is a server-scope option, and tmux reads it out of `global_options` at
/// the moment it combines a character, so a process-wide flag is the faithful
/// shape rather than a shortcut.
static VARIATION_SELECTOR_ALWAYS_WIDE: AtomicBool = AtomicBool::new(true);

/// Whether U+FE0F should force the cell it joins to two columns.
#[must_use]
pub(crate) fn variation_selector_always_wide() -> bool {
    VARIATION_SELECTOR_ALWAYS_WIDE.load(Ordering::Relaxed)
}

/// Apply `variation-selector-always-wide`.
pub fn set_variation_selector_always_wide(wide: bool) {
    VARIATION_SELECTOR_ALWAYS_WIDE.store(wide, Ordering::Relaxed);
}

/// One `codepoint-widths` entry: an inclusive codepoint range and the width it
/// forces.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Override {
    start: u32,
    end: u32,
    width: u8,
}

/// `codepoint-widths`, as the option array currently reads.
///
/// tmux keeps one width cache for the whole server and rebuilds it from the
/// whole array whenever the option is set, so this is process-wide and replaced
/// wholesale rather than amended. It is a list of ranges where tmux's cache has
/// an entry per codepoint; the difference is only in what a pathological range
/// costs to store.
static OVERRIDES: RwLock<Vec<Override>> = RwLock::new(Vec::new());

/// Rebuild the overrides from the `codepoint-widths` array. Entries that do not
/// parse are dropped, as tmux drops them.
pub fn set_codepoint_widths<'a>(entries: impl IntoIterator<Item = &'a str>) {
    let parsed = entries.into_iter().filter_map(parse_override).collect();
    if let Ok(mut overrides) = OVERRIDES.write() {
        *overrides = parsed;
    }
}

/// tmux's `utf8_add_to_width_cache`: `<codepoint>=<width>`, where the codepoint
/// is `U+hex`, a `U+hex-U+hex` range, or the character itself, and the width is
/// zero to two.
fn parse_override(entry: &str) -> Option<Override> {
    let (name, width) = entry.split_once('=')?;
    let width: u8 = width.parse().ok().filter(|width| *width <= 2)?;
    if let Some(hex) = name.strip_prefix("U+") {
        let (start, end) = match hex.split_once("-U+") {
            Some((start, end)) => (start, end),
            None => (hex, hex),
        };
        let start = u32::from_str_radix(start, 16).ok().filter(|cp| *cp != 0)?;
        let end = u32::from_str_radix(end, 16).ok().filter(|cp| *cp != 0)?;
        (start <= end).then_some(Override { start, end, width })
    } else {
        // Anything else names the character itself, and only one of them.
        let mut chars = name.chars();
        let character = chars.next()?;
        chars.next().is_none().then(|| Override {
            start: character as u32,
            end: character as u32,
            width,
        })
    }
}

/// The width `codepoint-widths` forces on this codepoint, if any. A later entry
/// wins, which is what rebuilding tmux's cache in array order leaves behind.
fn override_width(codepoint: u32) -> Option<u8> {
    let overrides = OVERRIDES.read().ok()?;
    overrides
        .iter()
        .rev()
        .find(|entry| (entry.start..=entry.end).contains(&codepoint))
        .map(|entry| entry.width)
}

/// The terminal-cell width of one Unicode codepoint.
///
/// The result is always 0, 1, or 2. This function is total: values above
/// U+10FFFF have width one, matching what tmux would do with a codepoint its
/// UTF-8 decoder would never hand to the width lookup in the first place.
#[must_use]
pub fn codepoint_width(codepoint: u32) -> u8 {
    // `codepoint-widths` goes into the same cache tmux consults first, on top
    // of the built-in entries, so an override beats both of the steps below.
    if let Some(width) = override_width(codepoint) {
        return width;
    }
    if let Ok(index) = defaults::DEFAULT_OVERRIDES.binary_search_by_key(&codepoint, |&(cp, _)| cp) {
        return defaults::DEFAULT_OVERRIDES[index].1;
    }
    match table::WIDTHS.binary_search_by(|&(start, end, _)| {
        if end < codepoint {
            std::cmp::Ordering::Less
        } else if start > codepoint {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }
    }) {
        Ok(index) => table::WIDTHS[index].2,
        Err(_) => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codepoint_width_matches_the_tmux_oracle() {
        assert_eq!(codepoint_width('a' as u32), 1);
        assert_eq!(codepoint_width('界' as u32), 2);
        assert_eq!(codepoint_width(0x0301), 0); // combining acute accent
        assert_eq!(codepoint_width('ㄱ' as u32), 2);
        assert_eq!(codepoint_width(0x110000), 1); // total for invalid codepoints
    }

    #[test]
    fn a_conjoining_hangul_jamo_vowel_is_one_cell_wide() {
        assert_eq!(codepoint_width(0x1161), 1);
    }

    #[test]
    fn the_table_records_which_utf8proc_it_was_generated_from() {
        // The widths only agree with tmux because tmux links this release. If
        // the pinned tmux moves to another one, regenerating the table will
        // trip this assertion, which is the point: the new oracle deserves a
        // conscious re-baseline rather than a silent shift.
        assert_eq!(table::UTF8PROC_VERSION, "2.11.3");
    }

    #[test]
    fn the_table_covers_every_codepoint_without_gaps_or_overlaps() {
        let mut next = 0;
        for &(start, end, width) in table::WIDTHS {
            assert_eq!(start, next, "gap or overlap at {start:#X}");
            assert!(start <= end, "inverted range at {start:#X}");
            assert!(width <= 2, "width {width} out of range at {start:#X}");
            next = end + 1;
        }
        assert_eq!(next, 0x110000, "table stops short of the Unicode maximum");
    }

    #[test]
    fn tmuxs_built_in_cache_overrides_the_utf8proc_fallback() {
        // tmux pins the regional indicators to one cell each, so that a flag
        // built from a surrogate-free pair occupies two columns and not four.
        // utf8proc calls them wide, which is exactly the disagreement the
        // built-in cache exists to settle.
        assert_eq!(codepoint_width(0x1F1E6), 1);
        assert_eq!(codepoint_width(0x1F1FF), 1);
        // And it widens emoji that utf8proc leaves narrow.
        assert_eq!(codepoint_width(0x261D), 2); // white up pointing index
        assert_eq!(codepoint_width(0x270A), 2); // raised fist
    }

    #[test]
    fn the_built_in_cache_is_sorted_for_binary_search() {
        assert!(defaults::DEFAULT_OVERRIDES
            .windows(2)
            .all(|pair| pair[0].0 < pair[1].0));
    }
}
