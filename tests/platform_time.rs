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
        "src/tmux.rs::CLOCK_REALTIME",
        ::core::ffi::c_int,
        hmux2::src::tmux::CLOCK_REALTIME
    );
    constant!(
        "src/window_clock.rs::CLOCK_REALTIME",
        ::core::ffi::c_int,
        hmux2::src::window_clock::CLOCK_REALTIME
    );
    layout!(
        "src/server.rs::timespec",
        hmux2::src::server::timespec,
        [tv_sec, tv_nsec]
    );
    layout!(
        "src/tmux.rs::timespec",
        hmux2::src::tmux::timespec,
        [tv_sec, tv_nsec]
    );
    layout!(
        "src/window_clock.rs::timespec",
        hmux2::src::window_clock::timespec,
        [tv_sec, tv_nsec]
    );
    layout!(
        "src/format.rs::tm",
        hmux2::src::format::tm,
        [
            tm_sec, tm_min, tm_hour, tm_mday, tm_mon, tm_year, tm_wday, tm_yday, tm_isdst,
            tm_gmtoff, tm_zone
        ]
    );
    layout!(
        "src/window_clock.rs::tm",
        hmux2::src::window_clock::tm,
        [
            tm_sec, tm_min, tm_hour, tm_mday, tm_mon, tm_year, tm_wday, tm_yday, tm_isdst,
            tm_gmtoff, tm_zone
        ]
    );
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/platform-time.txt"));
}
