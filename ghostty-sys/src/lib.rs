//! Raw FFI bindings for libghostty-vt.
//!
//! This crate is exactly what its name says: the hand-written declarations for
//! the subset of the libghostty-vt C API hmux links against, plus the build
//! logic that finds or builds the library (see `build.rs`). It carries no
//! terminal semantics of its own.
//!
//! The safe wrapper that turns these bindings into a screen the daemon can use
//! lives in hmux, behind its terminal-emulation seam.

pub mod ffi;
