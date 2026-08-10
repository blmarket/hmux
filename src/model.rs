//! A terminal model for out-of-process test harnesses.
//!
//! The conformance suite drives a real client tty and then has to say what the
//! screen should look like. It computes that with the same emulation the daemon
//! used to produce the bytes, so a difference in the assertion is a difference
//! in the daemon, not in two independent readings of a byte stream.
//!
//! This is the only public window onto the emulation, and it is
//! deliberately narrow: feed bytes, read the screen back. Nothing here is an
//! end-user contract — the daemon's contract is its tmux-compatible command
//! line and wire protocol.

pub use hmux_vt::TerminalModel;
