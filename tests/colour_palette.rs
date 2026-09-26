use hmux2::src::options::*;
use hmux2::src::shared::colour::{colour_palette, COLOUR_FLAG_256};
use hmux2::src::style::colour::*;
use std::ffi::CStr;
use std::ptr::null_mut;

#[test]
fn overrides_fall_back_to_defaults_and_clear_preserves_defaults() {
    unsafe {
        let mut p = colour_palette::default();
        colour_palette_init(&mut p);
        let mut defaults = Box::new([-1; 256]);
        defaults[1] = 2;
        p.default_palette = Some(defaults);
        assert_eq!((p.fg, p.bg), (8, 8));
        assert_eq!(colour_palette_get(&mut p, 1), 2);
        assert_eq!(colour_palette_set(&mut p, 1, 3), 1);
        assert_eq!(colour_palette_get(&mut p, 1), 3);
        assert_eq!(colour_palette_set(&mut p, 1, -1), 1);
        assert_eq!(colour_palette_get(&mut p, 1), 2);
        colour_palette_set(&mut p, 1, 4);
        p.fg = 1;
        p.bg = 2;
        colour_palette_clear(&mut p);
        assert_eq!((p.fg, p.bg), (8, 8));
        assert!(p.palette.is_none());
        assert_eq!(colour_palette_get(&mut p, 1), 2);
        colour_palette_free(&mut p);
        colour_palette_free(&mut p);
        assert!(p.default_palette.is_none());
        assert_eq!(colour_palette_get(&mut p, 1), -1);
        colour_palette_set(&mut p, 1, 5);
        colour_palette_init(&mut p);
        assert!(p.palette.is_none());
    }
}

#[test]
fn indices_are_normalized_and_invalid_indices_are_rejected() {
    unsafe {
        let mut p = colour_palette::default();
        assert_eq!(colour_palette_set(&mut p, 0, -1), 0);
        assert!(p.palette.is_none());
        for n in [0, 7, 8, 15, 255] {
            assert_eq!(colour_palette_set(&mut p, n, n + 100), 1);
            assert_eq!(colour_palette_get(&mut p, COLOUR_FLAG_256 | n), n + 100);
        }
        assert_eq!(colour_palette_get(&mut p, 0), 100);
        assert_eq!(colour_palette_get(&mut p, 7), 107);
        assert_eq!(colour_palette_get(&mut p, 90), 108);
        assert_eq!(colour_palette_get(&mut p, 97), 115);
        for n in [-1, i32::MIN, 8, 255, COLOUR_FLAG_256 | 256, i32::MAX] {
            assert_eq!(colour_palette_get(&mut p, n), -1);
        }
        for n in [-1, 256, i32::MAX] {
            assert_eq!(colour_palette_set(&mut p, n, 1), 0);
        }
        assert_eq!(colour_palette_get(null_mut(), 0), -1);
        assert_eq!(colour_palette_set(null_mut(), 0, 1), 0);
        colour_palette_clear(null_mut());
        colour_palette_free(null_mut());
        colour_palette_from_option(null_mut(), null_mut());
    }
}

#[test]
fn option_reload_replaces_defaults_without_changing_overrides() {
    unsafe {
        let oo = options_create(null_mut());
        let table = &raw const hmux2::src::options_table::options_table;
        let definition = (*table)
            .iter()
            .find(|oe| !oe.name.is_null() && CStr::from_ptr(oe.name) == c"pane-colours")
            .unwrap();
        let array = options_empty(oo, definition);
        let mut p = colour_palette::default();
        colour_palette_from_option(&mut p, oo);
        assert!(p.default_palette.is_none());
        assert_eq!(
            options_array_set(array, c"1".as_ptr(), c"red".as_ptr(), 0, null_mut()),
            0
        );
        colour_palette_from_option(&mut p, oo);
        assert_eq!(colour_palette_get(&mut p, 1), 1);
        colour_palette_set(&mut p, 1, 4);
        options_array_clear(array);
        assert_eq!(
            options_array_set(array, c"255".as_ptr(), c"green".as_ptr(), 0, null_mut()),
            0
        );
        colour_palette_from_option(&mut p, oo);
        assert_eq!(colour_palette_get(&mut p, 1), 4);
        colour_palette_set(&mut p, 1, -1);
        assert_eq!(colour_palette_get(&mut p, 1), -1);
        assert_eq!(colour_palette_get(&mut p, COLOUR_FLAG_256 | 255), 2);
        options_array_clear(array);
        colour_palette_from_option(&mut p, oo);
        assert!(p.default_palette.is_none());
        assert_eq!(colour_palette_get(&mut p, COLOUR_FLAG_256 | 255), -1);
        options_free(oo);
        // The arrays are released by normal Rust drop when p leaves scope.
    }
}
