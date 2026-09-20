//! Authoritative declarations shared by the translated C modules.
//!
//! The generated translation units intentionally keep declarations whose
//! identity depends on an anonymous C type private to the unit that uses it.
//! Only declarations with a verified, stable ABI belong here.

pub mod abi;
pub mod colour;
pub mod command;
pub mod grid;
pub mod key;
pub mod style;
pub mod utf8;
