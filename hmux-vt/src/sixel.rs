//! The sixel codec: tmux's `image-sixel.c`.
//!
//! A sixel image arrives as a DCS payload, is decoded into a plane of colour
//! register indices, and leaves again either as bytes for a client terminal
//! that speaks sixel or as the text placeholder for one that does not. Nothing
//! here touches a screen; the engine's image list is what anchors an image to
//! cells.
//!
//! The port is literal, quirks included, because the bytes hmux writes to a
//! client have to be the bytes tmux would have written. Two places where that
//! costs something are called out where they appear: the parser's arithmetic is
//! done on a *signed* `char`, as it is in C on the platforms tmux is built for,
//! and [`SixelImage::scale`] reproduces a raster-attribute computation whose
//! first two assignments are immediately overwritten.

/// tmux's `SIXEL_WIDTH_LIMIT` and `SIXEL_HEIGHT_LIMIT`.
const WIDTH_LIMIT: u32 = 10000;
const HEIGHT_LIMIT: u32 = 10000;

/// tmux's `SIXEL_COLOUR_REGISTERS`, which is also what XTSMGRAPHICS reports.
pub(crate) const COLOUR_REGISTERS: u32 = 1024;

/// One row of six pixels' worth of colour indices, tmux's `struct sixel_line`.
///
/// `x` is the allocated width, which is not the image's: a line is widened to
/// the image's width at the moment it is written to, so a line written early
/// and never touched again stays as narrow as the image was then.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct SixelLine {
    x: u32,
    /// Colour register index plus one; zero is "no pixel here".
    data: Vec<u16>,
}

/// A decoded sixel image, tmux's `struct sixel_image`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SixelImage {
    x: u32,
    y: u32,
    /// The cell size in pixels the image is measured against — the window's,
    /// when parsing, and the client tty's, when scaling for output.
    xpixel: u32,
    ypixel: u32,

    /// Whether the payload carried a `"` raster-attributes introducer.
    set_ra: bool,
    ra_x: u32,
    ra_y: u32,

    /// Colour registers, packed as tmux packs them: `(type << 25) | (c1 << 16)
    /// | (c2 << 8) | c3`.
    colours: Vec<u32>,
    used_colours: u32,
    /// The DCS's second parameter, the background-select mode, carried through
    /// so a re-serialization says what the original said.
    p2: u32,

    /// The write cursor, tmux's `dx`/`dy`/`dc`.
    dx: u32,
    dy: u32,
    dc: u32,

    lines: Vec<SixelLine>,
}

impl SixelImage {
    fn new(p2: u32, xpixel: u32, ypixel: u32) -> SixelImage {
        SixelImage {
            x: 0,
            y: 0,
            xpixel,
            ypixel,
            set_ra: false,
            ra_x: 0,
            ra_y: 0,
            colours: Vec::new(),
            used_colours: 0,
            p2,
            dx: 0,
            dy: 0,
            dc: 0,
            lines: Vec::new(),
        }
    }

    /// tmux's `sixel_parse_expand_lines`. The answer is C's: `true` is the
    /// failure the caller aborts the parse on.
    fn expand_lines(&mut self, y: u32) -> bool {
        if y <= self.y {
            return false;
        }
        if y > HEIGHT_LIMIT {
            return true;
        }
        self.lines.resize(y as usize, SixelLine::default());
        self.y = y;
        false
    }

    /// tmux's `sixel_parse_expand_line`. A line is widened to the *image's*
    /// width, not to the column being written, which is why every line touched
    /// after the image grew is as wide as the image was at that moment.
    fn expand_line(&mut self, y: usize, x: u32) -> bool {
        if x <= self.lines[y].x {
            return false;
        }
        if x > WIDTH_LIMIT {
            return true;
        }
        if x > self.x {
            self.x = x;
        }
        let width = self.x;
        let line = &mut self.lines[y];
        line.data.resize(width as usize, 0);
        line.x = width;
        false
    }

    /// tmux's `sixel_get_pixel`: outside the image is transparent.
    fn get_pixel(&self, x: u32, y: u32) -> u16 {
        if y >= self.y {
            return 0;
        }
        let line = &self.lines[y as usize];
        if x >= line.x {
            return 0;
        }
        line.data[x as usize]
    }

    /// tmux's `sixel_set_pixel`.
    fn set_pixel(&mut self, x: u32, y: u32, colour: u16) -> bool {
        if self.expand_lines(y + 1) {
            return true;
        }
        if self.expand_line(y as usize, x + 1) {
            return true;
        }
        self.lines[y as usize].data[x as usize] = colour;
        false
    }

    /// tmux's `sixel_parse_write`: one sixel character paints up to six pixels
    /// down one column.
    ///
    /// `ch` is a `u_int` in tmux and reaches it from a `char`, so a repeat
    /// introducer's data byte below `0x3f` arrives here sign-extended and lights
    /// bits the character never named. That is reproduced rather than fixed:
    /// see [`parse_repeat`].
    fn parse_write(&mut self, ch: u32) -> bool {
        for i in 0..6 {
            if ch & (1 << i) != 0 {
                let (dx, dy, dc) = (self.dx, self.dy + i, self.dc);
                if self.set_pixel(dx, dy, dc as u16) {
                    return true;
                }
            }
        }
        false
    }

    /// tmux's `sixel_size_in_cells`: pixel dimensions rounded up to whole cells.
    pub(crate) fn size_in_cells(&self) -> (u32, u32) {
        // tmux divides by the window's cell size, which `recalculate_sizes`
        // never leaves at zero. Clamping keeps a synthesized image from
        // dividing by it anyway.
        let (xpixel, ypixel) = (self.xpixel.max(1), self.ypixel.max(1));
        let x = self.x.div_ceil(xpixel);
        let y = self.y.div_ceil(ypixel);
        (x, y)
    }

    /// tmux's `sixel_scale`: take the section of the image at `ox`,`oy` in
    /// *image* cells, `sx` by `sy` cells across, and resample it onto the same
    /// number of cells measured in `xpixel` by `ypixel` pixels.
    ///
    /// Both jobs tmux uses this for come through here: cropping an image to the
    /// space a screen has for it (with the image's own cell size, so nothing is
    /// resampled) and retargeting one to a client tty's pixel geometry.
    // `sixel_scale`'s parameters, in its order: grouping them would only hide
    // which C argument each one is.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn scale(
        &self,
        xpixel: u32,
        ypixel: u32,
        ox: u32,
        oy: u32,
        sx: u32,
        sy: u32,
        colours: bool,
    ) -> Option<SixelImage> {
        let (cx, cy) = self.size_in_cells();
        if ox >= cx || oy >= cy {
            return None;
        }
        let sx = if ox.saturating_add(sx) >= cx {
            cx - ox
        } else {
            sx
        };
        let sy = if oy.saturating_add(sy) >= cy {
            cy - oy
        } else {
            sy
        };

        let xpixel = if xpixel == 0 { self.xpixel } else { xpixel };
        let ypixel = if ypixel == 0 { self.ypixel } else { ypixel };

        let (pox, poy) = (ox * self.xpixel, oy * self.ypixel);
        let (psx, psy) = (sx * self.xpixel, sy * self.ypixel);
        let (tsx, tsy) = (sx * xpixel, sy * ypixel);

        let mut new = SixelImage::new(self.p2, xpixel, ypixel);
        new.set_ra = self.set_ra;
        // tmux subtracts the offset from `new`'s raster attributes and then
        // overwrites both with the clamp against the source's, so the offset
        // never reaches the result. Reproduced as written: the raster
        // attributes are re-emitted by `print`, and a "corrected" value would
        // put different bytes on the client's terminal than tmux does.
        new.ra_x = self.ra_x.min(psx);
        new.ra_y = self.ra_y.min(psy);
        new.ra_x = new.ra_x * xpixel / self.xpixel.max(1);
        new.ra_y = new.ra_y * ypixel / self.ypixel.max(1);

        new.used_colours = self.used_colours;
        for y in 0..tsy {
            let py = poy + (f64::from(y) * f64::from(psy) / f64::from(tsy)) as u32;
            for x in 0..tsx {
                let px = pox + (f64::from(x) * f64::from(psx) / f64::from(tsx)) as u32;
                // tmux ignores the limit failure here, so a resample that runs
                // past one simply stops filling.
                new.set_pixel(x, y, self.get_pixel(px, py));
            }
        }

        if colours && !self.colours.is_empty() {
            new.colours = self.colours.clone();
        }
        Some(new)
    }

    /// tmux's `sixel_print`: re-serialize as a complete `DCS q … ST` string.
    ///
    /// The palette comes from `map` when one is given — tty output prints the
    /// *scaled* image's pixels against the *original's* colour registers, since
    /// scaling for a client deliberately does not copy them.
    ///
    /// `None` is tmux's NULL: an image that never selected a colour has nothing
    /// to say.
    pub(crate) fn print(&self, map: Option<&SixelImage>) -> Option<Vec<u8>> {
        let colours = match map {
            Some(map) => &map.colours,
            None => &self.colours,
        };

        let used_colours = self.used_colours as usize;
        if used_colours == 0 {
            return None;
        }

        let mut buf = Vec::with_capacity(8192);
        buf.extend_from_slice(format!("\x1bP0;{}q", self.p2).as_bytes());

        if self.set_ra {
            buf.extend_from_slice(format!("\"1;1;{};{}", self.ra_x, self.ra_y).as_bytes());
        }

        for (i, colour) in colours.iter().enumerate() {
            buf.extend_from_slice(
                format!(
                    "#{};{};{};{};{}",
                    i,
                    colour >> 25,
                    (colour >> 16) & 0x1ff,
                    (colour >> 8) & 0xff,
                    colour & 0xff
                )
                .as_bytes(),
            );
        }

        let mut chunks = vec![Chunk::default(); used_colours];
        let mut y = 0;
        while y < self.y {
            let active = self.compress_colours(&mut chunks, y);

            for c in active {
                let chunk = &mut chunks[c as usize];
                buf.extend_from_slice(format!("#{c}").as_bytes());
                buf.extend_from_slice(&chunk.data);
                print_repeat(&mut buf, chunk.count, chunk.pattern.wrapping_add(0x3f));
                buf.push(b'$');
                chunk.data.clear();
                chunk.next_x = 0;
                chunk.count = 0;
            }

            if buf.last() == Some(&b'$') {
                buf.pop();
            }
            buf.push(b'-');
            y += 6;
        }
        if buf.last() == Some(&b'-') {
            buf.pop();
        }

        buf.extend_from_slice(b"\x1b\\");
        Some(buf)
    }

    /// tmux's `sixel_print_compress_colors`: walk one band of six pixel rows
    /// and append, per colour, the run-length-encoded sixels that colour paints.
    ///
    /// The answer is tmux's `active` array: the colours this band uses, in the
    /// order they were first reached, which is the order they are emitted in.
    fn compress_colours(&self, chunks: &mut [Chunk], y: u32) -> Vec<u32> {
        let mut active = Vec::new();
        let mut colours = [0u16; 6];
        for x in 0..self.x {
            for (i, slot) in colours.iter_mut().enumerate() {
                *slot = 0;
                let row = y + i as u32;
                if row < self.y {
                    let line = &self.lines[row as usize];
                    if x < line.x && line.data[x as usize] != 0 {
                        *slot = line.data[x as usize];
                        let c = usize::from(line.data[x as usize] - 1);
                        if c < chunks.len() {
                            chunks[c].next_pattern |= 1 << i;
                        }
                    }
                }
            }

            for slot in colours {
                if slot == 0 {
                    continue;
                }
                let c = u32::from(slot - 1);
                let Some(chunk) = chunks.get_mut(c as usize) else {
                    continue;
                };
                if chunk.next_x == x + 1 {
                    continue;
                }

                if chunk.next_y < y + 1 {
                    chunk.next_y = y + 1;
                    active.push(c);
                }

                let dx = x - chunk.next_x;
                if chunk.pattern != chunk.next_pattern || dx != 0 {
                    let (count, pattern) = (chunk.count, chunk.pattern);
                    print_repeat(&mut chunk.data, count, pattern.wrapping_add(0x3f));
                    print_repeat(&mut chunk.data, dx, b'?');
                    chunk.pattern = chunk.next_pattern;
                    chunk.count = 0;
                }
                chunk.count += 1;
                chunk.next_pattern = 0;
                chunk.next_x = x + 1;
            }
        }
        active
    }

    /// tmux's `image_fallback`: the text placeholder a client that cannot draw
    /// sixel gets instead.
    ///
    /// Written exactly as tmux writes it — one `\r\n`-terminated line per image
    /// row, the label on the first — because tmux hands these bytes to the
    /// client's terminal raw, carriage returns and all.
    pub(crate) fn fallback_text(sx: u32, sy: u32) -> String {
        let label = format!("SIXEL IMAGE ({sx}x{sy})");
        let mut out = String::new();
        out.push_str(&label);
        if (sx as usize) >= label.len() {
            // Wide enough to pad; narrower than the label, tmux writes the
            // label anyway and lets it run past the image's own width.
            out.extend(std::iter::repeat_n('+', sx as usize - label.len()));
        }
        out.push_str("\r\n");
        for _ in 1..sy {
            out.extend(std::iter::repeat_n('+', sx as usize));
            out.push_str("\r\n");
        }
        out
    }
}

/// One colour's pending run inside a band, tmux's `struct sixel_chunk`.
#[derive(Clone, Debug, Default)]
struct Chunk {
    next_x: u32,
    next_y: u32,
    count: u32,
    pattern: u8,
    next_pattern: u8,
    data: Vec<u8>,
}

/// tmux's `sixel_print_repeat`: up to three of a character go out literally,
/// more take the `!count` form, and zero writes nothing.
fn print_repeat(buf: &mut Vec<u8>, count: u32, ch: u8) {
    match count {
        0 => {}
        1..=3 => buf.extend(std::iter::repeat_n(ch, count as usize)),
        _ => buf.extend_from_slice(format!("!{count}{}", ch as char).as_bytes()),
    }
}

/// C's `strtoul` over the digits at `pos`, answering the value and where it
/// stopped.
///
/// The parser only ever runs this over a region it has already checked contains
/// nothing but digits and `;`, so the leading-whitespace and sign handling of
/// the real thing has nothing to do. Overflow saturates the way `strtoul` does,
/// at `ULONG_MAX`, and the caller's `u_int` truncation is applied by the cast at
/// the call site.
fn strtoul(bytes: &[u8], pos: usize) -> (u64, usize) {
    let mut value: u64 = 0;
    let mut end = pos;
    while end < bytes.len() && bytes[end].is_ascii_digit() {
        value = value
            .saturating_mul(10)
            .saturating_add(u64::from(bytes[end] - b'0'));
        end += 1;
    }
    (value, end)
}

/// The byte at `pos`, or NUL past the end.
///
/// tmux's payload buffer is NUL-terminated, and the parser leans on that: a
/// number that runs to the very end of the payload is rejected because the byte
/// after it is not the `;` the grammar wanted, and that byte is the terminator.
fn at(bytes: &[u8], pos: usize) -> u8 {
    bytes.get(pos).copied().unwrap_or(0)
}

/// The end of the run of digits and `;` starting at `pos`, which is how far
/// tmux's parsers let a number reach.
fn scan_numeric(bytes: &[u8], pos: usize) -> usize {
    let mut last = pos;
    while last < bytes.len() && (bytes[last] == b';' || bytes[last].is_ascii_digit()) {
        last += 1;
    }
    last
}

/// tmux's `sixel_parse_attributes`, the `"` introducer.
///
/// `None` is tmux's NULL: the whole image is rejected. A payload with fewer
/// than four parameters is not an error — it is simply ignored, and parsing
/// resumes after it.
fn parse_attributes(si: &mut SixelImage, bytes: &[u8], pos: usize) -> Option<usize> {
    let last = scan_numeric(bytes, pos);

    let (_, mut end) = strtoul(bytes, pos);
    if end == last || at(bytes, end) != b';' {
        return Some(last);
    }
    let (_, next) = strtoul(bytes, end + 1);
    end = next;
    if end == last {
        return Some(last);
    }
    if at(bytes, end) != b';' {
        return None;
    }

    let (x, next) = strtoul(bytes, end + 1);
    end = next;
    let x = x as u32;
    if end == last || at(bytes, end) != b';' {
        return None;
    }
    if x > WIDTH_LIMIT {
        return None;
    }
    let (y, next) = strtoul(bytes, end + 1);
    end = next;
    let y = y as u32;
    if end != last {
        return None;
    }
    if y > HEIGHT_LIMIT {
        return None;
    }

    si.x = x;
    si.expand_lines(y);

    si.set_ra = true;
    si.ra_x = x;
    si.ra_y = y;

    Some(last)
}

/// tmux's `sixel_parse_colour`, the `#` introducer: define a register when the
/// full `Pc;Pu;Px;Py;Pz` form is given, and select it either way.
fn parse_colour(si: &mut SixelImage, bytes: &[u8], pos: usize) -> Option<usize> {
    let last = scan_numeric(bytes, pos);

    let (c, mut end) = strtoul(bytes, pos);
    let c = c as u32;
    if c > COLOUR_REGISTERS {
        return None;
    }
    if si.used_colours <= c {
        si.used_colours = c + 1;
    }
    si.dc = c + 1;
    if end == last || at(bytes, end) != b';' {
        return Some(last);
    }

    let (kind, next) = strtoul(bytes, end + 1);
    end = next;
    let kind = kind as u32;
    if end == last || at(bytes, end) != b';' {
        return None;
    }
    let (c1, next) = strtoul(bytes, end + 1);
    end = next;
    let c1 = c1 as u32;
    if end == last || at(bytes, end) != b';' {
        return None;
    }
    let (c2, next) = strtoul(bytes, end + 1);
    end = next;
    let c2 = c2 as u32;
    if end == last || at(bytes, end) != b';' {
        return None;
    }
    let (c3, next) = strtoul(bytes, end + 1);
    end = next;
    let c3 = c3 as u32;
    if end != last {
        return None;
    }

    // Type 1 is HLS, whose first component is an angle; type 2 is RGB, whose
    // components are all percentages.
    if (kind != 1 && kind != 2)
        || (kind == 1 && (c1 > 360 || c2 > 100 || c3 > 100))
        || (kind == 2 && (c1 > 100 || c2 > 100 || c3 > 100))
    {
        return None;
    }

    if c as usize + 1 > si.colours.len() {
        si.colours.resize(c as usize + 1, 0);
    }
    si.colours[c as usize] = (kind << 25) | (c1 << 16) | (c2 << 8) | c3;
    Some(last)
}

/// tmux's `sixel_parse_repeat`, the `!` introducer.
///
/// The repeated byte is not validated the way a bare sixel character is: tmux
/// subtracts `0x3f` from it as a signed `char` and hands the result to
/// `sixel_parse_write` as a `u_int`, so a byte below `0x3f` becomes a very
/// large number and paints whichever of the six rows its low bits happen to
/// name. Reproducing that is the point — the corpus contains payloads that do
/// it, and the grid has to end up the same.
fn parse_repeat(si: &mut SixelImage, bytes: &[u8], pos: usize) -> Option<usize> {
    let mut last = pos;
    let mut digits = 0;
    while last < bytes.len() && bytes[last].is_ascii_digit() {
        last += 1;
        digits += 1;
        // tmux's `tmp` is 32 bytes and it gives up rather than overrun it.
        if digits == 31 {
            return None;
        }
    }
    if digits == 0 || last >= bytes.len() {
        return None;
    }

    let (n, _) = strtoul(bytes, pos);
    // tmux's `strtonum(tmp, 1, SIXEL_WIDTH_LIMIT, …)`.
    if n < 1 || n > u64::from(WIDTH_LIMIT) {
        return None;
    }
    let n = n as u32;

    let ch = (bytes[last] as i8).wrapping_sub(0x3f) as u32;
    last += 1;
    for _ in 0..n {
        if si.parse_write(ch) {
            return None;
        }
        si.dx += 1;
    }
    Some(last)
}

/// tmux's `sixel_parse`.
///
/// `payload` is the DCS string *after* its final byte, which hmux's parser has
/// already split off; tmux's `buf[0] == 'q'` check is the caller's job here, and
/// its "empty image" rejection of a one-byte buffer becomes an empty payload.
pub(crate) fn parse(payload: &[u8], p2: u32, xpixel: u32, ypixel: u32) -> Option<SixelImage> {
    if payload.is_empty() {
        return None;
    }

    let mut si = SixelImage::new(p2, xpixel, ypixel);

    let mut pos = 0;
    while pos < payload.len() {
        let byte = payload[pos];
        pos += 1;
        match byte {
            b'"' => pos = parse_attributes(&mut si, payload, pos)?,
            b'#' => pos = parse_colour(&mut si, payload, pos)?,
            b'!' => pos = parse_repeat(&mut si, payload, pos)?,
            b'-' => {
                si.dx = 0;
                si.dy += 6;
            }
            b'$' => si.dx = 0,
            _ => {
                // tmux reads the payload as `char`, so every byte with the top
                // bit set compares below `0x20` and is skipped along with the
                // controls. Everything from `0x20` to `0x3e`, and `0x7f`,
                // rejects the image.
                let ch = byte as i8;
                if ch < 0x20 {
                    continue;
                }
                if !(0x3f..=0x7e).contains(&byte) {
                    return None;
                }
                if si.parse_write(u32::from(byte - 0x3f)) {
                    return None;
                }
                si.dx += 1;
            }
        }
    }

    if si.x == 0 || si.y == 0 {
        return None;
    }
    Some(si)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A two-by-two block of colour one, as the sixel grammar spells it.
    const BLOCK: &[u8] = b"#0;2;100;0;0#0~~";

    fn parse_ok(payload: &[u8]) -> SixelImage {
        parse(payload, 0, 10, 20).expect("the payload parses")
    }

    #[test]
    fn a_bare_sixel_character_paints_six_rows() {
        // `~` is 0x7e, which is 0x3f + 0b111111: all six pixels.
        let si = parse_ok(b"#0;2;100;0;0#0~");
        assert_eq!((si.x, si.y), (1, 6));
        for y in 0..6 {
            assert_eq!(si.get_pixel(0, y), 1, "row {y} is colour register zero");
        }
    }

    #[test]
    fn size_in_cells_rounds_up_to_whole_cells() {
        let si = parse_ok(BLOCK);
        assert_eq!((si.x, si.y), (2, 6));
        // Two pixels across a ten-pixel cell is one cell; six down a
        // twenty-pixel cell is one too.
        assert_eq!(si.size_in_cells(), (1, 1));

        let si = parse(b"#0;2;100;0;0#0!12~-!12~", 0, 10, 20).expect("parses");
        assert_eq!((si.x, si.y), (12, 12));
        assert_eq!(si.size_in_cells(), (2, 1), "twelve rows still fit one cell");
    }

    #[test]
    fn a_repeat_paints_the_same_column_many_times() {
        let si = parse_ok(b"#0;2;100;0;0#0!4~");
        assert_eq!(si.x, 4);
        assert_eq!(si.get_pixel(3, 5), 1);
    }

    #[test]
    fn a_dash_starts_the_next_band_and_a_dollar_returns_to_column_zero() {
        let si = parse_ok(b"#0;2;100;0;0#0~-~");
        assert_eq!((si.x, si.y), (1, 12));
        assert_eq!(si.get_pixel(0, 6), 1);

        let si = parse_ok(b"#0;2;100;0;0#0~$#0@");
        // The `$` rewound to column zero, so `@` (one pixel) overwrote row zero
        // of the same column.
        assert_eq!(si.x, 1);
        assert_eq!(si.get_pixel(0, 0), 1);
    }

    #[test]
    fn raster_attributes_set_the_declared_size() {
        let si = parse_ok(b"\"1;1;8;12#0;2;100;0;0#0~");
        assert!(si.set_ra);
        assert_eq!((si.ra_x, si.ra_y), (8, 12));
        assert_eq!(si.y, 12, "the declared height allocated the lines");
    }

    /// tmux ignores an attribute run it cannot read four parameters out of,
    /// rather than rejecting the image.
    #[test]
    fn short_raster_attributes_are_ignored_not_fatal() {
        let si = parse_ok(b"\"1#0;2;100;0;0#0~");
        assert!(!si.set_ra);
        assert_eq!(si.x, 1);
    }

    #[test]
    fn a_colour_selection_without_a_definition_still_selects() {
        let si = parse_ok(b"#3~");
        assert_eq!(si.used_colours, 4);
        assert_eq!(
            si.get_pixel(0, 0),
            4,
            "the stored index is the register + 1"
        );
        assert!(si.colours.is_empty(), "nothing was defined");
    }

    #[test]
    fn the_limits_reject_rather_than_allocate() {
        assert!(parse(b"\"1;1;20000;1#0~", 0, 10, 20).is_none(), "too wide");
        assert!(parse(b"\"1;1;1;20000#0~", 0, 10, 20).is_none(), "too tall");
        assert!(
            parse(b"#2000;2;100;0;0#0~", 0, 10, 20).is_none(),
            "past the last colour register"
        );
        assert!(
            parse(b"#0;3;100;0;0#0~", 0, 10, 20).is_none(),
            "neither HLS nor RGB"
        );
        assert!(
            parse(b"#0;1;400;0;0#0~", 0, 10, 20).is_none(),
            "an HLS angle past 360"
        );
    }

    #[test]
    fn a_payload_that_paints_nothing_is_not_an_image() {
        assert!(parse(b"", 0, 10, 20).is_none());
        assert!(parse(b"#0;2;100;0;0", 0, 10, 20).is_none(), "no pixels");
    }

    #[test]
    fn bytes_outside_the_sixel_range_reject_and_controls_do_not() {
        assert!(
            parse(b"#0;2;100;0;0#0~\n~", 0, 10, 20).is_some(),
            "a newline"
        );
        assert!(parse(b"#0;2;100;0;0#0~\x80~", 0, 10, 20).is_some(), "0x80");
        assert!(parse(b"#0;2;100;0;0#0~ ~", 0, 10, 20).is_none(), "a space");
        assert!(parse(b"#0;2;100;0;0#0~\x7f", 0, 10, 20).is_none(), "0x7f");
    }

    /// The repeat introducer does not check its data byte, and the subtraction
    /// is signed, so a byte below `0x3f` lights the rows its sign extension
    /// names rather than none at all.
    #[test]
    fn a_repeat_of_a_byte_below_the_sixel_range_paints_by_sign_extension() {
        let si = parse(b"#0;2;100;0;0#0!1\x20", 0, 10, 20).expect("parses");
        // 0x20 - 0x3f is -31, which as a `u_int` is 0xffffffe1: bits zero and
        // five of the low six.
        assert_eq!(si.get_pixel(0, 0), 1);
        assert_eq!(si.get_pixel(0, 1), 0);
        assert_eq!(si.get_pixel(0, 5), 1);
    }

    #[test]
    fn a_repeat_needs_a_count_and_a_byte_to_repeat() {
        assert!(parse(b"#0;2;100;0;0#0!~", 0, 10, 20).is_none(), "no count");
        assert!(parse(b"#0;2;100;0;0#0!4", 0, 10, 20).is_none(), "no byte");
        assert!(parse(b"#0;2;100;0;0#0!0~", 0, 10, 20).is_none(), "zero");
    }

    #[test]
    fn scaling_crops_to_the_cells_asked_for() {
        // Four cells across at ten pixels each, one cell down.
        let si = parse(b"#0;2;100;0;0#0!40~", 0, 10, 20).expect("parses");
        assert_eq!(si.size_in_cells(), (4, 1));
        let cropped = si.scale(0, 0, 1, 0, 2, 1, true).expect("crops");
        assert_eq!(cropped.size_in_cells(), (2, 1));
        assert_eq!(cropped.x, 20, "two cells of ten pixels");
        assert_eq!(
            cropped.colours, si.colours,
            "cropping keeps the palette when asked"
        );
    }

    #[test]
    fn scaling_resamples_onto_a_different_cell_size() {
        let si = parse(b"#0;2;100;0;0#0!10~", 0, 10, 20).expect("parses");
        let scaled = si.scale(20, 40, 0, 0, 1, 1, false).expect("scales");
        assert_eq!((scaled.x, scaled.y), (20, 40));
        assert!(scaled.colours.is_empty(), "the palette was not asked for");
        assert_eq!(scaled.used_colours, si.used_colours);
    }

    #[test]
    fn scaling_outside_the_image_has_no_answer() {
        let si = parse(b"#0;2;100;0;0#0!10~", 0, 10, 20).expect("parses");
        assert!(si.scale(0, 0, 1, 0, 1, 1, true).is_none());
        assert!(si.scale(0, 0, 0, 1, 1, 1, true).is_none());
    }

    #[test]
    fn a_printed_image_parses_back_to_the_same_pixels() {
        let si = parse(b"#0;2;100;0;0#1;2;0;100;0#0~~#1@@", 0, 10, 20).expect("parses");
        let printed = si.print(None).expect("prints");
        assert!(printed.starts_with(b"\x1bP0;0q"));
        assert!(printed.ends_with(b"\x1b\\"));

        // Strip the introducer and the terminator to get back to a payload.
        let payload = &printed[b"\x1bP0;0q".len()..printed.len() - 2];
        let again = parse(payload, 0, 10, 20).expect("the re-serialization parses");
        assert_eq!((again.x, again.y), (si.x, si.y));
        for y in 0..si.y {
            for x in 0..si.x {
                assert_eq!(again.get_pixel(x, y), si.get_pixel(x, y), "pixel {x},{y}");
            }
        }
        assert_eq!(again.colours, si.colours);
    }

    #[test]
    fn printing_carries_the_background_mode_and_raster_attributes() {
        let si = parse(b"\"1;1;2;6#0;2;100;0;0#0~~", 2, 10, 20).expect("parses");
        let printed = si.print(None).expect("prints");
        assert!(printed.starts_with(b"\x1bP0;2q\"1;1;2;6"));
    }

    #[test]
    fn printing_takes_the_palette_from_the_map_when_one_is_given() {
        let si = parse(b"#0;2;100;0;0#0~", 0, 10, 20).expect("parses");
        let scaled = si.scale(10, 20, 0, 0, 1, 1, false).expect("scales");
        let printed = scaled.print(Some(&si)).expect("prints");
        let definition = b"#0;2;100;0;0";
        assert!(
            printed
                .windows(definition.len())
                .any(|window| window == definition),
            "the original's register definition is in the output"
        );

        // Scaling for a client deliberately drops the palette, so without the
        // map there would be nothing to define the register with.
        let alone = scaled.print(None).expect("prints");
        assert!(!alone
            .windows(definition.len())
            .any(|window| window == definition));
    }

    #[test]
    fn an_image_that_selected_no_colour_prints_nothing() {
        let mut si = parse(b"#0;2;100;0;0#0~", 0, 10, 20).expect("parses");
        si.used_colours = 0;
        assert!(si.print(None).is_none());
    }

    #[test]
    fn the_placeholder_labels_the_image_and_fills_the_rest() {
        let expected = concat!(
            "SIXEL IMAGE (24x3)++++++\r\n",
            "++++++++++++++++++++++++\r\n",
            "++++++++++++++++++++++++\r\n",
        );
        assert_eq!(SixelImage::fallback_text(24, 3), expected);
    }

    #[test]
    fn a_placeholder_narrower_than_its_label_keeps_the_whole_label() {
        assert_eq!(
            SixelImage::fallback_text(4, 2),
            "SIXEL IMAGE (4x2)\r\n++++\r\n"
        );
    }
}
