//! Window layouts — the split tree, its tmux layout-string encoding, and the
//! resize that pushes a layout back onto the panes.
//!
//! [`LayoutCell`] mirrors tmux's `layout_cell`: a pane leaf or a row/column of
//! children, each with its own size and offset. The parser and dumper here
//! speak tmux's checksummed `bbbb,WWWxHHH,X,Y{...}` format.

use std::io;

use super::copy::reflow_copy_snapshot;
use super::target::pane_not_found;
use super::{RenderInvalidation, ServerState, Window};
use crate::server::options::WindowHook;

/// Direction used by the attach compositor for an evenly divided window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SplitDirection {
    TopBottom,
    LeftRight,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PaneRect {
    pub(crate) top: u16,
    pub(crate) left: u16,
    pub(crate) height: u16,
    pub(crate) width: u16,
}

impl PaneRect {
    fn axis(self, direction: SplitDirection) -> u16 {
        match direction {
            SplitDirection::TopBottom => self.height,
            SplitDirection::LeftRight => self.width,
        }
    }

    fn set_axis(&mut self, direction: SplitDirection, value: u16) {
        match direction {
            SplitDirection::TopBottom => self.height = value,
            SplitDirection::LeftRight => self.width = value,
        }
    }
}

/// Preserved tmux-style pane geometry. Leaves use stable pane IDs so pane-list
/// reordering cannot silently flatten the visible layout.
#[derive(Clone, Debug)]
pub(crate) enum LayoutCell {
    Pane {
        pane_id: u32,
        rect: PaneRect,
    },
    Split {
        direction: SplitDirection,
        rect: PaneRect,
        children: Vec<LayoutCell>,
    },
}

impl LayoutCell {
    pub(super) fn pane(pane_id: u32, width: u16, height: u16) -> Self {
        Self::Pane {
            pane_id,
            rect: PaneRect {
                top: 0,
                left: 0,
                height: height.max(1),
                width: width.max(1),
            },
        }
    }

    pub(super) fn even(pane_ids: &[u32], direction: SplitDirection, rect: PaneRect) -> Self {
        if pane_ids.len() == 1 {
            return Self::Pane {
                pane_id: pane_ids[0],
                rect,
            };
        }
        let total = rect
            .axis(direction)
            .saturating_sub(pane_ids.len().saturating_sub(1) as u16);
        let base = total / pane_ids.len() as u16;
        let remainder = total % pane_ids.len() as u16;
        let children = pane_ids
            .iter()
            .enumerate()
            .map(|(index, pane_id)| {
                let mut child_rect = rect;
                child_rect.set_axis(direction, base + u16::from((index as u16) < remainder));
                Self::Pane {
                    pane_id: *pane_id,
                    rect: child_rect,
                }
            })
            .collect();
        let mut layout = Self::Split {
            direction,
            rect,
            children,
        };
        layout.fix_offsets();
        layout
    }

    pub(super) fn main(
        pane_ids: &[u32],
        direction: SplitDirection,
        rect: PaneRect,
        requested_main: u16,
        requested_other: u16,
        mirrored: bool,
    ) -> Self {
        let available = rect.axis(direction).saturating_sub(1);
        let mut main = requested_main;
        let other;
        if main.saturating_add(1) >= available {
            main = if available <= 2 {
                1
            } else {
                available.saturating_sub(1)
            };
            other = 1;
        } else if requested_other == 0
            || requested_other > available
            || available - requested_other < main
        {
            other = available - main;
        } else {
            other = requested_other;
            main = available - other;
        }
        let mut main_rect = rect;
        main_rect.set_axis(direction, main);
        let mut other_rect = rect;
        other_rect.set_axis(direction, other);
        let cross = match direction {
            SplitDirection::TopBottom => SplitDirection::LeftRight,
            SplitDirection::LeftRight => SplitDirection::TopBottom,
        };
        let main_cell = Self::Pane {
            pane_id: pane_ids[0],
            rect: main_rect,
        };
        let other_cell = Self::even(&pane_ids[1..], cross, other_rect);
        let mut children = if mirrored {
            vec![other_cell, main_cell]
        } else {
            vec![main_cell, other_cell]
        };
        // Ensure the requested main dimension is retained after vector moves.
        for child in &mut children {
            let axis = child.rect().axis(direction);
            child.rect_mut().set_axis(direction, axis);
        }
        let mut layout = Self::Split {
            direction,
            rect,
            children,
        };
        layout.fix_offsets();
        layout
    }

    pub(super) fn tiled(pane_ids: &[u32], rect: PaneRect, max_columns: usize) -> Self {
        let count = pane_ids.len();
        let mut rows = 1usize;
        let mut columns = 1usize;
        while rows * columns < count {
            rows += 1;
            if rows * columns < count && (max_columns == 0 || columns < max_columns) {
                columns += 1;
            }
        }
        let base_height = rect.height.saturating_sub(rows.saturating_sub(1) as u16) / rows as u16;
        let mut offset = 0usize;
        let mut row_cells = Vec::new();
        for row in 0..rows {
            if offset == count {
                break;
            }
            let row_count = columns.min(count - offset);
            let mut row_rect = rect;
            row_rect.height = if row + 1 == rows {
                rect.height
                    .saturating_sub(base_height.saturating_add(1) * row as u16)
            } else {
                base_height
            };
            let row_layout = if row_count == 1 {
                Self::Pane {
                    pane_id: pane_ids[offset],
                    rect: row_rect,
                }
            } else {
                let base_width = rect
                    .width
                    .saturating_sub(row_count.saturating_sub(1) as u16)
                    / row_count as u16;
                let used = base_width * row_count as u16 + row_count as u16 - 1;
                let children = pane_ids[offset..offset + row_count]
                    .iter()
                    .enumerate()
                    .map(|(index, pane_id)| {
                        let mut child_rect = row_rect;
                        child_rect.width = base_width
                            + if index + 1 == row_count {
                                rect.width.saturating_sub(used)
                            } else {
                                0
                            };
                        Self::Pane {
                            pane_id: *pane_id,
                            rect: child_rect,
                        }
                    })
                    .collect();
                let mut row_layout = Self::Split {
                    direction: SplitDirection::LeftRight,
                    rect: row_rect,
                    children,
                };
                row_layout.fix_offsets();
                row_layout
            };
            row_cells.push(row_layout);
            offset += row_count;
        }
        let mut layout = Self::Split {
            direction: SplitDirection::TopBottom,
            rect,
            children: row_cells,
        };
        layout.fix_offsets();
        layout
    }

    pub(super) fn rect(&self) -> PaneRect {
        match self {
            Self::Pane { rect, .. } | Self::Split { rect, .. } => *rect,
        }
    }

    fn rect_mut(&mut self) -> &mut PaneRect {
        match self {
            Self::Pane { rect, .. } | Self::Split { rect, .. } => rect,
        }
    }

    fn shift(&mut self, dx: i32, dy: i32) {
        let rect = self.rect_mut();
        rect.left = (rect.left as i32 + dx).max(0) as u16;
        rect.top = (rect.top as i32 + dy).max(0) as u16;
        if let Self::Split { children, .. } = self {
            for child in children {
                child.shift(dx, dy);
            }
        }
    }

    fn set_rect(&mut self, rect: PaneRect) {
        let old = self.rect();
        self.resize(rect.width, rect.height);
        self.shift(
            rect.left as i32 - old.left as i32,
            rect.top as i32 - old.top as i32,
        );
    }

    fn contains(&self, pane_id: u32) -> bool {
        match self {
            Self::Pane { pane_id: id, .. } => *id == pane_id,
            Self::Split { children, .. } => children.iter().any(|child| child.contains(pane_id)),
        }
    }

    pub(crate) fn pane_rect(&self, pane_id: u32) -> Option<PaneRect> {
        match self {
            Self::Pane { pane_id: id, rect } => (*id == pane_id).then_some(*rect),
            Self::Split { children, .. } => {
                children.iter().find_map(|child| child.pane_rect(pane_id))
            }
        }
    }

    pub(crate) fn panes(&self) -> Vec<(u32, PaneRect)> {
        let mut panes = Vec::new();
        self.collect_panes(&mut panes);
        panes
    }

    pub(super) fn neighbour(
        &self,
        pane_id: u32,
        direction: SplitDirection,
        forward: bool,
    ) -> Option<u32> {
        let current = self.pane_rect(pane_id)?;
        self.panes()
            .into_iter()
            .filter(|(id, rect)| {
                if *id == pane_id {
                    return false;
                }
                let overlaps = match direction {
                    SplitDirection::LeftRight => {
                        rect.top < current.top.saturating_add(current.height)
                            && current.top < rect.top.saturating_add(rect.height)
                    }
                    SplitDirection::TopBottom => {
                        rect.left < current.left.saturating_add(current.width)
                            && current.left < rect.left.saturating_add(rect.width)
                    }
                };
                let ahead = match (direction, forward) {
                    (SplitDirection::LeftRight, true) => rect.left > current.left,
                    (SplitDirection::LeftRight, false) => rect.left < current.left,
                    (SplitDirection::TopBottom, true) => rect.top > current.top,
                    (SplitDirection::TopBottom, false) => rect.top < current.top,
                };
                overlaps && ahead
            })
            .min_by_key(|(_, rect)| match direction {
                SplitDirection::LeftRight => rect.left.abs_diff(current.left),
                SplitDirection::TopBottom => rect.top.abs_diff(current.top),
            })
            .map(|(id, _)| id)
    }

    fn collect_panes(&self, panes: &mut Vec<(u32, PaneRect)>) {
        match self {
            Self::Pane { pane_id, rect } => panes.push((*pane_id, *rect)),
            Self::Split { children, .. } => {
                for child in children {
                    child.collect_panes(panes);
                }
            }
        }
    }

    #[allow(dead_code)]
    pub(super) fn split(
        &mut self,
        target_id: u32,
        new_id: u32,
        direction: SplitDirection,
        before: bool,
    ) -> bool {
        self.split_sized(target_id, new_id, direction, before, None)
    }

    /// [`Self::split`] with an explicit size for the *new* pane on the split
    /// axis (`split-window -l`/`-p`); `None` keeps the even split.
    pub(super) fn split_sized(
        &mut self,
        target_id: u32,
        new_id: u32,
        direction: SplitDirection,
        before: bool,
        new_size: Option<u16>,
    ) -> bool {
        if matches!(self, Self::Pane { pane_id, .. } if *pane_id == target_id) {
            let old = self.rect();
            let (first, second) = split_axis_sized(old.axis(direction), new_size, before);
            let mut old_rect = old;
            old_rect.set_axis(direction, if before { second } else { first });
            let mut new_rect = old;
            new_rect.set_axis(direction, if before { first } else { second });
            let old_leaf = Self::Pane {
                pane_id: target_id,
                rect: old_rect,
            };
            let new_leaf = Self::Pane {
                pane_id: new_id,
                rect: new_rect,
            };
            *self = Self::Split {
                direction,
                rect: old,
                children: if before {
                    vec![new_leaf, old_leaf]
                } else {
                    vec![old_leaf, new_leaf]
                },
            };
            self.fix_offsets();
            return true;
        }

        let Self::Split {
            direction: parent_direction,
            children,
            ..
        } = self
        else {
            return false;
        };
        let Some(index) = children.iter().position(|child| child.contains(target_id)) else {
            return false;
        };
        if *parent_direction == direction
            && matches!(children[index], Self::Pane { pane_id, .. } if pane_id == target_id)
        {
            let old = children[index].rect();
            let (first, second) = split_axis_sized(old.axis(direction), new_size, before);
            let mut old_rect = old;
            old_rect.set_axis(direction, if before { second } else { first });
            let mut new_rect = old;
            new_rect.set_axis(direction, if before { first } else { second });
            children[index] = Self::Pane {
                pane_id: target_id,
                rect: old_rect,
            };
            children.insert(
                if before { index } else { index + 1 },
                Self::Pane {
                    pane_id: new_id,
                    rect: new_rect,
                },
            );
            self.fix_offsets();
            true
        } else {
            let changed =
                children[index].split_sized(target_id, new_id, direction, before, new_size);
            if changed {
                self.fix_offsets();
            }
            changed
        }
    }

    /// Full-window split: split the entire layout tree along `direction`
    /// rather than a single pane leaf (`split-window -f`).
    pub(super) fn split_full(
        &mut self,
        new_id: u32,
        direction: SplitDirection,
        before: bool,
        new_size: Option<u16>,
    ) -> bool {
        let total = self.rect();
        let (first, second) = split_axis_sized(total.axis(direction), new_size, before);
        let old_size = if before { second } else { first };
        let new_pane_size = if before { first } else { second };

        // A root that already splits along this axis takes the new cell as another
        // child, so the panes it holds are squeezed into the old layout's share
        // *before* the cell is inserted, the way tmux resizes the root's children
        // ahead of the insertion.
        if matches!(self, Self::Split { direction: dir, .. } if *dir == direction) {
            let mut shrunk = total;
            shrunk.set_axis(direction, old_size);
            self.resize(shrunk.width, shrunk.height);
            let mut new_rect = total;
            new_rect.set_axis(direction, new_pane_size);
            let new_leaf = Self::Pane {
                pane_id: new_id,
                rect: new_rect,
            };
            let Self::Split { children, rect, .. } = self else {
                return false;
            };
            let insert_at = if before { 0 } else { children.len() };
            children.insert(insert_at, new_leaf);
            *rect = total;
            self.fix_offsets();
            return true;
        }
        let mut old = self.clone();
        let mut old_target_rect = total;
        old_target_rect.set_axis(direction, old_size);
        old.resize(old_target_rect.width, old_target_rect.height);

        let mut new_rect = total;
        new_rect.set_axis(direction, new_pane_size);
        let new_leaf = Self::Pane {
            pane_id: new_id,
            rect: new_rect,
        };
        *self = Self::Split {
            direction,
            rect: total,
            children: if before {
                vec![new_leaf, old]
            } else {
                vec![old, new_leaf]
            },
        };
        self.fix_offsets();
        true
    }

    /// tmux's `layout_spread_out`: climb from the pane's innermost parent
    /// outward until one level's children can be evened out.
    pub(super) fn spread_pane(&mut self, pane_id: u32) -> bool {
        let Self::Split { children, .. } = self else {
            return false;
        };
        let Some(index) = children.iter().position(|child| child.contains(pane_id)) else {
            return false;
        };
        if children[index].spread_pane(pane_id) {
            return true;
        }
        self.spread_cell()
    }

    /// tmux's `layout_spread_cell`: give every child `(size - borders) /
    /// children` cells on the split axis, the first children absorbing the
    /// remainder.
    fn spread_cell(&mut self) -> bool {
        let Self::Split {
            direction,
            children,
            rect,
        } = self
        else {
            return false;
        };
        let direction = *direction;
        let number = children.len() as u16;
        if number <= 1 {
            return false;
        }
        let size = rect.axis(direction);
        if size < number - 1 {
            return false;
        }
        let each = (size - (number - 1)) / number;
        if each == 0 {
            return false;
        }
        let mut remainder = i32::from(size) - i32::from(number) * (i32::from(each) + 1) + 1;
        let mut changed = false;
        for child in children.iter_mut() {
            let mut target = each;
            if remainder > 0 {
                target += 1;
                remainder -= 1;
            }
            let change = i32::from(target) - i32::from(child.rect().axis(direction));
            if change != 0 {
                changed = true;
            }
            child.resize_axis(direction, change);
        }
        if changed {
            self.fix_offsets();
        }
        changed
    }

    pub(super) fn resize(&mut self, width: u16, height: u16) {
        let old = self.rect();
        self.resize_axis(
            SplitDirection::LeftRight,
            width.max(1) as i32 - old.width as i32,
        );
        self.resize_axis(
            SplitDirection::TopBottom,
            height.max(1) as i32 - old.height as i32,
        );
        self.fix_offsets();
    }

    pub(super) fn resize_pane_toward(
        &mut self,
        pane_id: u32,
        direction: SplitDirection,
        forward: bool,
        amount: u16,
    ) -> bool {
        let Self::Split {
            direction: own,
            children,
            ..
        } = self
        else {
            return false;
        };
        let Some(index) = children.iter().position(|child| child.contains(pane_id)) else {
            return false;
        };
        if children[index].resize_pane_toward(pane_id, direction, forward, amount) {
            self.fix_offsets();
            return true;
        }
        if *own != direction {
            return false;
        }
        let donor = if forward {
            index.checked_add(1).filter(|next| *next < children.len())
        } else {
            index.checked_sub(1)
        };
        let Some(donor) = donor else {
            return false;
        };
        let amount = amount.min(children[donor].available(direction));
        if amount == 0 {
            return false;
        }
        children[index].adjust(direction, i32::from(amount));
        children[donor].adjust(direction, -i32::from(amount));
        self.fix_offsets();
        true
    }

    pub(super) fn resize_pane_to(
        &mut self,
        pane_id: u32,
        direction: SplitDirection,
        requested: u16,
    ) -> bool {
        let Self::Split {
            direction: own,
            children,
            ..
        } = self
        else {
            return false;
        };
        let Some(index) = children.iter().position(|child| child.contains(pane_id)) else {
            return false;
        };
        if children[index].resize_pane_to(pane_id, direction, requested) {
            self.fix_offsets();
            return true;
        }
        if *own != direction || children.len() < 2 {
            return false;
        }
        let current = children[index].rect().axis(direction);
        let mut delta = i32::from(requested) - i32::from(current);
        let receiver = if index + 1 < children.len() {
            index + 1
        } else {
            index - 1
        };
        if delta > 0 {
            delta = delta.min(i32::from(children[receiver].available(direction)));
        } else {
            delta = delta.max(-i32::from(children[index].available(direction)));
        }
        if delta == 0 {
            return true;
        }
        children[index].adjust(direction, delta);
        children[receiver].adjust(direction, -delta);
        self.fix_offsets();
        true
    }

    fn resize_axis(&mut self, direction: SplitDirection, mut change: i32) {
        if change < 0 {
            change = change.max(-(self.available(direction) as i32));
        }
        if change != 0 {
            self.adjust(direction, change);
        }
    }

    fn available(&self, direction: SplitDirection) -> u16 {
        match self {
            Self::Pane { rect, .. } => rect.axis(direction).saturating_sub(1),
            Self::Split {
                direction: own,
                children,
                ..
            } if *own == direction => children
                .iter()
                .map(|child| child.available(direction))
                .sum(),
            Self::Split { children, .. } => children
                .iter()
                .map(|child| child.available(direction))
                .min()
                .unwrap_or(0),
        }
    }

    fn adjust(&mut self, direction: SplitDirection, change: i32) {
        let rect = self.rect_mut();
        let current = rect.axis(direction) as i32;
        rect.set_axis(direction, (current + change).max(1) as u16);
        let Self::Split {
            direction: own,
            children,
            ..
        } = self
        else {
            return;
        };
        if *own != direction {
            for child in children {
                child.adjust(direction, change);
            }
            return;
        }
        let step = change.signum();
        let mut remaining = change.unsigned_abs();
        while remaining > 0 {
            let mut progressed = false;
            for child in children.iter_mut() {
                if remaining == 0 {
                    break;
                }
                if step > 0 || child.available(direction) > 0 {
                    child.adjust(direction, step);
                    remaining -= 1;
                    progressed = true;
                }
            }
            if !progressed {
                break;
            }
        }
    }

    fn fix_offsets(&mut self) {
        let Self::Split {
            direction,
            rect,
            children,
        } = self
        else {
            return;
        };
        let mut left = rect.left;
        let mut top = rect.top;
        for child in children {
            let child_rect = child.rect_mut();
            child_rect.left = left;
            child_rect.top = top;
            match direction {
                SplitDirection::LeftRight => {
                    child_rect.height = rect.height;
                    left = left.saturating_add(child_rect.width).saturating_add(1);
                }
                SplitDirection::TopBottom => {
                    child_rect.width = rect.width;
                    top = top.saturating_add(child_rect.height).saturating_add(1);
                }
            }
            child.fix_offsets();
        }
    }

    pub(super) fn remove(&mut self, pane_id: u32) -> bool {
        let Self::Split {
            direction,
            rect,
            children,
        } = self
        else {
            return false;
        };
        if let Some(index) = children
            .iter()
            .position(|child| matches!(child, Self::Pane { pane_id: id, .. } if *id == pane_id))
        {
            let removed = children.remove(index);
            if children.is_empty() {
                return true;
            }
            let receiver = if index == 0 { 0 } else { index - 1 };
            children[receiver].adjust(*direction, removed.rect().axis(*direction) as i32 + 1);
            if children.len() == 1 {
                let mut survivor = children.remove(0);
                survivor.set_rect(*rect);
                *self = survivor;
            } else {
                self.fix_offsets();
            }
            return true;
        }
        if let Some(child) = children.iter_mut().find(|child| child.contains(pane_id)) {
            let removed = child.remove(pane_id);
            if removed {
                self.fix_offsets();
            }
            return removed;
        }
        false
    }

    pub(super) fn keep_only(&mut self, pane_id: u32) {
        let rect = self.rect();
        *self = Self::Pane { pane_id, rect };
    }

    pub(super) fn replace_pane(&mut self, old: u32, new: u32) -> bool {
        match self {
            Self::Pane { pane_id, .. } if *pane_id == old => {
                *pane_id = new;
                true
            }
            Self::Pane { .. } => false,
            Self::Split { children, .. } => children
                .iter_mut()
                .any(|child| child.replace_pane(old, new)),
        }
    }

    pub(super) fn swap_panes(&mut self, first: u32, second: u32) {
        if first == second {
            return;
        }
        self.replace_pane(first, u32::MAX);
        self.replace_pane(second, first);
        self.replace_pane(u32::MAX, second);
    }

    pub(super) fn assign_panes(&mut self, pane_ids: &mut impl Iterator<Item = u32>) {
        match self {
            Self::Pane { pane_id, .. } => {
                if let Some(id) = pane_ids.next() {
                    *pane_id = id;
                }
            }
            Self::Split { children, .. } => {
                for child in children {
                    child.assign_panes(pane_ids);
                }
            }
        }
    }

    pub(crate) fn dump(&self) -> String {
        match self {
            Self::Pane { pane_id, rect } => format!(
                "{}x{},{},{},{}",
                rect.width, rect.height, rect.left, rect.top, pane_id
            ),
            Self::Split {
                direction,
                rect,
                children,
            } => {
                let (open, close) = match direction {
                    SplitDirection::TopBottom => ('[', ']'),
                    SplitDirection::LeftRight => ('{', '}'),
                };
                format!(
                    "{}x{},{},{}{}{}{}",
                    rect.width,
                    rect.height,
                    rect.left,
                    rect.top,
                    open,
                    children
                        .iter()
                        .map(Self::dump)
                        .collect::<Vec<_>>()
                        .join(","),
                    close
                )
            }
        }
    }
}

fn split_axis(size: u16) -> (u16, u16) {
    let second = ((size.saturating_add(1)) / 2)
        .saturating_sub(1)
        .clamp(1, size.saturating_sub(2).max(1));
    (size.saturating_sub(second).saturating_sub(1).max(1), second)
}

/// [`split_axis`] with the *new* pane pinned at `new_size` cells; the border
/// line takes one cell and the target keeps the rest. The new pane is the
/// first child when the split inserts before, the second otherwise.
fn split_axis_sized(size: u16, new_size: Option<u16>, before: bool) -> (u16, u16) {
    let Some(new) = new_size else {
        return split_axis(size);
    };
    let new = new.clamp(1, size.saturating_sub(2).max(1));
    let old = size.saturating_sub(new).saturating_sub(1).max(1);
    if before {
        (new, old)
    } else {
        (old, new)
    }
}

/// The layout as a `select-layout`-consumable custom string: the dump body
/// behind tmux's 4-hex rotate-and-add checksum prefix.
pub(crate) fn checksummed_layout_dump(layout: &LayoutCell) -> String {
    let body = layout.dump();
    let checksum = body.bytes().fold(0u16, |sum, byte| {
        sum.rotate_right(1).wrapping_add(u16::from(byte))
    });
    format!("{checksum:04x},{body}")
}

pub(super) fn parse_custom_layout(value: &str) -> io::Result<LayoutCell> {
    let (checksum, body) = value
        .split_once(',')
        .ok_or_else(|| io::Error::other("invalid layout"))?;
    let checksum =
        u16::from_str_radix(checksum, 16).map_err(|_| io::Error::other("invalid layout"))?;
    let actual = body.bytes().fold(0u16, |sum, byte| {
        sum.rotate_right(1).wrapping_add(u16::from(byte))
    });
    if checksum != actual {
        return Err(io::Error::other("invalid layout"));
    }
    let mut parser = LayoutParser {
        bytes: body.as_bytes(),
        offset: 0,
    };
    let layout = parser.cell()?;
    if parser.offset != parser.bytes.len() {
        return Err(io::Error::other("invalid layout"));
    }
    Ok(layout)
}

struct LayoutParser<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl LayoutParser<'_> {
    fn number(&mut self) -> io::Result<u16> {
        let start = self.offset;
        while self.bytes.get(self.offset).is_some_and(u8::is_ascii_digit) {
            self.offset += 1;
        }
        if start == self.offset {
            return Err(io::Error::other("invalid layout"));
        }
        std::str::from_utf8(&self.bytes[start..self.offset])
            .ok()
            .and_then(|value| value.parse().ok())
            .ok_or_else(|| io::Error::other("invalid layout"))
    }

    fn expect(&mut self, byte: u8) -> io::Result<()> {
        if self.bytes.get(self.offset) != Some(&byte) {
            return Err(io::Error::other("invalid layout"));
        }
        self.offset += 1;
        Ok(())
    }

    fn cell(&mut self) -> io::Result<LayoutCell> {
        let width = self.number()?;
        self.expect(b'x')?;
        let height = self.number()?;
        self.expect(b',')?;
        let left = self.number()?;
        self.expect(b',')?;
        let top = self.number()?;
        let rect = PaneRect {
            top,
            left,
            height,
            width,
        };
        match self.bytes.get(self.offset).copied() {
            Some(b',') => {
                self.offset += 1;
                let pane_id = self.number()?;
                Ok(LayoutCell::Pane {
                    pane_id: u32::from(pane_id),
                    rect,
                })
            }
            Some(open @ (b'[' | b'{')) => {
                self.offset += 1;
                let close = if open == b'[' { b']' } else { b'}' };
                let direction = if open == b'[' {
                    SplitDirection::TopBottom
                } else {
                    SplitDirection::LeftRight
                };
                let mut children = Vec::new();
                loop {
                    children.push(self.cell()?);
                    match self.bytes.get(self.offset).copied() {
                        Some(byte) if byte == close => {
                            self.offset += 1;
                            break;
                        }
                        Some(b',') => self.offset += 1,
                        _ => return Err(io::Error::other("invalid layout")),
                    }
                }
                if children.is_empty() {
                    return Err(io::Error::other("invalid layout"));
                }
                Ok(LayoutCell::Split {
                    direction,
                    rect,
                    children,
                })
            }
            _ => Err(io::Error::other("invalid layout")),
        }
    }
}

/// tmux's `WINDOW_MINIMUM`/`WINDOW_MAXIMUM`.
pub(super) const WINDOW_MINIMUM: u16 = 1;

pub(super) const WINDOW_MAXIMUM: u16 = 10_000;

pub(super) fn resize_panes_to_layout(window: &mut Window) -> io::Result<()> {
    // The grid has to match the rect the pane is drawn in — `Window::pane_rect`
    // takes the scrollbar columns and the border-status row off the layout cell
    // — or the compositor dumps more rows than the pane has on screen and the
    // reserved row never fits in the frame.
    let rects = window
        .panes
        .iter()
        .map(|pane| window.pane_rect(pane.id))
        .collect::<Vec<_>>();
    for (pane, rect) in window.panes.iter_mut().zip(rects) {
        if let Some(rect) = rect {
            if let Some(copy) = pane.copy.as_mut() {
                reflow_copy_snapshot(copy, rect.width.max(1), rect.height.max(1))?;
            }
            pane.pane.resize(rect.width.max(1), rect.height.max(1))?;
        }
    }
    Ok(())
}

impl ServerState {
    pub(crate) fn select_named_layout(&mut self, target: &str, layout: usize) -> io::Result<()> {
        let resolved = self.resolve_window(target)?;
        let main_height = self
            .option_for_target(target, "main-pane-height")
            .and_then(|value| value.parse().ok())
            .unwrap_or(24);
        let main_width = self
            .option_for_target(target, "main-pane-width")
            .and_then(|value| value.parse().ok())
            .unwrap_or(80);
        let other_height = self
            .option_for_target(target, "other-pane-height")
            .and_then(|value| value.parse().ok())
            .unwrap_or(0);
        let other_width = self
            .option_for_target(target, "other-pane-width")
            .and_then(|value| value.parse().ok())
            .unwrap_or(0);
        let max_columns = self
            .option_for_target(target, "tiled-layout-max-columns")
            .and_then(|value| value.parse().ok())
            .unwrap_or(0);
        let window_id = self.sessions[resolved.session].windows[resolved.window].id;
        let affected_sessions = self
            .sessions
            .iter()
            .filter(|session| session.windows.iter().any(|link| link.id == window_id))
            .map(|session| session.id)
            .collect::<Vec<_>>();
        let window = self.window_mut(resolved.session, resolved.window);
        let pane_ids = window
            .panes
            .iter()
            .filter(|pane| pane.floating.is_none())
            .map(|pane| pane.id)
            .collect::<Vec<_>>();
        if pane_ids.len() > 1 {
            let rect = window.layout.rect();
            window.layout = match layout {
                0 => LayoutCell::even(&pane_ids, SplitDirection::LeftRight, rect),
                1 => LayoutCell::even(&pane_ids, SplitDirection::TopBottom, rect),
                2 => LayoutCell::main(
                    &pane_ids,
                    SplitDirection::TopBottom,
                    rect,
                    main_height,
                    other_height,
                    false,
                ),
                3 => LayoutCell::main(
                    &pane_ids,
                    SplitDirection::TopBottom,
                    rect,
                    main_height,
                    other_height,
                    true,
                ),
                4 => LayoutCell::main(
                    &pane_ids,
                    SplitDirection::LeftRight,
                    rect,
                    main_width,
                    other_width,
                    false,
                ),
                5 => LayoutCell::main(
                    &pane_ids,
                    SplitDirection::LeftRight,
                    rect,
                    main_width,
                    other_width,
                    true,
                ),
                6 => LayoutCell::tiled(&pane_ids, rect, max_columns),
                _ => return Err(io::Error::other("invalid layout")),
            };
            resize_panes_to_layout(window)?;
        }
        window.last_layout = Some(layout);
        window.zoomed = false;
        for session_id in affected_sessions {
            self.invalidate_session(
                session_id,
                RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
            );
        }
        // tmux notifies twice for a named layout: once from `layout_set_*` when
        // the cells are rebuilt, and once from the command itself.
        self.notify_window(WindowHook::LayoutChanged, window_id);
        self.notify_window(WindowHook::LayoutChanged, window_id);
        Ok(())
    }

    /// Snapshot the target window's layout into tmux's `w->old_layout` slot,
    /// returning the value it replaces — what `select-layout -o` restores.
    pub(crate) fn snapshot_window_layout(&mut self, target: &str) -> io::Result<Option<String>> {
        let resolved = self.resolve_window_target(target)?;
        let window = self.window_mut(resolved.session, resolved.window);
        let dump = checksummed_layout_dump(&window.layout);
        Ok(window.old_layout.replace(dump))
    }

    /// Put a previously snapshot `old_layout` back, for a failed
    /// `select-layout` (tmux's error path restores `w->old_layout`).
    pub(crate) fn restore_window_old_layout(&mut self, target: &str, value: Option<String>) {
        if let Ok(resolved) = self.resolve_window_target(target) {
            self.window_mut(resolved.session, resolved.window)
                .old_layout = value;
        }
    }

    /// The last preset layout applied to the target window, which a bare
    /// `select-layout` reapplies (tmux's `w->lastlayout`).
    pub(crate) fn window_last_preset_layout(&self, target: &str) -> io::Result<Option<usize>> {
        let resolved = self.resolve_window_target(target)?;
        Ok(self.window(resolved.session, resolved.window).last_layout)
    }

    /// `select-layout -E`: spread the target pane's siblings evenly, climbing
    /// outward from its innermost split (tmux's `layout_spread_out`).
    pub(crate) fn spread_window_layout(&mut self, target: &str) -> io::Result<()> {
        let t = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let window_id = self.sessions[t.session].windows[t.window].id;
        let affected_sessions = self
            .sessions
            .iter()
            .filter(|session| session.windows.iter().any(|link| link.id == window_id))
            .map(|session| session.id)
            .collect::<Vec<_>>();
        let win = self.window_mut(t.session, t.window);
        let pane_id = win.panes[t.pane].id;
        if win.layout.spread_pane(pane_id) {
            resize_panes_to_layout(win)?;
            for session_id in affected_sessions {
                self.invalidate_session(
                    session_id,
                    RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
                );
            }
            self.notify_window(WindowHook::LayoutChanged, window_id);
        }
        Ok(())
    }

    pub(crate) fn cycle_layout(&mut self, target: &str, forward: bool) -> io::Result<()> {
        let resolved = self.resolve_window_target(target)?;
        let current = self.window(resolved.session, resolved.window).last_layout;
        let next = match (current, forward) {
            (None, true) => 0,
            (None, false) => 6,
            (Some(6), true) => 0,
            (Some(0), false) => 6,
            (Some(value), true) => value + 1,
            (Some(value), false) => value - 1,
        };
        self.select_named_layout(target, next)
    }

    pub(crate) fn select_custom_layout(&mut self, target: &str, value: &str) -> io::Result<()> {
        let mut layout = parse_custom_layout(value)?;
        let resolved = self.resolve_window(target)?;
        let window_id = self.sessions[resolved.session].windows[resolved.window].id;
        let affected_sessions = self
            .sessions
            .iter()
            .filter(|session| session.windows.iter().any(|link| link.id == window_id))
            .map(|session| session.id)
            .collect::<Vec<_>>();
        let pane_ids = self
            .window(resolved.session, resolved.window)
            .panes
            .iter()
            .filter(|pane| pane.floating.is_none())
            .map(|pane| pane.id)
            .collect::<Vec<_>>();
        if layout.panes().len() != pane_ids.len() {
            return Err(io::Error::other(format!(
                "have {} panes but need {}",
                pane_ids.len(),
                layout.panes().len()
            )));
        }
        layout.assign_panes(&mut pane_ids.into_iter());
        let size = (layout.rect().width, layout.rect().height);
        let window = self.window_mut(resolved.session, resolved.window);
        window.layout = layout;
        // tmux's `layout_parse` resizes the window to the layout it just parsed,
        // then runs `recalculate_sizes` — so on an attached window the client
        // set wins straight back, and only a clientless one keeps this size.
        window.cols = size.0;
        window.rows = size.1;
        window.last_layout = None;
        window.zoomed = false;
        resize_panes_to_layout(window)?;
        for session_id in affected_sessions {
            self.invalidate_session(
                session_id,
                RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
            );
        }
        self.notify_window(WindowHook::LayoutChanged, window_id);
        Ok(())
    }
}
