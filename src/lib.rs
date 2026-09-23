#![feature(local_waker)]
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(unused_assignments)]
#![allow(unused_imports)]
#![allow(unused_mut)]
// The C2Rust translation intentionally retains these expression shapes while
// the generated modules are migrated incrementally.
#![allow(clippy::eq_op)]
#![allow(clippy::self_assignment)]
#![allow(clippy::while_immutable_condition)]
#![feature(c_variadic)]
#![feature(allocator_api)]
#![feature(extern_types)]
#![feature(label_break_value)]
#![feature(raw_ref_op)]

#[macro_use]
extern crate c2rust_bitfields;
extern crate libc;

/// Public modules; the historical `hmux2::src` namespace remains supported.
#[path = "modules.rs"]
pub mod src;
