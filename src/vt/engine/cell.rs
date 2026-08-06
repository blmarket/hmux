//! One grid cell: what is in it and how it is drawn.
//!
//! This is tmux's `struct grid_cell`, kept deliberately close to it. The
//! attribute bits, the colour encoding and the flag bits are the ones
//! `grid.c`/`screen-write.c` use, because the whole point of the in-house
//! engine is that the semantics are tmux's and can be read against tmux's
//! source without translation.

/// tmux's `GRID_ATTR_*`, as one bitmask.
pub(crate) mod attr {
    pub(crate) const BRIGHT: u16 = 0x1;
    pub(crate) const DIM: u16 = 0x2;
    pub(crate) const UNDERSCORE: u16 = 0x4;
    pub(crate) const BLINK: u16 = 0x8;
    pub(crate) const REVERSE: u16 = 0x10;
    pub(crate) const HIDDEN: u16 = 0x20;
    pub(crate) const ITALICS: u16 = 0x40;
    /// The alternate (line-drawing) character set is selected for this cell.
    pub(crate) const CHARSET: u16 = 0x80;
    pub(crate) const STRIKETHROUGH: u16 = 0x100;
    pub(crate) const UNDERSCORE_2: u16 = 0x200;
    pub(crate) const UNDERSCORE_3: u16 = 0x400;
    pub(crate) const UNDERSCORE_4: u16 = 0x800;
    pub(crate) const UNDERSCORE_5: u16 = 0x1000;
    pub(crate) const OVERLINE: u16 = 0x2000;

    /// Every underline style, so selecting one can clear the others.
    pub(crate) const ALL_UNDERSCORE: u16 =
        UNDERSCORE | UNDERSCORE_2 | UNDERSCORE_3 | UNDERSCORE_4 | UNDERSCORE_5;
}

/// tmux's `GRID_FLAG_*`, as one bitmask.
pub(crate) mod flag {
    /// The blank second half of a wide character.
    pub(crate) const PADDING: u8 = 0x4;
    /// The cell has never been written; a clear left it as it is.
    pub(crate) const CLEARED: u8 = 0x40;
    /// The cell is part of a run a horizontal tab created, rather than of
    /// spaces someone typed. `capture-pane -e` has to tell them apart.
    pub(crate) const TAB: u8 = 0x80;
}

/// tmux's colour encoding: a palette index, or an RGB triple, or "default".
///
/// tmux carries this as one `int` with flag bits above the value, and every
/// SGR handler reads and writes it in that form. Keeping the same encoding is
/// what lets those handlers be ported rather than reinterpreted.
pub(crate) mod colour {
    /// The 256-colour palette flag.
    pub(crate) const FLAG_256: i32 = 0x0100_0000;
    /// The direct-colour flag; the low 24 bits are then `0xrrggbb`.
    pub(crate) const FLAG_RGB: i32 = 0x0200_0000;
    /// tmux's "no colour chosen", which resolves against the pane's options.
    pub(crate) const DEFAULT: i32 = 8;

    /// One of the 256 palette entries.
    pub(crate) fn indexed(index: u8) -> i32 {
        i32::from(index) | FLAG_256
    }

    /// A direct colour.
    pub(crate) fn rgb(red: u8, green: u8, blue: u8) -> i32 {
        FLAG_RGB | (i32::from(red) << 16) | (i32::from(green) << 8) | i32::from(blue)
    }

    /// The `0xrrggbb` of a direct colour, if it is one.
    pub(crate) fn as_rgb(colour: i32) -> Option<(u8, u8, u8)> {
        if colour & FLAG_RGB == 0 {
            return None;
        }
        Some((
            (colour >> 16) as u8,
            (colour >> 8) as u8,
            (colour & 0xff) as u8,
        ))
    }

    /// The palette index of a 256-colour value, if it is one. The eight bright
    /// aixterm colours (90–97, 100–107) are folded onto 8–15, as tmux folds
    /// them.
    pub(crate) fn as_index(colour: i32) -> Option<u8> {
        if colour & FLAG_256 != 0 {
            return u8::try_from(colour & 0xff).ok();
        }
        match colour {
            0..=7 => u8::try_from(colour).ok(),
            90..=97 => u8::try_from(colour - 90 + 8).ok(),
            _ => None,
        }
    }
}

/// The largest number of codepoints one cell holds, matching tmux's
/// `UTF8_SIZE`. A cluster longer than this is truncated where tmux truncates.
pub(crate) const UTF8_SIZE: usize = 21;

/// The character content of a cell: a grapheme cluster and the number of
/// columns it occupies.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CellData {
    /// The cluster's UTF-8 bytes, at most [`UTF8_SIZE`] of them.
    pub(crate) bytes: Vec<u8>,
    /// Display width in columns: 0, 1 or 2.
    pub(crate) width: u8,
}

impl CellData {
    /// One ASCII or single-codepoint character.
    pub(crate) fn from_char(character: char, width: u8) -> CellData {
        let mut bytes = [0u8; 4];
        CellData {
            bytes: character.encode_utf8(&mut bytes).as_bytes().to_vec(),
            width,
        }
    }

    /// The cell content of a space, which is what a clear leaves behind.
    pub(crate) fn space() -> CellData {
        CellData {
            bytes: vec![b' '],
            width: 1,
        }
    }

    /// Whether the cell holds exactly one space.
    pub(crate) fn is_space(&self) -> bool {
        self.bytes == b" "
    }

    /// The cluster as text. Cell bytes are always valid UTF-8, because they
    /// only ever arrive as an encoded `char`.
    pub(crate) fn text(&self) -> &str {
        std::str::from_utf8(&self.bytes).unwrap_or("")
    }

    /// Append a zero-width codepoint to the cluster, as a combining mark
    /// arriving after the base character does. Beyond [`UTF8_SIZE`] bytes the
    /// mark is dropped rather than allowed to grow the cell without bound.
    pub(crate) fn combine(&mut self, character: char) {
        let mut buffer = [0u8; 4];
        let encoded = character.encode_utf8(&mut buffer).as_bytes();
        if self.bytes.len() + encoded.len() > UTF8_SIZE {
            return;
        }
        self.bytes.extend_from_slice(encoded);
    }
}

/// One grid cell: tmux's `struct grid_cell`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Cell {
    pub(crate) data: CellData,
    pub(crate) attr: u16,
    pub(crate) flags: u8,
    pub(crate) fg: i32,
    pub(crate) bg: i32,
    /// The underline colour, which SGR 58 sets separately from the foreground.
    pub(crate) us: i32,
    /// The OSC 8 hyperlink this cell belongs to, as an index into the screen's
    /// hyperlink table. Zero means none.
    pub(crate) link: u32,
}

impl Default for Cell {
    /// tmux's `grid_default_cell`: a space in the default colours.
    fn default() -> Cell {
        Cell {
            data: CellData::space(),
            attr: 0,
            flags: 0,
            fg: colour::DEFAULT,
            bg: colour::DEFAULT,
            us: colour::DEFAULT,
            link: 0,
        }
    }
}

impl Cell {
    /// tmux's `grid_default_cell` with the cleared flag set: a cell an erase
    /// produced rather than one a program wrote a space into.
    pub(crate) fn cleared(background: i32) -> Cell {
        Cell {
            bg: background,
            flags: flag::CLEARED,
            ..Cell::default()
        }
    }

    /// The blank right half of a wide character.
    pub(crate) fn padding(template: &Cell) -> Cell {
        Cell {
            data: CellData {
                bytes: Vec::new(),
                width: 1,
            },
            flags: template.flags | flag::PADDING,
            ..template.clone()
        }
    }

    pub(crate) fn is_padding(&self) -> bool {
        self.flags & flag::PADDING != 0
    }

    /// tmux's `grid_cells_look_equal`: whether two cells would be drawn the
    /// same, ignoring bookkeeping flags that do not reach the screen.
    pub(crate) fn looks_equal(&self, other: &Cell) -> bool {
        if self.fg != other.fg || self.bg != other.bg || self.us != other.us {
            return false;
        }
        if self.attr != other.attr || self.link != other.link {
            return false;
        }
        (self.flags & !(flag::CLEARED | flag::TAB)) == (other.flags & !(flag::CLEARED | flag::TAB))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_default_cell_is_a_space_in_default_colours() {
        let cell = Cell::default();
        assert!(cell.data.is_space());
        assert_eq!(cell.fg, colour::DEFAULT);
        assert_eq!(cell.attr, 0);
    }

    #[test]
    fn colours_round_trip_through_their_encodings() {
        assert_eq!(colour::as_index(colour::indexed(200)), Some(200));
        assert_eq!(colour::as_rgb(colour::rgb(1, 2, 3)), Some((1, 2, 3)));
        assert_eq!(colour::as_rgb(colour::indexed(4)), None);
        // aixterm brights fold onto the palette's own bright half.
        assert_eq!(colour::as_index(94), Some(12));
    }

    #[test]
    fn a_cluster_grows_only_to_the_cell_limit() {
        let mut data = CellData::from_char('e', 1);
        for _ in 0..20 {
            data.combine('\u{301}');
        }
        assert!(data.bytes.len() <= UTF8_SIZE);
        assert!(data.text().starts_with('e'));
    }

    #[test]
    fn cleared_and_tab_bookkeeping_does_not_change_how_a_cell_looks() {
        let plain = Cell::default();
        let cleared = Cell::cleared(colour::DEFAULT);
        assert!(plain.looks_equal(&cleared));
        let coloured = Cell {
            fg: colour::indexed(1),
            ..Cell::default()
        };
        assert!(!plain.looks_equal(&coloured));
    }
}
