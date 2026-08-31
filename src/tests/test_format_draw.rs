use super::*;
use crate::grid::{grid_default_cell, grid_string_cells};
use crate::screen::{screen_write_start, screen_write_stop};
use crate::tests::test_fixtures::{Screen, globals};
use ::core::ffi::{CStr, c_char, c_int};
use ::core::ptr::null_mut;

/// What one call to `format_draw` left behind: the row it drew and the
/// style ranges it reported, each as `<type>|<argument>|<string> start-end`.
struct Drawn {
    line: String,
    ranges: Vec<String>,
}

/// Draws `expanded` into a screen `available` columns wide.
fn draw(available: u_int, expanded: &CStr) -> Drawn {
    draw_over(available, expanded, 0, true)
}

fn draw_over(
    available: u_int,
    expanded: &CStr,
    default_colours: c_int,
    with_ranges: bool,
) -> Drawn {
    let _guard = globals();
    let mut s = Screen::new(available.max(1), 1, 0);
    let mut ctx = Box::new(screen_write_ctx::default());
    let mut srs = style_ranges::new();
    unsafe {
        screen_write_start(&mut ctx, &mut *s.ptr());
        let base = grid_default_cell;
        format_draw(
            &mut ctx,
            &base,
            available,
            expanded.to_bytes(),
            if with_ranges { Some(&mut srs) } else { None },
            default_colours,
        );
        screen_write_stop(&mut ctx);

        let line = grid_string_cells(&*s.grid(), 0, 0, available.max(1), None, 0, null_mut())
            .to_string_lossy()
            .into_owned();
        let ranges = srs
            .iter()
            .map(|sr| {
                format!(
                    "{}|{}|{} {}-{}",
                    sr.type_0,
                    sr.argument,
                    CStr::from_ptr(&raw const sr.string as *const c_char).to_string_lossy(),
                    sr.start,
                    sr.end
                )
            })
            .collect();
        Drawn { line, ranges }
    }
}

/// What `format_width` makes of a string.
fn width(expanded: &CStr) -> u_int {
    let _guard = globals();
    format_width(expanded.to_bytes())
}

/// What is left of `expanded` after trimming to `limit` from each end.
fn trim(expanded: &CStr, limit: u_int) -> (String, String) {
    let _guard = globals();
    (
        format_trim_left(expanded.to_bytes(), limit)
            .to_string_lossy()
            .into_owned(),
        format_trim_right(expanded.to_bytes(), limit)
            .to_string_lossy()
            .into_owned(),
    )
}

/// A range whose whole visible part is squeezed out — here the centre
/// screen shrunk to nothing — is dropped rather than reported empty.
#[test]
fn a_range_squeezed_to_nothing_is_dropped() {
    let drawn = draw(
        6,
        c"left#[align=centre,range=left]cc#[norange]#[align=right]rr",
    );
    assert_eq!(drawn.line, "leftrr");
    assert_eq!(drawn.ranges, Vec::<String>::new());
}

/// A range that opens and closes at the same column is dropped too.
#[test]
fn a_range_with_nothing_in_it_is_dropped() {
    let drawn = draw(10, c"#[range=left]#[norange]a");
    assert_eq!(drawn.line, "a");
    assert_eq!(drawn.ranges, Vec::<String>::new());
}

/// A list scrolled around a focus near its end stops at the end rather
/// than running past it.
#[test]
fn a_list_focused_near_its_end_is_scrolled_no_further_than_the_end() {
    assert_eq!(
        draw(3, c"#[align=left,list=on]abcd#[list=focus]ef#[nolist]").line,
        "def"
    );
}

/// With nothing else left to give up, the left screen is the last to be
/// shrunk, and a list shrunk to nothing takes the drawing without one.
#[test]
fn the_left_screen_is_the_last_to_be_shrunk() {
    assert_eq!(draw(1, c"#[align=left]lll#[list=on]x#[nolist]").line, "l");
    assert_eq!(
        draw(
            1,
            c"#[align=centre,list=on]L#[nolist]#[align=left]LL#[align=absolute-centre]ZZ"
        )
        .line,
        "Z"
    );
    assert_eq!(
        draw(
            1,
            c"#[align=right,list=on]L#[nolist]#[align=left]LL#[align=absolute-centre]ZZ"
        )
        .line,
        "Z"
    );
    assert_eq!(
        draw(
            1,
            c"#[align=absolute-centre,list=on]LL#[nolist]#[align=left]LL"
        )
        .line,
        "L"
    );
}

#[test]
fn each_list_alignment_gives_up_the_centre_the_right_and_the_left_in_turn() {
    assert_eq!(
        draw(
            2,
            c"#[align=centre,list=on]L#[nolist]#[align=centre]CC#[align=right]RR#[align=left]LL"
        )
        .line,
        "LL"
    );
    assert_eq!(
        draw(
            2,
            c"#[align=right,list=on]L#[nolist]a#[align=centre]CC#[align=right]RR#[align=left]LL"
        )
        .line,
        "LL"
    );
    assert_eq!(
        draw(
            2,
            c"#[align=absolute-centre,list=on]L#[nolist]a#[align=centre]CC#[align=right]RR#[align=left]LL"
        )
        .line,
        "La"
    );
}

/// An absolute centre wider than the line is held to it whichever
/// alignment the list was opened at.
#[test]
fn an_absolute_centre_wider_than_the_line_is_held_to_it() {
    assert_eq!(
        draw(
            2,
            c"#[align=left]l#[list=on]x#[nolist]#[align=absolute-centre]ZZZ"
        )
        .line,
        "ZZ"
    );
}

/// A marker named while the focus is still open throws the focus away, so
/// the list is drawn from its start.
#[test]
fn a_marker_named_inside_the_focus_gives_the_focus_up() {
    assert_eq!(
        draw(
            20,
            c"#[align=left,list=on]a#[list=focus]b#[list=left-marker]<#[nolist]"
        )
        .line,
        "ab"
    );
    assert_eq!(
        draw(
            20,
            c"#[align=left,list=on]a#[list=focus]b#[list=right-marker]>#[nolist]"
        )
        .line,
        "ab"
    );
}

#[test]
fn a_centred_list_gives_up_the_centre_screen_first() {
    assert_eq!(draw(1, c"#[align=centre]CC#[list=on]L#[nolist]").line, "C");
}

#[test]
fn an_absolute_centre_is_held_to_the_line_whatever_the_list_alignment() {
    assert_eq!(
        draw(
            3,
            c"#[align=centre]C#[list=on]L#[nolist]#[align=absolute-centre]ZZZZ"
        )
        .line,
        "ZZZ"
    );
    assert_eq!(
        draw(
            3,
            c"#[align=right]R#[list=on]L#[nolist]#[align=absolute-centre]ZZZZ"
        )
        .line,
        "ZZZ"
    );
    assert_eq!(
        draw(2, c"#[align=absolute-centre]ZZZ#[list=on]#[nolist]").line,
        "ZZ"
    );
}

#[test]
fn plain_text_is_drawn_where_it_stands() {
    assert_eq!(draw(10, c"hello").line, "hello");
    assert_eq!(draw(10, c"").line, "");
}

#[test]
fn a_run_of_hashes_is_halved() {
    assert_eq!(draw(10, c"#").line, "#");
    assert_eq!(draw(10, c"##").line, "#");
    assert_eq!(draw(10, c"###").line, "##");
    assert_eq!(draw(10, c"####").line, "##");
    assert_eq!(draw(10, c"#a").line, "#a");
}

#[test]
fn an_escaped_style_is_drawn_as_the_text_it_hides() {
    assert_eq!(draw(20, c"##[fg=red]x").line, "#[fg=red]x");
    assert_eq!(draw(20, c"###[fg=red]x").line, "#x");
}

#[test]
fn a_style_that_is_not_closed_draws_nothing_at_all() {
    assert_eq!(draw(10, c"abc#[fg=red").line, "");
}

#[test]
fn a_style_that_does_not_parse_is_stepped_over() {
    assert_eq!(draw(10, c"a#[nonsense]b").line, "ab");
}

#[test]
fn an_ignored_style_is_drawn_as_the_text_it_is() {
    assert_eq!(draw(20, c"#[ignore]a#[fg=red]b").line, "a#[fg=red]b");
}

/// An escaped `##[` under `ignore` is the one shape that loses text: the
/// escape is stepped over and then dropped rather than drawn, so only what
/// follows the bracket is left.
#[test]
fn an_escaped_style_under_ignore_loses_its_own_bracket() {
    assert_eq!(draw(20, c"#[ignore]##[fg=red]b").line, "fg=red]b");
}

#[test]
fn a_character_of_more_than_one_byte_is_drawn_whole() {
    assert_eq!(draw(10, c"a\u{4e2d}b").line, "a\u{4e2d}b");
}

#[test]
fn a_byte_that_is_not_a_character_is_stepped_over() {
    assert_eq!(draw(10, c"a\x01b").line, "ab");
    assert_eq!(draw(10, c"a\xffb").line, "ab");
    assert_eq!(draw(10, c"a\xc3b").line, "ab");
}

#[test]
fn text_is_placed_by_the_alignment_it_asks_for() {
    assert_eq!(draw(10, c"#[align=left]ab").line, "ab");
    assert_eq!(draw(10, c"#[align=centre]ab").line, "    ab");
    assert_eq!(draw(10, c"#[align=right]ab").line, "        ab");
    assert_eq!(draw(10, c"#[align=absolute-centre]ab").line, "    ab");
    assert_eq!(
        draw(12, c"l#[align=centre]c#[align=right]r").line,
        "l     c    r"
    );
}

#[test]
fn a_line_too_narrow_gives_up_the_centre_then_the_right_then_the_left() {
    assert_eq!(
        draw(6, c"left#[align=centre]cc#[align=right]rr").line,
        "leftrr"
    );
    assert_eq!(
        draw(4, c"left#[align=centre]cc#[align=right]rr").line,
        "left"
    );
    assert_eq!(draw(2, c"left#[align=centre]cc#[align=right]rr").line, "le");
}

#[test]
fn an_absolute_centre_is_held_to_what_there_is_room_for() {
    assert_eq!(draw(3, c"#[align=absolute-centre]abcde").line, "abc");
}

#[test]
fn a_list_is_drawn_where_the_list_style_left_it() {
    assert_eq!(
        draw(20, c"#[align=left]a#[list=on]bc#[nolist]d").line,
        "abcd"
    );
    assert_eq!(
        draw(20, c"#[align=left]a#[list=on]bc#[nolist]d#[align=right]r").line,
        "abcd               r"
    );
}

/// A list whose alignment was never named is thrown away: the drawing is
/// picked by the alignment in force when the list opened, and the default
/// one draws the left, centre, right and absolute-centre screens only.
#[test]
fn a_list_under_no_alignment_at_all_is_dropped() {
    assert_eq!(draw(20, c"a#[list=on]bc#[nolist]d").line, "a");
}

#[test]
fn a_list_that_does_not_fit_is_scrolled_around_its_focus() {
    assert_eq!(
        draw(
            6,
            c"#[align=left,list=on]ab#[list=focus]cd#[list=on]ef#[nolist]"
        )
        .line,
        "abcdef"
    );
    assert_eq!(
        draw(
            4,
            c"#[align=left,list=on]ab#[list=focus]cd#[list=on]ef#[nolist]"
        )
        .line,
        "bcde"
    );
}

#[test]
fn the_list_markers_are_drawn_at_each_end_of_a_scrolled_list() {
    assert_eq!(
        draw(
            6,
            c"#[align=left,list=on]abc#[list=focus]def#[list=on]ghi#[list=left-marker]<#[list=right-marker]>#[nolist]"
        )
        .line,
        "<cdef>"
    );
    assert_eq!(
        draw(
            6,
            c"#[align=left,list=on]abcdef#[list=focus]ghi#[list=left-marker]<#[list=right-marker]>#[nolist]"
        )
        .line,
        "abcde>"
    );
}

/// A marker named before anything is in the list takes the drawing with
/// it: everything after it goes to the marker's own screen until the next
/// `list=on`, and only that last part is the list.
#[test]
fn a_marker_named_first_takes_the_text_after_it() {
    let format =
        c"#[align=left,list=on]#[list=left-marker]<#[list=right-marker]>abc#[list=focus]def#[list=on]ghi#[nolist]";
    assert_eq!(draw(6, format).line, "ghi");
    assert_eq!(draw(2, format).line, "gh");
}

#[test]
fn a_list_is_drawn_at_the_alignment_in_force_when_it_opened() {
    assert_eq!(
        draw(20, c"#[align=centre]a#[list=on]bc#[nolist]d#[align=right]r").line,
        "       abcd        r"
    );
    assert_eq!(
        draw(20, c"#[align=right]a#[list=on]bc#[nolist]d").line,
        "                abcd"
    );
    assert_eq!(
        draw(20, c"#[align=absolute-centre]a#[list=on]bc#[nolist]d").line,
        "        abcd"
    );
    assert_eq!(
        draw(10, c"#[align=centre,list=on]ab#[nolist]").line,
        "    ab"
    );
    assert_eq!(
        draw(10, c"#[align=right,list=on]ab#[nolist]").line,
        "        ab"
    );
    assert_eq!(
        draw(10, c"#[align=absolute-centre,list=on]ab#[nolist]").line,
        "    ab"
    );
}

#[test]
fn a_list_with_no_room_for_it_is_given_up_before_the_text_around_it() {
    assert_eq!(
        draw(4, c"#[align=centre]left#[list=on]bc#[nolist]d").line,
        "left"
    );
    assert_eq!(
        draw(4, c"#[align=right]left#[list=on]bc#[nolist]d").line,
        "left"
    );
    assert_eq!(
        draw(4, c"#[align=absolute-centre]left#[list=on]bc#[nolist]d").line,
        "left"
    );
    assert_eq!(
        draw(
            2,
            c"#[align=left]l#[list=on]xy#[nolist]after#[align=centre]c#[align=right]r"
        )
        .line,
        "lr"
    );
}

/// An absolute centre is drawn last and over everything else, so a line
/// with no room for the rest is all absolute centre.
#[test]
fn an_absolute_centre_is_drawn_over_whatever_else_there_was_room_for() {
    assert_eq!(
        draw(
            1,
            c"#[align=left]l#[list=on]xy#[nolist]a#[align=absolute-centre]ZZ"
        )
        .line,
        "Z"
    );
    assert_eq!(
        draw(
            2,
            c"#[align=centre]l#[list=on]xy#[nolist]a#[align=absolute-centre]ZZ"
        )
        .line,
        "ZZ"
    );
    assert_eq!(
        draw(
            2,
            c"#[align=right]l#[list=on]xy#[nolist]a#[align=absolute-centre]ZZ"
        )
        .line,
        "ZZ"
    );
    assert_eq!(
        draw(
            2,
            c"#[align=absolute-centre]l#[list=on]xy#[nolist]a#[align=absolute-centre]ZZ"
        )
        .line,
        "la"
    );
}

/// An empty list draws nothing at all, whatever came after it: the text
/// after the list is copied into the left screen, but the width of that
/// screen was taken before the copy, so nothing of it is drawn.
#[test]
fn an_empty_list_takes_the_text_after_it_down_with_it() {
    assert_eq!(draw(20, c"#[align=left,list=on]#[nolist]after").line, "");
}

#[test]
fn a_fill_paints_the_whole_line_behind_the_text() {
    assert_eq!(draw(6, c"#[fill=red]ab").line, "ab    ");
}

#[test]
fn a_default_style_can_be_pushed_popped_and_set() {
    assert_eq!(draw(20, c"#[push-default]a#[pop-default]b").line, "ab");
    assert_eq!(draw(20, c"#[fg=red,set-default]a#[default]b").line, "ab");
}

#[test]
fn the_base_colours_win_when_the_caller_asks_for_them() {
    assert_eq!(draw_over(10, c"#[fg=red]ab", 1, true).line, "ab");
}

/// A range ends one column past the last it covers, and is then held to
/// what of the screen it is on was drawn, which is why the ranges below
/// that reach the end of a two-column line stop at two rather than three.
#[test]
fn a_range_is_reported_with_the_columns_it_covers() {
    assert_eq!(
        draw(10, c"#[range=left]ab#[norange]cd").ranges,
        vec!["1|0| 0-3".to_string()]
    );
    assert_eq!(
        draw(10, c"#[range=right]ab#[norange]").ranges,
        vec!["2|0| 0-2".to_string()]
    );
    assert_eq!(
        draw(10, c"#[range=pane|%3]ab#[norange]").ranges,
        vec!["3|3| 0-2".to_string()]
    );
    assert_eq!(
        draw(10, c"#[range=window|4]ab#[norange]").ranges,
        vec!["4|4| 0-2".to_string()]
    );
    assert_eq!(
        draw(10, c"#[range=session|$5]ab#[norange]").ranges,
        vec!["5|5| 0-2".to_string()]
    );
    assert_eq!(
        draw(10, c"#[range=user|name]ab#[norange]").ranges,
        vec!["6|0|name 0-2".to_string()]
    );
    assert_eq!(
        draw(10, c"#[range=control|3]ab#[norange]").ranges,
        vec!["7|3| 0-2".to_string()]
    );
}

/// A range only ends where the style changes to one it is not, so a second
/// range of the same kind and argument runs on into the first.
#[test]
fn a_range_of_the_same_kind_carries_on_rather_than_starting_again() {
    assert_eq!(
        draw(10, c"#[range=left]abc#[range=left]de#[norange]").ranges,
        vec!["1|0| 0-5".to_string()]
    );
    assert_eq!(
        draw(10, c"#[range=user|a]x#[range=user|b]y#[norange]").ranges,
        vec!["6|0|a 0-2".to_string(), "6|0|b 1-2".to_string()]
    );
    assert_eq!(
        draw(10, c"#[range=pane|%1]x#[range=pane|%2]y#[norange]").ranges,
        vec!["3|1| 0-2".to_string(), "3|2| 1-2".to_string()]
    );
}

#[test]
fn a_range_that_never_ends_reaches_the_end_of_what_was_drawn() {
    assert_eq!(draw(10, c"#[range=left]abc").ranges, Vec::<String>::new());
    assert_eq!(
        draw(10, c"#[range=left]abc#[range=right]de").ranges,
        vec!["1|0| 0-4".to_string()]
    );
}

#[test]
fn a_range_trimmed_away_by_the_width_is_dropped() {
    assert_eq!(
        draw(2, c"#[range=left]abcd#[norange]").ranges,
        vec!["1|0| 0-2".to_string()]
    );
    assert_eq!(
        draw(4, c"#[range=left]abcd#[norange]").ranges,
        vec!["1|0| 0-4".to_string()]
    );
    assert_eq!(
        draw(2, c"ab#[range=left]cd#[norange]").ranges,
        Vec::<String>::new()
    );
}

/// A range whose start is cut off by the right alignment keeps the part of
/// itself that was drawn.
#[test]
fn a_range_cut_at_its_start_keeps_what_is_left_of_it() {
    let drawn = draw(4, c"#[align=right,range=left]abcdefgh#[norange]");
    assert_eq!(drawn.line, "efgh");
    assert_eq!(drawn.ranges, vec!["1|0| 0-4".to_string()]);
}

/// A range open when the list opens or closes is thrown away, so only the
/// part after the last list boundary is reported.
#[test]
fn a_range_open_across_a_list_boundary_is_started_again() {
    assert_eq!(
        draw(
            20,
            c"#[align=left,range=left]a#[list=on]b#[list=focus]c#[list=on]d#[list=left-marker]<#[list=right-marker]>#[nolist]e#[norange]"
        )
        .ranges,
        vec!["1|0| 4-5".to_string()]
    );
    assert_eq!(
        draw(
            20,
            c"#[align=left,list=on]a#[range=left]b#[list=focus]c#[norange]#[nolist]"
        )
        .ranges,
        vec!["1|0| 1-3".to_string()]
    );
}

/// A caller that wants no ranges back is given none, and the styles that
/// would have opened them are otherwise drawn as usual.
#[test]
fn a_caller_that_asks_for_no_ranges_gets_none() {
    let drawn = draw_over(10, c"#[range=left]ab#[norange]cd", 0, false);
    assert_eq!(drawn.line, "abcd");
    assert_eq!(drawn.ranges, Vec::<String>::new());
}

/// A marker outside the list, or a second one of the same side, is not
/// taken: the drawing carries on where it was.
#[test]
fn a_marker_that_cannot_be_taken_leaves_the_drawing_where_it_is() {
    assert_eq!(
        draw(20, c"#[align=left,list=left-marker]<#[list=on]ab#[nolist]").line,
        "<ab"
    );
    assert_eq!(
        draw(
            20,
            c"#[align=left,list=on]a#[list=left-marker]<#[list=left-marker]!#[nolist]"
        )
        .line,
        "a"
    );
}

#[test]
fn a_style_that_is_not_closed_throws_the_ranges_away() {
    assert_eq!(
        draw(10, c"#[range=left]ab#[norange]cd#[fg=red").ranges,
        Vec::<String>::new()
    );
}

#[test]
fn the_width_of_a_string_leaves_out_its_styles() {
    assert_eq!(width(c""), 0);
    assert_eq!(width(c"abc"), 3);
    assert_eq!(width(c"#[fg=red]abc"), 3);
    assert_eq!(width(c"##abc"), 4);
    assert_eq!(width(c"###abc"), 5);
    assert_eq!(width(c"###[fg=red]abc"), 4);
    assert_eq!(width(c"abc#[fg=red"), 0);
    assert_eq!(width(c"a\u{4e2d}b"), 4);
    assert_eq!(width(c"a\x01b"), 2);
}

/// An even run of hashes in front of a bracket is counted as the hashes it
/// stands for, but the bracket is then read as text rather than as the
/// start of a style, so the whole of it counts too.
#[test]
fn an_escaped_style_counts_as_the_text_it_is_drawn_as() {
    assert_eq!(width(c"##[fg=red]"), 9);
}

#[test]
fn trimming_keeps_the_styles_and_counts_only_the_text() {
    assert_eq!(trim(c"abcdef", 3), ("abc".to_string(), "def".to_string()));
    assert_eq!(
        trim(c"abcdef", 10),
        ("abcdef".to_string(), "abcdef".to_string())
    );
    assert_eq!(trim(c"abcdef", 0), ("".to_string(), "".to_string()));
    assert_eq!(
        trim(c"#[fg=red]abcdef", 3),
        ("#[fg=red]abc".to_string(), "#[fg=red]def".to_string())
    );
    assert_eq!(
        trim(c"ab#[fg=red]cd", 3),
        ("ab#[fg=red]c".to_string(), "b#[fg=red]cd".to_string())
    );
}

#[test]
fn trimming_halves_the_hashes_it_keeps() {
    assert_eq!(trim(c"##abc", 1), ("##".to_string(), "c".to_string()));
    assert_eq!(trim(c"####abc", 1), ("##".to_string(), "c".to_string()));
    assert_eq!(trim(c"#abc", 2), ("#a".to_string(), "bc".to_string()));
    assert_eq!(
        trim(c"####abc", 3),
        ("####a".to_string(), "abc".to_string())
    );
    assert_eq!(
        trim(c"####abc", 4),
        ("####ab".to_string(), "##abc".to_string())
    );
    assert_eq!(
        trim(c"abc#def", 5),
        ("abc#d".to_string(), "c#def".to_string())
    );
    assert_eq!(
        trim(c"abc#def", 6),
        ("abc#de".to_string(), "bc#def".to_string())
    );
    assert_eq!(
        trim(c"##abc", 4),
        ("##abc".to_string(), "##abc".to_string())
    );
}

/// A string with a style that is never closed has no width at all, so
/// trimming from the right hands it back whole however small the limit,
/// while trimming from the left stops where the style begins.
#[test]
fn trimming_stops_at_a_style_that_is_never_closed() {
    assert_eq!(
        trim(c"abc#[fg=red", 2),
        ("ab".to_string(), "abc#[fg=red".to_string())
    );
    assert_eq!(
        trim(c"ab#[fg=red", 5),
        ("ab".to_string(), "ab#[fg=red".to_string())
    );
}

#[test]
fn trimming_counts_a_wide_character_as_the_columns_it_takes() {
    assert_eq!(trim(c"a\u{4e2d}b", 2), ("a".to_string(), "b".to_string()));
    assert_eq!(
        trim(c"a\u{4e2d}b", 3),
        ("a\u{4e2d}".to_string(), "\u{4e2d}b".to_string())
    );
}

#[test]
fn trimming_steps_over_bytes_that_are_not_characters() {
    assert_eq!(trim(c"a\x01b", 2).0, "ab");
    assert_eq!(trim(c"a\xffb", 2).0, "ab");
    assert_eq!(trim(c"a\xc3b", 2).0, "ab");
    assert_eq!(trim(c"a\x01bcd", 2).1, "cd");
    assert_eq!(trim(c"a\xffbcd", 2).1, "cd");
    assert_eq!(trim(c"a\xc3bcd", 2).1, "bcd");
}
