use hmux2::src::server_client::Client as _;
use hmux2::src::shared::client::ClientRef;
use hmux2::src::shared::style::{style_range, STYLE_RANGE_CONTROL};
use hmux2::src::status::status_get_range;

#[test]
fn status_ranges_are_bounded_snapshots() {
    unsafe {
        let owner = ClientRef::allocate();
        owner.borrow_status_mut().entries[0]
            .ranges
            .push(Box::new(style_range {
                type_0: STYLE_RANGE_CONTROL,
                argument: 7,
                string: [0; 16],
                start: 2,
                end: 5,
            }));
        assert!(status_get_range(&owner, 1, 0).is_none());
        assert!(status_get_range(&owner, 5, 0).is_none());
        assert!(status_get_range(&owner, 2, 1).is_none());
        assert!(status_get_range(&owner, 2, u32::MAX).is_none());
        let range = status_get_range(&owner, 2, 0).unwrap();
        owner.borrow_status_mut().entries[0].ranges.clear();
        assert!(status_get_range(&owner, 2, 0).is_none());
        drop(owner);
        assert_eq!(range.argument, 7);
        assert_eq!((range.start, range.end), (2, 5));
    }
}
