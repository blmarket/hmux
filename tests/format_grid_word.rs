//! A word's collected cells are converted into an owner before the local array
//! drops.

use hmux2::src::format::format_grid_word;
use hmux2::src::grid::{
    grid_create, grid_default_cell, grid_destroy, grid_set_cell, grid_set_padding,
};
use hmux2::src::options::{options_create, options_default, options_free};
use hmux2::src::options_table::options_table;
use hmux2::src::tmux::global_s_options;
use std::ffi::CStr;

#[test]
fn grid_word_collects_wide_cells_and_returns_owned_strings() {
    unsafe {
        let saved_s_options = global_s_options;
        let s_options = options_create(core::ptr::null_mut());
        let definition = (*(&raw const options_table))
            .iter()
            .find(|entry| !entry.name.is_null() && CStr::from_ptr(entry.name) == c"word-separators")
            .unwrap();
        options_default(s_options, definition);
        global_s_options = s_options;

        let gd = grid_create(8, 1, 0);
        assert!(!gd.is_null());
        let mut cell = grid_default_cell;
        for (x, byte) in [(0, b'H'), (1, b'i'), (2, b' ')] {
            cell.data.data[0] = byte;
            grid_set_cell(&mut *gd, x, 0, &cell);
        }
        cell.data.data[..3].copy_from_slice("漢".as_bytes());
        cell.data.size = 3;
        cell.data.have = 3;
        cell.data.width = 2;
        grid_set_cell(&mut *gd, 3, 0, &cell);
        grid_set_padding(&mut *gd, 4, 0, cell.bg);

        cell.data.data = [0; 32];
        cell.data.data[0] = b'Z';
        cell.data.size = 1;
        cell.data.have = 1;
        cell.data.width = 1;
        grid_set_cell(&mut *gd, 5, 0, &cell);
        cell.data.data[0] = b' ';
        grid_set_cell(&mut *gd, 6, 0, &cell);

        let first = format_grid_word(&*gd, 1, 0).unwrap();
        let second = format_grid_word(&*gd, 3, 0).unwrap();
        assert_eq!(first.as_bytes(), b"Hi");
        assert_eq!(second.as_bytes(), "漢Z".as_bytes());
        assert!(format_grid_word(&*gd, 6, 0).is_none());

        grid_destroy(gd);
        global_s_options = saved_s_options;
        options_free(s_options);
        assert_eq!(first.as_bytes(), b"Hi");
        assert_eq!(second.as_bytes(), "漢Z".as_bytes());
    }
}
