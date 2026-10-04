//! The scrolling strip. Every pane's rectangle derives from the pane order,
//! each pane's width preference and the window size; the Window's arrange step
//! is the only writer of pane geometry, so the arrangement holds by
//! construction.

use crate::src::shared::layout::{layout_geometry, PaneWidth};
use std::ffi::CString;

/// A half pane is half the window beside its separator column, so two halves
/// and their separator fit in the window. A full pane is the window's width.
/// Neither is narrower than `minimum`.
fn pane_width(width: PaneWidth, sx: u32, minimum: u32) -> u32 {
    match width {
        PaneWidth::Half => sx.saturating_sub(1) / 2,
        PaneWidth::Full => sx,
    }
    .max(minimum)
}

/// Rectangles of a strip whose panes have `widths`, in an `sx` by `sy`
/// window, in pane order. Each pane starts one separator column after the
/// previous one ends and uses the full height.
pub(super) fn cells(
    widths: &[PaneWidth],
    (sx, sy): (u32, u32),
    minimum: u32,
) -> Vec<layout_geometry> {
    let mut xoff = 0u32;
    widths
        .iter()
        .map(|&width| {
            let width = pane_width(width, sx, minimum);
            let cell = layout_geometry {
                sx: width,
                sy,
                xoff: xoff as i32,
                yoff: 0,
            };
            xoff = xoff.saturating_add(width + 1);
            cell
        })
        .collect()
}

/// Columns a strip whose panes have `widths` occupies, up to the last pane's
/// right edge.
pub(super) fn width(widths: &[PaneWidth], (sx, _): (u32, u32), minimum: u32) -> u32 {
    widths
        .iter()
        .fold(0u32, |columns, &width| {
            columns.saturating_add(pane_width(width, sx, minimum) + 1)
        })
        .saturating_sub(1)
}

/// One pane's entry in a layout string, copied before serialization.
pub(super) struct LayoutEntry {
    pub(super) geometry: layout_geometry,
    pub(super) id: u32,
    pub(super) index: u32,
    pub(super) active: bool,
    /// Position in the selection history, when the pane is in it.
    pub(super) last: Option<u32>,
}

fn layout_checksum(layout: &[u8]) -> u16 {
    layout.iter().fold(0u16, |checksum, &byte| {
        checksum
            .rotate_right(1)
            .wrapping_add(byte as ::core::ffi::c_char as u16)
    })
}

/// Serialize the strip as a single-row layout: a lone pane cell, or a
/// left-right node of pane cells spanning `strip`. `legacy` selects the
/// checksummed text format kept for older control clients.
pub(super) fn layout_string(
    entries: &[LayoutEntry],
    strip: (u32, u32),
    legacy: bool,
) -> Option<CString> {
    let lone = match entries {
        [] => return None,
        [entry] => Some(entry),
        _ => None,
    };
    let mut output = Vec::new();
    if legacy {
        let mut body = Vec::new();
        if let Some(entry) = lone {
            let g = entry.geometry;
            body.extend_from_slice(format!("{}x{},0,0,{}", g.sx, g.sy, entry.id).as_bytes());
        } else {
            body.extend_from_slice(format!("{}x{},0,0{{", strip.0, strip.1).as_bytes());
            for (position, entry) in entries.iter().enumerate() {
                if position != 0 {
                    body.push(b',');
                }
                let g = entry.geometry;
                body.extend_from_slice(
                    format!("{}x{},{},{},{}", g.sx, g.sy, g.xoff, g.yoff, entry.id).as_bytes(),
                );
            }
            body.push(b'}');
        }
        output.extend_from_slice(format!("{:04x},", layout_checksum(&body)).as_bytes());
        output.extend_from_slice(&body);
    } else {
        output.extend_from_slice(b"{\"V\":2,\"L\":");
        if let Some(entry) = lone {
            append_pane(&mut output, entry);
        } else {
            output.extend_from_slice(
                format!(
                    "{{\"t\":\"h\",\"w\":{},\"h\":{},\"x\":0,\"y\":0,\"c\":[",
                    strip.0, strip.1
                )
                .as_bytes(),
            );
            for (position, entry) in entries.iter().enumerate() {
                if position != 0 {
                    output.push(b',');
                }
                append_pane(&mut output, entry);
            }
            output.extend_from_slice(b"]}");
        }
        output.push(b'}');
    }
    Some(CString::new(output).expect("layout serializer produced an interior NUL"))
}

fn append_pane(output: &mut Vec<u8>, entry: &LayoutEntry) {
    let g = entry.geometry;
    output.extend_from_slice(
        format!(
            "{{\"t\":\"p\",\"w\":{},\"h\":{},\"x\":{},\"y\":{}",
            g.sx, g.sy, g.xoff, g.yoff
        )
        .as_bytes(),
    );
    if entry.active {
        output.extend_from_slice(b",\"a\":true");
    } else if let Some(last) = entry.last {
        output.extend_from_slice(format!(",\"l\":{last}").as_bytes());
    }
    output
        .extend_from_slice(format!(",\"i\":{},\"I\":\"%{}\"}}", entry.index, entry.id).as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn starts(cells: &[layout_geometry]) -> Vec<i32> {
        cells.iter().map(|cell| cell.xoff).collect()
    }

    const HALVES: [PaneWidth; 3] = [PaneWidth::Half; 3];

    #[test]
    fn panes_are_half_width_and_extend_the_strip() {
        // Odd width: two halves and their separator fill the window exactly.
        let odd = cells(&HALVES, (81, 24), 1);
        assert!(odd
            .iter()
            .all(|cell| (cell.sx, cell.sy, cell.yoff) == (40, 24, 0)));
        assert_eq!(starts(&odd), [0, 41, 82]);
        assert_eq!(width(&HALVES[..2], (81, 24), 1), 81);
        // Even width: one spare column remains at the right edge.
        let even = cells(&HALVES[..2], (80, 24), 1);
        assert_eq!((even[0].sx, starts(&even)), (39, vec![0, 40]));
        assert_eq!(width(&HALVES[..2], (80, 24), 1), 79);
        // A lone pane stays half-width.
        assert_eq!(width(&HALVES[..1], (80, 24), 1), 39);
        assert_eq!(width(&[], (80, 24), 1), 0);
    }

    #[test]
    fn full_panes_take_the_window_width_beside_halves() {
        use PaneWidth::{Full, Half};
        let mixed = cells(&[Half, Full, Half], (80, 24), 1);
        assert_eq!(
            mixed.iter().map(|cell| cell.sx).collect::<Vec<_>>(),
            [39, 80, 39]
        );
        assert_eq!(starts(&mixed), [0, 40, 121]);
        assert_eq!(width(&[Half, Full, Half], (80, 24), 1), 160);
        // A lone full pane fills the window exactly.
        assert_eq!(cells(&[Full], (81, 24), 1)[0].sx, 81);
        assert_eq!(width(&[Full], (81, 24), 1), 81);
        assert_eq!(width(&[Full, Full], (81, 24), 1), 163);
    }

    #[test]
    fn tiny_windows_raise_panes_to_the_minimum() {
        let tiny = cells(&HALVES[..2], (2, 3), 2);
        assert!(tiny.iter().all(|cell| cell.sx == 2));
        assert_eq!(starts(&tiny), [0, 3]);
        assert_eq!(width(&HALVES[..2], (2, 3), 2), 5);
        assert_eq!(cells(&HALVES[..1], (1, 1), 1)[0].sx, 1);
        assert_eq!(cells(&[PaneWidth::Full], (1, 1), 2)[0].sx, 2);
    }

    fn entry(xoff: i32, id: u32, index: u32, active: bool, last: Option<u32>) -> LayoutEntry {
        LayoutEntry {
            geometry: layout_geometry {
                sx: 39,
                sy: 24,
                xoff,
                yoff: 0,
            },
            id,
            index,
            active,
            last,
        }
    }

    #[test]
    fn layout_string_is_a_single_row() {
        let entries = [entry(0, 3, 0, false, Some(0)), entry(40, 5, 1, true, None)];
        assert_eq!(
            layout_string(&entries, (79, 24), false)
                .unwrap()
                .to_str()
                .unwrap(),
            concat!(
                r#"{"V":2,"L":{"t":"h","w":79,"h":24,"x":0,"y":0,"c":["#,
                r#"{"t":"p","w":39,"h":24,"x":0,"y":0,"l":0,"i":0,"I":"%3"},"#,
                r#"{"t":"p","w":39,"h":24,"x":40,"y":0,"a":true,"i":1,"I":"%5"}]}}"#
            )
        );
        let body = b"79x24,0,0{39x24,0,0,3,39x24,40,0,5}";
        assert_eq!(
            layout_string(&entries, (79, 24), true).unwrap().to_bytes(),
            [
                format!("{:04x},", layout_checksum(body)).as_bytes(),
                &body[..]
            ]
            .concat()
        );
        assert!(layout_string(&[], (0, 24), false).is_none());
    }

    #[test]
    fn a_lone_pane_is_the_root_cell() {
        let entries = [entry(0, 7, 1, true, None)];
        assert_eq!(
            layout_string(&entries, (39, 24), false)
                .unwrap()
                .to_str()
                .unwrap(),
            r#"{"V":2,"L":{"t":"p","w":39,"h":24,"x":0,"y":0,"a":true,"i":1,"I":"%7"}}"#
        );
        let body = b"39x24,0,0,7";
        assert_eq!(
            layout_string(&entries, (39, 24), true).unwrap().to_bytes(),
            [
                format!("{:04x},", layout_checksum(body)).as_bytes(),
                &body[..]
            ]
            .concat()
        );
    }
}
