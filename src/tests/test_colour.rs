use super::*;
use ::core::ffi::{CStr, c_int};

const RGB: c_int = COLOUR_FLAG_RGB;
const C256: c_int = COLOUR_FLAG_256;

fn tostring(c: c_int) -> String {
    colour_tostring(c).to_str().unwrap().to_owned()
}

fn fromstring(s: &CStr) -> c_int {
    unsafe { colour_fromstring(s.as_ptr()) }
}

fn byname(s: &CStr) -> c_int {
    unsafe { colour_byname(s.as_ptr()) }
}

fn parse_x11(s: &CStr) -> c_int {
    unsafe { colour_parseX11(s.as_ptr()) }
}

#[test]
fn the_distance_between_two_colours_is_the_squared_component_delta() {
    assert_eq!(colour_dist_sq(0, 0, 0, 1, 2, 3), 14);
    assert_eq!(colour_dist_sq(5, 5, 5, 5, 5, 5), 0);
    assert_eq!(colour_dist_sq(0, 0, 0, 255, 0, 0), 65025);
}

#[test]
fn the_six_cube_index_has_three_ranges() {
    assert_eq!(colour_to_6cube(0), 0);
    assert_eq!(colour_to_6cube(47), 0);
    assert_eq!(colour_to_6cube(48), 1);
    assert_eq!(colour_to_6cube(113), 1);
    assert_eq!(colour_to_6cube(114), 1);
    assert_eq!(colour_to_6cube(115), 2);
    assert_eq!(colour_to_6cube(255), 5);
}

#[test]
fn an_exact_cube_colour_maps_straight_to_its_index() {
    {
        assert_eq!(colour_find_rgb(0, 0, 0), 16 | C256);
        assert_eq!(colour_find_rgb(0x5f, 0x87, 0xaf), 67 | C256);
        assert_eq!(colour_find_rgb(0xff, 0xff, 0xff), 231 | C256);
    }
}

#[test]
fn a_nearer_grey_wins_over_the_cube() {
    {
        assert_eq!(colour_find_rgb(8, 8, 8), 232 | C256);
        assert_eq!(colour_find_rgb(100, 100, 100), 241 | C256);
    }
}

#[test]
fn a_nearer_cube_colour_wins_over_the_grey_ramp() {
    {
        assert_eq!(colour_find_rgb(250, 250, 250), 231 | C256);
        assert_eq!(colour_find_rgb(0xff, 0, 0), 196 | C256);
    }
}

#[test]
fn rgb_components_join_and_split_back() {
    unsafe {
        assert_eq!(colour_join_rgb(0x12, 0x34, 0x56), 0x123456 | RGB);
        assert_eq!(colour_split_rgb(0x123456 | RGB), (0x12, 0x34, 0x56));
    }
}

#[test]
fn forcing_rgb_expands_every_indexed_form() {
    {
        assert_eq!(colour_force_rgb(0x123456 | RGB), 0x123456 | RGB);
        assert_eq!(colour_force_rgb(1 | C256), 0x800000 | RGB);
        assert_eq!(colour_force_rgb(2), 0x8000 | RGB);
        assert_eq!(colour_force_rgb(0), RGB);
        assert_eq!(colour_force_rgb(90), 0x808080 | RGB);
        assert_eq!(colour_force_rgb(97), 0xffffff | RGB);
    }
}

#[test]
fn forcing_rgb_rejects_what_has_no_rgb_value() {
    {
        assert_eq!(colour_force_rgb(-1), -1);
        assert_eq!(colour_force_rgb(8), -1);
        assert_eq!(colour_force_rgb(9), -1);
        assert_eq!(colour_force_rgb(100), -1);
    }
}

#[test]
fn tostring_renders_every_colour_form() {
    assert_eq!(tostring(-1), "none");
    assert_eq!(tostring(0x123456 | RGB), "#123456");
    assert_eq!(tostring(5 | C256), "colour5");
    assert_eq!(tostring(300 | C256), "colour44");
    assert_eq!(tostring(10), "invalid");
    assert_eq!(tostring(89), "invalid");
}

#[test]
fn tostring_names_the_basic_colours() {
    for (c, name) in [
        (0, "black"),
        (1, "red"),
        (2, "green"),
        (3, "yellow"),
        (4, "blue"),
        (5, "magenta"),
        (6, "cyan"),
        (7, "white"),
        (8, "default"),
        (9, "terminal"),
        (90, "brightblack"),
        (91, "brightred"),
        (92, "brightgreen"),
        (93, "brightyellow"),
        (94, "brightblue"),
        (95, "brightmagenta"),
        (96, "brightcyan"),
        (97, "brightwhite"),
    ] {
        assert_eq!(tostring(c), name);
    }
}

#[test]
fn a_theme_follows_the_total_brightness_of_an_rgb_colour() {
    {
        assert_eq!(colour_totheme(-1), THEME_UNKNOWN);
        assert_eq!(colour_totheme(0xffffff | RGB), THEME_LIGHT);
        assert_eq!(colour_totheme(0x808080 | RGB), THEME_LIGHT);
        assert_eq!(colour_totheme(0x7f7f7f | RGB), THEME_DARK);
        assert_eq!(colour_totheme(RGB), THEME_DARK);
        assert_eq!(colour_totheme(231 | C256), THEME_LIGHT);
    }
}

#[test]
fn a_theme_of_a_basic_colour_goes_through_its_rgb_value() {
    {
        assert_eq!(colour_totheme(0), THEME_DARK);
        assert_eq!(colour_totheme(90), THEME_DARK);
        assert_eq!(colour_totheme(7), THEME_LIGHT);
        assert_eq!(colour_totheme(97), THEME_LIGHT);
        assert_eq!(colour_totheme(3), THEME_DARK);
        assert_eq!(colour_totheme(6), THEME_DARK);
        assert_eq!(colour_totheme(91), THEME_DARK);
        assert_eq!(colour_totheme(96), THEME_LIGHT);
        assert_eq!(colour_totheme(8), THEME_UNKNOWN);
        assert_eq!(colour_totheme(100), THEME_UNKNOWN);
    }
}

#[test]
fn fromstring_reads_a_hash_prefixed_rgb_triplet() {
    assert_eq!(fromstring(c"#ff0000"), 0xff0000 | RGB);
    assert_eq!(fromstring(c"#FF0000"), 0xff0000 | RGB);
    assert_eq!(fromstring(c"#000000"), RGB);
    assert_eq!(fromstring(c"#12345g"), -1);
    assert_eq!(fromstring(c"#12345"), -1);
    assert_eq!(fromstring(c"#1234567"), -1);
}

#[test]
fn fromstring_reads_the_colour_and_color_spellings() {
    assert_eq!(fromstring(c"colour0"), C256);
    assert_eq!(fromstring(c"colour255"), 255 | C256);
    assert_eq!(fromstring(c"COLOUR9"), 9 | C256);
    assert_eq!(fromstring(c"color12"), 12 | C256);
    assert_eq!(fromstring(c"colour256"), -1);
    assert_eq!(fromstring(c"colourx"), -1);
    assert_eq!(fromstring(c"colorx"), -1);
}

#[test]
fn fromstring_reads_the_basic_names_and_their_numbers() {
    for (name, number, c) in [
        (c"black", c"0", 0),
        (c"red", c"1", 1),
        (c"green", c"2", 2),
        (c"yellow", c"3", 3),
        (c"blue", c"4", 4),
        (c"magenta", c"5", 5),
        (c"cyan", c"6", 6),
        (c"white", c"7", 7),
        (c"brightblack", c"90", 90),
        (c"brightred", c"91", 91),
        (c"brightgreen", c"92", 92),
        (c"brightyellow", c"93", 93),
        (c"brightblue", c"94", 94),
        (c"brightmagenta", c"95", 95),
        (c"brightcyan", c"96", 96),
        (c"brightwhite", c"97", 97),
    ] {
        assert_eq!(fromstring(name), c, "{name:?}");
        assert_eq!(fromstring(number), c, "{number:?}");
    }
    assert_eq!(fromstring(c"RED"), 1);
    assert_eq!(fromstring(c"default"), 8);
    assert_eq!(fromstring(c"TERMINAL"), 9);
}

#[test]
fn fromstring_falls_back_to_the_x11_colour_names() {
    assert_eq!(fromstring(c"AliceBlue"), 0xf0f8ff | RGB);
    assert_eq!(fromstring(c"nosuch"), -1);
}

#[test]
fn the_256_colour_cube_has_an_rgb_value_for_every_index() {
    {
        assert_eq!(colour_256toRGB(0), RGB);
        assert_eq!(colour_256toRGB(1), 0x800000 | RGB);
        assert_eq!(colour_256toRGB(15), 0xffffff | RGB);
        assert_eq!(colour_256toRGB(16), RGB);
        assert_eq!(colour_256toRGB(231), 0xffffff | RGB);
        assert_eq!(colour_256toRGB(232), 0x080808 | RGB);
        assert_eq!(colour_256toRGB(255), 0xeeeeee | RGB);
        assert_eq!(colour_256toRGB(5 | C256), 0x800080 | RGB);
    }
}

#[test]
fn the_256_colour_cube_folds_down_to_sixteen_colours() {
    {
        assert_eq!(colour_256to16(0), 0);
        assert_eq!(colour_256to16(7), 7);
        assert_eq!(colour_256to16(8), 8);
        assert_eq!(colour_256to16(15), 15);
        assert_eq!(colour_256to16(16), 0);
        assert_eq!(colour_256to16(100), 3);
        assert_eq!(colour_256to16(231), 15);
        assert_eq!(colour_256to16(232), 0);
        assert_eq!(colour_256to16(243), 8);
        assert_eq!(colour_256to16(255), 15);
        assert_eq!(colour_256to16(9 | C256), 9);
    }
}

#[test]
fn grey_names_scale_a_percentage_onto_the_rgb_ramp() {
    assert_eq!(byname(c"grey"), 0xbebebe | RGB);
    assert_eq!(byname(c"gray"), 0xbebebe | RGB);
    assert_eq!(byname(c"GREY"), 0xbebebe | RGB);
    assert_eq!(byname(c"grey0"), RGB);
    assert_eq!(byname(c"grey50"), 0x7f7f7f | RGB);
    assert_eq!(byname(c"gray100"), 0xffffff | RGB);
    assert_eq!(byname(c"grey101"), -1);
    assert_eq!(byname(c"greyx"), -1);
}

#[test]
fn the_x11_colour_table_is_matched_without_regard_to_case() {
    assert_eq!(byname(c"AliceBlue"), 0xf0f8ff | RGB);
    assert_eq!(byname(c"aliceblue"), 0xf0f8ff | RGB);
    assert_eq!(byname(c"yellow4"), 0x8b8b00 | RGB);
    assert_eq!(byname(c"yellow green"), 0x9acd32 | RGB);
    assert_eq!(byname(c"nosuch"), -1);
    assert_eq!(byname(c""), -1);
}

#[test]
fn x11_colours_parse_from_every_accepted_spelling() {
    assert_eq!(parse_x11(c"rgb:11/22/33"), 0x112233 | RGB);
    assert_eq!(parse_x11(c"#aabbcc"), 0xaabbcc | RGB);
    assert_eq!(parse_x11(c"1,2,3"), 0x010203 | RGB);
    assert_eq!(parse_x11(c"rgb:1111/2222/3333"), 0x112233 | RGB);
    assert_eq!(parse_x11(c"#111122223333"), 0x112233 | RGB);
}

#[test]
fn x11_colours_parse_from_cmyk_and_cmy() {
    assert_eq!(parse_x11(c"cmyk:0/0/0/0"), 0xffffff | RGB);
    assert_eq!(parse_x11(c"cmyk:1/1/1/1"), RGB);
    assert_eq!(parse_x11(c"cmy:0/0/0"), 0xffffff | RGB);
    assert_eq!(parse_x11(c"cmy:1/1/1"), RGB);
    assert_eq!(parse_x11(c"cmyk:2/0/0/0"), -1);
}

#[test]
fn an_x11_colour_name_is_looked_up_after_trimming_spaces() {
    assert_eq!(parse_x11(c"  AliceBlue  "), 0xf0f8ff | RGB);
    assert_eq!(parse_x11(c"nosuch"), -1);
    assert_eq!(parse_x11(c""), -1);
    assert_eq!(parse_x11(c"   "), -1);
}

fn blank_palette() -> colour_palette {
    colour_palette {
        fg: 0,
        bg: 0,
        palette: None,
        default_palette: None,
    }
}

#[test]
fn a_palette_starts_out_default_and_empty() {
    let mut p = blank_palette();
    unsafe {
        colour_palette_init(&raw mut p);
    }
    assert_eq!(p.fg, 8);
    assert_eq!(p.bg, 8);
    assert!(p.palette.is_none());
    assert!(p.default_palette.is_none());
}

#[test]
fn clearing_a_palette_drops_its_entries_but_keeps_the_defaults() {
    let mut p = blank_palette();
    unsafe {
        colour_palette_init(&raw mut p);
        p.fg = 1;
        p.bg = 2;
        assert_eq!(colour_palette_set(&raw mut p, 3, 0x123456 | RGB), 1);
        colour_palette_clear(&raw mut p);
        assert_eq!(p.fg, 8);
        assert_eq!(p.bg, 8);
        assert!(p.palette.is_none());
        colour_palette_clear(::core::ptr::null_mut::<colour_palette>());
    }
}

#[test]
fn freeing_a_palette_drops_both_tables() {
    let mut p = blank_palette();
    unsafe {
        colour_palette_init(&raw mut p);
        colour_palette_set(&raw mut p, 3, 1);
        p.default_palette = Some(Box::new([-1; 256]));
        colour_palette_free(&raw mut p);
        assert!(p.palette.is_none());
        assert!(p.default_palette.is_none());
        colour_palette_free(::core::ptr::null_mut::<colour_palette>());
    }
}

#[test]
fn setting_a_palette_entry_rejects_what_it_cannot_hold() {
    let mut p = blank_palette();
    unsafe {
        colour_palette_init(&raw mut p);
        assert_eq!(colour_palette_set(::core::ptr::null_mut(), 0, 1), 0);
        assert_eq!(colour_palette_set(&raw mut p, -1, 1), 0);
        assert_eq!(colour_palette_set(&raw mut p, 256, 1), 0);
        assert_eq!(colour_palette_set(&raw mut p, 0, -1), 0);
        assert!(p.palette.is_none());
        assert_eq!(colour_palette_set(&raw mut p, 0, 1), 1);
        assert!(p.palette.is_some());
        assert_eq!(colour_palette_set(&raw mut p, 1, -1), 1);
        colour_palette_free(&raw mut p);
    }
}

#[test]
fn getting_a_palette_entry_maps_the_index_forms() {
    let mut p = blank_palette();
    unsafe {
        colour_palette_init(&raw mut p);
        assert_eq!(colour_palette_get(::core::ptr::null_mut(), 0), -1);
        assert_eq!(colour_palette_get(&raw mut p, 0), -1);
        colour_palette_set(&raw mut p, 0, 0x111111 | RGB);
        colour_palette_set(&raw mut p, 9, 0x222222 | RGB);
        colour_palette_set(&raw mut p, 20, 0x333333 | RGB);
        assert_eq!(colour_palette_get(&raw mut p, 0), 0x111111 | RGB);
        assert_eq!(colour_palette_get(&raw mut p, 91), 0x222222 | RGB);
        assert_eq!(colour_palette_get(&raw mut p, 20 | C256), 0x333333 | RGB);
        assert_eq!(colour_palette_get(&raw mut p, 8), -1);
        assert_eq!(colour_palette_get(&raw mut p, 1), -1);
        colour_palette_free(&raw mut p);
    }
}

#[test]
fn a_palette_entry_falls_back_to_the_default_table() {
    let mut p = blank_palette();
    unsafe {
        colour_palette_init(&raw mut p);
        let mut def = Box::new([-1; 256]);
        def[2] = 0x444444 | RGB;
        p.default_palette = Some(def);
        assert_eq!(colour_palette_get(&raw mut p, 2), 0x444444 | RGB);
        assert_eq!(colour_palette_get(&raw mut p, 3), -1);
        colour_palette_set(&raw mut p, 2, 0x555555 | RGB);
        assert_eq!(colour_palette_get(&raw mut p, 2), 0x555555 | RGB);
        colour_palette_free(&raw mut p);
    }
}

#[test]
fn no_defaults_leaves_no_default_table() {
    let mut p = blank_palette();
    unsafe {
        colour_palette_init(&raw mut p);
        colour_palette_from_defaults(&raw mut p, None);
        assert!(p.default_palette.is_none());
        p.default_palette = Some(Box::new([-1; 256]));
        colour_palette_from_defaults(&raw mut p, None);
        assert!(p.default_palette.is_none());
        colour_palette_from_defaults(::core::ptr::null_mut::<colour_palette>(), None);
    }
}

#[test]
fn defaults_fill_the_default_table_by_index() {
    let mut p = blank_palette();
    unsafe {
        colour_palette_init(&raw mut p);
        let mut def = [-1; 256];
        def[0] = 1;
        def[1] = 0x00ff00 | RGB;
        colour_palette_from_defaults(&raw mut p, Some(&def));
        assert!(p.default_palette.is_some());
        assert_eq!(colour_palette_get(&raw mut p, 0), 1);
        assert_eq!(colour_palette_get(&raw mut p, 1), 0x00ff00 | RGB);
        assert_eq!(colour_palette_get(&raw mut p, 2), -1);
        def[0] = 2;
        colour_palette_from_defaults(&raw mut p, Some(&def));
        assert_eq!(colour_palette_get(&raw mut p, 0), 2);
        colour_palette_free(&raw mut p);
    }
}

#[test]
fn a_second_fill_replaces_the_whole_default_table() {
    let mut p = blank_palette();
    unsafe {
        colour_palette_init(&raw mut p);
        let mut def = [-1; 256];
        def[5] = 4;
        colour_palette_from_defaults(&raw mut p, Some(&def));
        assert_eq!(colour_palette_get(&raw mut p, 5), 4);
        colour_palette_from_defaults(&raw mut p, Some(&[-1; 256]));
        assert_eq!(colour_palette_get(&raw mut p, 5), -1);
        colour_palette_free(&raw mut p);
    }
}
