use super::*;
use crate::tests::test_fixtures::globals;

fn terminal_with_caps(name: &CStr, caps: &[CString]) -> Result<TerminalRef, CString> {
    let mut owner = ClientRef::new(client::default());
    (unsafe { owner.as_client_mut() }).environ =
        Some(Box::new(crate::environ::RustEnvironment::empty()));
    let mut tty = tty::default();
    tty.owner = Some(owner.downgrade());
    tty_term_create(&mut tty, name, caps, &mut 0)
}

fn terminal(name: &CStr) -> TerminalRef {
    terminal_with_caps(
        name,
        &[
            c"clear=\x1b[H\x1b[2J".to_owned(),
            c"cup=\x1b[%i%p1%d;%p2%dH".to_owned(),
        ],
    )
    .unwrap()
}

#[test]
fn terminal_registry_removes_the_last_owner_drop() {
    let _guard = globals();
    let term = terminal(c"registry-test");
    assert_eq!(
        TTY_TERMS.with(|registry| registry.entries.borrow().len()),
        1
    );
    let retained = term.clone();
    let weak = std::rc::Rc::downgrade(&term);
    drop(term);
    assert!(weak.upgrade().is_some());
    assert_eq!(
        tty_term_snapshots(None)[0].name.as_c_str(),
        c"registry-test"
    );
    drop(retained);
    assert!(weak.upgrade().is_none());
    assert!(TTY_TERMS.with(|registry| registry.entries.borrow().is_empty()));
}

#[test]
fn terminal_registry_preserves_newest_first_order_and_explicit_cleanup() {
    let _guard = globals();
    let first = terminal(c"registry-first");
    let second = terminal(c"registry-second");
    let names: Vec<_> = tty_term_snapshots(None)
        .into_iter()
        .map(|snapshot| snapshot.name)
        .collect();
    assert_eq!(
        names,
        [c"registry-second".to_owned(), c"registry-first".to_owned()]
    );
    unsafe { tty_term_free(second) };
    let snapshots = tty_term_snapshots(None);
    assert_eq!(snapshots.len(), 1);
    assert_eq!(snapshots[0].name.as_c_str(), c"registry-first");
    drop(first);
    assert!(tty_term_snapshots(None).is_empty());
}

#[test]
fn terminal_registry_is_confined_and_cleanup_outlives_the_index_owner() {
    thread_local! {
        static LAST_TERM: std::cell::RefCell<Option<TerminalRef>> = const { std::cell::RefCell::new(None) };
    }
    std::thread::spawn(|| {
        LAST_TERM.with_borrow_mut(|last| {
            let term = std::rc::Rc::new(std::cell::RefCell::new(RustTerminalCapabilities::new(
                c"retained",
            )));
            term.borrow_mut().registration =
                Some(TTY_TERMS.with(|registry| registry.register(&term, None)));
            std::thread::spawn(|| assert!(tty_term_snapshots(None).is_empty()))
                .join()
                .unwrap();
            assert_eq!(tty_term_snapshots(None).len(), 1);
            *last = Some(term);
        });
    })
    .join()
    .unwrap();
}

#[test]
fn terminal_registry_removes_failed_construction() {
    let _guard = globals();
    assert!(terminal_with_caps(c"registry-invalid", &[]).is_err());
    assert!(tty_term_snapshots(None).is_empty());
}

#[test]
fn terminal_overrides_preserve_value_delimiters_and_removal_rules() {
    let _guard = globals();
    let mut term = RustTerminalCapabilities::new(c"overrides");
    term.apply_overrides(c":@:bel=left=right@:colors=23:colors=invalid:am");
    assert_eq!(term.string(TTYC_BEL), c"left=right@");
    assert_eq!(term.number(TTYC_COLORS), 23);
    assert_eq!(term.flag(TTYC_AM), 1);
    term.apply_overrides(c"bel@:am@");
    assert!(!term.has(TTYC_BEL));
    assert!(!term.has(TTYC_AM));
    term.apply_overrides(c"bel=");
    assert!(term.has(TTYC_BEL));
    assert_eq!(term.string(TTYC_BEL), c"");
    term.apply_overrides(&CString::new(b"bel=\xff=tail".as_slice()).unwrap());
    assert_eq!(term.string(TTYC_BEL).to_bytes(), b"\xff=tail");
}

#[test]
fn terminal_creation_uses_bounded_capability_values() {
    let _guard = globals();
    let term = terminal_with_caps(
        c"bounded-capabilities",
        &[
            c"clear=\x1b[H".to_owned(),
            c"cup=\x1b[%i%p1%d;%p2%dH".to_owned(),
            c"bel=left=right".to_owned(),
            c"bel".to_owned(),
            c"=ignored".to_owned(),
            c"".to_owned(),
            c"colors=23".to_owned(),
            c"am=1".to_owned(),
            c"AX=".to_owned(),
        ],
    )
    .unwrap();
    assert_eq!(term.borrow().string(TTYC_BEL), c"left=right");
    assert_eq!(term.borrow().number(TTYC_COLORS), 23);
    assert_eq!(term.borrow().flag(TTYC_AM), 1);
    assert!(term.borrow().has(TTYC_AX));
    assert_eq!(term.borrow().flag(TTYC_AX), 0);
}

#[test]
fn acs_entries_preserve_high_bytes_and_ignore_an_incomplete_pair() {
    let _guard = globals();
    let mut term = RustTerminalCapabilities::new(c"acs-bytes");
    term.apply_overrides(&CString::new(b"acsc=\xff\xfeq-X".as_slice()).unwrap());
    term.refresh_derived();
    assert_eq!(term.acs(0xff).unwrap().to_bytes(), b"\xfe");
    assert_eq!(term.acs(b'q'), Some(c"-"));
    assert_eq!(term.acs(b'X'), None);
    term.apply_overrides(c"acsc=x|");
    term.refresh_derived();
    assert_eq!(term.acs(0xff), None);
    assert_eq!(term.acs(b'q'), None);
    assert_eq!(term.acs(b'x'), Some(c"|"));
}

#[test]
fn terminal_registry_filters_by_identity_and_snapshots_outlive_owners() {
    let _guard = globals();
    let first = terminal(c"same-name");
    let second = terminal(c"same-name");
    first.borrow_mut().apply_overrides(c"bel=first");
    second.borrow_mut().apply_overrides(c"bel=second");
    let snapshots = tty_term_snapshots(Some(&first.borrow()));
    assert_eq!(snapshots.len(), 1);
    let expected = first.borrow().describe(TTYC_BEL);
    assert_eq!(snapshots[0].capabilities[TTYC_BEL as usize], expected);
    first.borrow_mut().apply_overrides(c"bel=replaced");
    drop(first);
    drop(second);
    assert!(tty_term_snapshots(None).is_empty());
    assert_eq!(snapshots[0].capabilities[TTYC_BEL as usize], expected);
}
