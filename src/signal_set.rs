//! Stable access to the native signal-set words.

/// The fixed-size word representation used by the platform signal set.
pub trait SignalSet {
    /// Builds a signal set from its native words.
    fn from_signal_words(words: [u64; 16]) -> Self
    where
        Self: Sized;

    /// Returns the native signal-set words.
    fn signal_words(&self) -> [u64; 16];
}

impl SignalSet for crate::types::__sigset_t {
    fn from_signal_words(words: [u64; 16]) -> Self {
        Self { __val: words }
    }
    fn signal_words(&self) -> [u64; 16] {
        self.__val
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::__sigset_t;

    #[test]
    fn native_words_round_trip() {
        let mut words = [0_u64; 16];
        words[0] = 3;
        words[15] = 9;
        assert_eq!(__sigset_t::from_signal_words(words).signal_words(), words);
    }
}
