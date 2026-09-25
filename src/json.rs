use crate::src::ffi::libc::{__ctype_b_loc, __errno_location, strlen, strncmp, strtoll};
use crate::src::log::fatalx;
use crate::src::reactor::{
    evbuffer_add, evbuffer_add_printf, evbuffer_free, evbuffer_get_length, evbuffer_new,
    evbuffer_pullup,
};
use crate::src::shared::abi::*;
use crate::src::shared::abi::{int64_t, ssize_t};
use crate::src::shared::ctype::{_ISdigit, _ISspace, _ISxdigit};
use crate::src::shared::event::*;
use crate::src::shared::json::{
    json_fields, json_fields_storage, json_members, json_members_storage, json_node, json_node_type,
};
use crate::src::shared::tree::RB_NEGINF;
use std::ffi::CStr;
use std::ffi::CString;

macro_rules! json_format_cause {
    ($cause:expr, $fmt:expr, $arg:expr $(,)?) => {{
        let cause = $cause;
        unsafe {
            if !cause.is_null() {
                *cause = Some(json_one_arg_cause(
                    CStr::from_ptr($fmt),
                    CStr::from_ptr($arg),
                ));
            }
        }
    }};
}

fn json_one_arg_cause(fmt: &CStr, arg: &CStr) -> CString {
    let fmt = fmt.to_bytes();
    let at = fmt
        .windows(2)
        .position(|part| part == b"%s")
        .expect("JSON diagnostic has %s");
    let arg = arg.to_bytes();
    let mut message = Vec::with_capacity(fmt.len() + arg.len());
    message.extend_from_slice(&fmt[..at]);
    message.extend_from_slice(arg);
    message.extend_from_slice(&fmt[at + 2..]);
    CString::new(message).expect("JSON diagnostic contains no NUL")
}

pub const NODE_ARRAY: json_node_type = 4;
pub const NODE_OBJECT: json_node_type = 3;
pub const NODE_BOOLEAN: json_node_type = 2;
pub const NODE_NUMBER: json_node_type = 1;
pub const NODE_STRING: json_node_type = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct json_parse_ctx {
    pub input: *const ::core::ffi::c_char,
    pub cause: *mut Option<CString>,
    pub depth: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct json_token {
    pub type_0: json_token_type,
    pub offset: ::core::ffi::c_int,
    pub len: ::core::ffi::c_int,
}
pub type json_token_type = ::core::ffi::c_uint;
pub const TOK_EOF: json_token_type = 8;
pub const TOK_VALUE: json_token_type = 7;
pub const TOK_QUOTE: json_token_type = 6;
pub const TOK_COLON: json_token_type = 5;
pub const TOK_COMMA: json_token_type = 4;
pub const TOK_CLOSEARRAY: json_token_type = 3;
pub const TOK_OPENARRAY: json_token_type = 2;
pub const TOK_CLOSEOBJECT: json_token_type = 1;
pub const TOK_OPENOBJECT: json_token_type = 0;
pub const ERROR_CTX_LEN: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const PARSE_DEPTH_MAX: ::core::ffi::c_int = 200 as ::core::ffi::c_int;

fn json_set_string(node: &mut json_node, string: CString) {
    node.c2rust_unnamed.str_0 = ::core::ptr::null_mut();
    node.string = Some(string);
    node.c2rust_unnamed.str_0 = node.string.as_ref().unwrap().as_ptr().cast_mut();
}

unsafe fn json_fields_insert(head: &mut json_fields, elm: &mut json_node) -> *mut json_node {
    if elm.key.is_none() {
        return ::core::ptr::null_mut::<json_node>();
    }
    if head.entries.is_null() {
        head.entries = Box::into_raw(Box::new(json_fields_storage::default()));
    }
    let key = elm.key.as_ref().unwrap().as_bytes().to_vec();
    match (*head.entries).entries.entry(key) {
        std::collections::btree_map::Entry::Occupied(entry) => *entry.get(),
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(&raw mut *elm);
            ::core::ptr::null_mut::<json_node>()
        }
    }
}

unsafe fn json_fields_remove(head: &mut json_fields, elm: &json_node) -> *mut json_node {
    if head.entries.is_null() || elm.key.is_none() {
        return ::core::ptr::null_mut::<json_node>();
    }
    let key = elm.key.as_ref().unwrap().as_bytes();
    let entries = &mut (*head.entries).entries;
    if entries.get(key).copied() != Some(elm as *const json_node as *mut json_node) {
        return ::core::ptr::null_mut::<json_node>();
    }
    entries
        .remove(key)
        .unwrap_or(::core::ptr::null_mut::<json_node>())
}

unsafe fn json_fields_minmax(head: &json_fields, val: ::core::ffi::c_int) -> *mut json_node {
    if head.entries.is_null() {
        return ::core::ptr::null_mut::<json_node>();
    }
    let entry = if val < 0 {
        (*head.entries).entries.values().next()
    } else {
        (*head.entries).entries.values().next_back()
    };
    entry
        .copied()
        .unwrap_or(::core::ptr::null_mut::<json_node>())
}

unsafe fn json_fields_find(head: &json_fields, key: &CStr) -> *mut json_node {
    if head.entries.is_null() {
        return ::core::ptr::null_mut::<json_node>();
    }
    (*head.entries)
        .entries
        .get(key.to_bytes())
        .copied()
        .unwrap_or(::core::ptr::null_mut::<json_node>())
}

unsafe fn json_fields_next(head: &json_fields, elm: &json_node) -> *mut json_node {
    if head.entries.is_null() || elm.key.is_none() {
        return ::core::ptr::null_mut::<json_node>();
    }
    let key = elm.key.as_ref().unwrap().as_bytes().to_vec();
    (*head.entries)
        .entries
        .range((std::ops::Bound::Excluded(key), std::ops::Bound::Unbounded))
        .next()
        .map(|(_, entry)| *entry)
        .unwrap_or(::core::ptr::null_mut::<json_node>())
}

unsafe fn json_members_first(head: &json_members) -> *mut json_node {
    if head.storage.is_null() {
        return ::core::ptr::null_mut::<json_node>();
    }
    (*head.storage)
        .members
        .first()
        .copied()
        .unwrap_or(::core::ptr::null_mut::<json_node>())
}

unsafe fn json_members_next(head: &json_members, elm: &json_node) -> *mut json_node {
    if head.storage.is_null() {
        return ::core::ptr::null_mut::<json_node>();
    }
    let storage = &*head.storage;
    storage
        .indices
        .get(&(elm as *const json_node as *mut json_node))
        .and_then(|index| storage.members.get(index + 1))
        .copied()
        .unwrap_or(::core::ptr::null_mut::<json_node>())
}

unsafe fn json_members_push(head: &mut json_members, elm: &mut json_node) {
    if head.storage.is_null() {
        return;
    }
    let storage = &mut *head.storage;
    let index = storage.members.len();
    let elm = &raw mut *elm;
    storage.members.push(elm);
    storage.indices.insert(elm, index);
}

pub unsafe fn json_parse(
    mut input: *const ::core::ffi::c_char,
    mut cause: *mut Option<CString>,
) -> *mut json_node {
    let mut pctx: json_parse_ctx = json_parse_ctx {
        input: ::core::ptr::null::<::core::ffi::c_char>(),
        cause: ::core::ptr::null_mut::<Option<CString>>(),
        depth: 0,
    };
    if *input as ::core::ffi::c_int == '\0' as i32 {
        json_error(cause.as_mut(), c"empty input", None);
        return ::core::ptr::null_mut::<json_node>();
    }
    let tokens = match json_tokenize_input(input, cause) {
        Some(tokens) => tokens,
        None => return ::core::ptr::null_mut::<json_node>(),
    };
    pctx.input = input;
    pctx.cause = cause;
    pctx.depth = 0 as ::core::ffi::c_int;
    json_parse_tokens(&tokens, &raw mut pctx)
}
#[no_mangle]
pub unsafe extern "C" fn json_find(
    mut jn: *mut json_node,
    mut key: *const ::core::ffi::c_char,
) -> *mut json_node {
    if (*jn).type_0 as ::core::ffi::c_uint
        != NODE_OBJECT as ::core::ffi::c_int as ::core::ffi::c_uint
        || key.is_null()
    {
        return ::core::ptr::null_mut::<json_node>();
    }
    return json_fields_find(&(*jn).c2rust_unnamed.fields, CStr::from_ptr(key));
}
#[no_mangle]
pub unsafe extern "C" fn json_array_first(mut jn: *mut json_node) -> *mut json_node {
    if (*jn).type_0 as ::core::ffi::c_uint
        != NODE_ARRAY as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return ::core::ptr::null_mut::<json_node>();
    }
    return json_members_first(&(*jn).c2rust_unnamed.members);
}
#[no_mangle]
pub unsafe extern "C" fn json_array_next(mut member: *mut json_node) -> *mut json_node {
    if member.is_null()
        || (*member).parent.is_null()
        || (*(*member).parent).type_0 as ::core::ffi::c_uint
            != NODE_ARRAY as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return ::core::ptr::null_mut::<json_node>();
    }
    return json_members_next(&(*(*member).parent).c2rust_unnamed.members, &*member);
}
#[no_mangle]
pub unsafe extern "C" fn json_get_string(
    mut jn: *mut json_node,
    mut s: *mut *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if (*jn).type_0 as ::core::ffi::c_uint
        != NODE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return -(1 as ::core::ffi::c_int);
    }
    *s = (*jn).c2rust_unnamed.str_0;
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn json_get_number(
    mut jn: *mut json_node,
    mut i: *mut int64_t,
) -> ::core::ffi::c_int {
    if (*jn).type_0 as ::core::ffi::c_uint
        != NODE_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return -(1 as ::core::ffi::c_int);
    }
    *i = (*jn).c2rust_unnamed.num;
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn json_get_boolean(
    mut jn: *mut json_node,
    mut b: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if (*jn).type_0 as ::core::ffi::c_uint
        != NODE_BOOLEAN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return -(1 as ::core::ffi::c_int);
    }
    *b = (*jn).c2rust_unnamed.boolean;
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn json_get_object(
    mut jn: *mut json_node,
    mut o: *mut *mut json_node,
) -> ::core::ffi::c_int {
    if (*jn).type_0 as ::core::ffi::c_uint
        != NODE_OBJECT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return -(1 as ::core::ffi::c_int);
    }
    *o = jn;
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn json_get_array(
    mut jn: *mut json_node,
    mut a: *mut *mut json_node,
) -> ::core::ffi::c_int {
    if (*jn).type_0 as ::core::ffi::c_uint
        != NODE_ARRAY as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return -(1 as ::core::ffi::c_int);
    }
    *a = jn;
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn json_find_string(
    mut jn: *mut json_node,
    mut key: *const ::core::ffi::c_char,
    mut out: *mut *const ::core::ffi::c_char,
    mut cause: *mut Option<CString>,
) -> ::core::ffi::c_int {
    let mut field: *mut json_node = ::core::ptr::null_mut::<json_node>();
    field = json_find(jn, key);
    if field.is_null() {
        if !cause.is_null() {
            json_format_cause!(
                cause,
                b"key \"%s\" not found\0" as *const u8 as *const ::core::ffi::c_char,
                key,
            );
        }
        return -(1 as ::core::ffi::c_int);
    }
    if (*field).type_0 as ::core::ffi::c_uint
        != NODE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if !cause.is_null() {
            json_format_cause!(
                cause,
                b"key \"%s\" expected a string\0" as *const u8 as *const ::core::ffi::c_char,
                key,
            );
        }
        return -(1 as ::core::ffi::c_int);
    }
    *out = (*field).c2rust_unnamed.str_0;
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn json_find_number(
    mut jn: *mut json_node,
    mut key: *const ::core::ffi::c_char,
    mut out: *mut int64_t,
    mut cause: *mut Option<CString>,
) -> ::core::ffi::c_int {
    let mut field: *mut json_node = ::core::ptr::null_mut::<json_node>();
    field = json_find(jn, key);
    if field.is_null() {
        if !cause.is_null() {
            json_format_cause!(
                cause,
                b"key \"%s\" not found\0" as *const u8 as *const ::core::ffi::c_char,
                key,
            );
        }
        return -(1 as ::core::ffi::c_int);
    }
    if (*field).type_0 as ::core::ffi::c_uint
        != NODE_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if !cause.is_null() {
            json_format_cause!(
                cause,
                b"key \"%s\" expected a number\0" as *const u8 as *const ::core::ffi::c_char,
                key,
            );
        }
        return -(1 as ::core::ffi::c_int);
    }
    *out = (*field).c2rust_unnamed.num;
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn json_find_boolean(
    mut jn: *mut json_node,
    mut key: *const ::core::ffi::c_char,
    mut out: *mut ::core::ffi::c_int,
    mut cause: *mut Option<CString>,
) -> ::core::ffi::c_int {
    let mut field: *mut json_node = ::core::ptr::null_mut::<json_node>();
    field = json_find(jn, key);
    if field.is_null() {
        if !cause.is_null() {
            json_format_cause!(
                cause,
                b"key \"%s\" not found\0" as *const u8 as *const ::core::ffi::c_char,
                key,
            );
        }
        return -(1 as ::core::ffi::c_int);
    }
    if (*field).type_0 as ::core::ffi::c_uint
        != NODE_BOOLEAN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if !cause.is_null() {
            json_format_cause!(
                cause,
                b"key \"%s\" expected a boolean\0" as *const u8 as *const ::core::ffi::c_char,
                key,
            );
        }
        return -(1 as ::core::ffi::c_int);
    }
    *out = (*field).c2rust_unnamed.boolean;
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn json_find_object(
    mut jn: *mut json_node,
    mut key: *const ::core::ffi::c_char,
    mut out: *mut *mut json_node,
    mut cause: *mut Option<CString>,
) -> ::core::ffi::c_int {
    let mut field: *mut json_node = ::core::ptr::null_mut::<json_node>();
    field = json_find(jn, key);
    if field.is_null() {
        if !cause.is_null() {
            json_format_cause!(
                cause,
                b"key \"%s\" not found\0" as *const u8 as *const ::core::ffi::c_char,
                key,
            );
        }
        return -(1 as ::core::ffi::c_int);
    }
    if (*field).type_0 as ::core::ffi::c_uint
        != NODE_OBJECT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if !cause.is_null() {
            json_format_cause!(
                cause,
                b"key \"%s\" expected an object\0" as *const u8 as *const ::core::ffi::c_char,
                key,
            );
        }
        return -(1 as ::core::ffi::c_int);
    }
    *out = field;
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn json_find_array(
    mut jn: *mut json_node,
    mut key: *const ::core::ffi::c_char,
    mut out: *mut *mut json_node,
    mut cause: *mut Option<CString>,
) -> ::core::ffi::c_int {
    let mut field: *mut json_node = ::core::ptr::null_mut::<json_node>();
    field = json_find(jn, key);
    if field.is_null() {
        if !cause.is_null() {
            json_format_cause!(
                cause,
                b"key \"%s\" not found\0" as *const u8 as *const ::core::ffi::c_char,
                key,
            );
        }
        return -(1 as ::core::ffi::c_int);
    }
    if (*field).type_0 as ::core::ffi::c_uint
        != NODE_ARRAY as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if !cause.is_null() {
            json_format_cause!(
                cause,
                b"key \"%s\" expected an array\0" as *const u8 as *const ::core::ffi::c_char,
                key,
            );
        }
        return -(1 as ::core::ffi::c_int);
    }
    *out = field;
    return 0 as ::core::ffi::c_int;
}
unsafe fn json_error(cause: Option<&mut Option<CString>>, reason: &CStr, loc: Option<&CStr>) {
    let Some(cause) = cause else {
        return;
    };
    let Some(loc) = loc.filter(|loc| !loc.to_bytes().is_empty()) else {
        *cause = Some(reason.to_owned());
        return;
    };
    let reason = reason.to_bytes();
    let loc = loc.to_bytes();
    let context_len = loc.len().min(ERROR_CTX_LEN as usize);
    let mut message = Vec::with_capacity(reason.len() + 2 + context_len + 3);
    message.extend_from_slice(reason);
    message.extend_from_slice(b": ");
    message.extend_from_slice(&loc[..context_len]);
    if loc.len() > context_len {
        message.extend_from_slice(b"...");
    }
    *cause = Some(CString::new(message).expect("JSON diagnostic contains no NUL"));
}
unsafe fn json_tokenize_input(
    mut input: *const ::core::ffi::c_char,
    mut cause: *mut Option<CString>,
) -> Option<Vec<json_token>> {
    let mut current_block: u64;
    let mut tokens = Vec::with_capacity(1024);
    let mut type_0: json_token_type = TOK_OPENOBJECT;
    let mut loc: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut start: *const ::core::ffi::c_char = input;
    let mut in_string: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut scan: ::core::ffi::c_int = 0;
    loop {
        if !(*input as ::core::ffi::c_int != '\0' as i32) {
            current_block = 16924917904204750491;
            break;
        }
        loc = input;
        scan = 1 as ::core::ffi::c_int;
        if in_string != 0 && *input as ::core::ffi::c_int != '"' as i32 {
            type_0 = TOK_VALUE;
        } else {
            match *input as ::core::ffi::c_int {
                32 | 9 | 10 | 13 => {
                    input = input.offset(1);
                    continue;
                }
                123 => {
                    type_0 = TOK_OPENOBJECT;
                }
                125 => {
                    type_0 = TOK_CLOSEOBJECT;
                }
                91 => {
                    type_0 = TOK_OPENARRAY;
                }
                93 => {
                    type_0 = TOK_CLOSEARRAY;
                }
                34 => {
                    type_0 = TOK_QUOTE;
                }
                58 => {
                    type_0 = TOK_COLON;
                }
                44 => {
                    type_0 = TOK_COMMA;
                }
                _ => {
                    type_0 = TOK_VALUE;
                }
            }
        }
        if type_0 as ::core::ffi::c_uint == TOK_VALUE as ::core::ffi::c_int as ::core::ffi::c_uint {
            scan = json_tokenize_value(&tokens, loc);
            if scan == -(1 as ::core::ffi::c_int) {
                current_block = 2526103352432062781;
                break;
            }
            input = input.offset((scan - 1 as ::core::ffi::c_int) as isize);
        }
        json_add_token(&mut tokens, type_0, start, loc, scan);
        if type_0 as ::core::ffi::c_uint == TOK_QUOTE as ::core::ffi::c_int as ::core::ffi::c_uint {
            in_string = (in_string == 0) as ::core::ffi::c_int;
        }
        input = input.offset(1);
    }
    match current_block {
        2526103352432062781 => {
            json_error(
                cause.as_mut(),
                c"tokenization error",
                Some(CStr::from_ptr(loc)),
            );
            return None;
        }
        _ => {
            json_add_token(&mut tokens, TOK_EOF, start, loc, 0 as ::core::ffi::c_int);
            return Some(tokens);
        }
    };
}
unsafe fn json_tokenize_value(
    tokens: &[json_token],
    mut loc: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0;
    let mut scan: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let Some(prev) = tokens.last() else {
        return -(1 as ::core::ffi::c_int);
    };
    if prev.type_0 as ::core::ffi::c_uint == TOK_QUOTE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        while *loc.offset(scan as isize) as ::core::ffi::c_int != '"' as i32 {
            if *loc.offset(scan as isize) as ::core::ffi::c_int == '\0' as i32
                || (*loc.offset(scan as isize) as u_char as ::core::ffi::c_int)
                    < 0x20 as ::core::ffi::c_int
            {
                return -(1 as ::core::ffi::c_int);
            }
            if *loc.offset(scan as isize) as ::core::ffi::c_int != '\\' as i32 {
                scan += 1;
            } else {
                scan += 1;
                match *loc.offset(scan as isize) as ::core::ffi::c_int {
                    34 | 92 | 47 | 98 | 102 | 110 | 114 | 116 => {
                        scan += 1;
                    }
                    117 => {
                        i = 1 as ::core::ffi::c_int;
                        while i <= 4 as ::core::ffi::c_int {
                            if *(*__ctype_b_loc())
                                .offset(*loc.offset((scan + i) as isize) as u_char
                                    as ::core::ffi::c_int
                                    as isize) as ::core::ffi::c_int
                                & _ISxdigit as ::core::ffi::c_int as ::core::ffi::c_ushort
                                    as ::core::ffi::c_int
                                == 0
                            {
                                return -(1 as ::core::ffi::c_int);
                            }
                            i += 1;
                        }
                        scan += 5 as ::core::ffi::c_int;
                    }
                    _ => return -(1 as ::core::ffi::c_int),
                }
            }
        }
    } else if prev.type_0 as ::core::ffi::c_uint
        == TOK_COLON as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        loop {
            if *loc.offset(scan as isize) as ::core::ffi::c_int == '\0' as i32 {
                return -(1 as ::core::ffi::c_int);
            }
            scan += 1;
            if !(*loc.offset(scan as isize) as ::core::ffi::c_int != ']' as i32
                && *loc.offset(scan as isize) as ::core::ffi::c_int != '}' as i32
                && *loc.offset(scan as isize) as ::core::ffi::c_int != ',' as i32
                && *(*__ctype_b_loc())
                    .offset(*loc.offset(scan as isize) as u_char as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    & _ISspace as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                    == 0)
            {
                break;
            }
        }
    } else {
        return -(1 as ::core::ffi::c_int);
    }
    return scan;
}
unsafe fn json_add_token(
    tokens: &mut Vec<json_token>,
    type_0: json_token_type,
    input: *const ::core::ffi::c_char,
    loc: *const ::core::ffi::c_char,
    len: ::core::ffi::c_int,
) {
    tokens.push(json_token {
        type_0,
        offset: loc.offset_from(input) as ::core::ffi::c_long as ::core::ffi::c_int,
        len,
    });
}
unsafe extern "C" fn json_create_node(
    mut parent: *mut json_node,
    mut type_0: json_node_type,
    mut key: *const ::core::ffi::c_char,
    mut val: *mut ::core::ffi::c_void,
) -> *mut json_node {
    let mut node: *mut json_node = ::core::ptr::null_mut::<json_node>();
    let mut owner = Box::new(json_node {
        key: if key.is_null() {
            None
        } else {
            Some(CStr::from_ptr(key).to_owned())
        },
        string: None,
        members: None,
        ..json_node::empty()
    });

    node = Box::into_raw(owner).cast::<json_node>();
    (*node).parent = parent;
    (*node).type_0 = type_0;
    if type_0 as ::core::ffi::c_uint == NODE_OBJECT as ::core::ffi::c_int as ::core::ffi::c_uint {
        (*node).c2rust_unnamed.fields.entries =
            Box::into_raw(Box::new(json_fields_storage::default()));
    } else if type_0 as ::core::ffi::c_uint
        == NODE_ARRAY as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let owner = node;
        (*owner).members = Some(Box::new(json_members_storage::default()));
        (*node).c2rust_unnamed.members.storage = (*owner).members.as_deref_mut().unwrap();
    }
    if !val.is_null() {
        json_assign_value(node, val);
    }
    return node;
}
#[no_mangle]
pub unsafe extern "C" fn json_destroy_node(mut node: *mut json_node) {
    let mut field: *mut json_node = ::core::ptr::null_mut::<json_node>();
    let mut field1: *mut json_node = ::core::ptr::null_mut::<json_node>();
    let mut member: *mut json_node = ::core::ptr::null_mut::<json_node>();
    if node.is_null() {
        return;
    }
    match (*node).type_0 as ::core::ffi::c_uint {
        0 => {}
        3 => {
            field = json_fields_minmax(&(*node).c2rust_unnamed.fields, RB_NEGINF);
            while !field.is_null() && {
                field1 = json_fields_next(&(*node).c2rust_unnamed.fields, &*field);
                1 as ::core::ffi::c_int != 0
            } {
                json_fields_remove(&mut (*node).c2rust_unnamed.fields, &*field);
                json_destroy_node(field);
                field = field1;
            }
            if !(*node).c2rust_unnamed.fields.entries.is_null() {
                drop(Box::from_raw((*node).c2rust_unnamed.fields.entries));
                (*node).c2rust_unnamed.fields.entries =
                    ::core::ptr::null_mut::<json_fields_storage>();
            }
        }
        4 => {
            let storage = (*node).c2rust_unnamed.members.storage;
            if !storage.is_null() {
                let members = ::core::mem::take(&mut (*storage).members);
                (*storage).indices.clear();
                for member in members {
                    json_destroy_node(member);
                }
            }
        }
        1 | 2 | _ => {}
    }
    drop(Box::from_raw(node));
}
unsafe extern "C" fn json_assign_value(
    mut node: *mut json_node,
    mut val: *mut ::core::ffi::c_void,
) {
    let mut child: *mut json_node = val as *mut json_node;
    match (*node).type_0 as ::core::ffi::c_uint {
        0 => {
            json_set_string(&mut *node, CStr::from_ptr(val.cast()).to_owned());
        }
        1 => {
            (*node).c2rust_unnamed.num = *(val as *mut int64_t);
        }
        2 => {
            (*node).c2rust_unnamed.boolean = *(val as *mut ::core::ffi::c_int);
        }
        3 => {
            if !child.is_null() {
                json_fields_insert(&mut (*node).c2rust_unnamed.fields, &mut *child);
            }
        }
        4 => {
            if !child.is_null() {
                json_members_push(&mut (*node).c2rust_unnamed.members, &mut *child);
            }
        }
        _ => {
            fatalx(b"unknown node type\0" as *const u8 as *const ::core::ffi::c_char);
        }
    };
}
unsafe fn json_parse_tokens(
    tokens: &[json_token],
    mut pctx: *mut json_parse_ctx,
) -> *mut json_node {
    // The parser advances raw cursors over this immutable Vec after tokenization.
    // No token is appended or moved until parsing returns.
    let mut tok: *mut json_token = tokens.as_ptr() as *mut json_token;
    let mut jn: *mut json_node = ::core::ptr::null_mut::<json_node>();
    if (*tok).type_0 as ::core::ffi::c_uint
        == TOK_OPENOBJECT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        jn = json_parse_object(
            &raw mut tok,
            pctx,
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<json_node>(),
        );
        if !jn.is_null() {
            if (*tok).type_0 as ::core::ffi::c_uint
                != TOK_EOF as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                json_error(
                    (*pctx).cause.as_mut(),
                    c"unexpected trailing data",
                    Some(CStr::from_ptr((*pctx).input.offset((*tok).offset as isize))),
                );
            } else {
                return jn;
            }
        }
    } else {
        json_error(
            (*pctx).cause.as_mut(),
            c"expected object",
            Some(CStr::from_ptr((*pctx).input.offset((*tok).offset as isize))),
        );
    }
    if !jn.is_null() {
        json_destroy_node(jn);
    }
    return ::core::ptr::null_mut::<json_node>();
}
unsafe fn json_parse_key(
    mut tok: *mut *mut json_token,
    mut pctx: *mut json_parse_ctx,
) -> Option<CString> {
    let mut len: ::core::ffi::c_int = 0;
    let mut loc: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut start: *const ::core::ffi::c_char = (*pctx).input.offset((**tok).offset as isize);
    if !((**tok).type_0 as ::core::ffi::c_uint
        != TOK_QUOTE as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        *tok = (*tok).offset(1);
        loc = (*pctx).input.offset((**tok).offset as isize);
        len = (**tok).len;
        if !((**tok).type_0 as ::core::ffi::c_uint
            != TOK_VALUE as ::core::ffi::c_int as ::core::ffi::c_uint)
        {
            *tok = (*tok).offset(1);
            if !((**tok).type_0 as ::core::ffi::c_uint
                != TOK_QUOTE as ::core::ffi::c_int as ::core::ffi::c_uint)
            {
                // Match strndup's byte-preserving, first-NUL truncation.
                let bytes = std::slice::from_raw_parts(loc.cast::<u8>(), len as usize);
                let end = bytes
                    .iter()
                    .position(|&byte| byte == 0)
                    .unwrap_or(bytes.len());
                let key = CString::new(&bytes[..end]).expect("key prefix contains no NUL");
                *tok = (*tok).offset(1);
                return Some(key);
            }
        }
    }
    json_error(
        (*pctx).cause.as_mut(),
        c"invalid key",
        Some(CStr::from_ptr(start)),
    );
    return None;
}
unsafe extern "C" fn json_parse_object(
    mut tok: *mut *mut json_token,
    mut pctx: *mut json_parse_ctx,
    mut key: *const ::core::ffi::c_char,
    mut parent: *mut json_node,
) -> *mut json_node {
    let mut current_block: u64;
    let mut object: *mut json_node = ::core::ptr::null_mut::<json_node>();
    let mut field: *mut json_node = ::core::ptr::null_mut::<json_node>();
    let mut valstr: *mut u_char = ::core::ptr::null_mut::<u_char>();
    if (**tok).type_0 as ::core::ffi::c_uint
        != TOK_OPENOBJECT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return ::core::ptr::null_mut::<json_node>();
    }
    (*pctx).depth += 1;
    if (*pctx).depth > PARSE_DEPTH_MAX {
        json_error(
            (*pctx).cause.as_mut(),
            c"parse depth exceeded",
            Some(CStr::from_ptr(
                (*pctx).input.offset((**tok).offset as isize),
            )),
        );
        return ::core::ptr::null_mut::<json_node>();
    }
    *tok = (*tok).offset(1);
    object = json_create_node(parent, NODE_OBJECT, key, NULL);
    loop {
        if !((**tok).type_0 as ::core::ffi::c_uint
            != TOK_CLOSEOBJECT as ::core::ffi::c_int as ::core::ffi::c_uint)
        {
            current_block = 9853141518545631134;
            break;
        }
        let Some(fkey) = json_parse_key(tok, pctx) else {
            current_block = 7971653673408253115;
            break;
        };
        // Lookup only borrows; child creation duplicates the key. No pointer
        // outlives this iteration, including on errors before object teardown.
        if !json_find(object, fkey.as_ptr()).is_null() {
            json_error(
                (*pctx).cause.as_mut(),
                c"duplicate key",
                Some(CStr::from_ptr(
                    (*pctx).input.offset((**tok).offset as isize),
                )),
            );
            current_block = 7971653673408253115;
            break;
        } else if (**tok).type_0 as ::core::ffi::c_uint
            != TOK_COLON as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            json_error(
                (*pctx).cause.as_mut(),
                c"missing colon",
                Some(CStr::from_ptr(
                    (*pctx).input.offset((**tok).offset as isize),
                )),
            );
            current_block = 7971653673408253115;
            break;
        } else {
            *tok = (*tok).offset(1);
            match (**tok).type_0 as ::core::ffi::c_uint {
                6 => {
                    field = json_parse_string(tok, pctx, fkey.as_ptr(), object);
                }
                7 => {
                    valstr = (*pctx).input.offset((**tok).offset as isize) as *mut u_char;
                    if *valstr as ::core::ffi::c_int == '-' as i32
                        && *(*__ctype_b_loc())
                            .offset(*valstr.offset(1 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int
                            & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort
                                as ::core::ffi::c_int
                            != 0
                        || *(*__ctype_b_loc()).offset(*valstr as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int
                            & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort
                                as ::core::ffi::c_int
                            != 0
                    {
                        field = json_parse_number(tok, pctx, fkey.as_ptr(), object);
                    } else {
                        field = json_parse_boolean(tok, pctx, fkey.as_ptr(), object);
                    }
                }
                0 => {
                    field = json_parse_object(tok, pctx, fkey.as_ptr(), object);
                }
                2 => {
                    field = json_parse_array(tok, pctx, fkey.as_ptr(), object);
                }
                _ => {
                    json_error(
                        (*pctx).cause.as_mut(),
                        c"unexpected value when parsing object",
                        Some(CStr::from_ptr(
                            (*pctx).input.offset((**tok).offset as isize),
                        )),
                    );
                    current_block = 7971653673408253115;
                    break;
                }
            }
            if field.is_null() {
                current_block = 7971653673408253115;
                break;
            }
            json_assign_value(object, field as *mut ::core::ffi::c_void);
            if (**tok).type_0 as ::core::ffi::c_uint
                == TOK_COMMA as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if (*(*tok).offset(1 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
                    == TOK_CLOSEOBJECT as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    json_error(
                        (*pctx).cause.as_mut(),
                        c"invalid object",
                        Some(CStr::from_ptr(
                            (*pctx).input.offset((**tok).offset as isize),
                        )),
                    );
                    current_block = 7971653673408253115;
                    break;
                } else {
                    *tok = (*tok).offset(1);
                }
            } else if (**tok).type_0 as ::core::ffi::c_uint
                != TOK_CLOSEOBJECT as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                json_error(
                    (*pctx).cause.as_mut(),
                    c"invalid object",
                    Some(CStr::from_ptr(
                        (*pctx).input.offset((**tok).offset as isize),
                    )),
                );
                current_block = 7971653673408253115;
                break;
            }
        }
    }
    match current_block {
        7971653673408253115 => {
            json_destroy_node(object);
            return ::core::ptr::null_mut::<json_node>();
        }
        _ => {
            *tok = (*tok).offset(1);
            (*pctx).depth -= 1;
            return object;
        }
    };
}
unsafe extern "C" fn json_parse_array(
    mut tok: *mut *mut json_token,
    mut pctx: *mut json_parse_ctx,
    mut key: *const ::core::ffi::c_char,
    mut parent: *mut json_node,
) -> *mut json_node {
    let mut current_block: u64;
    let mut array: *mut json_node = ::core::ptr::null_mut::<json_node>();
    let mut member: *mut json_node = ::core::ptr::null_mut::<json_node>();
    if (**tok).type_0 as ::core::ffi::c_uint
        != TOK_OPENARRAY as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return ::core::ptr::null_mut::<json_node>();
    }
    *tok = (*tok).offset(1);
    array = json_create_node(parent, NODE_ARRAY, key, NULL);
    loop {
        if !((**tok).type_0 as ::core::ffi::c_uint
            != TOK_CLOSEARRAY as ::core::ffi::c_int as ::core::ffi::c_uint)
        {
            current_block = 1054647088692577877;
            break;
        }
        match (**tok).type_0 as ::core::ffi::c_uint {
            0 => {
                member =
                    json_parse_object(tok, pctx, ::core::ptr::null::<::core::ffi::c_char>(), array);
                if member.is_null() {
                    current_block = 15988181977378875837;
                    break;
                }
                json_assign_value(array, member as *mut ::core::ffi::c_void);
                if (**tok).type_0 as ::core::ffi::c_uint
                    == TOK_COMMA as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    if (*(*tok).offset(1 as ::core::ffi::c_int as isize)).type_0
                        as ::core::ffi::c_uint
                        == TOK_CLOSEARRAY as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        json_error(
                            (*pctx).cause.as_mut(),
                            c"invalid array",
                            Some(CStr::from_ptr(
                                (*pctx).input.offset((**tok).offset as isize),
                            )),
                        );
                        current_block = 15988181977378875837;
                        break;
                    } else {
                        *tok = (*tok).offset(1);
                    }
                } else {
                    if !((**tok).type_0 as ::core::ffi::c_uint
                        != TOK_CLOSEARRAY as ::core::ffi::c_int as ::core::ffi::c_uint)
                    {
                        continue;
                    }
                    json_error(
                        (*pctx).cause.as_mut(),
                        c"invalid array",
                        Some(CStr::from_ptr(
                            (*pctx).input.offset((**tok).offset as isize),
                        )),
                    );
                    current_block = 15988181977378875837;
                    break;
                }
            }
            _ => {
                json_error(
                    (*pctx).cause.as_mut(),
                    c"invalid array member",
                    Some(CStr::from_ptr(
                        (*pctx).input.offset((**tok).offset as isize),
                    )),
                );
                current_block = 15988181977378875837;
                break;
            }
        }
    }
    match current_block {
        15988181977378875837 => {
            json_destroy_node(array);
            return ::core::ptr::null_mut::<json_node>();
        }
        _ => {
            *tok = (*tok).offset(1);
            return array;
        }
    };
}
unsafe extern "C" fn json_parse_string(
    mut tok: *mut *mut json_token,
    mut pctx: *mut json_parse_ctx,
    mut key: *const ::core::ffi::c_char,
    mut parent: *mut json_node,
) -> *mut json_node {
    let mut loc: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut start: *const ::core::ffi::c_char = (*pctx).input.offset((**tok).offset as isize);
    let mut len: ::core::ffi::c_int = 0;
    if !((**tok).type_0 as ::core::ffi::c_uint
        != TOK_QUOTE as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        *tok = (*tok).offset(1);
        if !((**tok).type_0 as ::core::ffi::c_uint
            != TOK_VALUE as ::core::ffi::c_int as ::core::ffi::c_uint)
        {
            loc = (*pctx).input.offset((**tok).offset as isize);
            len = (**tok).len;
            *tok = (*tok).offset(1);
            if !((**tok).type_0 as ::core::ffi::c_uint
                != TOK_QUOTE as ::core::ffi::c_int as ::core::ffi::c_uint)
            {
                *tok = (*tok).offset(1);
                let bytes = ::core::slice::from_raw_parts(loc.cast::<u8>(), len as usize);
                let string = CString::new(bytes).expect("JSON token contains no NUL");
                let node = json_create_node(parent, NODE_STRING, key, ::core::ptr::null_mut());
                json_set_string(&mut *node, string);
                return node;
            }
        }
    }
    json_error(
        (*pctx).cause.as_mut(),
        c"invalid string",
        Some(CStr::from_ptr(start)),
    );
    return ::core::ptr::null_mut::<json_node>();
}
unsafe extern "C" fn json_parse_number(
    mut tok: *mut *mut json_token,
    mut pctx: *mut json_parse_ctx,
    mut key: *const ::core::ffi::c_char,
    mut parent: *mut json_node,
) -> *mut json_node {
    let mut start: *const ::core::ffi::c_char = (*pctx).input.offset((**tok).offset as isize);
    let mut endptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut num: int64_t = 0;
    let mut len: ::core::ffi::c_int = (**tok).len;
    if !(*start.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '0' as i32
        && len != 1 as ::core::ffi::c_int
        || *start.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '-' as i32
            && *start.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '0' as i32
            && len != 2 as ::core::ffi::c_int)
    {
        *__errno_location() = 0 as ::core::ffi::c_int;
        num = strtoll(start, &raw mut endptr, 10 as ::core::ffi::c_int) as int64_t;
        if !(*__errno_location() != 0 as ::core::ffi::c_int
            || endptr != start.offset(len as isize) as *mut ::core::ffi::c_char)
        {
            *tok = (*tok).offset(1);
            return json_create_node(
                parent,
                NODE_NUMBER,
                key,
                &raw mut num as *mut ::core::ffi::c_void,
            );
        }
    }
    json_error(
        (*pctx).cause.as_mut(),
        c"invalid number",
        Some(CStr::from_ptr(start)),
    );
    return ::core::ptr::null_mut::<json_node>();
}
unsafe extern "C" fn json_parse_boolean(
    mut tok: *mut *mut json_token,
    mut pctx: *mut json_parse_ctx,
    mut key: *const ::core::ffi::c_char,
    mut parent: *mut json_node,
) -> *mut json_node {
    let mut len: ::core::ffi::c_int = (**tok).len;
    let mut boolean: ::core::ffi::c_int = 0;
    let mut start: *const ::core::ffi::c_char = (*pctx).input.offset((**tok).offset as isize);
    if strncmp(
        start,
        b"true\0" as *const u8 as *const ::core::ffi::c_char,
        len as size_t,
    ) == 0 as ::core::ffi::c_int
        && len == 4 as ::core::ffi::c_int
    {
        boolean = 1 as ::core::ffi::c_int;
    } else if strncmp(
        start,
        b"false\0" as *const u8 as *const ::core::ffi::c_char,
        len as size_t,
    ) == 0 as ::core::ffi::c_int
        && len == 5 as ::core::ffi::c_int
    {
        boolean = 0 as ::core::ffi::c_int;
    } else {
        json_error(
            (*pctx).cause.as_mut(),
            c"invalid boolean",
            Some(CStr::from_ptr(start)),
        );
        return ::core::ptr::null_mut::<json_node>();
    }
    *tok = (*tok).offset(1);
    return json_create_node(
        parent,
        NODE_BOOLEAN,
        key,
        &raw mut boolean as *mut ::core::ffi::c_void,
    );
}
unsafe extern "C" fn json_string_append(mut buffer: *mut evbuffer, mut node: *mut json_node) {
    let mut field: *mut json_node = ::core::ptr::null_mut::<json_node>();
    let mut member: *mut json_node = ::core::ptr::null_mut::<json_node>();
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut comma: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    match (*node).type_0 as ::core::ffi::c_uint {
        0 => {
            evbuffer_add_printf(
                buffer,
                b"\"%s\"\0" as *const u8 as *const ::core::ffi::c_char,
                (*node).c2rust_unnamed.str_0,
            );
        }
        1 => {
            evbuffer_add_printf(
                buffer,
                b"%lld\0" as *const u8 as *const ::core::ffi::c_char,
                (*node).c2rust_unnamed.num as ::core::ffi::c_longlong,
            );
        }
        2 => {
            if (*node).c2rust_unnamed.boolean != 0 {
                s = b"true\0" as *const u8 as *const ::core::ffi::c_char;
            } else {
                s = b"false\0" as *const u8 as *const ::core::ffi::c_char;
            }
            evbuffer_add(buffer, s as *const ::core::ffi::c_void, strlen(s));
        }
        3 => {
            evbuffer_add(
                buffer,
                b"{\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                1 as size_t,
            );
            field = json_fields_minmax(&(*node).c2rust_unnamed.fields, RB_NEGINF);
            while !field.is_null() {
                if comma != 0 {
                    evbuffer_add(
                        buffer,
                        b",\0" as *const u8 as *const ::core::ffi::c_char
                            as *const ::core::ffi::c_void,
                        1 as size_t,
                    );
                }
                evbuffer_add_printf(
                    buffer,
                    b"\"%s\":\0" as *const u8 as *const ::core::ffi::c_char,
                    ((*field).key)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                );
                json_string_append(buffer, field);
                comma = 1 as ::core::ffi::c_int;
                field = json_fields_next(&(*node).c2rust_unnamed.fields, &*field);
            }
            evbuffer_add(
                buffer,
                b"}\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                1 as size_t,
            );
        }
        4 => {
            evbuffer_add(
                buffer,
                b"[\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                1 as size_t,
            );
            member = json_members_first(&(*node).c2rust_unnamed.members);
            while !member.is_null() {
                if comma != 0 {
                    evbuffer_add(
                        buffer,
                        b",\0" as *const u8 as *const ::core::ffi::c_char
                            as *const ::core::ffi::c_void,
                        1 as size_t,
                    );
                }
                json_string_append(buffer, member);
                comma = 1 as ::core::ffi::c_int;
                member = json_members_next(&(*node).c2rust_unnamed.members, &*member);
            }
            evbuffer_add(
                buffer,
                b"]\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                1 as size_t,
            );
        }
        _ => {}
    };
}
pub unsafe fn json_to_string(node: *mut json_node) -> Option<CString> {
    let mut buffer: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    if node.is_null() {
        return None;
    }
    buffer = evbuffer_new();
    if buffer.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    json_string_append(buffer, node);
    let len = evbuffer_get_length(&*(buffer));
    let bytes = if len == 0 {
        Vec::new()
    } else {
        let ptr = evbuffer_pullup(buffer, -(1 as ::core::ffi::c_int) as ssize_t);
        ::core::slice::from_raw_parts(ptr.cast::<u8>(), len).to_vec()
    };
    evbuffer_free(buffer);
    Some(CString::new(bytes).expect("serialized JSON contains no NUL"))
}

#[cfg(test)]
mod json_fields_tests {
    use super::*;
    use std::ffi::{CStr, CString};

    #[test]
    fn diagnostics_preserve_bytes_and_context_truncation() {
        unsafe {
            let mut cause = None;
            json_error(Some(&mut cause), c"\xff", Some(c"abcdefghZ"));
            assert_eq!(cause.take().unwrap().to_bytes(), b"\xff: abcdefgh...");
            json_error(Some(&mut cause), c"\xff", Some(c"ab"));
            assert_eq!(cause.take().unwrap().to_bytes(), b"\xff: ab");
        }
    }

    unsafe fn new_node(key: &CString) -> *mut json_node {
        json_create_node(
            ::core::ptr::null_mut(),
            NODE_STRING,
            key.as_ptr(),
            ::core::ptr::null_mut(),
        )
    }

    unsafe fn free_tree(head: &mut json_fields) {
        let mut item = json_fields_minmax(head, RB_NEGINF);
        while !item.is_null() {
            let next = json_fields_next(head, &*item);
            assert_eq!(json_fields_remove(head, &*item), item);
            json_destroy_node(item);
            item = next;
        }
        drop(Box::from_raw(head.entries));
        head.entries = ::core::ptr::null_mut::<json_fields_storage>();
    }

    #[test]
    fn json_fields_preserves_strcmp_order_and_duplicate_keys() {
        unsafe {
            let names: &[&[u8]] = &[b"zeta", b"alpha", b"alpha-2", b"\x80high", b"alpha\x01"];
            let keys: Vec<CString> = names
                .iter()
                .map(|name| CString::new(*name).expect("test key has no NUL"))
                .collect();
            assert!(crate::src::ffi::libc::strcmp(keys[1].as_ptr(), keys[2].as_ptr()) < 0);
            assert!(crate::src::ffi::libc::strcmp(keys[3].as_ptr(), keys[0].as_ptr()) > 0);
            let mut head = json_fields {
                entries: Box::into_raw(Box::new(json_fields_storage::default())),
            };
            let mut items = Vec::new();
            for key in &keys {
                let item = new_node(key);
                assert!(json_fields_insert(&mut head, &mut *item).is_null());
                items.push(item);
            }
            drop(keys);

            let duplicate_key = CString::new(b"alpha".as_slice()).unwrap();
            let duplicate = new_node(&duplicate_key);
            assert_eq!(json_fields_insert(&mut head, &mut *duplicate), items[1]);
            json_destroy_node(duplicate);

            let probe_key = CString::new(b"alpha".as_slice()).unwrap();
            assert_eq!(json_fields_find(&head, probe_key.as_c_str()), items[1]);

            assert_eq!(
                CStr::from_ptr(
                    ((*json_fields_minmax(&mut head, RB_NEGINF)).key)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut())
                )
                .to_bytes(),
                b"alpha"
            );
            assert_eq!(
                CStr::from_ptr(
                    ((*json_fields_minmax(&mut head, crate::src::shared::tree::RB_INF)).key)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut())
                )
                .to_bytes(),
                b"\x80high"
            );

            let mut ordered = Vec::new();
            let mut item = json_fields_minmax(&head, RB_NEGINF);
            while !item.is_null() {
                ordered.push(
                    CStr::from_ptr(
                        ((*item).key)
                            .as_ref()
                            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                    )
                    .to_bytes()
                    .to_vec(),
                );
                item = json_fields_next(&head, &*item);
            }
            assert_eq!(
                ordered,
                vec![
                    b"alpha".to_vec(),
                    b"alpha\x01".to_vec(),
                    b"alpha-2".to_vec(),
                    b"zeta".to_vec(),
                    b"\x80high".to_vec(),
                ]
            );

            let removed = json_fields_remove(&mut head, &*items[2]);
            assert_eq!(removed, items[2]);
            let removed_probe = CString::new(b"alpha-2".as_slice()).unwrap();
            assert!(json_fields_find(&head, removed_probe.as_c_str()).is_null());
            json_destroy_node(removed);
            free_tree(&mut head);
        }
    }
}

#[cfg(test)]
mod json_string_owner_tests {
    use super::*;

    #[test]
    fn parsed_high_byte_string_survives_input_release() {
        unsafe {
            let input = CString::new(b"{\"nested\":[{\"value\":\"\xff\"}]}".as_slice()).unwrap();
            let root = json_parse(input.as_ptr(), ::core::ptr::null_mut());
            assert!(!root.is_null());
            drop(input);

            let nested = json_find(root, c"nested".as_ptr());
            let first = json_array_first(nested);
            let mut raw = ::core::ptr::null();
            assert_eq!(
                json_find_string(
                    first,
                    c"value".as_ptr(),
                    &raw mut raw,
                    ::core::ptr::null_mut()
                ),
                0
            );
            assert_eq!(CStr::from_ptr(raw).to_bytes(), b"\xff");
            json_destroy_node(root);
        }
    }
}
