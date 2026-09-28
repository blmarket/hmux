use hmux2::src::grid::{grid_cells_equal, grid_default_cell};
use hmux2::src::shared::grid::grid_cell;
use hmux2::src::shared::pane::window_pane;
use hmux2::src::shared::window::window;
use hmux2::src::tty::tty_default_colours;

#[test]
fn returned_defaults_preserve_active_fallback_and_dim_without_cached_attributes() {
    unsafe {
        let window_owner = window::new();
        let window = &mut *window_owner.get();
        let pane_owner = window_pane::new();
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
        for (active, foreground, background, expected_fg, expected_bg, expected_dim) in [
            (true, 8, 8, 1, 4, 50),
            (true, 9, 8, 9, 4, 50),
            (true, 8, 9, 1, 9, 50),
            (true, 2, 3, 2, 3, 50),
            (false, 2, 3, 1, 4, 20),
        ] {
            window.set_active(if active { pane as *mut window_pane } else { std::ptr::null_mut() });
            pane.cached_active_gc = grid_cell {
                fg: foreground,
                bg: background,
                attr: 2,
                link: 9,
                ..grid_default_cell
            };
            let cached = pane.cached_gc;
            let cached_active = pane.cached_active_gc;
            let (cell, dim) = tty_default_colours(pane as *mut window_pane);
            let expected = grid_cell {
                fg: expected_fg,
                bg: expected_bg,
                ..grid_default_cell
            };
            assert!(grid_cells_equal(&cell, &expected));
            assert_eq!(dim, expected_dim);
            assert!(grid_cells_equal(&pane.cached_gc, &cached));
            assert!(grid_cells_equal(&pane.cached_active_gc, &cached_active));
        }
        window.set_active(std::ptr::null_mut());
        pane.window = std::rc::Weak::new();
    }
}
