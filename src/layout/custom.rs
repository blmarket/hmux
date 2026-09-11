use super::{layout_cell, LayoutAccess};
use super::cells::{
    LAYOUT_CELL_FLOATING, LAYOUT_LEFTRIGHT, LAYOUT_TOPBOTTOM, LAYOUT_WINDOWPANE, LayoutCellPath,
    layout_cell_for_pane, layout_count_cells, layout_create_cell, layout_free_cell,
    layout_make_leaf, layout_print_cell,
};
use crate::fmt_args;
use crate::notify::notify_window;
use crate::resize::recalculate_sizes;
pub use crate::types::*;
use crate::window::window_count_panes;
use crate::xmalloc::xasprintf;
use ::core::ffi::{CStr, c_char, c_int};
use ::std::ffi::CString;

/// How much of `layout_dump`'s buffer one cell may need.
const CELL_MAX: usize = 64;

/// Whether `b` is one of the digits the layout string is built out of.
fn digit(b: c_char) -> bool {
    (b as u_char).is_ascii_digit()
}

/// The bottom-right leaf under `lc`, which is the one a layout with more cells
/// than panes drops first.
fn layout_find_bottomright(mut cell: &layout_cell) -> LayoutCellPath {
    let mut path = LayoutCellPath::root();
    while cell.type_0 != LAYOUT_WINDOWPANE {
        let index = cell
            .cells
            .len()
            .checked_sub(1)
            .expect("a layout node has children");
        path = path.child(index);
        cell = &cell.cells[index];
    }
    path
}

/// The checksum tmux writes in front of a layout: a sixteen-bit value rotated
/// right one bit per character, with the character added in as the signed
/// value the C's `char` carried.
fn layout_checksum(layout: &CStr) -> u_short {
    let mut csum: u_short = 0;
    for &b in layout.to_bytes() {
        csum = ((csum as c_int >> 1) + ((csum as c_int & 1) << 15)) as u_short;
        csum = (csum as c_int + b as c_char as c_int) as u_short;
    }
    csum
}

/// Writes `lc` and everything under it onto the end of `buf`, answering -1 if
/// what it holds plus what is written would reach `len` bytes counting the
/// terminator, which is where the C's `strlcat` gave up.
///
/// The "one cell did not fit its own sixty-four bytes" guard is kept as the C
/// wrote it, but no test reaches it: the widest numbers a cell can carry still
/// spell out well short of that.
fn layout_append(lc: Option<&layout_cell>, buf: &mut Vec<u8>, len: usize) -> c_int {
    if len == 0 {
        return -1;
    }
    let Some(lc) = lc else {
        return 0;
    };

    let tmp = if let Some(pane) = lc.wp.as_ref() {
        format!("{}x{},{},{},{}", lc.sx, lc.sy, lc.xoff, lc.yoff, pane.id())
    } else {
        format!("{}x{},{},{}", lc.sx, lc.sy, lc.xoff, lc.yoff)
    };
    if tmp.len() > CELL_MAX - 1 {
        return -1;
    }
    if buf.len() + tmp.len() >= len {
        return -1;
    }
    buf.extend_from_slice(tmp.as_bytes());

    let brackets: &[u8; 2] = match lc.type_0 {
        LAYOUT_LEFTRIGHT => b"}{",
        LAYOUT_TOPBOTTOM => b"][",
        _ => return 0,
    };
    if buf.len() + 1 >= len {
        return -1;
    }
    buf.push(brackets[1]);
    for lcchild in lc.cells.iter().map(Box::as_ref) {
        if layout_append(Some(lcchild), buf, len) != 0 {
            return -1;
        }
        if buf.len() + 1 >= len {
            return -1;
        }
        buf.push(b',');
    }
    *buf.last_mut().expect("a cell wrote its own text") = brackets[0];
    0
}

/// Whether every node's children add up to the size the node itself has.
fn layout_check(lc: &layout_cell) -> c_int {
    let leftright = match lc.type_0 {
        LAYOUT_LEFTRIGHT => true,
        LAYOUT_TOPBOTTOM => false,
        _ => return 1,
    };
    let mut n: u_int = 0;
    for lcchild in lc.cells.iter().map(Box::as_ref) {
        if leftright {
            if lcchild.sy != lc.sy {
                return 0;
            }
        } else if lcchild.sx != lc.sx {
            return 0;
        }
        if layout_check(lcchild) == 0 {
            return 0;
        }
        n = n.wrapping_add(if leftright { lcchild.sx } else { lcchild.sy }.wrapping_add(1));
    }
    let total = if leftright { lc.sx } else { lc.sy };
    (n.wrapping_sub(1) == total) as c_int
}

fn layout_parse_checksum(layout: &CStr) -> Option<u_short> {
    let prefix = layout.to_bytes().get(..5)?;
    if prefix[4] != b',' {
        return None;
    }
    let mut digits = &prefix[..4];
    while digits
        .first()
        .is_some_and(|&byte| unsafe { libc::isspace(byte as c_int) != 0 })
    {
        digits = &digits[1..];
    }
    let negative = digits.first() == Some(&b'-');
    if matches!(digits.first(), Some(b'+' | b'-')) {
        digits = &digits[1..];
    }
    if digits.starts_with(b"0x") || digits.starts_with(b"0X") {
        digits = &digits[2..];
    }
    if digits.is_empty() {
        return None;
    }
    let value = digits.iter().try_fold(0_u16, |value, &byte| {
        (byte as char)
            .to_digit(16)
            .map(|digit| (value << 4) | digit as u16)
    })?;
    Some(if negative {
        value.wrapping_neg()
    } else {
        value
    })
}

struct LayoutCursor<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> LayoutCursor<'a> {
    fn new(layout: &'a CStr, position: usize) -> Self {
        Self {
            bytes: layout.to_bytes_with_nul(),
            position,
        }
    }

    fn current(&self) -> u8 {
        self.bytes[self.position]
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn decimal(&mut self) -> Option<u64> {
        let start = self.position;
        let mut value = 0_u64;
        while self.current().is_ascii_digit() {
            value = value
                .saturating_mul(10)
                .saturating_add((self.current() - b'0') as u64);
            self.advance();
        }
        (self.position != start).then_some(value)
    }

    fn consume(&mut self, byte: u8) -> Option<()> {
        if self.current() != byte {
            return None;
        }
        self.advance();
        Some(())
    }

    fn as_cstr(&self) -> &CStr {
        CStr::from_bytes_with_nul(&self.bytes[self.position..]).unwrap()
    }
}

/// Hands the remaining panes to the leaves of `lc` in order.
unsafe fn layout_assign(
    panes: &mut std::slice::IterMut<'_, RustWindowPaneRef>,
    lc: Option<&mut layout_cell>,
    flags: c_int,
) {
    unsafe {
        let Some(lc) = lc else {
            return;
        };
        match lc.type_0 {
            LAYOUT_WINDOWPANE => {
                let Some(wp) = panes.next() else {
                    return;
                };
                layout_make_leaf(lc, wp.as_pane_mut());
                lc.flags |= flags;
            }
            LAYOUT_LEFTRIGHT | LAYOUT_TOPBOTTOM => {
                for lcchild in lc.cells.iter_mut().map(Box::as_mut) {
                    layout_assign(panes, Some(lcchild), flags);
                }
            }
            _ => {}
        }
    }
}

/// Reads one cell's geometry and optional pane ID, leaving the cursor at
/// its children or the next cell. A comma followed by another cell's width
/// remains unread so the enclosing node can consume it.
fn layout_construct_cell(
    lcparent: Option<&mut layout_cell>,
    layout: &mut LayoutCursor<'_>,
) -> Option<Box<layout_cell>> {
    let sx = layout.decimal()? as u_int;
    layout.consume(b'x')?;
    let sy = layout.decimal()? as u_int;
    layout.consume(b',')?;
    let xoff = layout.decimal()?.min(i64::MAX as u64) as c_int;
    layout.consume(b',')?;
    let yoff = layout.decimal()?.min(i64::MAX as u64) as c_int;

    if layout.current() == b',' {
        let saved = layout.position;
        layout.advance();
        while digit(layout.current() as c_char) {
            layout.advance();
        }
        if layout.current() == b'x' {
            layout.position = saved;
        }
    }

    let mut lc = layout_create_cell(lcparent);
    lc.sx = sx;
    lc.sy = sy;
    lc.xoff = xoff;
    lc.yoff = yoff;
    Some(lc)
}

/// Reads one cell and, if it opens a bracket, everything under it.
///
/// A bracket holding an empty slot — a comma with no cell, as in
/// `{,40x24,0,0}` — makes the recursive call answer success with no cell,
/// which the insert below would then link and dereference as a null pointer.
/// The `is_null` guard rejects the slot instead; 3.7b lacks it and kills the
/// server on such a layout, so this is a documented divergence matching the
/// patched oracle and tmux master (commit 97472e37).
unsafe fn layout_construct(
    lcparent: Option<&mut layout_cell>,
    layout: &mut LayoutCursor<'_>,
) -> Option<Box<layout_cell>> {
    unsafe {
        let mut lc = layout_construct_cell(lcparent, layout)?;
        let close = match layout.current() {
            b',' | b'}' | b']' | b'>' | b'\0' => return Some(lc),
            b'{' => {
                lc.type_0 = LAYOUT_LEFTRIGHT;
                b'}'
            }
            b'[' => {
                lc.type_0 = LAYOUT_TOPBOTTOM;
                b']'
            }
            _ => return None,
        };

        loop {
            layout.advance();
            let lcchild = layout_construct(Some(&mut *lc), layout)?;
            lc.cells.push(lcchild);
            if layout.current() != b',' {
                break;
            }
        }

        if layout.current() != close {
            return None;
        }
        layout.advance();
        Some(lc)
    }
}

#[cfg(test)]
#[path = "../tests/test_layout_custom.rs"]
mod tests;

impl WindowRef {
    /// The layout of `root` as the string `select-layout` takes, or `None` when it
    /// does not fit in the eight kilobytes this writes into. Any floating panes
    /// follow the tree inside angle brackets — and are written twice, since the
    /// tree they hang in carries them too.
    pub(super) fn dump_layout_cell(&self, root: Option<&layout_cell>) -> Option<CString> {
        let owner = self;

        let payload = owner.as_window();
        let w = &*payload;
        const LIMIT: usize = 8192;
        let mut buf: Vec<u8> = Vec::new();
        let mut bracket = false;

        if layout_append(root, &mut buf, LIMIT) != 0 {
            return None;
        }

        for pane in &w.z_index {
            let _id = pane.id();
            if !pane.is_alive() {
                break;
            }
            let Some((cell, _)) = layout_cell_for_pane(w.layout().root.as_deref(), pane) else {
                break;
            };
            if cell.flags & LAYOUT_CELL_FLOATING == 0 {
                break;
            }
            if !bracket {
                if buf.len() + 1 < LIMIT {
                    buf.push(b'<');
                }
                bracket = true;
            }
            if layout_append(Some(cell), &mut buf, LIMIT) != 0 {
                return None;
            }
            if buf.len() + 1 < LIMIT {
                buf.push(b',');
            }
        }
        if bracket && let Some(last) = buf.last_mut() {
            *last = b'>';
        }

        let text = CString::new(buf).expect("a layout has no NUL");
        let mut result = format!("{:04x},", layout_checksum(&text)).into_bytes();
        result.extend_from_slice(text.to_bytes());
        Some(CString::new(result).expect("a layout has no NUL"))
    }
    /// Reads `layout` into the window, answering -1 and a reason through `cause`
    /// when it will not do.
    pub unsafe fn parse_layout(&self, layout: &CStr) -> Result<(), CString> {
        let owner = self;

        unsafe {
            let csum = layout_parse_checksum(layout).ok_or_else(|| c"invalid layout".to_owned())?;
            let mut layout = LayoutCursor::new(layout, 5);
            if csum != layout_checksum(layout.as_cstr()) {
                return Err(c"invalid layout".to_owned());
            }

            let Some(tiled_lc) = layout_construct(None, &mut layout) else {
                return Err(c"invalid layout".to_owned());
            };

            owner.layout_apply(tiled_lc, layout.as_cstr())
        }
    }
    /// Puts the tree `tiled_lc` in the window, dropping the cells it has no pane
    /// for. The tree is the caller's until this answers `Ok`.
    unsafe fn layout_apply(&self, tree: Box<layout_cell>, layout: &CStr) -> Result<(), CString> {
        let owner = self;

        unsafe {
            let payload = owner.as_window();
            let w = &*payload;
            if !layout.is_empty() {
                return Err(c"invalid layout".to_owned());
            }

            let mut tree = Some(tree);
            let npanes = window_count_panes(w, 1);
            drop(payload);
            loop {
                let tiled_lc = tree
                    .as_deref_mut()
                    .ok_or_else(|| c"invalid layout".to_owned())?;
                let ncells = layout_count_cells(tiled_lc);
                if npanes > ncells {
                    return Err(xasprintf(
                        c"have %u panes but need %u",
                        fmt_args![npanes, ncells],
                    ));
                }
                if npanes == ncells {
                    break;
                }
                let lcchild = layout_find_bottomright(tiled_lc);
                owner.destroy_layout_cell(&mut tree, &lcchild);
            }

            let tiled_lc = tree
                .as_deref_mut()
                .ok_or_else(|| c"invalid layout".to_owned())?;
            let mut sx: u_int = 0;
            let mut sy: u_int = 0;
            match tiled_lc.type_0 {
                LAYOUT_LEFTRIGHT => {
                    for lcchild in tiled_lc.cells.iter().map(Box::as_ref) {
                        sy = lcchild.sy.wrapping_add(1);
                        sx = sx.wrapping_add(lcchild.sx.wrapping_add(1));
                    }
                }
                LAYOUT_TOPBOTTOM => {
                    for lcchild in tiled_lc.cells.iter().map(Box::as_ref) {
                        sx = lcchild.sx.wrapping_add(1);
                        sy = sy.wrapping_add(lcchild.sy.wrapping_add(1));
                    }
                }
                _ => {}
            }
            if tiled_lc.type_0 != LAYOUT_WINDOWPANE && (tiled_lc.sx != sx || tiled_lc.sy != sy) {
                layout_print_cell(Some(tiled_lc), c"layout_parse", 0);
                tiled_lc.sx = sx.wrapping_sub(1);
                tiled_lc.sy = sy.wrapping_sub(1);
            }

            if layout_check(tiled_lc) == 0 {
                return Err(c"size mismatch after applying layout".to_owned());
            }

            if sx != 0 && sy != 0 {
                owner.resize(tiled_lc.sx, tiled_lc.sy, -1, -1);
            }
            let mut payload = owner.as_window_mut();
            let w = &mut *payload;
            let root = w.layout_mut(LayoutAccess(())).root.take();
            layout_free_cell(root);
            w.layout_mut(LayoutAccess(())).root = tree.take();

            let (layout, panes) = w.layout_and_panes_mut(LayoutAccess(()));
            layout_assign(&mut panes.iter_mut(), layout.root.as_deref_mut(), 0);

            w.z_index.clear();
            drop(payload);
            owner.fix_layout_zindexes();
            owner.fix_layout_offsets();
            owner.fix_layout_panes(None);
            recalculate_sizes();
            let payload = owner.as_window();
            let w = &*payload;
            layout_print_cell(w.layout().root.as_deref(), c"layout_parse", 0);
            drop(payload);
            notify_window(c"window-layout-changed", Some(owner));
            Ok(())
        }
    }
}

impl WindowRef {
    pub(crate) fn dump_unzoomed_layout(&self) -> Option<CString> {
        let w = self.as_window();
        self.dump_layout_cell(w.layout().saved.as_deref().or(w.layout().root.as_deref()))
    }
}
