//! Authoritative colour-theme and palette declarations.

pub type client_theme = ::core::ffi::c_uint;
pub const THEME_DARK: client_theme = 2;
pub const THEME_LIGHT: client_theme = 1;
pub const THEME_UNKNOWN: client_theme = 0;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct colour_palette {
    pub fg: ::core::ffi::c_int,
    pub bg: ::core::ffi::c_int,
    pub palette: *mut ::core::ffi::c_int,
    pub default_palette: *mut ::core::ffi::c_int,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::core::mem::{align_of, offset_of, size_of};

    #[test]
    fn colour_palette_layout_matches_translated_c_baseline() {
        assert_eq!(size_of::<colour_palette>(), 24);
        assert_eq!(align_of::<colour_palette>(), 8);
        assert_eq!(offset_of!(colour_palette, fg), 0);
        assert_eq!(offset_of!(colour_palette, bg), 4);
        assert_eq!(offset_of!(colour_palette, palette), 8);
        assert_eq!(offset_of!(colour_palette, default_palette), 16);
    }
}
