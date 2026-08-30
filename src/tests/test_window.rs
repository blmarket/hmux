use super::*;
use crate::tests::test_fixtures::{ensure_reactor, globals};
use ::std::sync::MutexGuard;

/// A turn at the server-wide state these tests reach — the window tree,
/// the tree of every pane and the id the next window is given — starting
/// from empty trees and leaving them empty.
fn server() -> MutexGuard<'static, ()> {
    let guard = globals();
    ensure_reactor();
    assert!(windows.map().is_empty(), "the window tree is not empty");
    assert!(pane_walk().next().is_none(), "the pane tree is not empty");
    guard
}
