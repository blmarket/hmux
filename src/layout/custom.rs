use super::cells::{
    LAYOUT_LEFTRIGHT, LAYOUT_TOPBOTTOM, LAYOUT_WINDOWPANE, layout_count_cells, layout_create_cell,
    layout_destroy_cell, layout_fix_offsets, layout_fix_panes, layout_fix_zindexes,
    layout_free_cell, layout_make_leaf, layout_print_cell, layout_root_ptr,
};
use crate::ffi::{__ctype_b_loc, sscanf};
use crate::fmt_args;
use crate::list::foreach_owned;
use crate::notify::notify_window;
use crate::resize::recalculate_sizes;
pub use crate::types::*;
use crate::window::PaneStack;
use crate::window::{
    window_count_panes, window_pane_is_floating, window_pane_stack_first, window_pane_stack_next,
    window_panes_first, window_panes_next, window_resize,
};
use crate::xmalloc::xasprintf;
use ::core::ffi::{c_char, c_int};
use ::core::ptr::null_mut;
use ::std::ffi::CString;

pub type ctype_mask = ::core::ffi::c_uint;
pub const _ISdigit: ctype_mask = 2048;

/// How much of `layout_dump`'s buffer one cell may need.
const CELL_MAX: usize = 64;

/// Whether `b` is one of the digits the layout string is built out of, read
/// through the C library's locale table the way the C did.
fn digit(b: c_char) -> bool {
    unsafe { *(*__ctype_b_loc()).offset(b as u_char as isize) as c_int & _ISdigit as c_int != 0 }
}

/// The bottom-right leaf under `lc`, which is the one a layout with more cells
/// than panes drops first.
unsafe fn layout_find_bottomright(lc: *mut layout_cell) -> *mut layout_cell {
    unsafe {
        if (*lc).type_0 == LAYOUT_WINDOWPANE {
            return lc;
        }
        layout_find_bottomright(super::cells::last(lc))
    }
}

/// The checksum tmux writes in front of a layout: a sixteen-bit value rotated
/// right one bit per character, with the character added in.
unsafe fn layout_checksum(mut layout: *const c_char) -> u_short {
    unsafe {
        let mut csum: u_short = 0;
        while *layout != 0 {
            csum = ((csum as c_int >> 1) + ((csum as c_int & 1) << 15)) as u_short;
            csum = (csum as c_int + *layout as c_int) as u_short;
            layout = layout.offset(1);
        }
        csum
    }
}

/// The layout of `root` as the string `select-layout` takes, or `None` when it
/// does not fit in the eight kilobytes this writes into. Any floating panes
/// follow the tree inside angle brackets — and are written twice, since the
/// tree they hang in carries them too.
pub unsafe fn layout_dump(w: *mut window, root: *mut layout_cell) -> Option<CString> {
    unsafe {
        const LIMIT: usize = 8192;
        let mut buf: Vec<u8> = Vec::new();
        let mut bracket = false;

        if layout_append(root, &mut buf, LIMIT) != 0 {
            return None;
        }

        let mut wp = window_pane_stack_first(w, PaneStack::ZIndex);
        while !wp.is_null() {
            if window_pane_is_floating(wp) == 0 {
                break;
            }
            if !bracket {
                if buf.len() + 1 < LIMIT {
                    buf.push(b'<');
                }
                bracket = true;
            }
            if layout_append((*wp).layout_cell, &mut buf, LIMIT) != 0 {
                return None;
            }
            if buf.len() + 1 < LIMIT {
                buf.push(b',');
            }
            wp = window_pane_stack_next(w, PaneStack::ZIndex, wp);
        }
        if bracket && let Some(last) = buf.last_mut() {
            *last = b'>';
        }

        let text = CString::new(buf).expect("a layout has no NUL");
        Some(xasprintf(
            c"%04hx,%s".as_ptr(),
            fmt_args![layout_checksum(text.as_ptr()) as c_int, text.as_ptr()],
        ))
    }
}

/// Writes `lc` and everything under it onto the end of `buf`, answering -1 if
/// what it holds plus what is written would reach `len` bytes counting the
/// terminator, which is where the C's `strlcat` gave up.
///
/// The "one cell did not fit its own sixty-four bytes" guard is kept as the C
/// wrote it, but no test reaches it: the widest numbers a cell can carry still
/// spell out well short of that.
unsafe fn layout_append(lc: *mut layout_cell, buf: &mut Vec<u8>, len: usize) -> c_int {
    unsafe {
        if len == 0 {
            return -1;
        }
        if lc.is_null() {
            return 0;
        }

        let tmp = if let Some(id) = (*lc).wp_id {
            xasprintf(
                c"%ux%u,%d,%d,%u".as_ptr(),
                fmt_args![(*lc).sx, (*lc).sy, (*lc).xoff, (*lc).yoff, id],
            )
        } else {
            xasprintf(
                c"%ux%u,%d,%d".as_ptr(),
                fmt_args![(*lc).sx, (*lc).sy, (*lc).xoff, (*lc).yoff],
            )
        };
        if tmp.as_bytes().len() > CELL_MAX - 1 {
            return -1;
        }
        if buf.len() + tmp.as_bytes().len() >= len {
            return -1;
        }
        buf.extend_from_slice(tmp.as_bytes());

        let brackets: &[u8; 2] = match (*lc).type_0 {
            LAYOUT_LEFTRIGHT => b"}{",
            LAYOUT_TOPBOTTOM => b"][",
            _ => return 0,
        };
        if buf.len() + 1 >= len {
            return -1;
        }
        buf.push(brackets[1]);
        for lcchild in foreach_owned(&raw mut (*lc).cells) {
            if layout_append(lcchild, buf, len) != 0 {
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
}

/// Whether every node's children add up to the size the node itself has.
unsafe fn layout_check(lc: *mut layout_cell) -> c_int {
    unsafe {
        let leftright = match (*lc).type_0 {
            LAYOUT_LEFTRIGHT => true,
            LAYOUT_TOPBOTTOM => false,
            _ => return 1,
        };
        let mut n: u_int = 0;
        for lcchild in foreach_owned(&raw mut (*lc).cells) {
            if leftright {
                if (*lcchild).sy != (*lc).sy {
                    return 0;
                }
            } else if (*lcchild).sx != (*lc).sx {
                return 0;
            }
            if layout_check(lcchild) == 0 {
                return 0;
            }
            n = n.wrapping_add(
                if leftright {
                    (*lcchild).sx
                } else {
                    (*lcchild).sy
                }
                .wrapping_add(1),
            );
        }
        let total = if leftright { (*lc).sx } else { (*lc).sy };
        (n.wrapping_sub(1) == total) as c_int
    }
}

/// Reads `layout` into the window, answering -1 and a reason through `cause`
/// when it will not do.
pub unsafe fn layout_parse(w: *mut window, mut layout: *const c_char) -> Result<(), CString> {
    unsafe {
        let mut csum: u_short = 0;
        let mut n: c_int = 0;
        if sscanf(layout, c"%hx,%n".as_ptr(), &raw mut csum, &raw mut n) != 1 || n != 5 {
            return Err(c"invalid layout".to_owned());
        }
        layout = layout.offset(n as isize);
        if csum != layout_checksum(layout) {
            return Err(c"invalid layout".to_owned());
        }

        let Some(tiled_lc) = layout_construct(null_mut::<layout_cell>(), &raw mut layout) else {
            return Err(c"invalid layout".to_owned());
        };

        layout_apply(w, tiled_lc, layout)
    }
}

/// Puts the tree `tiled_lc` in the window, dropping the cells it has no pane
/// for. The tree is the caller's until this answers `Ok`.
unsafe fn layout_apply(
    w: *mut window,
    tree: Box<layout_cell>,
    layout: *const c_char,
) -> Result<(), CString> {
    unsafe {
        if *layout != 0 {
            return Err(c"invalid layout".to_owned());
        }

        let mut tree = Some(tree);
        let mut tiled_lc = layout_root_ptr(&tree);
        let npanes = window_count_panes(w, 1);
        loop {
            let ncells = layout_count_cells(tiled_lc);
            if npanes > ncells {
                return Err(xasprintf(
                    c"have %u panes but need %u".as_ptr(),
                    fmt_args![npanes, ncells],
                ));
            }
            if npanes == ncells {
                break;
            }
            let lcchild = layout_find_bottomright(tiled_lc);
            layout_destroy_cell(w, lcchild, &mut tree);
            tiled_lc = layout_root_ptr(&tree);
            if tiled_lc.is_null() {
                return Err(c"invalid layout".to_owned());
            }
        }

        let mut sx: u_int = 0;
        let mut sy: u_int = 0;
        match (*tiled_lc).type_0 {
            LAYOUT_LEFTRIGHT => {
                for lcchild in foreach_owned(&raw mut (*tiled_lc).cells) {
                    sy = (*lcchild).sy.wrapping_add(1);
                    sx = sx.wrapping_add((*lcchild).sx.wrapping_add(1));
                }
            }
            LAYOUT_TOPBOTTOM => {
                for lcchild in foreach_owned(&raw mut (*tiled_lc).cells) {
                    sx = (*lcchild).sx.wrapping_add(1);
                    sy = sy.wrapping_add((*lcchild).sy.wrapping_add(1));
                }
            }
            _ => {}
        }
        if (*tiled_lc).type_0 != LAYOUT_WINDOWPANE && ((*tiled_lc).sx != sx || (*tiled_lc).sy != sy)
        {
            layout_print_cell(tiled_lc, c"layout_parse".as_ptr(), 0);
            (*tiled_lc).sx = sx.wrapping_sub(1);
            (*tiled_lc).sy = sy.wrapping_sub(1);
        }

        if layout_check(tiled_lc) == 0 {
            return Err(c"size mismatch after applying layout".to_owned());
        }

        if sx != 0 && sy != 0 {
            window_resize(w, (*tiled_lc).sx, (*tiled_lc).sy, -1, -1);
        }
        layout_free_cell(w, (*w).layout_root.take());
        (*w).layout_root = tree.take();

        let mut wp = window_panes_first(w);
        layout_assign(w, &raw mut wp, tiled_lc, 0);

        (*w).z_index.clear();
        layout_fix_zindexes(w, tiled_lc);
        layout_fix_offsets(w);
        layout_fix_panes(w, null_mut::<window_pane>());
        recalculate_sizes();
        layout_print_cell(tiled_lc, c"layout_parse".as_ptr(), 0);
        notify_window(c"window-layout-changed".as_ptr(), w);
        Ok(())
    }
}

/// Hands the window's panes to the leaves of `lc` in order, `wp` walking the
/// window's pane list as it goes.
unsafe fn layout_assign(
    w: *mut window,
    wp: *mut *mut window_pane,
    lc: *mut layout_cell,
    flags: c_int,
) {
    unsafe {
        if lc.is_null() {
            return;
        }
        match (*lc).type_0 {
            LAYOUT_WINDOWPANE => {
                layout_make_leaf(lc, *wp);
                (*lc).flags |= flags;
                *wp = window_panes_next(w, *wp);
            }
            LAYOUT_LEFTRIGHT | LAYOUT_TOPBOTTOM => {
                for lcchild in foreach_owned(&raw mut (*lc).cells) {
                    layout_assign(w, wp, lcchild, flags);
                }
            }
            _ => {}
        }
    }
}

/// Reads one cell's size and place off the front of `layout`, leaving the
/// pointer on whatever follows. A trailing pane id is read too, unless what
/// follows it looks like another cell's size — which is how a node with
/// children is told from a leaf.
///
/// The "the size was not followed by an x" guard is kept as the C wrote it,
/// but no test reaches it: the `sscanf` above has already matched that same
/// literal `x`.
unsafe fn layout_construct_cell(
    lcparent: *mut layout_cell,
    layout: *mut *const c_char,
) -> Option<Box<layout_cell>> {
    unsafe {
        let mut sx: u_int = 0;
        let mut sy: u_int = 0;
        let mut xoff: c_int = 0;
        let mut yoff: c_int = 0;

        if !digit(**layout) {
            return None;
        }
        if sscanf(
            *layout,
            c"%ux%u,%d,%d".as_ptr(),
            &raw mut sx,
            &raw mut sy,
            &raw mut xoff,
            &raw mut yoff,
        ) != 4
        {
            return None;
        }

        while digit(**layout) {
            *layout = (*layout).offset(1);
        }
        if **layout != b'x' as c_char {
            return None;
        }
        *layout = (*layout).offset(1);
        for _ in 0..2 {
            while digit(**layout) {
                *layout = (*layout).offset(1);
            }
            if **layout != b',' as c_char {
                return None;
            }
            *layout = (*layout).offset(1);
        }
        while digit(**layout) {
            *layout = (*layout).offset(1);
        }

        if **layout == b',' as c_char {
            let saved = *layout;
            *layout = (*layout).offset(1);
            while digit(**layout) {
                *layout = (*layout).offset(1);
            }
            if **layout == b'x' as c_char {
                *layout = saved;
            }
        }

        let mut lc = layout_create_cell(lcparent);
        lc.sx = sx;
        lc.sy = sy;
        lc.xoff = xoff;
        lc.yoff = yoff;
        Some(lc)
    }
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
    lcparent: *mut layout_cell,
    layout: *mut *const c_char,
) -> Option<Box<layout_cell>> {
    unsafe {
        let mut lc = layout_construct_cell(lcparent, layout)?;
        let lc_ptr = &raw mut *lc;
        let close = match **layout as u8 {
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
            *layout = (*layout).offset(1);
            let lcchild = layout_construct(lc_ptr, layout)?;
            lc.cells.push(lcchild);
            if **layout != b',' as c_char {
                break;
            }
        }

        if **layout != close as c_char {
            return None;
        }
        *layout = (*layout).offset(1);
        Some(lc)
    }
}

#[cfg(test)]
#[path = "../tests/test_layout_custom.rs"]
mod tests;
