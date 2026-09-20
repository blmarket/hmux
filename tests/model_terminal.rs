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
        "src/tty_term.rs::TERMTYPE",
        hmux2::src::tty_term::TERMTYPE,
        []
    );
    record!(
        "src/tty_term.rs::term",
        hmux2::src::tty_term::term,
        [type_0]
    );
    record!(
        "src/tty_term.rs::termtype",
        hmux2::src::tty_term::termtype,
        [
            term_names,
            str_table,
            Booleans,
            Numbers,
            Strings,
            ext_str_table,
            ext_Names,
            num_Booleans,
            num_Numbers,
            num_Strings,
            ext_Booleans,
            ext_Numbers,
            ext_Strings
        ]
    );
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/model-terminal.txt"));
}
