use super::*;
use crate::tests::test_fixtures::Layout;
use crate::window_dimensions::WindowDimensionsState;

impl Layout {
    pub(in crate::layout) fn cell(
        &mut self,
        pane: usize,
    ) -> Option<crate::pane_geometry::PaneGeometry> {
        let id = unsafe { (*self.pane(pane)).pane_id() };
        let reference = self.reference();
        let w = reference.as_window();
        {
            crate::layout::layout_cell_for_pane(
                w.layout().root.as_deref(),
                &crate::window::window_pane_find_by_id(id).expect("the pane allocation exists"),
            )
            .map(|(cell, _)| crate::pane_geometry::PaneGeometry {
                sx: cell.sx,
                sy: cell.sy,
                xoff: cell.xoff,
                yoff: cell.yoff,
            })
        }
    }
}
/// One cell of a layout tree as a string, including its descendants and floating marker.
fn dump_cell(lc: Option<&layout_cell>) -> String {
    use crate::layout::{
        LAYOUT_CELL_FLOATING, LAYOUT_LEFTRIGHT, LAYOUT_TOPBOTTOM, LAYOUT_WINDOWPANE,
    };
    let Some(lc) = lc else {
        return "-".to_string();
    };
    let here = format!("{}x{}+{}+{}", lc.sx, lc.sy, lc.xoff, lc.yoff);
    let floating = if lc.flags & LAYOUT_CELL_FLOATING != 0 {
        "*"
    } else {
        ""
    };
    match lc.type_0 {
        LAYOUT_WINDOWPANE => format!(
            "%{}{floating} {here}",
            lc.wp.as_ref().map(|pane| pane.id()).unwrap_or(u_int::MAX)
        ),
        LAYOUT_LEFTRIGHT | LAYOUT_TOPBOTTOM => {
            let kids: Vec<String> = lc
                .cells
                .iter()
                .map(|child| dump_cell(Some(child)))
                .collect();
            let name = if lc.type_0 == LAYOUT_LEFTRIGHT {
                "LR"
            } else {
                "TB"
            };
            format!("{name}{floating} {here} [{}]", kids.join(" | "))
        }
        _ => format!("?{floating} {here}"),
    }
}

pub(crate) fn set_pane_floating(w: &mut window, pane_id: u_int, floating: bool) {
    use crate::layout::{
        LAYOUT_CELL_FLOATING, LAYOUT_LEFTRIGHT, LayoutCellPath, layout_create_cell,
    };

    let path = w.layout().root.as_deref().and_then(|root| {
        LayoutCellPath::for_pane(
            root,
            &crate::window::window_pane_find_by_id(pane_id).expect("the pane allocation exists"),
        )
    });
    if path.is_none() {
        let geometry = unsafe {
            w.panes
                .iter()
                .find(|pane| pane.pane_id() == pane_id)
                .unwrap()
                .as_pane()
        }
        .geometry();
        let mut cell = layout_create_cell(None);
        cell.wp = w
            .panes
            .iter()
            .find(|pane| pane.pane_id() == pane_id)
            .map(|pane| pane.downgrade());
        (cell.sx, cell.sy, cell.xoff, cell.yoff) =
            (geometry.sx, geometry.sy, geometry.xoff, geometry.yoff);
        if let Some(root) = w.layout().root.as_deref() {
            if root.wp.as_ref().map(|pane| pane.id()).is_some() {
                let mut parent = layout_create_cell(None);
                parent.type_0 = LAYOUT_LEFTRIGHT;
                let size = w.dimensions().size;
                (parent.sx, parent.sy, parent.xoff, parent.yoff) = (size.width, size.height, 0, 0);
                let mut only = w.layout_mut(LayoutAccess(())).root.replace(parent).unwrap();
                only.parent = true;
                w.layout_mut(LayoutAccess(()))
                    .root
                    .as_deref_mut()
                    .unwrap()
                    .cells
                    .push(only);
            }
            cell.parent = true;
            w.layout_mut(LayoutAccess(()))
                .root
                .as_deref_mut()
                .unwrap()
                .cells
                .push(cell);
        } else {
            w.layout_mut(LayoutAccess(())).root = Some(cell);
        }
    }
    let root = w.layout_mut(LayoutAccess(())).root.as_deref_mut().unwrap();
    let path = LayoutCellPath::for_pane(
        root,
        &crate::window::window_pane_find_by_id(pane_id).expect("the pane allocation exists"),
    )
    .unwrap();
    let cell = path.get_mut(root).unwrap();
    if floating {
        cell.flags |= LAYOUT_CELL_FLOATING;
    } else {
        cell.flags &= !LAYOUT_CELL_FLOATING;
    }
}

impl WindowRef {
    pub(crate) fn test_layout_dump(&self) -> String {
        dump_cell(self.as_window().layout().root.as_deref())
    }
}

impl window {
    pub(crate) fn test_layout_panes(
        &mut self,
        panes: &[RustWindowPaneWeak],
        split: Option<layout_type>,
    ) {
        let leaf = |pane: &RustWindowPaneWeak| {
            let mut cell = layout_create_cell(None);
            cell.wp = Some(pane.clone());
            if let Some(pane) = unsafe { pane.get() } {
                let geometry = pane.geometry();
                layout_set_size(
                    &mut cell,
                    geometry.sx,
                    geometry.sy,
                    geometry.xoff,
                    geometry.yoff,
                );
            }
            cell
        };
        let root = if let Some(split) = split {
            let mut root = layout_create_cell(None);
            root.type_0 = split;
            root.cells = panes
                .iter()
                .map(|pane| {
                    let mut cell = leaf(pane);
                    cell.parent = true;
                    cell
                })
                .collect();
            Some(root)
        } else {
            panes.first().map(leaf)
        };
        self.layout_mut(LayoutAccess(())).root = root;
    }
}

pub(crate) fn set_layout_floating(w: &mut window, pane: &RustWindowPaneWeak, floating: bool) {
    let root = w.layout_mut(LayoutAccess(())).root.as_deref_mut().unwrap();
    let path = LayoutCellPath::for_pane(root, pane).unwrap();
    let cell = path.get_mut(root).unwrap();
    if floating {
        cell.flags |= LAYOUT_CELL_FLOATING;
    } else {
        cell.flags &= !LAYOUT_CELL_FLOATING;
    }
}
