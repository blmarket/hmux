//! Frozen pre-migration sizes, alignments, and every named field offset.
use std::mem::{align_of, offset_of, size_of};
#[test]
fn original_copies_match() {
    let mut records = Vec::new();
    macro_rules! record {
        ($label:literal, $ty:ty, [$($field:ident),*]) => {
            records.push(format!("{} {} {} {:?}", $label, size_of::<$ty>(), align_of::<$ty>(),
                &[$(offset_of!($ty, $field)),*] as &[usize]));
        };
    }
    record!(
        "src/cmd_show_messages.rs::tty_terms",
        hmux2::src::cmd_show_messages::tty_terms,
        [lh_first]
    );
    record!(
        "src/tty_term.rs::tty_terms",
        hmux2::src::tty_term::tty_terms,
        [lh_first]
    );
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/remaining-tty.txt"));
}
