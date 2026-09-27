use hmux2::src::screen::screen_mode_display;
use hmux2::src::shared::screen::{
    ALL_MODES, MODE_CURSOR, MODE_CURSOR_VERY_VISIBLE, MODE_MOUSE_UTF8, MODE_SYNC,
};

#[test]
fn mode_descriptions_keep_order_special_values_and_independent_state() {
    let first = screen_mode_display(MODE_CURSOR | MODE_SYNC);
    let second = screen_mode_display(MODE_CURSOR_VERY_VISIBLE | MODE_MOUSE_UTF8);
    assert_eq!(
        format!("{first};{second};{first}"),
        "CURSOR,SYNC;CURSOR_VERY_VISIBLE,MOUSE_UTF8;CURSOR,SYNC"
    );
    assert_eq!(screen_mode_display(0).to_string(), "NONE");
    assert_eq!(screen_mode_display(ALL_MODES).to_string(), "ALL");
    assert_eq!(screen_mode_display(1 << 30).to_string(), "");
    assert_eq!(
        screen_mode_display((1 << 30) | MODE_CURSOR).to_string(),
        "CURSOR"
    );
    assert_eq!(
        screen_mode_display(-1).to_string(),
        concat!(
            "CURSOR,INSERT,KCURSOR,KKEYPAD,WRAP,MOUSE_STANDARD,MOUSE_BUTTON,",
            "CURSOR_BLINKING,CURSOR_VERY_VISIBLE,CURSOR_BLINKING_SET,MOUSE_UTF8,",
            "MOUSE_SGR,BRACKETPASTE,FOCUSON,MOUSE_ALL,ORIGIN,CRLF,KEYS_EXTENDED,",
            "KEYS_EXTENDED_2,THEME_UPDATES,SYNC"
        )
    );
}
