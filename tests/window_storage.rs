use hmux2::src::window::*;

#[test]
fn global_lookup_preserves_window_identity() {
    unsafe {
        let head = &raw mut windows;
        assert!((*head).storage.is_none());
        let mut window = window::default();
        window.id = 123;
        window.entry.owner = None;
        assert!(windows_insert(head, &mut window).is_null());
        assert_eq!(window_find_by_id(123), &mut window as *mut _);
        assert!(window_find_by_id(124).is_null());
        assert_eq!(windows_remove(head, &mut window), &mut window as *mut _);
        assert!(window_find_by_id(123).is_null());
        assert!((*head).storage.is_none());
    }
}
