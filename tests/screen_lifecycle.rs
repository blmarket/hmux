use hmux2::src::grid::{grid_default_cell, grid_get_cell, grid_set_cell};
use hmux2::src::hyperlinks::{hyperlinks_get, hyperlinks_put};
use hmux2::src::options::{options_create, options_default, options_free};
use hmux2::src::options_table::options_table;
use hmux2::src::screen::{
    screen_alternate_on, screen_free, screen_init, screen_push_title, screen_reinit,
    screen_set_path, screen_set_progress_bar, screen_set_selection, screen_set_title,
};
use hmux2::src::screen_write::screen_write_make_list;
use hmux2::src::shared::display::{PROGRESS_BAR_HIDDEN, PROGRESS_BAR_PAUSED};
use hmux2::src::shared::key::MODEKEY_VI;
use hmux2::src::shared::screen::{screen, MODE_CRLF, MODE_CURSOR, MODE_WRAP};
use hmux2::src::tmux::global_options;

#[test]
fn reinitialization_leaves_alternate_mode_and_clears_transient_state() {
    unsafe {
        let previous = global_options;
        let mut global_options_owner = options_create(None);
        global_options = &raw mut *global_options_owner;
        let definition = options_table
            .iter()
            .find(|entry| entry.name == Some(c"extended-keys"))
            .unwrap();
        options_default(global_options, definition);

        let mut s = screen::empty();
        screen_init(&mut s, 17, 4, 10);
        assert_eq!(s.tabs, [0, 1, 1]);
        screen_write_make_list(&mut s);
        let mut cell = grid_default_cell;
        cell.data.data[0] = b'A';
        grid_set_cell(s.grid_mut(), 0, 0, &cell);
        s.cx = 3;
        s.cy = 2;
        screen_alternate_on(&mut s, &cell, 1);
        s.tabs.fill(0);
        screen_set_selection(&mut s, 0, 0, 3, 1, 0, 0, MODEKEY_VI, &cell);
        screen_set_title(&mut s, c"kept title", 0);
        screen_push_title(&mut s);
        screen_set_path(&mut s, c"/kept/path");
        screen_set_progress_bar(&mut s, PROGRESS_BAR_PAUSED, 75);
        let link = hyperlinks_put(
            s.hyperlinks.as_ref().unwrap(),
            c"https://example.org",
            Some(c"saved"),
        );
        assert!(hyperlinks_get(s.hyperlinks.as_ref().unwrap(), link).is_some());
        s.mode |= MODE_CRLF;

        screen_reinit(&mut s, 1);
        assert!(s.saved_grid.is_none());
        assert!(s.sel.is_none());
        assert!(s.titles.is_empty());
        assert_eq!(s.tabs, [0, 1, 1]);
        assert_eq!((s.cx, s.cy, s.rupper, s.rlower), (0, 0, 0, 3));
        assert_eq!((s.saved_cx, s.saved_cy), (u32::MAX, u32::MAX));
        assert_eq!(s.mode, MODE_CURSOR | MODE_WRAP | MODE_CRLF);
        assert_eq!(s.progress_bar.state, PROGRESS_BAR_HIDDEN);
        assert_eq!(s.progress_bar.progress, 0);
        assert_eq!(s.title.to_bytes(), b"kept title");
        assert_eq!(s.path.as_deref().unwrap().to_bytes(), b"/kept/path");
        assert!(hyperlinks_get(s.hyperlinks.as_ref().unwrap(), link).is_none());
        grid_get_cell(s.grid(), 0, 0, &mut cell);
        assert_eq!(cell.data.data[0], b' ');
        assert!(s.write_list.is_some());

        screen_free(&mut s);
        assert!(s.grid.is_none());
        assert!(s.saved_grid.is_none());
        assert!(s.write_list.is_none());
        assert!(s.hyperlinks.is_none());
        assert!(s.title.is_empty());
        assert!(s.path.is_none());
        assert!(s.tabs.is_empty());
        options_free(global_options_owner);
        global_options = previous;
    }
}
