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
pub mod hyperlinks;
pub mod key;
pub mod layout;
pub mod menu;
pub mod message;
pub mod monitor;
pub mod mouse;
pub mod options;
pub mod pane;
pub mod prompt;
pub mod screen;
pub mod screen_write;
pub mod sort;
pub mod spawn;
pub mod stdio;
pub mod style;
pub mod terminal;
pub mod tree;
pub mod tty;
pub mod utf8;
pub mod variadic;
pub mod window;
pub mod time;
pub mod signal;
pub mod socket;
pub mod regex;
pub mod posix_terminal;
pub mod account;
pub mod limits;
pub mod errno;
