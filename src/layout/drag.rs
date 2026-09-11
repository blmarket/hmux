use crate::cmd::{cmd_mouse_pane, cmd_mouse_window};

pub use crate::types::*;
use crate::window::window_pane_show_scrollbar;

pub use crate::consts::{
    LAYOUT_LEFTRIGHT, LAYOUT_TOPBOTTOM, PANE_MINIMUM, PANE_SCROLLBARS_LEFT, PANE_SCROLLBARS_RIGHT,
};

unsafe fn cmd_resize_pane_mouse_update_floating(c: &mut ClientRef, m: &mouse_event) {
    unsafe {
        let mut y: core::ffi::c_int;
        let mut ly: core::ffi::c_int;

        let mut new_sx: core::ffi::c_int;
        let mut new_sy: core::ffi::c_int;
        let mut left: core::ffi::c_int;
        let mut right: core::ffi::c_int;
        let new_xoff: core::ffi::c_int;
        let new_yoff: core::ffi::c_int;
        let mut resizes: core::ffi::c_int = 0 as core::ffi::c_int;
        let Some((owner, reference)) =
            cmd_mouse_pane(m).and_then(|(_, _, pane)| Some((pane.window()?, pane)))
        else {
            c.as_tty_mut().mouse_drag_update = None;
            return;
        };
        let Some(pane) = reference.get() else {
            return;
        };
        let geometry = pane.geometry();
        let style = pane.scrollbar_style();
        let sx: core::ffi::c_int = geometry.sx as core::ffi::c_int;
        let sy: core::ffi::c_int = geometry.sy as core::ffi::c_int;
        left = geometry.xoff - 1;
        right = geometry.xoff + sx;
        if window_pane_show_scrollbar(pane, owner.scrollbar_settings().sb) != 0 {
            if owner.scrollbar_settings().sb_pos == PANE_SCROLLBARS_LEFT {
                left -= style.width + style.padding;
            } else if owner.scrollbar_settings().sb_pos == PANE_SCROLLBARS_RIGHT {
                right += style.width + style.padding;
            }
        }
        let Some(path) = owner.pane_layout_path(&reference) else {
            return;
        };
        let cell = owner
            .layout_cell_geometry(&path)
            .expect("the dragged cell path is unchanged");
        let (cell_sx, cell_sy, cell_xoff, cell_yoff) = (cell.sx, cell.sy, cell.xoff, cell.yoff);
        y = m.y.wrapping_add(m.oy) as core::ffi::c_int;
        let x: core::ffi::c_int = m.x.wrapping_add(m.ox) as core::ffi::c_int;
        if m.statusat == 0 as core::ffi::c_int && y >= m.statuslines as core::ffi::c_int {
            y = (y as u_int).wrapping_sub(m.statuslines) as core::ffi::c_int as core::ffi::c_int;
        } else if m.statusat > 0 as core::ffi::c_int && y >= m.statusat {
            y = m.statusat - 1 as core::ffi::c_int;
        }
        ly = m.ly.wrapping_add(m.oy) as core::ffi::c_int;
        let lx: core::ffi::c_int = m.lx.wrapping_add(m.ox) as core::ffi::c_int;
        if m.statusat == 0 as core::ffi::c_int && ly >= m.statuslines as core::ffi::c_int {
            ly = (ly as u_int).wrapping_sub(m.statuslines) as core::ffi::c_int as core::ffi::c_int;
        } else if m.statusat > 0 as core::ffi::c_int && ly >= m.statusat {
            ly = m.statusat - 1 as core::ffi::c_int;
        }
        if (lx == left || lx == left + 1 as core::ffi::c_int)
            && ly == geometry.yoff - 1 as core::ffi::c_int
        {
            new_sx = cell_sx.wrapping_add((lx - x) as u_int) as core::ffi::c_int;
            if new_sx < PANE_MINIMUM {
                new_sx = PANE_MINIMUM;
            }
            new_sy = cell_sy.wrapping_add((ly - y) as u_int) as core::ffi::c_int;
            if new_sy < PANE_MINIMUM {
                new_sy = PANE_MINIMUM;
            }
            new_xoff = x + 1 as core::ffi::c_int;
            new_yoff = y + 1 as core::ffi::c_int;
            owner.set_layout_cell_geometry(
                &path,
                crate::pane_geometry::PaneGeometry {
                    sx: new_sx as u_int,
                    sy: new_sy as u_int,
                    xoff: new_xoff,
                    yoff: new_yoff,
                },
            );
            resizes += 1;
        } else if (lx == right + 1 as core::ffi::c_int || lx == right)
            && ly == geometry.yoff - 1 as core::ffi::c_int
        {
            new_sx = x - cell_xoff;
            if new_sx < PANE_MINIMUM {
                new_sx = PANE_MINIMUM;
            }
            new_sy = cell_sy.wrapping_add((ly - y) as u_int) as core::ffi::c_int;
            if new_sy < PANE_MINIMUM {
                new_sy = PANE_MINIMUM;
            }
            new_yoff = y + 1 as core::ffi::c_int;
            owner.set_layout_cell_geometry(
                &path,
                crate::pane_geometry::PaneGeometry {
                    sx: new_sx as u_int,
                    sy: new_sy as u_int,
                    xoff: cell_xoff,
                    yoff: new_yoff,
                },
            );
            resizes += 1;
        } else if (lx == left || lx == left + 1 as core::ffi::c_int) && ly == geometry.yoff + sy {
            new_sx = cell_sx.wrapping_add((lx - x) as u_int) as core::ffi::c_int;
            if new_sx < PANE_MINIMUM {
                new_sx = PANE_MINIMUM;
            }
            new_sy = y - cell_yoff;
            if new_sy < PANE_MINIMUM {
                return;
            }
            new_xoff = x + 1 as core::ffi::c_int;
            owner.set_layout_cell_geometry(
                &path,
                crate::pane_geometry::PaneGeometry {
                    sx: new_sx as u_int,
                    sy: new_sy as u_int,
                    xoff: new_xoff,
                    yoff: cell_yoff,
                },
            );
            resizes += 1;
        } else if (lx == right + 1 as core::ffi::c_int || lx == right) && ly == geometry.yoff + sy {
            new_sx = x - cell_xoff;
            if new_sx < PANE_MINIMUM {
                new_sx = PANE_MINIMUM;
            }
            new_sy = y - cell_yoff;
            if new_sy < PANE_MINIMUM {
                new_sy = PANE_MINIMUM;
            }
            owner.set_layout_cell_geometry(
                &path,
                crate::pane_geometry::PaneGeometry {
                    sx: new_sx as u_int,
                    sy: new_sy as u_int,
                    xoff: cell_xoff,
                    yoff: cell_yoff,
                },
            );
            resizes += 1;
        } else if lx == right {
            new_sx = x - cell_xoff;
            if new_sx < PANE_MINIMUM {
                return;
            }
            owner.set_layout_cell_geometry(
                &path,
                crate::pane_geometry::PaneGeometry {
                    sx: new_sx as u_int,
                    sy: cell_sy,
                    xoff: cell_xoff,
                    yoff: cell_yoff,
                },
            );
            resizes += 1;
        } else if lx == left {
            new_sx = cell_sx.wrapping_add((lx - x) as u_int) as core::ffi::c_int;
            if new_sx < PANE_MINIMUM {
                return;
            }
            new_xoff = x + 1 as core::ffi::c_int;
            owner.set_layout_cell_geometry(
                &path,
                crate::pane_geometry::PaneGeometry {
                    sx: new_sx as u_int,
                    sy: cell_sy,
                    xoff: new_xoff,
                    yoff: cell_yoff,
                },
            );
            resizes += 1;
        } else if ly == geometry.yoff + sy {
            new_sy = y - cell_yoff;
            if new_sy < PANE_MINIMUM {
                return;
            }
            owner.set_layout_cell_geometry(
                &path,
                crate::pane_geometry::PaneGeometry {
                    sx: cell_sx,
                    sy: new_sy as u_int,
                    xoff: cell_xoff,
                    yoff: cell_yoff,
                },
            );
            resizes += 1;
        } else if ly == geometry.yoff - 1 as core::ffi::c_int {
            new_xoff = cell_xoff + (x - lx);
            new_yoff = y + 1 as core::ffi::c_int;
            owner.set_layout_cell_geometry(
                &path,
                crate::pane_geometry::PaneGeometry {
                    sx: cell_sx,
                    sy: cell_sy,
                    xoff: new_xoff,
                    yoff: new_yoff,
                },
            );
            resizes += 1;
        }
        if resizes != 0 as core::ffi::c_int {
            owner.fix_layout_panes(None);
            owner.redraw();
            owner.redraw_borders();
        }
    }
}
unsafe fn cmd_resize_pane_mouse_update_tiled(c: &mut ClientRef, m: &mouse_event) {
    unsafe {
        let mut y: u_int;
        let mut ly: u_int;

        static offsets: [[core::ffi::c_int; 2]; 5] = [
            [0 as core::ffi::c_int, 0 as core::ffi::c_int],
            [0 as core::ffi::c_int, 1 as core::ffi::c_int],
            [1 as core::ffi::c_int, 0 as core::ffi::c_int],
            [0 as core::ffi::c_int, -(1 as core::ffi::c_int)],
            [-(1 as core::ffi::c_int), 0 as core::ffi::c_int],
        ];
        let mut cells = Vec::new();
        let mut resizes = 0;
        let Some(owner) =
            cmd_mouse_window(m).and_then(|(_, link)| link?.get()?.window_handle().cloned())
        else {
            c.as_tty_mut().mouse_drag_update = None;
            return;
        };
        y = m.y.wrapping_add(m.oy);
        let x: u_int = m.x.wrapping_add(m.ox);
        if m.statusat == 0 as core::ffi::c_int && y >= m.statuslines {
            y = y.wrapping_sub(m.statuslines);
        } else if m.statusat > 0 as core::ffi::c_int && y >= m.statusat as u_int {
            y = (m.statusat - 1 as core::ffi::c_int) as u_int;
        }
        ly = m.ly.wrapping_add(m.oy);
        let lx: u_int = m.lx.wrapping_add(m.ox);
        if m.statusat == 0 as core::ffi::c_int && ly >= m.statuslines {
            ly = ly.wrapping_sub(m.statuslines);
        } else if m.statusat > 0 as core::ffi::c_int && ly >= m.statusat as u_int {
            ly = (m.statusat - 1 as core::ffi::c_int) as u_int;
        }
        for [dx, dy] in offsets {
            if let Some(path) =
                owner.layout_border_at(lx.wrapping_add(dx as u_int), ly.wrapping_add(dy as u_int))
                && !cells.contains(&path)
            {
                cells.push(path);
            }
        }
        for path in cells {
            let Some(type_0) = owner.layout_parent_type(&path) else {
                continue;
            };
            let change = match type_0 {
                LAYOUT_TOPBOTTOM if y != ly => y.wrapping_sub(ly) as core::ffi::c_int,
                LAYOUT_LEFTRIGHT if x != lx => x.wrapping_sub(lx) as core::ffi::c_int,
                _ => continue,
            };
            owner.resize_layout_cell(&path, type_0, change, 0);
            resizes += 1;
        }
        if resizes != 0 as u_int {
            owner.redraw();
        }
    }
}

impl ClientRef {
    /// # Safety
    /// Exclude conflicting client and pane access, including callbacks, while starting the drag.
    pub(crate) unsafe fn start_tiled_resize_drag(&mut self, mouse: &mouse_event) {
        unsafe {
            self.as_tty_mut().mouse_drag_update = Some(std::rc::Rc::new(|client, mouse| {
                cmd_resize_pane_mouse_update_tiled(
                    &mut crate::server::client_ref_of(client).expect("registered client"),
                    mouse,
                )
            }));
            cmd_resize_pane_mouse_update_tiled(self, mouse);
        }
    }
}

impl ClientRef {
    /// # Safety
    /// Exclude conflicting client and pane access, including callbacks, while starting the drag.
    pub(crate) unsafe fn start_floating_resize_drag(&mut self, mouse: &mouse_event) {
        unsafe {
            self.as_tty_mut().mouse_drag_update = Some(std::rc::Rc::new(|client, mouse| {
                cmd_resize_pane_mouse_update_floating(
                    &mut crate::server::client_ref_of(client).expect("registered client"),
                    mouse,
                )
            }));
            cmd_resize_pane_mouse_update_floating(self, mouse);
        }
    }
}
