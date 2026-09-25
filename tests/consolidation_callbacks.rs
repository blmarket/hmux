//! Function assignments check argument types, return types, and the C ABI.
use hmux2::src::shared::{abi::__compar_fn_t, prompt::*};
use std::ffi::{c_int, c_void};

unsafe extern "C" fn compare(left: *const c_void, right: *const c_void) -> c_int {
    assert!(left.is_null());
    assert!(right.is_null());
    -1
}

#[test]
fn local_prompt_callbacks_are_owned_and_comparators_keep_the_c_abi() {
    let free: prompt_free_cb = Some(Box::new(|| {}));
    let callback: prompt_input_cb = Some(Box::new(|text, key| {
        assert!(text.is_none());
        assert_eq!(key, PROMPT_KEY_CLOSE);
        PROMPT_CLOSE
    }));
    let comparator: __compar_fn_t = Some(compare);
    unsafe {
        free.unwrap()();
        assert_eq!(
            callback.unwrap()(None, PROMPT_KEY_CLOSE),
            PROMPT_CLOSE
        );
        assert_eq!(comparator.unwrap()(std::ptr::null(), std::ptr::null()), -1);
    }
    let free: prompt_free_cb = None;
    let input: prompt_input_cb = None;
    let comparator: __compar_fn_t = None;
    assert!(free.is_none() && input.is_none() && comparator.is_none());
}
