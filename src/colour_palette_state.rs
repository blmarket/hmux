//! Portable value state for a terminal colour palette.

/// Default colours and optional indexed replacement tables.
pub trait ColourPaletteState {
    /// Builds palette state from all four portable values.
    fn from_colour_palette(
        foreground: i32,
        background: i32,
        entries: Option<&[i32; 256]>,
        default_entries: Option<&[i32; 256]>,
    ) -> Self
    where
        Self: Sized;

    /// Returns the default foreground colour.
    fn colour_palette_foreground(&self) -> i32;

    /// Replaces the default foreground colour.
    fn set_colour_palette_foreground(&mut self, foreground: i32);

    /// Returns the default background colour.
    fn colour_palette_background(&self) -> i32;

    /// Replaces the default background colour.
    fn set_colour_palette_background(&mut self, background: i32);

    /// Returns the explicit indexed replacement table, when allocated.
    fn colour_palette_entries(&self) -> Option<&[i32; 256]>;

    /// Returns the explicit indexed replacement table mutably, when allocated.
    fn colour_palette_entries_mut(&mut self) -> Option<&mut [i32; 256]>;

    /// Replaces or removes the explicit indexed replacement table.
    fn set_colour_palette_entries(&mut self, entries: Option<&[i32; 256]>);

    /// Returns the configured default replacement table, when allocated.
    fn colour_palette_default_entries(&self) -> Option<&[i32; 256]>;

    /// Returns the configured default replacement table mutably, when allocated.
    fn colour_palette_default_entries_mut(&mut self) -> Option<&mut [i32; 256]>;

    /// Replaces or removes the configured default replacement table.
    fn set_colour_palette_default_entries(&mut self, entries: Option<&[i32; 256]>);
}

impl ColourPaletteState for crate::style::colour_palette {
    fn from_colour_palette(
        foreground: i32,
        background: i32,
        entries: Option<&[i32; 256]>,
        default_entries: Option<&[i32; 256]>,
    ) -> Self {
        Self {
            fg: foreground,
            bg: background,
            palette: entries.map(|entries| std::sync::Arc::new(*entries)),
            default_palette: default_entries.map(|entries| std::sync::Arc::new(*entries)),
        }
    }

    fn colour_palette_foreground(&self) -> i32 {
        self.fg
    }

    fn set_colour_palette_foreground(&mut self, foreground: i32) {
        self.fg = foreground;
    }

    fn colour_palette_background(&self) -> i32 {
        self.bg
    }

    fn set_colour_palette_background(&mut self, background: i32) {
        self.bg = background;
    }

    fn colour_palette_entries(&self) -> Option<&[i32; 256]> {
        self.palette.as_deref()
    }

    fn colour_palette_entries_mut(&mut self) -> Option<&mut [i32; 256]> {
        self.palette.as_mut().map(std::sync::Arc::make_mut)
    }

    fn set_colour_palette_entries(&mut self, entries: Option<&[i32; 256]>) {
        self.palette = entries.map(|entries| std::sync::Arc::new(*entries));
    }

    fn colour_palette_default_entries(&self) -> Option<&[i32; 256]> {
        self.default_palette.as_deref()
    }

    fn colour_palette_default_entries_mut(&mut self) -> Option<&mut [i32; 256]> {
        self.default_palette.as_mut().map(std::sync::Arc::make_mut)
    }

    fn set_colour_palette_default_entries(&mut self, entries: Option<&[i32; 256]>) {
        self.default_palette = entries.map(|entries| std::sync::Arc::new(*entries));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::colour_palette;

    #[test]
    fn palette_state_round_trips_and_updates() {
        let mut entries = [-1; 256];
        entries[17] = 23;
        let mut defaults = [-1; 256];
        defaults[29] = 31;
        let mut state = colour_palette::from_colour_palette(3, 5, Some(&entries), Some(&defaults));
        assert_eq!(state.colour_palette_foreground(), 3);
        assert_eq!(state.colour_palette_background(), 5);
        assert_eq!(state.colour_palette_entries().unwrap()[17], 23);
        assert_eq!(state.colour_palette_default_entries().unwrap()[29], 31);

        state.set_colour_palette_foreground(7);
        state.set_colour_palette_background(11);
        state.colour_palette_entries_mut().unwrap()[17] = 37;
        state.colour_palette_default_entries_mut().unwrap()[29] = 41;
        assert_eq!(state.colour_palette_foreground(), 7);
        assert_eq!(state.colour_palette_background(), 11);
        assert_eq!(state.colour_palette_entries().unwrap()[17], 37);
        assert_eq!(state.colour_palette_default_entries().unwrap()[29], 41);

        state.set_colour_palette_entries(None);
        state.set_colour_palette_default_entries(None);
        assert!(state.colour_palette_entries().is_none());
        assert!(state.colour_palette_default_entries().is_none());
    }
}

#[cfg(test)]
mod snapshot_tests {
    use super::ColourPaletteState;
    use crate::style::{ColourEngine, RustColourEngine, colour_palette};

    #[test]
    fn palette_snapshots_keep_values_when_sources_are_changed_or_freed() {
        let mut source =
            colour_palette::from_colour_palette(4, 5, Some(&[1; 256]), Some(&[2; 256]));
        let snapshot = source.clone();
        source.colour_palette_entries_mut().unwrap()[0] = 3;
        source.colour_palette_default_entries_mut().unwrap()[0] = 6;
        assert_eq!(snapshot.colour_palette_entries().unwrap()[0], 1);
        assert_eq!(snapshot.colour_palette_default_entries().unwrap()[0], 2);
        let next = source.clone();
        RustColourEngine.set_palette(Some(&mut source), 0, 7);
        RustColourEngine.set_palette_defaults(Some(&mut source), Some(&[8; 256]));
        RustColourEngine.free_palette(Some(&mut source));
        assert_eq!(next.colour_palette_entries().unwrap()[0], 3);
        assert_eq!(next.colour_palette_default_entries().unwrap()[0], 6);
        assert_eq!((snapshot.fg, snapshot.bg), (4, 5));
    }
}
