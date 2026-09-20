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
        "src/format.rs::REG_EXTENDED",
        ::core::ffi::c_int,
        hmux2::src::format::REG_EXTENDED
    );
    constant!(
        "src/window.rs::REG_EXTENDED",
        ::core::ffi::c_int,
        hmux2::src::window::REG_EXTENDED
    );
    constant!(
        "src/window_copy.rs::REG_EXTENDED",
        ::core::ffi::c_int,
        hmux2::src::window_copy::REG_EXTENDED
    );
    constant!(
        "src/format.rs::REG_ICASE",
        ::core::ffi::c_int,
        hmux2::src::format::REG_ICASE
    );
    constant!(
        "src/window.rs::REG_ICASE",
        ::core::ffi::c_int,
        hmux2::src::window::REG_ICASE
    );
    constant!(
        "src/window_copy.rs::REG_ICASE",
        ::core::ffi::c_int,
        hmux2::src::window_copy::REG_ICASE
    );
    layout!(
        "src/format.rs::__re_long_size_t",
        hmux2::src::format::__re_long_size_t,
        []
    );
    layout!(
        "src/regsub.rs::__re_long_size_t",
        hmux2::src::regsub::__re_long_size_t,
        []
    );
    layout!(
        "src/window.rs::__re_long_size_t",
        hmux2::src::window::__re_long_size_t,
        []
    );
    layout!(
        "src/window_copy.rs::__re_long_size_t",
        hmux2::src::window_copy::__re_long_size_t,
        []
    );
    layout!(
        "src/format.rs::re_pattern_buffer",
        hmux2::src::format::re_pattern_buffer,
        [
            buffer,
            allocated,
            used,
            syntax,
            fastmap,
            translate,
            re_nsub,
            can_be_null_regs_allocated_fastmap_accurate_no_sub_not_bol_not_eol_newline_anchor,
            c2rust_padding
        ]
    );
    layout!(
        "src/regsub.rs::re_pattern_buffer",
        hmux2::src::regsub::re_pattern_buffer,
        [
            buffer,
            allocated,
            used,
            syntax,
            fastmap,
            translate,
            re_nsub,
            can_be_null_regs_allocated_fastmap_accurate_no_sub_not_bol_not_eol_newline_anchor,
            c2rust_padding
        ]
    );
    layout!(
        "src/window.rs::re_pattern_buffer",
        hmux2::src::window::re_pattern_buffer,
        [
            buffer,
            allocated,
            used,
            syntax,
            fastmap,
            translate,
            re_nsub,
            can_be_null_regs_allocated_fastmap_accurate_no_sub_not_bol_not_eol_newline_anchor,
            c2rust_padding
        ]
    );
    layout!(
        "src/window_copy.rs::re_pattern_buffer",
        hmux2::src::window_copy::re_pattern_buffer,
        [
            buffer,
            allocated,
            used,
            syntax,
            fastmap,
            translate,
            re_nsub,
            can_be_null_regs_allocated_fastmap_accurate_no_sub_not_bol_not_eol_newline_anchor,
            c2rust_padding
        ]
    );
    layout!(
        "src/format.rs::reg_syntax_t",
        hmux2::src::format::reg_syntax_t,
        []
    );
    layout!(
        "src/regsub.rs::reg_syntax_t",
        hmux2::src::regsub::reg_syntax_t,
        []
    );
    layout!(
        "src/window.rs::reg_syntax_t",
        hmux2::src::window::reg_syntax_t,
        []
    );
    layout!(
        "src/window_copy.rs::reg_syntax_t",
        hmux2::src::window_copy::reg_syntax_t,
        []
    );
    layout!("src/format.rs::regex_t", hmux2::src::format::regex_t, []);
    layout!("src/regsub.rs::regex_t", hmux2::src::regsub::regex_t, []);
    layout!("src/window.rs::regex_t", hmux2::src::window::regex_t, []);
    layout!(
        "src/window_copy.rs::regex_t",
        hmux2::src::window_copy::regex_t,
        []
    );
    layout!(
        "src/format.rs::regmatch_t",
        hmux2::src::format::regmatch_t,
        [rm_so, rm_eo]
    );
    layout!(
        "src/regsub.rs::regmatch_t",
        hmux2::src::regsub::regmatch_t,
        [rm_so, rm_eo]
    );
    layout!(
        "src/window.rs::regmatch_t",
        hmux2::src::window::regmatch_t,
        [rm_so, rm_eo]
    );
    layout!(
        "src/window_copy.rs::regmatch_t",
        hmux2::src::window_copy::regmatch_t,
        [rm_so, rm_eo]
    );
    layout!("src/format.rs::regoff_t", hmux2::src::format::regoff_t, []);
    layout!("src/regsub.rs::regoff_t", hmux2::src::regsub::regoff_t, []);
    layout!("src/window.rs::regoff_t", hmux2::src::window::regoff_t, []);
    layout!(
        "src/window_copy.rs::regoff_t",
        hmux2::src::window_copy::regoff_t,
        []
    );
    macro_rules! bitfields {
        ($ty:path, $file:literal) => {
            for value in [0, 1, 2, 3, 7, 255, u32::MAX] {
                let mut r: $ty = unsafe { std::mem::zeroed() };
                r.c2rust_padding = [0xa5; 7];
                r.set_can_be_null(value);
                r.set_regs_allocated(value);
                r.set_fastmap_accurate(value);
                r.set_no_sub(value);
                r.set_not_bol(value);
                r.set_not_eol(value);
                r.set_newline_anchor(value);
                records.push(format!(concat!($file, " bits {} {:?} {:?} {:?}"), value,
                    r.can_be_null_regs_allocated_fastmap_accurate_no_sub_not_bol_not_eol_newline_anchor,
                    r.c2rust_padding,
                    [r.can_be_null(), r.regs_allocated(), r.fastmap_accurate(),
                     r.no_sub(), r.not_bol(), r.not_eol(), r.newline_anchor()]));
            }
        };
    }
    bitfields!(hmux2::src::format::re_pattern_buffer, "src/format.rs");
    bitfields!(hmux2::src::regsub::re_pattern_buffer, "src/regsub.rs");
    bitfields!(hmux2::src::window::re_pattern_buffer, "src/window.rs");
    bitfields!(
        hmux2::src::window_copy::re_pattern_buffer,
        "src/window_copy.rs"
    );
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/platform-regex.txt"));
}
