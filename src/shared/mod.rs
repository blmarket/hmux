//! Authoritative declarations shared by the translated C modules.
//!
//! The generated translation units intentionally keep declarations whose
//! identity depends on an anonymous C type private to the unit that uses it.
//! Only declarations with a verified, stable ABI belong here.

pub mod abi;
pub mod arguments;
pub mod client;
pub mod colour;
pub mod command;
pub mod display;
pub mod event;
pub mod grid;
pub mod key;
pub mod layout;
pub mod message;
pub mod options;
pub mod prompt;
pub mod style;
pub mod sort;
pub mod terminal;
pub mod tty;
pub mod utf8;
