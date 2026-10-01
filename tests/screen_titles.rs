use hmux::src::screen::{screen_pop_title, screen_push_title, screen_set_path, screen_set_title};
use hmux::src::shared::screen::screen;
use std::ffi::CString;

#[test]
fn borrowed_names_are_owned_and_title_stack_keeps_the_newest_ten() {
    let mut s = screen::empty();
    for index in 0..12 {
        let name = CString::new(format!("title {index}")).unwrap();
        assert_eq!(screen_set_title(&mut s, &name, 0), 1);
        unsafe { screen_push_title(&mut s) };
    }
    assert_eq!(s.titles.len(), 10);
    assert_eq!(screen_set_title(&mut s, c"replacement", 0), 1);
    for index in (2..12).rev() {
        unsafe { screen_pop_title(&mut s) };
        assert_eq!(s.title.to_bytes(), format!("title {index}").as_bytes());
    }
    unsafe { screen_pop_title(&mut s) };
    assert_eq!(s.title.to_bytes(), b"title 2");
    assert!(s.titles.is_empty());

    assert_eq!(screen_set_title(&mut s, c"unsafe#(command)", 1), 1);
    assert_eq!(s.title.to_bytes(), b"unsafe_(command)");
    let invalid = CString::new(vec![0xff]).unwrap();
    assert_eq!(screen_set_title(&mut s, &invalid, 1), 0);
    assert_eq!(s.title.to_bytes(), b"unsafe_(command)");
    assert_eq!(screen_set_path(&mut s, c"/tmp/#(command)"), 1);
    assert_eq!(screen_set_path(&mut s, &invalid), 0);
    assert_eq!(s.path.as_deref().unwrap().to_bytes(), b"/tmp/_(command)");
}
