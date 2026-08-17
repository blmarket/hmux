//! One grid cell: what is in it and how it is drawn.
//!
//! This is tmux's `struct grid_cell`, kept deliberately close to it. The
//! attribute bits, the colour encoding and the flag bits are the ones
//! `grid.c`/`screen-write.c` use, because the whole point of the in-house
//! engine is that the semantics are tmux's and can be read against tmux's
//! source without translation.

/// tmux's `GRID_ATTR_*`, as one bitmask.
pub mod attr {
    pub const BRIGHT: u16 = 0x1;
    pub const DIM: u16 = 0x2;
    pub const UNDERSCORE: u16 = 0x4;
    pub const BLINK: u16 = 0x8;
    pub const REVERSE: u16 = 0x10;
    pub const HIDDEN: u16 = 0x20;
    pub const ITALICS: u16 = 0x40;
    /// The alternate (line-drawing) character set is selected for this cell.
    pub const CHARSET: u16 = 0x80;
    pub const STRIKETHROUGH: u16 = 0x100;
    pub const UNDERSCORE_2: u16 = 0x200;
    pub const UNDERSCORE_3: u16 = 0x400;
    pub const UNDERSCORE_4: u16 = 0x800;
    pub const UNDERSCORE_5: u16 = 0x1000;
    pub const OVERLINE: u16 = 0x2000;

    /// Every underline style, so selecting one can clear the others.
    pub const ALL_UNDERSCORE: u16 =
        UNDERSCORE | UNDERSCORE_2 | UNDERSCORE_3 | UNDERSCORE_4 | UNDERSCORE_5;
}

/// tmux's `GRID_FLAG_*`, as one bitmask.
pub mod flag {
    /// The blank second half of a wide character.
    pub const PADDING: u8 = 0x4;
    /// The cell has never been written; a clear left it as it is.
    pub const CLEARED: u8 = 0x40;
    /// The cell is part of a run a horizontal tab created, rather than of
    /// spaces someone typed. `capture-pane -e` has to tell them apart.
    pub const TAB: u8 = 0x80;
}

/// tmux's colour encoding: a palette index, or an RGB triple, or "default".
///
/// tmux carries this as one `int` with flag bits above the value, and every
/// SGR handler reads and writes it in that form. Keeping the same encoding is
/// what lets those handlers be ported rather than reinterpreted.
pub mod colour {
    /// The 256-colour palette flag.
    pub const FLAG_256: i32 = 0x0100_0000;
    /// The direct-colour flag; the low 24 bits are then `0xrrggbb`.
    pub const FLAG_RGB: i32 = 0x0200_0000;
    /// tmux's "no colour chosen", which resolves against the pane's options.
    pub const DEFAULT: i32 = 8;

    /// One of the 256 palette entries.
    pub fn indexed(index: u8) -> i32 {
        i32::from(index) | FLAG_256
    }

    /// A direct colour.
    pub fn rgb(red: u8, green: u8, blue: u8) -> i32 {
        FLAG_RGB | (i32::from(red) << 16) | (i32::from(green) << 8) | i32::from(blue)
    }

    /// The `0xrrggbb` of a direct colour, if it is one.
    pub fn as_rgb(colour: i32) -> Option<(u8, u8, u8)> {
        if colour & FLAG_RGB == 0 {
            return None;
        }
        Some((
            (colour >> 16) as u8,
            (colour >> 8) as u8,
            (colour & 0xff) as u8,
        ))
    }

    // Consumed by the `capture-pane -e` style serializer in the server.
    #[allow(dead_code)]
    /// The palette index of a 256-colour value, if it is one. The eight bright
    /// aixterm colours (90–97, 100–107) are folded onto 8–15, as tmux folds
    /// them.
    pub fn as_index(colour: i32) -> Option<u8> {
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

/// The largest number of UTF-8 bytes one cell's cluster holds, matching tmux's
/// `UTF8_SIZE`. A cluster that would outgrow this starts a new cell instead,
/// which is observable: a seven-codepoint emoji ZWJ sequence is 26 bytes, so
/// the bound decides whether it lands in one cell or two.
pub const UTF8_SIZE: usize = 32;

/// The character content of a cell: a grapheme cluster and the number of
/// columns it occupies.
///
/// The cluster is stored inline, as tmux stores it in `struct utf8_data`. A
/// cell is written and read far more often than it is wide, and the cluster
/// can never outgrow [`UTF8_SIZE`] anyway, so the tail this wastes on the
/// common one-byte cell costs less than a heap allocation per cell would.
#[derive(Clone, Copy, Debug)]
pub struct CellData {
    /// The cluster's UTF-8 bytes. Only the first `len` are content; the rest
    /// are zero and never read, which is why equality goes through
    /// [`CellData::bytes`] rather than the array.
    bytes: [u8; UTF8_SIZE],
    /// How much of `bytes` is content.
    len: u8,
    /// Display width in columns: 0, 1 or 2.
    pub width: u8,
}

impl PartialEq for CellData {
    fn eq(&self, other: &CellData) -> bool {
        self.width == other.width && self.bytes() == other.bytes()
    }
}

impl Eq for CellData {}

impl CellData {
    /// The cell content of a space, which is what a clear leaves behind.
    pub const SPACE: CellData = {
        let mut bytes = [0u8; UTF8_SIZE];
        bytes[0] = b' ';
        CellData {
            bytes,
            len: 1,
            width: 1,
        }
    };

    /// The empty cluster: no content at all, which is what a padding cell
    /// holds.
    pub const EMPTY: CellData = CellData {
        bytes: [0u8; UTF8_SIZE],
        len: 0,
        width: 1,
    };

    /// One ASCII or single-codepoint character.
    pub fn from_char(character: char, width: u8) -> CellData {
        let mut bytes = [0u8; UTF8_SIZE];
        let len = character.encode_utf8(&mut bytes[..4]).len();
        CellData {
            bytes,
            len: len as u8,
            width,
        }
    }

    /// A run of `width` blanks in one cell, which is how a horizontal tab is
    /// stored. `width` is capped at [`UTF8_SIZE`]; the caller that produces a
    /// wider run writes no cell at all.
    pub fn blanks(width: usize) -> CellData {
        let width = width.min(UTF8_SIZE);
        let mut bytes = [0u8; UTF8_SIZE];
        bytes[..width].fill(b' ');
        CellData {
            bytes,
            len: width as u8,
            width: u8::try_from(width).unwrap_or(u8::MAX),
        }
    }

    /// The one-byte cluster a compact grid entry holds.
    ///
    /// Kept apart from [`CellData::from_bytes`] because this is the read every
    /// cell of an ordinary screen takes: a fixed-size store, with none of the
    /// variable-length copy a slice of unknown length compiles into.
    pub fn from_byte(byte: u8) -> CellData {
        let mut bytes = [0u8; UTF8_SIZE];
        bytes[0] = byte;
        CellData {
            bytes,
            len: 1,
            width: 1,
        }
    }

    /// A cluster from bytes the grid handed back, which are the bytes some
    /// earlier [`CellData`] held and are therefore already whole UTF-8 and
    /// already within [`UTF8_SIZE`].
    pub fn from_bytes(source: &[u8], width: u8) -> CellData {
        let mut bytes = [0u8; UTF8_SIZE];
        let len = source.len().min(UTF8_SIZE);
        bytes[..len].copy_from_slice(&source[..len]);
        CellData {
            bytes,
            len: len as u8,
            width,
        }
    }

    /// The cluster's bytes.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes[..usize::from(self.len)]
    }

    /// How many bytes the cluster holds.
    pub fn len(&self) -> usize {
        usize::from(self.len)
    }

    /// Append to the cluster, as combining a character into it does. The caller
    /// checks the room first: a cluster that would outgrow the cell starts its
    /// own instead, which is observable, so silently truncating here would be
    /// the wrong answer.
    pub fn extend(&mut self, extra: &[u8]) {
        let start = usize::from(self.len);
        let end = (start + extra.len()).min(UTF8_SIZE);
        self.bytes[start..end].copy_from_slice(&extra[..end - start]);
        self.len = end as u8;
    }

    /// Whether the cell holds exactly one space.
    pub fn is_space(&self) -> bool {
        self.bytes() == b" "
    }

    /// The cluster as text. Cell bytes are always valid UTF-8, because they
    /// only ever arrive as an encoded `char`.
    pub fn text(&self) -> &str {
        std::str::from_utf8(self.bytes()).unwrap_or("")
    }
}

/// One grid cell: tmux's `struct grid_cell`.
///
/// Every field is inline, so a cell copies with a `memcpy` and owns nothing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Cell {
    pub data: CellData,
    pub attr: u16,
    pub flags: u8,
    pub fg: i32,
    pub bg: i32,
    /// The underline colour, which SGR 58 sets separately from the foreground.
    pub us: i32,
    /// The OSC 8 hyperlink this cell belongs to, as an index into the screen's
    /// hyperlink table. Zero means none.
    pub link: u32,
}

impl Default for Cell {
    /// tmux's `grid_default_cell`: a space in the default colours.
    fn default() -> Cell {
        Cell::DEFAULT
    }
}

impl Cell {
    /// tmux's `grid_default_cell`, as a constant so that a read past a row's
    /// extent can borrow it rather than build one.
    pub const DEFAULT: Cell = Cell {
        data: CellData::SPACE,
        attr: 0,
        flags: 0,
        fg: colour::DEFAULT,
        bg: colour::DEFAULT,
        us: colour::DEFAULT,
        link: 0,
    };

    /// tmux's `grid_default_cell` with the cleared flag set: a cell an erase
    /// produced rather than one a program wrote a space into.
    pub fn cleared(background: i32) -> Cell {
        Cell {
            bg: background,
            flags: flag::CLEARED,
            ..Cell::default()
        }
    }

    /// The blank right half of a wide character.
    /// tmux's `grid_padding_cell`, which is a constant: the right half of a
    /// wide character inherits nothing from the character it belongs to — not
    /// its colours, not its attributes, not its hyperlink, not the tab flag.
    /// That is observable wherever a padding cell outlives its character, since
    /// what is left behind is a default cell rather than a coloured one.
    pub fn padding() -> Cell {
        Cell {
            data: CellData::EMPTY,
            flags: flag::PADDING,
            ..Cell::DEFAULT
        }
    }

    pub fn is_padding(&self) -> bool {
        self.flags & flag::PADDING != 0
    }

    /// tmux's `grid_cells_look_equal`: whether two cells would be drawn the
    /// same, ignoring bookkeeping flags that do not reach the screen.
    pub fn looks_equal(&self, other: &Cell) -> bool {
        if self.fg != other.fg || self.bg != other.bg || self.us != other.us {
            return false;
        }
        if self.attr != other.attr || self.link != other.link {
            return false;
        }
        (self.flags & !(flag::CLEARED | flag::TAB)) == (other.flags & !(flag::CLEARED | flag::TAB))
    }

    /// tmux's `grid_cells_equal`: the same appearance and the same content.
    ///
    /// This does not go through [`Cell::looks_equal`], because the two ask
    /// different questions. `grid_cells_look_equal` masks only the cleared
    /// flag, so a cell a tab produced is *not* equal to one holding a typed
    /// space — a distinction `capture-pane -e` depends on. It also never
    /// compares the underline colour, which is tmux's and stays tmux's.
    pub fn equals(&self, other: &Cell) -> bool {
        self.fg == other.fg
            && self.bg == other.bg
            && self.attr == other.attr
            && self.link == other.link
            && (self.flags & !flag::CLEARED) == (other.flags & !flag::CLEARED)
            && self.data == other.data
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
