//! Frozen measurements of every original copy; no fixture update mode.
use std::mem::{align_of, offset_of, size_of};
#[test]
fn original_declarations_match() {
    let mut records = Vec::new();
    let v: hmux2::src::input_keys::control_character_code = hmux2::src::input_keys::C0_ASC;
    records.push(format!(
        "src/input_keys.rs::C0_ASC {:?} {} {}",
        v,
        size_of::<hmux2::src::input_keys::control_character_code>(),
        align_of::<hmux2::src::input_keys::control_character_code>()
    ));
    let v: hmux2::src::key_string::control_character_code = hmux2::src::key_string::C0_ASC;
    records.push(format!(
        "src/key_string.rs::C0_ASC {:?} {} {}",
        v,
        size_of::<hmux2::src::key_string::control_character_code>(),
        align_of::<hmux2::src::key_string::control_character_code>()
    ));
    let v: hmux2::src::tty_keys::control_character_code = hmux2::src::tty_keys::C0_ASC;
    records.push(format!(
        "src/tty_keys.rs::C0_ASC {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::control_character_code>(),
        align_of::<hmux2::src::tty_keys::control_character_code>()
    ));
    let v: hmux2::src::input_keys::control_character_code = hmux2::src::input_keys::C0_BEL;
    records.push(format!(
        "src/input_keys.rs::C0_BEL {:?} {} {}",
        v,
        size_of::<hmux2::src::input_keys::control_character_code>(),
        align_of::<hmux2::src::input_keys::control_character_code>()
    ));
    let v: hmux2::src::key_string::control_character_code = hmux2::src::key_string::C0_BEL;
    records.push(format!(
        "src/key_string.rs::C0_BEL {:?} {} {}",
        v,
        size_of::<hmux2::src::key_string::control_character_code>(),
        align_of::<hmux2::src::key_string::control_character_code>()
    ));
    let v: hmux2::src::tty_keys::control_character_code = hmux2::src::tty_keys::C0_BEL;
    records.push(format!(
        "src/tty_keys.rs::C0_BEL {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::control_character_code>(),
        align_of::<hmux2::src::tty_keys::control_character_code>()
    ));
    let v: hmux2::src::input_keys::control_character_code = hmux2::src::input_keys::C0_BS;
    records.push(format!(
        "src/input_keys.rs::C0_BS {:?} {} {}",
        v,
        size_of::<hmux2::src::input_keys::control_character_code>(),
        align_of::<hmux2::src::input_keys::control_character_code>()
    ));
    let v: hmux2::src::key_string::control_character_code = hmux2::src::key_string::C0_BS;
    records.push(format!(
        "src/key_string.rs::C0_BS {:?} {} {}",
        v,
        size_of::<hmux2::src::key_string::control_character_code>(),
        align_of::<hmux2::src::key_string::control_character_code>()
    ));
    let v: hmux2::src::tty_keys::control_character_code = hmux2::src::tty_keys::C0_BS;
    records.push(format!(
        "src/tty_keys.rs::C0_BS {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::control_character_code>(),
        align_of::<hmux2::src::tty_keys::control_character_code>()
    ));
    let v: hmux2::src::input_keys::control_character_code = hmux2::src::input_keys::C0_CAN;
    records.push(format!(
        "src/input_keys.rs::C0_CAN {:?} {} {}",
        v,
        size_of::<hmux2::src::input_keys::control_character_code>(),
        align_of::<hmux2::src::input_keys::control_character_code>()
    ));
    let v: hmux2::src::key_string::control_character_code = hmux2::src::key_string::C0_CAN;
    records.push(format!(
        "src/key_string.rs::C0_CAN {:?} {} {}",
        v,
        size_of::<hmux2::src::key_string::control_character_code>(),
        align_of::<hmux2::src::key_string::control_character_code>()
    ));
    let v: hmux2::src::tty_keys::control_character_code = hmux2::src::tty_keys::C0_CAN;
    records.push(format!(
        "src/tty_keys.rs::C0_CAN {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::control_character_code>(),
        align_of::<hmux2::src::tty_keys::control_character_code>()
    ));
    let v: hmux2::src::input_keys::control_character_code = hmux2::src::input_keys::C0_CR;
    records.push(format!(
        "src/input_keys.rs::C0_CR {:?} {} {}",
        v,
        size_of::<hmux2::src::input_keys::control_character_code>(),
        align_of::<hmux2::src::input_keys::control_character_code>()
    ));
    let v: hmux2::src::key_string::control_character_code = hmux2::src::key_string::C0_CR;
    records.push(format!(
        "src/key_string.rs::C0_CR {:?} {} {}",
        v,
        size_of::<hmux2::src::key_string::control_character_code>(),
        align_of::<hmux2::src::key_string::control_character_code>()
    ));
    let v: hmux2::src::tty_keys::control_character_code = hmux2::src::tty_keys::C0_CR;
    records.push(format!(
        "src/tty_keys.rs::C0_CR {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::control_character_code>(),
        align_of::<hmux2::src::tty_keys::control_character_code>()
    ));
    let v: hmux2::src::input_keys::control_character_code = hmux2::src::input_keys::C0_DC1;
    records.push(format!(
        "src/input_keys.rs::C0_DC1 {:?} {} {}",
        v,
        size_of::<hmux2::src::input_keys::control_character_code>(),
        align_of::<hmux2::src::input_keys::control_character_code>()
    ));
    let v: hmux2::src::key_string::control_character_code = hmux2::src::key_string::C0_DC1;
    records.push(format!(
        "src/key_string.rs::C0_DC1 {:?} {} {}",
        v,
        size_of::<hmux2::src::key_string::control_character_code>(),
        align_of::<hmux2::src::key_string::control_character_code>()
    ));
    let v: hmux2::src::tty_keys::control_character_code = hmux2::src::tty_keys::C0_DC1;
    records.push(format!(
        "src/tty_keys.rs::C0_DC1 {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::control_character_code>(),
        align_of::<hmux2::src::tty_keys::control_character_code>()
    ));
    let v: hmux2::src::input_keys::control_character_code = hmux2::src::input_keys::C0_DC2;
    records.push(format!(
        "src/input_keys.rs::C0_DC2 {:?} {} {}",
        v,
        size_of::<hmux2::src::input_keys::control_character_code>(),
        align_of::<hmux2::src::input_keys::control_character_code>()
    ));
    let v: hmux2::src::key_string::control_character_code = hmux2::src::key_string::C0_DC2;
    records.push(format!(
        "src/key_string.rs::C0_DC2 {:?} {} {}",
        v,
        size_of::<hmux2::src::key_string::control_character_code>(),
        align_of::<hmux2::src::key_string::control_character_code>()
    ));
    let v: hmux2::src::tty_keys::control_character_code = hmux2::src::tty_keys::C0_DC2;
    records.push(format!(
        "src/tty_keys.rs::C0_DC2 {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::control_character_code>(),
        align_of::<hmux2::src::tty_keys::control_character_code>()
    ));
    let v: hmux2::src::input_keys::control_character_code = hmux2::src::input_keys::C0_DC3;
    records.push(format!(
        "src/input_keys.rs::C0_DC3 {:?} {} {}",
        v,
        size_of::<hmux2::src::input_keys::control_character_code>(),
        align_of::<hmux2::src::input_keys::control_character_code>()
    ));
    let v: hmux2::src::key_string::control_character_code = hmux2::src::key_string::C0_DC3;
    records.push(format!(
        "src/key_string.rs::C0_DC3 {:?} {} {}",
        v,
        size_of::<hmux2::src::key_string::control_character_code>(),
        align_of::<hmux2::src::key_string::control_character_code>()
    ));
    let v: hmux2::src::tty_keys::control_character_code = hmux2::src::tty_keys::C0_DC3;
    records.push(format!(
        "src/tty_keys.rs::C0_DC3 {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::control_character_code>(),
        align_of::<hmux2::src::tty_keys::control_character_code>()
    ));
    let v: hmux2::src::input_keys::control_character_code = hmux2::src::input_keys::C0_DC4;
    records.push(format!(
        "src/input_keys.rs::C0_DC4 {:?} {} {}",
        v,
        size_of::<hmux2::src::input_keys::control_character_code>(),
        align_of::<hmux2::src::input_keys::control_character_code>()
    ));
    let v: hmux2::src::key_string::control_character_code = hmux2::src::key_string::C0_DC4;
    records.push(format!(
        "src/key_string.rs::C0_DC4 {:?} {} {}",
        v,
        size_of::<hmux2::src::key_string::control_character_code>(),
        align_of::<hmux2::src::key_string::control_character_code>()
    ));
    let v: hmux2::src::tty_keys::control_character_code = hmux2::src::tty_keys::C0_DC4;
    records.push(format!(
        "src/tty_keys.rs::C0_DC4 {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::control_character_code>(),
        align_of::<hmux2::src::tty_keys::control_character_code>()
    ));
    let v: hmux2::src::input_keys::control_character_code = hmux2::src::input_keys::C0_DLE;
    records.push(format!(
        "src/input_keys.rs::C0_DLE {:?} {} {}",
        v,
        size_of::<hmux2::src::input_keys::control_character_code>(),
        align_of::<hmux2::src::input_keys::control_character_code>()
    ));
    let v: hmux2::src::key_string::control_character_code = hmux2::src::key_string::C0_DLE;
    records.push(format!(
        "src/key_string.rs::C0_DLE {:?} {} {}",
        v,
        size_of::<hmux2::src::key_string::control_character_code>(),
        align_of::<hmux2::src::key_string::control_character_code>()
    ));
    let v: hmux2::src::tty_keys::control_character_code = hmux2::src::tty_keys::C0_DLE;
    records.push(format!(
        "src/tty_keys.rs::C0_DLE {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::control_character_code>(),
        align_of::<hmux2::src::tty_keys::control_character_code>()
    ));
    let v: hmux2::src::input_keys::control_character_code = hmux2::src::input_keys::C0_EM;
    records.push(format!(
        "src/input_keys.rs::C0_EM {:?} {} {}",
        v,
        size_of::<hmux2::src::input_keys::control_character_code>(),
        align_of::<hmux2::src::input_keys::control_character_code>()
    ));
    let v: hmux2::src::key_string::control_character_code = hmux2::src::key_string::C0_EM;
    records.push(format!(
        "src/key_string.rs::C0_EM {:?} {} {}",
        v,
        size_of::<hmux2::src::key_string::control_character_code>(),
        align_of::<hmux2::src::key_string::control_character_code>()
    ));
    let v: hmux2::src::tty_keys::control_character_code = hmux2::src::tty_keys::C0_EM;
    records.push(format!(
        "src/tty_keys.rs::C0_EM {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::control_character_code>(),
        align_of::<hmux2::src::tty_keys::control_character_code>()
    ));
    let v: hmux2::src::input_keys::control_character_code = hmux2::src::input_keys::C0_ENQ;
    records.push(format!(
        "src/input_keys.rs::C0_ENQ {:?} {} {}",
        v,
        size_of::<hmux2::src::input_keys::control_character_code>(),
        align_of::<hmux2::src::input_keys::control_character_code>()
    ));
    let v: hmux2::src::key_string::control_character_code = hmux2::src::key_string::C0_ENQ;
    records.push(format!(
        "src/key_string.rs::C0_ENQ {:?} {} {}",
        v,
        size_of::<hmux2::src::key_string::control_character_code>(),
        align_of::<hmux2::src::key_string::control_character_code>()
    ));
    let v: hmux2::src::tty_keys::control_character_code = hmux2::src::tty_keys::C0_ENQ;
    records.push(format!(
        "src/tty_keys.rs::C0_ENQ {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::control_character_code>(),
        align_of::<hmux2::src::tty_keys::control_character_code>()
    ));
    let v: hmux2::src::input_keys::control_character_code = hmux2::src::input_keys::C0_EOT;
    records.push(format!(
        "src/input_keys.rs::C0_EOT {:?} {} {}",
        v,
        size_of::<hmux2::src::input_keys::control_character_code>(),
        align_of::<hmux2::src::input_keys::control_character_code>()
    ));
    let v: hmux2::src::key_string::control_character_code = hmux2::src::key_string::C0_EOT;
    records.push(format!(
        "src/key_string.rs::C0_EOT {:?} {} {}",
        v,
        size_of::<hmux2::src::key_string::control_character_code>(),
        align_of::<hmux2::src::key_string::control_character_code>()
    ));
    let v: hmux2::src::tty_keys::control_character_code = hmux2::src::tty_keys::C0_EOT;
    records.push(format!(
        "src/tty_keys.rs::C0_EOT {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::control_character_code>(),
        align_of::<hmux2::src::tty_keys::control_character_code>()
    ));
    let v: hmux2::src::input_keys::control_character_code = hmux2::src::input_keys::C0_ESC;
    records.push(format!(
        "src/input_keys.rs::C0_ESC {:?} {} {}",
        v,
        size_of::<hmux2::src::input_keys::control_character_code>(),
        align_of::<hmux2::src::input_keys::control_character_code>()
    ));
    let v: hmux2::src::key_string::control_character_code = hmux2::src::key_string::C0_ESC;
    records.push(format!(
        "src/key_string.rs::C0_ESC {:?} {} {}",
        v,
        size_of::<hmux2::src::key_string::control_character_code>(),
        align_of::<hmux2::src::key_string::control_character_code>()
    ));
    let v: hmux2::src::tty_keys::control_character_code = hmux2::src::tty_keys::C0_ESC;
    records.push(format!(
        "src/tty_keys.rs::C0_ESC {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::control_character_code>(),
        align_of::<hmux2::src::tty_keys::control_character_code>()
    ));
    let v: hmux2::src::input_keys::control_character_code = hmux2::src::input_keys::C0_ETB;
    records.push(format!(
        "src/input_keys.rs::C0_ETB {:?} {} {}",
        v,
        size_of::<hmux2::src::input_keys::control_character_code>(),
        align_of::<hmux2::src::input_keys::control_character_code>()
    ));
    let v: hmux2::src::key_string::control_character_code = hmux2::src::key_string::C0_ETB;
    records.push(format!(
        "src/key_string.rs::C0_ETB {:?} {} {}",
        v,
        size_of::<hmux2::src::key_string::control_character_code>(),
        align_of::<hmux2::src::key_string::control_character_code>()
    ));
    let v: hmux2::src::tty_keys::control_character_code = hmux2::src::tty_keys::C0_ETB;
    records.push(format!(
        "src/tty_keys.rs::C0_ETB {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::control_character_code>(),
        align_of::<hmux2::src::tty_keys::control_character_code>()
    ));
    let v: hmux2::src::input_keys::control_character_code = hmux2::src::input_keys::C0_ETX;
    records.push(format!(
        "src/input_keys.rs::C0_ETX {:?} {} {}",
        v,
        size_of::<hmux2::src::input_keys::control_character_code>(),
        align_of::<hmux2::src::input_keys::control_character_code>()
    ));
    let v: hmux2::src::key_string::control_character_code = hmux2::src::key_string::C0_ETX;
    records.push(format!(
        "src/key_string.rs::C0_ETX {:?} {} {}",
        v,
        size_of::<hmux2::src::key_string::control_character_code>(),
        align_of::<hmux2::src::key_string::control_character_code>()
    ));
    let v: hmux2::src::tty_keys::control_character_code = hmux2::src::tty_keys::C0_ETX;
    records.push(format!(
        "src/tty_keys.rs::C0_ETX {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::control_character_code>(),
        align_of::<hmux2::src::tty_keys::control_character_code>()
    ));
    let v: hmux2::src::input_keys::control_character_code = hmux2::src::input_keys::C0_FF;
    records.push(format!(
        "src/input_keys.rs::C0_FF {:?} {} {}",
        v,
        size_of::<hmux2::src::input_keys::control_character_code>(),
        align_of::<hmux2::src::input_keys::control_character_code>()
    ));
    let v: hmux2::src::key_string::control_character_code = hmux2::src::key_string::C0_FF;
    records.push(format!(
        "src/key_string.rs::C0_FF {:?} {} {}",
        v,
        size_of::<hmux2::src::key_string::control_character_code>(),
        align_of::<hmux2::src::key_string::control_character_code>()
    ));
    let v: hmux2::src::tty_keys::control_character_code = hmux2::src::tty_keys::C0_FF;
    records.push(format!(
        "src/tty_keys.rs::C0_FF {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::control_character_code>(),
        align_of::<hmux2::src::tty_keys::control_character_code>()
    ));
    let v: hmux2::src::input_keys::control_character_code = hmux2::src::input_keys::C0_FS;
    records.push(format!(
        "src/input_keys.rs::C0_FS {:?} {} {}",
        v,
        size_of::<hmux2::src::input_keys::control_character_code>(),
        align_of::<hmux2::src::input_keys::control_character_code>()
    ));
    let v: hmux2::src::key_string::control_character_code = hmux2::src::key_string::C0_FS;
    records.push(format!(
        "src/key_string.rs::C0_FS {:?} {} {}",
        v,
        size_of::<hmux2::src::key_string::control_character_code>(),
        align_of::<hmux2::src::key_string::control_character_code>()
    ));
    let v: hmux2::src::tty_keys::control_character_code = hmux2::src::tty_keys::C0_FS;
    records.push(format!(
        "src/tty_keys.rs::C0_FS {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::control_character_code>(),
        align_of::<hmux2::src::tty_keys::control_character_code>()
    ));
    let v: hmux2::src::input_keys::control_character_code = hmux2::src::input_keys::C0_GS;
    records.push(format!(
        "src/input_keys.rs::C0_GS {:?} {} {}",
        v,
        size_of::<hmux2::src::input_keys::control_character_code>(),
        align_of::<hmux2::src::input_keys::control_character_code>()
    ));
    let v: hmux2::src::key_string::control_character_code = hmux2::src::key_string::C0_GS;
    records.push(format!(
        "src/key_string.rs::C0_GS {:?} {} {}",
        v,
        size_of::<hmux2::src::key_string::control_character_code>(),
        align_of::<hmux2::src::key_string::control_character_code>()
    ));
    let v: hmux2::src::tty_keys::control_character_code = hmux2::src::tty_keys::C0_GS;
    records.push(format!(
        "src/tty_keys.rs::C0_GS {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::control_character_code>(),
        align_of::<hmux2::src::tty_keys::control_character_code>()
    ));
    let v: hmux2::src::input_keys::control_character_code = hmux2::src::input_keys::C0_HT;
    records.push(format!(
        "src/input_keys.rs::C0_HT {:?} {} {}",
        v,
        size_of::<hmux2::src::input_keys::control_character_code>(),
        align_of::<hmux2::src::input_keys::control_character_code>()
    ));
    let v: hmux2::src::key_string::control_character_code = hmux2::src::key_string::C0_HT;
    records.push(format!(
        "src/key_string.rs::C0_HT {:?} {} {}",
        v,
        size_of::<hmux2::src::key_string::control_character_code>(),
        align_of::<hmux2::src::key_string::control_character_code>()
    ));
    let v: hmux2::src::tty_keys::control_character_code = hmux2::src::tty_keys::C0_HT;
    records.push(format!(
        "src/tty_keys.rs::C0_HT {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::control_character_code>(),
        align_of::<hmux2::src::tty_keys::control_character_code>()
    ));
    let v: hmux2::src::input_keys::control_character_code = hmux2::src::input_keys::C0_LF;
    records.push(format!(
        "src/input_keys.rs::C0_LF {:?} {} {}",
        v,
        size_of::<hmux2::src::input_keys::control_character_code>(),
        align_of::<hmux2::src::input_keys::control_character_code>()
    ));
    let v: hmux2::src::key_string::control_character_code = hmux2::src::key_string::C0_LF;
    records.push(format!(
        "src/key_string.rs::C0_LF {:?} {} {}",
        v,
        size_of::<hmux2::src::key_string::control_character_code>(),
        align_of::<hmux2::src::key_string::control_character_code>()
    ));
    let v: hmux2::src::tty_keys::control_character_code = hmux2::src::tty_keys::C0_LF;
    records.push(format!(
        "src/tty_keys.rs::C0_LF {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::control_character_code>(),
        align_of::<hmux2::src::tty_keys::control_character_code>()
    ));
    let v: hmux2::src::input_keys::control_character_code = hmux2::src::input_keys::C0_NAK;
    records.push(format!(
        "src/input_keys.rs::C0_NAK {:?} {} {}",
        v,
        size_of::<hmux2::src::input_keys::control_character_code>(),
        align_of::<hmux2::src::input_keys::control_character_code>()
    ));
    let v: hmux2::src::key_string::control_character_code = hmux2::src::key_string::C0_NAK;
    records.push(format!(
        "src/key_string.rs::C0_NAK {:?} {} {}",
        v,
        size_of::<hmux2::src::key_string::control_character_code>(),
        align_of::<hmux2::src::key_string::control_character_code>()
    ));
    let v: hmux2::src::tty_keys::control_character_code = hmux2::src::tty_keys::C0_NAK;
    records.push(format!(
        "src/tty_keys.rs::C0_NAK {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::control_character_code>(),
        align_of::<hmux2::src::tty_keys::control_character_code>()
    ));
    let v: hmux2::src::input_keys::control_character_code = hmux2::src::input_keys::C0_NUL;
    records.push(format!(
        "src/input_keys.rs::C0_NUL {:?} {} {}",
        v,
        size_of::<hmux2::src::input_keys::control_character_code>(),
        align_of::<hmux2::src::input_keys::control_character_code>()
    ));
    let v: hmux2::src::key_string::control_character_code = hmux2::src::key_string::C0_NUL;
    records.push(format!(
        "src/key_string.rs::C0_NUL {:?} {} {}",
        v,
        size_of::<hmux2::src::key_string::control_character_code>(),
        align_of::<hmux2::src::key_string::control_character_code>()
    ));
    let v: hmux2::src::tty_keys::control_character_code = hmux2::src::tty_keys::C0_NUL;
    records.push(format!(
        "src/tty_keys.rs::C0_NUL {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::control_character_code>(),
        align_of::<hmux2::src::tty_keys::control_character_code>()
    ));
    let v: hmux2::src::input_keys::control_character_code = hmux2::src::input_keys::C0_RS;
    records.push(format!(
        "src/input_keys.rs::C0_RS {:?} {} {}",
        v,
        size_of::<hmux2::src::input_keys::control_character_code>(),
        align_of::<hmux2::src::input_keys::control_character_code>()
    ));
    let v: hmux2::src::key_string::control_character_code = hmux2::src::key_string::C0_RS;
    records.push(format!(
        "src/key_string.rs::C0_RS {:?} {} {}",
        v,
        size_of::<hmux2::src::key_string::control_character_code>(),
        align_of::<hmux2::src::key_string::control_character_code>()
    ));
    let v: hmux2::src::tty_keys::control_character_code = hmux2::src::tty_keys::C0_RS;
    records.push(format!(
        "src/tty_keys.rs::C0_RS {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::control_character_code>(),
        align_of::<hmux2::src::tty_keys::control_character_code>()
    ));
    let v: hmux2::src::input_keys::control_character_code = hmux2::src::input_keys::C0_SI;
    records.push(format!(
        "src/input_keys.rs::C0_SI {:?} {} {}",
        v,
        size_of::<hmux2::src::input_keys::control_character_code>(),
        align_of::<hmux2::src::input_keys::control_character_code>()
    ));
    let v: hmux2::src::key_string::control_character_code = hmux2::src::key_string::C0_SI;
    records.push(format!(
        "src/key_string.rs::C0_SI {:?} {} {}",
        v,
        size_of::<hmux2::src::key_string::control_character_code>(),
        align_of::<hmux2::src::key_string::control_character_code>()
    ));
    let v: hmux2::src::tty_keys::control_character_code = hmux2::src::tty_keys::C0_SI;
    records.push(format!(
        "src/tty_keys.rs::C0_SI {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::control_character_code>(),
        align_of::<hmux2::src::tty_keys::control_character_code>()
    ));
    let v: hmux2::src::input_keys::control_character_code = hmux2::src::input_keys::C0_SO;
    records.push(format!(
        "src/input_keys.rs::C0_SO {:?} {} {}",
        v,
        size_of::<hmux2::src::input_keys::control_character_code>(),
        align_of::<hmux2::src::input_keys::control_character_code>()
    ));
    let v: hmux2::src::key_string::control_character_code = hmux2::src::key_string::C0_SO;
    records.push(format!(
        "src/key_string.rs::C0_SO {:?} {} {}",
        v,
        size_of::<hmux2::src::key_string::control_character_code>(),
        align_of::<hmux2::src::key_string::control_character_code>()
    ));
    let v: hmux2::src::tty_keys::control_character_code = hmux2::src::tty_keys::C0_SO;
    records.push(format!(
        "src/tty_keys.rs::C0_SO {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::control_character_code>(),
        align_of::<hmux2::src::tty_keys::control_character_code>()
    ));
    let v: hmux2::src::input_keys::control_character_code = hmux2::src::input_keys::C0_SOH;
    records.push(format!(
        "src/input_keys.rs::C0_SOH {:?} {} {}",
        v,
        size_of::<hmux2::src::input_keys::control_character_code>(),
        align_of::<hmux2::src::input_keys::control_character_code>()
    ));
    let v: hmux2::src::key_string::control_character_code = hmux2::src::key_string::C0_SOH;
    records.push(format!(
        "src/key_string.rs::C0_SOH {:?} {} {}",
        v,
        size_of::<hmux2::src::key_string::control_character_code>(),
        align_of::<hmux2::src::key_string::control_character_code>()
    ));
    let v: hmux2::src::tty_keys::control_character_code = hmux2::src::tty_keys::C0_SOH;
    records.push(format!(
        "src/tty_keys.rs::C0_SOH {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::control_character_code>(),
        align_of::<hmux2::src::tty_keys::control_character_code>()
    ));
    let v: hmux2::src::input_keys::control_character_code = hmux2::src::input_keys::C0_STX;
    records.push(format!(
        "src/input_keys.rs::C0_STX {:?} {} {}",
        v,
        size_of::<hmux2::src::input_keys::control_character_code>(),
        align_of::<hmux2::src::input_keys::control_character_code>()
    ));
    let v: hmux2::src::key_string::control_character_code = hmux2::src::key_string::C0_STX;
    records.push(format!(
        "src/key_string.rs::C0_STX {:?} {} {}",
        v,
        size_of::<hmux2::src::key_string::control_character_code>(),
        align_of::<hmux2::src::key_string::control_character_code>()
    ));
    let v: hmux2::src::tty_keys::control_character_code = hmux2::src::tty_keys::C0_STX;
    records.push(format!(
        "src/tty_keys.rs::C0_STX {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::control_character_code>(),
        align_of::<hmux2::src::tty_keys::control_character_code>()
    ));
    let v: hmux2::src::input_keys::control_character_code = hmux2::src::input_keys::C0_SUB;
    records.push(format!(
        "src/input_keys.rs::C0_SUB {:?} {} {}",
        v,
        size_of::<hmux2::src::input_keys::control_character_code>(),
        align_of::<hmux2::src::input_keys::control_character_code>()
    ));
    let v: hmux2::src::key_string::control_character_code = hmux2::src::key_string::C0_SUB;
    records.push(format!(
        "src/key_string.rs::C0_SUB {:?} {} {}",
        v,
        size_of::<hmux2::src::key_string::control_character_code>(),
        align_of::<hmux2::src::key_string::control_character_code>()
    ));
    let v: hmux2::src::tty_keys::control_character_code = hmux2::src::tty_keys::C0_SUB;
    records.push(format!(
        "src/tty_keys.rs::C0_SUB {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::control_character_code>(),
        align_of::<hmux2::src::tty_keys::control_character_code>()
    ));
    let v: hmux2::src::input_keys::control_character_code = hmux2::src::input_keys::C0_SYN;
    records.push(format!(
        "src/input_keys.rs::C0_SYN {:?} {} {}",
        v,
        size_of::<hmux2::src::input_keys::control_character_code>(),
        align_of::<hmux2::src::input_keys::control_character_code>()
    ));
    let v: hmux2::src::key_string::control_character_code = hmux2::src::key_string::C0_SYN;
    records.push(format!(
        "src/key_string.rs::C0_SYN {:?} {} {}",
        v,
        size_of::<hmux2::src::key_string::control_character_code>(),
        align_of::<hmux2::src::key_string::control_character_code>()
    ));
    let v: hmux2::src::tty_keys::control_character_code = hmux2::src::tty_keys::C0_SYN;
    records.push(format!(
        "src/tty_keys.rs::C0_SYN {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::control_character_code>(),
        align_of::<hmux2::src::tty_keys::control_character_code>()
    ));
    let v: hmux2::src::input_keys::control_character_code = hmux2::src::input_keys::C0_US;
    records.push(format!(
        "src/input_keys.rs::C0_US {:?} {} {}",
        v,
        size_of::<hmux2::src::input_keys::control_character_code>(),
        align_of::<hmux2::src::input_keys::control_character_code>()
    ));
    let v: hmux2::src::key_string::control_character_code = hmux2::src::key_string::C0_US;
    records.push(format!(
        "src/key_string.rs::C0_US {:?} {} {}",
        v,
        size_of::<hmux2::src::key_string::control_character_code>(),
        align_of::<hmux2::src::key_string::control_character_code>()
    ));
    let v: hmux2::src::tty_keys::control_character_code = hmux2::src::tty_keys::C0_US;
    records.push(format!(
        "src/tty_keys.rs::C0_US {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::control_character_code>(),
        align_of::<hmux2::src::tty_keys::control_character_code>()
    ));
    let v: hmux2::src::input_keys::control_character_code = hmux2::src::input_keys::C0_VT;
    records.push(format!(
        "src/input_keys.rs::C0_VT {:?} {} {}",
        v,
        size_of::<hmux2::src::input_keys::control_character_code>(),
        align_of::<hmux2::src::input_keys::control_character_code>()
    ));
    let v: hmux2::src::key_string::control_character_code = hmux2::src::key_string::C0_VT;
    records.push(format!(
        "src/key_string.rs::C0_VT {:?} {} {}",
        v,
        size_of::<hmux2::src::key_string::control_character_code>(),
        align_of::<hmux2::src::key_string::control_character_code>()
    ));
    let v: hmux2::src::tty_keys::control_character_code = hmux2::src::tty_keys::C0_VT;
    records.push(format!(
        "src/tty_keys.rs::C0_VT {:?} {} {}",
        v,
        size_of::<hmux2::src::tty_keys::control_character_code>(),
        align_of::<hmux2::src::tty_keys::control_character_code>()
    ));
    records.push(format!(
        "src/input_keys.rs::C2RustUnnamed_35 {} {}",
        size_of::<hmux2::src::input_keys::control_character_code>(),
        align_of::<hmux2::src::input_keys::control_character_code>()
    ));
    records.push(format!(
        "src/key_string.rs::C2RustUnnamed {} {}",
        size_of::<hmux2::src::key_string::control_character_code>(),
        align_of::<hmux2::src::key_string::control_character_code>()
    ));
    records.push(format!(
        "src/tty_keys.rs::C2RustUnnamed_36 {} {}",
        size_of::<hmux2::src::tty_keys::control_character_code>(),
        align_of::<hmux2::src::tty_keys::control_character_code>()
    ));
    let actual = records.join("\n") + "\n";
    assert_eq!(
        actual,
        include_str!("fixtures/subjects-control_character.txt")
    );
}
