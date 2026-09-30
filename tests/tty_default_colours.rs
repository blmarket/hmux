use hmux2::src::window::Window as _;
use hmux2::src::grid::{grid_cells_equal, grid_default_cell};
use hmux2::src::shared::grid::grid_cell;
use hmux2::src::shared::pane::window_pane;
use hmux2::src::shared::window::window;
use hmux2::src::tty::tty_default_colours;
use hmux2::src::window::{PaneOrder, Window};

#[test]
fn returned_defaults_preserve_active_fallback_and_dim_without_cached_attributes() {
    unsafe {
        let window_owner = hmux2::src::shared::window::WindowRef::empty();
        let pane_owner = window_pane::new();
        {
            let pane = &mut *pane_owner.get();
            pane.window = std::rc::Rc::downgrade(&window_owner);
            pane.cached_gc = grid_cell {
                fg: 1,
                bg: 4,
                attr: 1,
                link: 7,
                ..grid_default_cell
            };
            pane.cached_dim = 20;
            pane.cached_active_dim = 50;
        }
        window_owner.initialize_pane(&pane_owner, None);
        for (active, foreground, background, expected_fg, expected_bg, expected_dim) in [
            (true, 8, 8, 1, 4, 50),
            (true, 9, 8, 9, 4, 50),
            (true, 8, 9, 1, 9, 50),
            (true, 2, 3, 2, 3, 50),
            (false, 2, 3, 1, 4, 20),
        ] {
            if !active {
                window_owner.forget_pane(&pane_owner);
            }
            let (cached, cached_active) = {
                let pane = &mut *pane_owner.get();
                pane.cached_active_gc = grid_cell {
                    fg: foreground,
                    bg: background,
                    attr: 2,
                    link: 9,
                    ..grid_default_cell
                };
                (pane.cached_gc, pane.cached_active_gc)
            };
            let (cell, dim) = tty_default_colours(&pane_owner);
            let expected = grid_cell {
                fg: expected_fg,
                bg: expected_bg,
                ..grid_default_cell
            };
            assert!(grid_cells_equal(&cell, &expected));
            assert_eq!(dim, expected_dim);
            let pane = &*pane_owner.get();
            assert!(grid_cells_equal(&pane.cached_gc, &cached));
            assert!(grid_cells_equal(&pane.cached_active_gc, &cached_active));
        }
        for order in [PaneOrder::Index, PaneOrder::Stacking] {
            window_owner.borrow_pane_order_mut(order).storage.clear();
        }
        (*pane_owner.get()).window = std::rc::Weak::new();
        window_owner.release(c"test owner");
    }
}
