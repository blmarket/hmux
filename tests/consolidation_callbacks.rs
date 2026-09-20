//! Function assignments check argument types, return types, and the C ABI.
use hmux2::src::shared::{abi::__compar_fn_t, prompt::*};
use std::ffi::{c_char, c_int, c_void};

unsafe extern "C" fn release(data: *mut c_void) {
    assert!(data.is_null());
}

unsafe extern "C" fn input(
    data: *mut c_void,
    text: *const c_char,
    key: prompt_key_result,
) -> prompt_result {
    assert!(data.is_null());
    assert!(text.is_null());
    assert_eq!(key, PROMPT_KEY_CLOSE);
    PROMPT_CLOSE
}

unsafe extern "C" fn compare(left: *const c_void, right: *const c_void) -> c_int {
    assert!(left.is_null());
    assert!(right.is_null());
    -1
}

#[test]
fn callbacks_retain_nullable_c_function_signatures() {
    let free: prompt_free_cb = Some(release);
    let callback: prompt_input_cb = Some(input);
    let comparator: __compar_fn_t = Some(compare);
    unsafe {
        free.unwrap()(std::ptr::null_mut());
        assert_eq!(
            callback.unwrap()(std::ptr::null_mut(), std::ptr::null(), PROMPT_KEY_CLOSE),
            PROMPT_CLOSE
        );
        assert_eq!(comparator.unwrap()(std::ptr::null(), std::ptr::null()), -1);
    }
    let free: prompt_free_cb = None;
    let input: prompt_input_cb = None;
    let comparator: __compar_fn_t = None;
    assert!(free.is_none() && input.is_none() && comparator.is_none());
}
