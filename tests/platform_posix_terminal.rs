//! Frozen measurements from every original translation-unit copy (Linux x86_64).
use std::mem::{align_of, offset_of, size_of};
#[test]
fn platform_layouts() {
    let mut records = Vec::new();
    macro_rules! layout {
    ($label:literal, $ty:path, [$($field:ident),*]) => {
        records.push(format!(concat!($label, " {} {}", $(" ", stringify!($field), "={}"),*),
            size_of::<$ty>(), align_of::<$ty>() $(, offset_of!($ty, $field))*));
    };
}
    macro_rules! constant {
        ($label:literal, $ty:ty, $value:path) => {{
            let value: $ty = $value;
            records.push(format!(
                concat!($label, " {:?} {} {}"),
                value,
                size_of::<$ty>(),
                align_of::<$ty>()
            ));
        }};
    }

    constant!(
        "src/client.rs::ICRNL",
        ::core::ffi::c_int,
        hmux2::src::client::ICRNL
    );
    constant!(
        "src/tty.rs::ICRNL",
        ::core::ffi::c_int,
        hmux2::src::tty::ICRNL
    );
    constant!(
        "src/client.rs::ONLCR",
        ::core::ffi::c_int,
        hmux2::src::client::ONLCR
    );
    constant!(
        "src/tty.rs::ONLCR",
        ::core::ffi::c_int,
        hmux2::src::tty::ONLCR
    );
    constant!(
        "src/client.rs::OPOST",
        ::core::ffi::c_int,
        hmux2::src::client::OPOST
    );
    constant!(
        "src/tty.rs::OPOST",
        ::core::ffi::c_int,
        hmux2::src::tty::OPOST
    );
    constant!(
        "src/client.rs::TCSANOW",
        ::core::ffi::c_int,
        hmux2::src::client::TCSANOW
    );
    constant!(
        "src/spawn.rs::TCSANOW",
        ::core::ffi::c_int,
        hmux2::src::spawn::TCSANOW
    );
    constant!(
        "src/tty.rs::TCSANOW",
        ::core::ffi::c_int,
        hmux2::src::tty::TCSANOW
    );
    constant!(
        "src/job.rs::TIOCSWINSZ",
        ::core::ffi::c_int,
        hmux2::src::job::TIOCSWINSZ
    );
    constant!(
        "src/window.rs::TIOCSWINSZ",
        ::core::ffi::c_int,
        hmux2::src::window::TIOCSWINSZ
    );
    constant!(
        "src/spawn.rs::VERASE",
        ::core::ffi::c_int,
        hmux2::src::spawn::VERASE
    );
    constant!(
        "src/tty_keys.rs::VERASE",
        ::core::ffi::c_int,
        hmux2::src::tty_keys::VERASE
    );
    constant!(
        "src/client.rs::VMIN",
        ::core::ffi::c_int,
        hmux2::src::client::VMIN
    );
    constant!(
        "src/tty.rs::VMIN",
        ::core::ffi::c_int,
        hmux2::src::tty::VMIN
    );
    constant!(
        "src/client.rs::VTIME",
        ::core::ffi::c_int,
        hmux2::src::client::VTIME
    );
    constant!(
        "src/tty.rs::VTIME",
        ::core::ffi::c_int,
        hmux2::src::tty::VTIME
    );
    layout!(
        "src/compat/fdforkpty.rs::winsize",
        hmux2::src::compat::fdforkpty::winsize,
        [ws_row, ws_col, ws_xpixel, ws_ypixel]
    );
    layout!(
        "src/job.rs::winsize",
        hmux2::src::job::winsize,
        [ws_row, ws_col, ws_xpixel, ws_ypixel]
    );
    layout!(
        "src/spawn.rs::winsize",
        hmux2::src::spawn::winsize,
        [ws_row, ws_col, ws_xpixel, ws_ypixel]
    );
    layout!(
        "src/tty.rs::winsize",
        hmux2::src::tty::winsize,
        [ws_row, ws_col, ws_xpixel, ws_ypixel]
    );
    layout!(
        "src/window.rs::winsize",
        hmux2::src::window::winsize,
        [ws_row, ws_col, ws_xpixel, ws_ypixel]
    );
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/platform-posix_terminal.txt"));
}
