//! Images anchored to a screen's cells: tmux's `image.c`.
//!
//! A sixel image is not made of cells, so it cannot live in the grid. It lives
//! beside the grid instead, anchored at the cursor cell it was written at and
//! sized in cells, and every operation that disturbs the cells it covers throws
//! it away — there is no way to redraw part of an image, so tmux does not try.
//!
//! # The cap is server-wide
//!
//! tmux caps the images *the whole server* holds at
//! [`MAX_IMAGE_COUNT`](self::MAX_IMAGE_COUNT), oldest evicted first, so a pane
//! that floods images pushes another pane's image out. That is a process-wide
//! registry inside the engine, which is what [`registry`] is, and it is
//! deliberate: the alternative — a cap per screen — is not observationally
//! equivalent, and the eviction is reachable from any two panes in one session.
//!
//! One difference remains and cannot be closed from here. tmux does not mark
//! the evicted image's pane for redraw, so that pane keeps showing an image the
//! server has forgotten until something else redraws it; hmux recomposes a
//! client's frame from the live image list, so the evicted image stops being
//! drawn at the next frame. hmux is the stricter of the two, and the window
//! where they differ is one frame.

use std::collections::{HashSet, VecDeque};
use std::sync::{Arc, Mutex, OnceLock};

use crate::sixel::SixelImage;

/// tmux's `MAX_IMAGE_COUNT`.
///
/// tmux evicts when the count *reaches* this after an insert, not when it would
/// pass it, so nineteen is what a server actually holds.
const MAX_IMAGE_COUNT: usize = 20;

/// The server-wide insertion order tmux keeps in `all_images`.
///
/// Only the identities live here. The images themselves stay with the screen
/// that owns them, which is what lets a screen drop its own without telling
/// anyone; a screen learns that one of *its* images was evicted by another
/// screen's write the next time it looks, through [`Images::prune`].
mod registry {
    use super::{HashSet, Mutex, OnceLock, VecDeque, MAX_IMAGE_COUNT};

    #[derive(Default)]
    struct Registry {
        next_id: u64,
        order: VecDeque<u64>,
        live: HashSet<u64>,
        /// How many images the cap has pushed out, ever. A screen folds this
        /// into its own revision so that an eviction it did not perform still
        /// reads as a change to whatever is drawing its images.
        evictions: u64,
    }

    fn registry() -> &'static Mutex<Registry> {
        static REGISTRY: OnceLock<Mutex<Registry>> = OnceLock::new();
        REGISTRY.get_or_init(|| Mutex::new(Registry::default()))
    }

    fn lock() -> std::sync::MutexGuard<'static, Registry> {
        // A poisoned registry is still a consistent one: every operation here
        // either completes or leaves the maps untouched.
        registry().lock().unwrap_or_else(|error| error.into_inner())
    }

    /// tmux's `TAILQ_INSERT_TAIL(&all_images, …)` and the eviction that follows
    /// it: the new image's id, and the id the insert pushed out.
    pub(super) fn insert() -> u64 {
        let mut registry = lock();
        registry.next_id += 1;
        let id = registry.next_id;
        registry.order.push_back(id);
        registry.live.insert(id);
        if registry.order.len() == MAX_IMAGE_COUNT {
            if let Some(oldest) = registry.order.pop_front() {
                registry.live.remove(&oldest);
                registry.evictions += 1;
            }
        }
        id
    }

    /// How many images the server-wide cap has evicted since the process
    /// started.
    pub(super) fn evictions() -> u64 {
        lock().evictions
    }

    /// A screen dropped this image itself.
    pub(super) fn remove(id: u64) {
        let mut registry = lock();
        if registry.live.remove(&id) {
            registry.order.retain(|entry| *entry != id);
        }
    }

    /// Whether the server still holds this image.
    pub(super) fn is_live(id: u64) -> bool {
        lock().live.contains(&id)
    }
}

/// One image on a screen, tmux's `struct image`.
///
/// Deliberately not `Clone`: the identity is the server-wide registry's, and a
/// second `Image` carrying it would outlive its own eviction.
#[derive(Debug)]
pub struct Image {
    /// The server-wide identity, which is what the eviction order names.
    id: u64,
    /// The anchor cell, in viewport coordinates.
    pub px: usize,
    pub py: usize,
    /// The size in cells.
    pub sx: usize,
    pub sy: usize,
    /// Shared because every attached client's frame reads the same image and
    /// none of them owns it.
    pub data: Arc<SixelImage>,
    /// The placeholder a client that cannot draw sixel gets, kept beside the
    /// image because tmux builds it once at store time and again after a crop.
    pub fallback: Arc<str>,
}

/// A screen's images, tmux's `s->images` — and, while the alternate screen is
/// up, its `s->saved_images`.
#[derive(Debug, Default)]
pub struct Images {
    list: Vec<Image>,
    /// Bumped whenever the list changes, so a renderer can tell that what it
    /// last drew is stale without comparing the images themselves.
    revision: u64,
}

impl Images {
    /// The images this screen still holds, oldest first.
    ///
    /// The ones another screen's write evicted are filtered out rather than
    /// removed, because reading is not a mutation; the next operation that does
    /// mutate reclaims them.
    pub fn live(&self) -> impl Iterator<Item = &Image> + '_ {
        self.list.iter().filter(|image| registry::is_live(image.id))
    }

    /// A counter that changes whenever what [`Self::live`] answers does.
    ///
    /// The server-wide eviction count is folded in because an eviction can
    /// shorten this screen's list without this screen doing anything. Both
    /// terms only ever grow, so the sum does too.
    pub fn revision(&self) -> u64 {
        self.revision.wrapping_add(registry::evictions())
    }

    /// Drop the images another screen's write evicted.
    ///
    /// tmux frees them at the moment of eviction, reaching into whichever
    /// screen owns them. A screen that owns none of the evicted images sees
    /// nothing happen here.
    fn prune(&mut self) -> bool {
        let before = self.list.len();
        self.list.retain(|image| registry::is_live(image.id));
        if self.list.len() != before {
            self.revision += 1;
            return true;
        }
        false
    }

    /// tmux's `image_store`: anchor an image at the cursor.
    pub fn store(&mut self, data: SixelImage, px: usize, py: usize) {
        let (sx, sy) = data.size_in_cells();
        let fallback = SixelImage::fallback_text(sx, sy).into();
        let (sx, sy) = (sx as usize, sy as usize);
        let id = registry::insert();
        self.list.push(Image {
            id,
            px,
            py,
            sx,
            sy,
            data: Arc::new(data),
            fallback,
        });
        self.revision += 1;
        // The insert may have evicted one of this screen's own images.
        self.prune();
    }

    /// tmux's `image_free_all`. The answer is whether the screen needs redrawing.
    pub fn free_all(&mut self) -> bool {
        if self.list.is_empty() {
            return false;
        }
        for image in self.list.drain(..) {
            registry::remove(image.id);
        }
        self.revision += 1;
        true
    }

    /// Free every image matching a predicate, answering whether any did.
    fn free_matching(&mut self, mut hit: impl FnMut(&Image) -> bool) -> bool {
        let mut redraw = false;
        self.list.retain(|image| {
            if hit(image) {
                registry::remove(image.id);
                redraw = true;
                return false;
            }
            true
        });
        if redraw {
            self.revision += 1;
        }
        redraw | self.prune()
    }

    /// tmux's `image_check_line`: free every image any of rows
    /// `[py, py + ny)` runs through.
    ///
    /// `ny` is deliberately not clamped. tmux passes `s->cy - 1` from
    /// `screen_write_clearstartofscreen`, which underflows to the whole
    /// unsigned range when the cursor is on the first row, and the effect —
    /// every image on the screen goes — is reproduced rather than corrected.
    pub fn check_line(&mut self, py: usize, ny: usize) -> bool {
        self.free_matching(|image| py.saturating_add(ny) > image.py && py < image.py + image.sy)
    }

    /// tmux's `image_check_area`: the same, bounded in both directions.
    pub fn check_area(&mut self, px: usize, py: usize, nx: usize, ny: usize) -> bool {
        self.free_matching(|image| {
            py < image.py + image.sy
                && py.saturating_add(ny) > image.py
                && px < image.px + image.sx
                && px.saturating_add(nx) > image.px
        })
    }

    /// tmux's `image_scroll_up`: move the anchors up, drop what left the top of
    /// the screen, and crop what is half way off it.
    pub fn scroll_up(&mut self, lines: usize) -> bool {
        let mut redraw = false;
        let mut dropped = Vec::new();
        for image in &mut self.list {
            if image.py >= lines {
                image.py -= lines;
                redraw = true;
                continue;
            }
            if image.py + image.sy <= lines {
                dropped.push(image.id);
                redraw = true;
                continue;
            }
            // Partly off the top: keep the rows that are still on the screen by
            // cropping that many cells off the image's own top.
            let sx = image.sx as u32;
            let sy = ((image.py + image.sy) - lines) as u32;
            let oy = image.sy as u32 - sy;
            let Some(new) = image.data.scale(0, 0, 0, oy, sx, sy, true) else {
                dropped.push(image.id);
                redraw = true;
                continue;
            };
            image.data = Arc::new(new);
            image.py = 0;
            let (sx, sy) = image.data.size_in_cells();
            image.sx = sx as usize;
            image.sy = sy as usize;
            image.fallback = SixelImage::fallback_text(sx, sy).into();
            redraw = true;
        }
        if !dropped.is_empty() {
            self.list.retain(|image| !dropped.contains(&image.id));
            for id in dropped {
                registry::remove(id);
            }
        }
        if redraw {
            self.revision += 1;
        }
        redraw | self.prune()
    }

    /// tmux's `TAILQ_CONCAT`: hand every image over to another list, leaving
    /// this one empty. The alternate screen swaps its list with the primary's
    /// this way, so nothing is freed and nothing leaves the server-wide order.
    pub fn take_from(&mut self, other: &mut Images) {
        self.list.append(&mut other.list);
        self.revision += 1;
        other.revision += 1;
    }
}

impl Drop for Images {
    fn drop(&mut self) {
        for image in &self.list {
            registry::remove(image.id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sixel;

    /// A one-cell image against a ten by twenty pixel cell.
    fn image(cells_across: u32) -> SixelImage {
        let mut payload = b"#0;2;100;0;0#0".to_vec();
        payload.extend_from_slice(format!("!{}~", cells_across * 10).as_bytes());
        sixel::parse(&payload, 0, 10, 20).expect("the payload parses")
    }

    #[test]
    fn an_image_is_anchored_where_it_was_stored_and_sized_in_cells() {
        let mut images = Images::default();
        images.store(image(3), 2, 4);
        let stored = images.live().next().expect("one image");
        assert_eq!((stored.px, stored.py), (2, 4));
        assert_eq!((stored.sx, stored.sy), (3, 1));
        assert!(stored.fallback.starts_with("SIXEL IMAGE (3x1)"));
    }

    #[test]
    fn a_line_check_frees_only_the_rows_it_crosses() {
        let mut images = Images::default();
        images.store(image(1), 0, 4);
        assert!(!images.check_line(0, 4), "the row above does not touch it");
        assert!(!images.check_line(5, 1), "nor the row below");
        assert!(images.check_line(4, 1));
        assert_eq!(images.live().count(), 0);
    }

    /// The underflow `screen_write_clearstartofscreen` performs on the first
    /// row: every image goes.
    #[test]
    fn a_line_check_with_an_underflowed_count_frees_everything() {
        let mut images = Images::default();
        images.store(image(1), 0, 9);
        assert!(images.check_line(0, usize::MAX));
        assert_eq!(images.live().count(), 0);
    }

    #[test]
    fn an_area_check_needs_the_columns_to_overlap_too() {
        let mut images = Images::default();
        images.store(image(2), 4, 1);
        assert!(
            !images.check_area(0, 1, 4, 1),
            "ends where the image starts"
        );
        assert!(!images.check_area(6, 1, 4, 1), "starts where it ends");
        assert!(images.check_area(5, 1, 1, 1));
    }

    #[test]
    fn scrolling_moves_anchors_up_and_drops_what_leaves_the_screen() {
        let mut images = Images::default();
        images.store(image(1), 0, 3);
        assert!(images.scroll_up(2));
        assert_eq!(images.live().next().expect("one image").py, 1);
        assert!(images.scroll_up(1));
        assert_eq!(
            images.live().next().expect("one image").py,
            0,
            "its last row is still on screen"
        );
        assert!(images.scroll_up(1));
        assert_eq!(images.live().count(), 0, "now it has left the screen");
    }

    #[test]
    fn scrolling_crops_an_image_that_is_only_half_off_the_top() {
        // Three cells tall: sixty pixels over a twenty-pixel cell.
        let payload = b"#0;2;100;0;0#0~-~-~".to_vec();
        let tall = sixel::parse(&payload, 0, 10, 20).expect("parses");
        assert_eq!(tall.size_in_cells(), (1, 1), "eighteen pixels is one cell");

        let mut payload = b"#0;2;100;0;0#0".to_vec();
        for _ in 0..10 {
            payload.extend_from_slice(b"~-");
        }
        let tall = sixel::parse(&payload, 0, 10, 20).expect("parses");
        assert_eq!(tall.size_in_cells(), (1, 3), "sixty pixels is three cells");

        let mut images = Images::default();
        images.store(tall, 0, 1);
        assert!(images.scroll_up(2));
        let cropped = images.live().next().expect("one image");
        assert_eq!(cropped.py, 0, "what is left starts at the top");
        assert_eq!((cropped.sx, cropped.sy), (1, 2), "one cell was cut away");
        assert!(cropped.fallback.starts_with("SIXEL IMAGE (1x2)"));
    }

    #[test]
    fn an_alternate_screen_hands_its_list_over_rather_than_freeing_it() {
        let mut live = Images::default();
        let mut saved = Images::default();
        live.store(image(1), 0, 0);
        saved.take_from(&mut live);
        assert_eq!(live.live().count(), 0);
        assert_eq!(saved.live().count(), 1, "the image survived the handover");
    }

    /// The cap is the server's, so a second screen's writes evict the first
    /// screen's images. Nextest gives each test its own process, so the
    /// registry this exercises is this test's alone.
    #[test]
    fn the_server_wide_cap_evicts_across_screens() {
        let mut first = Images::default();
        let mut second = Images::default();
        first.store(image(1), 0, 0);
        for _ in 0..MAX_IMAGE_COUNT {
            second.store(image(1), 0, 0);
        }
        assert_eq!(
            first.live().count(),
            0,
            "the other screen's flood pushed this screen's image out"
        );
        assert_eq!(
            second.live().count(),
            MAX_IMAGE_COUNT - 1,
            "tmux evicts on reaching the cap, so one less than it is held"
        );
    }
}
