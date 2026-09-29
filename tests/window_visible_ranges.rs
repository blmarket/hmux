use hmux2::src::shared::display::visible_range;
use hmux2::src::shared::pane::{PANE_SCROLLBARS_ALWAYS, PANE_SCROLLBARS_LEFT, window_pane};
use hmux2::src::shared::window::window;
use hmux2::src::window_visible::{window_position_is_visible, window_visible_ranges};
use std::cell::UnsafeCell;
use std::ptr::null_mut;
use std::rc::Rc;

fn segments(ranges: &[visible_range]) -> Vec<(u32, u32)> {
    ranges.iter().map(|range| (range.px, range.nx)).collect()
}

unsafe fn calculate_ranges(
    pane: Option<&window_pane>,
    x: i32,
    y: i32,
    width: u32,
) -> Vec<visible_range> {
    unsafe {
        let mut ranges = Vec::new();
        window_visible_ranges(pane, x, y, width, &mut ranges);
        ranges
    }
}

struct Scene {
    window: Rc<UnsafeCell<window>>,
    panes: Vec<Rc<UnsafeCell<window_pane>>>,
}

impl Scene {
    fn new() -> Self {
        let window = window::new();
        unsafe {
            (*window.get()).sx = 80;
            (*window.get()).sy = 24;
        }
        Self {
            window,
            panes: Vec::new(),
        }
    }

    unsafe fn free(self) {
        unsafe {
            hmux2::src::window::window_remove_ref(self.window, c"test scene".as_ptr());
        }
    }

    fn add_pane(&mut self, x: i32, y: i32, width: u32, height: u32) -> Rc<UnsafeCell<window_pane>> {
        // These panes have no display resources requiring model cleanup.
        let mut pane = window_pane::empty();
        pane.window = Rc::downgrade(&self.window);
        pane.xoff = x;
        pane.yoff = y;
        pane.sx = width;
        pane.sy = height;
        let owner = pane.into_shared();
        unsafe {
            (*self.window.get())
                .z_index
                .push_front(Rc::downgrade(&owner));
        }
        self.panes.push(owner.clone());
        owner
    }
}

#[test]
fn pane_less_results_survive_later_nonempty_and_empty_queries() {
    unsafe {
        let first = calculate_ranges(None, 10, 5, 2);
        let second = calculate_ranges(None, 0, 6, 80);
        let empty = calculate_ranges(None, 0, -1, 80);
        assert_eq!(segments(&first), [(10, 2)]);
        assert_eq!(segments(&second), [(0, 80)]);
        assert!(empty.is_empty());
        assert!(window_position_is_visible(&first, 10));
        assert!(!window_position_is_visible(&first, 12));
        assert!(!window_position_is_visible(&empty, 10));

        let third = calculate_ranges(None, -3, 0, 8);
        assert_eq!(segments(&third), [(0, 5)]);
        assert!(empty.is_empty());
    }
}

#[test]
fn pane_results_survive_flush_shaped_queries_and_pane_destruction() {
    let (character, line) = unsafe {
        let mut scene = Scene::new();
        let base = scene.add_pane(0, 0, 80, 24).get();
        // This pane covers the character on row 5, but not row 6.
        scene.add_pane(9, 5, 4, 1);
        let character = calculate_ranges((base).as_ref(), 10, 5, 2);
        let line = calculate_ranges((base).as_ref(), 0, 6, 80);
        // A subsequent empty query must not clear either result.
        assert!(calculate_ranges((base).as_ref(), 0, 24, 80).is_empty());
        scene.free();
        (character, line)
    };
    // Neither result depends on the lifetime of the pane or window.
    assert!(!window_position_is_visible(&character, 10));
    assert_eq!(segments(&line), [(0, 80)]);
}

#[test]
fn clips_to_window_and_preserves_split_ranges_and_border_rules() {
    unsafe {
        let mut scene = Scene::new();
        let base = scene.add_pane(0, 0, 80, 24).get();
        // Including side borders, these obscure [14, 21) and [24, 28).
        scene.add_pane(15, 5, 5, 1);
        scene.add_pane(25, 5, 2, 1);
        let ranges = calculate_ranges((base).as_ref(), 10, 5, 20);
        assert_eq!(segments(&ranges), [(10, 4), (21, 3), (28, 2)]);
        for x in 10..30 {
            assert_eq!(
                window_position_is_visible(&ranges, x),
                (10..14).contains(&x) || (21..24).contains(&x) || (28..30).contains(&x),
                "column {x}"
            );
        }
        // Non-floating panes do not obscure their horizontal border rows.
        assert_eq!(
            segments(&calculate_ranges((base).as_ref(), 10, 4, 20)),
            [(10, 20)]
        );
        assert_eq!(
            segments(&calculate_ranges((base).as_ref(), 10, 6, 20)),
            [(10, 20)]
        );
        assert_eq!(
            segments(&calculate_ranges((base).as_ref(), -3, 0, 8)),
            [(0, 5)]
        );
        assert_eq!(
            segments(&calculate_ranges((base).as_ref(), 78, 0, 8)),
            [(78, 2)]
        );
        for (x, y, width) in [(80, 0, 1), (0, 24, 1), (0, -1, 1), (-3, 0, 3), (0, 0, 0)] {
            assert!(calculate_ranges((base).as_ref(), x, y, width).is_empty());
        }
        scene.free();
    }
}

#[test]
fn reserved_scrollbar_is_included_in_occlusion() {
    unsafe {
        let mut scene = Scene::new();
        let base = scene.add_pane(0, 0, 80, 24).get();
        let cover = scene.add_pane(15, 5, 5, 1).get();
        (*cover).scrollbar_style.width = 2;
        (*cover).scrollbar_style.pad = 1;
        (*scene.window.get()).sb = PANE_SCROLLBARS_ALWAYS;
        (*scene.window.get()).sb_pos = PANE_SCROLLBARS_LEFT;
        let left = calculate_ranges((base).as_ref(), 10, 5, 20);
        assert_eq!(segments(&left), [(10, 1), (21, 9)]);
        (*scene.window.get()).sb_pos = 0;
        let right = calculate_ranges((base).as_ref(), 10, 5, 20);
        assert_eq!(segments(&right), [(10, 4), (24, 6)]);
        assert_eq!(segments(&left), [(10, 1), (21, 9)]);
        scene.free();
    }
}

#[test]
fn reuses_capacity_and_replaces_results_including_empty_queries() {
    unsafe {
        let mut scene = Scene::new();
        let base = scene.add_pane(0, 0, 80, 24).get();
        scene.add_pane(15, 5, 5, 1);
        scene.add_pane(25, 5, 2, 1);
        let mut ranges = Vec::with_capacity(8);
        let storage = ranges.as_ptr();
        let capacity = ranges.capacity();
        window_visible_ranges(base.as_ref(), 10, 5, 20, &mut ranges);
        assert_eq!(segments(&ranges), [(10, 4), (21, 3), (28, 2)]);
        assert_eq!(ranges.as_ptr(), storage);
        assert_eq!(ranges.capacity(), capacity);

        // Every early return must clear old segments without freeing capacity.
        for (pane, x, y, width) in [
            (base, 80, 0, 1),
            (base, 0, 24, 1),
            (base, 0, -1, 1),
            (base, -3, 0, 3),
            (base, 0, 0, 0),
            (null_mut(), 0, -1, 1),
            (null_mut(), -3, 0, 3),
            (null_mut(), 0, 0, 0),
        ] {
            window_visible_ranges(base.as_ref(), 10, 5, 20, &mut ranges);
            assert_eq!(ranges.len(), 3);
            window_visible_ranges(pane.as_ref(), x, y, width, &mut ranges);
            assert!(ranges.is_empty());
            assert_eq!(ranges.as_ptr(), storage);
            assert_eq!(ranges.capacity(), capacity);
        }
        window_visible_ranges(None, 2, 0, 7, &mut ranges);
        assert_eq!(segments(&ranges), [(2, 7)]);
        window_visible_ranges(base.as_ref(), 78, 0, 8, &mut ranges);
        assert_eq!(segments(&ranges), [(78, 2)]);
        assert_eq!(ranges.as_ptr(), storage);
        assert_eq!(ranges.capacity(), capacity);
        scene.free();
    }
}
